use suzunari_error::{suzunari_error, BoxedStackError};
use crate::domain::{
    CryptoError, DecryptionErrorSnafu, EncryptionErrorSnafu, KeyDerivationErrorSnafu, KeyGenerationErrorSnafu,
};

/// RustCrypto adapter layer errors
///
/// Note: External library errors (aes_gcm, rsa, etc.) are wrapped in Box<dyn Error>
/// because they don't implement snafu::Error trait required for source chaining.
#[suzunari_error]
#[snafu(visibility(pub(crate)))]
pub enum RustCryptoError {
    #[snafu(display("AES cipher creation failed"))]
    AesCipherCreation { source: BoxedStackError },

    #[snafu(display("AES encryption failed"))]
    AesEncryption { source: BoxedStackError },

    #[snafu(display("AES decryption failed"))]
    AesDecryption { source: BoxedStackError },

    #[snafu(display("RSA key generation failed"))]
    RsaKeyGeneration { source: BoxedStackError },

    #[snafu(display("RSA encryption failed"))]
    RsaEncryption { source: BoxedStackError },

    #[snafu(display("RSA decryption failed"))]
    RsaDecryption { source: BoxedStackError },

    #[snafu(display("RSA public key parsing failed"))]
    RsaPublicKeyParsing { source: BoxedStackError },

    #[snafu(display("RSA private key parsing failed"))]
    RsaPrivateKeyParsing { source: BoxedStackError },

    #[snafu(display("ECDSA key generation failed"))]
    EcdsaKeyGeneration { source: BoxedStackError },

    #[snafu(display("Ed25519 key generation failed"))]
    Ed25519KeyGeneration { source: BoxedStackError },

    #[snafu(display("Invalid PEM encoding"))]
    InvalidPemEncoding { source: BoxedStackError },

    #[snafu(display("Invalid key size: {size} bytes"))]
    InvalidKeySize { size: usize },

    #[snafu(display("Ciphertext too short: {size} bytes"))]
    CiphertextTooShort { size: usize },

    #[snafu(display("PBKDF2 derivation failed: {reason}"))]
    Pbkdf2Derivation { reason: String },
}

impl From<RustCryptoError> for CryptoError {
    fn from(e: RustCryptoError) -> Self {
        match &e {
            RustCryptoError::AesCipherCreation { .. }
            | RustCryptoError::AesEncryption { .. }
            | RustCryptoError::RsaEncryption { .. }
            | RustCryptoError::RsaPublicKeyParsing { .. }
            | RustCryptoError::InvalidPemEncoding { .. }
            | RustCryptoError::InvalidKeySize { .. } => EncryptionErrorSnafu {
                source: BoxedStackError::new(e),
            }
            .build()
            .into(),
            RustCryptoError::AesDecryption { .. }
            | RustCryptoError::RsaDecryption { .. }
            | RustCryptoError::RsaPrivateKeyParsing { .. }
            | RustCryptoError::CiphertextTooShort { .. } => DecryptionErrorSnafu {
                source: BoxedStackError::new(e),
            }
            .build()
            .into(),
            RustCryptoError::RsaKeyGeneration { .. }
            | RustCryptoError::EcdsaKeyGeneration { .. }
            | RustCryptoError::Ed25519KeyGeneration { .. } => KeyGenerationErrorSnafu {
                source: BoxedStackError::new(e),
            }
            .build()
            .into(),
            RustCryptoError::Pbkdf2Derivation { .. } => KeyDerivationErrorSnafu {
                source: BoxedStackError::new(e),
            }
            .build()
            .into(),
        }
    }
}
