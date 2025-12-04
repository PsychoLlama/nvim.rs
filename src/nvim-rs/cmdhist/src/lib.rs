//! Command history utilities for Neovim
//!
//! This module provides Rust implementations of the history functions from
//! `src/nvim/cmdhist.c`. These are pure functions with no external dependencies.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::os::raw::c_int;

/// History type constants (matching nvim's HistoryType enum)
pub const HIST_DEFAULT: c_int = -2;
pub const HIST_INVALID: c_int = -1;
pub const HIST_CMD: c_int = 0;
pub const HIST_SEARCH: c_int = 1;
pub const HIST_EXPR: c_int = 2;
pub const HIST_INPUT: c_int = 3;
pub const HIST_DEBUG: c_int = 4;

/// Translate a history character to the associated type number.
///
/// Maps history prefix characters to their corresponding history type:
/// - ':' -> HIST_CMD (command history)
/// - '=' -> HIST_EXPR (expression history)
/// - '@' -> HIST_INPUT (input history)
/// - '>' -> HIST_DEBUG (debug history)
/// - NUL, '/', '?' -> HIST_SEARCH (search history)
/// - other -> HIST_INVALID
#[no_mangle]
pub extern "C" fn rs_hist_char2type(c: c_int) -> c_int {
    match c as u8 as char {
        ':' => HIST_CMD,
        '=' => HIST_EXPR,
        '@' => HIST_INPUT,
        '>' => HIST_DEBUG,
        '\0' | '/' | '?' => HIST_SEARCH,
        _ => HIST_INVALID,
    }
}

/// Translate a history type number to the associated character.
///
/// Maps history types to their corresponding prefix characters:
/// - HIST_CMD -> ':'
/// - HIST_SEARCH -> '/'
/// - HIST_EXPR -> '='
/// - HIST_INPUT -> '@'
/// - HIST_DEBUG -> '>'
///
/// # Panics
/// Panics if an invalid history type is passed.
#[no_mangle]
pub extern "C" fn rs_hist_type2char(hist_type: c_int) -> c_int {
    match hist_type {
        HIST_CMD => b':' as c_int,
        HIST_SEARCH => b'/' as c_int,
        HIST_EXPR => b'=' as c_int,
        HIST_INPUT => b'@' as c_int,
        HIST_DEBUG => b'>' as c_int,
        _ => panic!("Invalid history type: {hist_type}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hist_char2type() {
        // Command history
        assert_eq!(rs_hist_char2type(':' as c_int), HIST_CMD);

        // Expression history
        assert_eq!(rs_hist_char2type('=' as c_int), HIST_EXPR);

        // Input history
        assert_eq!(rs_hist_char2type('@' as c_int), HIST_INPUT);

        // Debug history
        assert_eq!(rs_hist_char2type('>' as c_int), HIST_DEBUG);

        // Search history
        assert_eq!(rs_hist_char2type('\0' as c_int), HIST_SEARCH);
        assert_eq!(rs_hist_char2type('/' as c_int), HIST_SEARCH);
        assert_eq!(rs_hist_char2type('?' as c_int), HIST_SEARCH);

        // Invalid
        assert_eq!(rs_hist_char2type('a' as c_int), HIST_INVALID);
        assert_eq!(rs_hist_char2type('!' as c_int), HIST_INVALID);
        assert_eq!(rs_hist_char2type(' ' as c_int), HIST_INVALID);
    }

    #[test]
    fn test_hist_type2char() {
        // Command history
        assert_eq!(rs_hist_type2char(HIST_CMD), b':' as c_int);

        // Search history
        assert_eq!(rs_hist_type2char(HIST_SEARCH), b'/' as c_int);

        // Expression history
        assert_eq!(rs_hist_type2char(HIST_EXPR), b'=' as c_int);

        // Input history
        assert_eq!(rs_hist_type2char(HIST_INPUT), b'@' as c_int);

        // Debug history
        assert_eq!(rs_hist_type2char(HIST_DEBUG), b'>' as c_int);
    }

    #[test]
    fn test_roundtrip() {
        // Test that char2type and type2char are inverses for valid types
        for hist_type in [HIST_CMD, HIST_SEARCH, HIST_EXPR, HIST_INPUT, HIST_DEBUG] {
            let c = rs_hist_type2char(hist_type);
            assert_eq!(rs_hist_char2type(c), hist_type);
        }
    }
}
