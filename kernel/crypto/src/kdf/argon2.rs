use super::errors::{Argon2InvalidParamsError, KdfAlgorithms, KdfError, ZeroLengthOutputError};
use super::traits::PasswordDeriver;
use super::types::{Argon2Params, DerivedKey};
use argon2::{Algorithm, Argon2 as Argon2Impl, Params as Argon2InternalParams, Version};
use snafu::{ResultExt, ensure};

/// Argon2id key derivation function (RFC 9106).
///
/// Recommended Argon2 variant combining side-channel resistance and time-memory trade-off resistance.
/// OWASP 2023 recommendation: memory=64MiB, time=3, parallelism=4.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{Argon2id, PasswordDeriver, Argon2Params};
///
/// let deriver = Argon2id;
/// // Low-cost parameters for testing (use OWASP_RECOMMENDED in production)
/// let params = Argon2Params::new(8, 1, 1);
/// let key = deriver.derive(b"password", b"saltsaltsaltsalt", &params, 32).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Argon2id;

impl PasswordDeriver for Argon2id {
    type Params = Argon2Params;
    type Error = KdfError;

    fn derive(
        &self,
        password: &[u8],
        salt: &[u8],
        params: &Self::Params,
        length: usize,
    ) -> Result<DerivedKey, Self::Error> {
        ensure!(length > 0, ZeroLengthOutputError { algorithm: KdfAlgorithms::Argon2id });

        let internal_params = Argon2InternalParams::new(
            params.memory_kib(),
            params.time(),
            params.parallelism(),
            Some(length),
        )
        .context(Argon2InvalidParamsError {})?;

        let argon2 = Argon2Impl::new(Algorithm::Argon2id, Version::V0x13, internal_params);
        let mut output = vec![0u8; length];
        // argon2::Error variants are all input validation errors (salt too short/long,
        // password too long, output size, etc.), not runtime derivation failures.
        argon2
            .hash_password_into(password, salt, &mut output)
            .context(Argon2InvalidParamsError {})?;

        Ok(DerivedKey::new(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdf::helpers::test_helpers::*;

    // Low-cost parameters for fast testing
    fn fast_params() -> Argon2Params {
        Argon2Params::new(8, 1, 1)
    }

    #[test]
    fn test_argon2id_output_length() {
        let deriver = Argon2id;
        let key = deriver.derive(b"password", b"saltsaltsaltsalt", &fast_params(), 32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_argon2id_deterministic() {
        let deriver = Argon2id;
        assert_password_deriver_deterministic(
            &deriver,
            b"password",
            b"saltsaltsaltsalt",
            &fast_params(),
            32,
        );
    }

    #[test]
    fn test_argon2id_different_passwords_produce_different_keys() {
        let deriver = Argon2id;
        let key1 = deriver.derive(b"password1", b"saltsaltsaltsalt", &fast_params(), 32).unwrap();
        let key2 = deriver.derive(b"password2", b"saltsaltsaltsalt", &fast_params(), 32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_argon2id_different_salts_produce_different_keys() {
        let deriver = Argon2id;
        let key1 = deriver.derive(b"password", b"saltsaltsaltsalt", &fast_params(), 32).unwrap();
        let key2 = deriver.derive(b"password", b"differentsaltstr", &fast_params(), 32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_argon2id_variable_output_lengths() {
        let deriver = Argon2id;
        for length in [16, 32, 48, 64] {
            let key = deriver
                .derive(b"password", b"saltsaltsaltsalt", &fast_params(), length)
                .unwrap();
            assert_key_length(&key, length);
        }
    }

    #[test]
    #[should_panic(expected = "memory_kib must be greater than 0")]
    fn test_argon2_params_zero_memory_panics() {
        Argon2Params::new(0, 1, 1);
    }

    #[test]
    #[should_panic(expected = "time must be greater than 0")]
    fn test_argon2_params_zero_time_panics() {
        Argon2Params::new(8, 0, 1);
    }

    #[test]
    #[should_panic(expected = "parallelism must be greater than 0")]
    fn test_argon2_params_zero_parallelism_panics() {
        Argon2Params::new(8, 1, 0);
    }

    #[test]
    fn test_argon2id_zero_length_returns_error() {
        let deriver = Argon2id;
        let result = deriver.derive(b"password", b"saltsaltsaltsalt", &fast_params(), 0);
        assert!(matches!(result, Err(KdfError::ZeroLengthOutput { .. })));
    }
}
