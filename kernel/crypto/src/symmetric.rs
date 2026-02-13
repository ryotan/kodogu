//! # Symmetric ciphers
//!
//! ## Available Variants
//!
//! ### Block Ciphers (use [`InitializationVector`](crate::InitializationVector))
//!
//! - **AES-128-CBC**: 128-bit key, CBC mode, PKCS#7 padding
//! - **AES-256-CBC**: 256-bit key, CBC mode, PKCS#7 padding
//!
//! ### Stream Ciphers (use [`Nonce`](crate::Nonce))
//!
//! - **AES-128-CTR**: 128-bit key, CTR mode
//! - **AES-256-CTR**: 256-bit key, CTR mode
//! - **ChaCha20**: 256-bit key, 96-bit nonce
//!
//! ## Design Patterns
//!
//! 1. **trait**:
//!    - Block ciphers: [`BlockEncryptor`], [`BlockDecryptor`]
//!    - Stream ciphers: [`StreamEncryptor`], [`StreamDecryptor`]
//! 2. **ZST types** (compile-time dispatch): `Aes128Cbc`, `Aes256Cbc`, `Aes128Ctr`, `Aes256Ctr`, `ChaCha20`
//! 3. **Enum** (runtime selection): `AesCbc`, `AesCtr`
//! 4. **Cipher-specific trait** (polymorphism): `AesCbcCipher`, `AesCtrCipher`, `ChaCha20Cipher`
//!
//! ## Example (Block Cipher)
//!
//! ```
//! use crypto::{Aes128Cbc, BlockEncryptor, BlockDecryptor, Plaintext, InitializationVector};
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
//!
//! ## Example (Stream Cipher)
//!
//! ```
//! use crypto::{Aes128Ctr, StreamEncryptor, StreamDecryptor, Plaintext, Nonce};
//!
//! let cipher = Aes128Ctr;
//! let key = [0u8; 16];
//! let nonce = Nonce::from(vec![0u8; 16]);
//! let plaintext = Plaintext::from("Hello, World!");
//!
//! let ciphertext = cipher.encrypt(&plaintext, &key, &nonce).unwrap();
//! let decrypted = cipher.decrypt(&ciphertext, &key, &nonce).unwrap();
//! assert_eq!(plaintext, decrypted);
//! ```

mod aes_cbc;
mod aes_ctr;
mod chacha20;
mod errors;
mod helpers;
mod traits;

pub use aes_cbc::{Aes128Cbc, Aes256Cbc, AesCbc, AesCbcCipher};
pub use aes_ctr::{Aes128Ctr, Aes256Ctr, AesCtr, AesCtrCipher};
pub use chacha20::{ChaCha20, ChaCha20Cipher};
pub use errors::SymmetricCipherError;
pub use traits::{
    BlockCipher, BlockDecryptor, BlockEncryptor, StreamCipher, StreamDecryptor, StreamEncryptor,
};
