mod block;
mod blockchain;
mod message;
mod node;
mod transaction;

use blockchain::Blockchain;
use clap::Parser;
use node::Node;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "simple-pow-chain")]
#[command(about = "A simple Bitcoin-like PoW blockchain")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Peer address to connect to (e.g., 127.0.0.1:8080)
    #[arg(short = 'e', long)]
    peer: Option<String>,

    /// Start as genesis node (create genesis block)
    #[arg(long)]
    genesis: bool,

    /// Mining difficulty (number of leading zeros)
    #[arg(short, long, default_value = "4")]
    difficulty: usize,

    /// Miner address for rewards
    #[arg(short, long, default_value = "miner")]
    miner: String,

    /// Enable auto-mining
    #[arg(long)]
    mine: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("=== Simple PoW Chain ===");
    println!("Port: {}", args.port);
    println!("Difficulty: {}", args.difficulty);
    println!("Genesis: {}", args.genesis);

    // Create blockchain
    let blockchain = if args.genesis {
        println!("Creating genesis block...");
        Blockchain::new(args.difficulty)
    } else {
        Blockchain::empty(args.difficulty)
    };

    // Setup node
    let addr = format!("127.0.0.1:{}", args.port);
    let peers = args.peer.map(|p| vec![p]).unwrap_or_default();

    if !peers.is_empty() {
        println!("Peers: {:?}", peers);
    }

    let node = Arc::new(Node::new(blockchain, addr.clone(), peers.clone()));

    // Sync from peers if not genesis
    if !args.genesis && !peers.is_empty() {
        println!("Syncing from peers...");
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        if let Err(e) = node.sync().await {
            eprintln!("Sync error: {}", e);
        }
    }

    // Start mining in background if enabled
    if args.mine {
        let mining_node = node.clone();
        let miner = args.miner.clone();
        tokio::spawn(async move {
            println!("Starting miner...");
            loop {
                if let Some(block) = mining_node.mine(&miner).await {
                    mining_node.broadcast_block(&block).await;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    }

    // Start listening
    println!("Node starting on {}", addr);
    if let Err(e) = node.start().await {
        eprintln!("Node error: {}", e);
    }
}
