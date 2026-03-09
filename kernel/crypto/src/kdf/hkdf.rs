use super::errors::{HkdfOutputTooLongError, KdfAlgorithm, KdfError};
use super::traits::KeyDeriver;
use super::types::{DerivedKey, HkdfInput, OutputLength};
use hkdf::Hkdf;
use sha2::{Sha256, Sha384};
use suzunari_error::ResultExt;
use zeroize::Zeroizing;

// ========================================
// HKDF-SHA256
// ========================================

/// HKDF with SHA-256 (RFC 5869).
///
/// Maximum output length: 255 x 32 = 8160 bytes.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{HkdfSha256, KeyDeriver, HkdfInput, OutputLength};
///
/// let deriver = HkdfSha256;
/// let input = HkdfInput::without_salt(b"shared secret".to_vec());
/// let key = deriver.derive(&input, b"encryption", OutputLength::new(32)).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HkdfSha256;

impl KeyDeriver for HkdfSha256 {
    type Input = HkdfInput;
    type Error = KdfError;

    fn derive(
        &self,
        input: &Self::Input,
        info: &[u8],
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error> {
        let length = length.get();
        let salt = input.salt();
        let hk = Hkdf::<Sha256>::new(salt, input.ikm());
        let mut okm = Zeroizing::new(vec![0u8; length]);
        hk.expand(info, &mut okm)
            .context(HkdfOutputTooLongError { algorithm: KdfAlgorithm::HkdfSha256, length })?;
        Ok(DerivedKey::new(std::mem::take(&mut okm)))
    }
}

// ========================================
// HKDF-SHA384
// ========================================

/// HKDF with SHA-384 (RFC 5869).
///
/// Maximum output length: 255 x 48 = 12,240 bytes.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{HkdfSha384, KeyDeriver, HkdfInput, OutputLength};
///
/// let deriver = HkdfSha384;
/// let input = HkdfInput::without_salt(b"shared secret".to_vec());
/// let key = deriver.derive(&input, b"authentication", OutputLength::new(48)).unwrap();
/// assert_eq!(key.len(), 48);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HkdfSha384;

impl KeyDeriver for HkdfSha384 {
    type Input = HkdfInput;
    type Error = KdfError;

    fn derive(
        &self,
        input: &Self::Input,
        info: &[u8],
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error> {
        let length = length.get();
        let salt = input.salt();
        let hk = Hkdf::<Sha384>::new(salt, input.ikm());
        let mut okm = Zeroizing::new(vec![0u8; length]);
        hk.expand(info, &mut okm)
            .context(HkdfOutputTooLongError { algorithm: KdfAlgorithm::HkdfSha384, length })?;
        Ok(DerivedKey::new(std::mem::take(&mut okm)))
    }
}

// ========================================
// HkdfAlgorithm Enum (runtime selection)
// ========================================

/// Enum for runtime HKDF variant selection.
///
/// Use when the hash algorithm is determined at runtime.
///
/// # Example
///
/// ```
/// use crypto::{HkdfAlgorithm, KeyDeriver, HkdfInput, OutputLength};
///
/// let deriver = HkdfAlgorithm::Sha256;
/// let input = HkdfInput::without_salt(b"shared secret".to_vec());
/// let key = deriver.derive(&input, b"encryption", OutputLength::new(32)).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum HkdfAlgorithm {
    /// HKDF-SHA256 (max output: 8160 bytes)
    #[default]
    Sha256,
    /// HKDF-SHA384 (max output: 12240 bytes)
    Sha384,
}

impl KeyDeriver for HkdfAlgorithm {
    type Input = HkdfInput;
    type Error = KdfError;

    fn derive(
        &self,
        input: &Self::Input,
        info: &[u8],
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error> {
        match self {
            HkdfAlgorithm::Sha256 => KeyDeriver::derive(&HkdfSha256, input, info, length),
            HkdfAlgorithm::Sha384 => KeyDeriver::derive(&HkdfSha384, input, info, length),
        }
    }
}

/// Marker trait for HKDF family.
///
/// Enables generic programming over HKDF variants (SHA-256, SHA-384).
pub trait HkdfDeriver: KeyDeriver<Input = HkdfInput, Error = KdfError> {}

impl HkdfDeriver for HkdfSha256 {}
impl HkdfDeriver for HkdfSha384 {}
impl HkdfDeriver for HkdfAlgorithm {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdf::helpers::test_helpers::*;

    const LEN32: OutputLength = OutputLength::new(32);
    const LEN48: OutputLength = OutputLength::new(48);

    fn sample_input() -> HkdfInput {
        HkdfInput::new(vec![0x0bu8; 22], Some(vec![0x00, 0x01, 0x02, 0x03]))
    }

    // ========================================
    // HKDF-SHA256 Tests
    // ========================================

    #[test]
    fn test_hkdf_sha256_output_length() {
        let deriver = HkdfSha256;
        let input = sample_input();
        let key = deriver.derive(&input, b"info", LEN32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_hkdf_sha256_deterministic() {
        let deriver = HkdfSha256;
        let input = sample_input();
        assert_key_deriver_deterministic(&deriver, &input, b"info", LEN32);
    }

    #[test]
    fn test_hkdf_sha256_different_ikm_produces_different_keys() {
        let deriver = HkdfSha256;
        let input1 = HkdfInput::without_salt(vec![0x01u8; 32]);
        let input2 = HkdfInput::without_salt(vec![0x02u8; 32]);
        let key1 = deriver.derive(&input1, b"info", LEN32).unwrap();
        let key2 = deriver.derive(&input2, b"info", LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_hkdf_sha256_different_info_produces_different_keys() {
        let deriver = HkdfSha256;
        let input = sample_input();
        let key1 = deriver.derive(&input, b"encryption", LEN32).unwrap();
        let key2 = deriver.derive(&input, b"authentication", LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_hkdf_sha256_different_salt_produces_different_keys() {
        let deriver = HkdfSha256;
        let ikm = vec![0x0bu8; 22];
        let input1 = HkdfInput::new(ikm.clone(), Some(vec![0x00u8; 13]));
        let input2 = HkdfInput::new(ikm, Some(vec![0x01u8; 13]));
        let key1 = deriver.derive(&input1, b"info", LEN32).unwrap();
        let key2 = deriver.derive(&input2, b"info", LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_hkdf_sha256_without_salt() {
        let deriver = HkdfSha256;
        let input = HkdfInput::without_salt(vec![0x0bu8; 22]);
        let key = deriver.derive(&input, b"info", LEN32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_hkdf_sha256_empty_ikm() {
        // RFC 5869 allows zero-length IKM (e.g., test case 3).
        let input = HkdfInput::without_salt(vec![]);
        let key = HkdfSha256.derive(&input, b"info", LEN32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_hkdf_sha256_output_too_long() {
        let deriver = HkdfSha256;
        let input = sample_input();
        // SHA-256 max output length is 255 * 32 = 8160 bytes
        let result = deriver.derive(&input, b"info", OutputLength::new(8161));
        assert!(matches!(result, Err(KdfError::HkdfOutputTooLong { .. })));
    }

    // ========================================
    // HKDF-SHA384 Tests
    // ========================================

    #[test]
    fn test_hkdf_sha384_output_length() {
        let deriver = HkdfSha384;
        let input = sample_input();
        let key = deriver.derive(&input, b"info", LEN48).unwrap();
        assert_key_length(&key, 48);
    }

    #[test]
    fn test_hkdf_sha384_deterministic() {
        let deriver = HkdfSha384;
        let input = sample_input();
        assert_key_deriver_deterministic(&deriver, &input, b"info", LEN48);
    }

    #[test]
    fn test_hkdf_sha384_output_too_long() {
        let deriver = HkdfSha384;
        let input = sample_input();
        // SHA-384 max output length is 255 * 48 = 12,240 bytes
        let result = deriver.derive(&input, b"info", OutputLength::new(12241));
        assert!(matches!(result, Err(KdfError::HkdfOutputTooLong { .. })));
    }

    // ========================================
    // HkdfAlgorithm Enum Tests
    // ========================================

    #[test]
    fn test_hkdf_algorithm_sha256() {
        let deriver = HkdfAlgorithm::Sha256;
        let input = sample_input();
        let key = deriver.derive(&input, b"info", LEN32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_hkdf_algorithm_sha384() {
        let deriver = HkdfAlgorithm::Sha384;
        let input = sample_input();
        let key = deriver.derive(&input, b"info", LEN48).unwrap();
        assert_key_length(&key, 48);
    }

    #[test]
    fn test_hkdf_algorithm_sha256_matches_zst() {
        let input = sample_input();
        let key_enum = HkdfAlgorithm::Sha256.derive(&input, b"info", LEN32).unwrap();
        let key_zst = HkdfSha256.derive(&input, b"info", LEN32).unwrap();
        assert_eq!(key_enum, key_zst, "enum and ZST must produce the same key");
    }

    #[test]
    fn test_hkdf_algorithm_sha384_matches_zst() {
        let input = sample_input();
        let key_enum = HkdfAlgorithm::Sha384.derive(&input, b"info", LEN48).unwrap();
        let key_zst = HkdfSha384.derive(&input, b"info", LEN48).unwrap();
        assert_eq!(key_enum, key_zst, "enum and ZST must produce the same key");
    }
}
