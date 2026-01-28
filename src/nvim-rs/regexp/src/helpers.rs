//! Helper functions for regex matching.
//!
//! This module provides case-insensitive string comparison functions
//! used by both the BT and NFA regex engines.
//!
//! # Key Functions
//!
//! - [`cstrchr`]: Case-insensitive character search in string
//! - [`cstrncmp`]: Case-insensitive string comparison with length tracking
//!
//! These functions integrate with the global `rex` state (via exec_state)
//! for case-sensitivity settings.

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::decompose::mb_decompose;

// =============================================================================
// FFI declarations for C functions we need
// =============================================================================

extern "C" {
    /// Get the current ignore-case setting from rex.
    fn nvim_rex_get_reg_ic() -> bool;

    /// Get the ignore-combining-characters setting from rex.
    fn nvim_rex_get_reg_icombine() -> bool;

    /// Case-fold a Unicode character (lowercase).
    fn utf_fold(a: c_int) -> c_int;

    /// Search for character in string.
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    /// Get the byte length of a UTF-8 character (including composing chars).
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    /// Get the UTF-8 codepoint at a pointer.
    fn utf_ptr2char(p: *const c_char) -> c_int;

    /// Case-insensitive string comparison for UTF-8 strings.
    /// Compares at most n1 bytes of s1 and n2 bytes of s2.
    fn utf_strnicmp(s1: *const c_char, s2: *const c_char, n1: usize, n2: usize) -> c_int;

    /// Advance pointer by one UTF-8 character (skip composing chars).
    fn mb_ptr2char_adv(pp: *mut *const c_char) -> c_int;
}

// =============================================================================
// ASCII character class macros (matching C macros)
// =============================================================================

/// Check if character is ASCII uppercase.
#[inline]
fn ascii_isupper(c: u8) -> bool {
    c.is_ascii_uppercase()
}

/// Check if character is ASCII lowercase.
#[inline]
fn ascii_islower(c: u8) -> bool {
    c.is_ascii_lowercase()
}

/// Convert ASCII uppercase to lowercase.
#[inline]
fn tolower_asc(c: u8) -> u8 {
    if ascii_isupper(c) {
        c + (b'a' - b'A')
    } else {
        c
    }
}

/// Convert ASCII lowercase to uppercase.
#[inline]
fn toupper_asc(c: u8) -> u8 {
    if ascii_islower(c) {
        c - (b'a' - b'A')
    } else {
        c
    }
}

// =============================================================================
// Case-insensitive string search
// =============================================================================

/// Search for a character in a string, accounting for case-insensitive matching.
///
/// This is a Rust implementation of the C `cstrchr` function. It handles:
/// - Case-insensitive matching when `rex.reg_ic` is set
/// - ASCII and Unicode case folding
///
/// # Safety
///
/// `s` must be a valid NUL-terminated string pointer.
///
/// # Returns
///
/// Pointer to the found character, or null if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_cstrchr(s: *const c_char, c: c_int) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }

    // Fast path: case-sensitive matching
    if !nvim_rex_get_reg_ic() {
        return vim_strchr(s, c);
    }

    // Compute case variants
    let (cc, lc) = if c > 0x80 {
        // Unicode character: use utf_fold
        let folded = utf_fold(c);
        (folded, folded)
    } else if ascii_isupper(c as u8) {
        // ASCII uppercase: also search for lowercase
        let lower = tolower_asc(c as u8) as c_int;
        (lower, lower)
    } else if ascii_islower(c as u8) {
        // ASCII lowercase: also search for uppercase
        let upper = toupper_asc(c as u8) as c_int;
        (upper, c)
    } else {
        // Non-alphabetic: exact match only
        return vim_strchr(s, c);
    };

    // Search through the string
    let mut p = s;
    while unsafe { *p } != 0 {
        let cur = *p as u8;

        if cur < 0x80 {
            // ASCII character
            if cur as c_int == c || cur as c_int == cc {
                return p as *mut c_char;
            }
            p = p.add(1);
        } else {
            // UTF-8 character
            let uc = utf_ptr2char(p);
            let folded = utf_fold(uc);

            // Compare with folded version of target
            if uc == c || folded == lc {
                return p as *mut c_char;
            }

            // Advance by character length
            let len = utfc_ptr2len(p);
            if len <= 0 {
                break;
            }
            p = p.add(len as usize);
        }
    }

    ptr::null_mut()
}

// =============================================================================
// Case-insensitive string comparison
// =============================================================================

/// Compare two strings with case-insensitivity and combining character handling.
///
/// This is a Rust implementation of the C `cstrncmp` function. It handles:
/// - Case-insensitive comparison when `rex.reg_ic` is set
/// - Ignoring combining characters when `rex.reg_icombine` is set
/// - Adjusting the length `*n` when character lengths differ
///
/// # Safety
///
/// `s1`, `s2`, and `n` must be valid pointers.
///
/// # Returns
///
/// - 0 if strings match (up to `*n` bytes)
/// - Non-zero if strings differ
/// - `*n` may be modified to reflect actual match length
#[no_mangle]
pub unsafe extern "C" fn rs_cstrncmp(s1: *const c_char, s2: *const c_char, n: *mut c_int) -> c_int {
    if s1.is_null() || s2.is_null() || n.is_null() {
        return 1; // Non-match for invalid inputs
    }

    let len = *n;
    if len <= 0 {
        return 0; // Empty comparison always matches
    }

    // Fast path: case-sensitive comparison
    if !nvim_rex_get_reg_ic() {
        return libc::strncmp(s1, s2, len as usize);
    }

    // Count the number of characters (not bytes) in s1 for len bytes
    let mut p = s1;
    let mut n2 = 0;
    let mut n1 = len;

    while n1 > 0 && *p != 0 {
        n1 -= utfc_ptr2len(s1);
        // Advance p by one character (skip composing)
        let char_len = utfc_ptr2len(p);
        if char_len <= 0 {
            break;
        }
        p = p.add(char_len as usize);
        n2 += 1;
    }

    // Count bytes for same number of characters in s2
    p = s2;
    while n2 > 0 && *p != 0 {
        let char_len = utfc_ptr2len(p);
        if char_len <= 0 {
            break;
        }
        p = p.add(char_len as usize);
        n2 -= 1;
    }

    let s2_len = p.offset_from(s2) as usize;

    // Do case-insensitive comparison
    let result = utf_strnicmp(s1, s2, len as usize, s2_len);

    // Adjust n if s2 is shorter
    if result == 0 && (s2_len as c_int) < len {
        *n = s2_len as c_int;
    }

    // Handle combining character ignoring
    if result != 0 && nvim_rex_get_reg_icombine() {
        return cstrncmp_with_decompose(s1, s2, n);
    }

    result
}

/// Helper for cstrncmp that handles combining character decomposition.
///
/// # Safety
///
/// `s1`, `s2`, and `n` must be valid pointers.
unsafe fn cstrncmp_with_decompose(s1: *const c_char, s2: *const c_char, n: *mut c_int) -> c_int {
    let mut str1 = s1;
    let mut str2 = s2;
    let mut c1;
    let mut c2;

    while (str1.offset_from(s1) as c_int) < *n {
        c1 = mb_ptr2char_adv(&mut str1);
        c2 = mb_ptr2char_adv(&mut str2);

        // Check if characters match (with case folding)
        if c1 != c2 && (!nvim_rex_get_reg_ic() || utf_fold(c1) != utf_fold(c2)) {
            // Try decomposition - returns (base, compose1, compose2)
            let (c11, _, _) = mb_decompose(c1);
            let (c12, _, _) = mb_decompose(c2);

            c1 = c11;
            c2 = c12;

            if c11 != c12 && (!nvim_rex_get_reg_ic() || utf_fold(c11) != utf_fold(c12)) {
                return c2 - c1;
            }
        }
    }

    // Match found
    *n = str2.offset_from(s2) as c_int;
    0
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    // Note: These tests would need proper initialization of the rex state,
    // which isn't available in pure Rust tests. Integration tests should
    // verify the behavior through the full C/Rust interface.
}
