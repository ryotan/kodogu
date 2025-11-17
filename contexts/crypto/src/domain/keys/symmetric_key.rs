use zeroize::Zeroize;

/// Symmetric key value object with type-level size guarantee and automatic memory cleanup
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymmetricKey {
    Aes128(Box<[u8; 16]>),
    Aes256(Box<[u8; 32]>),
}

impl SymmetricKey {
    pub fn aes128(bytes: [u8; 16]) -> Self {
        Self::Aes128(Box::new(bytes))
    }

    pub fn aes256(bytes: [u8; 32]) -> Self {
        Self::Aes256(Box::new(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Aes128(bytes) => bytes.as_slice(),
            Self::Aes256(bytes) => bytes.as_slice(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Aes128(_) => 16,
            Self::Aes256(_) => 32,
        }
    }

    pub fn is_empty(&self) -> bool {
        false // Keys are never empty
    }
}

impl Zeroize for SymmetricKey {
    fn zeroize(&mut self) {
        match self {
            Self::Aes128(bytes) => bytes.zeroize(),
            Self::Aes256(bytes) => bytes.zeroize(),
        }
    }
}

impl Drop for SymmetricKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}
