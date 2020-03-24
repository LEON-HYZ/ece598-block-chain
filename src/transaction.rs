use serde::{Serialize,Deserialize};
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use crate::crypto::hash::{H256, Hashable, H160, Hashable_160};
use crate::network::message::{Message};
use crate::network::server::Handle as ServerHandle;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use ring::{digest};

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};

use std::thread;
use crate::crypto::key_pair;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct input {
    pub prevTransaction: H256,
    pub index: u32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct output {
    pub recpAddress: H160,
    pub value: u32,
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
        let mut signature = signature.as_ref().iter().cloned().collect();
        let mut publicKey = public_key.as_ref().iter().cloned().collect();
        return SignedTransaction{transaction: transaction, signature:signature, publicKey:publicKey}
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
    //unimplemented!()
    let t_serialized = bincode::serialize(&t).unwrap();
    let public_key_ = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key.as_ref());
    if public_key_.verify(&t_serialized,signature.as_ref()) == Ok(())  {   return true;    }
    else {   return false;   }

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
    Mempool: Arc<Mutex<Mempool>>,
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
    Mempool: &Arc<Mutex<Mempool>>,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        Mempool: Arc::clone(Mempool),
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
            .name("tran".to_string())
            .spawn(move || {
                self.transaction_loop();
            })
            .unwrap();
        info!("Transaction initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Transaction shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!("Transaction starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }
    //transaction generator
    fn transaction_loop(&mut self) {
        let mut transaction_counter:i32 = 0;
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
                    Err(TryRecvError::Disconnected) => panic!("Transaction control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            let test = generate_random_signed_transaction_();
            
            self.Mempool.lock().unwrap().Transactions.insert(test.hash(), test);

            let mut keyhashes = Vec::<H256>::new();

            for k in self.Mempool.lock().unwrap().Transactions.keys(){
                keyhashes.push(k.clone());
            }
            if keyhashes.capacity() > 0{
                self.server.broadcast(Message::NewTransactionHashes(keyhashes));
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





//transaction Handle Ends




pub fn generate_random_signed_transaction_() -> SignedTransaction {
        
        let new_hash = <H256>::from(digest::digest(&digest::SHA256, &[0x00 as u8]));
        let rand_addr = <H160>::from(digest::digest(&digest::SHA256,"442cabd17e40d95ac0932d977c0759397b9db4d93c4d62c368b95419db574db0".as_bytes()));
        let rand_u32:u32 = rand::thread_rng().gen();

        let input = input{
            prevTransaction : new_hash,
            index : rand_u32,
        };

        let output = output{
            recpAddress : rand_addr,
            value : rand_u32,
        };

        let mut inputVec = Vec::<input>::new();
        let mut outputVec = Vec::<output>::new();

        inputVec.push(input);
        outputVec.push(output);

        let mut transaction = Transaction{
            Input : inputVec,
            Output : outputVec,
        };

        let key = key_pair::random();
        let signature = sign(&transaction,&key);
        let SignedTransaction = SignedTransaction::new(&transaction,&signature,&key.public_key());

        return SignedTransaction;
    }

    pub fn generate_random_transaction_() -> Transaction {

        let new_hash = <H256>::from(digest::digest(&digest::SHA256, &[0x00 as u8]));
        let rand_addr = <H160>::from(digest::digest(&digest::SHA256,"442cabd17e40d95ac0932d977c0759397b9db4d93c4d62c368b95419db574db0".as_bytes()));
        let rand_u32:u32 = rand::thread_rng().gen();

        let input = input{
            prevTransaction : new_hash,
            index : rand_u32,
        };

        let output = output{
            recpAddress : rand_addr,
            value : rand_u32,
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
        let new_hash = <H256>::from(digest::digest(&digest::SHA256, &[0x00 as u8]));
        let rand_addr = <H160>::from(digest::digest(&digest::SHA256,"442cabd17e40d95ac0932d977c0759397b9db4d93c4d62c368b95419db574db0".as_bytes()));
        let rand_u32:u32 = rand::thread_rng().gen();

        let input = input{
            prevTransaction : new_hash,
            index : rand_u32,
        };

        let output = output{
            recpAddress : rand_addr,
            value : rand_u32,
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
