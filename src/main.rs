mod block;
mod blockchain;
mod message;
mod node;
mod seed;
mod transaction;

use blockchain::Blockchain;
use clap::{Parser, Subcommand};
use node::Node;
use seed::SeedNode;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "simple-pow-chain")]
#[command(about = "A simple Bitcoin-like PoW blockchain")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a blockchain node
    Node {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Seed node address to discover peers (e.g., 127.0.0.1:9000)
        #[arg(short, long)]
        seed: Option<String>,

        /// Direct peer address (e.g., 127.0.0.1:8080)
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
    },
    /// Run a seed node for peer discovery
    Seed {
        /// Port to listen on
        #[arg(short, long, default_value = "9000")]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Node {
            port,
            seed,
            peer,
            genesis,
            difficulty,
            miner,
            mine,
        } => {
            run_node(port, seed, peer, genesis, difficulty, miner, mine).await;
        }
        Commands::Seed { port } => {
            run_seed(port).await;
        }
    }
}

async fn run_node(
    port: u16,
    seed_addr: Option<String>,
    peer: Option<String>,
    genesis: bool,
    difficulty: usize,
    miner: String,
    mine: bool,
) {
    println!("=== Simple PoW Chain ===");
    println!("Port: {}", port);
    println!("Difficulty: {}", difficulty);
    println!("Genesis: {}", genesis);

    // Create blockchain
    let blockchain = if genesis {
        println!("Creating genesis block...");
        Blockchain::new(difficulty)
    } else {
        Blockchain::empty(difficulty)
    };

    // Setup node address
    let addr = format!("127.0.0.1:{}", port);

    // Collect initial peers
    let mut peers: Vec<String> = peer.map(|p| vec![p]).unwrap_or_default();

    // Discover peers from seed node
    if let Some(ref seed) = seed_addr {
        println!("Connecting to seed node: {}", seed);

        // Register ourselves with seed
        if let Err(e) = seed::register_with_seed(seed, &addr).await {
            eprintln!("Failed to register with seed: {}", e);
        } else {
            println!("Registered with seed node");
        }

        // Get peer list from seed
        match seed::get_peers_from_seed(seed).await {
            Ok(discovered) => {
                println!("Discovered {} peers from seed", discovered.len());
                for p in discovered {
                    if p != addr && !peers.contains(&p) {
                        peers.push(p);
                    }
                }
            }
            Err(e) => eprintln!("Failed to get peers from seed: {}", e),
        }
    }

    if !peers.is_empty() {
        println!("Peers: {:?}", peers);
    }

    let node = Arc::new(Node::new(blockchain, addr.clone(), peers.clone()));

    // Sync from peers if not genesis
    if !genesis && !peers.is_empty() {
        println!("Syncing from peers...");
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        if let Err(e) = node.sync().await {
            eprintln!("Sync error: {}", e);
        }
    }

    // Start mining in background if enabled
    if mine {
        let mining_node = node.clone();
        let miner_addr = miner.clone();
        tokio::spawn(async move {
            println!("Starting miner...");
            loop {
                if let Some(block) = mining_node.mine(&miner_addr).await {
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

async fn run_seed(port: u16) {
    println!("=== Seed Node ===");
    let addr = format!("127.0.0.1:{}", port);
    let seed = SeedNode::new(addr);

    if let Err(e) = seed.start().await {
        eprintln!("Seed node error: {}", e);
    }
}
