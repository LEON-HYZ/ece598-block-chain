#[macro_use]
use hex_literal;
use crate::crypto::hash::{H256, Hashable, H160};
use crate::transaction::{Transaction, SignedTransaction, StateWitness};
use std::collections::{HashMap, HashSet};
use rand::{thread_rng, Rng};
use modpow::modpow;
use num_bigint::BigInt;
use num_traits::One;
use std::sync::Mutex;
extern crate ramp;
use ramp::Int;
use rand::rngs::OsRng;
use self::ramp::RandomInt;
use rug::{Assign, Integer};
use rug::rand::RandState;
use chrono::format::Fixed::Internal;

fn little_fermat(candidate: &Integer) -> bool {

	let mut rand = RandState::new();
	let i = Integer::from(candidate);
	let result = i.random_below(&mut rand);
	result == Integer::from(1)
}

fn is_prime_naive(numb: &Integer) -> bool {
	let mut i = Integer::from(3);
	while &i < numb {
		if numb % &i == 0 {
			return false
		}
		i = i + 2;
	}
	return true;
}


fn is_prime(candidate: &Integer) -> bool {
	if *candidate == Integer::from(1) {
		return false;
	}
	if !little_fermat(candidate) {
		return false;
	}

	if !is_prime_naive(candidate) {
		return false;
	}
	true
}


fn genprime(n: u32) -> Integer {

	let mut rand = RandState::new();
	loop {
		let mut candidate = Integer::from(Integer::random_bits(n, &mut rand));
		candidate.set_bit(0, true);
		candidate.set_bit((n-1) as u32, true);
		if is_prime(&candidate) == true {
			return candidate;
		}
	}
}


pub fn parameters() -> (Integer, Integer, Integer) {
	let mut rng = thread_rng();
	let p = genprime(10);
	let q = genprime(10);
	//let g = genprime(j, 10u32.pow(j-1), 10u32.pow(j)-1);
	let g = genprime(1);
	return (p,q,g)
}

pub struct Accumulator {
	pub accumulator: HashMap<(H256,u32),(f32,H160,Integer)>,// prev TX Hash, prev Output Index <-> Output Value, Recp Addr, Prime
	pub prime_set : HashSet<Integer>,
	pub n: Integer,
	pub g: Integer,
	//pub product: Integer,
}

impl Accumulator {

	pub fn new() -> Self {
		let accumulator = HashMap::<(H256,u32),(f32,H160,BigInt)>::new(); //TX Hash, Output Index, Output Value, Recp Addr, Prime
		let mut prime_set = HashSet::<BigInt>::new();
		let (p, q, g) = parameters();
	    let mut _n = Integer::from(&p * &q);
	    let mut _g = g;
		return Accumulator{accumulator: accumulator, prime_set: prime_set,  n: _n, g: _g,}
	}

	pub fn hash_to_prime(&mut self, tx_hash: H256, output_index: u32,output_value:f32, recp_addr: H160 ){
		let mut rng = thread_rng();
		let prime = genprime(2);
		if self.prime_set.contains(&prime){
			self.hash_to_prime(tx_hash, output_index, output_value,recp_addr);
		}else{
			self.prime_set.insert(prime.clone());
	    	self.accumulator.insert((tx_hash, output_index),(output_value,recp_addr, prime.clone()));
		}

	}

	pub fn accumulate(&mut self) -> Integer {

		let mut x = Integer::from(1);
	    for (_, val) in self.accumulator.iter() {
	        x = Integer::from(&x * &val.2);
	    }
		self.product = x;
		println!( "g is {:?}, x is {:?}",self.g, x);
		let exp:usize = Integer::from(&'a x);
	    let a = (self.g).pow(exp);
	    //let a = a%(self.n);
	    println!("A is {:?}",a );
		return a
	}
/*
	pub fn update(&mut self, SignedTransactions: &Vec<SignedTransaction>){
	    for signedTransaction in SignedTransactions {
	        let mut hash = signedTransaction.transaction.hash();
	        for input in signedTransaction.transaction.Input.clone() {
	            if self.accumulator.contains_key(&(input.prevTransaction, input.preOutputIndex)){
	            	let p = self.accumulator.get(&(input.prevTransaction, input.preOutputIndex)).unwrap();
	                self.accumulator.remove(&(input.prevTransaction, input.preOutputIndex));
	                self.prime_set.remove(&p.2);
	            }
	        }
	        for output in signedTransaction.transaction.Output.clone() {
	            self.hash_to_prime(hash,output.index, output.value,output.recpAddress);
	        }
        }
		let (p_new, q_new, g_new) = parameters();
		self.n = p_new * q_new;
		self.g = g_new;
    }*/
}

