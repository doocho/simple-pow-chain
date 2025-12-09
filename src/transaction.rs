use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A transaction transferring coins between addresses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub signature: Option<String>,
    pub public_key: Option<String>,
}

impl Transaction {
    /// Create a new unsigned transaction
    pub fn new(from: String, to: String, amount: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            signature: None,
            public_key: None,
        }
    }

    /// Create a coinbase (mining reward) transaction
    pub fn coinbase(to: String, amount: u64) -> Self {
        Transaction {
            from: String::from("coinbase"),
            to,
            amount,
            signature: None,
            public_key: None,
        }
    }

    /// Calculate hash of the transaction
    pub fn hash(&self) -> String {
        let data = format!("{}{}{}", self.from, self.to, self.amount);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Sign the transaction with a private key
    pub fn sign(&mut self, secret_key_hex: &str) -> Result<(), String> {
        let secp = Secp256k1::new();

        let secret_bytes = hex::decode(secret_key_hex).map_err(|e| e.to_string())?;
        let secret_key = SecretKey::from_slice(&secret_bytes).map_err(|e| e.to_string())?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let hash_bytes = hex::decode(self.hash()).map_err(|e| e.to_string())?;
        let message = Message::from_digest_slice(&hash_bytes).map_err(|e| e.to_string())?;

        let signature = secp.sign_ecdsa(&message, &secret_key);

        self.public_key = Some(hex::encode(public_key.serialize()));
        self.signature = Some(hex::encode(signature.serialize_der()));

        Ok(())
    }

    /// Verify the transaction signature
    pub fn verify(&self) -> bool {
        // Coinbase transactions don't need verification
        if self.from == "coinbase" {
            return true;
        }

        let (sig_hex, pubkey_hex) = match (&self.signature, &self.public_key) {
            (Some(s), Some(p)) => (s, p),
            _ => return false,
        };

        let secp = Secp256k1::new();

        let sig_bytes = match hex::decode(sig_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let signature = match secp256k1::ecdsa::Signature::from_der(&sig_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let pubkey_bytes = match hex::decode(pubkey_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let public_key = match PublicKey::from_slice(&pubkey_bytes) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let hash_bytes = match hex::decode(self.hash()) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let message = match Message::from_digest_slice(&hash_bytes) {
            Ok(m) => m,
            Err(_) => return false,
        };

        secp.verify_ecdsa(&message, &signature, &public_key).is_ok()
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let signed = if self.signature.is_some() { " (signed)" } else { "" };
        write!(f, "{} -> {}: {}{}", self.from, self.to, self.amount, signed)
    }
}
