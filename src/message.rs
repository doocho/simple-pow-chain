use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::transaction::Transaction;

/// Network messages for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Broadcast a newly mined block
    NewBlock(Block),
    /// Broadcast a new transaction
    NewTransaction(Transaction),
    /// Request the full blockchain
    GetBlocks,
    /// Response with the full blockchain
    Blocks(Blockchain),
    /// Register with seed node (send our address)
    Register(String),
    /// Request peer list from seed node
    GetPeers,
    /// Response with peer list
    Peers(Vec<String>),
}
