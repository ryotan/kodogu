use crate::CryptoError::{Decryption, Encryption, KeyDerivation, KeyGeneration};
use crate::domain::CryptoError;
use suzunari_error::{BoxedStackError, suzunari_error};

/// RustCrypto adapter layer errors
///
/// Each variant wraps the concrete library error type to preserve the full error chain.
#[suzunari_error]
#[snafu(visibility(pub), context(suffix(Error)))]
pub enum RustCryptoError {
    #[snafu(display("AES cipher creation failed"))]
    AesCipherCreation { source: aes_gcm::aes::cipher::InvalidLength },

    #[snafu(display("AES encryption failed"))]
    AesEncryption { source: aes_gcm::Error },

    #[snafu(display("AES decryption failed"))]
    AesDecryption { source: aes_gcm::Error },

    #[snafu(display("RSA key generation failed"))]
    RsaKeyGeneration { source: rsa::Error },

    #[snafu(display("RSA PKCS8 encoding failed"))]
    RsaPkcs8Encoding { source: pkcs8::Error },

    #[snafu(display("RSA SPKI encoding failed"))]
    RsaSpkiEncoding { source: pkcs8::spki::Error },

    #[snafu(display("RSA encryption failed"))]
    RsaEncryption { source: rsa::Error },

    #[snafu(display("RSA decryption failed"))]
    RsaDecryption { source: rsa::Error },

    #[snafu(display("RSA public key parsing failed"))]
    RsaPublicKeyParsing { source: pkcs8::spki::Error },

    #[snafu(display("Invalid PEM encoding (not a valid PKCS8 or SPKI block"))]
    InvalidPemEncoding { source: pkcs8::Error },

    #[snafu(display("RSA private key parsing failed"))]
    RsaPrivateKeyParsing { source: pkcs8::Error },

    #[snafu(display("ECDSA PKCS8 encoding failed"))]
    EcdsaPkcs8Encoding { source: pkcs8::Error },

    #[snafu(display("ECDSA SPKI encoding failed"))]
    EcdsaSpkiEncoding { source: pkcs8::spki::Error },

    #[snafu(display("Ed25519 PKCS8 encoding failed"))]
    Ed25519Pkcs8Encoding { source: pkcs8::Error },

    #[snafu(display("Ed25519 SPKI encoding failed"))]
    Ed25519SpkiEncoding { source: pkcs8::spki::Error },

    #[snafu(display("Invalid UTF-8 in PEM data"))]
    InvalidUtf8 { source: std::string::FromUtf8Error },

    #[snafu(display("Ciphertext too short: {size} bytes"))]
    CiphertextTooShort { size: usize },

    #[snafu(display("Invalid decrypted key size: {size} bytes (expected 32)"))]
    InvalidDecryptedKeySize { size: usize },

    #[snafu(display("PBKDF2 derivation failed: {reason}"))]
    Pbkdf2Derivation { reason: String },
}

impl From<RustCryptoError> for CryptoError {
    fn from(e: RustCryptoError) -> Self {
        match e {
            RustCryptoError::RsaKeyGeneration { .. }
            | RustCryptoError::RsaPkcs8Encoding { .. }
            | RustCryptoError::RsaSpkiEncoding { .. }
            | RustCryptoError::EcdsaPkcs8Encoding { .. }
            | RustCryptoError::EcdsaSpkiEncoding { .. }
            | RustCryptoError::Ed25519Pkcs8Encoding { .. }
            | RustCryptoError::Ed25519SpkiEncoding { .. } => {
                KeyGeneration { source: BoxedStackError::new(e) }
            }
            RustCryptoError::AesCipherCreation { .. }
            | RustCryptoError::AesEncryption { .. }
            | RustCryptoError::RsaEncryption { .. }
            | RustCryptoError::RsaPublicKeyParsing { .. }
            | RustCryptoError::InvalidUtf8 { .. } => Encryption { source: BoxedStackError::new(e) },
            RustCryptoError::AesDecryption { .. }
            | RustCryptoError::RsaDecryption { .. }
            | RustCryptoError::InvalidPemEncoding { .. }
            | RustCryptoError::RsaPrivateKeyParsing { .. }
            | RustCryptoError::CiphertextTooShort { .. }
            | RustCryptoError::InvalidDecryptedKeySize { .. } => {
                Decryption { source: BoxedStackError::new(e) }
            }
            RustCryptoError::Pbkdf2Derivation { .. } => {
                KeyDerivation { source: BoxedStackError::new(e) }
            }
        }
    }
}
