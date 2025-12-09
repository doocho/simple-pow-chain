use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::message::Message;

/// A seed node that maintains a list of known peers
pub struct SeedNode {
    pub addr: String,
    pub peers: Arc<RwLock<HashSet<String>>>,
}

impl SeedNode {
    /// Create a new seed node
    pub fn new(addr: String) -> Self {
        SeedNode {
            addr,
            peers: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Start the seed node server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Seed node listening on {}", self.addr);

        loop {
            let (stream, addr) = listener.accept().await?;
            println!("Connection from {}", addr);

            let peers = self.peers.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_seed_connection(stream, peers).await {
                    eprintln!("Seed connection error: {}", e);
                }
            });
        }
    }

    /// Get current peer count
    pub fn peer_count(&self) -> usize {
        self.peers.read().unwrap().len()
    }
}

/// Handle incoming connection to seed node
async fn handle_seed_connection(
    mut stream: TcpStream,
    peers: Arc<RwLock<HashSet<String>>>,
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
        Message::Register(peer_addr) => {
            println!("Registering peer: {}", peer_addr);
            let mut peer_list = peers.write().unwrap();
            peer_list.insert(peer_addr.clone());
            println!("Total peers: {}", peer_list.len());
        }

        Message::GetPeers => {
            println!("Sending peer list");
            let peer_list: Vec<String> = {
                let peers = peers.read().unwrap();
                peers.iter().cloned().collect()
            };
            let response = Message::Peers(peer_list);
            let data = bincode::serialize(&response)?;
            let len = (data.len() as u32).to_be_bytes();
            stream.write_all(&len).await?;
            stream.write_all(&data).await?;
        }

        _ => {
            eprintln!("Seed node received unexpected message");
        }
    }

    Ok(())
}

/// Client functions to interact with seed node
pub async fn register_with_seed(seed_addr: &str, our_addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect(seed_addr).await?;
    let msg = Message::Register(our_addr.to_string());
    let data = bincode::serialize(&msg)?;
    let len = (data.len() as u32).to_be_bytes();

    stream.write_all(&len).await?;
    stream.write_all(&data).await?;

    Ok(())
}

pub async fn get_peers_from_seed(seed_addr: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect(seed_addr).await?;
    let msg = Message::GetPeers;
    let data = bincode::serialize(&msg)?;
    let len = (data.len() as u32).to_be_bytes();

    stream.write_all(&len).await?;
    stream.write_all(&data).await?;

    // Read response
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;

    let response: Message = bincode::deserialize(&buf)?;
    match response {
        Message::Peers(peers) => Ok(peers),
        _ => Ok(vec![]),
    }
}
