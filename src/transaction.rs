use hex;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub public_key: Option<String>,
    pub signature: Option<String>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            public_key: None,
            signature: None,
        }
    }

    /// Generate a hash based on the transaction contents
    pub fn hash(&self) -> String {
        let input = format!("{}{}{}", self.from, self.to, self.amount);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn sign(&mut self, secret_key_hex: &str) -> Result<(), String> {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&hex::decode(secret_key_hex).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let message = Message::from_digest_slice(&hex::decode(self.hash()).unwrap()).unwrap();
        let signature = secp.sign_ecdsa(&message, &secret_key);
        self.public_key = Some(hex::encode(public_key.serialize()));
        self.signature = Some(hex::encode(signature.serialize_der()));
        Ok(())
    }

    pub fn verify(&self) -> bool {
        if self.signature.is_none() || self.public_key.is_none() {
            return false;
        }

        let secp = Secp256k1::new();
        let sig = match secp256k1::ecdsa::Signature::from_der(
            &hex::decode(self.signature.as_ref().unwrap()).unwrap(),
        ) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let msg = match Message::from_digest_slice(&hex::decode(self.hash()).unwrap()) {
            Ok(m) => m,
            Err(_) => return false,
        };

        let pubkey = match PublicKey::from_slice(&hex::decode(self.public_key.as_ref().unwrap()).unwrap()) {
            Ok(p) => p,
            Err(_) => return false,
        };

        secp.verify_ecdsa(&msg, &sig, &pubkey).is_ok()
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} → {}: {} coins {}",
            self.from,
            self.to,
            self.amount,
            if self.signature.is_some() {
                "(signed)"
            } else {
                ""
            }
        )
    }
}
