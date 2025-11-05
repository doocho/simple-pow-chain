mod block;
mod blockchain;

use blockchain::Blockchain;

fn main() {
    let mut bc = Blockchain::new(4); // Difficulty 4 → hash must start with 0000

    println!("🔗 블록체인 시작!");
    println!("{:?}", bc.chain[0]);

    bc.add_block("Alice -> Bob: 10 BTC".to_string());
    bc.add_block("Bob -> Carol: 5 BTC".to_string());

    println!("\n📦 최종 체인:");
    for block in &bc.chain {
        println!("{}", block);
    }

    // JSON serialization example
    let json = serde_json::to_string_pretty(&bc.chain).unwrap();
    println!("\n📄 JSON 출력:\n{}", json);
}