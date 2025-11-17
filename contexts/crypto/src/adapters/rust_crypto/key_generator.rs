use rand::rngs::OsRng;
use rand::Rng;
use snafu::ResultExt;

use super::errors::{
    EcdsaPkcs8EncodingSnafu, EcdsaSpkiEncodingSnafu, Ed25519Pkcs8EncodingSnafu,
    Ed25519SpkiEncodingSnafu, RsaKeyGenerationSnafu, RsaPkcs8EncodingSnafu, RsaSpkiEncodingSnafu,
};
use crate::domain::{
    algorithms::{AsymmetricAlgorithm, EcdsaCurve, RsaKeySize, SymmetricAlgorithm},
    keys::{PrivateKey, PublicKey, SymmetricKey},
    providers::KeyGenerator,
    CryptoError, KeyPair,
};

/// RustCrypto-based key generator implementation
pub struct RustCryptoKeyGenerator;

impl RustCryptoKeyGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_rsa(&self, key_size: RsaKeySize) -> Result<KeyPair, CryptoError> {
        use pkcs8::EncodePrivateKey;
        use rsa::RsaPrivateKey;

        let bits = key_size.bits();
        let private_key = RsaPrivateKey::new(&mut OsRng, bits).context(RsaKeyGenerationSnafu)?;

        let private_pem = private_key
            .to_pkcs8_pem(pkcs8::LineEnding::LF)
            .context(RsaPkcs8EncodingSnafu)?;

        let public_key = private_key.to_public_key();
        let public_pem =
            pkcs8::EncodePublicKey::to_public_key_pem(&public_key, pkcs8::LineEnding::LF)
                .context(RsaSpkiEncodingSnafu)?;

        Ok(KeyPair::new(
            AsymmetricAlgorithm::Rsa(key_size),
            PublicKey::new(public_pem.as_bytes().to_vec()),
            PrivateKey::new(private_pem.as_bytes().to_vec()),
        ))
    }

    fn generate_ecdsa(&self, curve: EcdsaCurve) -> Result<KeyPair, CryptoError> {
        match curve {
            EcdsaCurve::P256 => self.generate_ecdsa_p256(),
            EcdsaCurve::P384 => self.generate_ecdsa_p384(),
        }
    }

    fn generate_ecdsa_p256(&self) -> Result<KeyPair, CryptoError> {
        use p256::ecdsa::SigningKey;
        use pkcs8::EncodePrivateKey;

        let signing_key = SigningKey::random(&mut OsRng);

        let private_pem = signing_key
            .to_pkcs8_pem(pkcs8::LineEnding::LF)
            .context(EcdsaPkcs8EncodingSnafu)?;

        let verifying_key = signing_key.verifying_key();
        let public_pem =
            pkcs8::EncodePublicKey::to_public_key_pem(verifying_key, pkcs8::LineEnding::LF)
                .context(EcdsaSpkiEncodingSnafu)?;

        Ok(KeyPair::new(
            AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P256),
            PublicKey::new(public_pem.as_bytes().to_vec()),
            PrivateKey::new(private_pem.as_bytes().to_vec()),
        ))
    }

    fn generate_ecdsa_p384(&self) -> Result<KeyPair, CryptoError> {
        use p384::ecdsa::SigningKey;
        use pkcs8::EncodePrivateKey;

        let signing_key = SigningKey::random(&mut OsRng);

        let private_pem = signing_key
            .to_pkcs8_pem(pkcs8::LineEnding::LF)
            .context(EcdsaPkcs8EncodingSnafu)?;

        let verifying_key = signing_key.verifying_key();
        let public_pem =
            pkcs8::EncodePublicKey::to_public_key_pem(verifying_key, pkcs8::LineEnding::LF)
                .context(EcdsaSpkiEncodingSnafu)?;

        Ok(KeyPair::new(
            AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P384),
            PublicKey::new(public_pem.as_bytes().to_vec()),
            PrivateKey::new(private_pem.as_bytes().to_vec()),
        ))
    }

    fn generate_ed25519(&self) -> Result<KeyPair, CryptoError> {
        use ed25519_dalek::SigningKey;
        use pkcs8::EncodePrivateKey;

        let mut key_bytes = [0u8; 32];
        OsRng.fill(&mut key_bytes);
        let signing_key = SigningKey::from_bytes(&key_bytes);

        let private_pem = signing_key
            .to_pkcs8_pem(pkcs8::LineEnding::LF)
            .context(Ed25519Pkcs8EncodingSnafu)?;

        let verifying_key = signing_key.verifying_key();
        let public_pem =
            pkcs8::EncodePublicKey::to_public_key_pem(&verifying_key, pkcs8::LineEnding::LF)
                .context(Ed25519SpkiEncodingSnafu)?;

        Ok(KeyPair::new(
            AsymmetricAlgorithm::Ed25519,
            PublicKey::new(public_pem.as_bytes().to_vec()),
            PrivateKey::new(private_pem.as_bytes().to_vec()),
        ))
    }
}

impl Default for RustCryptoKeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyGenerator for RustCryptoKeyGenerator {
    fn generate_key_pair(&self, algorithm: AsymmetricAlgorithm) -> Result<KeyPair, CryptoError> {
        match algorithm {
            AsymmetricAlgorithm::Rsa(key_size) => self.generate_rsa(key_size),
            AsymmetricAlgorithm::Ecdsa(curve) => self.generate_ecdsa(curve),
            AsymmetricAlgorithm::Ed25519 => self.generate_ed25519(),
        }
    }

    fn generate_symmetric_key(
        &self,
        algorithm: SymmetricAlgorithm,
    ) -> Result<SymmetricKey, CryptoError> {
        let key = match algorithm {
            SymmetricAlgorithm::Aes128 => {
                let mut key_bytes = [0u8; 16];
                OsRng.fill(&mut key_bytes);
                SymmetricKey::aes128(key_bytes)
            }
            SymmetricAlgorithm::Aes256 => {
                let mut key_bytes = [0u8; 32];
                OsRng.fill(&mut key_bytes);
                SymmetricKey::aes256(key_bytes)
            }
        };
        Ok(key)
    }

    fn generate_password_string(&self, length: usize) -> Result<String, CryptoError> {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let password: String = (0..length)
            .map(|_| {
                let idx = OsRng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        Ok(password)
    }
}
