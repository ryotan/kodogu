/// Plaintext value object (unencrypted data)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plaintext(Vec<u8>);

impl Plaintext {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn from_string(s: String) -> Self {
        Self(s.into_bytes())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.0).into_owned()
    }
}

impl From<String> for Plaintext {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for Plaintext {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_plaintext_from_bytes() {
        let bytes = vec![1, 2, 3, 4];
        let plaintext = Plaintext::new(bytes.clone());
        assert_eq!(plaintext.as_bytes(), &bytes);
    }

    #[test]
    fn new_accepts_empty_bytes() {
        let plaintext = Plaintext::new(vec![]);
        assert_eq!(plaintext.as_bytes(), &[]);
    }

    #[test]
    fn from_string_converts_string_to_plaintext() {
        let s = String::from("Hello, World!");
        let plaintext = Plaintext::from_string(s.clone());
        assert_eq!(plaintext.as_bytes(), s.as_bytes());
    }

    #[test]
    fn as_bytes_returns_reference() {
        let bytes = vec![5, 6, 7, 8];
        let plaintext = Plaintext::new(bytes.clone());
        assert_eq!(plaintext.as_bytes(), &bytes[..]);
    }

    #[test]
    fn into_bytes_consumes_and_returns_vec() {
        let bytes = vec![9, 10, 11, 12];
        let plaintext = Plaintext::new(bytes.clone());
        assert_eq!(plaintext.into_bytes(), bytes);
    }

    #[test]
    fn to_string_lossy_converts_valid_utf8() {
        let s = "Valid UTF-8 string";
        let plaintext = Plaintext::from(s);
        assert_eq!(plaintext.to_string_lossy(), s);
    }

    #[test]
    fn to_string_lossy_handles_invalid_utf8() {
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let plaintext = Plaintext::new(invalid_utf8);
        let result = plaintext.to_string_lossy();
        // Should not panic, result contains replacement characters
        assert!(!result.is_empty());
    }

    #[test]
    fn from_string_trait_implementation() {
        let s = String::from("Test");
        let plaintext: Plaintext = s.clone().into();
        assert_eq!(plaintext.as_bytes(), s.as_bytes());
    }

    #[test]
    fn from_str_trait_implementation() {
        let s = "String slice";
        let plaintext: Plaintext = s.into();
        assert_eq!(plaintext.as_bytes(), s.as_bytes());
    }

    #[test]
    fn clone_creates_independent_copy() {
        let plaintext1 = Plaintext::from("Original");
        let plaintext2 = plaintext1.clone();
        assert_eq!(plaintext1, plaintext2);
    }

    #[test]
    fn equality_compares_content() {
        let plaintext1 = Plaintext::from("Same");
        let plaintext2 = Plaintext::from("Same");
        let plaintext3 = Plaintext::from("Different");
        assert_eq!(plaintext1, plaintext2);
        assert_ne!(plaintext1, plaintext3);
    }
}
