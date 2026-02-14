//! RFC/NIST 公式テストベクターによる暗号アルゴリズムの検証
//!
//! 外部ツールとの相互運用性を保証するため、公式テストベクターを使用してテストを実行する。
//!
//! # 参照
//! - RFC 8439: ChaCha20 and Poly1305 for IETF Protocols
//! - RFC 4231: HMAC-SHA-224/256/384/512 Test Vectors
//! - RFC 4493: The AES-CMAC Algorithm
//! - NIST SP 800-38A: Recommendation for Block Cipher Modes of Operation
//! - NIST SP 800-38B: The CMAC Mode for Authentication
//! - NIST SP 800-38D: Galois/Counter Mode (GCM) and GMAC

use crypto::{
    AdditionalData, AeadDecryptor, AeadEncryptor, Aes128Cbc, Aes128Ctr, Aes128Gcm, Aes128GcmKey,
    Aes256Cbc, Aes256Ctr, Aes256Gcm, Aes256GcmKey, AuthenticationTag, BlockDecryptor,
    BlockEncryptor, ChaCha20Poly1305, ChaCha20Poly1305Key, Ciphertext, CmacAes, HmacAlgorithm,
    InitializationVector, MacGenerator, Nonce, Plaintext, StreamDecryptor, StreamEncryptor,
};
use hex_literal::hex;

// ============================================================================
// RFC 8439: ChaCha20-Poly1305 AEAD テストベクター
// https://www.rfc-editor.org/rfc/rfc8439.html Appendix A.5
// ============================================================================

mod chacha20_poly1305_rfc8439 {
    use super::*;

    /// RFC 8439 Appendix A.5 - AEAD Decryption Test Vector
    #[test]
    fn test_rfc8439_appendix_a5_decrypt() {
        let key = ChaCha20Poly1305Key::from(hex!(
            "1c9240a5eb55d38af333888604f6b5f0"
            "473917c1402b80099dca5cbc207075c0"
        ));
        let nonce = Nonce::from(hex!("000000000102030405060708").to_vec());
        let aad = AdditionalData::from(hex!("f33388860000000000004e91").as_slice());

        let ciphertext = Ciphertext::from(
            hex!(
                "64a0861575861af460f062c79be643bd"
                "5e805cfd345cf389f108670ac76c8cb2"
                "4c6cfc18755d43eea09ee94e382d26b0"
                "bdb7b73c321b0100d4f03b7f355894cf"
                "332f830e710b97ce98c8a84abd0b9481"
                "14ad176e008d33bd60f982b1ff37c855"
                "9797a06ef4f0ef61c186324e2b350638"
                "3606907b6a7c02b0f9f6157b53c867e4"
                "b9166c767b804d46a59b5216cde7a4e9"
                "9040c5a40433225ee282a1b0a06c523e"
                "af4534d7f83fa1155b0047718cbc546a"
                "0d072b04b3564eea1b422273f548271a"
                "0bb2316053fa76991955ebd63159434e"
                "cebb4e466dae5a1073a6727627097a10"
                "49e617d91d361094fa68f0ff77987130"
                "305beaba2eda04df997b714d6c6f2c29"
                "a6ad5cb4022b02709b"
            )
            .to_vec(),
        );

        let tag = AuthenticationTag::from(hex!("eead9d67890cbb22392336fea1851f38").to_vec());

        let plaintext = ChaCha20Poly1305
            .decrypt(&ciphertext, &tag, &key, &nonce, &aad)
            .expect("decryption should succeed");

        let plaintext_text = String::from_utf8_lossy(plaintext.as_bytes());
        assert!(
            plaintext_text.starts_with("Internet-Drafts are draft documents"),
            "plaintext should start with expected text, got: {:?}",
            &plaintext_text[..50.min(plaintext_text.len())]
        );
    }

    #[test]
    fn test_rfc8439_roundtrip() {
        let key = ChaCha20Poly1305Key::from(hex!(
            "1c9240a5eb55d38af333888604f6b5f0"
            "473917c1402b80099dca5cbc207075c0"
        ));
        let nonce = Nonce::from(hex!("000000000102030405060708").to_vec());
        let aad = AdditionalData::from(hex!("f33388860000000000004e91").as_slice());
        let plaintext = Plaintext::from(b"Hello, RFC 8439!".to_vec());

        let (ciphertext, tag) = ChaCha20Poly1305
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        let decrypted = ChaCha20Poly1305
            .decrypt(&ciphertext, &tag, &key, &nonce, &aad)
            .expect("decryption should succeed");

        assert_eq!(decrypted.as_bytes(), plaintext.as_bytes());
    }
}

// ============================================================================
// NIST GCM Spec: AES-GCM テストベクター
// https://csrc.nist.gov/pubs/sp/800/38/d/final
// ============================================================================

mod aes_gcm_nist {
    use super::*;

    // ------------------------------------------------------------------------
    // AES-128-GCM Test Cases (from GCM spec)
    // ------------------------------------------------------------------------

    /// Test Case 1: Empty plaintext, empty AAD
    #[test]
    fn test_case_1_aes128gcm_empty() {
        let key = Aes128GcmKey::from(hex!("00000000000000000000000000000000"));
        let nonce = Nonce::from(hex!("000000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(vec![]);

        let (ciphertext, tag) = Aes128Gcm
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert!(ciphertext.as_bytes().is_empty());
        assert_eq!(tag.as_bytes(), hex!("58e2fccefa7e3061367f1d57a4e7455a"));
    }

    /// Test Case 2: 16-byte plaintext, empty AAD
    #[test]
    fn test_case_2_aes128gcm_one_block() {
        let key = Aes128GcmKey::from(hex!("00000000000000000000000000000000"));
        let nonce = Nonce::from(hex!("000000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(hex!("00000000000000000000000000000000").to_vec());

        let (ciphertext, tag) = Aes128Gcm
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("0388dace60b6a392f328c2b971b2fe78"));
        assert_eq!(tag.as_bytes(), hex!("ab6e47d42cec13bdf53a67b21257bddf"));
    }

    /// Test Case 3: 64-byte plaintext, empty AAD
    #[test]
    fn test_case_3_aes128gcm_multi_block() {
        let key = Aes128GcmKey::from(hex!("feffe9928665731c6d6a8f9467308308"));
        let nonce = Nonce::from(hex!("cafebabefacedbaddecaf888").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(
            hex!(
                "d9313225f88406e5a55909c5aff5269a"
                "86a7a9531534f7da2e4c303d8a318a72"
                "1c3c0c95956809532fcf0e2449a6b525"
                "b16aedf5aa0de657ba637b391aafd255"
            )
            .to_vec(),
        );

        let (ciphertext, tag) = Aes128Gcm
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(
            ciphertext.as_bytes(),
            hex!(
                "42831ec2217774244b7221b784d0d49c"
                "e3aa212f2c02a4e035c17e2329aca12e"
                "21d514b25466931c7d8f6a5aac84aa05"
                "1ba30b396a0aac973d58e091473f5985"
            )
        );
        assert_eq!(tag.as_bytes(), hex!("4d5c2af327cd64a62cf35abd2ba6fab4"));
    }

    /// Test Case 4: 60-byte plaintext with AAD
    #[test]
    fn test_case_4_aes128gcm_with_aad() {
        let key = Aes128GcmKey::from(hex!("feffe9928665731c6d6a8f9467308308"));
        let nonce = Nonce::from(hex!("cafebabefacedbaddecaf888").to_vec());
        let aad = AdditionalData::from(hex!("feedfacedeadbeeffeedfacedeadbeefabaddad2").as_slice());
        let plaintext = Plaintext::from(
            hex!(
                "d9313225f88406e5a55909c5aff5269a"
                "86a7a9531534f7da2e4c303d8a318a72"
                "1c3c0c95956809532fcf0e2449a6b525"
                "b16aedf5aa0de657ba637b39"
            )
            .to_vec(),
        );

        let (ciphertext, tag) = Aes128Gcm
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(
            ciphertext.as_bytes(),
            hex!(
                "42831ec2217774244b7221b784d0d49c"
                "e3aa212f2c02a4e035c17e2329aca12e"
                "21d514b25466931c7d8f6a5aac84aa05"
                "1ba30b396a0aac973d58e091"
            )
        );
        assert_eq!(tag.as_bytes(), hex!("5bc94fbc3221a5db94fae95ae7121a47"));
    }

    // ------------------------------------------------------------------------
    // AES-256-GCM Test Cases (from GCM spec)
    // ------------------------------------------------------------------------

    /// Test Case 13: Empty plaintext, empty AAD
    #[test]
    fn test_case_13_aes256gcm_empty() {
        let key = Aes256GcmKey::from(hex!(
            "00000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("000000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(vec![]);

        let (ciphertext, tag) = Aes256Gcm
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert!(ciphertext.as_bytes().is_empty());
        assert_eq!(tag.as_bytes(), hex!("530f8afbc74536b9a963b4f1c4cb738b"));
    }

    /// Test Case 14: 16-byte plaintext, empty AAD
    #[test]
    fn test_case_14_aes256gcm_one_block() {
        let key = Aes256GcmKey::from(hex!(
            "00000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("000000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(hex!("00000000000000000000000000000000").to_vec());

        let (ciphertext, tag) = Aes256Gcm
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("cea7403d4d606b6e074ec5d3baf39d18"));
        assert_eq!(tag.as_bytes(), hex!("d0d1c8a799996bf0265b98b5d48ab919"));
    }

    /// Test Case 15: 64-byte plaintext, empty AAD
    #[test]
    fn test_case_15_aes256gcm_multi_block() {
        let key = Aes256GcmKey::from(hex!(
            "feffe9928665731c6d6a8f9467308308"
            "feffe9928665731c6d6a8f9467308308"
        ));
        let nonce = Nonce::from(hex!("cafebabefacedbaddecaf888").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(
            hex!(
                "d9313225f88406e5a55909c5aff5269a"
                "86a7a9531534f7da2e4c303d8a318a72"
                "1c3c0c95956809532fcf0e2449a6b525"
                "b16aedf5aa0de657ba637b391aafd255"
            )
            .to_vec(),
        );

        let (ciphertext, tag) = Aes256Gcm
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(
            ciphertext.as_bytes(),
            hex!(
                "522dc1f099567d07f47f37a32a84427d"
                "643a8cdcbfe5c0c97598a2bd2555d1aa"
                "8cb08e48590dbb3da7b08b1056828838"
                "c5f61e6393ba7a0abcc9f662898015ad"
            )
        );
        assert_eq!(tag.as_bytes(), hex!("b094dac5d93471bdec1a502270e3cc6c"));
    }

    /// Test Case 16: 60-byte plaintext with AAD
    #[test]
    fn test_case_16_aes256gcm_with_aad() {
        let key = Aes256GcmKey::from(hex!(
            "feffe9928665731c6d6a8f9467308308"
            "feffe9928665731c6d6a8f9467308308"
        ));
        let nonce = Nonce::from(hex!("cafebabefacedbaddecaf888").to_vec());
        let aad = AdditionalData::from(hex!("feedfacedeadbeeffeedfacedeadbeefabaddad2").as_slice());
        let plaintext = Plaintext::from(
            hex!(
                "d9313225f88406e5a55909c5aff5269a"
                "86a7a9531534f7da2e4c303d8a318a72"
                "1c3c0c95956809532fcf0e2449a6b525"
                "b16aedf5aa0de657ba637b39"
            )
            .to_vec(),
        );

        let (ciphertext, tag) = Aes256Gcm
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(
            ciphertext.as_bytes(),
            hex!(
                "522dc1f099567d07f47f37a32a84427d"
                "643a8cdcbfe5c0c97598a2bd2555d1aa"
                "8cb08e48590dbb3da7b08b1056828838"
                "c5f61e6393ba7a0abcc9f662"
            )
        );
        assert_eq!(tag.as_bytes(), hex!("76fc6ece0f4e1768cddf8853bb2d551b"));
    }
}

// ============================================================================
// RFC 4231: HMAC-SHA-256/384/512 テストベクター
// https://www.rfc-editor.org/rfc/rfc4231.html
// ============================================================================

mod hmac_rfc4231 {
    use super::*;

    #[test]
    fn test_case_1_hmac_sha256() {
        let key = hex!("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
        let data = b"Hi There";
        let expected = hex!("b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");
        let tag = HmacAlgorithm::Sha256.generate(data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_case_1_hmac_sha384() {
        let key = hex!("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
        let data = b"Hi There";
        let expected = hex!(
            "afd03944d84895626b0825f4ab46907f"
            "15f9dadbe4101ec682aa034c7cebc59c"
            "faea9ea9076ede7f4af152e8b2fa9cb6"
        );
        let tag = HmacAlgorithm::Sha384.generate(data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_case_1_hmac_sha512() {
        let key = hex!("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
        let data = b"Hi There";
        let expected = hex!(
            "87aa7cdea5ef619d4ff0b4241a1d6cb0"
            "2379f4e2ce4ec2787ad0b30545e17cde"
            "daa833b7d6b8a702038b274eaea3f4e4"
            "be9d914eeb61f1702e696c203a126854"
        );
        let tag = HmacAlgorithm::Sha512.generate(data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_case_2_hmac_sha256() {
        let key = b"Jefe";
        let data = b"what do ya want for nothing?";
        let expected = hex!("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843");
        let tag = HmacAlgorithm::Sha256.generate(data, key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_case_2_hmac_sha384() {
        let key = b"Jefe";
        let data = b"what do ya want for nothing?";
        let expected = hex!(
            "af45d2e376484031617f78d2b58a6b1b"
            "9c7ef464f5a01b47e42ec3736322445e"
            "8e2240ca5e69e2c78b3239ecfab21649"
        );
        let tag = HmacAlgorithm::Sha384.generate(data, key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_case_2_hmac_sha512() {
        let key = b"Jefe";
        let data = b"what do ya want for nothing?";
        let expected = hex!(
            "164b7a7bfcf819e2e395fbe73b56e0a3"
            "87bd64222e831fd610270cd7ea250554"
            "9758bf75c05a994a6d034f65f8f0e6fd"
            "caeab1a34d4a6b4b636e070a38bce737"
        );
        let tag = HmacAlgorithm::Sha512.generate(data, key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_case_3_hmac_sha256() {
        let key = [0xaa_u8; 20];
        let data = [0xdd_u8; 50];
        let expected = hex!("773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe");
        let tag = HmacAlgorithm::Sha256.generate(&data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_case_4_hmac_sha256() {
        let key = hex!("0102030405060708090a0b0c0d0e0f10111213141516171819");
        let data = [0xcd_u8; 50];
        let expected = hex!("82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b");
        let tag = HmacAlgorithm::Sha256.generate(&data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_case_6_hmac_sha256() {
        let key = [0xaa_u8; 131];
        let data = b"Test Using Larger Than Block-Size Key - Hash Key First";
        let expected = hex!("60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54");
        let tag = HmacAlgorithm::Sha256.generate(data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_case_7_hmac_sha256() {
        let key = [0xaa_u8; 131];
        let data = b"This is a test using a larger than block-size key and a larger than block-size data. The key needs to be hashed before being used by the HMAC algorithm.";
        let expected = hex!("9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2");
        let tag = HmacAlgorithm::Sha256.generate(data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }
}

// ============================================================================
// RFC 4493 / NIST SP 800-38B: CMAC-AES テストベクター
// https://www.rfc-editor.org/rfc/rfc4493.html
// ============================================================================

mod cmac_rfc4493 {
    use super::*;

    // ------------------------------------------------------------------------
    // AES-128-CMAC (RFC 4493)
    // ------------------------------------------------------------------------

    #[test]
    fn test_aes128_cmac_empty_message() {
        let key = hex!("2b7e151628aed2a6abf7158809cf4f3c");
        let data: &[u8] = &[];
        let expected = hex!("bb1d6929e95937287fa37d129b756746");
        let tag = CmacAes::Aes128.generate(data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_aes128_cmac_16_bytes() {
        let key = hex!("2b7e151628aed2a6abf7158809cf4f3c");
        let data = hex!("6bc1bee22e409f96e93d7e117393172a");
        let expected = hex!("070a16b46b4d4144f79bdd9dd04a287c");
        let tag = CmacAes::Aes128.generate(&data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_aes128_cmac_40_bytes() {
        let key = hex!("2b7e151628aed2a6abf7158809cf4f3c");
        let data = hex!(
            "6bc1bee22e409f96e93d7e117393172a"
            "ae2d8a571e03ac9c9eb76fac45af8e51"
            "30c81c46a35ce411"
        );
        let expected = hex!("dfa66747de9ae63030ca32611497c827");
        let tag = CmacAes::Aes128.generate(&data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_aes128_cmac_64_bytes() {
        let key = hex!("2b7e151628aed2a6abf7158809cf4f3c");
        let data = hex!(
            "6bc1bee22e409f96e93d7e117393172a"
            "ae2d8a571e03ac9c9eb76fac45af8e51"
            "30c81c46a35ce411e5fbc1191a0a52ef"
            "f69f2445df4f9b17ad2b417be66c3710"
        );
        let expected = hex!("51f0bebf7e3b9d92fc49741779363cfe");
        let tag = CmacAes::Aes128.generate(&data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    // ------------------------------------------------------------------------
    // AES-256-CMAC (NIST SP 800-38B)
    // ------------------------------------------------------------------------

    #[test]
    fn test_aes256_cmac_empty_message() {
        let key = hex!(
            "603deb1015ca71be2b73aef0857d7781"
            "1f352c073b6108d72d9810a30914dff4"
        );
        let data: &[u8] = &[];
        let expected = hex!("028962f61b7bf89efc6b551f4667d983");
        let tag = CmacAes::Aes256.generate(data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_aes256_cmac_16_bytes() {
        let key = hex!(
            "603deb1015ca71be2b73aef0857d7781"
            "1f352c073b6108d72d9810a30914dff4"
        );
        let data = hex!("6bc1bee22e409f96e93d7e117393172a");
        let expected = hex!("28a7023f452e8f82bd4bf28d8c37c35c");
        let tag = CmacAes::Aes256.generate(&data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_aes256_cmac_40_bytes() {
        let key = hex!(
            "603deb1015ca71be2b73aef0857d7781"
            "1f352c073b6108d72d9810a30914dff4"
        );
        let data = hex!(
            "6bc1bee22e409f96e93d7e117393172a"
            "ae2d8a571e03ac9c9eb76fac45af8e51"
            "30c81c46a35ce411"
        );
        let expected = hex!("aaf3d8f1de5640c232f5b169b9c911e6");
        let tag = CmacAes::Aes256.generate(&data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }

    #[test]
    fn test_aes256_cmac_64_bytes() {
        let key = hex!(
            "603deb1015ca71be2b73aef0857d7781"
            "1f352c073b6108d72d9810a30914dff4"
        );
        let data = hex!(
            "6bc1bee22e409f96e93d7e117393172a"
            "ae2d8a571e03ac9c9eb76fac45af8e51"
            "30c81c46a35ce411e5fbc1191a0a52ef"
            "f69f2445df4f9b17ad2b417be66c3710"
        );
        let expected = hex!("e1992190549f6ed5696a2c056c315410");
        let tag = CmacAes::Aes256.generate(&data, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected);
    }
}

// ============================================================================
// NIST SP 800-38A: AES-CBC テストベクター
// https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
//
// 注意: 実装は PKCS7 パディングを使用するため、生の NIST テストベクターとは
// 出力が異なる。暗号文の先頭ブロックが NIST ベクターと一致することを検証し、
// ラウンドトリップで正確性を確認する。
// ============================================================================

mod aes_cbc_nist {
    use super::*;

    // ------------------------------------------------------------------------
    // AES-128-CBC (F.2.1, F.2.2)
    // PKCS7 パディングにより暗号文は 32 バイトになるが、
    // 先頭 16 バイトは NIST ベクターと一致する
    // ------------------------------------------------------------------------

    #[test]
    fn test_aes128_cbc_encrypt_first_block_matches_nist() {
        let key = hex!("2b7e151628aed2a6abf7158809cf4f3c");
        let iv = InitializationVector::from(hex!("000102030405060708090a0b0c0d0e0f").to_vec());
        let plaintext = Plaintext::from(hex!("6bc1bee22e409f96e93d7e117393172a").to_vec());
        let expected_first_block = hex!("7649abac8119b246cee98e9b12e9197d");

        let ciphertext = Aes128Cbc.encrypt(&plaintext, &key, &iv).unwrap();

        // 先頭ブロックが NIST ベクターと一致
        assert_eq!(
            &ciphertext.as_bytes()[..16],
            expected_first_block,
            "first block should match NIST test vector"
        );
        // PKCS7 パディングで 32 バイト（16 + 16）
        assert_eq!(ciphertext.as_bytes().len(), 32);
    }

    #[test]
    fn test_aes128_cbc_roundtrip() {
        let key = hex!("2b7e151628aed2a6abf7158809cf4f3c");
        let iv = InitializationVector::from(hex!("000102030405060708090a0b0c0d0e0f").to_vec());
        let plaintext = Plaintext::from(hex!("6bc1bee22e409f96e93d7e117393172a").to_vec());

        let ciphertext = Aes128Cbc.encrypt(&plaintext, &key, &iv).unwrap();
        let decrypted = Aes128Cbc.decrypt(&ciphertext, &key, &iv).unwrap();
        assert_eq!(decrypted.as_bytes(), plaintext.as_bytes());
    }

    // ------------------------------------------------------------------------
    // AES-256-CBC (F.2.5, F.2.6)
    // ------------------------------------------------------------------------

    #[test]
    fn test_aes256_cbc_encrypt_first_block_matches_nist() {
        let key = hex!(
            "603deb1015ca71be2b73aef0857d7781"
            "1f352c073b6108d72d9810a30914dff4"
        );
        let iv = InitializationVector::from(hex!("000102030405060708090a0b0c0d0e0f").to_vec());
        let plaintext = Plaintext::from(hex!("6bc1bee22e409f96e93d7e117393172a").to_vec());
        let expected_first_block = hex!("f58c4c04d6e5f1ba779eabfb5f7bfbd6");

        let ciphertext = Aes256Cbc.encrypt(&plaintext, &key, &iv).unwrap();

        // 先頭ブロックが NIST ベクターと一致
        assert_eq!(
            &ciphertext.as_bytes()[..16],
            expected_first_block,
            "first block should match NIST test vector"
        );
        // PKCS7 パディングで 32 バイト
        assert_eq!(ciphertext.as_bytes().len(), 32);
    }

    #[test]
    fn test_aes256_cbc_roundtrip() {
        let key = hex!(
            "603deb1015ca71be2b73aef0857d7781"
            "1f352c073b6108d72d9810a30914dff4"
        );
        let iv = InitializationVector::from(hex!("000102030405060708090a0b0c0d0e0f").to_vec());
        let plaintext = Plaintext::from(hex!("6bc1bee22e409f96e93d7e117393172a").to_vec());

        let ciphertext = Aes256Cbc.encrypt(&plaintext, &key, &iv).unwrap();
        let decrypted = Aes256Cbc.decrypt(&ciphertext, &key, &iv).unwrap();
        assert_eq!(decrypted.as_bytes(), plaintext.as_bytes());
    }
}

// ============================================================================
// NIST SP 800-38A: AES-CTR テストベクター
// https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
// ============================================================================

mod aes_ctr_nist {
    use super::*;

    // ------------------------------------------------------------------------
    // AES-128-CTR (F.5.1, F.5.2)
    // ------------------------------------------------------------------------

    #[test]
    fn test_aes128_ctr_encrypt() {
        let key = hex!("2b7e151628aed2a6abf7158809cf4f3c");
        let nonce = Nonce::from(hex!("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff").to_vec());
        let plaintext = Plaintext::from(hex!("6bc1bee22e409f96e93d7e117393172a").to_vec());
        let expected = hex!("874d6191b620e3261bef6864990db6ce");

        let ciphertext = Aes128Ctr.encrypt(&plaintext, &key, &nonce).unwrap();
        assert_eq!(ciphertext.as_bytes(), expected);
    }

    #[test]
    fn test_aes128_ctr_decrypt() {
        let key = hex!("2b7e151628aed2a6abf7158809cf4f3c");
        let nonce = Nonce::from(hex!("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff").to_vec());
        let ciphertext = Ciphertext::from(hex!("874d6191b620e3261bef6864990db6ce").to_vec());
        let expected = hex!("6bc1bee22e409f96e93d7e117393172a");

        let plaintext = Aes128Ctr.decrypt(&ciphertext, &key, &nonce).unwrap();
        assert_eq!(plaintext.as_bytes(), expected);
    }

    // ------------------------------------------------------------------------
    // AES-256-CTR (F.5.5, F.5.6)
    // ------------------------------------------------------------------------

    #[test]
    fn test_aes256_ctr_encrypt() {
        let key = hex!(
            "603deb1015ca71be2b73aef0857d7781"
            "1f352c073b6108d72d9810a30914dff4"
        );
        let nonce = Nonce::from(hex!("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff").to_vec());
        let plaintext = Plaintext::from(hex!("6bc1bee22e409f96e93d7e117393172a").to_vec());
        let expected = hex!("601ec313775789a5b7a7f504bbf3d228");

        let ciphertext = Aes256Ctr.encrypt(&plaintext, &key, &nonce).unwrap();
        assert_eq!(ciphertext.as_bytes(), expected);
    }

    #[test]
    fn test_aes256_ctr_decrypt() {
        let key = hex!(
            "603deb1015ca71be2b73aef0857d7781"
            "1f352c073b6108d72d9810a30914dff4"
        );
        let nonce = Nonce::from(hex!("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff").to_vec());
        let ciphertext = Ciphertext::from(hex!("601ec313775789a5b7a7f504bbf3d228").to_vec());
        let expected = hex!("6bc1bee22e409f96e93d7e117393172a");

        let plaintext = Aes256Ctr.decrypt(&ciphertext, &key, &nonce).unwrap();
        assert_eq!(plaintext.as_bytes(), expected);
    }
}

// ============================================================================
// RFC 8439 Section 2.3.2: ChaCha20 テストベクター
// https://www.rfc-editor.org/rfc/rfc8439.html#section-2.3.2
//
// 注意: RFC 8439 Section 2.4.2 は counter=1 を使用するが、
// 実装は counter=0 から開始する。Section 2.3.2 の counter=0 ベクターを使用。
// ============================================================================

mod chacha20_rfc8439 {
    use super::*;
    use crypto::ChaCha20;

    /// RFC 8439 Section 2.3.2 - ChaCha20 Block Function Test Vector (counter=0)
    /// ゼロ平文を暗号化してキーストリームを検証
    #[test]
    fn test_rfc8439_section_2_3_2_keystream() {
        let key = hex!(
            "00000000000000000000000000000000"
            "00000000000000000000000000000000"
        );
        let nonce = Nonce::from(hex!("000000000000000000000000").to_vec());
        // ゼロ平文を暗号化するとキーストリームそのものが得られる
        let plaintext = Plaintext::from(vec![0u8; 64]);

        // RFC 8439 Section 2.3.2 のキーストリーム出力
        let expected_keystream = hex!(
            "76b8e0ada0f13d90405d6ae55386bd28"
            "bdd219b8a08ded1aa836efcc8b770dc7"
            "da41597c5157488d7724e03fb8d84a37"
            "6a43b8f41518a11cc387b669b2ee6586"
        );

        let ciphertext = ChaCha20.encrypt(&plaintext, &key, &nonce).unwrap();
        assert_eq!(ciphertext.as_bytes(), expected_keystream);
    }

    /// RFC 8439 Section 2.3.2 - Test Vector #2 (counter=0, different key/nonce)
    #[test]
    fn test_rfc8439_section_2_3_2_vector2() {
        let key = hex!(
            "00000000000000000000000000000000"
            "00000000000000000000000000000001"
        );
        let nonce = Nonce::from(hex!("000000000000000000000000").to_vec());
        let plaintext = Plaintext::from(vec![0u8; 64]);

        let expected_keystream = hex!(
            "4540f05a9f1fb296d7736e7b208e3c96"
            "eb4fe1834688d2604f450952ed432d41"
            "bbe2a0b6ea7566d2a5d1e7e20d42af2c"
            "53d792b1c43fea817e9ad275ae546963"
        );

        let ciphertext = ChaCha20.encrypt(&plaintext, &key, &nonce).unwrap();
        assert_eq!(ciphertext.as_bytes(), expected_keystream);
    }

    /// RFC 8439 Section 2.3.2 - Test Vector #3 (counter=0, different nonce)
    #[test]
    fn test_rfc8439_section_2_3_2_vector3() {
        let key = hex!(
            "00000000000000000000000000000000"
            "00000000000000000000000000000000"
        );
        let nonce = Nonce::from(hex!("000000000000000000000002").to_vec());
        let plaintext = Plaintext::from(vec![0u8; 64]);

        let expected_keystream = hex!(
            "c2c64d378cd536374ae204b9ef933fcd"
            "1a8b2288b3dfa49672ab765b54ee27c7"
            "8a970e0e955c14f3a88e741b97c286f7"
            "5f8fc299e8148362fa198a39531bed6d"
        );

        let ciphertext = ChaCha20.encrypt(&plaintext, &key, &nonce).unwrap();
        assert_eq!(ciphertext.as_bytes(), expected_keystream);
    }

    /// ラウンドトリップテスト
    #[test]
    fn test_chacha20_roundtrip() {
        let key = hex!(
            "000102030405060708090a0b0c0d0e0f"
            "101112131415161718191a1b1c1d1e1f"
        );
        let nonce = Nonce::from(hex!("000000000000004a00000000").to_vec());
        let plaintext = Plaintext::from(
            b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it."
                .to_vec(),
        );

        let ciphertext = ChaCha20.encrypt(&plaintext, &key, &nonce).unwrap();
        let decrypted = ChaCha20.decrypt(&ciphertext, &key, &nonce).unwrap();
        assert_eq!(decrypted.as_bytes(), plaintext.as_bytes());
    }
}

// ============================================================================
// RFC 8439 Section 2.5: Poly1305 テストベクター
// https://www.rfc-editor.org/rfc/rfc8439.html#section-2.5.2
// ============================================================================

mod poly1305_rfc8439 {
    use super::*;
    use crypto::{MacGenerator, Poly1305Key, Poly1305Mac};

    /// RFC 8439 Section 2.5.2 - Poly1305 Test Vector
    #[test]
    fn test_rfc8439_section_2_5_2() {
        let key = Poly1305Key::from(hex!(
            "85d6be7857556d337f4452fe42d506a8"
            "0103808afb0db2fd4abff6af4149f51b"
        ));
        let message = b"Cryptographic Forum Research Group";

        let expected_tag = hex!("a8061dc1305136c6c22b8baf0c0127a9");

        let tag = Poly1305Mac.generate(message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #1
    #[test]
    fn test_rfc8439_appendix_a3_vector1() {
        let key = Poly1305Key::from([0u8; 32]);
        let message = [0u8; 64];

        let expected_tag = hex!("00000000000000000000000000000000");

        let tag = Poly1305Mac.generate(&message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #2
    /// "Any submission to the IETF intended..."
    #[test]
    fn test_rfc8439_appendix_a3_vector2() {
        let key = Poly1305Key::from(hex!(
            "0000000000000000000000000000000036e5f6b5c5e06070f0efca96227a863e"
        ));
        let message = b"Any submission to the IETF intended by the Contributor for publication as all or part of an IETF Internet-Draft or RFC and any statement made within the context of an IETF activity is considered an \"IETF Contribution\". Such statements include oral statements in IETF sessions, as well as written and electronic communications made at any time or place, which are addressed to";

        let expected_tag = hex!("36e5f6b5c5e06070f0efca96227a863e");

        let tag = Poly1305Mac.generate(message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #3
    #[test]
    fn test_rfc8439_appendix_a3_vector3() {
        let key = Poly1305Key::from(hex!(
            "36e5f6b5c5e06070f0efca96227a863e00000000000000000000000000000000"
        ));
        let message = b"Any submission to the IETF intended by the Contributor for publication as all or part of an IETF Internet-Draft or RFC and any statement made within the context of an IETF activity is considered an \"IETF Contribution\". Such statements include oral statements in IETF sessions, as well as written and electronic communications made at any time or place, which are addressed to";

        let expected_tag = hex!("f3477e7cd95417af89a6b8794c310cf0");

        let tag = Poly1305Mac.generate(message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #4
    #[test]
    fn test_rfc8439_appendix_a3_vector4() {
        let key = Poly1305Key::from(hex!(
            "1c9240a5eb55d38af333888604f6b5f0473917c1402b80099dca5cbc207075c0"
        ));
        let message = b"'Twas brillig, and the slithy toves\nDid gyre and gimble in the wabe:\nAll mimsy were the borogoves,\nAnd the mome raths outgrabe.";

        let expected_tag = hex!("4541669a7eaaee61e708dc7cbcc5eb62");

        let tag = Poly1305Mac.generate(message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #5
    /// Edge case: r が 2^130-5 に近い値
    #[test]
    fn test_rfc8439_appendix_a3_vector5() {
        let key = Poly1305Key::from(hex!(
            "0200000000000000000000000000000000000000000000000000000000000000"
        ));
        let message = hex!("ffffffffffffffffffffffffffffffff");

        let expected_tag = hex!("03000000000000000000000000000000");

        let tag = Poly1305Mac.generate(&message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #6
    #[test]
    fn test_rfc8439_appendix_a3_vector6() {
        let key = Poly1305Key::from(hex!(
            "02000000000000000000000000000000ffffffffffffffffffffffffffffffff"
        ));
        let message = hex!("02000000000000000000000000000000");

        let expected_tag = hex!("03000000000000000000000000000000");

        let tag = Poly1305Mac.generate(&message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #7
    #[test]
    fn test_rfc8439_appendix_a3_vector7() {
        let key = Poly1305Key::from(hex!(
            "0100000000000000000000000000000000000000000000000000000000000000"
        ));
        let message = hex!(
            "ffffffffffffffffffffffffffffffff"
            "f0ffffffffffffffffffffffffffffff"
            "11000000000000000000000000000000"
        );

        let expected_tag = hex!("05000000000000000000000000000000");

        let tag = Poly1305Mac.generate(&message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #8
    #[test]
    fn test_rfc8439_appendix_a3_vector8() {
        let key = Poly1305Key::from(hex!(
            "0100000000000000000000000000000000000000000000000000000000000000"
        ));
        let message = hex!(
            "ffffffffffffffffffffffffffffffff"
            "fbfefefefefefefefefefefefefefefe"
            "01010101010101010101010101010101"
        );

        let expected_tag = hex!("00000000000000000000000000000000");

        let tag = Poly1305Mac.generate(&message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #9
    #[test]
    fn test_rfc8439_appendix_a3_vector9() {
        let key = Poly1305Key::from(hex!(
            "0200000000000000000000000000000000000000000000000000000000000000"
        ));
        let message = hex!("fdffffffffffffffffffffffffffffff");

        let expected_tag = hex!("faffffffffffffffffffffffffffffff");

        let tag = Poly1305Mac.generate(&message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #10
    #[test]
    fn test_rfc8439_appendix_a3_vector10() {
        let key = Poly1305Key::from(hex!(
            "0100000000000000040000000000000000000000000000000000000000000000"
        ));
        let message = hex!(
            "e33594d7505e43b90000000000000000"
            "3394d7505e4379cd0100000000000000"
            "00000000000000000000000000000000"
            "01000000000000000000000000000000"
        );

        let expected_tag = hex!("14000000000000005500000000000000");

        let tag = Poly1305Mac.generate(&message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// RFC 8439 Appendix A.3 - Poly1305 Test Vector #11
    #[test]
    fn test_rfc8439_appendix_a3_vector11() {
        let key = Poly1305Key::from(hex!(
            "0100000000000000040000000000000000000000000000000000000000000000"
        ));
        let message = hex!(
            "e33594d7505e43b90000000000000000"
            "3394d7505e4379cd0100000000000000"
            "00000000000000000000000000000000"
        );

        let expected_tag = hex!("13000000000000000000000000000000");

        let tag = Poly1305Mac.generate(&message, &key).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }
}

// ============================================================================
// RFC 8452: AES-GCM-SIV テストベクター
// https://www.rfc-editor.org/rfc/rfc8452.html Appendix C
// ============================================================================

mod aes_gcm_siv_rfc8452 {
    use super::*;
    use crypto::Aes256GcmSiv;

    /// RFC 8452 Appendix C.2 - AES-256-GCM-SIV Test Case
    /// (Test Case from AEAD_AES_256_GCM_SIV with empty plaintext)
    #[test]
    fn test_rfc8452_aes256_empty_plaintext() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(vec![]);

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert!(ciphertext.as_bytes().is_empty());
        assert_eq!(tag.as_bytes(), hex!("07f5f4169bbf55a8400cd47ea6fd400f"));
    }

    /// RFC 8452 Appendix C.2 - AES-256-GCM-SIV with 8-byte plaintext
    #[test]
    fn test_rfc8452_aes256_8byte_plaintext() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(hex!("0100000000000000").to_vec());

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("c2ef328e5c71c83b"));
        assert_eq!(tag.as_bytes(), hex!("843122130f7364b761e0b97427e3df28"));
    }

    /// RFC 8452 Appendix C.2 - AES-256-GCM-SIV with 12-byte plaintext
    #[test]
    fn test_rfc8452_aes256_12byte_plaintext() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(hex!("010000000000000000000000").to_vec());

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("9aab2aeb3faa0a34aea8e2b1"));
        assert_eq!(tag.as_bytes(), hex!("8ca50da9ae6559e48fd10f6e5c9ca17e"));
    }

    /// RFC 8452 Appendix C.2 - AES-256-GCM-SIV with 16-byte plaintext
    #[test]
    fn test_rfc8452_aes256_16byte_plaintext() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(hex!("01000000000000000000000000000000").to_vec());

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("85a01b63025ba19b7fd3ddfc033b3e76"));
        assert_eq!(tag.as_bytes(), hex!("c9eac6fa700942702e90862383c6c366"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #5 (32-byte plaintext)
    #[test]
    fn test_rfc8452_aes256_32byte_plaintext() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(
            hex!(
                "01000000000000000000000000000000"
                "02000000000000000000000000000000"
            )
            .to_vec(),
        );

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(
            ciphertext.as_bytes(),
            hex!(
                "4a6a9db4c8c6549201b9edb53006cba8"
                "21ec9cf850948a7c86c68ac7539d027f"
            )
        );
        assert_eq!(tag.as_bytes(), hex!("e819e63abcd020b006a976397632eb5d"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #6 (48-byte plaintext)
    #[test]
    fn test_rfc8452_aes256_48byte_plaintext() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(
            hex!(
                "01000000000000000000000000000000"
                "02000000000000000000000000000000"
                "03000000000000000000000000000000"
            )
            .to_vec(),
        );

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(
            ciphertext.as_bytes(),
            hex!(
                "c00d121893a9fa603f48ccc1ca3c57ce"
                "7499245ea0046db16c53c7c66fe717e3"
                "9cf6c748837b61f6ee3adcee17534ed5"
            )
        );
        assert_eq!(tag.as_bytes(), hex!("790bc96880a99ba804bd12c0e6a22cc4"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #7 (64-byte plaintext)
    #[test]
    fn test_rfc8452_aes256_64byte_plaintext() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(
            hex!(
                "01000000000000000000000000000000"
                "02000000000000000000000000000000"
                "03000000000000000000000000000000"
                "04000000000000000000000000000000"
            )
            .to_vec(),
        );

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(
            ciphertext.as_bytes(),
            hex!(
                "c2d5160a1f8683834910acdafc41fbb1"
                "632d4a353e8b905ec9a5499ac34f96c7"
                "e1049eb080883891a4db8caaa1f99dd0"
                "04d80487540735234e3744512c6f90ce"
            )
        );
        assert_eq!(tag.as_bytes(), hex!("112864c269fc0d9d88c61fa47e39aa08"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #8 (8-byte plaintext with 1-byte AAD)
    #[test]
    fn test_rfc8452_aes256_with_aad_8byte() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::from(hex!("01").as_slice());
        let plaintext = Plaintext::from(hex!("0200000000000000").to_vec());

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("1de22967237a8132"));
        assert_eq!(tag.as_bytes(), hex!("91213f267e3b452f02d01ae33e4ec854"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #9 (12-byte plaintext with 1-byte AAD)
    #[test]
    fn test_rfc8452_aes256_with_aad_12byte() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::from(hex!("01").as_slice());
        let plaintext = Plaintext::from(hex!("020000000000000000000000").to_vec());

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("163d6f9cc1b346cd453a2e4c"));
        assert_eq!(tag.as_bytes(), hex!("c1a4a19ae800941ccdc57cc8413c277f"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #10 (16-byte plaintext with 1-byte AAD)
    #[test]
    fn test_rfc8452_aes256_with_aad_16byte() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::from(hex!("01").as_slice());
        let plaintext = Plaintext::from(hex!("02000000000000000000000000000000").to_vec());

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("c91545823cc24f17dbb0e9e807d5ec17"));
        assert_eq!(tag.as_bytes(), hex!("b292d28ff61189e8e49f3875ef91aff7"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #11 (32-byte plaintext with 1-byte AAD)
    #[test]
    fn test_rfc8452_aes256_with_aad_32byte() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::from(hex!("01").as_slice());
        let plaintext = Plaintext::from(
            hex!(
                "02000000000000000000000000000000"
                "03000000000000000000000000000000"
            )
            .to_vec(),
        );

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(
            ciphertext.as_bytes(),
            hex!(
                "07dad364bfc2b9da89116d7bef6daaaf"
                "6f255510aa654f920ac81b94e8bad365"
            )
        );
        assert_eq!(tag.as_bytes(), hex!("aea1bad12702e1965604374aab96dbbc"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #12 (48-byte plaintext with 1-byte AAD)
    #[test]
    fn test_rfc8452_aes256_with_aad_48byte() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::from(hex!("01").as_slice());
        let plaintext = Plaintext::from(
            hex!(
                "02000000000000000000000000000000"
                "03000000000000000000000000000000"
                "04000000000000000000000000000000"
            )
            .to_vec(),
        );

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(
            ciphertext.as_bytes(),
            hex!(
                "c67a1f0f567a5198aa1fcc8e3f213143"
                "36f7f51ca8b1af61feac35a86416fa47"
                "fbca3b5f749cdf564527f2314f42fe25"
            )
        );
        assert_eq!(tag.as_bytes(), hex!("03332742b228c647173616cfd44c54eb"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #13 (64-byte plaintext with 1-byte AAD)
    #[test]
    fn test_rfc8452_aes256_with_aad_64byte() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::from(hex!("01").as_slice());
        let plaintext = Plaintext::from(
            hex!(
                "02000000000000000000000000000000"
                "03000000000000000000000000000000"
                "04000000000000000000000000000000"
                "05000000000000000000000000000000"
            )
            .to_vec(),
        );

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(
            ciphertext.as_bytes(),
            hex!(
                "67fd45e126bfb9a79930c43aad2d3696"
                "7d3f0e4d217c1e551f59727870beefc9"
                "8cb933a8fce9de887b1e40799988db1f"
                "c3f91880ed405b2dd298318858467c89"
            )
        );
        assert_eq!(tag.as_bytes(), hex!("5bde0285037c5de81e5b570a049b62a0"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #14 (12-byte AAD, 4-byte plaintext)
    #[test]
    fn test_rfc8452_aes256_long_aad_short_plaintext() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::from(hex!("010000000000000000000000").as_slice());
        let plaintext = Plaintext::from(hex!("02000000").to_vec());

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("22b3f4cd"));
        assert_eq!(tag.as_bytes(), hex!("1835e517741dfddccfa07fa4661b74cf"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #15 (18-byte AAD, 20-byte plaintext)
    #[test]
    fn test_rfc8452_aes256_mixed_aad_plaintext_1() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        // 18 bytes: 16 + 2
        let aad = AdditionalData::from(hex!("010000000000000000000000000000000200").as_slice());
        // 20 bytes: 16 + 4
        let plaintext = Plaintext::from(hex!("0300000000000000000000000000000004000000").to_vec());

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("43dd0163cdb48f9fe3212bf61b201976067f342b"));
        assert_eq!(tag.as_bytes(), hex!("b879ad976d8242acc188ab59cabfe307"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #16 (20-byte AAD, 18-byte plaintext)
    #[test]
    fn test_rfc8452_aes256_mixed_aad_plaintext_2() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        // 20 bytes: 16 + 4
        let aad = AdditionalData::from(hex!("0100000000000000000000000000000002000000").as_slice());
        // 18 bytes: 16 + 2
        let plaintext = Plaintext::from(hex!("030000000000000000000000000000000400").to_vec());

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), hex!("462401724b5ce6588d5a54aae5375513a075"));
        assert_eq!(tag.as_bytes(), hex!("cfcdf5042112aa29685c912fc2056543"));
    }

    /// RFC 8452 Appendix C.2 - Test Vector #17 (different key)
    #[test]
    fn test_rfc8452_aes256_different_key() {
        let key = Aes256GcmKey::from(hex!(
            "e66021d5eb8e4f4066d4adb9c33560e4"
            "f46e44bb3da0015c94f7088736864200"
        ));
        let nonce = Nonce::from(hex!("e0eaf5284d884a0e77d31646").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(vec![]);

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert!(ciphertext.as_bytes().is_empty());
        assert_eq!(tag.as_bytes(), hex!("169fbb2fbf389a995f6390af22228a62"));
    }

    /// Roundtrip test
    #[test]
    fn test_rfc8452_roundtrip() {
        let key = Aes256GcmKey::from(hex!(
            "01000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("030000000000000000000000").to_vec());
        let aad = AdditionalData::from(hex!("01").as_slice());
        let plaintext = Plaintext::from(hex!("0200000000000000").to_vec());

        let (ciphertext, tag) = Aes256GcmSiv
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        let decrypted = Aes256GcmSiv
            .decrypt(&ciphertext, &tag, &key, &nonce, &aad)
            .expect("decryption should succeed");

        assert_eq!(decrypted.as_bytes(), plaintext.as_bytes());
    }
}

// ============================================================================
// XChaCha20-Poly1305 テストベクター
// draft-irtf-cfrg-xchacha Appendix A.3.1
// https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha
// ============================================================================

mod xchacha20_poly1305_vectors {
    use super::*;
    use crypto::XChaCha20Poly1305;

    /// draft-irtf-cfrg-xchacha Appendix A.3.1 - AEAD_XCHACHA20_POLY1305 Test Vector
    #[test]
    fn test_xchacha20poly1305_ietf_vector() {
        let key = ChaCha20Poly1305Key::from(hex!(
            "808182838485868788898a8b8c8d8e8f"
            "909192939495969798999a9b9c9d9e9f"
        ));
        // 24-byte nonce
        let nonce = Nonce::from(hex!("404142434445464748494a4b4c4d4e4f5051525354555657").to_vec());
        let aad = AdditionalData::from(hex!("50515253c0c1c2c3c4c5c6c7").as_slice());
        let plaintext = Plaintext::from(
            b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it."
                .to_vec(),
        );

        let expected_ciphertext = hex!(
            "bd6d179d3e83d43b9576579493c0e939"
            "572a1700252bfaccbed2902c21396cbb"
            "731c7f1b0b4aa6440bf3a82f4eda7e39"
            "ae64c6708c54c216cb96b72e1213b452"
            "2f8c9ba40db5d945b11b69b982c1bb9e"
            "3f3fac2bc369488f76b2383565d3fff9"
            "21f9664c97637da9768812f615c68b13"
            "b52e"
        );
        let expected_tag = hex!("c0875924c1c7987947deafd8780acf49");

        let (ciphertext, tag) = XChaCha20Poly1305
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert_eq!(ciphertext.as_bytes(), expected_ciphertext);
        assert_eq!(tag.as_bytes(), expected_tag);

        // ラウンドトリップ
        let decrypted = XChaCha20Poly1305
            .decrypt(&ciphertext, &tag, &key, &nonce, &aad)
            .expect("decryption should succeed");
        assert_eq!(decrypted.as_bytes(), plaintext.as_bytes());
    }

    /// ラウンドトリップテスト
    #[test]
    fn test_xchacha20poly1305_roundtrip() {
        let key = ChaCha20Poly1305Key::from(hex!(
            "808182838485868788898a8b8c8d8e8f"
            "909192939495969798999a9b9c9d9e9f"
        ));
        let nonce = Nonce::from(hex!("404142434445464748494a4b4c4d4e4f5051525354555657").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(b"Hello, XChaCha20-Poly1305!".to_vec());

        let (ciphertext, tag) = XChaCha20Poly1305
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        let decrypted = XChaCha20Poly1305
            .decrypt(&ciphertext, &tag, &key, &nonce, &aad)
            .expect("decryption should succeed");
        assert_eq!(decrypted.as_bytes(), plaintext.as_bytes());
    }

    /// Empty plaintext test
    #[test]
    fn test_xchacha20poly1305_empty_plaintext() {
        let key = ChaCha20Poly1305Key::from(hex!(
            "808182838485868788898a8b8c8d8e8f"
            "909192939495969798999a9b9c9d9e9f"
        ));
        let nonce = Nonce::from(hex!("404142434445464748494a4b4c4d4e4f5051525354555657").to_vec());
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from(vec![]);

        let (ciphertext, tag) = XChaCha20Poly1305
            .encrypt(&plaintext, &key, &nonce, &aad)
            .expect("encryption should succeed");

        assert!(ciphertext.as_bytes().is_empty());
        assert_eq!(tag.as_bytes().len(), 16);

        let decrypted = XChaCha20Poly1305
            .decrypt(&ciphertext, &tag, &key, &nonce, &aad)
            .expect("decryption should succeed");
        assert!(decrypted.as_bytes().is_empty());
    }
}

// ============================================================================
// NIST SP 800-38D: GMAC テストベクター
// GMAC は GCM で空の平文を使用した場合と同等
// ============================================================================

mod gmac_nist {
    use super::*;
    use crypto::Gmac;

    /// GMAC test using NIST GCM Test Case 1 (empty plaintext = GMAC)
    /// Key, Nonce, AAD → Tag
    #[test]
    fn test_gmac_nist_vector1() {
        let key = Aes256GcmKey::from(hex!(
            "00000000000000000000000000000000"
            "00000000000000000000000000000000"
        ));
        let nonce = Nonce::from(hex!("000000000000000000000000").to_vec());
        let message: &[u8] = &[]; // 空のメッセージで GMAC

        let expected_tag = hex!("530f8afbc74536b9a963b4f1c4cb738b");

        let tag = Gmac.generate_with_nonce(message, &key, &nonce).unwrap();
        assert_eq!(tag.as_bytes(), expected_tag);
    }

    /// GMAC with non-empty AAD
    /// GCM Test Case 4 から AAD 部分を検証
    #[test]
    fn test_gmac_with_aad() {
        let key = Aes256GcmKey::from(hex!(
            "feffe9928665731c6d6a8f9467308308"
            "feffe9928665731c6d6a8f9467308308"
        ));
        let nonce = Nonce::from(hex!("cafebabefacedbaddecaf888").to_vec());
        let aad = hex!("feedfacedeadbeeffeedfacedeadbeefabaddad2");

        // GMAC: 空の平文 + AAD = AAD の認証タグ
        // GCM Test Case 16 のタグと同じはずだが、AAD のみの場合は異なる
        // ラウンドトリップで検証
        let tag = Gmac.generate_with_nonce(&aad, &key, &nonce).unwrap();

        // 検証が成功すること
        Gmac.verify_with_nonce(&aad, &tag, &key, &nonce).unwrap();
    }

    /// GMAC roundtrip
    #[test]
    fn test_gmac_roundtrip() {
        let key = Aes256GcmKey::from(hex!(
            "01020304050607080910111213141516"
            "17181920212223242526272829303132"
        ));
        let nonce = Nonce::from(hex!("010203040506070809101112").to_vec());
        let message = b"Hello, GMAC!";

        let tag = Gmac.generate_with_nonce(message, &key, &nonce).unwrap();
        Gmac.verify_with_nonce(message, &tag, &key, &nonce).unwrap();
    }
}
