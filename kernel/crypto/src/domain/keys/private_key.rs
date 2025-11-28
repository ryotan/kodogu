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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_private_key_from_bytes() {
        let bytes = vec![1, 2, 3, 4];
        let key = PrivateKey::new(bytes.clone());
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn as_bytes_returns_reference() {
        let bytes = vec![5, 6, 7, 8];
        let key = PrivateKey::new(bytes.clone());
        assert_eq!(key.as_bytes(), &bytes[..]);
    }

    #[test]
    fn into_bytes_consumes_and_returns_vec() {
        let bytes = vec![9, 10, 11, 12];
        let key = PrivateKey::new(bytes.clone());
        assert_eq!(key.into_bytes(), bytes);
    }

    #[test]
    fn clone_creates_independent_copy() {
        let key1 = PrivateKey::new(vec![1, 2, 3]);
        let key2 = key1.clone();
        assert_eq!(key1, key2);
    }

    #[test]
    fn equality_compares_content() {
        let key1 = PrivateKey::new(vec![1, 2, 3]);
        let key2 = PrivateKey::new(vec![1, 2, 3]);
        let key3 = PrivateKey::new(vec![4, 5, 6]);
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
