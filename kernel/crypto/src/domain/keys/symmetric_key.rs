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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes128_creates_16_byte_key() {
        let bytes = [1u8; 16];
        let key = SymmetricKey::aes128(bytes);
        assert_eq!(key.len(), 16);
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn aes256_creates_32_byte_key() {
        let bytes = [2u8; 32];
        let key = SymmetricKey::aes256(bytes);
        assert_eq!(key.len(), 32);
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn as_bytes_returns_correct_slice() {
        let aes128_key = SymmetricKey::aes128([42u8; 16]);
        assert_eq!(aes128_key.as_bytes().len(), 16);
        assert!(aes128_key.as_bytes().iter().all(|&b| b == 42));

        let aes256_key = SymmetricKey::aes256([99u8; 32]);
        assert_eq!(aes256_key.as_bytes().len(), 32);
        assert!(aes256_key.as_bytes().iter().all(|&b| b == 99));
    }

    #[test]
    fn len_returns_correct_size() {
        let aes128 = SymmetricKey::aes128([0u8; 16]);
        let aes256 = SymmetricKey::aes256([0u8; 32]);
        assert_eq!(aes128.len(), 16);
        assert_eq!(aes256.len(), 32);
    }

    #[test]
    fn is_empty_always_returns_false() {
        let aes128 = SymmetricKey::aes128([0u8; 16]);
        let aes256 = SymmetricKey::aes256([0u8; 32]);
        assert!(!aes128.is_empty());
        assert!(!aes256.is_empty());
    }

    #[test]
    fn equality_compares_key_type_and_content() {
        let key1 = SymmetricKey::aes128([1u8; 16]);
        let key2 = SymmetricKey::aes128([1u8; 16]);
        let key3 = SymmetricKey::aes128([2u8; 16]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn different_key_types_are_not_equal() {
        let aes128 = SymmetricKey::aes128([1u8; 16]);
        let aes256 = SymmetricKey::aes256([1u8; 32]);
        assert_ne!(aes128, aes256);
    }

    #[test]
    fn clone_creates_independent_copy() {
        let key1 = SymmetricKey::aes256([42u8; 32]);
        let key2 = key1.clone();
        assert_eq!(key1, key2);
    }
}
