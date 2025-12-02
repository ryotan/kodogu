//! PBKDF2-HMAC-SHA256 key derivation
//!
//! Derives cryptographic keys from passwords using PBKDF2 with HMAC-SHA256.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crypto::{Pbkdf2Aes256, Pbkdf2, SymmetricKey};
//!
//! let password = "my_secure_password";
//! let salt = b"random_salt_16by"; // Should be random in practice
//! let iterations = 100_000;
//!
//! // ZST type (compile-time dispatch)
//! let key = Pbkdf2Aes256.derive_key(password, salt, iterations)?;
//!
//! // Enum (runtime selection)
//! let deriver = Pbkdf2::aes256();
//! let key = deriver.derive_key(password, salt, iterations)?;
//! ```

use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use suzunari_error::suzunari_error;

use crate::SymmetricKey;

//
// Error Types
//

/// Errors that can occur during key derivation
#[suzunari_error]
#[snafu(visibility(pub))]
pub enum KeyDerivationError {
    /// PBKDF2 derivation failed
    #[snafu(display("PBKDF2 derivation failed: {reason}"))]
    Pbkdf2Derivation { reason: String },
}

//
// ZST Types
//

/// PBKDF2 AES-256 key deriver
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct Pbkdf2Aes256;

impl Pbkdf2Aes256 {
    /// Derive AES-256 key (32 bytes) from password
    ///
    /// # Arguments
    ///
    /// * `password` - Password to derive key from
    /// * `salt` - Salt bytes (should be at least 16 bytes and random)
    /// * `iterations` - Number of iterations (recommended: 100,000+)
    ///
    /// # Security Notes
    ///
    /// - Salt should be unique and random for each password
    /// - Higher iteration counts increase security but slow down derivation
    /// - 100,000 iterations is a reasonable default for 2024
    pub fn derive_key(
        &self,
        password: &str,
        salt: &[u8],
        iterations: u32,
    ) -> Result<SymmetricKey, KeyDerivationError> {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
        Ok(SymmetricKey::aes256(key))
    }
}

/// PBKDF2 AES-128 key deriver
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct Pbkdf2Aes128;

impl Pbkdf2Aes128 {
    /// Derive AES-128 key (16 bytes) from password
    ///
    /// # Arguments
    ///
    /// * `password` - Password to derive key from
    /// * `salt` - Salt bytes (should be at least 16 bytes and random)
    /// * `iterations` - Number of iterations (recommended: 100,000+)
    pub fn derive_key(
        &self,
        password: &str,
        salt: &[u8],
        iterations: u32,
    ) -> Result<SymmetricKey, KeyDerivationError> {
        let mut key = [0u8; 16];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
        Ok(SymmetricKey::aes128(key))
    }
}

//
// Enum Wrapper
//

/// PBKDF2 key deriver (runtime selection)
///
/// Enum wrapper for runtime dispatch between AES-128 and AES-256 key derivation.
#[derive(Debug, Clone, Copy)]
pub enum Pbkdf2 {
    /// AES-128 key derivation
    Aes128,
    /// AES-256 key derivation
    Aes256,
}

impl Pbkdf2 {
    /// Create AES-128 key deriver (static factory method)
    pub fn aes128() -> Self {
        Self::Aes128
    }

    /// Create AES-256 key deriver (static factory method)
    pub fn aes256() -> Self {
        Self::Aes256
    }

    /// Derive a symmetric key from password
    ///
    /// Delegates to the appropriate ZST implementation based on enum variant.
    pub fn derive_key(
        &self,
        password: &str,
        salt: &[u8],
        iterations: u32,
    ) -> Result<SymmetricKey, KeyDerivationError> {
        match self {
            Pbkdf2::Aes128 => Pbkdf2Aes128.derive_key(password, salt, iterations),
            Pbkdf2::Aes256 => Pbkdf2Aes256.derive_key(password, salt, iterations),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PASSWORD: &str = "my_secure_password";
    const TEST_SALT: &[u8] = b"random_salt_1234";
    const TEST_ITERATIONS: u32 = 10_000; // Lower for faster tests

    #[test]
    fn test_pbkdf2_aes256_derives_key() {
        let key = Pbkdf2Aes256
            .derive_key(TEST_PASSWORD, TEST_SALT, TEST_ITERATIONS)
            .unwrap();

        match key {
            SymmetricKey::Aes256(_) => (), // Success
            _ => panic!("Expected Aes256 key"),
        }
    }

    #[test]
    fn test_pbkdf2_aes128_derives_key() {
        let key = Pbkdf2Aes128
            .derive_key(TEST_PASSWORD, TEST_SALT, TEST_ITERATIONS)
            .unwrap();

        match key {
            SymmetricKey::Aes128(_) => (), // Success
            _ => panic!("Expected Aes128 key"),
        }
    }

    #[test]
    fn test_same_inputs_produce_same_key() {
        let key1 = Pbkdf2Aes256
            .derive_key(TEST_PASSWORD, TEST_SALT, TEST_ITERATIONS)
            .unwrap();
        let key2 = Pbkdf2Aes256
            .derive_key(TEST_PASSWORD, TEST_SALT, TEST_ITERATIONS)
            .unwrap();

        // Same password + salt + iterations should produce same key
        // Verify by encrypting with both keys and cross-decrypting
        use crate::{Aes256Gcm, Plaintext};

        let plaintext = Plaintext::from_string("test".to_string());
        let cipher = Aes256Gcm;

        let ciphertext1 = cipher.encrypt(&plaintext, &key1).unwrap();
        let decrypted = cipher.decrypt(&ciphertext1, &key2).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_passwords_produce_different_keys() {
        let key1 = Pbkdf2Aes256
            .derive_key("password1", TEST_SALT, TEST_ITERATIONS)
            .unwrap();
        let key2 = Pbkdf2Aes256
            .derive_key("password2", TEST_SALT, TEST_ITERATIONS)
            .unwrap();

        // Different passwords should produce different keys
        use crate::{Aes256Gcm, Plaintext};

        let plaintext = Plaintext::from_string("test".to_string());
        let cipher = Aes256Gcm;

        let ciphertext1 = cipher.encrypt(&plaintext, &key1).unwrap();

        // Decrypt with wrong key should fail
        assert!(cipher.decrypt(&ciphertext1, &key2).is_err());
    }

    #[test]
    fn test_different_salts_produce_different_keys() {
        let key1 = Pbkdf2Aes256
            .derive_key(TEST_PASSWORD, b"salt1_________16", TEST_ITERATIONS)
            .unwrap();
        let key2 = Pbkdf2Aes256
            .derive_key(TEST_PASSWORD, b"salt2_________16", TEST_ITERATIONS)
            .unwrap();

        // Different salts should produce different keys
        use crate::{Aes256Gcm, Plaintext};

        let plaintext = Plaintext::from_string("test".to_string());
        let cipher = Aes256Gcm;

        let ciphertext1 = cipher.encrypt(&plaintext, &key1).unwrap();

        // Decrypt with wrong key should fail
        assert!(cipher.decrypt(&ciphertext1, &key2).is_err());
    }

    #[test]
    fn test_enum_aes256() {
        let deriver = Pbkdf2::aes256();
        let key = deriver
            .derive_key(TEST_PASSWORD, TEST_SALT, TEST_ITERATIONS)
            .unwrap();

        match key {
            SymmetricKey::Aes256(_) => (), // Success
            _ => panic!("Expected Aes256 key"),
        }
    }

    #[test]
    fn test_enum_aes128() {
        let deriver = Pbkdf2::aes128();
        let key = deriver
            .derive_key(TEST_PASSWORD, TEST_SALT, TEST_ITERATIONS)
            .unwrap();

        match key {
            SymmetricKey::Aes128(_) => (), // Success
            _ => panic!("Expected Aes128 key"),
        }
    }

    #[test]
    fn test_integration_password_to_encryption() {
        // Full workflow: password -> key derivation -> encryption -> decryption
        let password = "my_password";
        let salt = b"unique_salt_____";
        let iterations = 10_000;

        // Derive key from password
        let key = Pbkdf2Aes256
            .derive_key(password, salt, iterations)
            .unwrap();

        // Use key for encryption
        use crate::{Aes256Gcm, Plaintext};

        let plaintext = Plaintext::from_string("Secret message".to_string());
        let cipher = Aes256Gcm;

        let ciphertext = cipher.encrypt(&plaintext, &key).unwrap();
        let decrypted = cipher.decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
