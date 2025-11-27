//! Integration tests for Base32 codec

use codec::{
    Base32, Base32Codec, Base32CrockfordLower, Base32CrockfordUpper, Base32Hex, Base32Standard,
    BinaryCodec,
};

#[test]
fn test_base32_basic_usage() {
    let data = b"Hello, World!";

    // Standard variant
    let encoded = Base32Standard.encode(data);
    assert_eq!(encoded, "JBSWY3DPFQQFO33SNRSCC===");
    let decoded = Base32Standard.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Hex variant
    let encoded = Base32Hex.encode(data);
    assert_eq!(encoded, "91IMOR3F5GG5ERRIDHI22===");
    let decoded = Base32Hex.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Crockford variants
    let encoded_upper = Base32CrockfordUpper.encode(data);
    let decoded = Base32CrockfordUpper.decode(&encoded_upper).unwrap();
    assert_eq!(decoded, data);

    let encoded_lower = Base32CrockfordLower.encode(data);
    let decoded = Base32CrockfordLower.decode(&encoded_lower).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_base32_codec_trait_object() {
    let data = b"Test data";
    let codec: &dyn Base32Codec = &Base32Standard;
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_base32_error_handling() {
    assert!(Base32::decode_standard("Invalid!@#").is_err());
    assert!(Base32Standard.decode("01234567").is_err()); // Hex chars in Standard
    assert!(Base32Hex.decode("WXYZ2345").is_err()); // Standard chars in Hex
}

#[test]
fn test_base32_crockford_features() {
    let data = b"Test";
    let encoded = Base32CrockfordUpper.encode(data);

    // Crockford confusion-resistant: I→1, O→0
    let with_i = encoded.replace('1', "I");
    assert_eq!(Base32CrockfordUpper.decode(&with_i).unwrap(), data);

    let with_o = encoded.replace('0', "O");
    assert_eq!(Base32CrockfordUpper.decode(&with_o).unwrap(), data);
}

#[test]
fn test_base32_custom_alphabet() {
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let codec = Base32::new(alphabet).unwrap();
    let data = b"Custom test";
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_large_data_base32() {
    let large_data = vec![0xAB; 1024 * 1024];
    let encoded = Base32::encode_standard(&large_data);
    let decoded = Base32::decode_standard(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}
