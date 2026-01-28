//! Diff mark adjustment utilities
//!
//! This module provides structures and helpers for adjusting diff blocks
//! when lines are inserted or deleted in a buffer.
//!
//! The main functions `diff_mark_adjust()` and `diff_mark_adjust_tp()` in C
//! handle updating diff blocks after buffer changes. This module provides
//! the computational helpers for those operations.

use std::ffi::c_int;

use crate::buffer::{DiffBlockHandle, TabpageHandle, DB_COUNT};

// Line number type matching linenr_T (i32)
type LinenrT = i32;

// MAXLNUM constant
const MAXLNUM: LinenrT = 0x7fff_ffff;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
    fn nvim_tabpage_get_diffbuf(tp: TabpageHandle, idx: c_int) -> *mut std::ffi::c_void;
}

// =============================================================================
// Mark Adjustment Parameters
// =============================================================================

/// Parameters for a mark adjustment operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MarkAdjustParams {
    /// First line of the change (1-based).
    pub line1: LinenrT,
    /// Last line of the change (1-based), or MAXLNUM for insert.
    pub line2: LinenrT,
    /// Amount to add to line numbers above line2.
    pub amount: LinenrT,
    /// Amount to add to line numbers below line2.
    pub amount_after: LinenrT,
}

impl MarkAdjustParams {
    /// Create parameters for a line insertion.
    #[must_use]
    pub const fn insert(line: LinenrT, count: LinenrT) -> Self {
        Self {
            line1: line,
            line2: MAXLNUM,
            amount: count,
            amount_after: 0,
        }
    }

    /// Create parameters for a line deletion.
    #[must_use]
    pub const fn delete(first: LinenrT, last: LinenrT) -> Self {
        Self {
            line1: first,
            line2: last,
            amount: MAXLNUM,
            amount_after: first - last - 1,
        }
    }

    /// Create parameters for a replacement (delete + insert).
    #[must_use]
    pub const fn replace(first: LinenrT, last: LinenrT, new_count: LinenrT) -> Self {
        let deleted = last - first + 1;
        Self {
            line1: first,
            line2: last,
            amount: MAXLNUM,
            amount_after: new_count - deleted,
        }
    }
}

/// Computed change metrics from mark adjustment parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ChangeMetrics {
    /// Number of lines inserted (0 if deletion).
    pub inserted: LinenrT,
    /// Number of lines deleted (0 if insertion).
    pub deleted: LinenrT,
    /// Whether this is an insertion operation.
    pub is_insert: bool,
    /// Whether this is a deletion operation.
    pub is_delete: bool,
    /// Whether this is a change that replaces lines.
    pub is_change: bool,
}

/// Compute change metrics from mark adjustment parameters.
///
/// This extracts the inserted/deleted line counts from the raw parameters
/// used by mark_adjust().
#[no_mangle]
pub extern "C" fn rs_diff_compute_change_metrics(params: &MarkAdjustParams) -> ChangeMetrics {
    let mut metrics = ChangeMetrics::default();

    if params.line2 == MAXLNUM {
        // mark_adjust(99, MAXLNUM, 9, 0): insert lines
        metrics.inserted = params.amount;
        metrics.deleted = 0;
        metrics.is_insert = true;
    } else if params.amount_after > 0 {
        // mark_adjust(99, 98, MAXLNUM, 9): a change that inserts lines
        metrics.inserted = params.amount_after;
        metrics.deleted = 0;
        metrics.is_change = true;
    } else {
        // mark_adjust(98, 99, MAXLNUM, -2): delete lines
        metrics.inserted = 0;
        metrics.deleted = -params.amount_after;
        metrics.is_delete = true;
    }

    metrics
}

// =============================================================================
// Block Position Analysis
// =============================================================================

/// Position of a change relative to a diff block.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangePosition {
    /// Change is completely above the block.
    Above = 0,
    /// Change is completely below the block.
    Below = 1,
    /// Change touches the top of the block.
    TouchTop = 2,
    /// Change touches the bottom of the block.
    TouchBottom = 3,
    /// Change is entirely inside the block.
    Inside = 4,
    /// Change spans the entire block.
    Spans = 5,
    /// Invalid or cannot determine.
    Invalid = 6,
}

/// Analyze where a change falls relative to a diff block.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_analyze_change_position(
    dp: DiffBlockHandle,
    idx: c_int,
    line1: LinenrT,
    line2: LinenrT,
) -> ChangePosition {
    if dp.is_null() || idx < 0 || idx >= DB_COUNT as c_int {
        return ChangePosition::Invalid;
    }

    let block_start = nvim_diffblock_get_lnum(dp, idx);
    let block_count = nvim_diffblock_get_count(dp, idx);
    let block_last = block_start + block_count - 1;

    // Handle MAXLNUM (insertion)
    let effective_line2 = if line2 == MAXLNUM { line1 } else { line2 };

    if effective_line2 < block_start - 1 {
        ChangePosition::Above
    } else if line1 > block_last + 1 {
        ChangePosition::Below
    } else if line1 <= block_start && effective_line2 >= block_last {
        ChangePosition::Spans
    } else if line1 >= block_start && effective_line2 <= block_last {
        ChangePosition::Inside
    } else if effective_line2 >= block_start - 1 && effective_line2 < block_last {
        ChangePosition::TouchTop
    } else {
        ChangePosition::TouchBottom
    }
}

// =============================================================================
// Block Adjustment Helpers
// =============================================================================

/// Result of checking if a new diff block should be created.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ShouldCreateBlock {
    /// Whether a new block should be created.
    pub should_create: bool,
    /// Line number for the new block (in the changed buffer).
    pub lnum: LinenrT,
    /// Line count for the new block (in the changed buffer).
    pub count: LinenrT,
}

/// Check if a new diff block should be created for a change.
///
/// A new block is created when the change is between existing blocks
/// (not touching either).
///
/// # Safety
/// `dp` and `dprev` must be valid handles or null.
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_should_create_block(
    tp: TabpageHandle,
    dprev: DiffBlockHandle,
    dp: DiffBlockHandle,
    idx: c_int,
    line1: LinenrT,
    line2: LinenrT,
    inserted: LinenrT,
) -> ShouldCreateBlock {
    if tp.is_null() || idx < 0 || idx >= DB_COUNT as c_int {
        return ShouldCreateBlock::default();
    }

    // Check condition: change doesn't touch dp (next block)
    let dp_condition = dp.is_null()
        || nvim_diffblock_get_lnum(dp, idx) - 1 > line2
        || (line2 == MAXLNUM && nvim_diffblock_get_lnum(dp, idx) > line1);

    // Check condition: change doesn't touch dprev (previous block)
    let dprev_condition = dprev.is_null()
        || (nvim_diffblock_get_lnum(dprev, idx) + nvim_diffblock_get_count(dprev, idx) < line1);

    if dp_condition && dprev_condition {
        ShouldCreateBlock {
            should_create: true,
            lnum: line1,
            count: inserted,
        }
    } else {
        ShouldCreateBlock::default()
    }
}

/// Calculate the new line number for buffer `i` when creating a new diff block.
///
/// # Safety
/// `dprev` must be a valid handle or null.
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_calc_new_block_lnum(
    tp: TabpageHandle,
    dprev: DiffBlockHandle,
    idx: c_int,
    other_idx: c_int,
    line1: LinenrT,
) -> LinenrT {
    if tp.is_null()
        || idx < 0
        || idx >= DB_COUNT as c_int
        || other_idx < 0
        || other_idx >= DB_COUNT as c_int
    {
        return line1;
    }

    // Check if other buffer is in diff
    if nvim_tabpage_get_diffbuf(tp, other_idx).is_null() {
        return 0; // Not in diff
    }

    if dprev.is_null() {
        line1
    } else {
        // line1 + (dprev->df_lnum[i] + dprev->df_count[i])
        //       - (dprev->df_lnum[idx] + dprev->df_count[idx])
        let dprev_end_other =
            nvim_diffblock_get_lnum(dprev, other_idx) + nvim_diffblock_get_count(dprev, other_idx);
        let dprev_end_idx =
            nvim_diffblock_get_lnum(dprev, idx) + nvim_diffblock_get_count(dprev, idx);
        line1 + dprev_end_other - dprev_end_idx
    }
}

// =============================================================================
// Line Adjustment
// =============================================================================

/// Adjust a line number after a change.
///
/// Returns the adjusted line number.
#[no_mangle]
pub const extern "C" fn rs_diff_adjust_lnum(
    lnum: LinenrT,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) -> LinenrT {
    if line2 == MAXLNUM {
        // Insertion
        if lnum >= line1 {
            lnum + amount
        } else {
            lnum
        }
    } else if lnum > line2 {
        // Line is after the change
        lnum + amount_after
    } else if lnum >= line1 {
        // Line is in the deleted range
        line1
    } else {
        // Line is before the change
        lnum
    }
}

/// Check if a diff block is above a line change (no adjustment needed).
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_is_above_change(
    dp: DiffBlockHandle,
    idx: c_int,
    line1: LinenrT,
) -> bool {
    if dp.is_null() || idx < 0 || idx >= DB_COUNT as c_int {
        return false;
    }

    let block_last =
        nvim_diffblock_get_lnum(dp, idx) + nvim_diffblock_get_count(dp, idx).max(1) - 1;
    block_last < line1 - 1
}

/// Check if a diff block is below a line change.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_is_below_change(
    dp: DiffBlockHandle,
    idx: c_int,
    line2: LinenrT,
) -> bool {
    if dp.is_null() || idx < 0 || idx >= DB_COUNT as c_int {
        return false;
    }

    let block_start = nvim_diffblock_get_lnum(dp, idx);
    block_start > line2 + 1
}

// =============================================================================
// Iterator for Mark Adjustment
// =============================================================================

/// Iterator state for traversing diff blocks during mark adjustment.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MarkAdjustIter {
    /// Current diff block.
    pub current: DiffBlockHandle,
    /// Previous diff block.
    pub prev: DiffBlockHandle,
    /// Buffer index being adjusted.
    pub idx: c_int,
    /// Remaining deleted lines to process.
    pub lnum_deleted: LinenrT,
    /// Whether iteration is complete.
    pub done: bool,
}

impl Default for MarkAdjustIter {
    fn default() -> Self {
        Self {
            current: DiffBlockHandle::null(),
            prev: DiffBlockHandle::null(),
            idx: 0,
            lnum_deleted: 0,
            done: true,
        }
    }
}

/// Create a new mark adjustment iterator.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_adjust_iter_new(
    tp: TabpageHandle,
    idx: c_int,
    line1: LinenrT,
) -> MarkAdjustIter {
    extern "C" {
        fn nvim_tabpage_get_first_diff(tp: TabpageHandle) -> DiffBlockHandle;
    }

    if tp.is_null() || idx < 0 || idx >= DB_COUNT as c_int {
        return MarkAdjustIter::default();
    }

    let first = nvim_tabpage_get_first_diff(tp);
    MarkAdjustIter {
        current: first,
        prev: DiffBlockHandle::null(),
        idx,
        lnum_deleted: line1,
        done: first.is_null(),
    }
}

/// Advance the mark adjustment iterator.
///
/// Returns true if there are more blocks, false if done.
///
/// # Safety
/// `iter` must be a valid pointer to a `MarkAdjustIter`.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_adjust_iter_next(iter: *mut MarkAdjustIter) -> bool {
    if iter.is_null() {
        return false;
    }

    let it = &mut *iter;
    if it.done || it.current.is_null() {
        it.done = true;
        return false;
    }

    it.prev = it.current;
    it.current = nvim_diffblock_get_next(it.current);
    it.done = it.current.is_null();

    !it.done
}

/// Check if the mark adjustment iterator is done.
#[no_mangle]
pub const extern "C" fn rs_mark_adjust_iter_done(iter: &MarkAdjustIter) -> bool {
    iter.done
}

/// Get the current block from the iterator.
#[no_mangle]
pub const extern "C" fn rs_mark_adjust_iter_current(iter: &MarkAdjustIter) -> DiffBlockHandle {
    iter.current
}

/// Get the previous block from the iterator.
#[no_mangle]
pub const extern "C" fn rs_mark_adjust_iter_prev(iter: &MarkAdjustIter) -> DiffBlockHandle {
    iter.prev
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_adjust_params_insert() {
        let params = MarkAdjustParams::insert(10, 5);
        assert_eq!(params.line1, 10);
        assert_eq!(params.line2, MAXLNUM);
        assert_eq!(params.amount, 5);
        assert_eq!(params.amount_after, 0);
    }

    #[test]
    fn test_mark_adjust_params_delete() {
        let params = MarkAdjustParams::delete(10, 15);
        assert_eq!(params.line1, 10);
        assert_eq!(params.line2, 15);
        assert_eq!(params.amount, MAXLNUM);
        assert_eq!(params.amount_after, -6); // 10 - 15 - 1 = -6
    }

    #[test]
    fn test_mark_adjust_params_replace() {
        let params = MarkAdjustParams::replace(10, 15, 3);
        assert_eq!(params.line1, 10);
        assert_eq!(params.line2, 15);
        assert_eq!(params.amount, MAXLNUM);
        assert_eq!(params.amount_after, -3); // 3 - 6 = -3
    }

    #[test]
    fn test_change_metrics_default() {
        let metrics = ChangeMetrics::default();
        assert_eq!(metrics.inserted, 0);
        assert_eq!(metrics.deleted, 0);
        assert!(!metrics.is_insert);
        assert!(!metrics.is_delete);
        assert!(!metrics.is_change);
    }

    #[test]
    fn test_compute_change_metrics_insert() {
        let params = MarkAdjustParams::insert(10, 5);
        let metrics = rs_diff_compute_change_metrics(&params);
        assert_eq!(metrics.inserted, 5);
        assert_eq!(metrics.deleted, 0);
        assert!(metrics.is_insert);
    }

    #[test]
    fn test_compute_change_metrics_delete() {
        let params = MarkAdjustParams {
            line1: 10,
            line2: 15,
            amount: MAXLNUM,
            amount_after: -6,
        };
        let metrics = rs_diff_compute_change_metrics(&params);
        assert_eq!(metrics.inserted, 0);
        assert_eq!(metrics.deleted, 6);
        assert!(metrics.is_delete);
    }

    #[test]
    fn test_should_create_block_default() {
        let result = ShouldCreateBlock::default();
        assert!(!result.should_create);
        assert_eq!(result.lnum, 0);
        assert_eq!(result.count, 0);
    }

    #[test]
    fn test_adjust_lnum_insert() {
        // Insert 5 lines at line 10
        assert_eq!(rs_diff_adjust_lnum(5, 10, MAXLNUM, 5, 0), 5); // Before
        assert_eq!(rs_diff_adjust_lnum(10, 10, MAXLNUM, 5, 0), 15); // At
        assert_eq!(rs_diff_adjust_lnum(15, 10, MAXLNUM, 5, 0), 20); // After
    }

    #[test]
    fn test_adjust_lnum_delete() {
        // Delete lines 10-15 (amount_after = -6)
        assert_eq!(rs_diff_adjust_lnum(5, 10, 15, MAXLNUM, -6), 5); // Before
        assert_eq!(rs_diff_adjust_lnum(12, 10, 15, MAXLNUM, -6), 10); // In range -> line1
        assert_eq!(rs_diff_adjust_lnum(20, 10, 15, MAXLNUM, -6), 14); // After
    }

    #[test]
    fn test_mark_adjust_iter_default() {
        let iter = MarkAdjustIter::default();
        assert!(iter.current.is_null());
        assert!(iter.prev.is_null());
        assert_eq!(iter.idx, 0);
        assert_eq!(iter.lnum_deleted, 0);
        assert!(iter.done);
    }

    #[test]
    fn test_mark_adjust_iter_done() {
        let iter = MarkAdjustIter::default();
        assert!(rs_mark_adjust_iter_done(&iter));
    }

    #[test]
    fn test_mark_adjust_iter_current() {
        let iter = MarkAdjustIter::default();
        assert!(rs_mark_adjust_iter_current(&iter).is_null());
    }

    #[test]
    fn test_mark_adjust_iter_prev() {
        let iter = MarkAdjustIter::default();
        assert!(rs_mark_adjust_iter_prev(&iter).is_null());
    }

    #[test]
    fn test_change_position_values() {
        assert_eq!(ChangePosition::Above as c_int, 0);
        assert_eq!(ChangePosition::Below as c_int, 1);
        assert_eq!(ChangePosition::TouchTop as c_int, 2);
        assert_eq!(ChangePosition::TouchBottom as c_int, 3);
        assert_eq!(ChangePosition::Inside as c_int, 4);
        assert_eq!(ChangePosition::Spans as c_int, 5);
        assert_eq!(ChangePosition::Invalid as c_int, 6);
    }
}
