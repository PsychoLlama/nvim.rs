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

    // List state accessors
    fn nvim_qf_get_nonevalid(qfl: QfListHandle) -> bool;

    // Error message function
    fn nvim_emsg_e_no_more_items();

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
// Phase Q4: Additional Navigation Helpers for :cnext/:cprev/:cfirst/:clast
// =============================================================================

/// Navigate to first entry in the list.
///
/// Updates the list pointer and index. Returns the new index (1) or 0 on failure.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_first(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return 0;
    }

    let first = nvim_qf_get_start(qfl);
    if first.is_null() {
        return 0;
    }

    nvim_qf_set_ptr(qfl, first);
    nvim_qf_set_index(qfl, 1);
    1
}

/// Navigate to last entry in the list.
///
/// Updates the list pointer and index. Returns the new index or 0 on failure.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_last(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return 0;
    }

    let last = nvim_qf_get_last(qfl);
    if last.is_null() {
        return 0;
    }

    nvim_qf_set_ptr(qfl, last);
    nvim_qf_set_index(qfl, count);
    count
}

/// Navigate to a specific entry by index.
///
/// Updates the list pointer and index. Returns the target index or 0 on failure.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_idx(qfl: QfListHandleMut, target_idx: c_int) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 || target_idx < 1 || target_idx > count {
        return 0;
    }

    // Navigate from start to target
    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;
    while !qfp.is_null() && idx < target_idx {
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    if qfp.is_null() {
        return 0;
    }

    nvim_qf_set_ptr(qfl, qfp);
    nvim_qf_set_index(qfl, target_idx);
    target_idx
}

/// Navigate to next valid entry.
///
/// Returns the new index or 0 if no valid entry found forward.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_next_valid(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut qfp = nvim_qf_get_ptr(qfl);
    let mut idx = nvim_qf_get_index(qfl);

    // Move past current entry
    if !qfp.is_null() {
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    // Find next valid
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

/// Navigate to previous valid entry.
///
/// Returns the new index or 0 if no valid entry found backward.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_prev_valid(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut qfp = nvim_qf_get_ptr(qfl);
    let mut idx = nvim_qf_get_index(qfl);

    // Move before current entry
    if !qfp.is_null() {
        qfp = nvim_qfline_get_prev(qfp);
        idx -= 1;
    }

    // Find previous valid
    while !qfp.is_null() && idx > 0 {
        if nvim_qfline_get_valid(qfp) {
            nvim_qf_set_ptr(qfl, qfp);
            nvim_qf_set_index(qfl, idx);
            return idx;
        }
        qfp = nvim_qfline_get_prev(qfp);
        idx -= 1;
    }

    0
}

/// Navigate to first valid entry in the list.
///
/// Returns the new index or 0 if no valid entries exist.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_first_valid_entry(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

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

/// Navigate to last valid entry in the list.
///
/// Returns the new index or 0 if no valid entries exist.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_last_valid_entry(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return 0;
    }

    // Find last valid by scanning from start (we don't have direct reverse iteration)
    let mut last_valid_ptr: QfLineHandle = std::ptr::null();
    let mut last_valid_idx = 0;

    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) {
            last_valid_ptr = qfp;
            last_valid_idx = idx;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    if last_valid_idx > 0 {
        nvim_qf_set_ptr(qfl, last_valid_ptr);
        nvim_qf_set_index(qfl, last_valid_idx);
        last_valid_idx
    } else {
        0
    }
}

/// Count valid entries in the list.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_valid_entry_count(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    count
}

/// Check if current entry is valid.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_current_is_valid(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return false;
    }

    let qfp = nvim_qf_get_ptr(qfl);
    if qfp.is_null() {
        return false;
    }

    nvim_qfline_get_valid(qfp)
}

/// Get position of current entry in valid entries (1-based).
///
/// Returns the position among valid entries, or 0 if current is not valid.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_current_valid_position(qfl: QfListHandle) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let current_idx = nvim_qf_get_index(qfl);
    let current_ptr = nvim_qf_get_ptr(qfl);

    if current_ptr.is_null() || !nvim_qfline_get_valid(current_ptr) {
        return 0;
    }

    // Count valid entries up to and including current
    let mut position = 0;
    let mut qfp = nvim_qf_get_start(qfl);
    let mut idx = 1;

    while !qfp.is_null() && idx <= current_idx {
        if nvim_qfline_get_valid(qfp) {
            position += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
        idx += 1;
    }

    position
}

// =============================================================================
// Phase 9.1: Entry Selection Logic for qf_jump
// =============================================================================

/// Navigation direction constants (matching `vim_defs.h`)
pub const FORWARD: c_int = 1;
pub const BACKWARD: c_int = -1;
pub const FORWARD_FILE: c_int = 3;
pub const BACKWARD_FILE: c_int = -3;

/// Result of getting an entry from the quickfix list.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfGetEntryResult {
    /// Pointer to the entry (null if not found)
    pub qf_ptr: QfLineHandle,
    /// New index (1-based)
    pub qf_index: c_int,
    /// Whether an error message was emitted
    pub errored: bool,
}

impl Default for QfGetEntryResult {
    fn default() -> Self {
        Self {
            qf_ptr: std::ptr::null(),
            qf_index: 0,
            errored: false,
        }
    }
}

/// Get the next valid entry in the quickfix list.
///
/// If `dir` is `FORWARD_FILE`, skip entries in the same file.
///
/// Returns the next valid entry or null if at the end.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `qf_ptr` must be a valid pointer to a `qfline_T` struct
unsafe fn get_next_valid_entry(
    qfl: QfListHandle,
    mut qf_ptr: QfLineHandle,
    qf_index: &mut c_int,
    dir: c_int,
) -> QfLineHandle {
    let count = nvim_qf_get_count(qfl);
    let old_qf_fnum = if qf_ptr.is_null() {
        0
    } else {
        nvim_qfline_get_fnum(qf_ptr)
    };
    let nonevalid = nvim_qf_get_nonevalid(qfl);

    loop {
        if *qf_index == count || qf_ptr.is_null() {
            return std::ptr::null();
        }

        let next = nvim_qfline_get_next(qf_ptr);
        if next.is_null() {
            return std::ptr::null();
        }

        *qf_index += 1;
        qf_ptr = next;

        // Check if this entry is acceptable
        let valid = nvim_qfline_get_valid(qf_ptr);
        let fnum = nvim_qfline_get_fnum(qf_ptr);

        // Continue if:
        // - Entry is not valid (and not in "nonevalid" mode)
        // - OR dir is FORWARD_FILE and we're still in the same file
        if (!nonevalid && !valid) || (dir == FORWARD_FILE && fnum == old_qf_fnum) {
            continue;
        }

        return qf_ptr;
    }
}

/// Get the previous valid entry in the quickfix list.
///
/// If `dir` is `BACKWARD_FILE`, skip entries in the same file.
///
/// Returns the previous valid entry or null if at the beginning.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
/// - `qf_ptr` must be a valid pointer to a `qfline_T` struct
unsafe fn get_prev_valid_entry(
    qfl: QfListHandle,
    mut qf_ptr: QfLineHandle,
    qf_index: &mut c_int,
    dir: c_int,
) -> QfLineHandle {
    let old_qf_fnum = if qf_ptr.is_null() {
        0
    } else {
        nvim_qfline_get_fnum(qf_ptr)
    };
    let nonevalid = nvim_qf_get_nonevalid(qfl);

    loop {
        if *qf_index == 1 || qf_ptr.is_null() {
            return std::ptr::null();
        }

        let prev = nvim_qfline_get_prev(qf_ptr);
        if prev.is_null() {
            return std::ptr::null();
        }

        *qf_index -= 1;
        qf_ptr = prev;

        // Check if this entry is acceptable
        let valid = nvim_qfline_get_valid(qf_ptr);
        let fnum = nvim_qfline_get_fnum(qf_ptr);

        // Continue if:
        // - Entry is not valid (and not in "nonevalid" mode)
        // - OR dir is BACKWARD_FILE and we're still in the same file
        if (!nonevalid && !valid) || (dir == BACKWARD_FILE && fnum == old_qf_fnum) {
            continue;
        }

        return qf_ptr;
    }
}

/// Get the n'th (errornr) previous/next valid entry from the current entry.
///
/// - `dir == FORWARD` or `FORWARD_FILE`: next valid entry
/// - `dir == BACKWARD` or `BACKWARD_FILE`: previous valid entry
///
/// Returns the found entry, or null if not found (with error message emitted).
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
unsafe fn get_nth_valid_entry(
    qfl: QfListHandle,
    mut errornr: c_int,
    dir: c_int,
    new_qfidx: &mut c_int,
) -> (QfLineHandle, bool) {
    let mut qf_ptr = nvim_qf_get_ptr(qfl);
    let mut qf_idx = nvim_qf_get_index(qfl);
    let mut should_emit_error = true;

    while errornr > 0 {
        errornr -= 1;

        let prev_qf_ptr = qf_ptr;
        let prev_index = qf_idx;

        qf_ptr = if dir == FORWARD || dir == FORWARD_FILE {
            get_next_valid_entry(qfl, qf_ptr, &mut qf_idx, dir)
        } else {
            get_prev_valid_entry(qfl, qf_ptr, &mut qf_idx, dir)
        };

        if qf_ptr.is_null() {
            qf_ptr = prev_qf_ptr;
            qf_idx = prev_index;
            if should_emit_error {
                nvim_emsg_e_no_more_items();
                *new_qfidx = qf_idx;
                return (std::ptr::null(), true);
            }
            break;
        }

        should_emit_error = false;
    }

    *new_qfidx = qf_idx;
    (qf_ptr, false)
}

/// Get n'th (errornr) quickfix entry from the current entry.
///
/// Returns the entry at the given index (1-based).
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
unsafe fn get_nth_entry(qfl: QfListHandle, errornr: c_int, new_qfidx: &mut c_int) -> QfLineHandle {
    let mut qf_ptr = nvim_qf_get_ptr(qfl);
    let mut qf_idx = nvim_qf_get_index(qfl);
    let count = nvim_qf_get_count(qfl);

    // New error number is less than the current error number
    while errornr < qf_idx && qf_idx > 1 {
        let prev = nvim_qfline_get_prev(qf_ptr);
        if prev.is_null() {
            break;
        }
        qf_idx -= 1;
        qf_ptr = prev;
    }

    // New error number is greater than the current error number
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

/// Get an entry specified by 'errornr' and 'dir' from the current quickfix/location list.
///
/// This version emits the "No more items" error message when navigation fails,
/// matching the original C behavior of `qf_get_entry`.
///
/// - `errornr` specifies the index of the entry (1-based) or count
/// - `dir` specifies the direction (`FORWARD`/`BACKWARD`/`FORWARD_FILE`/`BACKWARD_FILE`, or 0 for direct index)
///
/// Returns a pointer to the entry and the new index.
///
/// # Safety
///
/// - `qfl` may be null (returns default result)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_entry_with_msg(
    qfl: QfListHandle,
    errornr: c_int,
    dir: c_int,
) -> QfGetEntryResult {
    if qfl.is_null() {
        return QfGetEntryResult::default();
    }

    let mut new_qfidx = 0;
    let (qf_ptr, errored);

    if dir != 0 {
        // next/prev valid entry
        (qf_ptr, errored) = get_nth_valid_entry(qfl, errornr, dir, &mut new_qfidx);
    } else if errornr != 0 {
        // go to specified number
        qf_ptr = get_nth_entry(qfl, errornr, &mut new_qfidx);
        errored = false;
    } else {
        // stay at current entry
        qf_ptr = nvim_qf_get_ptr(qfl);
        new_qfidx = nvim_qf_get_index(qfl);
        errored = false;
    }

    QfGetEntryResult {
        qf_ptr,
        qf_index: new_qfidx,
        errored,
    }
}

// =============================================================================
// qf_jump_edit_buffer migration
// =============================================================================

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
mod jump_edit {
    use std::ffi::{c_int, c_uint, c_void};

    /// Opaque handles — match lib.rs signatures
    type QfInfoHandle = *const c_void;
    type QfInfoHandleMut = *mut c_void;
    type QfListHandle = *const c_void;
    type QfLineHandle = *const c_void;

    const FAIL: c_int = 0;
    const QFLT_QUICKFIX: c_int = 0;
    const QFLT_LOCATION: c_int = 1;
    const QF_ABORT: c_int = 6;

    extern "C" {
        // Existing accessors
        fn nvim_qf_get_curlist(qi: QfInfoHandle) -> QfListHandle;
        fn nvim_qf_get_curlist_idx(qi: QfInfoHandle) -> c_int;
        fn nvim_qf_get_changedtick(qfl: QfListHandle) -> c_int;
        fn nvim_qf_get_id(qfl: QfListHandle) -> c_uint;
        fn nvim_qf_get_qfl_type(qfl: QfListHandle) -> c_int;

        fn nvim_qfline_get_type(qfp: QfLineHandle) -> i8;
        fn nvim_qfline_get_fnum(qfp: QfLineHandle) -> c_int;

        // Validation (already exist)
        fn nvim_qflist_valid(qi: QfInfoHandle, qf_id: c_uint) -> bool;
        fn nvim_qf_entry_present(qfl: QfListHandle, qf_ptr: QfLineHandle) -> bool;

        // New Phase 5 high-level wrappers
        fn nvim_qf_jump_open_help(qf_fnum: c_int, forceit: c_int, prev_winid: c_int) -> c_int;
        fn nvim_qf_jump_open_file(
            qi: QfInfoHandleMut,
            fnum: c_int,
            forceit: c_int,
            opened_window: *mut bool,
        ) -> c_int;
        fn nvim_qf_jump_loc_win_closed(prev_winid: c_int, qi: QfInfoHandleMut) -> bool;
        fn nvim_qf_jump_emsg_win_closed();
        fn nvim_qf_jump_emsg_qf_changed();
        fn nvim_qf_jump_emsg_ll_changed();
    }

    /// Edit the selected file or help file from quickfix.
    ///
    /// # Safety
    ///
    /// All pointer parameters must be valid.
    #[no_mangle]
    pub unsafe extern "C" fn rs_qf_jump_edit_buffer(
        qi: QfInfoHandleMut,
        qf_ptr: QfLineHandle,
        forceit: c_int,
        prev_winid: c_int,
        opened_window: *mut bool,
    ) -> c_int {
        let qfl = nvim_qf_get_curlist(qi);
        let old_changetick = nvim_qf_get_changedtick(qfl);
        let old_qf_curlist = nvim_qf_get_curlist_idx(qi);
        let qfl_type = nvim_qf_get_qfl_type(qfl);
        let save_qfid = nvim_qf_get_id(qfl);

        let retval = if nvim_qfline_get_type(qf_ptr) == 1 {
            // Open help file
            let result = nvim_qf_jump_open_help(nvim_qfline_get_fnum(qf_ptr), forceit, prev_winid);
            if result == -2 {
                // can_abandon failed: skip post-validation
                return FAIL;
            }
            result
        } else {
            // Open normal file (handles winfixbuf logic)
            let fnum = nvim_qfline_get_fnum(qf_ptr);
            let result = nvim_qf_jump_open_file(qi, fnum, forceit, opened_window);
            if result == -2 {
                // Location list winfixbuf early return (skip post-validation)
                return FAIL;
            }
            result
        };

        // If a location list, check whether the associated window is still present.
        if qfl_type == QFLT_LOCATION && nvim_qf_jump_loc_win_closed(prev_winid, qi) {
            nvim_qf_jump_emsg_win_closed();
            *opened_window = false;
            return QF_ABORT;
        }

        // Check if the quickfix list is still valid.
        if qfl_type == QFLT_QUICKFIX && !nvim_qflist_valid(qi, save_qfid) {
            nvim_qf_jump_emsg_qf_changed();
            return QF_ABORT;
        }

        // Check if the list was changed by autocommands.
        if old_qf_curlist != nvim_qf_get_curlist_idx(qi)
            || old_changetick != nvim_qf_get_changedtick(qfl)
            || !nvim_qf_entry_present(qfl, qf_ptr)
        {
            if qfl_type == QFLT_QUICKFIX {
                nvim_qf_jump_emsg_qf_changed();
            } else {
                nvim_qf_jump_emsg_ll_changed();
            }
            return QF_ABORT;
        }

        retval
    }
}

// =============================================================================
// Jump machinery migration
// =============================================================================

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
mod jump_machinery {
    use std::ffi::{c_int, c_void};
    use std::ptr;

    /// Opaque handles
    type QfInfoHandleMut = *mut c_void;
    type WinHandle = *mut c_void;

    const OK: c_int = 1;
    const FAIL: c_int = 0;

    // WSP_ flags matching src/nvim/window.h
    const WSP_HELP: c_int = 0x20;
    const WSP_TOP: c_int = 0x08;
    const WSP_NEWLOC: c_int = 0x100;

    #[allow(dead_code)]
    extern "C" {
        // Window-finding wrappers (Phase 1)
        fn nvim_qf_find_help_win() -> WinHandle;
        fn nvim_qf_find_win_with_loclist(ll: *const c_void) -> WinHandle;
        fn nvim_qf_find_win_with_normal_buf() -> WinHandle;
        fn nvim_qf_goto_tabwin_with_file(fnum: c_int) -> bool;
        fn nvim_qf_open_new_file_win(ll_ref: *mut c_void) -> c_int;

        // Window/buffer state accessors (Phase 1)
        fn nvim_qf_curwin_get_llist_ref() -> *mut c_void;
        fn nvim_qf_curbuf_is_quickfix() -> bool;
        fn nvim_qf_curwin_buf_is_help() -> bool;
        fn nvim_qf_get_cmdmod_tab() -> c_int;
        fn nvim_qf_is_one_window() -> bool;
        fn nvim_qf_swb_has_usetab() -> bool;

        // Window operations (Phase 2)
        fn nvim_qf_win_goto(win: WinHandle);
        fn nvim_qf_win_enter(win: WinHandle);
        fn nvim_qf_win_buf_nwindows(win: *const c_void) -> c_int;
        fn nvim_qf_win_buf_fnum(win: *const c_void) -> c_int;
        fn nvim_qf_win_get_llist(win: *const c_void) -> *mut c_void;
        fn nvim_qf_win_set_loclist(win: WinHandle, qi: *mut c_void);
        fn nvim_qf_get_cmdmod_split() -> c_int;
        fn nvim_qf_curwin_width() -> c_int;
        fn nvim_qf_get_columns() -> c_int;
        fn nvim_qf_curwin_height() -> c_int;
        fn nvim_qf_get_p_hh() -> c_int;
        fn nvim_qf_win_split(size: c_int, flags: c_int) -> c_int;
        fn nvim_qf_win_setheight(height: c_int);
        fn nvim_qf_clear_restart_edit();
        fn nvim_qf_is_ll_stack_qi(qi: *const c_void) -> bool;
        fn nvim_qf_win_is_qf_window(win: *const c_void) -> bool;
        fn nvim_qf_win_prev(win: *const c_void) -> WinHandle;
        fn nvim_qf_win_next(win: *const c_void) -> WinHandle;
        fn nvim_qf_get_lastwin() -> WinHandle;
        fn nvim_qf_get_curwin() -> WinHandle;
        fn nvim_qf_win_bt_normal(win: *const c_void) -> bool;
        fn nvim_qf_swb_uselast_prevwin_ok() -> bool;
        fn nvim_qf_get_prevwin() -> WinHandle;
        fn nvim_qf_win_is_preview(win: *const c_void) -> bool;
        fn nvim_qf_win_is_wfb(win: *const c_void) -> bool;
    }

    /// Find or open a help window. Returns OK or FAIL.
    /// Sets `*opened_window = true` if a new window was split.
    ///
    /// # Safety
    ///
    /// All pointer parameters must be valid.
    #[no_mangle]
    pub unsafe extern "C" fn rs_qf_jump_to_help_window(
        qi: QfInfoHandleMut,
        newwin: bool,
        opened_window: *mut bool,
    ) -> c_int {
        let wp = if nvim_qf_get_cmdmod_tab() != 0 || newwin {
            ptr::null_mut()
        } else {
            nvim_qf_find_help_win()
        };

        if !wp.is_null() && nvim_qf_win_buf_nwindows(wp) > 0 {
            nvim_qf_win_enter(wp);
        } else {
            // Split off help window; put it at far top if no position
            // specified, the current window is vertically split and narrow.
            let mut flags = WSP_HELP;
            if nvim_qf_get_cmdmod_split() == 0
                && nvim_qf_curwin_width() != nvim_qf_get_columns()
                && nvim_qf_curwin_width() < 80
            {
                flags |= WSP_TOP;
            }

            // If the user asks to open a new window, then copy the location list.
            // Otherwise, don't copy the location list.
            if nvim_qf_is_ll_stack_qi(qi) && !newwin {
                flags |= WSP_NEWLOC;
            }

            if nvim_qf_win_split(0, flags) == FAIL {
                return FAIL;
            }

            *opened_window = true;

            if nvim_qf_curwin_height() < nvim_qf_get_p_hh() {
                nvim_qf_win_setheight(nvim_qf_get_p_hh());
            }

            // When using location list, the new window should use the supplied
            // location list.
            if nvim_qf_is_ll_stack_qi(qi) && !newwin {
                nvim_qf_win_set_loclist(nvim_qf_get_curwin(), qi);
            }
        }

        nvim_qf_clear_restart_edit(); // don't want insert mode in help file

        OK
    }

    /// Navigate to a window showing the given file, for location list context.
    /// `use_win` may be NULL.
    ///
    /// # Safety
    ///
    /// All pointer parameters must be valid (or null where noted).
    #[no_mangle]
    pub unsafe extern "C" fn rs_qf_goto_win_with_ll_file(
        use_win: WinHandle,
        qf_fnum: c_int,
        ll_ref: QfInfoHandleMut,
    ) {
        let mut win = use_win;

        if win.is_null() {
            // Find the window showing the selected file in the current tab page.
            // We use the C wrapper that iterates FOR_ALL_WINDOWS_IN_TAB.
            // We need to walk windows manually via w_next from firstwin.
            // Actually, we don't have a firstwin accessor. Let's use
            // nvim_qf_get_curwin() and walk via w_prev/w_next.
            // But that won't iterate all windows. We need a different approach.
            //
            // Use nvim_qf_find_win_with_loclist won't work here either because
            // we need to find by buffer fnum, not by loclist.
            // Let's walk from curwin backwards like the original C code.

            // First try: walk all windows to find one showing the file.
            // We walk backwards from curwin (like original C code does for the
            // fallback path), but first we need to try all windows.
            // Walk from lastwin backwards to find the file.
            let mut w = nvim_qf_get_lastwin();
            while !w.is_null() {
                if nvim_qf_win_buf_fnum(w) == qf_fnum {
                    win = w;
                    break;
                }
                w = nvim_qf_win_prev(w);
            }

            if win.is_null() {
                // Find a previous usable window (walk backwards from curwin)
                win = nvim_qf_get_curwin();
                loop {
                    if nvim_qf_win_bt_normal(win) {
                        break;
                    }
                    let prev = nvim_qf_win_prev(win);
                    if prev.is_null() {
                        win = nvim_qf_get_lastwin(); // wrap around the top
                    } else {
                        win = prev; // go to previous window
                    }
                    if win == nvim_qf_get_curwin() {
                        break;
                    }
                }
            }
        }
        nvim_qf_win_goto(win);

        // If the location list for the window is not set, then set it
        // to the location list from the location window
        if nvim_qf_win_get_llist(win).is_null() && !ll_ref.is_null() {
            nvim_qf_win_set_loclist(win, ll_ref);
        }
    }

    /// Navigate to a window showing the given file, for quickfix context.
    ///
    /// # Safety
    ///
    /// Globals must be in a valid state.
    #[no_mangle]
    pub unsafe extern "C" fn rs_qf_goto_win_with_qfl_file(qf_fnum: c_int) {
        let mut win = nvim_qf_get_curwin();
        let mut altwin: WinHandle = ptr::null_mut();

        loop {
            if nvim_qf_win_buf_fnum(win) == qf_fnum {
                break;
            }
            let prev = nvim_qf_win_prev(win);
            if prev.is_null() {
                win = nvim_qf_get_lastwin(); // wrap around the top
            } else {
                win = prev; // go to previous window
            }

            if nvim_qf_win_is_qf_window(win) {
                // Didn't find it, go to the window before the quickfix
                // window, unless 'switchbuf' contains 'uselast': in this case we
                // try to jump to the previously used window first.
                if nvim_qf_swb_uselast_prevwin_ok() {
                    win = nvim_qf_get_prevwin();
                } else if !altwin.is_null() {
                    win = altwin;
                } else {
                    let cur_prev = nvim_qf_win_prev(nvim_qf_get_curwin());
                    if cur_prev.is_null() {
                        win = nvim_qf_win_next(nvim_qf_get_curwin());
                    } else {
                        win = cur_prev;
                    }
                }
                break;
            }

            // Remember a usable window.
            if altwin.is_null()
                && !nvim_qf_win_is_preview(win)
                && !nvim_qf_win_is_wfb(win)
                && nvim_qf_win_bt_normal(win)
            {
                altwin = win;
            }
        }

        nvim_qf_win_goto(win);
    }

    /// Find a usable window and jump to it. Returns OK or FAIL.
    /// Sets `*opened_window = true` if a new window was opened.
    ///
    /// # Safety
    ///
    /// All pointer parameters must be valid.
    #[no_mangle]
    pub unsafe extern "C" fn rs_qf_jump_to_usable_window(
        qf_fnum: c_int,
        newwin: bool,
        opened_window: *mut bool,
    ) -> c_int {
        let mut usable_win = false;
        let mut usable_wp: WinHandle = ptr::null_mut();

        // If opening a new window, then don't use the location list referred by
        // the current window. Otherwise two windows will refer to the same
        // location list.
        let ll_ref = if newwin {
            ptr::null_mut()
        } else {
            nvim_qf_curwin_get_llist_ref()
        };

        if !ll_ref.is_null() {
            // Find a non-quickfix window with this location list
            usable_wp = nvim_qf_find_win_with_loclist(ll_ref);
            if !usable_wp.is_null() {
                usable_win = true;
            }
        }

        if !usable_win {
            // Locate a window showing a normal buffer
            let win = nvim_qf_find_win_with_normal_buf();
            if !win.is_null() {
                usable_win = true;
            }
        }

        // If no usable window is found and 'switchbuf' contains "usetab"
        // then search in other tabs.
        if !usable_win && nvim_qf_swb_has_usetab() {
            usable_win = nvim_qf_goto_tabwin_with_file(qf_fnum);
        }

        // If there is only one window and it is the quickfix window, create a
        // new one above the quickfix window.
        if (nvim_qf_is_one_window() && nvim_qf_curbuf_is_quickfix()) || !usable_win || newwin {
            if nvim_qf_open_new_file_win(ll_ref) != OK {
                return FAIL;
            }
            *opened_window = true; // close it when fail
        } else if !nvim_qf_curwin_get_llist_ref().is_null() {
            // In a location window
            rs_qf_goto_win_with_ll_file(usable_wp, qf_fnum, ll_ref);
        } else {
            // In a quickfix window
            rs_qf_goto_win_with_qfl_file(qf_fnum);
        }

        OK
    }

    // Phase 3 extern declarations
    type QfLineHandle = *const c_void;
    type BufHandle = *mut c_void;
    /// Line number type (matches `linenr_T` in Neovim)
    type LinenrT = i32;

    #[allow(clashing_extern_declarations)]
    extern "C" {
        // Phase 4 accessors (for qf_jump_open_window)
        fn nvim_qf_get_curlist(qi: *const c_void) -> *const c_void;
        fn nvim_qf_get_curlist_idx(qi: *const c_void) -> c_int;
        fn nvim_qf_get_changedtick(qfl: *const c_void) -> c_int;
        fn nvim_qf_get_qfl_type(qfl: *const c_void) -> c_int;
        fn nvim_qfline_get_type(qfp: QfLineHandle) -> i8;
        fn nvim_qf_entry_present(qfl: *const c_void, qf_ptr: QfLineHandle) -> bool;
        fn nvim_qf_jump_emsg_qf_changed();
        fn nvim_qf_jump_emsg_ll_changed();

        // Phase 3 wrappers
        fn nvim_qf_jump_goto_line(
            qf_lnum: LinenrT,
            qf_col: c_int,
            qf_viscol: i8,
            qf_pattern: *const i8,
        );
        fn nvim_qf_jump_print_msg(
            qi: QfInfoHandleMut,
            qf_index: c_int,
            qf_ptr: QfLineHandle,
            old_curbuf: BufHandle,
            old_lnum: LinenrT,
        );
        fn nvim_qf_get_curbuf() -> BufHandle;
        fn nvim_qf_fdo_quickfix() -> bool;
        fn nvim_qf_fold_open_cursor();
        fn nvim_qf_setpcmark();
        fn nvim_qf_curbuf_is(buf: *const c_void) -> bool;

        // Entry accessors (already exist)
        fn nvim_qfline_get_fnum(qfp: QfLineHandle) -> c_int;
        fn nvim_qfline_get_lnum(qfp: QfLineHandle) -> LinenrT;
        fn nvim_qfline_get_col(qfp: QfLineHandle) -> c_int;
        fn nvim_qfline_get_viscol(qfp: QfLineHandle) -> bool;
        fn nvim_qfline_get_pattern(qfp: QfLineHandle) -> *const i8;

        // Existing accessor
        fn nvim_qf_get_cursor_lnum() -> LinenrT;

        // Edit buffer (from jump_edit module)
        fn rs_qf_jump_edit_buffer(
            qi: QfInfoHandleMut,
            qf_ptr: QfLineHandle,
            forceit: c_int,
            prev_winid: c_int,
            opened_window: *mut bool,
        ) -> c_int;
    }

    /// Edit buffer, position cursor, open folds, and print message.
    /// Returns OK, FAIL, or `QF_ABORT`.
    ///
    /// # Safety
    ///
    /// All pointer parameters must be valid.
    #[no_mangle]
    pub unsafe extern "C" fn rs_qf_jump_to_buffer(
        qi: QfInfoHandleMut,
        qf_index: c_int,
        qf_ptr: QfLineHandle,
        forceit: c_int,
        prev_winid: c_int,
        opened_window: *mut bool,
        openfold: c_int,
        print_message: bool,
    ) -> c_int {
        // Save old state before buffer operations
        let old_curbuf = nvim_qf_get_curbuf();
        let old_lnum = nvim_qf_get_cursor_lnum();
        let mut retval = OK;

        // If there is a file name, read the wanted file if needed, and check
        // autowrite etc.
        if nvim_qfline_get_fnum(qf_ptr) != 0 {
            retval = rs_qf_jump_edit_buffer(qi, qf_ptr, forceit, prev_winid, opened_window);
            if retval != OK {
                return retval;
            }
        }

        // When not switched to another buffer, still need to set pc mark
        if nvim_qf_curbuf_is(old_curbuf) {
            nvim_qf_setpcmark();
        }

        nvim_qf_jump_goto_line(
            nvim_qfline_get_lnum(qf_ptr),
            nvim_qfline_get_col(qf_ptr),
            i8::from(nvim_qfline_get_viscol(qf_ptr)),
            nvim_qfline_get_pattern(qf_ptr),
        );

        if nvim_qf_fdo_quickfix() && openfold != 0 {
            nvim_qf_fold_open_cursor();
        }
        if print_message {
            nvim_qf_jump_print_msg(qi, qf_index, qf_ptr, old_curbuf, old_lnum);
        }

        retval
    }

    const QFLT_QUICKFIX: c_int = 0;
    const NOTDONE: c_int = 2;
    const QF_ABORT: c_int = 6;

    /// Check if the quickfix/location list was changed by an autocmd.
    /// Returns `Some(QF_ABORT)` if changed, `None` if unchanged.
    unsafe fn check_list_changed(
        qi: QfInfoHandleMut,
        qfl: *const c_void,
        old_changetick: c_int,
        old_qf_curlist: c_int,
        qf_ptr: QfLineHandle,
        qfl_type: c_int,
    ) -> Option<c_int> {
        if old_qf_curlist != nvim_qf_get_curlist_idx(qi)
            || old_changetick != nvim_qf_get_changedtick(qfl)
            || !nvim_qf_entry_present(qfl, qf_ptr)
        {
            if qfl_type == QFLT_QUICKFIX {
                nvim_qf_jump_emsg_qf_changed();
            } else {
                nvim_qf_jump_emsg_ll_changed();
            }
            Some(QF_ABORT)
        } else {
            None
        }
    }

    /// Open a window for the quickfix jump target.
    /// Returns OK, FAIL, NOTDONE, or `QF_ABORT`.
    ///
    /// # Safety
    ///
    /// All pointer parameters must be valid.
    #[no_mangle]
    pub unsafe extern "C" fn rs_qf_jump_open_window(
        qi: QfInfoHandleMut,
        qf_ptr: QfLineHandle,
        newwin: bool,
        opened_window: *mut bool,
    ) -> c_int {
        let qfl = nvim_qf_get_curlist(qi);
        let old_changetick = nvim_qf_get_changedtick(qfl);
        let old_qf_curlist = nvim_qf_get_curlist_idx(qi);
        let qfl_type = nvim_qf_get_qfl_type(qfl);

        // For ":helpgrep" find a help window or open one.
        if nvim_qfline_get_type(qf_ptr) == 1
            && (!nvim_qf_curwin_buf_is_help() || nvim_qf_get_cmdmod_tab() != 0)
            && rs_qf_jump_to_help_window(qi, newwin, opened_window) == FAIL
        {
            return FAIL;
        }

        // Check if list was changed by autocmds during help window opening
        if let Some(abort) =
            check_list_changed(qi, qfl, old_changetick, old_qf_curlist, qf_ptr, qfl_type)
        {
            return abort;
        }

        // If currently in the quickfix window, find another window to show the
        // file in.
        if nvim_qf_curbuf_is_quickfix() && !*opened_window {
            // If there is no file specified, we don't know where to go.
            // But do advance, otherwise ":cn" gets stuck.
            if nvim_qfline_get_fnum(qf_ptr) == 0 {
                return NOTDONE;
            }

            if rs_qf_jump_to_usable_window(nvim_qfline_get_fnum(qf_ptr), newwin, opened_window)
                == FAIL
            {
                return FAIL;
            }
        }

        // Second check if list was changed by autocmds
        if let Some(abort) =
            check_list_changed(qi, qfl, old_changetick, old_qf_curlist, qf_ptr, qfl_type)
        {
            return abort;
        }

        OK
    }
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

    // Phase Q4: Tests for additional navigation helpers
    #[test]
    fn test_null_goto_first() {
        unsafe {
            assert_eq!(rs_qf_goto_first(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_goto_last() {
        unsafe {
            assert_eq!(rs_qf_goto_last(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_goto_idx() {
        unsafe {
            assert_eq!(rs_qf_goto_idx(std::ptr::null_mut(), 1), 0);
        }
    }

    #[test]
    fn test_null_goto_next_valid() {
        unsafe {
            assert_eq!(rs_qf_goto_next_valid(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_goto_prev_valid() {
        unsafe {
            assert_eq!(rs_qf_goto_prev_valid(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_goto_first_valid_entry() {
        unsafe {
            assert_eq!(rs_qf_goto_first_valid_entry(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_goto_last_valid_entry() {
        unsafe {
            assert_eq!(rs_qf_goto_last_valid_entry(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_valid_entry_count() {
        unsafe {
            assert_eq!(rs_qf_valid_entry_count(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_current_is_valid() {
        unsafe {
            assert!(!rs_qf_current_is_valid(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_current_valid_position() {
        unsafe {
            assert_eq!(rs_qf_current_valid_position(std::ptr::null()), 0);
        }
    }

    // Phase 9.1: Tests for qf_get_entry
    #[test]
    fn test_null_qf_get_entry_with_msg() {
        unsafe {
            let result = rs_qf_get_entry_with_msg(std::ptr::null(), 1, 0);
            assert!(result.qf_ptr.is_null());
            assert_eq!(result.qf_index, 0);
            assert!(!result.errored);
        }
    }

    #[test]
    fn test_qf_get_entry_result_default() {
        let result = QfGetEntryResult::default();
        assert!(result.qf_ptr.is_null());
        assert_eq!(result.qf_index, 0);
        assert!(!result.errored);
    }

    #[test]
    fn test_direction_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
        assert_eq!(FORWARD_FILE, 3);
        assert_eq!(BACKWARD_FILE, -3);
    }
}
