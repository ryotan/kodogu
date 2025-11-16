use suzunari_error::suzunari_error;

/// RustCrypto adapter layer errors
#[suzunari_error]
#[snafu(visibility(pub(crate)))]
pub enum RustCryptoError {
    #[snafu(display("AES cipher creation failed: {source}"))]
    AesCipherCreation { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("AES encryption failed: {source}"))]
    AesEncryption { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("AES decryption failed: {source}"))]
    AesDecryption { source: Box<dyn std::error::Error + Send + Sync> },

    #[snafu(display("RSA key generation failed: {source}"))]
    RsaKeyGeneration { source: Box<dyn std::error::Error + Send + Sync> },

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
}
