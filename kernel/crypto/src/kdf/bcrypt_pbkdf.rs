use super::errors::{BcryptPbkdfDerivationFailedError, KdfError};
use super::traits::PasswordDeriver;
use super::types::{BcryptPbkdfParams, DerivedKey, OutputLength};
use suzunari_error::ResultExt;
use zeroize::Zeroizing;

/// bcrypt-pbkdf key derivation function (OpenBSD bcrypt_pbkdf).
///
/// Blowfish-based PBKDF. Used for OpenSSH key file encryption.
/// bcrypt-PBKDF differs from regular bcrypt used for password hashing.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{BcryptPbkdf, PasswordDeriver, BcryptPbkdfParams, OutputLength};
///
/// let deriver = BcryptPbkdf;
/// let params = BcryptPbkdfParams::new(4);
/// let key = deriver.derive(b"password", b"salt", &params, OutputLength::new(32)).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct BcryptPbkdf;

impl PasswordDeriver for BcryptPbkdf {
    type Params = BcryptPbkdfParams;
    type Error = KdfError;

    fn derive(
        &self,
        password: &[u8],
        salt: &[u8],
        params: &Self::Params,
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error> {
        let length = length.get();
        let mut output = Zeroizing::new(vec![0u8; length]);
        bcrypt_pbkdf::bcrypt_pbkdf(password, salt, params.rounds(), &mut output)
            .context(BcryptPbkdfDerivationFailedError {})?;
        Ok(DerivedKey::new(std::mem::take(&mut output)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdf::helpers::test_helpers::*;

    const LEN32: OutputLength = OutputLength::new(32);

    fn fast_params() -> BcryptPbkdfParams {
        // Low round count for fast testing (minimum is 1)
        BcryptPbkdfParams::new(4)
    }

    #[test]
    fn test_bcrypt_pbkdf_output_length() {
        let key = BcryptPbkdf.derive(b"password", b"salt", &fast_params(), LEN32).unwrap();
        assert_key_length(&key, 32);
    }

    #[test]
    fn test_bcrypt_pbkdf_deterministic() {
        assert_password_deriver_deterministic(
            &BcryptPbkdf,
            b"password",
            b"salt",
            &fast_params(),
            LEN32,
        );
    }

    #[test]
    fn test_bcrypt_pbkdf_different_passwords_produce_different_keys() {
        let key1 = BcryptPbkdf.derive(b"password1", b"salt", &fast_params(), LEN32).unwrap();
        let key2 = BcryptPbkdf.derive(b"password2", b"salt", &fast_params(), LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_bcrypt_pbkdf_different_salts_produce_different_keys() {
        let key1 = BcryptPbkdf.derive(b"password", b"salt1", &fast_params(), LEN32).unwrap();
        let key2 = BcryptPbkdf.derive(b"password", b"salt2", &fast_params(), LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_bcrypt_pbkdf_different_rounds_produce_different_keys() {
        let params1 = BcryptPbkdfParams::new(4);
        let params2 = BcryptPbkdfParams::new(8);
        let key1 = BcryptPbkdf.derive(b"password", b"salt", &params1, LEN32).unwrap();
        let key2 = BcryptPbkdf.derive(b"password", b"salt", &params2, LEN32).unwrap();
        assert_different_keys(&key1, &key2);
    }

    #[test]
    fn test_bcrypt_pbkdf_variable_output_lengths() {
        for length in [16, 32, 48, 64] {
            let ol = OutputLength::new(length);
            let key = BcryptPbkdf.derive(b"password", b"salt", &fast_params(), ol).unwrap();
            assert_key_length(&key, length);
        }
    }

    #[test]
    #[should_panic(expected = "rounds must be greater than 0")]
    fn test_bcrypt_pbkdf_params_zero_rounds_panics() {
        BcryptPbkdfParams::new(0);
    }
}
