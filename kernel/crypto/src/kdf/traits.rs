use super::types::{DerivedKey, OutputLength};

/// Key derivation from keying material (e.g., shared secrets).
///
/// Shared secrets must not be used directly as encryption keys; always derive via this trait.
///
/// # Empty input
///
/// Some algorithms (HKDF) accept empty input keying material per their specification (RFC 5869),
/// while others (ConcatKDF, X963KDF) reject it. Check each implementation's documentation.
pub trait KeyDeriver {
    /// Input keying material type.
    type Input: ?Sized;

    /// Error type.
    type Error;

    /// Derives a key of specified length from input material.
    ///
    /// - `input`: Input keying material
    /// - `info`: Context information (usage label, etc.; may be empty)
    /// - `length`: Output key length in bytes
    fn derive(
        &self,
        input: &Self::Input,
        info: &[u8],
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error>;
}

/// Key derivation from passwords (low-entropy input).
///
/// # Salt requirements
///
/// All implementations accept any salt length, including empty.
/// However, **using an empty or short salt is cryptographically dangerous** --
/// it makes the output deterministic across all users with the same password.
/// Always use a cryptographically random salt of at least 16 bytes (OWASP recommendation).
pub trait PasswordDeriver {
    /// Algorithm-specific parameter type (iterations, memory cost, etc.).
    type Params;

    /// Error type.
    type Error;

    /// Derives a key of specified length from a password and salt.
    ///
    /// - `password`: Password
    /// - `salt`: Cryptographically random salt (minimum 16 bytes recommended; see trait doc)
    /// - `params`: Algorithm-specific parameters
    /// - `length`: Output key length in bytes
    fn derive(
        &self,
        password: &[u8],
        salt: &[u8],
        params: &Self::Params,
        length: OutputLength,
    ) -> Result<DerivedKey, Self::Error>;
}
