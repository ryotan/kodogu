use snafu::Snafu;

/// Domain-level cryptographic errors
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CryptoError {
    #[snafu(display("Key generation failed: {reason}"))]
    KeyGeneration { reason: String },

    #[snafu(display("Encryption failed: {reason}"))]
    Encryption { reason: String },

    #[snafu(display("Decryption failed: {reason}"))]
    Decryption { reason: String },

    #[snafu(display("Invalid key format: {reason}"))]
    InvalidKeyFormat { reason: String },

    #[snafu(display("Key derivation failed: {reason}"))]
    KeyDerivation { reason: String },
}
