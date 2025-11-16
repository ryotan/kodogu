use aes_gcm::{
    Aes128Gcm, Aes256Gcm, KeyInit, Nonce,
    aead::Aead,
};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use pkcs8::DecodePrivateKey;
use snafu::{ensure, ResultExt};
use crate::domain::{
    CryptoError,
    ciphertext::Ciphertext,
    keys::{PrivateKey, SymmetricKey},
    plaintext::Plaintext,
    providers::Decryptor,
};
use super::errors::{
    AesCipherCreationSnafu, AesDecryptionSnafu, CiphertextTooShortSnafu,
    InvalidKeySizeSnafu, RsaDecryptionSnafu, RsaPrivateKeyParsingSnafu, InvalidUtf8Snafu
};

/// AES-GCM based decryptor using RustCrypto
pub struct RustCryptoDecryptor;

impl RustCryptoDecryptor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustCryptoDecryptor {
    fn default() -> Self {
        Self::new()
    }
}

impl Decryptor for RustCryptoDecryptor {
    fn decrypt_symmetric(&self, ciphertext: &Ciphertext, key: &SymmetricKey) -> Result<Plaintext, CryptoError> {
        let data = ciphertext.as_bytes();
        let key_bytes = key.as_bytes();

        // Extract nonce and ciphertext: [nonce (12 bytes) | ciphertext]
        ensure!(data.len() >= 12, CiphertextTooShortSnafu { size: data.len() });

        let (nonce_bytes, encrypted) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted = match key_bytes.len() {
            16 => {
                // AES-128-GCM
                let cipher = Aes128Gcm::new_from_slice(key_bytes).context(AesCipherCreationSnafu)?;
                cipher.decrypt(nonce, encrypted).context(AesDecryptionSnafu)?
            }
            32 => {
                // AES-256-GCM
                let cipher = Aes256Gcm::new_from_slice(key_bytes).context(AesCipherCreationSnafu)?;
                cipher.decrypt(nonce, encrypted).context(AesDecryptionSnafu)?
            }
            _ => {
                return Err(InvalidKeySizeSnafu { size: key_bytes.len() }.build().into());
            }
        };

        Ok(Plaintext::new(decrypted))
    }

    fn decrypt_asymmetric(&self, ciphertext: &Ciphertext, private_key: &PrivateKey) -> Result<Plaintext, CryptoError> {
        // Hybrid decryption: RSA + AES-256-GCM
        // Format: [encrypted_key_len (2 bytes big-endian) | encrypted_key | aes_ciphertext]

        let data = ciphertext.as_bytes();

        // 1. Extract encrypted key length
        if data.len() < 2 {
            return Err(CiphertextTooShortSnafu { size: data.len() }.build().into());
        }
        let key_len = u16::from_be_bytes([data[0], data[1]]) as usize;

        // 2. Extract encrypted key and AES ciphertext
        if data.len() < 2 + key_len {
            return Err(CiphertextTooShortSnafu { size: data.len() }.build().into());
        }
        let encrypted_key = &data[2..2 + key_len];
        let aes_ciphertext_bytes = &data[2 + key_len..];

        // 3. Parse RSA private key from PEM
        let pem_string = String::from_utf8(private_key.as_bytes().to_vec()).context(InvalidUtf8Snafu)?;
        let rsa_private_key = RsaPrivateKey::from_pkcs8_pem(&pem_string).context(RsaPrivateKeyParsingSnafu)?;

        // 4. Decrypt AES key with RSA private key
        let aes_key_bytes = rsa_private_key.decrypt(Pkcs1v15Encrypt, encrypted_key).context(RsaDecryptionSnafu)?;
        let aes_key = SymmetricKey::new(aes_key_bytes);

        // 5. Decrypt ciphertext with AES key
        let aes_ciphertext = Ciphertext::new(aes_ciphertext_bytes.to_vec());
        self.decrypt_symmetric(&aes_ciphertext, &aes_key)
    }
}
