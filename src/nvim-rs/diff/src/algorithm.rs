//! Core diff algorithm implementation
//!
//! This module provides the core diff computation functions for maintaining
//! diff state across buffer modifications. It handles:
//! - Diff block updates after buffer changes
//! - Hunk processing and integration
//! - Line number adjustments
//!
//! The main entry point is [`rs_diff_mark_adjust_tp`] which is called when
//! lines are inserted or deleted in a diff buffer.

use std::ffi::c_int;

use crate::buffer::{BufHandle, DiffBlockHandle, TabpageHandle, DB_COUNT};
use crate::rs_diff_internal;

// Line number type matching linenr_T (i32)
type LinenrT = i32;

// MAXLNUM value used to indicate unlimited range
const MAXLNUM: LinenrT = i32::MAX;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    fn nvim_tabpage_get_diffbuf(tp: TabpageHandle, idx: c_int) -> BufHandle;
    fn nvim_tabpage_get_first_diff(tp: TabpageHandle) -> DiffBlockHandle;
    fn nvim_tabpage_set_diff_invalid(tp: TabpageHandle, val: c_int);
    fn nvim_tabpage_set_diff_update(tp: TabpageHandle, val: c_int);

    fn nvim_diff_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
    fn nvim_diff_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diff_set_lnum(dp: DiffBlockHandle, idx: c_int, lnum: LinenrT);
    fn nvim_diff_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diff_set_count(dp: DiffBlockHandle, idx: c_int, count: LinenrT);

    fn nvim_diff_alloc_new(
        tp: TabpageHandle,
        prev: DiffBlockHandle,
        next: DiffBlockHandle,
    ) -> DiffBlockHandle;

    fn nvim_diff_is_busy() -> c_int;
}

// =============================================================================
// Line Adjustment State
// =============================================================================

/// State for tracking line adjustment during mark_adjust operations.
#[derive(Debug, Clone, Copy)]
struct AdjustState {
    /// Number of lines inserted
    inserted: LinenrT,
    /// Number of lines deleted
    deleted: LinenrT,
    /// Current line number of remaining deletion
    lnum_deleted: LinenrT,
}

impl AdjustState {
    /// Create adjustment state from mark_adjust parameters.
    ///
    /// The parameters come from mark_adjust() calls:
    /// - line1, MAXLNUM, amount, 0: insert lines
    /// - line1, line2, MAXLNUM, amount: a change that inserts lines
    /// - line1, line2, MAXLNUM, -amount: delete lines
    const fn from_params(
        line1: LinenrT,
        line2: LinenrT,
        amount: LinenrT,
        amount_after: LinenrT,
    ) -> Self {
        let (inserted, deleted) = if line2 == MAXLNUM {
            // mark_adjust(99, MAXLNUM, 9, 0): insert lines
            (amount, 0)
        } else if amount_after > 0 {
            // mark_adjust(99, 98, MAXLNUM, 9): a change that inserts lines
            (amount_after, 0)
        } else {
            // mark_adjust(98, 99, MAXLNUM, -2): delete lines
            (0, -amount_after)
        };

        Self {
            inserted,
            deleted,
            lnum_deleted: line1,
        }
    }
}

// =============================================================================
// Diff Block Operations
// =============================================================================

/// Check if a new diff block should be created for a change.
///
/// Returns true if the change is between existing diff blocks (not touching either).
unsafe fn should_create_new_block(
    dp: DiffBlockHandle,
    dprev: DiffBlockHandle,
    idx: c_int,
    line1: LinenrT,
    line2: LinenrT,
) -> bool {
    // Don't create new blocks during ex_diffgetput operations
    if nvim_diff_is_busy() != 0 {
        return false;
    }

    // Check if change is after the end of dprev (if it exists)
    let after_prev = if dprev.is_null() {
        true
    } else {
        let prev_end = nvim_diff_get_lnum(dprev, idx) + nvim_diff_get_count(dprev, idx);
        prev_end < line1
    };

    // Check if change is before the start of dp (if it exists)
    let before_next = if dp.is_null() {
        true
    } else {
        let dp_start = nvim_diff_get_lnum(dp, idx);
        dp_start - 1 > line2 || (line2 == MAXLNUM && dp_start > line1)
    };

    after_prev && before_next
}

/// Create a new diff block between two existing blocks.
unsafe fn create_diff_block(
    tp: TabpageHandle,
    dprev: DiffBlockHandle,
    dp: DiffBlockHandle,
    idx: c_int,
    line1: LinenrT,
    state: &AdjustState,
) {
    let dnext = nvim_diff_alloc_new(tp, dprev, dp);
    if dnext.is_null() {
        return;
    }

    nvim_diff_set_lnum(dnext, idx, line1);
    nvim_diff_set_count(dnext, idx, state.inserted);

    // Set line numbers for other buffers
    for i in 0..DB_COUNT as c_int {
        let buf = nvim_tabpage_get_diffbuf(tp, i);
        if !buf.is_null() && i != idx {
            let lnum = if dprev.is_null() {
                line1
            } else {
                let prev_lnum = nvim_diff_get_lnum(dprev, i);
                let prev_count = nvim_diff_get_count(dprev, i);
                let prev_lnum_idx = nvim_diff_get_lnum(dprev, idx);
                let prev_count_idx = nvim_diff_get_count(dprev, idx);
                line1 + (prev_lnum + prev_count) - (prev_lnum_idx + prev_count_idx)
            };
            nvim_diff_set_lnum(dnext, i, lnum);
            nvim_diff_set_count(dnext, i, state.deleted);
        }
    }
}

/// Adjust a diff block for line changes during diff_busy.
unsafe fn adjust_block_busy(
    dp: DiffBlockHandle,
    idx: c_int,
    line2: LinenrT,
    amount_after: LinenrT,
) {
    let dp_lnum = nvim_diff_get_lnum(dp, idx);
    if dp_lnum > line2 {
        nvim_diff_set_lnum(dp, idx, dp_lnum + amount_after);
    }
}

/// Adjust a diff block that is after line2.
///
/// Returns true if we should stop processing.
unsafe fn adjust_block_after(
    dp: DiffBlockHandle,
    idx: c_int,
    line2: LinenrT,
    amount_after: LinenrT,
    state: &AdjustState,
) -> bool {
    let dp_lnum = nvim_diff_get_lnum(dp, idx);
    let adjustment = state.deleted + state.inserted != 0;

    if dp_lnum - i32::from(adjustment) > line2 {
        if amount_after == 0 {
            // Nothing left to change
            return true;
        }
        nvim_diff_set_lnum(dp, idx, dp_lnum + amount_after);
    }
    false
}

// =============================================================================
// Main Entry Point
// =============================================================================

/// Update line numbers in tab page for the buffer with index idx.
///
/// This is called when lines are inserted/deleted in a buffer that is part
/// of a diff. It attempts to update the diff blocks as much as possible:
/// - When inserting/deleting outside existing blocks, create a new block
/// - When inserting/deleting in existing blocks, update them
///
/// # Safety
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_mark_adjust_tp(
    tp: TabpageHandle,
    idx: c_int,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) {
    if tp.is_null() || idx < 0 || idx >= DB_COUNT as c_int {
        return;
    }

    // If using internal diff, just mark as invalid
    if rs_diff_internal() != 0 {
        nvim_tabpage_set_diff_invalid(tp, 1);
        nvim_tabpage_set_diff_update(tp, 1);
    }

    // Calculate adjustment state
    let mut state = AdjustState::from_params(line1, line2, amount, amount_after);

    let mut dprev = DiffBlockHandle::null();
    let mut dp = nvim_tabpage_get_first_diff(tp);

    loop {
        // Check if we should create a new diff block
        if should_create_new_block(dp, dprev, idx, line1, line2) {
            create_diff_block(tp, dprev, dp, idx, line1, &state);
        }

        // If at end of list, quit
        if dp.is_null() {
            break;
        }

        // Compute last line of this diff block
        let last = nvim_diff_get_lnum(dp, idx) + nvim_diff_get_count(dp, idx) - 1;

        // Case 1: change completely above line1 - nothing to do, move to next
        if last < line1 - 1 {
            dprev = dp;
            dp = nvim_diff_get_next(dp);
            continue;
        }

        // Handle adjustments during diff_busy
        if nvim_diff_is_busy() != 0 {
            adjust_block_busy(dp, idx, line2, amount_after);
            dprev = dp;
            dp = nvim_diff_get_next(dp);
            continue;
        }

        // Case 6: change below line2
        if adjust_block_after(dp, idx, line2, amount_after, &state) {
            break;
        }

        // Cases 2-5: changes touching this diff block
        // Handle deletion
        if state.deleted > 0 {
            let dp_lnum = nvim_diff_get_lnum(dp, idx);
            let dp_count = nvim_diff_get_count(dp, idx);

            if dp_lnum >= line1 {
                if last <= line2 {
                    // Case 4: delete all lines of diff
                    let next_dp = nvim_diff_get_next(dp);
                    if !next_dp.is_null() {
                        let next_lnum = nvim_diff_get_lnum(next_dp, idx);
                        if next_lnum - 1 <= line2 {
                            // Delete continues in next diff
                            let n = next_lnum - state.lnum_deleted;
                            state.deleted -= n;
                            state.lnum_deleted = next_lnum;
                        }
                    }
                    nvim_diff_set_count(dp, idx, 0);
                } else {
                    // Case 5: delete lines at or just before top of diff
                    let new_count = dp_count - (line2 - dp_lnum + 1);
                    nvim_diff_set_count(dp, idx, new_count);
                }
                nvim_diff_set_lnum(dp, idx, line1);
            } else if last < line2 {
                // Case 2: delete at end of diff
                let new_count = dp_count - (last - state.lnum_deleted + 1);
                nvim_diff_set_count(dp, idx, new_count);
            } else {
                // Case 3: delete in middle of diff
                let new_count = dp_count - state.deleted;
                nvim_diff_set_count(dp, idx, new_count);
            }
        }

        // Handle insertion
        if state.inserted > 0 {
            let dp_lnum = nvim_diff_get_lnum(dp, idx);
            let dp_count = nvim_diff_get_count(dp, idx);

            if dp_lnum >= line1 {
                // Insertion at or before start of diff
                nvim_diff_set_lnum(dp, idx, dp_lnum + state.inserted);
            } else if last >= line1 - 1 {
                // Insertion within the diff
                nvim_diff_set_count(dp, idx, dp_count + state.inserted);
            }
        }

        dprev = dp;
        dp = nvim_diff_get_next(dp);
    }
}

/// Check if the adjustment would cross into the next diff block.
///
/// Helper for determining when deletion spans multiple blocks.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_adjustment_crosses_block(
    dp: DiffBlockHandle,
    idx: c_int,
    lnum_deleted: LinenrT,
    line2: LinenrT,
) -> bool {
    if dp.is_null() {
        return false;
    }

    let next_dp = nvim_diff_get_next(dp);
    if next_dp.is_null() {
        return false;
    }

    let next_lnum = nvim_diff_get_lnum(next_dp, idx);
    next_lnum - 1 <= line2 && lnum_deleted < next_lnum
}

/// Get the continuation parameters when deletion crosses into next block.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_get_continuation(
    dp: DiffBlockHandle,
    idx: c_int,
    lnum_deleted: LinenrT,
) -> LinenrT {
    if dp.is_null() {
        return 0;
    }

    let next_dp = nvim_diff_get_next(dp);
    if next_dp.is_null() {
        return 0;
    }

    let next_lnum = nvim_diff_get_lnum(next_dp, idx);
    next_lnum - lnum_deleted
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjust_state_insert() {
        let state = AdjustState::from_params(10, MAXLNUM, 5, 0);
        assert_eq!(state.inserted, 5);
        assert_eq!(state.deleted, 0);
        assert_eq!(state.lnum_deleted, 10);
    }

    #[test]
    fn test_adjust_state_change_insert() {
        let state = AdjustState::from_params(10, 15, MAXLNUM, 3);
        assert_eq!(state.inserted, 3);
        assert_eq!(state.deleted, 0);
        assert_eq!(state.lnum_deleted, 10);
    }

    #[test]
    fn test_adjust_state_delete() {
        let state = AdjustState::from_params(10, 15, MAXLNUM, -5);
        assert_eq!(state.inserted, 0);
        assert_eq!(state.deleted, 5);
        assert_eq!(state.lnum_deleted, 10);
    }
}
