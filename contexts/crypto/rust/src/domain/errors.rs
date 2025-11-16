use suzunari_error::{suzunari_error, BoxedStackError};

/// Domain-level cryptographic errors
#[suzunari_error]
#[snafu(visibility(pub))]
pub enum CryptoError {
    #[snafu(transparent)]
    KeyGeneration { source: KeyGenerationError },

    #[snafu(transparent)]
    Encryption { source: EncryptionError },

    #[snafu(transparent)]
    Decryption { source: DecryptionError },

    #[snafu(transparent)]
    InvalidKeyFormat { source: InvalidKeyFormatError },

    #[snafu(transparent)]
    KeyDerivation { source: KeyDerivationError },
}

#[suzunari_error]
#[snafu(visibility(pub))]
#[snafu(display("Key generation failed"))]
pub struct KeyGenerationError {
    pub source: BoxedStackError,
}

#[suzunari_error]
#[snafu(visibility(pub))]
#[snafu(display("Encryption failed"))]
pub struct EncryptionError {
    pub source: BoxedStackError,
}

#[suzunari_error]
#[snafu(visibility(pub))]
#[snafu(display("Decryption failed"))]
pub struct DecryptionError {
    pub source: BoxedStackError,
}

#[suzunari_error]
#[snafu(visibility(pub))]
#[snafu(display("Invalid key format"))]
pub struct InvalidKeyFormatError {
    pub source: BoxedStackError,
}

#[suzunari_error]
#[snafu(visibility(pub))]
#[snafu(display("Key derivation failed"))]
pub struct KeyDerivationError {
    pub source: BoxedStackError,
}
