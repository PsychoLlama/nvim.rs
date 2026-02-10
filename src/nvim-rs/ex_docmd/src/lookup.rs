//! Command lookup types and utilities for Ex commands.
//!
//! This module provides types and functions for looking up command names
//! in the command table.

use std::ffi::{c_char, c_int};

extern "C" {
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_docmd_cmd_k() -> c_int;
    fn nvim_docmd_cmd_substitute() -> c_int;
    fn nvim_docmd_cmd_match() -> c_int;
    fn nvim_docmd_cmd_size() -> c_int;
    fn nvim_docmd_cmd_exists_inner(
        name: *const c_char,
        out_cmdidx: *mut c_int,
        out_full: *mut c_int,
    ) -> *mut c_char;
}

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

// =============================================================================
// checkforcmd - Check command name prefix match
// =============================================================================

/// Check if the string at `*pp` matches the command name `cmd` with
/// at least `len` characters. If so, advance `*pp` past the match
/// and any trailing whitespace.
///
/// Matches C `checkforcmd()`.
///
/// # Safety
///
/// `pp` must point to a valid `*mut c_char` pointer.
/// `cmd` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_checkforcmd(
    pp: *mut *mut c_char,
    cmd: *const c_char,
    len: c_int,
) -> bool {
    if pp.is_null() || cmd.is_null() {
        return false;
    }

    let mut i = 0i32;
    loop {
        let c = *cmd.add(i as usize) as u8;
        if c == 0 {
            break;
        }
        if c != *(*pp).add(i as usize) as u8 {
            break;
        }
        i += 1;
    }

    if i >= len && !(*(*pp).add(i as usize) as u8).is_ascii_alphabetic() {
        *pp = skipwhite((*pp).add(i as usize) as *const c_char);
        return true;
    }
    false
}

// =============================================================================
// one_letter_cmd - Full implementation matching C
// =============================================================================

/// Check if the string at `p` starts a one-letter command.
///
/// If so, sets `*idx` to the command index (CMD_k or CMD_substitute)
/// and returns 1. Otherwise returns 0.
///
/// Matches C `one_letter_cmd()` exactly — including the complex 's' exclusions
/// for :scriptnames, :source, :simalt, :sign, :smagic, :snomagic, etc.
///
/// # Safety
///
/// `p` must be a valid null-terminated C string.
/// `idx` must be a valid pointer for writes.
#[no_mangle]
pub unsafe extern "C" fn rs_one_letter_cmd(p: *const c_char, idx: *mut c_int) -> c_int {
    if p.is_null() || idx.is_null() {
        return 0;
    }

    let p0 = *p as u8;
    let p1 = *p.add(1) as u8;
    let p2 = *p.add(2) as u8;

    // 'k' command - mark
    // Match: k followed by anything except "ee" (which would be :keepXXX)
    if p0 == b'k' && !(p1 == b'e' && p2 == b'e') {
        *idx = nvim_docmd_cmd_k();
        return 1;
    }

    // 's' command - substitute
    if p0 == b's' {
        let p3 = *p.add(3) as u8;
        let p4 = *p.add(4) as u8;

        if (p1 == b'c'
            && (p2 == 0 || (p2 != b's' && p2 != b'r' && (p3 == 0 || (p3 != b'i' && p4 != b'p')))))
            || p1 == b'g'
            || (p1 == b'i' && p2 != b'm' && p2 != b'l' && p2 != b'g')
            || p1 == b'I'
            || (p1 == b'r' && p2 != b'e')
        {
            *idx = nvim_docmd_cmd_substitute();
            return 1;
        }
    }

    0
}

// =============================================================================
// cmd_exists - Check if command name exists
// =============================================================================

/// Check if an Ex command `name` exists.
///
/// Returns:
/// - 0: command doesn't exist
/// - 1: partial match (abbreviation)
/// - 2: exact match
/// - 3: ambiguous match
///
/// Matches C `cmd_exists()`.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cmd_exists(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    // Check command modifiers first.
    let modifier_result = crate::modifiers::check_modifier(name);
    if modifier_result > 0 {
        return modifier_result;
    }

    // Check built-in commands and user defined commands.
    let mut cmdidx: c_int = 0;
    let mut full: c_int = 0;
    let p = nvim_docmd_cmd_exists_inner(name, &mut cmdidx, &mut full);

    if p.is_null() {
        return 3;
    }

    if (*name as u8).is_ascii_digit() && cmdidx != nvim_docmd_cmd_match() {
        return 0;
    }

    if *skipwhite(p as *const c_char) as u8 != 0 {
        return 0; // trailing garbage
    }

    if cmdidx == nvim_docmd_cmd_size() {
        0
    } else if full != 0 {
        2
    } else {
        1
    }
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

    // Note: rs_checkforcmd, rs_one_letter_cmd, rs_cmd_exists tests require C FFI
    // (skipwhite, nvim_docmd_cmd_k, etc.) and are verified through integration
    // tests (just smoke-test) instead.
}
