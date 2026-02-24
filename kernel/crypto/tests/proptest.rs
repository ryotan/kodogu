//! Property-based tests for crypto AEAD ciphers
//!
//! These tests verify AEAD properties with randomly generated inputs,
//! complementing the specific test cases in unit tests.

use crypto::{
    AdditionalData, AeadDecryptor, AeadEncryptor, Aes128Gcm, Aes128GcmKey, Aes256Gcm, Aes256GcmKey,
    Aes256GcmSiv, AuthenticationTag, ChaCha20Poly1305, ChaCha20Poly1305Key, Ciphertext, Nonce,
    Plaintext, XChaCha20Poly1305,
};
use proptest::prelude::*;

// ========================================
// Helper functions for test data generation
// ========================================

/// Generates valid nonce of the specified length
fn nonce_of_len(len: usize) -> impl Strategy<Value = Nonce> {
    prop::collection::vec(any::<u8>(), len).prop_map(Nonce::from)
}

/// Generates arbitrary AAD (including empty)
fn arbitrary_aad() -> impl Strategy<Value = AdditionalData> {
    prop::collection::vec(any::<u8>(), 0..100).prop_map(|v| AdditionalData::from(v.as_slice()))
}

/// Generates arbitrary plaintext
fn arbitrary_plaintext() -> impl Strategy<Value = Plaintext> {
    prop::collection::vec(any::<u8>(), 0..1024).prop_map(Plaintext::from)
}

// ========================================
// AES-128-GCM Property Tests
// ========================================

proptest! {
    /// Property: Encrypt → Decrypt returns the original plaintext
    #[test]
    fn prop_aes128gcm_roundtrip(
        key in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let (ciphertext, tag) = Aes128Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = Aes128Gcm.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }

    /// Property: Authentication tag is always 16 bytes
    #[test]
    fn prop_aes128gcm_tag_length(
        key in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
    ) {
        let aad = AdditionalData::empty();
        let (_, tag) = Aes128Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(tag.as_bytes().len(), 16);
    }

    /// Property: Ciphertext length equals plaintext length
    #[test]
    fn prop_aes128gcm_ciphertext_length(
        key in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, _) = Aes128Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(ciphertext.as_bytes().len(), plaintext.as_bytes().len());
    }

    /// Property: Different keys produce different ciphertext
    #[test]
    fn prop_aes128gcm_different_keys(
        key1 in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        key2 in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in prop::collection::vec(any::<u8>(), 1..100).prop_map(Plaintext::from),
    ) {
        let aad = AdditionalData::empty();
        if key1 != key2 {
            let (ct1, _) = Aes128Gcm.encrypt(&plaintext, &key1, &nonce, &aad).unwrap();
            let (ct2, _) = Aes128Gcm.encrypt(&plaintext, &key2, &nonce, &aad).unwrap();
            prop_assert_ne!(ct1, ct2);
        }
    }

    /// Property: Different nonce produces different ciphertext
    #[test]
    fn prop_aes128gcm_different_nonces(
        key in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        nonce1 in nonce_of_len(12),
        nonce2 in nonce_of_len(12),
        // 16バイト以上にして衝突確率を無視できるレベルに（1バイトだと1/256で衝突）
        plaintext in prop::collection::vec(any::<u8>(), 16..100).prop_map(Plaintext::from),
    ) {
        let aad = AdditionalData::empty();
        if nonce1 != nonce2 {
            let (ct1, _) = Aes128Gcm.encrypt(&plaintext, &key, &nonce1, &aad).unwrap();
            let (ct2, _) = Aes128Gcm.encrypt(&plaintext, &key, &nonce2, &aad).unwrap();
            prop_assert_ne!(ct1, ct2);
        }
    }

    /// Property: Tampered ciphertext fails authentication
    #[test]
    fn prop_aes128gcm_tampered_ciphertext_fails(
        key in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in prop::collection::vec(any::<u8>(), 1..100).prop_map(Plaintext::from),
        tamper_pos in 0usize..100,
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = Aes128Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        if !ciphertext.as_bytes().is_empty() {
            let mut bytes = ciphertext.into_bytes();
            let pos = tamper_pos % bytes.len();
            bytes[pos] ^= 0xFF;
            let tampered = Ciphertext::from(bytes);

            let result = Aes128Gcm.decrypt(&tampered, &tag, &key, &nonce, &aad);
            prop_assert!(result.is_err());
        }
    }

    /// Property: Tampered tag fails authentication
    #[test]
    fn prop_aes128gcm_tampered_tag_fails(
        key in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
        tamper_pos in 0usize..16,
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = Aes128Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let mut tag_bytes = tag.as_bytes().to_vec();
        tag_bytes[tamper_pos] ^= 0xFF;
        let tampered_tag = AuthenticationTag::from(tag_bytes);

        let result = Aes128Gcm.decrypt(&ciphertext, &tampered_tag, &key, &nonce, &aad);
        prop_assert!(result.is_err());
    }

    /// Property: Wrong key fails authentication
    #[test]
    fn prop_aes128gcm_wrong_key_fails(
        key1 in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        key2 in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
    ) {
        if key1 != key2 {
            let aad = AdditionalData::empty();
            let (ciphertext, tag) = Aes128Gcm.encrypt(&plaintext, &key1, &nonce, &aad).unwrap();

            let result = Aes128Gcm.decrypt(&ciphertext, &tag, &key2, &nonce, &aad);
            prop_assert!(result.is_err());
        }
    }

    /// Property: Wrong AAD fails authentication
    #[test]
    fn prop_aes128gcm_wrong_aad_fails(
        key in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
        aad1 in prop::collection::vec(any::<u8>(), 1..50).prop_map(|v| AdditionalData::from(v.as_slice())),
        aad2 in prop::collection::vec(any::<u8>(), 1..50).prop_map(|v| AdditionalData::from(v.as_slice())),
    ) {
        if aad1 != aad2 {
            let (ciphertext, tag) = Aes128Gcm.encrypt(&plaintext, &key, &nonce, &aad1).unwrap();

            let result = Aes128Gcm.decrypt(&ciphertext, &tag, &key, &nonce, &aad2);
            prop_assert!(result.is_err());
        }
    }

    /// Property: Combined API roundtrip
    #[test]
    fn prop_aes128gcm_combined_roundtrip(
        key in prop::collection::vec(any::<u8>(), 16).prop_map(|v| {
            let arr: [u8; 16] = v.try_into().unwrap();
            Aes128GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let combined = Aes128Gcm.encrypt_combined(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = Aes128Gcm.decrypt_combined(&combined, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }
}

// ========================================
// AES-256-GCM Property Tests
// ========================================

proptest! {
    /// Property: Encrypt → Decrypt returns the original plaintext
    #[test]
    fn prop_aes256gcm_roundtrip(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let (ciphertext, tag) = Aes256Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = Aes256Gcm.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }

    /// Property: Authentication tag is always 16 bytes
    #[test]
    fn prop_aes256gcm_tag_length(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
    ) {
        let aad = AdditionalData::empty();
        let (_, tag) = Aes256Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(tag.as_bytes().len(), 16);
    }

    /// Property: Ciphertext length equals plaintext length
    #[test]
    fn prop_aes256gcm_ciphertext_length(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, _) = Aes256Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(ciphertext.as_bytes().len(), plaintext.as_bytes().len());
    }

    /// Property: Tampered ciphertext fails authentication
    #[test]
    fn prop_aes256gcm_tampered_ciphertext_fails(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in prop::collection::vec(any::<u8>(), 1..100).prop_map(Plaintext::from),
        tamper_pos in 0usize..100,
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = Aes256Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        if !ciphertext.as_bytes().is_empty() {
            let mut bytes = ciphertext.into_bytes();
            let pos = tamper_pos % bytes.len();
            bytes[pos] ^= 0xFF;
            let tampered = Ciphertext::from(bytes);

            let result = Aes256Gcm.decrypt(&tampered, &tag, &key, &nonce, &aad);
            prop_assert!(result.is_err());
        }
    }

    /// Property: Tampered tag fails authentication
    #[test]
    fn prop_aes256gcm_tampered_tag_fails(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
        tamper_pos in 0usize..16,
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = Aes256Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let mut tag_bytes = tag.as_bytes().to_vec();
        tag_bytes[tamper_pos] ^= 0xFF;
        let tampered_tag = AuthenticationTag::from(tag_bytes);

        let result = Aes256Gcm.decrypt(&ciphertext, &tampered_tag, &key, &nonce, &aad);
        prop_assert!(result.is_err());
    }

    /// Property: Combined API roundtrip
    #[test]
    fn prop_aes256gcm_combined_roundtrip(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let combined = Aes256Gcm.encrypt_combined(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = Aes256Gcm.decrypt_combined(&combined, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }
}

// ========================================
// ChaCha20-Poly1305 Property Tests
// ========================================

proptest! {
    /// Property: Encrypt → Decrypt returns the original plaintext
    #[test]
    fn prop_chacha20poly1305_roundtrip(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(12),
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let (ciphertext, tag) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = ChaCha20Poly1305.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }

    /// Property: Authentication tag is always 16 bytes
    #[test]
    fn prop_chacha20poly1305_tag_length(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
    ) {
        let aad = AdditionalData::empty();
        let (_, tag) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(tag.as_bytes().len(), 16);
    }

    /// Property: Ciphertext length equals plaintext length
    #[test]
    fn prop_chacha20poly1305_ciphertext_length(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, _) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(ciphertext.as_bytes().len(), plaintext.as_bytes().len());
    }

    /// Property: Tampered ciphertext fails authentication
    #[test]
    fn prop_chacha20poly1305_tampered_ciphertext_fails(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in prop::collection::vec(any::<u8>(), 1..100).prop_map(Plaintext::from),
        tamper_pos in 0usize..100,
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        if !ciphertext.as_bytes().is_empty() {
            let mut bytes = ciphertext.into_bytes();
            let pos = tamper_pos % bytes.len();
            bytes[pos] ^= 0xFF;
            let tampered = Ciphertext::from(bytes);

            let result = ChaCha20Poly1305.decrypt(&tampered, &tag, &key, &nonce, &aad);
            prop_assert!(result.is_err());
        }
    }

    /// Property: Tampered tag fails authentication
    #[test]
    fn prop_chacha20poly1305_tampered_tag_fails(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
        tamper_pos in 0usize..16,
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let mut tag_bytes = tag.as_bytes().to_vec();
        tag_bytes[tamper_pos] ^= 0xFF;
        let tampered_tag = AuthenticationTag::from(tag_bytes);

        let result = ChaCha20Poly1305.decrypt(&ciphertext, &tampered_tag, &key, &nonce, &aad);
        prop_assert!(result.is_err());
    }

    /// Property: Combined API roundtrip
    #[test]
    fn prop_chacha20poly1305_combined_roundtrip(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(12),
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let combined = ChaCha20Poly1305.encrypt_combined(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = ChaCha20Poly1305.decrypt_combined(&combined, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }
}

// ========================================
// XChaCha20-Poly1305 Property Tests
// ========================================

proptest! {
    /// Property: Encrypt → Decrypt returns the original plaintext
    #[test]
    fn prop_xchacha20poly1305_roundtrip(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(24),  // XChaCha20 uses 24-byte nonce
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let (ciphertext, tag) = XChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = XChaCha20Poly1305.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }

    /// Property: Authentication tag is always 16 bytes
    #[test]
    fn prop_xchacha20poly1305_tag_length(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(24),
        plaintext in arbitrary_plaintext(),
    ) {
        let aad = AdditionalData::empty();
        let (_, tag) = XChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(tag.as_bytes().len(), 16);
    }

    /// Property: Ciphertext length equals plaintext length
    #[test]
    fn prop_xchacha20poly1305_ciphertext_length(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(24),
        plaintext in arbitrary_plaintext(),
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, _) = XChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(ciphertext.as_bytes().len(), plaintext.as_bytes().len());
    }

    /// Property: Tampered ciphertext fails authentication
    #[test]
    fn prop_xchacha20poly1305_tampered_ciphertext_fails(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(24),
        plaintext in prop::collection::vec(any::<u8>(), 1..100).prop_map(Plaintext::from),
        tamper_pos in 0usize..100,
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = XChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        if !ciphertext.as_bytes().is_empty() {
            let mut bytes = ciphertext.into_bytes();
            let pos = tamper_pos % bytes.len();
            bytes[pos] ^= 0xFF;
            let tampered = Ciphertext::from(bytes);

            let result = XChaCha20Poly1305.decrypt(&tampered, &tag, &key, &nonce, &aad);
            prop_assert!(result.is_err());
        }
    }

    /// Property: Combined API roundtrip
    #[test]
    fn prop_xchacha20poly1305_combined_roundtrip(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(24),
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let combined = XChaCha20Poly1305.encrypt_combined(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = XChaCha20Poly1305.decrypt_combined(&combined, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }
}

// ========================================
// AES-256-GCM-SIV Property Tests
// ========================================

proptest! {
    /// Property: Encrypt → Decrypt returns the original plaintext
    #[test]
    fn prop_aes256gcmsiv_roundtrip(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let (ciphertext, tag) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = Aes256GcmSiv.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }

    /// Property: Authentication tag is always 16 bytes
    #[test]
    fn prop_aes256gcmsiv_tag_length(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
    ) {
        let aad = AdditionalData::empty();
        let (_, tag) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(tag.as_bytes().len(), 16);
    }

    /// Property: Ciphertext length equals plaintext length
    #[test]
    fn prop_aes256gcmsiv_ciphertext_length(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, _) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(ciphertext.as_bytes().len(), plaintext.as_bytes().len());
    }

    /// Property: Tampered ciphertext fails authentication
    #[test]
    fn prop_aes256gcmsiv_tampered_ciphertext_fails(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in prop::collection::vec(any::<u8>(), 1..100).prop_map(Plaintext::from),
        tamper_pos in 0usize..100,
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        if !ciphertext.as_bytes().is_empty() {
            let mut bytes = ciphertext.into_bytes();
            let pos = tamper_pos % bytes.len();
            bytes[pos] ^= 0xFF;
            let tampered = Ciphertext::from(bytes);

            let result = Aes256GcmSiv.decrypt(&tampered, &tag, &key, &nonce, &aad);
            prop_assert!(result.is_err());
        }
    }

    /// Property: Tampered tag fails authentication
    #[test]
    fn prop_aes256gcmsiv_tampered_tag_fails(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in arbitrary_plaintext(),
        tamper_pos in 0usize..16,
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad).unwrap();

        let mut tag_bytes = tag.as_bytes().to_vec();
        tag_bytes[tamper_pos] ^= 0xFF;
        let tampered_tag = AuthenticationTag::from(tag_bytes);

        let result = Aes256GcmSiv.decrypt(&ciphertext, &tampered_tag, &key, &nonce, &aad);
        prop_assert!(result.is_err());
    }

    /// Property: Combined API roundtrip
    #[test]
    fn prop_aes256gcmsiv_combined_roundtrip(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let combined = Aes256GcmSiv.encrypt_combined(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = Aes256GcmSiv.decrypt_combined(&combined, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }
}

// ========================================
// Cross-Cipher Property Tests
// ========================================

proptest! {
    /// Property: The same plaintext encrypted with different ciphers produces different ciphertext
    #[test]
    fn prop_different_ciphers_produce_different_ciphertext(
        key_bytes in prop::collection::vec(any::<u8>(), 32),
        nonce_bytes in prop::collection::vec(any::<u8>(), 12),
        // 暗号文長 = 平文長なので、短い平文だと異なるアルゴリズム間で偶然一致する確率が無視できない（1バイトなら 1/256）。
        // 16バイト以上にすれば衝突確率は 1/2^128 で事実上ゼロ。
        plaintext in prop::collection::vec(any::<u8>(), 16..100).prop_map(Plaintext::from),
    ) {
        let key256: [u8; 32] = key_bytes.clone().try_into().unwrap();
        let key128: [u8; 16] = key_bytes[..16].try_into().unwrap();
        let nonce: [u8; 12] = nonce_bytes.try_into().unwrap();

        let key_aes128 = Aes128GcmKey::from(key128);
        let key_aes256 = Aes256GcmKey::from(key256);
        let key_chacha = ChaCha20Poly1305Key::from(key256);
        let nonce = Nonce::from(nonce.to_vec());
        let aad = AdditionalData::empty();

        let (ct_aes128, _) = Aes128Gcm.encrypt(&plaintext, &key_aes128, &nonce, &aad).unwrap();
        let (ct_aes256, _) = Aes256Gcm.encrypt(&plaintext, &key_aes256, &nonce, &aad).unwrap();
        let (ct_chacha, _) = ChaCha20Poly1305.encrypt(&plaintext, &key_chacha, &nonce, &aad).unwrap();
        let (ct_gcmsiv, _) = Aes256GcmSiv.encrypt(&plaintext, &key_aes256, &nonce, &aad).unwrap();

        // 全て異なる暗号文になる（同じ鍵・nonce でもアルゴリズムが違えば結果が異なる）
        prop_assert_ne!(ct_aes128, ct_aes256.clone());
        prop_assert_ne!(ct_aes256.clone(), ct_chacha);
        prop_assert_ne!(ct_aes256, ct_gcmsiv);
    }

    /// Property: Encryption is deterministic (same inputs → same outputs)
    #[test]
    fn prop_encryption_is_deterministic(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        aad in arbitrary_aad(),
        plaintext in arbitrary_plaintext(),
    ) {
        let (ct1, tag1) = Aes256Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        let (ct2, tag2) = Aes256Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(ct1, ct2);
        prop_assert_eq!(tag1, tag2);
    }
}

// ========================================
// Large Data Property Tests
// ========================================

proptest! {
    /// Property: Ciphers handle large data without panicking
    #[test]
    fn prop_large_data_aes256gcm(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in prop::collection::vec(any::<u8>(), 1024..4096).prop_map(Plaintext::from),
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = Aes256Gcm.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = Aes256Gcm.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn prop_large_data_chacha20poly1305(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            ChaCha20Poly1305Key::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in prop::collection::vec(any::<u8>(), 1024..4096).prop_map(Plaintext::from),
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = ChaCha20Poly1305.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = ChaCha20Poly1305.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn prop_large_data_aes256gcmsiv(
        key in prop::collection::vec(any::<u8>(), 32).prop_map(|v| {
            let arr: [u8; 32] = v.try_into().unwrap();
            Aes256GcmKey::from(arr)
        }),
        nonce in nonce_of_len(12),
        plaintext in prop::collection::vec(any::<u8>(), 1024..4096).prop_map(Plaintext::from),
    ) {
        let aad = AdditionalData::empty();
        let (ciphertext, tag) = Aes256GcmSiv.encrypt(&plaintext, &key, &nonce, &aad).unwrap();
        let decrypted = Aes256GcmSiv.decrypt(&ciphertext, &tag, &key, &nonce, &aad).unwrap();
        prop_assert_eq!(decrypted, plaintext);
    }
}

// ========================================
// Empty Input Tests
// ========================================

#[test]
fn test_empty_plaintext_all_ciphers() {
    let empty = Plaintext::from("");
    let aad = AdditionalData::empty();

    // AES-128-GCM
    let key128 = Aes128GcmKey::from([0u8; 16]);
    let nonce12 = Nonce::from(vec![0u8; 12]);
    let (ct, tag) = Aes128Gcm.encrypt(&empty, &key128, &nonce12, &aad).unwrap();
    assert!(ct.as_bytes().is_empty());
    let decrypted = Aes128Gcm.decrypt(&ct, &tag, &key128, &nonce12, &aad).unwrap();
    assert_eq!(decrypted, empty);

    // AES-256-GCM
    let key256 = Aes256GcmKey::from([0u8; 32]);
    let (ct, tag) = Aes256Gcm.encrypt(&empty, &key256, &nonce12, &aad).unwrap();
    assert!(ct.as_bytes().is_empty());
    let decrypted = Aes256Gcm.decrypt(&ct, &tag, &key256, &nonce12, &aad).unwrap();
    assert_eq!(decrypted, empty);

    // ChaCha20-Poly1305
    let key_chacha = ChaCha20Poly1305Key::from([0u8; 32]);
    let (ct, tag) = ChaCha20Poly1305.encrypt(&empty, &key_chacha, &nonce12, &aad).unwrap();
    assert!(ct.as_bytes().is_empty());
    let decrypted = ChaCha20Poly1305.decrypt(&ct, &tag, &key_chacha, &nonce12, &aad).unwrap();
    assert_eq!(decrypted, empty);

    // XChaCha20-Poly1305
    let nonce24 = Nonce::from(vec![0u8; 24]);
    let (ct, tag) = XChaCha20Poly1305.encrypt(&empty, &key_chacha, &nonce24, &aad).unwrap();
    assert!(ct.as_bytes().is_empty());
    let decrypted = XChaCha20Poly1305.decrypt(&ct, &tag, &key_chacha, &nonce24, &aad).unwrap();
    assert_eq!(decrypted, empty);

    // AES-256-GCM-SIV
    let (ct, tag) = Aes256GcmSiv.encrypt(&empty, &key256, &nonce12, &aad).unwrap();
    assert!(ct.as_bytes().is_empty());
    let decrypted = Aes256GcmSiv.decrypt(&ct, &tag, &key256, &nonce12, &aad).unwrap();
    assert_eq!(decrypted, empty);
}
