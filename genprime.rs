use crate::crypto::hash::{Hashable, H256};
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use modpow::modpow;
use num_bigint::BigInt;
use num_traits::One;

//citation: https://medium.com/snips-ai/prime-number-generation-2a02f28508ff
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

     // Second, Fermat's little theo test on the candidate
    if !little_fermat(candidate) {
        return false;
    }

    // Finally, Miller-Rabin test
    if !is_prime_naive(candidate) {
        return false;
    }
    true
}

fn genprime(n: u32, low: u32, high:u32) -> u32 {
    // use self::ramp::RandomInt;
    // let mut rng = OsRng::new().ok().expect("Failed to get OS random generator");
    let mut rng = thread_rng();
    loop {
        let mut candidate:u32 = rng.gen_range(low, high); 
        // candidate.set_bit(0, true);
        // candidate.set_bit((n-1) as u32, true);
        candidate |= 1 << 0;
        candidate |= 1 << n-1;
        if is_prime(&candidate) == true { 
            return candidate;
        }
    }
}

fn hash_to_prime(tx_hash: H256, output_index: u32){
	let mut hash_to_prime: HashMap<(H256, u32), u32> = HashMap::new();
	let n = 4u32;
	let prime = genprime(n, 10u32.pow(n-1), 10u32.pow(n)-1);
    hash_to_prime.insert((tx_hash, output_index),prime)
}

fn main(){
	let mut i = 0u32;
	while i < 10 {
		for j in 3..6{
			let low = 10u32.pow(j-1);
            let high = 10u32.pow(j)-1;
			let n = genprime(j,low, high);
	    	println!("{:?}", n);
	    	
		}
		i = i+1;
	}
    
}