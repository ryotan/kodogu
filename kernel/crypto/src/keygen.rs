//! Key generation (symmetric and asymmetric)
//!
//! Provides cryptographically secure key generation for AES and RSA.
//!
//! ## Symmetric Keys
//!
//! ```rust,ignore
//! use crypto::{RandomAes256Key, RandomAesKey};
//!
//! // ZST type
//! let key = RandomAes256Key.generate()?;
//!
//! // Enum
//! let key = RandomAesKey::Aes256.generate()?;
//! ```
//!
//! ## Asymmetric Keys (RSA)
//!
//! ```rust,ignore
//! use crypto::{Rsa2048, RsaKeyGenerator};
//!
//! // ZST type
//! let keypair = Rsa2048.generate()?;
//!
//! // Enum
//! let generator = RsaKeyGenerator::rsa2048();
//! let keypair = generator.generate()?;
//! ```

use p256::ecdsa::SigningKey as P256SigningKey;
use p384::ecdsa::SigningKey as P384SigningKey;
use pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rand::RngCore;
use rsa::{RsaPrivateKey, RsaPublicKey};
use snafu::ResultExt;
use suzunari_error::suzunari_error;

use crate::{Aes128Key, Aes256Key, AesKey, KeyPair, PrivateKey, PublicKey};
//
// Error Types
//

/// Errors that can occur during key generation
#[suzunari_error]
#[snafu(visibility(pub))]
pub enum KeyGenerationError {
    /// Random number generation failed
    #[snafu(display("Random number generation failed: {reason}"))]
    RandomGenerationFailed { reason: String },

    /// RSA key generation failed
    #[snafu(display("RSA key generation failed"))]
    RsaKeyGeneration { source: rsa::Error },

    /// RSA private key PEM encoding failed
    #[snafu(display("RSA private key PEM encoding failed"))]
    RsaPrivateKeyPemEncoding { source: pkcs8::Error },

    /// RSA public key PEM encoding failed
    #[snafu(display("RSA public key PEM encoding failed"))]
    RsaPublicKeyPemEncoding { source: pkcs8::spki::Error },

    /// ECDSA private key PEM encoding failed
    #[snafu(display("ECDSA private key PEM encoding failed"))]
    EcdsaPrivateKeyPemEncoding { source: pkcs8::Error },

    /// ECDSA public key PEM encoding failed
    #[snafu(display("ECDSA public key PEM encoding failed"))]
    EcdsaPublicKeyPemEncoding { source: pkcs8::spki::Error },

    /// Ed25519 private key PEM encoding failed
    #[snafu(display("Ed25519 private key PEM encoding failed"))]
    Ed25519PrivateKeyPemEncoding { source: pkcs8::Error },

    /// Ed25519 public key PEM encoding failed
    #[snafu(display("Ed25519 public key PEM encoding failed"))]
    Ed25519PublicKeyPemEncoding { source: pkcs8::spki::Error },
}

//
// ZST Types
//

/// Random AES-256 key generator
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct RandomAes256Key;

impl RandomAes256Key {
    /// Generate a random AES-256 key (32 bytes)
    ///
    /// Uses `OsRng` for cryptographically secure randomness.
    pub fn generate(&self) -> Result<Aes256Key, KeyGenerationError> {
        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        Ok(Aes256Key::from(key))
    }
}

/// Random AES-128 key generator
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct RandomAes128Key;

impl RandomAes128Key {
    /// Generate a random AES-128 key (16 bytes)
    ///
    /// Uses `OsRng` for cryptographically secure randomness.
    pub fn generate(&self) -> Result<Aes128Key, KeyGenerationError> {
        let mut key = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut key);
        Ok(Aes128Key::from(key))
    }
}

//
// Enum Wrapper
//

/// Random symmetric key generator (runtime selection)
///
/// Enum wrapper for runtime dispatch between AES-128 and AES-256 key generation.
#[derive(Debug, Clone, Copy)]
pub enum RandomAesKey {
    /// AES-128 key generation
    Aes128,
    /// AES-256 key generation
    Aes256,
}

impl RandomAesKey {
    /// Generate a random AES-256 key
    ///
    /// Convenience method for generating an AES-256 key wrapped in AesKey enum.
    pub fn aes256() -> Result<AesKey, KeyGenerationError> {
        RandomAes256Key.generate().map(AesKey::Aes256)
    }

    /// Generate a random AES-128 key
    ///
    /// Convenience method for generating an AES-128 key wrapped in AesKey enum.
    pub fn aes128() -> Result<AesKey, KeyGenerationError> {
        RandomAes128Key.generate().map(AesKey::Aes128)
    }

    /// Generate a random symmetric key
    ///
    /// Delegates to the appropriate ZST implementation based on enum variant.
    pub fn generate(&self) -> Result<AesKey, KeyGenerationError> {
        match self {
            RandomAesKey::Aes256 => Self::aes256(),
            RandomAesKey::Aes128 => Self::aes128(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys_are_different() {
        use crate::Plaintext;

        let key1 = RandomAes256Key.generate().unwrap();
        let key2 = RandomAes256Key.generate().unwrap();

        // Two random keys should produce different results
        // Verify by trying to decrypt with wrong key (should fail)
        let plaintext = Plaintext::from("test");
        let cipher = crate::Aes256Gcm;

        let ciphertext1 = cipher.encrypt(&plaintext, &key1).unwrap();

        // Decrypt with wrong key should fail
        assert!(cipher.decrypt(&ciphertext1, &key2).is_err());
    }

    #[test]
    fn test_enum_aes256() {
        let generator = RandomAesKey::Aes256;
        let key = generator.generate().unwrap();
        match key {
            AesKey::Aes256(_) => (), // Success
            _ => panic!("Expected Aes256 key"),
        }
    }

    #[test]
    fn test_enum_aes128() {
        let generator = RandomAesKey::Aes128;
        let key = generator.generate().unwrap();
        match key {
            AesKey::Aes128(_) => (), // Success
            _ => panic!("Expected Aes128 key"),
        }
    }

    #[test]
    fn test_integration_with_encryption() {
        use crate::{Aes256Gcm, Plaintext};

        // Generate key
        let key = RandomAes256Key.generate().unwrap();

        // Use key for encryption
        let plaintext = Plaintext::from("Test message");
        let ciphertext = Aes256Gcm.encrypt(&plaintext, &key).unwrap();
        let decrypted = Aes256Gcm.decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_static_aes256() {
        let key = RandomAesKey::aes256().unwrap();
        match key {
            AesKey::Aes256(_) => (), // Success
            _ => panic!("Expected Aes256 key"),
        }
    }

    #[test]
    fn test_static_aes128() {
        let key = RandomAesKey::aes128().unwrap();
        match key {
            AesKey::Aes128(_) => (), // Success
            _ => panic!("Expected Aes128 key"),
        }
    }
}

//
// RSA Key Generation
//

/// RSA-2048 key generator
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct Rsa2048;

impl Rsa2048 {
    /// Generate RSA-2048 key pair (2048-bit modulus)
    ///
    /// Uses `OsRng` for cryptographically secure randomness.
    /// Returns keys in PEM format.
    pub fn generate(&self) -> Result<KeyPair, KeyGenerationError> {
        let mut rng = rand::rngs::OsRng;

        // Generate RSA private key
        let private_key = RsaPrivateKey::new(&mut rng, 2048).context(RsaKeyGenerationSnafu)?;

        // Extract public key
        let public_key = RsaPublicKey::from(&private_key);

        // Encode to PEM
        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .context(RsaPrivateKeyPemEncodingSnafu)?;
        let public_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .context(RsaPublicKeyPemEncodingSnafu)?;

        Ok(KeyPair::new(
            PublicKey::from(public_pem),
            PrivateKey::from(private_pem.to_string()),
        ))
    }
}

/// RSA-4096 key generator
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct Rsa4096;

impl Rsa4096 {
    /// Generate RSA-4096 key pair (4096-bit modulus)
    ///
    /// Uses `OsRng` for cryptographically secure randomness.
    /// Returns keys in PEM format.
    pub fn generate(&self) -> Result<KeyPair, KeyGenerationError> {
        let mut rng = rand::rngs::OsRng;

        // Generate RSA private key
        let private_key = RsaPrivateKey::new(&mut rng, 4096).context(RsaKeyGenerationSnafu)?;

        // Extract public key
        let public_key = RsaPublicKey::from(&private_key);

        // Encode to PEM
        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .context(RsaPrivateKeyPemEncodingSnafu)?;
        let public_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .context(RsaPublicKeyPemEncodingSnafu)?;

        Ok(KeyPair::new(
            PublicKey::from(public_pem),
            PrivateKey::from(private_pem.to_string()),
        ))
    }
}

//
// Enum Wrapper (RSA)
//

/// RSA key generator (runtime selection)
///
/// Enum wrapper for runtime dispatch between different RSA key sizes.
#[derive(Debug, Clone, Copy)]
pub enum RsaKeyGenerator {
    /// RSA-2048 key generation
    Rsa2048,
    /// RSA-4096 key generation
    Rsa4096,
}

impl RsaKeyGenerator {
    /// Create RSA-2048 key generator (static factory method)
    pub fn rsa2048() -> Self {
        Self::Rsa2048
    }

    /// Create RSA-4096 key generator (static factory method)
    pub fn rsa4096() -> Self {
        Self::Rsa4096
    }

    /// Generate an RSA key pair
    ///
    /// Delegates to the appropriate ZST implementation based on enum variant.
    pub fn generate(&self) -> Result<KeyPair, KeyGenerationError> {
        match self {
            RsaKeyGenerator::Rsa2048 => Rsa2048.generate(),
            RsaKeyGenerator::Rsa4096 => Rsa4096.generate(),
        }
    }
}

#[cfg(test)]
mod rsa_tests {
    use super::*;

    #[test]
    fn test_rsa2048_generates_keypair() {
        let keypair = Rsa2048.generate().unwrap();

        // Check that we got PEM data
        assert!(!keypair.public_key.as_pem().is_empty());
        assert!(!keypair.private_key.as_pem().is_empty());

        // Check PEM format markers
        let public_pem = keypair.public_key.as_pem();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(public_pem.contains("END PUBLIC KEY"));

        let private_pem = keypair.private_key.as_pem();
        assert!(private_pem.contains("BEGIN PRIVATE KEY"));
        assert!(private_pem.contains("END PRIVATE KEY"));
    }

    #[test]
    fn test_rsa4096_generates_keypair() {
        let keypair = Rsa4096.generate().unwrap();

        // Check that we got PEM data
        assert!(!keypair.public_key.as_pem().is_empty());
        assert!(!keypair.private_key.as_pem().is_empty());

        // Check PEM format markers
        let public_pem = keypair.public_key.as_pem();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(public_pem.contains("END PUBLIC KEY"));

        let private_pem = keypair.private_key.as_pem();
        assert!(private_pem.contains("BEGIN PRIVATE KEY"));
        assert!(private_pem.contains("END PRIVATE KEY"));
    }

    #[test]
    fn test_rsa_keys_are_different() {
        let keypair1 = Rsa2048.generate().unwrap();
        let keypair2 = Rsa2048.generate().unwrap();

        // Two random key pairs should be different
        assert_ne!(keypair1.public_key.as_pem(), keypair2.public_key.as_pem());
        assert_ne!(keypair1.private_key.as_pem(), keypair2.private_key.as_pem());
    }

    #[test]
    fn test_enum_rsa2048() {
        let generator = RsaKeyGenerator::rsa2048();
        let keypair = generator.generate().unwrap();

        // Check that we got valid PEM data
        let public_pem = keypair.public_key.as_pem();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
    }

    #[test]
    fn test_enum_rsa4096() {
        let generator = RsaKeyGenerator::rsa4096();
        let keypair = generator.generate().unwrap();

        // Check that we got valid PEM data
        let public_pem = keypair.public_key.as_pem();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
    }
}

//
// ECDSA Key Generation
//

/// ECDSA P-256 key generator
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct EcdsaP256;

impl EcdsaP256 {
    /// Generate ECDSA P-256 key pair (NIST P-256 curve)
    ///
    /// Uses `OsRng` for cryptographically secure randomness.
    /// Returns keys in PEM format.
    pub fn generate(&self) -> Result<KeyPair, KeyGenerationError> {
        let mut rng = rand::rngs::OsRng;

        // Generate ECDSA signing key (includes private key)
        let signing_key = P256SigningKey::random(&mut rng);

        // Extract verifying key (public key)
        let verifying_key = signing_key.verifying_key();

        // Encode to PEM
        let private_pem = signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .context(EcdsaPrivateKeyPemEncodingSnafu)?;
        let public_pem = verifying_key
            .to_public_key_pem(LineEnding::LF)
            .context(EcdsaPublicKeyPemEncodingSnafu)?;

        Ok(KeyPair::new(
            PublicKey::from(public_pem),
            PrivateKey::from(private_pem.to_string()),
        ))
    }
}

/// ECDSA P-384 key generator
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct EcdsaP384;

impl EcdsaP384 {
    /// Generate ECDSA P-384 key pair (NIST P-384 curve)
    ///
    /// Uses `OsRng` for cryptographically secure randomness.
    /// Returns keys in PEM format.
    pub fn generate(&self) -> Result<KeyPair, KeyGenerationError> {
        let mut rng = rand::rngs::OsRng;

        // Generate ECDSA signing key (includes private key)
        let signing_key = P384SigningKey::random(&mut rng);

        // Extract verifying key (public key)
        let verifying_key = signing_key.verifying_key();

        // Encode to PEM
        let private_pem = signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .context(EcdsaPrivateKeyPemEncodingSnafu)?;
        let public_pem = verifying_key
            .to_public_key_pem(LineEnding::LF)
            .context(EcdsaPublicKeyPemEncodingSnafu)?;

        Ok(KeyPair::new(
            PublicKey::from(public_pem),
            PrivateKey::from(private_pem.to_string()),
        ))
    }
}

//
// Enum Wrapper (ECDSA)
//

/// ECDSA key generator (runtime selection)
///
/// Enum wrapper for runtime dispatch between P-256 and P-384 curves.
#[derive(Debug, Clone, Copy)]
pub enum EcdsaKeyGenerator {
    /// ECDSA P-256 key generation
    P256,
    /// ECDSA P-384 key generation
    P384,
}

impl EcdsaKeyGenerator {
    /// Create ECDSA P-256 key generator (static factory method)
    pub fn p256() -> Self {
        Self::P256
    }

    /// Create ECDSA P-384 key generator (static factory method)
    pub fn p384() -> Self {
        Self::P384
    }

    /// Generate an ECDSA key pair
    ///
    /// Delegates to the appropriate ZST implementation based on enum variant.
    pub fn generate(&self) -> Result<KeyPair, KeyGenerationError> {
        match self {
            EcdsaKeyGenerator::P256 => EcdsaP256.generate(),
            EcdsaKeyGenerator::P384 => EcdsaP384.generate(),
        }
    }
}

//
// Ed25519 Key Generation
//

/// Ed25519 key generator
///
/// Zero-sized type for compile-time dispatch.
#[derive(Debug, Clone, Copy)]
pub struct Ed25519KeyGen;

impl Ed25519KeyGen {
    /// Generate Ed25519 key pair
    ///
    /// Uses `OsRng` for cryptographically secure randomness.
    /// Returns keys in PEM format.
    pub fn generate(&self) -> Result<KeyPair, KeyGenerationError> {
        let mut rng = rand::rngs::OsRng;

        // Generate Ed25519 signing key (includes private key)
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);

        // Extract verifying key (public key)
        let verifying_key = signing_key.verifying_key();

        // Encode to PEM
        let private_pem = signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .context(Ed25519PrivateKeyPemEncodingSnafu)?;
        let public_pem = verifying_key
            .to_public_key_pem(LineEnding::LF)
            .context(Ed25519PublicKeyPemEncodingSnafu)?;

        Ok(KeyPair::new(
            PublicKey::from(public_pem),
            PrivateKey::from(private_pem.to_string()),
        ))
    }
}

#[cfg(test)]
mod ecdsa_ed25519_tests {
    use super::*;

    #[test]
    fn test_ecdsa_p256_generates_keypair() {
        let keypair = EcdsaP256.generate().unwrap();

        // Check that we got PEM data
        assert!(!keypair.public_key.as_pem().is_empty());
        assert!(!keypair.private_key.as_pem().is_empty());

        // Check PEM format markers
        let public_pem = keypair.public_key.as_pem();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(public_pem.contains("END PUBLIC KEY"));

        let private_pem = keypair.private_key.as_pem();
        assert!(private_pem.contains("BEGIN PRIVATE KEY"));
        assert!(private_pem.contains("END PRIVATE KEY"));
    }

    #[test]
    fn test_ecdsa_p384_generates_keypair() {
        let keypair = EcdsaP384.generate().unwrap();

        // Check that we got PEM data
        assert!(!keypair.public_key.as_pem().is_empty());
        assert!(!keypair.private_key.as_pem().is_empty());

        // Check PEM format markers
        let public_pem = keypair.public_key.as_pem();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(public_pem.contains("END PUBLIC KEY"));

        let private_pem = keypair.private_key.as_pem();
        assert!(private_pem.contains("BEGIN PRIVATE KEY"));
        assert!(private_pem.contains("END PRIVATE KEY"));
    }

    #[test]
    fn test_ecdsa_keys_are_different() {
        let keypair1 = EcdsaP256.generate().unwrap();
        let keypair2 = EcdsaP256.generate().unwrap();

        // Two random key pairs should be different
        assert_ne!(keypair1.public_key.as_pem(), keypair2.public_key.as_pem());
        assert_ne!(keypair1.private_key.as_pem(), keypair2.private_key.as_pem());
    }

    #[test]
    fn test_enum_ecdsa_p256() {
        let generator = EcdsaKeyGenerator::p256();
        let keypair = generator.generate().unwrap();

        // Check that we got valid PEM data
        let public_pem = keypair.public_key.as_pem();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
    }

    #[test]
    fn test_enum_ecdsa_p384() {
        let generator = EcdsaKeyGenerator::p384();
        let keypair = generator.generate().unwrap();

        // Check that we got valid PEM data
        let public_pem = keypair.public_key.as_pem();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
    }

    #[test]
    fn test_ed25519_generates_keypair() {
        let keypair = Ed25519KeyGen.generate().unwrap();

        // Check that we got PEM data
        assert!(!keypair.public_key.as_pem().is_empty());
        assert!(!keypair.private_key.as_pem().is_empty());

        // Check PEM format markers
        let public_pem = keypair.public_key.as_pem();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(public_pem.contains("END PUBLIC KEY"));

        let private_pem = keypair.private_key.as_pem();
        assert!(private_pem.contains("BEGIN PRIVATE KEY"));
        assert!(private_pem.contains("END PRIVATE KEY"));
    }

    #[test]
    fn test_ed25519_keys_are_different() {
        let keypair1 = Ed25519KeyGen.generate().unwrap();
        let keypair2 = Ed25519KeyGen.generate().unwrap();

        // Two random key pairs should be different
        assert_ne!(keypair1.public_key.as_pem(), keypair2.public_key.as_pem());
        assert_ne!(keypair1.private_key.as_pem(), keypair2.private_key.as_pem());
    }
}
