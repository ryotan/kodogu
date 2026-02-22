#[cfg(test)]
pub(crate) mod test_helpers {
    use crate::kdf::{DerivedKey, KeyDeriver, PasswordDeriver};

    /// Asserts that the same input produces the same key (deterministic).
    pub fn assert_key_deriver_deterministic<D>(
        deriver: &D,
        input: &D::Input,
        info: &[u8],
        length: usize,
    ) where
        D: KeyDeriver,
        D::Error: std::fmt::Debug,
        D::Input: Sized,
    {
        let key1 = deriver.derive(input, info, length).unwrap();
        let key2 = deriver.derive(input, info, length).unwrap();
        assert_eq!(key1, key2, "same input must produce the same key");
    }

    /// Asserts that the same input produces the same key (password-based).
    pub fn assert_password_deriver_deterministic<D>(
        deriver: &D,
        password: &[u8],
        salt: &[u8],
        params: &D::Params,
        length: usize,
    ) where
        D: PasswordDeriver,
        D::Error: std::fmt::Debug,
    {
        let key1 = deriver.derive(password, salt, params, length).unwrap();
        let key2 = deriver.derive(password, salt, params, length).unwrap();
        assert_eq!(key1, key2, "same input must produce the same key");
    }

    /// Asserts that the derived key has the expected length.
    pub fn assert_key_length(key: &DerivedKey, expected_length: usize) {
        assert_eq!(key.len(), expected_length, "derived key length must match expected");
    }

    /// Asserts that different inputs produce different keys.
    pub fn assert_different_keys(key1: &DerivedKey, key2: &DerivedKey) {
        assert_ne!(key1, key2, "different inputs must produce different keys");
    }
}
