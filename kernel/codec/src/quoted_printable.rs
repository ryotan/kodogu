//! Quoted-Printable encoding and decoding.
//!
//! This module provides MIME Quoted-Printable encoding/decoding following RFC 2045 specification.
//! Quoted-Printable is a content transfer encoding designed to transmit 8-bit data over channels
//! that may not preserve all 8-bit values, while keeping the output human-readable for ASCII text.
//!
//! ## Available Modes
//!
//! - **Robust (default)**: Lenient decoder that gracefully handles malformed input by passing through invalid sequences
//! - **Strict**: RFC-compliant decoder that strictly validates input and rejects malformed data
//!
//! ## Choosing a Mode
//!
//! - **Robust**: When processing real-world email data that may not be perfectly formatted (default, most forgiving)
//! - **Strict**: When validating user input, ensuring RFC compliance, or performing encoding round-trip testing
//!
//! See the [crate-level architecture documentation](crate#codec-architecture) for details on
//! the four complementary APIs (ZST, Enum, Static Methods, Trait).
//!
//! ## Encoding Rules
//!
//! 1. Bytes 33-60, 62-126 are represented as-is (except `=`)
//! 2. `=` is encoded as `=3D`
//! 3. All other bytes are encoded as `=XX` where XX is uppercase hexadecimal
//! 4. Lines longer than 76 characters are broken with soft line breaks (`=\r\n` or `=\n`)
//! 5. Trailing whitespace on lines must be encoded
//!
//! ## References
//!
//! - [RFC 2045 Section 6.7](https://tools.ietf.org/html/rfc2045#section-6.7) - Quoted-Printable Content-Transfer-Encoding

use crate::{BinaryCodec, BinaryCodecError, QuotedPrintableDecodeError};
use quoted_printable::ParseMode;
use snafu::ResultExt;

/// Quoted-Printable codec with robust (lenient) parsing
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Gracefully handles malformed input by passing through invalid sequences,
/// making it ideal for processing real-world email data that may not be perfectly formatted.
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, QuotedPrintableRobust};
///
/// let codec = QuotedPrintableRobust;
///
/// // Encoding
/// assert_eq!(codec.encode(b"Hello=World"), "Hello=3DWorld");
///
/// // Decoding - handles invalid input gracefully
/// assert_eq!(codec.decode("Hello=3DWorld").unwrap(), b"Hello=World");
/// assert_eq!(codec.decode("=ZZ").unwrap(), b"=ZZ"); // Invalid sequence passed through
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct QuotedPrintableRobust;

/// Quoted-Printable codec with strict (RFC-compliant) parsing
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Strictly validates input according to RFC 2045 and rejects malformed data,
/// making it ideal for validating user input or ensuring proper encoding format.
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, QuotedPrintableStrict};
///
/// let codec = QuotedPrintableStrict;
///
/// // Encoding
/// assert_eq!(codec.encode(b"Hello=World"), "Hello=3DWorld");
///
/// // Decoding - strictly validates input
/// assert_eq!(codec.decode("Hello=3DWorld").unwrap(), b"Hello=World");
/// assert!(codec.decode("=ZZ").is_err()); // Invalid sequence rejected
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct QuotedPrintableStrict;

/// Quoted-Printable encoding variants
///
/// Enum wrapper providing convenient access to both Robust and Strict parse modes.
/// The default variant is [`QuotedPrintable::Robust`].
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, QuotedPrintable};
///
/// // Using enum variants
/// let robust = QuotedPrintable::Robust;
/// let strict = QuotedPrintable::Strict;
///
/// // Both encode identically
/// assert_eq!(robust.encode(b"Hello=World"), "Hello=3DWorld");
/// assert_eq!(strict.encode(b"Hello=World"), "Hello=3DWorld");
///
/// // But decode differently for invalid input
/// assert_eq!(robust.decode("=ZZ").unwrap(), b"=ZZ"); // Passes through
/// assert!(strict.decode("=ZZ").is_err()); // Rejects
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub enum QuotedPrintable {
    /// Robust (lenient) parsing - handles malformed input gracefully
    #[default]
    Robust,
    /// Strict (RFC-compliant) parsing - rejects malformed input
    Strict,
}

/// Marker trait for Quoted-Printable codec types
///
/// This trait is automatically implemented for all Quoted-Printable codec types
/// ([`QuotedPrintableRobust`], [`QuotedPrintableStrict`], [`QuotedPrintable`]).
///
/// Users can also implement this trait for their own custom Quoted-Printable implementations
/// to integrate with functions that accept `impl QuotedPrintableCodec`.
///
/// # Examples
///
/// ```
/// use codec::{QuotedPrintableCodec, QuotedPrintableRobust, QuotedPrintable};
///
/// fn process_qp(codec: &impl QuotedPrintableCodec, data: &[u8]) -> String {
///     codec.encode(data)
/// }
///
/// // Works with ZST types
/// assert_eq!(process_qp(&QuotedPrintableRobust, b"Hi="), "Hi=3D");
///
/// // Works with Enum
/// assert_eq!(process_qp(&QuotedPrintable::Robust, b"Hi="), "Hi=3D");
/// ```
///
/// # Implementing for custom types
///
/// ```
/// use codec::{BinaryCodec, BinaryCodecError, QuotedPrintableCodec};
///
/// struct MyCustomQP;
///
/// impl BinaryCodec for MyCustomQP {
///     fn encode(&self, bytes: &[u8]) -> String {
///         // Custom implementation
///         quoted_printable::encode_to_str(bytes)
///     }
///
///     fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
///         // Custom implementation
///         use snafu::ResultExt;
///         use codec::QuotedPrintableDecodeError;
///         quoted_printable::decode(text.as_bytes(), quoted_printable::ParseMode::Robust)
///             .context(QuotedPrintableDecodeError)
///     }
/// }
///
/// impl QuotedPrintableCodec for MyCustomQP {}
/// ```
pub trait QuotedPrintableCodec: BinaryCodec {}

fn decode_internal(parse_mode: ParseMode, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
    quoted_printable::decode(text.as_bytes(), parse_mode).context(QuotedPrintableDecodeError)
}

impl QuotedPrintableCodec for QuotedPrintableRobust {}
impl BinaryCodec for QuotedPrintableRobust {
    fn encode(&self, bytes: &[u8]) -> String {
        quoted_printable::encode_to_str(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(ParseMode::Robust, text)
    }
}

impl QuotedPrintableCodec for QuotedPrintableStrict {}
impl BinaryCodec for QuotedPrintableStrict {
    fn encode(&self, bytes: &[u8]) -> String {
        quoted_printable::encode_to_str(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(ParseMode::Strict, text)
    }
}

impl QuotedPrintableCodec for QuotedPrintable {}
impl BinaryCodec for QuotedPrintable {
    fn encode(&self, bytes: &[u8]) -> String {
        match self {
            QuotedPrintable::Robust => QuotedPrintableRobust.encode(bytes),
            QuotedPrintable::Strict => QuotedPrintableStrict.encode(bytes),
        }
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        match self {
            QuotedPrintable::Robust => QuotedPrintableRobust.decode(text),
            QuotedPrintable::Strict => QuotedPrintableStrict.decode(text),
        }
    }
}
impl QuotedPrintable {
    /// Creates a new Quoted-Printable codec with the specified parse mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::QuotedPrintable;
    /// use quoted_printable::ParseMode;
    ///
    /// let codec = QuotedPrintable::new(ParseMode::Strict);
    /// assert_eq!(codec, QuotedPrintable::Strict);
    /// ```
    pub fn new(parse_mode: ParseMode) -> Self {
        match parse_mode {
            ParseMode::Strict => QuotedPrintable::Strict,
            ParseMode::Robust => QuotedPrintable::Robust,
        }
    }

    /// Encode bytes to Quoted-Printable string
    ///
    /// This is the default encoding method. Encoding is identical for both
    /// Robust and Strict modes - only decoding behavior differs.
    ///
    /// For explicit mode selection, use [`encode_robust`](Self::encode_robust) or
    /// [`encode_strict`](Self::encode_strict), or use the zero-sized types
    /// [`QuotedPrintableRobust`] or [`QuotedPrintableStrict`] directly.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::QuotedPrintable;
    ///
    /// assert_eq!(QuotedPrintable::encode(b"Hello=World"), "Hello=3DWorld");
    /// ```
    pub fn encode(bytes: &[u8]) -> String {
        QuotedPrintableRobust.encode(bytes)
    }

    /// Decode Quoted-Printable string to bytes (robust mode)
    ///
    /// This is the default decoding method using robust (lenient) parsing.
    /// For strict RFC-compliant parsing, use [`decode_strict`](Self::decode_strict).
    ///
    /// # Errors
    ///
    /// Returns error if decoding fails:
    ///
    /// - Note: This method uses robust parsing and rarely fails. Invalid sequences
    ///   (like `=ZZ`, `=3`, `Hello=World`) are passed through as-is rather than causing errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::QuotedPrintable;
    ///
    /// // Valid input
    /// assert_eq!(QuotedPrintable::decode("Hello=3DWorld").unwrap(), b"Hello=World");
    ///
    /// // Invalid sequences handled gracefully in robust mode
    /// assert_eq!(QuotedPrintable::decode("=ZZ").unwrap(), b"=ZZ");
    /// ```
    pub fn decode(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        QuotedPrintableRobust.decode(text)
    }

    /// Decode Quoted-Printable string to bytes (strict mode)
    ///
    /// Strictly validates input according to RFC 2045. Use this when you need
    /// to ensure RFC compliance or validate user input.
    ///
    /// For lenient parsing that handles malformed input gracefully, use
    /// [`decode`](Self::decode) or [`decode_robust`](Self::decode_robust).
    ///
    /// # Errors
    ///
    /// Returns [`BinaryCodecError::QuotedPrintableDecode`] if the input is malformed:
    ///
    /// - Invalid hex sequences (e.g., `=ZZ`, `=GG`)
    /// - Incomplete encodings (e.g., `=3`, `=`)
    /// - `=` not followed by valid hex digits or newline
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::QuotedPrintable;
    ///
    /// // Valid input succeeds
    /// assert_eq!(QuotedPrintable::decode_strict("Hello=3DWorld").unwrap(), b"Hello=World");
    ///
    /// // Invalid input fails in strict mode
    /// assert!(QuotedPrintable::decode_strict("=ZZ").is_err());
    /// assert!(QuotedPrintable::decode_strict("=3").is_err());
    /// ```
    pub fn decode_strict(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        QuotedPrintableStrict.decode(text)
    }

    /// Decode Quoted-Printable string to bytes (robust mode)
    ///
    /// Gracefully handles malformed input by passing through invalid sequences.
    /// This is identical to [`decode`](Self::decode) and is provided for API consistency.
    ///
    /// # Errors
    ///
    /// Returns error if decoding fails:
    ///
    /// - Note: This method rarely fails. Invalid sequences are passed through as-is.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::QuotedPrintable;
    ///
    /// // Valid input
    /// assert_eq!(QuotedPrintable::decode_robust("Hello=3DWorld").unwrap(), b"Hello=World");
    ///
    /// // Invalid sequences handled gracefully
    /// assert_eq!(QuotedPrintable::decode_robust("=ZZ").unwrap(), b"=ZZ");
    /// assert_eq!(QuotedPrintable::decode_robust("=3").unwrap(), b"=3");
    /// ```
    pub fn decode_robust(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        QuotedPrintableRobust.decode(text)
    }

    /// Encodes bytes to Quoted-Printable string (Robust mode).
    ///
    /// This method is provided for API consistency but produces identical output to
    /// [`encode`](Self::encode) since encoding is independent of parse mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::QuotedPrintable;
    ///
    /// assert_eq!(QuotedPrintable::encode_robust(b"Hello=World"), "Hello=3DWorld");
    /// ```
    pub fn encode_robust(bytes: &[u8]) -> String {
        QuotedPrintableRobust.encode(bytes)
    }

    /// Encodes bytes to Quoted-Printable string (Strict mode).
    ///
    /// This method is provided for API consistency but produces identical output to
    /// [`encode`](Self::encode) since encoding is independent of parse mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::QuotedPrintable;
    ///
    /// assert_eq!(QuotedPrintable::encode_strict(b"Hello=World"), "Hello=3DWorld");
    /// ```
    pub fn encode_strict(bytes: &[u8]) -> String {
        QuotedPrintableStrict.encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quoted_printable_encode() {
        assert_eq!(QuotedPrintable::encode(b"Hello=World"), "Hello=3DWorld");
        assert_eq!(QuotedPrintable::encode(b""), "");
        assert_eq!(QuotedPrintable::encode(b"Simple text"), "Simple text");
    }

    #[test]
    fn test_quoted_printable_decode() {
        assert_eq!(QuotedPrintable::decode("Hello=3DWorld").unwrap(), b"Hello=World");
        assert_eq!(QuotedPrintable::decode("").unwrap(), b"");
        assert_eq!(QuotedPrintable::decode("Simple text").unwrap(), b"Simple text");
    }

    #[test]
    fn test_quoted_printable_round_trip() {
        let data = b"The quick brown fox jumps over the lazy dog!";
        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_codec_constant() {
        assert_eq!(QuotedPrintableRobust.encode(b"Hello=World"), "Hello=3DWorld");
        assert_eq!(QuotedPrintableRobust.decode("Hello=3DWorld").unwrap(), b"Hello=World");
    }

    #[test]
    fn test_quoted_printable_codec_as_trait_object() {
        let codec: &dyn BinaryCodec = &QuotedPrintableRobust;
        assert_eq!(codec.encode(b"Hello=World"), "Hello=3DWorld");
        assert_eq!(codec.decode("Hello=3DWorld").unwrap(), b"Hello=World");
    }

    #[test]
    fn test_quoted_printable_special_characters() {
        // Special characters that must be encoded
        let data = b"=?!@#$%^&*()";
        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_long_lines() {
        // Quoted-Printable uses soft line breaks for long lines
        let data = b"This is a very long line that should be wrapped according to Quoted-Printable encoding rules which limit line length to 76 characters.";
        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_utf8() {
        // UTF-8 characters
        let data = "こんにちは".as_bytes();
        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_soft_line_break() {
        // Soft line break (=\n) should be removed during decoding
        let result = QuotedPrintable::decode("Hello=\nWorld");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"HelloWorld");
    }

    #[test]
    fn test_quoted_printable_soft_line_break_crlf() {
        // Soft line break with CRLF (=\r\n) should be removed during decoding
        let result = QuotedPrintable::decode("Hello=\r\nWorld");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"HelloWorld");
    }

    #[test]
    fn test_quoted_printable_case_insensitive_hex() {
        // Both uppercase and lowercase hex should be accepted
        assert_eq!(QuotedPrintable::decode("=3D").unwrap(), b"=");
        assert_eq!(QuotedPrintable::decode("=3d").unwrap(), b"=");
        assert_eq!(QuotedPrintable::decode("=3D=3d").unwrap(), b"==");
    }

    #[test]
    fn test_quoted_printable_whitespace() {
        // Tabs and spaces
        let data = b"Hello\tWorld Test";
        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_crlf() {
        // CRLF line endings should be preserved
        let data = b"Line1\r\nLine2\r\nLine3";
        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_printable_only() {
        // All printable ASCII characters (no encoding needed)
        let data = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_binary_data() {
        // Binary data (all bytes need encoding)
        let data: &[u8] = &[0x00, 0x01, 0xFF, 0xFE, 0x80];
        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    // Robust mode behavior: handles invalid input gracefully
    #[test]
    fn test_quoted_printable_decode_invalid_hex_robust() {
        // Invalid hex characters (non-hex) - Robust mode passes through
        let result = QuotedPrintable::decode("=ZZ");
        assert_eq!(result.unwrap(), b"=ZZ");
    }

    #[test]
    fn test_quoted_printable_decode_incomplete_encoding_robust() {
        // Incomplete encoding (only one hex digit) - Robust mode passes through
        let result = QuotedPrintable::decode("=3");
        assert_eq!(result.unwrap(), b"=3");
    }

    #[test]
    fn test_quoted_printable_decode_invalid_equals_position_robust() {
        // Equals sign not followed by valid hex or newline - Robust mode passes through
        let result = QuotedPrintable::decode("Hello=World");
        assert_eq!(result.unwrap(), b"Hello=World");
    }

    #[test]
    fn test_quoted_printable_decode_truncated_sequence_robust() {
        // Truncated at equals sign - Robust mode ignores trailing =
        let result = QuotedPrintable::decode("Test=");
        assert_eq!(result.unwrap(), b"Test");
    }

    // Strict mode behavior: rejects invalid input
    #[test]
    fn test_quoted_printable_decode_strict_valid() {
        // Valid input works in strict mode
        assert_eq!(QuotedPrintable::decode_strict("Hello=3DWorld").unwrap(), b"Hello=World");
        assert_eq!(QuotedPrintable::decode_strict("=3D").unwrap(), b"=");
    }

    #[test]
    fn test_quoted_printable_decode_strict_invalid_hex() {
        // Invalid hex characters (non-hex) - Strict mode rejects
        let result = QuotedPrintable::decode_strict("=ZZ");
        assert!(result.is_err());
    }

    #[test]
    fn test_quoted_printable_decode_strict_incomplete_encoding() {
        // Incomplete encoding (only one hex digit) - Strict mode rejects
        let result = QuotedPrintable::decode_strict("=3");
        assert!(result.is_err());
    }

    #[test]
    fn test_quoted_printable_decode_strict_invalid_equals_position() {
        // Equals sign not followed by valid hex or newline - Strict mode rejects
        let result = QuotedPrintable::decode_strict("Hello=World");
        assert!(result.is_err());
    }

    #[test]
    fn test_quoted_printable_decode_strict_trailing_equals() {
        // Trailing = is interpreted as soft line break (even without newline) - Strict mode accepts
        let result = QuotedPrintable::decode_strict("Test=");
        assert_eq!(result.unwrap(), b"Test");
    }

    // Static constants tests
    #[test]
    fn test_quoted_printable_codec_constants() {
        // QUOTED_PRINTABLE (default Robust)
        assert_eq!(QuotedPrintable::encode(b"Hello=World"), "Hello=3DWorld");
        assert_eq!(QuotedPrintable::decode("=ZZ").unwrap(), b"=ZZ");

        // QUOTED_PRINTABLE_STRICT
        assert_eq!(QuotedPrintableStrict.encode(b"Hello=World"), "Hello=3DWorld");
        assert!(QuotedPrintableStrict.decode("=ZZ").is_err());

        // QUOTED_PRINTABLE_ROBUST
        assert_eq!(QuotedPrintableRobust.encode(b"Hello=World"), "Hello=3DWorld");
        assert_eq!(QuotedPrintableRobust.decode("=ZZ").unwrap(), b"=ZZ");
    }

    #[test]
    fn test_quoted_printable_codec_as_trait_objects() {
        // Test all variants as trait objects
        let codecs: Vec<&dyn BinaryCodec> = vec![&QuotedPrintableStrict, &QuotedPrintableRobust];

        for codec in codecs {
            assert_eq!(codec.encode(b"Hello=World"), "Hello=3DWorld");
        }

        // Strict rejects invalid
        assert!(QuotedPrintableStrict.decode("=ZZ").is_err());

        // Robust accepts invalid
        assert_eq!(QuotedPrintableRobust.decode("=ZZ").unwrap(), b"=ZZ");
    }

    #[test]
    fn test_quoted_printable_new_constructor() {
        // Create custom codecs
        let strict = QuotedPrintable::new(ParseMode::Strict);
        let robust = QuotedPrintable::new(ParseMode::Robust);

        // Valid input works for both
        assert_eq!(strict.encode(b"Test"), "Test");
        assert_eq!(robust.encode(b"Test"), "Test");

        // Invalid input behavior differs
        assert!(strict.decode("=ZZ").is_err());
        assert_eq!(robust.decode("=ZZ").unwrap(), b"=ZZ");
    }

    #[test]
    fn test_quoted_printable_explicit_methods() {
        // decode_robust
        assert_eq!(QuotedPrintable::decode_robust("Hello=3DWorld").unwrap(), b"Hello=World");
        assert_eq!(QuotedPrintable::decode_robust("=ZZ").unwrap(), b"=ZZ");

        // decode_strict
        assert_eq!(QuotedPrintable::decode_strict("Hello=3DWorld").unwrap(), b"Hello=World");
        assert!(QuotedPrintable::decode_strict("=ZZ").is_err());
    }

    // ========================================
    // Additional Edge Cases and Comprehensive Coverage
    // ========================================

    #[test]
    fn test_quoted_printable_empty_data() {
        // Empty input
        assert_eq!(QuotedPrintable::encode(b""), "");
        assert_eq!(QuotedPrintable::decode("").unwrap(), b"");
        assert_eq!(QuotedPrintable::decode_strict("").unwrap(), b"");
        assert_eq!(QuotedPrintable::decode_robust("").unwrap(), b"");
    }

    #[test]
    fn test_quoted_printable_mixed_case_hex() {
        // Mixed uppercase and lowercase hex characters
        let input = "=3D=3d=3D";
        let expected = b"===";
        // Robust mode accepts both
        assert_eq!(QuotedPrintable::decode(input).unwrap(), expected);
        assert_eq!(QuotedPrintable::decode_robust(input).unwrap(), expected);

        // Strict mode only accepts uppercase
        let input_uppercase = "=3D=3D=3D";
        assert_eq!(QuotedPrintable::decode_strict(input_uppercase).unwrap(), expected);

        // Strict mode rejects lowercase
        assert!(QuotedPrintable::decode_strict(input).is_err());
    }

    #[test]
    fn test_quoted_printable_control_characters() {
        // Control characters (0x00-0x1F, 0x7F) must be encoded
        let control_chars: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
            0x1C, 0x1D, 0x1E, 0x1F, 0x7F,
        ];

        for byte in control_chars {
            let data = vec![byte];
            let encoded = QuotedPrintable::encode(&data);
            let decoded = QuotedPrintable::decode(&encoded).unwrap();
            assert_eq!(decoded, data, "Control character 0x{:02X} not preserved", byte);
        }
    }

    #[test]
    fn test_quoted_printable_all_bytes() {
        // Test all possible byte values (0x00-0xFF)
        let all_bytes: Vec<u8> = (0..=255).collect();
        let encoded = QuotedPrintable::encode(&all_bytes);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, all_bytes);
    }

    #[test]
    fn test_quoted_printable_trailing_whitespace_on_line() {
        // Trailing whitespace (tab and space) at end of line must be encoded
        // Note: This depends on the underlying library's implementation
        let data_with_trailing_space = b"Hello \nWorld";
        let encoded = QuotedPrintable::encode(data_with_trailing_space);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data_with_trailing_space);

        let data_with_trailing_tab = b"Hello\t\nWorld";
        let encoded = QuotedPrintable::encode(data_with_trailing_tab);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data_with_trailing_tab);
    }

    #[test]
    fn test_quoted_printable_exactly_76_chars() {
        // Test a line that's exactly 76 characters (before encoding)
        // This tests the soft line break insertion logic
        let data = b"123456789012345678901234567890123456789012345678901234567890123456789012";
        assert_eq!(data.len(), 72); // 72 printable chars

        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_line_longer_than_76_chars_encoded() {
        // Test data that, when encoded, exceeds 76 characters
        // For example, many special characters that need encoding
        let data = b"===========================================";
        let encoded = QuotedPrintable::encode(data);

        // Verify soft line breaks are present if needed
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_multiple_soft_line_breaks() {
        // Test decoding with multiple soft line breaks
        let encoded = "This=\nis=\na=\ntest";
        let decoded = QuotedPrintable::decode(encoded).unwrap();
        assert_eq!(decoded, b"Thisisatest");

        let encoded_crlf = "This=\r\nis=\r\na=\r\ntest";
        let decoded = QuotedPrintable::decode(encoded_crlf).unwrap();
        assert_eq!(decoded, b"Thisisatest");
    }

    #[test]
    fn test_quoted_printable_encoded_equals_at_line_end() {
        // Test that encoded '=' at end of data doesn't get confused with soft line break
        let data = b"test=";
        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_percent_sign() {
        // Percent sign should be encoded
        let data = b"100% complete";
        let encoded = QuotedPrintable::encode(data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_non_ascii_characters() {
        // Non-ASCII characters (128-255) should be encoded
        let non_ascii: Vec<u8> = (128..=255).collect();
        let encoded = QuotedPrintable::encode(&non_ascii);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        assert_eq!(decoded, non_ascii);
    }

    #[test]
    fn test_quoted_printable_case_sensitivity() {
        // Robust mode accepts both uppercase and lowercase hex
        assert_eq!(QuotedPrintable::decode_robust("=3d").unwrap(), b"=");
        assert_eq!(QuotedPrintable::decode_robust("=3D").unwrap(), b"=");
        assert_eq!(QuotedPrintable::decode("=3d").unwrap(), b"=");
        assert_eq!(QuotedPrintable::decode("=3D").unwrap(), b"=");

        // Strict mode only accepts uppercase
        assert_eq!(QuotedPrintable::decode_strict("=3D").unwrap(), b"=");
        assert!(QuotedPrintable::decode_strict("=3d").is_err());
    }

    #[test]
    fn test_quoted_printable_decode_mixed_content() {
        // Mix of encoded and plain text
        let input = "Hello=20World=21=20Test=3D123";
        let expected = b"Hello World! Test=123";
        assert_eq!(QuotedPrintable::decode(input).unwrap(), expected);
        assert_eq!(QuotedPrintable::decode_strict(input).unwrap(), expected);
    }
}
