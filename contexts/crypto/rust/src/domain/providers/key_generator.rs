use crate::domain::{
    CryptoError, KeyPair,
    algorithms::{AsymmetricAlgorithm, SymmetricAlgorithm},
    keys::SymmetricKey,
};

/// Key generation abstraction
pub trait KeyGenerator: Send + Sync {
    /// Generate asymmetric key pair
    fn generate_key_pair(&self, algorithm: AsymmetricAlgorithm) -> Result<KeyPair, CryptoError>;

    /// Generate symmetric key
    fn generate_symmetric_key(
        &self,
        algorithm: SymmetricAlgorithm,
    ) -> Result<SymmetricKey, CryptoError>;

    /// Generate password-style string for symmetric encryption
    fn generate_password_string(&self, length: usize) -> Result<String, CryptoError>;
}
