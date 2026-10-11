use super::errors::{InvalidNonceLengthError, MacAlgorithms, MacError, VerificationFailedError};
use super::traits::{Mac, MacGenerator, MacVerifier};
use super::types::MacTag;
use crate::{Aes256GcmKey, Nonce};
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit};
use suzunari_error::ResultExt;

const GMAC_NONCE_SIZE: usize = 12;
const GMAC_TAG_SIZE: usize = 16;

// ========================================
// GMAC
// ========================================

/// GMAC (Galois Message Authentication Code).
///
/// GMAC is the MAC-only mode of AES-GCM. It provides authentication
/// without encryption by using AES-GCM with empty plaintext.
///
/// Uses AES-256 with 96-bit nonce and 128-bit output.
///
/// # Security Warning
///
/// **Never reuse a nonce with the same key.**
/// Nonce reuse completely breaks GMAC security.
///
/// # Example
///
/// ```
/// use crypto::{Gmac, MacGenerator, MacVerifier, Aes256GcmKey, Nonce};
///
/// let mac = Gmac;
/// let key = Aes256GcmKey::from([0u8; 32]);
/// let nonce = Nonce::from(vec![0u8; 12]);
/// let message = b"Hello, World!";
///
/// let tag = mac.generate_with_nonce(message, &key, &nonce).unwrap();
/// mac.verify_with_nonce(message, &tag, &key, &nonce).unwrap();
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Gmac;

impl Gmac {
    /// Generates a GMAC tag with the given nonce.
    ///
    /// GMAC requires a nonce (unlike HMAC/CMAC). The nonce must be
    /// unique for each message with the same key.
    pub fn generate_with_nonce(
        &self,
        message: &[u8],
        key: &Aes256GcmKey,
        nonce: &Nonce,
    ) -> Result<MacTag, MacError> {
        let nonce_array = validate_nonce_length(nonce)?;
        let cipher = Aes256Gcm::new(key.as_bytes().into());

        // GMAC is AES-GCM over an empty plaintext with the message as AAD
        let ciphertext_with_tag = cipher
            .encrypt(
                nonce_array.as_slice().into(),
                aes_gcm::aead::Payload { msg: &[], aad: message },
            )
            .map_err(|_| hmac::digest::MacError)
            .context(VerificationFailedError { algorithm: MacAlgorithms::Gmac })?;

        debug_assert_eq!(ciphertext_with_tag.len(), GMAC_TAG_SIZE);
        Ok(MacTag::from(ciphertext_with_tag))
    }

    /// Verifies a GMAC tag with the given nonce.
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    pub fn verify_with_nonce(
        &self,
        message: &[u8],
        tag: &MacTag,
        key: &Aes256GcmKey,
        nonce: &Nonce,
    ) -> Result<(), MacError> {
        let nonce_array = validate_nonce_length(nonce)?;
        let cipher = Aes256Gcm::new(key.as_bytes().into());

        // Verify by decrypting an empty ciphertext followed by the tag
        cipher
            .decrypt(
                nonce_array.as_slice().into(),
                aes_gcm::aead::Payload { msg: tag.as_bytes(), aad: message },
            )
            .map_err(|_| hmac::digest::MacError)
            .context(VerificationFailedError { algorithm: MacAlgorithms::Gmac })?;

        Ok(())
    }

    /// Returns the required nonce size in bytes.
    pub fn nonce_size(&self) -> usize {
        GMAC_NONCE_SIZE
    }

    /// Returns the output tag size in bytes.
    pub fn tag_size(&self) -> usize {
        GMAC_TAG_SIZE
    }
}

fn validate_nonce_length(nonce: &Nonce) -> Result<[u8; GMAC_NONCE_SIZE], MacError> {
    nonce.as_bytes().try_into().context(InvalidNonceLengthError {
        algorithm: MacAlgorithms::Gmac,
        expected: GMAC_NONCE_SIZE,
        actual: nonce.len(),
    })
}

// ========================================
// MacGenerator / MacVerifier
// GMAC needs a nonce, so the key is a (key, nonce) pair
// ========================================

/// GMAC key with embedded nonce.
///
/// GMAC requires both a key and a nonce for each operation.
/// This type bundles them together for the MacGenerator/MacVerifier traits.
#[derive(Debug, Clone)]
pub struct GmacKeyWithNonce {
    /// The AES-256 key.
    pub key: Aes256GcmKey,
    /// The nonce (12 bytes).
    pub nonce: Nonce,
}

impl GmacKeyWithNonce {
    /// Creates a new GMAC key with nonce.
    pub fn new(key: Aes256GcmKey, nonce: Nonce) -> Self {
        Self { key, nonce }
    }
}

impl MacGenerator for Gmac {
    type Key = GmacKeyWithNonce;
    type Error = MacError;

    fn generate(&self, message: &[u8], key: &Self::Key) -> Result<MacTag, Self::Error> {
        self.generate_with_nonce(message, &key.key, &key.nonce)
    }
}

impl MacVerifier for Gmac {
    type Key = GmacKeyWithNonce;
    type Error = MacError;

    fn verify(&self, message: &[u8], tag: &MacTag, key: &Self::Key) -> Result<(), Self::Error> {
        self.verify_with_nonce(message, tag, &key.key, &key.nonce)
    }
}

impl Mac for Gmac {
    type Key = GmacKeyWithNonce;
    type Error = MacError;
}

/// Marker trait for GMAC cipher.
pub trait GmacCipher: Mac<Error = MacError> {}

impl GmacCipher for Gmac {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Aes256GcmKey {
        Aes256GcmKey::from([0u8; 32])
    }

    fn test_nonce() -> Nonce {
        Nonce::from(vec![0u8; 12])
    }

    // ========================================
    // GMAC with nonce Tests
    // ========================================

    #[test]
    fn test_gmac_generate_verify_roundtrip() {
        let mac = Gmac;
        let key = test_key();
        let nonce = test_nonce();
        let message = b"Hello, World!";

        let tag = mac.generate_with_nonce(message, &key, &nonce).unwrap();
        assert_eq!(tag.len(), GMAC_TAG_SIZE);
        mac.verify_with_nonce(message, &tag, &key, &nonce).unwrap();
    }

    #[test]
    fn test_gmac_verification_fails_on_wrong_message() {
        let mac = Gmac;
        let key = test_key();
        let nonce = test_nonce();
        let message = b"Hello, World!";

        let tag = mac.generate_with_nonce(message, &key, &nonce).unwrap();
        let result = mac.verify_with_nonce(b"Wrong message", &tag, &key, &nonce);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_gmac_verification_fails_on_wrong_key() {
        let mac = Gmac;
        let key1 = test_key();
        let key2 = Aes256GcmKey::from([1u8; 32]);
        let nonce = test_nonce();
        let message = b"Hello, World!";

        let tag = mac.generate_with_nonce(message, &key1, &nonce).unwrap();
        let result = mac.verify_with_nonce(message, &tag, &key2, &nonce);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_gmac_verification_fails_on_wrong_nonce() {
        let mac = Gmac;
        let key = test_key();
        let nonce1 = test_nonce();
        let nonce2 = Nonce::from(vec![1u8; 12]);
        let message = b"Hello, World!";

        let tag = mac.generate_with_nonce(message, &key, &nonce1).unwrap();
        let result = mac.verify_with_nonce(message, &tag, &key, &nonce2);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_gmac_verification_fails_on_tampered_tag() {
        let mac = Gmac;
        let key = test_key();
        let nonce = test_nonce();
        let message = b"Hello, World!";

        let tag = mac.generate_with_nonce(message, &key, &nonce).unwrap();
        let mut tampered = tag.into_bytes();
        tampered[0] ^= 0xFF;
        let tampered_tag = MacTag::from(tampered);

        let result = mac.verify_with_nonce(message, &tampered_tag, &key, &nonce);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_gmac_different_keys_produce_different_tags() {
        let mac = Gmac;
        let key1 = test_key();
        let key2 = Aes256GcmKey::from([1u8; 32]);
        let nonce = test_nonce();
        let message = b"Hello, World!";

        let tag1 = mac.generate_with_nonce(message, &key1, &nonce).unwrap();
        let tag2 = mac.generate_with_nonce(message, &key2, &nonce).unwrap();
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_gmac_different_nonces_produce_different_tags() {
        let mac = Gmac;
        let key = test_key();
        let nonce1 = test_nonce();
        let nonce2 = Nonce::from(vec![1u8; 12]);
        let message = b"Hello, World!";

        let tag1 = mac.generate_with_nonce(message, &key, &nonce1).unwrap();
        let tag2 = mac.generate_with_nonce(message, &key, &nonce2).unwrap();
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_gmac_different_messages_produce_different_tags() {
        let mac = Gmac;
        let key = test_key();
        let nonce = test_nonce();

        let tag1 = mac.generate_with_nonce(b"Message 1", &key, &nonce).unwrap();
        let tag2 = mac.generate_with_nonce(b"Message 2", &key, &nonce).unwrap();
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_gmac_empty_message() {
        let mac = Gmac;
        let key = test_key();
        let nonce = test_nonce();

        let tag = mac.generate_with_nonce(b"", &key, &nonce).unwrap();
        assert_eq!(tag.len(), GMAC_TAG_SIZE);
        mac.verify_with_nonce(b"", &tag, &key, &nonce).unwrap();
    }

    #[test]
    fn test_gmac_invalid_nonce_length() {
        let mac = Gmac;
        let key = test_key();
        let bad_nonce = Nonce::from(vec![0u8; 16]); // 16 bytes instead of 12

        let result = mac.generate_with_nonce(b"Hello", &key, &bad_nonce);
        assert!(matches!(result, Err(MacError::InvalidNonceLength { .. })));
    }

    // ========================================
    // MacGenerator/MacVerifier trait Tests
    // ========================================

    #[test]
    fn test_gmac_trait_generate_verify() {
        let mac = Gmac;
        let key = GmacKeyWithNonce::new(test_key(), test_nonce());
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        assert_eq!(tag.len(), GMAC_TAG_SIZE);
        MacVerifier::verify(&mac, message, &tag, &key).unwrap();
    }

    #[test]
    fn test_gmac_sizes() {
        let mac = Gmac;
        assert_eq!(mac.nonce_size(), 12);
        assert_eq!(mac.tag_size(), 16);
    }
}
