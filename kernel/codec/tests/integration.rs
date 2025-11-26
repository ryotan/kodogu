//! Integration test harness for codec-specific tests

mod integration {
    mod hex;
    mod base32;
    mod base58;
    mod base64;
    mod url_encoding;
    mod quoted_printable;
    mod utf8;
}
