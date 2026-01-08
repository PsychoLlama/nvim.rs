//! Command lookup types and utilities for Ex commands.
//!
//! This module provides types and functions for looking up command names
//! in the command table.

use std::ffi::{c_char, c_int};

// =============================================================================
// Command index special values
// =============================================================================

/// Command index for unknown/invalid commands
pub const CMD_SIZE: c_int = -1; // Actually varies, but used as "not found"

// =============================================================================
// One-letter command helpers
// =============================================================================

/// Check if the character at position is a one-letter command.
///
/// One-letter commands are:
/// - 'k' (mark)
/// - 's' (substitute) followed by non-alpha or the substitute pattern delimiter
///
/// Returns true if it's a one-letter command.
#[inline]
pub fn is_one_letter_cmd_char(c: u8, next: u8) -> bool {
    // 'k' is always a one-letter command (mark)
    if c == b'k' {
        return true;
    }

    // 's' is a one-letter command if followed by specific characters
    if c == b's' {
        // 's' followed by non-alpha, or by a delimiter character
        // is the substitute command
        if !next.is_ascii_alphabetic() {
            return true;
        }
    }

    false
}

/// FFI wrapper for checking one-letter commands.
#[no_mangle]
pub extern "C" fn rs_is_one_letter_cmd_char(c: c_int, next: c_int) -> c_int {
    c_int::from(is_one_letter_cmd_char(c as u8, next as u8))
}

// =============================================================================
// Command name classification
// =============================================================================

/// Check if a character can start a command name.
///
/// Command names can start with:
/// - ASCII letters (a-z, A-Z)
/// - Special characters: @ ! = > < & ~ #
#[inline]
pub const fn is_cmd_name_start(c: u8) -> bool {
    c.is_ascii_alphabetic()
        || c == b'@'
        || c == b'!'
        || c == b'='
        || c == b'>'
        || c == b'<'
        || c == b'&'
        || c == b'~'
        || c == b'#'
}

/// FFI wrapper for command name start check.
#[no_mangle]
pub extern "C" fn rs_is_cmd_name_start(c: c_int) -> c_int {
    c_int::from(is_cmd_name_start(c as u8))
}

/// Check if a character can be part of a command name.
///
/// Command names consist of ASCII letters.
#[inline]
pub const fn is_cmd_name_char(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// FFI wrapper for command name character check.
#[no_mangle]
pub extern "C" fn rs_is_cmd_name_char(c: c_int) -> c_int {
    c_int::from(is_cmd_name_char(c as u8))
}

/// Check if this could be a Python command prefix.
///
/// Python commands start with "py" and can be followed by alphanumeric
/// characters (e.g., py3, python3, py3file).
#[inline]
pub fn is_python_cmd_prefix(cmd: &[u8]) -> bool {
    cmd.len() >= 2 && cmd[0] == b'p' && cmd[1] == b'y'
}

/// FFI wrapper for Python command prefix check.
///
/// # Safety
///
/// The pointer must be valid and point to at least 2 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_is_python_cmd_prefix(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        return 0;
    }
    let c0 = *cmd as u8;
    let c1 = *cmd.add(1) as u8;
    c_int::from(c0 == b'p' && c1 == b'y')
}

// =============================================================================
// User command detection
// =============================================================================

/// Check if a character can start a user-defined command.
///
/// User commands must start with an uppercase letter (A-Z).
#[inline]
pub const fn is_user_cmd_start(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// FFI wrapper for user command start check.
#[no_mangle]
pub extern "C" fn rs_is_user_cmd_start(c: c_int) -> c_int {
    c_int::from(is_user_cmd_start(c as u8))
}

/// Check if a character can be part of a user-defined command name.
///
/// User commands can contain letters and digits.
#[inline]
pub const fn is_user_cmd_char(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

/// FFI wrapper for user command character check.
#[no_mangle]
pub extern "C" fn rs_is_user_cmd_char(c: c_int) -> c_int {
    c_int::from(is_user_cmd_char(c as u8))
}

// =============================================================================
// Command index calculation helpers
// =============================================================================

/// Calculate the ordinal of a lowercase letter (0-25).
///
/// Returns the ordinal value, or 0 if not a lowercase letter.
#[inline]
pub const fn cmd_char_ord_low(c: u8) -> usize {
    if c >= b'a' && c <= b'z' {
        (c - b'a') as usize
    } else {
        0
    }
}

/// FFI wrapper for lowercase character ordinal for command lookup.
#[no_mangle]
pub extern "C" fn rs_cmd_char_ord_low(c: c_int) -> c_int {
    cmd_char_ord_low(c as u8) as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_one_letter_cmd_char() {
        // 'k' is always a one-letter command
        assert!(is_one_letter_cmd_char(b'k', b'x'));
        assert!(is_one_letter_cmd_char(b'k', b' '));
        assert!(is_one_letter_cmd_char(b'k', 0));

        // 's' followed by non-alpha is substitute
        assert!(is_one_letter_cmd_char(b's', b'/'));
        assert!(is_one_letter_cmd_char(b's', b' '));
        assert!(is_one_letter_cmd_char(b's', 0));

        // 's' followed by alpha is not a one-letter command
        assert!(!is_one_letter_cmd_char(b's', b'e')); // :set
        assert!(!is_one_letter_cmd_char(b's', b'o')); // :sort

        // Other letters are not one-letter commands
        assert!(!is_one_letter_cmd_char(b'w', b' '));
        assert!(!is_one_letter_cmd_char(b'q', b' '));
    }

    #[test]
    fn test_is_cmd_name_start() {
        // Letters
        assert!(is_cmd_name_start(b'a'));
        assert!(is_cmd_name_start(b'z'));
        assert!(is_cmd_name_start(b'A'));
        assert!(is_cmd_name_start(b'Z'));

        // Special characters
        assert!(is_cmd_name_start(b'@'));
        assert!(is_cmd_name_start(b'!'));
        assert!(is_cmd_name_start(b'='));
        assert!(is_cmd_name_start(b'>'));
        assert!(is_cmd_name_start(b'<'));
        assert!(is_cmd_name_start(b'&'));
        assert!(is_cmd_name_start(b'~'));
        assert!(is_cmd_name_start(b'#'));

        // Not valid starts
        assert!(!is_cmd_name_start(b'1'));
        assert!(!is_cmd_name_start(b' '));
        assert!(!is_cmd_name_start(b':'));
    }

    #[test]
    fn test_is_cmd_name_char() {
        assert!(is_cmd_name_char(b'a'));
        assert!(is_cmd_name_char(b'Z'));
        assert!(!is_cmd_name_char(b'1'));
        assert!(!is_cmd_name_char(b' '));
    }

    #[test]
    fn test_is_python_cmd_prefix() {
        assert!(is_python_cmd_prefix(b"py"));
        assert!(is_python_cmd_prefix(b"python"));
        assert!(is_python_cmd_prefix(b"py3"));
        assert!(!is_python_cmd_prefix(b"p"));
        assert!(!is_python_cmd_prefix(b"pe"));
    }

    #[test]
    fn test_is_user_cmd_start() {
        assert!(is_user_cmd_start(b'A'));
        assert!(is_user_cmd_start(b'Z'));
        assert!(!is_user_cmd_start(b'a'));
        assert!(!is_user_cmd_start(b'1'));
    }

    #[test]
    fn test_is_user_cmd_char() {
        assert!(is_user_cmd_char(b'A'));
        assert!(is_user_cmd_char(b'z'));
        assert!(is_user_cmd_char(b'5'));
        assert!(!is_user_cmd_char(b' '));
        assert!(!is_user_cmd_char(b'_'));
    }

    #[test]
    fn test_cmd_char_ord_low() {
        assert_eq!(cmd_char_ord_low(b'a'), 0);
        assert_eq!(cmd_char_ord_low(b'b'), 1);
        assert_eq!(cmd_char_ord_low(b'z'), 25);
        assert_eq!(cmd_char_ord_low(b'A'), 0); // Returns 0 for non-lowercase
    }
}
