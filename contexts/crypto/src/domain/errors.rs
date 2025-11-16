use std::fmt::Display;
use suzunari_error::BoxedStackError;

/// Domain-level cryptographic errors
#[derive(Debug)]
pub enum CryptoError {
    KeyGeneration { source: BoxedStackError },

    Encryption { source: BoxedStackError },

    Decryption { source: BoxedStackError },

    InvalidKeyEncoding { source: BoxedStackError },

    InvalidKeySize { source: BoxedStackError },

    KeyDerivation { source: BoxedStackError },
}

impl Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CryptoError {}
