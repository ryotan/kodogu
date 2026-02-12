use zeroize::Zeroize;

/// Authentication tag for AEAD ciphers.
///
/// Contains the authentication tag (typically 16 bytes) that ensures
/// ciphertext integrity and authenticity.
///
/// Tag is not secret, but implements [`Zeroize`] as a precaution.
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct AuthenticationTag(Vec<u8>);

impl AuthenticationTag {
    /// Returns the tag as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for AuthenticationTag {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for AuthenticationTag {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

/// Additional Authenticated Data (AAD) for AEAD ciphers.
///
/// AAD is authenticated but not encrypted. It can be used to bind
/// ciphertext to context (e.g., message ID, sender ID) without
/// revealing the plaintext.
///
/// AAD is not secret, but implements [`Zeroize`] as a precaution.
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct AdditionalData(Vec<u8>);

impl AdditionalData {
    /// Creates an empty AAD.
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// Returns the AAD as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for AdditionalData {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for AdditionalData {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

/// AEAD ciphertext with embedded authentication tag.
///
/// Combines ciphertext and authentication tag into a single value.
/// The tag is appended to the ciphertext (ciphertext || tag).
///
/// Not secret, but implements [`Zeroize`] as a precaution.
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct AeadCiphertext {
    ciphertext: Vec<u8>,
    tag: Vec<u8>,
}

impl AeadCiphertext {
    /// Creates a new AEAD ciphertext from separate ciphertext and tag.
    pub fn new(ciphertext: Vec<u8>, tag: Vec<u8>) -> Self {
        Self { ciphertext, tag }
    }

    /// Parses combined bytes (ciphertext || tag) with known tag length.
    ///
    /// Returns `None` if the input is shorter than the tag length.
    pub fn from_combined(combined: &[u8], tag_len: usize) -> Option<Self> {
        if combined.len() < tag_len {
            return None;
        }
        let split_point = combined.len() - tag_len;
        let ciphertext = combined[..split_point].to_vec();
        let tag = combined[split_point..].to_vec();
        Some(Self { ciphertext, tag })
    }

    /// Returns the ciphertext portion as a byte slice.
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    /// Returns the authentication tag as a byte slice.
    pub fn tag(&self) -> &[u8] {
        &self.tag
    }

    /// Returns combined bytes (ciphertext || tag).
    pub fn to_combined(&self) -> Vec<u8> {
        let mut combined = self.ciphertext.clone();
        combined.extend_from_slice(&self.tag);
        combined
    }

    /// Returns the total length (ciphertext + tag).
    pub fn len(&self) -> usize {
        self.ciphertext.len() + self.tag.len()
    }

    /// Returns true if both ciphertext and tag are empty.
    pub fn is_empty(&self) -> bool {
        self.ciphertext.is_empty() && self.tag.is_empty()
    }
}

/// 128-bit key for AES-128-GCM.
///
/// # Security
///
/// Implements [`Zeroize`] to clear memory on [`Drop`].
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct Aes128GcmKey(pub [u8; 16]);

impl Aes128GcmKey {
    /// Returns the key as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 16]> for Aes128GcmKey {
    fn from(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

/// 256-bit key for AES-256-GCM and AES-256-GCM-SIV.
///
/// # Security
///
/// Implements [`Zeroize`] to clear memory on [`Drop`].
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct Aes256GcmKey(pub [u8; 32]);

impl Aes256GcmKey {
    /// Returns the key as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for Aes256GcmKey {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

/// 256-bit key for ChaCha20-Poly1305 and XChaCha20-Poly1305.
///
/// # Security
///
/// Implements [`Zeroize`] to clear memory on [`Drop`].
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct ChaCha20Poly1305Key(pub [u8; 32]);

impl ChaCha20Poly1305Key {
    /// Returns the key as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for ChaCha20Poly1305Key {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // AuthenticationTag Tests
    // ========================================

    #[test]
    fn test_tag_from_vec() {
        let data = vec![1, 2, 3, 4];
        let tag = AuthenticationTag::from(data.clone());
        assert_eq!(tag.as_bytes(), &data);
    }

    #[test]
    fn test_tag_from_slice() {
        let data = [1, 2, 3, 4];
        let tag = AuthenticationTag::from(data.as_slice());
        assert_eq!(tag.as_bytes(), &data);
    }

    #[test]
    fn test_tag_len() {
        let tag = AuthenticationTag::from(vec![0u8; 16]);
        assert_eq!(tag.len(), 16);
    }

    #[test]
    fn test_tag_is_empty() {
        assert!(AuthenticationTag::from(vec![]).is_empty());
        assert!(!AuthenticationTag::from(vec![1]).is_empty());
    }

    // ========================================
    // AdditionalData Tests
    // ========================================

    #[test]
    fn test_aad_empty() {
        let aad = AdditionalData::empty();
        assert!(aad.is_empty());
        assert_eq!(aad.len(), 0);
    }

    #[test]
    fn test_aad_from_vec() {
        let data = vec![1, 2, 3, 4];
        let aad = AdditionalData::from(data.clone());
        assert_eq!(aad.as_bytes(), &data);
    }

    #[test]
    fn test_aad_from_slice() {
        let data = [1, 2, 3, 4];
        let aad = AdditionalData::from(data.as_slice());
        assert_eq!(aad.as_bytes(), &data);
    }

    #[test]
    fn test_aad_len() {
        let aad = AdditionalData::from(vec![0u8; 10]);
        assert_eq!(aad.len(), 10);
    }

    #[test]
    fn test_aad_is_empty() {
        assert!(AdditionalData::from(vec![]).is_empty());
        assert!(!AdditionalData::from(vec![1]).is_empty());
    }

    // ========================================
    // AeadCiphertext Tests
    // ========================================

    #[test]
    fn test_aead_ciphertext_new() {
        let ct = AeadCiphertext::new(vec![1, 2, 3], vec![4, 5, 6]);
        assert_eq!(ct.ciphertext(), &[1, 2, 3]);
        assert_eq!(ct.tag(), &[4, 5, 6]);
    }

    #[test]
    fn test_aead_ciphertext_from_combined() {
        let combined = vec![1, 2, 3, 4, 5, 6];
        let ct = AeadCiphertext::from_combined(&combined, 2).unwrap();
        assert_eq!(ct.ciphertext(), &[1, 2, 3, 4]);
        assert_eq!(ct.tag(), &[5, 6]);
    }

    #[test]
    fn test_aead_ciphertext_from_combined_too_short() {
        let combined = vec![1, 2, 3];
        assert!(AeadCiphertext::from_combined(&combined, 16).is_none());
    }

    #[test]
    fn test_aead_ciphertext_to_combined() {
        let ct = AeadCiphertext::new(vec![1, 2, 3], vec![4, 5]);
        assert_eq!(ct.to_combined(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_aead_ciphertext_len() {
        let ct = AeadCiphertext::new(vec![1, 2, 3], vec![4, 5]);
        assert_eq!(ct.len(), 5);
    }

    #[test]
    fn test_aead_ciphertext_is_empty() {
        assert!(AeadCiphertext::new(vec![], vec![]).is_empty());
        assert!(!AeadCiphertext::new(vec![1], vec![]).is_empty());
        assert!(!AeadCiphertext::new(vec![], vec![1]).is_empty());
    }

    // ========================================
    // Key Tests
    // ========================================

    #[test]
    fn test_aes128gcm_key() {
        let bytes = [1u8; 16];
        let key = Aes128GcmKey::from(bytes);
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_aes256gcm_key() {
        let bytes = [2u8; 32];
        let key = Aes256GcmKey::from(bytes);
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_chacha20poly1305_key() {
        let bytes = [3u8; 32];
        let key = ChaCha20Poly1305Key::from(bytes);
        assert_eq!(key.as_bytes(), &bytes);
    }
}
