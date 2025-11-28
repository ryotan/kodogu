use super::{PrivateKey, PublicKey};
use crate::domain::algorithms::AsymmetricAlgorithm;

/// Key pair aggregate root
#[derive(Debug, Clone)]
pub struct KeyPair {
    algorithm: AsymmetricAlgorithm,
    public_key: PublicKey,
    private_key: PrivateKey,
}

impl KeyPair {
    pub fn new(
        algorithm: AsymmetricAlgorithm,
        public_key: PublicKey,
        private_key: PrivateKey,
    ) -> Self {
        Self { algorithm, public_key, private_key }
    }

    pub fn algorithm(&self) -> AsymmetricAlgorithm {
        self.algorithm
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn private_key(&self) -> &PrivateKey {
        &self.private_key
    }

    pub fn into_parts(self) -> (AsymmetricAlgorithm, PublicKey, PrivateKey) {
        (self.algorithm, self.public_key, self.private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::algorithms::{EcdsaCurve, RsaKeySize};

    #[test]
    fn new_creates_key_pair() {
        let algorithm = AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048);
        let public = PublicKey::new(vec![1, 2, 3]);
        let private = PrivateKey::new(vec![4, 5, 6]);

        let key_pair = KeyPair::new(algorithm, public.clone(), private.clone());

        assert_eq!(key_pair.algorithm(), algorithm);
        assert_eq!(key_pair.public_key(), &public);
        assert_eq!(key_pair.private_key(), &private);
    }

    #[test]
    fn algorithm_returns_correct_algorithm() {
        let rsa_pair = KeyPair::new(
            AsymmetricAlgorithm::Rsa(RsaKeySize::Bits4096),
            PublicKey::new(vec![1]),
            PrivateKey::new(vec![2]),
        );
        assert_eq!(
            rsa_pair.algorithm(),
            AsymmetricAlgorithm::Rsa(RsaKeySize::Bits4096)
        );

        let ecdsa_pair = KeyPair::new(
            AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P256),
            PublicKey::new(vec![3]),
            PrivateKey::new(vec![4]),
        );
        assert_eq!(
            ecdsa_pair.algorithm(),
            AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P256)
        );
    }

    #[test]
    fn public_key_returns_reference() {
        let public = PublicKey::new(vec![10, 20, 30]);
        let key_pair = KeyPair::new(
            AsymmetricAlgorithm::Ed25519,
            public.clone(),
            PrivateKey::new(vec![40, 50, 60]),
        );
        assert_eq!(key_pair.public_key(), &public);
    }

    #[test]
    fn private_key_returns_reference() {
        let private = PrivateKey::new(vec![100, 200]);
        let key_pair = KeyPair::new(
            AsymmetricAlgorithm::Ed25519,
            PublicKey::new(vec![1, 2]),
            private.clone(),
        );
        assert_eq!(key_pair.private_key(), &private);
    }

    #[test]
    fn into_parts_decomposes_key_pair() {
        let algorithm = AsymmetricAlgorithm::Ecdsa(EcdsaCurve::P384);
        let public = PublicKey::new(vec![1, 2, 3]);
        let private = PrivateKey::new(vec![4, 5, 6]);

        let key_pair = KeyPair::new(algorithm, public.clone(), private.clone());
        let (alg, pub_key, priv_key) = key_pair.into_parts();

        assert_eq!(alg, algorithm);
        assert_eq!(pub_key, public);
        assert_eq!(priv_key, private);
    }

    #[test]
    fn clone_creates_independent_copy() {
        let key_pair1 = KeyPair::new(
            AsymmetricAlgorithm::Rsa(RsaKeySize::Bits2048),
            PublicKey::new(vec![1, 2]),
            PrivateKey::new(vec![3, 4]),
        );
        let key_pair2 = key_pair1.clone();

        assert_eq!(key_pair1.algorithm(), key_pair2.algorithm());
        assert_eq!(key_pair1.public_key(), key_pair2.public_key());
        assert_eq!(key_pair1.private_key(), key_pair2.private_key());
    }
}
