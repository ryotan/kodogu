use super::types::DerivedKey;

/// Key derivation from keying material (e.g., shared secrets).
///
/// Shared secrets must not be used directly as encryption keys; always derive via this trait.
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
        length: usize,
    ) -> Result<DerivedKey, Self::Error>;
}

/// Key derivation from passwords (low-entropy input).
pub trait PasswordDeriver {
    /// Algorithm-specific parameter type (iterations, memory cost, etc.).
    type Params;

    /// Error type.
    type Error;

    /// Derives a key of specified length from a password and salt.
    ///
    /// - `password`: Password
    /// - `salt`: Salt (randomly generated, minimum 16 bytes recommended)
    /// - `params`: Algorithm-specific parameters
    /// - `length`: Output key length in bytes
    fn derive(
        &self,
        password: &[u8],
        salt: &[u8],
        params: &Self::Params,
        length: usize,
    ) -> Result<DerivedKey, Self::Error>;
}
