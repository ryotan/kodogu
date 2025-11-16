use crate::domain::{
    ciphertext::Ciphertext,
    keys::{PublicKey, SymmetricKey},
    plaintext::Plaintext,
    CryptoError,
};

/// Encryption abstraction
pub trait Encryptor: Send + Sync {
    /// Encrypt with symmetric key
    fn encrypt_symmetric(
        &self,
        plaintext: &Plaintext,
        key: &SymmetricKey,
    ) -> Result<Ciphertext, CryptoError>;

    /// Encrypt with public key (hybrid encryption: RSA + AES)
    fn encrypt_asymmetric(
        &self,
        plaintext: &Plaintext,
        public_key: &PublicKey,
    ) -> Result<Ciphertext, CryptoError>;
}
