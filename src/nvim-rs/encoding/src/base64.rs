//! Base64 encoding and decoding
//!
//! Provides standard Base64 encoding/decoding compatible with nvim's base64.c.

use std::ffi::c_char;
use std::ptr;

use nvim_memory::xmalloc;

/// Base64 alphabet
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Reverse lookup table (0 = invalid, 1-64 = valid index + 1)
const CHAR_TO_INDEX: [u8; 256] = {
    let mut table = [0u8; 256];
    table[b'A' as usize] = 1;
    table[b'B' as usize] = 2;
    table[b'C' as usize] = 3;
    table[b'D' as usize] = 4;
    table[b'E' as usize] = 5;
    table[b'F' as usize] = 6;
    table[b'G' as usize] = 7;
    table[b'H' as usize] = 8;
    table[b'I' as usize] = 9;
    table[b'J' as usize] = 10;
    table[b'K' as usize] = 11;
    table[b'L' as usize] = 12;
    table[b'M' as usize] = 13;
    table[b'N' as usize] = 14;
    table[b'O' as usize] = 15;
    table[b'P' as usize] = 16;
    table[b'Q' as usize] = 17;
    table[b'R' as usize] = 18;
    table[b'S' as usize] = 19;
    table[b'T' as usize] = 20;
    table[b'U' as usize] = 21;
    table[b'V' as usize] = 22;
    table[b'W' as usize] = 23;
    table[b'X' as usize] = 24;
    table[b'Y' as usize] = 25;
    table[b'Z' as usize] = 26;
    table[b'a' as usize] = 27;
    table[b'b' as usize] = 28;
    table[b'c' as usize] = 29;
    table[b'd' as usize] = 30;
    table[b'e' as usize] = 31;
    table[b'f' as usize] = 32;
    table[b'g' as usize] = 33;
    table[b'h' as usize] = 34;
    table[b'i' as usize] = 35;
    table[b'j' as usize] = 36;
    table[b'k' as usize] = 37;
    table[b'l' as usize] = 38;
    table[b'm' as usize] = 39;
    table[b'n' as usize] = 40;
    table[b'o' as usize] = 41;
    table[b'p' as usize] = 42;
    table[b'q' as usize] = 43;
    table[b'r' as usize] = 44;
    table[b's' as usize] = 45;
    table[b't' as usize] = 46;
    table[b'u' as usize] = 47;
    table[b'v' as usize] = 48;
    table[b'w' as usize] = 49;
    table[b'x' as usize] = 50;
    table[b'y' as usize] = 51;
    table[b'z' as usize] = 52;
    table[b'0' as usize] = 53;
    table[b'1' as usize] = 54;
    table[b'2' as usize] = 55;
    table[b'3' as usize] = 56;
    table[b'4' as usize] = 57;
    table[b'5' as usize] = 58;
    table[b'6' as usize] = 59;
    table[b'7' as usize] = 60;
    table[b'8' as usize] = 61;
    table[b'9' as usize] = 62;
    table[b'+' as usize] = 63;
    table[b'/' as usize] = 64;
    table
};

/// Encode binary data to Base64.
///
/// Returns a null-terminated string allocated with xmalloc.
/// The caller must free the result.
pub fn encode(src: &[u8]) -> Vec<u8> {
    let out_len = ((src.len() + 2) / 3) * 4;
    let mut dest = Vec::with_capacity(out_len + 1);

    let mut i = 0;
    while i + 2 < src.len() {
        let b0 = src[i];
        let b1 = src[i + 1];
        let b2 = src[i + 2];

        dest.push(ALPHABET[(b0 >> 2) as usize]);
        dest.push(ALPHABET[(((b0 & 0x3) << 4) | (b1 >> 4)) as usize]);
        dest.push(ALPHABET[(((b1 & 0xF) << 2) | (b2 >> 6)) as usize]);
        dest.push(ALPHABET[(b2 & 0x3F) as usize]);

        i += 3;
    }

    // Handle remaining bytes
    if i + 1 < src.len() {
        let b0 = src[i];
        let b1 = src[i + 1];
        dest.push(ALPHABET[(b0 >> 2) as usize]);
        dest.push(ALPHABET[(((b0 & 0x3) << 4) | (b1 >> 4)) as usize]);
        dest.push(ALPHABET[((b1 & 0xF) << 2) as usize]);
        dest.push(b'=');
    } else if i < src.len() {
        let b0 = src[i];
        dest.push(ALPHABET[(b0 >> 2) as usize]);
        dest.push(ALPHABET[((b0 & 0x3) << 4) as usize]);
        dest.push(b'=');
        dest.push(b'=');
    }

    dest
}

/// Decode Base64 to binary data.
///
/// Returns None if the input is invalid.
pub fn decode(src: &[u8]) -> Option<Vec<u8>> {
    if src.len() % 4 != 0 {
        return None;
    }

    // Calculate output length
    let mut out_len = (src.len() / 4) * 3;
    if !src.is_empty() && src[src.len() - 1] == b'=' {
        out_len -= 1;
    }
    if src.len() >= 2 && src[src.len() - 2] == b'=' {
        out_len -= 1;
    }

    let mut dest = Vec::with_capacity(out_len);
    let mut acc = 0u32;
    let mut acc_len = 0;

    for &c in src {
        let d = CHAR_TO_INDEX[c as usize];
        if d == 0 {
            if c == b'=' {
                break;
            }
            return None;
        }

        acc = ((acc << 6) & 0xFFF) + (d as u32 - 1);
        acc_len += 6;
        if acc_len >= 8 {
            acc_len -= 8;
            dest.push((acc >> acc_len) as u8);
        }
    }

    // Validate remaining bits
    if acc_len > 4 || (acc & ((1 << acc_len) - 1)) != 0 {
        return None;
    }

    Some(dest)
}

/// Encode a string using Base64.
///
/// Returns a newly allocated null-terminated string.
/// The caller must free the result with xfree.
///
/// # Safety
///
/// `src` must be a valid pointer to at least `src_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_base64_encode(src: *const u8, src_len: usize) -> *mut c_char {
    if src.is_null() {
        return ptr::null_mut();
    }

    let src_slice = unsafe { std::slice::from_raw_parts(src, src_len) };
    let encoded = encode(src_slice);

    // Allocate with xmalloc for C compatibility
    let out_ptr = unsafe { xmalloc(encoded.len() + 1) as *mut u8 };
    if out_ptr.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        ptr::copy_nonoverlapping(encoded.as_ptr(), out_ptr, encoded.len());
        *out_ptr.add(encoded.len()) = 0; // null terminator
    }

    out_ptr.cast()
}

/// Decode a Base64 encoded string.
///
/// Returns a newly allocated buffer (NOT null-terminated).
/// The length is written to `out_len`.
/// Returns NULL on invalid input.
///
/// # Safety
///
/// `src` must be a valid pointer to at least `src_len` bytes.
/// `out_len` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_base64_decode(
    src: *const u8,
    src_len: usize,
    out_len: *mut usize,
) -> *mut c_char {
    if src.is_null() || out_len.is_null() {
        if !out_len.is_null() {
            unsafe { *out_len = 0 };
        }
        return ptr::null_mut();
    }

    let src_slice = unsafe { std::slice::from_raw_parts(src, src_len) };

    match decode(src_slice) {
        Some(decoded) => {
            let len = decoded.len();
            let out_ptr = unsafe { xmalloc(len) as *mut u8 };
            if out_ptr.is_null() {
                unsafe { *out_len = 0 };
                return ptr::null_mut();
            }

            if len > 0 {
                unsafe {
                    ptr::copy_nonoverlapping(decoded.as_ptr(), out_ptr, len);
                }
            }

            unsafe { *out_len = len };
            out_ptr.cast()
        }
        None => {
            unsafe { *out_len = 0 };
            ptr::null_mut()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode(b""), b"");
    }

    #[test]
    fn test_encode_one_byte() {
        assert_eq!(encode(b"M"), b"TQ==");
    }

    #[test]
    fn test_encode_two_bytes() {
        assert_eq!(encode(b"Ma"), b"TWE=");
    }

    #[test]
    fn test_encode_three_bytes() {
        assert_eq!(encode(b"Man"), b"TWFu");
    }

    #[test]
    fn test_encode_hello() {
        assert_eq!(encode(b"Hello, World!"), b"SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(decode(b""), Some(vec![]));
    }

    #[test]
    fn test_decode_one_byte() {
        assert_eq!(decode(b"TQ=="), Some(b"M".to_vec()));
    }

    #[test]
    fn test_decode_two_bytes() {
        assert_eq!(decode(b"TWE="), Some(b"Ma".to_vec()));
    }

    #[test]
    fn test_decode_three_bytes() {
        assert_eq!(decode(b"TWFu"), Some(b"Man".to_vec()));
    }

    #[test]
    fn test_decode_hello() {
        assert_eq!(
            decode(b"SGVsbG8sIFdvcmxkIQ=="),
            Some(b"Hello, World!".to_vec())
        );
    }

    #[test]
    fn test_decode_invalid_length() {
        assert_eq!(decode(b"abc"), None);
    }

    #[test]
    fn test_decode_invalid_char() {
        assert_eq!(decode(b"ab!d"), None);
    }

    #[test]
    fn test_roundtrip() {
        let original = b"The quick brown fox jumps over the lazy dog.";
        let encoded = encode(original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
