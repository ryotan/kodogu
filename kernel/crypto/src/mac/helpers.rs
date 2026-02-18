use super::errors::{InvalidKeyLengthError, MacAlgorithms, MacError};
use snafu::ResultExt;

/// 鍵長を検証し、固定サイズ配列に変換する。
pub(crate) fn validate_key_length<const N: usize>(
    key: &[u8],
    algorithm: MacAlgorithms,
) -> Result<[u8; N], MacError> {
    key.try_into()
        .context(InvalidKeyLengthError { algorithm, expected: N, actual: key.len() })
}
