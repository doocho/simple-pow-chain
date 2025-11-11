mod block;
mod blockchain;
mod message;
mod node;
mod transaction;
// Keys omitted (signature None demo)

use blockchain::Blockchain;
use clap::Parser;
use node::Node;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Peer node to connect to (format: ip:port)
    #[arg(short = 'e', long)]
    peer: Option<String>,

    /// Create genesis block (first node only)
    #[arg(long)]
    genesis: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Starting P2P blockchain node!");
    println!("Port: {}", args.port);
    if let Some(peer) = &args.peer {
        println!("Peer: {}", peer);
    }
    println!("Genesis: {}", args.genesis);

    // Initialize blockchain
    let blockchain = if args.genesis {
        Arc::new(RwLock::new(Blockchain::new(3)))
    } else {
        Arc::new(RwLock::new(Blockchain {
            chain: vec![],
            difficulty: 3,
        }))
    };

    // Set node address
    let node_addr = format!("127.0.0.1:{}", args.port);
    let peers = args.peer.map(|p| vec![p]).unwrap_or_default();

    // Create and start node
    let node = Arc::new(Node::new(
        blockchain.clone(),
        node_addr.clone(),
        peers.clone(),
    ));
    node.clone().start_listener();

    // If not genesis node and has peers, request blockchain
    if !args.genesis && !peers.is_empty() {
        sleep(Duration::from_millis(500)).await; // Wait for listener to be ready
        println!("Requesting blockchain from peers...");
        if let Err(e) = node.request_blockchain().await {
            println!("Failed to request blockchain: {}", e);
        }
    }

    // Start mining for all nodes
    node.clone().start_mining();

    // Keep node running and periodically add more transactions
    println!("Node is running. Press Ctrl+C to exit.");
    signal::ctrl_c().await.expect("Failed to listen for shutdown signal");
}
