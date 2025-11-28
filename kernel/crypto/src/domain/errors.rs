use std::fmt::Display;

use suzunari_error::BoxedStackError;

/// Domain-level cryptographic errors
#[derive(Debug)]
pub enum CryptoError {
    KeyGeneration { source: BoxedStackError },

    Encryption { source: BoxedStackError },

    Decryption { source: BoxedStackError },

    KeyDerivation { source: BoxedStackError },
}

impl Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CryptoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_error_implements_error_trait() {
        // CryptoError variants are tested through integration tests
        // Here we just verify that CryptoError implements the Error trait
        fn assert_error<T: std::error::Error>() {}
        assert_error::<CryptoError>();
    }

    #[test]
    fn crypto_error_implements_display() {
        // Verify CryptoError implements Display trait
        fn assert_display<T: Display>() {}
        assert_display::<CryptoError>();
    }

    #[test]
    fn crypto_error_has_four_variants() {
        // This test documents that CryptoError has exactly 4 variants
        // If a variant is added or removed, this test will fail at compile time
        fn check_exhaustiveness(err: &CryptoError) {
            match err {
                CryptoError::KeyGeneration { .. } => {}
                CryptoError::Encryption { .. } => {}
                CryptoError::Decryption { .. } => {}
                CryptoError::KeyDerivation { .. } => {}
            }
        }
        // Just to avoid unused function warning
        let _ = check_exhaustiveness;
    }
}
