use crate::domain::{
    CryptoError,
    ciphertext::Ciphertext,
    keys::{PublicKey, SymmetricKey},
    plaintext::Plaintext,
};

/// Encryption abstraction
pub trait Encryptor {
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
