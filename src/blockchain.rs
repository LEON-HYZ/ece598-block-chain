use crate::block::Block;
use crate::crypto::hash::{H256,Hashable};
use crate::block::test::generate_random_block;
use ring::{digest};

use std::collections::HashMap;

pub struct Blockchain {
     Blocks: HashMap<H256,Block>,
     tip: H256,
}



impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let mut Blocks:HashMap<H256,Block> = HashMap::new();
        let genesis_hash = <H256>::from(digest::digest(&digest::SHA256, &[0x00 as u8]));
        let block = generate_random_block(&genesis_hash);
        Blocks.insert(genesis_hash,block);
        let tip = genesis_hash;
        return Blockchain {Blocks: Blocks, tip: tip};
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {

        self.Blocks.insert(block.hash(), *block);
        self.tip = block.hash();
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        return self.tip;
    }

    /// Get the last block's hash of the longest chain
    #[cfg(any(test, test_utilities))]
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        unimplemented!()
    }
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::block::test::generate_random_block;
    use crate::crypto::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());
    }
}
