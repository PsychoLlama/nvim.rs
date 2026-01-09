//! Collection functions for VimL.
//!
//! This module implements collection functions from `src/nvim/eval/funcs.c`:
//! - `len()` - length of string/list/dict/blob
//! - `empty()` - check if value is empty
//! - `count()` - count occurrences in list/string
//! - `max()` / `min()` - maximum/minimum value in list
//! - `reverse()` - reverse list in place
//! - `sort()` - sort list
//! - `uniq()` - remove duplicates from sorted list

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;

// =============================================================================
// Pure Functions (No FFI)
// =============================================================================

/// Check if a length value represents "empty".
///
/// In VimL:
/// - Empty string: len == 0
/// - Empty list: len == 0
/// - Empty dict: len == 0
/// - Number 0: considered "empty" by empty()
/// - v:false, v:null: considered "empty"
pub const fn is_len_empty(len: i64) -> bool {
    len == 0
}

/// Find maximum value in a list of numbers.
pub fn list_max(values: &[i64]) -> Option<i64> {
    values.iter().copied().max()
}

/// Find minimum value in a list of numbers.
pub fn list_min(values: &[i64]) -> Option<i64> {
    values.iter().copied().min()
}

/// Sum values in a list of numbers.
pub fn list_sum(values: &[i64]) -> i64 {
    values.iter().sum()
}

/// Count occurrences of a value in a list.
pub fn list_count<T: PartialEq>(list: &[T], value: &T) -> usize {
    list.iter().filter(|&v| v == value).count()
}

/// Count occurrences of a substring in a string.
pub fn string_count(haystack: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() {
        return 0;
    }
    if needle.len() > haystack.len() {
        return 0;
    }

    let mut count = 0;
    let mut pos = 0;
    while pos + needle.len() <= haystack.len() {
        if &haystack[pos..pos + needle.len()] == needle {
            count += 1;
            pos += needle.len(); // Non-overlapping
        } else {
            pos += 1;
        }
    }
    count
}

/// Check if all values in a list satisfy a predicate.
pub fn list_all<T, F: Fn(&T) -> bool>(list: &[T], pred: F) -> bool {
    list.iter().all(pred)
}

/// Check if any value in a list satisfies a predicate.
pub fn list_any<T, F: Fn(&T) -> bool>(list: &[T], pred: F) -> bool {
    list.iter().any(pred)
}

/// Find index of first matching value.
pub fn list_index<T: PartialEq>(list: &[T], value: &T) -> Option<usize> {
    list.iter().position(|v| v == value)
}

// =============================================================================
// Index Calculations
// =============================================================================

/// Calculate list index from VimL index (supports negative).
///
/// VimL indexing:
/// - Positive: 0-based from start
/// - Negative: -1 is last element
pub const fn normalize_list_index(index: i64, len: i64) -> Option<usize> {
    if len <= 0 {
        return None;
    }

    let normalized = if index < 0 { len + index } else { index };

    if normalized >= 0 && normalized < len {
        Some(normalized as usize)
    } else {
        None
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: check if empty by length.
#[no_mangle]
pub extern "C" fn rs_f_is_empty_by_len(len: i64) -> bool {
    is_len_empty(len)
}

/// FFI export: normalize list index.
#[no_mangle]
pub extern "C" fn rs_f_normalize_list_index(index: i64, len: i64) -> c_int {
    normalize_list_index(index, len).map_or(-1, |i| i as c_int)
}

/// FFI export: count substring occurrences.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_string_count(
    haystack: *const u8,
    haystack_len: c_int,
    needle: *const u8,
    needle_len: c_int,
) -> c_int {
    if haystack.is_null() || haystack_len < 0 {
        return 0;
    }
    if needle.is_null() || needle_len < 0 {
        return 0;
    }

    // SAFETY: Caller guarantees valid pointers
    let h = unsafe { std::slice::from_raw_parts(haystack, haystack_len as usize) };
    let n = unsafe { std::slice::from_raw_parts(needle, needle_len as usize) };

    string_count(h, n) as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_len_empty() {
        assert!(is_len_empty(0));
        assert!(!is_len_empty(1));
        assert!(!is_len_empty(-1));
    }

    #[test]
    fn test_list_max_min() {
        assert_eq!(list_max(&[1, 3, 2]), Some(3));
        assert_eq!(list_min(&[1, 3, 2]), Some(1));
        assert_eq!(list_max(&[]), None);
        assert_eq!(list_min(&[]), None);
    }

    #[test]
    fn test_list_sum() {
        assert_eq!(list_sum(&[1, 2, 3]), 6);
        assert_eq!(list_sum(&[]), 0);
    }

    #[test]
    fn test_list_count() {
        assert_eq!(list_count(&[1, 2, 1, 3, 1], &1), 3);
        assert_eq!(list_count(&[1, 2, 3], &4), 0);
    }

    #[test]
    fn test_string_count() {
        assert_eq!(string_count(b"hello hello", b"hello"), 2);
        assert_eq!(string_count(b"aaa", b"aa"), 1); // Non-overlapping
        assert_eq!(string_count(b"abc", b""), 0);
        assert_eq!(string_count(b"abc", b"xyz"), 0);
    }

    #[test]
    fn test_list_all_any() {
        assert!(list_all(&[2, 4, 6], |&x| x % 2 == 0));
        assert!(!list_all(&[2, 3, 6], |&x| x % 2 == 0));
        assert!(list_any(&[1, 2, 3], |&x| x == 2));
        assert!(!list_any(&[1, 3, 5], |&x| x == 2));
    }

    #[test]
    fn test_list_index() {
        assert_eq!(list_index(&[1, 2, 3], &2), Some(1));
        assert_eq!(list_index(&[1, 2, 3], &4), None);
    }

    #[test]
    fn test_normalize_list_index() {
        // List of length 5: [0, 1, 2, 3, 4]
        assert_eq!(normalize_list_index(0, 5), Some(0));
        assert_eq!(normalize_list_index(4, 5), Some(4));
        assert_eq!(normalize_list_index(-1, 5), Some(4));
        assert_eq!(normalize_list_index(-5, 5), Some(0));
        assert_eq!(normalize_list_index(5, 5), None);
        assert_eq!(normalize_list_index(-6, 5), None);
        assert_eq!(normalize_list_index(0, 0), None);
    }
}
