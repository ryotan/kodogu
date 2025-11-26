//! Edge case tests for all codecs
//!
//! Tests boundary conditions like empty data, single byte, and all byte values.

use codec::{Base32, Base58, Base64, BinaryCodec, Hex, QuotedPrintable, UrlEncoding, Utf8, TextCodec};

#[test]
fn test_empty_data() {
    let empty: Vec<u8> = vec![];

    // All codecs should handle empty data
    assert_eq!(Hex::encode(&empty), "");
    assert_eq!(Hex::decode("").unwrap(), empty);

    assert_eq!(Base32::encode_standard(&empty), "");
    assert_eq!(Base32::decode_standard("").unwrap(), empty);

    assert_eq!(Base58::encode(&empty), "");
    assert_eq!(Base58::decode("").unwrap(), empty);

    assert_eq!(Base64::encode_standard(&empty), "");
    assert_eq!(Base64::decode_standard("").unwrap(), empty);

    assert_eq!(UrlEncoding::encode_rfc3986(&empty), "");
    assert_eq!(UrlEncoding::decode_rfc3986("").unwrap(), empty);

    assert_eq!(QuotedPrintable::encode(&empty), "");
    assert_eq!(QuotedPrintable::decode("").unwrap(), empty);

    assert_eq!(Utf8.encode(""), empty);
    assert_eq!(Utf8.decode(&empty), "");
}

#[test]
fn test_single_byte() {
    let single = vec![0x42]; // 'B'

    // Test round-trip for single byte
    let hex_encoded = Hex::encode(&single);
    assert_eq!(Hex::decode(&hex_encoded).unwrap(), single);

    let b64_encoded = Base64::encode_standard(&single);
    assert_eq!(Base64::decode_standard(&b64_encoded).unwrap(), single);

    let b32_encoded = Base32::encode_standard(&single);
    assert_eq!(Base32::decode_standard(&b32_encoded).unwrap(), single);
}

#[test]
fn test_all_bytes() {
    // Test with all possible byte values
    let all_bytes: Vec<u8> = (0..=255).collect();

    // Hex
    let hex_encoded = Hex::encode(&all_bytes);
    assert_eq!(Hex::decode(&hex_encoded).unwrap(), all_bytes);

    // Base64
    let b64_encoded = Base64::encode_standard(&all_bytes);
    assert_eq!(Base64::decode_standard(&b64_encoded).unwrap(), all_bytes);

    // Base32
    let b32_encoded = Base32::encode_standard(&all_bytes);
    assert_eq!(Base32::decode_standard(&b32_encoded).unwrap(), all_bytes);

    // Base58
    let b58_encoded = Base58::encode(&all_bytes);
    assert_eq!(Base58::decode(&b58_encoded).unwrap(), all_bytes);

    // URL Encoding
    let url_encoded = UrlEncoding::encode_rfc3986(&all_bytes);
    assert_eq!(UrlEncoding::decode_rfc3986(&url_encoded).unwrap(), all_bytes);

    // Quoted-Printable
    let qp_encoded = QuotedPrintable::encode(&all_bytes);
    assert_eq!(QuotedPrintable::decode(&qp_encoded).unwrap(), all_bytes);
}

#[test]
fn test_max_byte() {
    let max_byte = vec![0xFF];

    for codec in [
        &Hex::encode(&max_byte),
        &Base64::encode_standard(&max_byte),
        &Base32::encode_standard(&max_byte),
    ] {
        assert!(!codec.is_empty());
    }
}
