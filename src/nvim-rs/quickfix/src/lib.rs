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
}
