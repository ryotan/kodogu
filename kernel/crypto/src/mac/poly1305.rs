use super::errors::{MacAlgorithms, MacError, VerificationFailedError};
use super::traits::{Mac, MacGenerator, MacVerifier};
use super::types::{MacTag, Poly1305Key};
use poly1305::Poly1305 as Poly1305Impl;
use poly1305::universal_hash::KeyInit;
use snafu::ResultExt;
use subtle::ConstantTimeEq;

// Poly1305 固定サイズ
const POLY1305_TAG_SIZE: usize = 16;

// ========================================
// Poly1305
// ========================================

/// Poly1305 one-time message authenticator.
///
/// Poly1305 is a high-speed MAC designed for use with a single message.
/// Uses 256-bit (32-byte) key and produces 128-bit (16-byte) tag.
///
/// # Security Warning
///
/// **The key MUST be used only once.** Poly1305 is a one-time authenticator.
/// Reusing the same key for different messages completely breaks security.
///
/// In practice, Poly1305 keys are typically derived from a stream cipher
/// (like ChaCha20) for each message. The raw Poly1305 API is provided
/// for low-level use cases.
///
/// # Example
///
/// ```
/// use crypto::{Poly1305Mac, MacGenerator, MacVerifier, Poly1305Key};
///
/// let mac = Poly1305Mac;
/// // In real use, this key must be unique for EACH message
/// let key = Poly1305Key::from([0u8; 32]);
/// let message = b"Hello, World!";
///
/// let tag = mac.generate(message, &key).unwrap();
/// mac.verify(message, &tag, &key).unwrap();
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Poly1305Mac;

impl Poly1305Mac {
    /// Returns the required key size in bytes.
    pub fn key_size(&self) -> usize {
        32
    }

    /// Returns the output tag size in bytes.
    pub fn tag_size(&self) -> usize {
        POLY1305_TAG_SIZE
    }
}

impl MacGenerator for Poly1305Mac {
    type Key = Poly1305Key;
    type Error = MacError;

    fn generate(&self, message: &[u8], key: &Self::Key) -> Result<MacTag, Self::Error> {
        // Poly1305Key は常に 32 バイト
        let mac = Poly1305Impl::new(key.as_bytes().into());
        // compute_unpadded: 生の Poly1305（AEAD 用の padding なし）
        let result = mac.compute_unpadded(message);
        Ok(MacTag::from(result.as_slice().to_vec()))
    }
}

impl MacVerifier for Poly1305Mac {
    type Key = Poly1305Key;
    type Error = MacError;

    fn verify(&self, message: &[u8], tag: &MacTag, key: &Self::Key) -> Result<(), Self::Error> {
        let expected = MacGenerator::generate(self, message, key)?;

        // constant-time 比較
        if expected.as_bytes().ct_eq(tag.as_bytes()).into() {
            Ok(())
        } else {
            // Poly1305 は verify_slice を持たないため、hmac::digest::MacError を source として使用
            Err(hmac::digest::MacError)
                .context(VerificationFailedError { algorithm: MacAlgorithms::Poly1305 })
        }
    }
}

impl Mac for Poly1305Mac {
    type Key = Poly1305Key;
    type Error = MacError;
}

/// Marker trait for Poly1305 cipher.
pub trait Poly1305Cipher: Mac<Error = MacError> {}

impl Poly1305Cipher for Poly1305Mac {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Poly1305Key {
        // 非ゼロ鍵を使用（Poly1305 では全ゼロ鍵だと MAC も全ゼロになる）
        let mut key = [0u8; 32];
        for (i, byte) in key.iter_mut().enumerate() {
            *byte = (i + 1) as u8;
        }
        Poly1305Key::from(key)
    }

    // ========================================
    // Poly1305 Tests
    // ========================================

    #[test]
    fn test_poly1305_generate_verify_roundtrip() {
        let mac = Poly1305Mac;
        let key = test_key();
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        assert_eq!(tag.len(), POLY1305_TAG_SIZE);
        MacVerifier::verify(&mac, message, &tag, &key).unwrap();
    }

    #[test]
    fn test_poly1305_verification_fails_on_wrong_message() {
        let mac = Poly1305Mac;
        let key = test_key();
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        let result = MacVerifier::verify(&mac, b"Wrong message", &tag, &key);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_poly1305_verification_fails_on_wrong_key() {
        let mac = Poly1305Mac;
        let key1 = test_key();
        let key2 = Poly1305Key::from([1u8; 32]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key1).unwrap();
        let result = MacVerifier::verify(&mac, message, &tag, &key2);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_poly1305_verification_fails_on_tampered_tag() {
        let mac = Poly1305Mac;
        let key = test_key();
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        let mut tampered = tag.into_bytes();
        tampered[0] ^= 0xFF;
        let tampered_tag = MacTag::from(tampered);

        let result = MacVerifier::verify(&mac, message, &tampered_tag, &key);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_poly1305_different_keys_produce_different_tags() {
        let mac = Poly1305Mac;
        let key1 = test_key();
        let key2 = Poly1305Key::from([1u8; 32]);
        let message = b"Hello, World!";

        let tag1 = MacGenerator::generate(&mac, message, &key1).unwrap();
        let tag2 = MacGenerator::generate(&mac, message, &key2).unwrap();
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_poly1305_different_messages_produce_different_tags() {
        let mac = Poly1305Mac;
        let key = test_key();

        let tag1 = MacGenerator::generate(&mac, b"Message 1", &key).unwrap();
        let tag2 = MacGenerator::generate(&mac, b"Message 2", &key).unwrap();
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_poly1305_empty_message() {
        let mac = Poly1305Mac;
        let key = test_key();

        let tag = MacGenerator::generate(&mac, b"", &key).unwrap();
        assert_eq!(tag.len(), POLY1305_TAG_SIZE);
        MacVerifier::verify(&mac, b"", &tag, &key).unwrap();
    }

    #[test]
    fn test_poly1305_sizes() {
        let mac = Poly1305Mac;
        assert_eq!(mac.key_size(), 32);
        assert_eq!(mac.tag_size(), 16);
    }
}
