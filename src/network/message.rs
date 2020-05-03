use serde::{Serialize, Deserialize};
use crate::block::Block;
use crate::crypto::hash::{H256, Hashable, H160};
use crate::transaction::{Transaction, SignedTransaction,StateWitness};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping(String),
    Pong(String),
    NewBlockHashes(Vec<H256>),
    GetBlocks(Vec<H256>),
    Blocks(Vec<Block>),
    NewTransactionHashes(Vec<H256>),
    GetTransactions(Vec<H256>),
    Transactions(Vec<SignedTransaction>),
    //TODO:Update State Witness, Accumulator Proof
    NewStateWitness(Vec<(H256, u32, f32, H160, u32, u32)>,Vec<(H256,u32)>),
}

