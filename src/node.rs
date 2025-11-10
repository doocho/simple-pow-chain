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
    pub fn start_listener(&self) {
        let addr = self.listen_addr.clone();
        let bc = self.blockchain.clone();
        tokio::spawn(async move {
            let listener = TcpListener::bind(&addr).await.unwrap();
            println!("Node listening on {}", addr);
            loop {
                let (stream, addr) = listener.accept().await.unwrap();
                println!("Incoming connection from {}", addr);
                let bc_clone = bc.clone();
                tokio::spawn(handle_connection(stream, bc_clone));
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
}

/// Receive handler: Block validation + chain addition
async fn handle_connection(mut stream: TcpStream, blockchain: Arc<RwLock<Blockchain>>) {
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
            let mut bc = blockchain.write().unwrap();
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
        Err(e) => println!("Deserialize error: {}", e),
    }
}
