use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::transaction::Transaction;

/// A block in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: usize,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Create a new block (not yet mined)
    pub fn new(index: u64, prev_hash: String, transactions: Vec<Transaction>, difficulty: usize) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut block = Block {
            index,
            timestamp,
            prev_hash,
            hash: String::new(),
            nonce: 0,
            difficulty,
            transactions,
        };
        block.hash = block.calculate_hash();
        block
    }

    /// Calculate SHA-256 hash of the block
    pub fn calculate_hash(&self) -> String {
        let tx_data: String = self
            .transactions
            .iter()
            .map(|tx| tx.hash())
            .collect::<Vec<String>>()
            .join("");

        let data = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.prev_hash, self.nonce, tx_data
        );

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Mine the block by finding a valid nonce
    pub fn mine(&mut self) {
        let target = "0".repeat(self.difficulty);
        loop {
            self.hash = self.calculate_hash();
            if self.hash.starts_with(&target) {
                println!("Block {} mined! Hash: {}", self.index, self.hash);
                break;
            }
            self.nonce += 1;
        }
    }

    /// Verify if the block has valid proof of work
    pub fn is_valid_pow(&self) -> bool {
        let target = "0".repeat(self.difficulty);
        self.hash == self.calculate_hash() && self.hash.starts_with(&target)
    }

    /// Create genesis block
    pub fn genesis(difficulty: usize) -> Self {
        let mut block = Block::new(0, String::from("0"), vec![], difficulty);
        block.mine();
        block
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block #{} [hash: {}..., txs: {}]",
            self.index,
            &self.hash[..16],
            self.transactions.len()
        )
    }
}
