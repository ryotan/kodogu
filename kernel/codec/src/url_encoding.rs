//! URL encoding and decoding (Percent Encoding).
//!
//! This module provides URL encoding/decoding following RFC 3986 and WHATWG URL Standard specifications.
//!
//! ## Available Variants
//!
//! - **RFC 3986**: Standard URI encoding, encodes space as `%20`, preserves unreserved characters (A-Z, a-z, 0-9, -, ., _, ~)
//! - **Form URL Encoded**: Application/x-www-form-urlencoded (WHATWG), encodes space as `+`, preserves: A-Z, a-z, 0-9, *, -, ., _
//! - **Component**: Most aggressive encoding, encodes all non-alphanumeric characters, suitable for URL path components
//! - **Custom**: Define your own ASCII set for percent encoding
//!
//! ## Choosing a Variant
//!
//! - **RFC 3986**: Default choice for general URI encoding (query parameters, path segments in standard URIs)
//! - **Form URL Encoded**: When encoding HTML form data for POST requests (Content-Type: application/x-www-form-urlencoded)
//! - **Component**: When encoding individual URL path components that may contain special characters
//! - **Custom**: When you need specific character set requirements for specialized protocols
//!
//! See the [crate-level architecture documentation](crate#codec-architecture) for details on
//! the four complementary APIs (ZST, Enum, Static Methods, Trait).
//!
//! ## Examples
//!
//! ```
//! use codec::{UrlEncoding, BinaryCodec};
//!
//! // Using the default RFC 3986 variant
//! let encoded = UrlEncoding::encode(b"Hello, World!");
//! assert_eq!(encoded, "Hello%2C%20World%21");
//!
//! // Using specific variants
//! assert_eq!(UrlEncoding::encode_form(b"a b"), "a+b");
//! assert_eq!(UrlEncoding::encode_component(b"a-b"), "a%2Db");
//!
//! // Using codec instances
//! let codec = UrlEncoding::Rfc3986;
//! let data = codec.encode(b"Hello World");
//! assert_eq!(codec.decode(&data).unwrap(), b"Hello World");
//!
//! // Lenient decoding (handles malformed input gracefully)
//! assert_eq!(UrlEncoding::decode_rfc3986("%ZZ").unwrap(), b"%ZZ"); // Invalid hex passes through
//! assert_eq!(UrlEncoding::decode_form("a+b+c").unwrap(), b"a b c"); // + decoded as space
//! ```
//!
//! ## Lenient Decoding
//!
//! All URL encoding variants use lenient decoding that never fails:
//!
//! - Invalid percent sequences (e.g., `%ZZ`, `%2`) are passed through as-is
//! - Incomplete sequences (e.g., `test%`) are passed through as-is
//! - This matches browser behavior and ensures robustness
//!
//! ## References
//!
//! - [RFC 3986 - Uniform Resource Identifier (URI): Generic Syntax](https://tools.ietf.org/html/rfc3986)
//! - [WHATWG URL Standard - application/x-www-form-urlencoded](https://url.spec.whatwg.org/#application/x-www-form-urlencoded)

use crate::{BinaryCodec, BinaryCodecError};
use percent_encoding::{AsciiSet, CONTROLS, NON_ALPHANUMERIC, percent_decode_str, percent_encode};

/// RFC 3986 URL encoding codec (space → %20).
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Encodes all characters except unreserved characters (A-Z, a-z, 0-9, -, ., _, ~).
/// This is the standard URI encoding defined in RFC 3986.
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, UrlEncodingRfc3986};
///
/// let encoded = UrlEncodingRfc3986.encode(b"Hello World");
/// assert_eq!(encoded, "Hello%20World");
///
/// let decoded = UrlEncodingRfc3986.decode("Hello%20World").unwrap();
/// assert_eq!(decoded, b"Hello World");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct UrlEncodingRfc3986;

/// Form URL encoding codec (space → +).
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Encodes according to application/x-www-form-urlencoded (WHATWG URL Standard).
/// Space is encoded as '+' instead of '%20'.
/// Preserves: A-Z, a-z, 0-9, *, -, ., _
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, UrlEncodingForm};
///
/// let encoded = UrlEncodingForm.encode(b"Hello World");
/// assert_eq!(encoded, "Hello+World");
///
/// let decoded = UrlEncodingForm.decode("Hello+World").unwrap();
/// assert_eq!(decoded, b"Hello World");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct UrlEncodingForm;

/// Component URL encoding codec (encodes all non-alphanumeric).
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Encodes all characters except ASCII alphanumeric (A-Z, a-z, 0-9).
/// Most aggressive encoding, suitable for URL path components.
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, UrlEncodingComponent};
///
/// let encoded = UrlEncodingComponent.encode(b"Hello, World!");
/// assert_eq!(encoded, "Hello%2C%20World%21");
///
/// let decoded = UrlEncodingComponent.decode("Hello%2C%20World%21").unwrap();
/// assert_eq!(decoded, b"Hello, World!");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct UrlEncodingComponent;

/// Custom URL encoding codec with user-defined ASCII set.
///
/// Allows specifying a custom set of characters to be percent-encoded.
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, UrlEncodingCustom};
/// use percent_encoding::{CONTROLS, AsciiSet};
///
/// const CUSTOM_SET: &AsciiSet = &CONTROLS.add(b' ');
/// let codec = UrlEncodingCustom::new(CUSTOM_SET);
///
/// let encoded = codec.encode(b"Hello World");
/// let decoded = codec.decode(&encoded).unwrap();
/// assert_eq!(decoded, b"Hello World");
/// ```
#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub struct UrlEncodingCustom {
    ascii_set: &'static AsciiSet,
}

/// URL encoding codec variants.
///
/// Provides runtime selection of URL encoding variants:
/// - RFC 3986 (standard URI encoding, space → %20)
/// - Form URL encoding (application/x-www-form-urlencoded, space → +)
/// - Component encoding (encodes all non-alphanumeric)
/// - Custom (user-defined character set)
///
/// # Examples
///
/// ```
/// use codec::{BinaryCodec, UrlEncoding};
///
/// let rfc3986 = UrlEncoding::Rfc3986;
/// let form = UrlEncoding::Form;
/// let component = UrlEncoding::Component;
///
/// assert_eq!(rfc3986.encode(b"a b"), "a%20b");
/// assert_eq!(form.encode(b"a b"), "a+b");
/// assert_eq!(component.encode(b"a-b"), "a%2Db");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub enum UrlEncoding {
    /// RFC 3986 URL encoding (space → %20)
    #[default]
    Rfc3986,
    /// Form URL encoding (space → +)
    Form,
    /// Component encoding (all non-alphanumeric → %XX)
    Component,
    /// Custom encoding with user-defined ASCII set
    Custom(UrlEncodingCustom),
}

/// Marker trait for URL encoding codecs.
///
/// Implemented by all URL encoding variants to enable generic programming
/// over different URL encoding schemes while maintaining type safety.
///
/// # Example
/// ```
/// use codec::{UrlEncodingCodec, UrlEncodingRfc3986, UrlEncodingForm, BinaryCodec};
///
/// fn encode_with_url_codec<C: UrlEncodingCodec>(codec: &C, data: &[u8]) -> String {
///     codec.encode(data)
/// }
///
/// let rfc_result = encode_with_url_codec(&UrlEncodingRfc3986, b"Hello World");
/// let form_result = encode_with_url_codec(&UrlEncodingForm, b"Hello World");
///
/// assert_eq!(rfc_result, "Hello%20World");
/// assert_eq!(form_result, "Hello+World");
/// ```
pub trait UrlEncodingCodec: BinaryCodec {}

fn encode_internal(ascii_set: &'static AsciiSet, bytes: &[u8]) -> String {
    percent_encode(bytes, ascii_set).to_string()
}

fn decode_internal(str: &str) -> Result<Vec<u8>, BinaryCodecError> {
    Ok(percent_decode_str(str).collect::<Vec<u8>>())
}

/// RFC 3986 encode set - characters to be percent-encoded.
///
/// RFC 3986 Section 2.3 defines unreserved characters as:
/// `unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"`
///
/// Everything else must be percent-encoded for safe URI transmission.
///
/// Reference: <https://datatracker.ietf.org/doc/html/rfc3986#section-2.3>
///
/// This set encodes all characters except: `A-Z a-z 0-9 - . _ ~`
const RFC3986_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'!')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');
impl UrlEncodingCodec for UrlEncodingRfc3986 {}
impl BinaryCodec for UrlEncodingRfc3986 {
    fn encode(&self, bytes: &[u8]) -> String {
        encode_internal(RFC3986_ENCODE_SET, bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(text)
    }
}

/// Form URL encoding set - application/x-www-form-urlencoded.
///
/// WHATWG URL Standard defines the application/x-www-form-urlencoded percent-encode set as:
/// All code points except ASCII alphanumeric, U+002A (*), U+002D (-), U+002E (.), and U+005F (_).
///
/// Additionally, space (U+0020) is encoded as '+' instead of '%20'.
///
/// Reference: <https://url.spec.whatwg.org/#application-x-www-form-urlencoded-percent-encode-set>
///
/// This set encodes all characters except: `A-Z a-z 0-9 * - . _` (and space → +)
const FORM_URLENCODED: &AsciiSet =
    &NON_ALPHANUMERIC.remove(b'*').remove(b'-').remove(b'.').remove(b'_');
impl UrlEncodingCodec for UrlEncodingForm {}
impl BinaryCodec for UrlEncodingForm {
    fn encode(&self, bytes: &[u8]) -> String {
        // Form encoding: space → +, others → percent encoding
        encode_internal(FORM_URLENCODED, bytes).replace("%20", "+")
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        // Form decoding: + → space, then percent decode
        decode_internal(&text.replace('+', " "))
    }
}

impl UrlEncodingCodec for UrlEncodingComponent {}
impl BinaryCodec for UrlEncodingComponent {
    fn encode(&self, bytes: &[u8]) -> String {
        encode_internal(NON_ALPHANUMERIC, bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(text)
    }
}

impl UrlEncodingCodec for UrlEncodingCustom {}
impl BinaryCodec for UrlEncodingCustom {
    fn encode(&self, bytes: &[u8]) -> String {
        encode_internal(self.ascii_set, bytes)
    }
    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        decode_internal(text)
    }
}
impl UrlEncodingCustom {
    pub fn new(ascii_set: &'static AsciiSet) -> UrlEncodingCustom {
        UrlEncodingCustom { ascii_set }
    }
}

impl UrlEncodingCodec for UrlEncoding {}
impl BinaryCodec for UrlEncoding {
    fn encode(&self, bytes: &[u8]) -> String {
        match self {
            UrlEncoding::Rfc3986 => UrlEncodingRfc3986.encode(bytes),
            UrlEncoding::Form => UrlEncodingForm.encode(bytes),
            UrlEncoding::Component => UrlEncodingComponent.encode(bytes),
            UrlEncoding::Custom(encoding) => encoding.encode(bytes),
        }
    }
    fn decode(&self, string: &str) -> Result<Vec<u8>, BinaryCodecError> {
        match self {
            UrlEncoding::Rfc3986 => UrlEncodingRfc3986.decode(string),
            UrlEncoding::Form => UrlEncodingForm.decode(string),
            UrlEncoding::Component => UrlEncodingComponent.decode(string),
            UrlEncoding::Custom(encoding) => encoding.decode(string),
        }
    }
}
impl UrlEncoding {
    /// Encodes bytes to URL-encoded string (RFC 3986 variant).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::UrlEncoding;
    ///
    /// let encoded = UrlEncoding::encode(b"Hello World");
    /// assert_eq!(encoded, "Hello%20World");
    /// ```
    pub fn encode(bytes: &[u8]) -> String {
        UrlEncodingRfc3986.encode(bytes)
    }

    /// Encodes bytes to RFC 3986 URL-encoded string (space → %20).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::UrlEncoding;
    ///
    /// let encoded = UrlEncoding::encode_rfc3986(b"Hello World");
    /// assert_eq!(encoded, "Hello%20World");
    /// ```
    pub fn encode_rfc3986(bytes: &[u8]) -> String {
        UrlEncodingRfc3986.encode(bytes)
    }

    /// Encodes bytes to Form URL-encoded string (space → +).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::UrlEncoding;
    ///
    /// let encoded = UrlEncoding::encode_form(b"Hello World");
    /// assert_eq!(encoded, "Hello+World");
    /// ```
    pub fn encode_form(bytes: &[u8]) -> String {
        UrlEncodingForm.encode(bytes)
    }

    /// Encodes bytes to Component-encoded string (encodes all non-alphanumeric characters).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::UrlEncoding;
    ///
    /// let encoded = UrlEncoding::encode_component(b"Hello, World!");
    /// assert_eq!(encoded, "Hello%2C%20World%21");
    /// ```
    pub fn encode_component(bytes: &[u8]) -> String {
        UrlEncodingComponent.encode(bytes)
    }

    /// Decodes URL-encoded string to bytes (RFC 3986 variant).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::UrlEncoding;
    ///
    /// let decoded = UrlEncoding::decode("Hello%20World").unwrap();
    /// assert_eq!(decoded, b"Hello World");
    /// ```
    pub fn decode(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        UrlEncodingRfc3986.decode(text)
    }

    /// Decodes RFC 3986 URL-encoded string to bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::UrlEncoding;
    ///
    /// let decoded = UrlEncoding::decode_rfc3986("Hello%20World").unwrap();
    /// assert_eq!(decoded, b"Hello World");
    /// ```
    pub fn decode_rfc3986(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        UrlEncodingRfc3986.decode(text)
    }

    /// Decodes Form URL-encoded string to bytes (handles + as space).
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::UrlEncoding;
    ///
    /// let decoded = UrlEncoding::decode_form("Hello+World").unwrap();
    /// assert_eq!(decoded, b"Hello World");
    /// ```
    pub fn decode_form(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        UrlEncodingForm.decode(text)
    }

    /// Decodes Component-encoded string to bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::UrlEncoding;
    ///
    /// let decoded = UrlEncoding::decode_component("Hello%2C%20World!").unwrap();
    /// assert_eq!(decoded, b"Hello, World!");
    /// ```
    pub fn decode_component(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        UrlEncodingComponent.decode(text)
    }

    pub fn new(ascii_set: &'static AsciiSet) -> UrlEncoding {
        UrlEncoding::Custom(UrlEncodingCustom::new(ascii_set))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode_rfc3986() {
        assert_eq!(UrlEncoding::encode_rfc3986(b"Hello World"), "Hello%20World");
        assert_eq!(UrlEncoding::encode_rfc3986(b"test@example.com"), "test%40example.com");
        assert_eq!(UrlEncoding::encode_rfc3986(b""), "");
    }

    #[test]
    fn test_url_decode_rfc3986() {
        assert_eq!(UrlEncoding::decode_rfc3986("Hello%20World").unwrap(), b"Hello World");
        assert_eq!(UrlEncoding::decode_rfc3986("test%40example.com").unwrap(), b"test@example.com");
        assert_eq!(UrlEncoding::decode_rfc3986("").unwrap(), b"");
    }

    #[test]
    fn test_url_encode_form() {
        assert_eq!(UrlEncoding::encode_form(b"Hello World"), "Hello+World");
        assert_eq!(UrlEncoding::encode_form(b"a+b=c"), "a%2Bb%3Dc");
    }

    #[test]
    fn test_url_decode_form() {
        assert_eq!(UrlEncoding::decode_form("Hello+World").unwrap(), b"Hello World");
        assert_eq!(UrlEncoding::decode_form("a%2Bb%3Dc").unwrap(), b"a+b=c");
    }

    #[test]
    fn test_url_encode_component() {
        assert_eq!(UrlEncoding::encode_component(b"Hello, World!"), "Hello%2C%20World%21");
        assert_eq!(UrlEncoding::encode_component(b"test/path"), "test%2Fpath");
    }

    #[test]
    fn test_url_decode_component() {
        assert_eq!(UrlEncoding::decode_component("Hello%2C%20World%21").unwrap(), b"Hello, World!");
        assert_eq!(UrlEncoding::decode_component("test%2Fpath").unwrap(), b"test/path");
    }

    #[test]
    fn test_url_codec_constants() {
        // RFC 3986
        assert_eq!(UrlEncodingRfc3986.encode(b"Hello World"), "Hello%20World");
        assert_eq!(UrlEncodingRfc3986.decode("Hello%20World").unwrap(), b"Hello World");

        // Form URL
        assert_eq!(UrlEncodingForm.encode(b"Hello World"), "Hello+World");
        assert_eq!(UrlEncodingForm.decode("Hello+World").unwrap(), b"Hello World");

        // Component
        assert_eq!(UrlEncodingComponent.encode(b"Hello, World!"), "Hello%2C%20World%21");
        assert_eq!(UrlEncodingComponent.decode("Hello%2C%20World%21").unwrap(), b"Hello, World!");
    }

    #[test]
    fn test_url_codec_as_trait_object() {
        let codecs: Vec<&dyn BinaryCodec> =
            vec![&UrlEncodingRfc3986, &UrlEncodingForm, &UrlEncodingComponent];

        assert_eq!(codecs[0].encode(b"Hello World"), "Hello%20World");
        assert_eq!(codecs[1].encode(b"Hello World"), "Hello+World");
        assert_eq!(codecs[2].encode(b"Hello, World!"), "Hello%2C%20World%21");
    }

    #[test]
    fn test_url_round_trip() {
        let data = b"The quick brown fox jumps over the lazy dog!";

        // RFC 3986
        let encoded_rfc = UrlEncodingRfc3986.encode(data);
        assert_eq!(UrlEncodingRfc3986.decode(&encoded_rfc).unwrap(), data);

        // Form
        let encoded_form = UrlEncodingForm.encode(data);
        assert_eq!(UrlEncodingForm.decode(&encoded_form).unwrap(), data);

        // Component
        let encoded_comp = UrlEncodingComponent.encode(data);
        assert_eq!(UrlEncodingComponent.decode(&encoded_comp).unwrap(), data);
    }

    #[test]
    fn test_url_special_characters() {
        let data = b"!@#$%^&*()";

        // Encode and decode round trip
        let encoded = UrlEncodingRfc3986.encode(data);
        let decoded = UrlEncodingRfc3986.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_url_utf8_characters() {
        let data = "こんにちは".as_bytes();

        // Encode and decode round trip
        let encoded = UrlEncodingRfc3986.encode(data);
        let decoded = UrlEncodingRfc3986.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    // Decoder is lenient - handles malformed input gracefully
    #[test]
    fn test_url_decode_invalid_hex_lenient() {
        // Invalid hex characters - decoder passes through
        let result = UrlEncoding::decode("%ZZ");
        assert_eq!(result.unwrap(), b"%ZZ");
    }

    #[test]
    fn test_url_decode_incomplete_percent_lenient() {
        // Incomplete percent encoding - decoder passes through
        let result = UrlEncoding::decode("%2");
        assert_eq!(result.unwrap(), b"%2");
    }

    #[test]
    fn test_url_decode_truncated_percent_lenient() {
        // Truncated percent - decoder passes through
        let result = UrlEncoding::decode("test%");
        assert_eq!(result.unwrap(), b"test%");
    }

    #[test]
    fn test_url_decode_standalone_percent_lenient() {
        // Standalone percent - decoder passes through
        let result = UrlEncoding::decode("%");
        assert_eq!(result.unwrap(), b"%");
    }

    #[test]
    fn test_url_decode_mixed_valid_invalid() {
        // Mix of valid and invalid - decoder handles gracefully
        let result = UrlEncoding::decode("Hello%20World%ZZ");
        assert_eq!(result.unwrap(), b"Hello World%ZZ");
    }

    #[test]
    fn test_url_form_plus_handling() {
        // Form encoding should handle + as space
        assert_eq!(UrlEncoding::decode_form("a+b+c").unwrap(), b"a b c");
        assert_eq!(UrlEncoding::decode_form("test+%2B+value").unwrap(), b"test + value");
    }

    #[test]
    fn test_url_component_special_chars() {
        // Component encoding should encode all special characters
        let data = b"-_.!~*'()";
        let encoded = UrlEncoding::encode_component(data);
        let decoded = UrlEncoding::decode_component(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_url_empty_string() {
        assert_eq!(UrlEncoding::encode(b""), "");
        assert_eq!(UrlEncoding::decode("").unwrap(), b"");
        assert_eq!(UrlEncoding::encode_form(b""), "");
        assert_eq!(UrlEncoding::decode_form("").unwrap(), b"");
    }

    #[test]
    fn test_url_rfc3986_angle_brackets() {
        // RFC 3986: < and > must be percent-encoded
        assert_eq!(UrlEncoding::encode_rfc3986(b"<tag>"), "%3Ctag%3E");
        assert_eq!(UrlEncoding::decode_rfc3986("%3Ctag%3E").unwrap(), b"<tag>");
    }

    #[test]
    fn test_url_rfc3986_unreserved_not_encoded() {
        // RFC 3986 unreserved characters should not be encoded
        let unreserved = b"AZaz09-._~";
        let encoded = UrlEncoding::encode_rfc3986(unreserved);
        assert_eq!(encoded, "AZaz09-._~");
    }

    #[test]
    fn test_url_encoding_enum_variants() {
        let data = b"Hello World";

        // Test runtime selection via enum
        let rfc = UrlEncoding::Rfc3986;
        let form = UrlEncoding::Form;
        let component = UrlEncoding::Component;

        assert_eq!(rfc.encode(data), "Hello%20World");
        assert_eq!(form.encode(data), "Hello+World");
        assert_eq!(component.encode(data), "Hello%20World");

        assert_eq!(rfc.decode("Hello%20World").unwrap(), data);
        assert_eq!(form.decode("Hello+World").unwrap(), data);
        assert_eq!(component.decode("Hello%20World").unwrap(), data);
    }

    #[test]
    fn test_url_encoding_custom() {
        // Custom encoding with only space encoded
        const CUSTOM_SET: &AsciiSet = &CONTROLS.add(b' ');
        let custom = UrlEncodingCustom::new(CUSTOM_SET);

        assert_eq!(custom.encode(b"Hello World!"), "Hello%20World!");
        assert_eq!(custom.decode("Hello%20World!").unwrap(), b"Hello World!");

        // Via enum
        let custom_enum = UrlEncoding::new(CUSTOM_SET);
        assert_eq!(custom_enum.encode(b"Hello World!"), "Hello%20World!");
    }

    #[test]
    fn test_url_control_characters() {
        // Test control characters (0x00-0x1F)
        let control_chars = b"\x00\x01\x0A\x0D\x1F";

        let encoded = UrlEncodingRfc3986.encode(control_chars);
        let decoded = UrlEncodingRfc3986.decode(&encoded).unwrap();
        assert_eq!(decoded, control_chars);
    }

    #[test]
    fn test_url_all_bytes() {
        // Test all possible byte values (0x00-0xFF)
        let all_bytes: Vec<u8> = (0..=255).collect();

        // RFC 3986 round-trip
        let encoded_rfc = UrlEncodingRfc3986.encode(&all_bytes);
        let decoded_rfc = UrlEncodingRfc3986.decode(&encoded_rfc).unwrap();
        assert_eq!(decoded_rfc, all_bytes);

        // Form round-trip
        let encoded_form = UrlEncodingForm.encode(&all_bytes);
        let decoded_form = UrlEncodingForm.decode(&encoded_form).unwrap();
        assert_eq!(decoded_form, all_bytes);

        // Component round-trip
        let encoded_comp = UrlEncodingComponent.encode(&all_bytes);
        let decoded_comp = UrlEncodingComponent.decode(&encoded_comp).unwrap();
        assert_eq!(decoded_comp, all_bytes);
    }

    #[test]
    fn test_url_form_preserved_chars() {
        // Form encoding preserves: * - . _
        let preserved = b"*-._";
        let encoded = UrlEncodingForm.encode(preserved);
        assert_eq!(encoded, "*-._");

        // But RFC 3986 encodes * differently
        let encoded_rfc = UrlEncodingRfc3986.encode(b"*");
        assert_eq!(encoded_rfc, "%2A");
    }

    #[test]
    fn test_url_component_encodes_all_non_alphanumeric() {
        // Component encoding is most aggressive
        let data = b"-._~";
        let encoded = UrlEncodingComponent.encode(data);
        // All should be encoded (unlike RFC 3986 which preserves these)
        assert!(encoded.contains('%'));

        let decoded = UrlEncodingComponent.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_url_codec_trait_usage() {
        fn encode_with_trait(codec: &dyn UrlEncodingCodec, data: &[u8]) -> String {
            codec.encode(data)
        }

        let data = b"test data";
        let result1 = encode_with_trait(&UrlEncodingRfc3986, data);
        let result2 = encode_with_trait(&UrlEncodingForm, data);
        let result3 = encode_with_trait(&UrlEncodingComponent, data);

        assert_eq!(result1, "test%20data");
        assert_eq!(result2, "test+data");
        assert_eq!(result3, "test%20data");
    }

    #[test]
    fn test_url_large_data() {
        // 1KB of data
        let large_data = vec![0xAB; 1024];

        let encoded = UrlEncodingRfc3986.encode(&large_data);
        let decoded = UrlEncodingRfc3986.decode(&encoded).unwrap();
        assert_eq!(decoded, large_data);
    }
}
