//! KoDoGu Crypto - Cryptographic operations library
//!
//! This library provides cryptographic functionality including:
//! - Key pair generation (RSA, ECDSA, Ed25519)
//! - Symmetric encryption (AES-128-GCM, AES-256-GCM)
//! - Asymmetric encryption (RSA hybrid encryption only)
//! - Key derivation (PBKDF2-HMAC-SHA256)
//! - PEM format key encoding
//!
//! ## Supported Algorithms
//!
//! ### Key Generation
//! - RSA (2048-bit, 4096-bit)
//! - ECDSA (P-256, P-384)
//! - Ed25519
//!
//! ### Encryption/Decryption
//! - Symmetric: AES-128-GCM, AES-256-GCM
//! - Asymmetric: RSA hybrid encryption (RSA + AES-256-GCM)
//!   - Note: ECDSA and Ed25519 are for signatures only, not encryption
//!
//! ### Key Derivation
//! - PBKDF2-HMAC-SHA256 (100,000 iterations default)

pub mod adapters;
mod domain;

// Re-export public API at crate root
pub use domain::{
    CryptoError,
    algorithms::{AsymmetricAlgorithm, EcdsaCurve, RsaKeySize, SymmetricAlgorithm},
    ciphertext::Ciphertext,
    keys::{KeyPair, PrivateKey, PublicKey, SymmetricKey},
    plaintext::Plaintext,
    providers::{Decryptor, Encryptor, KeyDeriver, KeyGenerator},
};
