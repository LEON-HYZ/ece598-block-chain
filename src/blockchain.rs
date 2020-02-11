use crate::block::Block;
use crate::crypto::hash::{H256,Hashable};
use crate::block::test::generate_random_block;
use ring::{digest};

use std::collections::HashMap;

pub struct Blockchain {
     Blocks: HashMap<H256,Block>,
     genesis_hash: H256,
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
        return Blockchain {Blocks: Blocks,genesis_hash:genesis_hash, tip: tip};
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let lblock = block.clone();
        self.Blocks.insert(block.hash(), lblock);
        self.tip = block.hash();
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        return self.tip;
    }

    /// Get the last block's hash of the longest chain
    #[cfg(any(test, test_utilities))]
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let mut blockLists = Vec::<H256>::new();
        let mut hash = self.tip;
        while hash != self.genesis_hash {
            blockLists.push(hash);
            hash = (self.Blocks.get(&hash)).as_ref().unwrap().hash();
        }
        blockLists.push(self.genesis_hash);
        return blockLists;
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
        println!("genesis_hash:{:?}", genesis_hash);
        println!("tip1:{:?}", blockchain.tip);
        let block = generate_random_block(&genesis_hash);
        println!("tip2:{:?}", blockchain.tip);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());
    }
}
