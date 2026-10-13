use super::errors::{Argon2InvalidParamsError, KdfError};
use super::traits::PasswordDeriver;
use super::types::{Argon2Params, DerivedKey, OutputLength};
use argon2::{Algorithm, Argon2 as Argon2Impl, Version};
use suzunari_error::ResultExt;
use zeroize::Zeroizing;

/// Argon2id key derivation function (RFC 9106).
///
/// Recommended Argon2 variant combining side-channel resistance and time-memory trade-off resistance.
/// OWASP 2023 recommendation: memory=64MiB, time=3, parallelism=4.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{Argon2id, PasswordDeriver, Argon2Params, OutputLength};
///
/// let deriver = Argon2id;
/// // Low-cost parameters for testing (use OWASP_RECOMMENDED in production)
/// let params = Argon2Params::new(8, 1, 1);
/// let key = deriver.derive(b"password", b"saltsaltsaltsalt", &params, OutputLength::new(32)).unwrap();
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
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error> {
        let argon2 = Argon2Impl::new(Algorithm::Argon2id, Version::V0x13, params.inner().clone());
        let mut output = Zeroizing::new(vec![0u8; length.get()]);
        // argon2::Error variants are all input validation errors (salt too short/long,
        // password too long, output size, etc.), not runtime derivation failures.
        argon2
            .hash_password_into(password, salt, &mut output)
            .context(Argon2InvalidParamsError {})?;

        Ok(DerivedKey::new(std::mem::take(&mut output)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdf::helpers::test_helpers::*;

    const LEN32: OutputLength = OutputLength::new(32);

    // Low-cost parameters for fast testing
    fn fast_params() -> Argon2Params {
        Argon2Params::new(8, 1, 1)
    }

    #[test]
    fn test_argon2id_output_length() {
        let deriver = Argon2id;
        let key = deriver.derive(b"password", b"saltsaltsaltsalt", &fast_params(), LEN32).unwrap();
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
            LEN32,
        );
    }

    #[test]
    fn test_argon2id_different_passwords_produce_different_keys() {
        let deriver = Argon2id;
        let key1 = deriver
            .derive(b"password1", b"saltsaltsaltsalt", &fast_params(), LEN32)
            .unwrap();
        let key2 = deriver
            .derive(b"password2", b"saltsaltsaltsalt", &fast_params(), LEN32)
            .unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_argon2id_different_salts_produce_different_keys() {
        let deriver = Argon2id;
        let key1 = deriver.derive(b"password", b"saltsaltsaltsalt", &fast_params(), LEN32).unwrap();
        let key2 = deriver.derive(b"password", b"differentsaltstr", &fast_params(), LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_argon2id_variable_output_lengths() {
        let deriver = Argon2id;
        for length in [16, 32, 48, 64] {
            let ol = OutputLength::new(length);
            let key = deriver.derive(b"password", b"saltsaltsaltsalt", &fast_params(), ol).unwrap();
            assert_key_length(&key, length);
        }
    }

    #[test]
    fn test_argon2id_short_salt_returns_error() {
        // Argon2 requires salt >= 8 bytes (RFC 9106)
        let result = Argon2id.derive(b"password", b"short", &fast_params(), LEN32);
        assert!(matches!(result, Err(KdfError::Argon2InvalidParams { .. })));
    }
}
