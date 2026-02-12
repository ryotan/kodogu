use super::errors::{AeadCiphers, AeadError, AuthenticationFailedError};
use super::helpers::{validate_key_length, validate_nonce_length, validate_tag_length};
use super::traits::{AeadCipher, AeadDecryptor, AeadEncryptor};
use super::types::{AdditionalData, Aes128GcmKey, Aes256GcmKey, AuthenticationTag};
use crate::Nonce;
use crate::{Ciphertext, Plaintext};
use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes128Gcm as Aes128GcmImpl, Aes256Gcm as Aes256GcmImpl};

const AES_GCM_NONCE_SIZE: usize = 12;
const AES_GCM_TAG_SIZE: usize = 16;
const AES_128_KEY_SIZE: usize = 16;
const AES_256_KEY_SIZE: usize = 32;

/// AES-128-GCM cipher.
///
/// Uses 128-bit key with GCM mode for authenticated encryption.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{Aes128Gcm, AeadEncryptor, AeadDecryptor, Plaintext, Nonce, AdditionalData, Aes128GcmKey};
///
/// let cipher = Aes128Gcm;
/// let key = Aes128GcmKey::from([0u8; 16]);
/// let nonce = Nonce::from(vec![0u8; 12]);
/// let aad = AdditionalData::empty();
/// let plaintext = Plaintext::from("Hello");
///
/// let (ciphertext, tag) = cipher.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Aes128Gcm;

/// AES-256-GCM cipher.
///
/// Uses 256-bit key with GCM mode for authenticated encryption.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{Aes256Gcm, AeadEncryptor, AeadDecryptor, Plaintext, Nonce, AdditionalData, Aes256GcmKey};
///
/// let cipher = Aes256Gcm;
/// let key = Aes256GcmKey::from([0u8; 32]);
/// let nonce = Nonce::from(vec![0u8; 12]);
/// let aad = AdditionalData::empty();
/// let plaintext = Plaintext::from("Hello");
///
/// let (ciphertext, tag) = cipher.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Aes256Gcm;

/// AES-GCM cipher with runtime key size selection.
///
/// Use when key size is determined at runtime.
///
/// # Example
///
/// ```
/// use crypto::{AesGcm, AeadEncryptor, AeadDecryptor, Plaintext, Nonce, AdditionalData};
///
/// let cipher = AesGcm::Aes256;
/// let key: &[u8] = &[0u8; 32];
/// let nonce = Nonce::from(vec![0u8; 12]);
/// let aad = AdditionalData::empty();
/// let plaintext = Plaintext::from("Hello");
///
/// let (ciphertext, tag) = cipher.encrypt(&plaintext, key, &nonce, &aad).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &tag, key, &nonce, &aad).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum AesGcm {
    /// AES-128-GCM (128-bit key)
    Aes128,
    /// AES-256-GCM (256-bit key)
    #[default]
    Aes256,
}

/// Marker trait for AES-GCM cipher family.
///
/// Enables generic programming over all AES-GCM variants.
pub trait AesGcmCipher: AeadCipher<Error = AeadError> {}

impl AeadEncryptor for Aes128Gcm {
    type Key = Aes128GcmKey;
    type Error = AeadError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<(Ciphertext, AuthenticationTag), Self::Error> {
        let cipher = AeadCiphers::Aes128Gcm;
        let nonce_array = validate_nonce_length::<AES_GCM_NONCE_SIZE>(nonce, cipher)?;

        let aes_gcm = Aes128GcmImpl::new((&key.0).into());
        let payload = Payload { msg: plaintext.as_bytes(), aad: aad.as_bytes() };

        let combined = aes_gcm
            .encrypt((&nonce_array).into(), payload)
            .map_err(|_| AuthenticationFailedError { cipher }.build())?;

        // aes-gcm は ciphertext || tag を返す
        let (ct, tag) = combined.split_at(combined.len() - AES_GCM_TAG_SIZE);
        Ok((Ciphertext::from(ct), AuthenticationTag::from(tag)))
    }
}

impl AeadDecryptor for Aes128Gcm {
    type Key = Aes128GcmKey;
    type Error = AeadError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        tag: &AuthenticationTag,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<Plaintext, Self::Error> {
        let cipher = AeadCiphers::Aes128Gcm;
        let nonce_array = validate_nonce_length::<AES_GCM_NONCE_SIZE>(nonce, cipher)?;
        let tag_array = validate_tag_length::<AES_GCM_TAG_SIZE>(tag, cipher)?;

        let aes_gcm = Aes128GcmImpl::new((&key.0).into());

        // aes-gcm は ciphertext || tag を期待する
        let mut combined = ciphertext.as_bytes().to_vec();
        combined.extend_from_slice(&tag_array);

        let payload = Payload { msg: &combined, aad: aad.as_bytes() };

        let plaintext = aes_gcm
            .decrypt((&nonce_array).into(), payload)
            .map_err(|_| AuthenticationFailedError { cipher }.build())?;

        Ok(Plaintext::from(plaintext))
    }
}

impl AeadCipher for Aes128Gcm {
    type Key = Aes128GcmKey;
    type Error = AeadError;
}

impl AesGcmCipher for Aes128Gcm {}

impl AeadEncryptor for Aes256Gcm {
    type Key = Aes256GcmKey;
    type Error = AeadError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<(Ciphertext, AuthenticationTag), Self::Error> {
        let cipher = AeadCiphers::Aes256Gcm;
        let nonce_array = validate_nonce_length::<AES_GCM_NONCE_SIZE>(nonce, cipher)?;

        let aes_gcm = Aes256GcmImpl::new((&key.0).into());
        let payload = Payload { msg: plaintext.as_bytes(), aad: aad.as_bytes() };

        let combined = aes_gcm
            .encrypt((&nonce_array).into(), payload)
            .map_err(|_| AuthenticationFailedError { cipher }.build())?;

        let (ct, tag) = combined.split_at(combined.len() - AES_GCM_TAG_SIZE);
        Ok((Ciphertext::from(ct), AuthenticationTag::from(tag)))
    }
}

impl AeadDecryptor for Aes256Gcm {
    type Key = Aes256GcmKey;
    type Error = AeadError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        tag: &AuthenticationTag,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<Plaintext, Self::Error> {
        let cipher = AeadCiphers::Aes256Gcm;
        let nonce_array = validate_nonce_length::<AES_GCM_NONCE_SIZE>(nonce, cipher)?;
        let tag_array = validate_tag_length::<AES_GCM_TAG_SIZE>(tag, cipher)?;

        let aes_gcm = Aes256GcmImpl::new((&key.0).into());

        let mut combined = ciphertext.as_bytes().to_vec();
        combined.extend_from_slice(&tag_array);

        let payload = Payload { msg: &combined, aad: aad.as_bytes() };

        let plaintext = aes_gcm
            .decrypt((&nonce_array).into(), payload)
            .map_err(|_| AuthenticationFailedError { cipher }.build())?;

        Ok(Plaintext::from(plaintext))
    }
}

impl AeadCipher for Aes256Gcm {
    type Key = Aes256GcmKey;
    type Error = AeadError;
}

impl AesGcmCipher for Aes256Gcm {}

impl AeadEncryptor for AesGcm {
    type Key = [u8];
    type Error = AeadError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<(Ciphertext, AuthenticationTag), Self::Error> {
        match self {
            AesGcm::Aes128 => {
                let key_array = validate_key_length::<AES_128_KEY_SIZE>(key, AeadCiphers::Aes128Gcm)?;
                AeadEncryptor::encrypt(&Aes128Gcm, plaintext, &Aes128GcmKey(key_array), nonce, aad)
            }
            AesGcm::Aes256 => {
                let key_array = validate_key_length::<AES_256_KEY_SIZE>(key, AeadCiphers::Aes256Gcm)?;
                AeadEncryptor::encrypt(&Aes256Gcm, plaintext, &Aes256GcmKey(key_array), nonce, aad)
            }
        }
    }
}

impl AeadDecryptor for AesGcm {
    type Key = [u8];
    type Error = AeadError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        tag: &AuthenticationTag,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<Plaintext, Self::Error> {
        match self {
            AesGcm::Aes128 => {
                let key_array = validate_key_length::<AES_128_KEY_SIZE>(key, AeadCiphers::Aes128Gcm)?;
                AeadDecryptor::decrypt(
                    &Aes128Gcm,
                    ciphertext,
                    tag,
                    &Aes128GcmKey(key_array),
                    nonce,
                    aad,
                )
            }
            AesGcm::Aes256 => {
                let key_array = validate_key_length::<AES_256_KEY_SIZE>(key, AeadCiphers::Aes256Gcm)?;
                AeadDecryptor::decrypt(
                    &Aes256Gcm,
                    ciphertext,
                    tag,
                    &Aes256GcmKey(key_array),
                    nonce,
                    aad,
                )
            }
        }
    }
}

impl AeadCipher for AesGcm {
    type Key = [u8];
    type Error = AeadError;
}

impl AesGcmCipher for AesGcm {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aead::helpers::test_helpers::{
        assert_aead_roundtrip, assert_aead_roundtrip_with_aad, assert_combined_roundtrip,
    };

    #[test]
    fn test_aes128gcm_roundtrip() {
        let key = Aes128GcmKey::from([0u8; 16]);
        let nonce = Nonce::from(vec![0u8; 12]);

        assert_aead_roundtrip(
            &nonce,
            |pt, n, aad| Aes128Gcm.encrypt(pt, &key, n, aad).unwrap(),
            |ct, tag, n, aad| Aes128Gcm.decrypt(ct, tag, &key, n, aad).unwrap(),
        );
    }

    #[test]
    fn test_aes256gcm_roundtrip() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);

        assert_aead_roundtrip(
            &nonce,
            |pt, n, aad| Aes256Gcm.encrypt(pt, &key, n, aad).unwrap(),
            |ct, tag, n, aad| Aes256Gcm.decrypt(ct, tag, &key, n, aad).unwrap(),
        );
    }

    #[test]
    fn test_aes128gcm_with_aad() {
        let key = Aes128GcmKey::from([0u8; 16]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::from(b"authenticated context".as_slice());

        assert_aead_roundtrip_with_aad(
            &nonce,
            &aad,
            |pt, n, a| Aes128Gcm.encrypt(pt, &key, n, a).unwrap(),
            |ct, tag, n, a| Aes128Gcm.decrypt(ct, tag, &key, n, a).unwrap(),
        );
    }

    #[test]
    fn test_aes256gcm_with_aad() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::from(b"authenticated context".as_slice());

        assert_aead_roundtrip_with_aad(
            &nonce,
            &aad,
            |pt, n, a| Aes256Gcm.encrypt(pt, &key, n, a).unwrap(),
            |ct, tag, n, a| Aes256Gcm.decrypt(ct, tag, &key, n, a).unwrap(),
        );
    }

    #[test]
    fn test_aes128gcm_combined_api() {
        let key = Aes128GcmKey::from([0u8; 16]);
        let nonce = Nonce::from(vec![0u8; 12]);

        assert_combined_roundtrip(
            &nonce,
            |pt, n, aad| Aes128Gcm.encrypt_combined(pt, &key, n, aad).unwrap(),
            |ct, n, aad| Aes128Gcm.decrypt_combined(ct, &key, n, aad).unwrap(),
        );
    }

    #[test]
    fn test_aes256gcm_combined_api() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);

        assert_combined_roundtrip(
            &nonce,
            |pt, n, aad| Aes256Gcm.encrypt_combined(pt, &key, n, aad).unwrap(),
            |ct, n, aad| Aes256Gcm.decrypt_combined(ct, &key, n, aad).unwrap(),
        );
    }

    #[test]
    fn test_aes128gcm_tampered_ciphertext_fails() {
        let key = Aes128GcmKey::from([0u8; 16]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (mut ciphertext, tag) = Aes128Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        // 暗号文を改竄
        let mut bytes = ciphertext.into_bytes();
        bytes[0] ^= 0xFF;
        ciphertext = Ciphertext::from(bytes);

        let result = Aes128Gcm.decrypt(&ciphertext, &tag, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes256gcm_tampered_tag_fails() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, mut tag) = Aes256Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        // タグを改竄
        let mut bytes = tag.as_bytes().to_vec();
        bytes[0] ^= 0xFF;
        tag = AuthenticationTag::from(bytes);

        let result = Aes256Gcm.decrypt(&ciphertext, &tag, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes128gcm_wrong_aad_fails() {
        let key = Aes128GcmKey::from([0u8; 16]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad1 = AdditionalData::from(b"context1".as_slice());
        let aad2 = AdditionalData::from(b"context2".as_slice());
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, tag) = Aes128Gcm.encrypt(&plaintext, &key, &nonce, &aad1).unwrap();

        let result = Aes128Gcm.decrypt(&ciphertext, &tag, &key, &nonce, &aad2);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes256gcm_wrong_key_fails() {
        let key1 = Aes256GcmKey::from([0u8; 32]);
        let key2 = Aes256GcmKey::from([1u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, tag) = Aes256Gcm.encrypt(&plaintext, &key1, &nonce, &aad).unwrap();

        let result = Aes256Gcm.decrypt(&ciphertext, &tag, &key2, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes128gcm_invalid_nonce_length() {
        let key = Aes128GcmKey::from([0u8; 16]);
        let nonce = Nonce::from(vec![0u8; 8]); // invalid
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let result = Aes128Gcm.encrypt(&plaintext, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::InvalidNonceLength { .. })));
    }

    #[test]
    fn test_aes256gcm_invalid_tag_length() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, _) = Aes256Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let bad_tag = AuthenticationTag::from(vec![0u8; 8]); // invalid
        let result = Aes256Gcm.decrypt(&ciphertext, &bad_tag, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::InvalidTagLength { .. })));
    }

    #[test]
    fn test_aes_gcm_enum_aes128() {
        let cipher = AesGcm::Aes128;
        let key: &[u8] = &[0u8; 16];
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, tag) = cipher.encrypt(&plaintext, key, &nonce, &aad).unwrap();
        let decrypted = cipher.decrypt(&ciphertext, &tag, key, &nonce, &aad).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes_gcm_enum_aes256() {
        let cipher = AesGcm::Aes256;
        let key: &[u8] = &[0u8; 32];
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, tag) = cipher.encrypt(&plaintext, key, &nonce, &aad).unwrap();
        let decrypted = cipher.decrypt(&ciphertext, &tag, key, &nonce, &aad).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes_gcm_enum_invalid_key_length() {
        let cipher = AesGcm::Aes128;
        let key: &[u8] = &[0u8; 8]; // invalid
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let result = cipher.encrypt(&plaintext, key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_empty_plaintext() {
        let key = Aes128GcmKey::from([0u8; 16]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("");

        let (ciphertext, tag) = Aes128Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        assert!(ciphertext.as_bytes().is_empty());

        let decrypted = Aes128Gcm.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        let key1 = Aes128GcmKey::from([0u8; 16]);
        let key2 = Aes128GcmKey::from([1u8; 16]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ct1, _) = Aes128Gcm.encrypt(&plaintext, &key1, &nonce, &aad).unwrap();
        let (ct2, _) = Aes128Gcm.encrypt(&plaintext, &key2, &nonce, &aad).unwrap();

        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_different_nonce_produce_different_ciphertext() {
        let key = Aes128GcmKey::from([0u8; 16]);
        let nonce1 = Nonce::from(vec![0u8; 12]);
        let nonce2 = Nonce::from(vec![1u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ct1, _) = Aes128Gcm.encrypt(&plaintext, &key, &nonce1, &aad).unwrap();
        let (ct2, _) = Aes128Gcm.encrypt(&plaintext, &key, &nonce2, &aad).unwrap();

        assert_ne!(ct1, ct2);
    }
}
