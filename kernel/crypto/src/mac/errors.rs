use hmac::digest::InvalidLength;
use std::array::TryFromSliceError;
use suzunari_error::suzunari_error;

/// MAC algorithm identifier for error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacAlgorithms {
    HmacSha256,
    HmacSha384,
    HmacSha512,
    CmacAes128,
    CmacAes256,
    Gmac,
    Poly1305,
}

/// MAC operation error.
#[suzunari_error]
#[suzu(visibility(pub), context(suffix(Error)))]
pub enum MacError {
    /// Invalid key length
    #[suzu(display(
        "{algorithm:?}: invalid key length {actual} bytes (expected {expected} bytes)"
    ))]
    InvalidKeyLength {
        algorithm: MacAlgorithms,
        expected: usize,
        actual: usize,
        source: TryFromSliceError,
    },

    /// Invalid nonce length (for GMAC)
    #[suzu(display(
        "{algorithm:?}: invalid nonce length {actual} bytes (expected {expected} bytes)"
    ))]
    InvalidNonceLength {
        algorithm: MacAlgorithms,
        expected: usize,
        actual: usize,
        source: TryFromSliceError,
    },

    /// Verification failed
    ///
    /// 検証失敗時のエラー。タイミング攻撃を防ぐため、
    /// 失敗理由の詳細は意図的に提供しない。
    #[suzu(display("{algorithm:?}: verification failed"))]
    VerificationFailed { algorithm: MacAlgorithms, source: hmac::digest::MacError },

    /// Invalid tag length
    #[suzu(display(
        "{algorithm:?}: invalid tag length {actual} bytes (expected {expected} bytes)"
    ))]
    InvalidTagLength { algorithm: MacAlgorithms, expected: usize, actual: usize },

    /// MAC initialization failed
    #[suzu(display("{algorithm:?}: key initialization failed"))]
    KeyInitFailed { algorithm: MacAlgorithms, source: InvalidLength },
}
