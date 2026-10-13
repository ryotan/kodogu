use super::errors::{KdfError, ScryptDerivationFailedError};
use super::traits::PasswordDeriver;
use super::types::{DerivedKey, OutputLength, ScryptParams};
use scrypt::scrypt as scrypt_fn;
use suzunari_error::ResultExt;
use zeroize::Zeroizing;

/// Scrypt key derivation function (RFC 7914).
///
/// Memory-hard KDF.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{Scrypt, PasswordDeriver, ScryptParams, OutputLength};
///
/// let deriver = Scrypt;
/// // Low-cost parameters for testing (use `ScryptParams::recommended()` in production)
/// let params = ScryptParams::new(4, 1, 1);
/// let key = deriver.derive(b"password", b"salt", &params, OutputLength::new(32)).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Scrypt;

impl PasswordDeriver for Scrypt {
    type Params = ScryptParams;
    type Error = KdfError;

    fn derive(
        &self,
        password: &[u8],
        salt: &[u8],
        params: &Self::Params,
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error> {
        let mut output = Zeroizing::new(vec![0u8; length.get()]);
        scrypt_fn(password, salt, params.inner(), &mut output)
            .context(ScryptDerivationFailedError {})?;

        Ok(DerivedKey::new(std::mem::take(&mut output)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdf::helpers::test_helpers::*;

    const LEN32: OutputLength = OutputLength::new(32);

    // Low-cost parameters for fast testing (log_n=4 -> N=16)
    fn fast_params() -> ScryptParams {
        ScryptParams::new(4, 1, 1)
    }

    #[test]
    fn test_scrypt_output_length() {
        let deriver = Scrypt;
        let key = deriver.derive(b"password", b"salt", &fast_params(), LEN32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_scrypt_deterministic() {
        let deriver = Scrypt;
        assert_password_deriver_deterministic(
            &deriver,
            b"password",
            b"salt",
            &fast_params(),
            LEN32,
        );
    }

    #[test]
    fn test_scrypt_different_passwords_produce_different_keys() {
        let deriver = Scrypt;
        let key1 = deriver.derive(b"password1", b"salt", &fast_params(), LEN32).unwrap();
        let key2 = deriver.derive(b"password2", b"salt", &fast_params(), LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_scrypt_different_salts_produce_different_keys() {
        let deriver = Scrypt;
        let key1 = deriver.derive(b"password", b"salt1", &fast_params(), LEN32).unwrap();
        let key2 = deriver.derive(b"password", b"salt2", &fast_params(), LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_scrypt_variable_output_lengths() {
        let deriver = Scrypt;
        for length in [1, 9, 16, 32, 48, 64, 65, 128] {
            let ol = OutputLength::new(length);
            let key = deriver.derive(b"password", b"salt", &fast_params(), ol).unwrap();
            assert_key_length(&key, length);
        }
    }

    #[test]
    fn test_scrypt_empty_password() {
        let deriver = Scrypt;
        let key = deriver.derive(b"", b"salt", &fast_params(), LEN32).unwrap();
        assert_key_length(&key, 32);
    }
}
