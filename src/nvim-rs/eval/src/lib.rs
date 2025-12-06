//! Evaluation utilities for Neovim
//!
//! This crate provides functions for evaluating VimL/Lua expressions,
//! including character validation for variable and function names.

use std::ffi::c_int;

/// The autoload character used in function/variable names.
const AUTOLOAD_CHAR: u8 = b'#';

/// Check if a character is an ASCII uppercase letter (A-Z).
#[inline]
const fn ascii_isupper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// Check if a character is an ASCII lowercase letter (a-z).
#[inline]
const fn ascii_islower(c: u8) -> bool {
    c >= b'a' && c <= b'z'
}

/// Check if a character is an ASCII letter (A-Z or a-z).
#[inline]
const fn ascii_isalpha(c: u8) -> bool {
    ascii_isupper(c) || ascii_islower(c)
}

/// Check if a character is an ASCII digit (0-9).
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Check if a character is an ASCII alphanumeric character (A-Z, a-z, 0-9).
#[inline]
const fn ascii_isalnum(c: u8) -> bool {
    ascii_isalpha(c) || ascii_isdigit(c)
}

/// Check if character `c` can be used in a variable or function name.
/// Does not include '{' or '}' for magic braces.
///
/// Valid characters: alphanumeric, underscore, colon, or autoload char (#).
#[no_mangle]
pub extern "C" fn rs_eval_isnamec(c: c_int) -> bool {
    let Ok(c) = u8::try_from(c) else {
        return false;
    };
    ascii_isalnum(c) || c == b'_' || c == b':' || c == AUTOLOAD_CHAR
}

/// Check if character `c` can be used as the first character in a
/// variable or function name (excluding '{' and '}').
///
/// Valid first characters: alphabetic or underscore.
#[no_mangle]
pub extern "C" fn rs_eval_isnamec1(c: c_int) -> bool {
    let Ok(c) = u8::try_from(c) else {
        return false;
    };
    ascii_isalpha(c) || c == b'_'
}

/// Check if character `c` can be used as the first character of a
/// dictionary key.
///
/// Valid dictionary key characters: alphanumeric or underscore.
#[no_mangle]
pub extern "C" fn rs_eval_isdictc(c: c_int) -> bool {
    let Ok(c) = u8::try_from(c) else {
        return false;
    };
    ascii_isalnum(c) || c == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_isnamec() {
        // Alphanumeric
        assert!(rs_eval_isnamec(c_int::from(b'a')));
        assert!(rs_eval_isnamec(c_int::from(b'Z')));
        assert!(rs_eval_isnamec(c_int::from(b'0')));
        assert!(rs_eval_isnamec(c_int::from(b'9')));

        // Special allowed characters
        assert!(rs_eval_isnamec(c_int::from(b'_')));
        assert!(rs_eval_isnamec(c_int::from(b':')));
        assert!(rs_eval_isnamec(c_int::from(b'#'))); // AUTOLOAD_CHAR

        // Not allowed
        assert!(!rs_eval_isnamec(c_int::from(b'{')));
        assert!(!rs_eval_isnamec(c_int::from(b'}')));
        assert!(!rs_eval_isnamec(c_int::from(b' ')));
        assert!(!rs_eval_isnamec(c_int::from(b'.')));
        assert!(!rs_eval_isnamec(-1));
        assert!(!rs_eval_isnamec(256));
    }

    #[test]
    fn test_eval_isnamec1() {
        // Alphabetic
        assert!(rs_eval_isnamec1(c_int::from(b'a')));
        assert!(rs_eval_isnamec1(c_int::from(b'Z')));
        assert!(rs_eval_isnamec1(c_int::from(b'_')));

        // Not allowed as first char
        assert!(!rs_eval_isnamec1(c_int::from(b'0')));
        assert!(!rs_eval_isnamec1(c_int::from(b':')));
        assert!(!rs_eval_isnamec1(c_int::from(b'#')));
        assert!(!rs_eval_isnamec1(-1));
    }

    #[test]
    fn test_eval_isdictc() {
        // Alphanumeric
        assert!(rs_eval_isdictc(c_int::from(b'a')));
        assert!(rs_eval_isdictc(c_int::from(b'Z')));
        assert!(rs_eval_isdictc(c_int::from(b'0')));
        assert!(rs_eval_isdictc(c_int::from(b'_')));

        // Not allowed
        assert!(!rs_eval_isdictc(c_int::from(b':')));
        assert!(!rs_eval_isdictc(c_int::from(b'#')));
        assert!(!rs_eval_isdictc(c_int::from(b' ')));
        assert!(!rs_eval_isdictc(-1));
    }
}
