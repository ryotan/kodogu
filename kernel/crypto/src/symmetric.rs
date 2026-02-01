//! 対称鍵ブロック暗号 (AES-CBC, AES-CTR, ChaCha20)
//!
//! ## 利用可能なバリアント
//!
//! - **AES-128-CBC**: 128bit 鍵、CBC モード、PKCS#7 パディング
//! - **AES-256-CBC**: 256bit 鍵、CBC モード、PKCS#7 パディング
//!
//! ## 設計パターン
//!
//! `kernel/codec` と同じ 4 つの API パターン:
//!
//! 1. **trait**: `SymmetricEncryptor`, `SymmetricDecryptor`（lib.rs で定義）
//! 2. **ZST型**: `Aes128Cbc`, `Aes256Cbc`（コンパイル時ディスパッチ）
//! 3. **Enum**: `AesCbc`（ランタイム選択）
//! 4. **Cipher-specific trait**: `AesCbcCipher`（ポリモーフィズム）
//!
//! ## 使用例
//!
//! ```
//! use crypto::{Aes128Cbc, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
//!
//! let cipher = Aes128Cbc;
//! let key = [0u8; 16]; // 128bit 鍵
//! let iv = InitializationVector::from(vec![0u8; 16]);
//! let plaintext = Plaintext::from("Hello, World!");
//!
//! let ciphertext = cipher.encrypt(&plaintext, &key, &iv).unwrap();
//! let decrypted = cipher.decrypt(&ciphertext, &key, &iv).unwrap();
//! assert_eq!(plaintext, decrypted);
//! ```

use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use snafu::ensure;

use crate::{
    Ciphertext, InitializationVector, InvalidIvLengthError, InvalidKeyLengthError, PaddingError,
    Plaintext, SymmetricCipherError, SymmetricCiphers, SymmetricDecryptor, SymmetricEncryptor,
};

// AES ブロックサイズ: 16 bytes
const AES_BLOCK_SIZE: usize = 16;

// AES-128 鍵長: 16 bytes (128 bits)
const AES_128_KEY_SIZE: usize = 16;

// AES-256 鍵長: 32 bytes (256 bits)
const AES_256_KEY_SIZE: usize = 32;

//
// ZST 型
//

/// AES-128-CBC 暗号
///
/// 128bit 鍵を使用する AES-CBC 暗号。PKCS#7 パディングを使用。
/// ゼロサイズ型（ZST）でランタイムオーバーヘッドなし。
///
/// # 使用例
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

/// AES-256-CBC 暗号
///
/// 256bit 鍵を使用する AES-CBC 暗号。PKCS#7 パディングを使用。
/// ゼロサイズ型（ZST）でランタイムオーバーヘッドなし。
///
/// # 使用例
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

//
// Enum
//

/// AES-CBC 暗号バリアント
///
/// ランタイムで鍵長を選択する場合に使用。
///
/// # 使用例
///
/// ```
/// use crypto::{AesCbc, AesCbcCipher, Plaintext, InitializationVector};
///
/// let cipher = AesCbc::Aes128;
/// let key = vec![0u8; 16];
/// let iv = InitializationVector::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
///
/// let ciphertext = cipher.encrypt(&plaintext, &key, &iv).unwrap();
/// let decrypted = cipher.decrypt(&ciphertext, &key, &iv).unwrap();
/// assert_eq!(plaintext, decrypted);
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum AesCbc {
    /// AES-128-CBC (128bit 鍵)
    #[default]
    Aes128,
    /// AES-256-CBC (256bit 鍵)
    Aes256,
}

//
// Cipher-specific trait
//

/// AES-CBC 暗号のマーカー trait
///
/// 全 AES-CBC 実装に共通のインターフェース。
/// ジェネリックプログラミングで AES-CBC バリアントを統一的に扱える。
///
/// # 使用例
///
/// ```
/// use crypto::{AesCbcCipher, Aes128Cbc, Aes256Cbc, Plaintext, InitializationVector};
///
/// fn encrypt_with_aes_cbc<C: AesCbcCipher>(
///     cipher: &C,
///     plaintext: &Plaintext,
///     key: &[u8],
///     iv: &InitializationVector,
/// ) -> Result<crypto::Ciphertext, crypto::SymmetricCipherError> {
///     cipher.encrypt(plaintext, key, iv)
/// }
///
/// let key_128 = [0u8; 16];
/// let key_256 = [0u8; 32];
/// let iv = InitializationVector::from(vec![0u8; 16]);
/// let plaintext = Plaintext::from("Hello");
///
/// encrypt_with_aes_cbc(&Aes128Cbc, &plaintext, &key_128, &iv).unwrap();
/// encrypt_with_aes_cbc(&Aes256Cbc, &plaintext, &key_256, &iv).unwrap();
/// ```
pub trait AesCbcCipher {
    /// 暗号化
    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &[u8],
        iv: &InitializationVector,
    ) -> Result<Ciphertext, SymmetricCipherError>;

    /// 復号
    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &[u8],
        iv: &InitializationVector,
    ) -> Result<Plaintext, SymmetricCipherError>;
}

//
// AES-128-CBC 実装
//

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
        ensure!(
            iv.len() == AES_BLOCK_SIZE,
            InvalidIvLengthError {
                cipher: SymmetricCiphers::Aes128Cbc,
                expected: AES_BLOCK_SIZE,
                actual: iv.len(),
            }
        );

        let iv_array: [u8; AES_BLOCK_SIZE] = iv.as_bytes().try_into().unwrap();
        let encryptor = Aes128CbcEnc::new(key.into(), &iv_array.into());

        let plaintext_bytes = plaintext.as_bytes();
        let ciphertext = encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext_bytes);

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
        ensure!(
            iv.len() == AES_BLOCK_SIZE,
            InvalidIvLengthError {
                cipher: SymmetricCiphers::Aes128Cbc,
                expected: AES_BLOCK_SIZE,
                actual: iv.len(),
            }
        );

        let iv_array: [u8; AES_BLOCK_SIZE] = iv.as_bytes().try_into().unwrap();
        let decryptor = Aes128CbcDec::new(key.into(), &iv_array.into());

        let ciphertext_bytes = ciphertext.as_bytes();
        let plaintext = decryptor
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext_bytes)
            .map_err(|_| PaddingError { cipher: SymmetricCiphers::Aes128Cbc }.build())?;

        Ok(Plaintext::from(plaintext))
    }
}

impl AesCbcCipher for Aes128Cbc {
    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &[u8],
        iv: &InitializationVector,
    ) -> Result<Ciphertext, SymmetricCipherError> {
        ensure!(
            key.len() == AES_128_KEY_SIZE,
            InvalidKeyLengthError {
                cipher: SymmetricCiphers::Aes128Cbc,
                expected: AES_128_KEY_SIZE,
                actual: key.len(),
            }
        );

        let key_array: [u8; AES_128_KEY_SIZE] = key.try_into().unwrap();
        SymmetricEncryptor::encrypt(self, plaintext, &key_array, iv)
    }

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &[u8],
        iv: &InitializationVector,
    ) -> Result<Plaintext, SymmetricCipherError> {
        ensure!(
            key.len() == AES_128_KEY_SIZE,
            InvalidKeyLengthError {
                cipher: SymmetricCiphers::Aes128Cbc,
                expected: AES_128_KEY_SIZE,
                actual: key.len(),
            }
        );

        let key_array: [u8; AES_128_KEY_SIZE] = key.try_into().unwrap();
        SymmetricDecryptor::decrypt(self, ciphertext, &key_array, iv)
    }
}

//
// AES-256-CBC 実装
//

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
        ensure!(
            iv.len() == AES_BLOCK_SIZE,
            InvalidIvLengthError {
                cipher: SymmetricCiphers::Aes256Cbc,
                expected: AES_BLOCK_SIZE,
                actual: iv.len(),
            }
        );

        let iv_array: [u8; AES_BLOCK_SIZE] = iv.as_bytes().try_into().unwrap();
        let encryptor = Aes256CbcEnc::new(key.into(), &iv_array.into());

        let plaintext_bytes = plaintext.as_bytes();
        let ciphertext = encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext_bytes);

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
        ensure!(
            iv.len() == AES_BLOCK_SIZE,
            InvalidIvLengthError {
                cipher: SymmetricCiphers::Aes256Cbc,
                expected: AES_BLOCK_SIZE,
                actual: iv.len(),
            }
        );

        let iv_array: [u8; AES_BLOCK_SIZE] = iv.as_bytes().try_into().unwrap();
        let decryptor = Aes256CbcDec::new(key.into(), &iv_array.into());

        let ciphertext_bytes = ciphertext.as_bytes();
        let plaintext = decryptor
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext_bytes)
            .map_err(|_| PaddingError { cipher: SymmetricCiphers::Aes256Cbc }.build())?;

        Ok(Plaintext::from(plaintext))
    }
}

impl AesCbcCipher for Aes256Cbc {
    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &[u8],
        iv: &InitializationVector,
    ) -> Result<Ciphertext, SymmetricCipherError> {
        ensure!(
            key.len() == AES_256_KEY_SIZE,
            InvalidKeyLengthError {
                cipher: SymmetricCiphers::Aes256Cbc,
                expected: AES_256_KEY_SIZE,
                actual: key.len(),
            }
        );

        let key_array: [u8; AES_256_KEY_SIZE] = key.try_into().unwrap();
        SymmetricEncryptor::encrypt(self, plaintext, &key_array, iv)
    }

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &[u8],
        iv: &InitializationVector,
    ) -> Result<Plaintext, SymmetricCipherError> {
        ensure!(
            key.len() == AES_256_KEY_SIZE,
            InvalidKeyLengthError {
                cipher: SymmetricCiphers::Aes256Cbc,
                expected: AES_256_KEY_SIZE,
                actual: key.len(),
            }
        );

        let key_array: [u8; AES_256_KEY_SIZE] = key.try_into().unwrap();
        SymmetricDecryptor::decrypt(self, ciphertext, &key_array, iv)
    }
}

//
// AesCbc enum 実装
//

impl AesCbcCipher for AesCbc {
    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &[u8],
        iv: &InitializationVector,
    ) -> Result<Ciphertext, SymmetricCipherError> {
        match self {
            AesCbc::Aes128 => AesCbcCipher::encrypt(&Aes128Cbc, plaintext, key, iv),
            AesCbc::Aes256 => AesCbcCipher::encrypt(&Aes256Cbc, plaintext, key, iv),
        }
    }

    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &[u8],
        iv: &InitializationVector,
    ) -> Result<Plaintext, SymmetricCipherError> {
        match self {
            AesCbc::Aes128 => AesCbcCipher::decrypt(&Aes128Cbc, ciphertext, key, iv),
            AesCbc::Aes256 => AesCbcCipher::decrypt(&Aes256Cbc, ciphertext, key, iv),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //
    // AES-128-CBC テスト
    //

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
        let iv = InitializationVector::from(vec![0u8; 8]); // 不正な長さ
        let plaintext = Plaintext::from("Hello");

        let result = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes128cbc_padding() {
        let cipher = Aes128Cbc;
        let key = [0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);

        // 様々な長さの平文でテスト（パディングの境界条件）
        for len in 0..=32 {
            let plaintext = Plaintext::from(vec![0xABu8; len]);
            let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv).unwrap();
            let decrypted = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();
            assert_eq!(plaintext, decrypted, "Failed for length {}", len);
        }
    }

    #[test]
    fn test_aes128cbc_cipher_trait() {
        let cipher = Aes128Cbc;
        let key = vec![0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = AesCbcCipher::encrypt(&cipher, &plaintext, &key, &iv).unwrap();
        let decrypted = AesCbcCipher::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aes128cbc_cipher_trait_invalid_key_length() {
        let cipher = Aes128Cbc;
        let key = vec![0u8; 8]; // 不正な長さ
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let result = AesCbcCipher::encrypt(&cipher, &plaintext, &key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    //
    // AES-256-CBC テスト
    //

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
    fn test_aes256cbc_cipher_trait() {
        let cipher = Aes256Cbc;
        let key = vec![0u8; 32];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = AesCbcCipher::encrypt(&cipher, &plaintext, &key, &iv).unwrap();
        let decrypted = AesCbcCipher::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    //
    // AesCbc enum テスト
    //

    #[test]
    fn test_aescbc_enum_aes128() {
        let cipher = AesCbc::Aes128;
        let key = vec![0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = AesCbcCipher::encrypt(&cipher, &plaintext, &key, &iv).unwrap();
        let decrypted = AesCbcCipher::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aescbc_enum_aes256() {
        let cipher = AesCbc::Aes256;
        let key = vec![0u8; 32];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = AesCbcCipher::encrypt(&cipher, &plaintext, &key, &iv).unwrap();
        let decrypted = AesCbcCipher::decrypt(&cipher, &ciphertext, &key, &iv).unwrap();

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

        // 異なる鍵で復号 → パディングエラー
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key2, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::Padding { .. })));
    }

    //
    // 追加カバレッジテスト
    //

    #[test]
    fn test_aes128cbc_decrypt_invalid_iv_length() {
        let cipher = Aes128Cbc;
        let key = [0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = SymmetricEncryptor::encrypt(&cipher, &plaintext, &key, &iv).unwrap();

        // 復号時に無効なIV
        let bad_iv = InitializationVector::from(vec![0u8; 8]);
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &bad_iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes128cbc_cipher_trait_decrypt_invalid_key_length() {
        let cipher = Aes128Cbc;
        let key = vec![0u8; 16];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = AesCbcCipher::encrypt(&cipher, &plaintext, &key, &iv).unwrap();

        // 復号時に無効な鍵長
        let bad_key = vec![0u8; 8];
        let result = AesCbcCipher::decrypt(&cipher, &ciphertext, &bad_key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes256cbc_invalid_iv_length_encrypt() {
        let cipher = Aes256Cbc;
        let key = [0u8; 32];
        let iv = InitializationVector::from(vec![0u8; 8]); // 不正な長さ
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

        // 復号時に無効なIV
        let bad_iv = InitializationVector::from(vec![0u8; 8]);
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key, &bad_iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidIvLength { .. })));
    }

    #[test]
    fn test_aes256cbc_cipher_trait_invalid_key_length() {
        let cipher = Aes256Cbc;
        let key = vec![0u8; 16]; // AES-256には短すぎる
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let result = AesCbcCipher::encrypt(&cipher, &plaintext, &key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_aes256cbc_cipher_trait_decrypt_invalid_key_length() {
        let cipher = Aes256Cbc;
        let key = vec![0u8; 32];
        let iv = InitializationVector::from(vec![0u8; 16]);
        let plaintext = Plaintext::from("Hello");

        let ciphertext = AesCbcCipher::encrypt(&cipher, &plaintext, &key, &iv).unwrap();

        // 復号時に無効な鍵長
        let bad_key = vec![0u8; 16];
        let result = AesCbcCipher::decrypt(&cipher, &ciphertext, &bad_key, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::InvalidKeyLength { .. })));
    }

    #[test]
    fn test_plaintext_conversions() {
        // From<&[u8]>
        let bytes: &[u8] = b"Hello";
        let pt = Plaintext::from(bytes);
        assert_eq!(pt.as_bytes(), b"Hello");

        // From<String>
        let pt2 = Plaintext::from(String::from("World"));
        assert_eq!(pt2.as_bytes(), b"World");

        // into_bytes
        let bytes = pt2.into_bytes();
        assert_eq!(bytes, b"World");
    }

    #[test]
    fn test_ciphertext_conversions() {
        // From<&[u8]>
        let bytes: &[u8] = b"encrypted";
        let ct = Ciphertext::from(bytes);
        assert_eq!(ct.as_bytes(), b"encrypted");

        // into_bytes
        let bytes = ct.into_bytes();
        assert_eq!(bytes, b"encrypted");
    }

    #[test]
    fn test_iv_conversions() {
        // From<&[u8]>
        let bytes: &[u8] = &[1, 2, 3, 4];
        let iv = InitializationVector::from(bytes);
        assert_eq!(iv.as_bytes(), &[1, 2, 3, 4]);
        assert_eq!(iv.len(), 4);
        assert!(!iv.is_empty());

        // 空のIV
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

        // 異なる鍵で復号 → パディングエラー
        let result = SymmetricDecryptor::decrypt(&cipher, &ciphertext, &key2, &iv);
        assert!(matches!(result, Err(SymmetricCipherError::Padding { .. })));
    }
}
