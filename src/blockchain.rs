use crate::block::Block;
use crate::transaction::Transaction;
#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let mut bc = Self {
            chain: vec![],
            difficulty,
        };
        let genesis_tx = Transaction::new("genesis".to_string(), "genesis".to_string(), 0);
        bc.add_block(vec![genesis_tx]);
        bc
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let (index, previous_hash)= if self.chain.is_empty() {
            (0, "0".to_string())
        } else {
            let last = self.chain.last().unwrap();
            (last.index + 1, last.hash.clone())
        };
        
        let mut new_block = Block::new(
            index,
            previous_hash,
            transactions,
            self.difficulty,
        );
        new_block.mine();
        self.chain.push(new_block);
    }
}