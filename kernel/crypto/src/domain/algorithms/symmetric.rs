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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes128_key_size_is_16_bytes() {
        assert_eq!(SymmetricAlgorithm::Aes128.key_size_bytes(), 16);
    }

    #[test]
    fn aes256_key_size_is_32_bytes() {
        assert_eq!(SymmetricAlgorithm::Aes256.key_size_bytes(), 32);
    }

    #[test]
    fn clone_copies_algorithm() {
        let alg1 = SymmetricAlgorithm::Aes256;
        let alg2 = alg1;
        assert_eq!(alg1, alg2);
    }

    #[test]
    fn equality_compares_variants() {
        assert_eq!(SymmetricAlgorithm::Aes128, SymmetricAlgorithm::Aes128);
        assert_eq!(SymmetricAlgorithm::Aes256, SymmetricAlgorithm::Aes256);
        assert_ne!(SymmetricAlgorithm::Aes128, SymmetricAlgorithm::Aes256);
    }
}
