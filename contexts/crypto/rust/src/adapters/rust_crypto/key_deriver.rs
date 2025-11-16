use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use snafu::ensure;
use crate::domain::{
    CryptoError,
    keys::SymmetricKey,
    providers::KeyDeriver,
};
use super::errors::Pbkdf2DerivationSnafu;

/// PBKDF2-based key deriver using RustCrypto
pub struct RustCryptoKeyDeriver {
    iterations: u32,
}

impl RustCryptoKeyDeriver {
    /// Create key deriver with default iterations (100,000)
    pub fn new() -> Self {
        Self {
            iterations: 100_000,
        }
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
    fn derive_key(&self, password: &str, salt: &[u8], key_size: usize) -> Result<SymmetricKey, CryptoError> {
        ensure!(key_size == 16 || key_size == 32, Pbkdf2DerivationSnafu {
                reason: format!("Invalid key size: {} bytes (expected 16 or 32)", key_size),
            });

        let mut key = vec![0u8; key_size];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, self.iterations, &mut key);

        Ok(SymmetricKey::new(key))
    }

    fn derive_key_with_random_salt(&self, password: &str, key_size: usize) -> Result<(SymmetricKey, Vec<u8>), CryptoError> {
        let mut salt = vec![0u8; 32]; // 256-bit salt
        rand::thread_rng().fill_bytes(&mut salt);

        let key = self.derive_key(password, &salt, key_size)?;
        Ok((key, salt))
    }
}
