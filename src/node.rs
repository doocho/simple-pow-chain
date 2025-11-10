use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::message::NetworkMessage;
use bincode;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct Node {
    pub blockchain: Arc<RwLock<Blockchain>>,
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

    /// Request blockchain from peers and wait for response
    pub async fn request_blockchain(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message = NetworkMessage::RequestBlockchain;
        let encoded = bincode::serialize(&message)?;
        let len_bytes = (encoded.len() as u32).to_be_bytes();

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

                            // Validate received blockchain
                            if validate_blockchain(&received_bc) {
                                let mut bc = self.blockchain.write().unwrap();
                                *bc = received_bc;
                                println!("Blockchain updated! New chain length: {}", bc.chain.len());
                                return Ok(()); // Successfully received and applied blockchain
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

        println!("Failed to get blockchain from any peer");
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
            let mut bc = node.blockchain.write().unwrap();
            let (expected_index, expected_prev) = if bc.chain.is_empty() {
                (0u32, "0".to_string())
            } else {
                let last = bc.chain.last().unwrap();
                (last.index + 1, last.hash.clone())
            };

            println!("block: {:?}", block);
            println!(
                "block.index: {}, expected_index: {}",
                block.index, expected_index
            );
            println!(
                "block.previous_hash: {}, expected_prev: {}",
                block.previous_hash, expected_prev
            );
            println!(
                "block.hash: {}, block.calculate_hash(): {}",
                block.hash,
                block.calculate_hash()
            );
            println!(
                "block.hash.starts_with(&\"0\".repeat(block.difficulty)): {}",
                block.hash.starts_with(&"0".repeat(block.difficulty))
            );

            let is_valid = block.index == expected_index
                && block.previous_hash == expected_prev
                && block.hash == block.calculate_hash()  // 무결성
                && block.hash.starts_with(&"0".repeat(block.difficulty)); // PoW

            if is_valid {
                bc.chain.push(block);
                println!(
                    "Block #{} added! Chain length: {}",
                    expected_index,
                    bc.chain.len()
                );
            } else {
                println!("Invalid block rejected!");
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
