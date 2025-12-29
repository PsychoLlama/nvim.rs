//! Ex command utilities for Neovim
//!
//! Provides utility functions for Ex command parsing and processing.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::{c_char, c_int};
use std::ptr;

// FFI declarations for C helper functions
extern "C" {
    fn cmdname_first_char(cmdidx: c_int) -> c_int;
    fn nvim_get_ex_pressedreturn() -> c_int;
    fn nvim_get_expr_map_lock() -> c_int;
    fn nvim_curbuf_is_dummy() -> c_int;
    fn nvim_get_cmdwin_type() -> c_int;
    fn nvim_get_textlock() -> c_int;
}

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

/// Find the next command in a string.
///
/// Scans past the first '|' or '\n' character, returning the position after it.
/// Returns `None` if no command separator is found (i.e., reaches NUL).
///
/// This is used for parsing Ex command lines that can contain multiple
/// commands separated by '|' or '\n'.
#[inline]
pub fn find_nextcmd(s: &[u8]) -> Option<usize> {
    for (i, &c) in s.iter().enumerate() {
        if c == b'|' || c == b'\n' {
            return Some(i + 1);
        }
        if c == 0 {
            return None;
        }
    }
    None
}

/// FFI wrapper for `find_nextcmd`.
///
/// Returns a pointer to the character after the first '|' or '\n',
/// or NULL if no separator is found before NUL.
///
/// # Safety
///
/// `p` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_find_nextcmd(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return ptr::null();
    }

    let mut ptr = p;
    loop {
        let c = unsafe { *ptr } as u8;
        if c == b'|' || c == b'\n' {
            return unsafe { ptr.add(1) };
        }
        if c == 0 {
            return ptr::null();
        }
        ptr = unsafe { ptr.add(1) };
    }
}

/// Check if pointer is at a command separator, skipping whitespace.
///
/// Skips over whitespace (' ' and '\t'), then checks if the next character
/// is '|' or '\n'. If so, returns the position after the separator.
/// Returns `None` if not at a separator.
#[inline]
pub fn check_nextcmd(s: &[u8]) -> Option<usize> {
    let mut i = 0;
    // Skip whitespace
    while i < s.len() && (s[i] == b' ' || s[i] == b'\t') {
        i += 1;
    }
    // Check for separator
    if i < s.len() && (s[i] == b'|' || s[i] == b'\n') {
        Some(i + 1)
    } else {
        None
    }
}

/// FFI wrapper for `check_nextcmd`.
///
/// Skips whitespace, then returns a pointer to after the '|' or '\n',
/// or NULL if not at a separator.
///
/// # Safety
///
/// `p` must be a valid C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_check_nextcmd(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return ptr::null();
    }

    let mut ptr = p;
    // Skip whitespace
    loop {
        let c = unsafe { *ptr } as u8;
        if c != b' ' && c != b'\t' {
            break;
        }
        ptr = unsafe { ptr.add(1) };
    }

    let c = unsafe { *ptr } as u8;
    if c == b'|' || c == b'\n' {
        unsafe { ptr.add(1) }
    } else {
        ptr::null()
    }
}

/// Check if command index is for a location list command.
///
/// Returns true if the command at the given index starts with 'l',
/// indicating it's a location list command rather than a quickfix command.
/// Returns false if the index is out of bounds.
#[inline]
pub fn is_loclist_cmd(cmdidx: i32, cmd_size: i32) -> bool {
    if cmdidx < 0 || cmdidx >= cmd_size {
        return false;
    }
    // Call C helper to get first char of command name
    let first_char = unsafe { cmdname_first_char(cmdidx) };
    first_char == b'l' as c_int
}

/// FFI wrapper for `is_loclist_cmd`.
///
/// Returns 1 if the command is a location list command, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_is_loclist_cmd(cmdidx: c_int, cmd_size: c_int) -> c_int {
    c_int::from(is_loclist_cmd(cmdidx, cmd_size))
}

/// Get the current value of ex_pressedreturn.
///
/// Returns true if the user pressed Enter on an empty command line.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[no_mangle]
pub unsafe extern "C" fn rs_get_pressedreturn() -> c_int {
    nvim_get_ex_pressedreturn()
}

/// Check if expression mapping is locked.
///
/// Returns true if `expr_map_lock > 0` and current buffer is not a dummy buffer.
/// This prevents use of ex_normal() and text changes while running an expr mapping.
///
/// # Safety
///
/// Calls external C functions to access global variables.
#[no_mangle]
pub unsafe extern "C" fn rs_expr_map_locked() -> c_int {
    let lock = nvim_get_expr_map_lock();
    let is_dummy = nvim_curbuf_is_dummy();
    c_int::from(lock > 0 && is_dummy == 0)
}

/// Check if text is locked.
///
/// Returns true when the text must not be changed and we can't switch to
/// another window or buffer. True when editing the command line, etc.
///
/// This returns true if:
/// - cmdwin_type != 0 (in command-line window)
/// - expr_map_locked() is true (running expression mapping)
/// - textlock != 0 (text editing is locked)
///
/// # Safety
///
/// Calls external C functions to access global variables.
#[no_mangle]
pub unsafe extern "C" fn rs_text_locked() -> c_int {
    let cmdwin_type = nvim_get_cmdwin_type();
    if cmdwin_type != 0 {
        return 1;
    }
    if rs_expr_map_locked() != 0 {
        return 1;
    }
    let textlock = nvim_get_textlock();
    c_int::from(textlock != 0)
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

    #[test]
    fn test_find_nextcmd() {
        // Find pipe separator
        assert_eq!(find_nextcmd(b"cmd1|cmd2\0"), Some(5));
        assert_eq!(find_nextcmd(b"|cmd\0"), Some(1));

        // Find newline separator
        assert_eq!(find_nextcmd(b"cmd1\ncmd2\0"), Some(5));
        assert_eq!(find_nextcmd(b"\ncmd\0"), Some(1));

        // No separator - NUL first
        assert_eq!(find_nextcmd(b"cmd\0"), None);
        assert_eq!(find_nextcmd(b"\0"), None);

        // Empty slice returns None
        assert_eq!(find_nextcmd(b""), None);
    }

    #[test]
    fn test_ffi_find_nextcmd() {
        use std::ffi::CString;

        // Find pipe separator
        let s = CString::new("cmd1|cmd2").unwrap();
        let result = unsafe { rs_find_nextcmd(s.as_ptr()) };
        assert!(!result.is_null());
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_bytes(), b"cmd2");

        // Find newline separator
        let s = CString::new("cmd1\ncmd2").unwrap();
        let result = unsafe { rs_find_nextcmd(s.as_ptr()) };
        assert!(!result.is_null());
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_bytes(), b"cmd2");

        // No separator
        let s = CString::new("cmd").unwrap();
        let result = unsafe { rs_find_nextcmd(s.as_ptr()) };
        assert!(result.is_null());

        // NULL input
        let result = unsafe { rs_find_nextcmd(ptr::null()) };
        assert!(result.is_null());
    }

    #[test]
    fn test_check_nextcmd() {
        // Separator after whitespace
        assert_eq!(check_nextcmd(b"  |cmd\0"), Some(3));
        assert_eq!(check_nextcmd(b"\t\t\ncmd\0"), Some(3));
        assert_eq!(check_nextcmd(b" \t |rest\0"), Some(4));

        // Direct separator (no whitespace)
        assert_eq!(check_nextcmd(b"|cmd\0"), Some(1));
        assert_eq!(check_nextcmd(b"\ncmd\0"), Some(1));

        // Not a separator
        assert_eq!(check_nextcmd(b"cmd\0"), None);
        assert_eq!(check_nextcmd(b"  cmd\0"), None);
        assert_eq!(check_nextcmd(b"\0"), None);
    }

    #[test]
    fn test_ffi_check_nextcmd() {
        use std::ffi::CString;

        // Separator after whitespace
        let s = CString::new("  |cmd").unwrap();
        let result = unsafe { rs_check_nextcmd(s.as_ptr()) };
        assert!(!result.is_null());
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_bytes(), b"cmd");

        // Not a separator
        let s = CString::new("  cmd").unwrap();
        let result = unsafe { rs_check_nextcmd(s.as_ptr()) };
        assert!(result.is_null());

        // NULL input
        let result = unsafe { rs_check_nextcmd(ptr::null()) };
        assert!(result.is_null());
    }
}
