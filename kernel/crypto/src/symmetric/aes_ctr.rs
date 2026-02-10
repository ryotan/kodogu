use super::traits::{
    SymmetricCipher, SymmetricCipherError, SymmetricCiphers, SymmetricDecryptor,
    SymmetricEncryptor, validate_iv_length, validate_key_length,
};
use crate::{Ciphertext, InitializationVector, Plaintext};
use ctr::cipher::{KeyIvInit, StreamCipher};

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
/// use crypto::{Aes128Ctr, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
///
/// let cipher = Aes128Ctr;
/// let key = [0u8; 16];
/// let nonce = InitializationVector::from(vec![0u8; 16]);
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
/// use crypto::{Aes256Ctr, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
///
/// let cipher = Aes256Ctr;
/// let key = [0u8; 32];
/// let nonce = InitializationVector::from(vec![0u8; 16]);
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
/// use crypto::{AesCtr, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
///
/// let cipher = AesCtr::Aes128;
/// let key: &[u8] = &[0u8; 16];
/// let nonce = InitializationVector::from(vec![0u8; 16]);
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
/// Extends [`SymmetricCipher`] to indicate AES-CTR family membership.
///
/// # Example
///
/// ```
/// use crypto::{AesCtrCipher, Aes128Ctr, Aes256Ctr, AesCtr, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
///
/// // ZST type (compile-time key length verification)
/// let cipher = Aes128Ctr;
/// let key = [0u8; 16];
/// let nonce = InitializationVector::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
/// let ciphertext = cipher.encrypt(&plaintext, &key, &nonce).unwrap();
///
/// // Enum type (runtime key length verification)
/// let cipher = AesCtr::Aes128;
/// let key: &[u8] = &[0u8; 16];
/// let ciphertext = cipher.encrypt(&plaintext, key, &nonce).unwrap();
/// ```
pub trait AesCtrCipher: SymmetricCipher<Error = SymmetricCipherError> {}

type Aes128CtrCipher = ctr::Ctr128BE<aes::Aes128>;
type Aes256CtrCipher = ctr::Ctr128BE<aes::Aes256>;

impl SymmetricEncryptor for Aes128Ctr {
    type Key = [u8; AES_128_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, Self::Error> {
        let cipher = SymmetricCiphers::Aes128Ctr;
        let nonce = validate_iv_length::<AES_BLOCK_SIZE>(iv, cipher)?;
        let mut cipher = Aes128CtrCipher::new(key.into(), &nonce.into());
        let mut buffer = plaintext.as_bytes().to_vec();
        cipher.apply_keystream(&mut buffer);
        Ok(Ciphertext::from(buffer))
    }
}

impl SymmetricDecryptor for Aes128Ctr {
    type Key = [u8; AES_128_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Plaintext, Self::Error> {
        let cipher = SymmetricCiphers::Aes128Ctr;
        let nonce = validate_iv_length::<AES_BLOCK_SIZE>(iv, cipher)?;
        let mut cipher = Aes128CtrCipher::new(key.into(), &nonce.into());
        let mut buffer = ciphertext.as_bytes().to_vec();
        cipher.apply_keystream(&mut buffer);
        Ok(Plaintext::from(buffer))
    }
}

impl SymmetricCipher for Aes128Ctr {
    type Key = [u8; AES_128_KEY_SIZE];
    type Error = SymmetricCipherError;
}

impl AesCtrCipher for Aes128Ctr {}

impl SymmetricEncryptor for Aes256Ctr {
    type Key = [u8; AES_256_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, Self::Error> {
        let cipher = SymmetricCiphers::Aes256Ctr;
        let nonce = validate_iv_length::<AES_BLOCK_SIZE>(iv, cipher)?;
        let mut cipher = Aes256CtrCipher::new(key.into(), &nonce.into());
        let mut buffer = plaintext.as_bytes().to_vec();
        cipher.apply_keystream(&mut buffer);
        Ok(Ciphertext::from(buffer))
    }
}

impl SymmetricDecryptor for Aes256Ctr {
    type Key = [u8; AES_256_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Plaintext, Self::Error> {
        let cipher = SymmetricCiphers::Aes256Ctr;
        let nonce = validate_iv_length::<AES_BLOCK_SIZE>(iv, cipher)?;
        let mut cipher = Aes256CtrCipher::new(key.into(), &nonce.into());
        let mut buffer = ciphertext.as_bytes().to_vec();
        cipher.apply_keystream(&mut buffer);
        Ok(Plaintext::from(buffer))
    }
}

impl SymmetricCipher for Aes256Ctr {
    type Key = [u8; AES_256_KEY_SIZE];
    type Error = SymmetricCipherError;
}

impl AesCtrCipher for Aes256Ctr {}

impl SymmetricEncryptor for AesCtr {
    type Key = [u8];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, Self::Error> {
        match self {
            AesCtr::Aes128 => {
                let key_array = validate_key_length(key, SymmetricCiphers::Aes128Ctr)?;
                Aes128Ctr.encrypt(plaintext, &key_array, iv)
            }
            AesCtr::Aes256 => {
                let key_array = validate_key_length(key, SymmetricCiphers::Aes256Ctr)?;
                Aes256Ctr.encrypt(plaintext, &key_array, iv)
            }
        }
    }
}

impl SymmetricDecryptor for AesCtr {
    type Key = [u8];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Plaintext, Self::Error> {
        match self {
            AesCtr::Aes128 => {
                let key_array = validate_key_length(key, SymmetricCiphers::Aes128Ctr)?;
                Aes128Ctr.decrypt(ciphertext, &key_array, iv)
            }
            AesCtr::Aes256 => {
                let key_array = validate_key_length(key, SymmetricCiphers::Aes256Ctr)?;
                Aes256Ctr.decrypt(ciphertext, &key_array, iv)
            }
        }
    }
}

impl SymmetricCipher for AesCtr {
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
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes128ctr_invalid_nonce_length() {
        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce = InitializationVector::from(vec![0u8; 8]); // invalid
        let plaintext = Plaintext::from("Hello");

        let result = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes128ctr_decrypt_invalid_nonce_length() {
        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();

        let bad_nonce = InitializationVector::from(vec![0u8; 8]);
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &bad_nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes128ctr_no_padding_needed() {
        use super::super::traits::test_helpers::assert_no_padding_needed;

        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce = InitializationVector::from(vec![0u8; 16]);

        assert_no_padding_needed(
            &nonce,
            |pt, iv| SymmetricEncryptor::encrypt(&cipher, pt, &key, iv).unwrap(),
            |ct, iv| SymmetricDecryptor::decrypt(&cipher, ct, &key, iv).unwrap(),
        );
    }

    #[test]
    fn test_aes128ctr_aes_ctr_cipher_marker() {
        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = encrypt_with_aes_ctr(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256ctr_encrypt_decrypt_roundtrip() {
        let cipher = Aes256Ctr;
        let key = [0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256ctr_aes_ctr_cipher_marker() {
        let cipher = Aes256Ctr;
        let key = [0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = encrypt_with_aes_ctr(&cipher, &plaintext, &key, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256ctr_invalid_nonce_length_encrypt() {
        let cipher = Aes256Ctr;
        let key = [0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 8]); // invalid
        let plaintext = Plaintext::from("Hello");

        let result = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes256ctr_invalid_nonce_length_decrypt() {
        let cipher = Aes256Ctr;
        let key = [0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce).unwrap();

        let bad_nonce = InitializationVector::from(vec![0u8; 8]);
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &bad_nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes_ctr_enum_aes128() {
        let cipher = AesCtr::Aes128;
        let key: &[u8] = &[0u8; 16];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes_ctr_enum_aes256() {
        let cipher = AesCtr::Aes256;
        let key: &[u8] = &[0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes_ctr_enum_invalid_key_length() {
        let cipher = AesCtr::Aes128;
        let key: &[u8] = &[0u8; 8]; // invalid
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let result = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes_ctr_enum_decrypt_invalid_key_length() {
        let cipher = AesCtr::Aes128;
        let key: &[u8] = &[0u8; 16];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &nonce).unwrap();

        let bad_key: &[u8] = &[0u8; 8];
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, bad_key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes_ctr_enum_aes256_invalid_key_length() {
        let cipher = AesCtr::Aes256;
        let key: &[u8] = &[0u8; 16]; // too short for AES-256
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let result = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes_ctr_enum_aes256_decrypt_invalid_key_length() {
        let cipher = AesCtr::Aes256;
        let key: &[u8] = &[0u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &nonce).unwrap();

        let bad_key: &[u8] = &[0u8; 16];
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, bad_key, &nonce);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        use super::super::traits::test_helpers::assert_different_keys_produce_different_ciphertext;

        let cipher = Aes128Ctr;
        let key1 = [0u8; 16];
        let key2 = [1u8; 16];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext1 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key1, &nonce).unwrap();
        let ciphertext2 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key2, &nonce).unwrap();

        assert_different_keys_produce_different_ciphertext(&plaintext, &ciphertext1, &ciphertext2);
    }

    #[test]
    fn test_different_nonces_produce_different_ciphertext() {
        use super::super::traits::test_helpers::assert_different_ivs_produce_different_ciphertext;

        let cipher = Aes128Ctr;
        let key = [0u8; 16];
        let nonce1 = InitializationVector::from(vec![0u8; 16]);
        let nonce2 = InitializationVector::from(vec![1u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext1 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce1).unwrap();
        let ciphertext2 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &nonce2).unwrap();

        assert_different_ivs_produce_different_ciphertext(&plaintext, &ciphertext1, &ciphertext2);
    }

    #[test]
    fn test_wrong_key_produces_garbage() {
        let cipher = Aes128Ctr;
        let key1 = [0u8; 16];
        let key2 = [1u8; 16];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key1, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key2, &nonce).unwrap();

        // CTR doesn't detect the wrong key - just produces garbage
        assert_ne!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256ctr_wrong_key_produces_garbage() {
        let cipher = Aes256Ctr;
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let nonce = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key1, &nonce).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key2, &nonce).unwrap();

        // CTR doesn't detect the wrong key - just produces garbage
        assert_ne!(plaintext, decrypted);
    }

    fn encrypt_with_aes_ctr<C>(
        cipher: &C,
        plaintext: &Plaintext,
        key: &<C as SymmetricEncryptor>::Key,
        nonce: &InitializationVector,
    ) -> Result<Ciphertext, SymmetricCipherError>
    where
        C: AesCtrCipher,
    {
        cipher.encrypt(plaintext, key, nonce)
    }
}
