use std::fmt::Display;

use suzunari_error::{suzunari_error, BoxedStackError};

/// Domain-level validation errors (internal, with stack trace)
#[suzunari_error]
#[snafu(visibility(pub(crate)))]
pub enum DomainError {
    #[snafu(display("Invalid key size: {size} bytes (expected 16 or 32)"))]
    InvalidKeySize { size: usize },
}

/// Domain-level cryptographic errors
#[derive(Debug)]
pub enum CryptoError {
    KeyGeneration { source: BoxedStackError },

    Encryption { source: BoxedStackError },

    Decryption { source: BoxedStackError },

    InvalidKeySize { source: BoxedStackError },

    KeyDerivation { source: BoxedStackError },
}

impl From<DomainError> for CryptoError {
    fn from(e: DomainError) -> Self {
        match &e {
            DomainError::InvalidKeySize { .. } => CryptoError::InvalidKeySize {
                source: BoxedStackError::new(e),
            },
        }
    }
}

impl Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CryptoError {}
