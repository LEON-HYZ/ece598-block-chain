use super::message::Message;
use super::peer;
use crate::network::server::Handle as ServerHandle;
use crossbeam::channel;
use log::{debug, warn};

use std::sync::{Arc, Mutex};
use crate::crypto::hash::{H256, Hashable, H160};
use crate::blockchain::Blockchain;
use crate::block::{Block,Header,Content};
use crate::crypto::merkle::{MerkleTree};
use crate::transaction::{Mempool, State, SignedTransaction};
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};



use std::thread;
use crate::transaction;

#[derive(Debug, Clone)]
pub struct OrphanBuffer {
    //Using HashMap for orphan_buffer
    pub HashMap: HashMap<H256,Vec<Block>>,
}



impl OrphanBuffer {
    pub fn new() -> Self{
        let mut orphan_buffer:HashMap<H256,Vec<Block>> = HashMap::new(); // Parent <-> Vec<Orphan>
        return OrphanBuffer{HashMap:orphan_buffer};
    }

    pub fn getOrphanBlocks(&self, Hash:&H256) -> Vec<Block> {
        let mut Blocks = Vec::<Block>::new();
        if self.isParentIn(Hash) {
            Blocks = self.HashMap.get(Hash).unwrap().clone();
        }
        return Blocks;
    }

    pub fn insert(&mut self, Parent:H256, Blocks:Vec<Block>) {
        self.HashMap.insert(Parent,Blocks.clone());
    }

    pub fn remove(&mut self, Hash:&H256) {
        if self.isParentIn(Hash) {
            self.HashMap.remove(Hash);
        }
    }

    pub fn isParentIn(&self, Hash:&H256) -> bool {
        return !self.HashMap.get(Hash).is_none();
    }


}



#[derive(Clone)]
pub struct Context {
    blockchain: Arc<Mutex<Blockchain>>,
    orphanbuffer: Arc<Mutex<OrphanBuffer>>,
    mempool: Arc<Mutex<Mempool>>,
    state: Arc<Mutex<State>>,
    local_address: H160,
    // sum_delay: &Arc<Mutex<f32>>,
    // num_delay: &Arc<Mutex<u8>>,
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
}

pub fn new(
    blockchain: &Arc<Mutex<Blockchain>>,
    orphanbuffer: &Arc<Mutex<OrphanBuffer>>,
    mempool: &Arc<Mutex<Mempool>>,
    state: &Arc<Mutex<State>>,
    local_address: &H160,
    // sum_delay: &Arc<Mutex<f32>>,
    // num_delay: &Arc<Mutex<u8>>,
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
) -> Context {
    Context {
        blockchain: Arc::clone(blockchain),
        orphanbuffer: Arc::clone(orphanbuffer),
        mempool: Arc::clone(mempool),
        state: Arc::clone(state),
        local_address: *local_address,
        // sum_delay: Arc::clone(sum_delay),
        // num_delay: Arc::clone(num_delay),
        msg_chan: msg_src,
        num_worker,
        server: server.clone(),
    }
}

impl Context {
    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }



    fn worker_loop(&self) {
        loop {
            let msg = self.msg_chan.recv().unwrap();
            let (msg, peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(hashes) => {
                    //debug!("NewBlockHashes: {:?}", hashes);
                    //self.server.broadcast(Message::NewBlockHashes(hashes.clone()));
                    let mut notContainedHashes = Vec::<H256>::new();
                    let mut blockchain = self.blockchain.lock().unwrap();
                    let mut orphanbuffer = self.orphanbuffer.lock().unwrap();
                    if hashes.len() != 0 {
                        for hash in hashes.iter() {
                            if blockchain.Blocks.get(&hash).is_none() {
                                notContainedHashes.push(*hash);
                            }
                        }
                    }
                    if notContainedHashes.len() != 0 {
                        peer.write(Message::GetBlocks(notContainedHashes));
                    }


                }

                Message::GetBlocks(hashes) => {
                    //debug!("GetBlocks: {:?}", hashes);
                    let mut notContainedBlocks = Vec::<Block>::new();
                    let mut hashes = hashes.clone();
                    let mut blockchain = self.blockchain.lock().unwrap();
                    if hashes.len() != 0 {
                        for hash in hashes.iter() {
                            if !blockchain.Blocks.get(&hash).is_none(){
                                let block = blockchain.Blocks.get(&hash).unwrap().0.clone();
                                notContainedBlocks.push(block);
                            }
                        }
                    }
                    if notContainedBlocks.len() != 0 {
                        peer.write(Message::Blocks(notContainedBlocks));
                    }

                }

                Message::Blocks(blocks) => {
                    //debug!("Blocks: {:?}", blocks);
                    let mut blocks = blocks.clone();
                    let mut blockchain = self.blockchain.lock().unwrap();
                    let mut orphanbuffer = self.orphanbuffer.lock().unwrap();

                    let mut newlyOrphanParent = Vec::<H256>::new();
                    let mut newlyProcessedBlockHashes = Vec::<H256>::new();

                    for block in blocks.iter() {

                        //PoW check
                        let difficulty = blockchain.Blocks.get(&blockchain.tip.0).unwrap().0.getdifficulty();
                        if block.hash() <=  difficulty{
                            if block.Header.difficulty == difficulty{
                                if !blockchain.Blocks.get(&block.getparent()).is_none(){
                                    //println!("block parent: {:?}", block.getparent());
                                    //println!("tip H256: {:?}", blockchain.tip.0);
                                    blockchain.insert(&block);
                                    //TODO: Update State

                                    let mut now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
                                    let mut delay_u128 = now - block.gettimestamp();
                                    let delay = delay_u128 as f32;
                                    // sum_delay += delay;
                                    // num_delay += 1;
                                    println!("delay: {:?}", delay);
                                    // println!("sum_delay: {:?}", sum_delay);
                                    // println!("num_delay: {:?}", num_delay);
                                    newlyProcessedBlockHashes.push(block.hash());
                                }
                                else{// orphan blocks created only when blocks were not inserted.
                                    // Insert orphan blocks into buffer
                                    let mut newlyOrphans = Vec::<Block>::new();
                                    if orphanbuffer.isParentIn(&block.getparent()){
                                        newlyOrphans = orphanbuffer.getOrphanBlocks(&block.getparent());
                                    }
                                    newlyOrphans.push(block.clone());
                                    orphanbuffer.insert(block.getparent(),newlyOrphans);
                                    //println!("orphan inserted: {:?}", block.getparent());
                                    newlyOrphanParent.push(block.getparent());
                                }
                            }
                        }
                    }
                    peer.write(Message::GetBlocks(newlyOrphanParent)); //Send GetBlocks messages for getting parent blocks of orphans
                    // println!("orphan buffer length: {:?}", orphanbuffer.HashMap.len());
                    // Orphan Handler
                    for idx in 0..newlyProcessedBlockHashes.len(){
                        if orphanbuffer.isParentIn(&newlyProcessedBlockHashes[idx]){
                            let orphans = orphanbuffer.getOrphanBlocks(&newlyProcessedBlockHashes[idx]);
                            for orphan in orphans{
                                blockchain.insert(&orphan);
                                let mut now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
                                let mut delay_u128 = now - orphan.gettimestamp();
                                let delay = delay_u128 as f32;
                                println!("delay: {:?}", delay);
                                newlyProcessedBlockHashes.push(orphan.hash());
                            }
                            orphanbuffer.remove(&newlyProcessedBlockHashes[idx]);
                        }
                    }


                    // println!("Current height of blockchain: {:?}", blockchain.tip.1);
                    self.server.broadcast(Message::NewBlockHashes(newlyProcessedBlockHashes));
                }


                Message::NewTransactionHashes(hashes) => {
                    let mut mempool = self.mempool.lock().unwrap();
                    let mut notContainedHashes = Vec::<H256>::new();
                    if hashes.len() != 0 {
                        for hash in hashes.iter() {
                            if mempool.Transactions.get(&hash).is_none() {
                                notContainedHashes.push(*hash);
                            }
                        }
                    }
                    if notContainedHashes.len() != 0{
                        peer.write(Message::GetTransactions(notContainedHashes));
                    }
                }

                Message::GetTransactions(hashes) => {
                    let mut mempool = self.mempool.lock().unwrap();
                    let mut notContainedTransactions = Vec::<SignedTransaction>::new();
                    let mut hashes = hashes.clone();
                    if hashes.len() != 0 {
                        for hash in hashes.iter() {
                            if !mempool.Transactions.get(&hash).is_none() {
                                notContainedTransactions.push(mempool.Transactions.get(&hash).unwrap().clone());
                            }
                        }
                    }
                    if notContainedTransactions.len() != 0{
                        peer.write(Message::Transactions(notContainedTransactions));
                    }

                }

                Message::Transactions(Transactions) => {
                    let mut mempool = self.mempool.lock().unwrap();
                    let mut state = self.state.lock().unwrap();
                    let mut Transactions = Transactions.clone();
                    let mut addedTransactionHashes = Vec::<H256>::new();

                    for Transaction in Transactions.iter(){
                        if !mempool.Transactions.contains_key(&Transaction.hash()) {
                            //Transaction signature check
                            let public_key = &Transaction.publicKey[..];
                            let signature = &Transaction.signature[..];
                            let public_key_ = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key.as_ref());
                            if public_key_.verify(&bincode::serialize(&Transaction).unwrap()[..],signature.as_ref()) == Ok(()){
                                mempool.insert(Transaction);
                                addedTransactionHashes.push(Transaction.hash());
                            }
                        }
                    }
                    if addedTransactionHashes.capacity() > 0 {
                        self.server.broadcast(Message::NewTransactionHashes(addedTransactionHashes));
                    }

                }

            }
        }
    }
}
