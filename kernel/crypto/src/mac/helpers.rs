use super::errors::{InvalidKeyLengthError, MacAlgorithms, MacError};
use suzunari_error::ResultExt;

/// Checks the key length and converts the key to a fixed-size array.
pub(crate) fn validate_key_length<const N: usize>(
    key: &[u8],
    algorithm: MacAlgorithms,
) -> Result<[u8; N], MacError> {
    key.try_into()
        .context(InvalidKeyLengthError { algorithm, expected: N, actual: key.len() })
}
