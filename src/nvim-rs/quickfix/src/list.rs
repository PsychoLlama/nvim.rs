//! Quickfix/Location list management operations.
//!
//! This module provides Rust implementations for creating, modifying, and
//! managing quickfix and location lists, including stack navigation.

use std::ffi::{c_char, c_int, c_void};

/// Line number type (matches `linenr_T` in Neovim)
type LinenrT = i32;

/// Opaque handle to `qf_info_T` (quickfix stack)
type QfInfoHandle = *const c_void;
type QfInfoHandleMut = *mut c_void;

/// Opaque handle to `qf_list_T` (quickfix list)
type QfListHandle = *const c_void;
type QfListHandleMut = *mut c_void;

/// Opaque handle to `qfline_T` (quickfix entry)
type QfLineHandle = *const c_void;

// =============================================================================
// External C accessor functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Stack accessors
    fn nvim_qf_get_listcount(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_get_curlist(qi: QfInfoHandle) -> QfListHandle;
    fn nvim_qf_get_list_at(qi: QfInfoHandle, idx: c_int) -> QfListHandle;
    fn nvim_qf_get_curlist_idx(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_set_curlist_idx(qi: QfInfoHandleMut, idx: c_int);

    // List accessors
    fn nvim_qf_get_count(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_index(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_start(qfl: QfListHandle) -> QfLineHandle;
    fn nvim_qf_get_ptr(qfl: QfListHandle) -> QfLineHandle;
    fn nvim_qf_get_last(qfl: QfListHandle) -> QfLineHandle;
    fn nvim_qf_get_id(qfl: QfListHandle) -> u32;
    fn nvim_qf_get_changedtick(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_title(qfl: QfListHandle) -> *const c_char;
    fn nvim_qf_get_nonevalid(qfl: QfListHandle) -> bool;

    fn nvim_qf_set_index(qfl: QfListHandleMut, idx: c_int);
    fn nvim_qf_set_ptr(qfl: QfListHandleMut, ptr: QfLineHandle);

    // Entry accessors
    fn nvim_qfline_get_next(qfp: QfLineHandle) -> QfLineHandle;
    fn nvim_qfline_get_prev(qfp: QfLineHandle) -> QfLineHandle;
    fn nvim_qfline_get_fnum(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_valid(qfp: QfLineHandle) -> bool;

    // List operations
    fn nvim_qf_new_list(qi: QfInfoHandleMut, title: *const c_char);
    fn nvim_qf_store_title(qfl: QfListHandleMut, title: *const c_char);
    fn nvim_qf_free_list(qfl: QfListHandleMut);
    fn nvim_qf_free_items(qfl: QfListHandleMut);

    // Stack maxcount
    fn nvim_qf_get_maxcount(qi: QfInfoHandle) -> c_int;
}

// =============================================================================
// List Statistics
// =============================================================================

/// Statistics about a quickfix list.
#[repr(C)]
#[derive(Default)]
pub struct QfListStats {
    /// Total number of entries in the list
    pub total_entries: c_int,
    /// Number of valid entries
    pub valid_entries: c_int,
    /// Number of unique files referenced
    pub file_count: c_int,
    /// Current entry index (1-based)
    pub current_index: c_int,
    /// List ID
    pub list_id: u32,
    /// Changed tick counter
    pub changedtick: c_int,
    /// Whether the list has a title
    pub has_title: bool,
    /// Whether all entries are invalid
    pub all_invalid: bool,
}

/// Get comprehensive statistics about a quickfix list.
///
/// # Safety
///
/// - `qfl` may be null (returns default statistics)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_list_stats(qfl: QfListHandle) -> QfListStats {
    if qfl.is_null() {
        return QfListStats::default();
    }

    let total_entries = nvim_qf_get_count(qfl);
    let current_index = nvim_qf_get_index(qfl);
    let list_id = nvim_qf_get_id(qfl);
    let changedtick = nvim_qf_get_changedtick(qfl);
    let title = nvim_qf_get_title(qfl);
    let has_title = !title.is_null() && *title != 0;
    let all_invalid = nvim_qf_get_nonevalid(qfl);

    // Count valid entries and unique files
    let mut valid_entries = 0;
    let mut file_count = 0;
    let mut last_fnum = -1;

    let mut qfp = nvim_qf_get_start(qfl);
    while !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) {
            valid_entries += 1;
        }
        let fnum = nvim_qfline_get_fnum(qfp);
        if fnum > 0 && fnum != last_fnum {
            file_count += 1;
            last_fnum = fnum;
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    QfListStats {
        total_entries,
        valid_entries,
        file_count,
        current_index,
        list_id,
        changedtick,
        has_title,
        all_invalid,
    }
}

// =============================================================================
// Stack Operations
// =============================================================================

/// Information about a quickfix stack.
#[repr(C)]
#[derive(Default)]
pub struct QfStackInfo {
    /// Number of lists in the stack
    pub list_count: c_int,
    /// Current list index (0-based)
    pub current_index: c_int,
    /// Maximum number of lists the stack can hold
    pub max_count: c_int,
    /// Whether the stack is at capacity
    pub at_capacity: bool,
    /// Whether the stack is empty
    pub is_empty: bool,
    /// Whether we can go to older list
    pub can_go_older: bool,
    /// Whether we can go to newer list
    pub can_go_newer: bool,
}

/// Get information about a quickfix stack.
///
/// # Safety
///
/// - `qi` may be null (returns default info with `is_empty=true`)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_stack_info(qi: QfInfoHandle) -> QfStackInfo {
    if qi.is_null() {
        return QfStackInfo {
            is_empty: true,
            ..Default::default()
        };
    }

    let list_count = nvim_qf_get_listcount(qi);
    let current_index = nvim_qf_get_curlist_idx(qi);
    let max_count = nvim_qf_get_maxcount(qi);

    let is_empty = list_count <= 0;
    let at_capacity = list_count >= max_count;
    let can_go_older = current_index > 0;
    let can_go_newer = current_index < list_count - 1;

    QfStackInfo {
        list_count,
        current_index,
        max_count,
        at_capacity,
        is_empty,
        can_go_older,
        can_go_newer,
    }
}

/// Get the number of entries in the current list.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_curlist_entry_count(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return 0;
    }

    let qfl = nvim_qf_get_curlist(qi);
    if qfl.is_null() {
        return 0;
    }

    nvim_qf_get_count(qfl)
}

/// Get the number of valid entries in the current list.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_curlist_valid_count(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return 0;
    }

    let qfl = nvim_qf_get_curlist(qi);
    if qfl.is_null() {
        return 0;
    }

    // Count valid entries
    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);
    while !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    count
}

// =============================================================================
// List Navigation
// =============================================================================

/// Move to the next older list in the stack.
///
/// Returns the new list index (0-based) or -1 if at the oldest list.
///
/// # Safety
///
/// - `qi` may be null (returns -1)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_older_list(qi: QfInfoHandleMut) -> c_int {
    if qi.is_null() {
        return -1;
    }

    let current = nvim_qf_get_curlist_idx(qi);
    if current <= 0 {
        return -1;
    }

    let new_idx = current - 1;
    nvim_qf_set_curlist_idx(qi, new_idx);
    new_idx
}

/// Move to the next newer list in the stack.
///
/// Returns the new list index (0-based) or -1 if at the newest list.
///
/// # Safety
///
/// - `qi` may be null (returns -1)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_newer_list(qi: QfInfoHandleMut) -> c_int {
    if qi.is_null() {
        return -1;
    }

    let current = nvim_qf_get_curlist_idx(qi);
    let count = nvim_qf_get_listcount(qi);
    if current >= count - 1 {
        return -1;
    }

    let new_idx = current + 1;
    nvim_qf_set_curlist_idx(qi, new_idx);
    new_idx
}

/// Move to a specific list by index (0-based).
///
/// Returns true if successful, false if the index is invalid.
///
/// # Safety
///
/// - `qi` may be null (returns false)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_list_idx(qi: QfInfoHandleMut, idx: c_int) -> bool {
    if qi.is_null() {
        return false;
    }

    let count = nvim_qf_get_listcount(qi);
    if idx < 0 || idx >= count {
        return false;
    }

    nvim_qf_set_curlist_idx(qi, idx);
    true
}

/// Move to the oldest list in the stack.
///
/// Returns the list index (0) or -1 if the stack is empty.
///
/// # Safety
///
/// - `qi` may be null (returns -1)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_oldest_list(qi: QfInfoHandleMut) -> c_int {
    if qi.is_null() {
        return -1;
    }

    let count = nvim_qf_get_listcount(qi);
    if count <= 0 {
        return -1;
    }

    nvim_qf_set_curlist_idx(qi, 0);
    0
}

/// Move to the newest list in the stack.
///
/// Returns the list index or -1 if the stack is empty.
///
/// # Safety
///
/// - `qi` may be null (returns -1)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_newest_list(qi: QfInfoHandleMut) -> c_int {
    if qi.is_null() {
        return -1;
    }

    let count = nvim_qf_get_listcount(qi);
    if count <= 0 {
        return -1;
    }

    let new_idx = count - 1;
    nvim_qf_set_curlist_idx(qi, new_idx);
    new_idx
}

// =============================================================================
// Entry Position Management
// =============================================================================

/// Reset the current entry to the first entry in the list.
///
/// Returns the new index (1) or 0 if the list is empty.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_reset_to_first(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let start = nvim_qf_get_start(qfl);
    if start.is_null() {
        return 0;
    }

    nvim_qf_set_ptr(qfl, start);
    nvim_qf_set_index(qfl, 1);
    1
}

/// Reset the current entry to the last entry in the list.
///
/// Returns the new index or 0 if the list is empty.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_reset_to_last(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let last = nvim_qf_get_last(qfl);
    if last.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    nvim_qf_set_ptr(qfl, last);
    nvim_qf_set_index(qfl, count);
    count
}

/// Check if the current entry is at the first entry.
///
/// # Safety
///
/// - `qfl` may be null (returns true - empty list is at beginning)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_at_first(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return true;
    }

    nvim_qf_get_index(qfl) <= 1
}

/// Check if the current entry is at the last entry.
///
/// # Safety
///
/// - `qfl` may be null (returns true - empty list is at end)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_at_last(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return true;
    }

    let idx = nvim_qf_get_index(qfl);
    let count = nvim_qf_get_count(qfl);
    idx >= count
}

// =============================================================================
// List Comparison
// =============================================================================

/// Compare two lists by their ID.
///
/// Returns true if both lists have the same ID (or both are null).
///
/// # Safety
///
/// - Either or both handles may be null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_lists_same(a: QfListHandle, b: QfListHandle) -> bool {
    match (a.is_null(), b.is_null()) {
        (true, true) => true,
        (true, false) | (false, true) => false,
        (false, false) => nvim_qf_get_id(a) == nvim_qf_get_id(b),
    }
}

/// Check if a list has been modified since a given changedtick.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_list_modified_since(qfl: QfListHandle, tick: c_int) -> bool {
    if qfl.is_null() {
        return false;
    }

    nvim_qf_get_changedtick(qfl) > tick
}

// =============================================================================
// Entry Bounds Checking
// =============================================================================

/// Check if an entry index is valid for the list.
///
/// Entry indices are 1-based.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_valid_entry_idx(qfl: QfListHandle, idx: c_int) -> bool {
    if qfl.is_null() {
        return false;
    }

    let count = nvim_qf_get_count(qfl);
    idx >= 1 && idx <= count
}

/// Clamp an entry index to valid bounds.
///
/// Returns the clamped index (1 to count), or 0 if the list is empty.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_clamp_entry_idx(qfl: QfListHandle, idx: c_int) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return 0;
    }

    idx.clamp(1, count)
}

// =============================================================================
// List Capacity
// =============================================================================

/// Check if there's room for more lists in the stack.
///
/// # Safety
///
/// - `qi` may be null (returns false)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_stack_has_room(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return false;
    }

    let count = nvim_qf_get_listcount(qi);
    let max = nvim_qf_get_maxcount(qi);
    count < max
}

/// Get the number of available slots in the stack.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_stack_available(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return 0;
    }

    let count = nvim_qf_get_listcount(qi);
    let max = nvim_qf_get_maxcount(qi);
    if max > count {
        max - count
    } else {
        0
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_list_stats() {
        unsafe {
            let stats = rs_qf_list_stats(std::ptr::null());
            assert_eq!(stats.total_entries, 0);
            assert_eq!(stats.valid_entries, 0);
        }
    }

    #[test]
    fn test_null_stack_info() {
        unsafe {
            let info = rs_qf_stack_info(std::ptr::null());
            assert!(info.is_empty);
            assert_eq!(info.list_count, 0);
        }
    }

    #[test]
    fn test_null_curlist_entry_count() {
        unsafe {
            assert_eq!(rs_qf_curlist_entry_count(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_curlist_valid_count() {
        unsafe {
            assert_eq!(rs_qf_curlist_valid_count(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_older_list() {
        unsafe {
            assert_eq!(rs_qf_older_list(std::ptr::null_mut()), -1);
        }
    }

    #[test]
    fn test_null_newer_list() {
        unsafe {
            assert_eq!(rs_qf_newer_list(std::ptr::null_mut()), -1);
        }
    }

    #[test]
    fn test_null_goto_list_idx() {
        unsafe {
            assert!(!rs_qf_goto_list_idx(std::ptr::null_mut(), 0));
        }
    }

    #[test]
    fn test_null_oldest_list() {
        unsafe {
            assert_eq!(rs_qf_oldest_list(std::ptr::null_mut()), -1);
        }
    }

    #[test]
    fn test_null_newest_list() {
        unsafe {
            assert_eq!(rs_qf_newest_list(std::ptr::null_mut()), -1);
        }
    }

    #[test]
    fn test_null_reset_to_first() {
        unsafe {
            assert_eq!(rs_qf_reset_to_first(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_reset_to_last() {
        unsafe {
            assert_eq!(rs_qf_reset_to_last(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_at_first() {
        unsafe {
            assert!(rs_qf_at_first(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_at_last() {
        unsafe {
            assert!(rs_qf_at_last(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_lists_same() {
        unsafe {
            assert!(rs_qf_lists_same(std::ptr::null(), std::ptr::null()));
        }
    }

    #[test]
    fn test_null_list_modified_since() {
        unsafe {
            assert!(!rs_qf_list_modified_since(std::ptr::null(), 0));
        }
    }

    #[test]
    fn test_null_valid_entry_idx() {
        unsafe {
            assert!(!rs_qf_valid_entry_idx(std::ptr::null(), 1));
        }
    }

    #[test]
    fn test_null_clamp_entry_idx() {
        unsafe {
            assert_eq!(rs_qf_clamp_entry_idx(std::ptr::null(), 5), 0);
        }
    }

    #[test]
    fn test_null_stack_has_room() {
        unsafe {
            assert!(!rs_qf_stack_has_room(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_stack_available() {
        unsafe {
            assert_eq!(rs_qf_stack_available(std::ptr::null()), 0);
        }
    }
}
