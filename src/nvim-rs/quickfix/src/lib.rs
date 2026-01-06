//! Quickfix list functions for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations of quickfix/location list functions.

use std::ffi::{c_char, c_int, c_void};

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
}

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
