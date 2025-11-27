use zeroize::{Zeroize, ZeroizeOnDrop};

/// Private key value object with automatic memory cleanup
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct PrivateKey(Vec<u8>);

impl PrivateKey {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        std::mem::take(&mut self.0)
    }
}

impl PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for PrivateKey {}
