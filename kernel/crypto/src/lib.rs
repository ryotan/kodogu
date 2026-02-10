//! # KoDoGu Crypto Library
//!
//! Cryptographic utilities for symmetric/asymmetric encryption, signatures, and key derivation.
//!
//! ## Design Patterns
//!
//! 1. **trait**: Abstract interface for crypto operations
//! 2. **ZST types**: Compile-time dispatch (e.g., `Aes128Cbc`)
//! 3. **Enum**: Runtime selection (e.g., `AesCbc::Aes128`)
//! 4. **Cipher-specific trait**: Polymorphism (e.g., `AesCbcCipher`)
//!
//! ## Example (ZST)
//!
//! ```
//! use crypto::{Aes128Cbc, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
//!
//! let cipher = Aes128Cbc;
//! let key = [0u8; 16];
//! let iv = InitializationVector::from(vec![0u8; 16]);
//! let plaintext = Plaintext::from("Hello, World! (ZST)");
//!
//! let ciphertext = cipher.encrypt(&plaintext, &key, &iv).unwrap();
//! let decrypted = cipher.decrypt(&ciphertext, &key, &iv).unwrap();
//! assert_eq!(plaintext, decrypted);
//! ```
//!
//! ## Example (Enum)
//!
//! ```
//! use crypto::{AesCbc, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
//!
//! let cipher = AesCbc::Aes256;
//! let key = [0u8; 32];
//! let iv = InitializationVector::from(vec![0u8; 16]);
//! let plaintext = Plaintext::from("Hello, World! (Enum)");
//!
//! let ciphertext = cipher.encrypt(&plaintext, &key, &iv).unwrap();
//! let decrypted = cipher.decrypt(&ciphertext, &key, &iv).unwrap();
//! assert_eq!(plaintext, decrypted);
//! ```
//!
//! ## Example (Trait)
//!
//! ```
//! use crypto::{AesCbc, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
//!
//! let cipher = AesCbc::Aes256;
//! let key = [0u8; 32];
//! let iv = InitializationVector::from(vec![0u8; 16]);
//! let plaintext = Plaintext::from("Hello, World! (Cipher-specific Trait)");
//!
//! let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv).unwrap();
//! let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();
//! assert_eq!(plaintext, decrypted);
//! ```

use zeroize::Zeroize;

mod symmetric;

pub use symmetric::{
    Aes128Cbc, Aes128Ctr, Aes256Cbc, Aes256Ctr, AesCbc, AesCbcCipher, AesCtr, AesCtrCipher,
    ChaCha20, ChaCha20Cipher, SymmetricCipher, SymmetricCipherError, SymmetricDecryptor,
    SymmetricEncryptor,
};

/// Plaintext data.
///
/// # Security
///
/// Implements [`Zeroize`] to clear memory on [`Drop`].
/// Limit scope to ensure early cleanup when handling sensitive data.
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct Plaintext(Vec<u8>);

impl Plaintext {
    /// Returns the data as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consumes self and returns the underlying bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Plaintext {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for Plaintext {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl From<String> for Plaintext {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<&str> for Plaintext {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

/// Ciphertext data.
///
/// # Security
///
/// Ciphertext is not secret, but implements [`Zeroize`] as a precaution.
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct Ciphertext(Vec<u8>);

impl Ciphertext {
    /// Returns the data as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consumes self and returns the underlying bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Ciphertext {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for Ciphertext {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

/// Initialization Vector (IV).
///
/// # Security
///
/// **Never reuse an IV with the same key.**
/// Reusing IV+key pairs can leak plaintext information.
/// For CBC mode, generate a random IV and use it only once.
///
/// IV is not secret, but implements [`Zeroize`] as a precaution.
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct InitializationVector(Vec<u8>);

impl InitializationVector {
    /// Returns the IV as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for InitializationVector {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for InitializationVector {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}
