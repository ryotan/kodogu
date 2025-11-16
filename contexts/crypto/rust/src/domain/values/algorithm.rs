/// Asymmetric cryptography algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsymmetricAlgorithm {
    Rsa(RsaKeySize),
    Ecdsa(EcdsaCurve),
    Ed25519,
}

/// RSA key sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsaKeySize {
    Bits2048,
    Bits4096,
}

impl RsaKeySize {
    pub fn bits(&self) -> usize {
        match self {
            Self::Bits2048 => 2048,
            Self::Bits4096 => 4096,
        }
    }
}

/// ECDSA curves
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcdsaCurve {
    P256,
    P384,
}

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
