//! Cross-cutting integration tests for trait object usage
//!
//! Tests that all codecs work correctly when used through trait objects,
//! demonstrating polymorphism across the codec API.

use codec::{
    Base32Standard, Base58Bitcoin, Base64Standard, Base85Z85, BinaryCodec, HexLower,
    QuotedPrintableRobust, UrlEncodingRfc3986,
};

#[test]
fn test_binary_codec_trait_object() {
    // Z85はinputが4の倍数である必要があるため、24バイトのデータを使用
    let data = b"Test data for polymorphi";

    // All binary codecs work through BinaryCodec trait
    let codecs: Vec<&dyn BinaryCodec> = vec![
        &HexLower,
        &Base32Standard,
        &Base58Bitcoin,
        &Base64Standard,
        &Base85Z85,
        &UrlEncodingRfc3986,
        &QuotedPrintableRobust,
    ];

    for codec in codecs {
        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

#[test]
fn test_runtime_codec_selection() {
    // Z85はinputが4の倍数である必要があるため、12バイトのデータを使用
    let data = b"Runtime test";

    // Simulate runtime codec selection based on configuration
    fn encode_with_codec(codec_name: &str, data: &[u8]) -> String {
        let codec: &dyn BinaryCodec = match codec_name {
            "hex" => &HexLower,
            "base32" => &Base32Standard,
            "base58" => &Base58Bitcoin,
            "base64" => &Base64Standard,
            "base85" => &Base85Z85,
            _ => &HexLower,
        };
        codec.encode(data)
    }

    // Different codecs produce different outputs
    let hex_result = encode_with_codec("hex", data);
    let base64_result = encode_with_codec("base64", data);
    let base85_result = encode_with_codec("base85", data);
    assert_ne!(hex_result, base64_result);
    assert_ne!(hex_result, base85_result);
    assert_ne!(base64_result, base85_result);
}

#[test]
fn test_codec_in_vec() {
    // Z85はinputが4の倍数である必要があるため、16バイトのデータを使用
    let data = b"Collection tests";
    let codecs: Vec<Box<dyn BinaryCodec>> = vec![
        Box::new(HexLower),
        Box::new(Base64Standard),
        Box::new(Base32Standard),
        Box::new(Base85Z85),
    ];

    for codec in &codecs {
        let encoded = codec.encode(data);
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}
