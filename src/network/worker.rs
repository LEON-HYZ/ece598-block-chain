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
use crate::transaction::{Mempool, StateWitness, StateSet, SignedTransaction};
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use log::info;



use std::{thread, time};
use crate::transaction;
use std::ascii::escape_default;

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
    stateWitness: Arc<Mutex<StateWitness>>,
    //stateSet: Arc<Mutex<StateSet>>,
    local_address: H160,
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    ifArchival: bool,
}

pub fn new(
    blockchain: &Arc<Mutex<Blockchain>>,
    orphanbuffer: &Arc<Mutex<OrphanBuffer>>,
    mempool: &Arc<Mutex<Mempool>>,
    stateWitness: &Arc<Mutex<StateWitness>>,
    //stateSet: &Arc<Mutex<StateSet>>,
    local_address: &H160,
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
    ifArchival: bool
) -> Context {
    Context {
        blockchain: Arc::clone(blockchain),
        orphanbuffer: Arc::clone(orphanbuffer),
        mempool: Arc::clone(mempool),
        stateWitness: Arc::clone(stateWitness),
        //stateSet: Arc::clone(stateSet),
        local_address: *local_address,
        // sum_delay: Arc::clone(sum_delay),
        // num_delay: Arc::clone(num_delay),
        msg_chan: msg_src,
        num_worker,
        server: server.clone(),
        ifArchival: *ifArchival,
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
                    //info!("WORKER: RECEIVED BLOCK MESSAGES");
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
                    //info!("WORKER: BLOCKS REQUIRED");
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
                    //info!("WORKER: START RECEIVING BLOCKS");
                    let mut blocks = blocks.clone();
                    let mut blockchain = self.blockchain.lock().unwrap();
                    let mut orphanbuffer = self.orphanbuffer.lock().unwrap();

                    let mut newlyOrphanParent = Vec::<H256>::new();
                    let mut newlyProcessedBlockHashes = Vec::<H256>::new();

                    for block in blocks.iter() {
                        //info!("WORKER: RECEIVING BLOCKS...");
                        //PoW check
                        let difficulty = blockchain.Blocks.get(&blockchain.tip.0).unwrap().0.getdifficulty();
                        if block.hash() <=  difficulty{
                            //info!("WORKER: DIFFICULTY CHECK1 SUCCESS");
                            if block.Header.difficulty == difficulty{
                                //info!("WORKER: DIFFICULTY CHECK2 SUCCESS");
                                if !blockchain.Blocks.get(&block.getparent()).is_none(){
                                    //info!("WORKER: PARENT CHECK SUCCESS");
                                    //println!("block parent: {:?}", block.getparent());
                                    //println!("WORKER: PRESENT TIP {:?}", blockchain.tip.0);
                                    //double spend check & signature check
                                    let mut contents = block.Content.content.clone();
                                    //let mut state = self.state.lock().unwrap();
                                    let mut stateWitness = self.stateWitness.lock().unwrap();
                                    let mut mempool = self.mempool.lock().unwrap();
                                    let mut check = true;

                                    for content in contents.iter(){
                                        //println!("verify: {:?}, double: {:?}",content.verifySignedTransaction() , state.ifNotDoubleSpent(content));
                                        if stateWitness.ifNotDoubleSpent(&content.transaction.Input, &block.getparent()) && content.verifySignedTransaction() { //TODO BATCH VERIFICATION
                                            check = check && true;
                                        }
                                        else{
                                            check = check && false;
                                            break;
                                        }
                                    }

                                    std::mem::drop(stateWitness);
                                    std::mem::drop(mempool);

                                    if check{
                                        let tip_hash = blockchain.insert(&block);

                                        info!("WORKER: BLOCKS RECEIVED FROM THE OTHER SENDER");
                                        println!("WORKER: CURRENT BLOCKCHAIN HEIGHT: {:?}", blockchain.tip.1);
                                        // info!("Worker: Blocks mined by one can be received by the other.");
                                        //let mut stateWitness = self.stateWitness.lock().unwrap();
                                        let mut mempool = self.mempool.lock().unwrap();
                                        // CODE REVERSE WHEN A FORK APPEARS
                                        /*
                                        if stateSet.Set.contains_key(&tip_hash) {
                                            // let new_state = stateSet.Set.get(&tip_hash).unwrap().Outputs;
                                            state.Outputs.clear();
                                            for (key, value) in stateSet.Set.get(&tip_hash).unwrap().Outputs.clone() {
                                                state.Outputs.insert(key, value);
                                            }
                                        } */
                                        // Update State
                                        //state.updateState(&contents);
                                        // CODE STATE SET UPDATE
                                        //stateSet.Set.insert(block.hash(), state.clone());
                                        // Update Mempool
                                        mempool.updateMempool(&contents);
                                        /*for key in state.Outputs.keys() {
                                            println!("WORKER: RECP: {:?}, VALUE {:?} BTC", state.Outputs.get(key).unwrap().1, state.Outputs.get(key).unwrap().0);
                                        }*/
                                        if self.ifArchival { //TODO
                                            //update state witnesses and broadcast state witnesses with accumulator proof
                                        }

                                        newlyProcessedBlockHashes.push(block.hash());
                                        //std::mem::drop(state);
                                        std::mem::drop(mempool);
                                    }


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
                                let mut contents = orphan.Content.content.clone();
                                //let mut state = self.state.lock().unwrap(); //TODO
                                let mut stateWitness = self.stateWitness.lock().unwrap();
                                let mut mempool = self.mempool.lock().unwrap();
                                let mut check = true;
                                for content in contents.iter(){
                                    if stateWitness.ifNotDoubleSpent(&content.transaction.Input, &orphan.getparent()) && content.verifySignedTransaction() { //TODO
                                        check = check && true;
                                    }
                                    else{
                                        check = check && false;
                                        break;
                                    }
                                }
                                std::mem::drop(state);
                                std::mem::drop(mempool);
                                if check {
                                    blockchain.insert(&orphan);
                                    //let mut state = self.state.lock().unwrap();
                                    let mut mempool = self.mempool.lock().unwrap();
                                    //info!("WORKER: BLOCKS RECEIVED");
                                    // info!("Worker: Blocks mined by one can be received by the other.");
                                    // Update State
                                    //state.updateState(&contents); //TODO
                                    //Update Mempool
                                    mempool.updateMempool(&contents);

                                    info!("WORKER: ORPHAN BLOCKS RECEIVED");
                                    //std::mem::drop(state);
                                    std::mem::drop(mempool);
                                }


                            }
                            orphanbuffer.remove(&newlyProcessedBlockHashes[idx]);
                        }
                    }


                    // println!("Current height of blockchain: {:?}", blockchain.tip.1);
                    self.server.broadcast(Message::NewBlockHashes(newlyProcessedBlockHashes));
                }


                Message::NewTransactionHashes(hashes) => {
                    //println!("newTransactionhashes: {:?}",hashes);
                    //info!("WORKER: NEW TRANSACTION HASHES RECEIVED");
                    let mut mempool = self.mempool.lock().unwrap();
                    let mut notContainedHashes = Vec::<H256>::new();
                    if hashes.len() != 0 {
                        for hash in hashes.iter() {
                            if mempool.Transactions.get(&hash).is_none() {
                                notContainedHashes.push(*hash);
                            }
                        }
                    }
                    std::mem::drop(mempool);
                    if notContainedHashes.len() != 0{
                        peer.write(Message::GetTransactions(notContainedHashes));
                    }
                }

                Message::GetTransactions(hashes) => {
                    //info!("WORKER: NEW TRANSACTIONS REQUIRED");
                    //println!("getTransactionhashes: {:?}",hashes);
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
                    std::mem::drop(mempool);
                    if notContainedTransactions.len() != 0{
                        //info!("WORKER: SENDING TXS");
                        peer.write(Message::Transactions(notContainedTransactions));
                    }

                }

                Message::Transactions(Transactions) => {
                    //info!("WORKER: ADDING NEW TRANSACTIONS");
                    //println!("Transactions: {:?}",Transactions);
                    let mut mempool = self.mempool.lock().unwrap();
                    let mut stateWitness = self.stateWitness.lock().unwrap();
                    let mut Transactions = Transactions.clone();
                    let mut addedTransactionHashes = Vec::<H256>::new();

                    for Transaction in Transactions.iter(){
                        if !mempool.Transactions.contains_key(&Transaction.hash()) {
                            //Transaction signature check
                            //info!("checking");
                            //println!("verify: {:?}, double: {:?}",Transaction.verifySignedTransaction() , state.ifNotDoubleSpent(Transaction));
                            if Transaction.verifySignedTransaction() && stateWitness.ifNotDoubleSpent(&Transaction.transaction.Input, &self.blockchain.lock().unwrap().tip.0) {
                                //info!("added");
                                info!("WORKER: NEW TRANSACTIONS ADDED!");
                                mempool.insert(Transaction);
                                addedTransactionHashes.push(Transaction.hash());
                            }
                        }
                    }
                    std::mem::drop(mempool);
                    std::mem::drop(stateWitness);
                    if addedTransactionHashes.capacity() > 0 {
                        self.server.broadcast(Message::NewTransactionHashes(addedTransactionHashes));
                    }
                    //println!("updated mempool: {:?}",mempool.Transactions);
                }

                Message::NewStateWitness(newStateWitness) => {
                    if !self.ifArchival {
                        let mut stateWitness = self.stateWitness.lock().unwrap();
                        //add new states and update old states
                        //Might have "move" problems, to be solved later
                        let mut newState = newStateWitness.States;
                        for newStateKey in newState.keys(){
                            if stateWitness.States.contains_key(newStateKey) {
                                stateWitness.States.remove(newStateKey);
                                stateWitness.States.insert(*newStateKey,*newState.get(newStateKey));
                            }
                                //only add relevant states to local states
                            else if stateWitness.States.get(newStateKey).unwrap().1 == self.local_address{
                                stateWitness.States.insert(*newStateKey,*newState.get(newStateKey));
                            }
                        }
                        let mut newAccumulator  = newStateWitness.Accumulator;
                        for Block_Hash in newAccumulator.keys(){
                            if !stateWitness.Accumulator.contains_key(Block_Hash){
                                stateWitness.Accumulator.insert(*Block_Hash,*newAccumulator.get(Block_Hash));
                            }
                        }
                    }
                    std::mem::drop(stateWitness);
                }

            }

            //let interval = time::Duration::from_micros(100000 as u64);
            //thread::sleep(interval);
        }
    }
}
