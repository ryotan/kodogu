use super::errors::{KeyInitFailedError, MacAlgorithms, MacError, VerificationFailedError};
use super::traits::{Mac, MacGenerator, MacVerifier};
use super::types::{HmacSha256Key, HmacSha384Key, HmacSha512Key, MacTag};
use hmac::{Hmac, Mac as HmacMac};
use sha2::{Sha256, Sha384, Sha512};
use suzunari_error::ResultExt;

const HMAC_SHA256_OUTPUT_SIZE: usize = 32;
const HMAC_SHA384_OUTPUT_SIZE: usize = 48;
const HMAC_SHA512_OUTPUT_SIZE: usize = 64;

// ========================================
// HMAC-SHA256
// ========================================

/// HMAC-SHA256 message authentication code.
///
/// Uses SHA-256 hash function with 256-bit (32-byte) output.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{HmacSha256, MacGenerator, MacVerifier, HmacSha256Key};
///
/// let mac = HmacSha256;
/// let key = HmacSha256Key::from([0u8; 32]);
/// let message = b"Hello, World!";
///
/// let tag = mac.generate(message, &key).unwrap();
/// mac.verify(message, &tag, &key).unwrap();
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HmacSha256;

type HmacSha256Impl = Hmac<Sha256>;

impl MacGenerator for HmacSha256 {
    type Key = HmacSha256Key;
    type Error = MacError;

    fn generate(&self, message: &[u8], key: &Self::Key) -> Result<MacTag, Self::Error> {
        // Cannot fail: HMAC accepts any key length (RFC 2104)
        let mut mac = HmacSha256Impl::new_from_slice(key.as_bytes())
            .context(KeyInitFailedError { algorithm: MacAlgorithms::HmacSha256 })?;
        mac.update(message);
        let result = mac.finalize();
        Ok(MacTag::from(result.into_bytes().to_vec()))
    }
}

impl MacVerifier for HmacSha256 {
    type Key = HmacSha256Key;
    type Error = MacError;

    fn verify(&self, message: &[u8], tag: &MacTag, key: &Self::Key) -> Result<(), Self::Error> {
        // Cannot fail: HMAC accepts any key length (RFC 2104)
        let mut mac = HmacSha256Impl::new_from_slice(key.as_bytes())
            .context(KeyInitFailedError { algorithm: MacAlgorithms::HmacSha256 })?;
        mac.update(message);
        // verify_slice compares in constant time
        mac.verify_slice(tag.as_bytes())
            .context(VerificationFailedError { algorithm: MacAlgorithms::HmacSha256 })
    }
}

impl Mac for HmacSha256 {
    type Key = HmacSha256Key;
    type Error = MacError;
}

/// Marker trait for HMAC cipher family.
///
/// Enables generic programming over HMAC variants (SHA256, SHA384, SHA512).
pub trait HmacCipher: Mac<Error = MacError> {}

impl HmacCipher for HmacSha256 {}

// ========================================
// HMAC-SHA384
// ========================================

/// HMAC-SHA384 message authentication code.
///
/// Uses SHA-384 hash function with 384-bit (48-byte) output.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{HmacSha384, MacGenerator, MacVerifier, HmacSha384Key};
///
/// let mac = HmacSha384;
/// let key = HmacSha384Key::from([0u8; 48]);
/// let message = b"Hello, World!";
///
/// let tag = mac.generate(message, &key).unwrap();
/// mac.verify(message, &tag, &key).unwrap();
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HmacSha384;

type HmacSha384Impl = Hmac<Sha384>;

impl MacGenerator for HmacSha384 {
    type Key = HmacSha384Key;
    type Error = MacError;

    fn generate(&self, message: &[u8], key: &Self::Key) -> Result<MacTag, Self::Error> {
        // Cannot fail: HMAC accepts any key length (RFC 2104)
        let mut mac = HmacSha384Impl::new_from_slice(key.as_bytes())
            .context(KeyInitFailedError { algorithm: MacAlgorithms::HmacSha384 })?;
        mac.update(message);
        let result = mac.finalize();
        Ok(MacTag::from(result.into_bytes().to_vec()))
    }
}

impl MacVerifier for HmacSha384 {
    type Key = HmacSha384Key;
    type Error = MacError;

    fn verify(&self, message: &[u8], tag: &MacTag, key: &Self::Key) -> Result<(), Self::Error> {
        // Cannot fail: HMAC accepts any key length (RFC 2104)
        let mut mac = HmacSha384Impl::new_from_slice(key.as_bytes())
            .context(KeyInitFailedError { algorithm: MacAlgorithms::HmacSha384 })?;
        mac.update(message);
        mac.verify_slice(tag.as_bytes())
            .context(VerificationFailedError { algorithm: MacAlgorithms::HmacSha384 })
    }
}

impl Mac for HmacSha384 {
    type Key = HmacSha384Key;
    type Error = MacError;
}

impl HmacCipher for HmacSha384 {}

// ========================================
// HMAC-SHA512
// ========================================

/// HMAC-SHA512 message authentication code.
///
/// Uses SHA-512 hash function with 512-bit (64-byte) output.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{HmacSha512, MacGenerator, MacVerifier, HmacSha512Key};
///
/// let mac = HmacSha512;
/// let key = HmacSha512Key::from([0u8; 64]);
/// let message = b"Hello, World!";
///
/// let tag = mac.generate(message, &key).unwrap();
/// mac.verify(message, &tag, &key).unwrap();
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HmacSha512;

type HmacSha512Impl = Hmac<Sha512>;

impl MacGenerator for HmacSha512 {
    type Key = HmacSha512Key;
    type Error = MacError;

    fn generate(&self, message: &[u8], key: &Self::Key) -> Result<MacTag, Self::Error> {
        // Cannot fail: HMAC accepts any key length (RFC 2104)
        let mut mac = HmacSha512Impl::new_from_slice(key.as_bytes())
            .context(KeyInitFailedError { algorithm: MacAlgorithms::HmacSha512 })?;
        mac.update(message);
        let result = mac.finalize();
        Ok(MacTag::from(result.into_bytes().to_vec()))
    }
}

impl MacVerifier for HmacSha512 {
    type Key = HmacSha512Key;
    type Error = MacError;

    fn verify(&self, message: &[u8], tag: &MacTag, key: &Self::Key) -> Result<(), Self::Error> {
        // Cannot fail: HMAC accepts any key length (RFC 2104)
        let mut mac = HmacSha512Impl::new_from_slice(key.as_bytes())
            .context(KeyInitFailedError { algorithm: MacAlgorithms::HmacSha512 })?;
        mac.update(message);
        mac.verify_slice(tag.as_bytes())
            .context(VerificationFailedError { algorithm: MacAlgorithms::HmacSha512 })
    }
}

impl Mac for HmacSha512 {
    type Key = HmacSha512Key;
    type Error = MacError;
}

impl HmacCipher for HmacSha512 {}

// ========================================
// HmacAlgorithm enum
// ========================================

/// HMAC algorithm variants for runtime selection.
///
/// # Example
///
/// ```
/// use crypto::{HmacAlgorithm, MacGenerator, MacVerifier};
///
/// let mac = HmacAlgorithm::Sha256;
/// let key = [0u8; 32];
/// let message = b"Hello, World!";
///
/// let tag = mac.generate(message, &key).unwrap();
/// mac.verify(message, &tag, &key).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HmacAlgorithm {
    /// HMAC-SHA256 (32-byte output)
    Sha256,
    /// HMAC-SHA384 (48-byte output)
    Sha384,
    /// HMAC-SHA512 (64-byte output)
    Sha512,
}

impl HmacAlgorithm {
    /// Returns the expected output size in bytes.
    pub fn output_size(&self) -> usize {
        match self {
            HmacAlgorithm::Sha256 => HMAC_SHA256_OUTPUT_SIZE,
            HmacAlgorithm::Sha384 => HMAC_SHA384_OUTPUT_SIZE,
            HmacAlgorithm::Sha512 => HMAC_SHA512_OUTPUT_SIZE,
        }
    }

    /// Returns the recommended key size in bytes (matches output size).
    pub fn recommended_key_size(&self) -> usize {
        self.output_size()
    }
}

impl MacGenerator for HmacAlgorithm {
    type Key = [u8];
    type Error = MacError;

    fn generate(&self, message: &[u8], key: &Self::Key) -> Result<MacTag, Self::Error> {
        let algorithm = match self {
            HmacAlgorithm::Sha256 => MacAlgorithms::HmacSha256,
            HmacAlgorithm::Sha384 => MacAlgorithms::HmacSha384,
            HmacAlgorithm::Sha512 => MacAlgorithms::HmacSha512,
        };

        match self {
            HmacAlgorithm::Sha256 => {
                // Cannot fail: HMAC accepts any key length (RFC 2104)
                let mut mac = HmacSha256Impl::new_from_slice(key)
                    .context(KeyInitFailedError { algorithm })?;
                mac.update(message);
                Ok(MacTag::from(mac.finalize().into_bytes().to_vec()))
            }
            HmacAlgorithm::Sha384 => {
                let mut mac = HmacSha384Impl::new_from_slice(key)
                    .context(KeyInitFailedError { algorithm })?;
                mac.update(message);
                Ok(MacTag::from(mac.finalize().into_bytes().to_vec()))
            }
            HmacAlgorithm::Sha512 => {
                let mut mac = HmacSha512Impl::new_from_slice(key)
                    .context(KeyInitFailedError { algorithm })?;
                mac.update(message);
                Ok(MacTag::from(mac.finalize().into_bytes().to_vec()))
            }
        }
    }
}

impl MacVerifier for HmacAlgorithm {
    type Key = [u8];
    type Error = MacError;

    fn verify(&self, message: &[u8], tag: &MacTag, key: &Self::Key) -> Result<(), Self::Error> {
        let algorithm = match self {
            HmacAlgorithm::Sha256 => MacAlgorithms::HmacSha256,
            HmacAlgorithm::Sha384 => MacAlgorithms::HmacSha384,
            HmacAlgorithm::Sha512 => MacAlgorithms::HmacSha512,
        };

        match self {
            HmacAlgorithm::Sha256 => {
                let mut mac = HmacSha256Impl::new_from_slice(key)
                    .context(KeyInitFailedError { algorithm })?;
                mac.update(message);
                mac.verify_slice(tag.as_bytes()).context(VerificationFailedError { algorithm })
            }
            HmacAlgorithm::Sha384 => {
                let mut mac = HmacSha384Impl::new_from_slice(key)
                    .context(KeyInitFailedError { algorithm })?;
                mac.update(message);
                mac.verify_slice(tag.as_bytes()).context(VerificationFailedError { algorithm })
            }
            HmacAlgorithm::Sha512 => {
                let mut mac = HmacSha512Impl::new_from_slice(key)
                    .context(KeyInitFailedError { algorithm })?;
                mac.update(message);
                mac.verify_slice(tag.as_bytes()).context(VerificationFailedError { algorithm })
            }
        }
    }
}

impl Mac for HmacAlgorithm {
    type Key = [u8];
    type Error = MacError;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // HMAC-SHA256 Tests
    // ========================================

    #[test]
    fn test_hmac_sha256_generate_verify_roundtrip() {
        let mac = HmacSha256;
        let key = HmacSha256Key::from([0u8; 32]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        assert_eq!(tag.len(), HMAC_SHA256_OUTPUT_SIZE);
        MacVerifier::verify(&mac, message, &tag, &key).unwrap();
    }

    #[test]
    fn test_hmac_sha256_verification_fails_on_wrong_message() {
        let mac = HmacSha256;
        let key = HmacSha256Key::from([0u8; 32]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        let result = MacVerifier::verify(&mac, b"Wrong message", &tag, &key);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_hmac_sha256_verification_fails_on_wrong_key() {
        let mac = HmacSha256;
        let key1 = HmacSha256Key::from([0u8; 32]);
        let key2 = HmacSha256Key::from([1u8; 32]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key1).unwrap();
        let result = MacVerifier::verify(&mac, message, &tag, &key2);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_hmac_sha256_verification_fails_on_tampered_tag() {
        let mac = HmacSha256;
        let key = HmacSha256Key::from([0u8; 32]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        let mut tampered = tag.into_bytes();
        tampered[0] ^= 0xFF;
        let tampered_tag = MacTag::from(tampered);

        let result = MacVerifier::verify(&mac, message, &tampered_tag, &key);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_hmac_sha256_different_keys_produce_different_tags() {
        let mac = HmacSha256;
        let key1 = HmacSha256Key::from([0u8; 32]);
        let key2 = HmacSha256Key::from([1u8; 32]);
        let message = b"Hello, World!";

        let tag1 = MacGenerator::generate(&mac, message, &key1).unwrap();
        let tag2 = MacGenerator::generate(&mac, message, &key2).unwrap();
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_hmac_sha256_different_messages_produce_different_tags() {
        let mac = HmacSha256;
        let key = HmacSha256Key::from([0u8; 32]);

        let tag1 = MacGenerator::generate(&mac, b"Message 1", &key).unwrap();
        let tag2 = MacGenerator::generate(&mac, b"Message 2", &key).unwrap();
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_hmac_sha256_empty_message() {
        let mac = HmacSha256;
        let key = HmacSha256Key::from([0u8; 32]);

        let tag = MacGenerator::generate(&mac, b"", &key).unwrap();
        assert_eq!(tag.len(), HMAC_SHA256_OUTPUT_SIZE);
        MacVerifier::verify(&mac, b"", &tag, &key).unwrap();
    }

    // ========================================
    // HMAC-SHA384 Tests
    // ========================================

    #[test]
    fn test_hmac_sha384_generate_verify_roundtrip() {
        let mac = HmacSha384;
        let key = HmacSha384Key::from([0u8; 48]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        assert_eq!(tag.len(), HMAC_SHA384_OUTPUT_SIZE);
        MacVerifier::verify(&mac, message, &tag, &key).unwrap();
    }

    #[test]
    fn test_hmac_sha384_verification_fails_on_wrong_message() {
        let mac = HmacSha384;
        let key = HmacSha384Key::from([0u8; 48]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        let result = MacVerifier::verify(&mac, b"Wrong message", &tag, &key);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    // ========================================
    // HMAC-SHA512 Tests
    // ========================================

    #[test]
    fn test_hmac_sha512_generate_verify_roundtrip() {
        let mac = HmacSha512;
        let key = HmacSha512Key::from([0u8; 64]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        assert_eq!(tag.len(), HMAC_SHA512_OUTPUT_SIZE);
        MacVerifier::verify(&mac, message, &tag, &key).unwrap();
    }

    #[test]
    fn test_hmac_sha512_verification_fails_on_wrong_message() {
        let mac = HmacSha512;
        let key = HmacSha512Key::from([0u8; 64]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        let result = MacVerifier::verify(&mac, b"Wrong message", &tag, &key);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    // ========================================
    // HmacAlgorithm Enum Tests
    // ========================================

    #[test]
    fn test_hmac_algorithm_sha256_generate_verify() {
        let mac = HmacAlgorithm::Sha256;
        let key = [0u8; 32];
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, key.as_slice()).unwrap();
        assert_eq!(tag.len(), mac.output_size());
        MacVerifier::verify(&mac, message, &tag, key.as_slice()).unwrap();
    }

    #[test]
    fn test_hmac_algorithm_sha384_generate_verify() {
        let mac = HmacAlgorithm::Sha384;
        let key = [0u8; 48];
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, key.as_slice()).unwrap();
        assert_eq!(tag.len(), mac.output_size());
        MacVerifier::verify(&mac, message, &tag, key.as_slice()).unwrap();
    }

    #[test]
    fn test_hmac_algorithm_sha512_generate_verify() {
        let mac = HmacAlgorithm::Sha512;
        let key = [0u8; 64];
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, key.as_slice()).unwrap();
        assert_eq!(tag.len(), mac.output_size());
        MacVerifier::verify(&mac, message, &tag, key.as_slice()).unwrap();
    }

    #[test]
    fn test_hmac_algorithm_verification_fails() {
        let mac = HmacAlgorithm::Sha256;
        let key = [0u8; 32];
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, key.as_slice()).unwrap();
        let result = MacVerifier::verify(&mac, b"Wrong", &tag, key.as_slice());
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_hmac_algorithm_output_sizes() {
        assert_eq!(HmacAlgorithm::Sha256.output_size(), 32);
        assert_eq!(HmacAlgorithm::Sha384.output_size(), 48);
        assert_eq!(HmacAlgorithm::Sha512.output_size(), 64);
    }

    #[test]
    fn test_hmac_algorithm_recommended_key_sizes() {
        assert_eq!(HmacAlgorithm::Sha256.recommended_key_size(), 32);
        assert_eq!(HmacAlgorithm::Sha384.recommended_key_size(), 48);
        assert_eq!(HmacAlgorithm::Sha512.recommended_key_size(), 64);
    }
}
