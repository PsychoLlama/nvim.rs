//! Grid utilities for Neovim
//!
//! This crate provides Rust implementations of grid/screen character functions
//! from `src/nvim/grid.c`.

#![allow(unsafe_code)] // FFI requires unsafe

/// Type alias for screen character (matches C's `schar_T` which is `uint32_t`).
type ScharT = u32;

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
}
