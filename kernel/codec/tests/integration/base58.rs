//! Integration tests for Base58 codec

use codec::{Base58, Base58Bitcoin, Base58Codec, Base58Flickr, BinaryCodec};

#[test]
fn test_base58_basic_usage() {
    let data = b"Hello, World!";

    // Bitcoin variant
    let encoded = Base58Bitcoin.encode(data);
    assert_eq!(encoded, "72k1xXWG59fYdzSNoA");
    let decoded = Base58Bitcoin.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Flickr variant
    let encoded = Base58Flickr.encode(data);
    let decoded = Base58Flickr.decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Static methods
    let encoded = Base58::encode(data);
    assert_eq!(encoded, "72k1xXWG59fYdzSNoA");
}

#[test]
fn test_base58_codec_trait_object() {
    let data = b"Test data";
    let codec: &dyn Base58Codec = &Base58Bitcoin;
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_base58_error_handling() {
    // Invalid characters (0, O, I, l are excluded)
    assert!(Base58::decode_bitcoin("0OIl").is_err());
    assert!(Base58::decode("Invalid!@#").is_err());
}

#[test]
fn test_base58_custom_alphabet() {
    let data = b"Custom test";
    // Ripple alphabet
    let alphabet = "rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";
    let codec = Base58::new(alphabet).unwrap();
    let encoded = codec.encode(data);
    let decoded = codec.decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_large_data_base58() {
    // 10KB (Base58 is computationally expensive)
    let large_data = vec![0xAB; 10 * 1024];
    let encoded = Base58::encode_bitcoin(&large_data);
    let decoded = Base58::decode_bitcoin(&encoded).unwrap();
    assert_eq!(decoded, large_data);
}
