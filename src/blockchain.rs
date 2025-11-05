use crate::block::Block;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let genesis = Block::new(0, "0".to_string(), "Genesis Block".to_string(), difficulty);
        let mut genesis_block = genesis;
        genesis_block.mine();

        Blockchain {
            chain: vec![genesis_block],
            difficulty,
        }
    }

    pub fn add_block(&mut self, data: String) {
        let last_block = self.chain.last().unwrap();
        let mut new_block = Block::new(
            last_block.index + 1,
            last_block.hash.clone(),
            data,
            self.difficulty,
        );
        new_block.mine();
        self.chain.push(new_block);
    }
}