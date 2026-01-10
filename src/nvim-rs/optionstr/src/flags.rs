//! Flag list option handling
//!
//! This module provides utilities for validating single-character flag lists
//! used by options like 'shortmess', 'formatoptions', 'cpoptions', etc.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_char, c_int};

// =============================================================================
// Flag List Validation
// =============================================================================

/// Validate a flag list string against allowed flags
///
/// # Safety
/// The `value` pointer must be valid for reading up to the null terminator.
/// The `flags` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_flag_list(
    value: *const c_char,
    flags: *const c_char,
) -> c_int {
    if value.is_null() || flags.is_null() {
        return -1;
    }

    let mut val_ptr = value;
    while *val_ptr != 0 {
        let c = *val_ptr;

        // Check if this character is in the allowed flags
        let mut flag_ptr = flags;
        let mut found = false;
        while *flag_ptr != 0 {
            if *flag_ptr == c {
                found = true;
                break;
            }
            flag_ptr = flag_ptr.add(1);
        }

        if !found {
            return c_int::from(c);
        }

        val_ptr = val_ptr.add(1);
    }

    0 // All characters are valid
}

/// Check if a flag list contains a specific flag
///
/// # Safety
/// The `value` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_flag_list_contains(value: *const c_char, flag: c_int) -> bool {
    if value.is_null() || !(0..=127).contains(&flag) {
        return false;
    }

    let flag_char = flag as c_char;
    let mut ptr = value;
    while *ptr != 0 {
        if *ptr == flag_char {
            return true;
        }
        ptr = ptr.add(1);
    }

    false
}

/// Count flags in a flag list
///
/// # Safety
/// The `value` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_flag_list_count(value: *const c_char) -> c_int {
    if value.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut ptr = value;
    while *ptr != 0 {
        count += 1;
        ptr = ptr.add(1);
    }

    count
}

/// Check if flag list has duplicate flags
///
/// # Safety
/// The `value` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_flag_list_has_duplicates(value: *const c_char) -> bool {
    if value.is_null() {
        return false;
    }

    // Use a bitmap for ASCII characters
    let mut seen = [false; 128];

    let mut ptr = value;
    while *ptr != 0 {
        let c = *ptr as u8;
        if c < 128 {
            if seen[c as usize] {
                return true;
            }
            seen[c as usize] = true;
        }
        ptr = ptr.add(1);
    }

    false
}

/// Add a flag to a flag list (at end)
///
/// Returns the number of bytes written (including null terminator), or 0 if failed.
///
/// # Safety
/// The `value` pointer must be valid for reading up to the null terminator.
/// The `out` pointer must be valid for writing up to `out_size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_flag_list_add(
    value: *const c_char,
    flag: c_int,
    out: *mut c_char,
    out_size: usize,
) -> usize {
    if out.is_null() || out_size == 0 || !(0..=127).contains(&flag) {
        return 0;
    }

    let mut pos = 0;

    // Copy existing flags
    if !value.is_null() {
        let mut ptr = value;
        while *ptr != 0 {
            if pos + 2 > out_size {
                return 0; // Not enough space
            }
            *out.add(pos) = *ptr;
            pos += 1;
            ptr = ptr.add(1);
        }
    }

    // Add new flag
    if pos + 2 > out_size {
        return 0; // Not enough space
    }
    *out.add(pos) = flag as c_char;
    pos += 1;
    *out.add(pos) = 0;

    pos + 1
}

/// Remove a flag from a flag list
///
/// Returns the number of bytes written (including null terminator).
///
/// # Safety
/// The `value` pointer must be valid for reading up to the null terminator.
/// The `out` pointer must be valid for writing up to `out_size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_flag_list_remove(
    value: *const c_char,
    flag: c_int,
    out: *mut c_char,
    out_size: usize,
) -> usize {
    if out.is_null() || out_size == 0 {
        return 0;
    }

    if value.is_null() {
        *out = 0;
        return 1;
    }

    let flag_char = if (0..=127).contains(&flag) {
        flag as c_char
    } else {
        // Invalid flag, just copy input
        let mut ptr = value;
        let mut pos = 0;
        while *ptr != 0 && pos + 1 < out_size {
            *out.add(pos) = *ptr;
            pos += 1;
            ptr = ptr.add(1);
        }
        *out.add(pos) = 0;
        return pos + 1;
    };

    let mut ptr = value;
    let mut pos = 0;

    while *ptr != 0 {
        if *ptr != flag_char {
            if pos + 2 > out_size {
                *out.add(pos) = 0;
                return pos + 1;
            }
            *out.add(pos) = *ptr;
            pos += 1;
        }
        ptr = ptr.add(1);
    }

    *out.add(pos) = 0;
    pos + 1
}

// =============================================================================
// Specific Flag Options
// =============================================================================

/// Shortmess flag helpers
pub mod shortmess {
    use std::ffi::c_char;

    /// Check if RO flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_shm_has_ro(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b'r'))
    }

    /// Check if MODIFIED flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_shm_has_mod(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b'm'))
    }

    /// Check if ATTENTION flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_shm_has_attention(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b'A'))
    }

    /// Check if INTRO flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_shm_has_intro(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b'I'))
    }

    /// Check if SEARCH flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_shm_has_search(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b's'))
    }

    /// Check if FILEINFO flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_shm_has_fileinfo(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b'F'))
    }
}

/// Formatoptions flag helpers
pub mod formatoptions {
    use std::ffi::c_char;

    /// Check if auto-wrap text flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_fo_has_wrap(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b't'))
    }

    /// Check if auto-wrap comments flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_fo_has_wrap_coms(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b'c'))
    }

    /// Check if auto-format flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_fo_has_auto(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b'a'))
    }

    /// Check if numbered lists flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_fo_has_numbered(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b'n'))
    }

    /// Check if remove comment leader flag is set
    ///
    /// # Safety
    /// The `value` pointer must be valid for reading up to the null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn rs_fo_has_remove_coms(value: *const c_char) -> bool {
        super::rs_flag_list_contains(value, i32::from(b'j'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_flag_list() {
        unsafe {
            let flags = b"abc\0".as_ptr().cast::<c_char>();

            assert_eq!(rs_validate_flag_list(b"abc\0".as_ptr().cast(), flags), 0);
            assert_eq!(rs_validate_flag_list(b"aab\0".as_ptr().cast(), flags), 0);
            assert_eq!(
                rs_validate_flag_list(b"abcd\0".as_ptr().cast(), flags),
                i32::from(b'd')
            );
            assert_eq!(rs_validate_flag_list(b"\0".as_ptr().cast(), flags), 0);
        }
    }

    #[test]
    fn test_flag_list_contains() {
        unsafe {
            let value = b"abc\0".as_ptr().cast::<c_char>();

            assert!(rs_flag_list_contains(value, i32::from(b'a')));
            assert!(rs_flag_list_contains(value, i32::from(b'b')));
            assert!(!rs_flag_list_contains(value, i32::from(b'd')));
            assert!(!rs_flag_list_contains(std::ptr::null(), i32::from(b'a')));
        }
    }

    #[test]
    fn test_flag_list_count() {
        unsafe {
            assert_eq!(rs_flag_list_count(b"abc\0".as_ptr().cast()), 3);
            assert_eq!(rs_flag_list_count(b"\0".as_ptr().cast()), 0);
            assert_eq!(rs_flag_list_count(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_flag_list_has_duplicates() {
        unsafe {
            assert!(!rs_flag_list_has_duplicates(b"abc\0".as_ptr().cast()));
            assert!(rs_flag_list_has_duplicates(b"abca\0".as_ptr().cast()));
            assert!(!rs_flag_list_has_duplicates(b"\0".as_ptr().cast()));
        }
    }

    #[test]
    fn test_flag_list_add() {
        unsafe {
            let mut buf = [0i8; 10];

            let len = rs_flag_list_add(
                b"ab\0".as_ptr().cast(),
                i32::from(b'c'),
                buf.as_mut_ptr(),
                buf.len(),
            );
            assert_eq!(len, 4);
            assert_eq!(&buf[..4], &[b'a' as i8, b'b' as i8, b'c' as i8, 0]);
        }
    }

    #[test]
    fn test_flag_list_remove() {
        unsafe {
            let mut buf = [0i8; 10];

            let len = rs_flag_list_remove(
                b"abc\0".as_ptr().cast(),
                i32::from(b'b'),
                buf.as_mut_ptr(),
                buf.len(),
            );
            assert_eq!(len, 3);
            assert_eq!(&buf[..3], &[b'a' as i8, b'c' as i8, 0]);
        }
    }
}
