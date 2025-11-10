mod block;
mod blockchain;
mod transaction;
mod message;
mod node;
// Keys omitted (signature None demo)

use clap::Parser;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use transaction::Transaction;
use blockchain::Blockchain;
use node::Node;

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
    let node = Node::new(blockchain.clone(), node_addr, peers);
    node.start_listener();

    // If genesis node, broadcast genesis block
    if args.genesis {
        sleep(Duration::from_millis(500)).await;  // Wait for listener to be ready
        let genesis = blockchain.read().unwrap().chain[0].clone();
        println!("Broadcasting genesis block...");
        node.broadcast_new_block(genesis).await;
    }

    // Demo: Mine new transaction block (genesis node only)
    if args.genesis {
        sleep(Duration::from_millis(1000)).await;
        println!("Mining new block...");
        {
            let mut bc_w = blockchain.write().unwrap();
            let txs = vec![
                Transaction::new("alice".to_string(), "bob".to_string(), 10),
                Transaction::new("bob".to_string(), "carol".to_string(), 5),
            ];
            bc_w.add_block(txs);
        }
        let new_block = blockchain.read().unwrap().chain.last().unwrap().clone();
        println!("Broadcasting new block...");
        node.broadcast_new_block(new_block).await;
    }

    // Keep node running
    println!("Node is running. Press Ctrl+C to exit.");
    loop {
        if args.genesis {
            add_block(blockchain.clone(), &node).await;
        }
        sleep(Duration::from_secs(12)).await;
        let chain_len = blockchain.read().unwrap().chain.len();
        println!("Current chain length: {}", chain_len);
    }
}

async fn add_block(blockchain: Arc<RwLock<Blockchain>>, node: &Node) {
    sleep(Duration::from_millis(1000)).await;
    println!("Mining new block...");
    {
        let mut bc_w = blockchain.write().unwrap();
        let txs = vec![
            Transaction::new("alice".to_string(), "bob".to_string(), 10),
            Transaction::new("bob".to_string(), "carol".to_string(), 5),
        ];
        bc_w.add_block(txs);
    }
    let new_block = blockchain.read().unwrap().chain.last().unwrap().clone();
    println!("Broadcasting new block...");
    node.broadcast_new_block(new_block).await;
}