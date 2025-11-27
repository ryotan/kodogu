//! Base58 encoding and decoding.
//!
//! This module provides Base58 encoding/decoding with multiple alphabet variants.
//! Base58 is designed to avoid visually ambiguous characters (0, O, I, l) and is
//! commonly used for encoding cryptocurrency addresses and other identifiers.
//!
//! ## Available Variants
//!
//! - **Bitcoin (default)**: Uses alphabet `123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz`
//! - **Flickr**: Uses alphabet `123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ` (lowercase first)
//! - **Custom**: Define your own 58-character alphabet
//!
//! ## Choosing a Variant
//!
//! - **Bitcoin**: Default choice for cryptocurrency addresses, most widely used
//! - **Flickr**: When you need lowercase-first encoding (less common)
//! - **Custom**: When you need compatibility with legacy systems using non-standard alphabets
//!
//! ## Examples
//!
//! ```
//! use codec::{Base58, BinaryCodec};
//!
//! // Using the default Bitcoin variant
//! let encoded = Base58::encode(b"Hello");
//! assert_eq!(encoded, "9Ajdvzr");
//!
//! // Using specific variants
//! assert_eq!(Base58::encode_bitcoin(b"Hello"), "9Ajdvzr");
//! assert_eq!(Base58::encode_flickr(b"Hello"), "9aJCVZR");
//!
//! // Using codec instances
//! let codec = Base58::Bitcoind;
//! let data = codec.encode(b"Hello");
//! assert_eq!(codec.decode(&data).unwrap(), b"Hello");
//! ```
//!
//! ## References
//!
//! - [Base58 Encoding (Bitcoin Wiki)](https://en.bitcoin.it/wiki/Base58Check_encoding)
//! - [Flickr Base58 Alphabet](https://www.flickr.com/groups/api/discuss/72157616713786392/)

use crate::{
    Base58AlphabetError, Base58DecodeError, BinaryCodec, BinaryCodecError, BinaryCodecs,
    InvalidCharacterSetError,
};
use bs58::Alphabet;
use snafu::{ResultExt, ensure};

/// Base58 codec using Bitcoin alphabet.
///
/// Uses alphabet: `123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz`
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// The Bitcoin alphabet excludes visually confusing characters: 0 (zero), O (capital o), I (capital i), and l (lowercase L).
///
/// # Examples
///
/// ```
/// use codec::{Base58Bitcoin, BinaryCodec};
///
/// let codec = Base58Bitcoin;
///
/// // Encode
/// assert_eq!(codec.encode(b"Hello"), "9Ajdvzr");
///
/// // Decode
/// assert_eq!(codec.decode("9Ajdvzr").unwrap(), b"Hello");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base58Bitcoin;

/// Base58 codec using Flickr alphabet.
///
/// Uses alphabet: `123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ`
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// The Flickr alphabet is similar to Bitcoin but uses lowercase letters first, then uppercase.
///
/// # Examples
///
/// ```
/// use codec::{Base58Flickr, BinaryCodec};
///
/// let codec = Base58Flickr;
///
/// // Encode
/// assert_eq!(codec.encode(b"Hello"), "9aJCVZR");
///
/// // Decode
/// assert_eq!(codec.decode("9aJCVZR").unwrap(), b"Hello");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base58Flickr;

/// Base58 codec with a custom alphabet.
///
/// Allows encoding/decoding with a user-defined 58-character alphabet.
/// Use [`Base58Custom::new`] to create an instance with a custom alphabet.
///
/// # Examples
///
/// ```
/// use codec::{Base58Custom, BinaryCodec};
///
/// // Ripple alphabet (well-known Base58 variant)
/// let alphabet = "rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";
/// let codec = Base58Custom::new(alphabet).unwrap();
///
/// let encoded = codec.encode(b"Hello");
/// let decoded = codec.decode(&encoded).unwrap();
/// assert_eq!(decoded, b"Hello");
/// ```
#[derive(Debug, Clone)]
pub struct Base58Custom {
    alphabet: Alphabet,
}

/// Base58 encoding variants.
///
/// Enum wrapper providing convenient access to Bitcoin and Flickr alphabets,
/// as well as custom alphabet support. The default variant is [`Base58::Bitcoind`].
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, Base58};
///
/// // Using enum variants
/// let bitcoin = Base58::Bitcoind;
/// let flickr = Base58::Flickr;
///
/// // Different alphabets produce different encodings
/// assert_eq!(bitcoin.encode(b"Hello"), "9Ajdvzr");
/// assert_eq!(flickr.encode(b"Hello"), "9aJCVZR");
/// ```
#[derive(Default, Debug, Clone)]
pub enum Base58 {
    /// Bitcoin alphabet (default) - uppercase letters first
    #[default]
    Bitcoind,
    /// Flickr alphabet - lowercase letters first
    Flickr,
    /// Custom alphabet
    Custom(Base58Custom),
}

/// Marker trait for Base58-family codecs.
///
/// This trait is automatically implemented for all Base58 codec types
/// ([`Base58Bitcoin`], [`Base58Flickr`], [`Base58Custom`], [`Base58`]).
///
/// Users can also implement this trait for their own custom Base58 implementations
/// to integrate with functions that accept `impl Base58Codec`.
///
/// # Examples
///
/// ```
/// use codec::{Base58Codec, Base58Bitcoin, Base58};
///
/// fn process_base58(codec: &impl Base58Codec, data: &[u8]) -> String {
///     codec.encode(data)
/// }
///
/// // Works with ZST types
/// assert_eq!(process_base58(&Base58Bitcoin, b"Hi"), "6Wc");
///
/// // Works with Enum
/// assert_eq!(process_base58(&Base58::Bitcoind, b"Hi"), "6Wc");
/// ```
pub trait Base58Codec: BinaryCodec {}

fn encode_internal(bytes: &[u8], alphabet: &Alphabet) -> String {
    bs58::encode(bytes).with_alphabet(alphabet).into_string()
}

fn decode_internal(string: &str, alphabet: &Alphabet) -> Result<Vec<u8>, BinaryCodecError> {
    bs58::decode(string.as_bytes())
        .with_alphabet(alphabet)
        .into_vec()
        .context(Base58DecodeError)
}

impl Base58Codec for Base58Bitcoin {}
impl BinaryCodec for Base58Bitcoin {
    fn encode(&self, bytes: &[u8]) -> String {
        encode_internal(bytes, Alphabet::BITCOIN)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(text, Alphabet::BITCOIN)
    }
}

impl Base58Codec for Base58Flickr {}
impl BinaryCodec for Base58Flickr {
    fn encode(&self, bytes: &[u8]) -> String {
        encode_internal(bytes, Alphabet::FLICKR)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(text, Alphabet::FLICKR)
    }
}

impl Base58Codec for Base58Custom {}
impl BinaryCodec for Base58Custom {
    fn encode(&self, bytes: &[u8]) -> String {
        encode_internal(bytes, &self.alphabet)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(text, &self.alphabet)
    }
}
impl Base58Custom {
    /// Create a Base58 codec with a custom 58-character alphabet.
    ///
    /// # Arguments
    /// * `alphabet` - A 58-character string containing unique ASCII characters
    ///
    /// # Errors
    /// Returns [`BinaryCodecError`] if:
    /// - Alphabet length is not exactly 58 characters
    /// - Alphabet contains duplicate characters
    /// - Alphabet contains non-ASCII characters
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::{Base58Custom, BinaryCodec};
    ///
    /// // Ripple alphabet (well-known Base58 variant)
    /// let alphabet = "rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";
    /// let codec = Base58Custom::new(alphabet).unwrap();
    ///
    /// let encoded = codec.encode(b"Hello");
    /// let decoded = codec.decode(&encoded).unwrap();
    /// assert_eq!(decoded, b"Hello");
    /// ```
    pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> {
        let bytes = alphabet.as_bytes();

        // Validate length
        ensure!(
            bytes.len() == 58,
            InvalidCharacterSetError {
                encoding: BinaryCodecs::Base58,
                message: format!("Base58 alphabet must be 58 characters, got {}", bytes.len()),
            }
        );

        // Validate ASCII
        ensure!(
            alphabet.is_ascii(),
            InvalidCharacterSetError {
                encoding: BinaryCodecs::Base58,
                message: "Base58 alphabet must contain only ASCII characters",
            }
        );

        // Convert to fixed-size array
        let mut alphabet_array = [0u8; 58];
        alphabet_array.copy_from_slice(bytes);

        let alphabet = Alphabet::new(&alphabet_array).context(Base58AlphabetError)?;

        Ok(Self { alphabet })
    }
}

impl Base58 {
    /// Encodes bytes to Base58 string (Bitcoin alphabet variant).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base58;
    ///
    /// assert_eq!(Base58::encode(b"Hello"), "9Ajdvzr");
    /// ```
    pub fn encode(bytes: &[u8]) -> String {
        Base58Bitcoin.encode(bytes)
    }

    /// Encodes bytes to Base58 string using Bitcoin alphabet.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base58;
    ///
    /// assert_eq!(Base58::encode_bitcoin(b"Hello"), "9Ajdvzr");
    /// ```
    pub fn encode_bitcoin(bytes: &[u8]) -> String {
        Base58Bitcoin.encode(bytes)
    }

    /// Encodes bytes to Base58 string using Flickr alphabet.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base58;
    ///
    /// assert_eq!(Base58::encode_flickr(b"Hello"), "9aJCVZR");
    /// ```
    pub fn encode_flickr(bytes: &[u8]) -> String {
        Base58Flickr.encode(bytes)
    }

    /// Decodes Base58 string to bytes (Bitcoin alphabet variant).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base58;
    ///
    /// assert_eq!(Base58::decode("9Ajdvzr").unwrap(), b"Hello");
    /// ```
    pub fn decode(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base58Bitcoin.decode(text)
    }

    /// Decodes Base58 string to bytes using Bitcoin alphabet.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base58;
    ///
    /// assert_eq!(Base58::decode_bitcoin("9Ajdvzr").unwrap(), b"Hello");
    /// ```
    pub fn decode_bitcoin(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base58Bitcoin.decode(text)
    }

    /// Decodes Base58 string using Flickr alphabet.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base58;
    ///
    /// assert_eq!(Base58::decode_flickr("9aJCVZR").unwrap(), b"Hello");
    /// ```
    pub fn decode_flickr(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base58Flickr.decode(text)
    }

    /// Create a Base58 codec with custom alphabet (58 unique ASCII characters)
    ///
    /// # Arguments
    /// * `alphabet` - 58-character alphabet string
    ///
    /// # Errors
    /// Returns error if alphabet is invalid (wrong length, duplicate characters, non-ASCII, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base58;
    ///
    /// // Custom alphabet (Ripple)
    /// let alphabet = "rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";
    /// let codec = Base58::new(alphabet).unwrap();
    /// ```
    pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> {
        Base58Custom::new(alphabet).map(Base58::Custom)
    }
}

impl Base58Codec for Base58 {}
impl BinaryCodec for Base58 {
    fn encode(&self, bytes: &[u8]) -> String {
        match self {
            Base58::Bitcoind => Base58Bitcoin.encode(bytes),
            Base58::Flickr => Base58Flickr.encode(bytes),
            Base58::Custom(custom) => custom.encode(bytes),
        }
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        match self {
            Base58::Bitcoind => Base58Bitcoin.decode(text),
            Base58::Flickr => Base58Flickr.decode(text),
            Base58::Custom(custom) => custom.decode(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base58_bitcoin_encode() {
        assert_eq!(Base58::encode_bitcoin(b"Hello"), "9Ajdvzr");
        assert_eq!(Base58::encode_bitcoin(b""), "");
        assert_eq!(Base58::encode_bitcoin(b"Hello World!"), "2NEpo7TZRRrLZSi2U");
    }

    #[test]
    fn test_base58_bitcoin_decode() {
        assert_eq!(Base58::decode_bitcoin("9Ajdvzr").unwrap(), b"Hello");
        assert_eq!(Base58::decode_bitcoin("").unwrap(), b"");
        assert_eq!(Base58::decode_bitcoin("2NEpo7TZRRrLZSi2U").unwrap(), b"Hello World!");
    }

    #[test]
    fn test_base58_bitcoin_decode_invalid() {
        // Base58 doesn't include 0, O, I, l
        let result = Base58::decode_bitcoin("0OIl");
        assert!(matches!(result, Err(BinaryCodecError::Base58Decode { .. })));
    }

    #[test]
    fn test_base58_flickr_encode() {
        assert_eq!(Base58::encode_flickr(b"Hello"), "9aJCVZR");
        assert_eq!(Base58::encode_flickr(b""), "");
    }

    #[test]
    fn test_base58_flickr_decode() {
        assert_eq!(Base58::decode_flickr("9aJCVZR").unwrap(), b"Hello");
        assert_eq!(Base58::decode_flickr("").unwrap(), b"");
    }

    #[test]
    fn test_base58_codec_constants() {
        // BASE58_BITCOIN
        assert_eq!(Base58Bitcoin.encode(b"Hello"), "9Ajdvzr");
        assert_eq!(Base58Bitcoin.decode("9Ajdvzr").unwrap(), b"Hello");

        // BASE58_FLICKR
        assert_eq!(Base58Flickr.encode(b"Hello"), "9aJCVZR");
        assert_eq!(Base58Flickr.decode("9aJCVZR").unwrap(), b"Hello");
    }

    #[test]
    fn test_base58_codec_as_trait_object() {
        let codecs: Vec<&dyn BinaryCodec> = vec![&Base58Bitcoin, &Base58Flickr];

        assert_eq!(codecs[0].encode(b"Hello"), "9Ajdvzr");
        assert_eq!(codecs[1].encode(b"Hello"), "9aJCVZR");
    }

    #[test]
    fn test_base58_bitcoin_flickr_different() {
        // Bitcoin and Flickr produce different encodings
        assert_ne!(Base58Bitcoin.encode(b"test"), Base58Flickr.encode(b"test"));

        // Each codec rejects the other's encoding
        let bitcoin_encoded = Base58Bitcoin.encode(b"test");
        let flickr_encoded = Base58Flickr.encode(b"test");

        // Bitcoin codec should decode Bitcoin encoding
        assert_eq!(Base58Bitcoin.decode(&bitcoin_encoded).unwrap(), b"test");

        // Flickr codec should decode Flickr encoding
        assert_eq!(Base58Flickr.decode(&flickr_encoded).unwrap(), b"test");
    }

    #[test]
    fn test_base58_custom_alphabet() {
        // Ripple alphabet
        let alphabet = "rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";
        let codec = Base58::new(alphabet).unwrap();

        let encoded = codec.encode(b"Hello");
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn test_base58_custom_alphabet_invalid_length() {
        // Alphabet with wrong length
        let alphabet = "ABC"; // Only 3 chars
        let result = Base58::new(alphabet);
        assert!(matches!(result, Err(BinaryCodecError::InvalidCharacterSet { .. })));
    }

    #[test]
    fn test_base58_custom_alphabet_invalid_duplicate() {
        // Alphabet with duplicate character
        let alphabet = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1";
        let result = Base58::new(alphabet);
        assert!(matches!(result, Err(BinaryCodecError::Base58Alphabet { .. })));
    }

    #[test]
    fn test_base58_custom_alphabet_invalid_non_ascii() {
        // Alphabet with non-ASCII character (using UTF-8)
        let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstu🎉"; // emoji makes it invalid
        let result = Base58::new(alphabet);
        // Either length error (emoji is multi-byte) or alphabet error
        assert!(result.is_err());
    }

    #[test]
    fn test_base58_leading_zero_bytes() {
        // Base58 has special handling for leading zero bytes
        // Each leading zero byte is encoded as '1' in Bitcoin alphabet
        let data_with_leading_zeros = b"\x00\x00\x01\x02\x03";
        let encoded = Base58::encode_bitcoin(data_with_leading_zeros);

        // Should start with '11' (two leading zeros = two '1' chars)
        assert!(encoded.starts_with("11"));

        // Round trip should preserve all bytes including leading zeros
        let decoded = Base58::decode_bitcoin(&encoded).unwrap();
        assert_eq!(decoded, data_with_leading_zeros);

        // Flickr alphabet uses '1' for leading zeros too
        let encoded_flickr = Base58::encode_flickr(data_with_leading_zeros);
        assert!(encoded_flickr.starts_with("11"));
        assert_eq!(Base58::decode_flickr(&encoded_flickr).unwrap(), data_with_leading_zeros);
    }
}
