//! KoDoGu Crypto - Cryptographic operations library
//!
//! This library provides cryptographic functionality following a simple,
//! codec-inspired API design with three complementary APIs:
//! - ZST types: `Aes256Gcm`, `Rsa2048`, etc. (compile-time dispatch)
//! - Enums: `AesGcm`, `RsaKeyGenerator`, etc. (runtime selection)
//! - Static methods: `AesGcm::aes256()`, `RsaKeyGenerator::rsa2048()`, etc.
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
use suzunari_error::suzunari_error;
use zeroize::{Zeroize, ZeroizeOnDrop};

// Implementation modules
pub mod aes;
pub mod keygen;
pub mod pbkdf2;
pub mod rsa;

// Re-exports
pub use aes::{Aes128Gcm, Aes256Gcm, AesGcm};
pub use keygen::{
    EcdsaKeyGenerator, EcdsaP256, EcdsaP384, Ed25519KeyGen, KeyGenerationError, RandomAes128,
    RandomAes256, RandomKeyGenerator, Rsa2048, Rsa4096, RsaKeyGenerator,
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
    /// Create plaintext from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Create plaintext from string
    pub fn from_string(s: String) -> Self {
        Self(s.into_bytes())
    }

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

/// Encrypted ciphertext
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(Vec<u8>);

impl Ciphertext {
    /// Create ciphertext from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

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

/// Symmetric encryption key
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub enum SymmetricKey {
    /// AES-128 key (16 bytes)
    Aes128(Box<[u8; 16]>),
    /// AES-256 key (32 bytes)
    Aes256(Box<[u8; 32]>),
}

impl SymmetricKey {
    /// Create AES-128 key from bytes
    pub fn aes128(key: [u8; 16]) -> Self {
        Self::Aes128(Box::new(key))
    }

    /// Create AES-256 key from bytes
    pub fn aes256(key: [u8; 32]) -> Self {
        Self::Aes256(Box::new(key))
    }
}

impl std::fmt::Debug for SymmetricKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Aes128(_) => f.debug_tuple("Aes128").field(&"[REDACTED]").finish(),
            Self::Aes256(_) => f.debug_tuple("Aes256").field(&"[REDACTED]").finish(),
        }
    }
}

/// Public key for asymmetric encryption
#[derive(Clone, PartialEq, Eq)]
pub struct PublicKey(Vec<u8>);

impl PublicKey {
    /// Create public key from PEM bytes
    pub fn from_pem(pem: Vec<u8>) -> Self {
        Self(pem)
    }

    /// Get public key as PEM bytes
    pub fn as_pem(&self) -> &[u8] {
        &self.0
    }

    /// Convert to PEM bytes
    pub fn into_pem(self) -> Vec<u8> {
        self.0
    }
}

impl std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicKey([{} bytes])", self.0.len())
    }
}

/// Private key for asymmetric encryption
#[derive(Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct PrivateKey(Vec<u8>);

impl PrivateKey {
    /// Create private key from PEM bytes
    pub fn from_pem(pem: Vec<u8>) -> Self {
        Self(pem)
    }

    /// Get private key as PEM bytes
    pub fn as_pem(&self) -> &[u8] {
        &self.0
    }

    /// Convert to PEM bytes
    pub fn into_pem(self) -> Vec<u8> {
        self.0.clone()
    }
}

impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrivateKey([REDACTED])")
    }
}

/// Asymmetric key pair (public + private)
#[derive(Clone, PartialEq, Eq)]
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
    /// Wrong key type for the cipher
    #[snafu(display("Wrong key type: expected {expected}, got {got}"))]
    WrongKeyType { expected: String, got: String },

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
