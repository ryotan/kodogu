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
//! use crypto::{Aes256Gcm, Aes256Key, AesGcm, Plaintext};
//!
//! let plaintext = Plaintext::from_string("Hello, World!".to_string());
//!
//! // ZST type (compile-time dispatch) - separate key management
//! let key = Aes256Key::from([0u8; 32]);
//! let ciphertext = Aes256Gcm.encrypt(&plaintext, &key)?;
//! let decrypted = Aes256Gcm.decrypt(&ciphertext, &key)?;
//!
//! // Enum (runtime selection) - key embedded
//! let cipher = AesGcm::aes256([0u8; 32]);
//! let ciphertext = cipher.encrypt(&plaintext)?;
//! let decrypted = cipher.decrypt(&ciphertext)?;
//! ```

use aes_gcm::{
    Aes128Gcm as Aes128GcmImpl, Aes256Gcm as Aes256GcmImpl, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use rand::RngCore;
use snafu::ResultExt;
use subtle::ConstantTimeEq;

use crate::{
    AesDecryptionSnafu, AesEncryptionSnafu, CipherError, Ciphertext, CiphertextTooShortSnafu,
    Plaintext,
};
use snafu::ensure;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

const NONCE_SIZE: usize = 12; // 96 bits for AES-GCM

/// AES-256 encryption key (32 bytes)
///
/// This type ensures type safety by preventing accidental use of wrong key sizes.
/// The key data is automatically zeroized when dropped for security.
///
/// # Security
///
/// - Uses constant-time comparison to prevent timing attacks
/// - Automatically zeroized on drop
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Aes256Key(Box<[u8; 32]>);

impl Aes256Key {
    /// Convert the key into a byte array
    ///
    /// Consumes the key and returns the underlying 32-byte array wrapped in `Zeroizing`.
    /// The returned value will be automatically zeroized when dropped.
    ///
    /// # Security
    ///
    /// Returns `Zeroizing<[u8; 32]>` to ensure the key bytes are zeroized after use.
    pub fn into_bytes(self) -> Zeroizing<[u8; 32]> {
        Zeroizing::new(*self.0)
    }
}

impl From<[u8; 32]> for Aes256Key {
    fn from(key: [u8; 32]) -> Self {
        Self(Box::new(key))
    }
}

impl PartialEq for Aes256Key {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_slice().ct_eq(other.0.as_slice()).into()
    }
}

impl Eq for Aes256Key {}

impl std::fmt::Debug for Aes256Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Aes256").field(&"[REDACTED]").finish()
    }
}

/// AES-128 encryption key (16 bytes)
///
/// This type ensures type safety by preventing accidental use of wrong key sizes.
/// The key data is automatically zeroized when dropped for security.
///
/// # Security
///
/// - Uses constant-time comparison to prevent timing attacks
/// - Automatically zeroized on drop
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Aes128Key(Box<[u8; 16]>);

impl Aes128Key {
    /// Convert the key into a byte array
    ///
    /// Consumes the key and returns the underlying 16-byte array wrapped in `Zeroizing`.
    /// The returned value will be automatically zeroized when dropped.
    ///
    /// # Security
    ///
    /// Returns `Zeroizing<[u8; 16]>` to ensure the key bytes are zeroized after use.
    pub fn into_bytes(self) -> Zeroizing<[u8; 16]> {
        Zeroizing::new(*self.0)
    }
}

impl From<[u8; 16]> for Aes128Key {
    fn from(key: [u8; 16]) -> Self {
        Self(Box::new(key))
    }
}

impl PartialEq for Aes128Key {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_slice().ct_eq(other.0.as_slice()).into()
    }
}

impl Eq for Aes128Key {}

impl std::fmt::Debug for Aes128Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Aes128").field(&"[REDACTED]").finish()
    }
}

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
    /// * `key` - AES-256 key (32 bytes)
    ///
    /// # Returns
    ///
    /// Ciphertext in format: `[nonce (12 bytes) | ciphertext + auth_tag]`
    pub fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Aes256Key,
    ) -> Result<Ciphertext, CipherError> {
        // Check key type
        let key_bytes = key.0.as_ref();

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher
        let cipher = Aes256GcmImpl::new(key_bytes.into());

        // Encrypt
        let ciphertext_bytes =
            cipher.encrypt(nonce, plaintext.as_bytes()).context(AesEncryptionSnafu)?;

        // Combine nonce + ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext_bytes.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext_bytes);

        Ok(Ciphertext::from(result))
    }

    /// Decrypt ciphertext with AES-256-GCM
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Data to decrypt (must include nonce)
    /// * `key` - AES-256 key (32 bytes)
    pub fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Aes256Key,
    ) -> Result<Plaintext, CipherError> {
        // Check key type
        let key_bytes = key.0.as_ref();

        let bytes = ciphertext.as_bytes();

        // Check minimum length (nonce + at least some ciphertext)
        ensure!(bytes.len() >= NONCE_SIZE, CiphertextTooShortSnafu { size: bytes.len() });

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext_bytes) = bytes.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Create cipher
        let cipher = Aes256GcmImpl::new(key_bytes.into());

        // Decrypt
        let plaintext_bytes =
            cipher.decrypt(nonce, ciphertext_bytes).context(AesDecryptionSnafu)?;

        Ok(Plaintext::from(plaintext_bytes))
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
    /// * `key` - AES-128 key (16 bytes)
    pub fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Aes128Key,
    ) -> Result<Ciphertext, CipherError> {
        // Check key type
        let key_bytes = key.0.as_ref();

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher
        let cipher = Aes128GcmImpl::new(key_bytes.into());

        // Encrypt
        let ciphertext_bytes =
            cipher.encrypt(nonce, plaintext.as_bytes()).context(AesEncryptionSnafu)?;

        // Combine nonce + ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext_bytes.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext_bytes);

        Ok(Ciphertext::from(result))
    }

    /// Decrypt ciphertext with AES-128-GCM
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Data to decrypt (must include nonce)
    /// * `key` - AES-128 key (16 bytes)
    pub fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Aes128Key,
    ) -> Result<Plaintext, CipherError> {
        // Check key type
        let key_bytes = key.0.as_ref();

        let bytes = ciphertext.as_bytes();

        // Check minimum length
        ensure!(bytes.len() >= NONCE_SIZE, CiphertextTooShortSnafu { size: bytes.len() });

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext_bytes) = bytes.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Create cipher
        let cipher = Aes128GcmImpl::new(key_bytes.into());

        // Decrypt
        let plaintext_bytes =
            cipher.decrypt(nonce, ciphertext_bytes).context(AesDecryptionSnafu)?;

        Ok(Plaintext::from(plaintext_bytes))
    }
}

//
// Enum Wrapper
//

/// AES key enum for runtime key size selection
///
/// Wraps either an AES-128 or AES-256 key, enabling runtime dispatch
/// based on key size requirements.
/// The key data is automatically zeroized when dropped for security.
///
/// # Security
///
/// - Uses constant-time comparison to prevent timing attacks
/// - Automatically zeroized on drop
#[derive(Zeroize, ZeroizeOnDrop)]
pub enum AesKey {
    /// AES-128 key (16 bytes)
    Aes128(Aes128Key),
    /// AES-256 key (32 bytes)
    Aes256(Aes256Key),
}

impl PartialEq for AesKey {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AesKey::Aes128(a), AesKey::Aes128(b)) => a == b,
            (AesKey::Aes256(a), AesKey::Aes256(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for AesKey {}
impl From<Aes128Key> for AesKey {
    fn from(key: Aes128Key) -> Self {
        Self::Aes128(key)
    }
}
impl From<Aes256Key> for AesKey {
    fn from(key: Aes256Key) -> Self {
        Self::Aes256(key)
    }
}
impl From<[u8; 16]> for AesKey {
    fn from(key: [u8; 16]) -> Self {
        Self::Aes128(key.into())
    }
}
impl From<[u8; 32]> for AesKey {
    fn from(key: [u8; 32]) -> Self {
        Self::Aes256(key.into())
    }
}
impl std::fmt::Debug for AesKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AesKey::Aes128(key) => key.fmt(f),
            AesKey::Aes256(key) => key.fmt(f),
        }
    }
}

/// AES-GCM cipher with embedded key for runtime algorithm selection
///
/// This enum embeds the encryption key within the cipher instance,
/// allowing for convenient repeated encryption/decryption operations
/// without passing the key each time.
/// The embedded key data is automatically zeroized when dropped for security.
///
/// # Security
///
/// - Uses constant-time comparison to prevent timing attacks
/// - Automatically zeroized on drop
///
/// # Example
///
/// ```rust,ignore
/// let cipher = AesGcm::aes256([0u8; 32]);
/// let ct1 = cipher.encrypt(&plaintext1)?;
/// let ct2 = cipher.encrypt(&plaintext2)?;
/// ```
#[derive(Zeroize, ZeroizeOnDrop)]
pub enum AesGcm {
    /// AES-128-GCM cipher with embedded key
    Aes128(Aes128Key),
    /// AES-256-GCM cipher with embedded key
    Aes256(Aes256Key),
}

impl PartialEq for AesGcm {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AesGcm::Aes128(a), AesGcm::Aes128(b)) => a == b,
            (AesGcm::Aes256(a), AesGcm::Aes256(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for AesGcm {}
impl AesGcm {
    /// Create AES-256 key from bytes
    pub fn aes256(key: [u8; 32]) -> AesGcm {
        AesGcm::Aes256(Aes256Key::from(key))
    }

    /// Create AES-128 key from bytes
    pub fn aes128(key: [u8; 16]) -> AesGcm {
        AesGcm::Aes128(Aes128Key::from(key))
    }

    /// Encrypt plaintext
    ///
    /// Delegates to the appropriate ZST implementation based on enum variant.
    pub fn encrypt(&self, plaintext: &Plaintext) -> Result<Ciphertext, CipherError> {
        match self {
            AesGcm::Aes128(key) => Aes128Gcm.encrypt(plaintext, key),
            AesGcm::Aes256(key) => Aes256Gcm.encrypt(plaintext, key),
        }
    }

    /// Decrypt ciphertext
    ///
    /// Delegates to the appropriate ZST implementation based on enum variant.
    pub fn decrypt(&self, ciphertext: &Ciphertext) -> Result<Plaintext, CipherError> {
        match self {
            AesGcm::Aes128(key) => Aes128Gcm.decrypt(ciphertext, key),
            AesGcm::Aes256(key) => Aes256Gcm.decrypt(ciphertext, key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes256_round_trip() {
        let key = Aes256Key::from([42u8; 32]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = Aes256Gcm.encrypt(&plaintext, &key).unwrap();
        let decrypted = Aes256Gcm.decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes128_round_trip() {
        let key = Aes128Key::from([42u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = Aes128Gcm.encrypt(&plaintext, &key).unwrap();
        let decrypted = Aes128Gcm.decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_nonce_randomness() {
        let key = Aes256Key::from([42u8; 32]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext1 = Aes256Gcm.encrypt(&plaintext, &key).unwrap();
        let ciphertext2 = Aes256Gcm.encrypt(&plaintext, &key).unwrap();

        // Same plaintext encrypted twice should produce different ciphertexts
        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_enum_aes256() {
        let cipher = AesGcm::aes256([42u8; 32]);
        let plaintext = Plaintext::from("Test");

        let ciphertext = cipher.encrypt(&plaintext).unwrap();
        let decrypted = cipher.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_enum_aes128() {
        let cipher = AesGcm::aes128([42u8; 16]);
        let plaintext = Plaintext::from("Test");

        let ciphertext = cipher.encrypt(&plaintext).unwrap();
        let decrypted = cipher.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256_key_into_bytes() {
        let key_bytes = [42u8; 32];
        let key = Aes256Key::from(key_bytes);

        assert_eq!(*key.into_bytes(), key_bytes);
    }

    #[test]
    fn test_aes128_key_into_bytes() {
        let key_bytes = [42u8; 16];
        let key = Aes128Key::from(key_bytes);

        assert_eq!(*key.into_bytes(), key_bytes);
    }

    #[test]
    fn test_aes_key_from_aes256() {
        let key = Aes256Key::from([42u8; 32]);
        let aes_key: AesKey = key.into();

        match aes_key {
            AesKey::Aes256(_) => (), // Success
            _ => panic!("Expected AesKey::Aes256"),
        }
    }

    #[test]
    fn test_aes_key_from_aes128() {
        let key = Aes128Key::from([42u8; 16]);
        let aes_key: AesKey = key.into();

        match aes_key {
            AesKey::Aes128(_) => (), // Success
            _ => panic!("Expected AesKey::Aes128"),
        }
    }

    #[test]
    fn test_aes_key_from_bytes_256() {
        let aes_key: AesKey = [42u8; 32].into();

        match aes_key {
            AesKey::Aes256(_) => (), // Success
            _ => panic!("Expected AesKey::Aes256"),
        }
    }

    #[test]
    fn test_aes_key_from_bytes_128() {
        let aes_key: AesKey = [42u8; 16].into();

        match aes_key {
            AesKey::Aes128(_) => (), // Success
            _ => panic!("Expected AesKey::Aes128"),
        }
    }
}
