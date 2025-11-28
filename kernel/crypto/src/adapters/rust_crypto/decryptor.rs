use super::errors::{
    AesCipherCreationError, AesDecryptionError, CiphertextTooShortError,
    InvalidDecryptedKeySizeError, InvalidUtf8Error, RsaDecryptionError, RsaPrivateKeyParsingError,
};
use crate::domain::{
    CryptoError,
    ciphertext::Ciphertext,
    keys::{PrivateKey, SymmetricKey},
    plaintext::Plaintext,
    providers::Decryptor,
};
use aes_gcm::{Aes128Gcm, Aes256Gcm, KeyInit, Nonce, aead::Aead};
use pkcs8::DecodePrivateKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use snafu::{ResultExt, ensure};

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
    fn decrypt_symmetric(
        &self,
        ciphertext: &Ciphertext,
        key: &SymmetricKey,
    ) -> Result<Plaintext, CryptoError> {
        let data = ciphertext.as_bytes();

        // Extract nonce and ciphertext: [nonce (12 bytes) | ciphertext]
        ensure!(data.len() >= 12, CiphertextTooShortError { size: data.len() });

        let (nonce_bytes, encrypted) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted = match key {
            SymmetricKey::Aes128(key_bytes) => {
                let cipher = Aes128Gcm::new_from_slice(key_bytes.as_slice())
                    .context(AesCipherCreationError)?;
                cipher.decrypt(nonce, encrypted).context(AesDecryptionError)?
            }
            SymmetricKey::Aes256(key_bytes) => {
                let cipher = Aes256Gcm::new_from_slice(key_bytes.as_slice())
                    .context(AesCipherCreationError)?;
                cipher.decrypt(nonce, encrypted).context(AesDecryptionError)?
            }
        };

        Ok(Plaintext::new(decrypted))
    }

    fn decrypt_asymmetric(
        &self,
        ciphertext: &Ciphertext,
        private_key: &PrivateKey,
    ) -> Result<Plaintext, CryptoError> {
        // Hybrid decryption: RSA + AES-256-GCM
        // Format: [encrypted_key_len (2 bytes big-endian) | encrypted_key | aes_ciphertext]

        let data = ciphertext.as_bytes();

        // 1. Extract encrypted key length
        ensure!(data.len() >= 2, CiphertextTooShortError { size: data.len() });
        let key_len = u16::from_be_bytes([data[0], data[1]]) as usize;

        // 2. Extract encrypted key and AES ciphertext
        ensure!(data.len() >= 2 + key_len, CiphertextTooShortError { size: data.len() });
        let encrypted_key = &data[2..2 + key_len];
        let aes_ciphertext_bytes = &data[2 + key_len..];

        // 3. Parse RSA private key from PEM
        let pem_string =
            String::from_utf8(private_key.as_bytes().to_vec()).context(InvalidUtf8Error)?;
        let rsa_private_key =
            RsaPrivateKey::from_pkcs8_pem(&pem_string).context(RsaPrivateKeyParsingError)?;

        // 4. Decrypt AES key with RSA private key
        let aes_key_bytes = rsa_private_key
            .decrypt(Pkcs1v15Encrypt, encrypted_key)
            .context(RsaDecryptionError)?;

        // 5. Convert to fixed-size array (AES-256 key)
        ensure!(
            aes_key_bytes.len() == 32,
            InvalidDecryptedKeySizeError { size: aes_key_bytes.len() }
        );
        let aes_key_array: [u8; 32] =
            aes_key_bytes.try_into().expect("Length already verified to be 32");
        let aes_key = SymmetricKey::aes256(aes_key_array);

        // 6. Decrypt ciphertext with AES key
        let aes_ciphertext = Ciphertext::new(aes_ciphertext_bytes.to_vec());
        self.decrypt_symmetric(&aes_ciphertext, &aes_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_instance() {
        let _decryptor = RustCryptoDecryptor::new();
    }

    #[test]
    fn default_creates_instance() {
        #[allow(clippy::default_constructed_unit_structs)]
        let _decryptor = RustCryptoDecryptor::default();
    }
}
