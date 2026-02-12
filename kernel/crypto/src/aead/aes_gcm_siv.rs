use super::errors::{AeadCiphers, AeadError, AuthenticationFailedError};
use super::helpers::{validate_nonce_length, validate_tag_length};
use super::traits::{AeadCipher, AeadDecryptor, AeadEncryptor};
use super::types::{AdditionalData, Aes256GcmKey, AuthenticationTag};
use crate::Nonce;
use crate::{Ciphertext, Plaintext};
use aes_gcm_siv::Aes256GcmSiv as Aes256GcmSivImpl;
use aes_gcm_siv::aead::{Aead, KeyInit, Payload};

const AES_GCM_SIV_NONCE_SIZE: usize = 12;
const AES_GCM_SIV_TAG_SIZE: usize = 16;

/// AES-256-GCM-SIV cipher.
///
/// Uses 256-bit key with GCM-SIV mode for nonce misuse-resistant authenticated encryption.
/// GCM-SIV provides better security than GCM when nonce might be reused,
/// at the cost of slightly reduced performance.
///
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{Aes256GcmSiv, AeadEncryptor, AeadDecryptor, Plaintext, Nonce, AdditionalData, Aes256GcmKey};
///
/// let cipher = Aes256GcmSiv;
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
pub struct Aes256GcmSiv;

/// Marker trait for AES-GCM-SIV cipher family.
///
/// Enables generic programming over AES-GCM-SIV variants.
pub trait AesGcmSivCipher: AeadCipher<Error = AeadError> {}

impl AeadEncryptor for Aes256GcmSiv {
    type Key = Aes256GcmKey;
    type Error = AeadError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<(Ciphertext, AuthenticationTag), Self::Error> {
        let cipher = AeadCiphers::Aes256GcmSiv;
        let nonce_array = validate_nonce_length::<AES_GCM_SIV_NONCE_SIZE>(nonce, cipher)?;

        let aes_gcm_siv = Aes256GcmSivImpl::new((&key.0).into());
        let payload = Payload { msg: plaintext.as_bytes(), aad: aad.as_bytes() };

        let combined = aes_gcm_siv
            .encrypt((&nonce_array).into(), payload)
            .map_err(|_| AuthenticationFailedError { cipher }.build())?;

        // aes-gcm-siv は ciphertext || tag を返す
        let (ct, tag) = combined.split_at(combined.len() - AES_GCM_SIV_TAG_SIZE);
        Ok((Ciphertext::from(ct), AuthenticationTag::from(tag)))
    }
}

impl AeadDecryptor for Aes256GcmSiv {
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
        let cipher = AeadCiphers::Aes256GcmSiv;
        let nonce_array = validate_nonce_length::<AES_GCM_SIV_NONCE_SIZE>(nonce, cipher)?;
        let tag_array = validate_tag_length::<AES_GCM_SIV_TAG_SIZE>(tag, cipher)?;

        let aes_gcm_siv = Aes256GcmSivImpl::new((&key.0).into());

        // aes-gcm-siv は ciphertext || tag を期待する
        let mut combined = ciphertext.as_bytes().to_vec();
        combined.extend_from_slice(&tag_array);

        let payload = Payload { msg: &combined, aad: aad.as_bytes() };

        let plaintext = aes_gcm_siv
            .decrypt((&nonce_array).into(), payload)
            .map_err(|_| AuthenticationFailedError { cipher }.build())?;

        Ok(Plaintext::from(plaintext))
    }
}

impl AeadCipher for Aes256GcmSiv {
    type Key = Aes256GcmKey;
    type Error = AeadError;
}

impl AesGcmSivCipher for Aes256GcmSiv {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aead::helpers::test_helpers::{
        assert_aead_roundtrip, assert_aead_roundtrip_with_aad, assert_combined_roundtrip,
    };

    #[test]
    fn test_aes256_gcm_siv_roundtrip() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);

        assert_aead_roundtrip(
            &nonce,
            |pt, n, aad| Aes256GcmSiv.encrypt(pt, &key, n, aad).unwrap(),
            |ct, tag, n, aad| Aes256GcmSiv.decrypt(ct, tag, &key, n, aad).unwrap(),
        );
    }

    #[test]
    fn test_aes256_gcm_siv_with_aad() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::from(b"authenticated context".as_slice());

        assert_aead_roundtrip_with_aad(
            &nonce,
            &aad,
            |pt, n, a| Aes256GcmSiv.encrypt(pt, &key, n, a).unwrap(),
            |ct, tag, n, a| Aes256GcmSiv.decrypt(ct, tag, &key, n, a).unwrap(),
        );
    }

    #[test]
    fn test_aes256_gcm_siv_combined_api() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);

        assert_combined_roundtrip(
            &nonce,
            |pt, n, aad| Aes256GcmSiv.encrypt_combined(pt, &key, n, aad).unwrap(),
            |ct, n, aad| Aes256GcmSiv.decrypt_combined(ct, &key, n, aad).unwrap(),
        );
    }

    #[test]
    fn test_aes256_gcm_siv_tampered_ciphertext_fails() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (mut ciphertext, tag) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let mut bytes = ciphertext.into_bytes();
        bytes[0] ^= 0xFF;
        ciphertext = Ciphertext::from(bytes);

        let result = Aes256GcmSiv.decrypt(&ciphertext, &tag, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes256_gcm_siv_tampered_tag_fails() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, mut tag) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let mut bytes = tag.as_bytes().to_vec();
        bytes[0] ^= 0xFF;
        tag = AuthenticationTag::from(bytes);

        let result = Aes256GcmSiv.decrypt(&ciphertext, &tag, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes256_gcm_siv_wrong_aad_fails() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad1 = AdditionalData::from(b"context1".as_slice());
        let aad2 = AdditionalData::from(b"context2".as_slice());
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, tag) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad1).unwrap();

        let result = Aes256GcmSiv.decrypt(&ciphertext, &tag, &key, &nonce, &aad2);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes256_gcm_siv_wrong_key_fails() {
        let key1 = Aes256GcmKey::from([0u8; 32]);
        let key2 = Aes256GcmKey::from([1u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, tag) = Aes256GcmSiv.encrypt(&plaintext, &key1, &nonce, &aad).unwrap();

        let result = Aes256GcmSiv.decrypt(&ciphertext, &tag, &key2, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes256_gcm_siv_invalid_nonce_length() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 8]); // invalid
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let result = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::InvalidNonceLength { .. })));
    }

    #[test]
    fn test_aes256_gcm_siv_invalid_tag_length() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, _) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let bad_tag = AuthenticationTag::from(vec![0u8; 8]); // invalid
        let result = Aes256GcmSiv.decrypt(&ciphertext, &bad_tag, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::InvalidTagLength { .. })));
    }

    #[test]
    fn test_empty_plaintext() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("");

        let (ciphertext, tag) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        assert!(ciphertext.as_bytes().is_empty());

        let decrypted = Aes256GcmSiv.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        let key1 = Aes256GcmKey::from([0u8; 32]);
        let key2 = Aes256GcmKey::from([1u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ct1, _) = Aes256GcmSiv.encrypt(&plaintext, &key1, &nonce, &aad).unwrap();
        let (ct2, _) = Aes256GcmSiv.encrypt(&plaintext, &key2, &nonce, &aad).unwrap();

        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_different_nonce_produce_different_ciphertext() {
        let key = Aes256GcmKey::from([0u8; 32]);
        let nonce1 = Nonce::from(vec![0u8; 12]);
        let nonce2 = Nonce::from(vec![1u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ct1, _) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce1, &aad).unwrap();
        let (ct2, _) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce2, &aad).unwrap();

        assert_ne!(ct1, ct2);
    }
}
