use std::sync::{Arc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::message::Message;
use crate::transaction::Transaction;

/// A P2P node in the blockchain network
pub struct Node {
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub mempool: Arc<RwLock<Vec<Transaction>>>,
    pub addr: String,
    pub peers: Arc<RwLock<Vec<String>>>,
}

impl Node {
    /// Create a new node
    pub fn new(blockchain: Blockchain, addr: String, peers: Vec<String>) -> Self {
        Node {
            blockchain: Arc::new(RwLock::new(blockchain)),
            mempool: Arc::new(RwLock::new(Vec::new())),
            addr,
            peers: Arc::new(RwLock::new(peers)),
        }
    }

    /// Add a peer to the list
    pub fn add_peer(&self, peer: String) {
        let mut peers = self.peers.write().unwrap();
        if !peers.contains(&peer) && peer != self.addr {
            println!("Adding peer: {}", peer);
            peers.push(peer);
        }
    }

    /// Get current peer list
    pub fn get_peers(&self) -> Vec<String> {
        self.peers.read().unwrap().clone()
    }

    /// Start listening for connections
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Node listening on {}", self.addr);

        loop {
            let (stream, addr) = listener.accept().await?;
            println!("Connection from {}", addr);

            let blockchain = self.blockchain.clone();
            let mempool = self.mempool.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, blockchain, mempool).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }

    /// Send a message to a peer
    async fn send_message(peer: &str, msg: &Message) -> Result<Option<Message>, Box<dyn std::error::Error + Send + Sync>> {
        let mut stream = TcpStream::connect(peer).await?;
        let data = bincode::serialize(msg)?;
        let len = (data.len() as u32).to_be_bytes();

        stream.write_all(&len).await?;
        stream.write_all(&data).await?;

        // Read response if expected
        let mut len_buf = [0u8; 4];
        if stream.read_exact(&mut len_buf).await.is_ok() {
            let len = u32::from_be_bytes(len_buf) as usize;
            let mut buf = vec![0u8; len];
            stream.read_exact(&mut buf).await?;
            let response: Message = bincode::deserialize(&buf)?;
            return Ok(Some(response));
        }

        Ok(None)
    }

    /// Broadcast a block to all peers
    pub async fn broadcast_block(&self, block: &Block) {
        let msg = Message::NewBlock(block.clone());
        let peers = self.get_peers();
        for peer in peers {
            if let Err(e) = Self::send_message(&peer, &msg).await {
                eprintln!("Failed to send to {}: {}", peer, e);
            }
        }
    }

    /// Broadcast a transaction to all peers
    pub async fn broadcast_transaction(&self, tx: &Transaction) {
        let msg = Message::NewTransaction(tx.clone());
        let peers = self.get_peers();
        for peer in peers {
            if let Err(e) = Self::send_message(&peer, &msg).await {
                eprintln!("Failed to send to {}: {}", peer, e);
            }
        }
    }

    /// Sync blockchain from peers (longest chain rule)
    pub async fn sync(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let current_len = {
            let bc = self.blockchain.read().unwrap();
            bc.len()
        };

        let mut best_chain: Option<Blockchain> = None;
        let mut best_len = current_len;

        let peers = self.get_peers();
        for peer in peers {
            println!("Requesting blockchain from {}", peer);

            match Self::send_message(&peer, &Message::GetBlocks).await {
                Ok(Some(Message::Blocks(chain))) => {
                    if chain.is_valid() && chain.len() > best_len {
                        println!("Found longer valid chain from {} ({} blocks)", peer, chain.len());
                        best_len = chain.len();
                        best_chain = Some(chain);
                    }
                }
                Ok(_) => {}
                Err(e) => eprintln!("Failed to sync from {}: {}", peer, e),
            }
        }

        if let Some(chain) = best_chain {
            let mut bc = self.blockchain.write().unwrap();
            *bc = chain;
            println!("Blockchain updated to {} blocks", bc.len());
        }

        Ok(())
    }

    /// Mine a new block
    pub async fn mine(&self, miner_address: &str) -> Option<Block> {
        let (index, prev_hash, difficulty, transactions) = {
            let bc = self.blockchain.read().unwrap();
            let last = bc.last_block();
            let index = last.map(|b| b.index + 1).unwrap_or(0);
            let prev_hash = last.map(|b| b.hash.clone()).unwrap_or_else(|| String::from("0"));
            let difficulty = bc.difficulty;

            let mut mempool = self.mempool.write().unwrap();
            let mut txs: Vec<Transaction> = mempool.drain(..).collect();

            // Add coinbase transaction
            txs.insert(0, Transaction::coinbase(miner_address.to_string(), 50));
            (index, prev_hash, difficulty, txs)
        };

        let mut block = Block::new(index, prev_hash, transactions, difficulty);
        block.mine();

        // Add to blockchain
        {
            let mut bc = self.blockchain.write().unwrap();
            if bc.add_mined_block(block.clone()) {
                println!("Block #{} added to chain", block.index);
                return Some(block);
            }
        }

        None
    }

    /// Add transaction to mempool
    pub fn add_transaction(&self, tx: Transaction) {
        let mut mempool = self.mempool.write().unwrap();
        if !mempool.iter().any(|t| t.hash() == tx.hash()) {
            mempool.push(tx);
        }
    }
}

/// Handle incoming connection
async fn handle_connection(
    mut stream: TcpStream,
    blockchain: Arc<RwLock<Blockchain>>,
    mempool: Arc<RwLock<Vec<Transaction>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read message length
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    // Read message
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;

    let msg: Message = bincode::deserialize(&buf)?;

    match msg {
        Message::NewBlock(block) => {
            println!("Received block #{}", block.index);
            let mut bc = blockchain.write().unwrap();
            if bc.add_mined_block(block.clone()) {
                println!("Block #{} added", block.index);
                // Remove included transactions from mempool
                let mut pool = mempool.write().unwrap();
                let block_tx_hashes: std::collections::HashSet<_> =
                    block.transactions.iter().map(|t| t.hash()).collect();
                pool.retain(|t| !block_tx_hashes.contains(&t.hash()));
            }
        }

        Message::NewTransaction(tx) => {
            println!("Received transaction: {}", tx);
            let mut pool = mempool.write().unwrap();
            if !pool.iter().any(|t| t.hash() == tx.hash()) {
                pool.push(tx);
            }
        }

        Message::GetBlocks => {
            println!("Received GetBlocks request");
            let (data, len) = {
                let bc = blockchain.read().unwrap();
                let response = Message::Blocks(bc.clone());
                let data = bincode::serialize(&response)?;
                let len = (data.len() as u32).to_be_bytes();
                (data, len)
            };
            stream.write_all(&len).await?;
            stream.write_all(&data).await?;
        }

        Message::Blocks(_) => {
            // Handled by sync()
        }

        Message::Register(_) | Message::GetPeers | Message::Peers(_) => {
            // Handled by seed node
        }
    }

    Ok(())
}
