use super::errors::{SymmetricCipherError, SymmetricCiphers};
use super::helpers::validate_nonce_length;
use super::traits::{StreamCipher, StreamDecryptor, StreamEncryptor};
use crate::{Ciphertext, Nonce, Plaintext};
use chacha20::cipher::KeyIvInit;
use chacha20::cipher::StreamCipher as ChaChaStreamCipher;

const CHACHA20_KEY_SIZE: usize = 32;
const CHACHA20_NONCE_SIZE: usize = 12;

/// ChaCha20 stream cipher.
///
/// Uses 256-bit key with 96-bit nonce. No padding required.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Security Warning
///
/// Never reuse nonce with the same key. Doing so completely
/// compromises the security of ChaCha20.
///
/// # Example
///
/// ```
/// use crypto::{ChaCha20, StreamEncryptor, StreamDecryptor, Plaintext, Nonce};
///
/// let cipher = ChaCha20;
/// let key = [0u8; 32];
/// let nonce = Nonce::from(vec![0u8; 12]);
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
/// Extends [`StreamCipher`] to indicate ChaCha20 family membership.
pub trait ChaCha20Cipher: StreamCipher<Error = SymmetricCipherError> {}

type ChaCha20Impl = chacha20::ChaCha20;

impl StreamEncryptor for ChaCha20 {
    type Key = [u8; CHACHA20_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
    ) -> Result<Ciphertext, Self::Error> {
        let nonce_array =
            validate_nonce_length::<CHACHA20_NONCE_SIZE>(nonce, SymmetricCiphers::ChaCha20)?;
        let mut stream = ChaCha20Impl::new(key.into(), &nonce_array.into());
        let mut buffer = plaintext.as_bytes().to_vec();
        stream.apply_keystream(&mut buffer);
        Ok(Ciphertext::from(buffer))
    }
}

impl StreamDecryptor for ChaCha20 {
    type Key = [u8; CHACHA20_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        nonce: &Nonce,
    ) -> Result<Plaintext, Self::Error> {
        let nonce_array =
            validate_nonce_length::<CHACHA20_NONCE_SIZE>(nonce, SymmetricCiphers::ChaCha20)?;
        let mut stream = ChaCha20Impl::new(key.into(), &nonce_array.into());
        let mut buffer = ciphertext.as_bytes().to_vec();
        stream.apply_keystream(&mut buffer);
        Ok(Plaintext::from(buffer))
    }
}

impl StreamCipher for ChaCha20 {
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
        let nonce = Nonce::from(vec![0u8; 12]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_chacha20_invalid_nonce_length() {
        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce = Nonce::from(vec![0u8; 16]); // invalid (should be 12)
        let plaintext = Plaintext::from("Hello");

        let result = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidNonceLength { .. })));
    }

    #[test]
    fn test_chacha20_decrypt_invalid_nonce_length() {
        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce = Nonce::from(vec![0u8; 12]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();

        let bad_nonce = Nonce::from(vec![0u8; 8]);
        let result = StreamDecryptor::decrypt(&cipher, &ciphertext, &key, &bad_nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidNonceLength { .. })));
    }

    #[test]
    fn test_chacha20_no_padding_needed() {
        use super::super::helpers::test_helpers::assert_no_padding_needed;

        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce = Nonce::from(vec![0u8; 12]);

        assert_no_padding_needed(
            &nonce,
            |pt, n| StreamEncryptor::encrypt(&cipher, pt, &key, n).unwrap(),
            |ct, n| StreamDecryptor::decrypt(&cipher, ct, &key, n).unwrap(),
        );
    }

    #[test]
    fn test_chacha20_cipher_marker() {
        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce = Nonce::from(vec![0u8; 12]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = encrypt_with_chacha20(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        use super::super::helpers::test_helpers::assert_different_keys_produce_different_ciphertext;

        let cipher = ChaCha20;
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let nonce = Nonce::from(vec![0u8; 12]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext1 = StreamEncryptor::encrypt(&cipher, &plaintext, &key1, &nonce).unwrap();
        let ciphertext2 = StreamEncryptor::encrypt(&cipher, &plaintext, &key2, &nonce).unwrap();

        assert_different_keys_produce_different_ciphertext(&plaintext, &ciphertext1, &ciphertext2);
    }

    #[test]
    fn test_different_nonce_produce_different_ciphertext() {
        use super::super::helpers::test_helpers::assert_different_nonces_produce_different_ciphertext;

        let cipher = ChaCha20;
        let key = [0u8; 32];
        let nonce1 = Nonce::from(vec![0u8; 12]);
        let nonce2 = Nonce::from(vec![1u8; 12]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext1 = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce1).unwrap();
        let ciphertext2 = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce2).unwrap();

        assert_different_nonces_produce_different_ciphertext(
            &plaintext,
            &ciphertext1,
            &ciphertext2,
        );
    }

    #[test]
    fn test_wrong_key_produces_garbage() {
        let cipher = ChaCha20;
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let nonce = Nonce::from(vec![0u8; 12]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, &key1, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, &key2, &nonce).unwrap();

        // ChaCha20 doesn't detect the wrong key - just produces garbage
        assert_ne!(plaintext, decrypted);
    }

    fn encrypt_with_chacha20<C>(
        cipher: &C,
        plaintext: &Plaintext,
        key: &<C as StreamEncryptor>::Key,
        nonce: &Nonce,
    ) -> Result<Ciphertext, SymmetricCipherError>
    where
        C: ChaCha20Cipher,
    {
        cipher.encrypt(plaintext, key, nonce)
    }
}
