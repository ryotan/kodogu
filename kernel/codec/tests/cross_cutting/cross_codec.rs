//! Cross-codec integration tests
//!
//! Tests that validate codec layering and combinations work correctly.

use codec::{Base32, Base58, Base64, Hex, UrlEncoding};

#[test]
fn test_multiple_encoding_layers() {
    let data = b"Layered encoding test";

    // Layer 1: Base64 encode
    let layer1 = Base64::encode_standard(data);

    // Layer 2: Hex encode the Base64 string
    let layer2 = Hex::encode_lower(layer1.as_bytes());

    // Layer 3: URL encode the Hex string
    let layer3 = UrlEncoding::encode_rfc3986(layer2.as_bytes());

    // Reverse: URL decode
    let decoded3 = UrlEncoding::decode_rfc3986(&layer3).unwrap();
    let decoded3_str = String::from_utf8(decoded3).unwrap();

    // Reverse: Hex decode
    let decoded2 = Hex::decode_lower(&decoded3_str).unwrap();
    let decoded2_str = String::from_utf8(decoded2).unwrap();

    // Reverse: Base64 decode
    let decoded1 = Base64::decode_standard(&decoded2_str).unwrap();

    // Should recover original data
    assert_eq!(decoded1, data);
}

#[test]
fn test_hex_of_base64() {
    let data = b"Test";

    // Encode as Base64, then Hex
    let base64 = Base64::encode_standard(data);
    let hex = Hex::encode_lower(base64.as_bytes());

    // Decode Hex, then Base64
    let decoded_hex = Hex::decode_lower(&hex).unwrap();
    let decoded_base64 = Base64::decode_standard(&String::from_utf8(decoded_hex).unwrap()).unwrap();

    assert_eq!(decoded_base64, data);
}

#[test]
fn test_base32_of_base58() {
    let data = b"Chain test";

    // Base58 → Base32
    let base58 = Base58::encode_bitcoin(data);
    let base32 = Base32::encode_standard(base58.as_bytes());

    // Base32 → Base58
    let decoded32 = Base32::decode_standard(&base32).unwrap();
    let decoded58 = Base58::decode_bitcoin(&String::from_utf8(decoded32).unwrap()).unwrap();

    assert_eq!(decoded58, data);
}
