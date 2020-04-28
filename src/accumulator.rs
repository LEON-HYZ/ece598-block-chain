#[macro_use]
extern crate hex_literal;
use crate::crypto::hash::{H256,Hashable};
use crate::transaction::{Transaction, SignedTransaction, StateWitness};
use std::collections::{HashMap, HashSet};
use rand::{thread_rng, Rng};
use modpow::modpow;
use num_bigint::BigInt;
use num_traits::One;
use std::sync::Mutex;


fn little_fermat(candidate: &u32) -> bool {
	let mut rng = thread_rng();
	let random:u32 = rng.gen_range(0, candidate); 
	let result = modpow(&random, &(candidate - 1), candidate);
	let mut f1: BigInt = One::one();
	result == f1
}

fn is_prime_naive(numb: &u32) -> bool {
	let mut i = 3u32;
	while &i < numb {
		if numb % &i == 0 {
			return false
		}
		i = i + 2;
	}
	return true;
}

fn is_prime(candidate: &u32) -> bool { 
	if !little_fermat(candidate) {
		return false;
	}

	if !is_prime_naive(candidate) {
		return false;
	}
	true
}

pub fn genprime(j: u32, low: u32, high:u32) -> u32 {
	let mut rng = thread_rng();
	loop {
		let mut candidate:u32 = rng.gen_range(low, high); 
		candidate |= 1 << 0;
		candidate |= 1 << j-1;
		if is_prime(&candidate) == true { 
			return candidate;
		}
	}
}


pub fn parameters() -> (u32, u32, u32) {
	let mut rng = thread_rng();
	let mut j:u32 = rng.gen_range(3, 7);
	let p = genprime(j, 10u32.pow(j-1), 10u32.pow(j)-1);
	let q = genprime(j, 10u32.pow(j-1), 10u32.pow(j)-1);
	let g = genprime(j, 10u32.pow(j-1), 10u32.pow(j)-1);
	return (p,q,g)
}

pub struct Accumulator {
	pub accumulator: HashMap<(H256,u32),(u32)>,// prev TX Hash, prev Output Index <-> prime number
	pub prime_set : HashSet<u32>,
	pub n: u32,
	pub g: u32,
}

impl Accumulator {

	pub fn new() -> Self {
		let accumulator = HashMap::<(H256,u32),(u32)>::new();
		let mut prime_set = HashSet::<u32>::new();
		let (p, q, g) = parameters();
	    let mut _n = p*q;
	    let mut _g = g;
		return Accumulator{accumulator: accumulator, prime_set: prime_set,  n: _n, g: _g,}
	}

	pub fn hash_to_prime(&mut self, tx_hash: H256, output_index: u32){
		let mut rng = thread_rng();
		let mut j:u32 = rng.gen_range(3, 7);
		let prime = genprime(j, 10u32.pow(j-1), 10u32.pow(j)-1);
		if self.prime_set.contains(&prime){
			self.hash_to_prime(tx_hash, output_index);
		}else{
			self.prime_set.insert(prime);
	    	self.accumulator.insert((tx_hash, output_index),prime);
		}

	}

	pub fn accumulate(&self) -> u32 {
		let mut x = 1u32;
	    for (_, val) in self.hash_to_prime.iter() {
	        x = x*val;
	    }
	    let a = (self.g).overflowing_pow(x).0 ;
	    let a = a%(self.n);
	    println!("a is {:?}",a );
		return a
	}

	pub fn update(&mut self, SignedTransactions: &Vec<SignedTransaction>){
	    for signedTransaction in SignedTransactions {
	        let mut hash = signedTransaction.transaction.hash();
	        for input in signedTransaction.transaction.Input.clone() {
	            if self.hash_to_prime.contains_key(&(input.prevTransaction, input.preOutputIndex)){
	            	let p = self.hash_to_prime.get(&(input.prevTransaction, input.preOutputIndex)).unwrap();
	                self.hash_to_prime.remove(&(input.prevTransaction, input.preOutputIndex));
	                self.prime_vec.remove(p);
	            }
	        }
	        for output in signedTransaction.transaction.Output.clone() {
	            self.hash_to_prime(hash,output.index);
	        }
        }
		let (p_new, q_new, g_new) = parameters();
		self.n = p_new * q_new;
		self.g = g_new;
    }
}

