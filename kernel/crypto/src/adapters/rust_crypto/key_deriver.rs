use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

use super::errors::Pbkdf2DerivationError;
use crate::domain::{CryptoError, keys::SymmetricKey, providers::KeyDeriver};

/// PBKDF2-based key deriver using RustCrypto
pub struct RustCryptoKeyDeriver {
    iterations: u32,
}

impl RustCryptoKeyDeriver {
    /// Create key deriver with default iterations (100,000)
    pub fn new() -> Self {
        Self { iterations: 100_000 }
    }

    /// Create key deriver with custom iteration count
    pub fn with_iterations(iterations: u32) -> Self {
        Self { iterations }
    }
}

impl Default for RustCryptoKeyDeriver {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDeriver for RustCryptoKeyDeriver {
    fn derive_key(
        &self,
        password: &str,
        salt: &[u8],
        key_size: usize,
    ) -> Result<SymmetricKey, CryptoError> {
        let key = match key_size {
            16 => {
                let mut key_bytes = [0u8; 16];
                pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, self.iterations, &mut key_bytes);
                SymmetricKey::aes128(key_bytes)
            }
            32 => {
                let mut key_bytes = [0u8; 32];
                pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, self.iterations, &mut key_bytes);
                SymmetricKey::aes256(key_bytes)
            }
            _ => Pbkdf2DerivationError {
                reason: format!("Invalid key size: {} bytes (expected 16 or 32)", key_size),
            }
            .fail()?,
        };

        Ok(key)
    }

    fn derive_key_with_random_salt(
        &self,
        password: &str,
        key_size: usize,
    ) -> Result<(SymmetricKey, Vec<u8>), CryptoError> {
        let mut salt = vec![0u8; 32]; // 256-bit salt
        rand::thread_rng().fill_bytes(&mut salt);

        let key = self.derive_key(password, &salt, key_size)?;
        Ok((key, salt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_deriver_with_default_iterations() {
        let deriver = RustCryptoKeyDeriver::new();
        assert_eq!(deriver.iterations, 100_000);
    }

    #[test]
    fn default_creates_deriver_with_100k_iterations() {
        let deriver = RustCryptoKeyDeriver::default();
        assert_eq!(deriver.iterations, 100_000);
    }

    #[test]
    fn with_iterations_creates_custom_deriver() {
        let deriver = RustCryptoKeyDeriver::with_iterations(50_000);
        assert_eq!(deriver.iterations, 50_000);
    }

    #[test]
    fn derive_key_rejects_invalid_key_size() {
        let deriver = RustCryptoKeyDeriver::new();
        let result = deriver.derive_key("password", b"salt", 24);
        assert!(result.is_err());
    }
}
