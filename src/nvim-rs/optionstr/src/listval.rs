//! Comma-separated list option handling
//!
//! This module provides utilities for validating and parsing comma-separated
//! list options like 'sessionoptions', 'backupcopy', etc.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_char, c_int};

// =============================================================================
// List Value Parsing
// =============================================================================

/// State for iterating over comma-separated list values
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ListValueIter {
    /// Current position in the string
    pub pos: *const c_char,
    /// Start of current value
    pub value_start: *const c_char,
    /// Length of current value
    pub value_len: usize,
    /// Whether we're at end
    pub at_end: bool,
}

/// Initialize a list value iterator
///
/// # Safety
/// The `s` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_list_value_iter_init(s: *const c_char) -> ListValueIter {
    if s.is_null() || *s == 0 {
        return ListValueIter {
            pos: s,
            value_start: s,
            value_len: 0,
            at_end: true,
        };
    }

    // Find first value
    let mut end = s;
    while *end != 0 && *end != b',' as c_char {
        end = end.add(1);
    }

    ListValueIter {
        pos: if *end == 0 { end } else { end.add(1) },
        value_start: s,
        value_len: end.offset_from(s) as usize,
        at_end: false,
    }
}

/// Advance to the next value in the list
///
/// # Safety
/// The iterator must have been initialized with a valid string.
#[no_mangle]
pub unsafe extern "C" fn rs_list_value_iter_next(iter: *mut ListValueIter) -> bool {
    if iter.is_null() || (*iter).at_end || (*iter).pos.is_null() || *(*iter).pos == 0 {
        if !iter.is_null() {
            (*iter).at_end = true;
        }
        return false;
    }

    let start = (*iter).pos;
    let mut end = start;
    while *end != 0 && *end != b',' as c_char {
        end = end.add(1);
    }

    (*iter).value_start = start;
    (*iter).value_len = end.offset_from(start) as usize;
    (*iter).pos = if *end == 0 { end } else { end.add(1) };
    (*iter).at_end = (*iter).value_len == 0 && *end == 0;

    !(*iter).at_end
}

/// Check if a value exists in a comma-separated list
///
/// # Safety
/// The `list` and `value` pointers must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_list_value_contains(
    list: *const c_char,
    value: *const c_char,
    value_len: usize,
) -> bool {
    if list.is_null() || value.is_null() || value_len == 0 {
        return false;
    }

    let mut iter = rs_list_value_iter_init(list);
    if iter.at_end {
        return false;
    }

    loop {
        if iter.value_len == value_len {
            // Compare values
            let mut match_found = true;
            for i in 0..value_len {
                if *iter.value_start.add(i) != *value.add(i) {
                    match_found = false;
                    break;
                }
            }
            if match_found {
                return true;
            }
        }

        if !rs_list_value_iter_next(&mut iter) {
            break;
        }
    }

    false
}

/// Count values in a comma-separated list
///
/// # Safety
/// The `list` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_list_value_count(list: *const c_char) -> c_int {
    if list.is_null() || *list == 0 {
        return 0;
    }

    let mut count = 0;
    let mut iter = rs_list_value_iter_init(list);

    if iter.at_end {
        return 0;
    }

    count += 1;
    while rs_list_value_iter_next(&mut iter) {
        count += 1;
    }

    count
}

/// Check if list has empty values (consecutive commas)
///
/// # Safety
/// The `list` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_list_has_empty_values(list: *const c_char) -> bool {
    if list.is_null() {
        return false;
    }

    let mut prev_was_comma = false;
    let mut ptr = list;

    // Check for leading comma
    if *ptr == b',' as c_char {
        return true;
    }

    while *ptr != 0 {
        if *ptr == b',' as c_char {
            if prev_was_comma {
                return true;
            }
            prev_was_comma = true;
        } else {
            prev_was_comma = false;
        }
        ptr = ptr.add(1);
    }

    // Check for trailing comma
    if prev_was_comma {
        return true;
    }

    false
}

/// Check if list has duplicate values
///
/// # Safety
/// The `list` pointer must be valid for reading up to the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_list_has_duplicate_values(list: *const c_char) -> bool {
    if list.is_null() || *list == 0 {
        return false;
    }

    // Simple O(n²) check - for small lists this is fine
    let mut outer = rs_list_value_iter_init(list);
    if outer.at_end {
        return false;
    }

    loop {
        // Compare with all following values
        let mut inner = ListValueIter {
            pos: outer.pos,
            value_start: outer.pos,
            value_len: 0,
            at_end: false,
        };

        // Initialize inner to next value
        if *inner.pos != 0 && rs_list_value_iter_next(&mut inner) {
            loop {
                if outer.value_len == inner.value_len {
                    let mut match_found = true;
                    for i in 0..outer.value_len {
                        if *outer.value_start.add(i) != *inner.value_start.add(i) {
                            match_found = false;
                            break;
                        }
                    }
                    if match_found {
                        return true;
                    }
                }

                if !rs_list_value_iter_next(&mut inner) {
                    break;
                }
            }
        }

        if !rs_list_value_iter_next(&mut outer) {
            break;
        }
    }

    false
}

// =============================================================================
// List Value Modification
// =============================================================================

/// Add a value to a comma-separated list
///
/// Returns the number of bytes written (including null terminator), or 0 if failed.
///
/// # Safety
/// The `list` and `value` pointers must be valid for reading.
/// The `out` pointer must be valid for writing up to `out_size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_list_value_add(
    list: *const c_char,
    value: *const c_char,
    value_len: usize,
    out: *mut c_char,
    out_size: usize,
) -> usize {
    if out.is_null() || out_size == 0 || value.is_null() || value_len == 0 {
        return 0;
    }

    let mut pos = 0;

    // Copy existing list
    if !list.is_null() && *list != 0 {
        let mut ptr = list;
        while *ptr != 0 {
            if pos + 2 + value_len > out_size {
                return 0; // Not enough space
            }
            *out.add(pos) = *ptr;
            pos += 1;
            ptr = ptr.add(1);
        }

        // Add comma separator
        if pos + 1 + value_len > out_size {
            return 0;
        }
        *out.add(pos) = b',' as c_char;
        pos += 1;
    }

    // Add new value
    if pos + value_len + 1 > out_size {
        return 0;
    }
    for i in 0..value_len {
        *out.add(pos) = *value.add(i);
        pos += 1;
    }
    *out.add(pos) = 0;

    pos + 1
}

/// Remove a value from a comma-separated list
///
/// Returns the number of bytes written (including null terminator).
///
/// # Safety
/// The `list` and `value` pointers must be valid for reading.
/// The `out` pointer must be valid for writing up to `out_size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_list_value_remove(
    list: *const c_char,
    value: *const c_char,
    value_len: usize,
    out: *mut c_char,
    out_size: usize,
) -> usize {
    if out.is_null() || out_size == 0 {
        return 0;
    }

    if list.is_null() || *list == 0 {
        *out = 0;
        return 1;
    }

    if value.is_null() || value_len == 0 {
        // Just copy the list
        let mut ptr = list;
        let mut pos = 0;
        while *ptr != 0 && pos + 1 < out_size {
            *out.add(pos) = *ptr;
            pos += 1;
            ptr = ptr.add(1);
        }
        *out.add(pos) = 0;
        return pos + 1;
    }

    let mut iter = rs_list_value_iter_init(list);
    let mut pos = 0;
    let mut first = true;

    loop {
        // Check if this value matches the one to remove
        let mut matches = iter.value_len == value_len;
        if matches {
            for i in 0..value_len {
                if *iter.value_start.add(i) != *value.add(i) {
                    matches = false;
                    break;
                }
            }
        }

        if !matches {
            // Add comma if not first
            if !first {
                if pos + 1 >= out_size {
                    break;
                }
                *out.add(pos) = b',' as c_char;
                pos += 1;
            }

            // Copy value
            for i in 0..iter.value_len {
                if pos + 1 >= out_size {
                    break;
                }
                *out.add(pos) = *iter.value_start.add(i);
                pos += 1;
            }
            first = false;
        }

        if iter.at_end || !rs_list_value_iter_next(&mut iter) {
            break;
        }
    }

    *out.add(pos) = 0;
    pos + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_value_iter() {
        unsafe {
            let list = b"one,two,three\0".as_ptr().cast::<c_char>();

            let mut iter = rs_list_value_iter_init(list);
            assert!(!iter.at_end);
            assert_eq!(iter.value_len, 3);

            assert!(rs_list_value_iter_next(&mut iter));
            assert_eq!(iter.value_len, 3);

            assert!(rs_list_value_iter_next(&mut iter));
            assert_eq!(iter.value_len, 5);

            assert!(!rs_list_value_iter_next(&mut iter));
        }
    }

    #[test]
    fn test_list_value_contains() {
        unsafe {
            let list = b"one,two,three\0".as_ptr().cast::<c_char>();

            assert!(rs_list_value_contains(list, b"one\0".as_ptr().cast(), 3));
            assert!(rs_list_value_contains(list, b"two\0".as_ptr().cast(), 3));
            assert!(rs_list_value_contains(list, b"three\0".as_ptr().cast(), 5));
            assert!(!rs_list_value_contains(list, b"four\0".as_ptr().cast(), 4));
        }
    }

    #[test]
    fn test_list_value_count() {
        unsafe {
            assert_eq!(rs_list_value_count(b"one,two,three\0".as_ptr().cast()), 3);
            assert_eq!(rs_list_value_count(b"one\0".as_ptr().cast()), 1);
            assert_eq!(rs_list_value_count(b"\0".as_ptr().cast()), 0);
        }
    }

    #[test]
    fn test_list_has_empty_values() {
        unsafe {
            assert!(!rs_list_has_empty_values(
                b"one,two,three\0".as_ptr().cast()
            ));
            assert!(rs_list_has_empty_values(b"one,,three\0".as_ptr().cast()));
            assert!(rs_list_has_empty_values(b",two,three\0".as_ptr().cast()));
            assert!(rs_list_has_empty_values(b"one,two,\0".as_ptr().cast()));
        }
    }

    #[test]
    fn test_list_has_duplicate_values() {
        unsafe {
            assert!(!rs_list_has_duplicate_values(
                b"one,two,three\0".as_ptr().cast()
            ));
            assert!(rs_list_has_duplicate_values(
                b"one,two,one\0".as_ptr().cast()
            ));
        }
    }

    #[test]
    fn test_list_value_add() {
        unsafe {
            let mut buf = [0i8; 20];

            let len = rs_list_value_add(
                b"one,two\0".as_ptr().cast(),
                b"three\0".as_ptr().cast(),
                5,
                buf.as_mut_ptr(),
                buf.len(),
            );
            assert!(len > 0);

            // Verify result starts with "one,two,three"
            let result = std::ffi::CStr::from_ptr(buf.as_ptr());
            assert_eq!(result.to_str().unwrap(), "one,two,three");
        }
    }

    #[test]
    fn test_list_value_remove() {
        unsafe {
            let mut buf = [0i8; 20];

            let len = rs_list_value_remove(
                b"one,two,three\0".as_ptr().cast(),
                b"two\0".as_ptr().cast(),
                3,
                buf.as_mut_ptr(),
                buf.len(),
            );
            assert!(len > 0);

            let result = std::ffi::CStr::from_ptr(buf.as_ptr());
            assert_eq!(result.to_str().unwrap(), "one,three");
        }
    }
}
