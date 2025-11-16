/// Ciphertext value object (encrypted data)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(Vec<u8>);

impl Ciphertext {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub fn to_base64(&self) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &self.0)
    }

    pub fn from_base64(s: &str) -> Result<Self, base64::DecodeError> {
        let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)?;
        Ok(Self(bytes))
    }
}
