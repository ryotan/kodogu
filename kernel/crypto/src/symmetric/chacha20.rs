use super::traits::{
    SymmetricCipher, SymmetricCipherError, SymmetricCiphers, SymmetricDecryptor,
    SymmetricEncryptor, validate_iv_length,
};
use crate::{Ciphertext, InitializationVector, Plaintext};
use chacha20::cipher::{KeyIvInit, StreamCipher};

const CHACHA20_KEY_SIZE: usize = 32;
const CHACHA20_NONCE_SIZE: usize = 12;

/// ChaCha20 stream cipher.
///
/// Uses 256-bit key with 96-bit nonce. No padding required.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Security Warning
///
/// Never reuse a nonce with the same key. Doing so completely
/// compromises the security of ChaCha20.
///
/// # Example
///
/// ```
/// use crypto::{ChaCha20, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
///
/// let cipher = ChaCha20;
/// let key = [0u8; 32];
/// let nonce = InitializationVector::from(vec![0u8; 12]);
/// let plaintext = Plaintext::from("Hello");
///
/// let ciphertext = cipher.encrypt(&plaintext, &key, &nonce).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &key, &nonce).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct ChaCha20;

/// Marker trait for ChaCha20 cipher family.
///
/// Enables generic programming over ChaCha20 variants.
/// Extends [`SymmetricCipher`] to indicate ChaCha20 family membership.
pub trait ChaCha20Cipher: SymmetricCipher<Error = SymmetricCipherError> {}

type ChaCha20Core = chacha20::ChaCha20;

impl SymmetricEncryptor for ChaCha20 {
    type Key = [u8; CHACHA20_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, Self::Error> {
        let nonce = validate_iv_length::<CHACHA20_NONCE_SIZE>(iv, SymmetricCiphers::ChaCha20)?;
        let mut cipher = ChaCha20Core::new(key.into(), &nonce.into());
        let mut buffer = plaintext.as_bytes().to_vec();
        cipher.apply_keystream(&mut buffer);
        Ok(Ciphertext::from(buffer))
    }
}

impl SymmetricDecryptor for ChaCha20 {
    type Key = [u8; CHACHA20_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Plaintext, Self::Error> {
        let nonce = validate_iv_length::<CHACHA20_NONCE_SIZE>(iv, SymmetricCiphers::ChaCha20)?;
        let mut cipher = ChaCha20Core::new(key.into(), &nonce.into());
        let mut buffer = ciphertext.as_bytes().to_vec();
        cipher.apply_keystream(&mut buffer);
        Ok(Plaintext::from(buffer))
    }
}

impl SymmetricCipher for ChaCha20 {
    type Key = [u8; CHACHA20_KEY_SIZE];
    type Error = SymmetricCipherError;
}

impl ChaCha20Cipher for ChaCha20 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chacha20_encrypt_decrypt_roundtrip() {
        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 12]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_chacha20_invalid_nonce_length() {
        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 16]); // invalid (should be 12)
        let plaintext = Plaintext::from("Hello");

        let result = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_chacha20_decrypt_invalid_nonce_length() {
        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 12]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();

        let bad_nonce = InitializationVector::from(vec![0u8; 8]);
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &bad_nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_chacha20_no_padding_needed() {
        use super::super::traits::test_helpers::assert_no_padding_needed;

        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 12]);

        assert_no_padding_needed(
            &nonce,
            |pt, iv| SymmetricEncryptor::encrypt(&cipher, pt, &key, iv).unwrap(),
            |ct, iv| SymmetricDecryptor::decrypt(&cipher, ct, &key, iv).unwrap(),
        );
    }

    #[test]
    fn test_chacha20_cipher_marker() {
        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 12]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = encrypt_with_chacha20(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        use super::super::traits::test_helpers::assert_different_keys_produce_different_ciphertext;

        let cipher = ChaCha20;
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 12]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext1 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key1, &nonce).unwrap();
        let ciphertext2 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key2, &nonce).unwrap();

        assert_different_keys_produce_different_ciphertext(&plaintext, &ciphertext1, &ciphertext2);
    }

    #[test]
    fn test_different_nonces_produce_different_ciphertext() {
        use super::super::traits::test_helpers::assert_different_ivs_produce_different_ciphertext;

        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce1 = InitializationVector::from(vec![0u8; 12]);
        let nonce2 = InitializationVector::from(vec![1u8; 12]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext1 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce1).unwrap();
        let ciphertext2 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce2).unwrap();

        assert_different_ivs_produce_different_ciphertext(&plaintext, &ciphertext1, &ciphertext2);
    }

    #[test]
    fn test_wrong_key_produces_garbage() {
        let cipher = ChaCha20;
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 12]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key1, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key2, &nonce).unwrap();

        // ChaCha20 doesn't detect wrong key - just produces garbage
        assert_ne!(plaintext, decrypted);
    }

    fn encrypt_with_chacha20<C>(
        cipher: &C,
        plaintext: &Plaintext,
        key: &<C as SymmetricEncryptor>::Key,
        nonce: &InitializationVector,
    ) -> Result<Ciphertext, SymmetricCipherError>
    where
        C: ChaCha20Cipher,
    {
        cipher.encrypt(plaintext, key, nonce)
    }
}
