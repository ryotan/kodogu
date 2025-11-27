/// Symmetric cryptography algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetricAlgorithm {
    Aes128,
    Aes256,
}

impl SymmetricAlgorithm {
    pub fn key_size_bytes(&self) -> usize {
        match self {
            Self::Aes128 => 16,
            Self::Aes256 => 32,
        }
    }
}
