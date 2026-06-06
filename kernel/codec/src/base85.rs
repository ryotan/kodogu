//! Base85 encoding and decoding.
//!
//! This module provides Base85 encoding/decoding following multiple specifications.
//!
//! ## Available Variants
//!
//! - **Ascii85 (Adobe)**: Used in PostScript/PDF, wraps output in `<~` and `~>`, supports 'z' compression
//! - **Z85 (ZeroMQ)**: Quote-safe encoding, requires input length to be multiple of 4
//! - **RFC 1924**: Proposed for IPv6 representation, supports 128-bit values
//! - **btoa (Unix)**: Original Base85 implementation with `xbtoa Begin/End` wrapper
//!
//! ## Choosing a Variant
//!
//! - **Z85**: When encoding data that needs to be embedded in strings (avoids quotes/backslash)
//! - **Ascii85**: When embedding binary data in PostScript/PDF or similar text-based formats
//! - **RFC 1924**: When encoding IPv6 addresses or 128-bit values
//! - **btoa**: When compatibility with Unix btoa utility is needed
//!
//! ## Examples
//!
//! ```
//! use codec::{Base85, BinaryCodec};
//!
//! // Using the default Z85 variant
//! let encoded = Base85::encode(b"Test");
//! let decoded = Base85::decode(&encoded).unwrap();
//! assert_eq!(decoded, b"Test");
//!
//! // Using specific variants
//! let ascii85 = Base85::encode_ascii85(b"Hello");
//! assert!(ascii85.starts_with("<~"));
//! ```
//!
//! ## References
//!
//! - [ASCII85 - Wikipedia](https://en.wikipedia.org/wiki/Ascii85)
//! - [ZeroMQ RFC 32 - Z85](https://rfc.zeromq.org/spec/32/)
//! - [RFC 1924 - A Compact Representation of IPv6 Addresses](https://tools.ietf.org/html/rfc1924)

use crate::{
    Base85OverflowError, BinaryCodec, BinaryCodecError, BinaryCodecs, InvalidCharacterError,
    InvalidLengthError, InvalidWrapperError,
};
use std::slice::ChunksExact;
use suzunari_error::{OptionExt, ensure};

// ============================================================================
// Zero-Sized Types (ZST)
// ============================================================================

/// Ascii85 (Adobe) variant - used in PostScript and PDF
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Wraps output in `<~` and `~>`, uses 'z' as shorthand for all-zero blocks.
/// Uses characters '!' (33) through 'u' (117).
///
/// # Example
/// ```
/// use codec::{Base85Ascii85, BinaryCodec};
///
/// let encoded = Base85Ascii85.encode(b"Hello");
/// assert!(encoded.starts_with("<~"));
/// assert!(encoded.ends_with("~>"));
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base85Ascii85;

/// Z85 (ZeroMQ) variant - quote-safe encoding
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Avoids quotes (`'`,`"`,`\`) for safe embedding in strings.
/// Input length should be a multiple of 4 (pads with zeros if not).
///
/// # Example
/// ```
/// use codec::{Base85Z85, BinaryCodec};
///
/// let input: [u8; 8] = [0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
/// let encoded = Base85Z85.encode(&input);
/// assert_eq!(encoded, "HelloWorld");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base85Z85;

/// RFC 1924 variant - proposed for IPv6 representation
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Designed for 128-bit values (IPv6 addresses).
/// URL-safer character set.
///
/// # Example
/// ```
/// use codec::{Base85Rfc1924, BinaryCodec};
///
/// let encoded = Base85Rfc1924.encode(b"test");
/// let decoded = Base85Rfc1924.decode(&encoded).unwrap();
/// assert_eq!(decoded, b"test");
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base85Rfc1924;

/// Btoa (Unix) variant - original Base85 implementation
///
/// This is a zero-sized type (ZST) with no runtime overhead.
/// Wraps output in `xbtoa Begin` and `xbtoa End` with checksum in the footer.
/// Uses the same character set as Ascii85.
///
/// # Example
/// ```
/// use codec::{Base85Btoa, BinaryCodec};
///
/// let encoded = Base85Btoa.encode(b"Hello");
/// assert!(encoded.contains("xbtoa Begin"));
/// assert!(encoded.contains("xbtoa End"));
/// ```
#[derive(Default, Copy, Debug, Clone, Eq, PartialEq)]
pub struct Base85Btoa;

// ============================================================================
// Enum Type
// ============================================================================

/// Base85 encoding variants
///
/// Enum wrapper provides convenient access to all Base85 variants.
/// The default variant is Z85 as it's the most commonly used modern variant.
///
/// # Example
/// ```
/// use codec::{Base85, BinaryCodec};
///
/// let codec = Base85::Z85;
/// let encoded = codec.encode(b"Test");
/// assert_eq!(codec.decode(&encoded).unwrap(), b"Test");
/// ```
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum Base85 {
    #[default]
    Z85,
    Ascii85,
    Rfc1924,
    Btoa,
}

// ============================================================================
// Marker Trait
// ============================================================================

/// Marker trait for Base85 codec types
///
/// Implemented by all Base85 variants to enable generic programming
/// over different Base85 encoding schemes while maintaining type safety.
///
/// # Example
/// ```
/// use codec::{Base85Codec, Base85Z85, Base85Ascii85, BinaryCodec};
///
/// fn encode_with_base85<C: Base85Codec>(codec: &C, data: &[u8]) -> String {
///     codec.encode(data)
/// }
///
/// let z85_result = encode_with_base85(&Base85Z85, b"Test");
/// let ascii85_result = encode_with_base85(&Base85Ascii85, b"Test");
/// ```
pub trait Base85Codec: BinaryCodec {}

// ============================================================================
// Alphabets
// ============================================================================

/// Ascii85 alphabet: '!' (33) to 'u' (117)
const ALPHABET_ASCII85: &[u8; 85] = &{
    let mut arr = [0u8; 85];
    let mut i = 0;
    while i < 85 {
        arr[i] = b'!' + i as u8;
        i += 1;
    }
    arr
};

/// Z85 alphabet: 0-9 A-Z a-z . - : + = ^ ! / * ? & < > ( ) [ ] { } @ % $ #
const ALPHABET_Z85: &[u8; 85] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/// RFC 1924 alphabet: 0-9 A-Z a-z ! # $ % & ( ) * + - ; < = > ? @ ^ _ ` { | } ~
const ALPHABET_RFC1924: &[u8; 85] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";

// ============================================================================
// Decode tables (lazy static)
// ============================================================================

/// 逆引きテーブルを生成する
const fn build_decode_table(alphabet: &[u8; 85]) -> [u8; 256] {
    let mut table = [255u8; 256];
    let mut i = 0;
    while i < 85 {
        table[alphabet[i] as usize] = i as u8;
        i += 1;
    }
    table
}

const DECODE_TABLE_ASCII85: [u8; 256] = build_decode_table(ALPHABET_ASCII85);
const DECODE_TABLE_Z85: [u8; 256] = build_decode_table(ALPHABET_Z85);
const DECODE_TABLE_RFC1924: [u8; 256] = build_decode_table(ALPHABET_RFC1924);

// ============================================================================
// Core encoding/decoding functions (shared logic)
// ============================================================================

/// Encode a 4-byte chunk to 5 Base85 characters
#[inline]
fn encode_chunk(chunk: u32, alphabet: &[u8; 85], out: &mut Vec<u8>) {
    let mut value = chunk;
    let mut buf = [0u8; 5];
    for i in (0..5).rev() {
        buf[i] = alphabet[(value % 85) as usize];
        value /= 85;
    }
    out.extend_from_slice(&buf);
}

/// Encode a partial chunk (1-3 bytes) to Base85
#[inline]
fn encode_partial(bytes: &[u8], alphabet: &[u8; 85], out: &mut Vec<u8>) {
    if bytes.is_empty() {
        return;
    }
    // Pad with zeros to make 4 bytes
    let mut padded = [0u8; 4];
    padded[..bytes.len()].copy_from_slice(bytes);
    let value = u32::from_be_bytes(padded);

    // Encode and take only necessary characters (len + 1)
    let mut buf = [0u8; 5];
    let mut v = value;
    for i in (0..5).rev() {
        buf[i] = alphabet[(v % 85) as usize];
        v /= 85;
    }
    out.extend_from_slice(&buf[..bytes.len() + 1]);
}

/// Decode 5 Base85 characters to 4 bytes
#[inline]
fn decode_chunk(chars: &[u8], table: &[u8; 256], pos: usize) -> Result<u32, BinaryCodecError> {
    let mut value: u64 = 0;
    for (i, &c) in chars.iter().enumerate() {
        let digit = table[c as usize];
        ensure!(
            digit != 255,
            InvalidCharacterError {
                encoding: BinaryCodecs::Base85,
                char: c as char,
                position: pos + i,
            }
        );
        value = value * 85 + digit as u64;
    }
    ensure!(value <= u32::MAX as u64, Base85OverflowError { position: pos });
    Ok(value as u32)
}

/// Decode partial block for Ascii85/btoa (pads with 'u' = 84)
fn decode_partial_ascii85(
    chars: &[u8],
    table: &[u8; 256],
    pos: usize,
) -> Result<Vec<u8>, BinaryCodecError> {
    ensure!(
        chars.len() >= 2,
        InvalidLengthError { encoding: BinaryCodecs::Base85, length: chars.len() }
    );

    let values = pad_with_highest_value(chars, table, pos)?;

    let mut value: u64 = 0;
    for &digit in &values {
        value = value * 85 + digit as u64;
    }

    let bytes = (value as u32).to_be_bytes();
    Ok(bytes[..chars.len() - 1].to_vec())
}

/// Decode partial block for RFC 1924 (pads with '~' = 84)
fn decode_partial_rfc1924(
    chars: &[u8],
    table: &[u8; 256],
    pos: usize,
) -> Result<Vec<u8>, BinaryCodecError> {
    ensure!(
        chars.len() >= 2,
        InvalidLengthError { encoding: BinaryCodecs::Base85, length: chars.len() }
    );

    let values = pad_with_highest_value(chars, table, pos)?;

    let mut value: u64 = 0;
    for &digit in &values {
        value = value * 85 + digit as u64;
    }

    let bytes = (value as u32).to_be_bytes();
    Ok(bytes[..chars.len() - 1].to_vec())
}

/// Pad with the highest value in the table
#[inline]
fn pad_with_highest_value(
    chars: &[u8],
    table: &[u8; 256],
    pos: usize,
) -> Result<[u8; 5], BinaryCodecError> {
    let mut values = [84u8; 5];
    for (i, &c) in chars.iter().enumerate() {
        let digit = table[c as usize];
        ensure!(
            digit != 255,
            InvalidCharacterError {
                encoding: BinaryCodecs::Base85,
                char: c as char,
                position: pos + i,
            }
        );
        values[i] = digit;
    }
    Ok(values)
}

/// Encode ASCII85-style chunks (handles 'z' compression for all-zero blocks)
#[inline]
fn encode_chunks_ascii85(out: &mut Vec<u8>, chunks: ChunksExact<u8>) {
    for chunk in chunks {
        let value = u32::from_be_bytes(chunk.try_into().unwrap());
        if value == 0 {
            out.push(b'z');
        } else {
            encode_chunk(value, ALPHABET_ASCII85, out);
        }
    }
}

/// Decode ASCII85-style chunks (handles 'z' compression and partial blocks)
#[inline]
fn decode_ascii85_chunks(chars: &[u8]) -> Result<Vec<u8>, BinaryCodecError> {
    let mut out = Vec::with_capacity(chars.len() * 4 / 5);
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == b'z' {
            out.extend_from_slice(&[0, 0, 0, 0]);
            i += 1;
        } else if i + 5 <= chars.len() {
            let value = decode_chunk(&chars[i..i + 5], &DECODE_TABLE_ASCII85, i)?;
            out.extend_from_slice(&value.to_be_bytes());
            i += 5;
        } else {
            // Partial block at the end
            let partial = &chars[i..];
            out.extend(decode_partial_ascii85(partial, &DECODE_TABLE_ASCII85, i)?);
            break;
        }
    }

    Ok(out)
}

// ============================================================================
// Ascii85 Implementation
// ============================================================================
impl Base85Codec for Base85Ascii85 {}
impl BinaryCodec for Base85Ascii85 {
    fn encode(&self, bytes: &[u8]) -> String {
        let mut out = Vec::with_capacity(bytes.len() * 5 / 4 + 4);
        out.extend_from_slice(b"<~");

        let chunks = bytes.chunks_exact(4);
        let remainder = chunks.remainder();

        encode_chunks_ascii85(&mut out, chunks);

        encode_partial(remainder, ALPHABET_ASCII85, &mut out);

        out.extend_from_slice(b"~>");
        // SAFETY: ASCII85 alphabet consists only of ASCII characters
        String::from_utf8(out).unwrap()
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        let input = text.trim();

        // Check and strip wrapper
        let inner = input
            .strip_prefix("<~")
            .and_then(|s| s.strip_suffix("~>"))
            .context(InvalidWrapperError)?;

        // Filter whitespace and collect characters
        let chars: Vec<u8> = inner.bytes().filter(|&c| !c.is_ascii_whitespace()).collect();

        decode_ascii85_chunks(&chars)
    }
}

// ============================================================================
// Z85 Implementation
// ============================================================================
impl Base85Codec for Base85Z85 {}
impl BinaryCodec for Base85Z85 {
    fn encode(&self, bytes: &[u8]) -> String {
        if !bytes.len().is_multiple_of(4) {
            // Z85 requires input length to be multiple of 4
            // Pad with zeros
            let mut padded = bytes.to_vec();
            while !padded.len().is_multiple_of(4) {
                padded.push(0);
            }
            return self.encode_exact(&padded);
        }
        self.encode_exact(bytes)
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        let chars: Vec<u8> = text.bytes().collect();

        ensure!(
            chars.len().is_multiple_of(5),
            InvalidLengthError { encoding: BinaryCodecs::Base85, length: chars.len() }
        );

        let mut out = Vec::with_capacity(chars.len() * 4 / 5);

        for (i, chunk) in chars.chunks(5).enumerate() {
            let value = decode_chunk(chunk, &DECODE_TABLE_Z85, i * 5)?;
            out.extend_from_slice(&value.to_be_bytes());
        }

        Ok(out)
    }
}

impl Base85Z85 {
    /// Encode exact (input length must be multiple of 4)
    fn encode_exact(&self, input: &[u8]) -> String {
        let mut out = Vec::with_capacity(input.len() * 5 / 4);

        for chunk in input.chunks_exact(4) {
            let value = u32::from_be_bytes(chunk.try_into().unwrap());
            encode_chunk(value, ALPHABET_Z85, &mut out);
        }

        // SAFETY: Z85 alphabet consists only of ASCII characters
        String::from_utf8(out).unwrap()
    }

    /// Encode with padding info returned
    pub fn encode_with_padding(&self, input: &[u8]) -> (String, usize) {
        let padding = (4 - (input.len() % 4)) % 4;
        let mut padded = input.to_vec();
        padded.resize(input.len() + padding, 0);
        (self.encode_exact(&padded), padding)
    }

    /// Decode and strip padding
    pub fn decode_with_padding(
        &self,
        input: &str,
        padding: usize,
    ) -> Result<Vec<u8>, BinaryCodecError> {
        let mut decoded = self.decode(input)?;
        decoded.truncate(decoded.len() - padding);
        Ok(decoded)
    }
}

// ============================================================================
// RFC 1924 Implementation
// ============================================================================
impl Base85Codec for Base85Rfc1924 {}
impl BinaryCodec for Base85Rfc1924 {
    fn encode(&self, bytes: &[u8]) -> String {
        let mut out = Vec::with_capacity(bytes.len() * 5 / 4 + 2);

        let chunks = bytes.chunks_exact(4);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let value = u32::from_be_bytes(chunk.try_into().unwrap());
            encode_chunk(value, ALPHABET_RFC1924, &mut out);
        }

        encode_partial(remainder, ALPHABET_RFC1924, &mut out);

        // SAFETY: RFC 1924 alphabet consists only of ASCII characters
        String::from_utf8(out).unwrap()
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        let chars: Vec<u8> = text.bytes().collect();
        let mut out = Vec::with_capacity(chars.len() * 4 / 5);

        let chunks = chars.chunks_exact(5);
        let remainder = chunks.remainder();

        for (i, chunk) in chunks.enumerate() {
            let value = decode_chunk(chunk, &DECODE_TABLE_RFC1924, i * 5)?;
            out.extend_from_slice(&value.to_be_bytes());
        }

        if !remainder.is_empty() {
            out.extend(decode_partial_rfc1924(
                remainder,
                &DECODE_TABLE_RFC1924,
                chars.len() - remainder.len(),
            )?);
        }

        Ok(out)
    }
}

impl Base85Rfc1924 {
    /// Encode a 128-bit value (IPv6 address) to exactly 20 characters
    ///
    /// RFC 1924 specifies that the entire 128-bit value is treated as a single
    /// large number and converted to base 85, producing exactly 20 characters.
    ///
    /// # Example
    /// ```
    /// use codec::Base85Rfc1924;
    ///
    /// // IPv6: 1080:0:0:0:8:800:200C:417A
    /// let ipv6: u128 = 0x108000000000000000080800200C417A;
    /// let encoded = Base85Rfc1924::encode_u128(ipv6);
    /// assert_eq!(encoded, "4)+k&C#VzJ4br>0wv%Yp");
    /// ```
    pub fn encode_u128(mut value: u128) -> String {
        let mut buf = [0u8; 20];

        // Convert from base 256 to base 85, the least significant digit first
        for i in (0..20).rev() {
            buf[i] = ALPHABET_RFC1924[(value % 85) as usize];
            value /= 85;
        }

        // SAFETY: RFC 1924 alphabet consists only of ASCII characters
        String::from_utf8(buf.to_vec()).unwrap()
    }

    /// Decode exactly 20 characters to a 128-bit value
    ///
    /// # Errors
    /// Returns error if:
    /// - Input is not exactly 20 characters
    /// - Input contains invalid characters
    /// - Value overflows 128 bits
    ///
    /// # Example
    /// ```
    /// use codec::Base85Rfc1924;
    ///
    /// let decoded = Base85Rfc1924::decode_u128("4)+k&C#VzJ4br>0wv%Yp").unwrap();
    /// assert_eq!(decoded, 0x108000000000000000080800200C417A);
    /// ```
    pub fn decode_u128(input: &str) -> Result<u128, BinaryCodecError> {
        let bytes: Vec<u8> = input.bytes().collect();

        ensure!(
            bytes.len() == 20,
            InvalidLengthError { encoding: BinaryCodecs::Base85, length: bytes.len() }
        );

        let mut value: u128 = 0;

        for (i, &c) in bytes.iter().enumerate() {
            let digit = DECODE_TABLE_RFC1924[c as usize];
            ensure!(
                digit != 255,
                InvalidCharacterError {
                    encoding: BinaryCodecs::Base85,
                    char: c as char,
                    position: i,
                }
            );
            // Check for overflow before multiplication
            value = value
                .checked_mul(85)
                .and_then(|v| v.checked_add(digit as u128))
                .context(Base85OverflowError { position: i })?;
        }

        Ok(value)
    }
}

// ============================================================================
// btoa Implementation
// ============================================================================
impl Base85Codec for Base85Btoa {}
impl BinaryCodec for Base85Btoa {
    fn encode(&self, bytes: &[u8]) -> String {
        let mut out = Vec::with_capacity(bytes.len() * 5 / 4 + 64);
        out.extend_from_slice(b"xbtoa Begin\n");

        let chunks = bytes.chunks_exact(4);
        let remainder = chunks.remainder();

        encode_chunks_ascii85(&mut out, chunks);

        encode_partial(remainder, ALPHABET_ASCII85, &mut out);

        // Calculate checksum (simple sum of all bytes)
        let checksum: u32 = bytes.iter().map(|&b| b as u32).sum();

        out.push(b'\n');
        // btoa format: N <decimal len> <hex len> E <checksum> S <checksum> R <rotation checksum>
        let rot_checksum = bytes
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, &b)| acc.wrapping_add((b as u32).rotate_left(i as u32 & 31)));
        out.extend_from_slice(
            format!(
                "xbtoa End N {} {:x} E {:x} S {:x} R {:x}",
                bytes.len(),
                bytes.len(),
                bytes.len(),
                checksum,
                rot_checksum
            )
            .as_bytes(),
        );

        // SAFETY: btoa uses only ASCII characters
        String::from_utf8(out).unwrap()
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        let lines: Vec<&str> = text.lines().collect();

        // Find begin/end markers
        let begin_idx = lines
            .iter()
            .position(|l| l.trim().starts_with("xbtoa Begin"))
            .context(InvalidWrapperError)?;

        let end_idx = lines
            .iter()
            .position(|l| l.trim().starts_with("xbtoa End"))
            .context(InvalidWrapperError)?;

        // Extract data lines
        let data: String = lines[begin_idx + 1..end_idx]
            .iter()
            .flat_map(|l| l.chars())
            .filter(|c| !c.is_ascii_whitespace())
            .collect();

        let chars: Vec<u8> = data.bytes().collect();
        let mut out = decode_ascii85_chunks(&chars)?;

        // Parse footer for length info and truncate
        if let Some(end_line) = lines.get(end_idx)
            && let Some(n_pos) = end_line.find("N ")
        {
            let rest = &end_line[n_pos + 2..];
            if let Some(space_pos) = rest.find(' ')
                && let Ok(len) = rest[..space_pos].parse::<usize>()
            {
                out.truncate(len);
            }
        }

        Ok(out)
    }
}

// ============================================================================
// Enum Implementation
// ============================================================================
impl Base85Codec for Base85 {}
impl BinaryCodec for Base85 {
    fn encode(&self, bytes: &[u8]) -> String {
        match self {
            Base85::Ascii85 => Base85Ascii85.encode(bytes),
            Base85::Z85 => Base85Z85.encode(bytes),
            Base85::Rfc1924 => Base85Rfc1924.encode(bytes),
            Base85::Btoa => Base85Btoa.encode(bytes),
        }
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        match self {
            Base85::Ascii85 => Base85Ascii85.decode(text),
            Base85::Z85 => Base85Z85.decode(text),
            Base85::Rfc1924 => Base85Rfc1924.decode(text),
            Base85::Btoa => Base85Btoa.decode(text),
        }
    }
}

// ============================================================================
// Static Methods
// ============================================================================

impl Base85 {
    /// Encode bytes to Base85 string (Z85 variant)
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base85;
    ///
    /// let encoded = Base85::encode(b"Test");
    /// assert_eq!(Base85::decode(&encoded).unwrap(), b"Test");
    /// ```
    pub fn encode(bytes: &[u8]) -> String {
        Base85Z85.encode(bytes)
    }

    /// Decode Base85 string to bytes (Z85 variant)
    ///
    /// # Errors
    /// Returns error if:
    /// - Input length is not a multiple of 5
    /// - Input contains invalid characters
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base85;
    ///
    /// let data = [0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
    /// let decoded = Base85::decode("HelloWorld").unwrap();
    /// assert_eq!(decoded, data);
    /// ```
    pub fn decode(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base85Z85.decode(text)
    }

    /// Encode bytes to Ascii85 string
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base85;
    ///
    /// let encoded = Base85::encode_ascii85(b"Hello");
    /// assert!(encoded.starts_with("<~"));
    /// ```
    pub fn encode_ascii85(bytes: &[u8]) -> String {
        Base85Ascii85.encode(bytes)
    }

    /// Decode Ascii85 string to bytes
    ///
    /// # Errors
    /// Returns error if wrapper is missing or invalid characters are found
    pub fn decode_ascii85(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base85Ascii85.decode(text)
    }

    /// Encode bytes to Z85 string
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base85;
    ///
    /// let data = [0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
    /// assert_eq!(Base85::encode_z85(&data), "HelloWorld");
    /// ```
    pub fn encode_z85(bytes: &[u8]) -> String {
        Base85Z85.encode(bytes)
    }

    /// Decode Z85 string to bytes
    ///
    /// # Errors
    /// Returns error if length is not multiple of 5 or invalid characters found
    pub fn decode_z85(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base85Z85.decode(text)
    }

    /// Encode bytes to RFC 1924 Base85 string
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base85;
    ///
    /// let encoded = Base85::encode_rfc1924(b"test");
    /// let decoded = Base85::decode_rfc1924(&encoded).unwrap();
    /// assert_eq!(decoded, b"test");
    /// ```
    pub fn encode_rfc1924(bytes: &[u8]) -> String {
        Base85Rfc1924.encode(bytes)
    }

    /// Decode RFC 1924 Base85 string to bytes
    ///
    /// # Errors
    /// Returns error if invalid characters are found
    pub fn decode_rfc1924(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base85Rfc1924.decode(text)
    }

    /// Encode bytes to btoa format
    ///
    /// # Examples
    ///
    /// ```
    /// use codec::Base85;
    ///
    /// let encoded = Base85::encode_btoa(b"Hello");
    /// assert!(encoded.contains("xbtoa Begin"));
    /// ```
    pub fn encode_btoa(bytes: &[u8]) -> String {
        Base85Btoa.encode(bytes)
    }

    /// Decode btoa format to bytes
    ///
    /// # Errors
    /// Returns error if wrapper is missing or invalid characters are found
    pub fn decode_btoa(text: &str) -> Result<Vec<u8>, BinaryCodecError> {
        Base85Btoa.decode(text)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii85_encode_decode() {
        let input = b"Hello, World!";
        let encoded = Base85Ascii85.encode(input);
        assert!(encoded.starts_with("<~"));
        assert!(encoded.ends_with("~>"));
        assert_eq!(encoded, "<~87cURD_*#4DfTZ)+T~>");

        let decoded = Base85Ascii85.decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_ascii85_zero_compression() {
        let input = [0u8; 4];
        let encoded = Base85Ascii85.encode(&input);
        assert_eq!(encoded, "<~z~>");

        let decoded = Base85Ascii85.decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_ascii85_known_vector() {
        // "Man " encodes to "9jqo^" in Ascii85
        let input = b"Man ";
        let encoded = Base85Ascii85.encode(input);
        assert_eq!(encoded, "<~9jqo^~>");
    }

    #[test]
    fn test_z85_encode_decode() {
        let input = [0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
        let encoded = Base85Z85.encode(&input);
        assert_eq!(encoded, "HelloWorld");

        let decoded = Base85Z85.decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_z85_requires_multiple_of_5() {
        let result = Base85Z85.decode("Hell"); // 4 chars, not multiple of 5
        assert!(matches!(result, Err(BinaryCodecError::InvalidLength { .. })));
    }

    #[test]
    fn test_rfc1924_encode_decode() {
        let input = b"test";
        let encoded = Base85Rfc1924.encode(input);
        assert_eq!(encoded, "bY*jN");
        let decoded = Base85Rfc1924.decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_rfc1924_u128() {
        let value: u128 = 0x0102030405060708090a0b0c0d0e0f10;
        let encoded = Base85Rfc1924::encode_u128(value);
        assert_eq!(encoded.len(), 20);
        assert_eq!(encoded, "0O|s)GT}-*WUn!Z$mO1p");

        let decoded = Base85Rfc1924::decode_u128(&encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_btoa_encode_decode() {
        let input = b"Hello, World!";
        let encoded = Base85Btoa.encode(input);
        assert!(encoded.contains("xbtoa Begin"));
        assert!(encoded.contains("xbtoa End"));
        assert!(encoded.contains("87cURD_*#4DfTZ)+T"));

        let decoded = Base85Btoa.decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_enum_variants() {
        let input = b"Test";

        for variant in [Base85::Ascii85, Base85::Z85, Base85::Rfc1924, Base85::Btoa] {
            let encoded = variant.encode(input);
            let decoded = variant.decode(&encoded).unwrap();
            assert_eq!(decoded, input, "Failed for {:?}", variant);
        }
    }

    #[test]
    fn test_static_methods() {
        let input = b"Test";

        // Default (Z85)
        let encoded = Base85::encode(input);
        let decoded = Base85::decode(&encoded).unwrap();
        assert_eq!(decoded, input);

        // Ascii85
        let encoded = Base85::encode_ascii85(input);
        assert!(encoded.starts_with("<~"));
        let decoded = Base85::decode_ascii85(&encoded).unwrap();
        assert_eq!(decoded, input);

        // Z85
        let encoded = Base85::encode_z85(input);
        let decoded = Base85::decode_z85(&encoded).unwrap();
        assert_eq!(decoded, input);

        // RFC 1924
        let encoded = Base85::encode_rfc1924(input);
        let decoded = Base85::decode_rfc1924(&encoded).unwrap();
        assert_eq!(decoded, input);

        // btoa
        let encoded = Base85::encode_btoa(input);
        assert!(encoded.contains("xbtoa"));
        let decoded = Base85::decode_btoa(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(Base85Ascii85.encode(b""), "<~~>");
        assert_eq!(Base85Ascii85.decode("<~~>").unwrap(), vec![]);

        assert_eq!(Base85Z85.encode(b""), "");
        assert_eq!(Base85Z85.decode("").unwrap(), vec![]);
    }

    #[test]
    fn test_partial_blocks() {
        // 1 byte
        let input = b"A";
        let decoded = Base85Ascii85.decode(&Base85Ascii85.encode(input)).unwrap();
        assert_eq!(decoded, input);

        // 2 bytes
        let input = b"AB";
        let decoded = Base85Ascii85.decode(&Base85Ascii85.encode(input)).unwrap();
        assert_eq!(decoded, input);

        // 3 bytes
        let input = b"ABC";
        let decoded = Base85Ascii85.decode(&Base85Ascii85.encode(input)).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_invalid_char() {
        // Ascii85 doesn't include 'v' (beyond 'u')
        let result = Base85Ascii85.decode("<~vvvvv~>");
        assert!(matches!(result, Err(BinaryCodecError::InvalidCharacter { .. })));
    }

    #[test]
    fn test_trait_object() {
        // Z85はinputが4の倍数である必要があるため、8バイトのデータを使用
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
    fn test_base85_codec_marker_trait() {
        fn encode_with_base85<C: Base85Codec>(codec: &C, data: &[u8]) -> String {
            codec.encode(data)
        }

        let data = b"Test";
        let _ = encode_with_base85(&Base85Ascii85, data);
        let _ = encode_with_base85(&Base85Z85, data);
        let _ = encode_with_base85(&Base85Rfc1924, data);
        let _ = encode_with_base85(&Base85Btoa, data);
        let _ = encode_with_base85(&Base85::default(), data);
    }

    // ========================================================================
    // Official test vectors from specifications
    // ========================================================================

    /// Wikipedia / Go standard library official test vector
    /// Source: https://en.wikipedia.org/wiki/Ascii85
    #[test]
    fn test_ascii85_leviathan_official() {
        let plaintext = b"Man is distinguished, not only by his reason, but by this \
singular passion from other animals, which is a lust of the mind, that by a \
perseverance of delight in the continued and indefatigable generation of \
knowledge, exceeds the short vehemence of any carnal pleasure.";

        // Official encoded value (without <~ ~> wrapper, whitespace removed)
        let expected_inner = "9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,\
O<DJ+*.@<*K0@<6L(Df-\\0Ec5e;DffZ(EZee.Bl.9pF\"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKY\
i(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIa\
l(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G\
>uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c";

        let encoded = Base85Ascii85.encode(plaintext);

        // Strip wrapper and compare
        let inner = encoded.strip_prefix("<~").unwrap().strip_suffix("~>").unwrap();
        assert_eq!(inner, expected_inner);

        // Verify round-trip
        let decoded = Base85Ascii85.decode(&encoded).unwrap();
        assert_eq!(decoded, plaintext);
    }

    /// ZeroMQ RFC 32 official test vector #1
    /// Source: https://rfc.zeromq.org/spec/32/
    #[test]
    fn test_z85_rfc32_vector1() {
        let input: [u8; 8] = [0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
        let encoded = Base85Z85.encode(&input);
        assert_eq!(encoded, "HelloWorld");
    }

    /// ZeroMQ RFC 32 official test vector #2 (32 bytes)
    /// Source: https://github.com/zeromq/rfc/blob/master/src/spec_32.c
    #[test]
    fn test_z85_rfc32_vector2() {
        let input: [u8; 32] = [
            0x8E, 0x0B, 0xDD, 0x69, 0x76, 0x28, 0xB9, 0x1D, 0x8F, 0x24, 0x55, 0x87, 0xEE, 0x95,
            0xC5, 0xB0, 0x4D, 0x48, 0x96, 0x3F, 0x79, 0x25, 0x98, 0x77, 0xB4, 0x9C, 0xD9, 0x06,
            0x3A, 0xEA, 0xD3, 0xB7,
        ];

        // Verify a round-trip (we don't have the expected encoded value from RFC)
        let encoded = Base85Z85.encode(&input);
        assert_eq!(encoded.len(), 40); // 32 bytes -> 40 chars
        assert_eq!(encoded, "JTKVSB%%)wK0E.X)V>+}o?pNmC{O&4W4b!Ni{Lh6");

        let decoded = Base85Z85.decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    /// Verify Z85 alphabet matches ZeroMQ RFC 32 specification
    #[test]
    fn test_z85_alphabet_matches_spec() {
        // Official Z85 alphabet from RFC 32
        let official = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
        assert_eq!(ALPHABET_Z85, official);
        assert_eq!(ALPHABET_Z85.len(), 85);
    }

    /// Verify Ascii85 alphabet matches Adobe specification
    #[test]
    fn test_ascii85_alphabet_matches_spec() {
        // Adobe Ascii85: '!' (33) through 'u' (117)
        let expected: Vec<u8> = (b'!'..=b'u').collect();
        assert_eq!(ALPHABET_ASCII85.len(), 85);
        assert_eq!(ALPHABET_ASCII85[0], b'!');
        assert_eq!(ALPHABET_ASCII85[84], b'u');
        assert_eq!(ALPHABET_ASCII85.as_slice(), expected.as_slice());
    }

    /// Verify RFC 1924 alphabet
    #[test]
    fn test_rfc1924_alphabet_matches_spec() {
        // RFC 1924: 0-9, A-Z, a-z, !#$%&()*+-;<=>?@^_`{|}~
        let expected =
            b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";
        assert_eq!(ALPHABET_RFC1924.len(), 85);
        assert_eq!(ALPHABET_RFC1924, expected);
    }

    /// RFC 1924 official test vector: IPv6 address 1080:0:0:0:8:800:200C:417A
    /// Source: https://www.rfc-editor.org/rfc/rfc1924.txt Section 5
    #[test]
    fn test_rfc1924_ipv6_official_vector() {
        // IPv6: 1080:0:0:0:8:800:200C:417A
        // As 128-bit: 0x108000000000000000080800200C417A
        let ipv6: u128 = 0x108000000000000000080800200C417A;

        // RFC says this encodes to: 4)+k&C#VzJ4br>0wv%Yp
        let encoded = Base85Rfc1924::encode_u128(ipv6);
        assert_eq!(encoded, "4)+k&C#VzJ4br>0wv%Yp");

        // Verify round-trip
        let decoded = Base85Rfc1924::decode_u128(&encoded).unwrap();
        assert_eq!(decoded, ipv6);
    }

    /// Test zero compression in Ascii85
    #[test]
    fn test_ascii85_zero_compression_multiple() {
        // 8 zero bytes should encode as "zz"
        let input = [0u8; 8];
        let encoded = Base85Ascii85.encode(&input);
        assert_eq!(encoded, "<~zz~>");

        let decoded = Base85Ascii85.decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    /// Test that 's8W-!' is the maximum valid 5-char sequence in Ascii85
    /// Values greater than 2^32-1 should overflow
    #[test]
    fn test_ascii85_max_value() {
        // s8W-! decodes to 0xFFFFFFFF (max u32)
        let result = Base85Ascii85.decode("<~s8W-!~>");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0xFF, 0xFF, 0xFF, 0xFF]);
    }

    /// Test overflow detection
    #[test]
    fn test_ascii85_overflow() {
        // `s8W-"` would be 0xFFFFFFFF + 1, which overflows
        let result = Base85Ascii85.decode("<~s8W-\"~>");
        assert!(matches!(result, Err(BinaryCodecError::Base85Overflow { .. })));
    }

    /// Go standard library test: verify specific encodings
    #[test]
    fn test_ascii85_go_stdlib_vectors() {
        // From Go's encoding/ascii85/ascii85_test.go
        let cases = [
            (b"".as_slice(), ""),
            (b"\x00\x00\x00\x00".as_slice(), "z"), // zero compression
        ];

        for (input, expected_inner) in cases {
            let encoded = Base85Ascii85.encode(input);
            let inner = encoded.strip_prefix("<~").unwrap().strip_suffix("~>").unwrap();
            assert_eq!(inner, expected_inner, "Failed for input {:?}", input);
        }
    }

    /// Go stdlib: 'z' in the middle of a group is invalid
    /// Source: encoding/ascii85/ascii85_test.go TestDecodeCorrupt
    #[test]
    fn test_ascii85_z_in_middle_of_group() {
        // "!z!!!!!!!!!" - z at position 1 is invalid (in the middle of a group)
        let result = Base85Ascii85.decode("<~!z!!!!!!!!!~>");
        assert!(result.is_err(), "Should reject 'z' in middle of group");
    }

    // ========================================================================
    // Z85 Official Test Vectors from ZeroMQ RFC 32
    // Source: https://github.com/zeromq/rfc/blob/master/src/spec_32.c
    // ========================================================================

    /// Z85 RFC 32: Empty input
    #[test]
    fn test_z85_rfc32_empty() {
        let encoded = Base85Z85.encode(b"");
        assert_eq!(encoded, "");

        let decoded = Base85Z85.decode("").unwrap();
        assert_eq!(decoded, vec![]);
    }

    /// Z85 RFC 32: test_data_2 with expected output string
    #[test]
    fn test_z85_rfc32_vector2_with_expected() {
        let input: [u8; 32] = [
            0x8E, 0x0B, 0xDD, 0x69, 0x76, 0x28, 0xB9, 0x1D, 0x8F, 0x24, 0x55, 0x87, 0xEE, 0x95,
            0xC5, 0xB0, 0x4D, 0x48, 0x96, 0x3F, 0x79, 0x25, 0x98, 0x77, 0xB4, 0x9C, 0xD9, 0x06,
            0x3A, 0xEA, 0xD3, 0xB7,
        ];

        let encoded = Base85Z85.encode(&input);
        assert_eq!(encoded, "JTKVSB%%)wK0E.X)V>+}o?pNmC{O&4W4b!Ni{Lh6");

        let decoded = Base85Z85.decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    /// Z85 RFC 32: CurveZMQ client public key
    #[test]
    fn test_z85_rfc32_curve_client_public() {
        let client_public: [u8; 32] = [
            0xBB, 0x88, 0x47, 0x1D, 0x65, 0xE2, 0x65, 0x9B, 0x30, 0xC5, 0x5A, 0x53, 0x21, 0xCE,
            0xBB, 0x5A, 0xAB, 0x2B, 0x70, 0xA3, 0x98, 0x64, 0x5C, 0x26, 0xDC, 0xA2, 0xB2, 0xFC,
            0xB4, 0x3F, 0xC5, 0x18,
        ];

        let encoded = Base85Z85.encode(&client_public);
        assert_eq!(encoded.len(), 40);
        assert_eq!(encoded, "Yne@$w-vo<fVvi]a<NY6T1ed:M$fCG*[IaLV{hID");

        // Verify round-trip
        let decoded = Base85Z85.decode(&encoded).unwrap();
        assert_eq!(decoded, client_public);
    }

    /// Z85 RFC 32: CurveZMQ client secret key
    #[test]
    fn test_z85_rfc32_curve_client_secret() {
        let client_secret: [u8; 32] = [
            0x7B, 0xB8, 0x64, 0xB4, 0x89, 0xAF, 0xA3, 0x67, 0x1F, 0xBE, 0x69, 0x10, 0x1F, 0x94,
            0xB3, 0x89, 0x72, 0xF2, 0x48, 0x16, 0xDF, 0xB0, 0x1B, 0x51, 0x65, 0x6B, 0x3F, 0xEC,
            0x8D, 0xFD, 0x08, 0x88,
        ];

        let encoded = Base85Z85.encode(&client_secret);
        assert_eq!(encoded.len(), 40);
        assert_eq!(encoded, "D:)Q[IlAW!ahhC2ac:9*A}h:p?([4%wOTJ%JR%cs");

        let decoded = Base85Z85.decode(&encoded).unwrap();
        assert_eq!(decoded, client_secret);
    }

    /// Z85 RFC 32: CurveZMQ server public key
    #[test]
    fn test_z85_rfc32_curve_server_public() {
        let server_public: [u8; 32] = [
            0x54, 0xFC, 0xBA, 0x24, 0xE9, 0x32, 0x49, 0x96, 0x93, 0x16, 0xFB, 0x61, 0x7C, 0x87,
            0x2B, 0xB0, 0xC1, 0xD1, 0xFF, 0x14, 0x80, 0x04, 0x27, 0xC5, 0x94, 0xCB, 0xFA, 0xCF,
            0x1B, 0xC2, 0xD6, 0x52,
        ];

        let encoded = Base85Z85.encode(&server_public);
        assert_eq!(encoded.len(), 40);
        assert_eq!(encoded, "rq:rM>}U?@Lns47E1%kR.o@n%FcmmsL/@{H8]yf7");

        let decoded = Base85Z85.decode(&encoded).unwrap();
        assert_eq!(decoded, server_public);
    }

    /// Z85 RFC 32: CurveZMQ server secret key (same as test_data_2)
    #[test]
    fn test_z85_rfc32_curve_server_secret() {
        let server_secret: [u8; 32] = [
            0x8E, 0x0B, 0xDD, 0x69, 0x76, 0x28, 0xB9, 0x1D, 0x8F, 0x24, 0x55, 0x87, 0xEE, 0x95,
            0xC5, 0xB0, 0x4D, 0x48, 0x96, 0x3F, 0x79, 0x25, 0x98, 0x77, 0xB4, 0x9C, 0xD9, 0x06,
            0x3A, 0xEA, 0xD3, 0xB7,
        ];

        // This is the same as test_data_2
        let encoded = Base85Z85.encode(&server_secret);
        assert_eq!(encoded, "JTKVSB%%)wK0E.X)V>+}o?pNmC{O&4W4b!Ni{Lh6");
    }

    /// Z85 RFC 32: Invalid input length (not multiple of 4)
    #[test]
    fn test_z85_rfc32_invalid_input_length() {
        // The reference implementation returns NULL for input not divisible by 4
        // Our implementation pads, but decode should reject strings not divisible by 5
        let result = Base85Z85.decode("HelloWorl"); // 9 chars, not multiple of 5
        assert!(matches!(result, Err(BinaryCodecError::InvalidLength { .. })));
    }

    // ========================================================================
    // UUID Encoding/Decoding Tests
    // ========================================================================

    /// Test UUID encoding/decoding with Ascii85
    #[test]
    fn test_uuid_encode_decode_ascii85() {
        // UUID: 550e8400-e29b-41d4-a716-446655440000
        let uuid_bytes: [u8; 16] = [
            0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
            0x00, 0x00,
        ];

        let encoded = Base85Ascii85.encode(&uuid_bytes);
        assert_eq!(encoded, "<~<=XZ^igW6`V\\V[4<C?fe~>");

        let decoded = Base85Ascii85.decode(&encoded).unwrap();
        assert_eq!(decoded, uuid_bytes);
    }

    /// Test UUID encoding/decoding with Z85
    #[test]
    fn test_uuid_encode_decode_z85() {
        // UUID: 550e8400-e29b-41d4-a716-446655440000
        let uuid_bytes: [u8; 16] = [
            0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
            0x00, 0x00,
        ];

        let encoded = Base85Z85.encode(&uuid_bytes);
        assert_eq!(encoded, "rsTVZ&*Sl-RXRWjryu/!");

        let decoded = Base85Z85.decode(&encoded).unwrap();
        assert_eq!(decoded, uuid_bytes);
    }

    /// Test UUID encoding/decoding with RFC 1924 (128-bit)
    #[test]
    fn test_uuid_encode_decode_rfc1924() {
        // UUID: 550e8400-e29b-41d4-a716-446655440000
        let uuid_u128: u128 = 0x550e8400e29b41d4a71644665544000;

        let encoded = Base85Rfc1924::encode_u128(uuid_u128);
        assert_eq!(encoded, "1kz7d$i1_;SVnmd|MB(z");

        let decoded = Base85Rfc1924::decode_u128(&encoded).unwrap();
        assert_eq!(decoded, uuid_u128);

        // Test as bytes
        let uuid_bytes: [u8; 16] = [
            0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
            0x00, 0x00,
        ];

        let encoded_bytes = Base85Rfc1924.encode(&uuid_bytes);
        assert_eq!(encoded_bytes, "RStvz;+sL#rxrwJRYU*)");

        let decoded_bytes = Base85Rfc1924.decode(&encoded_bytes).unwrap();
        assert_eq!(decoded_bytes, uuid_bytes);
    }

    /// Test UUID encoding/decoding with btoa
    #[test]
    fn test_uuid_encode_decode_btoa() {
        // UUID: 550e8400-e29b-41d4-a716-446655440000
        let uuid_bytes: [u8; 16] = [
            0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
            0x00, 0x00,
        ];

        let encoded = Base85Btoa.encode(&uuid_bytes);
        assert!(encoded.contains("xbtoa Begin"));
        assert!(encoded.contains("xbtoa End"));
        assert!(encoded.contains("<=XZ^igW6`V\\V[4<C?fe"));

        let decoded = Base85Btoa.decode(&encoded).unwrap();
        assert_eq!(decoded, uuid_bytes);
    }

    // ========================================================================
    // Edge Case / Coverage Tests
    // ========================================================================

    /// Z85 encode with non-multiple-of-4 input (triggers padding path)
    #[test]
    fn test_z85_encode_non_aligned_input() {
        // 1 byte: pads to 4 bytes, encodes to 5 chars
        let input1 = [0x42u8];
        let encoded1 = Base85Z85.encode(&input1);
        assert_eq!(encoded1, "li3#=");
        assert_eq!(encoded1.len(), 5);

        // 2 bytes: pads to 4 bytes, encodes to 5 chars
        let input2 = [0x42u8, 0x43];
        let encoded2 = Base85Z85.encode(&input2);
        assert_eq!(encoded2, "lpg.M");
        assert_eq!(encoded2.len(), 5);

        // 3 bytes: pads to 4 bytes, encodes to 5 chars
        let input3 = [0x42u8, 0x43, 0x44];
        let encoded3 = Base85Z85.encode(&input3);
        assert_eq!(encoded3, "lpjcv");
        assert_eq!(encoded3.len(), 5);

        // 5 bytes: pads to 8 bytes, encodes to 10 chars
        let input5 = [0x42u8, 0x43, 0x44, 0x45, 0x46];
        let encoded5 = Base85Z85.encode(&input5);
        assert_eq!(encoded5, "lpjdfmGrz*");
        assert_eq!(encoded5.len(), 10);

        // Verify round-trip with padding API
        let (enc, pad) = Base85Z85.encode_with_padding(&input1);
        assert_eq!(enc, "li3#=");
        assert_eq!(pad, 3);
        let decoded = Base85Z85.decode_with_padding(&enc, pad).unwrap();
        assert_eq!(decoded, input1);
    }

    /// Ascii85 decode with single char partial block (triggers InvalidLength)
    #[test]
    fn test_ascii85_single_char_partial_block() {
        // 6文字 = 5文字の完全ブロック + 1文字の不完全ブロック
        // 不完全ブロックは最低2文字必要（1文字では1バイトも復元できない）
        let result = Base85Ascii85.decode("<~!!!!!!~>");
        assert!(matches!(result, Err(BinaryCodecError::InvalidLength { .. })));

        // 2文字の不完全ブロックは有効（1バイトを復元）
        let result = Base85Ascii85.decode("<~!!!!!00~>");
        assert!(result.is_ok());
    }

    /// RFC1924 partial block with single char (triggers InvalidLength)
    #[test]
    fn test_rfc1924_single_char_partial_block() {
        // 21文字 = 20文字の完全ブロック + 1文字の不完全ブロック
        // 不完全ブロックは最低2文字必要
        let result = Base85Rfc1924.decode("00000000000000000000A");
        assert!(matches!(result, Err(BinaryCodecError::InvalidLength { .. })));

        // 2文字の不完全ブロックは有効
        let result = Base85Rfc1924.decode("0000000000000000000000");
        assert!(result.is_ok());
    }

    /// Invalid char in partial block (triggers InvalidCharacterError in pad_with_highest_value)
    #[test]
    fn test_ascii85_invalid_char_in_partial_block() {
        // 5文字の完全ブロック + 2文字の不完全ブロック（無効な文字を含む）
        // Ascii85は '!' (33) から 'u' (117) を使用、'|' (124) は無効
        let result = Base85Ascii85.decode("<~!!!!!|x~>");
        assert!(matches!(result, Err(BinaryCodecError::InvalidCharacter { char: '|', .. })));
    }

    /// decode_u128 with invalid length (triggers InvalidLength at line 573)
    #[test]
    fn test_rfc1924_u128_invalid_length() {
        // 短すぎる（5文字）
        let result = Base85Rfc1924::decode_u128("short");
        assert!(matches!(result, Err(BinaryCodecError::InvalidLength { length: 5, .. })));

        // 長すぎる（21文字）
        let result = Base85Rfc1924::decode_u128("000000000000000000000");
        assert!(matches!(result, Err(BinaryCodecError::InvalidLength { length: 21, .. })));

        // 空文字列
        let result = Base85Rfc1924::decode_u128("");
        assert!(matches!(result, Err(BinaryCodecError::InvalidLength { length: 0, .. })));
    }

    /// decode_u128 with invalid character (triggers InvalidCharacter at lines 582-586)
    #[test]
    fn test_rfc1924_u128_invalid_char() {
        // RFC1924アルファベットに含まれない文字（スペース）を含む20文字
        // RFC1924アルファベット: 0-9 A-Z a-z !#$%&()*+-;<=>?@^_`{|}~
        let result = Base85Rfc1924::decode_u128("0000000000000000000 "); // 19個の0 + スペース
        assert!(matches!(
            result,
            Err(BinaryCodecError::InvalidCharacter { char: ' ', position: 19, .. })
        ));

        // 先頭に無効文字（20文字）
        let result = Base85Rfc1924::decode_u128(" 0000000000000000000"); // スペース + 19個の0
        assert!(matches!(
            result,
            Err(BinaryCodecError::InvalidCharacter { char: ' ', position: 0, .. })
        ));

        // 中間に無効文字
        let result = Base85Rfc1924::decode_u128("0000000000 000000000"); // 10個の0 + スペース + 9個の0
        assert!(matches!(
            result,
            Err(BinaryCodecError::InvalidCharacter { char: ' ', position: 10, .. })
        ));
    }

    /// btoa decode with malformed footer (triggers line 674 path)
    #[test]
    fn test_btoa_malformed_footer() {
        // N の後にスペースがない（解析失敗パス）
        let malformed = "xbtoa Begin\n87cURD_*#4DfTZ)+T\nxbtoa End N13 d E d S 469 R 87912";
        let result = Base85Btoa.decode(malformed);
        // デコード自体は成功するが、truncateは行われない
        assert!(result.is_ok());

        // N が存在しない
        let no_n = "xbtoa Begin\n87cURD_*#4DfTZ)+T\nxbtoa End";
        let result = Base85Btoa.decode(no_n);
        assert!(result.is_ok());
    }
}
