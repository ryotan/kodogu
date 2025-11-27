//! Integration tests for Base64 codec
//!
//! Tests the public API from a consumer's perspective, validating that
//! Base64 encoding/decoding works correctly across different variants.

use codec::{Base64, Base64Codec, Base64Standard, Base64UrlSafe, BinaryCodec};

// ========================================
// Basic Usage Tests
// ========================================

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

// ========================================
// Trait Object Tests
// ========================================

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

    // BinaryCodec trait (generic)
    let codec: &dyn BinaryCodec = &Base64Standard;
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

// ========================================
// Error Handling Tests
// ========================================

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

    // Empty string is valid
    let result = Base64::decode_standard("");
    assert_eq!(result.unwrap(), Vec::<u8>::new());
}

// ========================================
// Custom Alphabet Tests
// ========================================

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
fn test_base64_padding() {
    let data = b"Test";

    // Standard encoding includes padding
    let encoded = Base64::encode_standard(data);
    assert!(encoded.ends_with('='));

    let decoded = Base64::decode_standard(&encoded).unwrap();
    assert_eq!(decoded, data);
}

// ========================================
// Variant Differences Tests
// ========================================

#[test]
fn test_base64_variant_differences() {
    let data = b"Test \xFF\xFE data";

    // Standard uses +/
    let standard = Base64Standard.encode(data);
    assert!(standard.contains('+') || standard.contains('/'));

    // URL-safe uses -_
    let url_safe = Base64UrlSafe.encode(data);
    assert!(!url_safe.contains('+'));
    assert!(!url_safe.contains('/'));
}

// ========================================
// Large Data Tests
// ========================================

#[test]
fn test_large_data_base64() {
    // 1MB of data
    let large_data = vec![0xAB; 1024 * 1024];
    let encoded = Base64::encode_standard(&large_data);
    let decoded = Base64::decode_standard(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}
