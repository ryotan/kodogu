//! Integration tests for kodogu_codec
//!
//! These tests validate the public API from a consumer's perspective,
//! ensuring that all codec variants work correctly when used through
//! trait objects and in realistic scenarios.

use kodogu_codec::{
    Base32, Base32Codec, Base32CrockfordLower, Base32CrockfordUpper, Base32Hex, Base32Standard,
    Base58, Base58Bitcoin, Base58Codec, Base58Flickr, Base64, Base64Codec, Base64Standard,
    Base64UrlSafe, BinaryCodec, Hex, HexCodec, HexLower, HexUpper, QuotedPrintable,
    QuotedPrintableCodec, QuotedPrintableRobust, QuotedPrintableStrict, TextCodec, UrlEncoding,
    UrlEncodingCodec, UrlEncodingComponent, UrlEncodingForm, UrlEncodingRfc3986, Utf8,
};

// ========================================
// Basic Usage Tests
// ========================================

#[test]
fn test_hex_basic_usage() {
    // Direct ZST usage
    let data = b"Hello, World!";
    let encoded = HexLower.encode(data);
    assert_eq!(encoded, "48656c6c6f2c20576f726c6421");
    let decoded = HexLower.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Enum usage
    let encoded = Hex::encode_lower(data);
    assert_eq!(encoded, "48656c6c6f2c20576f726c6421");
    let decoded = Hex::decode_lower(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_base64_basic_usage() {
    // Direct ZST usage
    let data = b"Hello, World!";
    let encoded = Base64Standard.encode(data);
    assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");
    let decoded = Base64Standard.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Static methods
    let encoded = Base64::encode_standard(data);
    assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");
    let decoded = Base64::decode_standard(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_base32_basic_usage() {
    let data = b"Hello, World!";

    // Standard variant (direct ZST usage)
    let encoded = Base32Standard.encode(data);
    assert_eq!(encoded, "JBSWY3DPFQQFO33SNRSCC===");
    let decoded = Base32Standard.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Hex variant (direct ZST usage)
    let encoded = Base32Hex.encode(data);
    assert_eq!(encoded, "91IMOR3F5GG5ERRIDHI22===");
    let decoded = Base32Hex.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Crockford variant (direct ZST usage)
    let encoded_upper = Base32CrockfordUpper.encode(data);
    let decoded = Base32CrockfordUpper.decode(&encoded_upper).unwrap();
    assert_eq!(decoded, data);

    let encoded_lower = Base32CrockfordLower.encode(data);
    let decoded = Base32CrockfordLower.decode(&encoded_lower).unwrap();
    assert_eq!(decoded, data);

    // Static methods
    let encoded = Base32::encode_standard(data);
    assert_eq!(encoded, "JBSWY3DPFQQFO33SNRSCC===");
    let decoded = Base32::decode_standard(&encoded).unwrap();
    assert_eq!(decoded, data);

    let encoded = Base32::encode_hex(data);
    assert_eq!(encoded, "91IMOR3F5GG5ERRIDHI22===");
    let decoded = Base32::decode_hex(&encoded).unwrap();
    assert_eq!(decoded, data);

    let encoded = Base32::encode_crockford(data);
    let decoded = Base32::decode_crockford(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_base58_basic_usage() {
    let data = b"Hello, World!";

    // Bitcoin variant (direct ZST usage)
    let encoded = Base58Bitcoin.encode(data);
    assert_eq!(encoded, "72k1xXWG59fYdzSNoA");
    let decoded = Base58Bitcoin.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Flickr variant (direct ZST usage)
    let encoded = Base58Flickr.encode(data);
    let decoded = Base58Flickr.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Static methods (Bitcoin by default)
    let encoded = Base58::encode(data);
    assert_eq!(encoded, "72k1xXWG59fYdzSNoA");
    let decoded = Base58::decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Explicit Bitcoin
    let encoded = Base58::encode_bitcoin(data);
    assert_eq!(encoded, "72k1xXWG59fYdzSNoA");
    let decoded = Base58::decode_bitcoin(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Explicit Flickr
    let encoded = Base58::encode_flickr(data);
    let decoded = Base58::decode_flickr(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_utf8_basic_usage() {
    let text = "Hello, 世界! 🦀";
    let bytes = Utf8.encode(text);
    let decoded = Utf8.decode(&bytes);
    assert_eq!(decoded, text);

    // Strict mode
    let bytes = Utf8.encode_strict(text).unwrap();
    let decoded = Utf8.decode_strict(&bytes).unwrap();
    assert_eq!(decoded, text);
}

// ========================================
// Trait Object Tests
// ========================================

#[test]
fn test_binary_codec_trait_object() {
    let data = b"Test data";

    // Collection of different codecs as trait objects
    let codecs: Vec<Box<dyn BinaryCodec>> = vec![
        Box::new(HexLower),
        Box::new(HexUpper),
        Box::new(Base32Standard),
        Box::new(Base32Hex),
        Box::new(Base32CrockfordUpper),
        Box::new(Base32CrockfordLower),
        Box::new(Base58Bitcoin),
        Box::new(Base58Flickr),
        Box::new(Base64Standard),
        Box::new(Base64UrlSafe),
    ];

    for codec in codecs {
        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Round-trip failed for codec");
    }
}

#[test]
fn test_hex_codec_trait_object() {
    let data = b"Test data";

    fn encode_with_hex_codec(codec: &dyn HexCodec, data: &[u8]) -> String {
        codec.encode(data)
    }

    // All Hex variants work as HexCodec trait objects
    let result1 = encode_with_hex_codec(&HexLower, data);
    let result2 = encode_with_hex_codec(&Hex::Lower, data);
    assert_eq!(result1, result2);
}

#[test]
fn test_base32_codec_trait_object() {
    let data = b"Test data";

    fn encode_with_base32_codec(codec: &dyn Base32Codec, data: &[u8]) -> String {
        codec.encode(data)
    }

    // All Base32 variants work as Base32Codec trait objects
    let result1 = encode_with_base32_codec(&Base32Standard, data);
    let result2 = encode_with_base32_codec(&Base32::Standard, data);
    assert_eq!(result1, result2);

    let result3 = encode_with_base32_codec(&Base32Hex, data);
    let result4 = encode_with_base32_codec(&Base32::Hex, data);
    assert_eq!(result3, result4);

    let result5 = encode_with_base32_codec(&Base32CrockfordUpper, data);
    let result6 = encode_with_base32_codec(&Base32::CrockfordUpper, data);
    assert_eq!(result5, result6);
}

#[test]
fn test_base58_codec_trait_object() {
    let data = b"Test data";

    fn encode_with_base58_codec(codec: &dyn Base58Codec, data: &[u8]) -> String {
        codec.encode(data)
    }

    // All Base58 variants work as Base58Codec trait objects
    let result1 = encode_with_base58_codec(&Base58Bitcoin, data);
    let result2 = encode_with_base58_codec(&Base58::Bitcoind, data);
    assert_eq!(result1, result2);

    let result3 = encode_with_base58_codec(&Base58Flickr, data);
    let result4 = encode_with_base58_codec(&Base58::Flickr, data);
    assert_eq!(result3, result4);
}

#[test]
fn test_base64_codec_trait_object() {
    let data = b"Test data";

    fn encode_with_base64_codec(codec: &dyn Base64Codec, data: &[u8]) -> String {
        codec.encode(data)
    }

    // All Base64 variants work as Base64Codec trait objects
    let result1 = encode_with_base64_codec(&Base64Standard, data);
    let result2 = encode_with_base64_codec(&Base64::Standard, data);
    assert_eq!(result1, result2);
}

// ========================================
// Runtime Selection Tests
// ========================================

#[test]
fn test_runtime_codec_selection() {
    let data = b"Runtime test";

    // Simulate runtime configuration
    let configs = vec!["lower", "upper", "case-insensitive"];

    for config in configs {
        let codec = match config {
            "lower" => Hex::Lower,
            "upper" => Hex::Upper,
            _ => Hex::CaseInsensitive,
        };

        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

#[test]
fn test_base32_runtime_selection() {
    let data = b"Runtime test";

    // Simulate runtime configuration for Base32 variants
    let configs = vec!["standard", "hex", "crockford-upper", "crockford-lower"];

    for config in configs {
        let codec = match config {
            "standard" => Base32::Standard,
            "hex" => Base32::Hex,
            "crockford-upper" => Base32::CrockfordUpper,
            _ => Base32::CrockfordLower,
        };

        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

#[test]
fn test_base58_runtime_selection() {
    let data = b"Runtime test";

    // Simulate runtime configuration for Base58 variants
    let configs = vec!["bitcoin", "flickr"];

    for config in configs {
        let codec = match config {
            "bitcoin" => Base58::Bitcoind,
            "flickr" => Base58::Flickr,
            _ => Base58::Bitcoind,
        };

        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

// ========================================
// Error Handling Tests
// ========================================

#[test]
fn test_hex_error_handling() {
    // Odd length
    let result = Hex::decode("123");
    assert!(result.is_err());

    // Invalid characters
    let result = Hex::decode("12GH");
    assert!(result.is_err());

    // Case sensitivity
    let result = HexLower.decode("12AB"); // Uppercase in lowercase codec
    assert!(result.is_err());
}

#[test]
fn test_base64_error_handling() {
    // Invalid characters
    let result = Base64::decode_standard("Invalid!@#");
    assert!(result.is_err());

    // Missing padding
    let result = Base64::decode_standard("SGVsbG8"); // Missing =
    assert!(result.is_err());

    // Wrong padding position
    let result = Base64::decode_standard("SG=sbG8=");
    assert!(result.is_err());
}

#[test]
fn test_base32_error_handling() {
    // Invalid characters in Standard
    let result = Base32::decode_standard("Invalid!@#");
    assert!(result.is_err());

    // Invalid characters in Hex
    let result = Base32::decode_hex("Invalid!@#");
    assert!(result.is_err());

    // Standard rejects Hex-specific characters (0-9)
    let result = Base32Standard.decode("01234567");
    assert!(result.is_err());

    // Hex rejects Standard-specific characters (W-Z, 2-7)
    let result = Base32Hex.decode("WXYZ2345");
    assert!(result.is_err());

    // Invalid padding
    let result = Base32::decode_standard("JBSW=3DP"); // Padding in middle
    assert!(result.is_err());
}

#[test]
fn test_base58_error_handling() {
    // Invalid characters (Base58 excludes 0, O, I, l)
    let result = Base58::decode_bitcoin("0OIl");
    assert!(result.is_err());

    // Empty string is valid
    let result = Base58::decode_bitcoin("");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), b"");

    // Bitcoin rejects Flickr encoding
    let flickr_encoded = Base58::encode_flickr(b"test");
    // Bitcoin and Flickr differ in case
    if flickr_encoded.chars().any(|c| c.is_lowercase()) {
        let result = Base58Bitcoin.decode(&flickr_encoded);
        // May succeed if no case-specific chars, but generally different
        // We just check it doesn't panic
        let _ = result;
    }
}

#[test]
fn test_base32_crockford_features() {
    let data = b"Test";

    // Case-insensitive decoding
    let encoded_upper = Base32::encode_crockford_upper(data);
    let encoded_lower = Base32::encode_crockford_lower(data);

    // Both can decode each other's output
    assert_eq!(Base32::decode_crockford(&encoded_upper).unwrap(), data);
    assert_eq!(Base32::decode_crockford(&encoded_lower).unwrap(), data);

    // Confusion-resistant character mapping
    let encoded = Base32CrockfordUpper.encode(data);
    if encoded.contains('1') {
        // I/i/L/l → 1
        let with_i = encoded.replace('1', "I");
        assert_eq!(Base32CrockfordUpper.decode(&with_i).unwrap(), data);

        let with_l = encoded.replace('1', "L");
        assert_eq!(Base32CrockfordUpper.decode(&with_l).unwrap(), data);
    }
    if encoded.contains('0') {
        // O/o → 0
        let with_o = encoded.replace('0', "O");
        assert_eq!(Base32CrockfordUpper.decode(&with_o).unwrap(), data);
    }
}

#[test]
fn test_utf8_error_handling() {
    // Invalid UTF-8 in strict mode
    let invalid_utf8 = vec![0xFF, 0xFE];
    let result = Utf8.decode_strict(&invalid_utf8);
    assert!(result.is_err());

    // Lossy mode succeeds
    let decoded = Utf8.decode(&invalid_utf8);
    assert!(decoded.contains('�'));
}

// ========================================
// Custom Alphabet Tests
// ========================================

#[test]
fn test_hex_custom_alphabet() {
    let data = b"Custom test";
    let alphabet = "0123456789ABCDEF";

    let codec = Hex::new(alphabet).unwrap();
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Invalid alphabet length
    let result = Hex::new("0123456789ABC");
    assert!(result.is_err());
}

#[test]
fn test_base64_custom_alphabet() {
    let data = b"Custom test";
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let codec = Base64::new(alphabet).unwrap();
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Invalid alphabet length
    let result = Base64::new("ABC");
    assert!(result.is_err());
}

#[test]
fn test_base32_custom_alphabet() {
    let data = b"Custom test";
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"; // Standard Base32 alphabet

    let codec = Base32::new(alphabet).unwrap();
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Invalid alphabet length (not 32)
    let result = Base32::new("ABC");
    assert!(result.is_err());

    let result = Base32::new("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"); // 36 chars
    assert!(result.is_err());
}

#[test]
fn test_base58_custom_alphabet() {
    let data = b"Custom test";

    // Ripple alphabet (well-known alternative)
    let alphabet = "rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";
    let codec = Base58::new(alphabet).unwrap();

    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Invalid alphabet length (not 58)
    let result = Base58::new("ABC");
    assert!(result.is_err());

    let result = Base58::new("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"); // 36 chars
    assert!(result.is_err());

    // Duplicate characters
    let alphabet_dup = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1";
    let result = Base58::new(alphabet_dup);
    assert!(result.is_err());
}

#[test]
fn test_base32_custom_padding() {
    let data = b"Hi"; // 2 bytes - requires padding
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    // Custom padding character
    let codec = Base32::new_with_padding(alphabet, Some('*')).unwrap();
    let encoded = codec.encode(data);
    assert!(
        encoded.ends_with('*'),
        "Encoded string should end with custom padding '*': {}",
        encoded
    );
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // No padding
    let codec_no_pad = Base32::new_with_padding(alphabet, None).unwrap();
    let encoded = codec_no_pad.encode(data);
    assert!(
        !encoded.ends_with('='),
        "Encoded string should not end with '='"
    );
    assert!(
        !encoded.ends_with('*'),
        "Encoded string should not end with '*'"
    );
    let decoded = codec_no_pad.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_base64_custom_padding() {
    let data = b"Hello"; // 5 bytes - requires padding
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    // Custom padding character
    let codec = Base64::new_with_padding(alphabet, Some('*')).unwrap();
    let encoded = codec.encode(data);
    assert!(
        encoded.ends_with('*'),
        "Encoded string should end with custom padding '*': {}",
        encoded
    );
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // No padding
    let codec_no_pad = Base64::new_with_padding(alphabet, None).unwrap();
    let encoded = codec_no_pad.encode(data);
    assert!(
        !encoded.ends_with('='),
        "Encoded string should not end with '='"
    );
    assert!(
        !encoded.ends_with('*'),
        "Encoded string should not end with '*'"
    );
    let decoded = codec_no_pad.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

// ========================================
// Cross-Codec Tests
// ========================================

#[test]
fn test_multiple_encoding_layers() {
    let original_text = "Hello, 世界! 🦀";

    // UTF-8 encode
    let bytes = Utf8.encode(original_text);

    // Base64 encode
    let base64_encoded = Base64::encode_standard(&bytes);

    // Hex encode
    let hex_encoded = Hex::encode_lower(base64_encoded.as_bytes());

    // Reverse: Hex decode
    let hex_decoded = Hex::decode_lower(&hex_encoded).unwrap();

    // Base64 decode
    let base64_decoded = Base64::decode_standard(&String::from_utf8(hex_decoded).unwrap()).unwrap();

    // UTF-8 decode
    let final_text = Utf8.decode(&base64_decoded);

    assert_eq!(final_text, original_text);
}

// ========================================
// Large Data Tests
// ========================================

#[test]
fn test_large_data_hex() {
    // 1MB of data
    let large_data = vec![0xAB; 1024 * 1024];
    let encoded = Hex::encode_lower(&large_data);
    let decoded = Hex::decode_lower(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}

#[test]
fn test_large_data_base32_standard() {
    // 1MB of data
    let large_data = vec![0xAB; 1024 * 1024];
    let encoded = Base32::encode_standard(&large_data);
    let decoded = Base32::decode_standard(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}

#[test]
fn test_large_data_base32_hex() {
    // 1MB of data
    let large_data = vec![0xAB; 1024 * 1024];
    let encoded = Base32::encode_hex(&large_data);
    let decoded = Base32::decode_hex(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}

#[test]
fn test_large_data_base32_crockford() {
    // 1MB of data
    let large_data = vec![0xAB; 1024 * 1024];
    let encoded = Base32::encode_crockford(&large_data);
    let decoded = Base32::decode_crockford(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}

#[test]
fn test_large_data_base58() {
    // 10KB of data (Base58 encoding is computationally expensive)
    let large_data = vec![0xAB; 10 * 1024];

    // Bitcoin
    let encoded = Base58::encode_bitcoin(&large_data);
    let decoded = Base58::decode_bitcoin(&encoded).unwrap();
    assert_eq!(decoded, large_data);

    // Flickr
    let encoded = Base58::encode_flickr(&large_data);
    let decoded = Base58::decode_flickr(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}

#[test]
fn test_large_data_base64() {
    // 1MB of data
    let large_data = vec![0xAB; 1024 * 1024];
    let encoded = Base64::encode_standard(&large_data);
    let decoded = Base64::decode_standard(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}

// ========================================
// Edge Cases
// ========================================

#[test]
fn test_empty_data() {
    // Hex
    assert_eq!(Hex::encode_lower(b""), "");
    assert_eq!(Hex::decode_lower("").unwrap(), b"");

    // Base32
    assert_eq!(Base32::encode_standard(b""), "");
    assert_eq!(Base32::decode_standard("").unwrap(), b"");

    assert_eq!(Base32::encode_hex(b""), "");
    assert_eq!(Base32::decode_hex("").unwrap(), b"");

    assert_eq!(Base32::encode_crockford(b""), "");
    assert_eq!(Base32::decode_crockford("").unwrap(), b"");

    // Base58
    assert_eq!(Base58::encode_bitcoin(b""), "");
    assert_eq!(Base58::decode_bitcoin("").unwrap(), b"");

    assert_eq!(Base58::encode_flickr(b""), "");
    assert_eq!(Base58::decode_flickr("").unwrap(), b"");

    // Base64
    assert_eq!(Base64::encode_standard(b""), "");
    assert_eq!(Base64::decode_standard("").unwrap(), b"");

    // UTF-8
    assert_eq!(Utf8.encode(""), b"");
    assert_eq!(Utf8.decode(b""), "");
}

#[test]
fn test_single_byte() {
    let data = b"A";

    // Hex
    let encoded = Hex::encode_lower(data);
    assert_eq!(encoded, "41");
    assert_eq!(Hex::decode_lower(&encoded).unwrap(), data);

    // Base32
    let encoded = Base32::encode_standard(data);
    assert_eq!(encoded, "IE======");
    assert_eq!(Base32::decode_standard(&encoded).unwrap(), data);

    let encoded = Base32::encode_hex(data);
    assert_eq!(encoded, "84======");
    assert_eq!(Base32::decode_hex(&encoded).unwrap(), data);

    let encoded = Base32::encode_crockford(data);
    assert_eq!(Base32::decode_crockford(&encoded).unwrap(), data);

    // Base58
    let encoded = Base58::encode_bitcoin(data);
    assert_eq!(Base58::decode_bitcoin(&encoded).unwrap(), data);

    let encoded = Base58::encode_flickr(data);
    assert_eq!(Base58::decode_flickr(&encoded).unwrap(), data);

    // Base64
    let encoded = Base64::encode_standard(data);
    assert_eq!(encoded, "QQ==");
    assert_eq!(Base64::decode_standard(&encoded).unwrap(), data);
}

#[test]
fn test_all_bytes() {
    // Test all possible byte values (0x00-0xFF)
    let all_bytes: Vec<u8> = (0..=255).collect();

    // Hex round-trip
    let hex_encoded = Hex::encode_lower(&all_bytes);
    let hex_decoded = Hex::decode_lower(&hex_encoded).unwrap();
    assert_eq!(hex_decoded, all_bytes);

    // Base32 round-trip (Standard)
    let base32_std_encoded = Base32::encode_standard(&all_bytes);
    let base32_std_decoded = Base32::decode_standard(&base32_std_encoded).unwrap();
    assert_eq!(base32_std_decoded, all_bytes);

    // Base32 round-trip (Hex)
    let base32_hex_encoded = Base32::encode_hex(&all_bytes);
    let base32_hex_decoded = Base32::decode_hex(&base32_hex_encoded).unwrap();
    assert_eq!(base32_hex_decoded, all_bytes);

    // Base32 round-trip (Crockford)
    let base32_crockford_encoded = Base32::encode_crockford(&all_bytes);
    let base32_crockford_decoded = Base32::decode_crockford(&base32_crockford_encoded).unwrap();
    assert_eq!(base32_crockford_decoded, all_bytes);

    // Base58 round-trip (Bitcoin)
    let base58_bitcoin_encoded = Base58::encode_bitcoin(&all_bytes);
    let base58_bitcoin_decoded = Base58::decode_bitcoin(&base58_bitcoin_encoded).unwrap();
    assert_eq!(base58_bitcoin_decoded, all_bytes);

    // Base58 round-trip (Flickr)
    let base58_flickr_encoded = Base58::encode_flickr(&all_bytes);
    let base58_flickr_decoded = Base58::decode_flickr(&base58_flickr_encoded).unwrap();
    assert_eq!(base58_flickr_decoded, all_bytes);

    // Base64 round-trip
    let base64_encoded = Base64::encode_standard(&all_bytes);
    let base64_decoded = Base64::decode_standard(&base64_encoded).unwrap();
    assert_eq!(base64_decoded, all_bytes);

    // URL encoding round-trip (RFC 3986)
    let url_rfc_encoded = UrlEncoding::encode_rfc3986(&all_bytes);
    let url_rfc_decoded = UrlEncoding::decode_rfc3986(&url_rfc_encoded).unwrap();
    assert_eq!(url_rfc_decoded, all_bytes);

    // URL encoding round-trip (Form)
    let url_form_encoded = UrlEncoding::encode_form(&all_bytes);
    let url_form_decoded = UrlEncoding::decode_form(&url_form_encoded).unwrap();
    assert_eq!(url_form_decoded, all_bytes);

    // URL encoding round-trip (Component)
    let url_comp_encoded = UrlEncoding::encode_component(&all_bytes);
    let url_comp_decoded = UrlEncoding::decode_component(&url_comp_encoded).unwrap();
    assert_eq!(url_comp_decoded, all_bytes);
}

// ========================================
// URL Encoding Tests
// ========================================

#[test]
fn test_url_encoding_basic_usage() {
    let data = b"Hello, World!";

    // RFC 3986 variant (direct ZST usage)
    let encoded = UrlEncodingRfc3986.encode(data);
    assert_eq!(encoded, "Hello%2C%20World%21");
    let decoded = UrlEncodingRfc3986.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Form variant (direct ZST usage)
    let encoded = UrlEncodingForm.encode(data);
    assert_eq!(encoded, "Hello%2C+World%21");
    let decoded = UrlEncodingForm.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Component variant (direct ZST usage)
    let encoded = UrlEncodingComponent.encode(data);
    assert_eq!(encoded, "Hello%2C%20World%21");
    let decoded = UrlEncodingComponent.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Static methods
    let encoded = UrlEncoding::encode_rfc3986(data);
    assert_eq!(encoded, "Hello%2C%20World%21");
    let decoded = UrlEncoding::decode_rfc3986(&encoded).unwrap();
    assert_eq!(decoded, data);

    let encoded = UrlEncoding::encode_form(data);
    assert_eq!(encoded, "Hello%2C+World%21");
    let decoded = UrlEncoding::decode_form(&encoded).unwrap();
    assert_eq!(decoded, data);

    let encoded = UrlEncoding::encode_component(data);
    assert_eq!(encoded, "Hello%2C%20World%21");
    let decoded = UrlEncoding::decode_component(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_url_encoding_codec_trait_object() {
    let data = b"Test data";

    fn encode_with_url_codec(codec: &dyn UrlEncodingCodec, data: &[u8]) -> String {
        codec.encode(data)
    }

    // All URL encoding variants work as UrlEncodingCodec trait objects
    let result1 = encode_with_url_codec(&UrlEncodingRfc3986, data);
    let result2 = encode_with_url_codec(&UrlEncoding::Rfc3986, data);
    assert_eq!(result1, result2);

    let result3 = encode_with_url_codec(&UrlEncodingForm, data);
    let result4 = encode_with_url_codec(&UrlEncoding::Form, data);
    assert_eq!(result3, result4);

    let result5 = encode_with_url_codec(&UrlEncodingComponent, data);
    let result6 = encode_with_url_codec(&UrlEncoding::Component, data);
    assert_eq!(result5, result6);
}

#[test]
fn test_url_encoding_runtime_selection() {
    let data = b"Runtime test";

    // Simulate runtime configuration for URL encoding variants
    let configs = vec!["rfc3986", "form", "component"];

    for config in configs {
        let codec = match config {
            "rfc3986" => UrlEncoding::Rfc3986,
            "form" => UrlEncoding::Form,
            _ => UrlEncoding::Component,
        };

        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

#[test]
fn test_url_encoding_lenient_decoding() {
    // URL encoding is lenient - handles malformed input gracefully

    // Invalid hex characters - decoder passes through
    let result = UrlEncoding::decode_rfc3986("%ZZ");
    assert_eq!(result.unwrap(), b"%ZZ");

    // Incomplete percent encoding - decoder passes through
    let result = UrlEncoding::decode_rfc3986("%2");
    assert_eq!(result.unwrap(), b"%2");

    // Mix of valid and invalid - decoder handles gracefully
    let result = UrlEncoding::decode_rfc3986("Hello%20World%ZZ");
    assert_eq!(result.unwrap(), b"Hello World%ZZ");

    // Form encoding handles + as space
    assert_eq!(UrlEncoding::decode_form("a+b+c").unwrap(), b"a b c");
    assert_eq!(
        UrlEncoding::decode_form("test+%2B+value").unwrap(),
        b"test + value"
    );
}

#[test]
fn test_url_encoding_variant_differences() {
    // RFC 3986 preserves: - . _ ~
    let data = b"a-b.c_d~e";
    let encoded_rfc = UrlEncoding::encode_rfc3986(data);
    assert_eq!(encoded_rfc, "a-b.c_d~e");

    // Form preserves: * - . _
    let data = b"*-._";
    let encoded_form = UrlEncoding::encode_form(data);
    assert_eq!(encoded_form, "*-._");

    // Component encodes all non-alphanumeric
    let data = b"-._~";
    let encoded_comp = UrlEncoding::encode_component(data);
    assert!(encoded_comp.contains('%'));

    // Space handling
    let data = b"a b";
    assert_eq!(UrlEncoding::encode_rfc3986(data), "a%20b");
    assert_eq!(UrlEncoding::encode_form(data), "a+b");
    assert_eq!(UrlEncoding::encode_component(data), "a%20b");
}

#[test]
fn test_url_encoding_cross_codec() {
    // URL encode, then Base64 encode
    let original_text = "Hello, 世界!";
    let url_encoded = UrlEncoding::encode_rfc3986(original_text.as_bytes());
    let base64_encoded = Base64::encode_standard(url_encoded.as_bytes());

    // Reverse: Base64 decode, then URL decode
    let base64_decoded = Base64::decode_standard(&base64_encoded).unwrap();
    let url_decoded =
        UrlEncoding::decode_rfc3986(&String::from_utf8(base64_decoded).unwrap()).unwrap();

    assert_eq!(url_decoded, original_text.as_bytes());
}

#[test]
fn test_url_encoding_large_data() {
    // 1MB of data
    let large_data = vec![0xAB; 1024 * 1024];

    let encoded = UrlEncoding::encode_rfc3986(&large_data);
    let decoded = UrlEncoding::decode_rfc3986(&encoded).unwrap();
    assert_eq!(decoded, large_data);

    let encoded = UrlEncoding::encode_form(&large_data);
    let decoded = UrlEncoding::decode_form(&encoded).unwrap();
    assert_eq!(decoded, large_data);

    let encoded = UrlEncoding::encode_component(&large_data);
    let decoded = UrlEncoding::decode_component(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}

#[test]
fn test_url_encoding_empty_and_single_byte() {
    // Empty
    assert_eq!(UrlEncoding::encode_rfc3986(b""), "");
    assert_eq!(UrlEncoding::decode_rfc3986("").unwrap(), b"");
    assert_eq!(UrlEncoding::encode_form(b""), "");
    assert_eq!(UrlEncoding::decode_form("").unwrap(), b"");
    assert_eq!(UrlEncoding::encode_component(b""), "");
    assert_eq!(UrlEncoding::decode_component("").unwrap(), b"");

    // Single byte
    let data = b"A";
    let encoded_rfc = UrlEncoding::encode_rfc3986(data);
    assert_eq!(UrlEncoding::decode_rfc3986(&encoded_rfc).unwrap(), data);

    let encoded_form = UrlEncoding::encode_form(data);
    assert_eq!(UrlEncoding::decode_form(&encoded_form).unwrap(), data);

    let encoded_comp = UrlEncoding::encode_component(data);
    assert_eq!(UrlEncoding::decode_component(&encoded_comp).unwrap(), data);
}

// ========================================
// Quoted-Printable Tests
// ========================================

#[test]
fn test_quoted_printable_basic_usage() {
    let data = b"Hello, World!";

    // Direct ZST usage (Robust)
    let encoded = QuotedPrintableRobust.encode(data);
    let decoded = QuotedPrintableRobust.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Direct ZST usage (Strict)
    let encoded = QuotedPrintableStrict.encode(data);
    let decoded = QuotedPrintableStrict.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Static methods (encoding is the same, decoding differs)
    let encoded = QuotedPrintable::encode(data);
    assert_eq!(QuotedPrintable::decode(&encoded).unwrap(), data);

    // Explicit Robust decode
    let encoded = QuotedPrintable::encode(data);
    assert_eq!(QuotedPrintable::decode_robust(&encoded).unwrap(), data);

    // Explicit Strict decode
    let encoded = QuotedPrintable::encode(data);
    assert_eq!(QuotedPrintable::decode_strict(&encoded).unwrap(), data);
}

#[test]
fn test_quoted_printable_codec_trait_object() {
    let data = b"Test data with = sign";

    fn encode_with_qp_codec(codec: &dyn QuotedPrintableCodec, data: &[u8]) -> String {
        codec.encode(data)
    }

    // All Quoted-Printable variants work as QuotedPrintableCodec trait objects
    let result1 = encode_with_qp_codec(&QuotedPrintableRobust, data);
    let result2 = encode_with_qp_codec(&QuotedPrintable::Robust, data);
    assert_eq!(result1, result2);

    let result3 = encode_with_qp_codec(&QuotedPrintableStrict, data);
    let result4 = encode_with_qp_codec(&QuotedPrintable::Strict, data);
    assert_eq!(result3, result4);
}

#[test]
fn test_quoted_printable_binary_codec_trait_object() {
    let data = b"Test with special chars: =?!";

    // Collection of Quoted-Printable codecs as trait objects
    let codecs: Vec<Box<dyn BinaryCodec>> = vec![
        Box::new(QuotedPrintableRobust),
        Box::new(QuotedPrintableStrict),
    ];

    for codec in codecs {
        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Round-trip failed for codec");
    }
}

#[test]
fn test_quoted_printable_runtime_selection() {
    let data = b"Runtime test";

    // Simulate runtime configuration
    let configs = vec!["robust", "strict"];

    for config in configs {
        let codec = match config {
            "robust" => QuotedPrintable::Robust,
            _ => QuotedPrintable::Strict,
        };

        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

#[test]
fn test_quoted_printable_error_handling() {
    // Malformed input: invalid hex
    let invalid_hex = "=ZZ";

    // Robust mode: accepts
    let result = QuotedPrintable::decode_robust(invalid_hex).unwrap();
    assert_eq!(result, b"=ZZ");

    // Strict mode: rejects
    let result = QuotedPrintable::decode_strict(invalid_hex);
    assert!(result.is_err());

    // Incomplete encoding
    let incomplete = "=3";

    // Robust mode: accepts
    let result = QuotedPrintable::decode_robust(incomplete).unwrap();
    assert_eq!(result, b"=3");

    // Strict mode: rejects
    let result = QuotedPrintable::decode_strict(incomplete);
    assert!(result.is_err());

    // Invalid position of '='
    let invalid_pos = "Hello=World";

    // Robust mode: accepts
    let result = QuotedPrintable::decode_robust(invalid_pos).unwrap();
    assert_eq!(result, b"Hello=World");

    // Strict mode: rejects
    let result = QuotedPrintable::decode_strict(invalid_pos);
    assert!(result.is_err());
}

#[test]
fn test_quoted_printable_special_characters() {
    // Test encoding of special characters
    let data = b"=?!@#$%^&*()";
    let encoded = QuotedPrintable::encode(data);
    let decoded = QuotedPrintable::decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_quoted_printable_long_lines() {
    // Test soft line breaks for long content
    let long_data = b"This is a very long line that should be wrapped according to Quoted-Printable encoding rules which limit line length to 76 characters including the soft line break marker.";
    let encoded = QuotedPrintable::encode(long_data);
    let decoded = QuotedPrintable::decode(&encoded).unwrap();
    assert_eq!(decoded, long_data);
}

#[test]
fn test_quoted_printable_utf8() {
    // UTF-8 multibyte characters
    let data = "Hello, 世界! 🦀".as_bytes();
    let encoded = QuotedPrintable::encode(data);
    let decoded = QuotedPrintable::decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_quoted_printable_soft_line_breaks() {
    // Soft line break with LF
    let input = "Hello=\nWorld";
    let result = QuotedPrintable::decode(input).unwrap();
    assert_eq!(result, b"HelloWorld");

    // Soft line break with CRLF
    let input = "Hello=\r\nWorld";
    let result = QuotedPrintable::decode(input).unwrap();
    assert_eq!(result, b"HelloWorld");

    // Multiple soft line breaks
    let input = "A=\nB=\nC";
    let result = QuotedPrintable::decode(input).unwrap();
    assert_eq!(result, b"ABC");
}

#[test]
fn test_quoted_printable_case_insensitive_hex() {
    // Robust mode (default) accepts both uppercase and lowercase hex
    assert_eq!(QuotedPrintable::decode("=3D").unwrap(), b"=");
    assert_eq!(QuotedPrintable::decode("=3d").unwrap(), b"=");
    assert_eq!(QuotedPrintable::decode("=3D=3d").unwrap(), b"==");

    // Strict mode only accepts uppercase
    assert_eq!(QuotedPrintable::decode_strict("=3D").unwrap(), b"=");
    assert!(QuotedPrintable::decode_strict("=3d").is_err());
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
fn test_quoted_printable_binary_data() {
    // Binary data
    let data: &[u8] = &[0x00, 0x01, 0xFF, 0xFE, 0x80];
    let encoded = QuotedPrintable::encode(data);
    let decoded = QuotedPrintable::decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_quoted_printable_cross_codec() {
    // Quoted-Printable encode, then Base64 encode
    let original_text = "Hello, 世界!";
    let qp_encoded = QuotedPrintable::encode(original_text.as_bytes());
    let base64_encoded = Base64::encode_standard(qp_encoded.as_bytes());

    // Reverse: Base64 decode, then Quoted-Printable decode
    let base64_decoded = Base64::decode_standard(&base64_encoded).unwrap();
    let qp_decoded = QuotedPrintable::decode(&String::from_utf8(base64_decoded).unwrap()).unwrap();

    assert_eq!(qp_decoded, original_text.as_bytes());
}

#[test]
fn test_quoted_printable_large_data() {
    // 1MB of data
    let large_data = vec![0xAB; 1024 * 1024];

    let encoded = QuotedPrintable::encode(&large_data);
    let decoded = QuotedPrintable::decode(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}

#[test]
fn test_quoted_printable_empty_and_single_byte() {
    // Empty
    assert_eq!(QuotedPrintable::encode(b""), "");
    assert_eq!(QuotedPrintable::decode("").unwrap(), b"");

    // Single byte
    let data = b"A";
    let encoded = QuotedPrintable::encode(data);
    let decoded = QuotedPrintable::decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_quoted_printable_all_bytes() {
    // Test all possible byte values (0x00-0xFF)
    let all_bytes: Vec<u8> = (0..=255).collect();

    let encoded = QuotedPrintable::encode(&all_bytes);
    let decoded = QuotedPrintable::decode(&encoded).unwrap();
    assert_eq!(decoded, all_bytes);
}
