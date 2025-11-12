use serde::{Deserialize, Serialize};
use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    NewBlock(Block),
    RequestBlockchain,
    ResponseBlockchain(Blockchain),
    NewTransaction(Transaction),
}