pub mod crypto;
pub mod transaction;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate hex_literal;
use crate::crypto::hash::H256;
use crate::transaction::{Transaction,SignedTransaction};
use std::collections::HashMap;
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

pub struct Accumulator {
	pub hash_to_prime : HashMap<(H256, u32), u32>,
	pub n: u32,
	pub g: u32,
}

impl Accumulator {

	pub fn new() -> Self {
		let mut hashmap: HashMap<(H256, u32), u32> = HashMap::new();
		let mut rng = thread_rng();
		let mut j:u32 = rng.gen_range(3, 7);
	    let p = genprime(j, 10u32.pow(j-1), 10u32.pow(j)-1);
	    let q = genprime(j, 10u32.pow(j-1), 10u32.pow(j)-1);
	    let n = p*q;
	    let g = genprime(j, 10u32.pow(j-1), 10u32.pow(j)-1);
		return Accumulator{hash_to_prime: hashmap, n: n, g: g}
	}

	

	pub fn hash_to_prime(&mut self, tx_hash: H256, output_index: u32) {
		let mut rng = thread_rng();
		let mut j:u32 = rng.gen_range(3, 7);
		let prime = genprime(j, 10u32.pow(j-1), 10u32.pow(j)-1);
	    self.hash_to_prime.insert((tx_hash, output_index),prime);
	}

	pub fn accumulate(&self) {
		let mut x = 1u32;
	    for (_, val) in self.hash_to_prime.iter() {
	        x = x*val;
	    }
	    let a = (self.g).overflowing_pow(x).0 ;
	    println!("a is {:?}", a);
	    println!("n is {:?}", self.n);
	    let a = a%(self.n);
	    println!("a is {:?}",a );
	}

	pub fn update(&mut self, SignedTransactions: &Vec<SignedTransaction>){

	    for signedTransaction in SignedTransactions {
	        let mut hash = signedTransaction.hash();
	        for input in signedTransaction.transaction.Input.clone() {
	            if self.hash_to_prime.contains_key(&(input.prevTransaction, input.preOutputIndex)){
	                self.hash_to_prime.remove(&(input.prevTransaction, input.preOutputIndex));
	            }
	        }
	        for output in signedTransaction.transaction.Output.clone() {

	            self.hash_to_prime(&self,hash,output.index);
	        }

        }
    }
}

fn main(){
	let tx = (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into();
    let mut acc = Accumulator::new();
    acc.hash_to_prime(tx, 0);
    acc.accumulate();
}