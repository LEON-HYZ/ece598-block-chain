use crate::network::message::{Message};
use crate::network::server::Handle as ServerHandle;
use std::sync::{Arc, Mutex};

use crate::crypto::hash::{H256, Hashable, H160};
use crate::blockchain::Blockchain;
use crate::block::{Block,Header,Content};
use crate::crypto::merkle::{MerkleTree};
use crate::transaction::{Transaction, generate_random_transaction_, Mempool, State, StateSet, SignedTransaction};
use rand::{thread_rng, Rng};
use ring::{digest};

use log::{info,debug};

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::{time, fs};
use std::time::{SystemTime, UNIX_EPOCH};

use std::thread;
use ring::signature::Ed25519KeyPair;

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    local_address: H160,
    local_public_key: Vec<u8>,
    mempool: Arc<Mutex<Mempool>>,
    state: Arc<Mutex<State>>,
    stateSet: Arc<Mutex<StateSet>>,
    blockchain: Arc<Mutex<Blockchain>>,
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    server: &ServerHandle,
    mempool: &Arc<Mutex<Mempool>>,
    state: &Arc<Mutex<State>>,
    stateSet: &Arc<Mutex<StateSet>>,
    blockchain: &Arc<Mutex<Blockchain>>,
    local_public_key: &[u8],
    local_address: &H160,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        local_address: *local_address,
        local_public_key: (*local_public_key).to_owned(),
        mempool: Arc::clone(mempool),
        state: Arc::clone(state),
        stateSet: Arc::clone(stateSet),
        blockchain: Arc::clone(blockchain),
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!("Miner starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }

    fn miner_loop(&mut self) {
        let mut miner_counter:i32 = 0;
        let mut readICO = false;
        // main mining loop
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    self.handle_control_signal(signal);
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        self.handle_control_signal(signal);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            //Read ICO & Update initial state
            if !readICO {
                // Initialize State
                //println!("local: {:?}", self.local_address);
                let mut state = self.state.lock().unwrap();
                println!("ICO: THE ICO IS WORKING ON PROCESSES: {:?}",self.local_address);
                let data = fs::read("ICO.txt").expect("Unable to read file");
                let data_len: usize = (data.len() / 20) as usize;
                println!("data_length: {:?}", data.len());
                for i in 0..data_len {
                    let mut start = i * 20;
                    let mut end = (i + 1) * 20;
                    let mut addr_u8: [u8; 20] = [0; 20];
                    addr_u8.clone_from_slice(&data[start..end]);
                    let mut address: H160 = <H160>::from(addr_u8);
                    //println!("all: {:?}", address);
                    state.Outputs.insert((<H256>::from(digest::digest(&digest::SHA256, &[0x00 as u8])), i as u32), (100.0 as f32, address));
                }
                readICO = true;
                println!("LOCAL STATES: {:?}", state.Outputs);
                println!("PROCESS {:?} CAN START TO MINE BLOCKS.",self.local_address);
                std::mem::drop(state);
            }
            // TODO: actual mining

            if self.mempool.lock().unwrap().Transactions.keys().len() > 0 {
                //info!("MINER: STARTING...");
                let nonce:u32 = thread_rng().gen();
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();

                // difficulty
                let mut bytes32 = [255u8;32];
                bytes32[0]=10;
                bytes32[1]=20;
                let difficulty : H256 = bytes32.into();

                // read transactions from mempool
                let mut signedTransaction = Vec::<SignedTransaction>::new();
                let block_size_limit = 5;
                let mut tx_counter = 0;

                let mut state = self.state.lock().unwrap();
                let mut mempool = self.mempool.lock().unwrap();

                let mut key_iter= mempool.Transactions.keys();
                for key in mempool.Transactions.keys(){
                    //println!("MINER: MEMPOOL KEYS:{:?}, INPUT: {:?}, OUTPUT: {:?}", key, mempool.Transactions.get(key).unwrap().transaction.Input, mempool.Transactions.get(key).unwrap().transaction.Output);
                }
                while tx_counter < block_size_limit {
                    match key_iter.next() {
                        Some(hash) => {
                            //println!("Miner: tx: {:?}",hash);
                            //println!("Miner: preTx: {:?}, PreIndex: {:?}",mempool.getPreTxHash(hash), mempool.getPreIndex(hash));
                            //double spent check and verify signature
                            if state.ifNotDoubleSpent(mempool.Transactions.get(hash).unwrap())
                                && mempool.Transactions.get(hash).unwrap().verifySignedTransaction() {
                                //info!("Miner: Adding to block HERE");
                                signedTransaction.push(mempool.Transactions.get(hash).unwrap().clone());
                                tx_counter = tx_counter + 1;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
                std::mem::drop(mempool);
                std::mem::drop(state);

                if signedTransaction.capacity() > 0 {
                    //info!("MINER: ADDING...");

                    //info!("MINER: MERKLETREE CHECKING...");
                    let mut MerkleTree = MerkleTree::new(&signedTransaction);
                    //info!("MINER: MERKLETREE CHECKED");
                    let newContent = Content{
                        content: signedTransaction,
                    };

                    let newHeader = Header{
                        parent: self.blockchain.lock().unwrap().tip(),
                        nonce:  nonce,
                        difficulty: difficulty,
                        timestamp:  timestamp,
                        merkleRoot: MerkleTree.root(),
                    };

                    let newBlock = Block{
                        Header: newHeader,
                        Content: newContent,
                    };
                    //println!("1: {:?}", newBlock.hash() );
                    //println!("2: {:?}", difficulty );
                    //info!("MINER: BLOCK CREATED");

                    if newBlock.hash() <= difficulty {

                        let mut contents = newBlock.Content.content.clone();
                        let mut state = self.state.lock().unwrap();
                        let mut mempool = self.mempool.lock().unwrap();
                        let mut stateSet = self.stateSet.lock().unwrap();
                        let mut check = true;
                        for content in contents.iter(){
                            if state.ifNotDoubleSpent(content) && content.verifySignedTransaction() {
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

                            let tip_hash = self.blockchain.lock().unwrap().insert(&newBlock);


                            //info!("MINER: NEW BLOCK ADDED");
                            miner_counter += 1;
                            println!("MINER: CURRENT MINER COUNT: {:?}", miner_counter);
                            println!("MINER: CURRENT BLOCKCHAIN HEIGHT: {:?}", self.blockchain.lock().unwrap().tip.1);

                            let mut state = self.state.lock().unwrap();
                            let mut mempool = self.mempool.lock().unwrap();
                            if stateSet.Set.contains_key(&tip_hash) {
                                // let new_state = stateSet.Set.get(&tip_hash).unwrap().Outputs;
                                state.Outputs.clear();
                                for (key, value) in stateSet.Set.get(&tip_hash).unwrap().Outputs.clone() {
                                    state.Outputs.insert(key, value);
                                }
                            }
                            //Update Mempool

                            //println!("MINER: UPDATED MEMPOOL: {:?}", mempool.Transactions.keys());
                            //Update State
                            state.updateState(&contents);
                            stateSet.Set.insert(newBlock.hash(), state.clone());
                            mempool.updateMempool(&contents);
                            for key in state.Outputs.keys() {
                                println!("MINER: RECP: {:?}, VALUE {:?}", state.Outputs.get(key).unwrap().1, state.Outputs.get(key).unwrap().0);
                            }
                            self.server.broadcast(Message::NewBlockHashes(self.blockchain.lock().unwrap().all_blocks_in_longest_chain()));
                            //info!("MINER: BLOCK MESSAGES SENT");
                            std::mem::drop(state);
                            std::mem::drop(mempool);
                        }


                    }
                }
            }

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
            let interval = time::Duration::from_micros(1000 as u64);
            thread::sleep(interval);
        }
    }
}
