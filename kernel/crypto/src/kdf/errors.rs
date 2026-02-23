use core::fmt;
use scrypt::errors::{
    InvalidOutputLen as ScryptInvalidOutputLen, InvalidParams as ScryptInvalidParams,
};
use suzunari_error::suzunari_error;

/// KDF algorithm identifier (for error messages).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KdfAlgorithms {
    HkdfSha256,
    HkdfSha384,
    Pbkdf2Sha256,
    Argon2id,
    Scrypt,
    ConcatKdfSha256,
    ConcatKdfSha384,
    X963KdfSha256,
    X963KdfSha384,
    BcryptPbkdf,
}

/// X963KDF internal error (validation errors from inline implementation).
///
/// Input validation errors for ANSI X9.63 KDF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X963KdfError {
    /// Secret is empty.
    NoSecret,
    /// Output buffer is empty.
    NoOutput,
    /// Input (secret + shared_info) is too large.
    InputOverflow,
    /// Requested output length is too large.
    CounterOverflow,
}

impl fmt::Display for X963KdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            X963KdfError::NoSecret => f.write_str("secret has zero length"),
            X963KdfError::NoOutput => f.write_str("output has zero length"),
            X963KdfError::InputOverflow => f.write_str("input is too big"),
            X963KdfError::CounterOverflow => f.write_str("requested key length is too big"),
        }
    }
}

impl std::error::Error for X963KdfError {}

/// KDF operation error.
#[suzunari_error]
#[snafu(visibility(pub), context(suffix(Error)))]
pub enum KdfError {
    /// Output length is 0 bytes (empty key is meaningless).
    #[snafu(display("{algorithm:?}: output length must be greater than 0"))]
    ZeroLengthOutput { algorithm: KdfAlgorithms },

    /// HKDF output length exceeds algorithm maximum (255 * HashLen).
    ///
    /// SHA-256: max 8160 bytes, SHA-384: max 12240 bytes.
    #[snafu(display(
        "{algorithm:?}: output length {length} bytes exceeds algorithm maximum (255 * hash length)"
    ))]
    HkdfOutputTooLong { algorithm: KdfAlgorithms, length: usize, source: hkdf::InvalidLength },

    /// Argon2id invalid parameters (includes salt/password/output validation).
    #[snafu(display("Argon2id: invalid parameters: {source}"))]
    Argon2InvalidParams { source: argon2::Error },

    /// Scrypt invalid parameters (invalid log_n / r / p combination, etc.).
    #[snafu(display("Scrypt: invalid parameters: {source}"))]
    ScryptInvalidParams { source: ScryptInvalidParams },

    /// Scrypt key derivation failed (output length exceeded).
    #[snafu(display("Scrypt: key derivation failed: {source}"))]
    ScryptDerivationFailed { source: ScryptInvalidOutputLen },

    /// ConcatKDF derivation failed (empty input or output length exceeded).
    #[snafu(display("{algorithm:?}: ConcatKDF derivation failed: {source}"))]
    ConcatKdfDerivationFailed { algorithm: KdfAlgorithms, source: concat_kdf::Error },

    /// X963KDF derivation failed (empty input or output length exceeded).
    #[snafu(display("{algorithm:?}: X963KDF derivation failed: {source}"))]
    X963KdfDerivationFailed { algorithm: KdfAlgorithms, source: X963KdfError },

    /// bcrypt-pbkdf derivation failed (invalid parameters or output length exceeded).
    ///
    /// Note: upstream `bcrypt_pbkdf::Error` distinguishes `InvalidRounds` / `InvalidParamLen` /
    /// `InvalidOutputLen`, but our `BcryptPbkdfParams` and zero-length guard already prevent
    /// these, so a single variant suffices.
    #[snafu(display("bcrypt-pbkdf: derivation failed: {source}"))]
    BcryptPbkdfDerivationFailed { source: bcrypt_pbkdf::Error },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x963kdf_error_display() {
        assert_eq!(X963KdfError::NoSecret.to_string(), "secret has zero length");
        assert_eq!(X963KdfError::NoOutput.to_string(), "output has zero length");
        assert_eq!(X963KdfError::InputOverflow.to_string(), "input is too big");
        assert_eq!(X963KdfError::CounterOverflow.to_string(), "requested key length is too big");
    }
}
