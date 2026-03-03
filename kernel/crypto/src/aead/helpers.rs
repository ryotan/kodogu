use super::errors::{
    AeadCiphers, AeadError, InvalidKeyLengthError, InvalidNonceLengthError, InvalidTagLengthError,
};
use super::types::AuthenticationTag;
use crate::Nonce;
use suzunari_error::ResultExt;

/// Validates key length and converts to a fixed-size array.
pub(crate) fn validate_key_length<const N: usize>(
    key: &[u8],
    cipher: AeadCiphers,
) -> Result<[u8; N], AeadError> {
    key.try_into()
        .context(InvalidKeyLengthError { cipher, expected: N, actual: key.len() })
}

/// Validates nonce length and converts to a fixed-size array.
pub(crate) fn validate_nonce_length<const N: usize>(
    nonce: &Nonce,
    cipher: AeadCiphers,
) -> Result<[u8; N], AeadError> {
    nonce.as_bytes().try_into().context(InvalidNonceLengthError {
        cipher,
        expected: N,
        actual: nonce.len(),
    })
}

/// Validates authentication tag length and converts to a fixed-size array.
pub(crate) fn validate_tag_length<const N: usize>(
    tag: &AuthenticationTag,
    cipher: AeadCiphers,
) -> Result<[u8; N], AeadError> {
    tag.as_bytes().try_into().context(InvalidTagLengthError {
        cipher,
        expected: N,
        actual: tag.len(),
    })
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::super::types::{AdditionalData, AeadCiphertext, AuthenticationTag};
    use crate::Nonce;
    use crate::{Ciphertext, Plaintext};

    /// AEAD ラウンドトリップテスト（AAD なし）。
    pub fn assert_aead_roundtrip<E, D>(nonce: &Nonce, encrypt: E, decrypt: D)
    where
        E: Fn(&Plaintext, &Nonce, &AdditionalData) -> (Ciphertext, AuthenticationTag),
        D: Fn(&Ciphertext, &AuthenticationTag, &Nonce, &AdditionalData) -> Plaintext,
    {
        let aad = AdditionalData::empty();
        for len in [0, 1, 15, 16, 17, 31, 32, 33, 100] {
            let plaintext = Plaintext::from(vec![0xABu8; len]);
            let (ciphertext, tag) = encrypt(&plaintext, nonce, &aad);
            let decrypted = decrypt(&ciphertext, &tag, nonce, &aad);
            assert_eq!(plaintext, decrypted, "Failed for length {}", len);
        }
    }

    /// AEAD ラウンドトリップテスト（AAD あり）。
    pub fn assert_aead_roundtrip_with_aad<E, D>(
        nonce: &Nonce,
        aad: &AdditionalData,
        encrypt: E,
        decrypt: D,
    ) where
        E: Fn(&Plaintext, &Nonce, &AdditionalData) -> (Ciphertext, AuthenticationTag),
        D: Fn(&Ciphertext, &AuthenticationTag, &Nonce, &AdditionalData) -> Plaintext,
    {
        let plaintext = Plaintext::from("Hello, AEAD with AAD!");
        let (ciphertext, tag) = encrypt(&plaintext, nonce, aad);
        let decrypted = decrypt(&ciphertext, &tag, nonce, aad);
        assert_eq!(plaintext, decrypted);
    }

    /// 結合型 API ラウンドトリップテスト。
    pub fn assert_combined_roundtrip<E, D>(nonce: &Nonce, encrypt: E, decrypt: D)
    where
        E: Fn(&Plaintext, &Nonce, &AdditionalData) -> AeadCiphertext,
        D: Fn(&AeadCiphertext, &Nonce, &AdditionalData) -> Plaintext,
    {
        let aad = AdditionalData::empty();
        let plaintext = Plaintext::from("Combined API test");
        let aead_ciphertext = encrypt(&plaintext, nonce, &aad);
        let decrypted = decrypt(&aead_ciphertext, nonce, &aad);
        assert_eq!(plaintext, decrypted);
    }
}
