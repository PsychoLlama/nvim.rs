//! File format handling for buffer writing
//!
//! This module provides functions for handling line endings and file format
//! conversions when writing buffers.

#![allow(clippy::naive_bytecount)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::manual_range_contains)]

use std::ffi::c_int;

use crate::FileFormat;

// =============================================================================
// Line Ending Handling
// =============================================================================

/// Count line endings in a buffer.
///
/// Returns the number of newline characters (LF) found.
#[must_use]
pub fn count_line_endings(buf: &[u8]) -> usize {
    buf.iter().filter(|&&b| b == b'\n').count()
}

/// Calculate the size needed after line ending conversion.
///
/// # Arguments
///
/// * `len` - Original length
/// * `newlines` - Number of newline characters
/// * `format` - Target file format
///
/// # Returns
///
/// The size needed for the converted buffer
#[must_use]
pub const fn converted_size(len: usize, newlines: usize, format: FileFormat) -> usize {
    match format {
        FileFormat::Unix => len,           // No change
        FileFormat::Dos => len + newlines, // LF -> CRLF adds one byte per newline
        FileFormat::Mac => len,            // LF -> CR is same size
    }
}

/// Convert line endings in place (for Mac format only).
///
/// Converts LF to CR in the buffer.
pub fn convert_to_mac_inplace(buf: &mut [u8]) {
    for byte in buf.iter_mut() {
        if *byte == b'\n' {
            *byte = b'\r';
        }
    }
}

/// Check if a byte is a newline character.
#[must_use]
pub const fn is_newline(b: u8) -> bool {
    b == b'\n' || b == b'\r'
}

/// Check if a byte sequence starts with a line ending.
///
/// Returns the length of the line ending (1 for CR or LF, 2 for CRLF, 0 for none).
#[must_use]
pub fn line_ending_at(buf: &[u8]) -> usize {
    match buf.first() {
        Some(&b'\n') => 1,
        Some(&b'\r') => {
            if buf.get(1) == Some(&b'\n') {
                2
            } else {
                1
            }
        }
        _ => 0,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Count line endings in buffer.
///
/// # Safety
///
/// `buf` must point to valid memory of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_bufwrite_count_newlines(buf: *const u8, len: usize) -> c_int {
    if buf.is_null() {
        return 0;
    }
    // SAFETY: buf is checked for null and caller guarantees len bytes
    let slice = unsafe { std::slice::from_raw_parts(buf, len) };
    count_line_endings(slice) as c_int
}

/// Calculate converted buffer size.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_converted_size(len: c_int, newlines: c_int, format: c_int) -> c_int {
    if len < 0 || newlines < 0 {
        return len;
    }
    let fmt = FileFormat::from_raw(format).unwrap_or(FileFormat::Unix);
    converted_size(len as usize, newlines as usize, fmt) as c_int
}

/// Check if byte is a newline.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_newline(b: c_int) -> c_int {
    if b < 0 || b > 255 {
        return 0;
    }
    c_int::from(is_newline(b as u8))
}

/// Get line ending length at position.
///
/// # Safety
///
/// `buf` must point to valid memory of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_bufwrite_line_ending_at(buf: *const u8, len: usize) -> c_int {
    if buf.is_null() || len == 0 {
        return 0;
    }
    // SAFETY: buf is checked for null and caller guarantees len bytes
    let slice = unsafe { std::slice::from_raw_parts(buf, len) };
    line_ending_at(slice) as c_int
}

/// Convert LF to CR in place (for Mac format).
///
/// # Safety
///
/// `buf` must point to valid mutable memory of `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_bufwrite_convert_to_mac(buf: *mut u8, len: usize) {
    if buf.is_null() {
        return;
    }
    // SAFETY: buf is checked for null and caller guarantees len bytes
    let slice = unsafe { std::slice::from_raw_parts_mut(buf, len) };
    convert_to_mac_inplace(slice);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_line_endings() {
        assert_eq!(count_line_endings(b"hello"), 0);
        assert_eq!(count_line_endings(b"hello\n"), 1);
        assert_eq!(count_line_endings(b"hello\nworld\n"), 2);
        assert_eq!(count_line_endings(b"\n\n\n"), 3);
        assert_eq!(count_line_endings(b""), 0);
    }

    #[test]
    fn test_converted_size() {
        // Unix - no change
        assert_eq!(converted_size(10, 2, FileFormat::Unix), 10);

        // DOS - adds one byte per newline
        assert_eq!(converted_size(10, 2, FileFormat::Dos), 12);
        assert_eq!(converted_size(10, 0, FileFormat::Dos), 10);

        // Mac - no change in size
        assert_eq!(converted_size(10, 2, FileFormat::Mac), 10);
    }

    #[test]
    fn test_convert_to_mac_inplace() {
        let mut buf = b"hello\nworld\n".to_vec();
        convert_to_mac_inplace(&mut buf);
        assert_eq!(buf, b"hello\rworld\r");

        let mut buf = b"no newlines".to_vec();
        convert_to_mac_inplace(&mut buf);
        assert_eq!(buf, b"no newlines");
    }

    #[test]
    fn test_is_newline() {
        assert!(is_newline(b'\n'));
        assert!(is_newline(b'\r'));
        assert!(!is_newline(b' '));
        assert!(!is_newline(b'a'));
    }

    #[test]
    fn test_line_ending_at() {
        assert_eq!(line_ending_at(b"\n"), 1);
        assert_eq!(line_ending_at(b"\r"), 1);
        assert_eq!(line_ending_at(b"\r\n"), 2);
        assert_eq!(line_ending_at(b"hello"), 0);
        assert_eq!(line_ending_at(b""), 0);
    }
}
