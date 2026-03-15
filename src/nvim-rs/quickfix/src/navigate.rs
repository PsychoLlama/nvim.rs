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

    fn emsg(msg: *const std::ffi::c_char) -> bool;
    // (nvim_emsg_e_no_more_items deleted: use emsg directly)

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
                emsg(c"E553: No more items".as_ptr());
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

    const OK: c_int = 1;
    const FAIL: c_int = 0;
    const QFLT_QUICKFIX: c_int = 0;
    const QFLT_LOCATION: c_int = 1;
    const QF_ABORT: c_int = 6;
    // buflist_getfile flags (from buffer.h)
    const GETF_SETMARK: c_int = 0x01;
    const GETF_SWITCH: c_int = 0x04;

    extern "C" {
        // Existing accessors
        fn nvim_qf_get_curlist(qi: QfInfoHandle) -> QfListHandle;
        fn nvim_qf_get_curlist_idx(qi: QfInfoHandle) -> c_int;
        fn nvim_qf_get_changedtick(qfl: QfListHandle) -> c_int;
        fn nvim_qf_get_id(qfl: QfListHandle) -> c_uint;
        fn nvim_qf_get_qfl_type(qfl: QfListHandle) -> c_int;

        fn nvim_qfline_get_type(qfp: QfLineHandle) -> i8;
        fn nvim_qfline_get_fnum(qfp: QfLineHandle) -> c_int;

        // nvim_qflist_valid removed: use crate::qflist_valid_for_qi (Phase 16)
        // nvim_qf_entry_present removed: use crate::rs_qf_entry_present (Phase 16)

        fn emsg(msg: *const std::ffi::c_char) -> bool;
        // (nvim_qf_jump_emsg_win_closed deleted: use emsg directly)
        // (nvim_qf_jump_emsg_qf_changed deleted: use emsg directly)
        // (nvim_qf_jump_emsg_ll_changed deleted: use emsg directly)
        // (nvim_qf_emsg_winfixbuf deleted: use emsg directly)

        // Phase 15 thin primitives replacing thick jump-open wrappers
        fn nvim_can_abandon_curbuf(forceit: c_int) -> bool;
        fn nvim_no_write_message();
        fn nvim_do_ecmd_help(fnum: c_int, prev_winid: c_int) -> c_int;
        // nvim_qf_buflist_getfile deleted: use buflist_getfile directly
        fn buflist_getfile(n: c_int, lnum: c_int, options: c_int, forceit: c_int) -> c_int;
        fn nvim_curwin_get_wfb() -> bool;
        fn nvim_qf_curbuf_fnum() -> c_int;
        fn nvim_qf_get_qi_type(qi: *const c_void) -> c_int;
        fn nvim_qf_prevwin_valid_for_wfb() -> bool;
        fn nvim_win_id2wp(id: c_int) -> *mut c_void;
        fn nvim_qf_curwin_get_loclist() -> *mut c_void;
        static mut prevwin: *mut c_void;
        fn win_goto(win: *mut c_void);
        fn win_split(size: c_int, flags: c_int) -> c_int;
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

        let fnum = nvim_qfline_get_fnum(qf_ptr);
        let retval = if nvim_qfline_get_type(qf_ptr) == 1 {
            // Open help file: inline nvim_qf_jump_open_help
            if !nvim_can_abandon_curbuf(forceit) {
                nvim_no_write_message();
                return FAIL; // sentinel: skip post-validation
            }
            nvim_do_ecmd_help(fnum, prev_winid)
        } else {
            // Open normal file: inline nvim_qf_jump_open_file (winfixbuf logic)
            let mut retval = OK;
            if forceit == 0 && nvim_curwin_get_wfb() && nvim_qf_curbuf_fnum() != fnum {
                if nvim_qf_get_qi_type(qi) == QFLT_LOCATION {
                    emsg(c"E1513: Cannot switch buffer. 'winfixbuf' is enabled".as_ptr());
                    return FAIL; // sentinel: location list winfixbuf early return
                }
                if nvim_qf_prevwin_valid_for_wfb() {
                    win_goto(prevwin);
                }
                if nvim_curwin_get_wfb() {
                    if win_split(0, 0) == OK {
                        *opened_window = true;
                    }
                    if nvim_curwin_get_wfb() {
                        emsg(c"E1513: Cannot switch buffer. 'winfixbuf' is enabled".as_ptr());
                        retval = FAIL;
                    }
                }
            }
            if retval == OK {
                retval = buflist_getfile(fnum, 1, GETF_SETMARK | GETF_SWITCH, forceit);
            }
            retval
        };

        // If a location list, check whether the associated window is still present.
        // Inline nvim_qf_jump_loc_win_closed: win_id2wp returns NULL means win closed,
        // and curwin->w_llist != qi means we're not in the right location list window.
        if qfl_type == QFLT_LOCATION {
            let wp = nvim_win_id2wp(prev_winid);
            if wp.is_null() && nvim_qf_curwin_get_loclist() != qi {
                emsg(c"E924: Current window was closed".as_ptr());
                *opened_window = false;
                return QF_ABORT;
            }
        }

        // Check if the quickfix list is still valid.
        if qfl_type == QFLT_QUICKFIX && !crate::qflist_valid_for_qi(qi, save_qfid) {
            emsg(c"E925: Current quickfix list was changed".as_ptr());
            return QF_ABORT;
        }

        // Check if the list was changed by autocommands.
        if old_qf_curlist != nvim_qf_get_curlist_idx(qi)
            || old_changetick != nvim_qf_get_changedtick(qfl)
            || !crate::rs_qf_entry_present(qfl, qf_ptr)
        {
            if qfl_type == QFLT_QUICKFIX {
                emsg(c"E925: Current quickfix list was changed".as_ptr());
            } else {
                emsg(c"E926: Current location list was changed".as_ptr());
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
pub mod jump_machinery {
    use std::ffi::{c_int, c_void};
    use std::ptr;

    extern "C" {
        static must_redraw: c_int;
    }

    /// Opaque handles
    type QfInfoHandleMut = *mut c_void;
    type WinHandle = *mut c_void;

    const OK: c_int = 1;
    const FAIL: c_int = 0;

    // WSP_ flags matching src/nvim/window.h
    const WSP_HELP: c_int = 0x20;
    const WSP_TOP: c_int = 0x08;
    const WSP_NEWLOC: c_int = 0x100;
    const WSP_ABOVE: c_int = 0x80;

    #[allow(dead_code)]
    extern "C" {
        // Window-finding wrappers (Phase 1)
        fn nvim_qf_find_help_win() -> WinHandle;
        fn nvim_qf_find_win_with_loclist(ll: *const c_void) -> WinHandle;
        fn nvim_qf_find_win_with_normal_buf() -> WinHandle;
        fn nvim_qf_goto_tabwin_with_file(fnum: c_int) -> bool;
        // nvim_qf_open_new_file_win removed: inlined (Phase 15)
        // Phase 15 thin primitives for open_new_file_win inlining
        fn nvim_qf_set_swb_empty_option();
        fn nvim_qf_curwin_reset_binding();

        // Window/buffer state accessors (Phase 1)
        fn nvim_qf_curwin_get_llist_ref() -> *mut c_void;
        fn rs_bt_quickfix(buf: *mut c_void) -> bool;
        fn nvim_qf_curwin_buf_is_help() -> bool;
        fn nvim_qf_get_cmdmod_tab() -> c_int;
        fn nvim_qf_is_one_window() -> bool;
        fn nvim_qf_swb_has_usetab() -> bool;

        // Window operations (Phase 2)
        fn win_goto(win: WinHandle);
        fn win_enter(win: WinHandle, undo_sync: bool);
        fn win_close(win: WinHandle, free_buf: bool, force: bool) -> c_int;
        fn nvim_qf_win_buf_nwindows(win: *const c_void) -> c_int;
        fn nvim_qf_win_buf_fnum(win: *const c_void) -> c_int;
        fn nvim_qf_win_get_llist(win: *const c_void) -> *mut c_void;
        fn nvim_qf_win_set_loclist(win: WinHandle, qi: *mut c_void);
        fn nvim_qf_get_cmdmod_split() -> c_int;
        fn nvim_qf_curwin_width() -> c_int;
        static Columns: c_int;
        fn nvim_qf_curwin_height() -> c_int;
        static p_hh: i64;
        fn win_split(size: c_int, flags: c_int) -> c_int;
        #[link_name = "rs_win_setheight"]
        fn nvim_qf_win_setheight(height: c_int);
        static mut restart_edit: c_int;
        fn nvim_qf_is_ll_stack_qi(qi: *const c_void) -> bool;
        fn nvim_qf_win_is_qf_window(win: *const c_void) -> bool;
        fn nvim_qf_win_prev(win: *const c_void) -> WinHandle;
        fn nvim_qf_win_next(win: *const c_void) -> WinHandle;
        static lastwin: *mut c_void;
        static curwin: *mut c_void;
        fn nvim_qf_win_bt_normal(win: *const c_void) -> bool;
        fn nvim_qf_swb_uselast_prevwin_ok() -> bool;
        static mut prevwin: *mut c_void;
        fn nvim_qf_win_is_preview(win: *const c_void) -> bool;
        fn nvim_qf_win_is_wfb(win: *const c_void) -> bool;
        static mut curbuf: *mut c_void;
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
            win_enter(wp, true);
        } else {
            // Split off help window; put it at far top if no position
            // specified, the current window is vertically split and narrow.
            let mut flags = WSP_HELP;
            if nvim_qf_get_cmdmod_split() == 0
                && nvim_qf_curwin_width() != Columns
                && nvim_qf_curwin_width() < 80
            {
                flags |= WSP_TOP;
            }

            // If the user asks to open a new window, then copy the location list.
            // Otherwise, don't copy the location list.
            if nvim_qf_is_ll_stack_qi(qi) && !newwin {
                flags |= WSP_NEWLOC;
            }

            if win_split(0, flags) == FAIL {
                return FAIL;
            }

            *opened_window = true;

            if nvim_qf_curwin_height() < p_hh as c_int {
                nvim_qf_win_setheight(p_hh as c_int);
            }

            // When using location list, the new window should use the supplied
            // location list.
            if nvim_qf_is_ll_stack_qi(qi) && !newwin {
                nvim_qf_win_set_loclist(curwin, qi);
            }
        }

        restart_edit = 0; // don't want insert mode in help file

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
            let mut w = lastwin;
            while !w.is_null() {
                if nvim_qf_win_buf_fnum(w) == qf_fnum {
                    win = w;
                    break;
                }
                w = nvim_qf_win_prev(w);
            }

            if win.is_null() {
                // Find a previous usable window (walk backwards from curwin)
                win = curwin;
                loop {
                    if nvim_qf_win_bt_normal(win) {
                        break;
                    }
                    let prev = nvim_qf_win_prev(win);
                    if prev.is_null() {
                        win = lastwin; // wrap around the top
                    } else {
                        win = prev; // go to previous window
                    }
                    if win == curwin {
                        break;
                    }
                }
            }
        }
        win_goto(win);

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
        let mut win = curwin;
        let mut altwin: WinHandle = ptr::null_mut();

        loop {
            if nvim_qf_win_buf_fnum(win) == qf_fnum {
                break;
            }
            let prev = nvim_qf_win_prev(win);
            if prev.is_null() {
                win = lastwin; // wrap around the top
            } else {
                win = prev; // go to previous window
            }

            if nvim_qf_win_is_qf_window(win) {
                // Didn't find it, go to the window before the quickfix
                // window, unless 'switchbuf' contains 'uselast': in this case we
                // try to jump to the previously used window first.
                if nvim_qf_swb_uselast_prevwin_ok() {
                    win = prevwin;
                } else if !altwin.is_null() {
                    win = altwin;
                } else {
                    let cur_prev = nvim_qf_win_prev(curwin);
                    if cur_prev.is_null() {
                        win = nvim_qf_win_next(curwin);
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

        win_goto(win);
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
        // Inline nvim_qf_open_new_file_win
        if (nvim_qf_is_one_window() && rs_bt_quickfix(curbuf)) || !usable_win || newwin {
            let mut win_flags = WSP_ABOVE;
            if !ll_ref.is_null() {
                win_flags |= WSP_NEWLOC;
            }
            if win_split(0, win_flags) == FAIL {
                return FAIL;
            }
            nvim_qf_set_swb_empty_option();
            nvim_qf_curwin_reset_binding();
            if !ll_ref.is_null() {
                nvim_qf_win_set_loclist(curwin, ll_ref);
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
        // nvim_qf_entry_present removed: use crate::rs_qf_entry_present (Phase 16)
        fn emsg(msg: *const std::ffi::c_char) -> bool;
        // (nvim_qf_jump_emsg_qf_changed deleted: use emsg directly)
        // (nvim_qf_jump_emsg_ll_changed deleted: use emsg directly)

        // Phase 3 wrappers
        fn nvim_qf_fdo_quickfix() -> bool;
        #[link_name = "rs_foldOpenCursor"]
        fn nvim_qf_fold_open_cursor();
        fn setpcmark();

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

        // Phase 14 Phase 2: goto_line inlined accessors
        fn nvim_qf_curbuf_line_count() -> LinenrT;
        fn nvim_qf_curwin_set_cursor(lnum: LinenrT, col: c_int);
        fn nvim_qf_curwin_set_col(col: c_int);
        fn nvim_qf_curwin_set_coladd_zero();
        fn nvim_qf_curwin_set_curswant();
        fn nvim_qf_coladvance(col: c_int);
        fn nvim_qf_beginline_white_fix();
        fn nvim_qf_do_search_pattern(pat: *const std::ffi::c_char) -> bool;
        fn check_cursor(wp: *mut c_void);
        fn update_topline(wp: *mut c_void);
        // Phase 14 Phase 2: print_msg inlined accessors
        fn nvim_get_msg_scrolled() -> c_int;
        fn nvim_update_screen();
        fn nvim_qf_get_curlist_count(qi: *const c_void) -> c_int;
        fn nvim_qfline_get_cleared_bool(qfp: QfLineHandle) -> bool;
        fn nvim_qfline_get_type_char(qfp: QfLineHandle) -> std::ffi::c_char;
        fn nvim_qfline_get_nr_int(qfp: QfLineHandle) -> c_int;
        fn nvim_qfline_get_text_ptr(qfp: QfLineHandle) -> *const std::ffi::c_char;
        fn skipwhite(s: *const std::ffi::c_char) -> *mut std::ffi::c_char;
        fn nvim_get_msg_scroll() -> c_int;
        fn nvim_set_msg_scroll(val: c_int);
        fn nvim_ecmd_shortmess_overall() -> c_int;
        fn nvim_get_p_ch() -> i64;
        fn nvim_msg_ext_set_kind(kind: *const std::ffi::c_char);
        fn msg_keep(s: *const std::ffi::c_char, attr: c_int, keep: bool, multiline: bool) -> bool;
        // (nvim_msg_keep_qf deleted: use msg_keep directly)
        fn nvim_qf_gettext_line_deleted() -> *const std::ffi::c_char;
        // rs_qf_fmt_text (from window.rs, intra-crate symbol)
        #[link_name = "rs_qf_fmt_text"]
        fn navigate_qf_fmt_text(
            text: *const std::ffi::c_char,
            out: *mut std::ffi::c_char,
            out_size: usize,
        ) -> usize;
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
        let old_curbuf = curbuf;
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
        if curbuf == old_curbuf {
            setpcmark();
        }

        // Phase 14: inlined nvim_qf_jump_goto_line
        qf_jump_goto_line(
            nvim_qfline_get_lnum(qf_ptr),
            nvim_qfline_get_col(qf_ptr),
            nvim_qfline_get_viscol(qf_ptr),
            nvim_qfline_get_pattern(qf_ptr),
        );

        if nvim_qf_fdo_quickfix() && openfold != 0 {
            nvim_qf_fold_open_cursor();
        }
        if print_message {
            // Phase 14: inlined nvim_qf_jump_print_msg
            qf_jump_print_msg(qi, qf_index, qf_ptr, old_curbuf, old_lnum);
        }

        retval
    }

    /// Position cursor at the quickfix entry location.
    /// Inlined from C `nvim_qf_jump_goto_line` (Phase 14).
    ///
    /// # Safety
    ///
    /// - `qf_pattern` may be null (use line/col positioning)
    /// - If non-null, must be a valid NUL-terminated C string
    unsafe fn qf_jump_goto_line(
        qf_lnum: LinenrT,
        qf_col: c_int,
        qf_viscol: bool,
        qf_pattern: *const std::ffi::c_char,
    ) {
        if qf_pattern.is_null() {
            // Go to line with error, unless qf_lnum is 0.
            if qf_lnum > 0 {
                let line_count = nvim_qf_curbuf_line_count();
                let i = qf_lnum.min(line_count);
                nvim_qf_curwin_set_cursor(i, 0);
            }
            if qf_col > 0 {
                nvim_qf_curwin_set_coladd_zero();
                if qf_viscol {
                    nvim_qf_coladvance(qf_col - 1);
                } else {
                    nvim_qf_curwin_set_col(qf_col - 1);
                }
                nvim_qf_curwin_set_curswant();
                check_cursor(curwin);
            } else {
                nvim_qf_beginline_white_fix();
            }
        } else {
            // Pattern-based jump (helpgrep style)
            nvim_qf_do_search_pattern(qf_pattern);
        }
    }

    /// Print the "(N of M)" quickfix jump status message.
    /// Inlined from C `nvim_qf_jump_print_msg` (Phase 14).
    ///
    /// # Safety
    ///
    /// All pointer parameters must be valid.
    #[allow(clippy::too_many_lines)]
    unsafe fn qf_jump_print_msg(
        qi: QfInfoHandleMut,
        qf_index: c_int,
        qf_ptr: QfLineHandle,
        old_curbuf: BufHandle,
        old_lnum: LinenrT,
    ) {
        use std::ffi::c_char;

        // Update the screen before showing the message, unless messages scrolled.
        if nvim_get_msg_scrolled() == 0 {
            update_topline(curwin);
            if unsafe { must_redraw } != 0 {
                nvim_update_screen();
            }
        }

        // Build types string
        let mut type_buf = [0u8; 20];
        crate::display::qf_types_fmt(
            c_int::from(nvim_qfline_get_type_char(qf_ptr)),
            nvim_qfline_get_nr_int(qf_ptr),
            &mut type_buf,
        );
        let type_len = type_buf.iter().position(|&b| b == 0).unwrap_or(20);
        let type_str = std::str::from_utf8(&type_buf[..type_len]).unwrap_or("");

        // Build cleared string
        let cleared_str = if nvim_qfline_get_cleared_bool(qf_ptr) {
            std::ffi::CStr::from_ptr(nvim_qf_gettext_line_deleted())
                .to_str()
                .unwrap_or(" (line deleted)")
        } else {
            ""
        };

        let count = nvim_qf_get_curlist_count(qi);

        // Build the header prefix (mirrors C vim_snprintf(IObuff, ...))
        let header = format!("({qf_index} of {count}){cleared_str}{type_str}: ");

        // Build the fmt_text part: skip leading whitespace then qf_fmt_text
        let qf_text_ptr = nvim_qfline_get_text_ptr(qf_ptr);
        let fmt_text = if qf_text_ptr.is_null() {
            String::new()
        } else {
            let text_no_ws = skipwhite(qf_text_ptr);
            let mut fmt_buf = vec![0i8; 1025];
            let fmt_len =
                navigate_qf_fmt_text(text_no_ws.cast_const(), fmt_buf.as_mut_ptr(), fmt_buf.len());
            if fmt_len > 0 {
                let bytes: Vec<u8> = fmt_buf[..fmt_len].iter().map(|&b| b as u8).collect();
                String::from_utf8_lossy(&bytes).into_owned()
            } else {
                String::new()
            }
        };

        // Combine header + fmt_text and NUL-terminate
        let mut full_msg = header;
        full_msg.push_str(&fmt_text);
        full_msg.push('\0');

        // Output the message.  Overwrite to avoid scrolling when the 'O'
        // flag is present in 'shortmess'; But when not jumping, print the whole message.
        let saved_scroll = nvim_get_msg_scroll();
        if curbuf == old_curbuf && nvim_qf_get_cursor_lnum() == old_lnum {
            nvim_set_msg_scroll(1);
        } else if (nvim_get_msg_scrolled() == 0
            || (nvim_get_p_ch() == 0 && nvim_get_msg_scrolled() == 1))
            && nvim_ecmd_shortmess_overall() != 0
        {
            nvim_set_msg_scroll(0);
        }
        nvim_msg_ext_set_kind(c"quickfix".as_ptr());
        msg_keep(full_msg.as_ptr().cast::<c_char>(), 0, true, false);
        nvim_set_msg_scroll(saved_scroll);
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
            || !crate::rs_qf_entry_present(qfl, qf_ptr)
        {
            if qfl_type == QFLT_QUICKFIX {
                emsg(c"E925: Current quickfix list was changed".as_ptr());
            } else {
                emsg(c"E926: Current location list was changed".as_ptr());
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
        if rs_bt_quickfix(curbuf) && !*opened_window {
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

    // Phase 5 extern declarations
    extern "C" {
        fn nvim_get_ql_info() -> QfInfoHandleMut;
        fn rs_qf_stack_empty(qi: *const c_void) -> bool;
        fn rs_qf_list_empty(qfl: *const c_void) -> bool;
        // (nvim_emsg_no_errors deleted: use emsg directly, declared earlier in this module)
        #[link_name = "rs_incr_quickfix_busy"]
        fn nvim_incr_quickfix_busy();
        #[link_name = "rs_decr_quickfix_busy"]
        fn nvim_decr_quickfix_busy();
        fn nvim_qf_get_ptr(qfl: *const c_void) -> QfLineHandle;
        fn nvim_qf_get_index(qfl: *const c_void) -> c_int;
        fn nvim_qf_set_ptr(qfl: *mut c_void, ptr: QfLineHandle);
        fn nvim_qf_set_index(qfl: *mut c_void, idx: c_int);
        fn nvim_qf_curwin_handle() -> c_int;
        // win_close already declared above; win_close(curwin, true, false)
        fn nvim_qf_get_p_swb() -> *mut c_void;
        fn nvim_qf_get_swb_flags() -> u32;
        fn nvim_qf_restore_swb(old_swb: *mut c_void, old_swb_flags: u32);
        fn nvim_qf_get_key_typed() -> bool;
        fn rs_qf_get_entry_with_msg(
            qfl: *const c_void,
            errornr: c_int,
            dir: c_int,
        ) -> super::QfGetEntryResult;
    }

    /// Internal result for the jump operation, used to communicate
    /// state between the inner logic and the cleanup path.
    struct JumpResult {
        /// The `qi` pointer (may be nulled out on `QF_ABORT`)
        qi: QfInfoHandleMut,
        /// The `qf_ptr` to store back (may be reverted to old)
        qf_ptr: QfLineHandle,
        /// The `qf_index` to store back (may be reverted to old)
        qf_index: c_int,
    }

    /// Main jump orchestrator. Replaces `qf_jump_newwin`.
    ///
    /// # Safety
    ///
    /// `qi` may be null (uses global `ql_info`). Other globals must be valid.
    #[no_mangle]
    pub unsafe extern "C" fn rs_qf_jump_newwin(
        mut qi: QfInfoHandleMut,
        dir: c_int,
        errornr: c_int,
        forceit: c_int,
        newwin: bool,
    ) {
        let old_swb = nvim_qf_get_p_swb();
        let old_swb_flags = nvim_qf_get_swb_flags();
        let old_key_typed = nvim_qf_get_key_typed();

        if qi.is_null() {
            qi = nvim_get_ql_info();
        }

        if rs_qf_stack_empty(qi) || rs_qf_list_empty(nvim_qf_get_curlist(qi)) {
            emsg(c"E42: No Errors".as_ptr());
            return;
        }

        nvim_incr_quickfix_busy();

        let qfl = nvim_qf_get_curlist(qi).cast_mut();
        let result = jump_newwin_inner(qi, qfl, dir, errornr, forceit, newwin, old_key_typed);

        // Cleanup: restore qfl state and swb
        if !result.qi.is_null() {
            nvim_qf_set_ptr(qfl, result.qf_ptr);
            nvim_qf_set_index(qfl, result.qf_index);
        }
        nvim_qf_restore_swb(old_swb, old_swb_flags);
        nvim_decr_quickfix_busy();
    }

    /// Inner function implementing the jump logic with structured control flow.
    /// Returns the final state to be applied in the cleanup path.
    unsafe fn jump_newwin_inner(
        qi: QfInfoHandleMut,
        qfl: *mut c_void,
        dir: c_int,
        errornr: c_int,
        forceit: c_int,
        newwin: bool,
        old_key_typed: bool,
    ) -> JumpResult {
        let old_qf_ptr = nvim_qf_get_ptr(qfl);
        let old_qf_index = nvim_qf_get_index(qfl);

        // Select the entry
        let entry_result = rs_qf_get_entry_with_msg(qfl, errornr, dir);

        if entry_result.qf_ptr.is_null() || entry_result.errored {
            // Entry selection failed; restore old state
            return JumpResult {
                qi,
                qf_ptr: old_qf_ptr,
                qf_index: old_qf_index,
            };
        }

        let mut qf_ptr = entry_result.qf_ptr;
        let mut qf_index = entry_result.qf_index;

        // Set the new index/ptr on the list
        nvim_qf_set_index(qfl, qf_index);
        nvim_qf_set_ptr(qfl, qf_ptr);

        // No need to print the error message if it's visible in the error window
        let print_message = !crate::rs_qf_win_pos_update_impl(qi, old_qf_index);

        let prev_winid = nvim_qf_curwin_handle();

        let mut opened_window = false;
        let retval = rs_qf_jump_open_window(qi, qf_ptr, newwin, &raw mut opened_window);

        if retval == FAIL {
            // Jump to the "failed" path: restore old index
            return JumpResult {
                qi,
                qf_ptr: old_qf_ptr,
                qf_index: old_qf_index,
            };
        }
        if retval == QF_ABORT {
            // qi and qf_ptr are nulled out
            return JumpResult {
                qi: ptr::null_mut(),
                qf_ptr: ptr::null(),
                qf_index,
            };
        }
        if retval == NOTDONE {
            return JumpResult {
                qi,
                qf_ptr,
                qf_index,
            };
        }

        let retval = rs_qf_jump_to_buffer(
            qi,
            qf_index,
            qf_ptr,
            forceit,
            prev_winid,
            &raw mut opened_window,
            i32::from(old_key_typed),
            print_message,
        );

        if retval == QF_ABORT {
            // Quickfix/location list was modified by an autocmd
            return JumpResult {
                qi: ptr::null_mut(),
                qf_ptr: ptr::null(),
                qf_index,
            };
        }

        if retval != OK {
            if opened_window {
                win_close(curwin, true, false);
            }
            if !qf_ptr.is_null() && nvim_qfline_get_fnum(qf_ptr) != 0 {
                // Couldn't open file, so put index back where it was.
                qf_ptr = old_qf_ptr;
                qf_index = old_qf_index;
            }
        }

        JumpResult {
            qi,
            qf_ptr,
            qf_index,
        }
    }
}

// =============================================================================
// Phase 4: qf_mark_adjust, qf_get_valid_size, qf_get_cur_valid_idx
// =============================================================================

/// Maximum valid line number (sentinel for deletion: same as C MAXLNUM = 0x7fffffff)
const MAXLNUM: LinenrT = i32::MAX;

extern "C" {
    // qfl list-level accessors (additional to those in the main extern block above)
    fn nvim_qf_get_listcount(qi: *const c_void) -> c_int;
    fn nvim_qf_get_list_at(qi: *const c_void, idx: c_int) -> *const c_void;
    fn nvim_qf_list_is_empty(qfl: *const c_void) -> bool;

    // entry setters (mutable variants)
    fn nvim_qfline_set_lnum(qfp: *mut c_void, lnum: LinenrT);
    fn nvim_qfline_set_cleared(qfp: *mut c_void, cleared: i8);

    // rs_qf_list_has_valid_entries is in lib.rs but we need it here too
    fn rs_qf_list_has_valid_entries(qfl: *const c_void) -> bool;
}

/// Adjust quickfix entry line numbers when buffer lines are added/deleted.
///
/// Iterates all lists in `qi` and adjusts entries that match `buf_fnum`.
///
/// # Safety
/// `qi` must be a valid non-null `qf_info_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_mark_adjust(
    qi: *mut c_void,
    buf_fnum: c_int,
    _buf_has_flag: c_int,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) -> bool {
    let listcount = nvim_qf_get_listcount(qi);
    let mut found_one = false;

    for idx in 0..listcount {
        let qfl = nvim_qf_get_list_at(qi, idx);
        if nvim_qf_list_is_empty(qfl) {
            continue;
        }

        let count = nvim_qf_get_count(qfl);
        let mut qfp = nvim_qf_get_start(qfl);
        let mut i = 1;
        while i <= count && !qfp.is_null() {
            if nvim_qfline_get_fnum(qfp) == buf_fnum {
                found_one = true;
                let lnum = nvim_qfline_get_lnum(qfp);
                if lnum >= line1 && lnum <= line2 {
                    if amount == MAXLNUM {
                        nvim_qfline_set_cleared(qfp.cast_mut(), 1);
                    } else {
                        nvim_qfline_set_lnum(qfp.cast_mut(), lnum + amount);
                    }
                } else if amount_after != 0 && lnum > line2 {
                    nvim_qfline_set_lnum(qfp.cast_mut(), lnum + amount_after);
                }
            }
            qfp = nvim_qfline_get_next(qfp);
            i += 1;
        }
    }

    found_one
}

/// Count valid entries (or unique valid files) in the current quickfix list.
///
/// `count_files`: if true, count unique files instead of all valid entries.
///
/// # Safety
/// `qfl` must be a valid non-null `qf_list_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_valid_size(qfl: *const c_void, count_files: bool) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    let mut qfp = nvim_qf_get_start(qfl);
    let mut sz: c_int = 0;
    let mut prev_fnum: c_int = 0;
    let mut i = 1;

    while i <= count && !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) {
            if count_files {
                let fnum = nvim_qfline_get_fnum(qfp);
                if fnum > 0 && fnum != prev_fnum {
                    // Count unique files
                    sz += 1;
                    prev_fnum = fnum;
                }
            } else {
                // Count all valid entries
                sz += 1;
            }
        }
        qfp = nvim_qfline_get_next(qfp);
        i += 1;
    }

    sz
}

/// Compute the 1-based valid entry index at the current cursor position.
///
/// `qf_index`: the current 1-based index (qfl->qf_index).
/// `count_files`: if true, count unique files (for :cfdo/:lfdo).
///
/// Returns 1 if there are no valid entries before the cursor.
///
/// # Safety
/// `qfl` must be a valid non-null `qf_list_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_cur_valid_idx(
    qfl: *const c_void,
    qf_index: c_int,
    count_files: bool,
) -> c_int {
    if qfl.is_null() {
        return 1;
    }

    // Check if the list has valid entries
    if !rs_qf_list_has_valid_entries(qfl) {
        return 1;
    }

    let mut prev_fnum: c_int = 0;
    let mut eidx: c_int = 0;
    let mut qfp = nvim_qf_get_start(qfl);
    let mut i: c_int = 1;

    while i <= qf_index && !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) {
            if count_files {
                let fnum = nvim_qfline_get_fnum(qfp);
                if fnum > 0 && fnum != prev_fnum {
                    eidx += 1;
                    prev_fnum = fnum;
                }
            } else {
                eidx += 1;
            }
        }
        qfp = nvim_qfline_get_next(qfp);
        i += 1;
    }

    if eidx != 0 {
        eidx
    } else {
        1
    }
}

// =============================================================================
// Phase 2: qf_mark_adjust entry and qf_jump_first
// =============================================================================

type QfInfoHandleMut = *mut c_void;
type BufHandle = *const c_void;
type WinHandleP2 = *const c_void;

const BUF_HAS_QF_ENTRY_P2: c_int = 1;
const BUF_HAS_LL_ENTRY_P2: c_int = 2;
const FAIL_VAL: c_int = 0;

extern "C" {
    fn nvim_buf_get_has_qf_entry(buf: BufHandle) -> c_int;
    fn nvim_qf_buf_get_fnum(buf: BufHandle) -> c_int;
    fn nvim_buf_win_get_llist(wp: WinHandleP2) -> QfInfoHandleMut;
    fn nvim_check_can_set_curbuf_forceit(forceit: c_int) -> bool;
    fn nvim_get_ql_info() -> QfInfoHandleMut;
    #[link_name = "nvim_qf_get_curlist"]
    fn nvim_qf_get_curlist_nav(qi: *const c_void) -> QfListHandle;
}

/// Entry point for `qf_mark_adjust`: resolves `buf_T`/`win_T` to qi and
/// delegates to `rs_qf_mark_adjust`.
///
/// Returns false if buf has no qf entry, or if wp != NULL but has no llist.
///
/// # Panics
///
/// Panics if the global quickfix info pointer is null.
///
/// # Safety
///
/// - `buf` must be a valid `buf_T *`
/// - `wp` may be null (indicates global quickfix list)
/// - If `wp` is non-null, it must be a valid `win_T *`
#[export_name = "qf_mark_adjust"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_qf_mark_adjust_entry(
    buf: BufHandle,
    wp: WinHandleP2,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) -> bool {
    let buf_has_flag = if wp.is_null() {
        BUF_HAS_QF_ENTRY_P2
    } else {
        BUF_HAS_LL_ENTRY_P2
    };

    if (nvim_buf_get_has_qf_entry(buf) & buf_has_flag) == 0 {
        return false;
    }

    let qi: QfInfoHandleMut = if wp.is_null() {
        let ql = nvim_get_ql_info();
        assert!(!ql.is_null());
        ql
    } else {
        let llist = nvim_buf_win_get_llist(wp);
        if llist.is_null() {
            return false;
        }
        llist
    };

    let buf_fnum = nvim_qf_buf_get_fnum(buf);
    rs_qf_mark_adjust(
        qi,
        buf_fnum,
        buf_has_flag,
        line1,
        line2,
        amount,
        amount_after,
    )
}

/// Rust implementation of `qf_jump_first`: restores list, checks curbuf
/// safety, and jumps to the first entry.
///
/// # Safety
///
/// - `qi` must be a valid `qf_info_T *`
#[no_mangle]
pub unsafe extern "C" fn rs_qf_jump_first(qi: QfInfoHandleMut, save_qfid: u32, forceit: c_int) {
    if crate::rs_qf_restore_list(qi, save_qfid) == FAIL_VAL {
        return;
    }

    if !nvim_check_can_set_curbuf_forceit(forceit) {
        return;
    }

    // Autocommands might have cleared the list — check for that
    let qfl = nvim_qf_get_curlist_nav(qi.cast_const());
    if !crate::rs_qf_list_empty(qfl) {
        jump_machinery::rs_qf_jump_newwin(qi, 0, 0, forceit, false);
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
