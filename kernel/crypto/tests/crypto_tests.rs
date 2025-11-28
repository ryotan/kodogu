//! Integration tests for crypto functionality

use crypto::{
    AsymmetricAlgorithm, Decryptor, EcdsaCurve, Encryptor, KeyDeriver, KeyGenerator, Plaintext,
    RsaKeySize, SymmetricAlgorithm, SymmetricKey,
    adapters::rust_crypto::{
        RustCryptoDecryptor, RustCryptoEncryptor, RustCryptoKeyDeriver, RustCryptoKeyGenerator,
    },
};

mod key_generation {
    use super::*;

    #[test]
    fn rsa_2048_key_pair_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048))
            .expect("RSA 2048 key generation failed");

        // PEM format validation
        let public_pem = String::from_utf8(key_pair.public_key().as_bytes().to_vec()).unwrap();
        let private_pem = String::from_utf8(key_pair.private_key().as_bytes().to_vec()).unwrap();

        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(public_pem.contains("END PUBLIC KEY"));
        assert!(private_pem.contains("BEGIN PRIVATE KEY"));
        assert!(private_pem.contains("END PRIVATE KEY"));

        // Key quality validation
        assert_ne!(key_pair.public_key().as_bytes(), key_pair.private_key().as_bytes());
        assert!(key_pair.public_key().as_bytes().len() > 100); // RSA 2048 public key in PEM is ~400+ bytes
        assert!(key_pair.private_key().as_bytes().len() > 100); // RSA 2048 private key in PEM is ~1600+ bytes

        // Functional validation: generated keys should be usable for encryption/decryption
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();
        let plaintext = Plaintext::from("Test message for RSA 2048");

        let ciphertext = encryptor
            .encrypt_asymmetric(&plaintext, key_pair.public_key())
            .expect("Encryption with generated public key failed");
        let decrypted = decryptor
            .decrypt_asymmetric(&ciphertext, key_pair.private_key())
            .expect("Decryption with generated private key failed");

        assert_eq!(plaintext.as_bytes(), decrypted.as_bytes());
    }

    #[test]
    fn rsa_4096_key_pair_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Rsa(RsaKeySize::Bits4096))
            .expect("RSA 4096 key generation failed");

        let public_pem = String::from_utf8(key_pair.public_key().as_bytes().to_vec()).unwrap();
        let private_pem = String::from_utf8(key_pair.private_key().as_bytes().to_vec()).unwrap();

        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(private_pem.contains("BEGIN PRIVATE KEY"));

        // Key quality validation
        assert_ne!(key_pair.public_key().as_bytes(), key_pair.private_key().as_bytes());
        assert!(key_pair.public_key().as_bytes().len() > 100);
        assert!(key_pair.private_key().as_bytes().len() > 100);
    }

    #[test]
    fn ecdsa_p256_key_pair_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P256))
            .expect("ECDSA P-256 key generation failed");

        let public_pem = String::from_utf8(key_pair.public_key().as_bytes().to_vec()).unwrap();
        let private_pem = String::from_utf8(key_pair.private_key().as_bytes().to_vec()).unwrap();

        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(private_pem.contains("BEGIN PRIVATE KEY"));

        // Key quality validation
        assert_ne!(key_pair.public_key().as_bytes(), key_pair.private_key().as_bytes());
        assert!(key_pair.public_key().as_bytes().len() > 50);
        assert!(key_pair.private_key().as_bytes().len() > 50);
    }

    #[test]
    fn ecdsa_p384_key_pair_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P384))
            .expect("ECDSA P-384 key generation failed");

        let public_pem = String::from_utf8(key_pair.public_key().as_bytes().to_vec()).unwrap();
        let private_pem = String::from_utf8(key_pair.private_key().as_bytes().to_vec()).unwrap();

        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(private_pem.contains("BEGIN PRIVATE KEY"));

        // Key quality validation
        assert_ne!(key_pair.public_key().as_bytes(), key_pair.private_key().as_bytes());
        assert!(key_pair.public_key().as_bytes().len() > 50);
        assert!(key_pair.private_key().as_bytes().len() > 50);
    }

    #[test]
    fn ed25519_key_pair_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Ed25519)
            .expect("Ed25519 key generation failed");

        let public_pem = String::from_utf8(key_pair.public_key().as_bytes().to_vec()).unwrap();
        let private_pem = String::from_utf8(key_pair.private_key().as_bytes().to_vec()).unwrap();

        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(private_pem.contains("BEGIN PRIVATE KEY"));

        // Key quality validation
        assert_ne!(key_pair.public_key().as_bytes(), key_pair.private_key().as_bytes());
        assert!(key_pair.public_key().as_bytes().len() > 50);
        assert!(key_pair.private_key().as_bytes().len() > 50);
    }

    #[test]
    fn symmetric_key_aes128_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key = generator
            .generate_symmetric_key(SymmetricAlgorithm::Aes128)
            .expect("AES-128 key generation failed");

        assert_eq!(key.len(), 16);
        // Key should not be all zeros
        assert_ne!(key.as_bytes(), &[0u8; 16]);
    }

    #[test]
    fn symmetric_key_aes256_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key = generator
            .generate_symmetric_key(SymmetricAlgorithm::Aes256)
            .expect("AES-256 key generation failed");

        assert_eq!(key.len(), 32);
        // Key should not be all zeros
        assert_ne!(key.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn symmetric_key_randomness() {
        let generator = RustCryptoKeyGenerator::new();
        let key1 = generator.generate_symmetric_key(SymmetricAlgorithm::Aes256).unwrap();
        let key2 = generator.generate_symmetric_key(SymmetricAlgorithm::Aes256).unwrap();

        // Two generated keys should be different
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn password_string_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let password = generator.generate_password_string(32).expect("Password generation failed");

        assert_eq!(password.len(), 32);
        // Should contain only alphanumeric characters
        assert!(password.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn password_string_randomness() {
        let generator = RustCryptoKeyGenerator::new();
        let password1 = generator.generate_password_string(32).unwrap();
        let password2 = generator.generate_password_string(32).unwrap();

        // Two generated passwords should be different
        assert_ne!(password1, password2);
    }
}

mod symmetric_key_management {
    use super::*;

    #[test]
    fn pbkdf2_key_derivation() {
        let deriver = RustCryptoKeyDeriver::new();
        let password = "my-secure-password";
        let salt = b"random-salt-value";

        let key = deriver.derive_key(password, salt, 32).expect("Key derivation failed");

        assert_eq!(key.len(), 32);
        // Derived key should not be all zeros
        assert_ne!(key.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn pbkdf2_deterministic() {
        let deriver = RustCryptoKeyDeriver::new();
        let password = "same-password";
        let salt = b"same-salt";

        let key1 = deriver.derive_key(password, salt, 32).unwrap();
        let key2 = deriver.derive_key(password, salt, 32).unwrap();

        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn pbkdf2_different_salt_different_key() {
        let deriver = RustCryptoKeyDeriver::new();
        let password = "same-password";

        let key1 = deriver.derive_key(password, b"salt1", 32).unwrap();
        let key2 = deriver.derive_key(password, b"salt2", 32).unwrap();

        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn pbkdf2_with_random_salt() {
        let deriver = RustCryptoKeyDeriver::new();
        let password = "test-password";

        let (key1, salt1) = deriver.derive_key_with_random_salt(password, 32).unwrap();
        let (key2, salt2) = deriver.derive_key_with_random_salt(password, 32).unwrap();

        // Salt quality validation
        assert!(salt1.len() >= 16); // Salt should be at least 16 bytes
        assert!(salt2.len() >= 16);
        assert_ne!(&salt1[..], &vec![0u8; salt1.len()][..]); // Salt should not be all zeros
        assert_ne!(&salt2[..], &vec![0u8; salt2.len()][..]);

        // Different salts should be generated
        assert_ne!(salt1, salt2);
        // Different keys should be derived
        assert_ne!(key1.as_bytes(), key2.as_bytes());
        // Keys should not be all zeros
        assert_ne!(key1.as_bytes(), &[0u8; 32]);
        assert_ne!(key2.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn pbkdf2_with_empty_password() {
        let deriver = RustCryptoKeyDeriver::new();
        let password = "";
        let salt = b"some-salt";

        let key = deriver.derive_key(password, salt, 32).expect("Should handle empty password");

        assert_eq!(key.len(), 32);
        assert_ne!(key.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn pbkdf2_with_empty_salt() {
        let deriver = RustCryptoKeyDeriver::new();
        let password = "password";
        let salt = b"";

        let key = deriver.derive_key(password, salt, 32).expect("Should handle empty salt");

        assert_eq!(key.len(), 32);
        assert_ne!(key.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn pbkdf2_different_key_sizes() {
        let deriver = RustCryptoKeyDeriver::new();
        let password = "password";
        let salt = b"salt";

        let key16 = deriver.derive_key(password, salt, 16).expect("16-byte key derivation failed");
        let key32 = deriver.derive_key(password, salt, 32).expect("32-byte key derivation failed");

        assert_eq!(key16.len(), 16);
        assert_eq!(key32.len(), 32);
        assert_ne!(key16.as_bytes(), &[0u8; 16]);
        assert_ne!(key32.as_bytes(), &[0u8; 32]);
    }
}

mod symmetric_encryption {
    use super::*;

    #[test]
    fn aes128_encrypt_decrypt_round_trip() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key = generator.generate_symmetric_key(SymmetricAlgorithm::Aes128).unwrap();
        let plaintext = Plaintext::from("Hello, AES-128!");

        let ciphertext = encryptor.encrypt_symmetric(&plaintext, &key).expect("Encryption failed");
        let decrypted = decryptor.decrypt_symmetric(&ciphertext, &key).expect("Decryption failed");

        assert_eq!(plaintext.as_bytes(), decrypted.as_bytes());
    }

    #[test]
    fn aes256_encrypt_decrypt_round_trip() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key = generator.generate_symmetric_key(SymmetricAlgorithm::Aes256).unwrap();
        let plaintext = Plaintext::from("Hello, AES-256!");

        let ciphertext = encryptor.encrypt_symmetric(&plaintext, &key).unwrap();
        let decrypted = decryptor.decrypt_symmetric(&ciphertext, &key).unwrap();

        assert_eq!(plaintext.as_bytes(), decrypted.as_bytes());
    }

    #[test]
    fn wrong_key_fails_decryption() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key1 = generator.generate_symmetric_key(SymmetricAlgorithm::Aes256).unwrap();
        let key2 = generator.generate_symmetric_key(SymmetricAlgorithm::Aes256).unwrap();

        let plaintext = Plaintext::from("Secret message");
        let ciphertext = encryptor.encrypt_symmetric(&plaintext, &key1).unwrap();

        // Decryption with wrong key should fail with Decryption error
        let result = decryptor.decrypt_symmetric(&ciphertext, &key2);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crypto::CryptoError::Decryption { .. }));
    }

    #[test]
    fn ciphertext_contains_nonce() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();

        let key = generator.generate_symmetric_key(SymmetricAlgorithm::Aes256).unwrap();
        let plaintext = Plaintext::from("Test");

        let ciphertext = encryptor.encrypt_symmetric(&plaintext, &key).unwrap();

        // AES-GCM ciphertext structure: nonce(12 bytes) + encrypted_data + auth_tag(16 bytes)
        let expected_size = 12 + plaintext.as_bytes().len() + 16;
        assert_eq!(ciphertext.as_bytes().len(), expected_size);
    }

    #[test]
    fn same_plaintext_different_ciphertext() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();

        let key = generator.generate_symmetric_key(SymmetricAlgorithm::Aes256).unwrap();
        let plaintext = Plaintext::from("Same message");

        let ciphertext1 = encryptor.encrypt_symmetric(&plaintext, &key).unwrap();
        let ciphertext2 = encryptor.encrypt_symmetric(&plaintext, &key).unwrap();

        // Random nonce should make ciphertexts different
        assert_ne!(ciphertext1.as_bytes(), ciphertext2.as_bytes());
    }

    #[test]
    fn empty_plaintext_round_trip() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key = generator.generate_symmetric_key(SymmetricAlgorithm::Aes256).unwrap();
        let plaintext = Plaintext::new(vec![]);

        let ciphertext = encryptor.encrypt_symmetric(&plaintext, &key).unwrap();
        let decrypted = decryptor.decrypt_symmetric(&ciphertext, &key).unwrap();

        assert_eq!(plaintext.as_bytes(), decrypted.as_bytes());
        assert_eq!(decrypted.as_bytes().len(), 0);
    }

    #[test]
    fn corrupted_ciphertext_fails_decryption() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key = generator.generate_symmetric_key(SymmetricAlgorithm::Aes256).unwrap();
        let plaintext = Plaintext::from("Test message");

        let ciphertext = encryptor.encrypt_symmetric(&plaintext, &key).unwrap();

        // Corrupt the ciphertext by flipping a byte
        let mut corrupted_bytes = ciphertext.as_bytes().to_vec();
        if let Some(byte) = corrupted_bytes.last_mut() {
            *byte ^= 0xFF;
        }
        let corrupted_ciphertext = crypto::Ciphertext::new(corrupted_bytes);

        // Decryption should fail with corrupted ciphertext
        let result = decryptor.decrypt_symmetric(&corrupted_ciphertext, &key);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crypto::CryptoError::Decryption { .. }));
    }

    #[test]
    fn short_ciphertext_fails_decryption() {
        let decryptor = RustCryptoDecryptor::new();
        let key = SymmetricKey::aes256([0u8; 32]);

        // Ciphertext too short (less than nonce + tag)
        let short_ciphertext = crypto::Ciphertext::new(vec![0u8; 10]);

        let result = decryptor.decrypt_symmetric(&short_ciphertext, &key);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crypto::CryptoError::Decryption { .. }));
    }
}

mod asymmetric_encryption {
    use super::*;

    #[test]
    fn rsa_hybrid_encrypt_decrypt_round_trip() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048))
            .unwrap();

        let plaintext = Plaintext::from("Secret message for RSA hybrid encryption");

        let ciphertext = encryptor
            .encrypt_asymmetric(&plaintext, key_pair.public_key())
            .expect("RSA hybrid encryption failed");

        let decrypted = decryptor
            .decrypt_asymmetric(&ciphertext, key_pair.private_key())
            .expect("RSA hybrid decryption failed");

        assert_eq!(plaintext.as_bytes(), decrypted.as_bytes());
    }

    #[test]
    fn rsa_hybrid_large_plaintext() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048))
            .unwrap();

        // Large plaintext (larger than RSA can encrypt directly)
        let large_data = vec![0x42u8; 10000];
        let plaintext = Plaintext::new(large_data.clone());

        let ciphertext = encryptor.encrypt_asymmetric(&plaintext, key_pair.public_key()).unwrap();
        let decrypted = decryptor.decrypt_asymmetric(&ciphertext, key_pair.private_key()).unwrap();

        assert_eq!(large_data, decrypted.as_bytes());
    }

    // Note: ECDSA and Ed25519 are for signatures only, not encryption.
    // Hybrid encryption is only supported for RSA.

    #[test]
    fn empty_plaintext_asymmetric_round_trip() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048))
            .unwrap();

        let plaintext = Plaintext::new(vec![]);

        let ciphertext = encryptor.encrypt_asymmetric(&plaintext, key_pair.public_key()).unwrap();
        let decrypted = decryptor.decrypt_asymmetric(&ciphertext, key_pair.private_key()).unwrap();

        assert_eq!(plaintext.as_bytes(), decrypted.as_bytes());
        assert_eq!(decrypted.as_bytes().len(), 0);
    }

    #[test]
    fn corrupted_asymmetric_ciphertext_fails_decryption() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048))
            .unwrap();

        let plaintext = Plaintext::from("Test message");
        let ciphertext = encryptor.encrypt_asymmetric(&plaintext, key_pair.public_key()).unwrap();

        // Corrupt the ciphertext by flipping a byte
        let mut corrupted_bytes = ciphertext.as_bytes().to_vec();
        if let Some(byte) = corrupted_bytes.last_mut() {
            *byte ^= 0xFF;
        }
        let corrupted_ciphertext = crypto::Ciphertext::new(corrupted_bytes);

        // Decryption should fail with corrupted ciphertext
        let result = decryptor.decrypt_asymmetric(&corrupted_ciphertext, key_pair.private_key());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crypto::CryptoError::Decryption { .. }));
    }

    #[test]
    fn wrong_private_key_fails_decryption() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key_pair1 = generator
            .generate_key_pair(AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048))
            .unwrap();
        let key_pair2 = generator
            .generate_key_pair(AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048))
            .unwrap();

        let plaintext = Plaintext::from("Secret");

        let ciphertext = encryptor.encrypt_asymmetric(&plaintext, key_pair1.public_key()).unwrap();

        // Decryption with wrong private key should fail with Decryption error
        let result = decryptor.decrypt_asymmetric(&ciphertext, key_pair2.private_key());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crypto::CryptoError::Decryption { .. }));
    }
}
