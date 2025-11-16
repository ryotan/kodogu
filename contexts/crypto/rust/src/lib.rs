//! KoDoGu Crypto - Cryptographic operations library
//!
//! This library provides cryptographic functionality including:
//! - Key pair generation (RSA, ECDSA, Ed25519)
//! - Symmetric encryption (AES-128/256)
//! - Asymmetric encryption (RSA hybrid)
//! - Key management (import/export)

pub mod adapters;
mod domain;
mod usecases;

// Re-export public API at crate root
pub use domain::{
    CryptoError,
    algorithms::{AsymmetricAlgorithm, EcdsaCurve, RsaKeySize, SymmetricAlgorithm},
    ciphertext::Ciphertext,
    keys::{KeyPair, PrivateKey, PublicKey, SymmetricKey},
    plaintext::Plaintext,
    providers::{Decryptor, Encryptor, KeyDeriver, KeyGenerator},
};
