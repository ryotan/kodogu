use crate::domain::{keys::SymmetricKey, CryptoError};

/// Key derivation abstraction (e.g., PBKDF2)
pub trait KeyDeriver: Send + Sync {
    /// Derive symmetric key from password string
    fn derive_key(
        &self,
        password: &str,
        salt: &[u8],
        key_size: usize,
    ) -> Result<SymmetricKey, CryptoError>;

    /// Derive symmetric key from password with generated salt
    /// Returns (key, salt) tuple
    fn derive_key_with_random_salt(
        &self,
        password: &str,
        key_size: usize,
    ) -> Result<(SymmetricKey, Vec<u8>), CryptoError>;
}
