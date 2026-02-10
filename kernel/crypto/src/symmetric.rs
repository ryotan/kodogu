//! # Symmetric ciphers
//!
//! ## Available Variants
//!
//! - **AES-128-CBC**: 128-bit key, CBC mode, PKCS#7 padding
//! - **AES-256-CBC**: 256-bit key, CBC mode, PKCS#7 padding
//! - **AES-128-CTR**: 128-bit key, CTR mode (stream cipher)
//! - **AES-256-CTR**: 256-bit key, CTR mode (stream cipher)
//! - **ChaCha20**: 256-bit key, 96-bit nonce (stream cipher)
//!
//! ## Design Patterns
//!
//! 1. **trait**: `SymmetricEncryptor`, `SymmetricDecryptor`
//! 2. **ZST types** (compile-time dispatch): `Aes128Cbc`, `Aes256Cbc`, `Aes128Ctr`, `Aes256Ctr`, `ChaCha20`
//! 3. **Enum** (runtime selection): `AesCbc`, `AesCtr`
//! 4. **Cipher-specific trait** (polymorphism): `AesCbcCipher`, `AesCtrCipher`, `ChaCha20Cipher`
//!
//! ## Example
//!
//! ```
//! use crypto::{Aes128Cbc, SymmetricEncryptor, SymmetricDecryptor, Plaintext, InitializationVector};
//!
//! let cipher = Aes128Cbc;
//! let key = [0u8; 16];
//! let iv = InitializationVector::from(vec![0u8; 16]);
//! let plaintext = Plaintext::from("Hello, World!");
//!
//! let ciphertext = cipher.encrypt(&plaintext, &key, &iv).unwrap();
//! let decrypted = cipher.decrypt(&ciphertext, &key, &iv).unwrap();
//! assert_eq!(plaintext, decrypted);
//! ```

mod aes_cbc;
mod aes_ctr;
mod chacha20;
mod traits;

pub use aes_cbc::{Aes128Cbc, Aes256Cbc, AesCbc, AesCbcCipher};
pub use aes_ctr::{Aes128Ctr, Aes256Ctr, AesCtr, AesCtrCipher};
pub use chacha20::{ChaCha20, ChaCha20Cipher};
pub use traits::{SymmetricCipher, SymmetricCipherError, SymmetricDecryptor, SymmetricEncryptor};
