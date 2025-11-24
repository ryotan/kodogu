//! # KoDoGu Codec Library
//!
//! A comprehensive encoding and decoding library supporting multiple formats:
//! - UTF-8 text encoding

use std::string::FromUtf8Error;
use suzunari_error::suzunari_error;

mod utf8;

pub use utf8::Utf8;

/// # Codec Architecture
///
/// All binary codec modules in this crate follow a consistent architecture pattern
/// that provides flexibility and zero-cost abstractions:
///
/// ## Four Complementary APIs
///
/// ### 1. Zero-Sized Types (ZST) - Type-safe, zero-cost
///
/// Use these when the codec variant is known at compile time:
///
/// ```
/// use kodogu_codec::{BinaryCodec, HexLower, HexUpper};
///
/// // Zero-cost abstraction - no runtime overhead
/// let encoded = HexLower.encode(b"Hello");
/// assert_eq!(encoded, "48656c6c6f");
///
/// // Type-safe - only accepts lowercase during decode
/// let decoded = HexLower.decode("48656c6c6f").unwrap();
/// assert_eq!(decoded, b"Hello");
/// ```
///
/// ### 2. Enum Type - Runtime flexibility
///
/// Use this when the codec variant is determined at runtime:
///
/// ```
/// use kodogu_codec::{BinaryCodec, Hex};
///
/// // Runtime selection
/// let codec = match std::env::var("HEX_FORMAT").as_deref() {
///     Ok("upper") => Hex::Upper,
///     Ok("lower") => Hex::Lower,
///     _ => Hex::CaseInsensitive,
/// };
///
/// let encoded = codec.encode(b"Hello");
/// ```
///
/// ### 3. Static Methods - Convenience API
///
/// Use these for one-off encoding/decoding operations:
///
/// ```
/// use kodogu_codec::Hex;
///
/// // Convenient static methods
/// let encoded = Hex::encode(b"Hello");
/// assert_eq!(encoded, "48656c6c6f");
///
/// let decoded = Hex::decode("48656c6c6f").unwrap();
/// assert_eq!(decoded, b"Hello");
/// ```
///
/// ### 4. Codec-Specific Trait - Polymorphism
///
/// Use this to accept any variant of a specific codec family:
///
/// ```
/// use kodogu_codec::{HexCodec, HexLower, Hex};
///
/// fn process_with_hex(codec: &impl HexCodec, data: &[u8]) -> String {
///     codec.encode(data)
/// }
///
/// // Works with both ZST and Enum
/// assert_eq!(process_with_hex(&HexLower, b"Hi"), "4869");
/// assert_eq!(process_with_hex(&Hex::Upper, b"Hi"), "4869");
/// ```
///
/// ## Design Rationale
///
/// - **ZST Types**: Zero runtime cost, compile-time guarantees, optimal for known variants
/// - **Enum**: Flexible runtime selection, single dispatch, ergonomic pattern matching
/// - **Static Methods**: Quick one-off operations, minimal boilerplate
/// - **Traits**: Enable generic programming, dependency injection, trait objects
///
/// ## Performance Characteristics
///
/// - **Zero-Sized Types**: No memory overhead, fully inlined by compiler
/// - **Enum**: Single `match` dispatch overhead (typically optimized away)
/// - **Static Methods**: Delegates to ZST, same performance as direct ZST usage
/// - **Trait Objects**: Dynamic dispatch overhead (use generics when possible)
///
/// ## Which API to Use?
///
/// | Scenario | Recommended API | Example |
/// |----------|----------------|---------|
/// | Variant known at compile time | ZST | `HexLower.encode(data)` |
/// | Variant from config/environment | Enum | `Hex::Upper.encode(data)` |
/// | Quick one-off operation | Static method | `Hex::encode(data)` |
/// | Generic function parameter | Trait bound | `fn foo(codec: &impl HexCodec)` |
/// | Runtime polymorphism needed | Trait object | `Box<dyn BinaryCodec>` |
///
/// # Trait for binary-to-text codecs (Base64, Hex, Base32, Base58, etc.)
///
/// Encodes arbitrary binary data into text representation for safe transmission or storage
/// in text-based protocols (HTTP headers, JSON, URLs, etc.).
///
/// # Purpose
/// Binary-to-text encoding transforms arbitrary bytes into a restricted character set
/// (typically ASCII printable characters) to ensure safe transmission through systems
/// that may corrupt or misinterpret binary data.
///
/// # When to use
/// - Encoding binary data for transmission in text protocols (HTTP, JSON, XML)
/// - Embedding binary data in URLs or filenames
/// - Storing binary data in text-based formats
///
/// # Encoding behavior
/// - `encode()` always succeeds and returns a `String`
/// - `decode()` may fail on invalid input and returns `Result<Vec<u8>, BinaryCodecError>`
pub trait BinaryCodec {
    /// Encode bytes to string (always succeeds)
    fn encode(&self, bytes: &[u8]) -> String;

    /// Decode string to bytes (may fail on invalid input)
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError>;
}

/// Binary codec type for error messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryCodecs {
    Hex,
    Base32,
    Base58,
    Base64,
    Url,
}

/// Error type for binary codec operations (Base64, Hex, etc.)
#[suzunari_error]
#[snafu(visibility(pub), context(suffix(Error)))]
pub enum BinaryCodecError {
    // Common errors across encodings
    #[snafu(display("Invalid {encoding:?} character '{char}' at position {position}"))]
    InvalidCharacter {
        encoding: BinaryCodecs,
        char: char,
        position: usize,
    },

    #[snafu(display("Invalid {encoding:?} length: {length}"))]
    InvalidLength {
        encoding: BinaryCodecs,
        length: usize,
    },

    #[snafu(display("Invalid {encoding:?} padding"))]
    InvalidPadding { encoding: BinaryCodecs },

    #[snafu(display("Invalid {encoding:?} character set: {message}"))]
    InvalidCharacterSet {
        encoding: BinaryCodecs,
        message: String,
    },

    // Hex specific
    #[snafu(display("Odd length hex string: {length}"))]
    HexOddLength { length: usize },

    // URL encoding specific (for future url_encoding module)
    #[snafu(display("Invalid percent encoding at position {position}"))]
    UrlInvalidPercent { position: usize },

    #[snafu(display("Invalid UTF-8 in decoded URL: {source}"))]
    UrlInvalidUtf8 { source: FromUtf8Error },
}

/// Trait for text encoding codecs (UTF-8, Shift-JIS, EUC-JP, etc.)
///
/// Converts between Unicode strings (Rust's `str`) and byte sequences in various
/// character encodings. This is fundamentally different from binary-to-text encoding:
/// text encodings represent characters, not arbitrary bytes.
///
/// # Purpose
/// Text encoding transforms Unicode text into byte sequences according to specific
/// character set standards (UTF-8, Shift-JIS, etc.), enabling text interchange
/// between systems with different encoding conventions.
///
/// # When to use
/// - Converting text to/from legacy encodings (Shift-JIS, EUC-JP, Windows-31J)
/// - Reading/writing text files in non-UTF-8 encodings
/// - Interfacing with systems that require specific character encodings
///
/// # Encoding behavior
/// - `encode()` / `decode()`: Lossy mode - always succeeds by replacing unmappable/invalid sequences
/// - `encode_strict()` / `decode_strict()`: Strict mode - returns error on unmappable/invalid sequences
///
/// # Difference from BinaryCodec
/// - **BinaryCodec**: bytes → text for transmission (Base64, Hex)
/// - **TextCodec**: text → bytes for character encoding (UTF-8, Shift-JIS)
pub trait TextCodec {
    /// Encode text to bytes (lossy mode - replaces unmappable characters)
    fn encode(&self, text: &str) -> Vec<u8>;

    /// Decode bytes to text (lossy mode - replaces invalid sequences with �)
    fn decode(&self, bytes: &[u8]) -> String;

    /// Encode text to bytes (strict mode - returns error if characters are unmappable)
    fn encode_strict(&self, text: &str) -> Result<Vec<u8>, TextCodecError>;

    /// Decode bytes to text (strict mode - returns error on invalid byte sequences)
    fn decode_strict(&self, bytes: &[u8]) -> Result<String, TextCodecError>;
}

/// Encoding type for error messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextCodecs {
    Utf8,
}

/// Error type for text codec operations (UTF-8, Shift-JIS, etc.)
#[suzunari_error]
#[snafu(visibility(pub), context(suffix(Error)))]
pub enum TextCodecError {
    // UTF-8 specific
    #[snafu(display("Invalid UTF-8 sequence: {source}"))]
    Utf8Invalid { source: FromUtf8Error },

    // Invalid byte sequence in text encoding
    #[snafu(display("Invalid byte sequence for {encoding:?} at position {position}"))]
    InvalidSequence {
        encoding: TextCodecs,
        position: usize,
    },

    // Unmappable character in encoding
    #[snafu(display("Character '{char}' cannot be encoded as {encoding:?} (Unicode code point U+{code_point:04x}"))]
    UnmappableCharacter {
        encoding: TextCodecs,
        char: char,
        code_point: u32,
    },
}
