use super::types::MacTag;

/// MAC generation operation.
///
/// Abstracts message authentication code generation (HMAC, CMAC, etc.).
/// The caller must manage key secrecy and uniqueness requirements.
pub trait MacGenerator {
    /// Key type for MAC generation.
    type Key: ?Sized;

    /// Error type for MAC generation.
    type Error;

    /// Generates a MAC tag for `message` with the given `key`.
    fn generate(&self, message: &[u8], key: &Self::Key) -> Result<MacTag, Self::Error>;
}

/// MAC verification operation.
///
/// Abstracts message authentication code verification.
/// Verification uses constant-time comparison to prevent timing attacks.
pub trait MacVerifier {
    /// Key type for MAC verification.
    type Key: ?Sized;

    /// Error type for MAC verification.
    type Error;

    /// Verifies a MAC tag for `message` with the given `key`.
    ///
    /// Returns `Ok(())` if the tag is valid, `Err` if verification fails.
    /// Uses constant-time comparison to prevent timing attacks.
    fn verify(&self, message: &[u8], tag: &MacTag, key: &Self::Key) -> Result<(), Self::Error>;
}

/// Combined MAC generation and verification.
///
/// Marker trait for types implementing both [`MacGenerator`] and [`MacVerifier`].
/// Guarantees the same key and error types for both operations.
pub trait Mac:
    MacGenerator<Key = <Self as Mac>::Key, Error = <Self as Mac>::Error>
    + MacVerifier<Key = <Self as Mac>::Key, Error = <Self as Mac>::Error>
{
    /// Key type for MAC operations.
    type Key: ?Sized;

    /// Error type for MAC operations.
    type Error;
}
