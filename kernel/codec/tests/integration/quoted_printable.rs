//! Integration tests for Quoted-Printable codec

use codec::{
    BinaryCodec, QuotedPrintable, QuotedPrintableCodec, QuotedPrintableRobust,
    QuotedPrintableStrict,
};

#[test]
fn test_quoted_printable_basic_usage() {
    let data = b"Hello, World!";

    // Robust mode
    let encoded = QuotedPrintableRobust.encode(data);
    let decoded = QuotedPrintableRobust.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Strict mode
    let encoded = QuotedPrintableStrict.encode(data);
    let decoded = QuotedPrintableStrict.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_quoted_printable_codec_trait_object() {
    let data = b"Test data";
    let codec: &dyn QuotedPrintableCodec = &QuotedPrintableRobust;
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_quoted_printable_special_chars() {
    // Control characters and high bytes are encoded
    let data = vec![0x00, 0xFF, 0x0A];
    let encoded = QuotedPrintable::encode(&data);
    assert!(encoded.contains('='));

    let decoded = QuotedPrintable::decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_quoted_printable_case_insensitive_hex() {
    let data = vec![0x0A]; // Control character
    let encoded = QuotedPrintable::encode(&data);

    // Convert to lowercase
    let encoded_lower = encoded.to_lowercase();

    // Robust mode handles both cases
    let decoded = QuotedPrintableRobust.decode(&encoded_lower).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_large_data_quoted_printable() {
    let large_data = vec![0xAB; 10 * 1024];
    let encoded = QuotedPrintable::encode(&large_data);
    let decoded = QuotedPrintable::decode(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}
