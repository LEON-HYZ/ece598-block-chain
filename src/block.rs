use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
use crate::crypto::merkle::{MerkleTree};
use crate::transaction::{Transaction};
use crate::transaction::generate_random_transaction_;


use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use ring::{digest};

use std::time::{Duration, SystemTime};
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    parent: H256,
    nonce: u32,
    difficulty: H256,
    timestamp: i64,
    merkleRoot: H256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    content: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
	Header: Header,
    Content: Content,
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
}

pub fn timestamp() -> i64 {
    let now = Utc::now();
    now.timestamp_millis()
}

pub fn generate_random_block_(parent: &H256) -> Block {
        let mut nonce:u32 = thread_rng().gen();
        let mut timestamp = timestamp();
        let mut difficulty = <H256>::from(digest::digest(&digest::SHA256, b"difficulty"));
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
    	let mut timestamp = timestamp();
    	let mut difficulty = <H256>::from(digest::digest(&digest::SHA256, b"difficulty"));
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
