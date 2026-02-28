//! Quickfix list functions for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations of quickfix/location list functions.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Submodules
// =============================================================================

pub mod api;
pub mod commands;
pub mod dirstack;
pub mod display;
pub mod errorformat;
pub mod external;
pub mod filter;
pub mod helpgrep;
pub mod init;
pub mod lifecycle;
pub mod list;
pub mod listdo;
pub mod loclist;
pub mod make;
pub mod navigate;
pub mod parse;
pub mod reader;
pub mod stack;
pub mod types;
pub mod vimgrep;
pub mod window;

// Re-export commonly used types from submodules
pub use api::{QfAction, QfApiResult, QfListInfo, QfWhatFlags};
pub use display::{QfDisplayEntry, QfDisplayFormat, QfDisplayState};
pub use errorformat::{EfmFlags, EfmParseResult, QfErrorType};
pub use listdo::{FileEntry, FileListDoState, ListDoResult, ListDoState, ListDoType};
pub use stack::{NavDirection, QfPushResult, QfStackNavResult, QfStackState};
pub use types::{
    LineNr, QfEntry, QfEntryOption, QfEntrySummary, QfEntryType, QfList, QfListStats, QfListType,
    QfPosition, QfStack, QF_INVALID_BUFNR, QF_INVALID_IDX, QF_LISTCOUNT,
};

// =============================================================================
// External C accessor functions
// =============================================================================

/// Line number type (matches `linenr_T` in Neovim)
type LinenrT = i32;

/// Opaque handle to `qf_info_T` (quickfix stack)
type QfInfoHandle = *const c_void;
/// Opaque handle to `qf_list_T` (quickfix list)
type QfListHandle = *const c_void;
/// Opaque handle to `qfline_T` (quickfix entry)
type QfLineHandle = *const c_void;
/// Opaque handle to `pos_T` (position)
type PosHandle = *const c_void;

#[allow(dead_code)]
extern "C" {
    // Stack accessors
    fn nvim_qf_get_listcount(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_get_curlist(qi: QfInfoHandle) -> QfListHandle;
    fn nvim_qf_get_list_at(qi: QfInfoHandle, idx: c_int) -> QfListHandle;
    fn nvim_qf_get_curlist_idx(qi: QfInfoHandle) -> c_int;

    // List accessors
    fn nvim_qf_get_count(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_nonevalid(qfl: QfListHandle) -> bool;
    fn nvim_qf_get_index(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_ptr(qfl: QfListHandle) -> QfLineHandle;
    fn nvim_qf_get_start(qfl: QfListHandle) -> QfLineHandle;

    // Entry accessors
    fn nvim_qfline_get_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_col(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_next(qfp: QfLineHandle) -> QfLineHandle;
    fn nvim_qfline_get_prev(qfp: QfLineHandle) -> QfLineHandle;
    fn nvim_qfline_get_valid(qfp: QfLineHandle) -> bool;
    fn nvim_qfline_get_type(qfp: QfLineHandle) -> c_char;
    fn nvim_qfline_get_fnum(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_end_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_end_col(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_nr(qfp: QfLineHandle) -> c_int;

    // Phase 4: Additional entry accessors
    fn nvim_qfline_get_text(qfp: QfLineHandle) -> *const c_char;
    fn nvim_qfline_get_module(qfp: QfLineHandle) -> *const c_char;
    fn nvim_qfline_get_pattern(qfp: QfLineHandle) -> *const c_char;
    fn nvim_qfline_get_cleared(qfp: QfLineHandle) -> bool;
    fn nvim_qfline_get_viscol(qfp: QfLineHandle) -> bool;

    // Position accessors
    fn nvim_pos_get_lnum(pos: PosHandle) -> LinenrT;
    fn nvim_pos_get_col(pos: PosHandle) -> c_int;

    // Phase 1: Validation accessors
    fn nvim_win_valid(wp: *const c_void) -> bool;
    fn nvim_win_get_loclist(wp: *const c_void) -> QfInfoHandle;
    fn nvim_qf_find_win_handle(qi: QfInfoHandle) -> *const c_void;
    fn nvim_qf_win_get_handle(wp: *const c_void) -> c_int;
    // nvim_qflist_valid removed: logic inlined into rs_qflist_valid (Phase 16)
    // nvim_qf_entry_present removed: logic inlined into rs_qf_entry_present (Phase 16)
}

// =============================================================================
// Quickfix Stack Functions
// =============================================================================

/// Returns true if the specified quickfix/location stack is empty.
///
/// A stack is empty if the pointer is null or `qf_listcount` <= 0.
///
/// # Safety
///
/// - `qi` may be null (in which case it's considered empty)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_stack_empty(qi: *const c_void) -> bool {
    if qi.is_null() {
        return true;
    }
    nvim_qf_get_listcount(qi) <= 0
}

// =============================================================================
// Quickfix List Functions
// =============================================================================

/// Returns true if the specified quickfix/location list is empty.
///
/// A list is empty if the pointer is null or `qf_count` <= 0.
///
/// # Safety
///
/// - `qfl` may be null (in which case it's considered empty)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_list_empty(qfl: *const c_void) -> bool {
    if qfl.is_null() {
        return true;
    }
    nvim_qf_get_count(qfl) <= 0
}

/// Returns true if the specified quickfix/location list has valid entries.
///
/// A list has valid entries if it is not empty and `qf_nonevalid` is false.
///
/// # Safety
///
/// - `qfl` may be null (in which case it returns false - no valid entries)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_list_has_valid_entries(qfl: *const c_void) -> bool {
    if qfl.is_null() {
        return false;
    }
    // Has valid entries if list is not empty and not marked as nonevalid
    !rs_qf_list_empty(qfl) && !nvim_qf_get_nonevalid(qfl)
}

// =============================================================================
// Quickfix Entry Position Functions
// =============================================================================

/// Returns true if the specified quickfix entry is after the given position.
///
/// If `linewise` is true, compares only line numbers.
/// Otherwise, compares both line and column.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `pos` must be a valid pointer to a `pos_T` struct
#[no_mangle]
#[allow(clippy::suspicious_operation_groupings)]
pub unsafe extern "C" fn rs_qf_entry_after_pos(
    qfp: *const c_void,
    pos: *const c_void,
    linewise: bool,
) -> bool {
    let qf_lnum = nvim_qfline_get_lnum(qfp);
    let pos_lnum = nvim_pos_get_lnum(pos);

    if linewise {
        return qf_lnum > pos_lnum;
    }

    let qf_col = nvim_qfline_get_col(qfp);
    let pos_col = nvim_pos_get_col(pos);
    qf_lnum > pos_lnum || (qf_lnum == pos_lnum && qf_col > pos_col)
}

/// Returns true if the specified quickfix entry is before the given position.
///
/// If `linewise` is true, compares only line numbers.
/// Otherwise, compares both line and column.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `pos` must be a valid pointer to a `pos_T` struct
#[no_mangle]
#[allow(clippy::suspicious_operation_groupings)]
pub unsafe extern "C" fn rs_qf_entry_before_pos(
    qfp: *const c_void,
    pos: *const c_void,
    linewise: bool,
) -> bool {
    let qf_lnum = nvim_qfline_get_lnum(qfp);
    let pos_lnum = nvim_pos_get_lnum(pos);

    if linewise {
        return qf_lnum < pos_lnum;
    }

    let qf_col = nvim_qfline_get_col(qfp);
    let pos_col = nvim_pos_get_col(pos);
    qf_lnum < pos_lnum || (qf_lnum == pos_lnum && qf_col < pos_col)
}

/// Returns true if the specified quickfix entry is on or after the given position.
///
/// If `linewise` is true, compares only line numbers.
/// Otherwise, compares both line and column.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `pos` must be a valid pointer to a `pos_T` struct
#[no_mangle]
#[allow(clippy::suspicious_operation_groupings)]
pub unsafe extern "C" fn rs_qf_entry_on_or_after_pos(
    qfp: *const c_void,
    pos: *const c_void,
    linewise: bool,
) -> bool {
    let qf_lnum = nvim_qfline_get_lnum(qfp);
    let pos_lnum = nvim_pos_get_lnum(pos);

    if linewise {
        return qf_lnum >= pos_lnum;
    }

    let qf_col = nvim_qfline_get_col(qfp);
    let pos_col = nvim_pos_get_col(pos);
    qf_lnum > pos_lnum || (qf_lnum == pos_lnum && qf_col >= pos_col)
}

/// Returns true if the specified quickfix entry is on or before the given position.
///
/// If `linewise` is true, compares only line numbers.
/// Otherwise, compares both line and column.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `pos` must be a valid pointer to a `pos_T` struct
#[no_mangle]
#[allow(clippy::suspicious_operation_groupings)]
pub unsafe extern "C" fn rs_qf_entry_on_or_before_pos(
    qfp: *const c_void,
    pos: *const c_void,
    linewise: bool,
) -> bool {
    let qf_lnum = nvim_qfline_get_lnum(qfp);
    let pos_lnum = nvim_pos_get_lnum(pos);

    if linewise {
        return qf_lnum <= pos_lnum;
    }

    let qf_col = nvim_qfline_get_col(qfp);
    let pos_col = nvim_pos_get_col(pos);
    qf_lnum < pos_lnum || (qf_lnum == pos_lnum && qf_col <= pos_col)
}

// =============================================================================
// List Index Validation Functions
// =============================================================================

/// Get the number of quickfix lists in the stack.
///
/// Returns 0 if the stack is null.
///
/// # Safety
///
/// - `qi` may be null (in which case 0 is returned)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_listcount(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return 0;
    }
    nvim_qf_get_listcount(qi)
}

/// Get the current quickfix list index (0-based).
///
/// Returns -1 if the stack is null or empty.
///
/// # Safety
///
/// - `qi` may be null (in which case -1 is returned)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_curlist_idx(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return -1;
    }
    nvim_qf_get_curlist_idx(qi)
}

/// Get the current quickfix list.
///
/// Returns null if the stack is null or empty.
///
/// # Safety
///
/// - `qi` may be null (in which case null is returned)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_curlist(qi: QfInfoHandle) -> QfListHandle {
    if qi.is_null() {
        return std::ptr::null();
    }
    nvim_qf_get_curlist(qi)
}

/// Get a quickfix list at the specified index.
///
/// Returns null if the index is out of bounds.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
/// - `idx` must be in range [0, listcount)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_list_at(qi: QfInfoHandle, idx: c_int) -> QfListHandle {
    if qi.is_null() || !rs_qf_valid_idx(qi, idx) {
        return std::ptr::null();
    }
    nvim_qf_get_list_at(qi, idx)
}

/// Validates that the given index is valid for the quickfix stack.
///
/// Returns true if `idx` is in range [0, listcount).
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_valid_idx(qi: QfInfoHandle, idx: c_int) -> bool {
    if qi.is_null() {
        return false;
    }
    let listcount = nvim_qf_get_listcount(qi);
    idx >= 0 && idx < listcount
}

/// Returns the size (entry count) of the specified quickfix list.
///
/// Returns 0 if the list is null.
///
/// # Safety
///
/// - `qfl` may be null (in which case 0 is returned)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_size(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }
    nvim_qf_get_count(qfl)
}

// =============================================================================
// Phase 1: List Query and Validation Functions
// =============================================================================

/// Opaque handle to `win_T` (window)
type WinHandle = *const c_void;

/// Check if a quickfix/location list with the given identifier exists.
///
/// For quickfix lists (`wp` is NULL), checks the global quickfix stack.
/// For location lists (`wp` is non-NULL), checks the window's location list.
///
/// Returns true if the list with the given ID exists in the stack.
///
/// # Safety
///
/// - `wp` may be null (checks global quickfix stack)
/// - If non-null, `wp` must be a valid pointer to a `win_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qflist_valid(wp: WinHandle, qf_id: u32) -> bool {
    // Get the appropriate quickfix stack
    let qi = if wp.is_null() {
        // Use global quickfix stack
        nvim_get_ql_info()
    } else {
        // Check if window is valid first
        if !nvim_win_valid(wp) {
            return false;
        }
        // Get location list for window
        nvim_win_get_loclist(wp)
    };

    if qi.is_null() {
        return false;
    }

    // Iterate over all lists in the stack checking for qf_id match.
    // Replaces C nvim_qflist_valid (Phase 16).
    let listcount = nvim_qf_get_listcount(qi);
    for i in 0..listcount {
        let qfl = nvim_qf_get_list_at(qi, i);
        if !qfl.is_null() && nvim_qf_get_id(qfl) == qf_id {
            return true;
        }
    }
    false
}

/// Check if a quickfix entry is present in the quickfix list.
///
/// When loading a file from the quickfix, autocommands may modify it.
/// This may invalidate the current quickfix entry. This function checks
/// whether an entry is still present in the quickfix list.
///
/// Returns true if the entry pointer is found in the list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `qf_ptr` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_present(qfl: QfListHandle, qf_ptr: QfLineHandle) -> bool {
    if qfl.is_null() || qf_ptr.is_null() {
        return false;
    }
    // Walk the linked list checking for pointer equality.
    // Replaces C nvim_qf_entry_present (Phase 16).
    // Note: C version checked got_int at each step; we omit that because
    // false negatives from interruption were already acceptable there.
    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_start(qfl);
    for _ in 0..count {
        if qfp.is_null() {
            break;
        }
        if qfp == qf_ptr {
            return true;
        }
        qfp = nvim_qfline_get_next(qfp);
    }
    false
}

/// Convert a quickfix list ID to its index in the stack.
///
/// Returns the 0-based index of the list with the given ID, or -1 if not found.
/// This is equivalent to the C function `qf_id2nr()`.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_id2nr(qi: QfInfoHandle, qf_id: u32) -> c_int {
    // Use the existing rs_qf_find_list_by_id which does the same thing
    rs_qf_find_list_by_id(qi, qf_id)
}

/// Restore the current quickfix list to one with the given ID.
///
/// If the current list already has the specified ID, returns OK (1).
/// If the list with that ID is not found, returns FAIL (0).
/// Otherwise, sets the current list to the one with the specified ID.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_restore_list(qi: QfInfoHandleMut, save_qfid: u32) -> c_int {
    if qi.is_null() {
        return 0; // FAIL
    }

    // Check if current list already has the specified ID
    let curlist = nvim_qf_get_curlist(qi);
    if !curlist.is_null() {
        let cur_id = nvim_qf_get_id(curlist);
        if cur_id == save_qfid {
            return 1; // OK - already on the right list
        }
    }

    // Find the list with the specified ID
    let idx = rs_qf_id2nr(qi, save_qfid);
    if idx < 0 {
        return 0; // FAIL - list not found
    }

    // Set the current list
    nvim_qf_set_curlist_idx(qi, idx);
    1 // OK
}

/// Get the quickfix/location list window ID.
///
/// Returns the window handle (ID) of the quickfix window displaying the
/// specified quickfix stack, or 0 if no window is found.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_winid(qi: QfInfoHandle) -> c_int {
    // The quickfix window can be opened even if the quickfix list is not set
    // using ":copen". This is not true for location lists.
    // Uses Rust window iteration (migrated in Phase 10, Pass 10).
    if qi.is_null() {
        return 0;
    }

    let win = rs_qf_find_win_for_stack(qi);
    if win.is_null() {
        return 0;
    }

    nvim_qf_win_get_handle(win)
}

// =============================================================================
// Entry Navigation Functions
// =============================================================================

/// Find the first entry in the quickfix list with the specified file number.
///
/// Returns the entry pointer and sets `out_idx` to the 1-based index.
/// Returns NULL if no entry is found for the buffer.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `out_idx` must be a valid pointer to write the index
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_first_entry_in_buf(
    qfl: QfListHandle,
    bnr: c_int,
    out_idx: *mut c_int,
) -> QfLineHandle {
    if qfl.is_null() || out_idx.is_null() {
        return std::ptr::null();
    }

    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() && idx <= count {
        if nvim_qfline_get_fnum(qfp) == bnr {
            *out_idx = idx;
            return qfp;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    *out_idx = 0;
    std::ptr::null()
}

/// Find the first quickfix entry on the same line as the given entry.
///
/// Walks backward through entries with the same file number and line number.
/// Updates `out_idx` with the resulting 1-based index.
///
/// # Safety
///
/// - `entry` must be a valid pointer to a `qfline_T` struct
/// - `idx` is the current 1-based index of the entry
/// - `out_idx` must be a valid pointer to write the result index
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_first_entry_on_line(
    entry: QfLineHandle,
    idx: c_int,
    out_idx: *mut c_int,
) -> QfLineHandle {
    if out_idx.is_null() {
        return entry;
    }
    if entry.is_null() {
        *out_idx = idx;
        return entry;
    }

    let mut current = entry;
    let mut current_idx = idx;
    let target_fnum = nvim_qfline_get_fnum(entry);
    let target_line = nvim_qfline_get_lnum(entry);

    loop {
        let prev = nvim_qfline_get_prev(current);
        if prev.is_null() {
            break;
        }
        if nvim_qfline_get_fnum(prev) != target_fnum {
            break;
        }
        if nvim_qfline_get_lnum(prev) != target_line {
            break;
        }
        current = prev;
        current_idx -= 1;
    }

    *out_idx = current_idx;
    current
}

/// Find the last quickfix entry on the same line as the given entry.
///
/// Walks forward through entries with the same file number and line number.
/// Updates `out_idx` with the resulting 1-based index.
///
/// # Safety
///
/// - `entry` must be a valid pointer to a `qfline_T` struct
/// - `idx` is the current 1-based index of the entry
/// - `out_idx` must be a valid pointer to write the result index
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_last_entry_on_line(
    entry: QfLineHandle,
    idx: c_int,
    out_idx: *mut c_int,
) -> QfLineHandle {
    if out_idx.is_null() {
        return entry;
    }
    if entry.is_null() {
        *out_idx = idx;
        return entry;
    }

    let mut current = entry;
    let mut current_idx = idx;
    let target_fnum = nvim_qfline_get_fnum(entry);
    let target_line = nvim_qfline_get_lnum(entry);

    loop {
        let next = nvim_qfline_get_next(current);
        if next.is_null() {
            break;
        }
        if nvim_qfline_get_fnum(next) != target_fnum {
            break;
        }
        if nvim_qfline_get_lnum(next) != target_line {
            break;
        }
        current = next;
        current_idx += 1;
    }

    *out_idx = current_idx;
    current
}

/// Direction for navigation functions
#[repr(C)]
pub enum Direction {
    Forward = 1,
    Backward = -1,
}

/// Find the first quickfix entry after the specified position in the given buffer.
///
/// If `linewise` is true, compares only line numbers.
/// `qfp` should be the first entry in the buffer, `errornr` its 1-based index.
/// Returns NULL if no entry is found after the position.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `pos` must be a valid pointer to a `pos_T` struct
/// - `errornr` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_entry_after_pos(
    bnr: c_int,
    pos: PosHandle,
    linewise: bool,
    qfp: QfLineHandle,
    errornr: *mut c_int,
) -> QfLineHandle {
    if qfp.is_null() || pos.is_null() || errornr.is_null() {
        return std::ptr::null();
    }

    // If first entry is already after the position, return it
    if rs_qf_entry_after_pos(qfp, pos, linewise) {
        return qfp;
    }

    // Find the entry just before or at the position
    let mut current = qfp;
    loop {
        let next = nvim_qfline_get_next(current);
        if next.is_null() || nvim_qfline_get_fnum(next) != bnr {
            break;
        }
        if !rs_qf_entry_on_or_before_pos(next, pos, linewise) {
            break;
        }
        current = next;
        *errornr += 1;
    }

    let next = nvim_qfline_get_next(current);
    if next.is_null() || nvim_qfline_get_fnum(next) != bnr {
        return std::ptr::null();
    }

    *errornr += 1;
    next
}

/// Find the first quickfix entry before the specified position in the given buffer.
///
/// If `linewise` is true, compares only line numbers.
/// `qfp` should be the first entry in the buffer, `errornr` its 1-based index.
/// Returns NULL if no entry is found before the position.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `pos` must be a valid pointer to a `pos_T` struct
/// - `errornr` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_entry_before_pos(
    bnr: c_int,
    pos: PosHandle,
    linewise: bool,
    qfp: QfLineHandle,
    errornr: *mut c_int,
) -> QfLineHandle {
    if qfp.is_null() || pos.is_null() || errornr.is_null() {
        return std::ptr::null();
    }

    // Find the entry just before the position
    let mut current = qfp;
    loop {
        let next = nvim_qfline_get_next(current);
        if next.is_null() || nvim_qfline_get_fnum(next) != bnr {
            break;
        }
        if !rs_qf_entry_before_pos(next, pos, linewise) {
            break;
        }
        current = next;
        *errornr += 1;
    }

    // If the current entry is on or after the position, no entry exists before
    if rs_qf_entry_on_or_after_pos(current, pos, linewise) {
        return std::ptr::null();
    }

    // If linewise, find the first entry on this line
    if linewise {
        current = rs_qf_find_first_entry_on_line(current, *errornr, errornr);
    }

    current
}

/// Find a quickfix entry closest to the specified position in the given direction.
///
/// Returns the entry and sets `errornr` to its 1-based index.
/// Returns NULL if no entry is found.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `pos` must be a valid pointer to a `pos_T` struct
/// - `errornr` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_closest_entry(
    qfl: QfListHandle,
    bnr: c_int,
    pos: PosHandle,
    dir: c_int,
    linewise: bool,
    errornr: *mut c_int,
) -> QfLineHandle {
    if qfl.is_null() || pos.is_null() || errornr.is_null() {
        return std::ptr::null();
    }

    *errornr = 0;

    // Find the first entry in this file
    let qfp = rs_qf_find_first_entry_in_buf(qfl, bnr, errornr);
    if qfp.is_null() {
        return std::ptr::null();
    }

    if dir > 0 {
        // FORWARD
        rs_qf_find_entry_after_pos(bnr, pos, linewise, qfp, errornr)
    } else {
        // BACKWARD
        rs_qf_find_entry_before_pos(bnr, pos, linewise, qfp, errornr)
    }
}

// =============================================================================
// Phase 8: Nth Adjacent Entry Navigation
// =============================================================================

/// Get the nth quickfix entry below the specified entry.
///
/// Searches forward in the list. If `linewise` is true, treats multiple entries
/// on a single line as one.
///
/// Updates `errornr` to the final entry's 1-based index.
///
/// # Safety
///
/// - `entry` must be a valid pointer to a `qfline_T` struct
/// - `errornr` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_nth_below_entry(
    entry: QfLineHandle,
    n: LinenrT,
    linewise: bool,
    errornr: *mut c_int,
) {
    if entry.is_null() || errornr.is_null() || n <= 0 {
        return;
    }

    let mut current = entry;
    let mut remaining = n;
    let current_fnum = nvim_qfline_get_fnum(entry);

    while remaining > 0 {
        let first_errornr = *errornr;

        if linewise {
            // Treat all entries on the same line as one
            current = rs_qf_find_last_entry_on_line(current, *errornr, errornr);
        }

        // Check if we can move to next entry in same file
        let next = nvim_qfline_get_next(current);
        if next.is_null() || nvim_qfline_get_fnum(next) != current_fnum {
            if linewise {
                *errornr = first_errornr;
            }
            break;
        }

        current = next;
        *errornr += 1;
        remaining -= 1;
    }
}

/// Get the nth quickfix entry above the specified entry.
///
/// Searches backward in the list. If `linewise` is true, treats multiple entries
/// on a single line as one.
///
/// Updates `errornr` to the final entry's 1-based index.
///
/// # Safety
///
/// - `entry` must be a valid pointer to a `qfline_T` struct
/// - `errornr` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_nth_above_entry(
    entry: QfLineHandle,
    n: LinenrT,
    linewise: bool,
    errornr: *mut c_int,
) {
    if entry.is_null() || errornr.is_null() || n <= 0 {
        return;
    }

    let mut current = entry;
    let mut remaining = n;
    let current_fnum = nvim_qfline_get_fnum(entry);

    while remaining > 0 {
        // Check if we can move to previous entry in same file
        let prev = nvim_qfline_get_prev(current);
        if prev.is_null() || nvim_qfline_get_fnum(prev) != current_fnum {
            break;
        }

        current = prev;
        *errornr -= 1;

        if linewise {
            // Find first entry on this line
            current = rs_qf_find_first_entry_on_line(current, *errornr, errornr);
        }

        remaining -= 1;
    }
}

/// Find the nth adjacent quickfix entry to position in the specified direction.
///
/// Returns the 1-based error number of the found entry, or 0 if not found.
///
/// # Arguments
///
/// * `qfl` - Quickfix list handle
/// * `bnr` - Buffer number to search in
/// * `pos` - Position to search from
/// * `n` - Number of entries to skip (0 means find closest)
/// * `dir` - Direction (>0 for forward, <0 for backward)
/// * `linewise` - If true, treat multiple entries on same line as one
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `pos` must be a valid pointer to a `pos_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_nth_adj_entry(
    qfl: QfListHandle,
    bnr: c_int,
    pos: PosHandle,
    n: LinenrT,
    dir: c_int,
    linewise: bool,
) -> c_int {
    if qfl.is_null() || pos.is_null() {
        return 0;
    }

    let mut errornr: c_int = 0;

    // Find the closest entry to the specified position
    let adj_entry = rs_qf_find_closest_entry(qfl, bnr, pos, dir, linewise, &raw mut errornr);
    if adj_entry.is_null() {
        return 0;
    }

    // If n > 1, go to the nth entry in the current buffer
    let remaining = n - 1;
    if remaining > 0 {
        if dir > 0 {
            // FORWARD
            rs_qf_get_nth_below_entry(adj_entry, remaining, linewise, &raw mut errornr);
        } else {
            // BACKWARD
            rs_qf_get_nth_above_entry(adj_entry, remaining, linewise, &raw mut errornr);
        }
    }

    errornr
}

// =============================================================================
// Phase 4: Quickfix List Management Functions
// =============================================================================

/// Count the number of entries in the quickfix list for a specific file.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_count_in_file(qfl: QfListHandle, bnr: c_int) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_start(qfl);
    let mut file_count = 0;
    let mut idx = 0;

    while !qfp.is_null() && idx < count {
        if nvim_qfline_get_fnum(qfp) == bnr {
            file_count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    file_count
}

/// Count the number of valid entries in the quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_valid_entries(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_start(qfl);
    let mut valid_count = 0;
    let mut idx = 0;

    while !qfp.is_null() && idx < count {
        if nvim_qfline_get_valid(qfp) {
            valid_count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    valid_count
}

/// Get the Nth valid entry in the quickfix list (1-based N).
///
/// Returns the entry and sets `out_idx` to its 1-based index.
/// Returns NULL if N is out of range.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `out_idx` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_nth_valid_entry(
    qfl: QfListHandle,
    n: c_int,
    out_idx: *mut c_int,
) -> QfLineHandle {
    if qfl.is_null() || out_idx.is_null() || n <= 0 {
        return std::ptr::null();
    }

    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_start(qfl);
    let mut valid_count = 0;
    let mut idx = 1;

    while !qfp.is_null() && idx <= count {
        if nvim_qfline_get_valid(qfp) {
            valid_count += 1;
            if valid_count == n {
                *out_idx = idx;
                return qfp;
            }
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    *out_idx = 0;
    std::ptr::null()
}

/// Get the entry at the specified 1-based index in the quickfix list.
///
/// Returns NULL if the index is out of range.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_entry_at_idx(qfl: QfListHandle, idx: c_int) -> QfLineHandle {
    if qfl.is_null() || idx <= 0 {
        return std::ptr::null();
    }

    let count = nvim_qf_get_count(qfl);
    if idx > count {
        return std::ptr::null();
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut current_idx = 1;

    while !qfp.is_null() && current_idx < idx {
        qfp = nvim_qfline_get_next(qfp);
        current_idx += 1;
    }

    qfp
}

// =============================================================================
// Phase 4: Iterator Infrastructure
// =============================================================================

/// Iterator state for quickfix list traversal.
///
/// This structure maintains the state for iterating over quickfix entries.
/// It's designed to be used from C code via the `rs_qf_iter_*` functions.
#[repr(C)]
pub struct QfIterator {
    /// Current entry pointer
    current: QfLineHandle,
    /// Current 1-based index
    idx: c_int,
    /// Total count of entries
    count: c_int,
}

/// Initialize a new iterator for the quickfix list.
///
/// Returns an iterator starting at the first entry.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_iter_init(qfl: QfListHandle) -> QfIterator {
    if qfl.is_null() {
        return QfIterator {
            current: std::ptr::null(),
            idx: 0,
            count: 0,
        };
    }

    QfIterator {
        current: nvim_qf_get_start(qfl),
        idx: 1,
        count: nvim_qf_get_count(qfl),
    }
}

/// Check if the iterator has more entries.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" cannot be const
pub extern "C" fn rs_qf_iter_has_next(iter: &QfIterator) -> bool {
    !iter.current.is_null() && iter.idx <= iter.count
}

/// Get the current entry from the iterator.
///
/// Returns null if the iterator is exhausted.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" cannot be const
pub extern "C" fn rs_qf_iter_current(iter: &QfIterator) -> QfLineHandle {
    iter.current
}

/// Get the current 1-based index from the iterator.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" cannot be const
pub extern "C" fn rs_qf_iter_idx(iter: &QfIterator) -> c_int {
    iter.idx
}

/// Advance the iterator to the next entry.
///
/// # Safety
///
/// - `iter` must be a valid iterator obtained from `rs_qf_iter_init`
#[no_mangle]
pub unsafe extern "C" fn rs_qf_iter_next(iter: &mut QfIterator) {
    if !iter.current.is_null() {
        iter.current = nvim_qfline_get_next(iter.current);
        iter.idx += 1;
    }
}

/// Advance the iterator to the next valid entry.
///
/// Returns the found valid entry or null if none.
///
/// # Safety
///
/// - `iter` must be a valid iterator obtained from `rs_qf_iter_init`
#[no_mangle]
pub unsafe extern "C" fn rs_qf_iter_next_valid(iter: &mut QfIterator) -> QfLineHandle {
    while !iter.current.is_null() && iter.idx <= iter.count {
        if nvim_qfline_get_valid(iter.current) {
            let result = iter.current;
            rs_qf_iter_next(iter);
            return result;
        }
        rs_qf_iter_next(iter);
    }
    std::ptr::null()
}

/// Advance the iterator to the next entry in a specific file.
///
/// Returns the found entry or null if none in the file.
///
/// # Safety
///
/// - `iter` must be a valid iterator obtained from `rs_qf_iter_init`
#[no_mangle]
pub unsafe extern "C" fn rs_qf_iter_next_in_file(
    iter: &mut QfIterator,
    bnr: c_int,
) -> QfLineHandle {
    while !iter.current.is_null() && iter.idx <= iter.count {
        if nvim_qfline_get_fnum(iter.current) == bnr {
            let result = iter.current;
            rs_qf_iter_next(iter);
            return result;
        }
        rs_qf_iter_next(iter);
    }
    std::ptr::null()
}

/// Reset the iterator to the beginning of the list.
///
/// # Safety
///
/// - `iter` must be a valid iterator obtained from `rs_qf_iter_init`
/// - `qfl` must be the same list used to create the iterator
#[no_mangle]
pub unsafe extern "C" fn rs_qf_iter_reset(iter: &mut QfIterator, qfl: QfListHandle) {
    if qfl.is_null() {
        iter.current = std::ptr::null();
        iter.idx = 0;
        iter.count = 0;
    } else {
        iter.current = nvim_qf_get_start(qfl);
        iter.idx = 1;
        iter.count = nvim_qf_get_count(qfl);
    }
}

/// Get the first entry of a specific type.
///
/// Returns the entry and sets `out_idx` to its 1-based index.
/// Returns NULL if no entry of that type is found.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `out_idx` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_first_of_type(
    qfl: QfListHandle,
    type_char: c_char,
    out_idx: *mut c_int,
) -> QfLineHandle {
    if qfl.is_null() || out_idx.is_null() {
        return std::ptr::null();
    }

    let mut iter = rs_qf_iter_init(qfl);
    while rs_qf_iter_has_next(&iter) {
        let qfp = rs_qf_iter_current(&iter);
        if nvim_qfline_get_type(qfp) == type_char {
            *out_idx = rs_qf_iter_idx(&iter);
            return qfp;
        }
        rs_qf_iter_next(&mut iter);
    }

    *out_idx = 0;
    std::ptr::null()
}

/// Get the last entry of a specific type.
///
/// Returns the entry and sets `out_idx` to its 1-based index.
/// Returns NULL if no entry of that type is found.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `out_idx` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_last_of_type(
    qfl: QfListHandle,
    type_char: c_char,
    out_idx: *mut c_int,
) -> QfLineHandle {
    if qfl.is_null() || out_idx.is_null() {
        return std::ptr::null();
    }

    let mut iter = rs_qf_iter_init(qfl);
    let mut last_qfp: QfLineHandle = std::ptr::null();
    let mut last_idx: c_int = 0;

    while rs_qf_iter_has_next(&iter) {
        let qfp = rs_qf_iter_current(&iter);
        if nvim_qfline_get_type(qfp) == type_char {
            last_qfp = qfp;
            last_idx = rs_qf_iter_idx(&iter);
        }
        rs_qf_iter_next(&mut iter);
    }

    *out_idx = last_idx;
    last_qfp
}

// =============================================================================
// Phase 5: Error Type Helpers
// =============================================================================

/// Common quickfix error types.
#[allow(clippy::cast_possible_wrap)]
pub mod error_types {
    use std::ffi::c_char;

    /// Error type.
    pub const QF_TYPE_ERROR: c_char = b'E' as c_char;
    /// Warning type.
    pub const QF_TYPE_WARNING: c_char = b'W' as c_char;
    /// Info type.
    pub const QF_TYPE_INFO: c_char = b'I' as c_char;
    /// Note type.
    pub const QF_TYPE_NOTE: c_char = b'N' as c_char;
    /// No type (empty).
    pub const QF_TYPE_NONE: c_char = 0;
}

/// Check if the given character is a valid quickfix error type.
///
/// Valid types are: 'E' (error), 'W' (warning), 'I' (info), 'N' (note), or 0 (none).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_qf_valid_error_type(type_char: c_char) -> bool {
    matches!(
        type_char,
        error_types::QF_TYPE_ERROR
            | error_types::QF_TYPE_WARNING
            | error_types::QF_TYPE_INFO
            | error_types::QF_TYPE_NOTE
            | error_types::QF_TYPE_NONE
    )
}

/// Parse a character to a normalized error type.
///
/// Returns the uppercase version for e/E, w/W, i/I, n/N, or 0 for invalid.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)] // ASCII character literals are safe
pub extern "C" fn rs_qf_parse_type(type_char: c_char) -> c_char {
    // Convert to uppercase ASCII
    let upper = if type_char >= b'a' as c_char && type_char <= b'z' as c_char {
        type_char - 32 // ASCII lowercase to uppercase
    } else {
        type_char
    };

    if rs_qf_valid_error_type(upper) {
        upper
    } else {
        error_types::QF_TYPE_NONE
    }
}

/// Compare two quickfix entries for sorting purposes.
///
/// Returns:
/// - negative if a should come before b
/// - positive if a should come after b
/// - 0 if they are equal
///
/// Comparison order: file number, line number, column, error number.
///
/// # Safety
///
/// - `a` and `b` must be valid pointers to `qfline_T` structs
#[no_mangle]
pub unsafe extern "C" fn rs_qf_cmp_entries(a: QfLineHandle, b: QfLineHandle) -> c_int {
    if a.is_null() || b.is_null() {
        return 0;
    }

    // Compare by file number first
    let fnum_a = nvim_qfline_get_fnum(a);
    let fnum_b = nvim_qfline_get_fnum(b);
    if fnum_a != fnum_b {
        return fnum_a - fnum_b;
    }

    // Then by line number
    let lnum_a = nvim_qfline_get_lnum(a);
    let lnum_b = nvim_qfline_get_lnum(b);
    if lnum_a != lnum_b {
        return lnum_a - lnum_b;
    }

    // Then by column
    let col_a = nvim_qfline_get_col(a);
    let col_b = nvim_qfline_get_col(b);
    if col_a != col_b {
        return col_a - col_b;
    }

    // Finally by error number
    let nr_a = nvim_qfline_get_nr(a);
    let nr_b = nvim_qfline_get_nr(b);
    nr_a - nr_b
}

/// Check if `entry` is closer to a target position than `other_entry`.
///
/// Only returns true if `entry` is definitively closer. If it's further
/// away, or there's not enough information to tell, returns false.
///
/// Comparison order:
/// 1. File number - entries in the target file are closer
/// 2. Line number - smaller distance to target line is closer
/// 3. Column number - smaller distance to target column is closer
///
/// # Safety
///
/// - `entry` and `other_entry` must be valid pointers to `qfline_T` structs
#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn rs_qf_entry_is_closer_to_target(
    entry: QfLineHandle,
    other_entry: QfLineHandle,
    target_fnum: c_int,
    target_lnum: c_int,
    target_col: c_int,
) -> bool {
    if entry.is_null() || other_entry.is_null() {
        return false;
    }

    // Without a target file, we can't know which is closer
    if target_fnum == 0 {
        return false;
    }

    // Compare entries to target file
    let entry_fnum = nvim_qfline_get_fnum(entry);
    let other_fnum = nvim_qfline_get_fnum(other_entry);
    let is_target_file = entry_fnum != 0 && entry_fnum == target_fnum;
    let other_is_target_file = other_fnum != 0 && other_fnum == target_fnum;

    if !is_target_file && other_is_target_file {
        return false;
    } else if is_target_file && !other_is_target_file {
        return true;
    }

    // Both entries point at the same file. Now compare line numbers.
    if target_lnum == 0 {
        // Without a target line number, we can't know which is closer
        return false;
    }

    let entry_lnum = nvim_qfline_get_lnum(entry);
    let other_lnum = nvim_qfline_get_lnum(other_entry);
    let line_distance = if entry_lnum != 0 {
        (entry_lnum - target_lnum).abs()
    } else {
        i32::MAX
    };
    let other_line_distance = if other_lnum != 0 {
        (other_lnum - target_lnum).abs()
    } else {
        i32::MAX
    };

    if line_distance > other_line_distance {
        return false;
    } else if line_distance < other_line_distance {
        return true;
    }

    // Both entries point at the same line number. Now compare columns.
    if target_col == 0 {
        // Without a target column, we can't know which is closer
        return false;
    }

    let entry_col = nvim_qfline_get_col(entry);
    let other_col = nvim_qfline_get_col(other_entry);
    let column_distance = if entry_col != 0 {
        (entry_col - target_col).abs()
    } else {
        c_int::MAX
    };
    let other_column_distance = if other_col != 0 {
        (other_col - target_col).abs()
    } else {
        c_int::MAX
    };

    if column_distance > other_column_distance {
        return false;
    } else if column_distance < other_column_distance {
        return true;
    }

    // Complete tie: same file, line, and column
    false
}

/// Check if a quickfix entry matches a file buffer number.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_in_file(qfp: QfLineHandle, bnr: c_int) -> bool {
    if qfp.is_null() {
        return false;
    }
    nvim_qfline_get_fnum(qfp) == bnr
}

/// Check if a quickfix entry is valid and not cleared.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_is_active(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return false;
    }
    nvim_qfline_get_valid(qfp) && !nvim_qfline_get_cleared(qfp)
}

/// Check if a quickfix entry has a specific error type.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_has_type(qfp: QfLineHandle, type_char: c_char) -> bool {
    if qfp.is_null() {
        return false;
    }
    let entry_type = nvim_qfline_get_type(qfp);
    // Normalize both to uppercase for comparison
    rs_qf_parse_type(entry_type) == rs_qf_parse_type(type_char)
}

// =============================================================================
// Phase 6: List ID and Index Functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    fn nvim_qf_get_id(qfl: QfListHandle) -> u32;
    fn nvim_qf_get_changedtick(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_title(qfl: QfListHandle) -> *const c_char;
    fn nvim_qf_get_maxcount(qi: QfInfoHandle) -> c_int;
}

/// Get the unique identifier for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_id(qfl: QfListHandle) -> u32 {
    if qfl.is_null() {
        return 0;
    }
    nvim_qf_get_id(qfl)
}

/// Get the changedtick for a quickfix list.
///
/// The changedtick is incremented each time the list is modified.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_changedtick(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }
    nvim_qf_get_changedtick(qfl)
}

/// Check if a quickfix list has a title.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_has_title(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return false;
    }
    !nvim_qf_get_title(qfl).is_null()
}

/// Get the maximum number of lists in a quickfix stack.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_maxcount(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return 0;
    }
    nvim_qf_get_maxcount(qi)
}

/// Check if the quickfix stack is at maximum capacity.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_stack_at_max(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return false;
    }
    let listcount = nvim_qf_get_listcount(qi);
    let maxcount = nvim_qf_get_maxcount(qi);
    listcount >= maxcount
}

/// Find a quickfix list by its unique ID.
///
/// Returns the 0-based index of the list with the given ID, or -1 if not found.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_list_by_id(qi: QfInfoHandle, id: u32) -> c_int {
    if qi.is_null() || id == 0 {
        return -1;
    }

    let listcount = nvim_qf_get_listcount(qi);
    for i in 0..listcount {
        let qfl = nvim_qf_get_list_at(qi, i);
        if !qfl.is_null() && nvim_qf_get_id(qfl) == id {
            return i;
        }
    }
    -1
}

/// Check if a quickfix list with the given ID exists in the stack.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_id_exists(qi: QfInfoHandle, id: u32) -> bool {
    rs_qf_find_list_by_id(qi, id) >= 0
}

// =============================================================================
// Entry Range Functions
// =============================================================================

/// Check if an entry is in the specified line range.
///
/// Returns true if the entry's line number is between `start_lnum` and `end_lnum` inclusive.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_in_range(
    qfp: QfLineHandle,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
) -> bool {
    if qfp.is_null() {
        return false;
    }
    let lnum = nvim_qfline_get_lnum(qfp);
    lnum >= start_lnum && lnum <= end_lnum
}

/// Check if an entry covers a specific line (for entries with `end_lnum`).
///
/// Returns true if the line number is within the entry's range.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_covers_line(qfp: QfLineHandle, lnum: LinenrT) -> bool {
    if qfp.is_null() {
        return false;
    }
    let start = nvim_qfline_get_lnum(qfp);
    let end = nvim_qfline_get_end_lnum(qfp);
    if end == 0 {
        // No range, just check start line
        lnum == start
    } else {
        lnum >= start && lnum <= end
    }
}

/// Check if an entry covers a specific position (line and column).
///
/// Returns true if the position is within the entry's range.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_covers_pos(
    qfp: QfLineHandle,
    lnum: LinenrT,
    col: c_int,
) -> bool {
    if qfp.is_null() {
        return false;
    }
    let start_lnum = nvim_qfline_get_lnum(qfp);
    let end_lnum = nvim_qfline_get_end_lnum(qfp);
    let start_col = nvim_qfline_get_col(qfp);
    let end_col = nvim_qfline_get_end_col(qfp);

    // Check line range
    let effective_end_lnum = if end_lnum == 0 { start_lnum } else { end_lnum };
    if lnum < start_lnum || lnum > effective_end_lnum {
        return false;
    }

    // If on start line, must be at or after start column
    if lnum == start_lnum && col < start_col {
        return false;
    }

    // If on end line and has end column, must be at or before end column
    if lnum == effective_end_lnum && end_col > 0 && col > end_col {
        return false;
    }

    true
}

/// Count entries in a quickfix list matching a specific type.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_entries_of_type(
    qfl: QfListHandle,
    type_char: c_char,
) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_start(qfl);
    let mut type_count = 0;
    let mut idx = 0;

    while !qfp.is_null() && idx < count {
        if rs_qf_entry_has_type(qfp, type_char) {
            type_count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    type_count
}

/// Count error entries (type 'E') in a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_errors(qfl: QfListHandle) -> c_int {
    rs_qf_count_entries_of_type(qfl, error_types::QF_TYPE_ERROR)
}

/// Count warning entries (type 'W') in a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_warnings(qfl: QfListHandle) -> c_int {
    rs_qf_count_entries_of_type(qfl, error_types::QF_TYPE_WARNING)
}

// =============================================================================
// Entry Iteration Functions
// =============================================================================

/// Skip to the next valid entry in the quickfix list.
///
/// Returns the next valid entry starting from `qfp` (exclusive).
/// Updates `out_idx` with the 1-based index.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `out_idx` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_skip_to_valid(
    qfp: QfLineHandle,
    idx: c_int,
    out_idx: *mut c_int,
) -> QfLineHandle {
    if qfp.is_null() || out_idx.is_null() {
        return std::ptr::null();
    }

    let mut current = nvim_qfline_get_next(qfp);
    let mut current_idx = idx + 1;

    while !current.is_null() {
        if nvim_qfline_get_valid(current) {
            *out_idx = current_idx;
            return current;
        }
        current = nvim_qfline_get_next(current);
        current_idx += 1;
    }

    *out_idx = 0;
    std::ptr::null()
}

/// Skip to the next entry in a different file.
///
/// Returns the first entry in a file different from `qfp`'s file.
/// Updates `out_idx` with the 1-based index.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `out_idx` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_skip_to_file(
    qfp: QfLineHandle,
    idx: c_int,
    out_idx: *mut c_int,
) -> QfLineHandle {
    if qfp.is_null() || out_idx.is_null() {
        return std::ptr::null();
    }

    let current_fnum = nvim_qfline_get_fnum(qfp);
    let mut current = nvim_qfline_get_next(qfp);
    let mut current_idx = idx + 1;

    while !current.is_null() {
        if nvim_qfline_get_fnum(current) != current_fnum {
            *out_idx = current_idx;
            return current;
        }
        current = nvim_qfline_get_next(current);
        current_idx += 1;
    }

    *out_idx = 0;
    std::ptr::null()
}

/// Find the next valid entry in the same file.
///
/// Returns the next valid entry with the same file number.
/// Updates `out_idx` with the 1-based index.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `out_idx` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_next_valid_in_file(
    qfp: QfLineHandle,
    idx: c_int,
    out_idx: *mut c_int,
) -> QfLineHandle {
    if qfp.is_null() || out_idx.is_null() {
        return std::ptr::null();
    }

    let current_fnum = nvim_qfline_get_fnum(qfp);
    let mut current = nvim_qfline_get_next(qfp);
    let mut current_idx = idx + 1;

    while !current.is_null() {
        let fnum = nvim_qfline_get_fnum(current);
        if fnum != current_fnum {
            // Moved to a different file
            break;
        }
        if nvim_qfline_get_valid(current) {
            *out_idx = current_idx;
            return current;
        }
        current = nvim_qfline_get_next(current);
        current_idx += 1;
    }

    *out_idx = 0;
    std::ptr::null()
}

/// Find the previous valid entry in the same file.
///
/// Returns the previous valid entry with the same file number.
/// Updates `out_idx` with the 1-based index.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `out_idx` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_prev_valid_in_file(
    qfp: QfLineHandle,
    idx: c_int,
    out_idx: *mut c_int,
) -> QfLineHandle {
    if qfp.is_null() || out_idx.is_null() {
        return std::ptr::null();
    }

    let current_fnum = nvim_qfline_get_fnum(qfp);
    let mut current = nvim_qfline_get_prev(qfp);
    let mut current_idx = idx - 1;

    while !current.is_null() && current_idx > 0 {
        let fnum = nvim_qfline_get_fnum(current);
        if fnum != current_fnum {
            // Moved to a different file
            break;
        }
        if nvim_qfline_get_valid(current) {
            *out_idx = current_idx;
            return current;
        }
        current = nvim_qfline_get_prev(current);
        current_idx -= 1;
    }

    *out_idx = 0;
    std::ptr::null()
}

// =============================================================================
// Phase 5: List Management (expanded)
// =============================================================================

/// Mutable handle types for write operations
type QfInfoHandleMut = *mut c_void;
type QfListHandleMut = *mut c_void;
type QfLineHandleMut = *mut c_void;

#[allow(dead_code)]
extern "C" {
    // Phase 5: Setters for list management
    fn nvim_qf_set_curlist_idx(qi: QfInfoHandleMut, idx: c_int);
    fn nvim_qf_set_listcount(qi: QfInfoHandleMut, count: c_int);
    fn nvim_qf_set_index(qfl: QfListHandleMut, idx: c_int);
    fn nvim_qf_set_ptr(qfl: QfListHandleMut, ptr: QfLineHandle);

    // Phase 5: Additional getters
    fn nvim_qf_get_qfl_type(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_qi_type(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_get_last(qfl: QfListHandle) -> QfLineHandle;
    fn nvim_qfline_get_fname(qfp: QfLineHandle) -> *const c_char;
    fn nvim_qf_get_has_user_data(qfl: QfListHandle) -> bool;

    // Phase 5: List lifecycle wrappers (call C functions)
    // nvim_qf_store_title replaced by nvim_qf_set_title_dup (Phase 14)
    fn nvim_qf_set_title_dup(qfl: QfListHandleMut, title: *const c_char);
    fn nvim_get_ql_info() -> QfInfoHandleMut;
    fn nvim_qf_increment_listcount(qi: QfInfoHandleMut);
    fn nvim_qf_decrement_listcount(qi: QfInfoHandleMut);
    fn nvim_qf_set_start(qfl: QfListHandleMut, start: QfLineHandle);
    fn nvim_qf_set_last(qfl: QfListHandleMut, last: QfLineHandle);
    fn nvim_qf_set_count(qfl: QfListHandleMut, count: c_int);
    fn nvim_qf_increment_count(qfl: QfListHandleMut);
    fn nvim_qf_set_nonevalid(qfl: QfListHandleMut, nonevalid: bool);
    fn nvim_qfline_set_next(qfp: QfLineHandleMut, next: QfLineHandle);
    fn nvim_qfline_set_prev(qfp: QfLineHandleMut, prev: QfLineHandle);

    // Phase 4: File Stack and Path Resolution accessors
    fn nvim_qf_get_dir_stack(qfl: QfListHandle) -> *mut c_void;
    fn nvim_qf_set_dir_stack(qfl: QfListHandleMut, stack: *mut c_void);
    fn nvim_qf_get_file_stack(qfl: QfListHandle) -> *mut c_void;
    fn nvim_qf_set_file_stack(qfl: QfListHandleMut, stack: *mut c_void);
    fn nvim_qf_get_directory(qfl: QfListHandle) -> *const c_char;
    fn nvim_qf_set_directory(qfl: QfListHandleMut, dir: *mut c_char);
    fn nvim_qf_get_currfile(qfl: QfListHandle) -> *const c_char;
    fn nvim_qf_set_currfile(qfl: QfListHandleMut, file: *mut c_char);
    // nvim_qf_get_fnum removed: replaced by rs_qf_get_fnum (Phase 10 Pass 10 Phase 5)

    fn nvim_qf_get_multiline(qfl: QfListHandle) -> bool;
    fn nvim_qf_set_multiline(qfl: QfListHandleMut, multiline: bool);
    fn nvim_qf_get_multiignore(qfl: QfListHandle) -> bool;
    fn nvim_qf_set_multiignore(qfl: QfListHandleMut, multiignore: bool);

    // Phase 7: Window and Display Management accessors
    fn nvim_qf_win_pos_update(qi: QfInfoHandleMut, old_qf_index: c_int) -> bool;
    // nvim_qf_update_buffer removed: migrated to Rust rs_qf_update_buffer (Phase 10 Pass 10 Phase 4)
    fn nvim_qf_get_bufnr(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_set_bufnr(qi: QfInfoHandleMut, bufnr: c_int);
    fn nvim_win_is_qf_win(win: WinHandle) -> bool;
    fn nvim_win_get_llist_ref(win: WinHandle) -> QfInfoHandle;

    // Phase 10 (Pass 10): Window iteration primitives for is_qf_win/find_win/find_buf
    fn nvim_get_firstwin() -> *mut c_void;
    fn nvim_qf_win_next(win: *const c_void) -> *mut c_void;
    fn nvim_get_first_tabpage() -> *mut c_void;
    fn nvim_tabpage_get_next(tp: *const c_void) -> *mut c_void;
    fn nvim_tabpage_get_firstwin(tp: *const c_void) -> *mut c_void;
    // nvim_qf_win_get_handle already declared above (Phase 1)
    fn nvim_buflist_findnr(nr: c_int) -> BufHandle;
    fn nvim_qf_win_buf_fnum(win: *const c_void) -> c_int;

    // Phase 10 Pass 10 Phase 2: Window positioning accessors
    fn nvim_qf_win_set_redraw_bounds(win: *mut c_void, top: LinenrT, bot: LinenrT);
    fn nvim_qf_win_goto_impl(win: *mut c_void, lnum: LinenrT);
    fn nvim_qf_set_title_var_for_list(qfl: *const c_void);
    fn nvim_qf_save_curwin() -> *mut c_void;
    fn nvim_qf_restore_curwin(saved: *mut c_void);
    fn nvim_qf_set_curwin(win: *mut c_void);
    fn nvim_qf_win_get_buf_line_count(win: *const c_void) -> LinenrT;

    // Phase 10 Pass 10 Phase 3: qf_open_new_cwindow / did_set_quickfixtextfunc accessors
    fn nvim_qf_set_cwindow_options();
    fn nvim_qf_do_ecmd_existing_buf(fnum: c_int, oldwin: *mut c_void) -> c_int;
    fn nvim_qf_do_ecmd_new_buf(oldwin: *mut c_void) -> c_int;
    fn nvim_qf_get_curtab() -> *const c_void;
    fn nvim_qf_curwin_width() -> c_int;
    fn nvim_qf_curwin_set_llist_ref_incr(qi: *mut c_void);
    fn nvim_qf_curwin_set_wfh();
    fn nvim_qf_curwin_reset_binding();
    fn nvim_qf_set_prevwin(win: *mut c_void);
    fn nvim_qf_curtab_eq(saved_tab: *const c_void) -> bool;
    fn nvim_qf_option_set_callback_func_for_qftf() -> c_int;
    // nvim_qf_buf_get_fnum already declared in navigate.rs (with *const c_void)
    fn nvim_qf_curbuf_is_quickfix() -> bool;
    fn nvim_qf_curbuf_fnum() -> c_int;
    fn nvim_qf_get_columns() -> c_int;
    fn nvim_qf_win_split(size: c_int, flags: c_int) -> c_int;
    fn nvim_qf_get_cmdmod_split() -> c_int;
    fn nvim_qf_get_e_invarg() -> *const c_char;
    fn nvim_qf_curwin_is(win: *const c_void) -> bool;

    // Phase 10 Pass 10 Phase 4: qf_update_buffer accessors
    fn nvim_buf_get_ml_line_count_void(buf: *const c_void) -> LinenrT;
    fn nvim_ml_get_buf_len(buf: *mut c_void, lnum: LinenrT) -> c_int;
    fn nvim_qf_get_region_bytecount(
        buf: *mut c_void,
        l1: LinenrT,
        l2: LinenrT,
        c1: c_int,
        c2: c_int,
    ) -> i64;
    fn nvim_qf_extmark_splice(
        buf: *mut c_void,
        r1: c_int,
        c1: c_int,
        r2: c_int,
        c2: c_int,
        bc: i64,
        nr: c_int,
        nc: c_int,
        nbc: i64,
    );
    fn nvim_qf_changed_lines(
        buf: *mut c_void,
        lnum: LinenrT,
        col: c_int,
        lnume: LinenrT,
        xtra: LinenrT,
        do_win: bool,
    );
    fn nvim_qf_buf_set_changed_false(buf: *mut c_void);
    fn nvim_qf_redraw_buf_later(buf: *mut c_void);
    fn nvim_qf_win_botline(win: *const c_void) -> LinenrT;
    fn nvim_qf_aucmd_prepbuf_alloc(buf: *mut c_void) -> *mut c_void;
    fn nvim_qf_aucmd_restbuf_free(aco: *mut c_void);
    fn nvim_qf_find_win_with_loclist(ll: *const c_void) -> *mut c_void;
    fn nvim_buf_win_get_llist(win: *const c_void) -> *mut c_void;
    fn nvim_qf_get_curwin() -> *mut c_void;
    // nvim_qf_win_get_handle already declared above at line 102

    // Phase 10 Pass 10 Phase 5: qf_get_fnum accessors
    fn nvim_qf_fnum_cache_check(bufname: *const c_char) -> *mut c_void;
    fn nvim_qf_fnum_cache_update(bufname: *const c_char, buf: *mut c_void);
    fn nvim_qf_buflist_new(bufname: *mut c_char) -> *mut c_void;
    fn nvim_qf_buf_fnum_from_ptr(buf: *const c_void) -> c_int;
    fn nvim_qf_buf_set_has_qf_entry(buf: *mut c_void, is_qf_list: bool);
    fn nvim_qf_vim_is_abs_name(fname: *const c_char) -> bool;
    fn nvim_qf_concat_fnames(dir: *const c_char, fname: *const c_char) -> *mut c_char;
    fn nvim_qf_is_qf_list(qfl: *const c_void) -> bool;
    fn nvim_qf_clear_fnum_cache();
    fn nvim_qf_os_path_exists(path: *const c_char) -> bool;
    fn nvim_qf_xfree_buf(ptr: *mut c_void);
    fn nvim_qf_xstrdup(s: *const c_char) -> *mut c_char;

    // Phase 8: Ex Commands and API Functions accessors
    // (nvim_qf_get_title and nvim_qf_get_changedtick already declared above)
    fn nvim_qf_is_qf_stack(qi: QfInfoHandle) -> bool;
    fn nvim_qf_is_ll_stack(qi: QfInfoHandle) -> bool;
    fn nvim_qf_get_refcount(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_incr_refcount(qi: QfInfoHandleMut);
    fn nvim_qf_set_refcount(qi: QfInfoHandleMut, v: c_int);
    fn nvim_qf_get_ctx(qfl: QfListHandle) -> *mut c_void;
    fn nvim_qf_incr_changedtick(qfl: QfListHandleMut);

    // Phase 3 Extension: qfline_T allocation and field-setting
    fn nvim_qfline_alloc() -> QfLineHandleMut;
    // nvim_qfline_free replaced by nvim_qfline_free_fields + nvim_qf_xfree_buf (Phase 14)
    fn nvim_qfline_free_fields(qfp: QfLineHandleMut);
    fn nvim_qfline_set_fnum(qfp: QfLineHandleMut, fnum: c_int);
    fn nvim_qfline_set_lnum(qfp: QfLineHandleMut, lnum: LinenrT);
    fn nvim_qfline_set_end_lnum(qfp: QfLineHandleMut, end_lnum: LinenrT);
    fn nvim_qfline_set_col(qfp: QfLineHandleMut, col: c_int);
    fn nvim_qfline_set_end_col(qfp: QfLineHandleMut, end_col: c_int);
    fn nvim_qfline_set_nr(qfp: QfLineHandleMut, nr: c_int);
    fn nvim_qfline_set_type(qfp: QfLineHandleMut, type_char: c_char);
    fn nvim_qfline_set_viscol(qfp: QfLineHandleMut, viscol: c_char);
    fn nvim_qfline_set_valid(qfp: QfLineHandleMut, valid: c_char);
    fn nvim_qfline_set_cleared(qfp: QfLineHandleMut, cleared: c_char);
    fn nvim_qfline_set_text(qfp: QfLineHandleMut, text: *const c_char);
    fn nvim_qfline_set_module(qfp: QfLineHandleMut, module: *const c_char);
    fn nvim_qfline_set_fname(qfp: QfLineHandleMut, fname: *const c_char);
    fn nvim_qfline_set_pattern(qfp: QfLineHandleMut, pattern: *const c_char);
    fn nvim_qfline_set_user_data(
        qfp: QfLineHandleMut,
        qfl: QfListHandleMut,
        user_data: *const c_void,
    );
    // nvim_qf_mark_buf_has_entry replaced by nvim_buflist_findnr_ptr + nvim_qf_buf_or_has_entry (Phase 14)
    fn nvim_buflist_findnr_ptr(nr: c_int) -> *mut c_void;
    fn nvim_qf_buf_or_has_entry(buf: *mut c_void, is_location_list: bool);
    // nvim_qf_get_fnum_for_entry removed: replaced by rs_qf_get_fnum (Phase 10 Pass 10 Phase 5)
    fn nvim_qf_fix_fname(fname: *const c_char, bufnum: c_int) -> *mut c_char;
    fn nvim_qf_is_printc(c: c_int) -> bool;

    // Phase 1: Core List Lifecycle accessors
    fn nvim_qf_set_id(qfl: QfListHandleMut, id: u32);
    fn nvim_qf_set_qfl_type(qfl: QfListHandleMut, qfl_type: c_int);
    fn nvim_qf_set_has_user_data(qfl: QfListHandleMut, has_user_data: bool);
    fn nvim_qf_get_list_at_mut(qi: QfInfoHandleMut, idx: c_int) -> QfListHandleMut;
    fn nvim_qf_alloc_next_id() -> u32;
    fn nvim_qf_clear_list_struct(qfl: QfListHandleMut);
    fn nvim_qf_free_title(qfl: QfListHandleMut);
    fn nvim_qf_free_ctx(qfl: QfListHandleMut);
    fn nvim_qf_free_callback(qfl: QfListHandleMut);
    fn nvim_qf_set_changedtick(qfl: QfListHandleMut, changedtick: c_int);
    fn nvim_qf_shift_lists_down(qi: QfInfoHandleMut);
}

/// Opaque handle to buffer (Phase 7)
type BufHandle = *mut c_void;

/// `QFL_TYPE` values from C code
#[allow(dead_code)]
pub mod qfl_types {
    use std::ffi::c_int;

    /// Quickfix list type
    pub const QFLT_QUICKFIX: c_int = 0;
    /// Location list type
    pub const QFLT_LOCATION: c_int = 1;
    /// Internal type
    pub const QFLT_INTERNAL: c_int = 2;
}

/// Get the last entry in a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_last(qfl: QfListHandle) -> QfLineHandle {
    if qfl.is_null() {
        return std::ptr::null();
    }
    nvim_qf_get_last(qfl)
}

/// Get the file name from a quickfix entry.
///
/// Returns NULL if the entry has no file name or if qfp is null.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qfline_get_fname(qfp: QfLineHandle) -> *const c_char {
    if qfp.is_null() {
        return std::ptr::null();
    }
    nvim_qfline_get_fname(qfp)
}

/// Get the quickfix list type.
///
/// Returns `QFLT_QUICKFIX`, `QFLT_LOCATION`, or `QFLT_INTERNAL`.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_qfl_type(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return qfl_types::QFLT_QUICKFIX;
    }
    nvim_qf_get_qfl_type(qfl)
}

/// Check if a list is a quickfix list (not location list).
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_is_qf_list(qfl: QfListHandle) -> bool {
    rs_qf_get_qfl_type(qfl) == qfl_types::QFLT_QUICKFIX
}

/// Check if a list is a location list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_is_ll_list(qfl: QfListHandle) -> bool {
    rs_qf_get_qfl_type(qfl) == qfl_types::QFLT_LOCATION
}

/// Check if a quickfix list has user data.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_has_user_data(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return false;
    }
    nvim_qf_get_has_user_data(qfl)
}

/// Get the global quickfix stack.
///
/// Returns the pointer to the global `ql_info` quickfix stack.
///
/// # Safety
///
/// The returned pointer is only valid while Neovim is running and
/// the quickfix system is initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_get_ql_info() -> QfInfoHandleMut {
    nvim_get_ql_info()
}

/// Create a new quickfix list in the stack (implementation in Rust).
///
/// This creates a new list, potentially removing older lists if the stack is full.
/// The new list becomes the current list.
///
/// Algorithm:
/// 1. If current list is not the last, delete all lists after current
/// 2. If stack is full, remove the oldest list
/// 3. Create new list at the next position
/// 4. Initialize the new list with title, type, and unique ID
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
/// - `title` may be NULL
#[no_mangle]
pub unsafe extern "C" fn rs_qf_new_list(qi: QfInfoHandleMut, title: *const c_char) {
    if qi.is_null() {
        return;
    }

    let list_count = nvim_qf_get_listcount(qi);
    let cur_list = nvim_qf_get_curlist_idx(qi);
    let max_count = nvim_qf_get_maxcount(qi);

    // If the current entry is not the last entry, delete entries beyond
    // the current entry. This makes it possible to browse in a tree-like
    // way with ":grep".
    let mut count = list_count;
    while count > cur_list + 1 {
        count -= 1;
        let qfl = nvim_qf_get_list_at_mut(qi, count);
        if !qfl.is_null() {
            rs_qf_free_list(qfl);
        }
        nvim_qf_set_listcount(qi, count);
    }

    // When the stack is full, remove the oldest entry
    // Otherwise, add a new entry.
    let new_cur_list;
    if count == max_count {
        rs_qf_pop_stack(qi, false);
        new_cur_list = count - 1; // point to new empty list
    } else {
        new_cur_list = count;
        nvim_qf_set_listcount(qi, count + 1);
    }

    nvim_qf_set_curlist_idx(qi, new_cur_list);

    // Get the new list and initialize it
    let qfl = nvim_qf_get_list_at_mut(qi, new_cur_list);
    if qfl.is_null() {
        return;
    }

    // Clear the list structure
    nvim_qf_clear_list_struct(qfl);

    // Set the title
    nvim_qf_set_title_dup(qfl, title);

    // Set list type from stack type
    let qi_type = nvim_qf_get_qi_type(qi);
    nvim_qf_set_qfl_type(qfl, qi_type);

    // Allocate unique ID
    let new_id = nvim_qf_alloc_next_id();
    nvim_qf_set_id(qfl, new_id);

    // Initialize has_user_data to false
    nvim_qf_set_has_user_data(qfl, false);
}

/// Free all resources of a quickfix list (implementation in Rust).
///
/// This frees all entries, the title, context, and other resources.
///
/// Algorithm:
/// 1. Free all items (via `rs_qf_free_items`)
/// 2. Free title
/// 3. Free context typval
/// 4. Free callback
/// 5. Reset id and changedtick to 0
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - After calling this, the list structure should not be used
#[no_mangle]
pub unsafe extern "C" fn rs_qf_free_list(qfl: QfListHandleMut) {
    if qfl.is_null() {
        return;
    }

    // Free all entries
    rs_qf_free_items(qfl);

    // Free title
    nvim_qf_free_title(qfl);

    // Free context typval
    nvim_qf_free_ctx(qfl);

    // Free callback
    nvim_qf_free_callback(qfl);

    // Reset id and changedtick
    nvim_qf_set_id(qfl, 0);
    nvim_qf_set_changedtick(qfl, 0);
}

/// Free only the entries in a quickfix list (implementation in Rust).
///
/// This frees all entry items but preserves the list structure.
/// Used when repopulating a list with new content.
///
/// Algorithm:
/// 1. Walk through linked list freeing each entry
/// 2. Reset list pointers (start, last, ptr) to null
/// 3. Reset index and count to 0
/// 4. Set nonevalid to true
/// 5. Clean directory stacks
/// 6. Reset multiline flags
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_free_items(qfl: QfListHandleMut) {
    if qfl.is_null() {
        return;
    }

    // Get the start of the linked list
    let mut qfp = nvim_qf_get_start(qfl);
    let mut count = nvim_qf_get_count(qfl);

    // Walk through the list and free each entry
    while count > 0 && !qfp.is_null() {
        let next = nvim_qfline_get_next(qfp);

        // Check for circular reference (safety check)
        let is_circular = qfp == next.cast();
        if is_circular {
            // Force count to 1 to break the loop
            count = 1;
        }

        // Free the entry: fields first, then the struct itself
        nvim_qfline_free_fields(qfp.cast_mut());
        nvim_qf_xfree_buf(qfp.cast_mut());

        if !is_circular {
            qfp = next;
        }
        count -= 1;
    }

    // Reset list pointers
    nvim_qf_set_start(qfl, std::ptr::null());
    nvim_qf_set_last(qfl, std::ptr::null());
    nvim_qf_set_ptr(qfl, std::ptr::null());
    nvim_qf_set_index(qfl, 0);
    nvim_qf_set_count(qfl, 0);
    nvim_qf_set_nonevalid(qfl, true);

    // Clean directory stacks using Rust implementation
    let dir_stack = nvim_qf_get_dir_stack(qfl);
    let mut stack = dir_stack.cast::<dirstack::DirStackNode>();
    dirstack::clean_dir_stack_raw(&raw mut stack);
    nvim_qf_set_dir_stack(qfl, stack.cast::<c_void>());
    nvim_qf_set_directory(qfl, std::ptr::null_mut());

    let file_stack = nvim_qf_get_file_stack(qfl);
    let mut stack = file_stack.cast::<dirstack::DirStackNode>();
    dirstack::clean_dir_stack_raw(&raw mut stack);
    nvim_qf_set_file_stack(qfl, stack.cast::<c_void>());
    nvim_qf_set_currfile(qfl, std::ptr::null_mut());

    // Reset multiline flags
    nvim_qf_set_multiline(qfl, false);
    nvim_qf_set_multiignore(qfl, false);
    nvim_qf_set_multiscan(qfl, false);
}

/// Set the title of a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `title` may be NULL to clear the title
#[no_mangle]
pub unsafe extern "C" fn rs_qf_store_title(qfl: QfListHandleMut, title: *const c_char) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_title_dup(qfl, title);
}

/// Pop the oldest list from the quickfix stack (implementation in Rust).
///
/// This removes the first (oldest) list from the stack, shifting all
/// remaining lists down. If `adjust` is true, also decrements listcount
/// and adjusts curlist.
///
/// Algorithm:
/// 1. Free the first list (index 0)
/// 2. Shift all lists down by one position
/// 3. Zero the now-unused top list
/// 4. If adjust: decrement listcount and adjust curlist
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_pop_stack(qi: QfInfoHandleMut, adjust: bool) {
    if qi.is_null() {
        return;
    }

    // Free the first list
    let first_list = nvim_qf_get_list_at_mut(qi, 0);
    if !first_list.is_null() {
        rs_qf_free_list(first_list);
    }

    // Shift all lists down
    nvim_qf_shift_lists_down(qi);

    // Zero the now-unused top list
    stack::rs_qf_zero_top_list(stack::QfStackHandle(qi));

    // Adjust listcount and curlist if requested
    if adjust {
        stack::rs_qf_decr_listcount(stack::QfStackHandle(qi));
        stack::rs_qf_decr_curlist(stack::QfStackHandle(qi));
    }
}

/// Set the start (first entry) pointer for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_start(qfl: QfListHandleMut, start: QfLineHandle) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_start(qfl, start);
}

/// Set the last entry pointer for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_last(qfl: QfListHandleMut, last: QfLineHandle) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_last(qfl, last);
}

/// Set the entry count for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_count(qfl: QfListHandleMut, count: c_int) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_count(qfl, count);
}

/// Increment the entry count for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_increment_count(qfl: QfListHandleMut) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_increment_count(qfl);
}

/// Set the nonevalid flag for a quickfix list.
///
/// When true, all entries are considered valid even if `qf_valid` is false.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_nonevalid(qfl: QfListHandleMut, nonevalid: bool) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_nonevalid(qfl, nonevalid);
}

/// Set the next pointer for a quickfix entry.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qfline_set_next(qfp: QfLineHandleMut, next: QfLineHandle) {
    if qfp.is_null() {
        return;
    }
    nvim_qfline_set_next(qfp, next);
}

/// Set the prev pointer for a quickfix entry.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qfline_set_prev(qfp: QfLineHandleMut, prev: QfLineHandle) {
    if qfp.is_null() {
        return;
    }
    nvim_qfline_set_prev(qfp, prev);
}

// =============================================================================
// Phase 3 Extension: Entry Creation (qf_add_entry migration)
// =============================================================================

extern "C" {
    fn xfree(ptr: *mut c_void);
}

/// `QF_OK` constant for success
pub const QF_OK: c_int = 1;
/// `QF_FAIL` constant for failure
pub const QF_FAIL: c_int = 0;

/// Add a new entry to a quickfix list.
///
/// This is the Rust implementation of `qf_add_entry()`. It creates a new
/// quickfix entry with the specified properties and links it into the list.
///
/// # Arguments
///
/// * `qfl` - Handle to the quickfix list
/// * `dir` - Directory for filename resolution (may be NULL)
/// * `fname` - Filename (may be NULL if bufnum is provided)
/// * `module` - Module name (may be NULL)
/// * `bufnum` - Buffer number (0 to resolve from fname)
/// * `mesg` - Error message text
/// * `lnum` - Line number
/// * `end_lnum` - End line number (0 if not a range)
/// * `col` - Column number
/// * `end_col` - End column number (0 if not a range)
/// * `vis_col` - True if col/end\_col are screen columns
/// * `pattern` - Search pattern (may be NULL)
/// * `nr` - Error number
/// * `type_char` - Error type character ('E', 'W', etc.)
/// * `user_data` - Custom user data typval pointer (may be NULL)
/// * `valid` - Whether this is a valid entry
///
/// # Returns
///
/// `QF_OK` on success, `QF_FAIL` on failure.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - String parameters must be valid C strings or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_qf_add_entry(
    qfl: QfListHandleMut,
    dir: *mut c_char,
    fname: *const c_char,
    module: *const c_char,
    bufnum: c_int,
    mesg: *const c_char,
    lnum: LinenrT,
    end_lnum: LinenrT,
    col: c_int,
    end_col: c_int,
    vis_col: c_char,
    pattern: *const c_char,
    nr: c_int,
    type_char: c_char,
    user_data: *const c_void,
    valid: c_char,
) -> c_int {
    if qfl.is_null() {
        return QF_FAIL;
    }

    // Allocate a new quickfix entry
    let qfp = nvim_qfline_alloc();
    if qfp.is_null() {
        return QF_FAIL;
    }

    // Determine buffer number
    let fnum: c_int;
    if bufnum != 0 {
        fnum = bufnum;
        nvim_qfline_set_fnum(qfp, fnum);
        // Mark the buffer as having a quickfix/location list entry
        let is_location_list = rs_qf_get_qfl_type(qfl) == qfl_types::QFLT_LOCATION;
        let buf = nvim_buflist_findnr_ptr(fnum);
        if !buf.is_null() {
            nvim_qf_buf_or_has_entry(buf, is_location_list);
        }
    } else {
        fnum = rs_qf_get_fnum(qfl, dir, fname.cast_mut());
        nvim_qfline_set_fnum(qfp, fnum);
    }

    // Set the filename if it differs from buffer name
    if !fname.is_null() {
        let fixed_fname = nvim_qf_fix_fname(fname, fnum);
        if !fixed_fname.is_null() {
            nvim_qfline_set_fname(qfp, fixed_fname);
            // The C function duplicates the string, but nvim_qf_fix_fname
            // returns an allocated string that we need to handle properly.
            // Since set_fname duplicates it, we need to free the original.
            xfree(fixed_fname.cast());
        }
    }

    // Set the error message
    nvim_qfline_set_text(qfp, mesg);

    // Set position fields
    nvim_qfline_set_lnum(qfp, lnum);
    nvim_qfline_set_end_lnum(qfp, end_lnum);
    nvim_qfline_set_col(qfp, col);
    nvim_qfline_set_end_col(qfp, end_col);
    nvim_qfline_set_viscol(qfp, vis_col);

    // Set module and pattern
    nvim_qfline_set_module(qfp, module);
    nvim_qfline_set_pattern(qfp, pattern);

    // Set user data (copies the typval if provided)
    nvim_qfline_set_user_data(qfp, qfl, user_data);

    // Set error number
    nvim_qfline_set_nr(qfp, nr);

    // Validate and set type character
    #[allow(clippy::cast_sign_loss)]
    let final_type = if type_char != 1 && !nvim_qf_is_printc(c_int::from(type_char as u8)) {
        0
    } else {
        type_char
    };
    nvim_qfline_set_type(qfp, final_type);

    // Set valid flag
    nvim_qfline_set_valid(qfp, valid);

    // Set cleared to false
    nvim_qfline_set_cleared(qfp, 0);

    // Link the entry into the list
    let last = nvim_qf_get_last(qfl);
    if rs_qf_list_empty(qfl) {
        // First element in the list
        nvim_qf_set_start(qfl, qfp);
        nvim_qf_set_ptr(qfl, qfp);
        nvim_qf_set_index(qfl, 0);
        nvim_qfline_set_prev(qfp, std::ptr::null());
    } else {
        // Append to the end
        nvim_qfline_set_prev(qfp, last);
        nvim_qfline_set_next(last.cast_mut(), qfp);
    }
    nvim_qfline_set_next(qfp, std::ptr::null());
    nvim_qf_set_last(qfl, qfp);
    nvim_qf_increment_count(qfl);

    // If this is the first valid entry, update the current pointer
    let count = nvim_qf_get_count(qfl);
    let index = nvim_qf_get_index(qfl);
    if index == 0 && valid != 0 {
        nvim_qf_set_index(qfl, count);
        nvim_qf_set_ptr(qfl, qfp);
    }

    QF_OK
}

// =============================================================================
// Phase 4: File Stack and Path Resolution
// =============================================================================

/// Get the directory stack pointer from a quickfix list.
///
/// Returns an opaque handle to the directory stack, or null if not set.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_dir_stack(qfl: QfListHandle) -> *mut c_void {
    if qfl.is_null() {
        return std::ptr::null_mut();
    }
    nvim_qf_get_dir_stack(qfl)
}

/// Set the directory stack pointer for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `stack` must be a valid `dir_stack_T` pointer or null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_dir_stack(qfl: QfListHandleMut, stack: *mut c_void) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_dir_stack(qfl, stack);
}

/// Get the file stack pointer from a quickfix list.
///
/// Returns an opaque handle to the file stack, or null if not set.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_file_stack(qfl: QfListHandle) -> *mut c_void {
    if qfl.is_null() {
        return std::ptr::null_mut();
    }
    nvim_qf_get_file_stack(qfl)
}

/// Set the file stack pointer for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `stack` must be a valid `dir_stack_T` pointer or null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_file_stack(qfl: QfListHandleMut, stack: *mut c_void) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_file_stack(qfl, stack);
}

/// Get the current directory string from a quickfix list.
///
/// Returns the directory path used for relative path resolution, or null.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_directory(qfl: QfListHandle) -> *const c_char {
    if qfl.is_null() {
        return std::ptr::null();
    }
    nvim_qf_get_directory(qfl)
}

/// Set the current directory string for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `dir` must be a valid C string pointer or null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_directory(qfl: QfListHandleMut, dir: *mut c_char) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_directory(qfl, dir);
}

/// Get the current file string from a quickfix list.
///
/// Returns the current file path being parsed, or null.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_currfile(qfl: QfListHandle) -> *const c_char {
    if qfl.is_null() {
        return std::ptr::null();
    }
    nvim_qf_get_currfile(qfl)
}

/// Set the current file string for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `file` must be a valid C string pointer or null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_currfile(qfl: QfListHandleMut, file: *mut c_char) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_currfile(qfl, file);
}

/// Push a directory onto the directory or file stack.
///
/// Returns the actual directory name stored on the stack, or null on error.
/// The `is_file_stack` parameter selects which stack to use.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `dirbuf` must be a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_push_dir(
    qfl: QfListHandleMut,
    dirbuf: *mut c_char,
    is_file_stack: bool,
) -> *const c_char {
    if qfl.is_null() || dirbuf.is_null() {
        return std::ptr::null();
    }

    // Get the appropriate stack pointer from the qf_list_T
    let stack_ptr = if is_file_stack {
        nvim_qf_get_file_stack(qfl)
    } else {
        nvim_qf_get_dir_stack(qfl)
    };

    // Cast to the Rust type and call the Rust implementation
    let mut stack = stack_ptr.cast::<dirstack::DirStackNode>();
    let result = dirstack::push_dir_raw(dirbuf, &raw mut stack, is_file_stack);

    // Update the stack pointer in qf_list_T
    if is_file_stack {
        nvim_qf_set_file_stack(qfl, stack.cast::<c_void>());
    } else {
        nvim_qf_set_dir_stack(qfl, stack.cast::<c_void>());
    }

    result
}

/// Pop a directory from the directory or file stack.
///
/// Returns the new top directory name, or null if the stack is empty.
/// The `is_file_stack` parameter selects which stack to use.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_pop_dir(qfl: QfListHandleMut, is_file_stack: bool) -> *const c_char {
    if qfl.is_null() {
        return std::ptr::null();
    }

    // Get the appropriate stack pointer from the qf_list_T
    let stack_ptr = if is_file_stack {
        nvim_qf_get_file_stack(qfl)
    } else {
        nvim_qf_get_dir_stack(qfl)
    };

    // Cast to the Rust type and call the Rust implementation
    let mut stack = stack_ptr.cast::<dirstack::DirStackNode>();
    let result = dirstack::pop_dir_raw(&raw mut stack);

    // Update the stack pointer in qf_list_T
    if is_file_stack {
        nvim_qf_set_file_stack(qfl, stack.cast::<c_void>());
    } else {
        nvim_qf_set_dir_stack(qfl, stack.cast::<c_void>());
    }

    result
}

/// Clean up a directory or file stack, freeing all entries.
///
/// The `is_file_stack` parameter selects which stack to clean.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_clean_dir_stack(qfl: QfListHandleMut, is_file_stack: bool) {
    if qfl.is_null() {
        return;
    }

    // Get the appropriate stack pointer from the qf_list_T
    let stack_ptr = if is_file_stack {
        nvim_qf_get_file_stack(qfl)
    } else {
        nvim_qf_get_dir_stack(qfl)
    };

    // Cast to the Rust type and call the Rust implementation
    let mut stack = stack_ptr.cast::<dirstack::DirStackNode>();
    dirstack::clean_dir_stack_raw(&raw mut stack);

    // Update the stack pointer in qf_list_T (should now be NULL)
    if is_file_stack {
        nvim_qf_set_file_stack(qfl, stack.cast::<c_void>());
    } else {
        nvim_qf_set_dir_stack(qfl, stack.cast::<c_void>());
    }
}

/// Guess the filepath by searching the directory stack.
///
/// Searches the directory stack for a directory containing the given file.
/// Returns the directory where the file was found, or null if not found.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `filename` must be a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_guess_filepath(
    qfl: QfListHandleMut,
    filename: *mut c_char,
) -> *const c_char {
    if qfl.is_null() || filename.is_null() {
        return std::ptr::null();
    }

    // Get the directory stack from the qf_list_T
    let stack_ptr = nvim_qf_get_dir_stack(qfl);

    // Cast to the Rust type and call the Rust implementation
    let stack = stack_ptr.cast::<dirstack::DirStackNode>();
    dirstack::guess_filepath_raw(stack, filename)
}

/// Get the buffer number for a file, creating the buffer if needed.
///
/// Resolves the file path using the given directory (or the directory stack
/// if the path is relative) and returns the buffer number. Creates the
/// buffer if it doesn't exist. Uses a single-entry cache for speed.
///
/// Returns the buffer number, or 0 if the file couldn't be found/created.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `directory` can be null or a valid C string
/// - `fname` must be a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_fnum(
    qfl: QfListHandleMut,
    directory: *mut c_char,
    fname: *mut c_char,
) -> c_int {
    if qfl.is_null() {
        return 0;
    }
    // No file name: nothing to resolve.
    if fname.is_null() || *fname == 0 {
        return 0;
    }

    // Determine the effective buffer name.
    // If directory is given and fname is not absolute, concatenate them.
    let concat_ptr: *mut c_char = if !directory.is_null() && !nvim_qf_vim_is_abs_name(fname) {
        let ptr = nvim_qf_concat_fnames(directory, fname);
        // Verify the concatenated path exists; if not, guess via dir stack.
        if nvim_qf_os_path_exists(ptr.cast_const()) {
            ptr
        } else {
            nvim_qf_xfree_buf(ptr.cast());
            let guessed = rs_qf_guess_filepath(qfl, fname);
            if guessed.is_null() {
                // No guess available; use fname as-is (xstrdup for owned allocation).
                nvim_qf_xstrdup(fname)
            } else {
                nvim_qf_concat_fnames(guessed, fname)
            }
        }
    } else {
        std::ptr::null_mut()
    };
    let bufname: *mut c_char = if concat_ptr.is_null() {
        fname
    } else {
        concat_ptr
    };

    // Check the single-entry cache.
    let cached_buf = nvim_qf_fnum_cache_check(bufname.cast_const());
    let buf: *mut c_void = if cached_buf.is_null() {
        // Not in cache: create/find the buffer in the list.
        let new_buf = nvim_qf_buflist_new(bufname);
        nvim_qf_fnum_cache_update(bufname.cast_const(), new_buf);
        // Free the concat_ptr (cache_update made a copy of bufname).
        if !concat_ptr.is_null() {
            nvim_qf_xfree_buf(concat_ptr.cast());
        }
        new_buf
    } else {
        // Cache hit: free the concat_ptr if we allocated one.
        if !concat_ptr.is_null() {
            nvim_qf_xfree_buf(concat_ptr.cast());
        }
        cached_buf
    };

    if buf.is_null() {
        return 0;
    }

    // Mark the buffer as having a quickfix or location list entry.
    let is_qf = nvim_qf_is_qf_list(qfl.cast_const());
    nvim_qf_buf_set_has_qf_entry(buf, is_qf);

    nvim_qf_buf_fnum_from_ptr(buf.cast_const())
}

/// Get the multiline flag from a quickfix list.
///
/// Returns true if currently parsing a multi-line error message.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_multiline(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return false;
    }
    nvim_qf_get_multiline(qfl)
}

/// Set the multiline flag for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_multiline(qfl: QfListHandleMut, multiline: bool) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_multiline(qfl, multiline);
}

/// Get the multiignore flag from a quickfix list.
///
/// Returns true if ignoring lines until end of multi-line message.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_multiignore(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return false;
    }
    nvim_qf_get_multiignore(qfl)
}

/// Set the multiignore flag for a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_multiignore(qfl: QfListHandleMut, multiignore: bool) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_multiignore(qfl, multiignore);
}

// =============================================================================
// Phase 7: Window and Display Management
// =============================================================================

/// Check if a window is displaying the quickfix/location list for a given stack.
///
/// Equivalent to C `is_qf_win`: checks `bt_quickfix(w_buffer)` and then
/// compares `w_llist_ref` to the stack pointer.
///
/// A quickfix window has `w_llist_ref == NULL`.
/// A location list window has `w_llist_ref == qi`.
///
/// # Safety
///
/// - `win` must be a valid non-null pointer to a `win_T` struct
/// - `qi` must be a valid non-null pointer to a `qf_info_T` struct
#[allow(clippy::cast_ptr_alignment)]
unsafe fn is_qf_win_for_stack(win: *const c_void, qi: QfInfoHandle) -> bool {
    if !nvim_win_is_qf_win(win) {
        return false;
    }
    let llist_ref = nvim_win_get_llist_ref(win);
    if nvim_qf_is_qf_stack(qi) {
        llist_ref.is_null()
    } else {
        // location list: llist_ref must point to qi
        std::ptr::eq(llist_ref, qi)
    }
}

/// Find the quickfix window in the current tab for a given quickfix stack.
///
/// Equivalent to C `qf_find_win`. Iterates `FOR_ALL_WINDOWS_IN_TAB(win, curtab)`.
///
/// Returns the window handle, or null if no window in the current tab
/// is displaying this stack.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_win_for_stack(qi: QfInfoHandle) -> WinHandle {
    if qi.is_null() {
        return std::ptr::null();
    }
    // Iterate windows in current tab (FOR_ALL_WINDOWS_IN_TAB(win, curtab))
    let mut win = nvim_get_firstwin();
    while !win.is_null() {
        if is_qf_win_for_stack(win, qi) {
            return win.cast_const();
        }
        win = nvim_qf_win_next(win);
    }
    std::ptr::null()
}

/// Find the quickfix buffer for a given quickfix stack, searching all tab pages.
///
/// Equivalent to C `qf_find_buf`.
///
/// First checks the cached `qf_bufnr`. If valid, returns that buffer directly.
/// Otherwise iterates `FOR_ALL_TAB_WINDOWS` searching for a window displaying
/// this stack.
///
/// Returns the buffer handle, or null if no buffer exists for this stack.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_buf_for_stack(qi: QfInfoHandleMut) -> BufHandle {
    if qi.is_null() {
        return std::ptr::null_mut();
    }
    // Check cached bufnr first (INVALID_QFBUFNR == 0)
    let bufnr = nvim_qf_get_bufnr(qi);
    if bufnr != 0 {
        let qfbuf = nvim_buflist_findnr(bufnr);
        if !qfbuf.is_null() {
            return qfbuf;
        }
        // Buffer no longer present; clear the cache
        nvim_qf_set_bufnr(qi, 0);
    }
    // Search all tab windows (FOR_ALL_TAB_WINDOWS)
    let mut tp = nvim_get_first_tabpage();
    while !tp.is_null() {
        let mut win = nvim_tabpage_get_firstwin(tp);
        while !win.is_null() {
            if is_qf_win_for_stack(win, qi.cast_const()) {
                // Return the window's buffer via fnum lookup
                let fnum = nvim_qf_win_buf_fnum(win);
                if fnum > 0 {
                    let buf = nvim_buflist_findnr(fnum);
                    if !buf.is_null() {
                        return buf;
                    }
                }
            }
            win = nvim_qf_win_next(win);
        }
        tp = nvim_tabpage_get_next(tp);
    }
    std::ptr::null_mut()
}

/// Update the cursor position in the quickfix window.
///
/// Equivalent to C `qf_win_pos_update`. Finds the quickfix window and moves
/// the cursor to the current error entry. Updates redraw bounds.
/// Returns true if there is a quickfix window.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_win_pos_update(qi: QfInfoHandleMut, old_qf_index: c_int) -> bool {
    rs_qf_win_pos_update_impl(qi, old_qf_index)
}

// rs_qf_update_buffer: real implementation in Phase 10 Pass 10 Phase 4 below.

/// Get the buffer number from a quickfix info struct.
///
/// Returns the buffer number, or -1 (`INVALID_QFBUFNR`) if not set.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_bufnr(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return -1;
    }
    nvim_qf_get_bufnr(qi)
}

/// Set the buffer number in a quickfix info struct.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_bufnr(qi: QfInfoHandleMut, bufnr: c_int) {
    if qi.is_null() {
        return;
    }
    nvim_qf_set_bufnr(qi, bufnr);
}

/// Check if a window is a quickfix window.
///
/// Returns true if the window is displaying a quickfix or location list buffer.
///
/// # Safety
///
/// - `win` must be a valid pointer to a window struct
#[no_mangle]
pub unsafe extern "C" fn rs_win_is_qf_win(win: WinHandle) -> bool {
    if win.is_null() {
        return false;
    }
    nvim_win_is_qf_win(win)
}

/// Get the location list reference from a window.
///
/// Returns the location list handle, or null if the window is displaying
/// the global quickfix list (not a location list).
///
/// # Safety
///
/// - `win` must be a valid pointer to a window struct
#[no_mangle]
pub unsafe extern "C" fn rs_win_get_llist_ref(win: WinHandle) -> QfInfoHandle {
    if win.is_null() {
        return std::ptr::null();
    }
    nvim_win_get_llist_ref(win)
}

// =============================================================================
// Phase 10 Pass 10 Phase 2: Window Positioning and Title Management
// =============================================================================

/// Set the `w:quickfix_title` window variable for the current window.
///
/// Equivalent to C `qf_set_title_var`. Called with the current window
/// already set to the target quickfix window.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - Must be called with `curwin` pointing to the target quickfix window
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_title_var(qfl: QfListHandle) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_title_var_for_list(qfl);
}

/// Update `w:quickfix_title` for all windows displaying the given quickfix stack.
///
/// Equivalent to C `qf_update_win_titlevar`. Iterates all tab windows,
/// temporarily sets curwin to each matching window, and sets the title var.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_update_win_titlevar(qi: QfInfoHandleMut) {
    if qi.is_null() {
        return;
    }
    let qfl = nvim_qf_get_curlist(qi);
    if qfl.is_null() {
        return;
    }
    let saved_curwin = nvim_qf_save_curwin();

    // FOR_ALL_TAB_WINDOWS(tp, win)
    let mut tp = nvim_get_first_tabpage();
    while !tp.is_null() {
        let mut win = nvim_tabpage_get_firstwin(tp);
        while !win.is_null() {
            if is_qf_win_for_stack(win, qi.cast_const()) {
                nvim_qf_set_curwin(win);
                nvim_qf_set_title_var_for_list(qfl);
            }
            win = nvim_qf_win_next(win);
        }
        tp = nvim_tabpage_get_next(tp);
    }
    nvim_qf_restore_curwin(saved_curwin);
}

/// Update the cursor position in the quickfix window.
///
/// Equivalent to C `qf_win_pos_update`. Moves the cursor in the quickfix
/// window to the current error entry line.
///
/// Returns true if there is a quickfix window.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_win_pos_update_impl(
    qi: QfInfoHandleMut,
    old_qf_index: c_int,
) -> bool {
    if qi.is_null() {
        return false;
    }
    let qfl = nvim_qf_get_curlist(qi);
    if qfl.is_null() {
        return false;
    }
    let qf_index = nvim_qf_get_index(qfl);
    let win = rs_qf_find_win_for_stack(qi.cast_const()).cast_mut();
    if !win.is_null()
        && qf_index <= nvim_qf_win_get_buf_line_count(win)
        && window::rs_qf_should_update_cursor(qfl, old_qf_index)
    {
        let top = if old_qf_index < qf_index {
            old_qf_index
        } else {
            qf_index
        };
        let bot = if old_qf_index > qf_index {
            old_qf_index
        } else {
            qf_index
        };
        nvim_qf_win_set_redraw_bounds(win, top, bot);
        nvim_qf_win_goto_impl(win, qf_index);
    }
    !win.is_null()
}

// =============================================================================
// Phase 8: Ex Commands and API Functions
// =============================================================================

/// Get the title of a quickfix list.
///
/// Returns the title string, or null if not set.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_title(qfl: QfListHandle) -> *const c_char {
    if qfl.is_null() {
        return std::ptr::null();
    }
    nvim_qf_get_title(qfl)
}

/// Check if a quickfix stack is the global quickfix list.
///
/// Returns true if this is the global quickfix list, false if it's a location list.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_is_qf_stack(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return false;
    }
    nvim_qf_is_qf_stack(qi)
}

/// Check if a quickfix stack is a location list.
///
/// Returns true if this is a location list, false if it's the global quickfix list.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_is_ll_stack(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return false;
    }
    nvim_qf_is_ll_stack(qi)
}

/// Get the reference count of a quickfix info struct.
///
/// Used for location lists to track window references.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_refcount(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return 0;
    }
    nvim_qf_get_refcount(qi)
}

/// Increment the reference count of a quickfix info struct.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_incr_refcount(qi: QfInfoHandleMut) {
    if qi.is_null() {
        return;
    }
    nvim_qf_incr_refcount(qi);
}

/// Decrement the reference count of a quickfix info struct.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_decr_refcount(qi: QfInfoHandleMut) {
    if qi.is_null() {
        return;
    }
    let refcount = nvim_qf_get_refcount(qi.cast_const());
    if refcount > 0 {
        nvim_qf_set_refcount(qi, refcount - 1);
    }
}

/// Get the context of a quickfix list.
///
/// Returns the context typval pointer, or null if not set.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_ctx(qfl: QfListHandle) -> *mut c_void {
    if qfl.is_null() {
        return std::ptr::null_mut();
    }
    nvim_qf_get_ctx(qfl)
}

/// Increment the changedtick of a quickfix list.
///
/// Should be called whenever the list is modified.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_incr_changedtick(qfl: QfListHandleMut) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_incr_changedtick(qfl);
}

/// Set the current list index in a quickfix stack.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
/// - `idx` should be in range [0, listcount)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_curlist_idx(qi: QfInfoHandleMut, idx: c_int) {
    if qi.is_null() {
        return;
    }
    nvim_qf_set_curlist_idx(qi, idx);
}

/// Set the current entry index (cursor position) in a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_index(qfl: QfListHandleMut, idx: c_int) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_index(qfl, idx);
}

/// Set the current entry pointer in a quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `ptr` should be a valid entry in the list or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_ptr(qfl: QfListHandleMut, ptr: QfLineHandle) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_set_ptr(qfl, ptr);
}

/// Move to the next quickfix list in the stack.
///
/// Returns the new list index, or -1 if already at the last list.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_next_list(qi: QfInfoHandleMut) -> c_int {
    if qi.is_null() {
        return -1;
    }

    let current = nvim_qf_get_curlist_idx(qi);
    let count = nvim_qf_get_listcount(qi);

    if current + 1 >= count {
        return -1; // Already at the last list
    }

    let new_idx = current + 1;
    nvim_qf_set_curlist_idx(qi, new_idx);
    new_idx
}

/// Move to the previous quickfix list in the stack.
///
/// Returns the new list index, or -1 if already at the first list.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_prev_list(qi: QfInfoHandleMut) -> c_int {
    if qi.is_null() {
        return -1;
    }

    let current = nvim_qf_get_curlist_idx(qi);

    if current <= 0 {
        return -1; // Already at the first list
    }

    let new_idx = current - 1;
    nvim_qf_set_curlist_idx(qi, new_idx);
    new_idx
}

/// Move to a specific list in the quickfix stack.
///
/// Returns true if the index was valid and the list was selected,
/// false otherwise.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_list(qi: QfInfoHandleMut, idx: c_int) -> bool {
    if qi.is_null() {
        return false;
    }

    if !rs_qf_valid_idx(qi, idx) {
        return false;
    }

    nvim_qf_set_curlist_idx(qi, idx);
    true
}

/// Update the current entry position in the quickfix list.
///
/// This sets both the index and the pointer to the entry at that index.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `idx` should be a valid 1-based index in the list
#[no_mangle]
pub unsafe extern "C" fn rs_qf_set_cur_entry(qfl: QfListHandleMut, idx: c_int) {
    if qfl.is_null() || idx <= 0 {
        return;
    }

    let qfp = rs_qf_get_entry_at_idx(qfl, idx);
    if !qfp.is_null() {
        nvim_qf_set_index(qfl, idx);
        nvim_qf_set_ptr(qfl, qfp);
    }
}

/// Move to the next valid entry in the quickfix list.
///
/// Returns the new 1-based index, or 0 if no next valid entry exists.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_next_valid_entry(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current_ptr = nvim_qf_get_ptr(qfl);
    if current_ptr.is_null() {
        return 0;
    }

    let current_idx = nvim_qf_get_index(qfl);
    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qfline_get_next(current_ptr);
    let mut idx = current_idx + 1;

    while !qfp.is_null() && idx <= count {
        if nvim_qfline_get_valid(qfp) {
            nvim_qf_set_index(qfl, idx);
            nvim_qf_set_ptr(qfl, qfp);
            return idx;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    0 // No next valid entry
}

/// Move to the previous valid entry in the quickfix list.
///
/// Returns the new 1-based index, or 0 if no previous valid entry exists.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_prev_valid_entry(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current_ptr = nvim_qf_get_ptr(qfl);
    if current_ptr.is_null() {
        return 0;
    }

    let current_idx = nvim_qf_get_index(qfl);
    let mut qfp = nvim_qfline_get_prev(current_ptr);
    let mut idx = current_idx - 1;

    while !qfp.is_null() && idx >= 1 {
        if nvim_qfline_get_valid(qfp) {
            nvim_qf_set_index(qfl, idx);
            nvim_qf_set_ptr(qfl, qfp);
            return idx;
        }
        qfp = nvim_qfline_get_prev(qfp);
        idx -= 1;
    }

    0 // No previous valid entry
}

/// Move to the first valid entry in the quickfix list.
///
/// Returns the 1-based index of the entry, or 0 if none found.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_first_valid_entry(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() && idx <= count {
        if nvim_qfline_get_valid(qfp) {
            nvim_qf_set_index(qfl, idx);
            nvim_qf_set_ptr(qfl, qfp);
            return idx;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    0 // No valid entries
}

/// Move to the last valid entry in the quickfix list.
///
/// Returns the 1-based index of the entry, or 0 if none found.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_last_valid_entry(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_last(qfl);
    let mut idx = count;

    while !qfp.is_null() && idx >= 1 {
        if nvim_qfline_get_valid(qfp) {
            nvim_qf_set_index(qfl, idx);
            nvim_qf_set_ptr(qfl, qfp);
            return idx;
        }
        qfp = nvim_qfline_get_prev(qfp);
        idx -= 1;
    }

    0 // No valid entries
}

/// Get the next valid entry in a specific direction.
///
/// This is a lower-level function that doesn't modify the list state.
/// It returns the entry pointer and updates `out_idx` with the new index.
///
/// - `dir` == `FORWARD` or `FORWARD_FILE`: search forward
/// - `dir` == `BACKWARD` or `BACKWARD_FILE`: search backward
/// - For `FORWARD_FILE`/`BACKWARD_FILE`, skips to a different file
///
/// Returns the entry pointer, or NULL if not found.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `qf_index` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_next_valid_entry_dir(
    qfl: QfListHandle,
    qfp: QfLineHandle,
    qf_index: *mut c_int,
    dir: c_int,
) -> QfLineHandle {
    if qfl.is_null() || qfp.is_null() || qf_index.is_null() {
        return std::ptr::null();
    }

    let nonevalid = nvim_qf_get_nonevalid(qfl);
    let count = nvim_qf_get_count(qfl);
    let old_fnum = nvim_qfline_get_fnum(qfp);
    let mut idx = *qf_index;
    let mut current = nvim_qfline_get_next(qfp);

    loop {
        if idx == count || current.is_null() {
            return std::ptr::null();
        }
        idx += 1;

        let valid = nvim_qfline_get_valid(current);
        let fnum = nvim_qfline_get_fnum(current);

        // Skip invalid entries (unless all are invalid)
        // Skip same-file entries if dir is FORWARD_FILE
        if (nonevalid || valid) && (dir != direction::FORWARD_FILE || fnum != old_fnum) {
            *qf_index = idx;
            return current;
        }

        current = nvim_qfline_get_next(current);
    }
}

/// Get the previous valid entry in a specific direction.
///
/// This is a lower-level function that doesn't modify the list state.
/// It returns the entry pointer and updates `out_idx` with the new index.
///
/// - `dir` == `BACKWARD`: search backward
/// - `dir` == `BACKWARD_FILE`: search backward to a different file
///
/// Returns the entry pointer, or NULL if not found.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `qfp` must be a valid pointer to a `qfline_T` struct
/// - `qf_index` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_prev_valid_entry_dir(
    qfl: QfListHandle,
    qfp: QfLineHandle,
    qf_index: *mut c_int,
    dir: c_int,
) -> QfLineHandle {
    if qfl.is_null() || qfp.is_null() || qf_index.is_null() {
        return std::ptr::null();
    }

    let nonevalid = nvim_qf_get_nonevalid(qfl);
    let old_fnum = nvim_qfline_get_fnum(qfp);
    let mut idx = *qf_index;
    let mut current = nvim_qfline_get_prev(qfp);

    loop {
        if idx == 1 || current.is_null() {
            return std::ptr::null();
        }
        idx -= 1;

        let valid = nvim_qfline_get_valid(current);
        let fnum = nvim_qfline_get_fnum(current);

        // Skip invalid entries (unless all are invalid)
        // Skip same-file entries if dir is BACKWARD_FILE
        if (nonevalid || valid) && (dir != direction::BACKWARD_FILE || fnum != old_fnum) {
            *qf_index = idx;
            return current;
        }

        current = nvim_qfline_get_prev(current);
    }
}

/// Get the n'th valid entry in the specified direction from the current entry.
///
/// This is equivalent to the C function `get_nth_valid_entry()`.
///
/// - `errornr`: number of valid entries to skip
/// - `dir`: `FORWARD`/`BACKWARD`/`FORWARD_FILE`/`BACKWARD_FILE`
///
/// Returns the entry pointer and sets `new_qfidx` to the new index.
/// Returns NULL on failure.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `new_qfidx` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_nth_valid_entry_dir(
    qfl: QfListHandle,
    errornr: c_int,
    dir: c_int,
    new_qfidx: *mut c_int,
) -> QfLineHandle {
    if qfl.is_null() || new_qfidx.is_null() || errornr <= 0 {
        return std::ptr::null();
    }

    let mut qf_ptr = nvim_qf_get_ptr(qfl);
    let mut qf_idx = nvim_qf_get_index(qfl);

    let mut remaining = errornr;
    while remaining > 0 {
        let prev_ptr = qf_ptr;
        let prev_idx = qf_idx;

        qf_ptr = if dir == direction::FORWARD || dir == direction::FORWARD_FILE {
            rs_qf_get_next_valid_entry_dir(qfl, qf_ptr, &raw mut qf_idx, dir)
        } else {
            rs_qf_get_prev_valid_entry_dir(qfl, qf_ptr, &raw mut qf_idx, dir)
        };

        if qf_ptr.is_null() {
            // Can't move further, return previous position
            *new_qfidx = prev_idx;
            return prev_ptr;
        }

        remaining -= 1;
    }

    *new_qfidx = qf_idx;
    qf_ptr
}

/// Get an entry by index or direction-based navigation.
///
/// This is equivalent to the C function `qf_get_entry()`.
///
/// - If `dir` != 0: navigate to nth valid entry in direction
/// - If `errornr` != 0: navigate to specific entry number
/// - Otherwise: return current entry
///
/// Returns the entry pointer and sets `new_qfidx` to the new index.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `new_qfidx` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_entry(
    qfl: QfListHandle,
    errornr: c_int,
    dir: c_int,
    new_qfidx: *mut c_int,
) -> QfLineHandle {
    if qfl.is_null() || new_qfidx.is_null() {
        return std::ptr::null();
    }

    let mut qf_ptr = nvim_qf_get_ptr(qfl);
    let mut qfidx = nvim_qf_get_index(qfl);

    if dir != 0 {
        // next/prev valid entry in direction
        qf_ptr = rs_qf_get_nth_valid_entry_dir(qfl, errornr.max(1), dir, &raw mut qfidx);
    } else if errornr != 0 {
        // go to specified entry number
        qf_ptr = rs_qf_get_nth_entry(qfl, errornr, &raw mut qfidx);
    }

    *new_qfidx = qfidx;
    qf_ptr
}

/// Get the nth entry (by absolute index) from the current entry.
///
/// This is equivalent to the C function `get_nth_entry()`.
/// Navigates forward or backward to reach the specified entry number.
///
/// Returns the entry pointer and sets `new_qfidx` to the new index.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `new_qfidx` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_nth_entry(
    qfl: QfListHandle,
    errornr: c_int,
    new_qfidx: *mut c_int,
) -> QfLineHandle {
    if qfl.is_null() || new_qfidx.is_null() {
        return std::ptr::null();
    }

    let mut qf_ptr = nvim_qf_get_ptr(qfl);
    let mut qf_idx = nvim_qf_get_index(qfl);
    let count = nvim_qf_get_count(qfl);

    // Navigate backward if errornr < qf_idx
    while errornr < qf_idx && qf_idx > 1 {
        let prev = nvim_qfline_get_prev(qf_ptr);
        if prev.is_null() {
            break;
        }
        qf_idx -= 1;
        qf_ptr = prev;
    }

    // Navigate forward if errornr > qf_idx
    while errornr > qf_idx && qf_idx < count {
        let next = nvim_qfline_get_next(qf_ptr);
        if next.is_null() {
            break;
        }
        qf_idx += 1;
        qf_ptr = next;
    }

    *new_qfidx = qf_idx;
    qf_ptr
}

// =============================================================================
// Phase 6: Parsing Infrastructure
// =============================================================================

/// Parse status codes returned by quickfix parsing functions.
#[allow(dead_code)]
pub mod qf_status {
    use std::ffi::c_int;

    /// Parse failed
    pub const QF_FAIL: c_int = 0;
    /// Parse succeeded
    pub const QF_OK: c_int = 1;
    /// End of input reached
    pub const QF_END_OF_INPUT: c_int = 2;
    /// Out of memory
    pub const QF_NOMEM: c_int = 3;
    /// Line should be ignored
    pub const QF_IGNORE_LINE: c_int = 4;
    /// Multi-scan mode (restart pattern matching)
    pub const QF_MULTISCAN: c_int = 5;
    /// Abort parsing
    pub const QF_ABORT: c_int = 6;
}

/// Error format prefix characters for multiline message handling.
#[allow(dead_code)]
pub mod efm_prefix {
    use std::ffi::c_char;

    /// Enter directory
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_D: c_char = b'D' as c_char;
    /// Leave directory
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_X: c_char = b'X' as c_char;
    /// Start of multi-line message (first line)
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_A: c_char = b'A' as c_char;
    /// Error message
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_E: c_char = b'E' as c_char;
    /// Warning message
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_W: c_char = b'W' as c_char;
    /// Info message
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_I: c_char = b'I' as c_char;
    /// Note message
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_N: c_char = b'N' as c_char;
    /// Continuation line
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_C: c_char = b'C' as c_char;
    /// End of multi-line message
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_Z: c_char = b'Z' as c_char;
    /// General, unspecific message
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_G: c_char = b'G' as c_char;
    /// Push file (partial) message
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_P: c_char = b'P' as c_char;
    /// Pop/quit file (partial) message
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_Q: c_char = b'Q' as c_char;
    /// Overread (partial) message
    #[allow(clippy::cast_possible_wrap)]
    pub const EFM_O: c_char = b'O' as c_char;
}

#[allow(dead_code)]
extern "C" {
    // Phase 6: Multiline state accessors (multiline/multiignore already declared in Phase 5)
    fn nvim_qf_get_multiscan(qfl: QfListHandle) -> bool;
    fn nvim_qf_set_multiscan(qfl: QfListHandleMut, multiscan: bool);
}

/// Check if an error format prefix starts a multiline message.
///
/// Returns true for 'A', 'E', 'W', 'I', 'N' prefixes.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_efm_is_multiline_start(prefix: c_char) -> bool {
    matches!(
        prefix,
        efm_prefix::EFM_A
            | efm_prefix::EFM_E
            | efm_prefix::EFM_W
            | efm_prefix::EFM_I
            | efm_prefix::EFM_N
    )
}

/// Check if an error format prefix is a continuation line.
///
/// Returns true for 'C', 'Z' prefixes.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_efm_is_continuation(prefix: c_char) -> bool {
    matches!(prefix, efm_prefix::EFM_C | efm_prefix::EFM_Z)
}

/// Check if an error format prefix is a directory specifier.
///
/// Returns true for 'D', 'X' prefixes.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_efm_is_directory(prefix: c_char) -> bool {
    matches!(prefix, efm_prefix::EFM_D | efm_prefix::EFM_X)
}

/// Check if an error format prefix is a global file specifier.
///
/// Returns true for 'O', 'P', 'Q' prefixes.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_efm_is_global_file(prefix: c_char) -> bool {
    matches!(
        prefix,
        efm_prefix::EFM_O | efm_prefix::EFM_P | efm_prefix::EFM_Q
    )
}

/// Get the error type character for an error format prefix.
///
/// Returns 'E' for `EFM_E`, 'W' for `EFM_W`, 'I' for `EFM_I`, 'N' for `EFM_N`,
/// or 0 for other prefixes.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_efm_prefix_to_type(prefix: c_char) -> c_char {
    match prefix {
        efm_prefix::EFM_E => error_types::QF_TYPE_ERROR,
        efm_prefix::EFM_W => error_types::QF_TYPE_WARNING,
        efm_prefix::EFM_I => error_types::QF_TYPE_INFO,
        efm_prefix::EFM_N => error_types::QF_TYPE_NOTE,
        _ => error_types::QF_TYPE_NONE,
    }
}

/// Parse a string as a line number.
///
/// Returns the parsed line number or 0 if the input is invalid.
/// This is a safe wrapper around C's atol that handles null pointers.
///
/// # Safety
///
/// - `str_ptr` may be null (returns 0)
/// - If non-null, must point to a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parse_lnum(str_ptr: *const c_char) -> LinenrT {
    if str_ptr.is_null() {
        return 0;
    }

    // Use std::ffi::CStr for safe parsing
    let Ok(c_str) = std::ffi::CStr::from_ptr(str_ptr).to_str() else {
        return 0;
    };

    // Parse leading digits only (like atol)
    let trimmed = c_str.trim_start();
    let (negative, start) = if trimmed.starts_with('-') {
        (true, 1)
    } else if trimmed.starts_with('+') {
        (false, 1)
    } else {
        (false, 0)
    };

    let digits: String = trimmed
        .chars()
        .skip(start)
        .take_while(char::is_ascii_digit)
        .collect();

    if digits.is_empty() {
        return 0;
    }

    digits
        .parse::<LinenrT>()
        .map_or(0, |n| if negative { -n } else { n })
}

/// Parse a string as a column number.
///
/// Returns the parsed column number or 0 if the input is invalid.
///
/// # Safety
///
/// - `str_ptr` may be null (returns 0)
/// - If non-null, must point to a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parse_col(str_ptr: *const c_char) -> c_int {
    rs_qf_parse_lnum(str_ptr)
}

/// Calculate the visual column from a string position.
///
/// When 'viscol' is set in errorformat, the column represents
/// screen column, not byte offset. This helper converts between them.
///
/// Returns the visual column (1-based) for the given string and position.
///
/// # Safety
///
/// - `str_ptr` may be null (returns col as-is)
/// - If non-null, must point to a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_col_to_vcol(
    str_ptr: *const c_char,
    str_len: usize,
    col: c_int,
) -> c_int {
    if str_ptr.is_null() || col <= 0 {
        return col;
    }

    // For simple ASCII text, column == visual column
    // This is a simplified implementation; full implementation would
    // need to handle tabs and multibyte characters
    let bytes = std::slice::from_raw_parts(str_ptr.cast::<u8>(), str_len);

    // Count visual columns up to the byte position
    #[allow(clippy::cast_sign_loss)] // col > 0 verified above
    let target_byte = (col - 1) as usize;
    let mut vcol: c_int = 1;

    for (i, &byte) in bytes.iter().enumerate() {
        if i >= target_byte {
            break;
        }
        if byte == b'\t' {
            // Tab expands to next tabstop (assume 8)
            vcol = ((vcol - 1) / 8 + 1) * 8 + 1;
        } else if byte >= 0x80 {
            // Multibyte character - count as 1-2 columns depending on width
            // Simplified: count as 1 column
            vcol += 1;
        } else {
            vcol += 1;
        }
    }

    vcol
}

/// Trim leading whitespace from a string and return the offset.
///
/// Returns the number of leading whitespace bytes.
///
/// # Safety
///
/// - `str_ptr` may be null (returns 0)
/// - If non-null, must point to a valid C string of at least `str_len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_qf_skip_whitespace(str_ptr: *const c_char, str_len: usize) -> usize {
    if str_ptr.is_null() || str_len == 0 {
        return 0;
    }

    let bytes = std::slice::from_raw_parts(str_ptr.cast::<u8>(), str_len);

    bytes
        .iter()
        .take_while(|&&b| b == b' ' || b == b'\t')
        .count()
}

/// Check if a line appears to be an error/warning message.
///
/// This is a heuristic check for lines that look like compiler output.
/// Looks for common patterns like "error:", "warning:", `file:line:col`, etc.
///
/// # Safety
///
/// - `str_ptr` may be null (returns false)
/// - If non-null, must point to a valid C string of at least `str_len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_qf_looks_like_error(str_ptr: *const c_char, str_len: usize) -> bool {
    if str_ptr.is_null() || str_len == 0 {
        return false;
    }

    let bytes = std::slice::from_raw_parts(str_ptr.cast::<u8>(), str_len);
    let Ok(s) = std::str::from_utf8(bytes) else {
        return false;
    };
    let line = s.to_lowercase();

    // Common error/warning patterns
    line.contains("error:")
        || line.contains("error[")
        || line.contains("warning:")
        || line.contains("warning[")
        || line.contains(": fatal error")
        || line.contains(": error:")
        || line.contains(": warning:")
        // GCC/Clang style: "file.c:123:45: error:"
        || (line.contains(':')
            && line.split(':').take(2).all(|p| !p.is_empty())
            && line
                .split(':')
                .nth(1)
                .is_some_and(|p| p.chars().all(|c| c.is_ascii_digit())))
}

/// Validate a line number is reasonable.
///
/// Returns true if the line number is in a valid range (1 to 2^30).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_qf_valid_lnum(lnum: LinenrT) -> bool {
    (1..=(1 << 30)).contains(&lnum)
}

/// Validate a column number is reasonable.
///
/// Returns true if the column number is in a valid range (0 to 2^20).
/// Column 0 is valid (means "no column specified").
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" fn cannot be const
pub extern "C" fn rs_qf_valid_col(col: c_int) -> bool {
    (0..=(1 << 20)).contains(&col)
}

// =============================================================================
// Phase 7: Navigation Infrastructure
// =============================================================================

/// Direction constants for quickfix navigation.
#[allow(dead_code)]
pub mod direction {
    use std::ffi::c_int;

    /// Move forward (next entry)
    pub const FORWARD: c_int = 1;
    /// Move backward (previous entry)
    pub const BACKWARD: c_int = -1;
    /// Move forward to next file
    pub const FORWARD_FILE: c_int = 3;
    /// Move backward to previous file
    pub const BACKWARD_FILE: c_int = -3;
}

/// Navigate to the nth valid entry from the current position.
///
/// If `count` is positive, moves forward; if negative, moves backward.
/// Returns the new 1-based index, or 0 if navigation failed.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_nth_valid(qfl: QfListHandleMut, count: c_int) -> c_int {
    if qfl.is_null() || count == 0 {
        return 0;
    }

    let mut remaining = count.unsigned_abs() as usize;
    let forward = count > 0;

    while remaining > 0 {
        let new_idx = if forward {
            rs_qf_next_valid_entry(qfl)
        } else {
            rs_qf_prev_valid_entry(qfl)
        };

        if new_idx == 0 {
            // Can't move further, return current position
            return nvim_qf_get_index(qfl);
        }

        remaining -= 1;
    }

    nvim_qf_get_index(qfl)
}

/// Navigate to the nth entry (valid or invalid) from the current position.
///
/// This is a direct jump by index, not searching for valid entries.
/// Returns the new 1-based index, or 0 if the index is out of range.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_entry(qfl: QfListHandleMut, target_idx: c_int) -> c_int {
    if qfl.is_null() || target_idx <= 0 {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if target_idx > count {
        return 0;
    }

    let qfp = rs_qf_get_entry_at_idx(qfl, target_idx);
    if qfp.is_null() {
        return 0;
    }

    nvim_qf_set_index(qfl, target_idx);
    nvim_qf_set_ptr(qfl, qfp);
    target_idx
}

/// Navigate to the next entry in the same file.
///
/// Returns the new 1-based index, or 0 if no more entries in this file.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_next_entry_in_file(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current_ptr = nvim_qf_get_ptr(qfl);
    if current_ptr.is_null() {
        return 0;
    }

    let current_fnum = nvim_qfline_get_fnum(current_ptr);
    let current_idx = nvim_qf_get_index(qfl);
    let count = nvim_qf_get_count(qfl);

    let qfp = nvim_qfline_get_next(current_ptr);
    let idx = current_idx + 1;

    // Check if next entry exists and is in the same file
    if !qfp.is_null() && idx <= count && nvim_qfline_get_fnum(qfp) == current_fnum {
        nvim_qf_set_index(qfl, idx);
        nvim_qf_set_ptr(qfl, qfp);
        return idx;
    }

    0 // No more entries in this file
}

/// Navigate to the previous entry in the same file.
///
/// Returns the new 1-based index, or 0 if no more entries in this file.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_prev_entry_in_file(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current_ptr = nvim_qf_get_ptr(qfl);
    if current_ptr.is_null() {
        return 0;
    }

    let current_fnum = nvim_qfline_get_fnum(current_ptr);
    let current_idx = nvim_qf_get_index(qfl);

    let qfp = nvim_qfline_get_prev(current_ptr);
    let idx = current_idx - 1;

    // Check if previous entry exists and is in the same file
    if !qfp.is_null() && idx >= 1 && nvim_qfline_get_fnum(qfp) == current_fnum {
        nvim_qf_set_index(qfl, idx);
        nvim_qf_set_ptr(qfl, qfp);
        return idx;
    }

    0 // No more entries in this file
}

/// Navigate to the first entry in the next file.
///
/// Returns the new 1-based index, or 0 if already at the last file.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_next_file(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current_ptr = nvim_qf_get_ptr(qfl);
    if current_ptr.is_null() {
        return 0;
    }

    let current_fnum = nvim_qfline_get_fnum(current_ptr);
    let current_idx = nvim_qf_get_index(qfl);
    let count = nvim_qf_get_count(qfl);

    let mut qfp = nvim_qfline_get_next(current_ptr);
    let mut idx = current_idx + 1;

    // Skip entries in the current file
    while !qfp.is_null() && idx <= count {
        if nvim_qfline_get_fnum(qfp) != current_fnum {
            nvim_qf_set_index(qfl, idx);
            nvim_qf_set_ptr(qfl, qfp);
            return idx;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    0 // No more files
}

/// Navigate to the first entry in the previous file.
///
/// Returns the new 1-based index, or 0 if already at the first file.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_prev_file(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current_ptr = nvim_qf_get_ptr(qfl);
    if current_ptr.is_null() {
        return 0;
    }

    let current_fnum = nvim_qfline_get_fnum(current_ptr);
    let current_idx = nvim_qf_get_index(qfl);

    let mut qfp = nvim_qfline_get_prev(current_ptr);
    let mut idx = current_idx - 1;

    // Skip entries in the current file
    while !qfp.is_null() && idx >= 1 {
        if nvim_qfline_get_fnum(qfp) != current_fnum {
            // Found entry in previous file, now find the first entry in that file
            let new_fnum = nvim_qfline_get_fnum(qfp);

            // Keep going back to find the first entry of this file
            let mut first_qfp = qfp;
            let mut first_idx = idx;

            let mut prev = nvim_qfline_get_prev(qfp);
            while !prev.is_null() && (first_idx - 1) >= 1 {
                if nvim_qfline_get_fnum(prev) != new_fnum {
                    break;
                }
                first_qfp = prev;
                first_idx -= 1;
                prev = nvim_qfline_get_prev(first_qfp);
            }

            nvim_qf_set_index(qfl, first_idx);
            nvim_qf_set_ptr(qfl, first_qfp);
            return first_idx;
        }
        qfp = nvim_qfline_get_prev(qfp);
        idx -= 1;
    }

    0 // No previous files
}

/// Count entries in the current file.
///
/// Returns the number of entries with the same file number as the current entry.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_entries_in_file(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current_ptr = nvim_qf_get_ptr(qfl);
    if current_ptr.is_null() {
        return 0;
    }

    let current_fnum = nvim_qfline_get_fnum(current_ptr);
    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_start(qfl);
    let mut file_count: c_int = 0;
    let mut idx = 1;

    while !qfp.is_null() && idx <= count {
        if nvim_qfline_get_fnum(qfp) == current_fnum {
            file_count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    file_count
}

/// Count valid entries in the current file.
///
/// Returns the number of valid entries with the same file number as the current entry.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_valid_in_file(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current_ptr = nvim_qf_get_ptr(qfl);
    if current_ptr.is_null() {
        return 0;
    }

    let current_fnum = nvim_qfline_get_fnum(current_ptr);
    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_start(qfl);
    let mut valid_count: c_int = 0;
    let mut idx = 1;

    while !qfp.is_null() && idx <= count {
        if nvim_qfline_get_fnum(qfp) == current_fnum && nvim_qfline_get_valid(qfp) {
            valid_count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    valid_count
}

/// Get the number of unique files in the quickfix list.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_file_count(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return 0;
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut file_count: c_int = 0;
    let mut last_fnum: c_int = -1;
    let mut idx = 1;

    while !qfp.is_null() && idx <= count {
        let fnum = nvim_qfline_get_fnum(qfp);
        if fnum != last_fnum {
            file_count += 1;
            last_fnum = fnum;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    file_count
}

/// Get the current file's index (1-based) among files in the quickfix list.
///
/// For example, if the list has errors in files A, B, C and we're in B,
/// this returns 2.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_current_file_idx(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current_ptr = nvim_qf_get_ptr(qfl);
    if current_ptr.is_null() {
        return 0;
    }

    let current_fnum = nvim_qfline_get_fnum(current_ptr);
    let current_idx = nvim_qf_get_index(qfl);

    let mut qfp = nvim_qf_get_start(qfl);
    let mut file_idx: c_int = 0;
    let mut last_fnum: c_int = -1;
    let mut idx = 1;

    while !qfp.is_null() && idx <= current_idx {
        let fnum = nvim_qfline_get_fnum(qfp);
        if fnum != last_fnum {
            file_idx += 1;
            last_fnum = fnum;
        }
        if fnum == current_fnum {
            return file_idx;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    file_idx
}

// =============================================================================
// Phase 8: Command Infrastructure
// =============================================================================

/// Command types for quickfix operations.
#[allow(dead_code)]
pub mod qf_cmd {
    use std::ffi::c_int;

    /// :colder / :lolder - go to older list
    pub const CMD_OLDER: c_int = 1;
    /// :cnewer / :lnewer - go to newer list
    pub const CMD_NEWER: c_int = 2;
    /// :chistory / :lhistory - show list history
    pub const CMD_HISTORY: c_int = 3;
}

/// Format an entry for display in :clist output.
///
/// Returns a formatted string like "  1 src/main.c|10| error: msg"
/// The caller must free the returned string.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_format_entry(
    qfp: QfLineHandle,
    idx: c_int,
    is_current: bool,
) -> *mut c_char {
    if qfp.is_null() {
        return std::ptr::null_mut();
    }

    let marker = if is_current { ">" } else { " " };
    let lnum = nvim_qfline_get_lnum(qfp);
    let col = nvim_qfline_get_col(qfp);
    let type_char = nvim_qfline_get_type(qfp);
    let valid = nvim_qfline_get_valid(qfp);

    // Get text, handling null
    let text_ptr = nvim_qfline_get_text(qfp);
    let text = if text_ptr.is_null() {
        ""
    } else {
        std::ffi::CStr::from_ptr(text_ptr)
            .to_str()
            .unwrap_or_default()
    };

    // Build type indicator
    // type_char is a signed char from C, cast to u8 for matching ASCII values
    #[allow(clippy::cast_sign_loss)]
    let type_str = match type_char as u8 {
        b'E' => "error",
        b'W' => "warning",
        b'I' => "info",
        b'N' => "note",
        _ => "",
    };

    // Format the entry
    let formatted = if valid {
        if col > 0 && !type_str.is_empty() {
            format!("{marker}{idx:3}|{lnum}:{col}: {type_str}: {text}")
        } else if col > 0 {
            format!("{marker}{idx:3}|{lnum}:{col}: {text}")
        } else if !type_str.is_empty() {
            format!("{marker}{idx:3}|{lnum}: {type_str}: {text}")
        } else {
            format!("{marker}{idx:3}|{lnum}: {text}")
        }
    } else {
        format!("{marker}{idx:3}|| {text}")
    };

    // Convert to C string and return (caller must free)
    std::ffi::CString::new(formatted).map_or(std::ptr::null_mut(), std::ffi::CString::into_raw)
}

/// Free a string returned by `rs_qf_format_entry`.
///
/// # Safety
///
/// - `ptr` must be a pointer returned by `rs_qf_format_entry` or null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(std::ffi::CString::from_raw(ptr));
    }
}

/// Get a summary string for a quickfix list.
///
/// Returns something like "(3 of 10): :make" or "(empty)" for empty lists.
/// The caller must free the returned string.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct or null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_list_summary(qfl: QfListHandle) -> *mut c_char {
    if qfl.is_null() {
        let cstr = std::ffi::CString::new("(null)").unwrap_or_default();
        return cstr.into_raw();
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        let cstr = std::ffi::CString::new("(empty)").unwrap_or_default();
        return cstr.into_raw();
    }

    let idx = nvim_qf_get_index(qfl);
    let title_ptr = nvim_qf_get_title(qfl);
    let title = if title_ptr.is_null() {
        String::new()
    } else {
        std::ffi::CStr::from_ptr(title_ptr)
            .to_str()
            .map_or_else(|_| String::new(), |s| format!(": {s}"))
    };

    let summary = format!("({idx} of {count}){title}");

    std::ffi::CString::new(summary).map_or(std::ptr::null_mut(), std::ffi::CString::into_raw)
}

/// Get statistics for a quickfix list.
///
/// Returns total count, valid count, and error/warning counts.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - All out parameters must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_stats(
    qfl: QfListHandle,
    out_total: *mut c_int,
    out_valid: *mut c_int,
    out_errors: *mut c_int,
    out_warnings: *mut c_int,
) {
    if qfl.is_null()
        || out_total.is_null()
        || out_valid.is_null()
        || out_errors.is_null()
        || out_warnings.is_null()
    {
        return;
    }

    let count = nvim_qf_get_count(qfl);
    let mut total: c_int = 0;
    let mut valid: c_int = 0;
    let mut errors: c_int = 0;
    let mut warnings: c_int = 0;

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() && idx <= count {
        total += 1;
        if nvim_qfline_get_valid(qfp) {
            valid += 1;
        }
        // type_char is a signed char from C, cast to u8 for matching ASCII values
        #[allow(clippy::cast_sign_loss)]
        match nvim_qfline_get_type(qfp) as u8 {
            b'E' => errors += 1,
            b'W' => warnings += 1,
            _ => {}
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    *out_total = total;
    *out_valid = valid;
    *out_errors = errors;
    *out_warnings = warnings;
}

/// Check if a quickfix list operation should be aborted.
///
/// This checks various conditions that would make continuing the operation
/// pointless, such as an empty list or no valid entries.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct or null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_should_abort(qfl: QfListHandle) -> bool {
    qfl.is_null() || rs_qf_list_empty(qfl)
}

/// Get the range of entries to display for :clist command.
///
/// If `all` is true, returns 1 to count.
/// Otherwise, returns a range around the current entry.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `out_start` and `out_end` must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_display_range(
    qfl: QfListHandle,
    all: bool,
    out_start: *mut c_int,
    out_end: *mut c_int,
) {
    if qfl.is_null() || out_start.is_null() || out_end.is_null() {
        return;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        *out_start = 0;
        *out_end = 0;
        return;
    }

    if all {
        *out_start = 1;
        *out_end = count;
        return;
    }

    // Default: show entries around current position
    let current = nvim_qf_get_index(qfl);
    let context = 5; // Show 5 entries before and after

    let start = if current > context {
        current - context
    } else {
        1
    };
    let end = if current + context <= count {
        current + context
    } else {
        count
    };

    *out_start = start;
    *out_end = end;
}

/// Parse a :cc / :ll style argument to get the entry number.
///
/// Returns the entry number (1-based), or 0 if no number specified,
/// or -1 on error.
///
/// # Safety
///
/// - `arg` may be null (returns 0)
/// - If non-null, must point to a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parse_cc_arg(arg: *const c_char) -> c_int {
    if arg.is_null() {
        return 0;
    }

    let Ok(c_str) = std::ffi::CStr::from_ptr(arg).to_str() else {
        return -1;
    };

    let trimmed = c_str.trim();
    if trimmed.is_empty() {
        return 0;
    }

    match trimmed.parse::<c_int>() {
        Ok(n) if n >= 1 => n,
        _ => -1,
    }
}

/// Parse a count argument for :cnext/:cprev style commands.
///
/// Returns the count (default 1), or -1 on error.
///
/// # Safety
///
/// - `arg` may be null (returns 1)
/// - If non-null, must point to a valid C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parse_count_arg(arg: *const c_char) -> c_int {
    if arg.is_null() {
        return 1;
    }

    let Ok(c_str) = std::ffi::CStr::from_ptr(arg).to_str() else {
        return -1;
    };

    let trimmed = c_str.trim();
    if trimmed.is_empty() {
        return 1;
    }

    match trimmed.parse::<c_int>() {
        Ok(n) if n >= 1 => n,
        _ => -1,
    }
}

// =============================================================================
// Phase 79: Additional Quickfix System Helpers
// =============================================================================

/// Buffer number filtering
pub mod buffer_filter {
    use std::ffi::c_int;

    /// Filter type for quickfix buffer operations
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum QfBufFilter {
        /// All entries
        All = 0,
        /// Only entries with valid buffer numbers
        ValidBuf = 1,
        /// Only entries in a specific buffer
        SpecificBuf = 2,
        /// Only entries without buffer numbers
        NoBuf = 3,
    }

    impl From<c_int> for QfBufFilter {
        fn from(v: c_int) -> Self {
            match v {
                1 => Self::ValidBuf,
                2 => Self::SpecificBuf,
                3 => Self::NoBuf,
                _ => Self::All,
            }
        }
    }
}

/// Entry selection modes
pub mod selection {
    use std::ffi::c_int;

    /// How to select entries when navigating
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum QfSelectMode {
        /// Select absolute entry number
        Absolute = 0,
        /// Select relative to current (+/- n)
        Relative = 1,
        /// Select first entry
        First = 2,
        /// Select last entry
        Last = 3,
        /// Select next valid entry
        NextValid = 4,
        /// Select previous valid entry
        PrevValid = 5,
        /// Select first entry in current file
        FirstInFile = 6,
        /// Select last entry in current file
        LastInFile = 7,
    }

    impl From<c_int> for QfSelectMode {
        fn from(v: c_int) -> Self {
            match v {
                1 => Self::Relative,
                2 => Self::First,
                3 => Self::Last,
                4 => Self::NextValid,
                5 => Self::PrevValid,
                6 => Self::FirstInFile,
                7 => Self::LastInFile,
                _ => Self::Absolute,
            }
        }
    }
}

/// Window operation flags
pub mod window_flags {
    use std::ffi::c_int;

    /// Open entry in new window
    pub const QF_NEW_WINDOW: c_int = 0x01;
    /// Open entry in split window
    pub const QF_SPLIT_WINDOW: c_int = 0x02;
    /// Open entry in vertical split
    pub const QF_VSPLIT: c_int = 0x04;
    /// Open entry in new tab
    pub const QF_NEW_TAB: c_int = 0x08;
    /// Reuse existing window if possible
    pub const QF_REUSE_WINDOW: c_int = 0x10;
    /// Force opening even if buffer modified
    pub const QF_FORCE_OPEN: c_int = 0x20;
}

/// List operation flags
pub mod list_flags {
    use std::ffi::c_int;

    /// Create a new list
    pub const QF_LIST_NEW: c_int = 0x01;
    /// Replace current list
    pub const QF_LIST_REPLACE: c_int = 0x02;
    /// Append to current list
    pub const QF_LIST_APPEND: c_int = 0x04;
    /// Insert before current list
    pub const QF_LIST_INSERT: c_int = 0x08;
    /// Preserve list ID when replacing
    pub const QF_LIST_KEEP_ID: c_int = 0x10;
}

/// Convert selection mode to `c_int`.
#[no_mangle]
pub const extern "C" fn rs_qf_select_mode_to_int(mode: selection::QfSelectMode) -> c_int {
    mode as c_int
}

/// Convert `c_int` to selection mode.
#[no_mangle]
pub extern "C" fn rs_qf_int_to_select_mode(v: c_int) -> selection::QfSelectMode {
    selection::QfSelectMode::from(v)
}

/// Convert buffer filter to `c_int`.
#[no_mangle]
pub const extern "C" fn rs_qf_buf_filter_to_int(filter: buffer_filter::QfBufFilter) -> c_int {
    filter as c_int
}

/// Convert `c_int` to buffer filter.
#[no_mangle]
pub extern "C" fn rs_qf_int_to_buf_filter(v: c_int) -> buffer_filter::QfBufFilter {
    buffer_filter::QfBufFilter::from(v)
}

/// Check if window flags contain a specific flag.
#[no_mangle]
pub const extern "C" fn rs_qf_has_window_flag(flags: c_int, flag: c_int) -> bool {
    flags & flag != 0
}

/// Check if list flags contain a specific flag.
#[no_mangle]
pub const extern "C" fn rs_qf_has_list_flag(flags: c_int, flag: c_int) -> bool {
    flags & flag != 0
}

/// Combine window flags.
#[no_mangle]
pub const extern "C" fn rs_qf_combine_window_flags(a: c_int, b: c_int) -> c_int {
    a | b
}

/// Combine list flags.
#[no_mangle]
pub const extern "C" fn rs_qf_combine_list_flags(a: c_int, b: c_int) -> c_int {
    a | b
}

/// Calculate target entry index for navigation.
///
/// Given the current index, total count, and navigation command,
/// returns the new index (1-based) or -1 if navigation not possible.
#[no_mangle]
#[allow(clippy::match_same_arms)]
pub const extern "C" fn rs_qf_calc_target_idx(
    current: c_int,
    count: c_int,
    mode: selection::QfSelectMode,
    offset: c_int,
) -> c_int {
    if count == 0 {
        return -1;
    }

    match mode {
        selection::QfSelectMode::Absolute => {
            if offset >= 1 && offset <= count {
                offset
            } else {
                -1
            }
        }
        selection::QfSelectMode::Relative => {
            let target = current + offset;
            if target >= 1 && target <= count {
                target
            } else {
                -1
            }
        }
        selection::QfSelectMode::First => 1,
        selection::QfSelectMode::Last => count,
        selection::QfSelectMode::NextValid | selection::QfSelectMode::PrevValid => {
            // These require knowing which entries are valid
            // Return -1 to indicate caller needs more info
            -1
        }
        selection::QfSelectMode::FirstInFile | selection::QfSelectMode::LastInFile => {
            // These require file info
            -1
        }
    }
}

/// Clamp an entry index to valid range.
///
/// Returns the index clamped to `[1, count]`, or 0 if count is 0.
#[no_mangle]
pub const extern "C" fn rs_qf_clamp_idx(idx: c_int, count: c_int) -> c_int {
    if count == 0 {
        0
    } else if idx < 1 {
        1
    } else if idx > count {
        count
    } else {
        idx
    }
}

/// Check if an entry index is valid.
#[no_mangle]
pub const extern "C" fn rs_qf_idx_is_valid(idx: c_int, count: c_int) -> bool {
    idx >= 1 && idx <= count
}

/// Range for display
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfContextRange {
    /// Start index (1-based)
    pub start: c_int,
    /// End index (1-based)
    pub end: c_int,
}

/// Calculate the range of entries to display for `:clist` with context.
///
/// Returns start and end indices (1-based, inclusive).
#[no_mangle]
pub const extern "C" fn rs_qf_calc_context_range(
    current: c_int,
    count: c_int,
    show_all: bool,
    context_lines: c_int,
) -> QfContextRange {
    if count == 0 {
        return QfContextRange { start: 0, end: 0 };
    }

    if show_all {
        return QfContextRange {
            start: 1,
            end: count,
        };
    }

    let start = if current > context_lines {
        current - context_lines
    } else {
        1
    };
    let end = if current + context_lines <= count {
        current + context_lines
    } else {
        count
    };

    QfContextRange { start, end }
}

/// Parsed line:col position
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfLineCol {
    /// Line number (1-based), 0 on error
    pub line: c_int,
    /// Column number (1-based), 0 on error
    pub col: c_int,
}

/// Parse a line:col string to extract line and column numbers.
///
/// Format: "line" or "line:col" or "line,col"
/// Returns line and col where col defaults to 1 if not specified.
/// Returns (0, 0) on error.
///
/// # Safety
///
/// `s` must be null or point to a valid null-terminated C string.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn rs_qf_parse_line_col(s: *const c_char) -> QfLineCol {
    if s.is_null() {
        return QfLineCol::default();
    }

    let Ok(c_str) = std::ffi::CStr::from_ptr(s).to_str() else {
        return QfLineCol::default();
    };

    let trimmed = c_str.trim();
    if trimmed.is_empty() {
        return QfLineCol::default();
    }

    // Try "line:col" format
    if let Some(pos) = trimmed.find(':') {
        let line_str = &trimmed[..pos];
        let col_str = &trimmed[pos + 1..];

        if let (Ok(line), Ok(col)) = (line_str.parse::<c_int>(), col_str.parse::<c_int>()) {
            if line >= 1 && col >= 1 {
                return QfLineCol { line, col };
            }
        }
        return QfLineCol::default();
    }

    // Try "line,col" format
    if let Some(pos) = trimmed.find(',') {
        let line_str = &trimmed[..pos];
        let col_str = &trimmed[pos + 1..];

        if let (Ok(line), Ok(col)) = (line_str.parse::<c_int>(), col_str.parse::<c_int>()) {
            if line >= 1 && col >= 1 {
                return QfLineCol { line, col };
            }
        }
        return QfLineCol::default();
    }

    // Just line number
    if let Ok(line) = trimmed.parse::<c_int>() {
        if line >= 1 {
            return QfLineCol { line, col: 1 };
        }
    }

    QfLineCol::default()
}

/// Get the severity level for an error type character as integer.
///
/// Returns: 0 = info/note, 1 = warning, 2 = error
#[inline]
#[must_use]
#[allow(clippy::cast_sign_loss)]
pub const fn type_severity_int(type_char: c_char) -> c_int {
    match type_char as u8 {
        b'E' | b'e' => 2, // Error
        b'W' | b'w' => 1, // Warning
        // Info/Note and unknown types all return 0
        _ => 0,
    }
}

/// Compare two error types by severity.
///
/// Returns: negative if a < b, 0 if equal, positive if a > b
#[no_mangle]
pub const extern "C" fn rs_qf_compare_type_severity(type_a: c_char, type_b: c_char) -> c_int {
    type_severity_int(type_a) - type_severity_int(type_b)
}

/// Check if a type character indicates an error.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub const extern "C" fn rs_qf_type_is_error(type_char: c_char) -> bool {
    matches!(type_char as u8, b'E' | b'e')
}

/// Check if a type character indicates a warning.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub const extern "C" fn rs_qf_type_is_warning(type_char: c_char) -> bool {
    matches!(type_char as u8, b'W' | b'w')
}

/// Check if a type character indicates info/note.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub const extern "C" fn rs_qf_type_is_info(type_char: c_char) -> bool {
    matches!(type_char as u8, b'I' | b'i' | b'N' | b'n')
}

/// Normalize a type character to uppercase.
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub const extern "C" fn rs_qf_type_normalize(type_char: c_char) -> c_char {
    match type_char as u8 {
        b'e' => b'E' as c_char,
        b'w' => b'W' as c_char,
        b'i' => b'I' as c_char,
        b'n' => b'N' as c_char,
        c => c as c_char,
    }
}

// =============================================================================
// Phase 10 Pass 10 Phase 3: qf_open_new_cwindow + did_set_quickfixtextfunc
// =============================================================================

// Window split flags (from window.h)
const WSP_BOT: c_int = 0x10; // window at bottom-right of shell
const WSP_BELOW: c_int = 0x40; // put new window below/right
const WSP_NEWLOC: c_int = 0x100; // don't copy location list

// Return codes for Phase 3
const P3_OK: c_int = 1;
const P3_FAIL: c_int = 0;

extern "C" {
    #[link_name = "rs_win_setheight"]
    fn p3_rs_win_setheight(height: c_int);
    // nvim_qf_buf_get_fnum takes const void* in C; declared here separately to avoid
    // clashing with navigate.rs which uses a different type alias for BufHandle.
    #[link_name = "nvim_qf_buf_get_fnum"]
    fn p3_nvim_qf_buf_get_fnum(buf: *const c_void) -> c_int;
}

/// Open a new quickfix or location list window.
///
/// Equivalent to C `qf_open_new_cwindow`. Finds or creates the quickfix
/// buffer and opens it in a new split window with appropriate options.
///
/// Returns OK (1) on success or FAIL (0) on failure.
///
/// # Safety
///
/// - `qi` must be a valid mutable pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_open_new_cwindow(qi: QfInfoHandleMut, height: c_int) -> c_int {
    if qi.is_null() {
        return P3_FAIL;
    }

    // Save curwin as oldwin (= win, to be stored as prevwin later)
    let oldwin = nvim_qf_save_curwin();
    let prevtab = nvim_qf_get_curtab();

    // Find existing quickfix buffer (if any)
    let qf_buf = rs_qf_find_buf_for_stack(qi);

    // Default is to open the window below the current window or at the bottom,
    // except when :belowright or :aboveleft is used.
    let flags = if nvim_qf_get_cmdmod_split() == 0 {
        if nvim_qf_is_qf_stack(qi.cast_const()) {
            WSP_BOT
        } else {
            WSP_BELOW
        }
    } else {
        0
    } | WSP_NEWLOC;

    if nvim_qf_win_split(height, flags) == P3_FAIL {
        return P3_FAIL; // not enough room for window
    }
    nvim_qf_curwin_reset_binding();

    if nvim_qf_is_ll_stack(qi.cast_const()) {
        // For the location list window, create a reference to the
        // location list stack from the window 'win'.
        nvim_qf_curwin_set_llist_ref_incr(qi);
    }

    // If curwin changed after win_split (it usually does), don't store info.
    let effective_oldwin = if nvim_qf_curwin_is(oldwin) {
        oldwin
    } else {
        std::ptr::null_mut()
    };

    if qf_buf.is_null() {
        // Create a new quickfix buffer
        if nvim_qf_do_ecmd_new_buf(effective_oldwin) == P3_FAIL {
            return P3_FAIL;
        }
        // Save the number of the new buffer
        let curbuf_fnum = nvim_qf_curbuf_fnum();
        nvim_qf_set_bufnr(qi, curbuf_fnum);
    } else {
        // Use the existing quickfix buffer
        let fnum = p3_nvim_qf_buf_get_fnum(qf_buf.cast_const());
        if nvim_qf_do_ecmd_existing_buf(fnum, effective_oldwin) == P3_FAIL {
            return P3_FAIL;
        }
    }

    // Set the options for the quickfix buffer/window (if not already done).
    // Do this even if the quickfix buffer was already present, as an autocmd
    // might have previously deleted (:bdelete) the quickfix buffer.
    if !nvim_qf_curbuf_is_quickfix() {
        nvim_qf_set_cwindow_options();
    }

    // Only set the height when still in the same tab page and there is no
    // window to the side.
    if nvim_qf_curtab_eq(prevtab) && nvim_qf_curwin_width() == nvim_qf_get_columns() {
        p3_rs_win_setheight(height);
    }
    nvim_qf_curwin_set_wfh();
    if nvim_win_valid(oldwin) {
        nvim_qf_set_prevwin(oldwin);
    }
    P3_OK
}

/// Process the 'quickfixtextfunc' option value.
///
/// This is the Rust implementation of C `did_set_quickfixtextfunc`.
/// It calls `option_set_callback_func(p_qftf, &qftf_cb)` through a C accessor.
/// Returns NULL on success or the address of `e_invarg` on failure.
///
/// # Safety
///
/// May only be called from the option-setting machinery.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_quickfixtextfunc(_args: *const c_void) -> *const c_char {
    if nvim_qf_option_set_callback_func_for_qftf() == P3_FAIL {
        return nvim_qf_get_e_invarg();
    }
    std::ptr::null()
}

// =============================================================================
// Phase 10 Pass 10 Phase 4: qf_update_buffer
// =============================================================================

/// Find the quickfix buffer. If it exists, update its contents.
///
/// Equivalent to C `qf_update_buffer`. Called after entries are added or changed.
///
/// # Safety
///
/// - `qi` must be a valid mutable pointer to a `qf_info_T` struct
/// - `old_last` may be null (full update) or a valid `qfline_T*` (append update)
#[no_mangle]
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_qf_update_buffer(qi: QfInfoHandleMut, old_last: QfLineHandle) {
    if qi.is_null() {
        return;
    }

    // Check if a buffer for the quickfix list exists.
    let buf = rs_qf_find_buf_for_stack(qi);
    if buf.is_null() {
        return;
    }

    let old_line_count = nvim_buf_get_ml_line_count_void(buf.cast_const());
    let old_endcol = nvim_ml_get_buf_len(buf, old_line_count);
    let old_bytecount = nvim_qf_get_region_bytecount(buf, 1, old_line_count, 0, old_endcol);

    // For location list stacks, find the associated window and get its winid.
    let qf_winid: c_int = if nvim_qf_is_ll_stack(qi.cast_const()) {
        let curwin = nvim_qf_get_curwin();
        let curwin_llist = nvim_buf_win_get_llist(curwin.cast_const());
        let win = if std::ptr::eq(curwin_llist, qi) {
            curwin
        } else {
            let mut w = nvim_qf_find_win_with_loclist(qi.cast_const());
            if w.is_null() {
                w = rs_qf_find_win_for_stack(qi.cast_const()).cast_mut();
            }
            if w.is_null() {
                return;
            }
            w
        };
        nvim_qf_win_get_handle(win.cast_const())
    } else {
        0
    };

    // Autocommands may cause trouble - increment busy count
    crate::lifecycle::rs_incr_quickfix_busy();

    let aco = if old_last.is_null() {
        // Set curwin/curbuf to buf and save a few things
        nvim_qf_aucmd_prepbuf_alloc(buf)
    } else {
        std::ptr::null_mut()
    };

    rs_qf_update_win_titlevar(qi);

    let qfl = nvim_qf_get_curlist(qi.cast_const());
    crate::display::rs_qf_fill_buffer(qfl.cast_mut(), buf, old_last, qf_winid);

    let new_line_count = nvim_buf_get_ml_line_count_void(buf.cast_const());
    let new_endcol = nvim_ml_get_buf_len(buf, new_line_count);
    let delta = new_line_count - old_line_count;

    if old_last.is_null() {
        let new_byte_count = nvim_qf_get_region_bytecount(buf, 1, new_line_count, 0, new_endcol);
        nvim_qf_extmark_splice(
            buf,
            0,
            0,
            old_line_count - 1,
            0,
            old_bytecount,
            new_line_count - 1,
            new_endcol,
            new_byte_count,
        );
        let lnume = if old_line_count > 0 {
            old_line_count + 1
        } else {
            1
        };
        nvim_qf_changed_lines(buf, 1, 0, lnume, delta, true);
    } else if delta > 0 {
        let start_lnum = old_line_count + 1;
        let new_byte_count =
            nvim_qf_get_region_bytecount(buf, start_lnum, new_line_count, 0, new_endcol);
        nvim_qf_extmark_splice(
            buf,
            old_line_count - 1,
            old_endcol,
            0,
            0,
            0,
            delta,
            new_endcol,
            new_byte_count,
        );
        nvim_qf_changed_lines(buf, start_lnum, 0, start_lnum, delta, true);
    }
    nvim_qf_buf_set_changed_false(buf);

    if old_last.is_null() {
        rs_qf_win_pos_update_impl(qi, 0);
        nvim_qf_aucmd_restbuf_free(aco);
    }

    // Only redraw when added lines are visible.
    let qf_win = rs_qf_find_win_for_stack(qi.cast_const()).cast_mut();
    if !qf_win.is_null() && old_line_count < nvim_qf_win_botline(qf_win.cast_const()) {
        nvim_qf_redraw_buf_later(buf);
    }

    // Always called after rs_incr_quickfix_busy()
    crate::lifecycle::rs_decr_quickfix_busy();
}

// =============================================================================
// Phase 79 Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::manual_c_str_literals,
    clippy::cast_possible_wrap,
    clippy::ptr_as_ptr
)]
mod phase79_tests {
    use super::*;

    #[test]
    fn test_select_mode_conversion() {
        assert_eq!(
            rs_qf_select_mode_to_int(selection::QfSelectMode::Absolute),
            0
        );
        assert_eq!(
            rs_qf_select_mode_to_int(selection::QfSelectMode::Relative),
            1
        );
        assert_eq!(rs_qf_select_mode_to_int(selection::QfSelectMode::First), 2);
        assert_eq!(rs_qf_select_mode_to_int(selection::QfSelectMode::Last), 3);

        assert_eq!(
            rs_qf_int_to_select_mode(0),
            selection::QfSelectMode::Absolute
        );
        assert_eq!(
            rs_qf_int_to_select_mode(1),
            selection::QfSelectMode::Relative
        );
        assert_eq!(rs_qf_int_to_select_mode(2), selection::QfSelectMode::First);
        assert_eq!(rs_qf_int_to_select_mode(3), selection::QfSelectMode::Last);
        assert_eq!(
            rs_qf_int_to_select_mode(99),
            selection::QfSelectMode::Absolute
        );
    }

    #[test]
    fn test_buf_filter_conversion() {
        assert_eq!(rs_qf_buf_filter_to_int(buffer_filter::QfBufFilter::All), 0);
        assert_eq!(
            rs_qf_buf_filter_to_int(buffer_filter::QfBufFilter::ValidBuf),
            1
        );
        assert_eq!(rs_qf_int_to_buf_filter(0), buffer_filter::QfBufFilter::All);
        assert_eq!(
            rs_qf_int_to_buf_filter(1),
            buffer_filter::QfBufFilter::ValidBuf
        );
    }

    #[test]
    fn test_window_flags() {
        let flags = window_flags::QF_NEW_WINDOW | window_flags::QF_VSPLIT;
        assert!(rs_qf_has_window_flag(flags, window_flags::QF_NEW_WINDOW));
        assert!(rs_qf_has_window_flag(flags, window_flags::QF_VSPLIT));
        assert!(!rs_qf_has_window_flag(flags, window_flags::QF_NEW_TAB));

        let combined =
            rs_qf_combine_window_flags(window_flags::QF_NEW_WINDOW, window_flags::QF_FORCE_OPEN);
        assert!(rs_qf_has_window_flag(combined, window_flags::QF_NEW_WINDOW));
        assert!(rs_qf_has_window_flag(combined, window_flags::QF_FORCE_OPEN));
    }

    #[test]
    fn test_list_flags() {
        let flags = list_flags::QF_LIST_NEW | list_flags::QF_LIST_KEEP_ID;
        assert!(rs_qf_has_list_flag(flags, list_flags::QF_LIST_NEW));
        assert!(rs_qf_has_list_flag(flags, list_flags::QF_LIST_KEEP_ID));
        assert!(!rs_qf_has_list_flag(flags, list_flags::QF_LIST_APPEND));
    }

    #[test]
    fn test_calc_target_idx() {
        // Empty list
        assert_eq!(
            rs_qf_calc_target_idx(0, 0, selection::QfSelectMode::First, 0),
            -1
        );

        // First/Last
        assert_eq!(
            rs_qf_calc_target_idx(5, 10, selection::QfSelectMode::First, 0),
            1
        );
        assert_eq!(
            rs_qf_calc_target_idx(5, 10, selection::QfSelectMode::Last, 0),
            10
        );

        // Absolute
        assert_eq!(
            rs_qf_calc_target_idx(5, 10, selection::QfSelectMode::Absolute, 3),
            3
        );
        assert_eq!(
            rs_qf_calc_target_idx(5, 10, selection::QfSelectMode::Absolute, 15),
            -1
        );

        // Relative
        assert_eq!(
            rs_qf_calc_target_idx(5, 10, selection::QfSelectMode::Relative, 2),
            7
        );
        assert_eq!(
            rs_qf_calc_target_idx(5, 10, selection::QfSelectMode::Relative, -3),
            2
        );
        assert_eq!(
            rs_qf_calc_target_idx(5, 10, selection::QfSelectMode::Relative, 10),
            -1
        );
    }

    #[test]
    fn test_clamp_idx() {
        assert_eq!(rs_qf_clamp_idx(5, 10), 5);
        assert_eq!(rs_qf_clamp_idx(0, 10), 1);
        assert_eq!(rs_qf_clamp_idx(-5, 10), 1);
        assert_eq!(rs_qf_clamp_idx(15, 10), 10);
        assert_eq!(rs_qf_clamp_idx(5, 0), 0);
    }

    #[test]
    fn test_idx_is_valid() {
        assert!(rs_qf_idx_is_valid(1, 10));
        assert!(rs_qf_idx_is_valid(10, 10));
        assert!(rs_qf_idx_is_valid(5, 10));
        assert!(!rs_qf_idx_is_valid(0, 10));
        assert!(!rs_qf_idx_is_valid(11, 10));
        assert!(!rs_qf_idx_is_valid(1, 0));
    }

    #[test]
    fn test_calc_context_range() {
        // Empty list
        let r = rs_qf_calc_context_range(0, 0, false, 5);
        assert_eq!((r.start, r.end), (0, 0));

        // Show all
        let r = rs_qf_calc_context_range(5, 100, true, 5);
        assert_eq!((r.start, r.end), (1, 100));

        // With context
        let r = rs_qf_calc_context_range(50, 100, false, 5);
        assert_eq!((r.start, r.end), (45, 55));
        let r = rs_qf_calc_context_range(3, 100, false, 5);
        assert_eq!((r.start, r.end), (1, 8));
        let r = rs_qf_calc_context_range(98, 100, false, 5);
        assert_eq!((r.start, r.end), (93, 100));
    }

    #[test]
    fn test_parse_line_col() {
        unsafe {
            // Line only
            let r = rs_qf_parse_line_col(b"10\0".as_ptr() as *const c_char);
            assert_eq!((r.line, r.col), (10, 1));

            // Line:col
            let r = rs_qf_parse_line_col(b"10:5\0".as_ptr() as *const c_char);
            assert_eq!((r.line, r.col), (10, 5));

            // Line,col
            let r = rs_qf_parse_line_col(b"10,5\0".as_ptr() as *const c_char);
            assert_eq!((r.line, r.col), (10, 5));

            // Invalid
            let r = rs_qf_parse_line_col(b"abc\0".as_ptr() as *const c_char);
            assert_eq!((r.line, r.col), (0, 0));
            let r = rs_qf_parse_line_col(b"0\0".as_ptr() as *const c_char);
            assert_eq!((r.line, r.col), (0, 0));
            let r = rs_qf_parse_line_col(std::ptr::null());
            assert_eq!((r.line, r.col), (0, 0));
        }
    }

    #[test]
    fn test_type_severity() {
        assert_eq!(type_severity_int(b'E' as c_char), 2);
        assert_eq!(type_severity_int(b'e' as c_char), 2);
        assert_eq!(type_severity_int(b'W' as c_char), 1);
        assert_eq!(type_severity_int(b'w' as c_char), 1);
        assert_eq!(type_severity_int(b'I' as c_char), 0);
        assert_eq!(type_severity_int(b'N' as c_char), 0);
    }

    #[test]
    fn test_cmp_type_severity() {
        assert!(rs_qf_compare_type_severity(b'E' as c_char, b'W' as c_char) > 0);
        assert!(rs_qf_compare_type_severity(b'W' as c_char, b'E' as c_char) < 0);
        assert_eq!(
            rs_qf_compare_type_severity(b'E' as c_char, b'e' as c_char),
            0
        );
    }

    #[test]
    fn test_type_classification() {
        assert!(rs_qf_type_is_error(b'E' as c_char));
        assert!(rs_qf_type_is_error(b'e' as c_char));
        assert!(!rs_qf_type_is_error(b'W' as c_char));

        assert!(rs_qf_type_is_warning(b'W' as c_char));
        assert!(rs_qf_type_is_warning(b'w' as c_char));
        assert!(!rs_qf_type_is_warning(b'E' as c_char));

        assert!(rs_qf_type_is_info(b'I' as c_char));
        assert!(rs_qf_type_is_info(b'N' as c_char));
        assert!(!rs_qf_type_is_info(b'E' as c_char));
    }

    #[test]
    fn test_normalize_type() {
        assert_eq!(rs_qf_type_normalize(b'e' as c_char), b'E' as c_char);
        assert_eq!(rs_qf_type_normalize(b'E' as c_char), b'E' as c_char);
        assert_eq!(rs_qf_type_normalize(b'w' as c_char), b'W' as c_char);
        assert_eq!(rs_qf_type_normalize(b'i' as c_char), b'I' as c_char);
        assert_eq!(rs_qf_type_normalize(b'n' as c_char), b'N' as c_char);
        assert_eq!(rs_qf_type_normalize(b'X' as c_char), b'X' as c_char);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_stack_is_empty() {
        unsafe {
            assert!(rs_qf_stack_empty(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_list_is_empty() {
        unsafe {
            assert!(rs_qf_list_empty(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_list_no_valid_entries() {
        // Null list has no valid entries
        unsafe {
            assert!(!rs_qf_list_has_valid_entries(std::ptr::null()));
        }
    }

    #[test]
    fn test_linenr_type_size() {
        // LinenrT should be i32 (4 bytes)
        assert_eq!(std::mem::size_of::<LinenrT>(), 4);
    }

    #[test]
    fn test_linenr_range() {
        // LinenrT should be able to hold reasonable line numbers
        let max_lines: LinenrT = 1_000_000_000;
        assert!(max_lines > 0);
        // Minimum valid line number is 1
        let min_valid: LinenrT = 1;
        assert!(min_valid > 0);
    }

    #[test]
    fn test_null_pointers_safe() {
        // All null pointer cases should be safe
        unsafe {
            // Stack functions
            assert!(rs_qf_stack_empty(std::ptr::null()));
            // List functions
            assert!(rs_qf_list_empty(std::ptr::null()));
            assert!(!rs_qf_list_has_valid_entries(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_index_validation() {
        unsafe {
            // Null stack should always return false for valid_idx
            assert!(!rs_qf_valid_idx(std::ptr::null(), 0));
            assert!(!rs_qf_valid_idx(std::ptr::null(), 1));
        }
    }

    #[test]
    fn test_null_get_size() {
        unsafe {
            // Null list should return 0 size
            assert_eq!(rs_qf_get_size(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_find_first_entry_in_buf() {
        unsafe {
            let mut idx = 99;
            let result = rs_qf_find_first_entry_in_buf(std::ptr::null(), 1, &raw mut idx);
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_null_find_closest_entry() {
        unsafe {
            let mut errornr = 99;
            // Null list
            let result = rs_qf_find_closest_entry(
                std::ptr::null(),
                1,
                std::ptr::null(),
                1,
                false,
                &raw mut errornr,
            );
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_direction_values() {
        // Ensure Direction enum has expected values
        assert_eq!(Direction::Forward as c_int, 1);
        assert_eq!(Direction::Backward as c_int, -1);
    }

    // Phase 4 tests

    #[test]
    fn test_null_entry_count_in_file() {
        unsafe {
            assert_eq!(rs_qf_entry_count_in_file(std::ptr::null(), 1), 0);
        }
    }

    #[test]
    fn test_null_count_valid_entries() {
        unsafe {
            assert_eq!(rs_qf_count_valid_entries(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_get_nth_valid_entry() {
        unsafe {
            let mut idx = 99;
            let result = rs_qf_get_nth_valid_entry(std::ptr::null(), 1, &raw mut idx);
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_null_get_entry_at_idx() {
        unsafe {
            assert!(rs_qf_get_entry_at_idx(std::ptr::null(), 1).is_null());
        }
    }

    // Phase 5 tests

    #[test]
    fn test_valid_error_types() {
        assert!(rs_qf_valid_error_type(error_types::QF_TYPE_ERROR));
        assert!(rs_qf_valid_error_type(error_types::QF_TYPE_WARNING));
        assert!(rs_qf_valid_error_type(error_types::QF_TYPE_INFO));
        assert!(rs_qf_valid_error_type(error_types::QF_TYPE_NOTE));
        assert!(rs_qf_valid_error_type(error_types::QF_TYPE_NONE));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_invalid_error_types() {
        assert!(!rs_qf_valid_error_type(b'X' as c_char));
        assert!(!rs_qf_valid_error_type(b'e' as c_char)); // lowercase not valid
        assert!(!rs_qf_valid_error_type(b'1' as c_char));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_parse_type_uppercase() {
        assert_eq!(rs_qf_parse_type(b'E' as c_char), error_types::QF_TYPE_ERROR);
        assert_eq!(
            rs_qf_parse_type(b'W' as c_char),
            error_types::QF_TYPE_WARNING
        );
        assert_eq!(rs_qf_parse_type(b'I' as c_char), error_types::QF_TYPE_INFO);
        assert_eq!(rs_qf_parse_type(b'N' as c_char), error_types::QF_TYPE_NOTE);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_parse_type_lowercase() {
        // Lowercase should be normalized to uppercase
        assert_eq!(rs_qf_parse_type(b'e' as c_char), error_types::QF_TYPE_ERROR);
        assert_eq!(
            rs_qf_parse_type(b'w' as c_char),
            error_types::QF_TYPE_WARNING
        );
        assert_eq!(rs_qf_parse_type(b'i' as c_char), error_types::QF_TYPE_INFO);
        assert_eq!(rs_qf_parse_type(b'n' as c_char), error_types::QF_TYPE_NOTE);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_parse_type_invalid() {
        assert_eq!(rs_qf_parse_type(b'X' as c_char), error_types::QF_TYPE_NONE);
        assert_eq!(rs_qf_parse_type(b'1' as c_char), error_types::QF_TYPE_NONE);
    }

    #[test]
    fn test_null_cmp_entries() {
        unsafe {
            assert_eq!(rs_qf_cmp_entries(std::ptr::null(), std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_entry_in_file() {
        unsafe {
            assert!(!rs_qf_entry_in_file(std::ptr::null(), 1));
        }
    }

    #[test]
    fn test_null_entry_is_active() {
        unsafe {
            assert!(!rs_qf_entry_is_active(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_entry_has_type() {
        unsafe {
            assert!(!rs_qf_entry_has_type(
                std::ptr::null(),
                error_types::QF_TYPE_ERROR
            ));
        }
    }

    // Phase 6 tests

    #[test]
    fn test_null_skip_to_valid() {
        unsafe {
            let mut idx = 99;
            let result = rs_qf_skip_to_valid(std::ptr::null(), 1, &raw mut idx);
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_null_skip_to_file() {
        unsafe {
            let mut idx = 99;
            let result = rs_qf_skip_to_file(std::ptr::null(), 1, &raw mut idx);
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_null_next_valid_in_file() {
        unsafe {
            let mut idx = 99;
            let result = rs_qf_next_valid_in_file(std::ptr::null(), 1, &raw mut idx);
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_null_prev_valid_in_file() {
        unsafe {
            let mut idx = 99;
            let result = rs_qf_prev_valid_in_file(std::ptr::null(), 1, &raw mut idx);
            assert!(result.is_null());
        }
    }

    // Phase 6 new tests - List ID and Index Functions

    #[test]
    fn test_null_get_id() {
        unsafe {
            assert_eq!(rs_qf_get_id(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_get_changedtick() {
        unsafe {
            assert_eq!(rs_qf_get_changedtick(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_has_title() {
        unsafe {
            assert!(!rs_qf_has_title(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_get_maxcount() {
        unsafe {
            assert_eq!(rs_qf_get_maxcount(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_stack_at_max() {
        unsafe {
            assert!(!rs_qf_stack_at_max(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_find_list_by_id() {
        unsafe {
            assert_eq!(rs_qf_find_list_by_id(std::ptr::null(), 1), -1);
        }
    }

    #[test]
    fn test_find_list_by_id_zero() {
        unsafe {
            // ID of 0 should always return -1
            assert_eq!(rs_qf_find_list_by_id(std::ptr::null(), 0), -1);
        }
    }

    #[test]
    fn test_null_id_exists() {
        unsafe {
            assert!(!rs_qf_id_exists(std::ptr::null(), 1));
        }
    }

    // Entry Range Function tests

    #[test]
    fn test_null_entry_in_range() {
        unsafe {
            assert!(!rs_qf_entry_in_range(std::ptr::null(), 1, 10));
        }
    }

    #[test]
    fn test_null_entry_covers_line() {
        unsafe {
            assert!(!rs_qf_entry_covers_line(std::ptr::null(), 5));
        }
    }

    #[test]
    fn test_null_entry_covers_pos() {
        unsafe {
            assert!(!rs_qf_entry_covers_pos(std::ptr::null(), 5, 10));
        }
    }

    #[test]
    fn test_null_count_entries_of_type() {
        unsafe {
            assert_eq!(
                rs_qf_count_entries_of_type(std::ptr::null(), error_types::QF_TYPE_ERROR),
                0
            );
        }
    }

    #[test]
    fn test_null_count_errors() {
        unsafe {
            assert_eq!(rs_qf_count_errors(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_count_warnings() {
        unsafe {
            assert_eq!(rs_qf_count_warnings(std::ptr::null()), 0);
        }
    }
}
