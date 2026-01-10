//! List manipulation functions for VimL.
//!
//! This module implements list functions from `src/nvim/eval/funcs.c`:
//! - `add()` - append item to list
//! - `insert()` - insert item at position
//! - `remove()` - remove item at position
//! - `extend()` - extend list with another list
//! - `sort()` - sort list
//! - `reverse()` - reverse list
//! - `uniq()` - remove duplicates from sorted list
//! - `filter()` - filter list by predicate
//! - `map()` - transform list items
//! - `reduce()` - reduce list to single value

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;

// =============================================================================
// List Index Operations
// =============================================================================

/// Result of a list index validation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ListIndexResult {
    /// Whether the index is valid
    pub valid: bool,
    /// The normalized index (0-based, positive)
    pub index: c_int,
}

/// Validate and normalize a VimL list index.
///
/// VimL uses 0-based indexing with negative indices counting from the end.
/// - `0` is first element
/// - `-1` is last element
/// - `-len` is first element
pub const fn validate_list_index(index: i64, len: i64) -> ListIndexResult {
    if len <= 0 {
        return ListIndexResult {
            valid: false,
            index: 0,
        };
    }

    let normalized = if index < 0 { len + index } else { index };

    if normalized >= 0 && normalized < len {
        ListIndexResult {
            valid: true,
            index: normalized as c_int,
        }
    } else {
        ListIndexResult {
            valid: false,
            index: 0,
        }
    }
}

/// Validate list index for insertion (allows index == len for append).
pub const fn validate_insert_index(index: i64, len: i64) -> ListIndexResult {
    if len < 0 {
        return ListIndexResult {
            valid: false,
            index: 0,
        };
    }

    let normalized = if index < 0 { len + index + 1 } else { index };

    if normalized >= 0 && normalized <= len {
        ListIndexResult {
            valid: true,
            index: normalized as c_int,
        }
    } else {
        ListIndexResult {
            valid: false,
            index: 0,
        }
    }
}

// =============================================================================
// List Range Operations
// =============================================================================

/// Result of a list range validation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ListRangeResult {
    /// Whether the range is valid
    pub valid: bool,
    /// Start index (normalized)
    pub start: c_int,
    /// End index (normalized, exclusive)
    pub end: c_int,
}

/// Validate and normalize a VimL list range (for slicing).
///
/// VimL slice notation: `list[start:end]`
/// - Both indices are inclusive in VimL
/// - Negative indices count from end
/// - Missing start defaults to 0
/// - Missing end defaults to len-1
pub const fn validate_list_range(
    start: i64,
    has_start: bool,
    end: i64,
    has_end: bool,
    len: i64,
) -> ListRangeResult {
    if len <= 0 {
        return ListRangeResult {
            valid: true,
            start: 0,
            end: 0,
        };
    }

    // Normalize start
    let norm_start = if !has_start {
        0
    } else if start < 0 {
        let s = len + start;
        if s < 0 { 0 } else { s }
    } else if start >= len {
        len
    } else {
        start
    };

    // Normalize end (VimL end is inclusive, we convert to exclusive)
    let norm_end = if !has_end {
        len
    } else if end < 0 {
        let e = len + end + 1;
        if e < 0 { 0 } else { e }
    } else if end >= len {
        len
    } else {
        end + 1 // Convert inclusive to exclusive
    };

    // Empty range if start >= end
    if norm_start >= norm_end {
        return ListRangeResult {
            valid: true,
            start: 0,
            end: 0,
        };
    }

    ListRangeResult {
        valid: true,
        start: norm_start as c_int,
        end: norm_end as c_int,
    }
}

// =============================================================================
// List Comparison Operations
// =============================================================================

/// Compare two i64 values for sorting.
#[must_use]
pub const fn compare_i64(a: i64, b: i64) -> i32 {
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

/// Compare two strings lexicographically.
pub fn compare_strings(a: &[u8], b: &[u8]) -> i32 {
    for (x, y) in a.iter().zip(b.iter()) {
        if x < y {
            return -1;
        }
        if x > y {
            return 1;
        }
    }
    compare_i64(a.len() as i64, b.len() as i64)
}

/// Compare two strings case-insensitively.
pub fn compare_strings_ic(a: &[u8], b: &[u8]) -> i32 {
    for (x, y) in a.iter().zip(b.iter()) {
        let x_lower = x.to_ascii_lowercase();
        let y_lower = y.to_ascii_lowercase();
        if x_lower < y_lower {
            return -1;
        }
        if x_lower > y_lower {
            return 1;
        }
    }
    compare_i64(a.len() as i64, b.len() as i64)
}

// =============================================================================
// List Flatten Operations
// =============================================================================

/// Maximum recursion depth for flatten.
pub const MAX_FLATTEN_DEPTH: i64 = 999;

/// Check if flatten depth is valid.
pub const fn is_valid_flatten_depth(depth: i64) -> bool {
    depth >= 0 && depth <= MAX_FLATTEN_DEPTH
}

// =============================================================================
// List Repeat Operations
// =============================================================================

/// Calculate result size for list repeat.
///
/// VimL: `repeat(list, count)`
/// Returns the total number of elements in the result.
pub const fn repeat_result_len(list_len: i64, count: i64) -> i64 {
    if count <= 0 || list_len <= 0 {
        0
    } else {
        list_len.saturating_mul(count)
    }
}

// =============================================================================
// List Copy Operations
// =============================================================================

/// Copy type for list/dict copy operations.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CopyType {
    /// Shallow copy - only top level is copied
    #[default]
    Shallow = 0,
    /// Deep copy - recursively copy all nested structures
    Deep = 1,
}

impl CopyType {
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Deep,
            _ => Self::Shallow,
        }
    }

    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    #[must_use]
    pub const fn is_deep(self) -> bool {
        matches!(self, Self::Deep)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: validate and normalize list index.
#[no_mangle]
pub extern "C" fn rs_f_list_validate_index(index: i64, len: i64) -> ListIndexResult {
    validate_list_index(index, len)
}

/// FFI export: validate list index for insertion.
#[no_mangle]
pub extern "C" fn rs_f_list_validate_insert_index(index: i64, len: i64) -> ListIndexResult {
    validate_insert_index(index, len)
}

/// FFI export: validate and normalize list range.
#[no_mangle]
pub extern "C" fn rs_f_list_validate_range(
    start: i64,
    has_start: c_int,
    end: i64,
    has_end: c_int,
    len: i64,
) -> ListRangeResult {
    validate_list_range(start, has_start != 0, end, has_end != 0, len)
}

/// FFI export: compare i64 values.
#[no_mangle]
pub extern "C" fn rs_f_list_compare_i64(a: i64, b: i64) -> c_int {
    compare_i64(a, b)
}

/// FFI export: compare strings.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_list_compare_strings(
    a: *const u8,
    a_len: c_int,
    b: *const u8,
    b_len: c_int,
    ignore_case: c_int,
) -> c_int {
    if a.is_null() && b.is_null() {
        return 0;
    }
    if a.is_null() {
        return -1;
    }
    if b.is_null() {
        return 1;
    }

    let a_slice = std::slice::from_raw_parts(a, a_len.max(0) as usize);
    let b_slice = std::slice::from_raw_parts(b, b_len.max(0) as usize);

    if ignore_case != 0 {
        compare_strings_ic(a_slice, b_slice)
    } else {
        compare_strings(a_slice, b_slice)
    }
}

/// FFI export: check if flatten depth is valid.
#[no_mangle]
pub extern "C" fn rs_f_list_is_valid_flatten_depth(depth: i64) -> c_int {
    c_int::from(is_valid_flatten_depth(depth))
}

/// FFI export: calculate repeat result length.
#[no_mangle]
pub extern "C" fn rs_f_list_repeat_len(list_len: i64, count: i64) -> i64 {
    repeat_result_len(list_len, count)
}

/// FFI export: get copy type.
#[no_mangle]
pub extern "C" fn rs_f_list_copy_type(deep: c_int) -> c_int {
    CopyType::from_raw(deep).to_raw()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_list_index() {
        // Valid indices
        let r = validate_list_index(0, 5);
        assert!(r.valid);
        assert_eq!(r.index, 0);

        let r = validate_list_index(4, 5);
        assert!(r.valid);
        assert_eq!(r.index, 4);

        // Negative indices
        let r = validate_list_index(-1, 5);
        assert!(r.valid);
        assert_eq!(r.index, 4);

        let r = validate_list_index(-5, 5);
        assert!(r.valid);
        assert_eq!(r.index, 0);

        // Invalid indices
        let r = validate_list_index(5, 5);
        assert!(!r.valid);

        let r = validate_list_index(-6, 5);
        assert!(!r.valid);

        // Empty list
        let r = validate_list_index(0, 0);
        assert!(!r.valid);
    }

    #[test]
    fn test_validate_insert_index() {
        // Valid indices (0 to len inclusive)
        let r = validate_insert_index(0, 5);
        assert!(r.valid);
        assert_eq!(r.index, 0);

        let r = validate_insert_index(5, 5); // Append position
        assert!(r.valid);
        assert_eq!(r.index, 5);

        // Negative indices
        let r = validate_insert_index(-1, 5);
        assert!(r.valid);
        assert_eq!(r.index, 5);

        // Invalid
        let r = validate_insert_index(6, 5);
        assert!(!r.valid);
    }

    #[test]
    fn test_validate_list_range() {
        // Full range
        let r = validate_list_range(0, true, 4, true, 5);
        assert!(r.valid);
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 5);

        // Partial range
        let r = validate_list_range(1, true, 3, true, 5);
        assert!(r.valid);
        assert_eq!(r.start, 1);
        assert_eq!(r.end, 4);

        // Negative indices
        let r = validate_list_range(-3, true, -1, true, 5);
        assert!(r.valid);
        assert_eq!(r.start, 2);
        assert_eq!(r.end, 5);

        // No start
        let r = validate_list_range(0, false, 2, true, 5);
        assert!(r.valid);
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 3);

        // No end
        let r = validate_list_range(2, true, 0, false, 5);
        assert!(r.valid);
        assert_eq!(r.start, 2);
        assert_eq!(r.end, 5);
    }

    #[test]
    fn test_compare_strings() {
        assert_eq!(compare_strings(b"abc", b"abc"), 0);
        assert_eq!(compare_strings(b"abc", b"abd"), -1);
        assert_eq!(compare_strings(b"abd", b"abc"), 1);
        assert_eq!(compare_strings(b"ab", b"abc"), -1);
        assert_eq!(compare_strings(b"abc", b"ab"), 1);
    }

    #[test]
    fn test_compare_strings_ic() {
        assert_eq!(compare_strings_ic(b"ABC", b"abc"), 0);
        assert_eq!(compare_strings_ic(b"ABC", b"abd"), -1);
        assert_eq!(compare_strings_ic(b"Abd", b"ABC"), 1);
    }

    #[test]
    fn test_repeat_result_len() {
        assert_eq!(repeat_result_len(3, 4), 12);
        assert_eq!(repeat_result_len(0, 4), 0);
        assert_eq!(repeat_result_len(3, 0), 0);
        assert_eq!(repeat_result_len(3, -1), 0);
    }

    #[test]
    fn test_copy_type() {
        assert!(!CopyType::Shallow.is_deep());
        assert!(CopyType::Deep.is_deep());
        assert_eq!(CopyType::from_raw(0), CopyType::Shallow);
        assert_eq!(CopyType::from_raw(1), CopyType::Deep);
    }
}
