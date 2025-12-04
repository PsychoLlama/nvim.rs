//! Ex command utilities for Neovim
//!
//! Provides utility functions for Ex command parsing and processing.

#![allow(unsafe_code)]

use std::ffi::c_int;

/// Check if character ends an Ex command.
///
/// Returns true if the character is one of:
/// - NUL (0) - end of string
/// - '|' - command separator
/// - '"' - start of comment
/// - '\n' - newline
///
/// These characters terminate command parsing in Ex command lines.
#[inline]
pub fn ends_excmd(c: i32) -> bool {
    c == 0 || c == b'|' as i32 || c == b'"' as i32 || c == b'\n' as i32
}

/// FFI wrapper for `ends_excmd`.
///
/// Returns 1 if the character ends an Ex command, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_ends_excmd(c: c_int) -> c_int {
    c_int::from(ends_excmd(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ends_excmd() {
        // Command terminators
        assert!(ends_excmd(0)); // NUL
        assert!(ends_excmd(b'|' as i32)); // pipe separator
        assert!(ends_excmd(b'"' as i32)); // comment start
        assert!(ends_excmd(b'\n' as i32)); // newline

        // Non-terminators
        assert!(!ends_excmd(b'a' as i32));
        assert!(!ends_excmd(b' ' as i32));
        assert!(!ends_excmd(b':' as i32));
        assert!(!ends_excmd(b'!' as i32));
        assert!(!ends_excmd(b'#' as i32));
        assert!(!ends_excmd(b'\t' as i32));
        assert!(!ends_excmd(b'\r' as i32));
    }

    #[test]
    fn test_ffi_ends_excmd() {
        assert_eq!(rs_ends_excmd(0), 1);
        assert_eq!(rs_ends_excmd(b'|' as c_int), 1);
        assert_eq!(rs_ends_excmd(b'"' as c_int), 1);
        assert_eq!(rs_ends_excmd(b'\n' as c_int), 1);
        assert_eq!(rs_ends_excmd(b'a' as c_int), 0);
    }
}
