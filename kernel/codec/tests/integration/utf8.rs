//! Integration tests for UTF-8 codec

use codec::{TextCodec, Utf8};

#[test]
fn test_utf8_basic_usage() {
    let text = "Hello, World!";

    // Encode
    let bytes = Utf8.encode(text);
    assert_eq!(bytes, text.as_bytes());

    // Decode
    let decoded = Utf8.decode(&bytes);
    assert_eq!(decoded, text);
}

#[test]
fn test_utf8_emoji_and_special_chars() {
    let text = "こんにちは🦀Rust!";
    let bytes = Utf8.encode(text);
    let decoded = Utf8.decode(&bytes);
    assert_eq!(decoded, text);
}

#[test]
fn test_utf8_strict_mode() {
    let text = "Valid UTF-8";
    let bytes = Utf8.encode_strict(text).unwrap();
    let decoded = Utf8.decode_strict(&bytes).unwrap();
    assert_eq!(decoded, text);
}

#[test]
fn test_utf8_lossy_mode() {
    // Invalid UTF-8 sequence
    let invalid_bytes = vec![0xFF, 0xFE, 0xFD];

    // Lossy mode replaces invalid sequences with �
    let decoded = Utf8.decode(&invalid_bytes);
    assert!(decoded.contains('�'));

    // Strict mode returns error
    assert!(Utf8.decode_strict(&invalid_bytes).is_err());
}

#[test]
fn test_utf8_error_handling() {
    let invalid_bytes = vec![0xC0, 0x80]; // Overlong encoding
    assert!(Utf8.decode_strict(&invalid_bytes).is_err());

    // Lossy mode handles it gracefully
    let _decoded = Utf8.decode(&invalid_bytes);
}
