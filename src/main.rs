mod block;
mod blockchain;
mod transaction;

use blockchain::Blockchain;
use transaction::Transaction;

fn main() {
    let mut bc = Blockchain::new(4); // Difficulty 4 → hash must start with 0000

    println!("🔗 블록체인 시작!");
    println!("{:?}", bc.chain[0]);

    let txs1 = vec![
        Transaction::new("Alice".to_string(), "Bob".to_string(), 10),
        Transaction::new("Bob".to_string(), "Charlie".to_string(), 5),
    ];
    bc.add_block(txs1);
    
    let txs2 = vec![Transaction::new("Bob".to_string(), "Carol".to_string(), 5)];
    bc.add_block(txs2);

    println!("\n📦 최종 체인:");
    for block in &bc.chain {
        println!("{}", block);
    }

    // JSON serialization example
    let json = serde_json::to_string_pretty(&bc.chain).unwrap();
    println!("\n📄 JSON 출력:\n{}", json);
}
