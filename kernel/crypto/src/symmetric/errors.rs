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
#[suzu(visibility(pub), context(suffix(Error)))]
pub enum SymmetricCipherError {
    /// Invalid key length
    #[suzu(display("{cipher:?}: invalid key length {actual} bytes (expected {expected} bytes)"))]
    InvalidKeyLength {
        cipher: SymmetricCiphers,
        expected: usize,
        actual: usize,
        #[suzu(from)]
        source: TryFromSliceError,
    },

    /// Invalid IV length
    #[suzu(display("{cipher:?}: invalid IV length {actual} bytes (expected {expected} bytes)"))]
    InvalidIvLength {
        cipher: SymmetricCiphers,
        expected: usize,
        actual: usize,
        #[suzu(from)]
        source: TryFromSliceError,
    },

    /// Invalid nonce length
    #[suzu(display("{cipher:?}: invalid nonce length {actual} bytes (expected {expected} bytes)"))]
    InvalidNonceLength {
        cipher: SymmetricCiphers,
        expected: usize,
        actual: usize,
        #[suzu(from)]
        source: TryFromSliceError,
    },

    /// Padding error during encryption or decryption
    #[suzu(display("{cipher:?}: padding error"))]
    Padding {
        cipher: SymmetricCiphers,
        #[suzu(from)]
        source: UnpadError,
    },

    /// Ciphertext is too short for decryption
    #[suzu(display("{cipher:?}: ciphertext too short ({size} bytes)"))]
    CiphertextTooShort { cipher: SymmetricCiphers, size: usize },
}
