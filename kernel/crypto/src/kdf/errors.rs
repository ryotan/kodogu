use core::fmt;
use scrypt::errors::{
    InvalidOutputLen as ScryptInvalidOutputLen, InvalidParams as ScryptInvalidParams,
};
use suzunari_error::suzunari_error;

/// KDF algorithm identifier (for error messages).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KdfAlgorithm {
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

impl fmt::Display for KdfAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KdfAlgorithm::HkdfSha256 => f.write_str("HKDF-SHA256"),
            KdfAlgorithm::HkdfSha384 => f.write_str("HKDF-SHA384"),
            KdfAlgorithm::Pbkdf2Sha256 => f.write_str("PBKDF2-SHA256"),
            KdfAlgorithm::Argon2id => f.write_str("Argon2id"),
            KdfAlgorithm::Scrypt => f.write_str("scrypt"),
            KdfAlgorithm::ConcatKdfSha256 => f.write_str("ConcatKDF-SHA256"),
            KdfAlgorithm::ConcatKdfSha384 => f.write_str("ConcatKDF-SHA384"),
            KdfAlgorithm::X963KdfSha256 => f.write_str("X963KDF-SHA256"),
            KdfAlgorithm::X963KdfSha384 => f.write_str("X963KDF-SHA384"),
            KdfAlgorithm::BcryptPbkdf => f.write_str("bcrypt-pbkdf"),
        }
    }
}

/// X963KDF internal error (validation errors from inline implementation).
///
/// Input validation errors for ANSI X9.63 KDF.
#[suzunari_error]
#[suzu(visibility(pub(crate)), context(suffix(Error)))]
pub enum X963KdfError {
    /// Secret is empty.
    #[suzu(display("secret has zero length"))]
    NoSecret,
    /// Output buffer is empty.
    #[suzu(display("output has zero length"))]
    NoOutput,
    /// Input (secret + shared_info) is too large.
    #[suzu(display("input is too big"))]
    InputOverflow,
    /// Requested output length is too large.
    #[suzu(display("requested key length is too big"))]
    CounterOverflow,
}

/// KDF operation error.
#[suzunari_error]
#[suzu(visibility(pub), context(suffix(Error)))]
pub enum KdfError {
    /// HKDF output length exceeds algorithm maximum (255 * HashLen).
    ///
    /// SHA-256: max 8160 bytes, SHA-384: max 12240 bytes.
    #[suzu(display(
        "{algorithm}: output length {length} bytes exceeds algorithm maximum (255 * hash length)"
    ))]
    HkdfOutputTooLong {
        algorithm: KdfAlgorithm,
        length: usize,
        #[suzu(from)]
        source: hkdf::InvalidLength,
    },

    /// Argon2id invalid parameters (includes salt/password/output validation).
    #[suzu(display("Argon2id: invalid parameters: {source}"))]
    Argon2InvalidParams {
        #[suzu(from)]
        source: argon2::Error,
    },

    /// Scrypt invalid parameters (invalid log_n / r / p combination, etc.).
    #[suzu(display("Scrypt: invalid parameters: {source}"))]
    ScryptInvalidParams { source: ScryptInvalidParams },

    /// Scrypt key derivation failed (output length exceeded).
    #[suzu(display("Scrypt: key derivation failed: {source}"))]
    ScryptDerivationFailed { source: ScryptInvalidOutputLen },

    /// ConcatKDF derivation failed (empty input or output length exceeded).
    #[suzu(display("{algorithm}: ConcatKDF derivation failed: {source}"))]
    ConcatKdfDerivationFailed {
        algorithm: KdfAlgorithm,
        #[suzu(from)]
        source: concat_kdf::Error,
    },

    /// X963KDF derivation failed (empty input or output length exceeded).
    #[suzu(display("{algorithm}: X963KDF derivation failed: {source}"))]
    X963KdfDerivationFailed { algorithm: KdfAlgorithm, source: X963KdfError },

    /// bcrypt-pbkdf derivation failed (invalid parameters or output length exceeded).
    ///
    /// Note: upstream `bcrypt_pbkdf::Error` distinguishes `InvalidRounds` / `InvalidParamLen` /
    /// `InvalidOutputLen`, but our `BcryptPbkdfParams` and zero-length guard already prevent
    /// these, so a single variant suffices.
    #[suzu(display("bcrypt-pbkdf: derivation failed: {source}"))]
    BcryptPbkdfDerivationFailed {
        #[suzu(from)]
        source: bcrypt_pbkdf::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use suzunari_error::ensure;

    #[test]
    fn test_kdf_algorithms_display() {
        assert_eq!(KdfAlgorithm::HkdfSha256.to_string(), "HKDF-SHA256");
        assert_eq!(KdfAlgorithm::HkdfSha384.to_string(), "HKDF-SHA384");
        assert_eq!(KdfAlgorithm::Pbkdf2Sha256.to_string(), "PBKDF2-SHA256");
        assert_eq!(KdfAlgorithm::Argon2id.to_string(), "Argon2id");
        assert_eq!(KdfAlgorithm::Scrypt.to_string(), "scrypt");
        assert_eq!(KdfAlgorithm::ConcatKdfSha256.to_string(), "ConcatKDF-SHA256");
        assert_eq!(KdfAlgorithm::ConcatKdfSha384.to_string(), "ConcatKDF-SHA384");
        assert_eq!(KdfAlgorithm::X963KdfSha256.to_string(), "X963KDF-SHA256");
        assert_eq!(KdfAlgorithm::X963KdfSha384.to_string(), "X963KDF-SHA384");
        assert_eq!(KdfAlgorithm::BcryptPbkdf.to_string(), "bcrypt-pbkdf");
    }

    #[test]
    fn test_x963kdf_error_display() {
        // セレクタ経由で構築（suzunari_error が location フィールドを追加するため直接構築不可）
        fn make_err(f: fn() -> Result<(), X963KdfError>) -> String {
            f().unwrap_err().to_string()
        }
        assert_eq!(
            make_err(|| {
                ensure!(false, NoSecretError);
                Ok(())
            }),
            "secret has zero length"
        );
        assert_eq!(
            make_err(|| {
                ensure!(false, NoOutputError);
                Ok(())
            }),
            "output has zero length"
        );
        assert_eq!(
            make_err(|| {
                ensure!(false, InputOverflowError);
                Ok(())
            }),
            "input is too big"
        );
        assert_eq!(
            make_err(|| {
                ensure!(false, CounterOverflowError);
                Ok(())
            }),
            "requested key length is too big"
        );
    }
}
