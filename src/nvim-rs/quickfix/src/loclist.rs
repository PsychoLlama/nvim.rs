//! Location list specific functionality.
//!
//! This module provides functions specific to location lists, which are
//! window-local versions of quickfix lists. Location lists differ from
//! quickfix lists in that:
//! - Each window can have its own location list stack
//! - Location lists are freed when the window is closed
//! - Location list windows can reference another window's location list

use std::ffi::c_int;

use crate::{
    nvim_qf_get_count, nvim_qf_get_curlist_idx, nvim_qf_get_listcount, nvim_qf_get_refcount,
    nvim_win_get_llist_ref, nvim_win_get_loclist, QfInfoHandle, QfListHandle, WinHandle,
};

// =============================================================================
// External C functions for location lists
// =============================================================================

extern "C" {
    /// Get the `qfl_type` from `qf_info_T`: 0=quickfix, 1=location, 2=internal.
    fn nvim_qi_get_qfl_type(qi: QfInfoHandle) -> c_int;
}

/// Location list type constant (from quickfix.c `QFLT_LOCATION`).
const QFLT_LOCATION: c_int = 1;

// =============================================================================
// Location List Detection
// =============================================================================

/// Check if a quickfix stack is a location list.
///
/// # Safety
///
/// - `qi` may be null (returns false)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_is_loclist(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return false;
    }

    nvim_qi_get_qfl_type(qi) == QFLT_LOCATION
}

/// Check if a quickfix stack is a global quickfix (not a location list).
///
/// # Safety
///
/// - `qi` may be null (returns false)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_is_quickfix(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return false;
    }

    nvim_qi_get_qfl_type(qi) != QFLT_LOCATION
}

// =============================================================================
// Window-Local Location List Access
// =============================================================================

/// Get the location list stack for a window.
///
/// Returns the window's own location list (`w_llist`) or its reference
/// to another window's location list (`w_llist_ref`) if the window is
/// a location list window.
///
/// # Safety
///
/// - `wp` may be null (returns null)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_get_for_window(wp: WinHandle) -> QfInfoHandle {
    if wp.is_null() {
        return std::ptr::null();
    }

    nvim_win_get_loclist(wp)
}

/// Check if a window has a location list.
///
/// # Safety
///
/// - `wp` may be null (returns false)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_window_has_loclist(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    !nvim_win_get_loclist(wp).is_null()
}

/// Check if a window is a location list window (references another window's list).
///
/// # Safety
///
/// - `wp` may be null (returns false)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_is_loclist_window(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    !nvim_win_get_llist_ref(wp).is_null()
}

/// Check if a window has its own location list (not a reference).
///
/// # Safety
///
/// - `wp` may be null (returns false)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_window_owns_loclist(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    let ll = nvim_win_get_loclist(wp);
    let ll_ref = nvim_win_get_llist_ref(wp);

    !ll.is_null() && ll_ref.is_null()
}

// =============================================================================
// Location List Reference Counting
// =============================================================================

/// Get the reference count for a location list.
///
/// The reference count tracks how many windows reference this location list.
/// When the count drops to 0, the list can be freed.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_refcount(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return 0;
    }

    nvim_qf_get_refcount(qi)
}

/// Check if a location list has multiple references.
///
/// # Safety
///
/// - `qi` may be null (returns false)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_is_shared(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return false;
    }

    nvim_qf_get_refcount(qi) > 1
}

/// Check if a location list can be safely freed.
///
/// A location list can be freed when its reference count is 1 or less.
///
/// # Safety
///
/// - `qi` may be null (returns true - null can be "freed")
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_can_free(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return true;
    }

    nvim_qf_get_refcount(qi) <= 1
}

// =============================================================================
// Location List Summary
// =============================================================================

/// Summary information for a location list.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct LoclistInfo {
    /// Whether this is a location list (vs quickfix).
    pub is_loclist: bool,
    /// Number of lists in the stack.
    pub list_count: c_int,
    /// Current list index (0-based).
    pub cur_list_idx: c_int,
    /// Number of entries in the current list.
    pub entry_count: c_int,
    /// Reference count (for location lists).
    pub refcount: c_int,
    /// Whether the list is shared between windows.
    pub is_shared: bool,
}

/// Get summary information about a location list.
///
/// # Safety
///
/// - `qi` may be null (returns default info)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_info(qi: QfInfoHandle) -> LoclistInfo {
    let mut info = LoclistInfo::default();

    if qi.is_null() {
        return info;
    }

    info.is_loclist = nvim_qi_get_qfl_type(qi) == QFLT_LOCATION;
    info.list_count = nvim_qf_get_listcount(qi);
    info.cur_list_idx = nvim_qf_get_curlist_idx(qi);
    info.refcount = nvim_qf_get_refcount(qi);
    info.is_shared = info.refcount > 1;

    // Get entry count from current list if there is one
    // Note: We'd need to get the current list handle to get entry count
    // For now, leave it as 0 unless we add the accessor

    info
}

/// Get summary information about a window's location list.
///
/// # Safety
///
/// - `wp` may be null (returns default info with `is_loclist` = false)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_window_info(wp: WinHandle) -> LoclistInfo {
    if wp.is_null() {
        return LoclistInfo::default();
    }

    let qi = nvim_win_get_loclist(wp);
    if qi.is_null() {
        return LoclistInfo::default();
    }

    rs_ll_info(qi)
}

// =============================================================================
// Location List Entry Counts
// =============================================================================

/// Get the entry count for the current list in a location list stack.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
/// - `qfl` is the current list handle (may be null, returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_entry_count(qi: QfInfoHandle, qfl: QfListHandle) -> c_int {
    if qi.is_null() || qfl.is_null() {
        return 0;
    }

    if nvim_qi_get_qfl_type(qi) != QFLT_LOCATION {
        return 0;
    }

    nvim_qf_get_count(qfl)
}

/// Check if a window's location list is empty.
///
/// # Safety
///
/// - `wp` may be null (returns true - no list means empty)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_ll_window_is_empty(wp: WinHandle) -> bool {
    if wp.is_null() {
        return true;
    }

    let qi = nvim_win_get_loclist(wp);
    if qi.is_null() {
        return true;
    }

    nvim_qf_get_listcount(qi) <= 0
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_safety_detection() {
        unsafe {
            assert!(!rs_ll_is_loclist(std::ptr::null()));
            assert!(!rs_ll_is_quickfix(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_safety_window() {
        unsafe {
            assert!(rs_ll_get_for_window(std::ptr::null()).is_null());
            assert!(!rs_ll_window_has_loclist(std::ptr::null()));
            assert!(!rs_ll_is_loclist_window(std::ptr::null()));
            assert!(!rs_ll_window_owns_loclist(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_safety_refcount() {
        unsafe {
            assert_eq!(rs_ll_refcount(std::ptr::null()), 0);
            assert!(!rs_ll_is_shared(std::ptr::null()));
            assert!(rs_ll_can_free(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_safety_info() {
        unsafe {
            let info = rs_ll_info(std::ptr::null());
            assert!(!info.is_loclist);
            assert_eq!(info.list_count, 0);
            assert_eq!(info.refcount, 0);

            let info = rs_ll_window_info(std::ptr::null());
            assert!(!info.is_loclist);
        }
    }

    #[test]
    fn test_null_safety_entry_count() {
        unsafe {
            assert_eq!(rs_ll_entry_count(std::ptr::null(), std::ptr::null()), 0);
            assert!(rs_ll_window_is_empty(std::ptr::null()));
        }
    }

    #[test]
    fn test_loclist_info_default() {
        let info = LoclistInfo::default();
        assert!(!info.is_loclist);
        assert_eq!(info.list_count, 0);
        assert_eq!(info.cur_list_idx, 0);
        assert_eq!(info.entry_count, 0);
        assert_eq!(info.refcount, 0);
        assert!(!info.is_shared);
    }
}
