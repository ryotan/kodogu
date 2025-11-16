use snafu::ensure;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::domain::{errors::InvalidKeySizeSnafu, CryptoError};

/// Symmetric key value object with automatic memory cleanup
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct SymmetricKey(Vec<u8>);

impl SymmetricKey {
    pub fn new(bytes: Vec<u8>) -> Result<Self, CryptoError> {
        ensure!(
            bytes.len() == 16 || bytes.len() == 32,
            InvalidKeySizeSnafu { size: bytes.len() }
        );
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        std::mem::take(&mut self.0)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl PartialEq for SymmetricKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SymmetricKey {}
