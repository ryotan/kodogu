use core::fmt::{Debug, Formatter};
use subtle::ConstantTimeEq;
use suzunari_error::ensure;
use suzunari_error::suzunari_error;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// Error for [`OutputLength::try_new`].
#[suzunari_error]
#[suzu(visibility(pub(crate)), context(suffix(Error)))]
pub enum OutputLengthError {
    /// Length is zero.
    #[suzu(display("output length must be greater than 0"))]
    Zero,
}

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
        ensure!(length > 0, ZeroError);
        Ok(Self(length))
    }

    /// Returns the length in bytes.
    pub const fn get(&self) -> usize {
        self.0
    }
}

/// Derived key material.
///
/// # Security
///
/// Automatically zeroized on drop ([`ZeroizeOnDrop`]).
/// Equality comparison is constant-time to prevent timing attacks.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct DerivedKey(Vec<u8>);

impl Debug for DerivedKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("DerivedKey").field(&"***").finish()
    }
}

impl PartialEq for DerivedKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl Eq for DerivedKey {}

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

/// Error for [`Pbkdf2Params::try_new`].
#[suzunari_error]
#[suzu(visibility(pub(crate)), context(suffix(Error)))]
pub enum Pbkdf2ParamsError {
    /// `iterations` is zero.
    #[suzu(display("iterations must be greater than 0"))]
    IterationsZero,
}

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
        ensure!(iterations > 0, IterationsZeroError);
        Ok(Self { iterations })
    }

    /// Returns the iteration count.
    pub const fn iterations(&self) -> u32 {
        self.iterations
    }
}

/// Error for [`Argon2Params::try_new`].
#[suzunari_error]
#[suzu(visibility(pub(crate)), context(suffix(Error)))]
pub enum Argon2ParamsError {
    /// `memory_kib` is zero.
    #[suzu(display("memory_kib must be greater than 0"))]
    MemoryKibZero,
    /// `time` is zero.
    #[suzu(display("time must be greater than 0"))]
    TimeZero,
    /// `parallelism` is zero.
    #[suzu(display("parallelism must be greater than 0"))]
    ParallelismZero,
}

/// Argon2id parameters.
///
/// Note: the upstream `argon2` crate requires `memory_kib >= 8 * parallelism`.
/// Values violating this constraint will cause
/// [`KdfError::Argon2InvalidParams`](crate::KdfError::Argon2InvalidParams) at derive time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argon2Params {
    /// Memory cost in KiB.
    ///
    /// OWASP 2023 recommendation: 65,536 (64 MiB). Minimum: 19,456 (19 MiB).
    memory_kib: u32,

    /// Time cost (number of passes).
    ///
    /// OWASP 2023 recommendation: 3.
    time: u32,

    /// Parallelism (thread count).
    ///
    /// OWASP 2023 recommendation: 4.
    parallelism: u32,
}

impl Argon2Params {
    /// OWASP 2023 recommended values (memory=64MiB, time=3, parallelism=4).
    pub const OWASP_RECOMMENDED: Self = Self { memory_kib: 65536, time: 3, parallelism: 4 };

    /// Creates parameters.
    ///
    /// # Panics
    ///
    /// Panics if any parameter is 0.
    pub const fn new(memory_kib: u32, time: u32, parallelism: u32) -> Self {
        assert!(memory_kib > 0, "memory_kib must be greater than 0");
        assert!(time > 0, "time must be greater than 0");
        assert!(parallelism > 0, "parallelism must be greater than 0");
        Self { memory_kib, time, parallelism }
    }

    /// Returns the memory cost in KiB.
    pub const fn memory_kib(&self) -> u32 {
        self.memory_kib
    }

    /// Returns the time cost (number of passes).
    pub const fn time(&self) -> u32 {
        self.time
    }

    /// Returns the parallelism.
    pub const fn parallelism(&self) -> u32 {
        self.parallelism
    }

    /// Creates parameters with validation. For runtime input.
    pub fn try_new(
        memory_kib: u32,
        time: u32,
        parallelism: u32,
    ) -> Result<Self, Argon2ParamsError> {
        ensure!(memory_kib > 0, MemoryKibZeroError);
        ensure!(time > 0, TimeZeroError);
        ensure!(parallelism > 0, ParallelismZeroError);
        Ok(Self { memory_kib, time, parallelism })
    }
}

/// Error for [`ScryptParams::try_new`].
#[suzunari_error]
#[suzu(visibility(pub(crate)), context(suffix(Error)))]
pub enum ScryptParamsError {
    /// `r` is zero.
    #[suzu(display("r must be greater than 0"))]
    RZero,
    /// `p` is zero.
    #[suzu(display("p must be greater than 0"))]
    PZero,
}

/// Scrypt parameters.
///
/// Note: the upstream `scrypt` crate requires `r * p < 2^30`.
/// Values violating this constraint will cause
/// [`KdfError::ScryptInvalidParams`](crate::KdfError::ScryptInvalidParams) at derive time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScryptParams {
    /// Logarithm of CPU/memory cost parameter N (N = 2^log_n).
    ///
    /// Colin Percival recommendation: 17 (N = 131072).
    log_n: u8,

    /// Block size parameter. Recommended: 8.
    r: u32,

    /// Parallelization parameter. Recommended: 1.
    p: u32,
}

impl ScryptParams {
    /// Colin Percival recommended values (log_n=17, r=8, p=1).
    pub const RECOMMENDED: Self = Self { log_n: 17, r: 8, p: 1 };

    /// Creates parameters.
    ///
    /// # Panics
    ///
    /// Panics if `r` or `p` is 0.
    /// `log_n` of 0 is valid (N=1, for testing).
    pub const fn new(log_n: u8, r: u32, p: u32) -> Self {
        assert!(r > 0, "r must be greater than 0");
        assert!(p > 0, "p must be greater than 0");
        Self { log_n, r, p }
    }

    /// Returns log_n.
    pub const fn log_n(&self) -> u8 {
        self.log_n
    }

    /// Returns the block size r.
    pub const fn r(&self) -> u32 {
        self.r
    }

    /// Returns the parallelization p.
    pub const fn p(&self) -> u32 {
        self.p
    }

    /// Creates parameters with validation. For runtime input.
    pub fn try_new(log_n: u8, r: u32, p: u32) -> Result<Self, ScryptParamsError> {
        ensure!(r > 0, RZeroError);
        ensure!(p > 0, PZeroError);
        Ok(Self { log_n, r, p })
    }
}

/// Error for [`BcryptPbkdfParams::try_new`].
#[suzunari_error]
#[suzu(visibility(pub(crate)), context(suffix(Error)))]
pub enum BcryptPbkdfParamsError {
    /// `rounds` is zero.
    #[suzu(display("rounds must be greater than 0"))]
    RoundsZero,
}

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
        ensure!(rounds > 0, RoundsZeroError);
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
        assert!(matches!(OutputLength::try_new(0), Err(OutputLengthError::Zero { .. })));
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
    fn test_argon2_params_owasp_recommended() {
        let p = Argon2Params::OWASP_RECOMMENDED;
        assert_eq!(p.memory_kib(), 65536);
        assert_eq!(p.time(), 3);
        assert_eq!(p.parallelism(), 4);
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
        let p = ScryptParams::RECOMMENDED;
        assert_eq!(p.log_n(), 17);
        assert_eq!(p.r(), 8);
        assert_eq!(p.p(), 1);
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
        assert!(matches!(
            Pbkdf2Params::try_new(0),
            Err(Pbkdf2ParamsError::IterationsZero { .. })
        ));
    }

    #[test]
    fn test_argon2_params_try_new_ok() {
        let p = Argon2Params::try_new(8, 1, 1).unwrap();
        assert_eq!(p.memory_kib(), 8);
        assert_eq!(p.time(), 1);
        assert_eq!(p.parallelism(), 1);
    }

    #[test]
    fn test_argon2_params_try_new_zero_memory() {
        assert!(matches!(
            Argon2Params::try_new(0, 1, 1),
            Err(Argon2ParamsError::MemoryKibZero { .. })
        ));
    }

    #[test]
    fn test_argon2_params_try_new_zero_time() {
        assert!(matches!(
            Argon2Params::try_new(8, 0, 1),
            Err(Argon2ParamsError::TimeZero { .. })
        ));
    }

    #[test]
    fn test_argon2_params_try_new_zero_parallelism() {
        assert!(matches!(
            Argon2Params::try_new(8, 1, 0),
            Err(Argon2ParamsError::ParallelismZero { .. })
        ));
    }

    #[test]
    fn test_scrypt_params_try_new_ok() {
        let p = ScryptParams::try_new(4, 1, 1).unwrap();
        assert_eq!(p.log_n(), 4);
        assert_eq!(p.r(), 1);
        assert_eq!(p.p(), 1);
    }

    #[test]
    fn test_scrypt_params_try_new_zero_r() {
        assert!(matches!(ScryptParams::try_new(4, 0, 1), Err(ScryptParamsError::RZero { .. })));
    }

    #[test]
    fn test_scrypt_params_try_new_zero_p() {
        assert!(matches!(ScryptParams::try_new(4, 1, 0), Err(ScryptParamsError::PZero { .. })));
    }

    #[test]
    fn test_bcrypt_pbkdf_params_try_new_ok() {
        let p = BcryptPbkdfParams::try_new(16).unwrap();
        assert_eq!(p.rounds(), 16);
    }

    #[test]
    fn test_bcrypt_pbkdf_params_try_new_zero() {
        assert!(matches!(
            BcryptPbkdfParams::try_new(0),
            Err(BcryptPbkdfParamsError::RoundsZero { .. })
        ));
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
            Argon2Params::try_new(0, 1, 1).unwrap_err().to_string(),
            "memory_kib must be greater than 0"
        );
        assert_eq!(
            Argon2Params::try_new(8, 0, 1).unwrap_err().to_string(),
            "time must be greater than 0"
        );
        assert_eq!(
            Argon2Params::try_new(8, 1, 0).unwrap_err().to_string(),
            "parallelism must be greater than 0"
        );
    }

    #[test]
    fn test_scrypt_params_error_display() {
        assert_eq!(
            ScryptParams::try_new(4, 0, 1).unwrap_err().to_string(),
            "r must be greater than 0"
        );
        assert_eq!(
            ScryptParams::try_new(4, 1, 0).unwrap_err().to_string(),
            "p must be greater than 0"
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
    #[should_panic(expected = "memory_kib must be greater than 0")]
    fn test_argon2_params_new_zero_memory_panics() {
        Argon2Params::new(0, 1, 1);
    }

    #[test]
    #[should_panic(expected = "time must be greater than 0")]
    fn test_argon2_params_new_zero_time_panics() {
        Argon2Params::new(8, 0, 1);
    }

    #[test]
    #[should_panic(expected = "parallelism must be greater than 0")]
    fn test_argon2_params_new_zero_parallelism_panics() {
        Argon2Params::new(8, 1, 0);
    }

    #[test]
    #[should_panic(expected = "r must be greater than 0")]
    fn test_scrypt_params_new_zero_r_panics() {
        ScryptParams::new(4, 0, 1);
    }

    #[test]
    #[should_panic(expected = "p must be greater than 0")]
    fn test_scrypt_params_new_zero_p_panics() {
        ScryptParams::new(4, 1, 0);
    }

    #[test]
    #[should_panic(expected = "rounds must be greater than 0")]
    fn test_bcrypt_pbkdf_params_new_zero_panics() {
        BcryptPbkdfParams::new(0);
    }
}
