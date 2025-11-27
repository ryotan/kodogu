//! Integration tests for crypto functionality

use crypto::{
    adapters::rust_crypto::{
        RustCryptoDecryptor, RustCryptoEncryptor, RustCryptoKeyDeriver, RustCryptoKeyGenerator,
    },
    AsymmetricAlgorithm, Decryptor, EcdsaCurve, Encryptor, KeyDeriver, KeyGenerator, Plaintext,
    RsaKeySize, SymmetricAlgorithm, SymmetricKey,
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
    }

    #[test]
    fn rsa_4096_key_pair_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Rsa(RsaKeySize::Bits4096))
            .expect("RSA 4096 key generation failed");

        let public_pem = String::from_utf8(key_pair.public_key().as_bytes().to_vec()).unwrap();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
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
    }

    #[test]
    fn ecdsa_p384_key_pair_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key_pair = generator
            .generate_key_pair(AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P384))
            .expect("ECDSA P-384 key generation failed");

        let public_pem = String::from_utf8(key_pair.public_key().as_bytes().to_vec()).unwrap();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
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
    }

    #[test]
    fn symmetric_key_aes128_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key = generator
            .generate_symmetric_key(SymmetricAlgorithm::Aes128)
            .expect("AES-128 key generation failed");

        assert_eq!(key.len(), 16);
    }

    #[test]
    fn symmetric_key_aes256_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let key = generator
            .generate_symmetric_key(SymmetricAlgorithm::Aes256)
            .expect("AES-256 key generation failed");

        assert_eq!(key.len(), 32);
    }

    #[test]
    fn password_string_generation() {
        let generator = RustCryptoKeyGenerator::new();
        let password = generator
            .generate_password_string(32)
            .expect("Password generation failed");

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
    fn aes128_key_construction() {
        let key = SymmetricKey::aes128([0u8; 16]);
        assert_eq!(key.len(), 16);
        assert!(!key.is_empty());
    }

    #[test]
    fn aes256_key_construction() {
        let key = SymmetricKey::aes256([0u8; 32]);
        assert_eq!(key.len(), 32);
        assert!(!key.is_empty());
    }

    #[test]
    fn key_equality() {
        let key1 = SymmetricKey::aes256([1u8; 32]);
        let key2 = SymmetricKey::aes256([1u8; 32]);
        let key3 = SymmetricKey::aes256([2u8; 32]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn pbkdf2_key_derivation() {
        let deriver = RustCryptoKeyDeriver::new();
        let password = "my-secure-password";
        let salt = b"random-salt-value";

        let key = deriver
            .derive_key(password, salt, 32)
            .expect("Key derivation failed");

        assert_eq!(key.len(), 32);
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

        // Different salts should be generated
        assert_ne!(salt1, salt2);
        // Different keys should be derived
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }
}

mod symmetric_encryption {
    use super::*;

    #[test]
    fn aes128_encrypt_decrypt_round_trip() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key = generator
            .generate_symmetric_key(SymmetricAlgorithm::Aes128)
            .unwrap();
        let plaintext = Plaintext::from("Hello, AES-128!");

        let ciphertext = encryptor
            .encrypt_symmetric(&plaintext, &key)
            .expect("Encryption failed");
        let decrypted = decryptor
            .decrypt_symmetric(&ciphertext, &key)
            .expect("Decryption failed");

        assert_eq!(plaintext.as_bytes(), decrypted.as_bytes());
    }

    #[test]
    fn aes256_encrypt_decrypt_round_trip() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();
        let decryptor = RustCryptoDecryptor::new();

        let key = generator
            .generate_symmetric_key(SymmetricAlgorithm::Aes256)
            .unwrap();
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

        let key1 = generator
            .generate_symmetric_key(SymmetricAlgorithm::Aes256)
            .unwrap();
        let key2 = generator
            .generate_symmetric_key(SymmetricAlgorithm::Aes256)
            .unwrap();

        let plaintext = Plaintext::from("Secret message");
        let ciphertext = encryptor.encrypt_symmetric(&plaintext, &key1).unwrap();

        // Decryption with wrong key should fail
        let result = decryptor.decrypt_symmetric(&ciphertext, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn ciphertext_contains_nonce() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();

        let key = generator
            .generate_symmetric_key(SymmetricAlgorithm::Aes256)
            .unwrap();
        let plaintext = Plaintext::from("Test");

        let ciphertext = encryptor.encrypt_symmetric(&plaintext, &key).unwrap();

        // Ciphertext should include 12-byte nonce + encrypted data + auth tag
        assert!(ciphertext.as_bytes().len() > 12);
    }

    #[test]
    fn same_plaintext_different_ciphertext() {
        let generator = RustCryptoKeyGenerator::new();
        let encryptor = RustCryptoEncryptor::new();

        let key = generator
            .generate_symmetric_key(SymmetricAlgorithm::Aes256)
            .unwrap();
        let plaintext = Plaintext::from("Same message");

        let ciphertext1 = encryptor.encrypt_symmetric(&plaintext, &key).unwrap();
        let ciphertext2 = encryptor.encrypt_symmetric(&plaintext, &key).unwrap();

        // Random nonce should make ciphertexts different
        assert_ne!(ciphertext1.as_bytes(), ciphertext2.as_bytes());
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

        let ciphertext = encryptor
            .encrypt_asymmetric(&plaintext, key_pair.public_key())
            .unwrap();
        let decrypted = decryptor
            .decrypt_asymmetric(&ciphertext, key_pair.private_key())
            .unwrap();

        assert_eq!(large_data, decrypted.as_bytes());
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

        let ciphertext = encryptor
            .encrypt_asymmetric(&plaintext, key_pair1.public_key())
            .unwrap();

        // Decryption with wrong private key should fail
        let result = decryptor.decrypt_asymmetric(&ciphertext, key_pair2.private_key());
        assert!(result.is_err());
    }
}
