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
