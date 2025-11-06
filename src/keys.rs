// src/keys.rs
use hex;
use rand::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

pub struct Keypair {
    pub secret_key: String,
    pub public_key: String,
    pub address: String,
}

impl Keypair {
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);

        let secret_key = SecretKey::from_slice(&secret_bytes).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let pubkey_bytes = public_key.serialize(); // 65 bytes
        let hash = Sha256::digest(&pubkey_bytes);
        let address = format!("1{}", hex::encode(&hash[..10])); // 간단한 주소 형식

        Keypair {
            secret_key: hex::encode(secret_bytes),
            public_key: hex::encode(pubkey_bytes),
            address,
        }
    }
}
