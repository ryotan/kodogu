//! Integration tests for Base85 codec module
//!
//! Tests the public API as users would use it, including ZST types, enum variants,
//! static methods, trait objects, and marker traits.

use codec::{
    Base85, Base85Ascii85, Base85Btoa, Base85Codec, Base85Rfc1924, Base85Z85, BinaryCodec,
    BinaryCodecError,
};

// ============================================================================
// ZST API Tests
// ============================================================================

mod zst_api {
    use super::*;

    #[test]
    fn ascii85_basic_usage() {
        let data = b"Hello, World!";
        let encoded = Base85Ascii85.encode(data);
        assert!(encoded.starts_with("<~"));
        assert!(encoded.ends_with("~>"));

        let decoded = Base85Ascii85.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn z85_basic_usage() {
        // Z85 requires input length to be multiple of 4
        let data: [u8; 8] = [0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
        let encoded = Base85Z85.encode(&data);
        assert_eq!(encoded, "HelloWorld");

        let decoded = Base85Z85.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn rfc1924_basic_usage() {
        let data = b"Test";
        let encoded = Base85Rfc1924.encode(data);
        let decoded = Base85Rfc1924.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn btoa_basic_usage() {
        let data = b"Hello, World!";
        let encoded = Base85Btoa.encode(data);
        assert!(encoded.contains("xbtoa Begin"));
        assert!(encoded.contains("xbtoa End"));

        let decoded = Base85Btoa.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn rfc1924_u128_api() {
        let ipv6: u128 = 0x108000000000000000080800200C417A;
        let encoded = Base85Rfc1924::encode_u128(ipv6);
        assert_eq!(encoded, "4)+k&C#VzJ4br>0wv%Yp");
        assert_eq!(encoded.len(), 20);

        let decoded = Base85Rfc1924::decode_u128(&encoded).unwrap();
        assert_eq!(decoded, ipv6);
    }

    #[test]
    fn z85_padding_api() {
        // Input not a multiple of 4
        let data = b"Hello";
        let (encoded, padding) = Base85Z85.encode_with_padding(data);
        let decoded = Base85Z85.decode_with_padding(&encoded, padding).unwrap();
        assert_eq!(decoded, data);
    }
}

// ============================================================================
// Enum API Tests
// ============================================================================

mod enum_api {
    use super::*;

    #[test]
    fn default_is_z85() {
        let codec = Base85::default();
        assert_eq!(codec, Base85::Z85);
    }

    #[test]
    fn encode_decode_all_variants() {
        // Use data that's multiple of 4 for Z85 compatibility
        let data = b"TestData";

        for variant in [Base85::Ascii85, Base85::Z85, Base85::Rfc1924, Base85::Btoa] {
            let encoded = variant.encode(data);
            let decoded = variant.decode(&encoded).unwrap();
            assert_eq!(decoded, data, "Failed for {:?}", variant);
        }
    }

    #[test]
    fn runtime_variant_selection() {
        fn select_codec(name: &str) -> Base85 {
            match name {
                "ascii85" => Base85::Ascii85,
                "z85" => Base85::Z85,
                "rfc1924" => Base85::Rfc1924,
                "btoa" => Base85::Btoa,
                _ => Base85::default(),
            }
        }

        let data = b"TestData";
        let codec = select_codec("ascii85");
        let encoded = codec.encode(data);
        assert!(encoded.starts_with("<~"));

        let codec = select_codec("unknown");
        let encoded = codec.encode(data);
        // Default is Z85
        assert!(!encoded.starts_with("<~"));
    }
}

// ============================================================================
// Static Methods Tests
// ============================================================================

mod static_methods {
    use super::*;

    #[test]
    fn default_encode_decode() {
        // Static methods use Z85 by default
        let data = b"TestData";
        let encoded = Base85::encode(data);
        let decoded = Base85::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn variant_specific_static_methods() {
        let data = b"TestData";

        // Ascii85
        let encoded = Base85::encode_ascii85(data);
        assert!(encoded.starts_with("<~"));
        let decoded = Base85::decode_ascii85(&encoded).unwrap();
        assert_eq!(decoded, data);

        // Z85
        let encoded = Base85::encode_z85(data);
        let decoded = Base85::decode_z85(&encoded).unwrap();
        assert_eq!(decoded, data);

        // RFC 1924
        let encoded = Base85::encode_rfc1924(data);
        let decoded = Base85::decode_rfc1924(&encoded).unwrap();
        assert_eq!(decoded, data);

        // btoa
        let encoded = Base85::encode_btoa(data);
        assert!(encoded.contains("xbtoa"));
        let decoded = Base85::decode_btoa(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

// ============================================================================
// Trait Object Tests
// ============================================================================

mod trait_objects {
    use super::*;

    #[test]
    fn binary_codec_trait_object() {
        // Use data that's multiple of 4 for Z85 compatibility
        let data = b"TestData";
        let codecs: Vec<&dyn BinaryCodec> =
            vec![&Base85Ascii85, &Base85Z85, &Base85Rfc1924, &Base85Btoa];

        for codec in codecs {
            let encoded = codec.encode(data);
            let decoded = codec.decode(&encoded).unwrap();
            assert_eq!(decoded, data);
        }
    }

    #[test]
    fn base85_codec_trait_object() {
        fn encode_with_base85<C: Base85Codec>(codec: &C, data: &[u8]) -> String {
            codec.encode(data)
        }

        let data = b"TestData";
        let _ = encode_with_base85(&Base85Ascii85, data);
        let _ = encode_with_base85(&Base85Z85, data);
        let _ = encode_with_base85(&Base85Rfc1924, data);
        let _ = encode_with_base85(&Base85Btoa, data);
        let _ = encode_with_base85(&Base85::default(), data);
    }

    #[test]
    fn boxed_codecs() {
        // Use data that's multiple of 4 for Z85 compatibility
        let data = b"TestData";
        let codecs: Vec<Box<dyn BinaryCodec>> = vec![
            Box::new(Base85Ascii85),
            Box::new(Base85Z85),
            Box::new(Base85Rfc1924),
            Box::new(Base85Btoa),
        ];

        for codec in &codecs {
            let encoded = codec.encode(data);
            let decoded = codec.decode(&encoded).unwrap();
            assert_eq!(decoded, data);
        }
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn ascii85_missing_wrapper() {
        let result = Base85Ascii85.decode("HelloWorld");
        assert!(matches!(result, Err(BinaryCodecError::InvalidWrapper { .. })));
    }

    #[test]
    fn z85_invalid_length() {
        // Z85 requires encoded length to be multiple of 5
        let result = Base85Z85.decode("Hell");
        assert!(matches!(result, Err(BinaryCodecError::InvalidLength { .. })));
    }

    #[test]
    fn ascii85_invalid_character() {
        // 'v' is beyond the Ascii85 alphabet (! to u)
        let result = Base85Ascii85.decode("<~vvvvv~>");
        assert!(matches!(result, Err(BinaryCodecError::InvalidCharacter { .. })));
    }

    #[test]
    fn ascii85_overflow() {
        // `s8W-"` would overflow u32
        let result = Base85Ascii85.decode("<~s8W-\"~>");
        assert!(matches!(result, Err(BinaryCodecError::Base85Overflow { .. })));
    }

    #[test]
    fn btoa_missing_wrapper() {
        let result = Base85Btoa.decode("HelloWorld");
        assert!(matches!(result, Err(BinaryCodecError::InvalidWrapper { .. })));
    }

    #[test]
    fn rfc1924_u128_invalid_length() {
        let result = Base85Rfc1924::decode_u128("short");
        assert!(matches!(result, Err(BinaryCodecError::InvalidLength { .. })));
    }
}

// ============================================================================
// Variant Differences Tests
// ============================================================================

mod variant_differences {
    use super::*;

    #[test]
    fn ascii85_has_wrapper() {
        let data = b"Test";
        let encoded = Base85Ascii85.encode(data);
        assert!(encoded.starts_with("<~"));
        assert!(encoded.ends_with("~>"));
    }

    #[test]
    fn z85_has_no_wrapper() {
        let data = b"Test";
        let encoded = Base85Z85.encode(data);
        assert!(!encoded.starts_with("<~"));
        assert!(!encoded.contains("xbtoa"));
    }

    #[test]
    fn btoa_has_xbtoa_wrapper() {
        let data = b"Test";
        let encoded = Base85Btoa.encode(data);
        assert!(encoded.starts_with("xbtoa Begin"));
        assert!(encoded.contains("xbtoa End"));
    }

    #[test]
    fn ascii85_zero_compression() {
        let zeros = [0u8; 4];
        let encoded = Base85Ascii85.encode(&zeros);
        assert_eq!(encoded, "<~z~>");
    }

    #[test]
    fn btoa_zero_compression() {
        let zeros = [0u8; 4];
        let encoded = Base85Btoa.encode(&zeros);
        assert!(encoded.contains("z"));
    }

    #[test]
    fn z85_no_zero_compression() {
        let zeros = [0u8; 4];
        let encoded = Base85Z85.encode(&zeros);
        // Z85 doesn't use 'z' compression - encodes to "00000"
        assert!(!encoded.contains('z'));
    }

    #[test]
    fn different_alphabets() {
        let data = b"Test";

        // All produce different encodings
        let ascii85 = Base85Ascii85.encode(data);
        let z85 = Base85Z85.encode(data);
        let rfc1924 = Base85Rfc1924.encode(data);

        // Strip wrapper from ascii85 for comparison
        let ascii85_inner = ascii85.strip_prefix("<~").unwrap().strip_suffix("~>").unwrap();

        assert_ne!(ascii85_inner, z85);
        assert_ne!(z85, rfc1924);
    }
}

// ============================================================================
// Large Data Tests
// ============================================================================

mod large_data {
    use super::*;

    #[test]
    fn ascii85_large_data() {
        let data = vec![0xABu8; 1024];
        let encoded = Base85Ascii85.encode(&data);
        let decoded = Base85Ascii85.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn z85_large_data() {
        // Z85 requires multiple of 4
        let data = vec![0xABu8; 1024];
        let encoded = Base85Z85.encode(&data);
        let decoded = Base85Z85.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn rfc1924_large_data() {
        let data = vec![0xABu8; 1024];
        let encoded = Base85Rfc1924.encode(&data);
        let decoded = Base85Rfc1924.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn btoa_large_data() {
        let data = vec![0xABu8; 1024];
        let encoded = Base85Btoa.encode(&data);
        let decoded = Base85Btoa.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn empty_input() {
        // Ascii85
        assert_eq!(Base85Ascii85.encode(b""), "<~~>");
        assert_eq!(Base85Ascii85.decode("<~~>").unwrap(), vec![]);

        // Z85
        assert_eq!(Base85Z85.encode(b""), "");
        assert_eq!(Base85Z85.decode("").unwrap(), vec![]);

        // RFC 1924
        assert_eq!(Base85Rfc1924.encode(b""), "");
        assert_eq!(Base85Rfc1924.decode("").unwrap(), vec![]);
    }

    #[test]
    fn partial_blocks_ascii85() {
        for len in 1..=3 {
            let data: Vec<u8> = (0..len).map(|i| (65 + i) as u8).collect();
            let encoded = Base85Ascii85.encode(&data);
            let decoded = Base85Ascii85.decode(&encoded).unwrap();
            assert_eq!(decoded, data);
        }
    }

    #[test]
    fn partial_blocks_rfc1924() {
        for len in 1..=3 {
            let data: Vec<u8> = (0..len).map(|i| (65 + i) as u8).collect();
            let encoded = Base85Rfc1924.encode(&data);
            let decoded = Base85Rfc1924.decode(&encoded).unwrap();
            assert_eq!(decoded, data);
        }
    }

    #[test]
    fn rfc1924_u128_boundaries() {
        // Zero
        let encoded = Base85Rfc1924::encode_u128(0);
        assert_eq!(encoded.len(), 20);
        let decoded = Base85Rfc1924::decode_u128(&encoded).unwrap();
        assert_eq!(decoded, 0);

        // Max u128
        let max = u128::MAX;
        let encoded = Base85Rfc1924::encode_u128(max);
        assert_eq!(encoded.len(), 20);
        let decoded = Base85Rfc1924::decode_u128(&encoded).unwrap();
        assert_eq!(decoded, max);
    }
}
