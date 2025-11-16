use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::domain::{CryptoError, InvalidKeyEncodingSnafu, InvalidKeySizeErrorSnafu};

/// Symmetric key value object with automatic memory cleanup
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct SymmetricKey(Vec<u8>);

impl SymmetricKey {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Import symmetric key from Base64-encoded string
    pub fn from_base64(encoded: &str) -> Result<Self, CryptoError> {
        let bytes = BASE64.decode(encoded).map_err(|e| {
            InvalidKeyEncodingSnafu {
                reason: e.to_string(),
            }
            .build()
        })?;

        // Validate key size (AES-128: 16 bytes, AES-256: 32 bytes)
        if bytes.len() != 16 && bytes.len() != 32 {
            return Err(InvalidKeySizeErrorSnafu { size: bytes.len() }.build().into());
        }

        Ok(Self(bytes))
    }

    /// Export symmetric key to Base64-encoded string
    pub fn to_base64(&self) -> String {
        BASE64.encode(&self.0)
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
