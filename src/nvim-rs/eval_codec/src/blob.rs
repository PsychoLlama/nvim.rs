//! Binary blob encoding helpers.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Blob Representation
// =============================================================================

/// VimL blob representation format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BlobFormat {
    /// Raw bytes
    Raw = 0,
    /// Hexadecimal string (0zXXXX)
    Hex = 1,
    /// Base64 encoded
    Base64 = 2,
}

impl BlobFormat {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Hex,
            2 => Self::Base64,
            _ => Self::Raw,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Hex Encoding
// =============================================================================

/// Encode bytes as hexadecimal string.
pub fn encode_hex(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(input.len() * 2);
    for &byte in input {
        result.push(hex_digit_upper(byte >> 4));
        result.push(hex_digit_upper(byte & 0x0F));
    }
    result
}

/// Decode hexadecimal string to bytes.
pub fn decode_hex(input: &[u8]) -> Option<Vec<u8>> {
    if !input.len().is_multiple_of(2) {
        return None;
    }

    let mut result = Vec::with_capacity(input.len() / 2);
    let mut i = 0;

    while i < input.len() {
        let high = hex_value(input[i])?;
        let low = hex_value(input[i + 1])?;
        result.push((high << 4) | low);
        i += 2;
    }

    Some(result)
}

/// Get uppercase hex digit for value 0-15.
const fn hex_digit_upper(val: u8) -> u8 {
    match val {
        0..=9 => b'0' + val,
        10..=15 => b'A' + (val - 10),
        _ => b'?',
    }
}

/// Get value of hex digit (0-15) or None if invalid.
const fn hex_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// FFI export: encode bytes to hex.
///
/// Returns number of bytes written to output.
///
/// # Safety
/// - `input` must be valid pointer to `input_len` bytes, or null.
/// - `output` must be valid pointer to at least `input_len * 2` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_blob_encode_hex(
    input: *const u8,
    input_len: c_int,
    output: *mut u8,
    output_len: c_int,
) -> c_int {
    if input.is_null() || output.is_null() || input_len < 0 || output_len < 0 {
        return 0;
    }

    let input_slice = unsafe { std::slice::from_raw_parts(input, input_len as usize) };
    let output_slice = unsafe { std::slice::from_raw_parts_mut(output, output_len as usize) };

    let needed = input_len as usize * 2;
    if output_slice.len() < needed {
        return 0;
    }

    let encoded = encode_hex(input_slice);
    output_slice[..encoded.len()].copy_from_slice(&encoded);
    encoded.len() as c_int
}

// =============================================================================
// VimL Blob Literal
// =============================================================================

/// Check if string is a valid VimL blob literal (0zXXXX).
pub fn is_blob_literal(input: &[u8]) -> bool {
    if input.len() < 2 {
        return false;
    }
    if input[0] != b'0' || (input[1] != b'z' && input[1] != b'Z') {
        return false;
    }
    // Rest must be hex pairs (or dots for separation)
    let rest = &input[2..];
    if rest.is_empty() {
        return true; // Empty blob: 0z
    }

    let mut i = 0;
    while i < rest.len() {
        if rest[i] == b'.' {
            i += 1;
            continue;
        }
        // Must have two hex digits
        if i + 1 >= rest.len() {
            return false;
        }
        if hex_value(rest[i]).is_none() || hex_value(rest[i + 1]).is_none() {
            return false;
        }
        i += 2;
    }
    true
}

/// Parse VimL blob literal to bytes.
pub fn parse_blob_literal(input: &[u8]) -> Option<Vec<u8>> {
    if input.len() < 2 || input[0] != b'0' || (input[1] != b'z' && input[1] != b'Z') {
        return None;
    }

    let rest = &input[2..];
    if rest.is_empty() {
        return Some(Vec::new());
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < rest.len() {
        if rest[i] == b'.' {
            i += 1;
            continue;
        }
        if i + 1 >= rest.len() {
            return None;
        }
        let high = hex_value(rest[i])?;
        let low = hex_value(rest[i + 1])?;
        result.push((high << 4) | low);
        i += 2;
    }

    Some(result)
}

/// Format bytes as VimL blob literal.
pub fn format_blob_literal(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(2 + bytes.len() * 2);
    result.extend_from_slice(b"0z");
    for &byte in bytes {
        result.push(hex_digit_upper(byte >> 4));
        result.push(hex_digit_upper(byte & 0x0F));
    }
    result
}

/// FFI export: check if blob literal.
///
/// # Safety
/// - `input` must be valid pointer to `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_blob_is_literal(input: *const u8, len: c_int) -> bool {
    if input.is_null() || len < 0 {
        return false;
    }

    let slice = unsafe { std::slice::from_raw_parts(input, len as usize) };
    is_blob_literal(slice)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_format() {
        assert_eq!(BlobFormat::from_c_int(0), BlobFormat::Raw);
        assert_eq!(BlobFormat::from_c_int(1), BlobFormat::Hex);
        assert_eq!(BlobFormat::from_c_int(2), BlobFormat::Base64);
    }

    #[test]
    fn test_encode_hex() {
        assert_eq!(encode_hex(b""), b"");
        assert_eq!(encode_hex(b"\x00"), b"00");
        assert_eq!(encode_hex(b"\xFF"), b"FF");
        assert_eq!(encode_hex(b"AB"), b"4142");
    }

    #[test]
    fn test_decode_hex() {
        assert_eq!(decode_hex(b""), Some(vec![]));
        assert_eq!(decode_hex(b"00"), Some(vec![0x00]));
        assert_eq!(decode_hex(b"FF"), Some(vec![0xFF]));
        assert_eq!(decode_hex(b"4142"), Some(b"AB".to_vec()));
        assert_eq!(decode_hex(b"0"), None); // Odd length
        assert_eq!(decode_hex(b"GG"), None); // Invalid hex
    }

    #[test]
    fn test_is_blob_literal() {
        assert!(is_blob_literal(b"0z"));
        assert!(is_blob_literal(b"0z00"));
        assert!(is_blob_literal(b"0zFF"));
        assert!(is_blob_literal(b"0z00.11.22"));
        assert!(is_blob_literal(b"0Z00"));

        assert!(!is_blob_literal(b"0"));
        assert!(!is_blob_literal(b"0x00"));
        assert!(!is_blob_literal(b"0z0")); // Odd hex
    }

    #[test]
    fn test_parse_blob_literal() {
        assert_eq!(parse_blob_literal(b"0z"), Some(vec![]));
        assert_eq!(parse_blob_literal(b"0z00"), Some(vec![0x00]));
        assert_eq!(parse_blob_literal(b"0zFF"), Some(vec![0xFF]));
        assert_eq!(
            parse_blob_literal(b"0z00.11.22"),
            Some(vec![0x00, 0x11, 0x22])
        );
    }

    #[test]
    fn test_format_blob_literal() {
        assert_eq!(format_blob_literal(b""), b"0z");
        assert_eq!(format_blob_literal(&[0x00]), b"0z00");
        assert_eq!(format_blob_literal(&[0xFF]), b"0zFF");
        assert_eq!(format_blob_literal(&[0x00, 0x11, 0x22]), b"0z001122");
    }
}
