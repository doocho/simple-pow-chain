use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u64) -> Self {
        Transaction { from, to, amount }
    }

    /// Generate a hash based on the transaction contents
    pub fn hash(&self) -> String {
        let input = format!("{}{}{}", self.from, self.to, self.amount);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} → {}: {} coins", self.from, self.to, self.amount)
    }
}
