//! Location list specific functionality.
//!
//! This module provides functions specific to location lists, which are
//! window-local versions of quickfix lists. Location lists differ from
//! quickfix lists in that:
//! - Each window can have its own location list stack
//! - Location lists are freed when the window is closed
//! - Location list windows can reference another window's location list

use crate::ffi_types::QfListPtr;
use std::ffi::{c_int, c_void};

use crate::{
    nvim_qf_get_curlist_idx, nvim_qf_get_listcount, nvim_qf_get_refcount, nvim_win_get_loclist,
    QfInfoHandle, WinHandle,
};

#[allow(clippy::missing_const_for_fn)]
#[inline]
unsafe fn win_ref_const<'a>(wp: *const c_void) -> &'a nvim_window::win_struct::WinStruct {
    nvim_window::win_struct::win_ref(nvim_window::WinHandle::from_ptr(wp.cast_mut()))
}

// =============================================================================
// External C functions for location lists
// =============================================================================

extern "C" {
    /// Get the `qfl_type` from `qf_info_T`: 0=quickfix, 1=location, 2=internal.
    #[link_name = "nvim_qf_get_qi_type"]
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
        return std::ptr::null_mut();
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

    !win_ref_const(wp).w_llist_ref.is_null()
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
    let ll_ref = win_ref_const(wp).w_llist_ref;

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
pub unsafe extern "C" fn rs_ll_entry_count(qi: QfInfoHandle, qfl: QfListPtr) -> c_int {
    if qi.is_null() || qfl.is_null() {
        return 0;
    }

    if nvim_qi_get_qfl_type(qi) != QFLT_LOCATION {
        return 0;
    }

    (*qfl).qf_count
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
// Phase Q6: Location List Command Helpers (:lopen, :lnext, :lprev)
// =============================================================================

/// Location list navigation state.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct LoclistNavState {
    /// Current entry index (1-based)
    pub current_idx: c_int,
    /// Total entries in current list
    pub total_entries: c_int,
    /// Number of valid entries
    pub valid_entries: c_int,
    /// Current list index (0-based)
    pub list_idx: c_int,
    /// Total lists in stack
    pub list_count: c_int,
    /// Whether current entry is valid
    pub current_is_valid: bool,
    /// Whether we can go to next entry
    pub can_go_next: bool,
    /// Whether we can go to previous entry
    pub can_go_prev: bool,
    /// Whether we can go to newer list
    pub can_go_newer: bool,
    /// Whether we can go to older list
    pub can_go_older: bool,
}

/// Get navigation state for a window's location list.
///
/// # Safety
///
/// - `wp` may be null (returns defaults)
/// - `qfl` may be null (returns defaults)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_nav_state(wp: WinHandle, qfl: QfListPtr) -> LoclistNavState {
    let mut state = LoclistNavState::default();

    if wp.is_null() {
        return state;
    }

    let qi = nvim_win_get_loclist(wp);
    if qi.is_null() {
        return state;
    }

    state.list_count = nvim_qf_get_listcount(qi);
    state.list_idx = nvim_qf_get_curlist_idx(qi);
    state.can_go_older = state.list_idx > 0;
    state.can_go_newer = state.list_idx < state.list_count - 1;

    if !qfl.is_null() {
        state.total_entries = (*qfl).qf_count;
        state.current_idx = (*qfl).qf_index;
        state.can_go_next = state.current_idx < state.total_entries;
        state.can_go_prev = state.current_idx > 1;

        // Count valid entries and check if current is valid
        let mut qfp = (*qfl).qf_start;
        let mut idx = 1;
        while !qfp.is_null() {
            if (*qfp).qf_valid != 0 {
                state.valid_entries += 1;
                if idx == state.current_idx {
                    state.current_is_valid = true;
                }
            }
            qfp = (*qfp).qf_next;
            idx += 1;
        }
    }

    state
}

/// Check if a window can perform location list navigation.
///
/// Returns true if the window has a non-empty location list.
///
/// # Safety
///
/// - `wp` may be null (returns false)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_can_navigate(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    let qi = nvim_win_get_loclist(wp);
    if qi.is_null() {
        return false;
    }

    nvim_qf_get_listcount(qi) > 0
}

/// Calculate target index for :lnext/:lprev navigation.
///
/// Returns the target entry index, or 0 if navigation is not possible.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_calc_nav_target(
    qfl: QfListPtr,
    count: c_int,
    forward: bool,
) -> c_int {
    if qfl.is_null() || count <= 0 {
        return 0;
    }

    let current = (*qfl).qf_index;
    let total = (*qfl).qf_count;

    if total == 0 {
        return 0;
    }

    let target = if forward {
        current + count
    } else {
        current - count
    };

    // Clamp to valid range
    if target < 1 {
        1
    } else if target > total {
        total
    } else {
        target
    }
}

/// Calculate target list index for :lolder/:lnewer navigation.
///
/// Returns the target list index (0-based), or -1 if navigation is not possible.
///
/// # Safety
///
/// - `wp` may be null (returns -1)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_calc_age_target(
    wp: WinHandle,
    count: c_int,
    go_older: bool,
) -> c_int {
    if wp.is_null() || count <= 0 {
        return -1;
    }

    let qi = nvim_win_get_loclist(wp);
    if qi.is_null() {
        return -1;
    }

    let current = nvim_qf_get_curlist_idx(qi);
    let list_count = nvim_qf_get_listcount(qi);

    let target = if go_older {
        current - count
    } else {
        current + count
    };

    if target < 0 || target >= list_count {
        -1
    } else {
        target
    }
}

/// Get the number of steps available in a direction.
///
/// Returns how many entries can be navigated forward/backward.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_available_nav_steps(qfl: QfListPtr, forward: bool) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current = (*qfl).qf_index;
    let total = (*qfl).qf_count;

    if forward {
        total - current
    } else {
        current - 1
    }
}

/// Get the number of lists available in a direction.
///
/// Returns how many older/newer lists can be navigated to.
///
/// # Safety
///
/// - `wp` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_available_age_steps(wp: WinHandle, go_older: bool) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let qi = nvim_win_get_loclist(wp);
    if qi.is_null() {
        return 0;
    }

    let current = nvim_qf_get_curlist_idx(qi);
    let list_count = nvim_qf_get_listcount(qi);

    if go_older {
        current
    } else {
        list_count - current - 1
    }
}

/// Information for opening the location list window.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct LoclistOpenInfo {
    /// Whether we should open the window
    pub should_open: bool,
    /// Recommended height for the window
    pub height: c_int,
    /// Number of entries in the list
    pub entry_count: c_int,
    /// Current entry index
    pub current_idx: c_int,
    /// Whether this is a new list
    pub is_new_list: bool,
}

/// Get information needed to open a location list window.
///
/// # Safety
///
/// - `wp` may be null (returns defaults with `should_open`=false)
/// - `qfl` may be null (returns defaults with `should_open`=false)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_open_info(
    wp: WinHandle,
    qfl: QfListPtr,
    min_height: c_int,
    max_height: c_int,
) -> LoclistOpenInfo {
    let mut info = LoclistOpenInfo::default();

    if wp.is_null() || qfl.is_null() {
        return info;
    }

    let qi = nvim_win_get_loclist(wp);
    if qi.is_null() {
        return info;
    }

    let count = (*qfl).qf_count;
    if count == 0 {
        return info;
    }

    info.should_open = true;
    info.entry_count = count;
    info.current_idx = (*qfl).qf_index;
    info.height = count.clamp(min_height.max(1), max_height);
    info.is_new_list = nvim_qf_get_curlist_idx(qi) == nvim_qf_get_listcount(qi) - 1;

    info
}

/// Check if a location list window needs to be updated.
///
/// Returns true if the buffer line count differs from entry count.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_window_needs_update(qfl: QfListPtr, buf_line_count: c_int) -> bool {
    if qfl.is_null() {
        return false;
    }

    let count = (*qfl).qf_count;
    buf_line_count != count
}

/// Result of a location list command.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct LoclistCmdResult {
    /// Whether the command succeeded
    pub success: bool,
    /// New entry index (for navigation)
    pub new_idx: c_int,
    /// New list index (for age navigation)
    pub new_list_idx: c_int,
    /// Whether to update the window
    pub update_window: bool,
    /// Whether to jump to entry
    pub jump_to_entry: bool,
}

/// Calculate result for :ll (jump to entry) command.
///
/// # Safety
///
/// - `qfl` may be null (returns failure result)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_cmd_ll_result(
    qfl: QfListPtr,
    target_idx: c_int,
) -> LoclistCmdResult {
    let mut result = LoclistCmdResult::default();

    if qfl.is_null() {
        return result;
    }

    let count = (*qfl).qf_count;
    if count == 0 || target_idx < 1 || target_idx > count {
        return result;
    }

    result.success = true;
    result.new_idx = target_idx;
    result.update_window = true;
    result.jump_to_entry = true;

    result
}

/// Calculate result for :lnext/:lprev navigation command.
///
/// # Safety
///
/// - `qfl` may be null (returns failure result)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_cmd_nav_result(
    qfl: QfListPtr,
    count: c_int,
    forward: bool,
) -> LoclistCmdResult {
    let mut result = LoclistCmdResult::default();

    if qfl.is_null() || count <= 0 {
        return result;
    }

    let current = (*qfl).qf_index;
    let total = (*qfl).qf_count;

    if total == 0 {
        return result;
    }

    let target = if forward {
        (current + count).min(total)
    } else {
        (current - count).max(1)
    };

    if target == current {
        return result;
    }

    result.success = true;
    result.new_idx = target;
    result.update_window = true;
    result.jump_to_entry = true;

    result
}

/// Calculate result for :lolder/:lnewer command.
///
/// # Safety
///
/// - `wp` may be null (returns failure result)
#[no_mangle]
pub unsafe extern "C" fn rs_ll_cmd_age_result(
    wp: WinHandle,
    count: c_int,
    go_older: bool,
) -> LoclistCmdResult {
    let mut result = LoclistCmdResult::default();

    if wp.is_null() || count <= 0 {
        return result;
    }

    let qi = nvim_win_get_loclist(wp);
    if qi.is_null() {
        return result;
    }

    let current = nvim_qf_get_curlist_idx(qi);
    let list_count = nvim_qf_get_listcount(qi);

    let target = if go_older {
        current - count
    } else {
        current + count
    };

    if target < 0 || target >= list_count {
        return result;
    }

    result.success = true;
    result.new_list_idx = target;
    result.update_window = true;

    result
}

// =============================================================================
// Phase 3: copy_loclist_entries, copy_loclist
// =============================================================================

use std::ffi::c_char;

use crate::nvim_qf_alloc_next_id;

extern "C" {
    fn callback_copy(to: *mut ::std::ffi::c_void, from: *const ::std::ffi::c_void);
    fn xcalloc(count: usize, size: usize) -> *mut ::std::ffi::c_void;
    fn xfree(ptr: *mut ::std::ffi::c_void);
    fn xstrdup(s: *const ::std::ffi::c_char) -> *mut ::std::ffi::c_char;
    fn tv_copy(from: *const ::std::ffi::c_void, to: *mut ::std::ffi::c_void);
}

const QF_FAIL: c_int = 0;
const OK: c_int = 1;

/// Copy all entries from `from_qfl` to `to_qfl`.
///
/// After copying each entry, also copies `qf_fnum` and `qf_type` fields (which
/// `rs_qf_add_entry` does not set when no file/directory is supplied).
/// Also tracks the current entry pointer (`qf_ptr`) from the source.
///
/// Returns OK on success, FAIL on allocation error.
///
/// Mirrors C `copy_loclist_entries`.
///
/// # Safety
///
/// - `from_qfl` must be a valid non-null pointer to `qf_list_T`
/// - `to_qfl` must be a valid non-null pointer to `qf_list_T`
#[no_mangle]
pub unsafe extern "C" fn rs_copy_loclist_entries(from_qfl: QfListPtr, to_qfl: QfListPtr) -> c_int {
    if from_qfl.is_null() || to_qfl.is_null() {
        return QF_FAIL;
    }

    let from_ptr = (*from_qfl).qf_ptr;
    let mut qfp = (*from_qfl).qf_start;

    while !qfp.is_null() {
        let module = (*qfp).qf_module;
        let text = (*qfp).qf_text;
        let lnum = (*qfp).qf_lnum;
        let end_lnum = (*qfp).qf_end_lnum;
        let col = (*qfp).qf_col;
        let end_col = (*qfp).qf_end_col;
        let viscol: c_char = c_char::from((*qfp).qf_viscol != 0);
        let pattern = (*qfp).qf_pattern;
        let nr = (*qfp).qf_nr;
        let user_data = (&raw mut (*qfp).qf_user_data).cast::<::std::ffi::c_void>();
        let valid: c_char = c_char::from((*qfp).qf_valid != 0);

        let ret = crate::rs_qf_add_entry(
            to_qfl,
            std::ptr::null_mut(), // dir
            std::ptr::null_mut(), // fname
            module,
            0, // bufnum
            text,
            lnum,
            end_lnum,
            col,
            end_col,
            viscol,
            pattern,
            nr,
            0, // type (set below)
            user_data,
            valid,
        );

        if ret == QF_FAIL {
            return QF_FAIL;
        }

        // Copy fnum and type (not set by rs_qf_add_entry without fname/dir)
        let prevp = (*to_qfl.cast_const()).qf_last;
        if !prevp.is_null() {
            let prevp_mut = prevp;
            (*prevp_mut).qf_fnum = (*qfp).qf_fnum;
            (*prevp_mut).qf_type = (*qfp).qf_type;
            // Track current entry pointer
            if std::ptr::eq(qfp, from_ptr) {
                (*to_qfl).qf_ptr = prevp;
            }
        }

        qfp = (*qfp).qf_next;
    }

    OK
}

/// Copy location list metadata and entries from `from_qfl` to `to_qfl`.
///
/// Copies: type, nonevalid, `has_user_data`, title, ctx, callback, entries,
/// index, a fresh ID, and resets changedtick. When no valid entries exist,
/// sets ptr to start and index to 1.
///
/// Returns OK on success, FAIL on error.
///
/// Mirrors C `copy_loclist`.
///
/// # Safety
///
/// - `from_qfl` must be a valid non-null pointer to `qf_list_T`
/// - `to_qfl` must be a valid non-null pointer to `qf_list_T`
#[no_mangle]
pub unsafe extern "C" fn rs_copy_loclist(from_qfl: QfListPtr, to_qfl: QfListPtr) -> c_int {
    if from_qfl.is_null() || to_qfl.is_null() {
        return QF_FAIL;
    }

    // Copy metadata fields
    (*to_qfl).qfl_type = (*from_qfl).qfl_type;
    (*to_qfl).qf_nonevalid = (*from_qfl).qf_nonevalid;
    (*to_qfl).qf_has_user_data = (*from_qfl).qf_has_user_data;
    (*to_qfl).qf_count = 0;
    (*to_qfl).qf_index = 0;
    (*to_qfl).qf_start = std::ptr::null_mut();
    (*to_qfl).qf_last = std::ptr::null_mut();
    (*to_qfl).qf_ptr = std::ptr::null_mut();

    // Copy title
    {
        xfree((*to_qfl).qf_title.cast());
        (*to_qfl).qf_title = if ((*from_qfl).qf_title).is_null() {
            ::std::ptr::null_mut()
        } else {
            xstrdup((*from_qfl).qf_title)
        };
    };

    // Copy context (typval)
    if (*from_qfl).qf_ctx.is_null() {
        (*to_qfl).qf_ctx = ::std::ptr::null_mut();
    } else {
        (*to_qfl).qf_ctx = xcalloc(
            1,
            ::std::mem::size_of::<[u8; crate::ffi_types::TYPVAL_SIZE]>(),
        )
        .cast();
        tv_copy((*from_qfl).qf_ctx.cast(), (*to_qfl).qf_ctx.cast());
    }

    // Copy callback
    callback_copy(
        (*to_qfl).qf_qftf_cb.as_mut_ptr().cast(),
        (*from_qfl).qf_qftf_cb.as_ptr().cast(),
    );

    // Copy entries if any
    if (*from_qfl).qf_count > 0 && rs_copy_loclist_entries(from_qfl, to_qfl) == QF_FAIL {
        return QF_FAIL;
    }

    // Restore index (copy_loclist_entries may change it)
    (*to_qfl).qf_index = (*from_qfl).qf_index;

    // Assign a new ID and reset changedtick
    let new_id = nvim_qf_alloc_next_id();
    (*to_qfl).qf_id = new_id;
    (*to_qfl).qf_changedtick = 0;

    // When no valid entries: ptr -> start, index = 1
    if (*to_qfl.cast_const()).qf_nonevalid {
        (*to_qfl).qf_ptr = (*to_qfl.cast_const()).qf_start;
        (*to_qfl).qf_index = 1;
    }

    OK
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
            assert!(!rs_ll_is_loclist(std::ptr::null_mut()));
            assert!(!rs_ll_is_quickfix(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_safety_window() {
        unsafe {
            assert!(rs_ll_get_for_window(std::ptr::null_mut()).is_null());
            assert!(!rs_ll_window_has_loclist(std::ptr::null_mut()));
            assert!(!rs_ll_is_loclist_window(std::ptr::null_mut()));
            assert!(!rs_ll_window_owns_loclist(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_safety_refcount() {
        unsafe {
            assert_eq!(rs_ll_refcount(std::ptr::null_mut()), 0);
            assert!(!rs_ll_is_shared(std::ptr::null_mut()));
            assert!(rs_ll_can_free(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_safety_info() {
        unsafe {
            let info = rs_ll_info(std::ptr::null_mut());
            assert!(!info.is_loclist);
            assert_eq!(info.list_count, 0);
            assert_eq!(info.refcount, 0);

            let info = rs_ll_window_info(std::ptr::null_mut());
            assert!(!info.is_loclist);
        }
    }

    #[test]
    fn test_null_safety_entry_count() {
        unsafe {
            assert_eq!(
                rs_ll_entry_count(std::ptr::null_mut(), std::ptr::null_mut()),
                0
            );
            assert!(rs_ll_window_is_empty(std::ptr::null_mut()));
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

    // Phase Q6 tests
    #[test]
    fn test_null_nav_state() {
        unsafe {
            let state = rs_ll_nav_state(std::ptr::null_mut(), std::ptr::null_mut());
            assert_eq!(state.current_idx, 0);
            assert_eq!(state.total_entries, 0);
            assert!(!state.can_go_next);
            assert!(!state.can_go_prev);
        }
    }

    #[test]
    fn test_null_can_navigate() {
        unsafe {
            assert!(!rs_ll_can_navigate(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_calc_nav_target() {
        unsafe {
            assert_eq!(rs_ll_calc_nav_target(std::ptr::null_mut(), 1, true), 0);
        }
    }

    #[test]
    fn test_null_calc_age_target() {
        unsafe {
            assert_eq!(rs_ll_calc_age_target(std::ptr::null_mut(), 1, true), -1);
        }
    }

    #[test]
    fn test_null_available_nav_steps() {
        unsafe {
            assert_eq!(rs_ll_available_nav_steps(std::ptr::null_mut(), true), 0);
        }
    }

    #[test]
    fn test_null_available_age_steps() {
        unsafe {
            assert_eq!(rs_ll_available_age_steps(std::ptr::null_mut(), true), 0);
        }
    }

    #[test]
    fn test_null_open_info() {
        unsafe {
            let info = rs_ll_open_info(std::ptr::null_mut(), std::ptr::null_mut(), 3, 10);
            assert!(!info.should_open);
            assert_eq!(info.entry_count, 0);
        }
    }

    #[test]
    fn test_null_window_needs_update() {
        unsafe {
            assert!(!rs_ll_window_needs_update(std::ptr::null_mut(), 10));
        }
    }

    #[test]
    fn test_null_cmd_ll_result() {
        unsafe {
            let result = rs_ll_cmd_ll_result(std::ptr::null_mut(), 1);
            assert!(!result.success);
        }
    }

    #[test]
    fn test_null_cmd_nav_result() {
        unsafe {
            let result = rs_ll_cmd_nav_result(std::ptr::null_mut(), 1, true);
            assert!(!result.success);
        }
    }

    #[test]
    fn test_null_cmd_age_result() {
        unsafe {
            let result = rs_ll_cmd_age_result(std::ptr::null_mut(), 1, true);
            assert!(!result.success);
        }
    }

    #[test]
    fn test_nav_state_default() {
        let state = LoclistNavState::default();
        assert_eq!(state.current_idx, 0);
        assert_eq!(state.total_entries, 0);
        assert!(!state.can_go_next);
        assert!(!state.can_go_prev);
    }

    #[test]
    fn test_open_info_default() {
        let info = LoclistOpenInfo::default();
        assert!(!info.should_open);
        assert_eq!(info.height, 0);
        assert_eq!(info.entry_count, 0);
    }

    #[test]
    fn test_cmd_result_default() {
        let result = LoclistCmdResult::default();
        assert!(!result.success);
        assert_eq!(result.new_idx, 0);
        assert!(!result.update_window);
        assert!(!result.jump_to_entry);
    }
}
