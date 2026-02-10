use crate::{Ciphertext, InitializationVector, Plaintext};
use snafu::ResultExt;
use std::array::TryFromSliceError;
use suzunari_error::suzunari_error;

/// Symmetric encryption operation.
///
/// Abstracts block cipher encryption (AES-CBC, AES-CTR, etc.).
/// The caller must manage IV/nonce.
pub trait SymmetricEncryptor {
    /// Key type for encryption.
    type Key: ?Sized;

    /// Error type for encryption.
    type Error;

    /// Encrypts `plaintext` with the given `key` and `iv`.
    fn encrypt(
        &self,
        plaintext: &Plaintext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Ciphertext, Self::Error>;
}

/// Symmetric decryption operation.
///
/// Abstracts block cipher decryption (AES-CBC, AES-CTR, etc.).
pub trait SymmetricDecryptor {
    /// Key type for decryption.
    type Key: ?Sized;

    /// Error type for decryption.
    type Error;

    /// Decrypts `ciphertext` with the given `key` and `iv`.
    fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        key: &Self::Key,
        iv: &InitializationVector,
    ) -> Result<Plaintext, Self::Error>;
}

/// Combined symmetric encryption and decryption.
///
/// Marker trait for types implementing both [`SymmetricEncryptor`] and [`SymmetricDecryptor`].
/// Guarantees the same key and error types for both operations.
pub trait SymmetricCipher:
    SymmetricEncryptor<
        Key = <Self as SymmetricCipher>::Key,
        Error = <Self as SymmetricCipher>::Error,
    > + SymmetricDecryptor<
        Key = <Self as SymmetricCipher>::Key,
        Error = <Self as SymmetricCipher>::Error,
    >
{
    /// Key type for encryption/decryption.
    type Key: ?Sized;

    /// Error type for encryption/decryption.
    type Error;
}

/// Cipher algorithm identifier for error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetricCiphers {
    Aes128Cbc,
    Aes256Cbc,
    Aes128Ctr,
    Aes256Ctr,
    ChaCha20,
}

/// Symmetric cipher error.
#[suzunari_error]
#[snafu(visibility(pub), context(suffix(Error)))]
pub enum SymmetricCipherError {
    /// Invalid key length
    #[snafu(display("{cipher:?}: invalid key length {actual} bytes (expected {expected} bytes)"))]
    InvalidKeyLength {
        cipher: SymmetricCiphers,
        expected: usize,
        actual: usize,
        source: TryFromSliceError,
    },

    /// Invalid IV length
    #[snafu(display("{cipher:?}: invalid IV length {actual} bytes (expected {expected} bytes)"))]
    InvalidIvLength {
        cipher: SymmetricCiphers,
        expected: usize,
        actual: usize,
        source: TryFromSliceError,
    },

    /// Padding error during encryption or decryption
    #[snafu(display("{cipher:?}: padding error"))]
    Padding { cipher: SymmetricCiphers },

    /// Ciphertext is too short for decryption
    #[snafu(display("{cipher:?}: ciphertext too short ({size} bytes)"))]
    CiphertextTooShort { cipher: SymmetricCiphers, size: usize },
}

/// Validates AES key length and converts to the fixed-size array.
pub(crate) fn validate_key_length<const N: usize>(
    key: &[u8],
    cipher: SymmetricCiphers,
) -> Result<[u8; N], SymmetricCipherError> {
    key.try_into()
        .context(InvalidKeyLengthError { cipher, expected: N, actual: key.len() })
}

/// Validates IV length and converts to a fixed-size array.
pub(crate) fn validate_iv_length<const N: usize>(
    iv: &InitializationVector,
    cipher: SymmetricCiphers,
) -> Result<[u8; N], SymmetricCipherError> {
    iv.as_bytes()
        .try_into()
        .context(InvalidIvLengthError { cipher, expected: N, actual: iv.len() })
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use crate::{Ciphertext, InitializationVector, Plaintext};

    /// ストリーム暗号のパディング不要テスト。
    /// 平文長 0..=33 で暗号化/復号をテストし、暗号文長が平文長と等しいことを確認。
    pub fn assert_no_padding_needed<E, D>(iv: &InitializationVector, encrypt: E, decrypt: D)
    where
        E: Fn(&Plaintext, &InitializationVector) -> Ciphertext,
        D: Fn(&Ciphertext, &InitializationVector) -> Plaintext,
    {
        for len in 0..=33 {
            let plaintext = Plaintext::from(vec![0xABu8; len]);
            let ciphertext = encrypt(&plaintext, iv);

            // 暗号文長 == 平文長（パディングなし）
            assert_eq!(ciphertext.as_bytes().len(), len);

            let decrypted = decrypt(&ciphertext, iv);
            assert_eq!(plaintext, decrypted, "Failed for length {}", len);
        }
    }

    /// 異なる鍵で異なる暗号文が生成されることをテスト。
    pub fn assert_different_keys_produce_different_ciphertext(
        plaintext: &Plaintext,
        ciphertext1: &Ciphertext,
        ciphertext2: &Ciphertext,
    ) {
        assert_ne!(
            ciphertext1, ciphertext2,
            "Different keys should produce different ciphertext for plaintext {:?}",
            plaintext
        );
    }

    /// 異なる IV/nonce で異なる暗号文が生成されることをテスト。
    pub fn assert_different_ivs_produce_different_ciphertext(
        plaintext: &Plaintext,
        ciphertext1: &Ciphertext,
        ciphertext2: &Ciphertext,
    ) {
        assert_ne!(
            ciphertext1, ciphertext2,
            "Different IVs should produce different ciphertext for plaintext {:?}",
            plaintext
        );
    }
}
