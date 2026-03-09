use super::errors::{KeyInitFailedError, MacAlgorithms, MacError, VerificationFailedError};
use super::helpers::validate_key_length;
use super::traits::{Mac, MacGenerator, MacVerifier};
use super::types::{CmacAes128Key, CmacAes256Key, MacTag};
use aes::{Aes128, Aes256};
use cmac::{Cmac, Mac as CmacMac};
use suzunari_error::ResultExt;

// CMAC 出力サイズ（AES ブロックサイズ = 16 bytes）
const CMAC_OUTPUT_SIZE: usize = 16;
// CMAC 鍵サイズ
const CMAC_AES128_KEY_SIZE: usize = 16;
const CMAC_AES256_KEY_SIZE: usize = 32;

// ========================================
// CMAC-AES-128
// ========================================

/// CMAC-AES-128 message authentication code.
///
/// Uses AES-128 block cipher with 128-bit (16-byte) output.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{CmacAes128, MacGenerator, MacVerifier, CmacAes128Key};
///
/// let mac = CmacAes128;
/// let key = CmacAes128Key::from([0u8; 16]);
/// let message = b"Hello, World!";
///
/// let tag = mac.generate(message, &key).unwrap();
/// mac.verify(message, &tag, &key).unwrap();
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct CmacAes128;

type CmacAes128Impl = Cmac<Aes128>;

impl MacGenerator for CmacAes128 {
    type Key = CmacAes128Key;
    type Error = MacError;

    fn generate(&self, message: &[u8], key: &Self::Key) -> Result<MacTag, Self::Error> {
        // CmacAes128Key は常に 16 バイト
        let mut mac = CmacAes128Impl::new_from_slice(key.as_bytes())
            .context(KeyInitFailedError { algorithm: MacAlgorithms::CmacAes128 })?;
        mac.update(message);
        let result = mac.finalize();
        Ok(MacTag::from(result.into_bytes().to_vec()))
    }
}

impl MacVerifier for CmacAes128 {
    type Key = CmacAes128Key;
    type Error = MacError;

    fn verify(&self, message: &[u8], tag: &MacTag, key: &Self::Key) -> Result<(), Self::Error> {
        let mut mac = CmacAes128Impl::new_from_slice(key.as_bytes())
            .context(KeyInitFailedError { algorithm: MacAlgorithms::CmacAes128 })?;
        mac.update(message);
        // verify_slice は constant-time 比較を行う
        mac.verify_slice(tag.as_bytes())
            .context(VerificationFailedError { algorithm: MacAlgorithms::CmacAes128 })
    }
}

impl Mac for CmacAes128 {
    type Key = CmacAes128Key;
    type Error = MacError;
}

/// Marker trait for CMAC-AES family.
pub trait CmacAesCipher: Mac<Error = MacError> {}

impl CmacAesCipher for CmacAes128 {}

// ========================================
// CMAC-AES-256
// ========================================

/// CMAC-AES-256 message authentication code.
///
/// Uses AES-256 block cipher with 128-bit (16-byte) output.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{CmacAes256, MacGenerator, MacVerifier, CmacAes256Key};
///
/// let mac = CmacAes256;
/// let key = CmacAes256Key::from([0u8; 32]);
/// let message = b"Hello, World!";
///
/// let tag = mac.generate(message, &key).unwrap();
/// mac.verify(message, &tag, &key).unwrap();
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct CmacAes256;

type CmacAes256Impl = Cmac<Aes256>;

impl MacGenerator for CmacAes256 {
    type Key = CmacAes256Key;
    type Error = MacError;

    fn generate(&self, message: &[u8], key: &Self::Key) -> Result<MacTag, Self::Error> {
        // CmacAes256Key は常に 32 バイト
        let mut mac = CmacAes256Impl::new_from_slice(key.as_bytes())
            .context(KeyInitFailedError { algorithm: MacAlgorithms::CmacAes256 })?;
        mac.update(message);
        let result = mac.finalize();
        Ok(MacTag::from(result.into_bytes().to_vec()))
    }
}

impl MacVerifier for CmacAes256 {
    type Key = CmacAes256Key;
    type Error = MacError;

    fn verify(&self, message: &[u8], tag: &MacTag, key: &Self::Key) -> Result<(), Self::Error> {
        let mut mac = CmacAes256Impl::new_from_slice(key.as_bytes())
            .context(KeyInitFailedError { algorithm: MacAlgorithms::CmacAes256 })?;
        mac.update(message);
        mac.verify_slice(tag.as_bytes())
            .context(VerificationFailedError { algorithm: MacAlgorithms::CmacAes256 })
    }
}

impl Mac for CmacAes256 {
    type Key = CmacAes256Key;
    type Error = MacError;
}

impl CmacAesCipher for CmacAes256 {}

// ========================================
// CmacAes Enum (ランタイム選択)
// ========================================

/// CMAC-AES algorithm variants for runtime selection.
///
/// # Example
///
/// ```
/// use crypto::{CmacAes, MacGenerator, MacVerifier};
///
/// let mac = CmacAes::Aes128;
/// let key = [0u8; 16];
/// let message = b"Hello, World!";
///
/// let tag = mac.generate(message, &key).unwrap();
/// mac.verify(message, &tag, &key).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmacAes {
    /// CMAC-AES-128 (16-byte key)
    Aes128,
    /// CMAC-AES-256 (32-byte key)
    Aes256,
}

impl CmacAes {
    /// Returns the expected key size in bytes.
    pub fn key_size(&self) -> usize {
        match self {
            CmacAes::Aes128 => CMAC_AES128_KEY_SIZE,
            CmacAes::Aes256 => CMAC_AES256_KEY_SIZE,
        }
    }

    /// Returns the output size in bytes (always 16 for CMAC-AES).
    pub fn output_size(&self) -> usize {
        CMAC_OUTPUT_SIZE
    }
}

impl MacGenerator for CmacAes {
    type Key = [u8];
    type Error = MacError;

    fn generate(&self, message: &[u8], key: &Self::Key) -> Result<MacTag, Self::Error> {
        match self {
            CmacAes::Aes128 => {
                let key_array =
                    validate_key_length::<CMAC_AES128_KEY_SIZE>(key, MacAlgorithms::CmacAes128)?;
                let mut mac = CmacAes128Impl::new_from_slice(&key_array)
                    .context(KeyInitFailedError { algorithm: MacAlgorithms::CmacAes128 })?;
                mac.update(message);
                Ok(MacTag::from(mac.finalize().into_bytes().to_vec()))
            }
            CmacAes::Aes256 => {
                let key_array =
                    validate_key_length::<CMAC_AES256_KEY_SIZE>(key, MacAlgorithms::CmacAes256)?;
                let mut mac = CmacAes256Impl::new_from_slice(&key_array)
                    .context(KeyInitFailedError { algorithm: MacAlgorithms::CmacAes256 })?;
                mac.update(message);
                Ok(MacTag::from(mac.finalize().into_bytes().to_vec()))
            }
        }
    }
}

impl MacVerifier for CmacAes {
    type Key = [u8];
    type Error = MacError;

    fn verify(&self, message: &[u8], tag: &MacTag, key: &Self::Key) -> Result<(), Self::Error> {
        match self {
            CmacAes::Aes128 => {
                let key_array =
                    validate_key_length::<CMAC_AES128_KEY_SIZE>(key, MacAlgorithms::CmacAes128)?;
                let mut mac = CmacAes128Impl::new_from_slice(&key_array)
                    .context(KeyInitFailedError { algorithm: MacAlgorithms::CmacAes128 })?;
                mac.update(message);
                mac.verify_slice(tag.as_bytes())
                    .context(VerificationFailedError { algorithm: MacAlgorithms::CmacAes128 })
            }
            CmacAes::Aes256 => {
                let key_array =
                    validate_key_length::<CMAC_AES256_KEY_SIZE>(key, MacAlgorithms::CmacAes256)?;
                let mut mac = CmacAes256Impl::new_from_slice(&key_array)
                    .context(KeyInitFailedError { algorithm: MacAlgorithms::CmacAes256 })?;
                mac.update(message);
                mac.verify_slice(tag.as_bytes())
                    .context(VerificationFailedError { algorithm: MacAlgorithms::CmacAes256 })
            }
        }
    }
}

impl Mac for CmacAes {
    type Key = [u8];
    type Error = MacError;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // CMAC-AES-128 Tests
    // ========================================

    #[test]
    fn test_cmac_aes128_generate_verify_roundtrip() {
        let mac = CmacAes128;
        let key = CmacAes128Key::from([0u8; 16]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        assert_eq!(tag.len(), CMAC_OUTPUT_SIZE);
        MacVerifier::verify(&mac, message, &tag, &key).unwrap();
    }

    #[test]
    fn test_cmac_aes128_verification_fails_on_wrong_message() {
        let mac = CmacAes128;
        let key = CmacAes128Key::from([0u8; 16]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        let result = MacVerifier::verify(&mac, b"Wrong message", &tag, &key);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_cmac_aes128_verification_fails_on_wrong_key() {
        let mac = CmacAes128;
        let key1 = CmacAes128Key::from([0u8; 16]);
        let key2 = CmacAes128Key::from([1u8; 16]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key1).unwrap();
        let result = MacVerifier::verify(&mac, message, &tag, &key2);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_cmac_aes128_verification_fails_on_tampered_tag() {
        let mac = CmacAes128;
        let key = CmacAes128Key::from([0u8; 16]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        let mut tampered = tag.into_bytes();
        tampered[0] ^= 0xFF;
        let tampered_tag = MacTag::from(tampered);

        let result = MacVerifier::verify(&mac, message, &tampered_tag, &key);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_cmac_aes128_different_keys_produce_different_tags() {
        let mac = CmacAes128;
        let key1 = CmacAes128Key::from([0u8; 16]);
        let key2 = CmacAes128Key::from([1u8; 16]);
        let message = b"Hello, World!";

        let tag1 = MacGenerator::generate(&mac, message, &key1).unwrap();
        let tag2 = MacGenerator::generate(&mac, message, &key2).unwrap();
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_cmac_aes128_different_messages_produce_different_tags() {
        let mac = CmacAes128;
        let key = CmacAes128Key::from([0u8; 16]);

        let tag1 = MacGenerator::generate(&mac, b"Message 1", &key).unwrap();
        let tag2 = MacGenerator::generate(&mac, b"Message 2", &key).unwrap();
        assert_ne!(tag1, tag2);
    }

    #[test]
    fn test_cmac_aes128_empty_message() {
        let mac = CmacAes128;
        let key = CmacAes128Key::from([0u8; 16]);

        let tag = MacGenerator::generate(&mac, b"", &key).unwrap();
        assert_eq!(tag.len(), CMAC_OUTPUT_SIZE);
        MacVerifier::verify(&mac, b"", &tag, &key).unwrap();
    }

    // ========================================
    // CMAC-AES-256 Tests
    // ========================================

    #[test]
    fn test_cmac_aes256_generate_verify_roundtrip() {
        let mac = CmacAes256;
        let key = CmacAes256Key::from([0u8; 32]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        assert_eq!(tag.len(), CMAC_OUTPUT_SIZE);
        MacVerifier::verify(&mac, message, &tag, &key).unwrap();
    }

    #[test]
    fn test_cmac_aes256_verification_fails_on_wrong_message() {
        let mac = CmacAes256;
        let key = CmacAes256Key::from([0u8; 32]);
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, &key).unwrap();
        let result = MacVerifier::verify(&mac, b"Wrong message", &tag, &key);
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    // ========================================
    // CmacAes Enum Tests
    // ========================================

    #[test]
    fn test_cmac_aes_enum_aes128_generate_verify() {
        let mac = CmacAes::Aes128;
        let key = [0u8; 16];
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, key.as_slice()).unwrap();
        assert_eq!(tag.len(), mac.output_size());
        MacVerifier::verify(&mac, message, &tag, key.as_slice()).unwrap();
    }

    #[test]
    fn test_cmac_aes_enum_aes256_generate_verify() {
        let mac = CmacAes::Aes256;
        let key = [0u8; 32];
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, key.as_slice()).unwrap();
        assert_eq!(tag.len(), mac.output_size());
        MacVerifier::verify(&mac, message, &tag, key.as_slice()).unwrap();
    }

    #[test]
    fn test_cmac_aes_enum_verification_fails() {
        let mac = CmacAes::Aes128;
        let key = [0u8; 16];
        let message = b"Hello, World!";

        let tag = MacGenerator::generate(&mac, message, key.as_slice()).unwrap();
        let result = MacVerifier::verify(&mac, b"Wrong", &tag, key.as_slice());
        assert!(matches!(result, Err(MacError::VerificationFailed { .. })));
    }

    #[test]
    fn test_cmac_aes_enum_invalid_key_length() {
        let mac = CmacAes::Aes128;
        let key = [0u8; 15]; // 15 bytes instead of 16

        let result = MacGenerator::generate(&mac, b"Hello", key.as_slice());
        assert!(matches!(result, Err(MacError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_cmac_aes_key_sizes() {
        assert_eq!(CmacAes::Aes128.key_size(), 16);
        assert_eq!(CmacAes::Aes256.key_size(), 32);
    }

    #[test]
    fn test_cmac_aes_output_sizes() {
        assert_eq!(CmacAes::Aes128.output_size(), 16);
        assert_eq!(CmacAes::Aes256.output_size(), 16);
    }
}
