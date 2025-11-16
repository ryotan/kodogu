use crate::domain::{
    CryptoError,
    ciphertext::Ciphertext,
    keys::{PrivateKey, SymmetricKey},
    plaintext::Plaintext,
};

/// Decryption abstraction
pub trait Decryptor {
    /// Decrypt with symmetric key
    fn decrypt_symmetric(&self, ciphertext: &Ciphertext, key: &SymmetricKey) -> Result<Plaintext, CryptoError>;

    /// Decrypt with private key (hybrid decryption)
    fn decrypt_asymmetric(&self, ciphertext: &Ciphertext, private_key: &PrivateKey) -> Result<Plaintext, CryptoError>;
}
