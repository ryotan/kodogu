use super::errors::{AeadCiphers, AeadError, AuthenticationFailedError};
use super::helpers::{validate_key_length, validate_nonce_length, validate_tag_length};
use super::traits::{AeadCipher, AeadDecryptor, AeadEncryptor};
use super::types::{AdditionalData, AuthenticationTag, ChaCha20Poly1305Key};
use crate::Nonce;
use crate::{Ciphertext, Plaintext};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{
    ChaCha20Poly1305 as ChaCha20Poly1305Impl, XChaCha20Poly1305 as XChaCha20Poly1305Impl,
};

const CHACHA20_POLY1305_NONCE_SIZE: usize = 12;
const XCHACHA20_POLY1305_NONCE_SIZE: usize = 24;
const POLY1305_TAG_SIZE: usize = 16;
const CHACHA20_POLY1305_KEY_SIZE: usize = 32;

/// ChaCha20-Poly1305 cipher.
///
/// Uses 256-bit key with ChaCha20 stream cipher and Poly1305 MAC.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{ChaCha20Poly1305, AeadEncryptor, AeadDecryptor, Plaintext, Nonce, AdditionalData, ChaCha20Poly1305Key};
///
/// let cipher = ChaCha20Poly1305;
/// let key = ChaCha20Poly1305Key::from([0u8; 32]);
/// let nonce = Nonce::from(vec![0u8; 12]);
/// let aad = AdditionalData::empty();
/// let plaintext = Plaintext::from("Hello");
///
/// let (ciphertext, tag) = cipher.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct ChaCha20Poly1305;

/// XChaCha20-Poly1305 cipher.
///
/// Uses 256-bit key with extended nonce (24 bytes) for safer random nonce generation.
/// Zero-sized type (ZST) with no runtime overhead.
///
/// # Example
///
/// ```
/// use crypto::{XChaCha20Poly1305, AeadEncryptor, AeadDecryptor, Plaintext, Nonce, AdditionalData, ChaCha20Poly1305Key};
///
/// let cipher = XChaCha20Poly1305;
/// let key = ChaCha20Poly1305Key::from([0u8; 32]);
/// let nonce = Nonce::from(vec![0u8; 24]); // 24 bytes for XChaCha20
/// let aad = AdditionalData::empty();
/// let plaintext = Plaintext::from("Hello");
///
/// let (ciphertext, tag) = cipher.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct XChaCha20Poly1305;

/// ChaCha20-Poly1305 cipher family with runtime variant selection.
///
/// Use when cipher variant is determined at runtime.
///
/// # Example
///
/// ```
/// use crypto::{ChaCha20Poly1305Family, AeadEncryptor, AeadDecryptor, Plaintext, Nonce, AdditionalData};
///
/// let cipher = ChaCha20Poly1305Family::XChaCha20Poly1305;
/// let key: &[u8] = &[0u8; 32];
/// let nonce = Nonce::from(vec![0u8; 24]);
/// let aad = AdditionalData::empty();
/// let plaintext = Plaintext::from("Hello");
///
/// let (ciphertext, tag) = cipher.encrypt(&plaintext, key, &nonce, &aad).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &tag, key, &nonce, &aad).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum ChaCha20Poly1305Family {
    /// ChaCha20-Poly1305 (12-byte nonce)
    #[default]
    ChaCha20Poly1305,
    /// XChaCha20-Poly1305 (24-byte nonce)
    XChaCha20Poly1305,
}

/// Marker trait for ChaCha20-Poly1305 cipher family.
///
/// Enables generic programming over all ChaCha20-Poly1305 variants.
pub trait ChaCha20Poly1305Cipher: AeadCipher<Error = AeadError> {}

impl AeadEncryptor for ChaCha20Poly1305 {
    type Key = ChaCha20Poly1305Key;
    type Error = AeadError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<(Ciphertext, AuthenticationTag), Self::Error> {
        let cipher = AeadCiphers::ChaCha20Poly1305;
        let nonce_array = validate_nonce_length::<CHACHA20_POLY1305_NONCE_SIZE>(nonce, cipher)?;

        let chacha = ChaCha20Poly1305Impl::new((&key.0).into());
        let payload = Payload { msg: plaintext.as_bytes(), aad: aad.as_bytes() };

        let combined = chacha
            .encrypt((&nonce_array).into(), payload)
            .map_err(|_| AuthenticationFailedError { cipher }.build())?;

        let (ct, tag) = combined.split_at(combined.len() - POLY1305_TAG_SIZE);
        Ok((Ciphertext::from(ct), AuthenticationTag::from(tag)))
    }
}

impl AeadDecryptor for ChaCha20Poly1305 {
    type Key = ChaCha20Poly1305Key;
    type Error = AeadError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        tag: &AuthenticationTag,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<Plaintext, Self::Error> {
        let cipher = AeadCiphers::ChaCha20Poly1305;
        let nonce_array = validate_nonce_length::<CHACHA20_POLY1305_NONCE_SIZE>(nonce, cipher)?;
        let tag_array = validate_tag_length::<POLY1305_TAG_SIZE>(tag, cipher)?;

        let chacha = ChaCha20Poly1305Impl::new((&key.0).into());

        let mut combined = ciphertext.as_bytes().to_vec();
        combined.extend_from_slice(&tag_array);

        let payload = Payload { msg: &combined, aad: aad.as_bytes() };

        let plaintext = chacha
            .decrypt((&nonce_array).into(), payload)
            .map_err(|_| AuthenticationFailedError { cipher }.build())?;

        Ok(Plaintext::from(plaintext))
    }
}

impl AeadCipher for ChaCha20Poly1305 {
    type Key = ChaCha20Poly1305Key;
    type Error = AeadError;
}

impl ChaCha20Poly1305Cipher for ChaCha20Poly1305 {}

impl AeadEncryptor for XChaCha20Poly1305 {
    type Key = ChaCha20Poly1305Key;
    type Error = AeadError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<(Ciphertext, AuthenticationTag), Self::Error> {
        let cipher = AeadCiphers::XChaCha20Poly1305;
        let nonce_array = validate_nonce_length::<XCHACHA20_POLY1305_NONCE_SIZE>(nonce, cipher)?;

        let xchacha = XChaCha20Poly1305Impl::new((&key.0).into());
        let payload = Payload { msg: plaintext.as_bytes(), aad: aad.as_bytes() };

        let combined = xchacha
            .encrypt((&nonce_array).into(), payload)
            .map_err(|_| AuthenticationFailedError { cipher }.build())?;

        let (ct, tag) = combined.split_at(combined.len() - POLY1305_TAG_SIZE);
        Ok((Ciphertext::from(ct), AuthenticationTag::from(tag)))
    }
}

impl AeadDecryptor for XChaCha20Poly1305 {
    type Key = ChaCha20Poly1305Key;
    type Error = AeadError;

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        tag: &AuthenticationTag,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<Plaintext, Self::Error> {
        let cipher = AeadCiphers::XChaCha20Poly1305;
        let nonce_array = validate_nonce_length::<XCHACHA20_POLY1305_NONCE_SIZE>(nonce, cipher)?;
        let tag_array = validate_tag_length::<POLY1305_TAG_SIZE>(tag, cipher)?;

        let xchacha = XChaCha20Poly1305Impl::new((&key.0).into());

        let mut combined = ciphertext.as_bytes().to_vec();
        combined.extend_from_slice(&tag_array);

        let payload = Payload { msg: &combined, aad: aad.as_bytes() };

        let plaintext = xchacha
            .decrypt((&nonce_array).into(), payload)
            .map_err(|_| AuthenticationFailedError { cipher }.build())?;

        Ok(Plaintext::from(plaintext))
    }
}

impl AeadCipher for XChaCha20Poly1305 {
    type Key = ChaCha20Poly1305Key;
    type Error = AeadError;
}

impl ChaCha20Poly1305Cipher for XChaCha20Poly1305 {}

impl AeadEncryptor for ChaCha20Poly1305Family {
    type Key = [u8];
    type Error = AeadError;

    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        nonce: &Nonce,
        aad: &AdditionalData,
    ) -> Result<(Ciphertext, AuthenticationTag), Self::Error> {
        let cipher_id = match self {
            ChaCha20Poly1305Family::ChaCha20Poly1305 => AeadCiphers::ChaCha20Poly1305,
            ChaCha20Poly1305Family::XChaCha20Poly1305 => AeadCiphers::XChaCha20Poly1305,
        };
        let key_array = validate_key_length::<CHACHA20_POLY1305_KEY_SIZE>(key, cipher_id)?;
        let key_typed = ChaCha20Poly1305Key(key_array);

        match self {
            ChaCha20Poly1305Family::ChaCha20Poly1305 => {
                AeadEncryptor::encrypt(&ChaCha20Poly1305, plaintext, &key_typed, nonce, aad)
            }
            ChaCha20Poly1305Family::XChaCha20Poly1305 => {
                AeadEncryptor::encrypt(&XChaCha20Poly1305, plaintext, &key_typed, nonce, aad)
            }
        }
    }
}

impl AeadDecryptor for ChaCha20Poly1305Family {
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
        let cipher_id = match self {
            ChaCha20Poly1305Family::ChaCha20Poly1305 => AeadCiphers::ChaCha20Poly1305,
            ChaCha20Poly1305Family::XChaCha20Poly1305 => AeadCiphers::XChaCha20Poly1305,
        };
        let key_array = validate_key_length::<CHACHA20_POLY1305_KEY_SIZE>(key, cipher_id)?;
        let key_typed = ChaCha20Poly1305Key(key_array);

        match self {
            ChaCha20Poly1305Family::ChaCha20Poly1305 => {
                AeadDecryptor::decrypt(&ChaCha20Poly1305, ciphertext, tag, &key_typed, nonce, aad)
            }
            ChaCha20Poly1305Family::XChaCha20Poly1305 => {
                AeadDecryptor::decrypt(&XChaCha20Poly1305, ciphertext, tag, &key_typed, nonce, aad)
            }
        }
    }
}

impl AeadCipher for ChaCha20Poly1305Family {
    type Key = [u8];
    type Error = AeadError;
}

impl ChaCha20Poly1305Cipher for ChaCha20Poly1305Family {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aead::helpers::test_helpers::{
        assert_aead_roundtrip, assert_aead_roundtrip_with_aad, assert_combined_roundtrip,
    };

    #[test]
    fn test_chacha20poly1305_roundtrip() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);

        assert_aead_roundtrip(
            &nonce,
            |pt, n, aad| ChaCha20Poly1305.encrypt(pt, &key, n, aad).unwrap(),
            |ct, tag, n, aad| ChaCha20Poly1305.decrypt(ct, tag, &key, n, aad).unwrap(),
        );
    }

    #[test]
    fn test_xchacha20poly1305_roundtrip() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 24]);

        assert_aead_roundtrip(
            &nonce,
            |pt, n, aad| XChaCha20Poly1305.encrypt(pt, &key, n, aad).unwrap(),
            |ct, tag, n, aad| XChaCha20Poly1305.decrypt(ct, tag, &key, n, aad).unwrap(),
        );
    }

    #[test]
    fn test_chacha20poly1305_with_aad() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::from(b"authenticated context".as_slice());

        assert_aead_roundtrip_with_aad(
            &nonce,
            &aad,
            |pt, n, a| ChaCha20Poly1305.encrypt(pt, &key, n, a).unwrap(),
            |ct, tag, n, a| ChaCha20Poly1305.decrypt(ct, tag, &key, n, a).unwrap(),
        );
    }

    #[test]
    fn test_xchacha20poly1305_with_aad() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 24]);
        let aad = AdditionalData::from(b"authenticated context".as_slice());

        assert_aead_roundtrip_with_aad(
            &nonce,
            &aad,
            |pt, n, a| XChaCha20Poly1305.encrypt(pt, &key, n, a).unwrap(),
            |ct, tag, n, a| XChaCha20Poly1305.decrypt(ct, tag, &key, n, a).unwrap(),
        );
    }

    #[test]
    fn test_chacha20poly1305_combined_api() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);

        assert_combined_roundtrip(
            &nonce,
            |pt, n, aad| ChaCha20Poly1305.encrypt_combined(pt, &key, n, aad).unwrap(),
            |ct, n, aad| ChaCha20Poly1305.decrypt_combined(ct, &key, n, aad).unwrap(),
        );
    }

    #[test]
    fn test_xchacha20poly1305_combined_api() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 24]);

        assert_combined_roundtrip(
            &nonce,
            |pt, n, aad| XChaCha20Poly1305.encrypt_combined(pt, &key, n, aad).unwrap(),
            |ct, n, aad| XChaCha20Poly1305.decrypt_combined(ct, &key, n, aad).unwrap(),
        );
    }

    #[test]
    fn test_chacha20poly1305_tampered_ciphertext_fails() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (mut ciphertext, tag) =
            ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let mut bytes = ciphertext.into_bytes();
        bytes[0] ^= 0xFF;
        ciphertext = Ciphertext::from(bytes);

        let result = ChaCha20Poly1305.decrypt(&ciphertext, &tag, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_xchacha20poly1305_tampered_tag_fails() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 24]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, mut tag) =
            XChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let mut bytes = tag.as_bytes().to_vec();
        bytes[0] ^= 0xFF;
        tag = AuthenticationTag::from(bytes);

        let result = XChaCha20Poly1305.decrypt(&ciphertext, &tag, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_chacha20poly1305_wrong_aad_fails() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad1 = AdditionalData::from(b"context1".as_slice());
        let aad2 = AdditionalData::from(b"context2".as_slice());
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, tag) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad1).unwrap();

        let result = ChaCha20Poly1305.decrypt(&ciphertext, &tag, &key, &nonce, &aad2);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_xchacha20poly1305_wrong_key_fails() {
        let key1 = ChaCha20Poly1305Key::from([0u8; 32]);
        let key2 = ChaCha20Poly1305Key::from([1u8; 32]);
        let nonce = Nonce::from(vec![0u8; 24]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, tag) = XChaCha20Poly1305.encrypt(&plaintext, &key1, &nonce, &aad).unwrap();

        let result = XChaCha20Poly1305.decrypt(&ciphertext, &tag, &key2, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_chacha20poly1305_invalid_nonce_length() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 8]); // invalid
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let result = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::InvalidNonceLength { .. })));
    }

    #[test]
    fn test_xchacha20poly1305_invalid_nonce_length() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]); // 12 は XChaCha には短い
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let result = XChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::InvalidNonceLength { .. })));
    }

    #[test]
    fn test_chacha20poly1305_invalid_tag_length() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, _) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let bad_tag = AuthenticationTag::from(vec![0u8; 8]); // invalid
        let result = ChaCha20Poly1305.decrypt(&ciphertext, &bad_tag, &key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::InvalidTagLength { .. })));
    }

    #[test]
    fn test_chacha20poly1305_family_chacha20() {
        let cipher = ChaCha20Poly1305Family::ChaCha20Poly1305;
        let key: &[u8] = &[0u8; 32];
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, tag) = cipher.encrypt(&plaintext, key, &nonce, &aad).unwrap();
        let decrypted = cipher.decrypt(&ciphertext, &tag, key, &nonce, &aad).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_chacha20poly1305_family_xchacha20() {
        let cipher = ChaCha20Poly1305Family::XChaCha20Poly1305;
        let key: &[u8] = &[0u8; 32];
        let nonce = Nonce::from(vec![0u8; 24]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ciphertext, tag) = cipher.encrypt(&plaintext, key, &nonce, &aad).unwrap();
        let decrypted = cipher.decrypt(&ciphertext, &tag, key, &nonce, &aad).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_chacha20poly1305_family_invalid_key_length() {
        let cipher = ChaCha20Poly1305Family::ChaCha20Poly1305;
        let key: &[u8] = &[0u8; 16]; // invalid
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let result = cipher.encrypt(&plaintext, key, &nonce, &aad);
        assert!(matches!(result, Err(AeadError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_empty_plaintext() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("");

        let (ciphertext, tag) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        assert!(ciphertext.as_bytes().is_empty());

        let decrypted = ChaCha20Poly1305.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        let key1 = ChaCha20Poly1305Key::from([0u8; 32]);
        let key2 = ChaCha20Poly1305Key::from([1u8; 32]);
        let nonce = Nonce::from(vec![0u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ct1, _) = ChaCha20Poly1305.encrypt(&plaintext, &key1, &nonce, &aad).unwrap();
        let (ct2, _) = ChaCha20Poly1305.encrypt(&plaintext, &key2, &nonce, &aad).unwrap();

        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_different_nonce_produce_different_ciphertext() {
        let key = ChaCha20Poly1305Key::from([0u8; 32]);
        let nonce1 = Nonce::from(vec![0u8; 12]);
        let nonce2 = Nonce::from(vec![1u8; 12]);
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Hello");

        let (ct1, _) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce1, &aad).unwrap();
        let (ct2, _) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce2, &aad).unwrap();

        assert_ne!(ct1, ct2);
    }
}
