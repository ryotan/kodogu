use suzunari_error::suzunari_error;

/// RustCrypto adapter layer errors
#[suzunari_error]
#[snafu(visibility(pub(crate)))]
pub enum RustCryptoError {
    #[snafu(display("AES cipher creation failed: {source}"))]
    AesCipherCreation { source: aes_gcm::aead::Error },

    #[snafu(display("AES encryption failed: {source}"))]
    AesEncryption { source: aes_gcm::aead::Error },

    #[snafu(display("AES decryption failed: {source}"))]
    AesDecryption { source: aes_gcm::aead::Error },

    #[snafu(display("RSA key generation failed: {source}"))]
    RsaKeyGeneration { source: rsa::Error },

    #[snafu(display("RSA encryption failed: {source}"))]
    RsaEncryption { source: rsa::Error },

    #[snafu(display("RSA decryption failed: {source}"))]
    RsaDecryption { source: rsa::Error },

    #[snafu(display("RSA public key parsing failed: {source}"))]
    RsaPublicKeyParsing { source: rsa::pkcs8::spki::Error },

    #[snafu(display("RSA private key parsing failed: {source}"))]
    RsaPrivateKeyParsing { source: rsa::pkcs8::Error },

    #[snafu(display("ECDSA key generation failed: {source}"))]
    EcdsaKeyGeneration { source: p256::elliptic_curve::Error },

    #[snafu(display("Ed25519 key generation failed: {reason}"))]
    Ed25519KeyGeneration { reason: String },

    #[snafu(display("Invalid PEM encoding: {source}"))]
    InvalidPemEncoding { source: std::string::FromUtf8Error },

    #[snafu(display("Invalid key size: {size} bytes"))]
    InvalidKeySize { size: usize },

    #[snafu(display("Ciphertext too short: {size} bytes"))]
    CiphertextTooShort { size: usize },
}
