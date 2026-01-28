//! Diff put/get operations
//!
//! This module provides structures and helpers for the :diffget and :diffput
//! commands. It handles:
//! - Finding source and target buffers
//! - Line range calculations
//! - Text transfer between diff buffers
//! - Undo integration

use std::ffi::c_int;

use crate::buffer::{BufHandle, DiffBlockHandle, TabpageHandle, DB_COUNT};

// Line number type matching linenr_T (i32)
type LinenrT = i32;

// Result constants
#[allow(dead_code)]
const OK: c_int = 1;
#[allow(dead_code)]
const FAIL: c_int = 0;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    fn nvim_tabpage_get_diffbuf(tp: TabpageHandle, idx: c_int) -> BufHandle;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
}

// =============================================================================
// Buffer Finding
// =============================================================================

/// Result of finding the other buffer for diffget/diffput.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffOtherBufResult {
    /// Index of the other buffer in diff (-1 if not found)
    pub idx: c_int,
    /// Error code if failed
    pub error: DiffOpError,
    /// Whether multiple modifiable buffers were found
    pub multiple_found: bool,
}

impl Default for DiffOtherBufResult {
    fn default() -> Self {
        Self {
            idx: -1,
            error: DiffOpError::None,
            multiple_found: false,
        }
    }
}

/// Error codes for diffget/diffput operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffOpError {
    /// No error
    #[default]
    None = 0,
    /// Current buffer not in diff mode
    NotInDiff = 1,
    /// No other buffer in diff mode
    NoOtherBuffer = 2,
    /// Other buffer not modifiable (for diffput)
    NotModifiable = 3,
    /// More than two buffers, ambiguous
    AmbiguousBuffer = 4,
    /// Buffer not found
    BufferNotFound = 5,
    /// Specified buffer not in diff
    SpecifiedNotInDiff = 6,
    /// Buffer changed unexpectedly
    BufferChanged = 7,
}

/// Find the other buffer for diffget/diffput when no argument given.
///
/// For diffput, only modifiable buffers are considered.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
/// `curbuf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_find_other_buf(
    tp: TabpageHandle,
    curbuf: BufHandle,
    cur_idx: c_int,
    is_diffput: bool,
    check_modifiable: extern "C" fn(BufHandle) -> bool,
) -> DiffOtherBufResult {
    if tp.is_null() || curbuf.is_null() || cur_idx < 0 || cur_idx >= DB_COUNT as c_int {
        return DiffOtherBufResult {
            error: DiffOpError::NotInDiff,
            ..Default::default()
        };
    }

    let mut result = DiffOtherBufResult::default();
    let mut found_not_modifiable = false;
    let mut first_other_idx: c_int = -1;

    // Find first other buffer
    for i in 0..DB_COUNT as c_int {
        let buf = nvim_tabpage_get_diffbuf(tp, i);
        if buf.is_null() || buf == curbuf {
            continue;
        }

        // For diffput, check modifiable
        if is_diffput && !check_modifiable(buf) {
            found_not_modifiable = true;
            continue;
        }

        if first_other_idx < 0 {
            first_other_idx = i;
        } else {
            // Found a second candidate
            result.multiple_found = true;
            result.error = DiffOpError::AmbiguousBuffer;
            return result;
        }
    }

    if first_other_idx < 0 {
        result.error = if found_not_modifiable {
            DiffOpError::NotModifiable
        } else {
            DiffOpError::NoOtherBuffer
        };
        return result;
    }

    result.idx = first_other_idx;
    result
}

// =============================================================================
// Range Calculation
// =============================================================================

/// Range for a diffget/diffput operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffOpRange {
    /// Start line in source buffer
    pub from_start: LinenrT,
    /// End line in source buffer (inclusive)
    pub from_end: LinenrT,
    /// Start line in target buffer
    pub to_start: LinenrT,
    /// End line in target buffer (inclusive)
    pub to_end: LinenrT,
    /// Number of lines being added/removed
    pub line_diff: LinenrT,
    /// Whether this is an addition (no lines in source)
    pub is_addition: bool,
    /// Whether this is a deletion (no lines in target)
    pub is_deletion: bool,
    /// Whether this range is valid
    pub valid: bool,
}

/// Calculate the operation range from a diff block.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_calc_op_range(
    dp: DiffBlockHandle,
    idx_from: c_int,
    idx_to: c_int,
) -> DiffOpRange {
    if dp.is_null()
        || idx_from < 0
        || idx_from >= DB_COUNT as c_int
        || idx_to < 0
        || idx_to >= DB_COUNT as c_int
    {
        return DiffOpRange::default();
    }

    let from_lnum = nvim_diffblock_get_lnum(dp, idx_from);
    let from_count = nvim_diffblock_get_count(dp, idx_from);
    let to_lnum = nvim_diffblock_get_lnum(dp, idx_to);
    let to_count = nvim_diffblock_get_count(dp, idx_to);

    DiffOpRange {
        from_start: from_lnum,
        from_end: from_lnum + from_count - 1,
        to_start: to_lnum,
        to_end: to_lnum + to_count - 1,
        line_diff: from_count - to_count,
        is_addition: from_count > 0 && to_count == 0,
        is_deletion: from_count == 0 && to_count > 0,
        valid: true,
    }
}

/// Check if a line range overlaps with a diff block.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_overlaps_range(
    dp: DiffBlockHandle,
    buf_idx: c_int,
    line1: LinenrT,
    line2: LinenrT,
) -> bool {
    if dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return false;
    }

    let block_start = nvim_diffblock_get_lnum(dp, buf_idx);
    let block_count = nvim_diffblock_get_count(dp, buf_idx);
    let block_end = block_start + block_count - 1;

    // Ranges overlap if they share any lines
    // Handle the case of empty blocks (count == 0)
    if block_count == 0 {
        // Empty block at block_start, check if it's within or adjacent to range
        return line1 <= block_start && block_start <= line2 + 1;
    }

    line1 <= block_end && line2 >= block_start
}

// =============================================================================
// Operation State
// =============================================================================

/// State for a diffget/diffput operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffOpState {
    /// Index of current buffer in diff
    pub idx_cur: c_int,
    /// Index of source buffer (for diffget: other, for diffput: current)
    pub idx_from: c_int,
    /// Index of target buffer (for diffget: current, for diffput: other)
    pub idx_to: c_int,
    /// First line of operation range
    pub line1: LinenrT,
    /// Last line of operation range
    pub line2: LinenrT,
    /// Whether operation is in progress
    pub in_progress: bool,
    /// Number of lines added (positive) or removed (negative)
    pub lines_changed: LinenrT,
}

impl Default for DiffOpState {
    fn default() -> Self {
        Self {
            idx_cur: -1,
            idx_from: -1,
            idx_to: -1,
            line1: 0,
            line2: 0,
            in_progress: false,
            lines_changed: 0,
        }
    }
}

/// Initialize operation state for diffget.
#[no_mangle]
pub const extern "C" fn rs_diff_op_init_get(
    idx_cur: c_int,
    idx_other: c_int,
    line1: LinenrT,
    line2: LinenrT,
) -> DiffOpState {
    DiffOpState {
        idx_cur,
        idx_from: idx_other, // get from other
        idx_to: idx_cur,     // put to current
        line1,
        line2,
        in_progress: true,
        lines_changed: 0,
    }
}

/// Initialize operation state for diffput.
#[no_mangle]
pub const extern "C" fn rs_diff_op_init_put(
    idx_cur: c_int,
    idx_other: c_int,
    line1: LinenrT,
    line2: LinenrT,
) -> DiffOpState {
    DiffOpState {
        idx_cur,
        idx_from: idx_cur, // put from current
        idx_to: idx_other, // put to other
        line1,
        line2,
        in_progress: true,
        lines_changed: 0,
    }
}

/// Update operation state after processing a block.
#[no_mangle]
pub const extern "C" fn rs_diff_op_update(
    state: &mut DiffOpState,
    lines_added: LinenrT,
    lines_removed: LinenrT,
) {
    state.lines_changed += lines_added - lines_removed;
}

/// Mark operation as complete.
#[no_mangle]
pub const extern "C" fn rs_diff_op_complete(state: &mut DiffOpState) {
    state.in_progress = false;
}

// =============================================================================
// Line Range Adjustment
// =============================================================================

/// Result of adjusting line range before operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct AdjustedRange {
    /// Adjusted start line
    pub line1: LinenrT,
    /// Adjusted end line
    pub line2: LinenrT,
    /// Whether the range was adjusted
    pub adjusted: bool,
}

/// Adjust line range to include adjacent lines when no explicit range given.
///
/// This makes :diffget on the last line work by including the line below
/// when there's no difference above the cursor.
#[no_mangle]
pub const extern "C" fn rs_diff_adjust_range_no_count(
    line1: LinenrT,
    line2: LinenrT,
    line_count: LinenrT,
    is_last_line: bool,
    no_diff_above: bool,
    has_diff_at_line: bool,
) -> AdjustedRange {
    let mut result = AdjustedRange {
        line1,
        line2,
        adjusted: false,
    };

    if is_last_line && !has_diff_at_line && no_diff_above {
        // Extend to include line below (if there is one)
        if line2 < line_count {
            result.line2 = line2 + 1;
            result.adjusted = true;
        }
    } else if line1 > 1 {
        // Include line above
        result.line1 = line1 - 1;
        result.adjusted = true;
    }

    result
}

// =============================================================================
// Block Iteration for Operations
// =============================================================================

/// Find the first diff block that overlaps with a line range.
///
/// # Safety
/// `first_dp` must be a valid diff block handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_find_first_overlapping_block(
    first_dp: DiffBlockHandle,
    buf_idx: c_int,
    line1: LinenrT,
    line2: LinenrT,
) -> DiffBlockHandle {
    if first_dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return DiffBlockHandle::null();
    }

    let mut dp = first_dp;
    while !dp.is_null() {
        if rs_diff_block_overlaps_range(dp, buf_idx, line1, line2) {
            return dp;
        }

        // If block is completely after our range, no more blocks will overlap
        let block_start = nvim_diffblock_get_lnum(dp, buf_idx);
        if block_start > line2 {
            break;
        }

        dp = nvim_diffblock_get_next(dp);
    }

    DiffBlockHandle::null()
}

/// Count blocks that overlap with a line range.
///
/// # Safety
/// `first_dp` must be a valid diff block handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_count_overlapping_blocks(
    first_dp: DiffBlockHandle,
    buf_idx: c_int,
    line1: LinenrT,
    line2: LinenrT,
) -> c_int {
    if first_dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return 0;
    }

    let mut count = 0;
    let mut dp = first_dp;

    while !dp.is_null() {
        let block_start = nvim_diffblock_get_lnum(dp, buf_idx);

        // If block is completely after our range, stop
        if block_start > line2 {
            break;
        }

        if rs_diff_block_overlaps_range(dp, buf_idx, line1, line2) {
            count += 1;
        }

        dp = nvim_diffblock_get_next(dp);
    }

    count
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_other_buf_result_default() {
        let result = DiffOtherBufResult::default();
        assert_eq!(result.idx, -1);
        assert_eq!(result.error, DiffOpError::None);
        assert!(!result.multiple_found);
    }

    #[test]
    fn test_diff_op_range_default() {
        let range = DiffOpRange::default();
        assert!(!range.valid);
        assert_eq!(range.from_start, 0);
        assert_eq!(range.to_start, 0);
    }

    #[test]
    fn test_diff_op_init_get() {
        let state = rs_diff_op_init_get(0, 1, 10, 20);
        assert_eq!(state.idx_cur, 0);
        assert_eq!(state.idx_from, 1); // from other
        assert_eq!(state.idx_to, 0); // to current
        assert_eq!(state.line1, 10);
        assert_eq!(state.line2, 20);
        assert!(state.in_progress);
    }

    #[test]
    fn test_diff_op_init_put() {
        let state = rs_diff_op_init_put(0, 1, 10, 20);
        assert_eq!(state.idx_cur, 0);
        assert_eq!(state.idx_from, 0); // from current
        assert_eq!(state.idx_to, 1); // to other
        assert_eq!(state.line1, 10);
        assert_eq!(state.line2, 20);
        assert!(state.in_progress);
    }

    #[test]
    fn test_diff_op_update() {
        let mut state = rs_diff_op_init_get(0, 1, 10, 20);
        rs_diff_op_update(&mut state, 5, 3);
        assert_eq!(state.lines_changed, 2);

        rs_diff_op_update(&mut state, 2, 4);
        assert_eq!(state.lines_changed, 0);
    }

    #[test]
    fn test_diff_op_complete() {
        let mut state = rs_diff_op_init_get(0, 1, 10, 20);
        assert!(state.in_progress);

        rs_diff_op_complete(&mut state);
        assert!(!state.in_progress);
    }

    #[test]
    fn test_adjust_range_last_line() {
        let result = rs_diff_adjust_range_no_count(100, 100, 100, true, true, false);
        // Can't extend past file end
        assert!(!result.adjusted);
        assert_eq!(result.line1, 100);
        assert_eq!(result.line2, 100);

        let result = rs_diff_adjust_range_no_count(50, 50, 100, true, true, false);
        assert!(result.adjusted);
        assert_eq!(result.line2, 51);
    }

    #[test]
    fn test_adjust_range_include_above() {
        let result = rs_diff_adjust_range_no_count(50, 50, 100, false, false, true);
        assert!(result.adjusted);
        assert_eq!(result.line1, 49);
        assert_eq!(result.line2, 50);
    }

    #[test]
    fn test_adjust_range_first_line() {
        let result = rs_diff_adjust_range_no_count(1, 1, 100, false, false, true);
        // Can't go before line 1
        assert!(!result.adjusted);
        assert_eq!(result.line1, 1);
    }
}
