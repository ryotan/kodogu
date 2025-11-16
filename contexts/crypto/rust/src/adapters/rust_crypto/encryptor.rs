use aes_gcm::{
    Aes128Gcm, Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use rand::RngCore;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
use pkcs8::DecodePublicKey;
use snafu::ResultExt;
use suzunari_error::BoxedStackError;

use crate::domain::{
    CryptoError,
    ciphertext::Ciphertext,
    keys::{PublicKey, SymmetricKey},
    plaintext::Plaintext,
    providers::Encryptor,
};
use super::errors::{
    AesCipherCreationSnafu, AesEncryptionSnafu, InvalidKeySizeSnafu,
    InvalidPemEncodingSnafu, RsaEncryptionSnafu, RsaPublicKeyParsingSnafu,
};

/// AES-GCM based encryptor using RustCrypto
pub struct RustCryptoEncryptor;

impl RustCryptoEncryptor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustCryptoEncryptor {
    fn default() -> Self {
        Self::new()
    }
}

impl Encryptor for RustCryptoEncryptor {
    fn encrypt_symmetric(&self, plaintext: &Plaintext, key: &SymmetricKey) -> Result<Ciphertext, CryptoError> {
        let key_bytes = key.as_bytes();

        // Generate random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = match key_bytes.len() {
            16 => {
                // AES-128-GCM
                let cipher = Aes128Gcm::new_from_slice(key_bytes)
                    .map_err(BoxedStackError::new)
                    .context(AesCipherCreationSnafu)?;
                cipher
                    .encrypt(nonce, plaintext.as_bytes())
                    .map_err(BoxedStackError::new)
                    .context(AesEncryptionSnafu)?
            }
            32 => {
                // AES-256-GCM
                let cipher = Aes256Gcm::new_from_slice(key_bytes)
                    .map_err(BoxedStackError::new)
                    .context(AesCipherCreationSnafu)?;
                cipher
                    .encrypt(nonce, plaintext.as_bytes())
                    .map_err(BoxedStackError::new)
                    .context(AesEncryptionSnafu)?
            }
            _ => {
                return Err(InvalidKeySizeSnafu { size: key_bytes.len() }.build().into());
            }
        };

        // Prepend nonce to ciphertext: [nonce (12 bytes) | ciphertext]
        let mut result = Vec::with_capacity(12 + encrypted.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&encrypted);

        Ok(Ciphertext::new(result))
    }

    fn encrypt_asymmetric(&self, plaintext: &Plaintext, public_key: &PublicKey) -> Result<Ciphertext, CryptoError> {
        // Hybrid encryption: RSA + AES-256-GCM
        // Format: [encrypted_key_len (2 bytes big-endian) | encrypted_key | aes_ciphertext]

        // 1. Parse RSA public key from PEM
        let pem_string = String::from_utf8(public_key.as_bytes().to_vec())
            .map_err(BoxedStackError::new)
            .context(InvalidPemEncodingSnafu)?;
        let rsa_public_key = RsaPublicKey::from_public_key_pem(&pem_string)
            .map_err(BoxedStackError::new)
            .context(RsaPublicKeyParsingSnafu)?;

        // 2. Generate random AES-256 key
        let mut aes_key_bytes = vec![0u8; 32];
        OsRng.fill_bytes(&mut aes_key_bytes);
        let aes_key = SymmetricKey::new(aes_key_bytes);

        // 3. Encrypt plaintext with AES key
        let aes_ciphertext = self.encrypt_symmetric(plaintext, &aes_key)?;

        // 4. Encrypt AES key with RSA public key
        let encrypted_key = rsa_public_key
            .encrypt(&mut OsRng, Pkcs1v15Encrypt, aes_key.as_bytes())
            .map_err(BoxedStackError::new)
            .context(RsaEncryptionSnafu)?;

        // 5. Pack result: [key_len (2 bytes) | encrypted_key | ciphertext]
        let key_len = encrypted_key.len() as u16;
        let mut result = Vec::with_capacity(2 + encrypted_key.len() + aes_ciphertext.as_bytes().len());
        result.extend_from_slice(&key_len.to_be_bytes());
        result.extend_from_slice(&encrypted_key);
        result.extend_from_slice(aes_ciphertext.as_bytes());

        Ok(Ciphertext::new(result))
    }
}
