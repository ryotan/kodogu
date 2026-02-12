use aes_gcm::aead;
use std::array::TryFromSliceError;
use suzunari_error::suzunari_error;

/// AEAD cipher algorithm identifier for error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AeadCiphers {
    Aes128Gcm,
    Aes256Gcm,
    ChaCha20Poly1305,
    XChaCha20Poly1305,
    Aes256GcmSiv,
}

/// AEAD cipher error.
#[suzunari_error]
#[snafu(visibility(pub), context(suffix(Error)))]
pub enum AeadError {
    /// Invalid key length
    #[snafu(display("{cipher:?}: invalid key length {actual} bytes (expected {expected} bytes)"))]
    InvalidKeyLength {
        cipher: AeadCiphers,
        expected: usize,
        actual: usize,
        source: TryFromSliceError,
    },

    /// Invalid nonce length
    #[snafu(display(
        "{cipher:?}: invalid nonce length {actual} bytes (expected {expected} bytes)"
    ))]
    InvalidNonceLength {
        cipher: AeadCiphers,
        expected: usize,
        actual: usize,
        source: TryFromSliceError,
    },

    /// Authentication failed during decryption
    #[snafu(display("{cipher:?}: authentication failed"))]
    AuthenticationFailed { cipher: AeadCiphers, source: aead::Error },

    /// Invalid authentication tag length
    #[snafu(display("{cipher:?}: invalid tag length {actual} bytes (expected {expected} bytes)"))]
    InvalidTagLength {
        cipher: AeadCiphers,
        expected: usize,
        actual: usize,
        source: TryFromSliceError,
    },

    /// Ciphertext too short for decryption
    #[snafu(display("{cipher:?}: ciphertext too short ({size} bytes, minimum {min_size} bytes)"))]
    CiphertextTooShort { cipher: AeadCiphers, size: usize, min_size: usize },
}
