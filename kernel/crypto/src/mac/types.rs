use zeroize::{Zeroize, ZeroizeOnDrop};

/// MAC tag (Message Authentication Code).
///
/// Contains the authentication tag that ensures message integrity
/// and authenticity.
///
/// Tag is not secret, but implements [`Zeroize`] as a precaution.
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
pub struct MacTag(Vec<u8>);

impl MacTag {
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

    /// Consumes self and returns the underlying bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for MacTag {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for MacTag {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

/// 256-bit key for HMAC-SHA256.
///
/// # Security
///
/// HMAC accepts variable-length keys, but 256-bit (32 bytes) is recommended
/// for HMAC-SHA256 to match the hash output size.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct HmacSha256Key(pub(crate) [u8; 32]);

impl_secret_traits!(HmacSha256Key);

impl HmacSha256Key {
    /// Returns the key as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for HmacSha256Key {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

/// 384-bit key for HMAC-SHA384.
///
/// # Security
///
/// HMAC accepts variable-length keys, but 384-bit (48 bytes) is recommended
/// for HMAC-SHA384 to match the hash output size.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct HmacSha384Key(pub(crate) [u8; 48]);

impl_secret_traits!(HmacSha384Key);

impl HmacSha384Key {
    /// Returns the key as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 48]> for HmacSha384Key {
    fn from(bytes: [u8; 48]) -> Self {
        Self(bytes)
    }
}

/// 512-bit key for HMAC-SHA512.
///
/// # Security
///
/// HMAC accepts variable-length keys, but 512-bit (64 bytes) is recommended
/// for HMAC-SHA512 to match the hash output size.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct HmacSha512Key(pub(crate) [u8; 64]);

impl_secret_traits!(HmacSha512Key);

impl HmacSha512Key {
    /// Returns the key as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 64]> for HmacSha512Key {
    fn from(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }
}

/// 128-bit key for CMAC-AES-128.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CmacAes128Key(pub(crate) [u8; 16]);

impl_secret_traits!(CmacAes128Key);

impl CmacAes128Key {
    /// Returns the key as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 16]> for CmacAes128Key {
    fn from(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

/// 256-bit key for CMAC-AES-256.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CmacAes256Key(pub(crate) [u8; 32]);

impl_secret_traits!(CmacAes256Key);

impl CmacAes256Key {
    /// Returns the key as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for CmacAes256Key {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

/// 256-bit key for Poly1305.
///
/// # Security
///
/// **This key MUST be used only once.** Poly1305 is a one-time authenticator.
/// Reusing the same key for different messages completely breaks security.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Poly1305Key(pub(crate) [u8; 32]);

impl_secret_traits!(Poly1305Key);

impl Poly1305Key {
    /// Returns the key as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for Poly1305Key {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // MacTag Tests
    // ========================================

    #[test]
    fn test_mac_tag_from_vec() {
        let data = vec![1, 2, 3, 4];
        let tag = MacTag::from(data.clone());
        assert_eq!(tag.as_bytes(), &data);
    }

    #[test]
    fn test_mac_tag_from_slice() {
        let data = [1, 2, 3, 4];
        let tag = MacTag::from(data.as_slice());
        assert_eq!(tag.as_bytes(), &data);
    }

    #[test]
    fn test_mac_tag_len() {
        let tag = MacTag::from(vec![0u8; 32]);
        assert_eq!(tag.len(), 32);
    }

    #[test]
    fn test_mac_tag_is_empty() {
        assert!(MacTag::from(vec![]).is_empty());
        assert!(!MacTag::from(vec![1]).is_empty());
    }

    #[test]
    fn test_mac_tag_into_bytes() {
        let data = vec![1, 2, 3, 4];
        let tag = MacTag::from(data.clone());
        assert_eq!(tag.into_bytes(), data);
    }

    // ========================================
    // Key Tests
    // ========================================

    #[test]
    fn test_hmac_sha256_key() {
        let bytes = [1u8; 32];
        let key = HmacSha256Key::from(bytes);
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_hmac_sha384_key() {
        let bytes = [2u8; 48];
        let key = HmacSha384Key::from(bytes);
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_hmac_sha512_key() {
        let bytes = [3u8; 64];
        let key = HmacSha512Key::from(bytes);
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_cmac_aes128_key() {
        let bytes = [4u8; 16];
        let key = CmacAes128Key::from(bytes);
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_cmac_aes256_key() {
        let bytes = [5u8; 32];
        let key = CmacAes256Key::from(bytes);
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_poly1305_key() {
        let bytes = [6u8; 32];
        let key = Poly1305Key::from(bytes);
        assert_eq!(key.as_bytes(), &bytes);
    }
}
