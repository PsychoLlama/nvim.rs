//! Navigation and positioning for quickfix entries.
//!
//! This module provides functions for navigating between quickfix entries,
//! finding entries by various criteria, and calculating jump targets.

use std::ffi::{c_int, c_void};

/// Line number type (matches `linenr_T` in Neovim)
type LinenrT = i32;

/// Opaque handle to `qf_list_T` (quickfix list)
type QfListHandle = *const c_void;
type QfListHandleMut = *mut c_void;

/// Opaque handle to `qfline_T` (quickfix entry)
type QfLineHandle = *const c_void;

/// Opaque handle to `pos_T` (position)
type PosHandle = *const c_void;

// =============================================================================
// External C accessor functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // List accessors
    fn nvim_qf_get_count(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_index(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_start(qfl: QfListHandle) -> QfLineHandle;
    fn nvim_qf_get_ptr(qfl: QfListHandle) -> QfLineHandle;
    fn nvim_qf_get_last(qfl: QfListHandle) -> QfLineHandle;

    fn nvim_qf_set_index(qfl: QfListHandleMut, idx: c_int);
    fn nvim_qf_set_ptr(qfl: QfListHandleMut, ptr: QfLineHandle);

    // Entry accessors
    fn nvim_qfline_get_next(qfp: QfLineHandle) -> QfLineHandle;
    fn nvim_qfline_get_prev(qfp: QfLineHandle) -> QfLineHandle;
    fn nvim_qfline_get_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_col(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_end_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_end_col(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_fnum(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_valid(qfp: QfLineHandle) -> bool;

    // Position accessors
    fn nvim_pos_get_lnum(pos: PosHandle) -> LinenrT;
    fn nvim_pos_get_col(pos: PosHandle) -> c_int;
}

// =============================================================================
// Jump Target Calculation
// =============================================================================

/// Result of calculating a jump target.
#[repr(C)]
#[derive(Default)]
pub struct QfJumpTarget {
    /// Entry index to jump to (1-based), or 0 if none found
    pub entry_idx: c_int,
    /// Line number of target entry
    pub lnum: LinenrT,
    /// Column of target entry
    pub col: c_int,
    /// Buffer number of target entry
    pub fnum: c_int,
    /// Whether the target is valid
    pub valid: bool,
}

/// Find the Nth valid entry from the current position.
///
/// `n` is positive to move forward, negative to move backward.
/// Returns the index (1-based) of the found entry, or 0 if not found.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_find_nth_valid(qfl: QfListHandle, n: c_int) -> c_int {
    if qfl.is_null() || n == 0 {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return 0;
    }

    let start_idx = nvim_qf_get_index(qfl);
    let forward = n > 0;
    #[allow(clippy::cast_possible_wrap)]
    let mut remaining = n.unsigned_abs() as c_int;

    // Get current entry pointer
    let mut qfp = nvim_qf_get_ptr(qfl);
    let mut idx = start_idx;

    while remaining > 0 && !qfp.is_null() {
        // Move to next/prev entry
        qfp = if forward {
            idx += 1;
            nvim_qfline_get_next(qfp)
        } else {
            idx -= 1;
            nvim_qfline_get_prev(qfp)
        };

        if qfp.is_null() {
            break;
        }

        if nvim_qfline_get_valid(qfp) {
            remaining -= 1;
            if remaining == 0 {
                return idx;
            }
        }
    }

    0 // Not found
}

/// Calculate the jump target for a specific entry index.
///
/// # Safety
///
/// - `qfl` may be null (returns default target)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_calc_jump_target(qfl: QfListHandle, idx: c_int) -> QfJumpTarget {
    if qfl.is_null() {
        return QfJumpTarget::default();
    }

    let count = nvim_qf_get_count(qfl);
    if idx < 1 || idx > count {
        return QfJumpTarget::default();
    }

    // Navigate to the entry
    let mut qfp = nvim_qf_get_start(qfl);
    let mut current_idx = 1;
    while !qfp.is_null() && current_idx < idx {
        qfp = nvim_qfline_get_next(qfp);
        current_idx += 1;
    }

    if qfp.is_null() {
        return QfJumpTarget::default();
    }

    QfJumpTarget {
        entry_idx: idx,
        lnum: nvim_qfline_get_lnum(qfp),
        col: nvim_qfline_get_col(qfp),
        fnum: nvim_qfline_get_fnum(qfp),
        valid: nvim_qfline_get_valid(qfp),
    }
}

/// Find the entry index for a given line number in the current buffer.
///
/// Returns the index (1-based) of the first entry matching the line,
/// or 0 if not found.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_idx_for_lnum(qfl: QfListHandle, bnr: c_int, lnum: LinenrT) -> c_int {
    if qfl.is_null() || lnum <= 0 {
        return 0;
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() {
        let file_num = nvim_qfline_get_fnum(qfp);
        let line_num = nvim_qfline_get_lnum(qfp);

        if file_num == bnr && line_num == lnum {
            return idx;
        }

        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    0
}

// =============================================================================
// Position-based Entry Finding
// =============================================================================

/// Find the closest entry to a given position.
///
/// Returns the index (1-based) of the closest entry, or 0 if not found.
/// `direction`: 0 = closest, 1 = forward only, -1 = backward only
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - `pos` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_closest_entry(
    qfl: QfListHandle,
    bnr: c_int,
    pos: PosHandle,
    direction: c_int,
) -> c_int {
    if qfl.is_null() || pos.is_null() {
        return 0;
    }

    let pos_lnum = nvim_pos_get_lnum(pos);
    let pos_col = nvim_pos_get_col(pos);

    let mut best_idx = 0;
    let mut best_distance = i64::MAX;

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() {
        let fnum = nvim_qfline_get_fnum(qfp);

        if fnum == bnr && nvim_qfline_get_valid(qfp) {
            let lnum = nvim_qfline_get_lnum(qfp);
            let col = nvim_qfline_get_col(qfp);

            // Check direction constraint
            let is_after = lnum > pos_lnum || (lnum == pos_lnum && col > pos_col);
            let is_before = lnum < pos_lnum || (lnum == pos_lnum && col < pos_col);

            let ok_direction = match direction {
                1 => is_after,   // forward only
                -1 => is_before, // backward only
                _ => true,       // any direction
            };

            if ok_direction {
                // Calculate distance (line distance * 1000 + col distance)
                let line_dist = (i64::from(lnum) - i64::from(pos_lnum)).abs();
                let col_dist = (i64::from(col) - i64::from(pos_col)).abs();
                let distance = line_dist * 10000 + col_dist;

                if distance < best_distance {
                    best_distance = distance;
                    best_idx = idx;
                }
            }
        }

        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    best_idx
}

/// Find the first entry after a given position.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - `pos` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_after_pos_idx(
    qfl: QfListHandle,
    bnr: c_int,
    pos: PosHandle,
) -> c_int {
    rs_qf_closest_entry(qfl, bnr, pos, 1)
}

/// Find the first entry before a given position.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - `pos` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_before_pos_idx(
    qfl: QfListHandle,
    bnr: c_int,
    pos: PosHandle,
) -> c_int {
    rs_qf_closest_entry(qfl, bnr, pos, -1)
}

// =============================================================================
// File-based Navigation
// =============================================================================

/// Find the first entry in a specific file.
///
/// Returns the entry index (1-based) or 0 if not found.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_first_entry_in_file(qfl: QfListHandle, bnr: c_int) -> c_int {
    if qfl.is_null() || bnr <= 0 {
        return 0;
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) == bnr {
            return idx;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    0
}

/// Find the last entry in a specific file.
///
/// Returns the entry index (1-based) or 0 if not found.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_last_entry_in_file(qfl: QfListHandle, bnr: c_int) -> c_int {
    if qfl.is_null() || bnr <= 0 {
        return 0;
    }

    let mut last_idx = 0;
    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) == bnr {
            last_idx = idx;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    last_idx
}

/// Count the number of files referenced in the list.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_unique_file_count(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut last_fnum = -1;

    let mut qfp = nvim_qf_get_start(qfl);
    while !qfp.is_null() {
        let fnum = nvim_qfline_get_fnum(qfp);
        if fnum > 0 && fnum != last_fnum {
            count += 1;
            last_fnum = fnum;
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    count
}

/// Get the file number (buffer) of the Nth unique file in the list.
///
/// `n` is 1-based. Returns 0 if not found.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_nth_file_fnum(qfl: QfListHandle, n: c_int) -> c_int {
    if qfl.is_null() || n <= 0 {
        return 0;
    }

    let mut file_count = 0;
    let mut last_fnum = -1;

    let mut qfp = nvim_qf_get_start(qfl);
    while !qfp.is_null() {
        let fnum = nvim_qfline_get_fnum(qfp);
        if fnum > 0 && fnum != last_fnum {
            file_count += 1;
            last_fnum = fnum;
            if file_count == n {
                return fnum;
            }
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    0
}

// =============================================================================
// Entry Movement
// =============================================================================

/// Move the current entry by a relative offset.
///
/// Returns the new entry index or 0 if movement failed.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_move_entry(qfl: QfListHandleMut, offset: c_int) -> c_int {
    if qfl.is_null() || offset == 0 {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return 0;
    }

    let current_idx = nvim_qf_get_index(qfl);
    let new_idx = (current_idx + offset).clamp(1, count);

    if new_idx == current_idx {
        return current_idx;
    }

    // Navigate to new entry
    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;
    while !qfp.is_null() && idx < new_idx {
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    if qfp.is_null() {
        return 0;
    }

    nvim_qf_set_ptr(qfl, qfp);
    nvim_qf_set_index(qfl, new_idx);
    new_idx
}

/// Move to the first valid entry at or after the current position.
///
/// Returns the new entry index or 0 if no valid entry found.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_move_to_valid(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut qfp = nvim_qf_get_ptr(qfl);
    let mut idx = nvim_qf_get_index(qfl);

    // Check if current is valid
    if !qfp.is_null() && nvim_qfline_get_valid(qfp) {
        return idx;
    }

    // Look forward for valid entry
    while !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) {
            nvim_qf_set_ptr(qfl, qfp);
            nvim_qf_set_index(qfl, idx);
            return idx;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    0
}

// =============================================================================
// Range Navigation
// =============================================================================

/// Find entries within a line range.
///
/// Returns the count of entries in the range.
/// If `first_idx` is not null, stores the first entry index.
/// If `last_idx` is not null, stores the last entry index.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - `first_idx` and `last_idx` may be null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entries_in_range(
    qfl: QfListHandle,
    bnr: c_int,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
    first_idx: *mut c_int,
    last_idx: *mut c_int,
) -> c_int {
    if qfl.is_null() || start_lnum > end_lnum {
        return 0;
    }

    let mut count = 0;
    let mut first = 0;
    let mut last = 0;

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() {
        let fnum = nvim_qfline_get_fnum(qfp);
        let lnum = nvim_qfline_get_lnum(qfp);

        if fnum == bnr && lnum >= start_lnum && lnum <= end_lnum {
            count += 1;
            if first == 0 {
                first = idx;
            }
            last = idx;
        }

        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    if !first_idx.is_null() {
        *first_idx = first;
    }
    if !last_idx.is_null() {
        *last_idx = last;
    }

    count
}

/// Check if an entry's position overlaps with a line range.
///
/// # Safety
///
/// - `qfp` may be null (returns false)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_overlaps_range(
    qfp: QfLineHandle,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
) -> bool {
    if qfp.is_null() {
        return false;
    }

    let entry_start = nvim_qfline_get_lnum(qfp);
    let entry_end = nvim_qfline_get_end_lnum(qfp);
    let entry_end = if entry_end > 0 {
        entry_end
    } else {
        entry_start
    };

    // Ranges overlap if neither is completely before or after the other
    entry_start <= end_lnum && entry_end >= start_lnum
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_find_nth_valid() {
        unsafe {
            assert_eq!(rs_qf_find_nth_valid(std::ptr::null(), 1), 0);
        }
    }

    #[test]
    fn test_null_calc_jump_target() {
        unsafe {
            let target = rs_qf_calc_jump_target(std::ptr::null(), 1);
            assert!(!target.valid);
            assert_eq!(target.entry_idx, 0);
        }
    }

    #[test]
    fn test_null_idx_for_lnum() {
        unsafe {
            assert_eq!(rs_qf_idx_for_lnum(std::ptr::null(), 1, 10), 0);
        }
    }

    #[test]
    fn test_null_closest_entry() {
        unsafe {
            assert_eq!(
                rs_qf_closest_entry(std::ptr::null(), 1, std::ptr::null(), 0),
                0
            );
        }
    }

    #[test]
    fn test_null_first_entry_in_file() {
        unsafe {
            assert_eq!(rs_qf_first_entry_in_file(std::ptr::null(), 1), 0);
        }
    }

    #[test]
    fn test_null_last_entry_in_file() {
        unsafe {
            assert_eq!(rs_qf_last_entry_in_file(std::ptr::null(), 1), 0);
        }
    }

    #[test]
    fn test_null_unique_file_count() {
        unsafe {
            assert_eq!(rs_qf_unique_file_count(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_nth_file_fnum() {
        unsafe {
            assert_eq!(rs_qf_nth_file_fnum(std::ptr::null(), 1), 0);
        }
    }

    #[test]
    fn test_null_move_entry() {
        unsafe {
            assert_eq!(rs_qf_move_entry(std::ptr::null_mut(), 1), 0);
        }
    }

    #[test]
    fn test_null_move_to_valid() {
        unsafe {
            assert_eq!(rs_qf_move_to_valid(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_entries_in_range() {
        unsafe {
            assert_eq!(
                rs_qf_entries_in_range(
                    std::ptr::null(),
                    1,
                    1,
                    10,
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
        }
    }

    #[test]
    fn test_null_entry_overlaps_range() {
        unsafe {
            assert!(!rs_qf_entry_overlaps_range(std::ptr::null(), 1, 10));
        }
    }
}
