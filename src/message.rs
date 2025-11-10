use serde::{Deserialize, Serialize};
use crate::block::Block;
use crate::blockchain::Blockchain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    NewBlock(Block),
    RequestBlockchain,
    ResponseBlockchain(Blockchain),
}