use super::types::{AdditionalData, AeadCiphertext, AuthenticationTag};
use crate::Nonce;
use crate::{Ciphertext, Plaintext};

/// AEAD encryption operation.
///
/// Abstracts authenticated encryption with associated data.
/// All AEAD ciphers provide both confidentiality and authenticity.
pub trait AeadEncryptor {
    /// Key type for encryption.
    type Key: ?Sized;

    /// Error type for encryption.
    type Error;

    /// Encrypts `plaintext` with the given `key`, `nonce`, and `aad`.
    ///
    /// Returns separate ciphertext and authentication tag.
    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<(Ciphertext, AuthenticationTag), Self::Error>;

    /// Encrypts `plaintext` and returns combined ciphertext with embedded tag.
    ///
    /// Convenience method that combines ciphertext and tag into [`AeadCiphertext`].
    fn encrypt_combined(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<AeadCiphertext, Self::Error> {
        let (ciphertext, tag) = self.encrypt(plaintext, key, nonce, aad)?;
        Ok(AeadCiphertext::new(ciphertext.into_bytes(), tag.as_bytes().to_vec()))
    }
}

/// AEAD decryption operation.
///
/// Abstracts authenticated decryption with associated data.
/// Decryption fails if authentication tag verification fails.
pub trait AeadDecryptor {
    /// Key type for decryption.
    type Key: ?Sized;

    /// Error type for decryption.
    type Error;

    /// Decrypts `ciphertext` with the given `tag`, `key`, `nonce`, and `aad`.
    ///
    /// Returns an error if authentication fails.
    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        tag: &AuthenticationTag,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<Plaintext, Self::Error>;

    /// Decrypts combined ciphertext (with embedded tag).
    ///
    /// Convenience method that extracts ciphertext and tag from [`AeadCiphertext`].
    fn decrypt_combined(
        &self,
        aead_ciphertext: &AeadCiphertext,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<Plaintext, Self::Error> {
        let ciphertext = Ciphertext::from(aead_ciphertext.ciphertext());
        let tag = AuthenticationTag::from(aead_ciphertext.tag());
        self.decrypt(&ciphertext, &tag, key, nonce, aad)
    }
}

/// Combined AEAD encryption and decryption.
///
/// Marker trait for types implementing both [`AeadEncryptor`] and [`AeadDecryptor`].
/// Guarantees the same key and error types for both operations.
pub trait AeadCipher:
    AeadEncryptor<Key = <Self as AeadCipher>::Key, Error = <Self as AeadCipher>::Error>
    + AeadDecryptor<Key = <Self as AeadCipher>::Key, Error = <Self as AeadCipher>::Error>
{
    /// Key type for encryption/decryption.
    type Key: ?Sized;

    /// Error type for encryption/decryption.
    type Error;
}
