use core::fmt::{Debug, Formatter};
use subtle::ConstantTimeEq;
use suzunari_error::suzunari_error;
use suzunari_error::{ResultExt, ensure};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// Error for [`OutputLength::try_new`]: the length is zero.
#[suzunari_error]
#[suzu(visibility(pub(crate)), display("output length must be greater than 0"))]
pub struct OutputLengthError {}

/// Output key length in bytes. Guaranteed to be > 0.
///
/// Follows the same dual-constructor pattern as other parameter types in this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputLength(usize);

impl OutputLength {
    /// Creates an output length.
    ///
    /// # Panics
    ///
    /// Panics if `length` is 0.
    pub const fn new(length: usize) -> Self {
        assert!(length > 0, "output length must be greater than 0");
        Self(length)
    }

    /// Creates an output length with validation. For runtime input.
    pub fn try_new(length: usize) -> Result<Self, OutputLengthError> {
        ensure!(length > 0, OutputLengthSnafu);
        Ok(Self(length))
    }

    /// Returns the length in bytes.
    pub const fn get(&self) -> usize {
        self.0
    }
}

/// Derived key material.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct DerivedKey(Vec<u8>);

impl_secret_traits!(DerivedKey);

impl DerivedKey {
    /// Creates key material from a KDF implementation.
    ///
    /// Cannot be constructed from external crates. Only obtained via KDF `derive` methods.
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        assert!(!bytes.is_empty(), "DerivedKey must not be empty");
        Self(bytes)
    }

    /// Returns as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Always returns `false`. Non-empty is guaranteed by construction.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consumes self and returns the underlying bytes.
    ///
    /// The returned [`Zeroizing`] wrapper ensures the bytes are cleared on drop.
    pub fn into_bytes(mut self) -> Zeroizing<Vec<u8>> {
        // take moves the bytes out, so drop zeroizes an empty Vec
        Zeroizing::new(std::mem::take(&mut self.0))
    }
}

/// HKDF input: IKM (Input Keying Material) and optional salt.
///
/// # Security
///
/// IKM may contain sensitive data (e.g., shared secrets).
/// Automatically zeroized on drop ([`ZeroizeOnDrop`]).
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct HkdfInput {
    /// Input Keying Material. E.g., shared secrets.
    ikm: Vec<u8>,
    /// Salt (optional). When None, HKDF-Extract uses a zero-filled salt of hash length (RFC 5869 §2.2).
    salt: Option<Vec<u8>>,
}

impl Debug for HkdfInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HkdfInput")
            .field("ikm", &"***")
            .field("salt", &self.salt.as_ref().map(|_| "[present]"))
            .finish()
    }
}

impl PartialEq for HkdfInput {
    fn eq(&self, other: &Self) -> bool {
        let ikm_eq: bool = self.ikm.ct_eq(&other.ikm).into();
        let salt_eq: bool = match (&self.salt, &other.salt) {
            (Some(a), Some(b)) => a.ct_eq(b).into(),
            (None, None) => true,
            _ => false,
        };
        // Bitwise AND to avoid short-circuit evaluation (timing attack mitigation)
        ikm_eq & salt_eq
    }
}

impl Eq for HkdfInput {}

impl HkdfInput {
    /// Creates with IKM and salt.
    pub fn new(ikm: Vec<u8>, salt: Option<Vec<u8>>) -> Self {
        Self { ikm, salt }
    }

    /// Creates without salt (RFC 5869 default salt).
    pub fn without_salt(ikm: Vec<u8>) -> Self {
        Self { ikm, salt: None }
    }

    /// Returns the input keying material.
    pub fn ikm(&self) -> &[u8] {
        &self.ikm
    }

    /// Returns the salt.
    pub fn salt(&self) -> Option<&[u8]> {
        self.salt.as_deref()
    }
}

/// Error for [`Pbkdf2Params::try_new`]: `iterations` is zero.
#[suzunari_error]
#[suzu(visibility(pub(crate)), display("iterations must be greater than 0"))]
pub struct Pbkdf2ParamsError {}

/// PBKDF2 parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pbkdf2Params {
    /// Iteration count.
    ///
    /// OWASP 2023 recommendation: minimum 600,000 for PBKDF2-SHA256.
    iterations: u32,
}

impl Pbkdf2Params {
    /// OWASP 2023 recommended value (PBKDF2-SHA256: 600,000 iterations).
    pub const OWASP_RECOMMENDED: Self = Self { iterations: 600_000 };

    /// Creates parameters.
    ///
    /// # Panics
    ///
    /// Panics if `iterations` is 0.
    pub const fn new(iterations: u32) -> Self {
        assert!(iterations > 0, "iterations must be greater than 0");
        Self { iterations }
    }

    /// Creates parameters with validation. For runtime input.
    pub fn try_new(iterations: u32) -> Result<Self, Pbkdf2ParamsError> {
        ensure!(iterations > 0, Pbkdf2ParamsSnafu);
        Ok(Self { iterations })
    }

    /// Returns the iteration count.
    pub const fn iterations(&self) -> u32 {
        self.iterations
    }
}

/// Error for [`Argon2Params::try_new`]: the `argon2` crate rejects the parameters.
#[suzunari_error]
#[suzu(visibility(pub(crate)), display("invalid Argon2 parameters"))]
pub struct Argon2ParamsError {
    #[suzu(from)]
    source: argon2::Error,
}

/// Argon2id parameters, validated by the `argon2` crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argon2Params(argon2::Params);

impl Argon2Params {
    /// RFC 9106 §4's FIRST RECOMMENDED option: 2 GiB, t=1, p=4.
    pub const RFC9106_FIRST_RECOMMENDED: Self = Self::new(2_097_152, 1, 4);

    /// RFC 9106 §4's SECOND RECOMMENDED option, for much less memory: 64 MiB, t=3, p=4.
    pub const RFC9106_SECOND_RECOMMENDED: Self = Self::new(65_536, 3, 4);

    /// OWASP Password Storage Cheat Sheet's minimum configuration as of 2026-10: 19 MiB, t=2, p=1.
    pub const OWASP_MINIMUM: Self = Self::new(19_456, 2, 1);

    /// Creates parameters.
    ///
    /// # Panics
    ///
    /// Panics if the `argon2` crate rejects the parameters (see [`Self::try_new`]).
    pub const fn new(memory_kib: u32, time: u32, parallelism: u32) -> Self {
        match argon2::Params::new(memory_kib, time, parallelism, None) {
            Ok(params) => Self(params),
            Err(_) => panic!("invalid Argon2 parameters"),
        }
    }

    /// Creates parameters with validation. For runtime input.
    ///
    /// # Errors
    /// - [`Argon2ParamsError`] if the `argon2` crate rejects the parameters, e.g. `memory_kib` below
    ///   8 × `parallelism` (RFC 9106)
    pub fn try_new(
        memory_kib: u32,
        time: u32,
        parallelism: u32,
    ) -> Result<Self, Argon2ParamsError> {
        argon2::Params::new(memory_kib, time, parallelism, None)
            .map(Self)
            .context(Argon2ParamsSnafu)
    }

    /// Returns the memory cost in KiB.
    pub const fn memory_kib(&self) -> u32 {
        self.0.m_cost()
    }

    /// Returns the time cost (number of passes).
    pub const fn time(&self) -> u32 {
        self.0.t_cost()
    }

    /// Returns the parallelism.
    pub const fn parallelism(&self) -> u32 {
        self.0.p_cost()
    }

    pub(crate) fn inner(&self) -> &argon2::Params {
        &self.0
    }
}

/// Error for [`ScryptParams::try_new`]: the `scrypt` crate rejects the parameters.
#[suzunari_error]
#[suzu(visibility(pub(crate)), display("invalid scrypt parameters"))]
pub struct ScryptParamsError {
    source: scrypt::errors::InvalidParams,
}

/// Scrypt parameters, validated by the `scrypt` crate.
#[derive(Debug, Clone)]
pub struct ScryptParams(scrypt::Params);

impl ScryptParams {
    /// The `scrypt` crate's recommended parameters ([`scrypt::Params::recommended`]).
    pub fn recommended() -> Self {
        Self(scrypt::Params::recommended())
    }

    /// Creates parameters.
    ///
    /// # Panics
    ///
    /// Panics if the `scrypt` crate rejects the parameters (see [`Self::try_new`]).
    pub fn new(log_n: u8, r: u32, p: u32) -> Self {
        match Self::try_new(log_n, r, p) {
            Ok(params) => params,
            Err(error) => panic!("{error}"),
        }
    }

    /// Creates parameters with validation. For runtime input.
    ///
    /// # Errors
    /// - [`ScryptParamsError`] if the `scrypt` crate rejects the parameters, e.g. `log_n` not below
    ///   16 × `r` or `r` × `p` not below 2^30 (RFC 7914)
    pub fn try_new(log_n: u8, r: u32, p: u32) -> Result<Self, ScryptParamsError> {
        // scrypt 0.11 requires an output length, but only its password-hash API uses it; `scrypt::scrypt` takes the
        // length from the output buffer.
        scrypt::Params::new(log_n, r, p, scrypt::Params::RECOMMENDED_LEN)
            .map(Self)
            .context(ScryptParamsSnafu)
    }

    /// Returns log_n, the base-2 logarithm of the CPU/memory cost N.
    pub fn log_n(&self) -> u8 {
        self.0.log_n()
    }

    /// Returns the block size r.
    pub fn r(&self) -> u32 {
        self.0.r()
    }

    /// Returns the parallelization p.
    pub fn p(&self) -> u32 {
        self.0.p()
    }

    pub(crate) fn inner(&self) -> &scrypt::Params {
        &self.0
    }
}

impl PartialEq for ScryptParams {
    fn eq(&self, other: &Self) -> bool {
        (self.log_n(), self.r(), self.p()) == (other.log_n(), other.r(), other.p())
    }
}

impl Eq for ScryptParams {}

/// Error for [`BcryptPbkdfParams::try_new`]: `rounds` is zero.
#[suzunari_error]
#[suzu(visibility(pub(crate)), display("rounds must be greater than 0"))]
pub struct BcryptPbkdfParamsError {}

/// bcrypt-pbkdf parameters.
///
/// Used for OpenSSH key file encryption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcryptPbkdfParams {
    /// Round count (minimum 1).
    ///
    /// OpenSSH default: 16. For higher security: 64+.
    rounds: u32,
}

impl BcryptPbkdfParams {
    /// OpenSSH default value (16 rounds).
    pub const OPENSSH_DEFAULT: Self = Self { rounds: 16 };

    /// Creates parameters.
    ///
    /// # Panics
    ///
    /// Panics if `rounds` is 0.
    pub const fn new(rounds: u32) -> Self {
        assert!(rounds > 0, "rounds must be greater than 0");
        Self { rounds }
    }

    /// Creates parameters with validation. For runtime input.
    pub fn try_new(rounds: u32) -> Result<Self, BcryptPbkdfParamsError> {
        ensure!(rounds > 0, BcryptPbkdfParamsSnafu);
        Ok(Self { rounds })
    }

    /// Returns the round count.
    pub const fn rounds(&self) -> u32 {
        self.rounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // OutputLength tests
    // ========================================

    #[test]
    fn test_output_length_new() {
        let ol = OutputLength::new(32);
        assert_eq!(ol.get(), 32);
    }

    #[test]
    fn test_output_length_try_new_ok() {
        let ol = OutputLength::try_new(32).unwrap();
        assert_eq!(ol.get(), 32);
    }

    #[test]
    fn test_output_length_try_new_zero() {
        assert!(OutputLength::try_new(0).is_err());
    }

    #[test]
    #[should_panic(expected = "output length must be greater than 0")]
    fn test_output_length_new_zero_panics() {
        OutputLength::new(0);
    }

    #[test]
    fn test_output_length_error_display() {
        assert_eq!(
            OutputLength::try_new(0).unwrap_err().to_string(),
            "output length must be greater than 0"
        );
    }

    #[test]
    fn test_derived_key_new() {
        let data = vec![1, 2, 3, 4];
        let key = DerivedKey::new(data.clone());
        assert_eq!(key.as_bytes(), &data);
        assert_eq!(key.len(), 4);
        assert!(!key.is_empty());
    }

    #[test]
    fn test_derived_key_into_bytes() {
        let data = vec![1, 2, 3, 4];
        let key = DerivedKey::new(data.clone());
        assert_eq!(*key.into_bytes(), data);
    }

    #[test]
    fn test_derived_key_debug_hides_key_material() {
        let key = DerivedKey::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let debug = format!("{:?}", key);
        assert!(debug.contains("***"), "Debug output must hide key material");
        assert!(!debug.contains("dead"), "Debug output must not contain key material");
        assert!(!debug.contains("DEAD"), "Debug output must not contain key material");
    }

    #[test]
    fn test_hkdf_input_without_salt() {
        let input = HkdfInput::without_salt(b"ikm".to_vec());
        assert_eq!(input.ikm(), b"ikm");
        assert!(input.salt().is_none());
    }

    #[test]
    fn test_hkdf_input_with_salt() {
        let input = HkdfInput::new(b"ikm".to_vec(), Some(b"salt".to_vec()));
        assert_eq!(input.ikm(), b"ikm");
        assert_eq!(input.salt(), Some(b"salt".as_slice()));
    }

    #[test]
    fn test_hkdf_input_debug_hides_ikm() {
        let input = HkdfInput::new(b"secret-ikm".to_vec(), Some(b"salt".to_vec()));
        let debug = format!("{:?}", input);
        assert!(debug.contains("***"), "Debug output must hide IKM");
        assert!(!debug.contains("secret-ikm"), "Debug output must not contain IKM");
    }

    #[test]
    fn test_pbkdf2_params_owasp_recommended() {
        assert_eq!(Pbkdf2Params::OWASP_RECOMMENDED.iterations(), 600_000);
    }

    #[test]
    fn test_argon2_params_presets() {
        let presets = [
            (Argon2Params::RFC9106_FIRST_RECOMMENDED, (2_097_152, 1, 4)),
            (Argon2Params::RFC9106_SECOND_RECOMMENDED, (65_536, 3, 4)),
            (Argon2Params::OWASP_MINIMUM, (19_456, 2, 1)),
        ];
        for (params, expected) in presets {
            assert_eq!((params.memory_kib(), params.time(), params.parallelism()), expected);
        }
    }

    #[test]
    fn test_derived_key_constant_time_eq() {
        let key1 = DerivedKey::new(vec![1, 2, 3]);
        let key2 = DerivedKey::new(vec![1, 2, 3]);
        let key3 = DerivedKey::new(vec![4, 5, 6]);
        let key4 = DerivedKey::new(vec![1, 2, 3, 4]);
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key1, key4, "different lengths must not be equal");
    }

    #[test]
    fn test_hkdf_input_constant_time_eq() {
        let input1 = HkdfInput::new(b"ikm".to_vec(), Some(b"salt".to_vec()));
        let input2 = HkdfInput::new(b"ikm".to_vec(), Some(b"salt".to_vec()));
        let input3 = HkdfInput::new(b"other".to_vec(), Some(b"salt".to_vec()));
        let input4 = HkdfInput::new(b"ikm".to_vec(), Some(b"other".to_vec()));
        let input5 = HkdfInput::new(b"ikm".to_vec(), None);
        let input6 = HkdfInput::without_salt(b"ikm".to_vec());
        assert_eq!(input1, input2);
        assert_ne!(input1, input3);
        assert_ne!(input1, input4);
        assert_ne!(input1, input5);
        assert_eq!(input5, input6);
    }

    #[test]
    fn test_scrypt_params_recommended() {
        let p = ScryptParams::recommended();
        assert_eq!(p.log_n(), scrypt::Params::RECOMMENDED_LOG_N);
        assert_eq!(p.r(), scrypt::Params::RECOMMENDED_R);
        assert_eq!(p.p(), scrypt::Params::RECOMMENDED_P);
    }

    // ========================================
    // try_new tests
    // ========================================

    #[test]
    fn test_pbkdf2_params_try_new_ok() {
        let p = Pbkdf2Params::try_new(1000).unwrap();
        assert_eq!(p.iterations(), 1000);
    }

    #[test]
    fn test_pbkdf2_params_try_new_zero() {
        assert!(Pbkdf2Params::try_new(0).is_err());
    }

    #[test]
    fn test_argon2_params_try_new_ok() {
        let p = Argon2Params::try_new(8, 1, 1).unwrap();
        assert_eq!(p.memory_kib(), 8);
        assert_eq!(p.time(), 1);
        assert_eq!(p.parallelism(), 1);
    }

    #[test]
    fn test_scrypt_params_try_new_ok() {
        let p = ScryptParams::try_new(4, 1, 1).unwrap();
        assert_eq!(p.log_n(), 4);
        assert_eq!(p.r(), 1);
        assert_eq!(p.p(), 1);
    }

    #[test]
    fn test_argon2_params_try_new_rejects() {
        // Zero memory, time, and parallelism; memory below 8 × parallelism
        for (memory_kib, time, parallelism) in [(0, 1, 1), (8, 0, 1), (8, 1, 0), (8, 1, 4)] {
            assert!(
                Argon2Params::try_new(memory_kib, time, parallelism).is_err(),
                "{memory_kib}, {time}, {parallelism}"
            );
        }
    }

    #[test]
    fn test_scrypt_params_try_new_rejects() {
        // Zero r and p; log_n not below 16 × r; r × p not below 2^30
        for (log_n, r, p) in [(4, 0, 1), (4, 1, 0), (16, 1, 1), (4, 1 << 15, 1 << 15)] {
            assert!(ScryptParams::try_new(log_n, r, p).is_err(), "{log_n}, {r}, {p}");
        }
    }

    #[test]
    fn test_bcrypt_pbkdf_params_try_new_ok() {
        let p = BcryptPbkdfParams::try_new(16).unwrap();
        assert_eq!(p.rounds(), 16);
    }

    #[test]
    fn test_bcrypt_pbkdf_params_try_new_zero() {
        assert!(BcryptPbkdfParams::try_new(0).is_err());
    }

    #[test]
    fn test_bcrypt_pbkdf_params_openssh_default() {
        assert_eq!(BcryptPbkdfParams::OPENSSH_DEFAULT.rounds(), 16);
    }

    // ========================================
    // Display tests
    // ========================================

    #[test]
    fn test_pbkdf2_params_error_display() {
        assert_eq!(
            Pbkdf2Params::try_new(0).unwrap_err().to_string(),
            "iterations must be greater than 0"
        );
    }

    #[test]
    fn test_argon2_params_error_display() {
        assert_eq!(
            Argon2Params::try_new(8, 1, 4).unwrap_err().to_string(),
            "invalid Argon2 parameters"
        );
    }

    #[test]
    fn test_scrypt_params_error_display() {
        assert_eq!(
            ScryptParams::try_new(16, 1, 1).unwrap_err().to_string(),
            "invalid scrypt parameters"
        );
    }

    #[test]
    fn test_bcrypt_pbkdf_params_error_display() {
        assert_eq!(
            BcryptPbkdfParams::try_new(0).unwrap_err().to_string(),
            "rounds must be greater than 0"
        );
    }

    // ========================================
    // Panic guard tests
    // ========================================

    #[test]
    #[should_panic(expected = "DerivedKey must not be empty")]
    fn test_derived_key_new_empty_panics() {
        DerivedKey::new(vec![]);
    }

    #[test]
    #[should_panic(expected = "iterations must be greater than 0")]
    fn test_pbkdf2_params_new_zero_panics() {
        Pbkdf2Params::new(0);
    }

    #[test]
    #[should_panic(expected = "invalid Argon2 parameters")]
    fn test_argon2_params_new_invalid_panics() {
        Argon2Params::new(8, 1, 4);
    }

    #[test]
    #[should_panic(expected = "invalid scrypt parameters")]
    fn test_scrypt_params_new_invalid_panics() {
        ScryptParams::new(16, 1, 1);
    }

    #[test]
    #[should_panic(expected = "rounds must be greater than 0")]
    fn test_bcrypt_pbkdf_params_new_zero_panics() {
        BcryptPbkdfParams::new(0);
    }
}
