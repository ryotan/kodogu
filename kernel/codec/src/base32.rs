//! Base32 encoding and decoding.
//!
//! This module provides Base32 encoding/decoding following RFC 4648 and Crockford Base32 specifications.
//!
//! ## Available Variants
//!
//! - **Standard (RFC 4648)**: Uses alphabet A-Z2-7, commonly used in TOTP and general encoding
//! - **Hex (RFC 4648)**: Uses alphabet 0-9A-V, useful when lexicographic sorting is important
//! - **Crockford**: Uses 0-9A-HJKMNP-TV-Z (excludes I/L/O/U), designed for human readability and error resistance
//! - **Custom**: Define your own 32-character alphabet with optional padding
//!
//! ## Choosing a Variant
//!
//! - **Standard**: Default choice for general purpose encoding
//! - **Hex**: When you need case-insensitive encoding with natural sorting
//! - **Crockford**: When encoding IDs/tokens that humans will read/type (confusion-resistant)
//! - **Custom**: When you need specific alphabet requirements
//!
//! ## Examples
//!
//! ```
//! use codec::{Base32, BinaryCodec};
//!
//! // Using the default Standard variant
//! let encoded = Base32::encode(b"Hello");
//! assert_eq!(encoded, "JBSWY3DP");
//!
//! // Using specific variants
//! assert_eq!(Base32::encode_hex(b"Hello"), "91IMOR3F");
//! assert_eq!(Base32::encode_crockford(b"Hello"), "91JPRV3F");
//!
//! // Using codec instances
//! let codec = Base32::Standard;
//! let data = codec.encode(b"Hello");
//! assert_eq!(codec.decode(&data).unwrap(), b"Hello");
//! ```
//!
//! ## References
//!
//! - [RFC 4648 - The Base16, Base32, and Base64 Data Encodings](https://tools.ietf.org/html/rfc4648)
//! - [Crockford Base32](https://www.crockford.com/base32.html)

use crate::{
    BinaryCodec, BinaryCodecError, BinaryCodecs, DataEncodingDecodeError, DataEncodingSpecError,
    InvalidCharacterSetError,
};
use data_encoding::{BASE32, BASE32HEX, Encoding, Specification};
use data_encoding_macro::new_encoding;
use snafu::{ResultExt, ensure};

/// RFC 4648 Base32 encoding with alphabet A-Z2-7.
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// The standard Base32 encoding defined in RFC 4648.
/// Uses padding ('=') to align output to 8-character blocks.
///
/// # Examples
///
/// ```
/// use codec::{Base32Standard, BinaryCodec};
///
/// let codec = Base32Standard;
/// assert_eq!(codec.encode(b"Hello"), "JBSWY3DP");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base32Standard;

/// RFC 4648 Base32 Hex encoding with alphabet 0-9A-V.
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Also known as "base32hex", this variant uses a different alphabet
/// that preserves lexicographic ordering when encoded strings are sorted.
///
/// # Examples
///
/// ```
/// use codec::{Base32Hex, BinaryCodec};
///
/// let codec = Base32Hex;
/// assert_eq!(codec.encode(b"Hello"), "91IMOR3F");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base32Hex;

/// Crockford Base32 encoding (uppercase variant).
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Uses alphabet 0-9A-HJKMNP-TV-Z (excludes I/L/O/U to avoid confusion).
/// Decoding accepts both uppercase and lowercase, and maps confusion-prone
/// characters: I/i/L/l→1, O/o→0.
///
/// This encoding does not use padding.
///
/// # Examples
///
/// ```
/// use codec::{Base32CrockfordUpper, BinaryCodec};
///
/// let codec = Base32CrockfordUpper;
/// assert_eq!(codec.encode(b"Hello"), "91JPRV3F");
/// // Decoding is case-insensitive and confusion-resistant
/// assert_eq!(codec.decode("91jprv3f").unwrap(), b"Hello");
/// assert_eq!(codec.decode("9IJPRV3F").unwrap(), b"Hello"); // I→1
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base32CrockfordUpper;

/// Crockford Base32 encoding (lowercase variant).
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Same as [`Base32CrockfordUpper`] but encodes to lowercase.
/// Decoding behavior is identical (case-insensitive with confusion-resistant mappings).
///
/// # Examples
///
/// ```
/// use codec::{Base32CrockfordLower, BinaryCodec};
///
/// let codec = Base32CrockfordLower;
/// assert_eq!(codec.encode(b"Hello"), "91jprv3f");
/// assert_eq!(codec.decode("91JPRV3F").unwrap(), b"Hello"); // Case-insensitive
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base32CrockfordLower;

/// Custom Base32 encoding with user-defined alphabet.
///
/// Allows defining a custom 32-character alphabet and optional padding character.
///
/// # Examples
///
/// ```
/// use codec::{Base32Custom, BinaryCodec};
///
/// let codec = Base32Custom::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567").unwrap();
/// assert_eq!(codec.encode(b"Hi"), "JBUQ====");
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Base32Custom {
    encoding: Encoding,
}

/// Base32 encoding variants.
///
/// This enum provides a unified interface for different Base32 encoding schemes.
/// The default variant is [`Base32::Standard`].
///
/// # Examples
///
/// ```
/// use codec::{Base32, BinaryCodec};
///
/// // Using enum variants
/// let std_codec = Base32::Standard;
/// let hex_codec = Base32::Hex;
/// let crockford = Base32::CrockfordUpper;
///
/// assert_eq!(std_codec.encode(b"Hello"), "JBSWY3DP");
/// assert_eq!(hex_codec.encode(b"Hello"), "91IMOR3F");
/// assert_eq!(crockford.encode(b"Hello"), "91JPRV3F");
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum Base32 {
    /// RFC 4648 Base32 (A-Z2-7)
    #[default]
    Standard,
    /// RFC 4648 Base32 Hex (0-9A-V)
    Hex,
    /// Crockford Base32 (uppercase)
    CrockfordUpper,
    /// Crockford Base32 (lowercase)
    CrockfordLower,
    /// Custom alphabet
    Custom(Base32Custom),
}

/// Marker trait for Base32 codec types.
///
/// Implemented by all Base32 variants to enable generic programming
/// over different Base32 encoding schemes while maintaining type safety.
///
/// # Example
/// ```
/// use codec::{Base32Codec, Base32Standard, Base32Hex, BinaryCodec};
///
/// fn encode_with_base32<C: Base32Codec>(codec: &C, data: &[u8]) -> String {
///     codec.encode(data)
/// }
///
/// let standard_result = encode_with_base32(&Base32Standard, b"Hello");
/// let hex_result = encode_with_base32(&Base32Hex, b"Hello");
///
/// assert_eq!(standard_result, "JBSWY3DP");
/// assert_eq!(hex_result, "91IMOR3F");
/// ```
pub trait Base32Codec: BinaryCodec {}

fn decode_internal(
    encoding: &Encoding,
    text: &str,
    variant: &str,
) -> Result<Vec<u8>, BinaryCodecError> {
    encoding.decode(text.as_bytes()).context(DataEncodingDecodeError { variant })
}

impl Base32Codec for Base32Standard {}
impl BinaryCodec for Base32Standard {
    fn encode(&self, bytes: &[u8]) -> String {
        BASE32.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&BASE32, text, "Base32 Standard")
    }
}

impl Base32Codec for Base32Hex {}
impl BinaryCodec for Base32Hex {
    fn encode(&self, bytes: &[u8]) -> String {
        BASE32HEX.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&BASE32HEX, text, "Base32 Hex")
    }
}

/// Crockford Base32 encoding (uppercase).
///
/// Symbol set: 0-9, A-H, J-K, M-N, P-T, V-Z (excludes I, L, O, U)
///
/// Character translation for confusion-resistant decoding:
/// - I/i/L/l → 1 (confusion with digit one)
/// - O/o → 0 (confusion with digit zero)
/// - Lowercase a-z → uppercase A-Z (case-insensitive)
const CROCKFORD_UPPER: Encoding = new_encoding! {
    symbols: "0123456789ABCDEFGHJKMNPQRSTVWXYZ",
    translate_from: "abcdefghjkmnpqrstvwxyzIiLlOo",
    translate_to: "ABCDEFGHJKMNPQRSTVWXYZ111100",
};
impl Base32Codec for Base32CrockfordUpper {}
impl BinaryCodec for Base32CrockfordUpper {
    fn encode(&self, bytes: &[u8]) -> String {
        CROCKFORD_UPPER.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&CROCKFORD_UPPER, text, "Base32 Crockford")
    }
}

/// Crockford Base32 encoding (lowercase).
///
/// Symbol set: 0-9, a-h, j-k, m-n, p-t, v-z (excludes i, l, o, u)
///
/// Character translation for confusion-resistant decoding:
/// - I/i/L/l → 1 (confusion with digit one)
/// - O/o → 0 (confusion with digit zero)
/// - Uppercase A-Z → lowercase a-z (case-insensitive)
const CROCKFORD_LOWER: Encoding = new_encoding! {
    symbols: "0123456789abcdefghjkmnpqrstvwxyz",
    translate_from: "ABCDEFGHJKMNPQRSTVWXYZIiLlOo",
    translate_to: "abcdefghjkmnpqrstvwxyz111100",
};
impl Base32Codec for Base32CrockfordLower {}
impl BinaryCodec for Base32CrockfordLower {
    fn encode(&self, bytes: &[u8]) -> String {
        CROCKFORD_LOWER.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&CROCKFORD_LOWER, text, "Base32 Crockford")
    }
}

impl Base32Codec for Base32Custom {}
impl BinaryCodec for Base32Custom {
    fn encode(&self, bytes: &[u8]) -> String {
        self.encoding.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&self.encoding, text, "Base32 Custom")
    }
}
impl Base32Custom {
    /// Creates a custom Base32 codec with a 32-character alphabet and default padding ('=').
    ///
    /// # Arguments
    ///
    /// * `alphabet` - A 32-character string defining the encoding alphabet
    ///
    /// # Errors
    ///
    /// Returns `BinaryCodecError::InvalidCharacterSet` if:
    /// - Alphabet length is not exactly 32 characters
    /// - Alphabet contains duplicate characters
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::{Base32Custom, BinaryCodec};
    ///
    /// let codec = Base32Custom::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567").unwrap();
    /// assert_eq!(codec.encode(b"Hi"), "JBUQ====");
    /// assert_eq!(codec.decode("JBUQ====").unwrap(), b"Hi");
    /// ```
    pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> {
        Self::new_with_padding(alphabet, Some('='))
    }

    /// Creates a custom Base32 codec with a 32-character alphabet and custom padding.
    ///
    /// # Arguments
    ///
    /// * `alphabet` - A 32-character string defining the encoding alphabet
    /// * `padding` - Optional padding character. Use `None` for no padding.
    ///
    /// # Errors
    ///
    /// Returns `BinaryCodecError::InvalidCharacterSet` if:
    /// - Alphabet length is not exactly 32 characters
    /// - Alphabet contains duplicate characters
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::{Base32Custom, BinaryCodec};
    ///
    /// // Without padding
    /// let codec = Base32Custom::new_with_padding("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567", None).unwrap();
    /// assert_eq!(codec.encode(b"Hi"), "JBUQ");
    /// assert_eq!(codec.decode("JBUQ").unwrap(), b"Hi");
    ///
    /// // Custom padding character
    /// let codec = Base32Custom::new_with_padding("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567", Some('*')).unwrap();
    /// assert_eq!(codec.encode(b"Hi"), "JBUQ****");
    /// ```
    pub fn new_with_padding(
        alphabet: &str,
        padding: Option<char>,
    ) -> Result<Self, BinaryCodecError> {
        ensure!(
            alphabet.len() == 32,
            InvalidCharacterSetError {
                encoding: BinaryCodecs::Base32,
                message: format!("Base32 alphabet must be 32 characters, got {}", alphabet.len()),
            }
        );

        ensure!(
            alphabet.is_ascii(),
            InvalidCharacterSetError {
                encoding: BinaryCodecs::Base32,
                message: "Base32 alphabet must contain only ASCII characters",
            }
        );

        let mut spec = Specification::new();
        spec.symbols.push_str(alphabet);
        spec.padding = padding;

        let encoding = spec.encoding().context(DataEncodingSpecError)?;
        Ok(Self { encoding })
    }
}

impl Base32Codec for Base32 {}
impl BinaryCodec for Base32 {
    fn encode(&self, bytes: &[u8]) -> String {
        match self {
            Base32::Standard => Base32Standard.encode(bytes),
            Base32::Hex => Base32Hex.encode(bytes),
            Base32::CrockfordUpper => Base32CrockfordUpper.encode(bytes),
            Base32::CrockfordLower => Base32CrockfordLower.encode(bytes),
            Base32::Custom(encoding) => encoding.encode(bytes),
        }
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        match self {
            Base32::Standard => Base32Standard.decode(text),
            Base32::Hex => Base32Hex.decode(text),
            Base32::CrockfordUpper => Base32CrockfordUpper.decode(text),
            Base32::CrockfordLower => Base32CrockfordLower.decode(text),
            Base32::Custom(encoding) => encoding.decode(text),
        }
    }
}
impl Base32 {
    /// Encodes bytes to Base32 string (RFC 4648 standard variant).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    ///
    /// assert_eq!(Base32::encode(b"Hello"), "JBSWY3DP");
    /// ```
    pub fn encode(bytes: &[u8]) -> String {
        Base32Standard.encode(bytes)
    }

    /// Encodes bytes to Base32 string (RFC 4648 standard with A-Z2-7).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    ///
    /// assert_eq!(Base32::encode_standard(b"Hello"), "JBSWY3DP");
    /// ```
    pub fn encode_standard(bytes: &[u8]) -> String {
        Base32Standard.encode(bytes)
    }

    /// Encodes bytes to Base32 Hex string (RFC 4648 with 0-9A-V).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    ///
    /// assert_eq!(Base32::encode_hex(b"Hello"), "91IMOR3F");
    /// ```
    pub fn encode_hex(bytes: &[u8]) -> String {
        Base32Hex.encode(bytes)
    }

    /// Encodes bytes to Base32 Crockford string (0-9, A-H, J-K, M-N, P-T, V-Z, excludes I/L/O/U, default variant).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    ///
    /// assert_eq!(Base32::encode_crockford(b"Hello"), "91JPRV3F");
    /// ```
    pub fn encode_crockford(bytes: &[u8]) -> String {
        Base32CrockfordUpper.encode(bytes)
    }

    /// Encodes bytes to Base32 Crockford uppercase string (0-9, A-H, J-K, M-N, P-T, V-Z, excludes I/L/O/U).
    ///
    /// This is the canonical Crockford encoding (uppercase).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    ///
    /// assert_eq!(Base32::encode_crockford_upper(b"Hello"), "91JPRV3F");
    /// ```
    pub fn encode_crockford_upper(bytes: &[u8]) -> String {
        Base32CrockfordUpper.encode(bytes)
    }

    /// Encodes bytes to Base32 Crockford lowercase string (0-9, a-h, j-k, m-n, p-t, v-z, excludes i/l/o/u).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    ///
    /// assert_eq!(Base32::encode_crockford_lower(b"Hello"), "91jprv3f");
    /// ```
    pub fn encode_crockford_lower(bytes: &[u8]) -> String {
        Base32CrockfordLower.encode(bytes)
    }

    /// Decodes Base32 string to bytes (RFC 4648 standard variant).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    ///
    /// assert_eq!(Base32::decode("JBSWY3DP").unwrap(), b"Hello");
    /// ```
    pub fn decode(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base32Standard.decode(text)
    }

    /// Decodes Base32 string to bytes (RFC 4648 standard).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    ///
    /// assert_eq!(Base32::decode_standard("JBSWY3DP").unwrap(), b"Hello");
    /// ```
    pub fn decode_standard(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base32Standard.decode(text)
    }

    /// Decodes Base32 Hex string to bytes (RFC 4648 hex variant).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    ///
    /// assert_eq!(Base32::decode_hex("91IMOR3F").unwrap(), b"Hello");
    /// ```
    pub fn decode_hex(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base32Hex.decode(text)
    }

    /// Decodes Base32 Crockford string to bytes.
    ///
    /// Crockford Base32 decoding accepts uppercase, lowercase, and confusion-resistant
    /// character mappings (I/i/L/l→1, O/o→0) per the Crockford specification.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    ///
    /// // Accepts uppercase (canonical encoding)
    /// assert_eq!(Base32::decode_crockford("91JPRV3F").unwrap(), b"Hello");
    /// // Accepts lowercase
    /// assert_eq!(Base32::decode_crockford("91jprv3f").unwrap(), b"Hello");
    /// // Accepts confusion-resistant characters: I→1, O→0
    /// assert_eq!(Base32::decode_crockford("9IJPRV3F").unwrap(), b"Hello");
    /// ```
    pub fn decode_crockford(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base32CrockfordUpper.decode(text)
    }

    /// Creates a new Base32 codec with a custom 32-character alphabet and default padding ('=').
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    /// use codec::BinaryCodec;
    ///
    /// let codec = Base32::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567").unwrap();
    /// assert_eq!(codec.encode(b"Hi"), "JBUQ====");
    /// ```
    pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> {
        Self::new_with_padding(alphabet, Some('='))
    }

    /// Creates a new Base32 codec with a custom 32-character alphabet and custom padding.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base32;
    /// use codec::BinaryCodec;
    ///
    /// // Without padding
    /// let codec = Base32::new_with_padding("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567", None).unwrap();
    /// assert_eq!(codec.encode(b"Hi"), "JBUQ");
    /// ```
    pub fn new_with_padding(
        alphabet: &str,
        padding: Option<char>,
    ) -> Result<Self, BinaryCodecError> {
        Base32Custom::new_with_padding(alphabet, padding).map(Base32::Custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base32_encode() {
        assert_eq!(Base32::encode_standard(b"Hello"), "JBSWY3DP");
        assert_eq!(Base32::encode_standard(b""), "");
        assert_eq!(Base32::encode_standard(b"f"), "MY======");
        assert_eq!(Base32::encode_standard(b"fo"), "MZXQ====");
        assert_eq!(Base32::encode_standard(b"foo"), "MZXW6===");
    }

    #[test]
    fn test_base32_decode() {
        assert_eq!(Base32::decode_standard("JBSWY3DP").unwrap(), b"Hello");
        assert_eq!(Base32::decode_standard("").unwrap(), b"");
        assert_eq!(Base32::decode_standard("MY======").unwrap(), b"f");
        assert_eq!(Base32::decode_standard("MZXQ====").unwrap(), b"fo");
        assert_eq!(Base32::decode_standard("MZXW6===").unwrap(), b"foo");
    }

    #[test]
    fn test_base32_decode_invalid() {
        let result = Base32::decode_standard("invalid!");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    #[test]
    fn test_base32_hex_encode() {
        assert_eq!(Base32::encode_hex(b"Hello"), "91IMOR3F");
        assert_eq!(Base32::encode_hex(b""), "");
        assert_eq!(Base32::encode_hex(b"f"), "CO======");
        assert_eq!(Base32::encode_hex(b"fo"), "CPNG====");
    }

    #[test]
    fn test_base32_hex_decode() {
        assert_eq!(Base32::decode_hex("91IMOR3F").unwrap(), b"Hello");
        assert_eq!(Base32::decode_hex("").unwrap(), b"");
        assert_eq!(Base32::decode_hex("CO======").unwrap(), b"f");
        assert_eq!(Base32::decode_hex("CPNG====").unwrap(), b"fo");
    }

    #[test]
    fn test_base32_codec_constants() {
        // BASE32_STANDARD
        assert_eq!(Base32Standard.encode(b"Hello"), "JBSWY3DP");
        assert_eq!(Base32Standard.decode("JBSWY3DP").unwrap(), b"Hello");

        // BASE32_HEX
        assert_eq!(Base32Hex.encode(b"Hello"), "91IMOR3F");
        assert_eq!(Base32Hex.decode("91IMOR3F").unwrap(), b"Hello");
    }

    #[test]
    fn test_base32_codec_as_trait_object() {
        let codecs: Vec<&dyn BinaryCodec> = vec![&Base32Standard, &Base32Hex];

        assert_eq!(codecs[0].encode(b"Hello"), "JBSWY3DP");
        assert_eq!(codecs[1].encode(b"Hello"), "91IMOR3F");
    }

    #[test]
    fn test_base32_custom_alphabet() {
        let codec = Base32::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567").unwrap();
        assert_eq!(codec.encode(b"Hello"), "JBSWY3DP");
        assert_eq!(codec.decode("JBSWY3DP").unwrap(), b"Hello");
    }

    #[test]
    fn test_base32_with_padding() {
        // With padding (default)
        let codec = Base32::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567").unwrap();
        assert_eq!(codec.encode(b"Hi"), "JBUQ====");

        // Without padding
        let codec_no_pad =
            Base32::new_with_padding("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567", None).unwrap();
        assert_eq!(codec_no_pad.encode(b"Hi"), "JBUQ");
    }

    #[test]
    fn test_base32_invalid_alphabet_length() {
        let result = Base32::new("ABC");
        assert!(matches!(result, Err(BinaryCodecError::InvalidCharacterSet { .. })));
    }

    #[test]
    fn test_base32_padding_variations() {
        // 1 byte (6 padding chars: ======)
        assert_eq!(Base32::encode_standard(b"f"), "MY======");
        assert_eq!(Base32::decode_standard("MY======").unwrap(), b"f");

        // 2 bytes (4 padding chars: ====)
        assert_eq!(Base32::encode_standard(b"fo"), "MZXQ====");
        assert_eq!(Base32::decode_standard("MZXQ====").unwrap(), b"fo");

        // 3 bytes (3 padding chars: ===)
        assert_eq!(Base32::encode_standard(b"foo"), "MZXW6===");
        assert_eq!(Base32::decode_standard("MZXW6===").unwrap(), b"foo");

        // 4 bytes (1 padding char: =)
        assert_eq!(Base32::encode_standard(b"foob"), "MZXW6YQ=");
        assert_eq!(Base32::decode_standard("MZXW6YQ=").unwrap(), b"foob");

        // 5 bytes (no padding) - exact multiple of 5
        assert_eq!(Base32::encode_standard(b"fooba"), "MZXW6YTB");
        assert_eq!(Base32::decode_standard("MZXW6YTB").unwrap(), b"fooba");
    }

    #[test]
    fn test_base32_duplicate_alphabet() {
        // Alphabet with duplicate character
        let result = Base32::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingSpec { .. })));
    }

    #[test]
    fn test_base32_standard_strict() {
        // Accepts standard alphabet (A-Z2-7)
        assert_eq!(Base32Standard.decode("JBSWY3DP").unwrap(), b"Hello");

        // Rejects hex alphabet (0-9A-V)
        let result = Base32Standard.decode("91IMOR3F");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    #[test]
    fn test_base32_hex_strict() {
        // Accepts hex alphabet (0-9A-V)
        assert_eq!(Base32Hex.decode("91IMOR3F").unwrap(), b"Hello");

        // Rejects standard alphabet (A-Z2-7)
        let result = Base32Hex.decode("JBSWY3DP");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    #[test]
    fn test_base32_crockford_encode() {
        assert_eq!(Base32::encode_crockford(b"Hello"), "91JPRV3F");
        assert_eq!(Base32::encode_crockford(b""), "");
    }

    #[test]
    fn test_base32_crockford_decode() {
        assert_eq!(Base32::decode_crockford("91JPRV3F").unwrap(), b"Hello");
        assert_eq!(Base32::decode_crockford("").unwrap(), b"");

        // Accepts lowercase
        assert_eq!(Base32::decode_crockford("91jprv3f").unwrap(), b"Hello");

        // Accepts confusion-resistant characters: I/i/L/l→1, O/o→0
        // Test with actual positions where 1 and 0 appear
        assert_eq!(Base32::decode_crockford("9IJPRV3F").unwrap(), b"Hello"); // I→1
        assert_eq!(Base32::decode_crockford("9iJPRV3F").unwrap(), b"Hello"); // i→1
        assert_eq!(Base32::decode_crockford("9LJPRV3F").unwrap(), b"Hello"); // L→1
        assert_eq!(Base32::decode_crockford("9lJPRV3F").unwrap(), b"Hello"); // l→1

        // For O/o→0 test, use a different input that has 0 in encoding
        let bytes_with_zero = b"\x00\x00"; // Should encode with leading 0s
        let encoded = Base32::encode_crockford(bytes_with_zero);
        assert_eq!(Base32::decode_crockford(&encoded.replace('0', "O")).unwrap(), bytes_with_zero); // O→0
        assert_eq!(Base32::decode_crockford(&encoded.replace('0', "o")).unwrap(), bytes_with_zero);
        // o→0
    }

    #[test]
    fn test_base32_crockford_lower_encode() {
        assert_eq!(Base32::encode_crockford_lower(b"Hello"), "91jprv3f");
        assert_eq!(Base32::encode_crockford_lower(b""), "");
    }

    #[test]
    fn test_base32_crockford_codec_constants() {
        assert_eq!(Base32CrockfordUpper.encode(b"Hello"), "91JPRV3F");
        assert_eq!(Base32CrockfordUpper.decode("91JPRV3F").unwrap(), b"Hello");

        assert_eq!(Base32CrockfordLower.encode(b"Hello"), "91jprv3f");
        assert_eq!(Base32CrockfordLower.decode("91jprv3f").unwrap(), b"Hello");
    }

    #[test]
    fn test_base32_crockford_as_trait_object() {
        let codecs: Vec<&dyn BinaryCodec> =
            vec![&Base32Standard, &Base32Hex, &Base32CrockfordUpper, &Base32CrockfordLower];

        assert_eq!(codecs[0].encode(b"Hello"), "JBSWY3DP");
        assert_eq!(codecs[1].encode(b"Hello"), "91IMOR3F");
        assert_eq!(codecs[2].encode(b"Hello"), "91JPRV3F");
        assert_eq!(codecs[3].encode(b"Hello"), "91jprv3f");
    }

    #[test]
    fn test_base32_crockford_no_padding() {
        // Crockford Base32 does not use padding (unlike standard Base32)
        let bytes = b"Hi";
        let encoded = Base32::encode_crockford(bytes);

        // Should not contain padding characters
        assert!(!encoded.contains('='));

        // Verify encoding result
        assert_eq!(encoded, "91MG");
    }

    // Larger datasets
    #[test]
    fn test_base32_larger_data() {
        // Test with 10 bytes
        let data_10 = b"HelloWorld";
        let encoded_std = Base32::encode_standard(data_10);
        assert_eq!(Base32::decode_standard(&encoded_std).unwrap(), data_10);

        let encoded_hex = Base32::encode_hex(data_10);
        assert_eq!(Base32::decode_hex(&encoded_hex).unwrap(), data_10);

        let encoded_crockford = Base32::encode_crockford(data_10);
        assert_eq!(Base32::decode_crockford(&encoded_crockford).unwrap(), data_10);

        // Test with 20 bytes
        let data_20 = b"Hello, Base32 World!";
        let encoded_std = Base32::encode_standard(data_20);
        assert_eq!(Base32::decode_standard(&encoded_std).unwrap(), data_20);

        let encoded_hex = Base32::encode_hex(data_20);
        assert_eq!(Base32::decode_hex(&encoded_hex).unwrap(), data_20);

        let encoded_crockford = Base32::encode_crockford(data_20);
        assert_eq!(Base32::decode_crockford(&encoded_crockford).unwrap(), data_20);
    }

    // Binary data (non-ASCII)
    #[test]
    fn test_base32_binary_data() {
        // Test with binary data containing all byte values
        let binary_data: Vec<u8> = vec![0x00, 0x01, 0xFF, 0x7F, 0x80, 0xAA, 0x55];

        let encoded_std = Base32::encode_standard(&binary_data);
        assert_eq!(Base32::decode_standard(&encoded_std).unwrap(), binary_data);

        let encoded_hex = Base32::encode_hex(&binary_data);
        assert_eq!(Base32::decode_hex(&encoded_hex).unwrap(), binary_data);

        let encoded_crockford = Base32::encode_crockford(&binary_data);
        assert_eq!(Base32::decode_crockford(&encoded_crockford).unwrap(), binary_data);
    }

    // UTF-8 multi-byte characters
    #[test]
    fn test_base32_utf8_data() {
        let utf8_data = "こんにちは世界";

        let encoded_std = Base32::encode_standard(utf8_data.as_bytes());
        assert_eq!(Base32::decode_standard(&encoded_std).unwrap(), utf8_data.as_bytes());

        let encoded_hex = Base32::encode_hex(utf8_data.as_bytes());
        assert_eq!(Base32::decode_hex(&encoded_hex).unwrap(), utf8_data.as_bytes());

        let encoded_crockford = Base32::encode_crockford(utf8_data.as_bytes());
        assert_eq!(Base32::decode_crockford(&encoded_crockford).unwrap(), utf8_data.as_bytes());
    }

    // Base32 enum Default variant
    #[test]
    fn test_base32_enum_default() {
        let codec = Base32::default();
        assert_eq!(codec, Base32::Standard);

        // Default should behave like Standard
        assert_eq!(codec.encode(b"Hello"), "JBSWY3DP");
        assert_eq!(codec.decode("JBSWY3DP").unwrap(), b"Hello");
    }

    // Hex padding variations (1-5 bytes)
    #[test]
    fn test_base32_hex_padding_variations() {
        // 1 byte (6 padding chars)
        assert_eq!(Base32::encode_hex(b"f"), "CO======");
        assert_eq!(Base32::decode_hex("CO======").unwrap(), b"f");

        // 2 bytes (4 padding chars)
        assert_eq!(Base32::encode_hex(b"fo"), "CPNG====");
        assert_eq!(Base32::decode_hex("CPNG====").unwrap(), b"fo");

        // 3 bytes (3 padding chars)
        assert_eq!(Base32::encode_hex(b"foo"), "CPNMU===");
        assert_eq!(Base32::decode_hex("CPNMU===").unwrap(), b"foo");

        // 4 bytes (1 padding char)
        assert_eq!(Base32::encode_hex(b"foob"), "CPNMUOG=");
        assert_eq!(Base32::decode_hex("CPNMUOG=").unwrap(), b"foob");

        // 5 bytes (no padding)
        assert_eq!(Base32::encode_hex(b"fooba"), "CPNMUOJ1");
        assert_eq!(Base32::decode_hex("CPNMUOJ1").unwrap(), b"fooba");
    }

    // Crockford padding variations (1-5 bytes)
    #[test]
    fn test_base32_crockford_padding_variations() {
        // Crockford does not use padding, but test different lengths
        // 1 byte
        let encoded_1 = Base32::encode_crockford(b"f");
        assert!(!encoded_1.contains('='));
        assert_eq!(Base32::decode_crockford(&encoded_1).unwrap(), b"f");

        // 2 bytes
        let encoded_2 = Base32::encode_crockford(b"fo");
        assert!(!encoded_2.contains('='));
        assert_eq!(Base32::decode_crockford(&encoded_2).unwrap(), b"fo");

        // 3 bytes
        let encoded_3 = Base32::encode_crockford(b"foo");
        assert!(!encoded_3.contains('='));
        assert_eq!(Base32::decode_crockford(&encoded_3).unwrap(), b"foo");

        // 4 bytes
        let encoded_4 = Base32::encode_crockford(b"foob");
        assert!(!encoded_4.contains('='));
        assert_eq!(Base32::decode_crockford(&encoded_4).unwrap(), b"foob");

        // 5 bytes
        let encoded_5 = Base32::encode_crockford(b"fooba");
        assert!(!encoded_5.contains('='));
        assert_eq!(Base32::decode_crockford(&encoded_5).unwrap(), b"fooba");
    }

    // Custom variant decode error
    #[test]
    fn test_base32_custom_decode_invalid() {
        let codec = Base32::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567").unwrap();

        // Invalid character not in custom alphabet
        let result = codec.decode("INVALID!");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));

        // Character not in alphabet
        let result = codec.decode("Z1234567"); // '1' is in alphabet, but '0' is not
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    // Round-trip tests for all variants
    #[test]
    fn test_base32_all_variants_round_trip() {
        let test_data = b"Base32 round-trip test!";

        // Standard
        let encoded_std = Base32Standard.encode(test_data);
        assert_eq!(Base32Standard.decode(&encoded_std).unwrap(), test_data);

        // Hex
        let encoded_hex = Base32Hex.encode(test_data);
        assert_eq!(Base32Hex.decode(&encoded_hex).unwrap(), test_data);

        // Crockford Upper
        let encoded_crockford_upper = Base32CrockfordUpper.encode(test_data);
        assert_eq!(Base32CrockfordUpper.decode(&encoded_crockford_upper).unwrap(), test_data);

        // Crockford Lower
        let encoded_crockford_lower = Base32CrockfordLower.encode(test_data);
        assert_eq!(Base32CrockfordLower.decode(&encoded_crockford_lower).unwrap(), test_data);

        // Custom
        let custom = Base32Custom::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567").unwrap();
        let encoded_custom = custom.encode(test_data);
        assert_eq!(custom.decode(&encoded_custom).unwrap(), test_data);
    }

    // Enum variants round-trip
    #[test]
    fn test_base32_enum_variants_round_trip() {
        let test_data = b"Enum variant test";

        let variants = vec![
            Base32::Standard,
            Base32::Hex,
            Base32::CrockfordUpper,
            Base32::CrockfordLower,
            Base32::Custom(Base32Custom::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567").unwrap()),
        ];

        for variant in variants {
            let encoded = variant.encode(test_data);
            assert_eq!(variant.decode(&encoded).unwrap(), test_data);
        }
    }

    // Edge case - maximum padding
    #[test]
    fn test_base32_maximum_padding() {
        // 1 byte requires maximum padding (6 '=' characters)
        let one_byte = b"a";
        let encoded_std = Base32::encode_standard(one_byte);
        assert_eq!(encoded_std.matches('=').count(), 6);
        assert_eq!(Base32::decode_standard(&encoded_std).unwrap(), one_byte);

        let encoded_hex = Base32::encode_hex(one_byte);
        assert_eq!(encoded_hex.matches('=').count(), 6);
        assert_eq!(Base32::decode_hex(&encoded_hex).unwrap(), one_byte);
    }

    // Edge case - no padding needed
    #[test]
    fn test_base32_no_padding_needed() {
        // 5 bytes is exact multiple, no padding needed
        let five_bytes = b"abcde";
        let encoded_std = Base32::encode_standard(five_bytes);
        assert!(!encoded_std.contains('='));
        assert_eq!(Base32::decode_standard(&encoded_std).unwrap(), five_bytes);

        let encoded_hex = Base32::encode_hex(five_bytes);
        assert!(!encoded_hex.contains('='));
        assert_eq!(Base32::decode_hex(&encoded_hex).unwrap(), five_bytes);
    }
}
