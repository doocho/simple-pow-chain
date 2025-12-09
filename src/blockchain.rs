use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::transaction::Transaction;

/// The blockchain - a chain of blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
}

impl Blockchain {
    /// Create a new blockchain with genesis block
    pub fn new(difficulty: usize) -> Self {
        let genesis = Block::genesis(difficulty);
        Blockchain {
            chain: vec![genesis],
            difficulty,
        }
    }

    /// Create an empty blockchain (for syncing from peers)
    pub fn empty(difficulty: usize) -> Self {
        Blockchain {
            chain: vec![],
            difficulty,
        }
    }

    /// Get the latest block
    pub fn last_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    /// Add a new block with transactions
    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> &Block {
        let (index, prev_hash) = match self.last_block() {
            Some(block) => (block.index + 1, block.hash.clone()),
            None => (0, String::from("0")),
        };

        let mut block = Block::new(index, prev_hash, transactions, self.difficulty);
        block.mine();
        self.chain.push(block);
        self.chain.last().unwrap()
    }

    /// Add an already mined block (received from network)
    pub fn add_mined_block(&mut self, block: Block) -> bool {
        if self.is_valid_new_block(&block) {
            self.chain.push(block);
            true
        } else {
            false
        }
    }

    /// Check if a new block is valid
    pub fn is_valid_new_block(&self, block: &Block) -> bool {
        let (expected_index, expected_prev_hash) = match self.last_block() {
            Some(last) => (last.index + 1, &last.hash),
            None => (0, &String::from("0")),
        };

        // Check index
        if block.index != expected_index {
            return false;
        }

        // Check previous hash
        if &block.prev_hash != expected_prev_hash {
            return false;
        }

        // Check proof of work
        if !block.is_valid_pow() {
            return false;
        }

        true
    }

    /// Validate the entire blockchain
    pub fn is_valid(&self) -> bool {
        if self.chain.is_empty() {
            return false;
        }

        // Check genesis block
        let genesis = &self.chain[0];
        if genesis.index != 0 || genesis.prev_hash != "0" {
            return false;
        }

        if !genesis.is_valid_pow() {
            return false;
        }

        // Check each subsequent block
        for i in 1..self.chain.len() {
            let block = &self.chain[i];
            let prev_block = &self.chain[i - 1];

            // Check index continuity
            if block.index != prev_block.index + 1 {
                return false;
            }

            // Check hash chain
            if block.prev_hash != prev_block.hash {
                return false;
            }

            // Check proof of work
            if !block.is_valid_pow() {
                return false;
            }
        }

        true
    }

    /// Get chain length
    pub fn len(&self) -> usize {
        self.chain.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }
}

impl std::fmt::Display for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Blockchain (difficulty: {}, blocks: {})", self.difficulty, self.len())?;
        for block in &self.chain {
            writeln!(f, "  {}", block)?;
        }
        Ok(())
    }
}
