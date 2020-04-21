//#[cfg(test)]
#[macro_use]
extern crate hex_literal;

pub mod api;
pub mod block;
pub mod blockchain;
pub mod crypto;
pub mod miner;
pub mod network;
pub mod transaction;

use clap::clap_app;
use crossbeam::channel;
use log::{error, info};
use api::Server as ApiServer;
use network::{server, worker};
use std::net;
use std::process;
use std::thread;
use std::time;
use std::sync::{Arc, Mutex};
use crate::crypto::key_pair;
use ring::signature::KeyPair;
use crate::crypto::hash::{H256, H160};
use ring::digest;
use std::fs::OpenOptions;
use std::io::Write;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::fs;
//use std::intrinsics::prefetch_read_instruction;


fn main() {
    // parse command line arguments
    let matches = clap_app!(Bitcoin =>
     (version: "0.1")
     (about: "Bitcoin client")
     (@arg verbose: -v ... "Increases the verbosity of logging")
     (@arg peer_addr: --p2p [ADDR] default_value("127.0.0.1:6000") "Sets the IP address and the port of the P2P server")
     (@arg api_addr: --api [ADDR] default_value("127.0.0.1:7000") "Sets the IP address and the port of the API server")
     (@arg known_peer: -c --connect ... [PEER] "Sets the peers to connect to at start")
     (@arg p2p_workers: --("p2p-workers") [INT] default_value("4") "Sets the number of worker threads for P2P server")
    )
    .get_matches();

    // init logger
    let verbosity = matches.occurrences_of("verbose") as usize;
    stderrlog::new().verbosity(verbosity).init().unwrap();

    // parse p2p server address
    let p2p_addr = matches
        .value_of("peer_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P server address: {}", e);
            process::exit(1);
        });

    // parse api server address
    let api_addr = matches
        .value_of("api_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing API server address: {}", e);
            process::exit(1);
        });

    // create channels between server and worker
    let (msg_tx, msg_rx) = channel::unbounded();

    // start the p2p server
    let (server_ctx, server) = server::new(p2p_addr, msg_tx).unwrap();
    server_ctx.start().unwrap();



    // start the worker
    let p2p_workers = matches
        .value_of("p2p_workers")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P workers: {}", e);
            process::exit(1);
        });

    let key_pair = key_pair::random();
    let local_public_key = key_pair.public_key().as_ref().to_vec();
    let local_address = <H160>::from(<H256>::from(digest::digest(&digest::SHA256, &local_public_key[..])));
    let local_addr_u8: [u8; 20] = <[u8; 20]>::from(local_address);
    println!("generate: {:?}",local_address);
        //create new blockchain
    let mut new_blockchain = blockchain::Blockchain::new();
    let blockchain = Arc::new(Mutex::new(new_blockchain));
    let mut new_orphanbuffer = worker::OrphanBuffer::new();
    let orphanbuffer = Arc::new(Mutex::new(new_orphanbuffer));
    let mut new_Mempool = transaction::Mempool::new();
    let mempool = Arc::new(Mutex::new(new_Mempool));
    let mut new_State = transaction::State::new();
    let state = Arc::new(Mutex::new(new_State));
    let mut new_StateSet = transaction::StateSet::new();
    let stateSet = Arc::new(Mutex::new(new_StateSet));
    //TODO: Add ICO
    // let new_file = std::fs::File::create("ICO.txt").expect("create failed");
    let new_file = OpenOptions::new().write(true).create_new(true).open("ICO.txt");
    let mut file = OpenOptions::new().append(true).open("ICO.txt").expect("cannot open file");
    file.write_all(&local_addr_u8).expect("write failed");
    // file.write_all("/n".as_bytes()).expect("write failed");
    // let mut new_sum_delay:f32 = 0.0;
    // let sum_delay = Arc::new(Mutex::new(new_sum_delay));
    // let mut new_num_delay:u8 = 0.0;
    // let num_delay = Arc::new(Mutex::new(new_num_delay));


    //println!("{:?}", data);



    let worker_ctx = worker::new(
        &blockchain,
        &orphanbuffer,
        &mempool,
        &state,
        &stateSet,
        &local_address,
        p2p_workers,
        msg_rx,
        &server,
    );
    worker_ctx.start();

    //start the transaction
    let (transaction_ctx, transaction) = transaction::new(
        &server,
        &mempool,
        &state,
        &stateSet,
        key_pair,
        &local_address,
    );
    transaction_ctx.start();


    // start the miner
    let (miner_ctx, miner) = miner::new(
        &server,
        &mempool,
        &state,
        &stateSet,
        &blockchain,
        &local_public_key[..],
        &local_address,
    );
    miner_ctx.start();

    // connect to known peers
    if let Some(known_peers) = matches.values_of("known_peer") {
        let known_peers: Vec<String> = known_peers.map(|x| x.to_owned()).collect();
        let server = server.clone();
        thread::spawn(move || {
            for peer in known_peers {
                loop {
                    let addr = match peer.parse::<net::SocketAddr>() {
                        Ok(x) => x,
                        Err(e) => {
                            error!("Error parsing peer address {}: {}", &peer, e);
                            break;
                        }
                    };
                    match server.connect(addr) {
                        Ok(_) => {
                            info!("Connected to outgoing peer {}", &addr);
                            break;
                        }
                        Err(e) => {
                            error!(
                                "Error connecting to peer {}, retrying in one second: {}",
                                addr, e
                            );
                            thread::sleep(time::Duration::from_millis(1000));
                            continue;
                        }
                    }
                }
            }
        });
    }


    // start the API server
    ApiServer::start(
        api_addr,
        &miner,
        &server,
        &transaction,
    );

    loop {
        std::thread::park();
    }
}
