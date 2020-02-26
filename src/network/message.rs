use serde::{Serialize, Deserialize};
use crate::block::Block;
use crate::crypto::hash::{H256,Hashable};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping(String),
    Pong(String),
    NewBlockHashes(Vec<H256>),
    GetBlocks(Vec<H256>),
    Blocks(Vec<Block>),
}

