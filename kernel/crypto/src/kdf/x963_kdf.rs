use super::errors::{
    CounterOverflowError, InputOverflowError, KdfAlgorithm, KdfError, NoOutputError, NoSecretError,
    X963KdfDerivationFailedError, X963KdfError,
};
use super::traits::KeyDeriver;
use super::types::{DerivedKey, OutputLength};
use sha2::{
    Digest, Sha256, Sha384,
    digest::{FixedOutputReset, OutputSizeUser},
};
use snafu::{ResultExt, ensure};
use zeroize::Zeroizing;

// ========================================
// Inline X963KDF implementation (ANSI X9.63-2011)
// ========================================
//
// Algorithm: K_i = Hash(Z || counter || SharedInfo)
//   counter: 4-byte big-endian, starting from 1
//   Z: shared secret
//   SharedInfo: context information (info)
//
// Difference from ConcatKDF: field order differs
//   ConcatKDF: Hash(counter || Z || OtherInfo)
//   X963KDF:   Hash(Z || counter || SharedInfo)

/// Inline implementation of X9.63 KDF.
///
/// The `ansi-x963-kdf` crate (v0.1) depends on `digest 0.11.0-rc.0`,
/// which conflicts with workspace `sha2 0.10` (`digest 0.10`), so the algorithm is implemented inline.
/// TODO: Replace with `ansi-x963-kdf` once `sha2` upgrades to `digest 0.11+` stable.
fn x963_kdf_inner<D>(
    secret: &[u8],
    shared_info: &[u8],
    output: &mut [u8],
) -> Result<(), X963KdfError>
where
    D: Digest + FixedOutputReset,
{
    ensure!(!secret.is_empty(), NoSecretError);
    ensure!(!output.is_empty(), NoOutputError);

    let hash_len = <D as OutputSizeUser>::output_size() as u64;
    let max_len = hash_len * (u32::MAX as u64);

    // Approximate check: secret + shared_info + 4 (counter) < hashmaxlen
    ensure!(max_len > secret.len() as u64 + shared_info.len() as u64 + 4, InputOverflowError);

    // keydatalen < hashlen × (2^32 - 1)
    ensure!(max_len > output.len() as u64, CounterOverflowError);

    let mut hasher = D::new();
    let mut counter: u32 = 1;
    let hash_usize = hash_len as usize;

    let chunks: Vec<_> = output.chunks_mut(hash_usize).collect();
    let num_chunks = chunks.len();
    for (i, chunk) in chunks.into_iter().enumerate() {
        // X963: Hash(Z || counter || SharedInfo)
        Digest::update(&mut hasher, secret);
        Digest::update(&mut hasher, counter.to_be_bytes());
        Digest::update(&mut hasher, shared_info);
        chunk.copy_from_slice(&hasher.finalize_reset()[..chunk.len()]);
        // Only increment counter if there are more chunks to process.
        // The pre-check already ensures counter won't overflow for the required number of blocks.
        if i + 1 < num_chunks {
            counter += 1;
        }
    }

    Ok(())
}

// ========================================
// X963KDF-SHA256
// ========================================

/// X9.63 KDF with SHA-256 (ANSI X9.63-2011).
///
/// Used for SEC 1v2 and JWA (RFC 7518) ECDH-ES key agreement.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{X963KdfSha256, KeyDeriver, OutputLength};
///
/// let deriver = X963KdfSha256;
/// let key = deriver.derive(b"shared-secret", b"shared-info", OutputLength::new(32)).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct X963KdfSha256;

impl KeyDeriver for X963KdfSha256 {
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
        x963_kdf_inner::<Sha256>(input, info, &mut okm)
            .context(X963KdfDerivationFailedError { algorithm: KdfAlgorithm::X963KdfSha256 })?;
        Ok(DerivedKey::new(std::mem::take(&mut okm)))
    }
}

// ========================================
// X963KDF-SHA384
// ========================================

/// X9.63 KDF with SHA-384 (ANSI X9.63-2011).
///
/// Recommended for use with P-384 key agreement.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{X963KdfSha384, KeyDeriver, OutputLength};
///
/// let deriver = X963KdfSha384;
/// let key = deriver.derive(b"shared-secret", b"shared-info", OutputLength::new(48)).unwrap();
/// assert_eq!(key.len(), 48);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct X963KdfSha384;

impl KeyDeriver for X963KdfSha384 {
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
        x963_kdf_inner::<Sha384>(input, info, &mut okm)
            .context(X963KdfDerivationFailedError { algorithm: KdfAlgorithm::X963KdfSha384 })?;
        Ok(DerivedKey::new(std::mem::take(&mut okm)))
    }
}

// ========================================
// X963KdfAlgorithm Enum (runtime selection)
// ========================================

/// Enum for runtime X963KDF variant selection.
///
/// # Example
///
/// ```
/// use crypto::{X963KdfAlgorithm, KeyDeriver, OutputLength};
///
/// let deriver = X963KdfAlgorithm::Sha256;
/// let key = deriver.derive(b"shared-secret", b"shared-info", OutputLength::new(32)).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum X963KdfAlgorithm {
    /// X963KDF-SHA256
    #[default]
    Sha256,
    /// X963KDF-SHA384
    Sha384,
}

impl KeyDeriver for X963KdfAlgorithm {
    type Input = [u8];
    type Error = KdfError;

    fn derive(
        &self,
        input: &Self::Input,
        info: &[u8],
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error> {
        match self {
            X963KdfAlgorithm::Sha256 => KeyDeriver::derive(&X963KdfSha256, input, info, length),
            X963KdfAlgorithm::Sha384 => KeyDeriver::derive(&X963KdfSha384, input, info, length),
        }
    }
}

/// Marker trait for X963KDF family.
pub trait X963KdfDeriver: KeyDeriver<Input = [u8], Error = KdfError> {}

impl X963KdfDeriver for X963KdfSha256 {}
impl X963KdfDeriver for X963KdfSha384 {}
impl X963KdfDeriver for X963KdfAlgorithm {}

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
    // X963KdfSha256 Tests
    // ========================================

    #[test]
    fn test_x963_kdf_sha256_output_length() {
        let key = X963KdfSha256.derive(b"secret", b"info", LEN32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_x963_kdf_sha256_deterministic() {
        let key1 = X963KdfSha256.derive(b"secret", b"info", LEN32).unwrap();
        let key2 = X963KdfSha256.derive(b"secret", b"info", LEN32).unwrap();
        assert_eq!(key1, key2, "same input must produce the same key");
    }

    #[test]
    fn test_x963_kdf_sha256_different_secrets_produce_different_keys() {
        let key1 = X963KdfSha256.derive(b"secret1", b"info", LEN32).unwrap();
        let key2 = X963KdfSha256.derive(b"secret2", b"info", LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_x963_kdf_sha256_different_info_produces_different_keys() {
        let secret = sample_secret();
        let key1 = X963KdfSha256.derive(&secret, b"encryption", LEN32).unwrap();
        let key2 = X963KdfSha256.derive(&secret, b"authentication", LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_x963_kdf_sha256_empty_info() {
        let key = X963KdfSha256.derive(b"secret", b"", LEN32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_x963_kdf_sha256_empty_secret_returns_error() {
        let result = X963KdfSha256.derive(b"", b"info", LEN32);
        assert!(matches!(result, Err(KdfError::X963KdfDerivationFailed { .. })));
    }

    // ========================================
    // X963KdfSha384 Tests
    // ========================================

    #[test]
    fn test_x963_kdf_sha384_output_length() {
        let key = X963KdfSha384.derive(b"secret", b"info", LEN48).unwrap();
        assert_key_length(&key, 48);
    }

    #[test]
    fn test_x963_kdf_sha384_deterministic() {
        let key1 = X963KdfSha384.derive(b"secret", b"info", LEN48).unwrap();
        let key2 = X963KdfSha384.derive(b"secret", b"info", LEN48).unwrap();
        assert_eq!(key1, key2, "same input must produce the same key");
    }

    #[test]
    fn test_x963_kdf_sha384_empty_secret_returns_error() {
        let result = X963KdfSha384.derive(b"", b"info", LEN48);
        assert!(matches!(result, Err(KdfError::X963KdfDerivationFailed { .. })));
    }

    // ========================================
    // X963KdfAlgorithm Enum Tests
    // ========================================

    #[test]
    fn test_x963_kdf_algorithm_sha256_matches_zst() {
        let secret = sample_secret();
        let key_enum = X963KdfAlgorithm::Sha256.derive(&secret, b"info", LEN32).unwrap();
        let key_zst = X963KdfSha256.derive(&secret, b"info", LEN32).unwrap();
        assert_eq!(key_enum, key_zst, "enum and ZST must produce the same key");
    }

    #[test]
    fn test_x963_kdf_algorithm_sha384_matches_zst() {
        let secret = sample_secret();
        let key_enum = X963KdfAlgorithm::Sha384.derive(&secret, b"info", LEN48).unwrap();
        let key_zst = X963KdfSha384.derive(&secret, b"info", LEN48).unwrap();
        assert_eq!(key_enum, key_zst, "enum and ZST must produce the same key");
    }
}
