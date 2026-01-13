//! List operations.
//!
//! This module provides helpers for list operations:
//! list_alloc, list_append, list_copy, list_extend

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;
use std::ptr::NonNull;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a list_T structure.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ListHandle(NonNull<std::ffi::c_void>);

/// Opaque handle to a listitem_T structure.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ListItemHandle(NonNull<std::ffi::c_void>);

// =============================================================================
// List Operation Constants
// =============================================================================

/// List is empty.
pub const LIST_EMPTY: c_int = 0;
/// List has items.
pub const LIST_HAS_ITEMS: c_int = 1;

// =============================================================================
// List Flags
// =============================================================================

/// List is locked (cannot be modified).
pub const LIST_LOCKED: c_int = 0x01;
/// List items are frozen (deep lock).
pub const LIST_FROZEN: c_int = 0x02;
/// List is being copied.
pub const LIST_COPYING: c_int = 0x04;

// =============================================================================
// List Helpers
// =============================================================================

/// Check if index is valid for given length.
fn is_valid_list_index(idx: i64, len: i64) -> bool {
    if idx >= 0 {
        idx < len
    } else {
        idx.abs() <= len
    }
}

/// Normalize negative index to positive.
fn normalize_list_index(idx: i64, len: i64) -> i64 {
    if idx < 0 {
        len + idx
    } else {
        idx
    }
}

/// Calculate slice length.
fn slice_length(start: i64, end: i64, len: i64) -> i64 {
    let start = normalize_list_index(start, len);
    let end = normalize_list_index(end, len);
    if start > end || start >= len {
        0
    } else {
        (end.min(len - 1) - start + 1).max(0)
    }
}

/// Check if list is locked.
fn is_list_locked(flags: c_int) -> bool {
    (flags & LIST_LOCKED) != 0
}

/// Check if list is frozen.
fn is_list_frozen(flags: c_int) -> bool {
    (flags & LIST_FROZEN) != 0
}

/// Check if list can be modified.
fn can_modify_list(flags: c_int) -> bool {
    (flags & (LIST_LOCKED | LIST_FROZEN)) == 0
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get LIST_EMPTY constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_list_empty() -> c_int {
    LIST_EMPTY
}

/// FFI: Get LIST_HAS_ITEMS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_list_has_items() -> c_int {
    LIST_HAS_ITEMS
}

/// FFI: Get LIST_LOCKED constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_list_locked_flag() -> c_int {
    LIST_LOCKED
}

/// FFI: Get LIST_FROZEN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_list_frozen_flag() -> c_int {
    LIST_FROZEN
}

/// FFI: Check if valid list index.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_valid_list_index(idx: i64, len: i64) -> c_int {
    c_int::from(is_valid_list_index(idx, len))
}

/// FFI: Normalize list index.
#[unsafe(no_mangle)]
pub extern "C" fn rs_normalize_list_index(idx: i64, len: i64) -> i64 {
    normalize_list_index(idx, len)
}

/// FFI: Calculate slice length.
#[unsafe(no_mangle)]
pub extern "C" fn rs_slice_length(start: i64, end: i64, len: i64) -> i64 {
    slice_length(start, end, len)
}

/// FFI: Check if list is locked.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_list_locked(flags: c_int) -> c_int {
    c_int::from(is_list_locked(flags))
}

/// FFI: Check if list is frozen.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_list_frozen(flags: c_int) -> c_int {
    c_int::from(is_list_frozen(flags))
}

/// FFI: Check if list can be modified.
#[unsafe(no_mangle)]
pub extern "C" fn rs_can_modify_list(flags: c_int) -> c_int {
    c_int::from(can_modify_list(flags))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_constants() {
        assert_eq!(LIST_EMPTY, 0);
        assert_eq!(LIST_HAS_ITEMS, 1);
    }

    #[test]
    fn test_list_flags() {
        assert_eq!(LIST_LOCKED, 0x01);
        assert_eq!(LIST_FROZEN, 0x02);
    }

    #[test]
    fn test_is_valid_list_index() {
        // Positive indices
        assert!(is_valid_list_index(0, 5));
        assert!(is_valid_list_index(4, 5));
        assert!(!is_valid_list_index(5, 5));
        assert!(!is_valid_list_index(10, 5));

        // Negative indices
        assert!(is_valid_list_index(-1, 5)); // last item
        assert!(is_valid_list_index(-5, 5)); // first item
        assert!(!is_valid_list_index(-6, 5)); // before first

        // Empty list
        assert!(!is_valid_list_index(0, 0));
        assert!(!is_valid_list_index(-1, 0));
    }

    #[test]
    fn test_normalize_list_index() {
        assert_eq!(normalize_list_index(0, 5), 0);
        assert_eq!(normalize_list_index(3, 5), 3);
        assert_eq!(normalize_list_index(-1, 5), 4);
        assert_eq!(normalize_list_index(-5, 5), 0);
    }

    #[test]
    fn test_slice_length() {
        assert_eq!(slice_length(0, 4, 5), 5);
        assert_eq!(slice_length(1, 3, 5), 3);
        assert_eq!(slice_length(0, 0, 5), 1);
        assert_eq!(slice_length(-2, -1, 5), 2);
        assert_eq!(slice_length(3, 1, 5), 0); // start > end
    }

    #[test]
    fn test_list_flag_checks() {
        assert!(is_list_locked(LIST_LOCKED));
        assert!(!is_list_locked(0));

        assert!(is_list_frozen(LIST_FROZEN));
        assert!(!is_list_frozen(LIST_LOCKED));

        assert!(can_modify_list(0));
        assert!(!can_modify_list(LIST_LOCKED));
        assert!(!can_modify_list(LIST_FROZEN));
        assert!(!can_modify_list(LIST_LOCKED | LIST_FROZEN));
    }
}
