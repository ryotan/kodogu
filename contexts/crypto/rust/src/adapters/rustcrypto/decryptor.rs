use aes_gcm::{
    Aes128Gcm, Aes256Gcm, KeyInit, Nonce,
    aead::Aead,
};

use crate::domain::{
    CryptoError, DecryptionSnafu,
    traits::Decryptor,
    values::{Ciphertext, Plaintext, PrivateKey, SymmetricKey},
};

/// AES-GCM based decryptor using RustCrypto
pub struct RustCryptoDecryptor;

impl RustCryptoDecryptor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustCryptoDecryptor {
    fn default() -> Self {
        Self::new()
    }
}

impl Decryptor for RustCryptoDecryptor {
    fn decrypt_symmetric(&self, ciphertext: &Ciphertext, key: &SymmetricKey) -> Result<Plaintext, CryptoError> {
        let data = ciphertext.as_bytes();
        let key_bytes = key.as_bytes();

        // Extract nonce and ciphertext: [nonce (12 bytes) | ciphertext]
        if data.len() < 12 {
            return Err(DecryptionSnafu {
                reason: "Ciphertext too short (missing nonce)".to_string(),
            }
            .build());
        }

        let (nonce_bytes, encrypted) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted = match key_bytes.len() {
            16 => {
                // AES-128-GCM
                let cipher = Aes128Gcm::new_from_slice(key_bytes).map_err(|e| {
                    DecryptionSnafu {
                        reason: format!("Failed to create AES-128 cipher: {}", e),
                    }
                    .build()
                })?;
                cipher.decrypt(nonce, encrypted).map_err(|e| {
                    DecryptionSnafu {
                        reason: format!("AES-128 decryption failed: {}", e),
                    }
                    .build()
                })?
            }
            32 => {
                // AES-256-GCM
                let cipher = Aes256Gcm::new_from_slice(key_bytes).map_err(|e| {
                    DecryptionSnafu {
                        reason: format!("Failed to create AES-256 cipher: {}", e),
                    }
                    .build()
                })?;
                cipher.decrypt(nonce, encrypted).map_err(|e| {
                    DecryptionSnafu {
                        reason: format!("AES-256 decryption failed: {}", e),
                    }
                    .build()
                })?
            }
            _ => {
                return Err(DecryptionSnafu {
                    reason: format!("Invalid key size: {} bytes (expected 16 or 32)", key_bytes.len()),
                }
                .build());
            }
        };

        Ok(Plaintext::new(decrypted))
    }

    fn decrypt_asymmetric(&self, _ciphertext: &Ciphertext, _private_key: &PrivateKey) -> Result<Plaintext, CryptoError> {
        // TODO: Implement hybrid decryption (RSA + AES)
        Err(DecryptionSnafu {
            reason: "Asymmetric decryption not yet implemented".to_string(),
        }
        .build())
    }
}
