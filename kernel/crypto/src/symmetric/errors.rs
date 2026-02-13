use cbc::cipher::block_padding::UnpadError;
use std::array::TryFromSliceError;
use suzunari_error::suzunari_error;

/// Cipher algorithm identifier for error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetricCiphers {
    Aes128Cbc,
    Aes256Cbc,
    Aes128Ctr,
    Aes256Ctr,
    ChaCha20,
}

/// Symmetric cipher error.
#[suzunari_error]
#[snafu(visibility(pub), context(suffix(Error)))]
pub enum SymmetricCipherError {
    /// Invalid key length
    #[snafu(display("{cipher:?}: invalid key length {actual} bytes (expected {expected} bytes)"))]
    InvalidKeyLength {
        cipher: SymmetricCiphers,
        expected: usize,
        actual: usize,
        source: TryFromSliceError,
    },

    /// Invalid IV length
    #[snafu(display("{cipher:?}: invalid IV length {actual} bytes (expected {expected} bytes)"))]
    InvalidIvLength {
        cipher: SymmetricCiphers,
        expected: usize,
        actual: usize,
        source: TryFromSliceError,
    },

    /// Invalid nonce length
    #[snafu(display(
        "{cipher:?}: invalid nonce length {actual} bytes (expected {expected} bytes)"
    ))]
    InvalidNonceLength {
        cipher: SymmetricCiphers,
        expected: usize,
        actual: usize,
        source: TryFromSliceError,
    },

    /// Padding error during encryption or decryption
    #[snafu(display("{cipher:?}: padding error"))]
    Padding { cipher: SymmetricCiphers, source: UnpadError },

    /// Ciphertext is too short for decryption
    #[snafu(display("{cipher:?}: ciphertext too short ({size} bytes)"))]
    CiphertextTooShort { cipher: SymmetricCiphers, size: usize },
}
