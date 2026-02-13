use super::errors::{SymmetricCipherError, SymmetricCiphers};
use super::helpers::{validate_key_length, validate_nonce_length};
use super::traits::{StreamCipher, StreamDecryptor, StreamEncryptor};
use crate::{Ciphertext, Nonce, Plaintext};
use ctr::cipher::KeyIvInit;
use ctr::cipher::StreamCipher as CtrStreamCipher;

const AES_BLOCK_SIZE: usize = 16;
const AES_128_KEY_SIZE: usize = 16;
const AES_256_KEY_SIZE: usize = 32;

/// AES-128-CTR cipher.
///
/// Uses 128-bit key with CTR mode. No padding required.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Security Warning
///
/// Never reuse a nonce with the same key. Doing so completely
/// compromises the security of CTR mode.
///
/// # Example
///
/// ```
/// use crypto::{Aes128Ctr, StreamEncryptor, StreamDecryptor, Plaintext, Nonce};
///
/// let cipher = Aes128Ctr;
/// let key = [0u8; 16];
/// let nonce = Nonce::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
///
/// let ciphertext = cipher.encrypt(&plaintext, &key, &nonce).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &key, &nonce).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Aes128Ctr;

/// AES-256-CTR cipher.
///
/// Uses 256-bit key with CTR mode. No padding required.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Security Warning
///
/// Never reuse a nonce with the same key. Doing so completely
/// compromises the security of CTR mode.
///
/// # Example
///
/// ```
/// use crypto::{Aes256Ctr, StreamEncryptor, StreamDecryptor, Plaintext, Nonce};
///
/// let cipher = Aes256Ctr;
/// let key = [0u8; 32];
/// let nonce = Nonce::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
///
/// let ciphertext = cipher.encrypt(&plaintext, &key, &nonce).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &key, &nonce).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Aes256Ctr;

/// AES-CTR cipher with runtime key size selection.
///
/// Use when key size is determined at runtime.
///
/// # Example
///
/// ```
/// use crypto::{AesCtr, StreamEncryptor, StreamDecryptor, Plaintext, Nonce};
///
/// let cipher = AesCtr::Aes128;
/// let key: &[u8] = &[0u8; 16];
/// let nonce = Nonce::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
///
/// let ciphertext = cipher.encrypt(&plaintext, key, &nonce).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, key, &nonce).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum AesCtr {
    /// AES-128-CTR (128-bit key)
    #[default]
    Aes128,
    /// AES-256-CTR (256-bit key)
    Aes256,
}

/// Marker trait for AES-CTR cipher family.
///
/// Enables generic programming over all AES-CTR variants.
/// Extends [`StreamCipher`] to indicate AES-CTR family membership.
///
/// # Example
///
/// ```
/// use crypto::{AesCtrCipher, Aes128Ctr, Aes256Ctr, AesCtr, StreamEncryptor, StreamDecryptor, Plaintext, Nonce};
///
/// // ZST type (compile-time key length verification)
/// let cipher = Aes128Ctr;
/// let key = [0u8; 16];
/// let nonce = Nonce::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
/// let ciphertext = cipher.encrypt(&plaintext, &key, &nonce).unwrap();
///
/// // Enum type (runtime key length verification)
/// let cipher = AesCtr::Aes128;
/// let key: &[u8] = &[0u8; 16];
/// let ciphertext = cipher.encrypt(&plaintext, key, &nonce).unwrap();
/// ```
pub trait AesCtrCipher: StreamCipher<Error = SymmetricCipherError> {}

type Aes128CtrImpl = ctr::Ctr128BE<aes::Aes128>;
type Aes256CtrImpl = ctr::Ctr128BE<aes::Aes256>;

impl StreamEncryptor for Aes128Ctr {
    type Key = [u8; AES_128_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
    ) -> Result<Ciphertext, Self::Error> {
        let cipher = SymmetricCiphers::Aes128Ctr;
        let nonce_array = validate_nonce_length::<AES_BLOCK_SIZE>(nonce, cipher)?;
        let mut stream = Aes128CtrImpl::new(key.into(), &nonce_array.into());
        let mut buffer = plaintext.as_bytes().to_vec();
        stream.apply_keystream(&mut buffer);
        Ok(Ciphertext::from(buffer))
    }
}

impl StreamDecryptor for Aes128Ctr {
    type Key = [u8; AES_128_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        nonce: &Nonce,
    ) -> Result<Plaintext, Self::Error> {
        let cipher = SymmetricCiphers::Aes128Ctr;
        let nonce_array = validate_nonce_length::<AES_BLOCK_SIZE>(nonce, cipher)?;
        let mut stream = Aes128CtrImpl::new(key.into(), &nonce_array.into());
        let mut buffer = ciphertext.as_bytes().to_vec();
        stream.apply_keystream(&mut buffer);
        Ok(Plaintext::from(buffer))
    }
}

impl StreamCipher for Aes128Ctr {
    type Key = [u8; AES_128_KEY_SIZE];
    type Error = SymmetricCipherError;
}

impl AesCtrCipher for Aes128Ctr {}

impl StreamEncryptor for Aes256Ctr {
    type Key = [u8; AES_256_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
    ) -> Result<Ciphertext, Self::Error> {
        let cipher = SymmetricCiphers::Aes256Ctr;
        let nonce_array = validate_nonce_length::<AES_BLOCK_SIZE>(nonce, cipher)?;
        let mut stream = Aes256CtrImpl::new(key.into(), &nonce_array.into());
        let mut buffer = plaintext.as_bytes().to_vec();
        stream.apply_keystream(&mut buffer);
        Ok(Ciphertext::from(buffer))
    }
}

impl StreamDecryptor for Aes256Ctr {
    type Key = [u8; AES_256_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        nonce: &Nonce,
    ) -> Result<Plaintext, Self::Error> {
        let cipher = SymmetricCiphers::Aes256Ctr;
        let nonce_array = validate_nonce_length::<AES_BLOCK_SIZE>(nonce, cipher)?;
        let mut stream = Aes256CtrImpl::new(key.into(), &nonce_array.into());
        let mut buffer = ciphertext.as_bytes().to_vec();
        stream.apply_keystream(&mut buffer);
        Ok(Plaintext::from(buffer))
    }
}

impl StreamCipher for Aes256Ctr {
    type Key = [u8; AES_256_KEY_SIZE];
    type Error = SymmetricCipherError;
}

impl AesCtrCipher for Aes256Ctr {}

impl StreamEncryptor for AesCtr {
    type Key = [u8];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
    ) -> Result<Ciphertext, Self::Error> {
        match self {
            AesCtr::Aes128 => {
                let key_array = validate_key_length(key, SymmetricCiphers::Aes128Ctr)?;
                Aes128Ctr.encrypt(plaintext, &key_array, nonce)
            }
            AesCtr::Aes256 => {
                let key_array = validate_key_length(key, SymmetricCiphers::Aes256Ctr)?;
                Aes256Ctr.encrypt(plaintext, &key_array, nonce)
            }
        }
    }
}

impl StreamDecryptor for AesCtr {
    type Key = [u8];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        nonce: &Nonce,
    ) -> Result<Plaintext, Self::Error> {
        match self {
            AesCtr::Aes128 => {
                let key_array = validate_key_length(key, SymmetricCiphers::Aes128Ctr)?;
                Aes128Ctr.decrypt(ciphertext, &key_array, nonce)
            }
            AesCtr::Aes256 => {
                let key_array = validate_key_length(key, SymmetricCiphers::Aes256Ctr)?;
                Aes256Ctr.decrypt(ciphertext, &key_array, nonce)
            }
        }
    }
}

impl StreamCipher for AesCtr {
    type Key = [u8];
    type Error = SymmetricCipherError;
}

impl AesCtrCipher for AesCtr {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes128ctr_encrypt_decrypt_roundtrip() {
        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes128ctr_invalid_nonce_length() {
        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce = Nonce::from(vec![0u8; 8]); // invalid
        let plaintext = Plaintext::from("Hello");

        let result = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidNonceLength { .. })));
    }

    #[test]
    fn test_aes128ctr_decrypt_invalid_nonce_length() {
        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();

        let bad_nonce = Nonce::from(vec![0u8; 8]);
        let result = StreamDecryptor::decrypt(&cipher, &ciphertext, &key, &bad_nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidNonceLength { .. })));
    }

    #[test]
    fn test_aes128ctr_no_padding_needed() {
        use super::super::helpers::test_helpers::assert_no_padding_needed;

        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce = Nonce::from(vec![0u8; 16]);

        assert_no_padding_needed(
            &nonce,
            |pt, n| StreamEncryptor::encrypt(&cipher, pt, &key, n).unwrap(),
            |ct, n| StreamDecryptor::decrypt(&cipher, ct, &key, n).unwrap(),
        );
    }

    #[test]
    fn test_aes128ctr_aes_ctr_cipher_marker() {
        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = encrypt_with_aes_ctr(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256ctr_encrypt_decrypt_roundtrip() {
        let cipher = Aes256Ctr;
        let key = [0u8; 32];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256ctr_aes_ctr_cipher_marker() {
        let cipher = Aes256Ctr;
        let key = [0u8; 32];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = encrypt_with_aes_ctr(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256ctr_invalid_nonce_length_encrypt() {
        let cipher = Aes256Ctr;
        let key = [0u8; 32];
        let nonce = Nonce::from(vec![0u8; 8]); // invalid
        let plaintext = Plaintext::from("Hello");

        let result = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidNonceLength { .. })));
    }

    #[test]
    fn test_aes256ctr_invalid_nonce_length_decrypt() {
        let cipher = Aes256Ctr;
        let key = [0u8; 32];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();

        let bad_nonce = Nonce::from(vec![0u8; 8]);
        let result = StreamDecryptor::decrypt(&cipher, &ciphertext, &key, &bad_nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidNonceLength { .. })));
    }

    #[test]
    fn test_aes_ctr_enum_aes128() {
        let cipher = AesCtr::Aes128;
        let key: &[u8] = &[0u8; 16];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, key, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes_ctr_enum_aes256() {
        let cipher = AesCtr::Aes256;
        let key: &[u8] = &[0u8; 32];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, key, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes_ctr_enum_invalid_key_length() {
        let cipher = AesCtr::Aes128;
        let key: &[u8] = &[0u8; 8]; // invalid
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let result = StreamEncryptor::encrypt(&cipher, &plaintext, key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes_ctr_enum_decrypt_invalid_key_length() {
        let cipher = AesCtr::Aes128;
        let key: &[u8] = &[0u8; 16];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, key, &nonce).unwrap();

        let bad_key: &[u8] = &[0u8; 8];
        let result = StreamDecryptor::decrypt(&cipher, &ciphertext, bad_key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes_ctr_enum_aes256_invalid_key_length() {
        let cipher = AesCtr::Aes256;
        let key: &[u8] = &[0u8; 16]; // too short for AES-256
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let result = StreamEncryptor::encrypt(&cipher, &plaintext, key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes_ctr_enum_aes256_decrypt_invalid_key_length() {
        let cipher = AesCtr::Aes256;
        let key: &[u8] = &[0u8; 32];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, key, &nonce).unwrap();

        let bad_key: &[u8] = &[0u8; 16];
        let result = StreamDecryptor::decrypt(&cipher, &ciphertext, bad_key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        use super::super::helpers::test_helpers::assert_different_keys_produce_different_ciphertext;

        let cipher = Aes128Ctr;
        let key1 = [0u8; 16];
        let key2 = [1u8; 16];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext1 = StreamEncryptor::encrypt(&cipher, &plaintext, &key1, &nonce).unwrap();
        let ciphertext2 = StreamEncryptor::encrypt(&cipher, &plaintext, &key2, &nonce).unwrap();

        assert_different_keys_produce_different_ciphertext(&plaintext, &ciphertext1, &ciphertext2);
    }

    #[test]
    fn test_different_nonce_produce_different_ciphertext() {
        use super::super::helpers::test_helpers::assert_different_nonces_produce_different_ciphertext;

        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce1 = Nonce::from(vec![0u8; 16]);
        let nonce2 = Nonce::from(vec![1u8; 16]);
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
        let cipher = Aes128Ctr;
        let key1 = [0u8; 16];
        let key2 = [1u8; 16];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, &key1, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, &key2, &nonce).unwrap();

        // CTR doesn't detect the wrong key - just produces garbage
        assert_ne!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256ctr_wrong_key_produces_garbage() {
        let cipher = Aes256Ctr;
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let nonce = Nonce::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = StreamEncryptor::encrypt(&cipher, &plaintext, &key1, &nonce).unwrap();
        let decrypted = StreamDecryptor::decrypt(&cipher, &ciphertext, &key2, &nonce).unwrap();

        // CTR doesn't detect the wrong key - just produces garbage
        assert_ne!(plaintext, decrypted);
    }

    fn encrypt_with_aes_ctr<C>(
        cipher: &C,
        plaintext: &Plaintext,
        key: &<C as StreamEncryptor>::Key,
        nonce: &Nonce,
    ) -> Result<Ciphertext, SymmetricCipherError>
    where
        C: AesCtrCipher,
    {
        cipher.encrypt(plaintext, key, nonce)
    }
}
