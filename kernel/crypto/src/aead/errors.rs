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
#[suzu(visibility(pub), context(suffix(Error)))]
pub enum AeadError {
    /// Invalid key length
    #[suzu(display("{cipher:?}: invalid key length {actual} bytes (expected {expected} bytes)"))]
    InvalidKeyLength {
        cipher: AeadCiphers,
        expected: usize,
        actual: usize,
        #[suzu(from)]
        source: TryFromSliceError,
    },

    /// Invalid nonce length
    #[suzu(display("{cipher:?}: invalid nonce length {actual} bytes (expected {expected} bytes)"))]
    InvalidNonceLength {
        cipher: AeadCiphers,
        expected: usize,
        actual: usize,
        #[suzu(from)]
        source: TryFromSliceError,
    },

    /// Authentication failed during decryption
    #[suzu(display("{cipher:?}: authentication failed"))]
    AuthenticationFailed {
        cipher: AeadCiphers,
        #[suzu(from)]
        source: aead::Error,
    },

    /// Invalid authentication tag length
    #[suzu(display("{cipher:?}: invalid tag length {actual} bytes (expected {expected} bytes)"))]
    InvalidTagLength {
        cipher: AeadCiphers,
        expected: usize,
        actual: usize,
        #[suzu(from)]
        source: TryFromSliceError,
    },

    /// Ciphertext too short for decryption
    #[suzu(display("{cipher:?}: ciphertext too short ({size} bytes, minimum {min_size} bytes)"))]
    CiphertextTooShort { cipher: AeadCiphers, size: usize, min_size: usize },
}
