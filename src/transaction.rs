use serde::{Serialize,Deserialize};
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use crate::crypto::hash::{H256, Hashable, H160, Hashable_160};
use crate::network::message::{Message};
use crate::network::server::Handle as ServerHandle;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use ring::{digest};

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};

use std::thread;
use crate::crypto::key_pair;
use std::path::Prefix::Verbatim;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::fs;
//use std::intrinsics::prefetch_read_data;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct input {
    pub prevTransaction: H256,
    pub preOutputIndex: u32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct output {
    pub recpAddress: H160,
    pub value: f32,
    pub index: u32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    pub Input: Vec<input>,
    pub Output: Vec<output>,

}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        let t_serialized = bincode::serialize(&self).unwrap();
        return ring::digest::digest(&ring::digest::SHA256, &t_serialized).into();
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Vec<u8>,
    pub publicKey: Vec<u8>,
}

impl Hashable for SignedTransaction {
    fn hash(&self) -> H256 {
        return self.transaction.hash()
    }
}

impl SignedTransaction {
    pub fn new(t: &Transaction, signature: &Signature, public_key: &<Ed25519KeyPair as KeyPair>::PublicKey) -> Self{
        let mut transaction = t.clone();
        let mut signature = signature.as_ref().to_vec();
        let mut publicKey = public_key.as_ref().to_vec();
        return SignedTransaction{transaction: transaction, signature:signature, publicKey:publicKey}
    }

    pub fn verifySignedTransaction(&self) -> bool {
        //info!("checking signature...");
        let public_key = self.publicKey.clone();
        let signature= self.signature.clone();
        //println!("pubkey: {:?}, sig: {:?} " ,public_key, signature);
        let public_key_ = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key[..].as_ref());
        return public_key_.verify(&bincode::serialize(&self.transaction).unwrap()[..],signature[..].as_ref()) == Ok(());
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    //unimplemented!()
    let t_serialized = bincode::serialize(t).unwrap();
    let t_signature = key.sign(&t_serialized);
    t_signature
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &<Ed25519KeyPair as KeyPair>::PublicKey, signature: &Signature) -> bool {
    let t_serialized = bincode::serialize(&t).unwrap();
    let public_key_ = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key.as_ref());
    if public_key_.verify(&t_serialized,signature.as_ref()) == Ok(())  {   return true;    }
    else {   return false;   }

}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct StateSet {
    pub Set: HashMap<H256,State>, // hash <-> state
}

impl StateSet {
    pub fn new() -> Self{
        let hashmap:HashMap<H256,State> = HashMap::new();
        return StateSet{Set:hashmap,}
    }
}


//transaction Handle Begins
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Mempool {
    pub Transactions: HashMap<H256,SignedTransaction>, // hash <-> transaction(signed)
}

impl Mempool {
    pub fn new() -> Self{
        let hashmap:HashMap<H256,SignedTransaction> = HashMap::new();
        return Mempool{Transactions:hashmap,}
    }

    pub fn insert(&mut self, tx: &SignedTransaction) {
        let last_tx = tx.clone();
        self.Transactions.insert(tx.hash(), last_tx);
    }

    pub fn updateMempool(&mut self, SignedTransaction: &Vec<SignedTransaction>){
        for signedTransaction in SignedTransaction{
            if self.Transactions.contains_key(&signedTransaction.hash()){
                self.Transactions.remove(&signedTransaction.hash());
            }
        }

    }
}
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
    stateSet: Arc<Mutex<StateSet>>,
    key_pair: Ed25519KeyPair,
    local_address: H160,
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
    key_pair: Ed25519KeyPair,
    local_address: &H160,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        mempool: Arc::clone(mempool),
        state: Arc::clone(state),
        stateSet: Arc::clone(stateSet),
        key_pair: key_pair,
        local_address: *local_address,
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
            .name("transaction".to_string())
            .spawn(move || {
                self.transaction_loop();
            })
            .unwrap();
        info!("Transaction generator initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Transaction generator shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!("Transaction generator starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }
    //transaction generator
    fn transaction_loop(&mut self) {
        let mut tx_counter: i32 = 0;
        let mut readADD: bool = false;
        let mut other_address = Vec::<H160>::new();
        let mut all_address = Vec::<H160>::new();
        let mut hashset = HashSet::<(H256,u32)>::new();
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
                    Err(TryRecvError::Disconnected) => panic!("Transaction generator control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }



            //Read Address just once
            if !readADD {
                // Initialize State
                //println!("local: {:?}", self.local_address);
                //let mut state = self.state.lock().unwrap();
                info!("The ICO is working on local process");
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
                    //state.Outputs.insert((<H256>::from(digest::digest(&digest::SHA256, &[0x00 as u8])), i as u32), (100.0 as f32, address));
                    all_address.push(address);

                    //Record addresses
                    if !(address == self.local_address) {
                        other_address.push(address);
                    }
                }

                readADD = true;
                println!("LOCAL ADDR: {:?},", self.local_address);
                //println!("LOCAL ADDR: {:?}, LOCAL STATES:{:?}", self.local_address, state.Outputs);
                //std::mem::drop(state);
            }


            //check if valid in state
            let mut state = self.state.lock().unwrap();
            let mut myStateKeys = Vec::<(H256, u32)>::new();
            let mut all_value = 0 as f32;
            if state.Outputs.keys().len() > 0 {
                for State in state.Outputs.keys() {
                    //state check to avoid double spent
                    //State: hash, output index <-> value, recipient address
                    if state.Outputs.get(State).unwrap().1 == self.local_address {
                        myStateKeys.push(State.clone());
                        all_value = all_value + state.Outputs.get(&State).unwrap().0; //balance
                    }
                }
            }
            std::mem::drop(state);

            if myStateKeys.capacity() > 0 {
                //input
                let mut pre_hash = Vec::<H256>::new();
                let mut pre_index = Vec::<u32>::new();
                //output
                let mut out_value = Vec::<f32>::new();
                let mut recp_addr = Vec::<H160>::new();


                for Key in myStateKeys.iter() {
                    pre_hash.push(Key.0);
                    pre_index.push(Key.1);
                }
                //recipient value
                let mut dest_value:f32 = 0.0;
                if all_value > 10.0 {
                    let mut rng = rand::thread_rng();
                    let dest_ = rng.gen_range(1,10);
                    dest_value = dest_ as f32;
                }
                else if all_value <= 10.0 && all_value >= 2.0 {
                    let mut rng = rand::thread_rng();
                    let dest_= rng.gen_range(1,all_value as usize);
                    dest_value = dest_ as f32;
                }
                else{
                    dest_value = 1.0;
                }

                let rest_value = all_value - (dest_value as f32);

                //recipient adresses
                let mut rng = rand::thread_rng();
                let mut num = rng.gen_range(0, other_address.len());
                let dest_addr: H160 = other_address[num];
                recp_addr.push(dest_addr);
                if rest_value >= 0.0 {
                    out_value.push(dest_value);
                    if rest_value > 0.0 {
                        out_value.push(rest_value);
                        recp_addr.push(self.local_address);
                    }

                    //generating signed transactions
                    let mut transaction = generate_transaction(&pre_hash, &pre_index, &out_value, &recp_addr);
                    let signature = sign(&transaction, &self.key_pair);
                    let SignedTransaction = SignedTransaction::new(&transaction, &signature, &self.key_pair.public_key());

                    //need to check signature before inserting to mempool

                    let mut mempool = self.mempool.lock().unwrap();
                    let mut state = self.state.lock().unwrap();
                    let mut valid = true;
                    for input in transaction.Input.clone() {
                        if hashset.contains(&(input.prevTransaction,input.preOutputIndex)){
                            valid = false;
                        }
                    }


                    if (!mempool.Transactions.contains_key(&SignedTransaction.hash()))
                        && SignedTransaction.verifySignedTransaction()
                        && state.ifNotDoubleSpent(&SignedTransaction)
                        && valid{
                        mempool.insert(&SignedTransaction);
                        for input in transaction.Input.clone() {
                            hashset.insert((input.prevTransaction,input.preOutputIndex));
                        }
                        tx_counter = tx_counter + 1;
                        //println!("{:?}",tx_counter);
                        //info!("There is a transaction generator that can put transactions into these clients.");
                        let mut txHash = Vec::<H256>::new();
                        for key in mempool.Transactions.keys(){
                            txHash.push(key.clone());
                            println!("TXG: MEMPOOL KEYS:{:?}, INPUT: {:?}, OUTPUT: {:?}", key, mempool.Transactions.get(key).unwrap().transaction.Input, mempool.Transactions.get(key).unwrap().transaction.Output);
                        }
                        //txHash.push(SignedTransaction.hash().clone());
                        self.server.broadcast(Message::NewTransactionHashes(txHash));
                        println!("TXG: {:?} PAID {:?} {:?} BTC", self.local_address, dest_addr,dest_value);
                    }
                    std::mem::drop(mempool);
                    std::mem::drop(state);
                }

            }


            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
            //let interval = time::Duration::from_micros(100000 as u64);
            //thread::sleep(interval);
        }
    }
}


//generally generate a transaction without signature.
pub fn generate_transaction(preHash:&Vec<H256>, preIndex:&Vec<u32>, outValue:&Vec<f32>, recpAddress:&Vec<H160>) -> Transaction {

    let mut inputVec = Vec::<input>::new();
    let mut outputVec = Vec::<output>::new();

    for in_ in 0..preHash.len(){
        let input = input{
            prevTransaction : preHash[in_],
            preOutputIndex: preIndex[in_],
        };
        inputVec.push(input);
    }
    let mut out_count = 0;
    for out_ in 0..outValue.len() {
        let output = output{
            recpAddress : recpAddress[out_],
            value : outValue[out_],
            index: out_count,
        };
        out_count = out_count + 1;
        outputVec.push(output);
    }

    return Transaction{Input : inputVec,Output : outputVec,};
}



//transaction Handle Ends

//state Begins
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct State {
    pub Outputs: HashMap<(H256, u32),(f32, H160)>, // hash, output index <-> value, recipient address
}

impl State {
    pub fn new() -> Self{
        let hashmap:HashMap<(H256, u32),(f32, H160)> = HashMap::new();
        return State{Outputs:hashmap,}
    }

    // double spent check
    pub fn ifNotDoubleSpent(&self, SignedTransaction:&SignedTransaction) -> bool {
        //info!("checking double spend");
        let mut isnotdoublespent = true;
        for input in SignedTransaction.transaction.Input.clone() {
            if self.Outputs.contains_key(&(input.prevTransaction, input.preOutputIndex)){
                isnotdoublespent = isnotdoublespent && true;
            }
            else{
                isnotdoublespent = isnotdoublespent && false;
                break;
            }
        }
        return isnotdoublespent;
    }

    pub fn updateState(&mut self, SignedTransactions: &Vec<SignedTransaction>) {
        for signedTransaction in SignedTransactions {
            let mut hash = signedTransaction.hash();
            for input in signedTransaction.transaction.Input.clone() {
                if self.Outputs.contains_key(&(input.prevTransaction, input.preOutputIndex)) {
                    self.Outputs.remove(&(input.prevTransaction, input.preOutputIndex));
                }
            }
            for output in signedTransaction.transaction.Output.clone() {
                self.Outputs.insert((hash, output.index), (output.value, output.recpAddress));
            }
        }

    }

    // pub fn backtrack(&mut self) {
    //     self.
    // }

}
//state Ends





pub fn generate_random_signed_transaction_() -> SignedTransaction {

    let new_hash = <H256>::from(digest::digest(&digest::SHA256, &[0x00 as u8]));
    let mut new_hash_vec = Vec::<H256>::new();
    new_hash_vec.push(new_hash);
    let mut rand_addr_H160 = <H160>::from(digest::digest(&digest::SHA256,"442cabd17e40d95ac0932d977c0759397b9db4d93c4d62c368b95419db574db0".as_bytes()));
    let mut rand_addr = Vec::<H160>::new();
    rand_addr.push(rand_addr_H160);
    let mut rand_u32:u32 = rand::thread_rng().gen();
    let mut rand_u32_vec = [rand_u32].to_vec();
    let mut rand_f32:f32 = rand::thread_rng().gen();
    let mut rand_f32_vec = [rand_f32].to_vec();

    let mut transaction = generate_transaction(&new_hash_vec,&rand_u32_vec,&rand_f32_vec,&rand_addr);
    let key = key_pair::random();
    let signature = sign(&transaction,&key);
    let SignedTransaction = SignedTransaction::new(&transaction,&signature,&key.public_key());

    return SignedTransaction;
}

pub fn generate_random_transaction_() -> Transaction {

    let new_hash = <H256>::from(digest::digest(&digest::SHA256, &[0x00 as u8]));
    let rand_addr = <H160>::from(digest::digest(&digest::SHA256,"442cabd17e40d95ac0932d977c0759397b9db4d93c4d62c368b95419db574db0".as_bytes()));
    let rand_u32:u32 = rand::thread_rng().gen();
    let rand_f32:f32 = rand::thread_rng().gen();

    let input = input{
        prevTransaction : new_hash,
        preOutputIndex: rand_u32,
    };

    let output = output{
        recpAddress : rand_addr,
        value : rand_f32,
        index: rand_u32,
    };

    let mut inputVec = Vec::<input>::new();
    let mut outputVec = Vec::<output>::new();

    inputVec.push(input);
    outputVec.push(output);

    let mut Transaction = Transaction{
        Input : inputVec,
        Output : outputVec,
    };

    return Transaction;
}

#[cfg(any(test, test_utilities))]
pub mod tests {
    use super::*;
    use crate::crypto::key_pair;

    pub fn generate_random_transaction() -> Transaction {
        return generate_random_transaction_()
    }

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, &(key.public_key()), &signature));
    }

    #[test]
    fn assignment2_transaction_1() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, &(key.public_key()), &signature));
    }
    #[test]
    fn assignment2_transaction_2() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, &(key.public_key()), &signature));
        assert!(!verify(&t, &(key_2.public_key()), &signature));
    }
}
