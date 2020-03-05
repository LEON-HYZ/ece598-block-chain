use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
use crate::crypto::merkle::{MerkleTree};
use crate::transaction::{Transaction};
use crate::transaction::generate_random_transaction_;


use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use ring::{digest};

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::prelude::*;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub parent: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128,
    pub merkleRoot: H256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub content: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
	pub Header: Header,
    pub Content: Content,
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        let header_serialized = bincode::serialize(&self.Header).unwrap();
        return ring::digest::digest(&ring::digest::SHA256, &header_serialized).into();
    }
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        let t_serialized = bincode::serialize(&self).unwrap();
        return ring::digest::digest(&ring::digest::SHA256, &t_serialized).into();
    }
}

impl Block{
    pub fn getparent(&self) -> H256 {
        self.Header.parent
    }

    pub fn getdifficulty(&self) -> H256 {
        self.Header.difficulty
    }
}


pub fn generate_random_block_(parent: &H256) -> Block {
        let mut nonce:u32 = thread_rng().gen();
        let mut timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
        let mut bytes32 = [255u8;32];
        bytes32[0]=0;
        bytes32[1]=5;
        let difficulty : H256 = bytes32.into();
        let mut transaction = Vec::<Transaction>::new();
        transaction.push(generate_random_transaction_());
        let mut MerkleTree = MerkleTree::new(&transaction);



        let newHeader = Header{
            parent: *parent,
            nonce: nonce,
            difficulty: difficulty,
            timestamp: timestamp,
            merkleRoot: MerkleTree.root(),
        };

        let newContent = Content{
            content: transaction,
        };

        let newBlock = Block{
            Header: newHeader,
            Content: newContent,
        };

        return newBlock;
}

#[cfg(any(test, test_utilities))]
pub mod test {
    use super::*;
    use crate::crypto::hash::H256;
    use crate::transaction::tests::generate_random_transaction;

    pub fn generate_random_block(parent: &H256) -> Block {
    	let mut nonce:u32 = thread_rng().gen();
    	let mut timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
    	
        let mut bytes32 = [255u8;32];
        bytes32[0]=0;
        bytes32[1]=5;
        let difficulty : H256 = bytes32.into();
        
    	let mut transaction = Vec::<Transaction>::new();
    	transaction.push(generate_random_transaction());
    	let mut MerkleTree = MerkleTree::new(&transaction);



        let newHeader = Header{
        	parent: *parent,
    		nonce: nonce,
    		difficulty: difficulty,
    		timestamp: timestamp,
    		merkleRoot: MerkleTree.root(),
        };

        let newContent = Content{
        	content: transaction,
        };

        let newBlock = Block{
        	Header: newHeader,
    		Content: newContent,
        };

        return newBlock;
    }
}
