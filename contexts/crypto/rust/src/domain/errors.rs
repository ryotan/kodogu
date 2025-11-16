use suzunari_error::{BoxedStackError, Location, StackError};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// Domain-level cryptographic errors
#[derive(Debug)]
pub enum CryptoError {
    KeyGeneration {
        cause: BoxedStackError,
        location: Location,
    },
    Encryption {
        cause: BoxedStackError,
        location: Location,
    },
    Decryption {
        cause: BoxedStackError,
        location: Location,
    },
    InvalidKeyFormat {
        cause: BoxedStackError,
        location: Location,
    },
    KeyDerivation {
        cause: BoxedStackError,
        location: Location,
    },
}

impl Display for CryptoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::KeyGeneration { cause, .. } => write!(f, "Key generation failed: {}", cause),
            Self::Encryption { cause, .. } => write!(f, "Encryption failed: {}", cause),
            Self::Decryption { cause, .. } => write!(f, "Decryption failed: {}", cause),
            Self::InvalidKeyFormat { cause, .. } => write!(f, "Invalid key format: {}", cause),
            Self::KeyDerivation { cause, .. } => write!(f, "Key derivation failed: {}", cause),
        }
    }
}

impl Error for CryptoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::KeyGeneration { cause, .. }
            | Self::Encryption { cause, .. }
            | Self::Decryption { cause, .. }
            | Self::InvalidKeyFormat { cause, .. }
            | Self::KeyDerivation { cause, .. } => Some(cause),
        }
    }
}

impl StackError for CryptoError {
    fn location(&self) -> &Location {
        match self {
            Self::KeyGeneration { location, .. }
            | Self::Encryption { location, .. }
            | Self::Decryption { location, .. }
            | Self::InvalidKeyFormat { location, .. }
            | Self::KeyDerivation { location, .. } => location,
        }
    }
}

// Helper macros for creating errors at call site
#[macro_export]
macro_rules! encryption_error {
    ($cause:expr) => {
        $crate::CryptoError::Encryption {
            cause: $crate::suzunari_error::BoxedStackError::new($cause),
            location: $crate::suzunari_error::Location::new(file!(), line!(), column!()),
        }
    };
}

#[macro_export]
macro_rules! decryption_error {
    ($cause:expr) => {
        $crate::CryptoError::Decryption {
            cause: $crate::suzunari_error::BoxedStackError::new($cause),
            location: $crate::suzunari_error::Location::new(file!(), line!(), column!()),
        }
    };
}

#[macro_export]
macro_rules! key_generation_error {
    ($cause:expr) => {
        $crate::CryptoError::KeyGeneration {
            cause: $crate::suzunari_error::BoxedStackError::new($cause),
            location: $crate::suzunari_error::Location::new(file!(), line!(), column!()),
        }
    };
}

#[macro_export]
macro_rules! key_derivation_error {
    ($cause:expr) => {
        $crate::CryptoError::KeyDerivation {
            cause: $crate::suzunari_error::BoxedStackError::new($cause),
            location: $crate::suzunari_error::Location::new(file!(), line!(), column!()),
        }
    };
}

#[macro_export]
macro_rules! invalid_key_format_error {
    ($cause:expr) => {
        $crate::CryptoError::InvalidKeyFormat {
            cause: $crate::suzunari_error::BoxedStackError::new($cause),
            location: $crate::suzunari_error::Location::new(file!(), line!(), column!()),
        }
    };
}
