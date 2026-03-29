//! Quickfix/location list filtering and search operations.
//!
//! This module provides functions for filtering entries by various criteria
//! and searching text within quickfix entries.

use crate::ffi_types::QfListPtr;
use std::ffi::{c_char, c_int};

use crate::{LinenrT, QfLinePtr};

extern "C" {
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
}

// =============================================================================
// Type-based Filtering
// =============================================================================

/// Quickfix entry type constants.
#[allow(clippy::cast_possible_wrap)]
pub mod entry_types {
    use std::ffi::c_char;

    /// Error type ('E').
    pub const TYPE_ERROR: c_char = b'E' as c_char;
    /// Warning type ('W').
    pub const TYPE_WARNING: c_char = b'W' as c_char;
    /// Info type ('I').
    pub const TYPE_INFO: c_char = b'I' as c_char;
    /// Note type ('N').
    pub const TYPE_NOTE: c_char = b'N' as c_char;
    /// No type specified (empty/space).
    pub const TYPE_NONE: c_char = b' ' as c_char;
}

/// Count entries of a specific type in the quickfix list.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_by_type(qfl: QfListPtr, entry_type: c_char) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        if (*qfp).qf_type == entry_type {
            count += 1;
        }
        qfp = (*qfp).qf_next;
    }

    count
}

// Note: rs_qf_count_errors and rs_qf_count_warnings already exist in lib.rs

/// Count info entries ('I') in the quickfix list.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_info(qfl: QfListPtr) -> c_int {
    rs_qf_count_by_type(qfl, entry_types::TYPE_INFO)
}

// =============================================================================
// File-based Filtering
// =============================================================================

/// Count entries for a specific buffer in the quickfix list.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_in_buffer(qfl: QfListPtr, bnr: c_int) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        if (*qfp).qf_fnum == bnr {
            count += 1;
        }
        qfp = (*qfp).qf_next;
    }

    count
}

/// Count valid entries for a specific buffer in the quickfix list.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_valid_in_buffer(qfl: QfListPtr, bnr: c_int) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        if (*qfp).qf_fnum == bnr && ((*qfp).qf_valid != 0) {
            count += 1;
        }
        qfp = (*qfp).qf_next;
    }

    count
}

/// Count entries on a specific line in a buffer.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_on_line(qfl: QfListPtr, bnr: c_int, lnum: LinenrT) -> c_int {
    if qfl.is_null() || lnum <= 0 {
        return 0;
    }

    let mut count = 0;
    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        if (*qfp).qf_fnum == bnr && (*qfp).qf_lnum == lnum {
            count += 1;
        }
        qfp = (*qfp).qf_next;
    }

    count
}

// =============================================================================
// Text Search
// =============================================================================

/// Check if an entry's text contains a substring (case-sensitive).
///
/// # Safety
///
/// - `qfp` may be null (returns false)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
/// - `pattern` may be null (returns false)
/// - If non-null, `pattern` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_text_contains(qfp: QfLinePtr, pattern: *const c_char) -> bool {
    if qfp.is_null() || pattern.is_null() {
        return false;
    }

    let text = (*qfp).qf_text;
    if text.is_null() {
        return false;
    }

    // Simple substring search using libc strstr
    !strstr(text, pattern).is_null()
}

/// Find the first entry containing the given text pattern.
///
/// Returns the 1-based index, or 0 if not found.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `pattern` may be null (returns 0)
/// - If non-null, `pattern` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_text(qfl: QfListPtr, pattern: *const c_char) -> c_int {
    if qfl.is_null() || pattern.is_null() {
        return 0;
    }

    let mut qfp = (*qfl).qf_start;
    let mut idx = 1;

    while !qfp.is_null() {
        if rs_qf_entry_text_contains(qfp, pattern) {
            return idx;
        }
        qfp = (*qfp).qf_next;
        idx += 1;
    }

    0
}

/// Find the next entry containing the given text pattern after `start_idx`.
///
/// Returns the 1-based index, or 0 if not found.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `pattern` may be null (returns 0)
/// - If non-null, `pattern` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_text_after(
    qfl: QfListPtr,
    pattern: *const c_char,
    start_idx: c_int,
) -> c_int {
    if qfl.is_null() || pattern.is_null() {
        return 0;
    }

    let count = (*qfl).qf_count;
    if start_idx >= count {
        return 0;
    }

    let mut qfp = (*qfl).qf_start;
    let mut idx = 1;

    // Skip to start position
    while !qfp.is_null() && idx <= start_idx {
        qfp = (*qfp).qf_next;
        idx += 1;
    }

    // Search from there
    while !qfp.is_null() {
        if rs_qf_entry_text_contains(qfp, pattern) {
            return idx;
        }
        qfp = (*qfp).qf_next;
        idx += 1;
    }

    0
}

/// Count entries containing the given text pattern.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `pattern` may be null (returns 0)
/// - If non-null, `pattern` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_text_matches(qfl: QfListPtr, pattern: *const c_char) -> c_int {
    if qfl.is_null() || pattern.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        if rs_qf_entry_text_contains(qfp, pattern) {
            count += 1;
        }
        qfp = (*qfp).qf_next;
    }

    count
}

// =============================================================================
// Validity Filtering
// =============================================================================

/// Check if a list has any valid entries in a specific file.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_has_valid_in_buffer(qfl: QfListPtr, bnr: c_int) -> bool {
    if qfl.is_null() {
        return false;
    }

    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        if (*qfp).qf_fnum == bnr && ((*qfp).qf_valid != 0) {
            return true;
        }
        qfp = (*qfp).qf_next;
    }

    false
}

/// Check if a list has only invalid entries.
///
/// Returns true if the list is non-empty but all entries are invalid.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_all_invalid(qfl: QfListPtr) -> bool {
    if qfl.is_null() {
        return false;
    }

    let count = (*qfl).qf_count;
    if count == 0 {
        return false;
    }

    (*qfl).qf_nonevalid
}

/// Check if a list has at least one valid entry with the specified type.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_has_valid_type(qfl: QfListPtr, entry_type: c_char) -> bool {
    if qfl.is_null() {
        return false;
    }

    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        if ((*qfp).qf_valid != 0) && (*qfp).qf_type == entry_type {
            return true;
        }
        qfp = (*qfp).qf_next;
    }

    false
}

// =============================================================================
// Combined Filtering
// =============================================================================

/// Filter statistics for a quickfix list.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfFilterStats {
    /// Total number of entries.
    pub total: c_int,
    /// Number of valid entries.
    pub valid: c_int,
    /// Number of error entries.
    pub errors: c_int,
    /// Number of warning entries.
    pub warnings: c_int,
    /// Number of info entries.
    pub info: c_int,
    /// Number of entries in the current buffer (if bnr > 0).
    pub in_buffer: c_int,
}

/// Get comprehensive filter statistics for a quickfix list.
///
/// If `bnr` is > 0, also counts entries in that specific buffer.
///
/// # Safety
///
/// - `qfl` may be null (returns zeroed stats)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_filter_stats(qfl: QfListPtr, bnr: c_int) -> QfFilterStats {
    let mut stats = QfFilterStats::default();

    if qfl.is_null() {
        return stats;
    }

    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        stats.total += 1;

        if (*qfp).qf_valid != 0 {
            stats.valid += 1;
        }

        let entry_type = (*qfp).qf_type;
        if entry_type == entry_types::TYPE_ERROR {
            stats.errors += 1;
        } else if entry_type == entry_types::TYPE_WARNING {
            stats.warnings += 1;
        } else if entry_type == entry_types::TYPE_INFO {
            stats.info += 1;
        }

        if bnr > 0 && (*qfp).qf_fnum == bnr {
            stats.in_buffer += 1;
        }

        qfp = (*qfp).qf_next;
    }

    stats
}

/// Find the first entry matching multiple criteria.
///
/// Returns the 1-based index, or 0 if not found.
///
/// - `bnr`: Filter by buffer number (0 to skip)
/// - `entry_type`: Filter by type (0 to skip)
/// - `valid_only`: Only match valid entries
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_matching(
    qfl: QfListPtr,
    bnr: c_int,
    entry_type: c_char,
    valid_only: bool,
) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut qfp = (*qfl).qf_start;
    let mut idx = 1;

    while !qfp.is_null() {
        let matches = (bnr == 0 || (*qfp).qf_fnum == bnr)
            && (entry_type == 0 || (*qfp).qf_type == entry_type)
            && (!valid_only || ((*qfp).qf_valid != 0));

        if matches {
            return idx;
        }

        qfp = (*qfp).qf_next;
        idx += 1;
    }

    0
}

/// Find the next entry matching multiple criteria after `start_idx`.
///
/// Returns the 1-based index, or 0 if not found.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_matching_after(
    qfl: QfListPtr,
    start_idx: c_int,
    bnr: c_int,
    entry_type: c_char,
    valid_only: bool,
) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = (*qfl).qf_count;
    if start_idx >= count {
        return 0;
    }

    let mut qfp = (*qfl).qf_start;
    let mut idx = 1;

    // Skip to start position
    while !qfp.is_null() && idx <= start_idx {
        qfp = (*qfp).qf_next;
        idx += 1;
    }

    // Search from there
    while !qfp.is_null() {
        let matches = (bnr == 0 || (*qfp).qf_fnum == bnr)
            && (entry_type == 0 || (*qfp).qf_type == entry_type)
            && (!valid_only || ((*qfp).qf_valid != 0));

        if matches {
            return idx;
        }

        qfp = (*qfp).qf_next;
        idx += 1;
    }

    0
}

// =============================================================================
// Line Range Filtering
// =============================================================================

/// Check if a list has any entries in a line range.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_has_entries_in_range(
    qfl: QfListPtr,
    bnr: c_int,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
) -> bool {
    if qfl.is_null() || start_lnum > end_lnum {
        return false;
    }

    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        if (*qfp).qf_fnum == bnr {
            let lnum = (*qfp).qf_lnum;
            if lnum >= start_lnum && lnum <= end_lnum {
                return true;
            }
        }
        qfp = (*qfp).qf_next;
    }

    false
}

/// Count entries in a line range.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_count_in_range(
    qfl: QfListPtr,
    bnr: c_int,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
) -> c_int {
    if qfl.is_null() || start_lnum > end_lnum {
        return 0;
    }

    let mut count = 0;
    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        if (*qfp).qf_fnum == bnr {
            let lnum = (*qfp).qf_lnum;
            if lnum >= start_lnum && lnum <= end_lnum {
                count += 1;
            }
        }
        qfp = (*qfp).qf_next;
    }

    count
}

/// Find first entry in a line range.
///
/// Returns the 1-based index, or 0 if not found.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_in_range(
    qfl: QfListPtr,
    bnr: c_int,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
) -> c_int {
    if qfl.is_null() || start_lnum > end_lnum {
        return 0;
    }

    let mut qfp = (*qfl).qf_start;
    let mut idx = 1;

    while !qfp.is_null() {
        if (*qfp).qf_fnum == bnr {
            let lnum = (*qfp).qf_lnum;
            if lnum >= start_lnum && lnum <= end_lnum {
                return idx;
            }
        }
        qfp = (*qfp).qf_next;
        idx += 1;
    }

    0
}

// =============================================================================
// Phase 2: qf_get_nth_valid_entry with fdo (file-do) mode
// =============================================================================

/// Get the 1-based index of the n-th valid entry.
///
/// When `fdo` is true (for `:cfdo`/`:lfdo`), counts unique file numbers rather
/// than individual valid entries.
///
/// Returns 1 if the list has no valid entries.
///
/// Mirrors C `qf_get_nth_valid_entry`.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T`
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_nth_valid_entry_do(
    qfl: QfListPtr,
    n: c_int,
    fdo: bool,
) -> c_int {
    if qfl.is_null() {
        return 1;
    }

    // Return 1 if the list has no valid entries
    if (*qfl).qf_nonevalid {
        return 1;
    }

    let count = (*qfl).qf_count;
    if count <= 0 || n <= 0 {
        return 1;
    }

    let n = n.unsigned_abs() as usize;
    let mut prev_fnum: c_int = 0;
    let mut eidx: usize = 0;
    let mut i: c_int = 1;
    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() && i <= count {
        if (*qfp).qf_valid != 0 {
            if fdo {
                let fnum = (*qfp).qf_fnum;
                if fnum > 0 && fnum != prev_fnum {
                    eidx += 1;
                    prev_fnum = fnum;
                }
            } else {
                eidx += 1;
            }
        }

        if eidx == n {
            break;
        }

        qfp = (*qfp).qf_next;
        i += 1;
    }

    if i <= count {
        i
    } else {
        1
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_entry_types() {
        assert_eq!(entry_types::TYPE_ERROR, b'E' as c_char);
        assert_eq!(entry_types::TYPE_WARNING, b'W' as c_char);
        assert_eq!(entry_types::TYPE_INFO, b'I' as c_char);
        assert_eq!(entry_types::TYPE_NOTE, b'N' as c_char);
        assert_eq!(entry_types::TYPE_NONE, b' ' as c_char);
    }

    #[test]
    fn test_null_safety_counts() {
        unsafe {
            assert_eq!(
                rs_qf_count_by_type(std::ptr::null_mut(), entry_types::TYPE_ERROR),
                0
            );
            // rs_qf_count_errors and rs_qf_count_warnings are in lib.rs
            assert_eq!(rs_qf_count_info(std::ptr::null_mut()), 0);
            assert_eq!(rs_qf_count_in_buffer(std::ptr::null_mut(), 1), 0);
            assert_eq!(rs_qf_count_valid_in_buffer(std::ptr::null_mut(), 1), 0);
            assert_eq!(rs_qf_count_on_line(std::ptr::null_mut(), 1, 10), 0);
        }
    }

    #[test]
    fn test_null_safety_text_search() {
        unsafe {
            let pattern = c"test".as_ptr();
            assert!(!rs_qf_entry_text_contains(std::ptr::null_mut(), pattern));
            assert!(!rs_qf_entry_text_contains(
                std::ptr::null_mut(),
                std::ptr::null()
            ));
            assert_eq!(rs_qf_find_text(std::ptr::null_mut(), pattern), 0);
            assert_eq!(rs_qf_find_text(std::ptr::null_mut(), std::ptr::null()), 0);
            assert_eq!(rs_qf_find_text_after(std::ptr::null_mut(), pattern, 0), 0);
            assert_eq!(rs_qf_count_text_matches(std::ptr::null_mut(), pattern), 0);
        }
    }

    #[test]
    fn test_null_safety_validity() {
        unsafe {
            assert!(!rs_qf_has_valid_in_buffer(std::ptr::null_mut(), 1));
            assert!(!rs_qf_all_invalid(std::ptr::null_mut()));
            assert!(!rs_qf_has_valid_type(
                std::ptr::null_mut(),
                entry_types::TYPE_ERROR
            ));
        }
    }

    #[test]
    fn test_null_safety_filter_stats() {
        unsafe {
            let stats = rs_qf_filter_stats(std::ptr::null_mut(), 0);
            assert_eq!(stats.total, 0);
            assert_eq!(stats.valid, 0);
            assert_eq!(stats.errors, 0);
            assert_eq!(stats.warnings, 0);
            assert_eq!(stats.info, 0);
            assert_eq!(stats.in_buffer, 0);
        }
    }

    #[test]
    fn test_null_safety_matching() {
        unsafe {
            assert_eq!(rs_qf_find_matching(std::ptr::null_mut(), 0, 0, false), 0);
            assert_eq!(
                rs_qf_find_matching_after(std::ptr::null_mut(), 0, 0, 0, false),
                0
            );
        }
    }

    #[test]
    fn test_null_safety_range() {
        unsafe {
            assert!(!rs_qf_has_entries_in_range(std::ptr::null_mut(), 1, 1, 10));
            assert_eq!(rs_qf_count_in_range(std::ptr::null_mut(), 1, 1, 10), 0);
            assert_eq!(rs_qf_find_in_range(std::ptr::null_mut(), 1, 1, 10), 0);
        }
    }

    #[test]
    fn test_invalid_range() {
        unsafe {
            // End before start should return 0/false
            assert!(!rs_qf_has_entries_in_range(std::ptr::null_mut(), 1, 10, 1));
            assert_eq!(rs_qf_count_in_range(std::ptr::null_mut(), 1, 10, 1), 0);
            assert_eq!(rs_qf_find_in_range(std::ptr::null_mut(), 1, 10, 1), 0);
        }
    }

    #[test]
    fn test_filter_stats_default() {
        let stats = QfFilterStats::default();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.valid, 0);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.warnings, 0);
        assert_eq!(stats.info, 0);
        assert_eq!(stats.in_buffer, 0);
    }
}
