use super::errors::{ConcatKdfDerivationFailedError, KdfAlgorithm, KdfError};
use super::traits::KeyDeriver;
use super::types::{DerivedKey, OutputLength};
use sha2::{Sha256, Sha384};
use suzunari_error::ResultExt;
use zeroize::Zeroizing;

// ========================================
// ConcatKDF-SHA256
// ========================================

/// ConcatKDF with SHA-256 (NIST SP 800-56A Rev. 3, Section 5.8.2).
///
/// Single-step hash KDF using a 32-bit counter (max output: (2^32 - 1) x 32 bytes).
/// Standard method for deriving encryption keys from ECDH shared secrets.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{ConcatKdfSha256, KeyDeriver, OutputLength};
///
/// let deriver = ConcatKdfSha256;
/// let key = deriver.derive(b"shared-secret", b"algorithm-id", OutputLength::new(32)).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct ConcatKdfSha256;

impl KeyDeriver for ConcatKdfSha256 {
    type Input = [u8];
    type Error = KdfError;

    fn derive(
        &self,
        input: &Self::Input,
        info: &[u8],
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error> {
        let length = length.get();
        let mut okm = Zeroizing::new(vec![0u8; length]);
        concat_kdf::derive_key_into::<Sha256>(input, info, &mut okm)
            .context(ConcatKdfDerivationFailedError { algorithm: KdfAlgorithm::ConcatKdfSha256 })?;
        Ok(DerivedKey::new(std::mem::take(&mut okm)))
    }
}

// ========================================
// ConcatKDF-SHA384
// ========================================

/// ConcatKDF with SHA-384 (NIST SP 800-56A Rev. 3, Section 5.8.2).
///
/// Single-step hash KDF. Recommended for use with P-384 and other 384-bit key agreements.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{ConcatKdfSha384, KeyDeriver, OutputLength};
///
/// let deriver = ConcatKdfSha384;
/// let key = deriver.derive(b"shared-secret", b"algorithm-id", OutputLength::new(48)).unwrap();
/// assert_eq!(key.len(), 48);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct ConcatKdfSha384;

impl KeyDeriver for ConcatKdfSha384 {
    type Input = [u8];
    type Error = KdfError;

    fn derive(
        &self,
        input: &Self::Input,
        info: &[u8],
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error> {
        let length = length.get();
        let mut okm = Zeroizing::new(vec![0u8; length]);
        concat_kdf::derive_key_into::<Sha384>(input, info, &mut okm)
            .context(ConcatKdfDerivationFailedError { algorithm: KdfAlgorithm::ConcatKdfSha384 })?;
        Ok(DerivedKey::new(std::mem::take(&mut okm)))
    }
}

// ========================================
// ConcatKdfAlgorithm Enum (runtime selection)
// ========================================

/// Enum for runtime ConcatKDF variant selection.
///
/// # Example
///
/// ```
/// use crypto::{ConcatKdfAlgorithm, KeyDeriver, OutputLength};
///
/// let deriver = ConcatKdfAlgorithm::Sha256;
/// let key = deriver.derive(b"shared-secret", b"algorithm-id", OutputLength::new(32)).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum ConcatKdfAlgorithm {
    /// ConcatKDF-SHA256
    #[default]
    Sha256,
    /// ConcatKDF-SHA384
    Sha384,
}

impl KeyDeriver for ConcatKdfAlgorithm {
    type Input = [u8];
    type Error = KdfError;

    fn derive(
        &self,
        input: &Self::Input,
        info: &[u8],
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error> {
        match self {
            ConcatKdfAlgorithm::Sha256 => KeyDeriver::derive(&ConcatKdfSha256, input, info, length),
            ConcatKdfAlgorithm::Sha384 => KeyDeriver::derive(&ConcatKdfSha384, input, info, length),
        }
    }
}

/// Marker trait for ConcatKDF family.
pub trait ConcatKdfDeriver: KeyDeriver<Input = [u8], Error = KdfError> {}

impl ConcatKdfDeriver for ConcatKdfSha256 {}
impl ConcatKdfDeriver for ConcatKdfSha384 {}
impl ConcatKdfDeriver for ConcatKdfAlgorithm {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdf::helpers::test_helpers::*;

    const LEN32: OutputLength = OutputLength::new(32);
    const LEN48: OutputLength = OutputLength::new(48);

    fn sample_secret() -> Vec<u8> {
        vec![0xba, 0x5e, 0xba, 0x11, 0xbe, 0xda, 0xbb, 0x1e]
    }

    // ========================================
    // ConcatKdfSha256 Tests
    // ========================================

    #[test]
    fn test_concat_kdf_sha256_output_length() {
        let key = ConcatKdfSha256.derive(b"secret", b"info", LEN32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_concat_kdf_sha256_deterministic() {
        let key1 = ConcatKdfSha256.derive(b"secret", b"info", LEN32).unwrap();
        let key2 = ConcatKdfSha256.derive(b"secret", b"info", LEN32).unwrap();
        assert_eq!(key1, key2, "same input must produce the same key");
    }

    #[test]
    fn test_concat_kdf_sha256_different_secrets_produce_different_keys() {
        let key1 = ConcatKdfSha256.derive(b"secret1", b"info", LEN32).unwrap();
        let key2 = ConcatKdfSha256.derive(b"secret2", b"info", LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_concat_kdf_sha256_different_info_produces_different_keys() {
        let secret = sample_secret();
        let key1 = ConcatKdfSha256.derive(&secret, b"encryption", LEN32).unwrap();
        let key2 = ConcatKdfSha256.derive(&secret, b"authentication", LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_concat_kdf_sha256_empty_info() {
        let key = ConcatKdfSha256.derive(b"secret", b"", LEN32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_concat_kdf_sha256_empty_secret_returns_error() {
        // concat_kdf treats zero-length secret as an error
        let result = ConcatKdfSha256.derive(b"", b"info", LEN32);
        assert!(matches!(result, Err(KdfError::ConcatKdfDerivationFailed { .. })));
    }

    // ========================================
    // ConcatKdfSha384 Tests
    // ========================================

    #[test]
    fn test_concat_kdf_sha384_output_length() {
        let key = ConcatKdfSha384.derive(b"secret", b"info", LEN48).unwrap();
        assert_key_length(&key, 48);
    }

    #[test]
    fn test_concat_kdf_sha384_deterministic() {
        let key1 = ConcatKdfSha384.derive(b"secret", b"info", LEN48).unwrap();
        let key2 = ConcatKdfSha384.derive(b"secret", b"info", LEN48).unwrap();
        assert_eq!(key1, key2, "same input must produce the same key");
    }

    // ========================================
    // ConcatKdfAlgorithm Enum Tests
    // ========================================

    #[test]
    fn test_concat_kdf_sha384_empty_secret_returns_error() {
        let result = ConcatKdfSha384.derive(b"", b"info", LEN48);
        assert!(matches!(result, Err(KdfError::ConcatKdfDerivationFailed { .. })));
    }

    #[test]
    fn test_concat_kdf_algorithm_sha256_matches_zst() {
        let secret = sample_secret();
        let key_enum = ConcatKdfAlgorithm::Sha256.derive(&secret, b"info", LEN32).unwrap();
        let key_zst = ConcatKdfSha256.derive(&secret, b"info", LEN32).unwrap();
        assert_eq!(key_enum, key_zst, "enum and ZST must produce the same key");
    }

    #[test]
    fn test_concat_kdf_algorithm_sha384_matches_zst() {
        let secret = sample_secret();
        let key_enum = ConcatKdfAlgorithm::Sha384.derive(&secret, b"info", LEN48).unwrap();
        let key_zst = ConcatKdfSha384.derive(&secret, b"info", LEN48).unwrap();
        assert_eq!(key_enum, key_zst, "enum and ZST must produce the same key");
    }
}
