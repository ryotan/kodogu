use std::fmt::Display;
use suzunari_error::{BoxedStackError, Location, StackError};

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

/// Simple domain-level validation error that implements StackError
/// Used for domain layer validation failures without depending on adapter layer
pub struct DomainValidationError {
    message: String,
    location: Location,
}

impl DomainValidationError {
    #[track_caller]
    pub fn invalid_key_size(size: usize) -> Self {
        Self {
            message: format!("Invalid key size: {} bytes", size),
            location: Location::current(),
        }
    }

    #[track_caller]
    pub fn invalid_encoding(reason: &str) -> Self {
        Self {
            message: format!("Invalid encoding: {}", reason),
            location: Location::current(),
        }
    }
}

impl Display for DomainValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Debug for DomainValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, at {:?}", self.message, self.location)
    }
}

impl std::error::Error for DomainValidationError {}

impl StackError for DomainValidationError {
    fn location(&self) -> &Location {
        &self.location
    }
}
