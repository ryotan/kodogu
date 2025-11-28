/// Ciphertext value object (encrypted data)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(Vec<u8>);

impl Ciphertext {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_ciphertext_from_bytes() {
        let bytes = vec![1, 2, 3, 4];
        let ciphertext = Ciphertext::new(bytes.clone());
        assert_eq!(ciphertext.as_bytes(), &bytes);
    }

    #[test]
    fn new_accepts_empty_bytes() {
        let ciphertext = Ciphertext::new(vec![]);
        assert_eq!(ciphertext.as_bytes(), &[]);
    }

    #[test]
    fn as_bytes_returns_reference() {
        let bytes = vec![5, 6, 7, 8];
        let ciphertext = Ciphertext::new(bytes.clone());
        assert_eq!(ciphertext.as_bytes(), &bytes[..]);
    }

    #[test]
    fn into_bytes_consumes_and_returns_vec() {
        let bytes = vec![9, 10, 11, 12];
        let ciphertext = Ciphertext::new(bytes.clone());
        assert_eq!(ciphertext.into_bytes(), bytes);
    }

    #[test]
    fn clone_creates_independent_copy() {
        let ciphertext1 = Ciphertext::new(vec![1, 2, 3]);
        let ciphertext2 = ciphertext1.clone();
        assert_eq!(ciphertext1, ciphertext2);
    }

    #[test]
    fn equality_compares_content() {
        let ciphertext1 = Ciphertext::new(vec![1, 2, 3]);
        let ciphertext2 = Ciphertext::new(vec![1, 2, 3]);
        let ciphertext3 = Ciphertext::new(vec![4, 5, 6]);
        assert_eq!(ciphertext1, ciphertext2);
        assert_ne!(ciphertext1, ciphertext3);
    }
}
