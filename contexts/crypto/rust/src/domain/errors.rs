use suzunari_error::suzunari_error;

/// Domain-level cryptographic errors
#[suzunari_error]
pub enum CryptoError {
    #[snafu(display("Key generation failed: {message}"))]
    KeyGeneration { message: String },

    #[snafu(display("Encryption failed: {message}"))]
    Encryption { message: String },

    #[snafu(display("Decryption failed: {message}"))]
    Decryption { message: String },

    #[snafu(display("Invalid key format: {message}"))]
    InvalidKeyFormat { message: String },

    #[snafu(display("Key derivation failed: {message}"))]
    KeyDerivation { message: String },
}
