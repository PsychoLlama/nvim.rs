//! VimL list operations for Neovim
//!
//! This crate provides Rust implementations of list-related functions
//! from `src/nvim/eval/typval.c`. It handles VimL list creation,
//! manipulation, and iteration.
//!
//! ## Phase 28.3: List Operations
//!
//! This module provides:
//! - List length specials enum (Unknown, ShouldKnow, MayKnow)
//! - List iteration helpers
//! - List index normalization (positive/negative indexing)
//! - List comparison utilities
//! - FFI exports for list operations
//!
//! ## Architecture
//!
//! VimL lists are reference-counted and can contain any VimL value.
//! Lists support:
//! - Negative indexing (l[-1] is the last element)
//! - Slicing (l[start:end])
//! - Nested structures (lists of lists)
//! - Watchers for change notification

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_int, c_void};

// Re-export handle types from typval
pub use nvim_typval::{ListHandle, ListItemHandle, TypevalHandle, VarLockStatus};

// =============================================================================
// List Length Specials (matching C's ListLenSpecials)
// =============================================================================

/// Special values for list length in tv_list_alloc().
///
/// Matches C's `ListLenSpecials` enum in `typval_defs.h`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListLenSpecial {
    /// List length is not known in advance.
    /// Used when there is neither a way to know how many elements will be
    /// needed nor any educated guesses.
    Unknown = -1,
    /// List length should be known, but is actually not.
    /// All occurrences should eventually be removed - this is for cases
    /// where the only reason length is unknown is that it would be hard
    /// to code without refactoring.
    ShouldKnow = -2,
    /// List length may be known in advance, but requires too much effort.
    /// Used when it looks impractical to determine list length.
    MayKnow = -3,
}

impl ListLenSpecial {
    /// Check if this is a special length value.
    #[inline]
    pub const fn is_special(len: i32) -> bool {
        len < 0
    }

    /// Convert from C integer.
    #[inline]
    pub const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            -1 => Some(Self::Unknown),
            -2 => Some(Self::ShouldKnow),
            -3 => Some(Self::MayKnow),
            _ => None,
        }
    }
}

// =============================================================================
// C accessor functions
// =============================================================================

extern "C" {
    fn nvim_list_get_len(l: *const c_void) -> c_int;
    fn nvim_list_get_first(l: *const c_void) -> *const c_void;
    fn nvim_list_get_last(l: *const c_void) -> *const c_void;
    fn nvim_list_get_lock(l: *const c_void) -> c_int;
    fn nvim_list_has_watchers(l: *const c_void) -> c_int;
    fn nvim_listitem_get_next(li: *const c_void) -> *const c_void;
    fn nvim_listitem_get_prev(li: *const c_void) -> *const c_void;
    fn nvim_listitem_get_tv(li: *const c_void) -> *const c_void;
}

// =============================================================================
// List Index Normalization
// =============================================================================

/// Normalize a list index to handle negative indices.
///
/// Negative indices count from the end: -1 is last, -2 is second-to-last, etc.
/// Returns None if the index is out of bounds.
///
/// # Examples
/// ```ignore
/// normalize_index(3, 5)   // Some(3) - positive index in bounds
/// normalize_index(-1, 5)  // Some(4) - last element
/// normalize_index(-5, 5)  // Some(0) - first element
/// normalize_index(5, 5)   // None - out of bounds
/// normalize_index(-6, 5)  // None - out of bounds
/// ```
#[inline]
pub const fn normalize_index(idx: i64, len: i64) -> Option<i64> {
    if len <= 0 {
        return None;
    }

    let normalized = if idx < 0 { len + idx } else { idx };

    if normalized >= 0 && normalized < len {
        Some(normalized)
    } else {
        None
    }
}

/// Normalize a list index, returning -1 for invalid indices.
///
/// This matches the C `tv_list_uidx` function behavior.
#[inline]
pub const fn normalize_index_or_neg1(idx: i64, len: i64) -> i64 {
    match normalize_index(idx, len) {
        Some(n) => n,
        None => -1,
    }
}

/// FFI: Normalize a list index.
#[no_mangle]
pub extern "C" fn rs_list_normalize_index(idx: i64, len: i64) -> i64 {
    normalize_index_or_neg1(idx, len)
}

// =============================================================================
// List Length Operations
// =============================================================================

/// Get the length of a list, returning 0 for NULL lists.
#[inline]
fn list_len_impl(list: ListHandle) -> c_int {
    if list.is_null() {
        0
    } else {
        unsafe { nvim_list_get_len(list.as_ptr()) }
    }
}

/// FFI: Get list length.
#[no_mangle]
pub extern "C" fn rs_list_len(list: ListHandle) -> c_int {
    list_len_impl(list)
}

/// Check if a list is empty (NULL or zero length).
#[inline]
fn list_is_empty_impl(list: ListHandle) -> bool {
    list_len_impl(list) == 0
}

/// FFI: Check if list is empty.
#[no_mangle]
pub extern "C" fn rs_list_is_empty(list: ListHandle) -> bool {
    list_is_empty_impl(list)
}

// =============================================================================
// List Navigation
// =============================================================================

/// Get the first item in a list.
#[inline]
fn list_first_impl(list: ListHandle) -> ListItemHandle {
    if list.is_null() {
        ListItemHandle::null()
    } else {
        let ptr = unsafe { nvim_list_get_first(list.as_ptr()) };
        unsafe { ListItemHandle::from_ptr(ptr) }
    }
}

/// FFI: Get first list item.
#[no_mangle]
pub extern "C" fn rs_list_first(list: ListHandle) -> ListItemHandle {
    list_first_impl(list)
}

/// Get the last item in a list.
#[inline]
fn list_last_impl(list: ListHandle) -> ListItemHandle {
    if list.is_null() {
        ListItemHandle::null()
    } else {
        let ptr = unsafe { nvim_list_get_last(list.as_ptr()) };
        unsafe { ListItemHandle::from_ptr(ptr) }
    }
}

/// FFI: Get last list item.
#[no_mangle]
pub extern "C" fn rs_list_last(list: ListHandle) -> ListItemHandle {
    list_last_impl(list)
}

/// Get the next item after the given item.
#[inline]
fn listitem_next_impl(item: ListItemHandle) -> ListItemHandle {
    if item.is_null() {
        ListItemHandle::null()
    } else {
        let ptr = unsafe { nvim_listitem_get_next(item.as_ptr()) };
        unsafe { ListItemHandle::from_ptr(ptr) }
    }
}

/// FFI: Get next list item.
#[no_mangle]
pub extern "C" fn rs_listitem_next(item: ListItemHandle) -> ListItemHandle {
    listitem_next_impl(item)
}

/// Get the previous item before the given item.
#[inline]
fn listitem_prev_impl(item: ListItemHandle) -> ListItemHandle {
    if item.is_null() {
        ListItemHandle::null()
    } else {
        let ptr = unsafe { nvim_listitem_get_prev(item.as_ptr()) };
        unsafe { ListItemHandle::from_ptr(ptr) }
    }
}

/// FFI: Get previous list item.
#[no_mangle]
pub extern "C" fn rs_listitem_prev(item: ListItemHandle) -> ListItemHandle {
    listitem_prev_impl(item)
}

/// Get the typval for a list item.
#[inline]
fn listitem_tv_impl(item: ListItemHandle) -> TypevalHandle {
    if item.is_null() {
        unsafe { TypevalHandle::from_ptr(std::ptr::null()) }
    } else {
        let ptr = unsafe { nvim_listitem_get_tv(item.as_ptr()) };
        unsafe { TypevalHandle::from_ptr(ptr) }
    }
}

/// FFI: Get typval from list item.
#[no_mangle]
pub extern "C" fn rs_listitem_tv(item: ListItemHandle) -> TypevalHandle {
    listitem_tv_impl(item)
}

// =============================================================================
// List Lock Status
// =============================================================================

/// Get the lock status of a list.
#[inline]
fn list_locked_impl(list: ListHandle) -> c_int {
    if list.is_null() {
        VarLockStatus::Fixed as c_int // NULL list is considered fixed
    } else {
        unsafe { nvim_list_get_lock(list.as_ptr()) }
    }
}

/// FFI: Get list lock status.
#[no_mangle]
pub extern "C" fn rs_list_locked(list: ListHandle) -> c_int {
    list_locked_impl(list)
}

/// Check if a list is locked or fixed.
#[inline]
fn list_is_locked_impl(list: ListHandle) -> bool {
    list_locked_impl(list) != 0
}

/// FFI: Check if list is locked.
#[no_mangle]
pub extern "C" fn rs_list_is_locked(list: ListHandle) -> bool {
    list_is_locked_impl(list)
}

// =============================================================================
// List Watchers
// =============================================================================

/// Check if a list has watchers attached.
#[inline]
fn list_has_watchers_impl(list: ListHandle) -> bool {
    if list.is_null() {
        false
    } else {
        unsafe { nvim_list_has_watchers(list.as_ptr()) != 0 }
    }
}

/// FFI: Check if list has watchers.
#[no_mangle]
pub extern "C" fn rs_list_has_watchers(list: ListHandle) -> bool {
    list_has_watchers_impl(list)
}

// =============================================================================
// List Iteration Helpers
// =============================================================================

/// An iterator over list items.
///
/// This is a Rust iterator that wraps the linked-list traversal.
pub struct ListIter {
    current: ListItemHandle,
}

impl ListIter {
    /// Create a new iterator starting from the first item.
    #[inline]
    pub fn new(list: ListHandle) -> Self {
        Self {
            current: list_first_impl(list),
        }
    }

    /// Create a new iterator starting from a specific item.
    #[inline]
    pub const fn from_item(item: ListItemHandle) -> Self {
        Self { current: item }
    }
}

impl Iterator for ListIter {
    type Item = ListItemHandle;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let result = self.current;
            self.current = listitem_next_impl(self.current);
            Some(result)
        }
    }
}

/// A reverse iterator over list items.
pub struct ListIterRev {
    current: ListItemHandle,
}

impl ListIterRev {
    /// Create a new reverse iterator starting from the last item.
    #[inline]
    pub fn new(list: ListHandle) -> Self {
        Self {
            current: list_last_impl(list),
        }
    }
}

impl Iterator for ListIterRev {
    type Item = ListItemHandle;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let result = self.current;
            self.current = listitem_prev_impl(self.current);
            Some(result)
        }
    }
}

// =============================================================================
// List Range Validation
// =============================================================================

/// Validate and normalize a range for list slicing.
///
/// Returns (start, end) tuple where start and end are normalized indices.
/// Returns None if the range is invalid.
#[inline]
pub const fn validate_range(start: i64, end: i64, len: i64) -> Option<(i64, i64)> {
    if len <= 0 {
        return None;
    }

    // Normalize start
    let norm_start = if start < 0 {
        let s = len + start;
        if s < 0 {
            0
        } else {
            s
        }
    } else {
        start
    };

    // Normalize end
    let norm_end = if end < 0 {
        let e = len + end;
        if e < 0 {
            -1
        } else {
            e
        }
    } else if end >= len {
        len - 1
    } else {
        end
    };

    // Check valid range
    if norm_start > norm_end || norm_start >= len {
        None
    } else {
        Some((norm_start, norm_end))
    }
}

/// FFI: Validate a list range, returning start in out_start and end in out_end.
/// Returns true if valid, false if invalid.
///
/// # Safety
/// `out_start` and `out_end` must be valid pointers or null.
#[no_mangle]
pub unsafe extern "C" fn rs_list_validate_range(
    start: i64,
    end: i64,
    len: i64,
    out_start: *mut i64,
    out_end: *mut i64,
) -> bool {
    match validate_range(start, end, len) {
        Some((s, e)) => {
            if !out_start.is_null() {
                *out_start = s;
            }
            if !out_end.is_null() {
                *out_end = e;
            }
            true
        }
        None => false,
    }
}

// =============================================================================
// List Comparison Constants
// =============================================================================

/// Comparison result for lists.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListCompareResult {
    /// Lists are equal
    Equal = 0,
    /// First list is less than second
    Less = -1,
    /// First list is greater than second
    Greater = 1,
    /// Lists are not comparable (different types, etc.)
    Incomparable = 2,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_len_special() {
        assert_eq!(ListLenSpecial::Unknown as i32, -1);
        assert_eq!(ListLenSpecial::ShouldKnow as i32, -2);
        assert_eq!(ListLenSpecial::MayKnow as i32, -3);

        assert!(ListLenSpecial::is_special(-1));
        assert!(ListLenSpecial::is_special(-2));
        assert!(ListLenSpecial::is_special(-3));
        assert!(!ListLenSpecial::is_special(0));
        assert!(!ListLenSpecial::is_special(10));

        assert_eq!(
            ListLenSpecial::from_c_int(-1),
            Some(ListLenSpecial::Unknown)
        );
        assert_eq!(
            ListLenSpecial::from_c_int(-2),
            Some(ListLenSpecial::ShouldKnow)
        );
        assert_eq!(
            ListLenSpecial::from_c_int(-3),
            Some(ListLenSpecial::MayKnow)
        );
        assert_eq!(ListLenSpecial::from_c_int(0), None);
    }

    #[test]
    fn test_normalize_index() {
        // Positive indices
        assert_eq!(normalize_index(0, 5), Some(0));
        assert_eq!(normalize_index(2, 5), Some(2));
        assert_eq!(normalize_index(4, 5), Some(4));
        assert_eq!(normalize_index(5, 5), None); // Out of bounds

        // Negative indices
        assert_eq!(normalize_index(-1, 5), Some(4)); // Last
        assert_eq!(normalize_index(-2, 5), Some(3));
        assert_eq!(normalize_index(-5, 5), Some(0)); // First
        assert_eq!(normalize_index(-6, 5), None); // Out of bounds

        // Edge cases
        assert_eq!(normalize_index(0, 0), None); // Empty list
        assert_eq!(normalize_index(0, 1), Some(0)); // Single element
        assert_eq!(normalize_index(-1, 1), Some(0)); // Last of single
    }

    #[test]
    fn test_normalize_index_or_neg1() {
        assert_eq!(normalize_index_or_neg1(0, 5), 0);
        assert_eq!(normalize_index_or_neg1(-1, 5), 4);
        assert_eq!(normalize_index_or_neg1(10, 5), -1);
        assert_eq!(normalize_index_or_neg1(-10, 5), -1);
    }

    #[test]
    fn test_validate_range() {
        // Normal ranges
        assert_eq!(validate_range(0, 4, 5), Some((0, 4)));
        assert_eq!(validate_range(1, 3, 5), Some((1, 3)));
        assert_eq!(validate_range(0, 0, 5), Some((0, 0)));

        // Negative indices in range
        assert_eq!(validate_range(0, -1, 5), Some((0, 4)));
        assert_eq!(validate_range(-3, -1, 5), Some((2, 4)));
        assert_eq!(validate_range(-5, -1, 5), Some((0, 4)));

        // End beyond list length (clamped)
        assert_eq!(validate_range(0, 10, 5), Some((0, 4)));

        // Invalid ranges
        assert_eq!(validate_range(3, 1, 5), None); // Start > end
        assert_eq!(validate_range(5, 6, 5), None); // Start beyond length
        assert_eq!(validate_range(0, 0, 0), None); // Empty list
    }

    #[test]
    fn test_list_compare_result() {
        assert_eq!(ListCompareResult::Equal as i32, 0);
        assert_eq!(ListCompareResult::Less as i32, -1);
        assert_eq!(ListCompareResult::Greater as i32, 1);
        assert_eq!(ListCompareResult::Incomparable as i32, 2);
    }

    #[test]
    fn test_null_list_handle() {
        let null_list = ListHandle::null();
        assert!(null_list.is_null());
        assert_eq!(list_len_impl(null_list), 0);
        assert!(list_is_empty_impl(null_list));
        assert_eq!(list_locked_impl(null_list), VarLockStatus::Fixed as c_int);
        assert!(!list_has_watchers_impl(null_list));
        assert!(list_first_impl(null_list).is_null());
        assert!(list_last_impl(null_list).is_null());
    }

    #[test]
    fn test_null_listitem_handle() {
        let null_item = ListItemHandle::null();
        assert!(null_item.is_null());
        assert!(listitem_next_impl(null_item).is_null());
        assert!(listitem_prev_impl(null_item).is_null());
        assert!(listitem_tv_impl(null_item).is_null());
    }

    #[test]
    fn test_list_iter_empty() {
        let null_list = ListHandle::null();
        let mut iter = ListIter::new(null_list);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_list_iter_rev_empty() {
        let null_list = ListHandle::null();
        let mut iter = ListIterRev::new(null_list);
        assert!(iter.next().is_none());
    }
}
