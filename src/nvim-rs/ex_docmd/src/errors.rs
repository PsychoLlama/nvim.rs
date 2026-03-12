//! Error handling and message types for Ex commands.
//!
//! This module provides error message constants and utilities for
//! generating and formatting Ex command error messages.

use std::ffi::{c_char, c_int};

/// Get the length of a null-terminated C string.
///
/// # Safety
/// `s` must be a valid null-terminated string.
unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

// =============================================================================
// Error message string constants (error codes for reference)
// =============================================================================

/// Error code for invalid argument (E474)
pub const E_INVARG: c_int = 474;
/// Error code for invalid argument with detail (E475)
pub const E_INVARG2: c_int = 475;
/// Error code for invalid range (E16)
pub const E_INVRANGE: c_int = 16;
/// Error code for no range allowed (E481)
pub const E_NORANGE: c_int = 481;
/// Error code for trailing characters (E488)
pub const E_TRAILING: c_int = 488;

// =============================================================================
// FFI declarations for C error strings and functions
// =============================================================================

extern "C" {
    fn nvim_get_e_invarg() -> *const c_char;
    fn nvim_get_e_invarg2() -> *const c_char;
    fn nvim_get_e_invargval() -> *const c_char;
    fn nvim_get_e_invrange() -> *const c_char;
    fn nvim_get_e_norange() -> *const c_char;
    fn nvim_get_e_trailing_arg() -> *const c_char;

    fn nvim_emsg(s: *const c_char);

    // IObuff accessors for append_command
    fn nvim_docmd_get_iobuff() -> *mut c_char;
    fn nvim_docmd_get_iosize() -> c_int;
    fn nvim_docmd_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn nvim_docmd_utfc_ptr2len(p: *const c_char) -> c_int;
    fn nvim_docmd_mb_copy_char(fp: *mut *const c_char, tp: *mut *mut c_char);
    fn nvim_docmd_xstrlcat_iobuff(src: *const c_char);
}

// =============================================================================
// Error string accessors
// =============================================================================

/// Get the "Invalid argument" error message string.
///
/// Returns the C string for E474.
///
/// # Safety
///
/// Returns a pointer to a static C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_e_invarg() -> *const c_char {
    nvim_get_e_invarg()
}

/// Get the "Invalid argument: %s" error message string.
///
/// Returns the C string for E475 (format string).
///
/// # Safety
///
/// Returns a pointer to a static C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_e_invarg2() -> *const c_char {
    nvim_get_e_invarg2()
}

/// Get the "Invalid value for argument %s" error message string.
///
/// Returns the C string for E475 (format string for values).
///
/// # Safety
///
/// Returns a pointer to a static C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_e_invargval() -> *const c_char {
    nvim_get_e_invargval()
}

/// Get the "Invalid range" error message string.
///
/// Returns the C string for E16.
///
/// # Safety
///
/// Returns a pointer to a static C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_e_invrange() -> *const c_char {
    nvim_get_e_invrange()
}

/// Get the "No range allowed" error message string.
///
/// Returns the C string for E481.
///
/// # Safety
///
/// Returns a pointer to a static C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_e_norange() -> *const c_char {
    nvim_get_e_norange()
}

/// Get the "Trailing characters: %s" error message string.
///
/// Returns the C string for E488 (format string).
///
/// # Safety
///
/// Returns a pointer to a static C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_e_trailing_arg() -> *const c_char {
    nvim_get_e_trailing_arg()
}

// =============================================================================
// Error emission utilities
// =============================================================================

/// Emit the "Invalid range" error message (E16).
///
/// # Safety
///
/// Calls external C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_invrange() {
    nvim_emsg(nvim_get_e_invrange());
}

/// Emit the "Invalid argument" error message (E474).
///
/// # Safety
///
/// Calls external C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_invarg() {
    nvim_emsg(nvim_get_e_invarg());
}

/// Emit the "No range allowed" error message (E481).
///
/// # Safety
///
/// Calls external C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_norange() {
    nvim_emsg(nvim_get_e_norange());
}

// =============================================================================
// Range validation helpers with error emission
// =============================================================================

/// Check if line numbers are in valid order.
///
/// Returns true if line1 <= line2 and both are non-negative.
#[inline]
pub const fn lines_in_order(line1: i64, line2: i64) -> bool {
    line1 >= 0 && line2 >= 0 && line1 <= line2
}

/// FFI wrapper for line order check.
#[no_mangle]
pub extern "C" fn rs_lines_in_order(line1: i64, line2: i64) -> c_int {
    c_int::from(lines_in_order(line1, line2))
}

/// Validate a line range and emit an error if invalid.
///
/// Checks:
/// - line1 >= 0
/// - line2 >= 0
/// - line1 <= line2
///
/// Returns 0 if valid, 1 if invalid (and error was emitted).
///
/// # Safety
///
/// Calls external C functions to emit errors.
#[no_mangle]
pub unsafe extern "C" fn rs_check_range_valid(line1: i64, line2: i64) -> c_int {
    if !lines_in_order(line1, line2) {
        rs_emsg_invrange();
        return 1;
    }
    0
}

// =============================================================================
// Error message buffer utilities
// =============================================================================

/// Maximum length for an error message buffer.
pub const MSG_BUF_LEN: usize = 480;

/// Check if a command disallows a range.
///
/// Returns true if the command flags indicate no range is allowed
/// but a range was provided.
#[inline]
pub const fn range_not_allowed(argt: u32, addr_count: c_int) -> bool {
    // EX_RANGE flag is 0x001 in the C code
    const EX_RANGE: u32 = 0x001;
    (argt & EX_RANGE) == 0 && addr_count > 0
}

/// FFI wrapper for range_not_allowed check.
#[no_mangle]
pub extern "C" fn rs_range_not_allowed(argt: u32, addr_count: c_int) -> c_int {
    c_int::from(range_not_allowed(argt, addr_count))
}

/// Check if a command disallows a range and emit error if one was provided.
///
/// Returns 0 if OK, 1 if range not allowed (error emitted).
///
/// # Safety
///
/// Calls external C functions to emit errors.
#[no_mangle]
pub unsafe extern "C" fn rs_check_no_range(argt: u32, addr_count: c_int) -> c_int {
    if range_not_allowed(argt, addr_count) {
        rs_emsg_norange();
        return 1;
    }
    0
}

// =============================================================================
// Common error patterns
// =============================================================================

/// Check if an argument string is empty.
///
/// An empty argument is NULL, points to NUL, or points to whitespace.
///
/// # Safety
///
/// `arg` must be a valid pointer or NULL.
#[inline]
pub unsafe fn arg_is_empty(arg: *const c_char) -> bool {
    if arg.is_null() {
        return true;
    }
    let c = *arg as u8;
    c == 0 || c == b' ' || c == b'\t'
}

/// FFI wrapper for empty argument check.
///
/// # Safety
///
/// `arg` must be a valid pointer or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_arg_is_empty(arg: *const c_char) -> c_int {
    c_int::from(arg_is_empty(arg))
}

/// Check if there are trailing characters after the command argument.
///
/// Returns true if there are non-whitespace characters after the expected
/// end of the argument.
///
/// # Safety
///
/// `arg` must be a valid C string pointer or NULL.
#[inline]
pub unsafe fn has_trailing_chars(arg: *const c_char) -> bool {
    if arg.is_null() {
        return false;
    }
    let mut p = arg;
    // Skip whitespace
    while (*p as u8) == b' ' || (*p as u8) == b'\t' {
        p = p.add(1);
    }
    // Check for non-terminating characters
    let c = *p as u8;
    c != 0 && c != b'|' && c != b'"' && c != b'\n'
}

/// FFI wrapper for trailing characters check.
///
/// # Safety
///
/// `arg` must be a valid C string pointer or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_has_trailing_chars(arg: *const c_char) -> c_int {
    c_int::from(has_trailing_chars(arg))
}

// =============================================================================
// append_command - Append command text to IObuff for error display
// =============================================================================

/// Append "cmd" to the error message in IObuff.
///
/// Takes care of limiting the length and handling 0xa0, which would be
/// invisible otherwise (displays as `<a0>`).
///
/// Matches C `append_command()`.
///
/// # Safety
///
/// `cmd` must be a valid null-terminated C string.
/// Accesses global `IObuff` via C accessors.
#[export_name = "append_command"]
pub unsafe extern "C" fn rs_append_command(cmd: *const c_char) {
    if cmd.is_null() {
        return;
    }

    let iobuff = nvim_docmd_get_iobuff();
    let iosize = nvim_docmd_get_iosize() as usize;

    let len = c_strlen(iobuff as *const c_char);

    if len > iosize - 100 {
        // Not enough space, truncate and put in "...".
        let mut d = iobuff.add(iosize - 100);
        let head_off = nvim_docmd_utf_head_off(iobuff as *const c_char, d as *const c_char);
        d = d.sub(head_off as usize);
        // Write "..." followed by NUL
        *d = b'.' as c_char;
        *d.add(1) = b'.' as c_char;
        *d.add(2) = b'.' as c_char;
        *d.add(3) = 0;
    }

    // Append ": "
    nvim_docmd_xstrlcat_iobuff(c": ".as_ptr());

    let mut d = iobuff.add(c_strlen(iobuff as *const c_char));
    let mut s = cmd;

    while *s as u8 != 0 && (d as usize - iobuff as usize) + 5 < iosize {
        if *s as u8 == 0xc2 && *s.add(1) as u8 == 0xa0 {
            // Non-breaking space: display as "<a0>"
            s = s.add(2);
            *d = b'<' as c_char;
            *d.add(1) = b'a' as c_char;
            *d.add(2) = b'0' as c_char;
            *d.add(3) = b'>' as c_char;
            d = d.add(4);
        } else if (d as usize - iobuff as usize)
            + nvim_docmd_utfc_ptr2len(s as *const c_char) as usize
            + 1
            >= iosize
        {
            break;
        } else {
            nvim_docmd_mb_copy_char(&mut s as *mut *const c_char, &mut d as *mut *mut c_char);
        }
    }
    *d = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(E_INVARG, 474);
        assert_eq!(E_INVARG2, 475);
        assert_eq!(E_INVRANGE, 16);
        assert_eq!(E_NORANGE, 481);
        assert_eq!(E_TRAILING, 488);
    }

    #[test]
    fn test_lines_in_order() {
        assert!(lines_in_order(1, 5));
        assert!(lines_in_order(1, 1));
        assert!(lines_in_order(0, 0));
        assert!(!lines_in_order(5, 1));
        assert!(!lines_in_order(-1, 5));
        assert!(!lines_in_order(1, -1));
        assert!(!lines_in_order(-1, -1));
    }

    #[test]
    fn test_range_not_allowed() {
        const EX_RANGE: u32 = 0x001;

        // Range allowed by command flags
        assert!(!range_not_allowed(EX_RANGE, 0));
        assert!(!range_not_allowed(EX_RANGE, 1));
        assert!(!range_not_allowed(EX_RANGE, 2));

        // Range not allowed by command flags
        assert!(!range_not_allowed(0, 0)); // No range given, OK
        assert!(range_not_allowed(0, 1)); // Range given but not allowed
        assert!(range_not_allowed(0, 2)); // Range given but not allowed
    }

    #[test]
    fn test_arg_is_empty() {
        use std::ffi::CString;
        use std::ptr;

        unsafe {
            // NULL is empty
            assert!(arg_is_empty(ptr::null()));

            // Empty string is empty
            let empty = CString::new("").unwrap();
            assert!(arg_is_empty(empty.as_ptr()));

            // Whitespace-only is empty
            let space = CString::new(" ").unwrap();
            assert!(arg_is_empty(space.as_ptr()));

            let tab = CString::new("\t").unwrap();
            assert!(arg_is_empty(tab.as_ptr()));

            // Non-empty
            let content = CString::new("hello").unwrap();
            assert!(!arg_is_empty(content.as_ptr()));

            let mixed = CString::new("x ").unwrap();
            assert!(!arg_is_empty(mixed.as_ptr()));
        }
    }

    #[test]
    fn test_has_trailing_chars() {
        use std::ffi::CString;
        use std::ptr;

        unsafe {
            // NULL has no trailing
            assert!(!has_trailing_chars(ptr::null()));

            // Empty string has no trailing
            let empty = CString::new("").unwrap();
            assert!(!has_trailing_chars(empty.as_ptr()));

            // Just whitespace then NUL is not trailing
            let space = CString::new("   ").unwrap();
            assert!(!has_trailing_chars(space.as_ptr()));

            // Pipe is a command separator, not trailing
            let pipe = CString::new("  |").unwrap();
            assert!(!has_trailing_chars(pipe.as_ptr()));

            // Quote starts a comment, not trailing
            let comment = CString::new("  \"").unwrap();
            assert!(!has_trailing_chars(comment.as_ptr()));

            // Newline is not trailing
            let newline = CString::new("  \n").unwrap();
            assert!(!has_trailing_chars(newline.as_ptr()));

            // Actual trailing characters
            let trailing = CString::new("  extra").unwrap();
            assert!(has_trailing_chars(trailing.as_ptr()));
        }
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_lines_in_order(1, 5), 1);
        assert_eq!(rs_lines_in_order(5, 1), 0);

        const EX_RANGE: u32 = 0x001;
        assert_eq!(rs_range_not_allowed(0, 1), 1);
        assert_eq!(rs_range_not_allowed(EX_RANGE, 1), 0);
    }

    #[test]
    fn test_msg_buf_len() {
        assert_eq!(MSG_BUF_LEN, 480);
    }
}
