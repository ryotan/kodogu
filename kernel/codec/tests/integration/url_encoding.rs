//! Integration tests for URL Encoding codec

use codec::{
    BinaryCodec, UrlEncoding, UrlEncodingCodec, UrlEncodingComponent, UrlEncodingForm,
    UrlEncodingRfc3986,
};

#[test]
fn test_url_encoding_basic_usage() {
    let data = b"Hello World!";

    // RFC 3986 (space as %20)
    let encoded = UrlEncodingRfc3986.encode(data);
    assert!(encoded.contains("%20"));
    let decoded = UrlEncodingRfc3986.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Form encoding (space as +)
    let encoded = UrlEncodingForm.encode(data);
    assert!(encoded.contains('+'));
    let decoded = UrlEncodingForm.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Component (most restrictive)
    let encoded = UrlEncodingComponent.encode(data);
    let decoded = UrlEncodingComponent.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_url_encoding_codec_trait_object() {
    let data = b"Test data";
    let codec: &dyn UrlEncodingCodec = &UrlEncodingRfc3986;
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_url_encoding_variant_differences() {
    let data = b"a b";

    // RFC 3986: space → %20
    let rfc = UrlEncoding::encode_rfc3986(data);
    assert_eq!(rfc, "a%20b");

    // Form: space → +
    let form = UrlEncoding::encode_form(data);
    assert_eq!(form, "a+b");
}

#[test]
fn test_url_encoding_lenient_decoding() {
    // Lenient decoder never fails
    assert!(UrlEncoding::decode_rfc3986("any string").is_ok());
    assert!(UrlEncoding::decode_form("any+string").is_ok());
}

#[test]
fn test_large_data_url_encoding() {
    let large_data = vec![0xAB; 10 * 1024];
    let encoded = UrlEncoding::encode_rfc3986(&large_data);
    let decoded = UrlEncoding::decode_rfc3986(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}
