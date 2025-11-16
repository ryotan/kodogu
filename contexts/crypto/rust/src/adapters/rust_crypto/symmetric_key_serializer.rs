use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use snafu::{ResultExt, ensure};

use super::errors::{Base64DecodingSnafu, InvalidKeySizeSnafu};
use crate::domain::{CryptoError, keys::SymmetricKey};

/// Serialize symmetric key to Base64
pub fn to_base64(key: &SymmetricKey) -> String {
    BASE64.encode(key.as_bytes())
}

/// Deserialize symmetric key from Base64
pub fn from_base64(encoded: &str) -> Result<SymmetricKey, CryptoError> {
    let bytes = BASE64.decode(encoded).context(Base64DecodingSnafu)?;

    // Validate key size (AES-128: 16 bytes, AES-256: 32 bytes)
    ensure!(
        bytes.len() == 16 || bytes.len() == 32,
        InvalidKeySizeSnafu { size: bytes.len() }
    );

    Ok(SymmetricKey::new(bytes))
}
