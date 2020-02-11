use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
use crate::transaction::{Transaction};

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
	Header: Option<Box<Header>>,
    Content: Option<Box<Content>>,
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        let header = self.Header.as_ref().unwrap();
        let header_serialized = bincode::serialize(&header).unwrap();
        return ring::digest::digest(&ring::digest::SHA256, &header_serialized).into();
    }
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        let t_serialized = bincode::serialize(&self).unwrap();
        return ring::digest::digest(&ring::digest::SHA256, &t_serialized).into();
    }
}

pub fn timestamp() -> i64 {
    let now = Utc::now();
    now.timestamp_millis()
}

#[cfg(any(test, test_utilities))]
pub mod test {
    use super::*;
    use crate::crypto::hash::H256;

    pub fn generate_random_block(parent: &H256) -> Block {
    	let mut nonce:u32 = thread_rng().gen();
    	let mut timestamp = timestamp();
    	let mut difficulty = <H256>::from(digest::digest(&digest::SHA256, b"difficulty"));
    	let mut transaction = Vec::<Transaction>::new();

        let newHeader = Header{
        	parent: *parent,
    		nonce: nonce,
    		difficulty: difficulty,
    		timestamp: timestamp,
    		merkleRoot: transaction[0].hash(),
        };

        let newContent = Content{
        	content: transaction,
        };

        let newBlock = Block{
        	Header: Some(Box::new(newHeader)),
    		Content: Some(Box::new(newContent)),
        };

        return newBlock;
    }
}
