//! UTF-8 text encoding and decoding.
//!
//! This module provides conversion between Unicode strings and UTF-8 byte sequences.
//!
//! ## Overview
//!
//! UTF-8 is the dominant character encoding for the web and modern software systems.
//! It's a variable-width encoding that uses 1-4 bytes per character, efficiently
//! representing ASCII (1 byte) while supporting the full Unicode range.
//!
//! Since Rust's `str` type is always valid UTF-8, encoding never fails.
//! Decoding offers two modes:
//!
//! - **Lossy mode** (default): Replaces invalid UTF-8 sequences with � (U+FFFD REPLACEMENT CHARACTER)
//! - **Strict mode**: Returns an error on invalid UTF-8 sequences
//!
//! ## Available Modes
//!
//! ### Lossy Mode (Recommended for user input)
//!
//! Use when you need guaranteed success and can tolerate replacement characters:
//!
//! ```
//! use kodogu_codec::{TextCodec, Utf8};
//!
//! // Always succeeds, even with invalid UTF-8
//! let text = Utf8.decode(&[0xFF, 0xFE]);
//! assert_eq!(text, "��");  // Invalid bytes replaced
//!
//! // Valid UTF-8 passes through unchanged
//! let text = Utf8.decode("Hello 世界".as_bytes());
//! assert_eq!(text, "Hello 世界");
//! ```
//!
//! ### Strict Mode (Recommended for protocol parsing)
//!
//! Use when invalid UTF-8 should be treated as an error:
//!
//! ```
//! use kodogu_codec::{TextCodec, Utf8};
//!
//! // Valid UTF-8 succeeds
//! assert_eq!(Utf8.decode_strict(b"Hello").unwrap(), "Hello");
//!
//! // Invalid UTF-8 returns error
//! assert!(Utf8.decode_strict(&[0xFF, 0xFE]).is_err());
//! ```
//!
//! ## Choosing a Mode
//!
//! - **Lossy mode**: User-facing content, file I/O, logging, tolerant parsing
//! - **Strict mode**: Network protocols, API validation, security-sensitive parsing
//!
//! ## Examples
//!
//! ```
//! use kodogu_codec::{TextCodec, Utf8};
//!
//! // Encoding (always succeeds - Rust strings are UTF-8)
//! let bytes = Utf8.encode("Hello 🦀");
//! assert_eq!(bytes, "Hello 🦀".as_bytes());
//!
//! // Decoding valid UTF-8
//! let text = Utf8.decode("Hello".as_bytes());
//! assert_eq!(text, "Hello");
//!
//! // Decoding with multi-byte characters
//! let text = Utf8.decode("日本語".as_bytes());
//! assert_eq!(text, "日本語");
//! ```
//!
//! ## References
//!
//! - [RFC 3629 - UTF-8, a transformation format of ISO 10646](https://tools.ietf.org/html/rfc3629)
//! - [Unicode Standard](https://www.unicode.org/standard/standard.html)

use crate::{TextCodec, TextCodecError};
use snafu::ResultExt;

/// UTF-8 text encoding codec
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Provides conversion between Unicode strings (`str`) and UTF-8 byte sequences.
/// Since Rust's `str` is already UTF-8, encoding always succeeds. Decoding can
/// operate in lossy mode (replacing invalid sequences with �) or strict mode
/// (returning errors).
///
/// # Usage
///
/// Use the [`Utf8`] for trait object access, or methods like
/// [`Utf8.encode()`], [`Utf8.decode()`], [`Utf8.decode_strict()`] for convenience.
///
/// # Examples
///
/// ```
/// use kodogu_codec::{TextCodec, Utf8};
///
/// // Encoding (always succeeds)
/// let bytes = Utf8.encode("Hello");
/// assert_eq!(bytes, b"Hello");
///
/// // Decoding (lossy mode)
/// let text = Utf8.decode(b"Hello");
/// assert_eq!(text, "Hello");
///
/// // Invalid UTF-8 replaced with �
/// let text = Utf8.decode(&[0xFF, 0xFE]);
/// assert_eq!(text, "��");
///
/// // Strict mode returns error on invalid UTF-8
/// assert!(Utf8.decode_strict(&[0xFF, 0xFE]).is_err());
/// ```
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Utf8;

impl TextCodec for Utf8 {
    fn encode(&self, text: &str) -> Vec<u8> {
        text.as_bytes().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> String {
        // Lossy mode: replace invalid UTF-8 sequences with �
        String::from_utf8_lossy(bytes).into_owned()
    }

    /// Encode text to bytes (strict mode - returns error if characters are unmappable)
    ///
    /// # Errors
    /// This method never fails because Rust strings are always valid UTF-8:
    ///
    /// - No error variants (provided for API consistency)
    fn encode_strict(&self, text: &str) -> Result<Vec<u8>, TextCodecError> {
        // UTF-8 encoding always succeeds for valid Rust strings
        Ok(text.as_bytes().to_vec())
    }

    /// Decode bytes to text (strict mode - returns error on invalid byte sequences)
    ///
    /// # Errors
    /// Returns error if decoding fails:
    ///
    /// - [`TextCodecError::Utf8Invalid`]
    ///   - Input contains invalid UTF-8 sequences
    fn decode_strict(&self, bytes: &[u8]) -> Result<String, TextCodecError> {
        // Strict mode: return error on invalid UTF-8
        String::from_utf8(bytes.to_vec()).context(crate::Utf8InvalidError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_methods() {
        // encode() - always succeeds
        assert_eq!(Utf8.encode("Hello"), b"Hello");
        assert_eq!(Utf8.encode("こんにちは"), "こんにちは".as_bytes());

        // decode() - lossy mode
        assert_eq!(Utf8.decode(b"Hello"), "Hello");
        assert_eq!(Utf8.decode(&[0xFF, 0xFE]), "��");

        // decode_strict() - strict mode
        assert_eq!(Utf8.decode_strict(b"Hello").unwrap(), "Hello");
        let result = Utf8.decode_strict(&[0xFF, 0xFE]);
        assert!(matches!(result, Err(TextCodecError::Utf8Invalid { .. })));
    }

    #[test]
    fn test_utf8_empty_string() {
        // Empty string
        assert_eq!(Utf8.encode(""), b"");
        assert_eq!(Utf8.decode(b""), "");
        assert_eq!(Utf8.decode_strict(b"").unwrap(), "");
    }

    #[test]
    fn test_utf8_long_string() {
        // Long string (1000 characters)
        let long_text = "Hello, 世界! ".repeat(100);
        let bytes = Utf8.encode(&long_text);
        assert_eq!(Utf8.decode(&bytes), long_text);
        assert_eq!(Utf8.decode_strict(&bytes).unwrap(), long_text);
    }

    #[test]
    fn test_utf8_partial_invalid_sequence() {
        // Valid text followed by invalid bytes
        let mut data = Vec::from("Hello");
        data.extend_from_slice(&[0xFF, 0xFE]);

        // Lossy mode replaces invalid with �
        let result = Utf8.decode(&data);
        assert_eq!(result, "Hello��");

        // Strict mode returns error
        let result = Utf8.decode_strict(&data);
        assert!(matches!(result, Err(TextCodecError::Utf8Invalid { .. })));
    }

    #[test]
    fn test_utf8_emoji_and_special_chars() {
        // Emoji, combining characters, various Unicode
        let text = "Hello 👋 🌍 こんにちは नमस्ते 你好";
        let bytes = Utf8.encode(text);
        assert_eq!(Utf8.decode(&bytes), text);
        assert_eq!(Utf8.decode_strict(&bytes).unwrap(), text);
    }

    #[test]
    fn test_utf8_four_byte_sequences() {
        // Rust strings don't contain surrogates, but test various code points
        let text = "𝕳𝖊𝖑𝖑𝖔"; // Mathematical bold characters (outside BMP)
        let bytes = Utf8.encode(text);
        assert_eq!(Utf8.decode(&bytes), text);
        assert_eq!(Utf8.decode_strict(&bytes).unwrap(), text);
    }

    #[test]
    fn test_utf8_invalid_continuation_byte() {
        // Invalid continuation byte
        let invalid = &[0xC2, 0x00]; // 0xC2 expects continuation byte, but 0x00 isn't

        // Lossy mode replaces with �
        let result = Utf8.decode(invalid);
        assert!(result.contains('�'));

        // Strict mode returns error
        assert!(Utf8.decode_strict(invalid).is_err());
    }

    #[test]
    fn test_utf8_overlong_encoding() {
        // Overlong encoding (security issue, should be rejected)
        let overlong = &[0xC0, 0x80]; // Overlong encoding of NULL

        // Both modes should handle this (std library rejects overlong)
        let result = Utf8.decode(overlong);
        assert!(result.contains('�'));
        assert!(Utf8.decode_strict(overlong).is_err());
    }

    #[test]
    fn test_utf8_can_be_used_as_dyn_trait_object() {
        // Verify that Utf8 can be used through the TextCodec trait object
        // This is important for runtime codec selection scenarios
        let codec: Box<dyn TextCodec> = get_text_codec_by_name("utf8").unwrap();

        // Verify basic encode/decode operations work through trait object
        let text = "Hello";
        let bytes = codec.encode(text);
        assert_eq!(bytes, text.as_bytes());

        let decoded = codec.decode(&bytes);
        assert_eq!(decoded, text);
    }

    fn get_text_codec_by_name(name: &str) -> Result<Box<dyn TextCodec>, TextCodecError> {
        match name {
            "utf8" => Ok(Box::new(Utf8)),
            _ => panic!("Unknown codec: {}", name),
        }
    }
}
