//! AES-GCM symmetric encryption implementations
//!
//! Provides AES-128-GCM and AES-256-GCM encryption with automatic nonce generation.
//!
//! ## Ciphertext Format
//!
//! ```text
//! [nonce (12 bytes) | ciphertext + auth_tag]
//! ```
//!
//! - Nonce: Automatically generated using `OsRng` (cryptographically secure)
//! - Auth tag: Automatically appended by AES-GCM (16 bytes)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crypto::{Aes256Gcm, AesGcm, Plaintext, SymmetricKey};
//!
//! let key = SymmetricKey::aes256([0u8; 32]);
//! let plaintext = Plaintext::from_string("Hello, World!".to_string());
//!
//! // ZST type (compile-time dispatch)
//! let ciphertext = Aes256Gcm.encrypt(&plaintext, &key)?;
//! let decrypted = Aes256Gcm.decrypt(&ciphertext, &key)?;
//!
//! // Enum (runtime selection)
//! let cipher = AesGcm::aes256();
//! let ciphertext = cipher.encrypt(&plaintext, &key)?;
//! ```

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes128Gcm as Aes128GcmImpl, Aes256Gcm as Aes256GcmImpl, Nonce,
};
use rand::RngCore;
use snafu::ResultExt;

use crate::{
    AesDecryptionSnafu, AesEncryptionSnafu, CipherError, Ciphertext, CiphertextTooShortSnafu,
    Plaintext, SymmetricKey, WrongKeyTypeSnafu,
};
use snafu::ensure;

const NONCE_SIZE: usize = 12; // 96 bits for AES-GCM

//
// ZST Types
//

/// AES-256-GCM encryption
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct Aes256Gcm;

impl Aes256Gcm {
    /// Encrypt plaintext with AES-256-GCM
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt
    /// * `key` - Must be `SymmetricKey::Aes256`, otherwise returns `WrongKeyType` error
    ///
    /// # Returns
    ///
    /// Ciphertext in format: `[nonce (12 bytes) | ciphertext + auth_tag]`
    pub fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &SymmetricKey,
    ) -> Result<Ciphertext, CipherError> {
        // Check key type
        let key_bytes = match key {
            SymmetricKey::Aes256(k) => k.as_ref(),
            SymmetricKey::Aes128(_) => {
                return WrongKeyTypeSnafu {
                    expected: "Aes256",
                    got: "Aes128",
                }
                .fail()
            }
        };

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher
        let cipher = Aes256GcmImpl::new(key_bytes.into());

        // Encrypt
        let ciphertext_bytes = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .context(AesEncryptionSnafu)?;

        // Combine nonce + ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext_bytes.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext_bytes);

        Ok(Ciphertext::from_bytes(result))
    }

    /// Decrypt ciphertext with AES-256-GCM
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Data to decrypt (must include nonce)
    /// * `key` - Must be `SymmetricKey::Aes256`, otherwise returns `WrongKeyType` error
    pub fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &SymmetricKey,
    ) -> Result<Plaintext, CipherError> {
        // Check key type
        let key_bytes = match key {
            SymmetricKey::Aes256(k) => k.as_ref(),
            SymmetricKey::Aes128(_) => {
                return WrongKeyTypeSnafu {
                    expected: "Aes256",
                    got: "Aes128",
                }
                .fail()
            }
        };

        let bytes = ciphertext.as_bytes();

        // Check minimum length (nonce + at least some ciphertext)
        ensure!(
            bytes.len() >= NONCE_SIZE,
            CiphertextTooShortSnafu { size: bytes.len() }
        );

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext_bytes) = bytes.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Create cipher
        let cipher = Aes256GcmImpl::new(key_bytes.into());

        // Decrypt
        let plaintext_bytes = cipher
            .decrypt(nonce, ciphertext_bytes)
            .context(AesDecryptionSnafu)?;

        Ok(Plaintext::from_bytes(plaintext_bytes))
    }
}

/// AES-128-GCM encryption
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct Aes128Gcm;

impl Aes128Gcm {
    /// Encrypt plaintext with AES-128-GCM
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt
    /// * `key` - Must be `SymmetricKey::Aes128`, otherwise returns `WrongKeyType` error
    pub fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &SymmetricKey,
    ) -> Result<Ciphertext, CipherError> {
        // Check key type
        let key_bytes = match key {
            SymmetricKey::Aes128(k) => k.as_ref(),
            SymmetricKey::Aes256(_) => {
                return WrongKeyTypeSnafu {
                    expected: "Aes128",
                    got: "Aes256",
                }
                .fail()
            }
        };

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher
        let cipher = Aes128GcmImpl::new(key_bytes.into());

        // Encrypt
        let ciphertext_bytes = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .context(AesEncryptionSnafu)?;

        // Combine nonce + ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext_bytes.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext_bytes);

        Ok(Ciphertext::from_bytes(result))
    }

    /// Decrypt ciphertext with AES-128-GCM
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Data to decrypt (must include nonce)
    /// * `key` - Must be `SymmetricKey::Aes128`, otherwise returns `WrongKeyType` error
    pub fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &SymmetricKey,
    ) -> Result<Plaintext, CipherError> {
        // Check key type
        let key_bytes = match key {
            SymmetricKey::Aes128(k) => k.as_ref(),
            SymmetricKey::Aes256(_) => {
                return WrongKeyTypeSnafu {
                    expected: "Aes128",
                    got: "Aes256",
                }
                .fail()
            }
        };

        let bytes = ciphertext.as_bytes();

        // Check minimum length
        ensure!(
            bytes.len() >= NONCE_SIZE,
            CiphertextTooShortSnafu { size: bytes.len() }
        );

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext_bytes) = bytes.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Create cipher
        let cipher = Aes128GcmImpl::new(key_bytes.into());

        // Decrypt
        let plaintext_bytes = cipher
            .decrypt(nonce, ciphertext_bytes)
            .context(AesDecryptionSnafu)?;

        Ok(Plaintext::from_bytes(plaintext_bytes))
    }
}

//
// Enum Wrapper
//

/// AES-GCM encryption (runtime selection)
///
/// Enum wrapper for runtime dispatch between AES-128 and AES-256.
#[derive(Debug, Clone, Copy)]
pub enum AesGcm {
    /// AES-128-GCM
    Aes128,
    /// AES-256-GCM
    Aes256,
}

impl AesGcm {
    /// Create AES-128-GCM cipher (static factory method)
    pub fn aes128() -> Self {
        Self::Aes128
    }

    /// Create AES-256-GCM cipher (static factory method)
    pub fn aes256() -> Self {
        Self::Aes256
    }

    /// Encrypt plaintext
    ///
    /// Delegates to the appropriate ZST implementation based on enum variant.
    pub fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &SymmetricKey,
    ) -> Result<Ciphertext, CipherError> {
        match self {
            AesGcm::Aes128 => Aes128Gcm.encrypt(plaintext, key),
            AesGcm::Aes256 => Aes256Gcm.encrypt(plaintext, key),
        }
    }

    /// Decrypt ciphertext
    ///
    /// Delegates to the appropriate ZST implementation based on enum variant.
    pub fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &SymmetricKey,
    ) -> Result<Plaintext, CipherError> {
        match self {
            AesGcm::Aes128 => Aes128Gcm.decrypt(ciphertext, key),
            AesGcm::Aes256 => Aes256Gcm.decrypt(ciphertext, key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes256_round_trip() {
        let key = SymmetricKey::aes256([42u8; 32]);
        let plaintext = Plaintext::from_string("Hello, World!".to_string());

        let ciphertext = Aes256Gcm.encrypt(&plaintext, &key).unwrap();
        let decrypted = Aes256Gcm.decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes128_round_trip() {
        let key = SymmetricKey::aes128([42u8; 16]);
        let plaintext = Plaintext::from_string("Hello, World!".to_string());

        let ciphertext = Aes128Gcm.encrypt(&plaintext, &key).unwrap();
        let decrypted = Aes128Gcm.decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256_wrong_key_type() {
        let key = SymmetricKey::aes128([42u8; 16]); // Wrong key type
        let plaintext = Plaintext::from_string("Hello, World!".to_string());

        let result = Aes256Gcm.encrypt(&plaintext, &key);
        assert!(matches!(result, Err(CipherError::WrongKeyType { .. })));
    }

    #[test]
    fn test_aes128_wrong_key_type() {
        let key = SymmetricKey::aes256([42u8; 32]); // Wrong key type
        let plaintext = Plaintext::from_string("Hello, World!".to_string());

        let result = Aes128Gcm.encrypt(&plaintext, &key);
        assert!(matches!(result, Err(CipherError::WrongKeyType { .. })));
    }

    #[test]
    fn test_nonce_randomness() {
        let key = SymmetricKey::aes256([42u8; 32]);
        let plaintext = Plaintext::from_string("Hello, World!".to_string());

        let ciphertext1 = Aes256Gcm.encrypt(&plaintext, &key).unwrap();
        let ciphertext2 = Aes256Gcm.encrypt(&plaintext, &key).unwrap();

        // Same plaintext encrypted twice should produce different ciphertexts
        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_enum_aes256() {
        let cipher = AesGcm::aes256();
        let key = SymmetricKey::aes256([42u8; 32]);
        let plaintext = Plaintext::from_string("Test".to_string());

        let ciphertext = cipher.encrypt(&plaintext, &key).unwrap();
        let decrypted = cipher.decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_enum_aes128() {
        let cipher = AesGcm::aes128();
        let key = SymmetricKey::aes128([42u8; 16]);
        let plaintext = Plaintext::from_string("Test".to_string());

        let ciphertext = cipher.encrypt(&plaintext, &key).unwrap();
        let decrypted = cipher.decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
