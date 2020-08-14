# Stateless Full Node Client

In most current blockchain designs, nodes participating in transaction validation store the whole UTXO set and use it to verify whether a coin was unspent. Instead, we consider a blockchain design where the network maintains the UTXO set in a dynamic accumulator. This blockchain design consists of 4 stateless full nodes and one archival node. Stateless full nodes only need to store their own UTXO sets and membership proofs. [Documentation](https://docs.google.com/document/d/1FRtSDOEkNel9UhohquYTxjjDOgytqyQ8zpr4wt8V1zM/edit?usp=sharing). [Video Demo](https://www.youtube.com/watch?v=Q5XV8xm4l0M).

## Disclaimer
This is a course project repository for ECE 598 PV: Principles of Blockchains, Spring 2020 at University of Illinois, Urbana-Champaign. [Main website of the course](https://courses.grainger.illinois.edu/ece598pv/sp2020/).

## Introduction

### Overview Diagram in Our Implementation
The diagram contains 4 nodes including 3 stateless full nodes connected to 1 archival node; the process of sending state witnesses from the archival node and updating witnesses in each stateless full node.
![Overall Diagram](https://github.com/LEON-HYZ/ece598-block-chain/blob/master/Overall%20Diagram.png?raw=true)
## Designing logistics  
### Full Node
We Add (witness, prime number) pair in TX Input. Full Nodes store StateWitness which only relates to themselves in order to verify generated transactions as well as calculate balance for each client and they could communicate with the Archival Node for recipients when verifying blocks. In this new StateWitness, we store a state-like structure including (prev TX Hash, prev Output Index) <-> (Output Value, Recipient Addr, Prime_number, Witness). Besides updating the accumulator proof, Full Nodes store StateWitness which only relates to themselves in order to verify generated transactions (as well as calculating balance for each client).
### Archival Node
In our archival node, we store the state witnesses in the RSA accumulator. We let Archival Node generate initial state and witnesses in ICO and broadcast it to the network. In this new StateWitness, we store a state-like structure including (prev TX Hash, prev Output Index) <-> (Output Value, Recipient Addr, Prime_number, Witness). The Archival Node stores the whole state(UTXO sets) in order to provide all transaction witnesses and accumulator proof so that each node can verify all transactions in their local storage without the knowledge of all states. When an Archival Node receives a block, it will add it to its own blockchain and update the UTXO sets, then it will update and broadcast the witness to all stateless full nodes.

### Accumulator Design
In our final project, we decided to use the RSA accumulator as a substitute for the Merkle tree in state witness implementation. The RSA accumulator is based on the function A = g^a mod N. 
First we need to choose a modulus N which is the product of two secret primes. These two secret primes are randomly generated and they should be large enough to ensure security. And we need a hash function to map our elements to primes. Then we initialize the accumulator with the initial base g. So we can add something to the accumulator by raising the current accumulator to the value we get from the hash function. We will have a structure to store g, N, prime_set and Hashmap. Hashmap key is (TX Hash, Output Index) and value is a unique prime number. When receiving a new block, before broadcasting new witnesses, we will convert the hash of each transaction to a unique prime number (we assure the generated number is unique by creating a prime_set and check whether the prime already exists in the set, if so, then regenerate) and recalculate the Accumulator proof, after that, we update the witness of each transaction by using formula: primeAcc. To prove the membership, we just need the value of the element, and a witness. The exponential part of the witness is the product of all the values in the accumulator except the value being proven. To prove the non-membership, we need to use Bezout Coefficients to prove that the element and the product of all elements in the set are co-prime.
Aggregating and batching make the RSA accumulator more efficient. Aggregating means combining many proofs in 1 constant size proof. Batching means verifying many proofs at once. However, the exponential calculation would be expensive and it’s hard to transmit such large values. Therefore, we can use NI-PoKE2 to prove that we have the cofactor but not necessarily do the expensive calculations.

## Double Spending Verification
In a live Bitcoin client, we do double spending checks when generating transactions, receiving new transactions, mining new blocks and receiving new blocks. In our previous implementation, we used hashmap in implementing full states in each client, though requiring large storage in each client, we could check double spending to see if the current transaction inputs are in the current states (UTXO sets).
### Verification Process (Normal Membership and NonMembership Verification)
The following graph illustrates the process of double spending check verification. We first include the state witnesses in each transaction while in the generation. Specifically, the state witnesses are included in “Input” of each transaction. When a miner mines a new block, it checks the state witness with the given proof from the accumulator proof hashmap (key:Block Hash, value: proof). When the worker receives either new transactions or new blocks, it would always check these (inside) transactions’ state witnesses along with the proof (using the parent of the current block to look up, since we accept the block who is not the current tip of our blockchain).

### Methods dealing with Big Number Issues and Applications of NI-PoKE2
#### Big Number Transmission
The normal data type for integers is not sufficient for storing and transmitting such big numbers in Rust. We use string to store big integers, every time when we want to use them, we do BigInt packages (crates) transfer/convert string to bytes or bytes to string. For normal exponentiation, we might use bit calculation, but the work is too much for computers; therefore, we use PoKE2 crates to accomplish the work.
### NI-PoKE2 
The exponential calculation would be expensive if we’d like to create an aggregating proof of lots of elements, and it’s hard to transmit such large values. Therefore, we can use NI-PoKE2 to prove we have the cofactor but not necessarily do the expensive calculation.
NI-PoKE2 utilizes the Fiat Shamir heuristic to turn PoKE2 from an interactive protocol into non-interactive. Assume secure hash functions: HG which outputs group elements of unknown order, Hprime which outputs primes, and H. Claim ux = w.
Prover calculates generator g = HG(u,w) and z = gx
Prover calculates l = Hprime(u,w,z)
Prover calculates α = H(u,w,z,l)
Prover performs euclidean division, and produces (q,r): q = floor(x / l) and r = x mod l.
Prover transmits z, Q = uqgαq and r to the Verifier
Verifier calculates generator g = HG(u,w)
Verifier calculates l = Hprime(u,w,z)
Verifier calculates α = H(u,w,z,l)
Verifier checks that Qlurgαr = wzα holds
### Batch Verification Design
We’d like to do batch verification in a set of transactions in a block. Initially, the witnesses for transactions in a block are combined together into a newly generated witness and included in this block’s “Header” using Shamir’s Trick. Afterwards, in each time of verification for a single block, we can do batch verification for all transactions in a constant time complexity.
