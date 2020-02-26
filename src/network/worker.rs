use super::message::Message;
use super::peer;
use crate::network::server::Handle as ServerHandle;
use crossbeam::channel;
use log::{debug, warn};

use std::sync::{Arc, Mutex};
use crate::crypto::hash::{H256, Hashable};
use crate::blockchain::Blockchain;
use crate::block::{Block,Header,Content};
use crate::crypto::merkle::{MerkleTree};


use std::thread;

#[derive(Clone)]
pub struct Context {
    blockchain: Arc<Mutex<Blockchain>>,
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
}

pub fn new(
    blockchain: &Arc<Mutex<Blockchain>>,
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
) -> Context {
    Context {
        blockchain: Arc::clone(blockchain),
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
                    for block in blocks.iter() {
                        blockchain.insert(&block);
                    }
                    self.server.broadcast(Message::NewBlockHashes(blockchain.all_blocks_in_longest_chain()));
                    //println!("Current height of worker blockchain: {:?}", blockchain.tip.1);
                }

            }
        }
    }
}
