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
