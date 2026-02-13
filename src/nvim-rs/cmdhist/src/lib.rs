//! Command history utilities for Neovim
//!
//! This module provides Rust implementations of the history functions from
//! `src/nvim/cmdhist.c`. These are pure functions with no external dependencies.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::os::raw::c_int;

pub mod command;
pub mod delete;
pub mod ffi;
pub mod helpers;
pub mod iter;
pub mod modify;
pub mod viml;

/// Number of history tables (HIST_CMD through HIST_DEBUG).
pub const HIST_COUNT: c_int = 5;

/// VimL typval type constants (verified by _Static_assert in C).
pub const VAR_UNKNOWN: c_int = 0;
pub const VAR_NUMBER: c_int = 1;
pub const VAR_STRING: c_int = 2;

/// Buffer length for number-to-string conversions.
pub const NUMBUFLEN: usize = 65;

/// Return the length of the history tables.
///
/// # Safety
/// Calls C accessor function for `hislen`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_hislen() -> c_int {
    ffi::nvim_get_hislen()
}

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

    #[test]
    fn test_history_type_constants() {
        // Verify history type constants match C definitions
        assert_eq!(HIST_DEFAULT, -2);
        assert_eq!(HIST_INVALID, -1);
        assert_eq!(HIST_CMD, 0);
        assert_eq!(HIST_SEARCH, 1);
        assert_eq!(HIST_EXPR, 2);
        assert_eq!(HIST_INPUT, 3);
        assert_eq!(HIST_DEBUG, 4);
    }

    #[test]
    fn test_history_types_sequential() {
        // Valid history types should be sequential 0-4
        let valid_types = [HIST_CMD, HIST_SEARCH, HIST_EXPR, HIST_INPUT, HIST_DEBUG];
        for (i, &hist_type) in valid_types.iter().enumerate() {
            assert_eq!(hist_type, i as c_int);
        }
    }

    #[test]
    fn test_special_history_types() {
        // HIST_DEFAULT and HIST_INVALID should be distinct from valid types
        let special_types = [HIST_DEFAULT, HIST_INVALID];
        let valid_types = [HIST_CMD, HIST_SEARCH, HIST_EXPR, HIST_INPUT, HIST_DEBUG];

        for special in special_types {
            for valid in valid_types {
                assert_ne!(special, valid, "Special type should differ from valid type");
            }
        }
    }

    #[test]
    fn test_hist_type_count() {
        // There should be exactly 5 valid history types (0-4)
        let valid_types = [HIST_CMD, HIST_SEARCH, HIST_EXPR, HIST_INPUT, HIST_DEBUG];
        assert_eq!(valid_types.len(), 5);
        assert_eq!(HIST_DEBUG - HIST_CMD + 1, 5);
    }

    #[test]
    fn test_hist_count_constant() {
        assert_eq!(HIST_COUNT, 5);
        assert_eq!(HIST_COUNT, HIST_DEBUG + 1);
    }
}
