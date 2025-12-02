//! RSA hybrid encryption (RSA + AES-256-GCM)
//!
//! Combines RSA asymmetric encryption with AES-256-GCM symmetric encryption for
//! efficient encryption of arbitrarily large data.
//!
//! ## How it works
//!
//! **Encryption**:
//! 1. Generate random AES-256 key
//! 2. Encrypt plaintext with AES-256-GCM
//! 3. Encrypt AES key with RSA public key (OAEP padding)
//! 4. Return: [encrypted_key_length (4 bytes) | encrypted_key | aes_ciphertext]
//!
//! **Decryption**:
//! 1. Extract encrypted AES key length and data
//! 2. Decrypt AES key with RSA private key
//! 3. Decrypt ciphertext with recovered AES-256 key
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crypto::{RsaHybrid2048, Plaintext};
//!
//! // ZST type
//! let keypair = crypto::Rsa2048.generate()?;
//! let plaintext = Plaintext::from_string("Secret message".to_string());
//!
//! let ciphertext = RsaHybrid2048.encrypt(&plaintext, &keypair.public_key)?;
//! let decrypted = RsaHybrid2048.decrypt(&ciphertext, &keypair.private_key)?;
//!
//! // Enum
//! let cipher = RsaHybrid::rsa2048();
//! let ciphertext = cipher.encrypt(&plaintext, &keypair.public_key)?;
//! ```

use pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::{Oaep, RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;
use snafu::ResultExt;
use suzunari_error::suzunari_error;

use crate::{Aes256Gcm, Ciphertext, Plaintext, PrivateKey, PublicKey, RandomAes256, SymmetricKey};

//
// Error Types
//

/// Errors that can occur during RSA hybrid encryption/decryption
#[suzunari_error]
#[snafu(visibility(pub))]
pub enum RsaHybridError {
    /// AES key generation failed
    #[snafu(display("AES key generation failed"))]
    AesKeyGeneration {
        source: crate::KeyGenerationError,
    },

    /// AES encryption failed
    #[snafu(display("AES encryption failed"))]
    AesEncryption { source: crate::CipherError },

    /// AES decryption failed
    #[snafu(display("AES decryption failed"))]
    AesDecryption { source: crate::CipherError },

    /// RSA public key parsing failed
    #[snafu(display("RSA public key parsing failed"))]
    RsaPublicKeyParsing { source: pkcs8::spki::Error },

    /// RSA private key parsing failed
    #[snafu(display("RSA private key parsing failed"))]
    RsaPrivateKeyParsing { source: pkcs8::Error },

    /// RSA encryption failed
    #[snafu(display("RSA encryption failed"))]
    RsaEncryption { source: rsa::Error },

    /// RSA decryption failed
    #[snafu(display("RSA decryption failed"))]
    RsaDecryption { source: rsa::Error },

    /// Ciphertext format invalid
    #[snafu(display("Ciphertext format invalid: {reason}"))]
    InvalidCiphertextFormat { reason: String },
}

//
// ZST Types
//

/// RSA-2048 hybrid cipher (RSA + AES-256-GCM)
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct RsaHybrid2048;

impl RsaHybrid2048 {
    /// Encrypt plaintext using RSA-2048 hybrid encryption
    ///
    /// Generates random AES-256 key, encrypts plaintext with AES-256-GCM,
    /// then encrypts the AES key with RSA-2048 public key.
    pub fn encrypt(
        &self,
        plaintext: &Plaintext,
        public_key: &PublicKey,
    ) -> Result<Ciphertext, RsaHybridError> {
        // 1. Generate random AES-256 key
        let aes_key = RandomAes256.generate().context(AesKeyGenerationSnafu)?;

        // 2. Encrypt plaintext with AES-256-GCM
        let aes_ciphertext = Aes256Gcm
            .encrypt(plaintext, &aes_key)
            .context(AesEncryptionSnafu)?;

        // 3. Extract AES key bytes
        let aes_key_bytes = match &aes_key {
            SymmetricKey::Aes256(key) => key.as_ref(),
            _ => unreachable!("RandomAes256 always generates Aes256 key"),
        };

        // 4. Parse RSA public key
        let rsa_public_key = RsaPublicKey::from_public_key_pem(
            std::str::from_utf8(public_key.as_pem()).unwrap(),
        )
        .context(RsaPublicKeyParsingSnafu)?;

        // 5. Encrypt AES key with RSA
        let padding = Oaep::new::<Sha256>();
        let mut rng = rand::rngs::OsRng;
        let encrypted_key = rsa_public_key
            .encrypt(&mut rng, padding, aes_key_bytes)
            .context(RsaEncryptionSnafu)?;

        // 6. Combine: [encrypted_key_length (4 bytes) | encrypted_key | aes_ciphertext]
        let encrypted_key_len = (encrypted_key.len() as u32).to_be_bytes();
        let mut result = Vec::new();
        result.extend_from_slice(&encrypted_key_len);
        result.extend_from_slice(&encrypted_key);
        result.extend_from_slice(aes_ciphertext.as_bytes());

        Ok(Ciphertext::from_bytes(result))
    }

    /// Decrypt ciphertext using RSA-2048 hybrid decryption
    ///
    /// Extracts and decrypts the AES key with RSA-2048 private key,
    /// then decrypts the ciphertext with the recovered AES-256 key.
    pub fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        private_key: &PrivateKey,
    ) -> Result<Plaintext, RsaHybridError> {
        let data = ciphertext.as_bytes();

        // 1. Extract encrypted key length
        if data.len() < 4 {
            return InvalidCiphertextFormatSnafu {
                reason: "too short to contain key length",
            }
            .fail();
        }
        let encrypted_key_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

        // 2. Extract encrypted key and AES ciphertext
        if data.len() < 4 + encrypted_key_len {
            return InvalidCiphertextFormatSnafu {
                reason: "too short to contain encrypted key",
            }
            .fail();
        }
        let encrypted_key = &data[4..4 + encrypted_key_len];
        let aes_ciphertext = &data[4 + encrypted_key_len..];

        // 3. Parse RSA private key
        let rsa_private_key = RsaPrivateKey::from_pkcs8_pem(
            std::str::from_utf8(private_key.as_pem()).unwrap(),
        )
        .context(RsaPrivateKeyParsingSnafu)?;

        // 4. Decrypt AES key with RSA
        let padding = Oaep::new::<Sha256>();
        let aes_key_bytes = rsa_private_key
            .decrypt(padding, encrypted_key)
            .context(RsaDecryptionSnafu)?;

        // 5. Reconstruct AES key
        if aes_key_bytes.len() != 32 {
            return InvalidCiphertextFormatSnafu {
                reason: "decrypted AES key is not 32 bytes",
            }
            .fail();
        }
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&aes_key_bytes);
        let aes_key = SymmetricKey::aes256(key_array);

        // 6. Decrypt with AES-256-GCM
        let plaintext = Aes256Gcm
            .decrypt(&Ciphertext::from_bytes(aes_ciphertext.to_vec()), &aes_key)
            .context(AesDecryptionSnafu)?;

        Ok(plaintext)
    }
}

/// RSA-4096 hybrid cipher (RSA + AES-256-GCM)
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct RsaHybrid4096;

impl RsaHybrid4096 {
    /// Encrypt plaintext using RSA-4096 hybrid encryption
    pub fn encrypt(
        &self,
        plaintext: &Plaintext,
        public_key: &PublicKey,
    ) -> Result<Ciphertext, RsaHybridError> {
        // Same implementation as RsaHybrid2048 (key size detected from PEM)
        RsaHybrid2048.encrypt(plaintext, public_key)
    }

    /// Decrypt ciphertext using RSA-4096 hybrid decryption
    pub fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        private_key: &PrivateKey,
    ) -> Result<Plaintext, RsaHybridError> {
        // Same implementation as RsaHybrid2048 (key size detected from PEM)
        RsaHybrid2048.decrypt(ciphertext, private_key)
    }
}

//
// Enum Wrapper
//

/// RSA hybrid cipher (runtime selection)
///
/// Enum wrapper for runtime dispatch between RSA-2048 and RSA-4096 hybrid encryption.
#[derive(Debug, Clone, Copy)]
pub enum RsaHybrid {
    /// RSA-2048 hybrid cipher
    Rsa2048,
    /// RSA-4096 hybrid cipher
    Rsa4096,
}

impl RsaHybrid {
    /// Create RSA-2048 hybrid cipher (static factory method)
    pub fn rsa2048() -> Self {
        Self::Rsa2048
    }

    /// Create RSA-4096 hybrid cipher (static factory method)
    pub fn rsa4096() -> Self {
        Self::Rsa4096
    }

    /// Encrypt plaintext
    ///
    /// Delegates to the appropriate ZST implementation based on enum variant.
    pub fn encrypt(
        &self,
        plaintext: &Plaintext,
        public_key: &PublicKey,
    ) -> Result<Ciphertext, RsaHybridError> {
        match self {
            RsaHybrid::Rsa2048 => RsaHybrid2048.encrypt(plaintext, public_key),
            RsaHybrid::Rsa4096 => RsaHybrid4096.encrypt(plaintext, public_key),
        }
    }

    /// Decrypt ciphertext
    ///
    /// Delegates to the appropriate ZST implementation based on enum variant.
    pub fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        private_key: &PrivateKey,
    ) -> Result<Plaintext, RsaHybridError> {
        match self {
            RsaHybrid::Rsa2048 => RsaHybrid2048.decrypt(ciphertext, private_key),
            RsaHybrid::Rsa4096 => RsaHybrid4096.decrypt(ciphertext, private_key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rsa2048, Rsa4096};

    #[test]
    fn test_rsa2048_hybrid_round_trip() {
        // Generate keypair
        let keypair = Rsa2048.generate().unwrap();

        // Encrypt and decrypt
        let plaintext = Plaintext::from_string("Secret message".to_string());
        let ciphertext = RsaHybrid2048
            .encrypt(&plaintext, &keypair.public_key)
            .unwrap();
        let decrypted = RsaHybrid2048
            .decrypt(&ciphertext, &keypair.private_key)
            .unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_rsa4096_hybrid_round_trip() {
        // Generate keypair
        let keypair = Rsa4096.generate().unwrap();

        // Encrypt and decrypt
        let plaintext = Plaintext::from_string("Secret message".to_string());
        let ciphertext = RsaHybrid4096
            .encrypt(&plaintext, &keypair.public_key)
            .unwrap();
        let decrypted = RsaHybrid4096
            .decrypt(&ciphertext, &keypair.private_key)
            .unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_wrong_private_key_fails() {
        // Generate two different keypairs
        let keypair1 = Rsa2048.generate().unwrap();
        let keypair2 = Rsa2048.generate().unwrap();

        // Encrypt with keypair1
        let plaintext = Plaintext::from_string("Secret".to_string());
        let ciphertext = RsaHybrid2048
            .encrypt(&plaintext, &keypair1.public_key)
            .unwrap();

        // Decrypt with keypair2 should fail
        assert!(RsaHybrid2048
            .decrypt(&ciphertext, &keypair2.private_key)
            .is_err());
    }

    #[test]
    fn test_ciphertext_randomness() {
        let keypair = Rsa2048.generate().unwrap();
        let plaintext = Plaintext::from_string("Same plaintext".to_string());

        let ciphertext1 = RsaHybrid2048
            .encrypt(&plaintext, &keypair.public_key)
            .unwrap();
        let ciphertext2 = RsaHybrid2048
            .encrypt(&plaintext, &keypair.public_key)
            .unwrap();

        // Same plaintext should produce different ciphertexts (random AES key + nonce)
        assert_ne!(ciphertext1.as_bytes(), ciphertext2.as_bytes());
    }

    #[test]
    fn test_enum_rsa2048() {
        let keypair = Rsa2048.generate().unwrap();
        let cipher = RsaHybrid::rsa2048();

        let plaintext = Plaintext::from_string("Test".to_string());
        let ciphertext = cipher.encrypt(&plaintext, &keypair.public_key).unwrap();
        let decrypted = cipher.decrypt(&ciphertext, &keypair.private_key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_enum_rsa4096() {
        let keypair = Rsa4096.generate().unwrap();
        let cipher = RsaHybrid::rsa4096();

        let plaintext = Plaintext::from_string("Test".to_string());
        let ciphertext = cipher.encrypt(&plaintext, &keypair.public_key).unwrap();
        let decrypted = cipher.decrypt(&ciphertext, &keypair.private_key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_large_plaintext() {
        let keypair = Rsa2048.generate().unwrap();

        // Test with large data (RSA alone would fail, but hybrid works)
        let large_text = "A".repeat(10000);
        let plaintext = Plaintext::from_string(large_text);

        let ciphertext = RsaHybrid2048
            .encrypt(&plaintext, &keypair.public_key)
            .unwrap();
        let decrypted = RsaHybrid2048
            .decrypt(&ciphertext, &keypair.private_key)
            .unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_ciphertext_too_short_fails() {
        let keypair = Rsa2048.generate().unwrap();

        // Ciphertext with less than 4 bytes (key length size)
        let short_ciphertext = Ciphertext::from_bytes(vec![1, 2, 3]);

        let result = RsaHybrid2048.decrypt(&short_ciphertext, &keypair.private_key);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, RsaHybridError::InvalidCiphertextFormat { .. }));
    }

    #[test]
    fn test_invalid_encrypted_key_length_fails() {
        let keypair = Rsa2048.generate().unwrap();

        // Malformed ciphertext: claims 1000 bytes for key but only has 10 total
        // Format: [encrypted_key_length (4 bytes) | encrypted_key | aes_ciphertext]
        let mut malformed = vec![];
        malformed.extend_from_slice(&1000u32.to_be_bytes()); // Claim 1000 bytes
        malformed.extend_from_slice(&[0u8; 6]); // But only provide 6 bytes

        let malformed_ciphertext = Ciphertext::from_bytes(malformed);

        let result = RsaHybrid2048.decrypt(&malformed_ciphertext, &keypair.private_key);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, RsaHybridError::InvalidCiphertextFormat { .. }));
    }

    #[test]
    fn test_corrupted_ciphertext_fails() {
        let keypair = Rsa2048.generate().unwrap();

        let plaintext = Plaintext::from_string("Test".to_string());
        let ciphertext = RsaHybrid2048
            .encrypt(&plaintext, &keypair.public_key)
            .unwrap();

        // Corrupt the ciphertext by flipping a byte in the middle
        let mut corrupted_bytes = ciphertext.as_bytes().to_vec();
        if corrupted_bytes.len() > 10 {
            corrupted_bytes[10] ^= 0xFF;
        }
        let corrupted_ciphertext = Ciphertext::from_bytes(corrupted_bytes);

        // Decryption should fail
        let result = RsaHybrid2048.decrypt(&corrupted_ciphertext, &keypair.private_key);
        assert!(result.is_err());
    }
}
