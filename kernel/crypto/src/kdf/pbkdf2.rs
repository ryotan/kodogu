use super::errors::{KdfAlgorithms, KdfError, ZeroLengthOutputError};
use super::traits::PasswordDeriver;
use super::types::{DerivedKey, Pbkdf2Params};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use snafu::ensure;

/// PBKDF2 with HMAC-SHA256 (RFC 8018).
///
/// OWASP 2023 recommendation: minimum 600,000 iterations.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{Pbkdf2Sha256, PasswordDeriver, Pbkdf2Params};
///
/// let deriver = Pbkdf2Sha256;
/// let key = deriver.derive(b"password", b"random_salt", &Pbkdf2Params::OWASP_RECOMMENDED, 32).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Pbkdf2Sha256;

impl PasswordDeriver for Pbkdf2Sha256 {
    type Params = Pbkdf2Params;
    type Error = KdfError;

    fn derive(
        &self,
        password: &[u8],
        salt: &[u8],
        params: &Self::Params,
        length: usize,
    ) -> Result<DerivedKey, Self::Error> {
        ensure!(length > 0, ZeroLengthOutputError { algorithm: KdfAlgorithms::Pbkdf2Sha256 });

        let mut output = vec![0u8; length];
        pbkdf2_hmac::<Sha256>(password, salt, params.iterations(), &mut output);
        Ok(DerivedKey::new(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdf::helpers::test_helpers::*;

    fn fast_params() -> Pbkdf2Params {
        // Low iteration count for fast testing
        Pbkdf2Params::new(1)
    }

    #[test]
    fn test_pbkdf2_sha256_output_length() {
        let deriver = Pbkdf2Sha256;
        let key = deriver.derive(b"password", b"salt", &fast_params(), 32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_pbkdf2_sha256_deterministic() {
        let deriver = Pbkdf2Sha256;
        assert_password_deriver_deterministic(&deriver, b"password", b"salt", &fast_params(), 32);
    }

    #[test]
    fn test_pbkdf2_sha256_different_passwords_produce_different_keys() {
        let deriver = Pbkdf2Sha256;
        let key1 = deriver.derive(b"password1", b"salt", &fast_params(), 32).unwrap();
        let key2 = deriver.derive(b"password2", b"salt", &fast_params(), 32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_pbkdf2_sha256_different_salts_produce_different_keys() {
        let deriver = Pbkdf2Sha256;
        let key1 = deriver.derive(b"password", b"salt1", &fast_params(), 32).unwrap();
        let key2 = deriver.derive(b"password", b"salt2", &fast_params(), 32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_pbkdf2_sha256_different_iterations_produce_different_keys() {
        let deriver = Pbkdf2Sha256;
        let key1 = deriver.derive(b"password", b"salt", &Pbkdf2Params::new(1), 32).unwrap();
        let key2 = deriver.derive(b"password", b"salt", &Pbkdf2Params::new(2), 32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_pbkdf2_sha256_variable_output_lengths() {
        let deriver = Pbkdf2Sha256;
        for length in [16, 24, 32, 48, 64] {
            let key = deriver.derive(b"password", b"salt", &fast_params(), length).unwrap();
            assert_key_length(&key, length);
        }
    }

    #[test]
    fn test_pbkdf2_sha256_empty_password() {
        let deriver = Pbkdf2Sha256;
        let key = deriver.derive(b"", b"salt", &fast_params(), 32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    #[should_panic(expected = "iterations must be greater than 0")]
    fn test_pbkdf2_params_zero_iterations_panics() {
        Pbkdf2Params::new(0);
    }

    #[test]
    fn test_pbkdf2_sha256_zero_length_returns_error() {
        let deriver = Pbkdf2Sha256;
        let result = deriver.derive(b"password", b"salt", &fast_params(), 0);
        assert!(matches!(result, Err(KdfError::ZeroLengthOutput { .. })));
    }
}
