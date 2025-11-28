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

#[cfg(test)]
mod tests {
    use super::*;

    mod rsa_key_size {
        use super::*;

        #[test]
        fn bits2048_returns_2048() {
            assert_eq!(RsaKeySize::Bits2048.bits(), 2048);
        }

        #[test]
        fn bits4096_returns_4096() {
            assert_eq!(RsaKeySize::Bits4096.bits(), 4096);
        }

        #[test]
        fn equality_compares_variants() {
            assert_eq!(RsaKeySize::Bits2048, RsaKeySize::Bits2048);
            assert_eq!(RsaKeySize::Bits4096, RsaKeySize::Bits4096);
            assert_ne!(RsaKeySize::Bits2048, RsaKeySize::Bits4096);
        }
    }

    mod ecdsa_curve {
        use super::*;

        #[test]
        fn equality_compares_variants() {
            assert_eq!(EcdsaCurve::P256, EcdsaCurve::P256);
            assert_eq!(EcdsaCurve::P384, EcdsaCurve::P384);
            assert_ne!(EcdsaCurve::P256, EcdsaCurve::P384);
        }
    }

    mod asymmetric_algorithm {
        use super::*;

        #[test]
        fn rsa_variants_are_distinct() {
            let rsa2048 = AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048);
            let rsa4096 = AsymmetricAlgorithm::Rsa(RsaKeySize::Bits4096);
            assert_ne!(rsa2048, rsa4096);
        }

        #[test]
        fn ecdsa_variants_are_distinct() {
            let p256 = AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P256);
            let p384 = AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P384);
            assert_ne!(p256, p384);
        }

        #[test]
        fn ed25519_is_unique_variant() {
            let ed25519_1 = AsymmetricAlgorithm::Ed25519;
            let ed25519_2 = AsymmetricAlgorithm::Ed25519;
            assert_eq!(ed25519_1, ed25519_2);
        }

        #[test]
        fn different_algorithm_types_are_not_equal() {
            let rsa = AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048);
            let ecdsa = AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P256);
            let ed25519 = AsymmetricAlgorithm::Ed25519;

            assert_ne!(rsa, ecdsa);
            assert_ne!(rsa, ed25519);
            assert_ne!(ecdsa, ed25519);
        }

        #[test]
        fn clone_copies_algorithm() {
            let alg1 = AsymmetricAlgorithm::Rsa(RsaKeySize::Bits4096);
            let alg2 = alg1;
            assert_eq!(alg1, alg2);
        }
    }
}
