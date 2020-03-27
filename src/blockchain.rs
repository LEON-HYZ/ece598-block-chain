use crate::block::Block;
use crate::crypto::hash::{H256,Hashable};
use crate::block::generate_random_block_;
use ring::{digest};
extern crate rand;

use std::collections::HashMap;

pub struct Blockchain {
     pub Blocks: HashMap<H256,(Block, u32)>, //H256 - <Block,Height>
     pub genesis_hash: H256,
     pub tip: (H256, u32),
}



impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let mut Blocks:HashMap<H256,(Block, u32)> = HashMap::new();
        let genesis_hash = <H256>::from(digest::digest(&digest::SHA256, &[0x00 as u8]));
        let mut block = generate_random_block_(&genesis_hash);
        Blocks.insert(genesis_hash,(block, 0));
        let tip = (genesis_hash, 0);
        return Blockchain {Blocks: Blocks,genesis_hash:genesis_hash, tip: tip,};
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) -> H256 {
        let last_block = block.clone();
        let h = self.Blocks.get(&(last_block.getparent())).as_ref().unwrap().1 + 1;
        self.Blocks.insert(block.hash(), (last_block, h));
        if self.tip.1 < h{
            self.tip = (block.hash(), h);

        }
        // self.tip = block.hash();
        return self.tip.0;
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        return self.tip.0;
    }

    /// Get the last block's hash of the longest chain
    //#[cfg(any(test, test_utilities))]
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let mut blockLists = Vec::<H256>::new();
        let mut hash = self.tip.0;
        let mut height = self.tip.1;
        while height != 0 {
            blockLists.insert(0,hash);
            let parentHash = self.Blocks.get(&hash).as_ref().unwrap().0.getparent();
            hash = parentHash;
            height = self.Blocks.get(&hash).as_ref().unwrap().1;
        }
        blockLists.insert(0,self.genesis_hash);
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
