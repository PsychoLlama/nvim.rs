//! Autocommand state checking for Neovim
//!
//! This module provides Rust implementations for checking autocommand state.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]

pub mod event;
pub mod group;
pub mod pattern;

use std::ffi::{c_char, c_void};
use std::os::raw::c_int;

/// Opaque handle to a Neovim window (`win_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinHandle(*mut c_void);

// Mode constants from state_defs.h
const MODE_NORMAL: c_int = 0x01;
const MODE_NORMAL_BUSY: c_int = 0x1000 | MODE_NORMAL;
const MODE_INSERT: c_int = 0x10;

// Event constants from auevents_enum.generated.h
const EVENT_CURSORHOLD: c_int = 37;
const EVENT_CURSORHOLDI: c_int = 38;
const NUM_EVENTS: c_int = 141;

// C accessors for static data
extern "C" {
    fn nvim_get_autocmd_blocked() -> c_int;
    fn nvim_get_event_name(event: c_int) -> *const c_char;
    fn nvim_get_autocmds_count(event: c_int) -> usize;

    // Accessors for aucmd_win array
    fn nvim_get_aucmd_win_count() -> c_int;
    fn nvim_aucmd_win_used(idx: c_int) -> c_int;
    fn nvim_aucmd_win_get_win(idx: c_int) -> WinHandle;

    // From event crate - get the real editor state
    fn rs_get_real_state() -> c_int;

    // Accessors for trigger_cursorhold
    fn nvim_get_did_cursorhold() -> c_int;
    fn nvim_get_reg_recording() -> c_int;
    fn nvim_get_typebuf_len() -> c_int;

    // From insexpand crate - check if completion is active
    fn rs_ins_compl_active() -> c_int;
}

// Static "Unknown" string for invalid events
static UNKNOWN_EVENT: &[u8] = b"Unknown\0";

/// Check if autocommands are blocked.
///
/// Returns true if autocmd_blocked != 0.
#[no_mangle]
pub unsafe extern "C" fn rs_is_autocmd_blocked() -> c_int {
    c_int::from(nvim_get_autocmd_blocked() != 0)
}

/// Return the name for an event.
///
/// Returns "Unknown" for invalid or out-of-range events.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the program (static data).
#[no_mangle]
pub unsafe extern "C" fn rs_event_nr2name(event: c_int, num_events: c_int) -> *const c_char {
    if event >= 0 && event < num_events {
        let name = nvim_get_event_name(event);
        if !name.is_null() {
            return name;
        }
    }
    UNKNOWN_EVENT.as_ptr().cast()
}

/// Check if there are any autocommands registered for an event.
///
/// Returns 1 if the event has at least one autocommand, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_has_event(event: c_int, num_events: c_int) -> c_int {
    if event >= 0 && event < num_events {
        c_int::from(nvim_get_autocmds_count(event) > 0)
    } else {
        0
    }
}

/// Internal helper to check if an event has autocommands.
fn has_event_impl(event: c_int) -> bool {
    if (0..NUM_EVENTS).contains(&event) {
        unsafe { nvim_get_autocmds_count(event) > 0 }
    } else {
        false
    }
}

/// Check if there is a CursorHold/CursorHoldI autocommand defined for
/// the current mode.
///
/// Returns 1 if there is a cursorhold autocommand, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_has_cursorhold() -> c_int {
    let state = rs_get_real_state();
    let event = if state == MODE_NORMAL_BUSY {
        EVENT_CURSORHOLD
    } else {
        EVENT_CURSORHOLDI
    };
    c_int::from(has_event_impl(event))
}

/// Check if the CursorHold/CursorHoldI event can be triggered.
///
/// Returns 1 if the event can be triggered, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_trigger_cursorhold() -> c_int {
    // Check preconditions: cursorhold not yet triggered, has autocommand,
    // not recording, type-ahead buffer empty, and completion not active
    if nvim_get_did_cursorhold() != 0
        || rs_has_cursorhold() == 0
        || nvim_get_reg_recording() != 0
        || nvim_get_typebuf_len() != 0
        || rs_ins_compl_active() != 0
    {
        return 0;
    }

    // Check if we're in the right mode (normal-busy or insert)
    let state = rs_get_real_state();
    if state == MODE_NORMAL_BUSY || (state & MODE_INSERT) != 0 {
        return 1;
    }
    0
}

/// Check if "win" is an active entry in the aucmd_win array.
///
/// Returns 1 if the window is found in the autocmd window array and is in use, 0 otherwise.
///
/// # Safety
/// The `win` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_is_aucmd_win(win: WinHandle) -> c_int {
    let count = nvim_get_aucmd_win_count();
    for i in 0..count {
        if nvim_aucmd_win_used(i) != 0 && nvim_aucmd_win_get_win(i) == win {
            return 1;
        }
    }
    0
}

/// Returns the length of the first pattern in a comma-separated pattern list.
///
/// Handles brace groups like `*.{obj,o}` where the comma is not a separator.
/// Returns 0 if the pattern is empty (NUL).
///
/// # Safety
/// `pat` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_aucmd_pattern_length(pat: *const c_char) -> usize {
    if pat.is_null() {
        return 0;
    }

    let mut p = pat;

    // Check for empty string
    if *p == 0 {
        return 0;
    }

    loop {
        let endpat_start = p;

        // Ignore single comma at start
        if *p == b',' as c_char {
            p = p.add(1);
            if *p == 0 {
                break;
            }
            continue;
        }

        // Find end of the pattern, watching for comma in braces
        let mut endpat = p;
        let mut brace_level = 0i32;

        loop {
            let c = *endpat;
            if c == 0 {
                break;
            }
            if c == b',' as c_char && brace_level == 0 {
                // Check if previous char was backslash (escaped comma)
                if endpat > endpat_start && *endpat.sub(1) != b'\\' as c_char {
                    break;
                }
            }
            if c == b'{' as c_char {
                brace_level += 1;
            } else if c == b'}' as c_char {
                brace_level -= 1;
            }
            endpat = endpat.add(1);
        }

        // Return length of this pattern segment
        return endpat.offset_from(p) as usize;
    }

    // Fallback: return strlen of remaining pattern
    let mut len = 0usize;
    while *p.add(len) != 0 {
        len += 1;
    }
    len
}

/// Returns a pointer to the next pattern in a comma-separated pattern list.
///
/// Given a pattern `pat` and its length `patlen`, returns a pointer to the
/// start of the next pattern (skipping the comma separator if present).
///
/// # Safety
/// `pat` must be a valid pointer within a NUL-terminated C string, and
/// `patlen` must not exceed the remaining length of the string.
#[no_mangle]
pub unsafe extern "C" fn rs_aucmd_next_pattern(pat: *const c_char, patlen: usize) -> *const c_char {
    let mut p = pat.add(patlen);
    if *p == b',' as c_char {
        p = p.add(1);
    }
    p
}

/// Checks if an autocommand pattern is buffer-local.
///
/// A pattern is buffer-local if it starts with "<buffer" and ends with ">".
/// Examples: "<buffer>", "<buffer=1>", "<buffer=abuf>"
///
/// # Safety
/// `pat` must be a valid pointer to at least `patlen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_aupat_is_buflocal(pat: *const c_char, patlen: c_int) -> c_int {
    if pat.is_null() || patlen < 8 {
        return 0;
    }

    let patlen = patlen as usize;

    // Check starts with "<buffer" (7 chars)
    let buffer_prefix = b"<buffer";
    for (i, &expected) in buffer_prefix.iter().enumerate() {
        let c = *pat.add(i) as u8;
        // Case-insensitive comparison for 'b', 'u', 'f', 'e', 'r'
        if i == 0 {
            if c != b'<' {
                return 0;
            }
        } else if c.to_ascii_lowercase() != expected {
            return 0;
        }
    }

    // Check ends with ">"
    let last = *pat.add(patlen - 1) as u8;
    c_int::from(last == b'>')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_aucmd_pattern_length() {
        unsafe {
            // Empty pattern
            let empty = CString::new("").unwrap();
            assert_eq!(rs_aucmd_pattern_length(empty.as_ptr()), 0);

            // Simple pattern
            let simple = CString::new("*.c").unwrap();
            assert_eq!(rs_aucmd_pattern_length(simple.as_ptr()), 3);

            // Pattern with comma
            let with_comma = CString::new("*.c,*.h").unwrap();
            assert_eq!(rs_aucmd_pattern_length(with_comma.as_ptr()), 3);

            // Pattern with braces containing comma
            let with_braces = CString::new("*.{c,h}").unwrap();
            assert_eq!(rs_aucmd_pattern_length(with_braces.as_ptr()), 7);
        }
    }

    #[test]
    fn test_aucmd_next_pattern() {
        unsafe {
            let patterns = CString::new("*.c,*.h").unwrap();
            let ptr = patterns.as_ptr();

            // First pattern is "*.c" (length 3)
            let next = rs_aucmd_next_pattern(ptr, 3);
            // Should point to "*.h"
            assert_eq!(*next, b'*' as c_char);
        }
    }

    #[test]
    fn test_aupat_is_buflocal() {
        unsafe {
            // Valid buffer-local patterns
            let buf = CString::new("<buffer>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(buf.as_ptr(), 8), 1);

            let buf_eq = CString::new("<buffer=1>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(buf_eq.as_ptr(), 10), 1);

            let buf_abuf = CString::new("<buffer=abuf>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(buf_abuf.as_ptr(), 13), 1);

            // Case insensitive
            let buf_upper = CString::new("<BUFFER>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(buf_upper.as_ptr(), 8), 1);

            // Invalid patterns
            let short = CString::new("<buf>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(short.as_ptr(), 5), 0);

            let no_end = CString::new("<buffer").unwrap();
            assert_eq!(rs_aupat_is_buflocal(no_end.as_ptr(), 7), 0);

            let wrong_start = CString::new("buffer>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(wrong_start.as_ptr(), 7), 0);

            let normal = CString::new("*.c").unwrap();
            assert_eq!(rs_aupat_is_buflocal(normal.as_ptr(), 3), 0);
        }
    }

    #[test]
    fn test_aucmd_pattern_length_escaped_comma() {
        unsafe {
            // Escaped comma should not be treated as separator
            let escaped = CString::new("*.\\,c,*.h").unwrap();
            // Length should include the escaped comma
            assert_eq!(rs_aucmd_pattern_length(escaped.as_ptr()), 5);
        }
    }

    #[test]
    fn test_aucmd_pattern_length_nested_braces() {
        unsafe {
            // Nested braces pattern
            let nested = CString::new("*.{{a,b},{c,d}}").unwrap();
            assert_eq!(rs_aucmd_pattern_length(nested.as_ptr()), 15);
        }
    }

    #[test]
    fn test_aucmd_pattern_length_null() {
        unsafe {
            // Null pointer should return 0
            assert_eq!(rs_aucmd_pattern_length(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_aupat_is_buflocal_null() {
        unsafe {
            // Null pointer should return 0
            assert_eq!(rs_aupat_is_buflocal(std::ptr::null(), 8), 0);
        }
    }

    #[test]
    fn test_unknown_event_string() {
        // Verify UNKNOWN_EVENT is properly null-terminated
        assert!(UNKNOWN_EVENT.ends_with(&[0]));
        assert_eq!(&UNKNOWN_EVENT[..7], b"Unknown");
    }

    #[test]
    fn test_mode_constants() {
        // Verify mode constants match expected values from state_defs.h
        assert_eq!(MODE_NORMAL, 0x01);
        assert_eq!(MODE_NORMAL_BUSY, 0x1001);
        assert_eq!(MODE_INSERT, 0x10);
    }

    #[test]
    fn test_event_constants() {
        // Verify event constants match expected values from auevents_enum.generated.h
        assert_eq!(EVENT_CURSORHOLD, 37);
        assert_eq!(EVENT_CURSORHOLDI, 38);
        assert_eq!(NUM_EVENTS, 141);
    }
}
