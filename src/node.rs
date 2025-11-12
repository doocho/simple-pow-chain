use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::message::NetworkMessage;
use crate::transaction::Transaction;
use bincode;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct Node {
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub tx_pool: Arc<RwLock<Vec<Transaction>>>,
    listen_addr: String,
    peers: Vec<String>,
}

impl Node {
    pub fn new(
        blockchain: Arc<RwLock<Blockchain>>,
        listen_addr: String,
        peers: Vec<String>,
    ) -> Self {
        Self {
            blockchain,
            tx_pool: Arc::new(RwLock::new(Vec::new())),
            listen_addr,
            peers,
        }
    }

    /// Start listener (background with spawn)
    pub fn start_listener(self: Arc<Self>) {
        let addr = self.listen_addr.clone();
        let node = self.clone();
        tokio::spawn(async move {
            let listener = TcpListener::bind(&addr).await.unwrap();
            println!("Node listening on {}", addr);
            loop {
                let (stream, addr) = listener.accept().await.unwrap();
                println!("Incoming connection from {}", addr);
                let node_clone = node.clone();
                tokio::spawn(handle_connection(stream, node_clone));
            }
        });
    }

    /// Broadcast new block!
    pub async fn broadcast_new_block(&self, block: Block) {
        let block_index = block.index;
        let message = NetworkMessage::NewBlock(block);
        let encoded = bincode::serialize(&message).unwrap();
        let len_bytes = (encoded.len() as u32).to_be_bytes();

        for peer in &self.peers {
            println!("Broadcasting to {}", peer);
            match TcpStream::connect(peer).await {
                Ok(mut stream) => {
                    // Send length prefix + data
                    let _ = stream.write_all(&len_bytes).await;
                    let _ = stream.write_all(&encoded).await;
                    println!("Broadcasted block #{} to {}", block_index, peer);
                }
                Err(e) => println!("Failed to broadcast to {}: {}", peer, e),
            }
        }
    }

    /// Broadcast new transaction!
    pub async fn broadcast_new_transaction(&self, transaction: Transaction) {
        let message = NetworkMessage::NewTransaction(transaction);
        let encoded = bincode::serialize(&message).unwrap();
        let len_bytes = (encoded.len() as u32).to_be_bytes();

        for peer in &self.peers {
            println!("Broadcasting transaction to {}", peer);
            match TcpStream::connect(peer).await {
                Ok(mut stream) => {
                    // Send length prefix + data
                    let _ = stream.write_all(&len_bytes).await;
                    let _ = stream.write_all(&encoded).await;
                    println!("Broadcasted transaction to {}", peer);
                }
                Err(e) => println!("Failed to broadcast transaction to {}: {}", peer, e),
            }
        }
    }

    /// Request blockchain from peers and wait for response
    pub async fn request_blockchain(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message = NetworkMessage::RequestBlockchain;
        let encoded = bincode::serialize(&message)?;
        let len_bytes = (encoded.len() as u32).to_be_bytes();

        // Track the best (longest) valid chain received
        let mut best_chain: Option<Blockchain> = None;
        let mut best_len: usize = {
            let bc = self.blockchain.read().unwrap();
            bc.chain.len()
        };

        for peer in &self.peers {
            println!("Requesting blockchain from {}", peer);
            match TcpStream::connect(peer).await {
                Ok(mut stream) => {
                    // Send request
                    stream.write_all(&len_bytes).await?;
                    stream.write_all(&encoded).await?;
                    println!("Sent blockchain request to {}", peer);

                    // Wait for response
                    let mut len_buf = [0u8; 4];
                    if stream.read_exact(&mut len_buf).await.is_err() {
                        println!("Failed to read response length from {}", peer);
                        continue;
                    }
                    let len = u32::from_be_bytes(len_buf) as usize;

                    let mut buf = vec![0u8; len];
                    if stream.read_exact(&mut buf).await.is_err() {
                        println!("Failed to read response data from {}", peer);
                        continue;
                    }

                    match bincode::deserialize::<NetworkMessage>(&buf) {
                        Ok(NetworkMessage::ResponseBlockchain(received_bc)) => {
                            println!("Received blockchain response with {} blocks from {}", received_bc.chain.len(), peer);

                            // Validate received blockchain and keep if it's longer
                            if validate_blockchain(&received_bc) {
                                let received_len = received_bc.chain.len();
                                if received_len > best_len {
                                    println!("Found a longer valid chain from {} ({} > {})", peer, received_len, best_len);
                                    best_len = received_len;
                                    best_chain = Some(received_bc);
                                } else {
                                    println!("Received chain from {} is not longer ({} <= {}), ignoring", peer, received_len, best_len);
                                }
                            } else {
                                println!("Received blockchain from {} is invalid, rejected!", peer);
                            }
                        }
                        Ok(other) => {
                            println!("Received unexpected message from {}: {:?}", peer, other);
                        }
                        Err(e) => {
                            println!("Failed to deserialize response from {}: {}", peer, e);
                        }
                    }
                }
                Err(e) => println!("Failed to connect to {}: {}", peer, e),
            }
        }

        // After querying all peers, apply the best chain if any
        if let Some(new_chain) = best_chain {
            let mut bc = self.blockchain.write().unwrap();
            *bc = new_chain;
            println!("Blockchain updated to longest valid chain. New chain length: {}", bc.chain.len());
        } else {
            println!("No longer valid chain found from peers");
        }
        Ok(())
    }

    /// Respond with current blockchain
    pub async fn respond_blockchain(&self, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let blockchain = self.blockchain.read().unwrap().clone();
        let message = NetworkMessage::ResponseBlockchain(blockchain);
        let encoded = bincode::serialize(&message)?;
        let len_bytes = (encoded.len() as u32).to_be_bytes();

        stream.write_all(&len_bytes).await?;
        stream.write_all(&encoded).await?;
        println!("Responded with blockchain");
        Ok(())
    }

    /// Mine a new block with transactions from the pool
    pub async fn mine_block(&self) -> Option<Block> {
        // First, get the current blockchain state to determine what to mine
        let (index, previous_hash, difficulty, transactions_to_mine) = {
            let blockchain = self.blockchain.read().unwrap();
            let last_block = blockchain.chain.last();
            let index = last_block.map(|b| b.index + 1).unwrap_or(0);
            let previous_hash = last_block.map(|b| b.hash.clone()).unwrap_or("0".to_string());
            let difficulty = blockchain.difficulty;
            drop(blockchain);

            let transactions_to_mine: Vec<Transaction> = {
                let mut tx_pool = self.tx_pool.write().unwrap();
                if tx_pool.is_empty() {
                    // If no transactions, create a dummy transaction
                    let dummy_tx = Transaction::new("system".to_string(), "system".to_string(), 0);
                    tx_pool.push(dummy_tx);
                }
                tx_pool.drain(..).collect()
            };

            (index, previous_hash, difficulty, transactions_to_mine)
        };

        let mut new_block = Block::new(
            index,
            previous_hash,
            transactions_to_mine,
            difficulty,
        );

        println!("Starting to mine block #{} with {} transactions", index, new_block.transactions.len());
        new_block.mine();
        println!("Successfully mined block #{} with hash: {}", index, new_block.hash);

        // Check if another block was mined while we were mining
        {
            let blockchain = self.blockchain.read().unwrap();
            let current_last_block = blockchain.chain.last();
            let current_index = current_last_block.map(|b| b.index).unwrap_or(0);

            if current_index >= index {
                println!("Block #{} was already mined by another node (current chain at #{}), discarding our block",
                         index, current_index);
                // Put transactions back in pool
                let mut tx_pool = self.tx_pool.write().unwrap();
                for tx in new_block.transactions {
                    tx_pool.push(tx);
                }
                return None;
            }
            drop(blockchain);
        }

        // Double-check and add to blockchain atomically
        {
            let mut bc = self.blockchain.write().unwrap();
            let current_last = bc.chain.last();
            let expected_index = current_last.map(|b| b.index + 1).unwrap_or(0);
            let expected_prev = current_last.map(|b| b.hash.clone()).unwrap_or("0".to_string());

            if expected_index == new_block.index && expected_prev == new_block.previous_hash {
                bc.chain.push(new_block.clone());
                println!("Block #{} added to local blockchain. Chain length: {}", index, bc.chain.len());
            } else {
                println!("Block #{} is no longer valid (chain changed during mining), discarding", index);
                // Put transactions back in pool
                let mut tx_pool = self.tx_pool.write().unwrap();
                for tx in new_block.transactions {
                    tx_pool.push(tx);
                }
                return None;
            }
        }

        Some(new_block)
    }

    /// Start background mining process
    pub fn start_mining(self: Arc<Self>) {
        let node = self.clone();

        tokio::spawn(async move {
            println!("Started background mining process");

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                // Check if we have transactions to mine and if we're behind the network
                let should_mine = {
                    let tx_pool = node.tx_pool.read().unwrap();
                    let has_transactions = !tx_pool.is_empty();

                    // Also check if we're behind - if tx pool has reward transactions from mining
                    let has_pending_rewards = tx_pool.iter().any(|tx| tx.to == "reward");
                    drop(tx_pool);

                    has_transactions || has_pending_rewards
                };

                if should_mine {
                    let pool_size = {
                        let tx_pool = node.tx_pool.read().unwrap();
                        tx_pool.len()
                    };

                    println!("*****************************************************");
                    println!("Mining with {} transactions in pool", pool_size);

                    if let Some(mined_block) = node.mine_block().await {
                        println!("Broadcasting mined block #{}", mined_block.index);
                        node.broadcast_new_block(mined_block).await;

                        // Mining successful, wait a bit before continuing
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    } else {
                        // Mining failed (another node mined first), wait longer
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                } else {
                    // Add a dummy reward transaction to keep mining active
                    {
                        let mut tx_pool = node.tx_pool.write().unwrap();
                        let dummy_tx = Transaction::new(
                            format!("miner_{}", node.listen_addr),
                            "reward".to_string(),
                            1,
                        );
                        tx_pool.push(dummy_tx);
                    }
                }
            }
        });
    }
}

/// Receive handler: Block validation + chain addition
async fn handle_connection(mut stream: TcpStream, node: Arc<Node>) {
    // Read length
    let mut len_buf = [0u8; 4];
    if stream.read_exact(&mut len_buf).await.is_err() {
        return;
    }
    let len = u32::from_be_bytes(len_buf) as usize;

    // Read data
    let mut buf = vec![0u8; len];
    if stream.read_exact(&mut buf).await.is_err() {
        return;
    }

    match bincode::deserialize::<NetworkMessage>(&buf) {
        Ok(NetworkMessage::NewBlock(block)) => {
            println!("Received block: {:?}", block);

            // Get blockchain state
            let (current_chain_len, current_last_block) = {
                let bc = node.blockchain.read().unwrap();
                (bc.chain.len(), bc.chain.last().cloned())
            };

            // Check if this block extends our current chain
            let expected_index = current_last_block.as_ref().map(|b| b.index + 1).unwrap_or(0);
            let expected_prev = current_last_block.as_ref().map(|b| b.hash.clone()).unwrap_or("0".to_string());

            // Basic validation
            let is_valid = block.index == expected_index
                && block.previous_hash == expected_prev
                && block.hash == block.calculate_hash()  // 무결성
                && block.hash.starts_with(&"0".repeat(block.difficulty)); // PoW

            if is_valid {
                // Acquire write lock and add block
                let mut bc = node.blockchain.write().unwrap();

                // Double-check after acquiring write lock (another block might have been added)
                let current_last = bc.chain.last();
                if current_last.map(|b| b.index + 1).unwrap_or(0) == block.index
                    && current_last.map(|b| b.hash.clone()).unwrap_or("0".to_string()) == block.previous_hash {

                    bc.chain.push(block.clone());
                    println!(
                        "Block #{} added! Chain length: {}",
                        block.index,
                        bc.chain.len()
                    );

                    // Clean up transaction pool - remove transactions that are now in the block
                    let mut tx_pool = node.tx_pool.write().unwrap();
                    let block_tx_hashes: std::collections::HashSet<String> =
                        block.transactions.iter().map(|tx| tx.hash()).collect();
                    tx_pool.retain(|tx| !block_tx_hashes.contains(&tx.hash()));
                    println!("Transaction pool updated after adding block, remaining: {}", tx_pool.len());
                    drop(tx_pool);
                } else {
                    println!("Block #{} no longer valid after acquiring write lock (chain changed)", block.index);
                }
            } else {
                // Check if this might be a fork - if block index is higher than our current chain
                let current_max_index = current_last_block.as_ref().map(|b| b.index).unwrap_or(0);
                let current_last_hash = current_last_block.as_ref().map(|b| b.hash.clone()).unwrap_or("0".to_string());

                if block.index > current_max_index {
                    println!("Received block #{} with higher index than our chain ({}), attempting fork resolution",
                             block.index, current_max_index);

                    // Try to resolve fork by requesting the full blockchain from the peer
                    // For now, we'll request blockchain from peers to get the longer chain
                    let node_clone = node.clone();
                    tokio::spawn(async move {
                        println!("Requesting full blockchain for fork resolution");
                        if let Err(e) = node_clone.request_blockchain().await {
                            println!("Failed to resolve fork: {}", e);
                        }
                    });
                } else if block.index == current_max_index && block.previous_hash != current_last_hash {
                    println!("Received competing block #{} with different previous_hash, this indicates a fork at height {}", block.index, block.index - 1);
                    // For now, just log - in a real implementation you'd compare work or use other consensus rules
                } else {
                    println!("Invalid block rejected! Expected index: {}, got: {}", expected_index, block.index);
                }
            }
        }
        Ok(NetworkMessage::NewTransaction(transaction)) => {
            println!("Received new transaction: {}", transaction);
            let mut tx_pool = node.tx_pool.write().unwrap();

            // Check if transaction already exists in pool
            let exists = tx_pool.iter().any(|tx| tx.hash() == transaction.hash());
            if !exists {
                tx_pool.push(transaction);
                println!("Transaction added to pool. Pool size: {}", tx_pool.len());
            } else {
                println!("Transaction already exists in pool, ignoring");
            }
        }
        Ok(NetworkMessage::RequestBlockchain) => {
            println!("Received blockchain request, responding...");
            let _ = node.respond_blockchain(stream).await;
        }
        Ok(NetworkMessage::ResponseBlockchain(_)) => {
            // ResponseBlockchain should be handled by the requesting node directly
            // in request_blockchain method, not through handle_connection
            println!("Received unexpected ResponseBlockchain message in handle_connection");
        }
        Err(e) => println!("Deserialize error: {}", e),
    }
}

/// Validate received blockchain
fn validate_blockchain(blockchain: &Blockchain) -> bool {
    if blockchain.chain.is_empty() {
        return false;
    }

    // Validate genesis block
    let genesis = &blockchain.chain[0];
    if genesis.index != 0 || genesis.previous_hash != "0" {
        println!("Invalid genesis block");
        return false;
    }

    // Validate each block
    for i in 0..blockchain.chain.len() {
        let block = &blockchain.chain[i];

        // Check hash integrity
        if block.hash != block.calculate_hash() {
            println!("Block {} hash integrity check failed", i);
            return false;
        }

        // Check PoW
        if !block.hash.starts_with(&"0".repeat(block.difficulty)) {
            println!("Block {} PoW check failed", i);
            return false;
        }

        // Check previous hash (except genesis)
        if i > 0 {
            let prev_block = &blockchain.chain[i - 1];
            if block.previous_hash != prev_block.hash {
                println!("Block {} previous hash mismatch", i);
                return false;
            }

            // Check index continuity
            if block.index != prev_block.index + 1 {
                println!("Block {} index discontinuity", i);
                return false;
            }
        }
    }

    println!("Blockchain validation passed");
    true
}
