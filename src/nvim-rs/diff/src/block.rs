//! Diff block management
//!
//! This module provides structures and utilities for managing diff blocks,
//! which represent contiguous regions of differing lines between buffers.
//!
//! Diff blocks form a linked list, where each block stores:
//! - Line numbers and counts for each buffer (up to DB_COUNT buffers)
//! - Whether the block has been line-matched
//! - Inline change information for character-level diffs

use std::ffi::c_int;

use crate::buffer::{BufHandle, DiffBlockHandle, TabpageHandle, DB_COUNT};

// Line number type matching linenr_T
type LinenrT = i32;

// DB_COUNT as usize for array dimensions
const DB_COUNT_USIZE: usize = DB_COUNT as usize;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Diff block accessors
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_is_linematched(dp: DiffBlockHandle) -> bool;
    fn nvim_diffblock_has_changes(dp: DiffBlockHandle) -> bool;

    // Diff block setters
    fn nvim_diffblock_set_lnum(dp: DiffBlockHandle, idx: c_int, lnum: LinenrT);
    fn nvim_diffblock_set_count(dp: DiffBlockHandle, idx: c_int, count: LinenrT);

    // Tabpage diff accessors
    fn nvim_tabpage_get_first_diff(tp: TabpageHandle) -> DiffBlockHandle;
    fn nvim_tabpage_get_diffbuf(tp: TabpageHandle, idx: c_int) -> BufHandle;
    #[allow(dead_code)]
    fn nvim_tabpage_is_diff_invalid(tp: TabpageHandle) -> bool;
}

// =============================================================================
// Diff Block Properties
// =============================================================================

/// Information about a single diff block.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffBlockInfo {
    /// Block handle
    pub handle: DiffBlockHandle,
    /// Starting line numbers for each buffer
    pub lnum: [LinenrT; DB_COUNT_USIZE],
    /// Line counts for each buffer
    pub count: [LinenrT; DB_COUNT_USIZE],
    /// Whether this block has been line-matched
    pub is_linematched: bool,
    /// Whether this block has inline changes
    pub has_changes: bool,
}

impl Default for DiffBlockInfo {
    fn default() -> Self {
        Self {
            handle: DiffBlockHandle::null(),
            lnum: [0; DB_COUNT_USIZE],
            count: [0; DB_COUNT_USIZE],
            is_linematched: false,
            has_changes: false,
        }
    }
}

/// Get information about a diff block.
///
/// # Safety
/// `dp` must be a valid diff block handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_info(dp: DiffBlockHandle) -> DiffBlockInfo {
    if dp.is_null() {
        return DiffBlockInfo::default();
    }

    let mut info = DiffBlockInfo {
        handle: dp,
        lnum: [0; DB_COUNT_USIZE],
        count: [0; DB_COUNT_USIZE],
        is_linematched: nvim_diffblock_is_linematched(dp),
        has_changes: nvim_diffblock_has_changes(dp),
    };

    for i in 0..DB_COUNT_USIZE {
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        let idx = i as c_int;
        info.lnum[i] = nvim_diffblock_get_lnum(dp, idx);
        info.count[i] = nvim_diffblock_get_count(dp, idx);
    }

    info
}

// =============================================================================
// Diff Block Iteration
// =============================================================================

/// Iterator state for walking diff blocks.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffBlockIter {
    /// Current block
    pub current: DiffBlockHandle,
    /// Previous block (for linked list operations)
    pub prev: DiffBlockHandle,
    /// Whether iteration is complete
    pub done: bool,
}

impl Default for DiffBlockIter {
    fn default() -> Self {
        Self {
            current: DiffBlockHandle::null(),
            prev: DiffBlockHandle::null(),
            done: true,
        }
    }
}

/// Create a new iterator starting at the first diff block for a tabpage.
///
/// # Safety
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_iter_new(tp: TabpageHandle) -> DiffBlockIter {
    if tp.is_null() {
        return DiffBlockIter::default();
    }

    let first = nvim_tabpage_get_first_diff(tp);
    DiffBlockIter {
        current: first,
        prev: DiffBlockHandle::null(),
        done: first.is_null(),
    }
}

/// Advance the iterator to the next diff block.
///
/// Returns true if there are more blocks, false if iteration is complete.
///
/// # Safety
/// `iter` must be a valid pointer to a `DiffBlockIter`.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_iter_next(iter: *mut DiffBlockIter) -> bool {
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

/// Check if the iterator has finished.
#[no_mangle]
pub const extern "C" fn rs_diff_block_iter_done(iter: &DiffBlockIter) -> bool {
    iter.done
}

/// Get the current block from the iterator.
#[no_mangle]
pub const extern "C" fn rs_diff_block_iter_current(iter: &DiffBlockIter) -> DiffBlockHandle {
    iter.current
}

// =============================================================================
// Diff Block Queries
// =============================================================================

/// Check if a diff block contains a given line number for a buffer.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_contains_line(
    dp: DiffBlockHandle,
    buf_idx: c_int,
    lnum: LinenrT,
) -> bool {
    if dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return false;
    }

    let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
    let block_count = nvim_diffblock_get_count(dp, buf_idx);

    lnum >= block_lnum && lnum < block_lnum + block_count
}

/// Get the total number of lines in a diff block for a buffer.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_line_count(dp: DiffBlockHandle, buf_idx: c_int) -> LinenrT {
    if dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return 0;
    }
    nvim_diffblock_get_count(dp, buf_idx)
}

/// Get the starting line number for a buffer in a diff block.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_start_line(dp: DiffBlockHandle, buf_idx: c_int) -> LinenrT {
    if dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return 0;
    }
    nvim_diffblock_get_lnum(dp, buf_idx)
}

/// Get the ending line number (exclusive) for a buffer in a diff block.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_end_line(dp: DiffBlockHandle, buf_idx: c_int) -> LinenrT {
    if dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return 0;
    }
    nvim_diffblock_get_lnum(dp, buf_idx) + nvim_diffblock_get_count(dp, buf_idx)
}

/// Check if a diff block is empty (all buffers have zero lines).
///
/// # Safety
/// `dp` must be a valid diff block handle.
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_all_empty(dp: DiffBlockHandle, tp: TabpageHandle) -> bool {
    if dp.is_null() || tp.is_null() {
        return true;
    }

    for i in 0..DB_COUNT_USIZE {
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        let idx = i as c_int;
        if !nvim_tabpage_get_diffbuf(tp, idx).is_null() && nvim_diffblock_get_count(dp, idx) > 0 {
            return false;
        }
    }

    true
}

/// Check if two diff blocks can be merged (are adjacent).
///
/// Blocks can be merged if for all buffers, the end of dp1 equals the start of dp2.
///
/// # Safety
/// `dp1` and `dp2` must be valid diff block handles.
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_blocks_can_merge(
    dp1: DiffBlockHandle,
    dp2: DiffBlockHandle,
    tp: TabpageHandle,
) -> bool {
    if dp1.is_null() || dp2.is_null() || tp.is_null() {
        return false;
    }

    for i in 0..DB_COUNT_USIZE {
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        let idx = i as c_int;
        if !nvim_tabpage_get_diffbuf(tp, idx).is_null() {
            let end1 = nvim_diffblock_get_lnum(dp1, idx) + nvim_diffblock_get_count(dp1, idx);
            let start2 = nvim_diffblock_get_lnum(dp2, idx);
            if end1 != start2 {
                return false;
            }
        }
    }

    true
}

// =============================================================================
// Diff Block Modification
// =============================================================================

/// Set the line range for a buffer in a diff block.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_set_range(
    dp: DiffBlockHandle,
    buf_idx: c_int,
    lnum: LinenrT,
    count: LinenrT,
) {
    if dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return;
    }
    nvim_diffblock_set_lnum(dp, buf_idx, lnum);
    nvim_diffblock_set_count(dp, buf_idx, count);
}

/// Adjust line numbers in a diff block after lines are inserted or deleted.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_adjust_lines(
    dp: DiffBlockHandle,
    buf_idx: c_int,
    line: LinenrT,
    amount: LinenrT,
) {
    if dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return;
    }

    let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);

    // If adjustment is before this block, shift the start line
    if line <= block_lnum {
        nvim_diffblock_set_lnum(dp, buf_idx, block_lnum + amount);
    }
}

// =============================================================================
// Diff Block Comparison
// =============================================================================

/// Compare two diff blocks by their starting line in a given buffer.
///
/// Returns:
/// - negative if dp1 starts before dp2
/// - 0 if they start at the same line
/// - positive if dp1 starts after dp2
///
/// # Safety
/// `dp1` and `dp2` must be valid diff block handles.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_compare(
    dp1: DiffBlockHandle,
    dp2: DiffBlockHandle,
    buf_idx: c_int,
) -> c_int {
    if dp1.is_null() || dp2.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return 0;
    }

    let lnum1 = nvim_diffblock_get_lnum(dp1, buf_idx);
    let lnum2 = nvim_diffblock_get_lnum(dp2, buf_idx);

    (lnum1 - lnum2) as c_int
}

// =============================================================================
// Filler Lines Calculation
// =============================================================================

/// Calculate the number of filler lines for a diff block in a buffer.
///
/// Filler lines are needed when a buffer has fewer lines in a block
/// than other buffers to maintain alignment.
///
/// # Safety
/// `dp` must be a valid diff block handle.
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_filler_count(
    dp: DiffBlockHandle,
    buf_idx: c_int,
    tp: TabpageHandle,
) -> LinenrT {
    if dp.is_null() || tp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return 0;
    }

    let my_count = nvim_diffblock_get_count(dp, buf_idx);
    let mut max_count: LinenrT = 0;

    // Find the maximum line count across all diff buffers
    for i in 0..DB_COUNT_USIZE {
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        let idx = i as c_int;
        if !nvim_tabpage_get_diffbuf(tp, idx).is_null() {
            let count = nvim_diffblock_get_count(dp, idx);
            if count > max_count {
                max_count = count;
            }
        }
    }

    // Filler lines needed = max - my_count
    if max_count > my_count {
        max_count - my_count
    } else {
        0
    }
}

/// Calculate the maximum virtual line count for a diff block.
///
/// This is the maximum line count across all buffers, representing
/// the total virtual height needed to show the block with filler lines.
///
/// # Safety
/// `dp` must be a valid diff block handle.
/// `tp` must be a valid tabpage handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_block_max_lines(
    dp: DiffBlockHandle,
    tp: TabpageHandle,
) -> LinenrT {
    if dp.is_null() || tp.is_null() {
        return 0;
    }

    let mut max_count: LinenrT = 0;

    for i in 0..DB_COUNT_USIZE {
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        let idx = i as c_int;
        if !nvim_tabpage_get_diffbuf(tp, idx).is_null() {
            let count = nvim_diffblock_get_count(dp, idx);
            if count > max_count {
                max_count = count;
            }
        }
    }

    max_count
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_block_info_default() {
        let info = DiffBlockInfo::default();
        assert!(info.handle.is_null());
        assert!(!info.is_linematched);
        assert!(!info.has_changes);
        for i in 0..DB_COUNT_USIZE {
            assert_eq!(info.lnum[i], 0);
            assert_eq!(info.count[i], 0);
        }
    }

    #[test]
    fn test_diff_block_iter_default() {
        let iter = DiffBlockIter::default();
        assert!(iter.current.is_null());
        assert!(iter.prev.is_null());
        assert!(iter.done);
    }

    #[test]
    fn test_diff_block_iter_done() {
        let iter = DiffBlockIter::default();
        assert!(rs_diff_block_iter_done(&iter));
    }

    #[test]
    fn test_diff_block_iter_current() {
        let iter = DiffBlockIter::default();
        assert!(rs_diff_block_iter_current(&iter).is_null());
    }
}
