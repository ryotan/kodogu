use super::errors::{
    InvalidIvLengthError, InvalidKeyLengthError, InvalidNonceLengthError, SymmetricCipherError,
    SymmetricCiphers,
};
use crate::{InitializationVector, Nonce};
use snafu::ResultExt;

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

/// Validates nonce length and converts to a fixed-size array.
pub(crate) fn validate_nonce_length<const N: usize>(
    nonce: &Nonce,
    cipher: SymmetricCiphers,
) -> Result<[u8; N], SymmetricCipherError> {
    nonce.as_bytes().try_into().context(InvalidNonceLengthError {
        cipher,
        expected: N,
        actual: nonce.len(),
    })
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use crate::{Ciphertext, Nonce, Plaintext};

    /// ストリーム暗号のパディング不要テスト。
    /// 平文長 0..=33 で暗号化/復号をテストし、暗号文長が平文長と等しいことを確認。
    pub fn assert_no_padding_needed<E, D>(nonce: &Nonce, encrypt: E, decrypt: D)
    where
        E: Fn(&Plaintext, &Nonce) -> Ciphertext,
        D: Fn(&Ciphertext, &Nonce) -> Plaintext,
    {
        for len in 0..=33 {
            let plaintext = Plaintext::from(vec![0xABu8; len]);
            let ciphertext = encrypt(&plaintext, nonce);

            // 暗号文長 == 平文長（パディングなし）
            assert_eq!(ciphertext.as_bytes().len(), len);

            let decrypted = decrypt(&ciphertext, nonce);
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

    /// 異なる nonce で異なる暗号文が生成されることをテスト。
    pub fn assert_different_nonces_produce_different_ciphertext(
        plaintext: &Plaintext,
        ciphertext1: &Ciphertext,
        ciphertext2: &Ciphertext,
    ) {
        assert_ne!(
            ciphertext1, ciphertext2,
            "Different nonces should produce different ciphertext for plaintext {:?}",
            plaintext
        );
    }
}
