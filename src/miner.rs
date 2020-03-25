use crate::network::message::{Message};
use crate::network::server::Handle as ServerHandle;
use std::sync::{Arc, Mutex};

use crate::crypto::hash::{H256, Hashable};
use crate::blockchain::Blockchain;
use crate::block::{Block,Header,Content};
use crate::crypto::merkle::{MerkleTree};
use crate::transaction::{Transaction, generate_random_transaction_, Mempool, State, SignedTransaction};
use rand::{thread_rng, Rng};
use ring::{digest};

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};

use std::thread;

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
    mempool: Arc<Mutex<Mempool>>,
    state: Arc<Mutex<State>>,
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
    blockchain: &Arc<Mutex<Blockchain>>,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        mempool: Arc::clone(mempool),
        state: Arc::clone(state),
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

            // TODO: actual mining
            //let mut blockchain = self.blockchain.lock().unwrap();

            let nonce:u32 = thread_rng().gen();
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();

            let mut bytes32 = [255u8;32];
            bytes32[0]=0;
            bytes32[1]=5;
            let difficulty : H256 = bytes32.into();
            let mut transaction = Vec::<SignedTransaction>::new();
            let mut mempool = self.mempool.lock().unwrap();
            let block_size_limit = 8;
            let mut trans_count = 0;
            //transaction.push(generate_random_transaction_());

            for hash in mempool.Transactions.keys(){
                if trans_count < block_size_limit{
                    transaction.push(mempool.Transactions.get(hash).unwrap().clone());
                    trans_count = trans_count + 1;
                }
            }




            let mut MerkleTree = MerkleTree::new(&transaction);

            let newContent = Content{
                content: transaction,
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





            if newBlock.hash() <= difficulty {
                //println!("miner tip: {:?}", self.blockchain.lock().unwrap().tip.0);
                self.blockchain.lock().unwrap().insert(&newBlock);
                miner_counter += 1;
                println!("Current miner counter: {:?}", miner_counter);
                println!("Current height of blockchain: {:?}", self.blockchain.lock().unwrap().tip.1);

                //println!("Current tip: {:?}", blockchain.tip() );

                self.server.broadcast(Message::NewBlockHashes(self.blockchain.lock().unwrap().all_blocks_in_longest_chain()));

            }

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}
