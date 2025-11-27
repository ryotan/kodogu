//! Integration test harness for codec-specific tests

mod integration {
    mod base32;
    mod base58;
    mod base64;
    mod hex;
    mod quoted_printable;
    mod url_encoding;
    mod utf8;
}
