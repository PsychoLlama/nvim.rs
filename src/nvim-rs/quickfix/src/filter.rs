//! Quickfix/location list filtering and search operations.
//!
//! This module provides functions for filtering entries by various criteria
//! and searching text within quickfix entries.

use std::ffi::{c_char, c_int};

use crate::{
    nvim_qf_get_count, nvim_qf_get_nonevalid, nvim_qf_get_start, nvim_qfline_get_fnum,
    nvim_qfline_get_lnum, nvim_qfline_get_next, nvim_qfline_get_text, nvim_qfline_get_type,
    nvim_qfline_get_valid, LinenrT, QfLineHandle, QfListHandle,
};

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
pub unsafe extern "C" fn rs_qf_count_by_type(qfl: QfListHandle, entry_type: c_char) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_type(qfp) == entry_type {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
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
pub unsafe extern "C" fn rs_qf_count_info(qfl: QfListHandle) -> c_int {
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
pub unsafe extern "C" fn rs_qf_count_in_buffer(qfl: QfListHandle, bnr: c_int) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) == bnr {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
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
pub unsafe extern "C" fn rs_qf_count_valid_in_buffer(qfl: QfListHandle, bnr: c_int) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) == bnr && nvim_qfline_get_valid(qfp) {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
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
pub unsafe extern "C" fn rs_qf_count_on_line(
    qfl: QfListHandle,
    bnr: c_int,
    lnum: LinenrT,
) -> c_int {
    if qfl.is_null() || lnum <= 0 {
        return 0;
    }

    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) == bnr && nvim_qfline_get_lnum(qfp) == lnum {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
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
pub unsafe extern "C" fn rs_qf_entry_text_contains(
    qfp: QfLineHandle,
    pattern: *const c_char,
) -> bool {
    if qfp.is_null() || pattern.is_null() {
        return false;
    }

    let text = nvim_qfline_get_text(qfp);
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
pub unsafe extern "C" fn rs_qf_find_text(qfl: QfListHandle, pattern: *const c_char) -> c_int {
    if qfl.is_null() || pattern.is_null() {
        return 0;
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() {
        if rs_qf_entry_text_contains(qfp, pattern) {
            return idx;
        }
        qfp = nvim_qfline_get_next(qfp);
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
    qfl: QfListHandle,
    pattern: *const c_char,
    start_idx: c_int,
) -> c_int {
    if qfl.is_null() || pattern.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if start_idx >= count {
        return 0;
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    // Skip to start position
    while !qfp.is_null() && idx <= start_idx {
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    // Search from there
    while !qfp.is_null() {
        if rs_qf_entry_text_contains(qfp, pattern) {
            return idx;
        }
        qfp = nvim_qfline_get_next(qfp);
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
pub unsafe extern "C" fn rs_qf_count_text_matches(
    qfl: QfListHandle,
    pattern: *const c_char,
) -> c_int {
    if qfl.is_null() || pattern.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if rs_qf_entry_text_contains(qfp, pattern) {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
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
pub unsafe extern "C" fn rs_qf_has_valid_in_buffer(qfl: QfListHandle, bnr: c_int) -> bool {
    if qfl.is_null() {
        return false;
    }

    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) == bnr && nvim_qfline_get_valid(qfp) {
            return true;
        }
        qfp = nvim_qfline_get_next(qfp);
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
pub unsafe extern "C" fn rs_qf_all_invalid(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return false;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return false;
    }

    nvim_qf_get_nonevalid(qfl)
}

/// Check if a list has at least one valid entry with the specified type.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_has_valid_type(qfl: QfListHandle, entry_type: c_char) -> bool {
    if qfl.is_null() {
        return false;
    }

    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) && nvim_qfline_get_type(qfp) == entry_type {
            return true;
        }
        qfp = nvim_qfline_get_next(qfp);
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
pub unsafe extern "C" fn rs_qf_filter_stats(qfl: QfListHandle, bnr: c_int) -> QfFilterStats {
    let mut stats = QfFilterStats::default();

    if qfl.is_null() {
        return stats;
    }

    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        stats.total += 1;

        if nvim_qfline_get_valid(qfp) {
            stats.valid += 1;
        }

        let entry_type = nvim_qfline_get_type(qfp);
        if entry_type == entry_types::TYPE_ERROR {
            stats.errors += 1;
        } else if entry_type == entry_types::TYPE_WARNING {
            stats.warnings += 1;
        } else if entry_type == entry_types::TYPE_INFO {
            stats.info += 1;
        }

        if bnr > 0 && nvim_qfline_get_fnum(qfp) == bnr {
            stats.in_buffer += 1;
        }

        qfp = nvim_qfline_get_next(qfp);
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
    qfl: QfListHandle,
    bnr: c_int,
    entry_type: c_char,
    valid_only: bool,
) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() {
        let matches = (bnr == 0 || nvim_qfline_get_fnum(qfp) == bnr)
            && (entry_type == 0 || nvim_qfline_get_type(qfp) == entry_type)
            && (!valid_only || nvim_qfline_get_valid(qfp));

        if matches {
            return idx;
        }

        qfp = nvim_qfline_get_next(qfp);
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
    qfl: QfListHandle,
    start_idx: c_int,
    bnr: c_int,
    entry_type: c_char,
    valid_only: bool,
) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if start_idx >= count {
        return 0;
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    // Skip to start position
    while !qfp.is_null() && idx <= start_idx {
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    // Search from there
    while !qfp.is_null() {
        let matches = (bnr == 0 || nvim_qfline_get_fnum(qfp) == bnr)
            && (entry_type == 0 || nvim_qfline_get_type(qfp) == entry_type)
            && (!valid_only || nvim_qfline_get_valid(qfp));

        if matches {
            return idx;
        }

        qfp = nvim_qfline_get_next(qfp);
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
    qfl: QfListHandle,
    bnr: c_int,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
) -> bool {
    if qfl.is_null() || start_lnum > end_lnum {
        return false;
    }

    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) == bnr {
            let lnum = nvim_qfline_get_lnum(qfp);
            if lnum >= start_lnum && lnum <= end_lnum {
                return true;
            }
        }
        qfp = nvim_qfline_get_next(qfp);
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
    qfl: QfListHandle,
    bnr: c_int,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
) -> c_int {
    if qfl.is_null() || start_lnum > end_lnum {
        return 0;
    }

    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) == bnr {
            let lnum = nvim_qfline_get_lnum(qfp);
            if lnum >= start_lnum && lnum <= end_lnum {
                count += 1;
            }
        }
        qfp = nvim_qfline_get_next(qfp);
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
    qfl: QfListHandle,
    bnr: c_int,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
) -> c_int {
    if qfl.is_null() || start_lnum > end_lnum {
        return 0;
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) == bnr {
            let lnum = nvim_qfline_get_lnum(qfp);
            if lnum >= start_lnum && lnum <= end_lnum {
                return idx;
            }
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    0
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
                rs_qf_count_by_type(std::ptr::null(), entry_types::TYPE_ERROR),
                0
            );
            // rs_qf_count_errors and rs_qf_count_warnings are in lib.rs
            assert_eq!(rs_qf_count_info(std::ptr::null()), 0);
            assert_eq!(rs_qf_count_in_buffer(std::ptr::null(), 1), 0);
            assert_eq!(rs_qf_count_valid_in_buffer(std::ptr::null(), 1), 0);
            assert_eq!(rs_qf_count_on_line(std::ptr::null(), 1, 10), 0);
        }
    }

    #[test]
    fn test_null_safety_text_search() {
        unsafe {
            let pattern = c"test".as_ptr();
            assert!(!rs_qf_entry_text_contains(std::ptr::null(), pattern));
            assert!(!rs_qf_entry_text_contains(
                std::ptr::null_mut(),
                std::ptr::null()
            ));
            assert_eq!(rs_qf_find_text(std::ptr::null(), pattern), 0);
            assert_eq!(rs_qf_find_text(std::ptr::null(), std::ptr::null()), 0);
            assert_eq!(rs_qf_find_text_after(std::ptr::null(), pattern, 0), 0);
            assert_eq!(rs_qf_count_text_matches(std::ptr::null(), pattern), 0);
        }
    }

    #[test]
    fn test_null_safety_validity() {
        unsafe {
            assert!(!rs_qf_has_valid_in_buffer(std::ptr::null(), 1));
            assert!(!rs_qf_all_invalid(std::ptr::null()));
            assert!(!rs_qf_has_valid_type(
                std::ptr::null(),
                entry_types::TYPE_ERROR
            ));
        }
    }

    #[test]
    fn test_null_safety_filter_stats() {
        unsafe {
            let stats = rs_qf_filter_stats(std::ptr::null(), 0);
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
            assert_eq!(rs_qf_find_matching(std::ptr::null(), 0, 0, false), 0);
            assert_eq!(
                rs_qf_find_matching_after(std::ptr::null(), 0, 0, 0, false),
                0
            );
        }
    }

    #[test]
    fn test_null_safety_range() {
        unsafe {
            assert!(!rs_qf_has_entries_in_range(std::ptr::null(), 1, 1, 10));
            assert_eq!(rs_qf_count_in_range(std::ptr::null(), 1, 1, 10), 0);
            assert_eq!(rs_qf_find_in_range(std::ptr::null(), 1, 1, 10), 0);
        }
    }

    #[test]
    fn test_invalid_range() {
        unsafe {
            // End before start should return 0/false
            assert!(!rs_qf_has_entries_in_range(std::ptr::null(), 1, 10, 1));
            assert_eq!(rs_qf_count_in_range(std::ptr::null(), 1, 10, 1), 0);
            assert_eq!(rs_qf_find_in_range(std::ptr::null(), 1, 10, 1), 0);
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
