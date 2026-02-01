//! # KoDoGu Crypto Library
//!
//! 暗号化ユーティリティライブラリ。対称鍵暗号、非対称鍵暗号、署名、鍵導出機能を提供。
//!
//! ## 設計パターン
//!
//! `kernel/codec` と同じ 4 つの API パターンを採用:
//!
//! 1. **trait**: 暗号操作の抽象インターフェース
//! 2. **ZST型**: コンパイル時ディスパッチ用（`Aes128Cbc` など）
//! 3. **Enum**: ランタイム選択用（`AesCbc` など）
//! 4. **Cipher-specific trait**: ポリモーフィズム用（`AesCbcCipher` など）
//!
//! ## 使用例
//!
//! ```
//! use crypto::{Aes128Cbc, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
//!
//! let cipher = Aes128Cbc;
//! let key = [0u8; 16];
//! let iv = InitializationVector::from(vec![0u8; 16]);
//! let plaintext = Plaintext::from("Hello, World!");
//!
//! let ciphertext = cipher.encrypt(&plaintext, &key, &iv).unwrap();
//! let decrypted = cipher.decrypt(&ciphertext, &key, &iv).unwrap();
//! assert_eq!(plaintext, decrypted);
//! ```

use suzunari_error::suzunari_error;

mod symmetric;

pub use symmetric::{Aes128Cbc, Aes256Cbc, AesCbc, AesCbcCipher};

//
// 値オブジェクト
//

/// 平文データ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plaintext(Vec<u8>);

impl Plaintext {
    /// バイト列として取得
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// バイト列に変換
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Plaintext {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for Plaintext {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl From<String> for Plaintext {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<&str> for Plaintext {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

/// 暗号文データ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(Vec<u8>);

impl Ciphertext {
    /// バイト列として取得
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// バイト列に変換
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Ciphertext {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for Ciphertext {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

/// 初期化ベクトル (IV)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializationVector(Vec<u8>);

impl InitializationVector {
    /// バイト列として取得
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// バイト長を取得
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 空かどうか
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for InitializationVector {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for InitializationVector {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

//
// 対称鍵暗号 trait
//

/// 対称鍵暗号の暗号化操作
///
/// ブロック暗号（AES-CBC, AES-CTR など）による暗号化を抽象化する。
/// IV/Nonce は呼び出し側が管理する。
pub trait SymmetricEncryptor {
    /// 暗号化に使用する鍵の型
    type Key;

    /// 暗号化エラーの型
    type Error;

    /// 平文を暗号化する
    ///
    /// # Arguments
    /// * `plaintext` - 暗号化する平文
    /// * `key` - 暗号化鍵
    /// * `iv` - 初期化ベクトル
    ///
    /// # Returns
    /// 暗号文、または暗号化エラー
    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, Self::Error>;
}

/// 対称鍵暗号の復号操作
///
/// ブロック暗号（AES-CBC, AES-CTR など）による復号を抽象化する。
pub trait SymmetricDecryptor {
    /// 復号に使用する鍵の型
    type Key;

    /// 復号エラーの型
    type Error;

    /// 暗号文を復号する
    ///
    /// # Arguments
    /// * `ciphertext` - 復号する暗号文
    /// * `key` - 復号鍵
    /// * `iv` - 初期化ベクトル
    ///
    /// # Returns
    /// 平文、または復号エラー
    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Plaintext, Self::Error>;
}

//
// エラー型
//

/// 暗号化操作の種別（エラーメッセージ用）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetricCiphers {
    Aes128Cbc,
    Aes256Cbc,
    Aes128Ctr,
    Aes256Ctr,
    ChaCha20,
}

/// 対称鍵暗号エラー
#[suzunari_error]
#[snafu(visibility(pub), context(suffix(Error)))]
pub enum SymmetricCipherError {
    /// 無効な鍵長
    #[snafu(display("{cipher:?}: 無効な鍵長 {actual} bytes (期待値: {expected} bytes)"))]
    InvalidKeyLength { cipher: SymmetricCiphers, expected: usize, actual: usize },

    /// 無効な IV 長
    #[snafu(display("{cipher:?}: 無効な IV 長 {actual} bytes (期待値: {expected} bytes)"))]
    InvalidIvLength { cipher: SymmetricCiphers, expected: usize, actual: usize },

    /// パディングエラー（暗号化時または復号時の検証失敗）
    #[snafu(display("{cipher:?}: パディングエラー"))]
    Padding { cipher: SymmetricCiphers },

    /// 暗号文が短すぎる（復号時）
    #[snafu(display("{cipher:?}: 暗号文が短すぎます ({size} bytes)"))]
    CiphertextTooShort { cipher: SymmetricCiphers, size: usize },
}
