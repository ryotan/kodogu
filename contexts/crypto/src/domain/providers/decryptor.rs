use crate::domain::{
    ciphertext::Ciphertext,
    keys::{PrivateKey, SymmetricKey},
    plaintext::Plaintext,
    CryptoError,
};

/// Decryption abstraction
pub trait Decryptor: Send + Sync {
    /// Decrypt with symmetric key
    fn decrypt_symmetric(
        &self,
        ciphertext: &Ciphertext,
        key: &SymmetricKey,
    ) -> Result<Plaintext, CryptoError>;

    /// Decrypt with private key (hybrid decryption)
    fn decrypt_asymmetric(
        &self,
        ciphertext: &Ciphertext,
        private_key: &PrivateKey,
    ) -> Result<Plaintext, CryptoError>;
}
