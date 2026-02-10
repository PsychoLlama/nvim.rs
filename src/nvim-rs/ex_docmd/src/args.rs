//! Argument parsing types and utilities for Ex commands.
//!
//! This module provides types and functions for parsing command arguments,
//! including ++opt options, counts, registers, and filename expansion.

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::ExArgHandle;

// =============================================================================
// FFI declarations for exarg_T accessors and command index values
// =============================================================================

extern "C" {
    fn nvim_eap_get_arg(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_get_cmdidx(eap: ExArgHandle) -> c_int;
    fn nvim_eap_set_line1(eap: ExArgHandle, line: i32);
    fn nvim_eap_get_line2(eap: ExArgHandle) -> i32;
    fn nvim_eap_set_line2(eap: ExArgHandle, line: i32);
    fn nvim_eap_get_addr_type(eap: ExArgHandle) -> c_int;
    fn nvim_eap_get_addr_count(eap: ExArgHandle) -> c_int;
    fn nvim_eap_set_addr_count(eap: ExArgHandle, count: c_int);

    fn nvim_docmd_cmd_substitute() -> c_int;
    fn nvim_docmd_cmd_smagic() -> c_int;
    fn nvim_docmd_cmd_snomagic() -> c_int;
    fn nvim_docmd_cmd_vimgrep() -> c_int;
    fn nvim_docmd_cmd_lvimgrep() -> c_int;
    fn nvim_docmd_cmd_vimgrepadd() -> c_int;
    fn nvim_docmd_cmd_lvimgrepadd() -> c_int;
    fn nvim_docmd_grep_internal(cmdidx: c_int) -> c_int;
    fn nvim_docmd_get_curbuf_line_count() -> i32;

    fn rs_skip_vimgrep_pat(p: *mut c_char, s: *mut *mut c_char, flags: *mut c_int) -> *mut c_char;
}

/// Address type: lines in current buffer (matches ADDR_LINES in C).
const ADDR_LINES: c_int = 0;

// =============================================================================
// Force binary mode constants
// =============================================================================

/// Don't force binary mode
pub const FORCE_BIN_NONE: c_int = 0;
/// Force binary mode (:edit ++bin)
pub const FORCE_BIN: c_int = 1;
/// Force no binary mode (:edit ++nobin)
pub const FORCE_NOBIN: c_int = 2;

// =============================================================================
// ++opt argument parsing helpers
// =============================================================================

/// Check if the argument starts with "++" (option argument).
#[inline]
pub fn starts_with_plus_plus(arg: &[u8]) -> bool {
    arg.len() >= 2 && arg[0] == b'+' && arg[1] == b'+'
}

/// FFI wrapper for ++opt check.
///
/// # Safety
///
/// `arg` must be a valid pointer to at least 2 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_starts_with_plus_plus(arg: *const c_char) -> c_int {
    if arg.is_null() {
        return 0;
    }
    let c0 = *arg as u8;
    let c1 = *arg.add(1) as u8;
    c_int::from(c0 == b'+' && c1 == b'+')
}

/// Check if argument matches a ++opt prefix.
///
/// Returns true if `arg` starts with `prefix`.
#[inline]
pub fn matches_argopt(arg: &[u8], prefix: &[u8]) -> bool {
    arg.len() >= prefix.len() && &arg[..prefix.len()] == prefix
}

// =============================================================================
// Count argument helpers
// =============================================================================

/// Check if a character is a valid count digit.
#[inline]
pub const fn is_count_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// FFI wrapper for count digit check.
#[no_mangle]
pub extern "C" fn rs_is_count_digit(c: c_int) -> c_int {
    c_int::from(is_count_digit(c as u8))
}

/// Parse a count from a string.
///
/// Returns the parsed count and the number of digits consumed.
pub fn parse_count(s: &[u8]) -> (u64, usize) {
    let mut count: u64 = 0;
    let mut pos = 0;

    while pos < s.len() && is_count_digit(s[pos]) {
        count = count
            .saturating_mul(10)
            .saturating_add((s[pos] - b'0') as u64);
        pos += 1;
    }

    (count, pos)
}

/// FFI wrapper for count parsing.
///
/// Returns the parsed count value, or 0 if no digits found.
///
/// # Safety
///
/// `s` must be a valid null-terminated string.
/// `consumed` must be a valid pointer for writing the number of digits.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_count(s: *const c_char, consumed: *mut c_int) -> u64 {
    if s.is_null() {
        if !consumed.is_null() {
            *consumed = 0;
        }
        return 0;
    }

    let mut count: u64 = 0;
    let mut pos = 0;
    let mut ptr = s;

    loop {
        let c = *ptr as u8;
        if c == 0 || !is_count_digit(c) {
            break;
        }
        count = count.saturating_mul(10).saturating_add((c - b'0') as u64);
        pos += 1;
        ptr = ptr.add(1);
    }

    if !consumed.is_null() {
        *consumed = pos;
    }
    count
}

// =============================================================================
// Register argument helpers
// =============================================================================

/// Check if a character is a valid register name.
///
/// Valid registers are:
/// - 0-9 (numbered)
/// - a-z, A-Z (named)
/// - ", -, _, +, *, ~, /, :, ., %, # (special)
#[inline]
pub fn is_valid_register(c: u8) -> bool {
    c.is_ascii_alphanumeric()
        || c == b'"'
        || c == b'-'
        || c == b'_'
        || c == b'+'
        || c == b'*'
        || c == b'~'
        || c == b'/'
        || c == b':'
        || c == b'.'
        || c == b'%'
        || c == b'#'
        || c == b'='
}

/// FFI wrapper for register validation.
#[no_mangle]
pub extern "C" fn rs_is_valid_register(c: c_int) -> c_int {
    c_int::from(is_valid_register(c as u8))
}

/// Check if this is a read-only register.
///
/// Read-only registers: %, #, :, .
#[inline]
pub const fn is_readonly_register(c: u8) -> bool {
    c == b'%' || c == b'#' || c == b':' || c == b'.'
}

/// FFI wrapper for read-only register check.
#[no_mangle]
pub extern "C" fn rs_is_readonly_register(c: c_int) -> c_int {
    c_int::from(is_readonly_register(c as u8))
}

/// Check if this is a special register.
///
/// Special registers: ", -, _, +, *, ~, /, =
#[inline]
pub const fn is_special_register(c: u8) -> bool {
    c == b'"'
        || c == b'-'
        || c == b'_'
        || c == b'+'
        || c == b'*'
        || c == b'~'
        || c == b'/'
        || c == b'='
}

/// FFI wrapper for special register check.
#[no_mangle]
pub extern "C" fn rs_is_special_register(c: c_int) -> c_int {
    c_int::from(is_special_register(c as u8))
}

// =============================================================================
// Filename expansion helpers
// =============================================================================

/// Check if character needs expansion in filenames.
///
/// Characters that trigger expansion: %, #, <
#[inline]
pub const fn needs_filename_expansion(c: u8) -> bool {
    c == b'%' || c == b'#' || c == b'<'
}

/// FFI wrapper for filename expansion check.
#[no_mangle]
pub extern "C" fn rs_needs_filename_expansion(c: c_int) -> c_int {
    c_int::from(needs_filename_expansion(c as u8))
}

/// Check if the position is at a backslash-escaped character.
///
/// Returns true if position has a backslash before it.
#[inline]
pub fn is_escaped(s: &[u8], pos: usize) -> bool {
    if pos == 0 {
        return false;
    }

    // Count consecutive backslashes before pos
    let mut count = 0;
    let mut i = pos;
    while i > 0 && s[i - 1] == b'\\' {
        count += 1;
        i -= 1;
    }

    // Odd number of backslashes means the character is escaped
    count % 2 == 1
}

// =============================================================================
// parse_bang - Check for `!` after command
// =============================================================================

/// Check if `!` follows the command (and it's not a substitute variant).
///
/// Returns true if bang is found and consumed. Advances `*p` past the `!`.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
/// `p` must point to a valid `*mut c_char` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_bang(eap: ExArgHandle, p: *mut *mut c_char) -> bool {
    if eap.is_null() || p.is_null() {
        return false;
    }

    let cmdidx = nvim_eap_get_cmdidx(eap);

    if *(*p) as u8 == b'!'
        && cmdidx != nvim_docmd_cmd_substitute()
        && cmdidx != nvim_docmd_cmd_smagic()
        && cmdidx != nvim_docmd_cmd_snomagic()
    {
        *p = (*p).add(1);
        return true;
    }
    false
}

// =============================================================================
// skip_grep_pat - Skip grep pattern in arguments
// =============================================================================

/// Skip the grep pattern in command arguments for vimgrep-like commands.
///
/// Returns a pointer past the pattern, or the original arg if not a grep command.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_grep_pat(eap: ExArgHandle) -> *mut c_char {
    if eap.is_null() {
        return ptr::null_mut();
    }

    let arg = nvim_eap_get_arg(eap);
    if arg.is_null() || *arg as u8 == 0 {
        return arg;
    }

    let cmdidx = nvim_eap_get_cmdidx(eap);

    if cmdidx == nvim_docmd_cmd_vimgrep()
        || cmdidx == nvim_docmd_cmd_lvimgrep()
        || cmdidx == nvim_docmd_cmd_vimgrepadd()
        || cmdidx == nvim_docmd_cmd_lvimgrepadd()
        || nvim_docmd_grep_internal(cmdidx) != 0
    {
        let p = rs_skip_vimgrep_pat(arg, ptr::null_mut(), ptr::null_mut());
        if p.is_null() {
            return arg;
        }
        return p;
    }
    arg
}

// =============================================================================
// set_cmd_count - Set count from address into eap fields
// =============================================================================

/// Set the command count from an address value.
///
/// For non-line address types (e.g. `:buffer 2`), stores count in line2.
/// For line addresses, treats count as an offset from line2.
/// If `validate` is non-zero, clamps line2 to buffer line count.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[no_mangle]
pub unsafe extern "C" fn rs_set_cmd_count(eap: ExArgHandle, count: c_int, validate: c_int) {
    if eap.is_null() {
        return;
    }

    let addr_type = nvim_eap_get_addr_type(eap);

    if addr_type != ADDR_LINES {
        // e.g. :buffer 2, :sleep 3
        nvim_eap_set_line2(eap, count);
        if nvim_eap_get_addr_count(eap) == 0 {
            nvim_eap_set_addr_count(eap, 1);
        }
    } else {
        let line2 = nvim_eap_get_line2(eap);
        nvim_eap_set_line1(eap, line2);

        if line2 >= i32::MAX - (count - 1) {
            nvim_eap_set_line2(eap, i32::MAX);
        } else {
            nvim_eap_set_line2(eap, line2 + count - 1);
        }

        nvim_eap_set_addr_count(eap, nvim_eap_get_addr_count(eap) + 1);

        // Be vi compatible: no error message for out of range.
        if validate != 0 {
            let line_count = nvim_docmd_get_curbuf_line_count();
            let new_line2 = nvim_eap_get_line2(eap);
            if new_line2 > line_count {
                nvim_eap_set_line2(eap, line_count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_with_plus_plus() {
        assert!(starts_with_plus_plus(b"++bin"));
        assert!(starts_with_plus_plus(b"++"));
        assert!(!starts_with_plus_plus(b"+bin"));
        assert!(!starts_with_plus_plus(b"bin"));
        assert!(!starts_with_plus_plus(b"+"));
        assert!(!starts_with_plus_plus(b""));
    }

    #[test]
    fn test_matches_argopt() {
        assert!(matches_argopt(b"binary", b"bin"));
        assert!(matches_argopt(b"bin", b"bin"));
        assert!(!matches_argopt(b"bi", b"bin"));
        assert!(matches_argopt(b"nobinary", b"nobin"));
    }

    #[test]
    fn test_parse_count() {
        assert_eq!(parse_count(b"123abc"), (123, 3));
        assert_eq!(parse_count(b"0"), (0, 1));
        assert_eq!(parse_count(b"42"), (42, 2));
        assert_eq!(parse_count(b"abc"), (0, 0));
        assert_eq!(parse_count(b""), (0, 0));
    }

    #[test]
    fn test_is_valid_register() {
        // Named registers
        assert!(is_valid_register(b'a'));
        assert!(is_valid_register(b'z'));
        assert!(is_valid_register(b'A'));
        assert!(is_valid_register(b'Z'));

        // Numbered registers
        assert!(is_valid_register(b'0'));
        assert!(is_valid_register(b'9'));

        // Special registers
        assert!(is_valid_register(b'"'));
        assert!(is_valid_register(b'-'));
        assert!(is_valid_register(b'_'));
        assert!(is_valid_register(b'+'));
        assert!(is_valid_register(b'*'));
        assert!(is_valid_register(b'/'));

        // Invalid
        assert!(!is_valid_register(b' '));
        assert!(!is_valid_register(b'\n'));
    }

    #[test]
    fn test_is_readonly_register() {
        assert!(is_readonly_register(b'%'));
        assert!(is_readonly_register(b'#'));
        assert!(is_readonly_register(b':'));
        assert!(is_readonly_register(b'.'));
        assert!(!is_readonly_register(b'a'));
        assert!(!is_readonly_register(b'"'));
    }

    #[test]
    fn test_needs_filename_expansion() {
        assert!(needs_filename_expansion(b'%'));
        assert!(needs_filename_expansion(b'#'));
        assert!(needs_filename_expansion(b'<'));
        assert!(!needs_filename_expansion(b'a'));
        assert!(!needs_filename_expansion(b'/'));
    }

    #[test]
    fn test_is_escaped() {
        assert!(!is_escaped(b"abc", 0));
        assert!(!is_escaped(b"abc", 1));
        assert!(is_escaped(b"a\\bc", 2));
        assert!(!is_escaped(b"a\\\\bc", 3)); // Even backslashes
        assert!(is_escaped(b"a\\\\\\bc", 4)); // Odd backslashes
    }

    #[test]
    fn test_ffi_parse_count() {
        use std::ffi::CString;

        let s = CString::new("123abc").unwrap();
        let mut consumed: c_int = 0;
        let count = unsafe { rs_parse_count(s.as_ptr(), &mut consumed) };
        assert_eq!(count, 123);
        assert_eq!(consumed, 3);

        let s = CString::new("abc").unwrap();
        let count = unsafe { rs_parse_count(s.as_ptr(), &mut consumed) };
        assert_eq!(count, 0);
        assert_eq!(consumed, 0);
    }
}
