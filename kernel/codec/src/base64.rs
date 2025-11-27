//! Base64 encoding and decoding.
//!
//! This module provides Base64 encoding/decoding following RFC 4648 and RFC 2045 (MIME) specifications.
//!
//! ## Available Variants
//!
//! - **Standard (RFC 4648)**: Uses alphabet A-Za-z0-9+/ with = padding, most common variant
//! - **URL-safe (RFC 4648 §5)**: Uses alphabet A-Za-z0-9-_ with = padding, safe for URLs/filenames
//! - **MIME (RFC 2045)**: Standard Base64 with line wrapping at 76 characters using CRLF
//! - **Custom**: Define your own 64-character alphabet with optional padding
//!
//! ## Choosing a Variant
//!
//! - **Standard**: Default choice for general purpose encoding (API tokens, data serialization)
//! - **URL-safe**: When encoding data for URLs, query parameters, or filenames (replaces +/ with -_)
//! - **MIME**: When encoding for email (MIME) or PEM certificates (requires line breaks)
//! - **Custom**: When you need specific alphabet requirements or compatibility with legacy systems
//!
//! ## Examples
//!
//! ```
//! use codec::{Base64, BinaryCodec};
//!
//! // Using the default Standard variant
//! let encoded = Base64::encode(b"Hello");
//! assert_eq!(encoded, "SGVsbG8=");
//!
//! // Using specific variants
//! assert_eq!(Base64::encode_url_safe(b"\xfb\xff"), "-_8=");
//! assert!(Base64::encode_mime(b"Hello").starts_with("SGVsbG8="));
//!
//! // Using codec instances
//! let codec = Base64::Standard;
//! let data = codec.encode(b"Hello");
//! assert_eq!(codec.decode(&data).unwrap(), b"Hello");
//! ```
//!
//! ## References
//!
//! - [RFC 4648 - The Base16, Base32, and Base64 Data Encodings](https://tools.ietf.org/html/rfc4648)
//! - [RFC 2045 - MIME Part One: Format of Internet Message Bodies](https://tools.ietf.org/html/rfc2045)

use crate::{
    BinaryCodec, BinaryCodecError, BinaryCodecs, DataEncodingDecodeError, DataEncodingSpecError,
    InvalidCharacterSetError,
};
use data_encoding::{
    BASE64, BASE64_MIME, BASE64_MIME_PERMISSIVE, BASE64URL, Encoding, Specification,
};
use snafu::{ResultExt, ensure};

/// Standard Base64 encoding (RFC 4648)
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Uses alphabet: `A-Z`, `a-z`, `0-9`, `+`, `/` with `=` padding.
/// This is the most common Base64 variant.
///
/// # Example
/// ```
/// use codec::{Base64Standard, BinaryCodec};
///
/// let encoded = Base64Standard.encode(b"Hello");
/// assert_eq!(encoded, "SGVsbG8=");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base64Standard;

/// URL-safe Base64 encoding (RFC 4648 Section 5)
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Uses alphabet: `A-Z`, `a-z`, `0-9`, `-`, `_` with `=` padding.
/// Safe for use in URLs and filenames (replaces `+/` with `-_`).
///
/// # Example
/// ```
/// use codec::{Base64UrlSafe, BinaryCodec};
///
/// let encoded = Base64UrlSafe.encode(b"\xfb\xff");
/// assert_eq!(encoded, "-_8=");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base64UrlSafe;

/// MIME Base64 encoding (RFC 2045)
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Standard Base64 with line wrapping at 76 characters using CRLF (`\r\n`).
/// Used in email (MIME) and PEM certificate formats.
///
/// # Example
/// ```
/// use codec::{Base64Mime, BinaryCodec};
///
/// let encoded = Base64Mime.encode(b"Hello");
/// assert!(encoded.starts_with("SGVsbG8="));
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base64Mime;

/// Custom Base64 encoding with user-defined alphabet
///
/// Allows custom 64-character alphabet and optional padding character.
/// Useful for specialized encoding schemes or legacy system compatibility.
///
/// # Example
/// ```
/// use codec::{Base64Custom, BinaryCodec};
///
/// let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
/// let codec = Base64Custom::new(alphabet).unwrap();
/// assert_eq!(codec.encode(b"Hello"), "SGVsbG8=");
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Base64Custom {
    encoding: Encoding,
}

/// Base64 encoding variants
///
/// Enum wrapper providing convenient access to all Base64 variants.
/// Supports Standard (RFC 4648), URL-safe, MIME, and custom alphabets.
///
/// # Example
/// ```
/// use codec::{Base64, BinaryCodec};
///
/// let codec = Base64::Standard;
/// assert_eq!(codec.encode(b"Hello"), "SGVsbG8=");
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum Base64 {
    #[default]
    Standard,
    UrlSafe,
    Mime,
    Custom(Base64Custom),
}

/// Marker trait for Base64 codec types
///
/// Implemented by all Base64 variants to enable generic programming
/// over different Base64 encoding schemes while maintaining type safety.
///
/// # Example
/// ```
/// use codec::{Base64Codec, Base64Standard, Base64UrlSafe, BinaryCodec};
///
/// fn encode_with_base64<C: Base64Codec>(codec: &C, data: &[u8]) -> String {
///     codec.encode(data)
/// }
///
/// let standard_result = encode_with_base64(&Base64Standard, b"Hello");
/// let url_safe_result = encode_with_base64(&Base64UrlSafe, b"Hello");
/// assert_eq!(standard_result, "SGVsbG8=");
/// assert_eq!(url_safe_result, "SGVsbG8=");
/// ```
pub trait Base64Codec: BinaryCodec {}

fn decode_internal(
    encoding: &Encoding,
    text: &str,
    variant: &str,
) -> Result<Vec<u8>, BinaryCodecError> {
    encoding.decode(text.as_bytes()).context(DataEncodingDecodeError { variant })
}

impl Base64Codec for Base64Standard {}
impl BinaryCodec for Base64Standard {
    fn encode(&self, bytes: &[u8]) -> String {
        BASE64.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&BASE64, text, "Base64 Standard")
    }
}

impl Base64Codec for Base64UrlSafe {}
impl BinaryCodec for Base64UrlSafe {
    fn encode(&self, bytes: &[u8]) -> String {
        BASE64URL.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&BASE64URL, text, "Base64 URL-safe")
    }
}

impl Base64Codec for Base64Mime {}
impl BinaryCodec for Base64Mime {
    fn encode(&self, bytes: &[u8]) -> String {
        // Use BASE64_MIME for encoding (proper MIME format with CRLF line wrapping)
        BASE64_MIME.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        // Use PERMISSIVE for decoding (accepts line breaks: LF and CRLF)
        decode_internal(&BASE64_MIME_PERMISSIVE, text, "Base64 MIME")
    }
}

impl Base64Codec for Base64Custom {}
impl BinaryCodec for Base64Custom {
    fn encode(&self, bytes: &[u8]) -> String {
        self.encoding.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&self.encoding, text, "Base64 Custom")
    }
}
impl Base64Custom {
    /// Create a custom Base64 codec with specified alphabet (default padding '=')
    ///
    /// # Arguments
    /// * `alphabet` - 64-character alphabet string
    ///
    /// # Errors
    /// Returns error if the alphabet configuration is invalid:
    ///
    /// - [`BinaryCodecError::InvalidCharacterSet`]
    ///   - Alphabet length is not 64 characters
    /// - [`BinaryCodecError::DataEncodingSpec`]
    ///   - Alphabet contains duplicate characters
    ///
    /// # Examples
    /// ```
    /// use codec::{Base64Custom, BinaryCodec};
    ///
    /// let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    /// let codec = Base64Custom::new(alphabet).unwrap();
    /// assert_eq!(codec.encode(b"Hello"), "SGVsbG8=");
    /// ```
    pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> {
        Self::new_with_padding(alphabet, Some('='))
    }

    /// Create a custom Base64 codec with specified alphabet and padding character
    ///
    /// # Arguments
    /// * `alphabet` - 64-character alphabet string
    /// * `padding` - Optional padding character (None for no padding)
    ///
    /// # Errors
    /// Returns error if the alphabet or padding configuration is invalid:
    ///
    /// - [`BinaryCodecError::InvalidCharacterSet`]
    ///   - Alphabet length is not 64 characters
    ///   - Padding character conflicts with alphabet
    /// - [`BinaryCodecError::DataEncodingSpec`]
    ///   - Alphabet contains duplicate characters
    ///
    /// # Examples
    /// ```
    /// use codec::{Base64Custom, BinaryCodec};
    ///
    /// let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    /// let codec = Base64Custom::new_with_padding(alphabet, Some('*')).unwrap();
    /// let encoded = codec.encode(b"Hello");
    /// assert!(encoded.ends_with('*'));
    /// ```
    pub fn new_with_padding(
        alphabet: &str,
        padding: Option<char>,
    ) -> Result<Self, BinaryCodecError> {
        ensure!(
            alphabet.len() == 64,
            InvalidCharacterSetError {
                encoding: BinaryCodecs::Base64,
                message: format!("Base64 alphabet must be 64 characters, got {}", alphabet.len()),
            }
        );

        ensure!(
            alphabet.is_ascii(),
            InvalidCharacterSetError {
                encoding: BinaryCodecs::Base64,
                message: "Base64 alphabet must contain only ASCII characters",
            }
        );

        let mut spec = Specification::new();
        spec.symbols.push_str(alphabet);
        spec.padding = padding;

        let encoding = spec.encoding().context(DataEncodingSpecError)?;

        Ok(Base64Custom { encoding })
    }
}

impl Base64Codec for Base64 {}
impl BinaryCodec for Base64 {
    fn encode(&self, bytes: &[u8]) -> String {
        match self {
            Base64::Standard => Base64Standard.encode(bytes),
            Base64::UrlSafe => Base64UrlSafe.encode(bytes),
            Base64::Mime => Base64Mime.encode(bytes),
            Base64::Custom(codec) => codec.encode(bytes),
        }
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        match self {
            Base64::Standard => Base64Standard.decode(text),
            Base64::UrlSafe => Base64UrlSafe.decode(text),
            Base64::Mime => Base64Mime.decode(text),
            Base64::Custom(codec) => codec.decode(text),
        }
    }
}
impl Base64 {
    /// Create a Base64 codec with custom alphabet (default padding '=')
    ///
    /// # Arguments
    /// * `alphabet` - 64-character alphabet string
    ///
    /// # Errors
    /// Returns error if the alphabet configuration is invalid:
    ///
    /// - [`BinaryCodecError::InvalidCharacterSet`]
    ///   - Alphabet length is not 64 characters
    /// - [`BinaryCodecError::DataEncodingSpec`]
    ///   - Alphabet contains duplicate characters
    pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> {
        Self::new_with_padding(alphabet, Some('='))
    }

    /// Create a Base64 codec with custom alphabet and padding character
    ///
    /// # Arguments
    /// * `alphabet` - 64-character alphabet string
    /// * `padding` - Optional padding character (None for no padding)
    ///
    /// # Errors
    /// Returns error if the alphabet or padding configuration is invalid:
    ///
    /// - [`BinaryCodecError::InvalidCharacterSet`]
    ///   - Alphabet length is not 64 characters
    ///   - Padding character conflicts with alphabet
    /// - [`BinaryCodecError::DataEncodingSpec`]
    ///   - Alphabet contains duplicate characters
    pub fn new_with_padding(
        alphabet: &str,
        padding: Option<char>,
    ) -> Result<Self, BinaryCodecError> {
        Base64Custom::new_with_padding(alphabet, padding).map(Base64::Custom)
    }

    /// Encode bytes to Base64 string (standard variant)
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base64;
    ///
    /// assert_eq!(Base64::encode(b"Hello"), "SGVsbG8=");
    /// ```
    pub fn encode(bytes: &[u8]) -> String {
        Base64Standard.encode(bytes)
    }

    /// Encode bytes to standard Base64 string
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base64;
    ///
    /// assert_eq!(Base64::encode_standard(b"Hello"), "SGVsbG8=");
    /// ```
    pub fn encode_standard(bytes: &[u8]) -> String {
        Base64Standard.encode(bytes)
    }

    /// Encode bytes to URL-safe Base64 string
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base64;
    ///
    /// assert_eq!(Base64::encode_url_safe(b"Hello"), "SGVsbG8=");
    /// ```
    pub fn encode_url_safe(bytes: &[u8]) -> String {
        Base64UrlSafe.encode(bytes)
    }

    /// Encode bytes to MIME Base64 string (with line wrapping)
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base64;
    ///
    /// let encoded = Base64::encode_mime(b"Hello");
    /// assert!(encoded.starts_with("SGVsbG8="));
    /// ```
    pub fn encode_mime(bytes: &[u8]) -> String {
        Base64Mime.encode(bytes)
    }

    /// Decode Base64 string to bytes (standard variant)
    ///
    /// # Errors
    /// Returns error if decoding fails:
    ///
    /// - [`BinaryCodecError::DataEncodingDecode`]
    ///   - Invalid Base64 characters
    ///   - Incorrect padding
    ///   - Malformed encoding
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base64;
    ///
    /// assert_eq!(Base64::decode("SGVsbG8=").unwrap(), b"Hello");
    /// ```
    pub fn decode(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base64Standard.decode(text)
    }

    /// Decode Base64 string to bytes (accepts standard format)
    ///
    /// # Errors
    /// Returns error if decoding fails:
    ///
    /// - [`BinaryCodecError::DataEncodingDecode`]
    ///   - Invalid Base64 characters
    ///   - Incorrect padding
    ///   - Malformed encoding
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base64;
    ///
    /// assert_eq!(Base64::decode_standard("SGVsbG8=").unwrap(), b"Hello");
    /// ```
    pub fn decode_standard(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base64Standard.decode(text)
    }

    /// Decode URL-safe Base64 string to bytes
    ///
    /// # Errors
    /// Returns error if decoding fails:
    ///
    /// - [`BinaryCodecError::DataEncodingDecode`]
    ///   - Invalid URL-safe Base64 characters
    ///   - Incorrect padding
    ///   - Malformed encoding
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base64;
    ///
    /// assert_eq!(Base64::decode_url_safe("SGVsbG8=").unwrap(), b"Hello");
    /// ```
    pub fn decode_url_safe(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base64UrlSafe.decode(text)
    }

    /// Decode MIME Base64 string to bytes (accepts line breaks)
    ///
    /// Accepts both CRLF and LF line breaks.
    ///
    /// # Errors
    /// Returns error if decoding fails:
    ///
    /// - [`BinaryCodecError::DataEncodingDecode`]
    ///   - Invalid Base64 characters
    ///   - Incorrect padding
    ///   - Malformed encoding
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base64;
    ///
    /// let encoded = Base64::encode_mime(b"Hello");
    /// assert_eq!(Base64::decode_mime(&encoded).unwrap(), b"Hello");
    /// ```
    pub fn decode_mime(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base64Mime.decode(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        assert_eq!(Base64::encode_standard(b"Hello"), "SGVsbG8=");
        assert_eq!(Base64::encode_standard(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
        assert_eq!(Base64::encode_standard(b"\x00\xFF"), "AP8=");
    }

    #[test]
    fn test_base64_decode() {
        assert_eq!(Base64::decode_standard("SGVsbG8=").unwrap(), b"Hello");
        assert_eq!(Base64::decode_standard("SGVsbG8sIFdvcmxkIQ==").unwrap(), b"Hello, World!");
        assert_eq!(Base64::decode_standard("AP8=").unwrap(), b"\x00\xFF");
    }

    #[test]
    fn test_base64_url_encode() {
        assert_eq!(Base64::encode_url_safe(b"Hello"), "SGVsbG8=");
        assert_eq!(Base64::encode_url_safe(b"\xfb\xff"), "-_8="); // Uses - and _ instead of + and /
    }

    #[test]
    fn test_base64_url_decode() {
        assert_eq!(Base64::decode_url_safe("SGVsbG8=").unwrap(), b"Hello");
        assert_eq!(Base64::decode_url_safe("-_8=").unwrap(), b"\xfb\xff");
    }

    #[test]
    fn test_base64_mime_encode() {
        // MIME wraps at 76 characters
        let long_input = b"The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog.";
        let encoded = Base64::encode_mime(long_input);
        assert!(encoded.contains('\r') || encoded.contains('\n'));
    }

    #[test]
    fn test_base64_decode_invalid() {
        let result = Base64::decode_standard("Invalid!@#");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    #[test]
    fn test_base64_custom_alphabet() {
        let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let codec = Base64::new(alphabet).unwrap();
        assert_eq!(codec.encode(b"Hello"), "SGVsbG8=");
        assert_eq!(codec.decode("SGVsbG8=").unwrap(), b"Hello");
    }

    #[test]
    fn test_base64_invalid_alphabet_length() {
        let result = Base64::new("ABC"); // Only 3 chars
        assert!(matches!(result, Err(BinaryCodecError::InvalidCharacterSet { .. })));
    }

    #[test]
    fn test_base64_with_padding() {
        let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let codec = Base64::new_with_padding(alphabet, Some('=')).unwrap();
        assert_eq!(codec.encode(b"Hello"), "SGVsbG8=");

        let codec_no_pad = Base64::new_with_padding(alphabet, None).unwrap();
        assert_eq!(codec_no_pad.encode(b"Hello"), "SGVsbG8");
    }

    #[test]
    fn test_base64_codec_constants() {
        // BASE64_STANDARD
        assert_eq!(Base64Standard.encode(b"Hello"), "SGVsbG8=");
        assert_eq!(Base64Standard.decode("SGVsbG8=").unwrap(), b"Hello");

        // BASE64_URL_SAFE
        assert_eq!(Base64UrlSafe.encode(b"Hello"), "SGVsbG8=");
        assert_eq!(Base64UrlSafe.decode("SGVsbG8=").unwrap(), b"Hello");

        // BASE64_MIME - includes line breaks
        let encoded = Base64Mime.encode(b"Hello");
        assert!(encoded.starts_with("SGVsbG8="));
        assert_eq!(Base64Mime.decode(&encoded).unwrap(), b"Hello");
    }

    #[test]
    fn test_base64_codec_as_trait_object() {
        // Test that constants can be used where BinaryCodec trait is expected
        fn encode_with_codec(codec: &dyn BinaryCodec, data: &[u8]) -> String {
            codec.encode(data)
        }

        assert_eq!(encode_with_codec(&Base64Standard, b"Hi"), "SGk=");
        assert_eq!(encode_with_codec(&Base64UrlSafe, b"Hi"), "SGk=");

        // MIME includes line breaks even for short inputs
        let mime_result = encode_with_codec(&Base64Mime, b"Hi");
        assert!(mime_result.starts_with("SGk="));
    }

    #[test]
    fn test_base64_empty_bytes() {
        assert_eq!(Base64::encode_standard(b""), "");
        assert_eq!(Base64::decode_standard("").unwrap(), b"");

        assert_eq!(Base64::encode_url_safe(b""), "");
        assert_eq!(Base64::decode_url_safe("").unwrap(), b"");
    }

    #[test]
    fn test_base64_padding_variations() {
        // 1 byte (2 padding chars: ==)
        assert_eq!(Base64::encode_standard(b"a"), "YQ==");
        assert_eq!(Base64::decode_standard("YQ==").unwrap(), b"a");

        // 2 bytes (1 padding char: =)
        assert_eq!(Base64::encode_standard(b"ab"), "YWI=");
        assert_eq!(Base64::decode_standard("YWI=").unwrap(), b"ab");

        // 3 bytes (no padding) - exact multiple of 3
        assert_eq!(Base64::encode_standard(b"abc"), "YWJj");
        assert_eq!(Base64::decode_standard("YWJj").unwrap(), b"abc");

        // 4 bytes (2 padding chars: ==)
        assert_eq!(Base64::encode_standard(b"abcd"), "YWJjZA==");
        assert_eq!(Base64::decode_standard("YWJjZA==").unwrap(), b"abcd");

        // 5 bytes (1 padding char: =)
        assert_eq!(Base64::encode_standard(b"abcde"), "YWJjZGU=");
        assert_eq!(Base64::decode_standard("YWJjZGU=").unwrap(), b"abcde");

        // 6 bytes (no padding) - exact multiple of 3
        assert_eq!(Base64::encode_standard(b"abcdef"), "YWJjZGVm");
        assert_eq!(Base64::decode_standard("YWJjZGVm").unwrap(), b"abcdef");
    }

    #[test]
    fn test_base64_duplicate_alphabet() {
        // Alphabet with duplicate character (64 'A's)
        let result =
            Base64::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingSpec { .. })));
    }

    // === Comprehensive test cases ===

    #[test]
    fn test_base64_all_bytes() {
        // Test encoding/decoding all possible byte values (0x00-0xFF)
        let all_bytes: Vec<u8> = (0..=255).collect();
        let encoded = Base64::encode_standard(&all_bytes);
        let decoded = Base64::decode_standard(&encoded).unwrap();
        assert_eq!(decoded, all_bytes);
    }

    #[test]
    fn test_base64_standard_vs_url_safe_alphabet() {
        // Test data that produces + and / in standard encoding
        // 0xfb = 11111011, 0xff = 11111111 -> produces +/8= in standard
        let data = b"\xfb\xff";

        let standard = Base64::encode_standard(data);
        let url_safe = Base64::encode_url_safe(data);

        // Standard uses +/
        assert_eq!(standard, "+/8=");
        // URL-safe uses -_
        assert_eq!(url_safe, "-_8=");

        // Cross-decoding should fail
        assert!(Base64::decode_standard(&url_safe).is_err());
        assert!(Base64::decode_url_safe(&standard).is_err());
    }

    #[test]
    fn test_base64_mime_line_wrapping() {
        // MIME wraps at 76 characters (not including CRLF)
        // 57 bytes encodes to 76 characters (57 * 4/3 = 76)
        let input = vec![0u8; 57];
        let encoded = Base64::encode_mime(&input);

        // Should contain line break (CRLF)
        assert!(encoded.contains("\r\n"), "MIME encoding should contain CRLF");

        // First line should be exactly 76 chars (excluding CRLF)
        let first_line = encoded.split("\r\n").next().unwrap();
        assert_eq!(first_line.len(), 76, "First line should be 76 characters");

        // Decoding should work
        assert_eq!(Base64::decode_mime(&encoded).unwrap(), input);
    }

    #[test]
    fn test_base64_mime_multiple_line_wraps() {
        // Test data that requires multiple line wraps
        // 200 bytes encodes to ~267 chars, requiring 3+ lines
        let input = vec![0u8; 200];
        let encoded = Base64::encode_mime(&input);

        let lines: Vec<&str> = encoded.split("\r\n").collect();
        assert!(
            lines.len() >= 4,
            "Should have at least 4 lines (3 full lines + 1 partial + empty last line)"
        );

        // All non-empty lines except the last data-containing line should be 76 chars
        // The last line with data may be shorter
        let non_empty_lines: Vec<&str> = lines.iter().filter(|l| !l.is_empty()).copied().collect();
        for line in &non_empty_lines[..non_empty_lines.len() - 1] {
            assert_eq!(line.len(), 76, "Full lines should be 76 characters");
        }

        // Last line with data should be <= 76
        assert!(
            non_empty_lines.last().unwrap().len() <= 76,
            "Last line should be at most 76 characters"
        );

        // Round-trip
        assert_eq!(Base64::decode_mime(&encoded).unwrap(), input);
    }

    #[test]
    fn test_base64_large_data() {
        // Test with 1MB of data
        let large_data = vec![0xABu8; 1024 * 1024];
        let encoded = Base64::encode_standard(&large_data);
        let decoded = Base64::decode_standard(&encoded).unwrap();
        assert_eq!(decoded, large_data);
    }

    #[test]
    fn test_base64_utf8_multibyte() {
        // Test UTF-8 multibyte characters
        let text = "Hello 世界 🦀 Rust";
        let bytes = text.as_bytes();
        let encoded = Base64::encode_standard(bytes);
        let decoded = Base64::decode_standard(&encoded).unwrap();
        assert_eq!(decoded, bytes);
        assert_eq!(String::from_utf8(decoded).unwrap(), text);
    }

    #[test]
    fn test_base64_invalid_padding_missing() {
        // Missing padding (should fail in strict mode)
        let result = Base64::decode_standard("SGVsbG8"); // Missing =
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    #[test]
    fn test_base64_invalid_padding_wrong_position() {
        // Padding in wrong position
        let result = Base64::decode_standard("SG=sbG8=");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    #[test]
    fn test_base64_invalid_characters() {
        // Test various invalid characters
        let invalid_inputs = vec![
            "SGVs@bG8=",  // @ is invalid
            "SGVs bG8=",  // space is invalid in standard
            "SGVs\nbG8=", // newline is invalid in standard
            "SGVs!bG8=",  // ! is invalid
        ];

        for input in invalid_inputs {
            let result = Base64::decode_standard(input);
            assert!(
                matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })),
                "Input '{}' should fail",
                input
            );
        }
    }

    #[test]
    fn test_base64_mime_permissive_whitespace() {
        // MIME PERMISSIVE decoder accepts CRLF and LF (but not space or tab)
        let encoded = "SGVs\r\nbG8="; // CRLF in middle
        assert_eq!(Base64::decode_mime(encoded).unwrap(), b"Hello");

        let encoded = "SGVs\nbG8="; // LF only
        assert_eq!(Base64::decode_mime(encoded).unwrap(), b"Hello");

        // Note: BASE64_MIME_PERMISSIVE does NOT accept spaces or tabs
        // It only accepts newlines (LF, CRLF)
    }

    #[test]
    fn test_base64_custom_alphabet_various() {
        // Test custom alphabet with different character sets
        let custom_alphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/";
        let codec = Base64::new(custom_alphabet).unwrap();

        let data = b"Test data";
        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);

        // Should be different from standard encoding
        let standard_encoded = Base64::encode_standard(data);
        assert_ne!(encoded, standard_encoded);
    }

    #[test]
    fn test_base64_custom_padding_char() {
        let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        // Custom padding character '*'
        let codec = Base64::new_with_padding(alphabet, Some('*')).unwrap();
        let encoded = codec.encode(b"Hello");
        assert!(encoded.ends_with('*'), "Should use custom padding '*'");
        assert_eq!(codec.decode(&encoded).unwrap(), b"Hello");
    }

    #[test]
    fn test_base64_boundary_lengths() {
        // Test various input lengths around Base64 chunk boundaries
        for len in 0..=20 {
            let data = vec![0xAAu8; len];
            let encoded = Base64::encode_standard(&data);
            let decoded = Base64::decode_standard(&encoded).unwrap();
            assert_eq!(decoded, data, "Failed for length {}", len);
        }
    }

    #[test]
    fn test_base64_custom_invalid_padding_character() {
        let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        // Padding character that conflicts with alphabet should fail
        let result = Base64::new_with_padding(alphabet, Some('A')); // 'A' is in alphabet
        assert!(
            matches!(result, Err(BinaryCodecError::DataEncodingSpec { .. })),
            "Should reject padding character that conflicts with alphabet"
        );
    }
}
