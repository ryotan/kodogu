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
use suzunari_error::{ResultExt, ensure};
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

trait X963Hash: Digest + FixedOutputReset {
    const MAX_INPUT_LEN: u128;
}

impl X963Hash for Sha256 {
    const MAX_INPUT_LEN: u128 = (1u128 << 61) - 1;
}

impl X963Hash for Sha384 {
    const MAX_INPUT_LEN: u128 = (1u128 << 125) - 1;
}

fn validate_lengths<D: X963Hash>(
    secret_len: u128,
    shared_info_len: u128,
    output_len: u128,
) -> Result<(), X963KdfError> {
    ensure!(secret_len > 0, NoSecretError);
    ensure!(output_len > 0, NoOutputError);
    let max_output_len = <D as OutputSizeUser>::output_size() as u128 * u128::from(u32::MAX);
    let input_len = secret_len.checked_add(shared_info_len).and_then(|len| len.checked_add(4));
    ensure!(input_len.is_some_and(|len| len < D::MAX_INPUT_LEN), InputOverflowError);
    ensure!(output_len < max_output_len, CounterOverflowError);
    Ok(())
}

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
    D: X963Hash,
{
    validate_lengths::<D>(secret.len() as u128, shared_info.len() as u128, output.len() as u128)?;
    let hash_usize = <D as OutputSizeUser>::output_size();

    let mut hasher = D::new();
    let mut counter: u32 = 1;

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

    macro_rules! length_cases {
        ($($name:ident: ($hash:ty, $secret:expr, $info:expr, $output:expr, $error:expr)),+ $(,)?) => {
            $(#[test]
            fn $name() {
                let expected: Option<&str> = $error;
                assert_eq!(
                    validate_lengths::<$hash>($secret, $info, $output).err().map(|error| error.to_string()),
                    expected.map(str::to_owned),
                );
            })+
        };
    }

    length_cases! {
        sha256_accepts_old_input_boundary: (Sha256, 32 * u32::MAX as u128, 0, 32, None),
        sha384_accepts_old_input_boundary: (Sha384, 48 * u32::MAX as u128, 0, 48, None),
        sha256_accepts_below_hash_limit: (Sha256, (1u128 << 61) - 6, 0, 32, None),
        sha256_rejects_at_hash_limit: (Sha256, (1u128 << 61) - 5, 0, 32, Some("input is too big")),
        sha256_rejects_above_hash_limit: (Sha256, (1u128 << 61) - 4, 0, 32, Some("input is too big")),
        sha384_accepts_below_hash_limit: (Sha384, 1, (1u128 << 125) - 7, 48, None),
        sha384_rejects_at_hash_limit: (Sha384, 1, (1u128 << 125) - 6, 48, Some("input is too big")),
        sha384_rejects_above_hash_limit: (Sha384, 1, (1u128 << 125) - 5, 48, Some("input is too big")),
        sha256_accepts_below_output_limit: (Sha256, 1, 0, 32 * u32::MAX as u128 - 1, None),
        sha256_rejects_at_output_limit: (Sha256, 1, 0, 32 * u32::MAX as u128, Some("requested key length is too big")),
        sha384_accepts_below_output_limit: (Sha384, 1, 0, 48 * u32::MAX as u128 - 1, None),
        sha384_rejects_at_output_limit: (Sha384, 1, 0, 48 * u32::MAX as u128, Some("requested key length is too big")),
        rejects_empty_secret_first: (Sha256, 0, u128::MAX, 0, Some("secret has zero length")),
        rejects_empty_output_before_input_limit: (Sha384, u128::MAX, 1, 0, Some("output has zero length")),
        rejects_combined_length_overflow: (Sha384, u128::MAX, 1, 48, Some("input is too big")),
        rejects_counter_length_overflow: (Sha384, u128::MAX - 3, 0, 48, Some("input is too big")),
    }

    proptest::proptest! {
        #[test]
        fn input_validation_is_independent_of_secret_info_partition(
            total in 1u128..u128::MAX - 4,
            partition in proptest::prelude::any::<u128>(),
        ) {
            let secret = 1 + partition % total;
            let info = total - secret;
            proptest::prop_assert_eq!(
                validate_lengths::<Sha256>(secret, info, 1).map_err(|error| error.to_string()),
                validate_lengths::<Sha256>(total, 0, 1).map_err(|error| error.to_string()),
            );
            proptest::prop_assert_eq!(
                validate_lengths::<Sha384>(secret, info, 1).map_err(|error| error.to_string()),
                validate_lengths::<Sha384>(total, 0, 1).map_err(|error| error.to_string()),
            );
        }
    }
}
