//! Property-based tests for codec
//!
//! These tests verify codec properties with randomly generated inputs,
//! complementing the specific test cases in unit and integration tests.

use codec::{
    Base32, Base32CrockfordLower, Base32CrockfordUpper, Base32Hex, Base32Standard, Base58,
    Base58Bitcoin, Base58Flickr, Base64, Base64Standard, Base64UrlSafe, Base85, Base85Ascii85,
    Base85Btoa, Base85Rfc1924, Base85Z85, BinaryCodec, Hex, HexLower, HexUpper, QuotedPrintable,
    QuotedPrintableRobust, QuotedPrintableStrict, TextCodec, UrlEncoding, UrlEncodingComponent,
    UrlEncodingForm, UrlEncodingRfc3986, Utf8,
};
use proptest::prelude::*;

// ========================================
// Hex Property Tests
// ========================================

proptest! {
    /// Property: Encoding and then decoding any byte sequence returns the original data
    #[test]
    fn prop_hex_roundtrip_lower(data: Vec<u8>) {
        let encoded = HexLower.encode(&data);
        let decoded = HexLower.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_hex_roundtrip_upper(data: Vec<u8>) {
        let encoded = HexUpper.encode(&data);
        let decoded = HexUpper.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_hex_roundtrip_case_insensitive(data: Vec<u8>) {
        let encoded = Hex::encode(&data);
        let decoded = Hex::decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoded hex string length is always 2x the input byte length
    #[test]
    fn prop_hex_encoded_length(data: Vec<u8>) {
        let encoded = Hex::encode_lower(&data);
        prop_assert_eq!(encoded.len(), data.len() * 2);
    }

    /// Property: Encoded hex string contains only valid hex characters
    #[test]
    fn prop_hex_lowercase_valid_chars(data: Vec<u8>) {
        let encoded = Hex::encode_lower(&data);
        prop_assert!(encoded.chars().all(|c| c.is_ascii_hexdigit() && c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn prop_hex_uppercase_valid_chars(data: Vec<u8>) {
        let encoded = Hex::encode_upper(&data);
        prop_assert!(encoded.chars().all(|c| c.is_ascii_hexdigit() && (c.is_ascii_uppercase() || c.is_ascii_digit())));
    }

    /// Property: Decoding odd-length strings always fails
    #[test]
    fn prop_hex_odd_length_fails(s in "[0-9a-fA-F]{1,100}") {
        // Ensure odd length
        let odd_string = if s.len() % 2 == 0 {
            s.chars().take(s.len() - 1).collect::<String>()
        } else {
            s
        };

        if odd_string.len() % 2 == 1 {
            prop_assert!(Hex::decode(&odd_string).is_err());
        }
    }

    /// Property: Decoding strings with invalid characters fails
    #[test]
    fn prop_hex_invalid_chars_fail(s in "[G-Zg-z]{2,20}") {
        // G-Z and g-z are not valid hex characters
        prop_assert!(Hex::decode(&s).is_err());
    }

    /// Property: Lower codec rejects uppercase, upper codec rejects lowercase
    #[test]
    fn prop_hex_case_strictness(data: Vec<u8>) {
        if !data.is_empty() {
            let lower = Hex::encode_lower(&data);
            let upper = Hex::encode_upper(&data);

            // If there are any letter characters (not just digits)
            if lower.chars().any(|c| c.is_ascii_alphabetic()) {
                prop_assert!(HexLower.decode(&upper).is_err());
                prop_assert!(HexUpper.decode(&lower).is_err());
            }
        }
    }
}

// ========================================
// Base64 Property Tests
// ========================================

proptest! {
    /// Property: Encoding and then decoding returns the original data
    #[test]
    fn prop_base64_roundtrip_standard(data: Vec<u8>) {
        let encoded = Base64Standard.encode(&data);
        let decoded = Base64Standard.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_base64_roundtrip_url_safe(data: Vec<u8>) {
        let encoded = Base64UrlSafe.encode(&data);
        let decoded = Base64UrlSafe.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoded base64 length is roughly 4/3 of the input length
    #[test]
    fn prop_base64_encoded_length(data: Vec<u8>) {
        let encoded = Base64::encode_standard(&data);
        // Base64 encodes 3 bytes into 4 characters, with padding
        let expected_min = (data.len() * 4) / 3;
        let expected_max = ((data.len() + 2) * 4) / 3 + 4; // +4 for padding
        prop_assert!(encoded.len() >= expected_min && encoded.len() <= expected_max);
    }

    /// Property: Standard Base64 contains only valid characters
    #[test]
    fn prop_base64_standard_valid_chars(data: Vec<u8>) {
        let encoded = Base64::encode_standard(&data);
        let all_valid = encoded.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
        });
        prop_assert!(all_valid);
    }

    /// Property: URL-safe Base64 contains only URL-safe characters
    #[test]
    fn prop_base64_url_safe_valid_chars(data: Vec<u8>) {
        let encoded = Base64::encode_url_safe(&data);
        let all_valid = encoded.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '='
        });
        prop_assert!(all_valid);
    }

    /// Property: Standard and URL-safe differ only in specific characters
    #[test]
    fn prop_base64_standard_vs_url_safe(data: Vec<u8>) {
        let standard = Base64::encode_standard(&data);
        let url_safe = Base64::encode_url_safe(&data);

        // Same length
        prop_assert_eq!(standard.len(), url_safe.len());

        // Only differ in +/- and //_
        for (s, u) in standard.chars().zip(url_safe.chars()) {
            if s != u {
                prop_assert!((s == '+' && u == '-') || (s == '/' && u == '_'));
            }
        }
    }

    /// Property: Padding always appears at the end (if present)
    #[test]
    fn prop_base64_padding_at_end(data: Vec<u8>) {
        let encoded = Base64::encode_standard(&data);
        if let Some(pos) = encoded.find('=') {
            // All characters after the first '=' must also be '='
            prop_assert!(encoded[pos..].chars().all(|c| c == '='));
        }
    }

    /// Property: At most 2 padding characters
    #[test]
    fn prop_base64_max_two_padding(data: Vec<u8>) {
        let encoded = Base64::encode_standard(&data);
        let padding_count = encoded.chars().filter(|&c| c == '=').count();
        prop_assert!(padding_count <= 2);
    }
}

// ========================================
// Base58 Property Tests
// ========================================

proptest! {
    /// Property: Encoding and then decoding returns the original data (Bitcoin)
    #[test]
    fn prop_base58_roundtrip_bitcoin(data: Vec<u8>) {
        let encoded = Base58Bitcoin.encode(&data);
        let decoded = Base58Bitcoin.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoding and then decoding returns the original data (Flickr)
    #[test]
    fn prop_base58_roundtrip_flickr(data: Vec<u8>) {
        let encoded = Base58Flickr.encode(&data);
        let decoded = Base58Flickr.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Bitcoin Base58 contains only valid characters (excludes 0, O, I, l)
    #[test]
    fn prop_base58_bitcoin_valid_chars(data: Vec<u8>) {
        let encoded = Base58::encode_bitcoin(&data);
        let bitcoin_alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        let all_valid = encoded.chars().all(|c| bitcoin_alphabet.contains(c));
        prop_assert!(all_valid, "Invalid character in encoded Base58: {}", encoded);
    }

    /// Property: Flickr Base58 contains only valid characters
    #[test]
    fn prop_base58_flickr_valid_chars(data: Vec<u8>) {
        let encoded = Base58::encode_flickr(&data);
        let flickr_alphabet = "123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ";
        let all_valid = encoded.chars().all(|c| flickr_alphabet.contains(c));
        prop_assert!(all_valid, "Invalid character in encoded Base58 Flickr: {}", encoded);
    }

    /// Property: Leading zero bytes are preserved
    #[test]
    fn prop_base58_leading_zeros(leading_zeros: u8, data: Vec<u8>) {
        let leading_zeros = (leading_zeros % 10) as usize; // Limit to 0-9 leading zeros
        let mut full_data = vec![0u8; leading_zeros];
        full_data.extend_from_slice(&data);

        // Count actual leading zeros in full_data (including any from data)
        let actual_leading_zeros = full_data.iter().take_while(|&&b| b == 0).count();

        let encoded = Base58::encode_bitcoin(&full_data);
        let decoded = Base58::decode_bitcoin(&encoded).unwrap();

        prop_assert_eq!(decoded, full_data, "Leading zeros not preserved");

        // Leading zeros should produce leading '1's in the Bitcoin alphabet
        if actual_leading_zeros > 0 {
            let leading_ones = encoded.chars().take_while(|&c| c == '1').count();
            prop_assert_eq!(leading_ones, actual_leading_zeros, "Leading '1's count mismatch");
        }
    }

    /// Property: Bitcoin and Flickr produce different encodings (for non-empty data)
    #[test]
    fn prop_base58_bitcoin_flickr_different(data in prop::collection::vec(any::<u8>(), 1..100)) {
        let bitcoin = Base58::encode_bitcoin(&data);
        let flickr = Base58::encode_flickr(&data);

        // Same length
        prop_assert_eq!(bitcoin.len(), flickr.len());

        // Different encodings (for non-trivial data with letters)
        if bitcoin.chars().any(|c| c.is_alphabetic()) {
            // Should differ because alphabets differ in case
            prop_assert_ne!(bitcoin, flickr);
        }
    }

    /// Property: Custom alphabet round-trip
    #[test]
    fn prop_base58_custom_alphabet_roundtrip(data: Vec<u8>) {
        // Use Ripple alphabet as a known valid custom alphabet
        let ripple = "rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";
        let codec = Base58::new(ripple).unwrap();

        let encoded = codec.encode(&data);
        let decoded = codec.decode(&encoded).unwrap();

        prop_assert_eq!(decoded, data);
    }
}

// Test empty input separately (no randomization needed)
#[test]
fn test_base58_empty_input() {
    let empty: Vec<u8> = vec![];

    // Bitcoin
    let encoded = Base58::encode_bitcoin(&empty);
    assert!(encoded.is_empty());
    assert_eq!(Base58::decode_bitcoin(&encoded).unwrap(), empty);

    // Flickr
    let encoded = Base58::encode_flickr(&empty);
    assert!(encoded.is_empty());
    assert_eq!(Base58::decode_flickr(&encoded).unwrap(), empty);
}

// ========================================
// Base85 Property Tests
// ========================================

proptest! {
    /// Property: Ascii85 encoding and decoding returns the original data
    #[test]
    fn prop_base85_roundtrip_ascii85(data: Vec<u8>) {
        let encoded = Base85Ascii85.encode(&data);
        let decoded = Base85Ascii85.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Z85 encoding and decoding returns the original data (multiple of 4)
    #[test]
    fn prop_base85_roundtrip_z85(data in prop::collection::vec(any::<u8>(), 0..100).prop_filter("multiple of 4", |v| v.len() % 4 == 0)) {
        let encoded = Base85Z85.encode(&data);
        let decoded = Base85Z85.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: RFC 1924 encoding and decoding returns the original data
    #[test]
    fn prop_base85_roundtrip_rfc1924(data: Vec<u8>) {
        let encoded = Base85Rfc1924.encode(&data);
        let decoded = Base85Rfc1924.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: btoa encoding and decoding returns the original data
    #[test]
    fn prop_base85_roundtrip_btoa(data: Vec<u8>) {
        let encoded = Base85Btoa.encode(&data);
        let decoded = Base85Btoa.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Ascii85 encoded output starts with <~ and ends with ~>
    #[test]
    fn prop_base85_ascii85_wrapper(data: Vec<u8>) {
        let encoded = Base85::encode_ascii85(&data);
        prop_assert!(encoded.starts_with("<~"));
        prop_assert!(encoded.ends_with("~>"));
    }

    /// Property: Ascii85 zero compression
    #[test]
    fn prop_base85_ascii85_zero_compression(count in 1usize..10) {
        // Multiple of 4 zeros should encode to 'z' characters
        let zeros = vec![0u8; count * 4];
        let encoded = Base85Ascii85.encode(&zeros);
        let inner = encoded.strip_prefix("<~").unwrap().strip_suffix("~>").unwrap();
        prop_assert_eq!(inner.len(), count);
        prop_assert!(inner.chars().all(|c| c == 'z'));
    }

    /// Property: Z85 encoded length is 5/4 of the input length (for multiple of 4 inputs)
    #[test]
    fn prop_base85_z85_encoded_length(data in prop::collection::vec(any::<u8>(), 0..100).prop_filter("multiple of 4", |v| v.len() % 4 == 0)) {
        let encoded = Base85Z85.encode(&data);
        prop_assert_eq!(encoded.len(), data.len() * 5 / 4);
    }

    /// Property: Ascii85 alphabet contains only valid characters
    #[test]
    fn prop_base85_ascii85_valid_chars(data: Vec<u8>) {
        let encoded = Base85Ascii85.encode(&data);
        let inner = encoded.strip_prefix("<~").unwrap().strip_suffix("~>").unwrap();
        // Ascii85 uses '!' (33) to 'u' (117), plus 'z' for zero compression
        let all_valid = inner.chars().all(|c| {
            ('!'..='u').contains(&c) || c == 'z'
        });
        prop_assert!(all_valid, "Invalid character in Ascii85 encoded output: {}", inner);
    }

    /// Property: Z85 alphabet contains only valid characters
    #[test]
    fn prop_base85_z85_valid_chars(data in prop::collection::vec(any::<u8>(), 0..100).prop_filter("multiple of 4", |v| v.len() % 4 == 0)) {
        let encoded = Base85Z85.encode(&data);
        let z85_alphabet = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
        let all_valid = encoded.chars().all(|c| z85_alphabet.contains(c));
        prop_assert!(all_valid, "Invalid character in Z85 encoded output: {}", encoded);
    }

    /// Property: RFC 1924 u128 round-trip
    #[test]
    fn prop_base85_rfc1924_u128_roundtrip(value: u128) {
        let encoded = Base85Rfc1924::encode_u128(value);
        prop_assert_eq!(encoded.len(), 20);
        let decoded = Base85Rfc1924::decode_u128(&encoded).unwrap();
        prop_assert_eq!(decoded, value);
    }

    /// Property: RFC 1924 alphabet contains only valid characters
    #[test]
    fn prop_base85_rfc1924_valid_chars(data: Vec<u8>) {
        let encoded = Base85Rfc1924.encode(&data);
        let rfc1924_alphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";
        let all_valid = encoded.chars().all(|c| rfc1924_alphabet.contains(c));
        prop_assert!(all_valid, "Invalid character in RFC 1924 encoded output: {}", encoded);
    }

    /// Property: btoa output contains wrapper
    #[test]
    fn prop_base85_btoa_wrapper(data: Vec<u8>) {
        let encoded = Base85::encode_btoa(&data);
        prop_assert!(encoded.contains("xbtoa Begin"));
        prop_assert!(encoded.contains("xbtoa End"));
    }

    /// Property: Ascii85 differs from Z85 (different alphabet structure)
    ///
    /// Note: Z85 と RFC1924 は 0-9 が同じ位置にあるため、
    /// 特定の入力では同じ出力になる可能性がある。
    /// Ascii85 は '!' から始まる連続 ASCII を使うため、常に異なる。
    #[test]
    fn prop_base85_ascii85_differs_from_z85(data in prop::collection::vec(any::<u8>(), 4..20).prop_filter("multiple of 4 and has non-zero", |v| v.len() % 4 == 0 && v.iter().any(|&b| b != 0))) {
        let ascii85 = Base85Ascii85.encode(&data);
        let z85 = Base85Z85.encode(&data);

        // Strip wrapper from ascii85
        let ascii85_inner = ascii85.strip_prefix("<~").unwrap().strip_suffix("~>").unwrap();

        // Ascii85 uses sequential ASCII starting from '!', so the output always differs from Z85
        prop_assert_ne!(ascii85_inner, &z85);
    }

    /// Property: Z85 with padding API round-trip
    #[test]
    fn prop_base85_z85_padding_roundtrip(data: Vec<u8>) {
        let (encoded, padding) = Base85Z85.encode_with_padding(&data);
        let decoded = Base85Z85.decode_with_padding(&encoded, padding).unwrap();
        prop_assert_eq!(decoded, data);
    }
}

// Test empty input separately (no randomization needed)
#[test]
fn test_base85_empty_input() {
    let empty: Vec<u8> = vec![];

    // Ascii85
    let encoded = Base85::encode_ascii85(&empty);
    assert_eq!(encoded, "<~~>");
    assert_eq!(Base85::decode_ascii85(&encoded).unwrap(), empty);

    // Z85
    let encoded = Base85::encode_z85(&empty);
    assert!(encoded.is_empty());
    assert_eq!(Base85::decode_z85(&encoded).unwrap(), empty);

    // RFC 1924
    let encoded = Base85::encode_rfc1924(&empty);
    assert!(encoded.is_empty());
    assert_eq!(Base85::decode_rfc1924(&encoded).unwrap(), empty);
}

// ========================================
// Base32 Property Tests
// ========================================

proptest! {
    /// Property: Encoding and then decoding returns the original data (Standard)
    #[test]
    fn prop_base32_roundtrip_standard(data: Vec<u8>) {
        let encoded = Base32Standard.encode(&data);
        let decoded = Base32Standard.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoding and then decoding returns the original data (Hex)
    #[test]
    fn prop_base32_roundtrip_hex(data: Vec<u8>) {
        let encoded = Base32Hex.encode(&data);
        let decoded = Base32Hex.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoding and then decoding returns the original data (Crockford Upper)
    #[test]
    fn prop_base32_roundtrip_crockford_upper(data: Vec<u8>) {
        let encoded = Base32CrockfordUpper.encode(&data);
        let decoded = Base32CrockfordUpper.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoding and then decoding returns the original data (Crockford Lower)
    #[test]
    fn prop_base32_roundtrip_crockford_lower(data: Vec<u8>) {
        let encoded = Base32CrockfordLower.encode(&data);
        let decoded = Base32CrockfordLower.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoded base32 length is a multiple of 8 (with padding)
    #[test]
    fn prop_base32_standard_length_multiple_of_8(data: Vec<u8>) {
        let encoded = Base32::encode_standard(&data);
        if !data.is_empty() {
            prop_assert_eq!(encoded.len() % 8, 0);
        }
    }

    /// Property: Encoded base32 hex length is a multiple of 8 (with padding)
    #[test]
    fn prop_base32_hex_length_multiple_of_8(data: Vec<u8>) {
        let encoded = Base32::encode_hex(&data);
        if !data.is_empty() {
            prop_assert_eq!(encoded.len() % 8, 0);
        }
    }

    /// Property: Standard Base32 contains only valid characters (A-Z2-7 and =)
    #[test]
    fn prop_base32_standard_valid_chars(data: Vec<u8>) {
        let encoded = Base32::encode_standard(&data);
        let all_valid = encoded.chars().all(|c| {
            matches!(c, 'A'..='Z' | '2'..='7' | '=')
        });
        prop_assert!(all_valid);
    }

    /// Property: Hex Base32 contains only valid characters (0-9A-V and =)
    #[test]
    fn prop_base32_hex_valid_chars(data: Vec<u8>) {
        let encoded = Base32::encode_hex(&data);
        let all_valid = encoded.chars().all(|c| {
            matches!(c, '0'..='9' | 'A'..='V' | '=')
        });
        prop_assert!(all_valid);
    }

    /// Property: Crockford Base32 contains only valid characters (0-9A-HJKMNP-TV-Z, no padding)
    #[test]
    fn prop_base32_crockford_upper_valid_chars(data: Vec<u8>) {
        let encoded = Base32::encode_crockford_upper(&data);
        let all_valid = encoded.chars().all(|c| {
            matches!(c, '0'..='9' | 'A'..='H' | 'J'..='K' | 'M'..='N' | 'P'..='T' | 'V'..='Z')
        });
        prop_assert!(all_valid);
    }

    /// Property: Crockford Base32 lowercase contains only valid characters
    #[test]
    fn prop_base32_crockford_lower_valid_chars(data: Vec<u8>) {
        let encoded = Base32::encode_crockford_lower(&data);
        let all_valid = encoded.chars().all(|c| {
            matches!(c, '0'..='9' | 'a'..='h' | 'j'..='k' | 'm'..='n' | 'p'..='t' | 'v'..='z')
        });
        prop_assert!(all_valid);
    }

    /// Property: Padding always appears at the end (if present)
    #[test]
    fn prop_base32_standard_padding_at_end(data: Vec<u8>) {
        let encoded = Base32::encode_standard(&data);
        if let Some(pos) = encoded.find('=') {
            // All characters after the first '=' must also be '='
            prop_assert!(encoded[pos..].chars().all(|c| c == '='));
        }
    }

    /// Property: At most 6 padding characters for Standard
    #[test]
    fn prop_base32_standard_max_padding(data: Vec<u8>) {
        let encoded = Base32::encode_standard(&data);
        let padding_count = encoded.chars().filter(|&c| c == '=').count();
        prop_assert!(padding_count <= 6);
    }

    /// Property: At most 6 padding characters for Hex
    #[test]
    fn prop_base32_hex_max_padding(data: Vec<u8>) {
        let encoded = Base32::encode_hex(&data);
        let padding_count = encoded.chars().filter(|&c| c == '=').count();
        prop_assert!(padding_count <= 6);
    }

    /// Property: Crockford Base32 has no padding
    #[test]
    fn prop_base32_crockford_no_padding(data: Vec<u8>) {
        let encoded_upper = Base32::encode_crockford_upper(&data);
        let encoded_lower = Base32::encode_crockford_lower(&data);
        prop_assert!(!encoded_upper.contains('='));
        prop_assert!(!encoded_lower.contains('='));
    }

    /// Property: Crockford decoding is case-insensitive
    #[test]
    fn prop_base32_crockford_case_insensitive(data: Vec<u8>) {
        let encoded_upper = Base32CrockfordUpper.encode(&data);
        let encoded_lower = Base32CrockfordLower.encode(&data);

        // Both uppercase and lowercase should decode successfully
        let decoded_from_upper = Base32CrockfordUpper.decode(&encoded_upper).unwrap();
        let decoded_from_lower = Base32CrockfordUpper.decode(&encoded_lower).unwrap();

        prop_assert_eq!(decoded_from_upper.as_slice(), data.as_slice());
        prop_assert_eq!(decoded_from_lower.as_slice(), data.as_slice());
    }

    /// Property: Crockford confusion-resistant character mapping (I→1)
    #[test]
    fn prop_base32_crockford_confusion_i_to_1(data: Vec<u8>) {
        let encoded = Base32CrockfordUpper.encode(&data);
        // Replace '1' with 'I' (confusion-resistant)
        if encoded.contains('1') {
            let with_i = encoded.replace('1', "I");
            let decoded = Base32CrockfordUpper.decode(&with_i).unwrap();
            prop_assert_eq!(decoded, data);
        }
    }

    /// Property: Crockford confusion-resistant character mapping (O→0)
    #[test]
    fn prop_base32_crockford_confusion_o_to_0(data: Vec<u8>) {
        let encoded = Base32CrockfordUpper.encode(&data);
        // Replace '0' with 'O' (confusion-resistant)
        if encoded.contains('0') {
            let with_o = encoded.replace('0', "O");
            let decoded = Base32CrockfordUpper.decode(&with_o).unwrap();
            prop_assert_eq!(decoded, data);
        }
    }

    /// Property: Crockford confusion-resistant character mapping (L→1)
    #[test]
    fn prop_base32_crockford_confusion_l_to_1(data: Vec<u8>) {
        let encoded = Base32CrockfordLower.encode(&data);
        // Replace '1' with 'l' (confusion-resistant)
        if encoded.contains('1') {
            let with_l = encoded.replace('1', "l");
            let decoded = Base32CrockfordLower.decode(&with_l).unwrap();
            prop_assert_eq!(decoded, data);
        }
    }
}

// ========================================
// UTF-8 Property Tests
// ========================================

proptest! {
    /// Property: Valid UTF-8 strings always encode and decode successfully
    #[test]
    fn prop_utf8_valid_roundtrip(text in "\\PC*") {
        // Rust strings are always valid UTF-8
        let bytes = Utf8.encode(&text);
        let decoded = Utf8.decode(&bytes);
        prop_assert_eq!(decoded, text);
    }

    /// Property: Strict mode succeeds for valid UTF-8
    #[test]
    fn prop_utf8_strict_valid(text in "\\PC*") {
        let bytes = Utf8.encode_strict(&text).unwrap();
        let decoded = Utf8.decode_strict(&bytes).unwrap();
        prop_assert_eq!(decoded, text);
    }

    /// Property: Lossy mode never fails
    #[test]
    fn prop_utf8_lossy_never_fails(data: Vec<u8>) {
        // decode() should never panic, even with invalid UTF-8
        let _result = Utf8.decode(&data);
        // If we get here without panicking, the test passes
    }

    /// Property: Encoding length equals byte length (UTF-8 is identity for Rust strings)
    #[test]
    fn prop_utf8_encode_length(text in "\\PC*") {
        let bytes = Utf8.encode(&text);
        prop_assert_eq!(bytes.len(), text.len());
    }
}

// ========================================
// Cross-Codec Property Tests
// ========================================

proptest! {
    /// Property: Double encoding and decoding (Hex of Base64)
    #[test]
    fn prop_double_encoding_hex_base64(data in prop::collection::vec(any::<u8>(), 0..100)) {
        // Base64 encode
        let base64_encoded = Base64::encode_standard(&data);

        // Hex encodes the Base64 string
        let hex_encoded = Hex::encode_lower(base64_encoded.as_bytes());

        // Reverse: Hex decode
        let hex_decoded = Hex::decode_lower(&hex_encoded).unwrap();

        // Base64 decode
        let base64_decoded = Base64::decode_standard(&String::from_utf8(hex_decoded).unwrap()).unwrap();

        prop_assert_eq!(base64_decoded, data);
    }

}

// Test empty input separately (no randomization needed)
#[test]
fn test_empty_input_all_codecs() {
    let empty: Vec<u8> = vec![];

    // Hex
    let hex = Hex::encode(&empty);
    assert!(hex.is_empty());
    assert_eq!(Hex::decode(&hex).unwrap(), empty);

    // Base32
    let b32_std = Base32::encode_standard(&empty);
    assert!(b32_std.is_empty());
    assert_eq!(Base32::decode_standard(&b32_std).unwrap(), empty);

    let b32_hex = Base32::encode_hex(&empty);
    assert!(b32_hex.is_empty());
    assert_eq!(Base32::decode_hex(&b32_hex).unwrap(), empty);

    let b32_crockford = Base32::encode_crockford(&empty);
    assert!(b32_crockford.is_empty());
    assert_eq!(Base32::decode_crockford(&b32_crockford).unwrap(), empty);

    // Base64
    let b64 = Base64::encode_standard(&empty);
    assert!(b64.is_empty());
    assert_eq!(Base64::decode_standard(&b64).unwrap(), empty);

    // Base85
    let b85_ascii85 = Base85::encode_ascii85(&empty);
    assert_eq!(b85_ascii85, "<~~>");
    assert_eq!(Base85::decode_ascii85(&b85_ascii85).unwrap(), empty);

    let b85_z85 = Base85::encode_z85(&empty);
    assert!(b85_z85.is_empty());
    assert_eq!(Base85::decode_z85(&b85_z85).unwrap(), empty);

    let b85_rfc1924 = Base85::encode_rfc1924(&empty);
    assert!(b85_rfc1924.is_empty());
    assert_eq!(Base85::decode_rfc1924(&b85_rfc1924).unwrap(), empty);

    // UTF-8
    let utf8 = Utf8.encode("");
    assert!(utf8.is_empty());
    assert_eq!(Utf8.decode(&utf8), "");
}

// ========================================
// Large Data Property Tests
// ========================================

proptest! {
    /// Property: Codecs handle large data without panicking
    #[test]
    fn prop_large_data_hex(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = Hex::encode_lower(&data);
        let decoded = Hex::decode_lower(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_base32_standard(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = Base32::encode_standard(&data);
        let decoded = Base32::decode_standard(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_base32_hex(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = Base32::encode_hex(&data);
        let decoded = Base32::decode_hex(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_base32_crockford(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = Base32::encode_crockford(&data);
        let decoded = Base32::decode_crockford(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_base58_bitcoin(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = Base58::encode_bitcoin(&data);
        let decoded = Base58::decode_bitcoin(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_base58_flickr(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = Base58::encode_flickr(&data);
        let decoded = Base58::decode_flickr(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_base64(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = Base64::encode_standard(&data);
        let decoded = Base64::decode_standard(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_base85_ascii85(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = Base85::encode_ascii85(&data);
        let decoded = Base85::decode_ascii85(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_base85_z85(data in prop::collection::vec(any::<u8>(), 1024..2048).prop_filter("multiple of 4", |v| v.len() % 4 == 0)) {
        let encoded = Base85::encode_z85(&data);
        let decoded = Base85::decode_z85(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_base85_rfc1924(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = Base85::encode_rfc1924(&data);
        let decoded = Base85::decode_rfc1924(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_base85_btoa(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = Base85::encode_btoa(&data);
        let decoded = Base85::decode_btoa(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_url_rfc3986(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = UrlEncoding::encode_rfc3986(&data);
        let decoded = UrlEncoding::decode_rfc3986(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_url_form(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = UrlEncoding::encode_form(&data);
        let decoded = UrlEncoding::decode_form(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_url_component(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = UrlEncoding::encode_component(&data);
        let decoded = UrlEncoding::decode_component(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }
}

// ========================================
// URL Encoding Property Tests
// ========================================

proptest! {
    /// Property: Encoding and then decoding returns the original data (RFC 3986)
    #[test]
    fn prop_url_roundtrip_rfc3986(data: Vec<u8>) {
        let encoded = UrlEncodingRfc3986.encode(&data);
        let decoded = UrlEncodingRfc3986.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoding and then decoding returns the original data (Form)
    #[test]
    fn prop_url_roundtrip_form(data: Vec<u8>) {
        let encoded = UrlEncodingForm.encode(&data);
        let decoded = UrlEncodingForm.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoding and then decoding returns the original data (Component)
    #[test]
    fn prop_url_roundtrip_component(data: Vec<u8>) {
        let encoded = UrlEncodingComponent.encode(&data);
        let decoded = UrlEncodingComponent.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoded URL contains only valid characters
    #[test]
    fn prop_url_rfc3986_valid_chars(data: Vec<u8>) {
        let encoded = UrlEncoding::encode_rfc3986(&data);
        let all_valid = encoded.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~' | '%')
        });
        prop_assert!(all_valid, "Invalid character in RFC 3986 encoded URL: {}", encoded);
    }

    /// Property: Form encoding contains only valid characters (includes '+' for space)
    #[test]
    fn prop_url_form_valid_chars(data: Vec<u8>) {
        let encoded = UrlEncoding::encode_form(&data);
        let all_valid = encoded.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '*' | '%' | '+')
        });
        prop_assert!(all_valid, "Invalid character in Form encoded URL: {}", encoded);
    }

    /// Property: Component encoding contains only alphanumeric and percent-encoded characters
    #[test]
    fn prop_url_component_valid_chars(data: Vec<u8>) {
        let encoded = UrlEncoding::encode_component(&data);
        let all_valid = encoded.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '%'
        });
        prop_assert!(all_valid, "Invalid character in Component encoded URL: {}", encoded);
    }

    /// Property: Percent encoding always follows %XX pattern
    #[test]
    fn prop_url_percent_encoding_pattern(data: Vec<u8>) {
        let encoded = UrlEncoding::encode_rfc3986(&data);
        let mut chars = encoded.chars();
        while let Some(c) = chars.next() {
            if c == '%' {
                // The next two chars must be hex digits
                let hex1 = chars.next();
                let hex2 = chars.next();
                prop_assert!(hex1.is_some() && hex2.is_some(), "Incomplete percent encoding");
                prop_assert!(hex1.unwrap().is_ascii_hexdigit(), "Invalid hex digit after %");
                prop_assert!(hex2.unwrap().is_ascii_hexdigit(), "Invalid hex digit after %");
            }
        }
    }

    /// Property: Form encoding converts space to +
    #[test]
    fn prop_url_form_space_to_plus(data in prop::collection::vec(any::<u8>(), 0..100)) {
        let mut with_space = data;
        with_space.push(b' '); // Add a space

        let encoded = UrlEncoding::encode_form(&with_space);
        prop_assert!(encoded.contains('+'), "Space should be encoded as + in Form encoding");

        let decoded = UrlEncoding::decode_form(&encoded).unwrap();
        prop_assert_eq!(decoded, with_space);
    }

    /// Property: RFC 3986 encodes space as %20 (not +)
    #[test]
    fn prop_url_rfc3986_space_to_percent20(data in prop::collection::vec(any::<u8>(), 0..100)) {
        let mut with_space = data;
        with_space.push(b' '); // Add a space

        let encoded = UrlEncoding::encode_rfc3986(&with_space);
        prop_assert!(encoded.contains("%20"), "Space should be encoded as %20 in RFC 3986");
        prop_assert!(!encoded.contains('+'), "RFC 3986 should not use + for space");

        let decoded = UrlEncoding::decode_rfc3986(&encoded).unwrap();
        prop_assert_eq!(decoded, with_space);
    }

    /// Property: Deterministic encoding (same input → same output)
    #[test]
    fn prop_url_deterministic(data: Vec<u8>) {
        let encoded1 = UrlEncoding::encode_rfc3986(&data);
        let encoded2 = UrlEncoding::encode_rfc3986(&data);
        prop_assert_eq!(encoded1, encoded2);

        let encoded1 = UrlEncoding::encode_form(&data);
        let encoded2 = UrlEncoding::encode_form(&data);
        prop_assert_eq!(encoded1, encoded2);

        let encoded1 = UrlEncoding::encode_component(&data);
        let encoded2 = UrlEncoding::encode_component(&data);
        prop_assert_eq!(encoded1, encoded2);
    }

    /// Property: RFC 3986 preserves unreserved characters (A-Z, a-z, 0-9, -, ., _, ~)
    #[test]
    fn prop_url_rfc3986_preserves_unreserved(data in "[A-Za-z0-9._~-]{1,100}") {
        // Data contains only unreserved characters
        let encoded = UrlEncoding::encode_rfc3986(data.as_bytes());
        prop_assert_eq!(encoded, data, "RFC 3986 should preserve unreserved characters");
    }

    /// Property: Form encoding preserves: A-Z, a-z, 0-9, *, -, ., _
    #[test]
    fn prop_url_form_preserves_chars(data in "[A-Za-z0-9*._-]{1,100}") {
        // Data contains only Form-preserved characters
        let encoded = UrlEncoding::encode_form(data.as_bytes());
        prop_assert_eq!(encoded, data, "Form encoding should preserve: A-Z, a-z, 0-9, *, -, ., _");
    }

    /// Property: Component encoding only preserves alphanumeric
    #[test]
    fn prop_url_component_only_alphanumeric(data in "[A-Za-z0-9]{1,100}") {
        // Data contains only alphanumeric
        let encoded = UrlEncoding::encode_component(data.as_bytes());
        prop_assert_eq!(encoded, data, "Component encoding should preserve alphanumeric");
    }

    /// Property: Lenient decoding never fails (always returns Ok)
    #[test]
    fn prop_url_lenient_decode_never_fails(s in "[\\PC]{0,100}") {
        // URL encoding uses lenient decoder that never fails
        let result_rfc = UrlEncoding::decode_rfc3986(&s);
        prop_assert!(result_rfc.is_ok(), "RFC 3986 lenient decode should never fail");

        let result_form = UrlEncoding::decode_form(&s);
        prop_assert!(result_form.is_ok(), "Form lenient decode should never fail");

        let result_comp = UrlEncoding::decode_component(&s);
        prop_assert!(result_comp.is_ok(), "Component lenient decode should never fail");
    }
}

// ========================================
// Quoted-Printable Property Tests
// ========================================

proptest! {
    /// Property: Encoding and then decoding returns the original data (Robust)
    #[test]
    fn prop_quoted_printable_roundtrip_robust(data: Vec<u8>) {
        let encoded = QuotedPrintableRobust.encode(&data);
        let decoded = QuotedPrintableRobust.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoding and then decoding returns the original data (Strict)
    #[test]
    fn prop_quoted_printable_roundtrip_strict(data: Vec<u8>) {
        let encoded = QuotedPrintableStrict.encode(&data);
        let decoded = QuotedPrintableStrict.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoding and then decoding returns the original data (Enum default)
    #[test]
    fn prop_quoted_printable_roundtrip_enum(data: Vec<u8>) {
        let encoded = QuotedPrintable::encode(&data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Encoded output contains only valid characters
    #[test]
    fn prop_quoted_printable_valid_chars(data: Vec<u8>) {
        let encoded = QuotedPrintable::encode(&data);

        for c in encoded.chars() {
            // Valid characters: printable ASCII (33-126) or whitespace
            // Tabs (0x09), spaces (0x20), CR (0x0D), LF (0x0A) are allowed
            // '!' ..= '~' covers 33-126, which includes '='
            let is_valid = matches!(c, '\t' | '\n' | '\r' | ' ' | '!' ..= '~');
            prop_assert!(is_valid, "Invalid character in encoded output: {:?}", c);
        }
    }

    /// Property: All equals signs are followed by valid hex or newline (in valid encoding)
    #[test]
    fn prop_quoted_printable_equals_followed_by_valid(data: Vec<u8>) {
        let encoded = QuotedPrintable::encode(&data);
        let chars: Vec<char> = encoded.chars().collect();

        for i in 0..chars.len() {
            if chars[i] == '=' {
                // '=' must be followed by:
                // 1. Two hex digits (=XX)
                // 2. Newline (\n or \r\n) for soft line break
                // 3. Nothing (end of string) for soft line break
                if i + 1 < chars.len() {
                    let next = chars[i + 1];
                    if next == '\n' || next == '\r' {
                        // Soft line break
                        continue;
                    }
                    // Must be hex digit
                    prop_assert!(next.is_ascii_hexdigit(),
                        "Character after '=' must be hex digit or newline, got: {:?}", next);

                    if i + 2 < chars.len() {
                        let next2 = chars[i + 2];
                        prop_assert!(next2.is_ascii_hexdigit(),
                            "Second character after '=' must be hex digit, got: {:?}", next2);
                    }
                }
            }
        }
    }

    /// Property: Robust mode never fails on decoding
    #[test]
    fn prop_quoted_printable_robust_never_fails(data: Vec<u8>) {
        let encoded = QuotedPrintable::encode(&data);
        let result = QuotedPrintableRobust.decode(&encoded);
        prop_assert!(result.is_ok(), "Robust decode should never fail on valid encoding");
    }

    /// Property: Strict mode succeeds on valid encoding
    #[test]
    fn prop_quoted_printable_strict_valid_succeeds(data: Vec<u8>) {
        let encoded = QuotedPrintable::encode(&data);
        let result = QuotedPrintableStrict.decode(&encoded);
        prop_assert!(result.is_ok(), "Strict decode should succeed on valid encoding");
    }

    /// Property: Robust and Strict produce same encoding
    #[test]
    fn prop_quoted_printable_robust_strict_same_encoding(data: Vec<u8>) {
        let encoded_robust = QuotedPrintableRobust.encode(&data);
        let encoded_strict = QuotedPrintableStrict.encode(&data);
        prop_assert_eq!(encoded_robust, encoded_strict,
            "Robust and Strict should produce identical encodings");
    }

    /// Property: Deterministic encoding (same input → same output)
    #[test]
    fn prop_quoted_printable_deterministic(data: Vec<u8>) {
        let encoded1 = QuotedPrintable::encode(&data);
        let encoded2 = QuotedPrintable::encode(&data);
        prop_assert_eq!(encoded1, encoded2, "Encoding should be deterministic");
    }

    /// Property: Printable ASCII characters may be preserved
    #[test]
    fn prop_quoted_printable_preserves_some_ascii(data in prop::collection::vec(33u8..=126, 1..100)) {
        // Printable ASCII (33-126)
        let encoded = QuotedPrintable::encode(&data);
        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data, "Printable ASCII should round-trip correctly");
    }

    /// Property: Non-printable bytes are encoded
    #[test]
    fn prop_quoted_printable_encodes_non_printable(data in prop::collection::vec(0u8..=31, 1..50)) {
        // Control characters (0-31)
        let encoded = QuotedPrintable::encode(&data);

        // Encoded output should contain '=' for encoding
        prop_assert!(encoded.contains('='),
            "Non-printable characters should be encoded with '='");

        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data, "Non-printable bytes should round-trip correctly");
    }

    /// Property: High bytes (128-255) are encoded
    #[test]
    fn prop_quoted_printable_encodes_high_bytes(data in prop::collection::vec(128u8..=255, 1..50)) {
        let encoded = QuotedPrintable::encode(&data);

        // Encoded output should contain '=' for encoding
        prop_assert!(encoded.contains('='),
            "High bytes should be encoded with '='");

        let decoded = QuotedPrintable::decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data, "High bytes should round-trip correctly");
    }

    /// Property: Robust mode handles malformed input gracefully
    #[test]
    fn prop_quoted_printable_robust_handles_malformed(s in "[=A-F0-9]{2,20}") {
        // Random string with hex-like characters
        // Robust mode should never fail
        let result = QuotedPrintableRobust.decode(&s);
        prop_assert!(result.is_ok(), "Robust mode should handle any input");
    }

    /// Property: Case-insensitive hex decoding for hex-encoded bytes
    #[test]
    fn prop_quoted_printable_case_insensitive(byte in 0u8..32) {
        // Only test control characters that are always hex-encoded
        let data = vec![byte];
        let encoded = QuotedPrintable::encode(&data);

        // Encoded control characters should contain '=XX'
        prop_assert!(encoded.contains('='), "Control character should be hex-encoded");

        // Convert hex digits to lowercase
        let encoded_lower = encoded
            .chars()
            .map(|c| if c.is_ascii_uppercase() && c != 'A' { c.to_ascii_lowercase() } else { c })
            .collect::<String>();

        // Robust mode should handle both cases
        let decoded_upper = QuotedPrintableRobust.decode(&encoded).unwrap();
        let decoded_lower = QuotedPrintableRobust.decode(&encoded_lower).unwrap();
        prop_assert_eq!(decoded_upper, decoded_lower,
            "Robust mode should handle case-insensitive hex");
    }
}

// ========================================
// Large Data Property Tests for Quoted-Printable
// ========================================

proptest! {
    /// Property: Codecs handle large data without panicking
    #[test]
    fn prop_large_data_quoted_printable_robust(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = QuotedPrintableRobust.encode(&data);
        let decoded = QuotedPrintableRobust.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn prop_large_data_quoted_printable_strict(data in prop::collection::vec(any::<u8>(), 1024..2048)) {
        let encoded = QuotedPrintableStrict.encode(&data);
        let decoded = QuotedPrintableStrict.decode(&encoded).unwrap();
        prop_assert_eq!(decoded, data);
    }
}

// Test empty input separately (no randomization needed)
#[test]
fn test_quoted_printable_empty_input() {
    let empty: Vec<u8> = vec![];

    // Robust
    let encoded = QuotedPrintable::encode(&empty);
    assert!(encoded.is_empty());
    assert_eq!(QuotedPrintable::decode(&encoded).unwrap(), empty);

    // Strict
    let encoded = QuotedPrintableStrict.encode(&empty);
    assert!(encoded.is_empty());
    assert_eq!(QuotedPrintableStrict.decode(&encoded).unwrap(), empty);
}
