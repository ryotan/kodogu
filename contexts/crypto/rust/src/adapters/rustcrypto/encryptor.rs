use aes_gcm::{
    Aes128Gcm, Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use rand::RngCore;

use crate::domain::{
    CryptoError, EncryptionSnafu,
    traits::Encryptor,
    values::{Ciphertext, Plaintext, PublicKey, SymmetricKey},
};

/// AES-GCM based encryptor using RustCrypto
pub struct RustCryptoEncryptor;

impl RustCryptoEncryptor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustCryptoEncryptor {
    fn default() -> Self {
        Self::new()
    }
}

impl Encryptor for RustCryptoEncryptor {
    fn encrypt_symmetric(&self, plaintext: &Plaintext, key: &SymmetricKey) -> Result<Ciphertext, CryptoError> {
        let key_bytes = key.as_bytes();

        // Generate random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = match key_bytes.len() {
            16 => {
                // AES-128-GCM
                let cipher = Aes128Gcm::new_from_slice(key_bytes).map_err(|e| {
                    EncryptionSnafu {
                        reason: format!("Failed to create AES-128 cipher: {}", e),
                    }
                    .build()
                })?;
                cipher.encrypt(nonce, plaintext.as_bytes()).map_err(|e| {
                    EncryptionSnafu {
                        reason: format!("AES-128 encryption failed: {}", e),
                    }
                    .build()
                })?
            }
            32 => {
                // AES-256-GCM
                let cipher = Aes256Gcm::new_from_slice(key_bytes).map_err(|e| {
                    EncryptionSnafu {
                        reason: format!("Failed to create AES-256 cipher: {}", e),
                    }
                    .build()
                })?;
                cipher.encrypt(nonce, plaintext.as_bytes()).map_err(|e| {
                    EncryptionSnafu {
                        reason: format!("AES-256 encryption failed: {}", e),
                    }
                    .build()
                })?
            }
            _ => {
                return Err(EncryptionSnafu {
                    reason: format!("Invalid key size: {} bytes (expected 16 or 32)", key_bytes.len()),
                }
                .build());
            }
        };

        // Prepend nonce to ciphertext: [nonce (12 bytes) | ciphertext]
        let mut result = Vec::with_capacity(12 + encrypted.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&encrypted);

        Ok(Ciphertext::new(result))
    }

    fn encrypt_asymmetric(&self, _plaintext: &Plaintext, _public_key: &PublicKey) -> Result<Ciphertext, CryptoError> {
        // TODO: Implement hybrid encryption (RSA + AES)
        Err(EncryptionSnafu {
            reason: "Asymmetric encryption not yet implemented".to_string(),
        }
        .build())
    }
}
