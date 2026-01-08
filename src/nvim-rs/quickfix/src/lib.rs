//! Quickfix list functions for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations of quickfix/location list functions.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Submodules
// =============================================================================

pub mod filter;
pub mod list;
pub mod navigate;
pub mod parse;

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
    fn nvim_qflist_valid(qi: QfInfoHandle, qf_id: u32) -> bool;
    fn nvim_qf_entry_present(qfl: QfListHandle, qf_ptr: QfLineHandle) -> bool;
    fn nvim_qf_types(c: c_int, nr: c_int) -> *const c_char;
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

    nvim_qflist_valid(qi, qf_id)
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
    nvim_qf_entry_present(qfl, qf_ptr)
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

/// Get the error type string for display.
///
/// Returns a formatted string like " error", " warning", " info", " note",
/// or a custom type string. The nr parameter adds a number suffix if > 0.
///
/// The returned pointer points to a static buffer in C and must not be freed.
///
/// # Safety
///
/// - The returned pointer is only valid until the next call to this function
#[no_mangle]
pub unsafe extern "C" fn rs_qf_types(c: c_int, nr: c_int) -> *const c_char {
    nvim_qf_types(c, nr)
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
    if qi.is_null() {
        return 0;
    }

    let win = nvim_qf_find_win_handle(qi);
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
    fn nvim_qf_new_list(qi: QfInfoHandleMut, title: *const c_char);
    fn nvim_qf_free_list(qfl: QfListHandleMut);
    fn nvim_qf_free_items(qfl: QfListHandleMut);
    fn nvim_qf_store_title(qfl: QfListHandleMut, title: *const c_char);
    fn nvim_get_ql_info() -> QfInfoHandleMut;

    // Phase 3: List modification accessors
    fn nvim_qf_pop_stack(qi: QfInfoHandleMut, adjust: bool);
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
    fn nvim_qf_push_dir(
        qfl: QfListHandleMut,
        dirbuf: *mut c_char,
        is_file_stack: bool,
    ) -> *const c_char;
    fn nvim_qf_pop_dir(qfl: QfListHandleMut, is_file_stack: bool) -> *const c_char;
    fn nvim_qf_clean_dir_stack(qfl: QfListHandleMut, is_file_stack: bool);
    fn nvim_qf_guess_filepath(qfl: QfListHandleMut, filename: *mut c_char) -> *const c_char;
    fn nvim_qf_get_fnum(qfl: QfListHandleMut, directory: *mut c_char, fname: *mut c_char) -> c_int;

    // Phase 5: Error Format Parsing accessors
    fn nvim_qf_parse_efm_option(efm: *mut c_char) -> EfmHandle;
    fn nvim_qf_free_efm_list(efm_first: EfmHandle);
    fn nvim_efm_get_next(efm: EfmHandle) -> EfmHandle;
    fn nvim_efm_get_prefix(efm: EfmHandle) -> c_char;
    fn nvim_efm_get_flags(efm: EfmHandle) -> c_char;
    fn nvim_efm_get_conthere(efm: EfmHandle) -> c_int;
    fn nvim_efm_get_addr(efm: EfmHandle, idx: c_int) -> c_char;
    fn nvim_qf_get_multiline(qfl: QfListHandle) -> bool;
    fn nvim_qf_set_multiline(qfl: QfListHandleMut, multiline: bool);
    fn nvim_qf_get_multiignore(qfl: QfListHandle) -> bool;
    fn nvim_qf_set_multiignore(qfl: QfListHandleMut, multiignore: bool);

    // Phase 6: Input Sources and Buffer Operations accessors
    fn nvim_qf_state_alloc() -> QfStateHandle;
    fn nvim_qf_state_free(state: QfStateHandle);
    fn nvim_qf_state_setup_file(
        state: QfStateHandle,
        enc: *mut c_char,
        efile: *const c_char,
    ) -> c_int;
    fn nvim_qf_state_setup_buffer(
        state: QfStateHandle,
        buf: *mut c_void,
        lnumfirst: c_int,
        lnumlast: c_int,
    ) -> c_int;
    fn nvim_qf_state_get_nextline(state: QfStateHandle) -> c_int;
    fn nvim_qf_state_get_linebuf(state: QfStateHandle) -> *const c_char;
    fn nvim_qf_state_get_linelen(state: QfStateHandle) -> usize;
    fn nvim_qf_state_has_fd(state: QfStateHandle) -> bool;
    fn nvim_qf_state_has_tv(state: QfStateHandle) -> bool;
    fn nvim_qf_state_has_buf(state: QfStateHandle) -> bool;

    // Phase 7: Window and Display Management accessors
    fn nvim_qf_find_win_for_stack(qi: QfInfoHandle) -> WinHandle;
    fn nvim_qf_find_buf_for_stack(qi: QfInfoHandleMut) -> BufHandle;
    fn nvim_qf_win_pos_update(qi: QfInfoHandleMut, old_qf_index: c_int) -> bool;
    fn nvim_qf_update_buffer(qi: QfInfoHandleMut, old_last: QfLineHandle);
    fn nvim_qf_get_bufnr(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_set_bufnr(qi: QfInfoHandleMut, bufnr: c_int);
    fn nvim_win_is_qf_win(win: WinHandle) -> bool;
    fn nvim_win_get_llist_ref(win: WinHandle) -> QfInfoHandle;

    // Phase 8: Ex Commands and API Functions accessors
    // (nvim_qf_get_title and nvim_qf_get_changedtick already declared above)
    fn nvim_qf_is_qf_stack(qi: QfInfoHandle) -> bool;
    fn nvim_qf_is_ll_stack(qi: QfInfoHandle) -> bool;
    fn nvim_qf_get_refcount(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_incr_refcount(qi: QfInfoHandleMut);
    fn nvim_qf_decr_refcount(qi: QfInfoHandleMut);
    fn nvim_qf_get_ctx(qfl: QfListHandle) -> *mut c_void;
    fn nvim_qf_incr_changedtick(qfl: QfListHandleMut);

    // Phase 3 Extension: qfline_T allocation and field-setting
    fn nvim_qfline_alloc() -> QfLineHandleMut;
    fn nvim_qfline_free(qfp: QfLineHandleMut);
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
    fn nvim_qf_mark_buf_has_entry(bufnum: c_int, is_location_list: bool);
    fn nvim_qf_get_fnum_for_entry(
        qfl: QfListHandleMut,
        directory: *mut c_char,
        fname: *mut c_char,
    ) -> c_int;
    fn nvim_qf_fix_fname(fname: *const c_char, bufnum: c_int) -> *mut c_char;
    fn nvim_qf_is_printc(c: c_int) -> bool;
}

/// Opaque handle to buffer (Phase 7)
type BufHandle = *mut c_void;

/// Opaque handle to errorformat pattern list (`efm_T`)
type EfmHandle = *mut c_void;

/// Opaque handle to parser state (`qfstate_T`)
type QfStateHandle = *mut c_void;

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

/// Create a new quickfix list in the stack.
///
/// This creates a new list, potentially removing older lists if the stack is full.
/// The new list becomes the current list.
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
    nvim_qf_new_list(qi, title);
}

/// Free all resources of a quickfix list.
///
/// This frees all entries, the title, context, and other resources.
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
    nvim_qf_free_list(qfl);
}

/// Free only the entries in a quickfix list.
///
/// This frees all entry items but preserves the list structure.
/// Used when repopulating a list with new content.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_free_items(qfl: QfListHandleMut) {
    if qfl.is_null() {
        return;
    }
    nvim_qf_free_items(qfl);
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
    nvim_qf_store_title(qfl, title);
}

/// Pop the oldest list from the quickfix stack.
///
/// This removes the first (oldest) list from the stack, shifting all
/// remaining lists down. If `adjust` is true, also decrements listcount
/// and adjusts curlist.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_pop_stack(qi: QfInfoHandleMut, adjust: bool) {
    if qi.is_null() {
        return;
    }
    nvim_qf_pop_stack(qi, adjust);
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
        // Mark the buffer as having a quickfix entry
        let is_location_list = rs_qf_get_qfl_type(qfl) == qfl_types::QFLT_LOCATION;
        nvim_qf_mark_buf_has_entry(fnum, is_location_list);
    } else {
        fnum = nvim_qf_get_fnum_for_entry(qfl, dir, fname.cast_mut());
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
    nvim_qf_push_dir(qfl, dirbuf, is_file_stack)
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
    nvim_qf_pop_dir(qfl, is_file_stack)
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
    nvim_qf_clean_dir_stack(qfl, is_file_stack);
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
    nvim_qf_guess_filepath(qfl, filename)
}

/// Get the buffer number for a file, creating the buffer if needed.
///
/// Resolves the file path using the given directory (or the directory stack
/// if the path is relative) and returns the buffer number. Creates the
/// buffer if it doesn't exist.
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
    nvim_qf_get_fnum(qfl, directory, fname)
}

// =============================================================================
// Phase 5: Error Format Parsing
// =============================================================================

/// Parse the errorformat option string and return a handle to the pattern list.
///
/// The returned handle must be freed with `rs_qf_free_efm_list` when done.
/// Returns null if parsing fails.
///
/// # Safety
///
/// - `efm` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_parse_efm_option(efm: *mut c_char) -> EfmHandle {
    if efm.is_null() {
        return std::ptr::null_mut();
    }
    nvim_qf_parse_efm_option(efm)
}

/// Free an errorformat pattern list.
///
/// # Safety
///
/// - `efm_first` must be a valid errorformat handle or null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_free_efm_list(efm_first: EfmHandle) {
    if efm_first.is_null() {
        return;
    }
    nvim_qf_free_efm_list(efm_first);
}

/// Get the next pattern in the errorformat list.
///
/// Returns null when there are no more patterns.
///
/// # Safety
///
/// - `efm` must be a valid errorformat handle
#[no_mangle]
pub unsafe extern "C" fn rs_efm_get_next(efm: EfmHandle) -> EfmHandle {
    if efm.is_null() {
        return std::ptr::null_mut();
    }
    nvim_efm_get_next(efm)
}

/// Get the prefix character of an errorformat pattern.
///
/// Prefix characters indicate the type of line:
/// - 'D': enter directory
/// - 'X': leave directory
/// - 'A': start of multi-line message
/// - 'E': error message
/// - 'W': warning message
/// - 'I': informational message
/// - 'N': note message
/// - 'C': continuation line
/// - 'Z': end of multi-line message
/// - 'G': general, unspecific message
/// - 'P': push file (partial) message
/// - 'Q': pop/quit file (partial) message
/// - 'O': overread (partial) message
///
/// # Safety
///
/// - `efm` must be a valid errorformat handle
#[no_mangle]
pub unsafe extern "C" fn rs_efm_get_prefix(efm: EfmHandle) -> c_char {
    if efm.is_null() {
        return 0;
    }
    nvim_efm_get_prefix(efm)
}

/// Get the flags character of an errorformat pattern.
///
/// Flags modify behavior:
/// - '-': do not include this line
/// - '+': include whole line in message
///
/// # Safety
///
/// - `efm` must be a valid errorformat handle
#[no_mangle]
pub unsafe extern "C" fn rs_efm_get_flags(efm: EfmHandle) -> c_char {
    if efm.is_null() {
        return 0;
    }
    nvim_efm_get_flags(efm)
}

/// Check if an errorformat pattern has a conthere marker (%>).
///
/// Returns non-zero if the pattern uses %> (continue here).
///
/// # Safety
///
/// - `efm` must be a valid errorformat handle
#[no_mangle]
pub unsafe extern "C" fn rs_efm_get_conthere(efm: EfmHandle) -> c_int {
    if efm.is_null() {
        return 0;
    }
    nvim_efm_get_conthere(efm)
}

/// Get the address array entry for a pattern index.
///
/// Returns the capture group number (1-based) or 0 if the pattern is not used.
/// Index values 0-13 correspond to format specifiers: f, n, l, e, c, k, t, m, r, p, v, s, o, b.
///
/// # Safety
///
/// - `efm` must be a valid errorformat handle
/// - `idx` should be in range [0, 14)
#[no_mangle]
pub unsafe extern "C" fn rs_efm_get_addr(efm: EfmHandle, idx: c_int) -> c_char {
    if efm.is_null() {
        return 0;
    }
    nvim_efm_get_addr(efm, idx)
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
// Phase 6: Input Sources and Buffer Operations
// =============================================================================

/// Result codes for quickfix parsing operations.
#[allow(dead_code)]
pub mod qf_result {
    use std::ffi::c_int;

    /// Operation failed
    pub const QF_FAIL: c_int = 0;
    /// Operation succeeded
    pub const QF_OK: c_int = 1;
    /// End of input reached
    pub const QF_END_OF_INPUT: c_int = 2;
    /// Out of memory
    pub const QF_NOMEM: c_int = 3;
    /// Line should be ignored
    pub const QF_IGNORE_LINE: c_int = 4;
    /// Multi-scan mode active
    pub const QF_MULTISCAN: c_int = 5;
    /// Parsing aborted
    pub const QF_ABORT: c_int = 6;
}

/// Allocate a new parser state object.
///
/// The returned handle must be freed with `rs_qf_state_free` when done.
///
/// # Safety
///
/// This function allocates memory and returns a valid handle.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_state_alloc() -> QfStateHandle {
    nvim_qf_state_alloc()
}

/// Free a parser state object.
///
/// This also cleans up any resources held by the state (file handles, etc).
///
/// # Safety
///
/// - `state` must be a valid parser state handle or null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_state_free(state: QfStateHandle) {
    if state.is_null() {
        return;
    }
    nvim_qf_state_free(state);
}

/// Setup the parser state for reading from a file.
///
/// Returns OK (1) on success, FAIL (0) on error (e.g., file not found).
///
/// # Safety
///
/// - `state` must be a valid parser state handle
/// - `enc` can be null or a valid encoding string
/// - `efile` must be a valid file path or "-" for stdin
#[no_mangle]
pub unsafe extern "C" fn rs_qf_state_setup_file(
    state: QfStateHandle,
    enc: *mut c_char,
    efile: *const c_char,
) -> c_int {
    if state.is_null() {
        return 0; // FAIL
    }
    nvim_qf_state_setup_file(state, enc, efile)
}

/// Setup the parser state for reading from a buffer.
///
/// Returns OK (1) on success, FAIL (0) on error.
///
/// # Safety
///
/// - `state` must be a valid parser state handle
/// - `buf` must be a valid buffer pointer
/// - `lnumfirst` and `lnumlast` define the line range to read
#[no_mangle]
pub unsafe extern "C" fn rs_qf_state_setup_buffer(
    state: QfStateHandle,
    buf: *mut c_void,
    lnumfirst: c_int,
    lnumlast: c_int,
) -> c_int {
    if state.is_null() || buf.is_null() {
        return 0; // FAIL
    }
    nvim_qf_state_setup_buffer(state, buf, lnumfirst, lnumlast)
}

/// Get the next line from the input source.
///
/// Returns `QF_OK` on success, `QF_END_OF_INPUT` when done, or `QF_FAIL` on error.
/// After calling, use `rs_qf_state_get_linebuf` to access the line content.
///
/// # Safety
///
/// - `state` must be a valid parser state handle that has been set up
#[no_mangle]
pub unsafe extern "C" fn rs_qf_state_get_nextline(state: QfStateHandle) -> c_int {
    if state.is_null() {
        return qf_result::QF_FAIL;
    }
    nvim_qf_state_get_nextline(state)
}

/// Get the current line buffer from the parser state.
///
/// Returns the line content read by the last `get_nextline` call, or null.
///
/// # Safety
///
/// - `state` must be a valid parser state handle
#[no_mangle]
pub unsafe extern "C" fn rs_qf_state_get_linebuf(state: QfStateHandle) -> *const c_char {
    if state.is_null() {
        return std::ptr::null();
    }
    nvim_qf_state_get_linebuf(state)
}

/// Get the current line length from the parser state.
///
/// Returns the length of the line read by the last `get_nextline` call, or 0.
///
/// # Safety
///
/// - `state` must be a valid parser state handle
#[no_mangle]
pub unsafe extern "C" fn rs_qf_state_get_linelen(state: QfStateHandle) -> usize {
    if state.is_null() {
        return 0;
    }
    nvim_qf_state_get_linelen(state)
}

/// Check if the parser state is reading from a file.
///
/// # Safety
///
/// - `state` must be a valid parser state handle
#[no_mangle]
pub unsafe extern "C" fn rs_qf_state_has_fd(state: QfStateHandle) -> bool {
    if state.is_null() {
        return false;
    }
    nvim_qf_state_has_fd(state)
}

/// Check if the parser state has a typval (string or list input).
///
/// # Safety
///
/// - `state` must be a valid parser state handle
#[no_mangle]
pub unsafe extern "C" fn rs_qf_state_has_tv(state: QfStateHandle) -> bool {
    if state.is_null() {
        return false;
    }
    nvim_qf_state_has_tv(state)
}

/// Check if the parser state is reading from a buffer.
///
/// # Safety
///
/// - `state` must be a valid parser state handle
#[no_mangle]
pub unsafe extern "C" fn rs_qf_state_has_buf(state: QfStateHandle) -> bool {
    if state.is_null() {
        return false;
    }
    nvim_qf_state_has_buf(state)
}

// =============================================================================
// Phase 7: Window and Display Management
// =============================================================================

/// Find the quickfix window for a given quickfix stack.
///
/// Returns the window handle, or null if no window is displaying this stack.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_win_for_stack(qi: QfInfoHandle) -> WinHandle {
    if qi.is_null() {
        return std::ptr::null();
    }
    nvim_qf_find_win_for_stack(qi)
}

/// Find the quickfix buffer for a given quickfix stack.
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
    nvim_qf_find_buf_for_stack(qi)
}

/// Update the cursor position in the quickfix window.
///
/// Moves the cursor to the current error entry and updates the redraw range.
/// Returns true if there is a quickfix window.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_win_pos_update(qi: QfInfoHandleMut, old_qf_index: c_int) -> bool {
    if qi.is_null() {
        return false;
    }
    nvim_qf_win_pos_update(qi, old_qf_index)
}

/// Update the quickfix buffer contents.
///
/// Refreshes the buffer to reflect changes to the quickfix list.
/// Pass `old_last` to only update entries added after that point.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T` struct
/// - `old_last` can be null to refresh the entire list
#[no_mangle]
pub unsafe extern "C" fn rs_qf_update_buffer(qi: QfInfoHandleMut, old_last: QfLineHandle) {
    if qi.is_null() {
        return;
    }
    nvim_qf_update_buffer(qi, old_last);
}

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
    nvim_qf_decr_refcount(qi);
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
