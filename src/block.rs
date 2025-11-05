use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::fmt;
use crate::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub timestamp: u64,
    pub previous_hash: String,
    pub hash: String,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub difficulty: usize, // Number of leading zeros required
}

impl Block {
    pub fn new(index: u32, previous_hash: String, transactions: Vec<Transaction>, difficulty: usize) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Block {
            index,
            timestamp,
            previous_hash,
            transactions,
            hash: String::new(),
            nonce: 0,
            difficulty,
        }
    }

    pub fn calculate_hash(&self) -> String {
        let txs_json = serde_json::to_string(&self.transactions).unwrap();
        let input = format!(
            "{}{}{}{}{}{}",
            self.index, self.timestamp, &self.previous_hash, txs_json, self.nonce, self.difficulty
        );
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn mine(&mut self) {
        let target = "0".repeat(self.difficulty);
        loop {
            self.hash = self.calculate_hash();
            if self.hash.starts_with(&target) {
                println!("⛏️  Mined! nonce: {}, hash: {}", self.nonce, self.hash);
                break;
            }
            self.nonce += 1;
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Block[{}]: {:?} | nonce: {} | hash: {}",
            self.index, self.transactions, self.nonce, self.hash
        )
    }
}