use crate::domain::{
    KeyPair,
    CryptoError,
    values::{AsymmetricAlgorithm, SymmetricKey, SymmetricAlgorithm},
};

/// Key generation abstraction
pub trait KeyGenerator {
    /// Generate asymmetric key pair
    fn generate_key_pair(&self, algorithm: AsymmetricAlgorithm) -> Result<KeyPair, CryptoError>;

    /// Generate symmetric key
    fn generate_symmetric_key(&self, algorithm: SymmetricAlgorithm) -> Result<SymmetricKey, CryptoError>;

    /// Generate password-style string for symmetric encryption
    fn generate_password_string(&self, length: usize) -> Result<String, CryptoError>;
}
