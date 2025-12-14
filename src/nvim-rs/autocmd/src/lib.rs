//! Autocommand state checking for Neovim
//!
//! This module provides Rust implementations for checking autocommand state.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::c_char;
use std::os::raw::c_int;

// C accessor for the static autocmd_blocked variable
extern "C" {
    fn nvim_get_autocmd_blocked() -> c_int;
}

/// Check if autocommands are blocked.
///
/// Returns true if autocmd_blocked != 0.
#[no_mangle]
pub unsafe extern "C" fn rs_is_autocmd_blocked() -> c_int {
    c_int::from(nvim_get_autocmd_blocked() != 0)
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
}
