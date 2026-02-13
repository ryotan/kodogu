use crate::{Ciphertext, InitializationVector, Nonce, Plaintext};

/// Block cipher encryption operation.
///
/// Abstracts block cipher encryption (AES-CBC, etc.).
/// The caller must manage IV (initialization vector).
pub trait BlockEncryptor {
    /// Key type for encryption.
    type Key: ?Sized;

    /// Error type for encryption.
    type Error;

    /// Encrypts `plaintext` with the given `key` and `iv`.
    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, Self::Error>;
}

/// Block cipher decryption operation.
///
/// Abstracts block cipher decryption (AES-CBC, etc.).
pub trait BlockDecryptor {
    /// Key type for decryption.
    type Key: ?Sized;

    /// Error type for decryption.
    type Error;

    /// Decrypts `ciphertext` with the given `key` and `iv`.
    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Plaintext, Self::Error>;
}

/// Combined block cipher encryption and decryption.
///
/// Marker trait for types implementing both [`BlockEncryptor`] and [`BlockDecryptor`].
/// Guarantees the same key and error types for both operations.
pub trait BlockCipher:
    BlockEncryptor<Key = <Self as BlockCipher>::Key, Error = <Self as BlockCipher>::Error>
    + BlockDecryptor<Key = <Self as BlockCipher>::Key, Error = <Self as BlockCipher>::Error>
{
    /// Key type for encryption/decryption.
    type Key: ?Sized;

    /// Error type for encryption/decryption.
    type Error;
}

/// Stream cipher encryption operation.
///
/// Abstracts stream cipher encryption (AES-CTR, ChaCha20, etc.).
/// Stream ciphers use nonce (number used once) instead of IV.
/// The caller must ensure nonce uniqueness per key.
///
/// # Security Warning
///
/// **Never reuse nonce with the same key.**
/// Nonce reuse completely breaks security for stream ciphers.
pub trait StreamEncryptor {
    /// Key type for encryption.
    type Key: ?Sized;

    /// Error type for encryption.
    type Error;

    /// Encrypts `plaintext` with the given `key` and `nonce`.
    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
    ) -> Result<Ciphertext, Self::Error>;
}

/// Stream cipher decryption operation.
///
/// Abstracts stream cipher decryption (AES-CTR, ChaCha20, etc.).
pub trait StreamDecryptor {
    /// Key type for decryption.
    type Key: ?Sized;

    /// Error type for decryption.
    type Error;

    /// Decrypts `ciphertext` with the given `key` and `nonce`.
    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        nonce: &Nonce,
    ) -> Result<Plaintext, Self::Error>;
}

/// Combined stream cipher encryption and decryption.
///
/// Marker trait for types implementing both [`StreamEncryptor`] and [`StreamDecryptor`].
/// Guarantees the same key and error types for both operations.
pub trait StreamCipher:
    StreamEncryptor<Key = <Self as StreamCipher>::Key, Error = <Self as StreamCipher>::Error>
    + StreamDecryptor<Key = <Self as StreamCipher>::Key, Error = <Self as StreamCipher>::Error>
{
    /// Key type for encryption/decryption.
    type Key: ?Sized;

    /// Error type for encryption/decryption.
    type Error;
}
