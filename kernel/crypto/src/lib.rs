//! # KoDoGu Crypto Library
//!
//! Cryptographic utilities for symmetric encryption, AEAD, MACs, and key derivation.
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
//! use crypto::{Aes128Cbc, BlockEncryptor, BlockDecryptor, Plaintext, InitializationVector};
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
//! use crypto::{AesCbc, BlockEncryptor, BlockDecryptor, Plaintext, InitializationVector};
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
//! use crypto::{AesCbc, BlockEncryptor, BlockDecryptor, Plaintext, InitializationVector};
//!
//! let cipher = AesCbc::Aes256;
//! let key = [0u8; 32];
//! let iv = InitializationVector::from(vec![0u8; 16]);
//! let plaintext = Plaintext::from("Hello, World! (Trait)");
//!
//! let ciphertext = BlockEncryptor::encrypt(&cipher, &plaintext, &key, &iv).unwrap();
//! let decrypted = BlockDecryptor::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();
//! assert_eq!(plaintext, decrypted);
//! ```

use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// Implements `Debug` that hides the bytes, and constant-time `PartialEq` and `Eq`, for a secret type with `as_bytes`.
macro_rules! impl_secret_traits {
    ($type:ident) => {
        impl core::fmt::Debug for $type {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_tuple(stringify!($type)).field(&"***").finish()
            }
        }

        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                subtle::ConstantTimeEq::ct_eq(self.as_bytes(), other.as_bytes()).into()
            }
        }

        impl Eq for $type {}
    };
}

mod aead;
mod kdf;
mod mac;
mod symmetric;

pub use aead::{
    AdditionalData, AeadCipher, AeadCiphers, AeadCiphertext, AeadDecryptor, AeadEncryptor,
    AeadError, Aes128Gcm, Aes128GcmKey, Aes256Gcm, Aes256GcmKey, Aes256GcmSiv, AesGcm,
    AesGcmCipher, AesGcmSivCipher, AuthenticationTag, ChaCha20Poly1305, ChaCha20Poly1305Cipher,
    ChaCha20Poly1305Family, ChaCha20Poly1305Key, XChaCha20Poly1305,
};
pub use kdf::{
    Argon2Params, Argon2ParamsError, Argon2id, BcryptPbkdf, BcryptPbkdfParams,
    BcryptPbkdfParamsError, ConcatKdfAlgorithm, ConcatKdfDeriver, ConcatKdfSha256, ConcatKdfSha384,
    DerivedKey, HkdfAlgorithm, HkdfDeriver, HkdfInput, HkdfSha256, HkdfSha384, KdfAlgorithm,
    KdfError, KeyDeriver, OutputLength, OutputLengthError, PasswordDeriver, Pbkdf2Params,
    Pbkdf2ParamsError, Pbkdf2Sha256, Scrypt, ScryptParams, ScryptParamsError, X963KdfAlgorithm,
    X963KdfDeriver, X963KdfSha256, X963KdfSha384,
};
pub use mac::{
    CmacAes, CmacAes128, CmacAes128Key, CmacAes256, CmacAes256Key, CmacAesCipher, Gmac, GmacCipher,
    GmacKeyWithNonce, HmacAlgorithm, HmacCipher, HmacSha256, HmacSha256Key, HmacSha384,
    HmacSha384Key, HmacSha512, HmacSha512Key, Mac, MacAlgorithms, MacError, MacGenerator, MacTag,
    MacVerifier, Poly1305Cipher, Poly1305Key, Poly1305Mac,
};
pub use symmetric::{
    Aes128Cbc, Aes128Ctr, Aes256Cbc, Aes256Ctr, AesCbc, AesCbcCipher, AesCtr, AesCtrCipher,
    BlockCipher, BlockDecryptor, BlockEncryptor, ChaCha20, ChaCha20Cipher, StreamCipher,
    StreamDecryptor, StreamEncryptor, SymmetricCipherError,
};

/// Plaintext data.
///
/// # Security
///
/// Automatically zeroized on drop ([`ZeroizeOnDrop`]).
/// Equality comparison is constant-time to prevent timing attacks.
/// Limit scope to ensure early cleanup when handling sensitive data.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Plaintext(Vec<u8>);

impl_secret_traits!(Plaintext);

impl Plaintext {
    /// Returns the data as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consumes self and returns the underlying bytes.
    ///
    /// The returned [`Zeroizing`] wrapper ensures the bytes are cleared on drop.
    pub fn into_bytes(mut self) -> Zeroizing<Vec<u8>> {
        Zeroizing::new(std::mem::take(&mut self.0))
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

/// Nonce (Number used once).
///
/// # Security
///
/// **Never reuse nonce with the same key.**
/// Nonce reuse can completely break security guarantees for AEAD and stream ciphers.
///
/// - AEAD (AES-GCM, ChaCha20-Poly1305): 12 bytes (96 bits)
/// - XChaCha20-Poly1305: 24 bytes (192 bits)
/// - GMAC: 12 bytes (96 bits)
///
/// Unlike IV, nonce does not need to be unpredictable—only unique.
/// A counter is often used for nonce generation.
///
/// Nonce is not secret, but implements [`Zeroize`] as a precaution.
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct Nonce(Vec<u8>);

impl Nonce {
    /// Returns the nonce as a byte slice.
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

impl From<Vec<u8>> for Nonce {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for Nonce {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Secret Type Tests
    // ========================================

    #[test]
    fn test_secret_types_zeroize_on_drop() {
        use core::mem::needs_drop;
        assert!(needs_drop::<Plaintext>());
        assert!(needs_drop::<DerivedKey>());
        assert!(needs_drop::<Aes128GcmKey>());
        assert!(needs_drop::<Aes256GcmKey>());
        assert!(needs_drop::<ChaCha20Poly1305Key>());
        assert!(needs_drop::<HmacSha256Key>());
        assert!(needs_drop::<HmacSha384Key>());
        assert!(needs_drop::<HmacSha512Key>());
        assert!(needs_drop::<CmacAes128Key>());
        assert!(needs_drop::<CmacAes256Key>());
        assert!(needs_drop::<Poly1305Key>());
    }

    #[test]
    fn test_secret_types_debug_hides_bytes() {
        assert_eq!(format!("{:?}", Plaintext::from("secret")), r#"Plaintext("***")"#);
        assert_eq!(format!("{:?}", Aes128GcmKey::from([1; 16])), r#"Aes128GcmKey("***")"#);
        assert_eq!(format!("{:?}", Aes256GcmKey::from([1; 32])), r#"Aes256GcmKey("***")"#);
        assert_eq!(
            format!("{:?}", ChaCha20Poly1305Key::from([1; 32])),
            r#"ChaCha20Poly1305Key("***")"#
        );
        assert_eq!(format!("{:?}", HmacSha256Key::from([1; 32])), r#"HmacSha256Key("***")"#);
        assert_eq!(format!("{:?}", HmacSha384Key::from([1; 48])), r#"HmacSha384Key("***")"#);
        assert_eq!(format!("{:?}", HmacSha512Key::from([1; 64])), r#"HmacSha512Key("***")"#);
        assert_eq!(format!("{:?}", CmacAes128Key::from([1; 16])), r#"CmacAes128Key("***")"#);
        assert_eq!(format!("{:?}", CmacAes256Key::from([1; 32])), r#"CmacAes256Key("***")"#);
        assert_eq!(format!("{:?}", Poly1305Key::from([1; 32])), r#"Poly1305Key("***")"#);
    }

    #[test]
    fn test_secret_types_eq_compares_bytes() {
        assert_eq!(Plaintext::from("secret"), Plaintext::from("secret"));
        assert_ne!(Plaintext::from("secret"), Plaintext::from("secreT"));
        assert_ne!(Plaintext::from("secret"), Plaintext::from("secret!"));
        assert_eq!(Aes256GcmKey::from([1; 32]), Aes256GcmKey::from([1; 32]));
        assert_ne!(Aes256GcmKey::from([1; 32]), Aes256GcmKey::from([2; 32]));
    }

    #[test]
    fn test_plaintext_into_bytes() {
        assert_eq!(*Plaintext::from("secret").into_bytes(), b"secret".to_vec());
    }

    // ========================================
    // Nonce Tests
    // ========================================

    #[test]
    fn test_nonce_from_vec() {
        let data = vec![1, 2, 3, 4];
        let nonce = Nonce::from(data.clone());
        assert_eq!(nonce.as_bytes(), &data);
    }

    #[test]
    fn test_nonce_from_slice() {
        let data = [1, 2, 3, 4];
        let nonce = Nonce::from(data.as_slice());
        assert_eq!(nonce.as_bytes(), &data);
    }

    #[test]
    fn test_nonce_len() {
        let nonce = Nonce::from(vec![0u8; 12]);
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_nonce_is_empty() {
        assert!(Nonce::from(vec![]).is_empty());
        assert!(!Nonce::from(vec![1]).is_empty());
    }
}
