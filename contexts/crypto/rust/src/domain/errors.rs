use suzunari_error::{suzunari_error, BoxedStackError};

/// Domain-level cryptographic errors
#[suzunari_error]
#[snafu(visibility(pub))]
pub enum CryptoError {
    #[snafu(display("Key generation failed"))]
    KeyGeneration { source: BoxedStackError },

    #[snafu(display("Encryption failed"))]
    Encryption { source: BoxedStackError },

    #[snafu(display("Decryption failed"))]
    Decryption { source: BoxedStackError },

    #[snafu(display("Invalid key format"))]
    InvalidKeyFormat { source: BoxedStackError },

    #[snafu(display("Key derivation failed"))]
    KeyDerivation { source: BoxedStackError },
}
