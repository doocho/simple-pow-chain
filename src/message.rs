use serde::{Deserialize, Serialize};
use crate::block::Block;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    NewBlock(Block),
}