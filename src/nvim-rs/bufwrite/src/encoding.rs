//! Encoding conversion utilities for buffer writing
//!
//! This module provides functions for converting between different text encodings
//! when writing buffers to files.

#![allow(clippy::unreadable_literal)]
#![allow(clippy::manual_range_contains)]

use std::ffi::c_int;

use crate::{FIO_ENDIAN_L, FIO_UCS2, FIO_UCS4, FIO_UTF16};

// =============================================================================
// UCS-2/UCS-4/UTF-16 Conversion
// =============================================================================

/// Result of a Unicode to bytes conversion
#[derive(Debug, Clone, Copy)]
pub struct ConversionResult {
    /// Number of bytes written
    pub bytes_written: usize,
    /// Whether an error occurred
    pub error: bool,
}

/// Convert a Unicode codepoint to UCS-4 bytes.
///
/// # Arguments
///
/// * `c` - Unicode codepoint
/// * `little_endian` - Whether to use little endian byte order
/// * `buf` - Output buffer (must be at least 4 bytes)
///
/// # Returns
///
/// Number of bytes written (always 4 for UCS-4)
#[must_use]
pub fn ucs4_to_bytes(c: u32, little_endian: bool, buf: &mut [u8]) -> usize {
    debug_assert!(buf.len() >= 4);

    if little_endian {
        buf[0] = c as u8;
        buf[1] = (c >> 8) as u8;
        buf[2] = (c >> 16) as u8;
        buf[3] = (c >> 24) as u8;
    } else {
        buf[0] = (c >> 24) as u8;
        buf[1] = (c >> 16) as u8;
        buf[2] = (c >> 8) as u8;
        buf[3] = c as u8;
    }
    4
}

/// Convert a Unicode codepoint to UCS-2 bytes.
///
/// # Arguments
///
/// * `c` - Unicode codepoint (must be <= 0xFFFF for UCS-2)
/// * `little_endian` - Whether to use little endian byte order
/// * `buf` - Output buffer (must be at least 2 bytes)
///
/// # Returns
///
/// `ConversionResult` with bytes_written and error flag
#[must_use]
pub fn ucs2_to_bytes(c: u32, little_endian: bool, buf: &mut [u8]) -> ConversionResult {
    debug_assert!(buf.len() >= 2);

    let error = c >= 0x10000;
    let c = if error { 0xFFFD } else { c }; // Use replacement char on error

    if little_endian {
        buf[0] = c as u8;
        buf[1] = (c >> 8) as u8;
    } else {
        buf[0] = (c >> 8) as u8;
        buf[1] = c as u8;
    }

    ConversionResult {
        bytes_written: 2,
        error,
    }
}

/// Convert a Unicode codepoint to UTF-16 bytes.
///
/// # Arguments
///
/// * `c` - Unicode codepoint
/// * `little_endian` - Whether to use little endian byte order
/// * `buf` - Output buffer (must be at least 4 bytes for surrogate pairs)
///
/// # Returns
///
/// `ConversionResult` with bytes_written (2 or 4) and error flag
#[must_use]
pub fn utf16_to_bytes(c: u32, little_endian: bool, buf: &mut [u8]) -> ConversionResult {
    debug_assert!(buf.len() >= 4);

    if c < 0x10000 {
        // BMP character - single code unit
        if little_endian {
            buf[0] = c as u8;
            buf[1] = (c >> 8) as u8;
        } else {
            buf[0] = (c >> 8) as u8;
            buf[1] = c as u8;
        }
        ConversionResult {
            bytes_written: 2,
            error: false,
        }
    } else if c < 0x110000 {
        // Supplementary character - surrogate pair
        let c = c - 0x10000;
        let high = ((c >> 10) & 0x3FF) + 0xD800;
        let low = (c & 0x3FF) + 0xDC00;

        if little_endian {
            buf[0] = high as u8;
            buf[1] = (high >> 8) as u8;
            buf[2] = low as u8;
            buf[3] = (low >> 8) as u8;
        } else {
            buf[0] = (high >> 8) as u8;
            buf[1] = high as u8;
            buf[2] = (low >> 8) as u8;
            buf[3] = low as u8;
        }
        ConversionResult {
            bytes_written: 4,
            error: false,
        }
    } else {
        // Invalid codepoint
        let replacement = 0xFFFD_u32;
        if little_endian {
            buf[0] = replacement as u8;
            buf[1] = (replacement >> 8) as u8;
        } else {
            buf[0] = (replacement >> 8) as u8;
            buf[1] = replacement as u8;
        }
        ConversionResult {
            bytes_written: 2,
            error: true,
        }
    }
}

/// Convert a Unicode codepoint to Latin-1 byte.
///
/// # Arguments
///
/// * `c` - Unicode codepoint (should be <= 0xFF for valid Latin-1)
/// * `buf` - Output buffer (must be at least 1 byte)
///
/// # Returns
///
/// `ConversionResult` with bytes_written (always 1) and error flag
#[must_use]
pub fn latin1_to_bytes(c: u32, buf: &mut [u8]) -> ConversionResult {
    debug_assert!(!buf.is_empty());

    if c < 0x100 {
        buf[0] = c as u8;
        ConversionResult {
            bytes_written: 1,
            error: false,
        }
    } else {
        buf[0] = 0xBF; // Inverted question mark as replacement
        ConversionResult {
            bytes_written: 1,
            error: true,
        }
    }
}

/// Convert Unicode codepoint to bytes based on FIO flags.
///
/// # Arguments
///
/// * `c` - Unicode codepoint
/// * `flags` - FIO_ flags specifying encoding
/// * `buf` - Output buffer (must be at least 4 bytes)
///
/// # Returns
///
/// `ConversionResult` with bytes_written and error flag
#[must_use]
pub fn unicode_to_bytes(c: u32, flags: u32, buf: &mut [u8]) -> ConversionResult {
    debug_assert!(buf.len() >= 4);

    let little_endian = flags & FIO_ENDIAN_L != 0;

    if flags & FIO_UCS4 != 0 {
        let len = ucs4_to_bytes(c, little_endian, buf);
        ConversionResult {
            bytes_written: len,
            error: false,
        }
    } else if flags & FIO_UTF16 != 0 {
        utf16_to_bytes(c, little_endian, buf)
    } else if flags & FIO_UCS2 != 0 {
        ucs2_to_bytes(c, little_endian, buf)
    } else {
        latin1_to_bytes(c, buf)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Convert Unicode codepoint to bytes.
///
/// # Safety
///
/// `buf` must point to a valid buffer of at least 4 bytes.
/// `bytes_written` must be a valid pointer.
#[unsafe(no_mangle)]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_bufwrite_ucs_to_bytes(
    c: u32,
    flags: u32,
    buf: *mut u8,
    bytes_written: *mut c_int,
) -> c_int {
    if buf.is_null() || bytes_written.is_null() {
        return -1;
    }

    // SAFETY: buf is checked for null and caller guarantees at least 4 bytes
    let slice = unsafe { std::slice::from_raw_parts_mut(buf, 4) };
    let result = unicode_to_bytes(c, flags, slice);

    // SAFETY: bytes_written is checked for null above
    unsafe { *bytes_written = result.bytes_written as c_int };
    c_int::from(result.error)
}

/// Convert a Unicode character to bytes, advancing the output pointer.
///
/// Matches the C `ucs2bytes(unsigned c, char **pp, int flags)` calling convention.
///
/// # Safety
///
/// `pp` must point to a valid `*mut u8` pointer with at least 4 bytes available.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_ucs2bytes(c: u32, pp: *mut *mut u8, flags: c_int) -> c_int {
    if pp.is_null() {
        return 1; // error
    }
    let p = unsafe { *pp };
    if p.is_null() {
        return 1;
    }
    // SAFETY: caller guarantees at least 4 bytes at *pp
    let slice = unsafe { std::slice::from_raw_parts_mut(p, 4) };
    let result = unicode_to_bytes(c, flags as u32, slice);
    // Advance pointer by bytes written
    unsafe { *pp = p.add(result.bytes_written) };
    c_int::from(result.error)
}

extern "C" {
    fn nvim_bw_get_fio_flags(name: *const std::ffi::c_char) -> c_int;
}

/// Generate a BOM in `buf[4]` for encoding `name`.
///
/// Replaces C `make_bom()`. Returns the length of the BOM (0 when no BOM).
///
/// # Safety
///
/// `buf` must point to a buffer of at least 4 bytes.
/// `name` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_make_bom(buf: *mut u8, name: *const std::ffi::c_char) -> c_int {
    if buf.is_null() || name.is_null() {
        return 0;
    }
    let flags = unsafe { nvim_bw_get_fio_flags(name) };

    // Can't put a BOM in a non-Unicode file.
    if flags == crate::FIO_LATIN1 as c_int || flags == 0 {
        return 0;
    }

    if flags == crate::FIO_UTF8 as c_int {
        // UTF-8 BOM
        unsafe {
            *buf = 0xEF;
            *buf.add(1) = 0xBB;
            *buf.add(2) = 0xBF;
        }
        return 3;
    }

    // Use ucs2bytes logic for other encodings (UCS-2, UCS-4, UTF-16)
    let mut p = buf;
    let _error = unsafe { rs_ucs2bytes(0xFEFF, &raw mut p, flags) };
    unsafe { p.offset_from(buf) as c_int }
}

/// Check if codepoint needs surrogate pair in UTF-16.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_needs_surrogate(c: u32) -> c_int {
    c_int::from(c >= 0x10000 && c < 0x110000)
}

/// Get high surrogate for supplementary character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_high_surrogate(c: u32) -> u32 {
    if c >= 0x10000 {
        ((c - 0x10000) >> 10) + 0xD800
    } else {
        c
    }
}

/// Get low surrogate for supplementary character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_low_surrogate(c: u32) -> u32 {
    if c >= 0x10000 {
        ((c - 0x10000) & 0x3FF) + 0xDC00
    } else {
        c
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ucs4_little_endian() {
        let mut buf = [0u8; 4];
        let len = ucs4_to_bytes(0x12345678, true, &mut buf);
        assert_eq!(len, 4);
        assert_eq!(buf, [0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_ucs4_big_endian() {
        let mut buf = [0u8; 4];
        let len = ucs4_to_bytes(0x12345678, false, &mut buf);
        assert_eq!(len, 4);
        assert_eq!(buf, [0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_ucs2_valid() {
        let mut buf = [0u8; 2];
        let result = ucs2_to_bytes(0x1234, true, &mut buf);
        assert!(!result.error);
        assert_eq!(result.bytes_written, 2);
        assert_eq!(buf, [0x34, 0x12]);
    }

    #[test]
    fn test_ucs2_invalid() {
        let mut buf = [0u8; 2];
        let result = ucs2_to_bytes(0x10000, true, &mut buf);
        assert!(result.error);
        assert_eq!(result.bytes_written, 2);
    }

    #[test]
    fn test_utf16_bmp() {
        let mut buf = [0u8; 4];
        let result = utf16_to_bytes(0x0041, true, &mut buf); // 'A'
        assert!(!result.error);
        assert_eq!(result.bytes_written, 2);
        assert_eq!(&buf[..2], &[0x41, 0x00]);
    }

    #[test]
    fn test_utf16_surrogate_pair() {
        let mut buf = [0u8; 4];
        let result = utf16_to_bytes(0x1F600, true, &mut buf); // Emoji
        assert!(!result.error);
        assert_eq!(result.bytes_written, 4);
        // 0x1F600 -> high: 0xD83D, low: 0xDE00
        assert_eq!(buf, [0x3D, 0xD8, 0x00, 0xDE]);
    }

    #[test]
    fn test_utf16_invalid() {
        let mut buf = [0u8; 4];
        let result = utf16_to_bytes(0x110000, true, &mut buf); // Invalid
        assert!(result.error);
    }

    #[test]
    fn test_latin1_valid() {
        let mut buf = [0u8; 1];
        let result = latin1_to_bytes(0x41, &mut buf); // 'A'
        assert!(!result.error);
        assert_eq!(buf[0], 0x41);
    }

    #[test]
    fn test_latin1_invalid() {
        let mut buf = [0u8; 1];
        let result = latin1_to_bytes(0x100, &mut buf); // Out of range
        assert!(result.error);
        assert_eq!(buf[0], 0xBF);
    }

    #[test]
    fn test_rs_ucs2bytes_ucs4_le() {
        let mut buf = [0u8; 4];
        let mut p = buf.as_mut_ptr();
        let original = p;
        let error = unsafe {
            rs_ucs2bytes(
                0x41,
                &raw mut p,
                (crate::FIO_UCS4 | crate::FIO_ENDIAN_L) as c_int,
            )
        };
        assert_eq!(error, 0);
        assert_eq!(unsafe { p.offset_from(original) }, 4);
        assert_eq!(buf, [0x41, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_rs_ucs2bytes_ucs2_be() {
        let mut buf = [0u8; 4];
        let mut p = buf.as_mut_ptr();
        let original = p;
        let error = unsafe { rs_ucs2bytes(0x1234, &raw mut p, crate::FIO_UCS2 as c_int) };
        assert_eq!(error, 0);
        assert_eq!(unsafe { p.offset_from(original) }, 2);
        assert_eq!(&buf[..2], &[0x12, 0x34]);
    }

    #[test]
    fn test_rs_ucs2bytes_latin1_error() {
        let mut buf = [0u8; 4];
        let mut p = buf.as_mut_ptr();
        let error = unsafe { rs_ucs2bytes(0x100, &raw mut p, crate::FIO_LATIN1 as c_int) };
        assert_eq!(error, 1); // error
        assert_eq!(buf[0], 0xBF);
    }

    #[test]
    fn test_surrogates() {
        // Test emoji: 😀 (U+1F600)
        let c = 0x1F600_u32;
        assert_eq!(rs_bufwrite_needs_surrogate(c), 1);
        assert_eq!(rs_bufwrite_high_surrogate(c), 0xD83D);
        assert_eq!(rs_bufwrite_low_surrogate(c), 0xDE00);

        // BMP character
        let c = 0x0041_u32; // 'A'
        assert_eq!(rs_bufwrite_needs_surrogate(c), 0);
    }
}
