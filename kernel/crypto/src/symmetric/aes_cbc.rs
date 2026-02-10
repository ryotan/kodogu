use super::traits::{
    PaddingError, SymmetricCipher, SymmetricCipherError, SymmetricCiphers, SymmetricDecryptor,
    SymmetricEncryptor, validate_iv_length, validate_key_length,
};
use crate::{Ciphertext, InitializationVector, Plaintext};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};

const AES_BLOCK_SIZE: usize = 16;
const AES_128_KEY_SIZE: usize = 16;
const AES_256_KEY_SIZE: usize = 32;

/// AES-128-CBC cipher.
///
/// Uses 128-bit key with CBC mode and PKCS#7 padding.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{Aes128Cbc, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
///
/// let cipher = Aes128Cbc;
/// let key = [0u8; 16];
/// let iv = InitializationVector::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
///
/// let ciphertext = cipher.encrypt(&plaintext, &key, &iv).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &key, &iv).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Aes128Cbc;

/// AES-256-CBC cipher.
///
/// Uses 256-bit key with CBC mode and PKCS#7 padding.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{Aes256Cbc, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
///
/// let cipher = Aes256Cbc;
/// let key = [0u8; 32];
/// let iv = InitializationVector::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
///
/// let ciphertext = cipher.encrypt(&plaintext, &key, &iv).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &key, &iv).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Aes256Cbc;

/// AES-CBC cipher with runtime key size selection.
///
/// Use when key size is determined at runtime.
///
/// # Example
///
/// ```
/// use crypto::{AesCbc, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
///
/// let cipher = AesCbc::Aes128;
/// let key: &[u8] = &[0u8; 16];
/// let iv = InitializationVector::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
///
/// let ciphertext = cipher.encrypt(&plaintext, key, &iv).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, key, &iv).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum AesCbc {
    /// AES-128-CBC (128-bit key)
    #[default]
    Aes128,
    /// AES-256-CBC (256-bit key)
    Aes256,
}

/// Marker trait for AES-CBC cipher family.
///
/// Enables generic programming over all AES-CBC variants.
/// Extends [`SymmetricCipher`] to indicate AES-CBC family membership.
///
/// # Example
///
/// ```
/// use crypto::{AesCbcCipher, Aes128Cbc, Aes256Cbc, AesCbc, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
///
/// // ZST type (compile-time key length verification)
/// let cipher = Aes128Cbc;
/// let key = [0u8; 16];
/// let iv = InitializationVector::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
/// let ciphertext = cipher.encrypt(&plaintext, &key, &iv).unwrap();
///
/// // Enum type (runtime key length verification)
/// let cipher = AesCbc::Aes128;
/// let key: &[u8] = &[0u8; 16];
/// let ciphertext = cipher.encrypt(&plaintext, key, &iv).unwrap();
/// ```
pub trait AesCbcCipher: SymmetricCipher<Error = SymmetricCipherError> {}

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

impl SymmetricEncryptor for Aes128Cbc {
    type Key = [u8; AES_128_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, Self::Error> {
        let cipher = SymmetricCiphers::Aes128Cbc;
        let iv_array = validate_iv_length::<AES_BLOCK_SIZE>(iv, cipher)?;
        let encryptor = Aes128CbcEnc::new(key.into(), &iv_array.into());
        let ciphertext = encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());
        Ok(Ciphertext::from(ciphertext))
    }
}

impl SymmetricDecryptor for Aes128Cbc {
    type Key = [u8; AES_128_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Plaintext, Self::Error> {
        let cipher = SymmetricCiphers::Aes128Cbc;
        let iv_array = validate_iv_length::<AES_BLOCK_SIZE>(iv, cipher)?;
        let decryptor = Aes128CbcDec::new(key.into(), &iv_array.into());
        let plaintext = decryptor
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext.as_bytes())
            .map_err(|_| PaddingError { cipher: SymmetricCiphers::Aes128Cbc }.build())?;
        Ok(Plaintext::from(plaintext))
    }
}

impl SymmetricCipher for Aes128Cbc {
    type Key = [u8; AES_128_KEY_SIZE];
    type Error = SymmetricCipherError;
}

impl AesCbcCipher for Aes128Cbc {}

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

impl SymmetricEncryptor for Aes256Cbc {
    type Key = [u8; AES_256_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, Self::Error> {
        let cipher = SymmetricCiphers::Aes256Cbc;
        let iv_array = validate_iv_length::<AES_BLOCK_SIZE>(iv, cipher)?;
        let encryptor = Aes256CbcEnc::new(key.into(), &iv_array.into());
        let ciphertext = encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());
        Ok(Ciphertext::from(ciphertext))
    }
}

impl SymmetricDecryptor for Aes256Cbc {
    type Key = [u8; AES_256_KEY_SIZE];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Plaintext, Self::Error> {
        let cipher = SymmetricCiphers::Aes256Cbc;
        let iv_array = validate_iv_length::<AES_BLOCK_SIZE>(iv, cipher)?;
        let decryptor = Aes256CbcDec::new(key.into(), &iv_array.into());
        let plaintext = decryptor
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext.as_bytes())
            .map_err(|_| PaddingError { cipher: SymmetricCiphers::Aes256Cbc }.build())?;
        Ok(Plaintext::from(plaintext))
    }
}

impl SymmetricCipher for Aes256Cbc {
    type Key = [u8; AES_256_KEY_SIZE];
    type Error = SymmetricCipherError;
}

impl AesCbcCipher for Aes256Cbc {}

impl SymmetricEncryptor for AesCbc {
    type Key = [u8];
    type Error = SymmetricCipherError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, Self::Error> {
        match self {
            AesCbc::Aes128 => {
                let key_array =
                    validate_key_length::<AES_128_KEY_SIZE>(key, SymmetricCiphers::Aes128Cbc)?;
                SymmetricEncryptor::encrypt(&Aes128Cbc, plaintext, &key_array, iv)
            }
            AesCbc::Aes256 => {
                let key_array =
                    validate_key_length::<AES_256_KEY_SIZE>(key, SymmetricCiphers::Aes256Cbc)?;
                SymmetricEncryptor::encrypt(&Aes256Cbc, plaintext, &key_array, iv)
            }
        }
    }
}

impl SymmetricDecryptor for AesCbc {
    type Key = [u8];
    type Error = SymmetricCipherError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Plaintext, Self::Error> {
        match self {
            AesCbc::Aes128 => {
                let key_array =
                    validate_key_length::<AES_128_KEY_SIZE>(key, SymmetricCiphers::Aes128Cbc)?;
                SymmetricDecryptor::decrypt(&Aes128Cbc, ciphertext, &key_array, iv)
            }
            AesCbc::Aes256 => {
                let key_array =
                    validate_key_length::<AES_256_KEY_SIZE>(key, SymmetricCiphers::Aes256Cbc)?;
                SymmetricDecryptor::decrypt(&Aes256Cbc, ciphertext, &key_array, iv)
            }
        }
    }
}

impl SymmetricCipher for AesCbc {
    type Key = [u8];
    type Error = SymmetricCipherError;
}

impl AesCbcCipher for AesCbc {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes128cbc_encrypt_decrypt_roundtrip() {
        let cipher = Aes128Cbc;
        let key = [0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes128cbc_invalid_iv_length() {
        let cipher = Aes128Cbc;
        let key = [0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 8]); // invalid length
        let plaintext = Plaintext::from("Hello");

        let result = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes128cbc_padding() {
        let cipher = Aes128Cbc;
        let key = [0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);

        // Test various plaintext lengths (padding boundary conditions)
        for len in 0..=32 {
            let plaintext = Plaintext::from(vec![0xABu8; len]);
            let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv).unwrap();
            let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();
            assert_eq!(plaintext, decrypted, "Failed for length {}", len);
        }
    }

    #[test]
    fn test_aes128cbc_aes_cbc_cipher_marker() {
        let cipher = Aes128Cbc;
        let key = [0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = encrypt_with_aes_cbc(&cipher, &plaintext, &key, &iv).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes_cbc_enum_invalid_key_length() {
        let cipher = AesCbc::Aes128;
        let key: &[u8] = &[0u8; 8]; // invalid length
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let result = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes256cbc_encrypt_decrypt_roundtrip() {
        let cipher = Aes256Cbc;
        let key = [0u8; 32];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes256cbc_aes_cbc_cipher_marker() {
        let cipher = Aes256Cbc;
        let key = [0u8; 32];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = encrypt_with_aes_cbc(&cipher, &plaintext, &key, &iv).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes_cbc_enum_aes128() {
        let cipher = AesCbc::Aes128;
        let key: &[u8] = &[0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &iv).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, key, &iv).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes_cbc_enum_aes256() {
        let cipher = AesCbc::Aes256;
        let key: &[u8] = &[0u8; 32];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &iv).unwrap();
        let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, key, &iv).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        let cipher = Aes128Cbc;
        let key1 = [0u8; 16];
        let key2 = [1u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext1 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key1, &iv).unwrap();
        let ciphertext2 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key2, &iv).unwrap();

        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_different_ivs_produce_different_ciphertext() {
        let cipher = Aes128Cbc;
        let key = [0u8; 16];
        let iv1 = InitializationVector::from(vec![0u8; 16]);
        let iv2 = InitializationVector::from(vec![1u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext1 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv1).unwrap();
        let ciphertext2 = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv2).unwrap();

        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let cipher = Aes128Cbc;
        let key1 = [0u8; 16];
        let key2 = [1u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key1, &iv).unwrap();

        // Decryption with a wrong key causes a padding error
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key2, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::Padding { .. })));
    }

    #[test]
    fn test_aes128cbc_decrypt_invalid_iv_length() {
        let cipher = Aes128Cbc;
        let key = [0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv).unwrap();

        let bad_iv = InitializationVector::from(vec![0u8; 8]);
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &bad_iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes_cbc_enum_decrypt_invalid_key_length() {
        let cipher = AesCbc::Aes128;
        let key: &[u8] = &[0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &iv).unwrap();

        let bad_key: &[u8] = &[0u8; 8];
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, bad_key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes256cbc_invalid_iv_length_encrypt() {
        let cipher = Aes256Cbc;
        let key = [0u8; 32];
        let iv = InitializationVector::from(vec![0u8; 8]); // invalid length
        let plaintext = Plaintext::from("Hello");

        let result = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes256cbc_invalid_iv_length_decrypt() {
        let cipher = Aes256Cbc;
        let key = [0u8; 32];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv).unwrap();

        let bad_iv = InitializationVector::from(vec![0u8; 8]);
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &bad_iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes_cbc_enum_aes256_invalid_key_length() {
        let cipher = AesCbc::Aes256;
        let key: &[u8] = &[0u8; 16]; // too short for AES-256
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let result = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes_cbc_enum_aes256_decrypt_invalid_key_length() {
        let cipher = AesCbc::Aes256;
        let key: &[u8] = &[0u8; 32];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, key, &iv).unwrap();

        let bad_key: &[u8] = &[0u8; 16];
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, bad_key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_plaintext_conversions() {
        let bytes: &[u8] = b"Hello";
        let pt = Plaintext::from(bytes);
        assert_eq!(pt.as_bytes(), b"Hello");

        let pt2 = Plaintext::from(String::from("World"));
        assert_eq!(pt2.as_bytes(), b"World");

        let bytes = pt2.into_bytes();
        assert_eq!(bytes, b"World");
    }

    #[test]
    fn test_ciphertext_conversions() {
        let bytes: &[u8] = b"encrypted";
        let ct = Ciphertext::from(bytes);
        assert_eq!(ct.as_bytes(), b"encrypted");

        let bytes = ct.into_bytes();
        assert_eq!(bytes, b"encrypted");
    }

    #[test]
    fn test_iv_conversions() {
        let bytes: &[u8] = &[1, 2, 3, 4];
        let iv = InitializationVector::from(bytes);
        assert_eq!(iv.as_bytes(), &[1, 2, 3, 4]);
        assert_eq!(iv.len(), 4);
        assert!(!iv.is_empty());

        let empty_iv = InitializationVector::from(vec![]);
        assert!(empty_iv.is_empty());
    }

    #[test]
    fn test_aes256cbc_wrong_key_fails_decryption() {
        let cipher = Aes256Cbc;
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello, World!");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key1, &iv).unwrap();

        // Decryption with a wrong key causes a padding error
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key2, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::Padding { .. })));
    }

    fn encrypt_with_aes_cbc<C>(
        cipher: &C,
        plaintext: &Plaintext,
        key: &<C as SymmetricEncryptor>::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, SymmetricCipherError>
    where
        C: AesCbcCipher,
    {
        cipher.encrypt(plaintext, key, iv)
    }
}
