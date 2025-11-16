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
    #[snafu(display("AES cipher creation failed: {source}"))]
    AesCipherCreation { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("AES encryption failed: {source}"))]
    AesEncryption { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("AES decryption failed: {source}"))]
    AesDecryption { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("RSA key generation failed"))]
    RsaKeyGeneration { source: BoxedStackError },

    #[snafu(display("RSA encryption failed: {source}"))]
    RsaEncryption { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("RSA decryption failed: {source}"))]
    RsaDecryption { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("RSA public key parsing failed: {source}"))]
    RsaPublicKeyParsing { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("RSA private key parsing failed: {source}"))]
    RsaPrivateKeyParsing { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("ECDSA key generation failed: {source}"))]
    EcdsaKeyGeneration { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("Ed25519 key generation failed: {reason}"))]
    Ed25519KeyGeneration { reason: String },

    #[snafu(display("Invalid PEM encoding: {source}"))]
    InvalidPemEncoding { source: std::string::FromUtf8Error },

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
