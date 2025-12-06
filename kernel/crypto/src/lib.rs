//! KoDoGu Crypto - Cryptographic operations library
//!
//! This library provides cryptographic functionality following a simple,
//! codec-inspired API design with two complementary APIs:
//! - ZST types: `Aes256Gcm`, `Rsa2048`, etc. (compile-time dispatch)
//! - Enums: `AesGcm`, `RsaKeyGenerator`, etc. (runtime selection)
//!
//! ## Supported Operations
//!
//! ### Symmetric Encryption
//! - AES-128-GCM, AES-256-GCM
//!
//! ### Asymmetric Encryption
//! - RSA hybrid encryption (RSA + AES-256-GCM)
//!
//! ### Key Generation
//! - RSA (2048-bit, 4096-bit)
//! - ECDSA (P-256, P-384)
//! - Ed25519
//! - Random symmetric keys (AES-128, AES-256)
//!
//! ### Key Derivation
//! - PBKDF2-HMAC-SHA256

use aes_gcm::Error;
use aes_gcm::aes::cipher::InvalidLength;
use subtle::ConstantTimeEq;
use suzunari_error::suzunari_error;
use zeroize::{Zeroize, ZeroizeOnDrop};

mod aes;
mod keygen;
mod pbkdf2;
mod rsa;

// Re-exports
pub use aes::{Aes128Gcm, Aes128Key, Aes256Gcm, Aes256Key, AesGcm, AesKey};
pub use keygen::{
    EcdsaKeyGenerator, EcdsaP256, EcdsaP384, Ed25519KeyGen, KeyGenerationError, RandomAes128Key,
    RandomAes256Key, RandomAesKey, Rsa2048, Rsa4096, RsaKeyGenerator,
};
pub use pbkdf2::{KeyDerivationError, Pbkdf2, Pbkdf2Aes128, Pbkdf2Aes256};
pub use rsa::{RsaHybrid, RsaHybrid2048, RsaHybrid4096, RsaHybridError};

//
// Value Objects
//

/// Plaintext data to be encrypted
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plaintext(Vec<u8>);

impl Plaintext {
    /// Get plaintext as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Plaintext {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
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

/// Encrypted ciphertext
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(Vec<u8>);

impl Ciphertext {
    /// Get ciphertext as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Ciphertext {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

/// Public key for asymmetric encryption
///
/// PEM format is a text-based encoding defined in RFC 7468.
/// It consists of Base64-encoded binary data with header/footer lines.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PublicKey(String);

impl PublicKey {
    /// Create public key from PEM string
    ///
    /// PEM format example:
    /// ```text
    /// -----BEGIN PUBLIC KEY-----
    /// MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...
    /// -----END PUBLIC KEY-----
    /// ```
    pub fn from_pem(pem: String) -> Self {
        Self(pem)
    }

    /// Get public key as PEM string slice
    pub fn as_pem(&self) -> &str {
        &self.0
    }
}

impl From<String> for PublicKey {
    fn from(pem: String) -> Self {
        Self(pem)
    }
}

impl From<&str> for PublicKey {
    fn from(pem: &str) -> Self {
        Self(pem.to_string())
    }
}

impl std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicKey([{} chars])", self.0.len())
    }
}

/// Private key for asymmetric encryption
///
/// PEM format is a text-based encoding defined in RFC 7468.
/// The private key data is automatically zeroized when dropped for security.
///
/// # Security
///
/// - Uses constant-time comparison to prevent timing attacks
/// - Automatically zeroized on drop
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct PrivateKey(String);

impl PrivateKey {
    /// Create private key from PEM string
    ///
    /// PEM format example:
    /// ```text
    /// -----BEGIN PRIVATE KEY-----
    /// MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC...
    /// -----END PRIVATE KEY-----
    /// ```
    pub fn from_pem(pem: String) -> Self {
        Self(pem)
    }

    /// Get private key as PEM string slice
    pub fn as_pem(&self) -> &str {
        &self.0
    }
}

impl From<String> for PrivateKey {
    fn from(pem: String) -> Self {
        Self(pem)
    }
}

impl From<&str> for PrivateKey {
    fn from(pem: &str) -> Self {
        Self(pem.to_string())
    }
}

impl PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_bytes().ct_eq(other.0.as_bytes()).into()
    }
}

impl Eq for PrivateKey {}

impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrivateKey([REDACTED])")
    }
}

/// Asymmetric key pair (public + private)
#[derive(PartialEq, Eq)]
pub struct KeyPair {
    /// Public key
    pub public_key: PublicKey,
    /// Private key
    pub private_key: PrivateKey,
}

impl KeyPair {
    /// Create a new key pair
    pub fn new(public_key: PublicKey, private_key: PrivateKey) -> Self {
        Self { public_key, private_key }
    }
}

impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("public_key", &self.public_key)
            .field("private_key", &self.private_key)
            .finish()
    }
}

//
// Error Types
//

/// Errors that can occur during encryption and decryption operations
#[suzunari_error]
#[snafu(visibility(pub))]
pub enum CipherError {
    /// AES cipher creation failed
    #[snafu(display("AES cipher creation failed"))]
    AesCipherCreation { source: InvalidLength },

    /// AES encryption failed
    #[snafu(display("AES encryption failed"))]
    AesEncryption { source: Error },

    /// AES decryption failed
    #[snafu(display("AES decryption failed"))]
    AesDecryption { source: Error },

    /// Ciphertext too short
    #[snafu(display("Ciphertext too short: {size} bytes"))]
    CiphertextTooShort { size: usize },
}
