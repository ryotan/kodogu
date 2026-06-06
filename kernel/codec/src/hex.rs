//! Hex (Base16) encoding and decoding.
//!
//! This module provides hexadecimal encoding with lowercase, uppercase, case-insensitive, and custom alphabet support.
//!
//! ## Available Variants
//!
//! - **Lowercase**: Encodes to 0-9a-f, decodes only lowercase (strict)
//! - **Uppercase**: Encodes to 0-9A-F, decodes only uppercase (strict)
//! - **Case-Insensitive**: Encodes to lowercase, accepts any case during decoding (permissive)
//! - **Custom**: Define your own 16-character alphabet for specialized applications
//!
//! ## Choosing a Variant
//!
//! - **Lowercase**: When you need strict lowercase format (common in hashes, checksums)
//! - **Uppercase**: When you need strict uppercase format (some legacy protocols)
//! - **Case-Insensitive**: Most flexible default - encodes lowercase, accepts any case during decode
//! - **Custom**: When you need non-standard hex alphabets for specific protocols
//!
//! See the [crate-level architecture documentation](crate#codec-architecture) for details on
//! the four complementary APIs (ZST, Enum, Static Methods, Trait).
//!
//! ## References
//!
//! - [RFC 4648 - The Base16, Base32, and Base64 Data Encodings](https://tools.ietf.org/html/rfc4648)

use crate::{
    BinaryCodec, BinaryCodecError, BinaryCodecs, DataEncodingDecodeError, DataEncodingSpecError,
    HexOddLengthError, InvalidCharacterSetError,
};
use data_encoding::{Encoding, HEXLOWER, HEXLOWER_PERMISSIVE, HEXUPPER, Specification};
use suzunari_error::{ResultExt, ensure};

/// Hex codec that encodes as lowercase and accepts any case when decoding
///
/// This is a zero-sized type (ZST) with no runtime overhead.
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, HexCaseInsensitive};
///
/// let codec = HexCaseInsensitive;
///
/// // Encodes as lowercase
/// assert_eq!(codec.encode(&[0xAB, 0xCD]), "abcd");
///
/// // Decodes any case
/// assert_eq!(codec.decode("abcd").unwrap(), vec![0xAB, 0xCD]);
/// assert_eq!(codec.decode("ABCD").unwrap(), vec![0xAB, 0xCD]);
/// assert_eq!(codec.decode("AbCd").unwrap(), vec![0xAB, 0xCD]); // Mixed case
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HexCaseInsensitive;

/// Hex codec that only accepts lowercase characters
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Strict variant that rejects uppercase characters during decoding.
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, HexLower};
///
/// let codec = HexLower;
///
/// // Encodes as lowercase
/// assert_eq!(codec.encode(&[0xAB, 0xCD]), "abcd");
///
/// // Only accepts lowercase
/// assert_eq!(codec.decode("abcd").unwrap(), vec![0xAB, 0xCD]);
/// assert!(codec.decode("ABCD").is_err()); // Uppercase rejected
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HexLower;

/// Hex codec that only accepts uppercase characters
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Strict variant that rejects lowercase characters during decoding.
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, HexUpper};
///
/// let codec = HexUpper;
///
/// // Encodes as uppercase
/// assert_eq!(codec.encode(&[0xAB, 0xCD]), "ABCD");
///
/// // Only accepts uppercase
/// assert_eq!(codec.decode("ABCD").unwrap(), vec![0xAB, 0xCD]);
/// assert!(codec.decode("abcd").is_err()); // Lowercase rejected
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct HexUpper;

/// Hex codec with custom alphabet
///
/// Unlike the zero-sized codec types, this stores runtime configuration data.
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, HexCustom};
///
/// let codec = HexCustom::new("0123456789ABCDEF").unwrap();
/// assert_eq!(codec.encode(b"Hi"), "4869");
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HexCustom {
    encoding: Encoding,
}

/// Hex codec variants for runtime selection
///
/// Use this enum when the codec variant needs to be determined at runtime
/// (e.g., from configuration files, user input, or environment variables).
///
/// For compile-time known variants, prefer the zero-sized types
/// ([`HexLower`], [`HexUpper`], [`HexCaseInsensitive`], [`HexCustom`])
/// for better performance.
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, Hex};
///
/// // Runtime selection
/// let use_uppercase = true;
/// let codec = if use_uppercase {
///     Hex::Upper
/// } else {
///     Hex::Lower
/// };
///
/// let encoded = codec.encode(b"Hello");
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum Hex {
    /// Case-insensitive variant (encodes lowercase, decodes any case)
    #[default]
    CaseInsensitive,
    /// Lowercase-only variant
    Lower,
    /// Uppercase-only variant
    Upper,
    /// Custom alphabet variant
    Custom(HexCustom),
}

/// Marker trait for Hex-family codecs
///
/// This trait is automatically implemented for all Hex codec types
/// ([`HexLower`], [`HexUpper`], [`HexCaseInsensitive`], [`HexCustom`], [`Hex`]).
///
/// Users can also implement this trait for their own custom Hex implementations
/// to integrate with functions that accept `impl HexCodec`.
///
/// # Examples
///
/// ```
/// use codec::{HexCodec, HexLower, Hex};
///
/// fn process_hex(codec: &impl HexCodec, data: &[u8]) -> String {
///     codec.encode(data)
/// }
///
/// // Works with ZST types
/// assert_eq!(process_hex(&HexLower, b"Hi"), "4869");
///
/// // Works with Enum
/// assert_eq!(process_hex(&Hex::Upper, b"Hi"), "4869");
/// ```
///
/// # Implementing for custom types
///
/// ```ignore
/// use codec::{BinaryCodec, BinaryCodecError, HexCodec};
///
/// struct MyCustomHex;
///
/// impl BinaryCodec for MyCustomHex {
///     fn encode(&self, bytes: &[u8]) -> String {
///         // Custom implementation
///         hex::encode(bytes)
///     }
///
///     fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
///         // Custom implementation
///         hex::decode(text).map_err(|_| unimplemented!())
///     }
/// }
///
/// impl HexCodec for MyCustomHex {}
/// ```
pub trait HexCodec: BinaryCodec {}

fn decode_internal(
    encoding: &Encoding,
    text: &str,
    variant: &str,
) -> Result<Vec<u8>, BinaryCodecError> {
    ensure!(text.len().is_multiple_of(2), HexOddLengthError { length: text.len() });
    encoding.decode(text.as_bytes()).context(DataEncodingDecodeError { variant })
}

impl HexCodec for HexCaseInsensitive {}
impl BinaryCodec for HexCaseInsensitive {
    fn encode(&self, bytes: &[u8]) -> String {
        HEXLOWER.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&HEXLOWER_PERMISSIVE, text, "Hex")
    }
}

impl HexCodec for HexLower {}
impl BinaryCodec for HexLower {
    fn encode(&self, bytes: &[u8]) -> String {
        HEXLOWER.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&HEXLOWER, text, "Hex Lower")
    }
}

impl HexCodec for HexUpper {}
impl BinaryCodec for HexUpper {
    fn encode(&self, bytes: &[u8]) -> String {
        HEXUPPER.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&HEXUPPER, text, "Hex Upper")
    }
}

impl HexCodec for HexCustom {}
impl BinaryCodec for HexCustom {
    fn encode(&self, bytes: &[u8]) -> String {
        self.encoding.encode(bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(&self.encoding, text, "Hex Custom")
    }
}
impl HexCustom {
    /// Create a hex codec with custom alphabet
    ///
    /// # Arguments
    ///
    /// * `alphabet` - 16-character alphabet string (e.g., "0123456789ABCDEF")
    ///
    /// # Errors
    /// Returns error if the alphabet configuration is invalid:
    ///
    /// - [`BinaryCodecError::InvalidCharacterSet`]
    ///   - Alphabet length is not exactly 16 characters
    /// - [`BinaryCodecError::DataEncodingSpec`]
    ///   - Alphabet contains duplicate characters
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::{BinaryCodec, HexCustom};
    ///
    /// // Standard alphabet
    /// let codec = HexCustom::new("0123456789ABCDEF").unwrap();
    /// assert_eq!(codec.encode(b"Hi"), "4869");
    ///
    /// // Custom alphabet
    /// let codec = HexCustom::new("0123456789abcdef").unwrap();
    /// assert_eq!(codec.encode(b"Hi"), "4869");
    /// ```
    pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> {
        ensure!(
            alphabet.len() == 16,
            InvalidCharacterSetError {
                encoding: BinaryCodecs::Hex,
                message: format!("Hex alphabet must be 16 characters, got {}", alphabet.len()),
            }
        );

        ensure!(
            alphabet.is_ascii(),
            InvalidCharacterSetError {
                encoding: BinaryCodecs::Hex,
                message: "Hex alphabet must contain only ASCII characters",
            }
        );

        let mut spec = Specification::new();
        spec.symbols.push_str(alphabet);

        let encoding = spec.encoding().context(DataEncodingSpecError)?;

        Ok(HexCustom { encoding })
    }
}

impl HexCodec for Hex {}
impl BinaryCodec for Hex {
    fn encode(&self, bytes: &[u8]) -> String {
        match self {
            Hex::CaseInsensitive => HexCaseInsensitive.encode(bytes),
            Hex::Lower => HexLower.encode(bytes),
            Hex::Upper => HexUpper.encode(bytes),
            Hex::Custom(e) => e.encode(bytes),
        }
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        match self {
            Hex::CaseInsensitive => HexCaseInsensitive.decode(text),
            Hex::Lower => HexLower.decode(text),
            Hex::Upper => HexUpper.decode(text),
            Hex::Custom(e) => e.decode(text),
        }
    }
}
impl Hex {
    /// Create a hex codec with custom alphabet
    ///
    /// # Arguments
    /// * `alphabet` - 16-character alphabet string
    ///
    /// # Note
    /// Hex encoding does not use padding (unlike Base64/Base32) as it always
    /// produces an even number of characters (2 per byte).
    ///
    /// # Errors
    /// Returns error if alphabet is invalid (wrong length, duplicate characters, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::{BinaryCodec, Hex};
    ///
    /// let codec = Hex::new("0123456789ABCDEF").unwrap();
    /// assert_eq!(codec.encode(b"Hi"), "4869");
    /// ```
    pub fn new(alphabet: &str) -> Result<Self, BinaryCodecError> {
        HexCustom::new(alphabet).map(Hex::Custom)
    }

    /// Encode bytes to hex string (lowercase, case-insensitive decode)
    ///
    /// This is the default encoding method. For explicit variant selection,
    /// use [`encode_lower`](Self::encode_lower) or [`encode_upper`](Self::encode_upper).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Hex;
    ///
    /// assert_eq!(Hex::encode(b"Hello"), "48656c6c6f");
    /// ```
    pub fn encode(bytes: &[u8]) -> String {
        HexCaseInsensitive.encode(bytes)
    }

    /// Encode bytes to lowercase hex string
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Hex;
    ///
    /// assert_eq!(Hex::encode_lower(b"Hello"), "48656c6c6f");
    /// ```
    pub fn encode_lower(bytes: &[u8]) -> String {
        HexLower.encode(bytes)
    }

    /// Encode bytes to uppercase hex string
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Hex;
    ///
    /// assert_eq!(Hex::encode_upper(b"Hello"), "48656C6C6F");
    /// ```
    pub fn encode_upper(bytes: &[u8]) -> String {
        HexUpper.encode(bytes)
    }

    /// Decode hex string to bytes (case-insensitive)
    ///
    /// This is the default decoding method. For strict variant checking,
    /// use [`decode_lower`](Self::decode_lower) or [`decode_upper`](Self::decode_upper).
    ///
    /// # Errors
    /// Returns error if decoding fails:
    ///
    /// - [`BinaryCodecError::HexOddLength`]
    ///   - Input length is odd
    /// - [`BinaryCodecError::DataEncodingDecode`]
    ///   - Invalid hex characters
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Hex;
    ///
    /// assert_eq!(Hex::decode("48656c6c6f").unwrap(), b"Hello");
    /// assert_eq!(Hex::decode("48656C6C6F").unwrap(), b"Hello");
    /// ```
    pub fn decode(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        HexCaseInsensitive.decode(text)
    }

    /// Decode lowercase hex string to bytes (accepts only lowercase)
    ///
    /// # Errors
    /// Returns error if decoding fails:
    ///
    /// - [`BinaryCodecError::HexOddLength`]
    ///   - Input length is odd
    /// - [`BinaryCodecError::DataEncodingDecode`]
    ///   - Invalid hex characters
    ///   - Uppercase characters
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Hex;
    ///
    /// assert_eq!(Hex::decode_lower("48656c6c6f").unwrap(), b"Hello");
    /// ```
    pub fn decode_lower(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        HexLower.decode(text)
    }

    /// Decode uppercase hex string to bytes (accepts only uppercase)
    ///
    /// # Errors
    /// Returns error if decoding fails:
    ///
    /// - [`BinaryCodecError::HexOddLength`]
    ///   - Input length is odd
    /// - [`BinaryCodecError::DataEncodingDecode`]
    ///   - Invalid hex characters
    ///   - Lowercase characters
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Hex;
    ///
    /// assert_eq!(Hex::decode_upper("48656C6C6F").unwrap(), b"Hello");
    /// ```
    pub fn decode_upper(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        HexUpper.decode(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Type Properties Tests
    // ========================================

    #[test]
    fn test_zst_sizes() {
        // ZST types should have zero size
        assert_eq!(size_of::<HexLower>(), 0);
        assert_eq!(size_of::<HexUpper>(), 0);
        assert_eq!(size_of::<HexCaseInsensitive>(), 0);
    }

    #[test]
    fn test_equality() {
        // Test Eq and PartialEq
        assert_eq!(HexLower, HexLower);
        assert_eq!(HexUpper, HexUpper);

        let custom1 = HexCustom::new("0123456789ABCDEF").unwrap();
        let custom2 = HexCustom::new("0123456789ABCDEF").unwrap();
        assert_eq!(custom1, custom2);
    }

    #[test]
    fn test_debug_format() {
        // Test Debug trait
        let lower = HexLower;
        let debug_str = format!("{:?}", lower);
        assert_eq!(debug_str, "HexLower");
    }

    #[test]
    fn test_default() {
        // Test Default trait
        let lower: HexLower = Default::default();
        assert_eq!(lower, HexLower);
    }

    // ========================================
    // Trait Object Tests
    // ========================================

    #[test]
    fn test_as_binary_codec_trait_object() {
        fn encode_with_codec(codec: &dyn BinaryCodec, data: &[u8]) -> String {
            codec.encode(data)
        }

        // ZST types
        assert_eq!(encode_with_codec(&HexLower, b"Hi"), "4869");
        assert_eq!(encode_with_codec(&HexUpper, b"Hi"), "4869");
        assert_eq!(encode_with_codec(&HexCaseInsensitive, b"Hi"), "4869");

        // Enum variants
        assert_eq!(encode_with_codec(&Hex::Lower, b"Hi"), "4869");
        assert_eq!(encode_with_codec(&Hex::Upper, b"Hi"), "4869");

        // Custom
        let custom = HexCustom::new("0123456789ABCDEF").unwrap();
        assert_eq!(encode_with_codec(&custom, b"Hi"), "4869");
    }

    #[test]
    fn test_as_hex_codec_trait_object() {
        fn encode_with_hex(codec: &dyn HexCodec, data: &[u8]) -> String {
            codec.encode(data)
        }

        // ZST types
        assert_eq!(encode_with_hex(&HexLower, b"Hi"), "4869");
        assert_eq!(encode_with_hex(&HexUpper, b"Hi"), "4869");

        // Enum variants
        assert_eq!(encode_with_hex(&Hex::Lower, b"Hi"), "4869");
        assert_eq!(encode_with_hex(&Hex::Upper, b"Hi"), "4869");
    }

    #[test]
    fn test_hex_codec_generic() {
        fn process<C: HexCodec>(codec: &C, data: &[u8]) -> String {
            codec.encode(data)
        }

        // ZST types
        assert_eq!(process(&HexLower, b"Hi"), "4869");
        assert_eq!(process(&HexUpper, b"Hi"), "4869");

        // Enum
        assert_eq!(process(&Hex::Lower, b"Hi"), "4869");
    }

    // ========================================
    // Type Inference Tests
    // ========================================

    #[test]
    fn test_type_inference_zst() {
        // Type can be inferred from usage
        let codec = HexLower;
        let _: String = codec.encode(b"test");
    }

    #[test]
    fn test_type_inference_enum() {
        // Enum variant inference
        let codec = Hex::Lower;
        let _: String = codec.encode(b"test");
    }

    // ========================================
    // Functional Tests
    // ========================================

    #[test]
    fn test_hex_encode_lower() {
        assert_eq!(Hex::encode_lower(b"Hello"), "48656c6c6f");
        assert_eq!(Hex::encode_lower(b"\x00\xFF"), "00ff");
        assert_eq!(Hex::encode_lower(b"\x42"), "42");
        assert_eq!(Hex::encode_lower(b""), "");

        // Large data (256 bytes)
        let large_data: Vec<u8> = (0..=255).collect();
        let encoded = Hex::encode_lower(&large_data);
        assert_eq!(encoded.len(), 512); // 256 bytes = 512 hex chars
        assert_eq!(Hex::decode_lower(&encoded).unwrap(), large_data);
    }

    #[test]
    fn test_hex_encode_upper() {
        assert_eq!(Hex::encode_upper(b"Hello"), "48656C6C6F");
        assert_eq!(Hex::encode_upper(b"\x00\xFF"), "00FF");
        assert_eq!(Hex::encode_upper(b""), "");
    }

    #[test]
    fn test_hex_decode() {
        // Lowercase
        assert_eq!(Hex::decode("48656c6c6f").unwrap(), b"Hello");
        assert_eq!(Hex::decode("00ff").unwrap(), b"\x00\xFF");
        // Uppercase
        assert_eq!(Hex::decode("48656C6C6F").unwrap(), b"Hello");
        assert_eq!(Hex::decode("00FF").unwrap(), b"\x00\xFF");
        // Mixed case
        assert_eq!(Hex::decode("48656C6c6F").unwrap(), b"Hello");
        // Empty
        assert_eq!(Hex::decode("").unwrap(), b"");
    }

    #[test]
    fn test_hex_decode_odd_length_enum() {
        let result = Hex::decode("123");
        assert!(matches!(result, Err(BinaryCodecError::HexOddLength { length: 3, .. })));
    }

    #[test]
    fn test_hex_decode_odd_length_zst() {
        // ZST types should also return HexOddLength error
        let result = HexLower.decode("123");
        assert!(matches!(result, Err(BinaryCodecError::HexOddLength { length: 3, .. })));

        let result = HexUpper.decode("123");
        assert!(matches!(result, Err(BinaryCodecError::HexOddLength { length: 3, .. })));

        let result = HexCaseInsensitive.decode("123");
        assert!(matches!(result, Err(BinaryCodecError::HexOddLength { length: 3, .. })));
    }

    #[test]
    fn test_hex_decode_invalid_char() {
        let result = Hex::decode("12GH");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    #[test]
    fn test_hex_custom_alphabet() {
        let codec = Hex::new("0123456789ABCDEF").unwrap();
        assert_eq!(codec.encode(b"Hi"), "4869");
        assert_eq!(codec.decode("4869").unwrap(), b"Hi");
    }

    #[test]
    fn test_hex_custom_direct() {
        // Test HexCustom::new() directly
        let codec = HexCustom::new("0123456789abcdef").unwrap();
        assert_eq!(codec.encode(b"Hi"), "4869");
        assert_eq!(codec.decode("4869").unwrap(), b"Hi");
    }

    #[test]
    fn test_hex_invalid_alphabet_length() {
        let result = Hex::new("0123456789ABC"); // Only 13 chars
        assert!(matches!(result, Err(BinaryCodecError::InvalidCharacterSet { .. })));

        // Test HexCustom directly
        let result = HexCustom::new("0123456789ABC");
        assert!(matches!(result, Err(BinaryCodecError::InvalidCharacterSet { .. })));
    }

    #[test]
    fn test_hex_duplicate_characters() {
        let result = Hex::new("0123456789ABCDEA"); // 'A' appears twice
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingSpec { .. })));
    }

    #[test]
    fn test_hex_enum_variants() {
        // CaseInsensitive
        assert_eq!(Hex::CaseInsensitive.encode(b"Hello"), "48656c6c6f");
        assert_eq!(Hex::CaseInsensitive.decode("48656c6c6f").unwrap(), b"Hello");
        assert_eq!(Hex::CaseInsensitive.decode("48656C6C6F").unwrap(), b"Hello");
        assert_eq!(Hex::CaseInsensitive.decode("48656C6c6F").unwrap(), b"Hello");

        // Lower
        assert_eq!(Hex::Lower.encode(b"Hello"), "48656c6c6f");
        assert_eq!(Hex::Lower.decode("48656c6c6f").unwrap(), b"Hello");

        // Upper
        assert_eq!(Hex::Upper.encode(b"Hello"), "48656C6C6F");
        assert_eq!(Hex::Upper.decode("48656C6C6F").unwrap(), b"Hello");
    }

    #[test]
    fn test_hex_zst_lower_strict() {
        // Accepts lowercase
        assert_eq!(HexLower.decode("48656c6c6f").unwrap(), b"Hello");

        // Rejects uppercase
        let result = HexLower.decode("48656C6C6F");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));

        // Rejects mixed case
        let result = HexLower.decode("48656C6c6F");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    #[test]
    fn test_hex_zst_upper_strict() {
        // Accepts uppercase
        assert_eq!(HexUpper.decode("48656C6C6F").unwrap(), b"Hello");

        // Rejects lowercase
        let result = HexUpper.decode("48656c6c6f");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));

        // Rejects mixed case
        let result = HexUpper.decode("48656C6c6F");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    #[test]
    fn test_hex_enum_lower_strict() {
        // Accepts lowercase
        assert_eq!(Hex::Lower.decode("48656c6c6f").unwrap(), b"Hello");

        // Rejects uppercase
        let result = Hex::Lower.decode("48656C6C6F");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }

    #[test]
    fn test_hex_enum_upper_strict() {
        // Accepts uppercase
        assert_eq!(Hex::Upper.decode("48656C6C6F").unwrap(), b"Hello");

        // Rejects lowercase
        let result = Hex::Upper.decode("48656c6c6f");
        assert!(matches!(result, Err(BinaryCodecError::DataEncodingDecode { .. })));
    }
}
