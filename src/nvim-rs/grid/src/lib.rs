//! Grid utilities for Neovim
//!
//! This crate provides Rust implementations of grid/screen character functions
//! from `src/nvim/grid.c`.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::c_int;

/// Type alias for screen character (matches C's `schar_T` which is `uint32_t`).
type ScharT = u32;

/// Unicode replacement character
const REPLACEMENT_CHAR: i32 = 0xFFFD;

/// Check if a screen character is stored in the high (cache) format.
///
/// Returns true if the schar uses the glyph cache format (high byte is 0xFF).
/// The format depends on endianness:
/// - Big endian: high bit is in first byte position
/// - Little endian: high bit is in last byte position (position 0 in memory)
#[inline]
const fn schar_high_impl(sc: ScharT) -> bool {
    // On little-endian systems (most common), check if lowest byte is 0xFF
    // On big-endian systems, check if highest byte is 0xFF
    #[cfg(target_endian = "big")]
    {
        (sc & 0xFF00_0000) == 0xFF00_0000
    }
    #[cfg(target_endian = "little")]
    {
        (sc & 0xFF) == 0xFF
    }
}

/// FFI wrapper for `schar_high`.
///
/// Check if a screen character is stored in the high (cache) format.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_schar_high(sc: ScharT) -> bool {
    schar_high_impl(sc)
}

/// Get ASCII character from an schar, or NUL if not ASCII.
///
/// Returns the ASCII character if the schar represents a single ASCII byte,
/// otherwise returns NUL (0). The check and extraction depend on endianness.
#[inline]
#[allow(clippy::cast_possible_truncation)] // intentionally truncating to char
const fn schar_get_ascii_impl(sc: ScharT) -> i8 {
    #[cfg(target_endian = "big")]
    {
        // On big-endian: check if lower 3 bytes are 0 and high byte < 0x80
        if (sc & 0x80FF_FFFF) == 0 {
            // Extract the high byte as ASCII char
            (sc >> 24) as i8
        } else {
            0
        }
    }
    #[cfg(target_endian = "little")]
    {
        // On little-endian: check if value < 0x80 (fits in one ASCII byte)
        if sc < 0x80 {
            sc as i8
        } else {
            0
        }
    }
}

/// FFI wrapper for `schar_get_ascii`.
///
/// Get ASCII character from an schar, or NUL if not ASCII.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_schar_get_ascii(sc: ScharT) -> i8 {
    schar_get_ascii_impl(sc)
}

/// Put a unicode character in a screen cell.
///
/// Converts a Unicode codepoint to an `schar_T` by encoding it as UTF-8
/// and storing the bytes in the u32 value.
/// Characters >= 0x200000 are replaced with the Unicode replacement character (0xFFFD).
#[inline]
fn schar_from_char_impl(c: c_int) -> ScharT {
    let c = if c >= 0x20_0000 { REPLACEMENT_CHAR } else { c };

    // Write UTF-8 bytes into a buffer
    let mut buf = [0u8; 4];
    nvim_mbyte::utf_char2bytes(c, &mut buf);

    // Convert the buffer to schar_T (native endianness)
    // On little-endian: first UTF-8 byte goes to lowest byte of u32
    // On big-endian: first UTF-8 byte goes to highest byte of u32
    ScharT::from_ne_bytes(buf)
}

/// FFI wrapper for `schar_from_char`.
///
/// Put a unicode character in a screen cell.
#[no_mangle]
pub extern "C" fn rs_schar_from_char(c: c_int) -> ScharT {
    schar_from_char_impl(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schar_high_true() {
        // Test values that should return true (cache format)
        #[cfg(target_endian = "little")]
        {
            assert!(schar_high_impl(0x0000_00FF)); // Lowest byte is 0xFF
            assert!(schar_high_impl(0x1234_56FF)); // High bytes set, lowest is 0xFF
            assert!(schar_high_impl(0xFFFF_FFFF)); // All bytes are 0xFF
        }
        #[cfg(target_endian = "big")]
        {
            assert!(schar_high_impl(0xFF00_0000)); // Highest byte is 0xFF
            assert!(schar_high_impl(0xFF12_3456)); // Low bytes set, highest is 0xFF
            assert!(schar_high_impl(0xFFFF_FFFF)); // All bytes are 0xFF
        }
    }

    #[test]
    fn test_schar_high_false() {
        // Test values that should return false (inline format)
        #[cfg(target_endian = "little")]
        {
            assert!(!schar_high_impl(0x0000_0000)); // All zeros
            assert!(!schar_high_impl(0x0000_0041)); // ASCII 'A'
            assert!(!schar_high_impl(0xFFFF_FF00)); // High bytes 0xFF but lowest is 0
            assert!(!schar_high_impl(0x0000_00FE)); // Close to 0xFF but not quite
        }
        #[cfg(target_endian = "big")]
        {
            assert!(!schar_high_impl(0x0000_0000)); // All zeros
            assert!(!schar_high_impl(0x4100_0000)); // ASCII 'A'
            assert!(!schar_high_impl(0x00FF_FFFF)); // Low bytes 0xFF but highest is 0
            assert!(!schar_high_impl(0xFE00_0000)); // Close to 0xFF but not quite
        }
    }

    #[test]
    fn test_ffi_schar_high() {
        #[cfg(target_endian = "little")]
        {
            assert!(rs_schar_high(0x0000_00FF));
            assert!(!rs_schar_high(0x0000_0041));
        }
        #[cfg(target_endian = "big")]
        {
            assert!(rs_schar_high(0xFF00_0000));
            assert!(!rs_schar_high(0x4100_0000));
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)] // b'A' as i8 is safe for ASCII
    fn test_schar_get_ascii_valid() {
        // Test valid ASCII characters
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_get_ascii_impl(0x0000_0041), b'A' as i8); // 'A'
            assert_eq!(schar_get_ascii_impl(0x0000_0061), b'a' as i8); // 'a'
            assert_eq!(schar_get_ascii_impl(0x0000_0020), b' ' as i8); // space
            assert_eq!(schar_get_ascii_impl(0x0000_007F), 0x7F_i8); // DEL (max ASCII)
            assert_eq!(schar_get_ascii_impl(0x0000_0000), 0); // NUL
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_get_ascii_impl(0x4100_0000), b'A' as i8); // 'A'
            assert_eq!(schar_get_ascii_impl(0x6100_0000), b'a' as i8); // 'a'
            assert_eq!(schar_get_ascii_impl(0x2000_0000), b' ' as i8); // space
            assert_eq!(schar_get_ascii_impl(0x7F00_0000), 0x7F_i8); // DEL (max ASCII)
            assert_eq!(schar_get_ascii_impl(0x0000_0000), 0); // NUL
        }
    }

    #[test]
    fn test_schar_get_ascii_invalid() {
        // Test non-ASCII characters return NUL
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_get_ascii_impl(0x0000_0080), 0); // >= 0x80 is not ASCII
            assert_eq!(schar_get_ascii_impl(0x0000_00FF), 0); // 0xFF is not ASCII
            assert_eq!(schar_get_ascii_impl(0x1234_0041), 0); // Multi-byte, not pure ASCII
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_get_ascii_impl(0x8000_0000), 0); // >= 0x80 is not ASCII
            assert_eq!(schar_get_ascii_impl(0xFF00_0000), 0); // 0xFF is not ASCII
            assert_eq!(schar_get_ascii_impl(0x4100_0001), 0); // Multi-byte, not pure ASCII
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)] // b'A' as i8 is safe for ASCII
    fn test_ffi_schar_get_ascii() {
        #[cfg(target_endian = "little")]
        {
            assert_eq!(rs_schar_get_ascii(0x0000_0041), b'A' as i8);
            assert_eq!(rs_schar_get_ascii(0x0000_0080), 0);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(rs_schar_get_ascii(0x4100_0000), b'A' as i8);
            assert_eq!(rs_schar_get_ascii(0x8000_0000), 0);
        }
    }

    #[test]
    fn test_schar_from_char_ascii() {
        // ASCII characters should be stored in the first byte
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_from_char_impl(b'A' as c_int), 0x0000_0041);
            assert_eq!(schar_from_char_impl(b'a' as c_int), 0x0000_0061);
            assert_eq!(schar_from_char_impl(b' ' as c_int), 0x0000_0020);
            assert_eq!(schar_from_char_impl(0), 0x0000_0000);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_from_char_impl(b'A' as c_int), 0x4100_0000);
            assert_eq!(schar_from_char_impl(b'a' as c_int), 0x6100_0000);
            assert_eq!(schar_from_char_impl(b' ' as c_int), 0x2000_0000);
            assert_eq!(schar_from_char_impl(0), 0x0000_0000);
        }
    }

    #[test]
    fn test_schar_from_char_multibyte() {
        // 2-byte UTF-8: U+00E9 (é) = 0xC3 0xA9
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_from_char_impl(0x00E9), 0x0000_A9C3);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_from_char_impl(0x00E9), 0xC3A9_0000);
        }

        // 3-byte UTF-8: U+4E2D (中) = 0xE4 0xB8 0xAD
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_from_char_impl(0x4E2D), 0x00AD_B8E4);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_from_char_impl(0x4E2D), 0xE4B8_AD00);
        }

        // 4-byte UTF-8: U+1F600 (😀) = 0xF0 0x9F 0x98 0x80
        #[cfg(target_endian = "little")]
        {
            assert_eq!(schar_from_char_impl(0x1_F600), 0x8098_9FF0);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(schar_from_char_impl(0x1_F600), 0xF09F_9880);
        }
    }

    #[test]
    fn test_schar_from_char_replacement() {
        // Characters >= 0x200000 should be replaced with U+FFFD
        // U+FFFD = 0xEF 0xBF 0xBD
        #[cfg(target_endian = "little")]
        {
            let replacement = 0x00BD_BFEF_u32;
            assert_eq!(schar_from_char_impl(0x20_0000), replacement);
            assert_eq!(schar_from_char_impl(0x7FFF_FFFF), replacement);
        }
        #[cfg(target_endian = "big")]
        {
            let replacement = 0xEFBF_BD00_u32;
            assert_eq!(schar_from_char_impl(0x20_0000), replacement);
            assert_eq!(schar_from_char_impl(0x7FFF_FFFF), replacement);
        }
    }

    #[test]
    fn test_ffi_schar_from_char() {
        // Just verify the FFI wrapper works
        #[cfg(target_endian = "little")]
        {
            assert_eq!(rs_schar_from_char(b'A' as c_int), 0x0000_0041);
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(rs_schar_from_char(b'A' as c_int), 0x4100_0000);
        }
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_schar_roundtrip() {
        // Verify that schar_from_char and schar_get_ascii are consistent for ASCII
        for c in 0..0x80 {
            let sc = schar_from_char_impl(c);
            assert_eq!(schar_get_ascii_impl(sc), c as i8);
        }
    }
}
