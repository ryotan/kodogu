use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use snafu::ResultExt;

use super::errors::Base64DecodingSnafu;
use crate::domain::{CryptoError, ciphertext::Ciphertext};

/// Serialize ciphertext to Base64
pub fn to_base64(ciphertext: &Ciphertext) -> String {
    BASE64.encode(ciphertext.as_bytes())
}

/// Deserialize ciphertext from Base64
pub fn from_base64(encoded: &str) -> Result<Ciphertext, CryptoError> {
    let bytes = BASE64.decode(encoded).context(Base64DecodingSnafu)?;
    Ok(Ciphertext::new(bytes))
}
