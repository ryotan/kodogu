use aes_gcm::{
    Aes128Gcm, Aes256Gcm, KeyInit, Nonce,
    aead::Aead,
};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use pkcs8::DecodePrivateKey;

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

    fn decrypt_asymmetric(&self, ciphertext: &Ciphertext, private_key: &PrivateKey) -> Result<Plaintext, CryptoError> {
        // Hybrid decryption: RSA + AES-256-GCM
        // Format: [encrypted_key_len (2 bytes big-endian) | encrypted_key | aes_ciphertext]

        let data = ciphertext.as_bytes();

        // 1. Extract encrypted key length
        if data.len() < 2 {
            return Err(DecryptionSnafu {
                reason: "Ciphertext too short (missing key length)".to_string(),
            }
            .build());
        }
        let key_len = u16::from_be_bytes([data[0], data[1]]) as usize;

        // 2. Extract encrypted key and AES ciphertext
        if data.len() < 2 + key_len {
            return Err(DecryptionSnafu {
                reason: "Ciphertext too short (incomplete encrypted key)".to_string(),
            }
            .build());
        }
        let encrypted_key = &data[2..2 + key_len];
        let aes_ciphertext_bytes = &data[2 + key_len..];

        // 3. Parse RSA private key from PEM
        let pem_string = String::from_utf8(private_key.as_bytes().to_vec()).map_err(|e| {
            DecryptionSnafu {
                reason: format!("Invalid PEM encoding: {}", e),
            }
            .build()
        })?;
        let rsa_private_key = RsaPrivateKey::from_pkcs8_pem(&pem_string).map_err(|e| {
            DecryptionSnafu {
                reason: format!("Failed to parse RSA private key: {}", e),
            }
            .build()
        })?;

        // 4. Decrypt AES key with RSA private key
        let aes_key_bytes = rsa_private_key
            .decrypt(Pkcs1v15Encrypt, encrypted_key)
            .map_err(|e| {
                DecryptionSnafu {
                    reason: format!("RSA decryption failed: {}", e),
                }
                .build()
            })?;
        let aes_key = SymmetricKey::new(aes_key_bytes);

        // 5. Decrypt ciphertext with AES key
        let aes_ciphertext = Ciphertext::new(aes_ciphertext_bytes.to_vec());
        self.decrypt_symmetric(&aes_ciphertext, &aes_key)
    }
}
