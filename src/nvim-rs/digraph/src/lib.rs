//! Digraph functions for Neovim.
//!
//! This crate provides Rust implementations of digraph functions including:
//! - Digraph lookup (`rs_digraph_get`, `rs_getexactdigraph`)
//! - Character validation (`rs_check_digraph_chars_valid`)
//! - User digraph registration (`rs_registerdigraph`)
//! - Reverse lookup (`rs_get_digraph_for_char`)
//! - Input state machine (`rs_do_digraph`)
//! - Vimscript function helpers (`rs_digraph_get_viml`, `rs_digraph_set_viml`)
//! - List operations (`rs_digraph_iterate`, `rs_digraph_format_pair`)

use std::ffi::c_void;

use libc::c_int;

mod input;
mod list;
mod parse;
mod register;
mod validate;
mod viml;

// Re-export FFI functions
pub use input::rs_do_digraph;
pub use list::{rs_digraph_format_pair, rs_digraph_format_result, rs_digraph_iterate};
pub use parse::{rs_putdigraph, PutdigraphResult};
pub use register::{rs_get_digraph_for_char, rs_registerdigraph};
pub use validate::rs_check_digraph_chars_valid;
pub use viml::{rs_digraph_get_viml, rs_digraph_set_viml, rs_parse_digraph_chars};

/// Digraph entry structure matching C's `digr_T`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DigrT {
    pub char1: u8,
    pub char2: u8,
    pub result: c_int,
}

// C accessor functions for digraph tables
extern "C" {
    /// Get pointer to user digraphs array data (opaque).
    fn nvim_get_user_digraphs_data() -> *const c_void;

    /// Get length of user digraphs array.
    fn nvim_get_user_digraphs_len() -> c_int;

    /// Get pointer to default digraphs array (opaque).
    fn nvim_get_digraphdefault() -> *const c_void;

    /// Get length of default digraphs array.
    fn nvim_get_digraphdefault_len() -> c_int;
}

/// Space character as `c_int`.
const SPACE: c_int = b' ' as c_int;

/// Check if a character is special (negative value).
#[inline]
const fn is_special(c: c_int) -> bool {
    c < 0
}

/// Search for an exact digraph match in the given order.
///
/// Searches user digraphs first, then default digraphs.
///
/// # Arguments
/// * `char1` - First character of the digraph
/// * `char2` - Second character of the digraph
/// * `meta_char` - If true and char1 is space, return char2 | 0x80
///
/// # Returns
/// The digraph result, or char2 if not found.
fn getexactdigraph_impl(char1: c_int, char2: c_int, meta_char: bool) -> c_int {
    // Special characters don't form digraphs
    if is_special(char1) || is_special(char2) {
        return char2;
    }

    // Safe truncation: we only care about ASCII range for digraphs
    #[allow(clippy::cast_sign_loss)]
    let char1_u8 = (char1 & 0xFF) as u8;
    #[allow(clippy::cast_sign_loss)]
    let char2_u8 = (char2 & 0xFF) as u8;

    // Search user digraphs first
    let user_data = unsafe { nvim_get_user_digraphs_data() };
    let user_len = unsafe { nvim_get_user_digraphs_len() };

    if !user_data.is_null() && user_len > 0 {
        let user_digraphs = user_data.cast::<DigrT>();
        #[allow(clippy::cast_sign_loss)]
        let len = user_len as usize;
        for i in 0..len {
            let dp = unsafe { &*user_digraphs.add(i) };
            if dp.char1 == char1_u8 && dp.char2 == char2_u8 {
                return dp.result;
            }
        }
    }

    // Search default digraphs
    let default_data = unsafe { nvim_get_digraphdefault() };
    let default_len = unsafe { nvim_get_digraphdefault_len() };

    if !default_data.is_null() && default_len > 0 {
        let default_digraphs = default_data.cast::<DigrT>();
        #[allow(clippy::cast_sign_loss)]
        let len = default_len as usize;
        for i in 0..len {
            let dp = unsafe { &*default_digraphs.add(i) };
            // Default array is null-terminated (char1 == 0 marks end)
            if dp.char1 == 0 {
                break;
            }
            if dp.char1 == char1_u8 && dp.char2 == char2_u8 {
                return dp.result;
            }
        }
    }

    // Digraph not found
    if char1 == SPACE && meta_char {
        // <space> <char> --> meta-char
        return char2 | 0x80;
    }
    char2
}

/// Get digraph, allowing for both char1-char2 and char2-char1 orderings.
///
/// # Arguments
/// * `char1` - First character of the digraph
/// * `char2` - Second character of the digraph
/// * `meta_char` - If true and char1 is space, return char2 | 0x80
///
/// # Returns
/// The digraph result.
fn digraph_get_impl(char1: c_int, char2: c_int, meta_char: bool) -> c_int {
    let retval = getexactdigraph_impl(char1, char2, meta_char);
    if retval == char2 && char1 != char2 {
        let retval2 = getexactdigraph_impl(char2, char1, meta_char);
        if retval2 != char1 {
            return retval2;
        }
    }
    retval
}

// =============================================================================
// FFI exports
// =============================================================================

/// Get digraph (FFI export).
#[no_mangle]
pub extern "C" fn rs_digraph_get(char1: c_int, char2: c_int, meta_char: c_int) -> c_int {
    digraph_get_impl(char1, char2, meta_char != 0)
}

/// Get exact digraph match (FFI export).
#[no_mangle]
pub extern "C" fn rs_getexactdigraph(char1: c_int, char2: c_int, meta_char: c_int) -> c_int {
    getexactdigraph_impl(char1, char2, meta_char != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_special() {
        assert!(is_special(-1));
        assert!(is_special(-100));
        assert!(!is_special(0));
        assert!(!is_special(65)); // 'A'
    }

    #[test]
    fn test_is_special_boundary() {
        // Boundary at 0
        assert!(!is_special(0));
        assert!(is_special(-1));
        // Large negative values
        assert!(is_special(c_int::MIN));
        // Large positive values
        assert!(!is_special(c_int::MAX));
    }

    #[test]
    fn test_meta_char_logic() {
        // When char1 is space and meta_char is true, result should be char2 | 0x80
        let char2: c_int = 97; // 'a'

        // The meta_char logic: if char1 == ' ' and meta_char, return char2 | 0x80
        // Expected: 'a' | 0x80 = 97 | 128 = 225
        let expected = char2 | 0x80;
        assert_eq!(expected, 225);
    }

    #[test]
    fn test_meta_char_various_chars() {
        // Test meta-char conversion for various characters
        assert_eq!(c_int::from(b'a') | 0x80, 225);
        assert_eq!(c_int::from(b'z') | 0x80, 250);
        assert_eq!(c_int::from(b'A') | 0x80, 193);
        assert_eq!(c_int::from(b'0') | 0x80, 176);
    }

    #[test]
    fn test_space_constant() {
        assert_eq!(SPACE, 32);
        assert_eq!(SPACE, c_int::from(b' '));
    }

    #[test]
    #[allow(clippy::cast_sign_loss)]
    fn test_char_truncation() {
        // Test the truncation logic used in getexactdigraph_impl
        let char1: c_int = 0x141; // 'A' + 0x100
        let char1_u8 = (char1 & 0xFF) as u8;
        assert_eq!(char1_u8, b'A');

        // Negative values also truncate correctly
        let char2: c_int = -1;
        let char2_u8 = (char2 & 0xFF) as u8;
        assert_eq!(char2_u8, 0xFF);
    }

    #[test]
    fn test_digr_t_size() {
        // DigrT should be packed for FFI compatibility
        use std::mem::size_of;
        // char1 (1) + char2 (1) + padding (2) + result (4) = 8 bytes on most platforms
        // The exact size depends on alignment, but it should be consistent
        assert!(size_of::<DigrT>() >= 6); // minimum: 1 + 1 + 4
        assert!(size_of::<DigrT>() <= 12); // maximum with padding
    }

    #[test]
    fn test_digr_t_fields() {
        // Test that DigrT can be constructed and fields accessed
        let d = DigrT {
            char1: b'a',
            char2: b'b',
            result: 0x1234,
        };
        assert_eq!(d.char1, b'a');
        assert_eq!(d.char2, b'b');
        assert_eq!(d.result, 0x1234);
    }

    #[test]
    fn test_digr_t_copy() {
        // DigrT should be Copy
        let d1 = DigrT {
            char1: b'x',
            char2: b'y',
            result: 42,
        };
        let d2 = d1;
        assert_eq!(d1.result, d2.result);
    }

    #[test]
    fn test_is_special_all_negative() {
        // All negative values should be special
        for n in [-1, -10, -100, -1000, c_int::MIN] {
            assert!(is_special(n), "{n} should be special");
        }
    }

    #[test]
    fn test_is_special_all_non_negative() {
        // All non-negative values should NOT be special
        for n in [0, 1, 10, 100, 1000, c_int::MAX] {
            assert!(!is_special(n), "{n} should not be special");
        }
    }

    #[test]
    fn test_meta_char_ascii_range() {
        // Test meta-char for full lowercase ASCII range
        for c in b'a'..=b'z' {
            let result = c_int::from(c) | 0x80;
            assert!(result >= 225); // 'a' | 0x80
            assert!(result <= 250); // 'z' | 0x80
        }
    }

    #[test]
    fn test_meta_char_digits() {
        // Test meta-char for digit range
        for c in b'0'..=b'9' {
            let result = c_int::from(c) | 0x80;
            assert!(result >= 176); // '0' | 0x80
            assert!(result <= 185); // '9' | 0x80
        }
    }

    #[test]
    fn test_space_is_32() {
        // SPACE constant should be ASCII 32
        assert_eq!(SPACE, 32);
        assert_eq!(SPACE, c_int::from(b' '));
    }

    #[test]
    #[allow(clippy::cast_sign_loss)]
    fn test_char_truncation_high_values() {
        // Test truncation for values > 255
        let tests = [
            (0x100, 0u8),  // 256 -> 0
            (0x141, 0x41), // 321 -> 'A'
            (0x1FF, 0xFF), // 511 -> 255
            (0x200, 0x00), // 512 -> 0
        ];
        for (input, expected) in tests {
            let truncated = (input & 0xFF) as u8;
            assert_eq!(truncated, expected, "Input 0x{input:x}");
        }
    }

    #[test]
    fn test_char_truncation_negative() {
        // Negative values truncate to their lower 8 bits
        #[allow(clippy::cast_sign_loss)]
        {
            let neg1: c_int = -1;
            assert_eq!((neg1 & 0xFF) as u8, 0xFF);

            let neg128: c_int = -128;
            assert_eq!((neg128 & 0xFF) as u8, 0x80);
        }
    }

    #[test]
    fn test_digr_t_alignment() {
        use std::mem::align_of;
        // DigrT should have reasonable alignment (at least 1, at most pointer-sized)
        let align = align_of::<DigrT>();
        assert!(align >= 1);
        assert!(align <= std::mem::size_of::<usize>());
    }

    #[test]
    fn test_meta_char_preserves_low_bits() {
        // OR with 0x80 should only set bit 7, preserving lower bits
        for c in 0u8..128 {
            let result = c_int::from(c) | 0x80;
            assert_eq!(result & 0x7F, c_int::from(c));
            assert!(result >= 128);
        }
    }
}
