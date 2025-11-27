//! Integration tests for Hex codec
//!
//! Tests the public API from a consumer's perspective, validating that
//! Hex encoding/decoding works correctly across different usage patterns.

use codec::{BinaryCodec, Hex, HexCodec, HexLower, HexUpper};

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

// ========================================
// Trait Object Tests
// ========================================

#[test]
fn test_hex_codec_trait_object() {
    let data = b"test data";

    // HexCodec trait (specific to Hex)
    let codecs: Vec<&dyn HexCodec> = vec![&HexLower, &HexUpper, &Hex::Lower];

    for codec in codecs {
        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    // BinaryCodec trait (generic)
    let codec: &dyn BinaryCodec = &HexLower;
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

// ========================================
// Error Handling Tests
// ========================================

#[test]
fn test_hex_error_handling() {
    // Odd length hex string
    let result = Hex::decode("123");
    assert!(result.is_err());

    // Invalid characters
    let result = Hex::decode("12GH");
    assert!(result.is_err());

    // Empty string is valid (decodes to empty vec)
    let result = Hex::decode("");
    assert_eq!(result.unwrap(), Vec::<u8>::new());
}

// ========================================
// Custom Alphabet Tests
// ========================================

#[test]
fn test_hex_custom_alphabet() {
    // Create custom hex codec
    let codec = Hex::new("0123456789ABCDEF").unwrap();
    let data = b"Hi";

    let encoded = codec.encode(data);
    assert_eq!(encoded, "4869");

    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Invalid alphabet (wrong length)
    let result = Hex::new("0123456789ABC");
    assert!(result.is_err());

    // Invalid alphabet (duplicate characters)
    let result = Hex::new("0123456789ABCDEA");
    assert!(result.is_err());
}

// ========================================
// Large Data Tests
// ========================================

#[test]
fn test_large_data_hex() {
    // Test with 1MB of data
    let data: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();

    let encoded = HexLower.encode(&data);
    assert_eq!(encoded.len(), data.len() * 2); // Hex doubles the size

    let decoded = HexLower.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}
