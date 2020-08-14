# Stateless Full Node Client

In most current blockchain designs, nodes participating in transaction validation store the whole UTXO set and use it to verify whether a coin was unspent. Instead, we consider a blockchain design where the network maintains the UTXO set in a dynamic accumulator. This blockchain design consists of 4 stateless full nodes and one archival node. Stateless full nodes only need to store their own UTXO sets and membership proofs.

## Disclaimer
This is a course project repository for ECE 598 PV: Principles of Blockchains, Spring 2020 at University of Illinois, Urbana-Champaign. [Main website of the course](https://courses.grainger.illinois.edu/ece598pv/sp2020/).

## Introduction

### Overview Diagram in Our Implementation
The diagram contains 4 nodes including 3 stateless full nodes connected to 1 archival node; the process of sending state witnesses from the archival node and updating witnesses in each stateless full node.
![Image of Yaktocat](https://octodex.github.com/images/yaktocat.png)
