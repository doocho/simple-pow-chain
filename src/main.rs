// src/main.rs
mod block;
mod blockchain;
mod transaction;
mod keys;

use transaction::Transaction;
use blockchain::Blockchain;
use keys::Keypair;

fn main() {
    // 1. generate keypair
    let alice = Keypair::new();
    let bob = Keypair::new();

    println!("Alice address: {}", alice.address);
    println!("Bob address: {}", bob.address);

    // 2. create transaction
    let mut tx = Transaction::new(alice.address.clone(), bob.address.clone(), 50);

    // 3. sign transaction
    tx.sign(&alice.secret_key).unwrap();
    println!("signed transaction: {}", tx);

    // 4. verify transaction
    assert!(tx.verify(), "signature verification failed!");
    println!("signature verification successful!");

    // 5. add transaction to blockchain
    let mut bc = Blockchain::new(3);
    bc.add_block(vec![tx]);

    println!("\nfinal chain:");
    for block in &bc.chain {
        println!("{}", block);
    }
}