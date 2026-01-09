//! Diff Ex command implementations
//!
//! This module provides Rust implementations for diff-related Ex commands
//! like :diffget, :diffput, and related operations.

#![allow(clippy::must_use_candidate)]

use std::os::raw::c_int;

use crate::buffer::{BufHandle, DiffBlockHandle, DB_COUNT};

/// Line number type matching linenr_T (i32).
type LinenrT = i32;

/// Result constants.
const OK: c_int = 1;
const FAIL: c_int = 0;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_curtab_diffbuf(idx: c_int) -> BufHandle;
    fn nvim_get_diff_first_block() -> DiffBlockHandle;
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;
}

// =============================================================================
// Diff Get/Put Operations
// =============================================================================

/// Operation type for diffget/diffput.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffOperation {
    /// Get changes from another buffer.
    Get = 0,
    /// Put changes to another buffer.
    Put = 1,
}

/// Result of a diff get/put operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffOpResult {
    /// Status of the operation (OK or FAIL).
    pub status: c_int,
    /// Number of lines affected.
    pub lines_changed: LinenrT,
    /// First line affected.
    pub first_line: LinenrT,
    /// Last line affected.
    pub last_line: LinenrT,
}

impl DiffOpResult {
    /// Create a failed result.
    #[must_use]
    pub const fn fail() -> Self {
        Self {
            status: FAIL,
            lines_changed: 0,
            first_line: 0,
            last_line: 0,
        }
    }

    /// Create a success result.
    #[must_use]
    pub const fn success(lines: LinenrT, first: LinenrT, last: LinenrT) -> Self {
        Self {
            status: OK,
            lines_changed: lines,
            first_line: first,
            last_line: last,
        }
    }
}

/// Range for diff operations.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffRange {
    /// First line (inclusive).
    pub first: LinenrT,
    /// Last line (inclusive).
    pub last: LinenrT,
}

impl DiffRange {
    /// Create a new range.
    #[must_use]
    pub const fn new(first: LinenrT, last: LinenrT) -> Self {
        Self { first, last }
    }

    /// Create an empty range.
    #[must_use]
    pub const fn empty() -> Self {
        Self { first: 0, last: 0 }
    }

    /// Check if the range is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.first > 0 && self.last >= self.first
    }

    /// Get the number of lines in the range.
    #[must_use]
    pub const fn count(&self) -> LinenrT {
        if self.is_valid() {
            self.last - self.first + 1
        } else {
            0
        }
    }
}

// =============================================================================
// Diff Block Selection
// =============================================================================

/// Find the diff block(s) that overlap with a line range.
///
/// Returns the first and last diff blocks that overlap with the range.
pub fn diff_find_blocks_in_range(
    buf_idx: c_int,
    range: DiffRange,
) -> (DiffBlockHandle, DiffBlockHandle) {
    if !(0..DB_COUNT).contains(&buf_idx) || !range.is_valid() {
        return (DiffBlockHandle::null(), DiffBlockHandle::null());
    }

    unsafe {
        let mut first_dp = DiffBlockHandle::null();
        let mut last_dp = DiffBlockHandle::null();

        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);
            let block_end = block_lnum + block_count.max(1) - 1;

            // Check if this block overlaps with the range
            if block_end >= range.first && block_lnum <= range.last {
                if first_dp.is_null() {
                    first_dp = dp;
                }
                last_dp = dp;
            }

            // If we've passed the range, stop
            if block_lnum > range.last {
                break;
            }

            dp = nvim_diffblock_get_next(dp);
        }

        (first_dp, last_dp)
    }
}

/// Count the number of diff blocks in a range.
pub fn diff_count_blocks_in_range(buf_idx: c_int, range: DiffRange) -> c_int {
    if !(0..DB_COUNT).contains(&buf_idx) || !range.is_valid() {
        return 0;
    }

    unsafe {
        let mut count = 0;
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);
            let block_end = block_lnum + block_count.max(1) - 1;

            // Check if this block overlaps with the range
            if block_end >= range.first && block_lnum <= range.last {
                count += 1;
            }

            // If we've passed the range, stop
            if block_lnum > range.last {
                break;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        count
    }
}

// =============================================================================
// Diff Validation
// =============================================================================

/// Validate that a diff block is usable for get/put operations.
pub fn diff_validate_block(dp: DiffBlockHandle) -> bool {
    if dp.is_null() {
        return false;
    }

    unsafe {
        // Check that at least two buffers have content
        let mut valid_count = 0;
        for i in 0..DB_COUNT {
            let buf = nvim_get_curtab_diffbuf(i);
            if !buf.is_null() {
                valid_count += 1;
            }
        }
        valid_count >= 2
    }
}

/// Find the source buffer for a diffget operation.
///
/// If there's only one other buffer in diff mode, use that.
/// Otherwise, return -1 to indicate ambiguity.
pub fn diff_find_source_buffer(cur_idx: c_int) -> c_int {
    if !(0..DB_COUNT).contains(&cur_idx) {
        return -1;
    }

    unsafe {
        let mut source_idx = -1;
        let mut count = 0;

        for i in 0..DB_COUNT {
            if i != cur_idx && !nvim_get_curtab_diffbuf(i).is_null() {
                source_idx = i;
                count += 1;
            }
        }

        // Only return a source if there's exactly one other buffer
        if count == 1 {
            source_idx
        } else {
            -1
        }
    }
}

/// Calculate the line adjustment after a diff operation.
///
/// Returns the number of lines added (positive) or removed (negative).
pub fn diff_calc_line_adjustment(dp: DiffBlockHandle, idx_from: c_int, idx_to: c_int) -> LinenrT {
    if dp.is_null() {
        return 0;
    }

    unsafe {
        let count_from = nvim_diffblock_get_count(dp, idx_from);
        let count_to = nvim_diffblock_get_count(dp, idx_to);
        count_from - count_to
    }
}

// =============================================================================
// Corresponding Line Calculation
// =============================================================================

/// Calculate the corresponding line in another buffer.
///
/// This is used to position the cursor after switching between diff buffers.
pub fn diff_get_corresponding_line(from_idx: c_int, to_idx: c_int, lnum: LinenrT) -> LinenrT {
    if !(0..DB_COUNT).contains(&from_idx) || !(0..DB_COUNT).contains(&to_idx) {
        return lnum;
    }

    unsafe {
        let mut baseline: LinenrT = 0;
        let mut dp = nvim_get_diff_first_block();

        while !dp.is_null() {
            let from_lnum = nvim_diffblock_get_lnum(dp, from_idx);
            let from_count = nvim_diffblock_get_count(dp, from_idx);
            let to_lnum = nvim_diffblock_get_lnum(dp, to_idx);
            let to_count = nvim_diffblock_get_count(dp, to_idx);

            if from_lnum > lnum {
                // Line is before this diff block
                return lnum - baseline;
            }

            if from_lnum + from_count > lnum {
                // Line is inside this diff block
                let offset = lnum - from_lnum;
                let adjusted_offset = offset.min(to_count);
                return to_lnum + adjusted_offset;
            }

            // Update baseline for the next iteration
            baseline = (from_lnum + from_count) - (to_lnum + to_count);
            dp = nvim_diffblock_get_next(dp);
        }

        // Line is after all diff blocks
        lnum - baseline
    }
}

/// Calculate the corresponding line, clamped to buffer bounds.
pub fn diff_get_corresponding_line_clamped(
    from_idx: c_int,
    to_idx: c_int,
    lnum: LinenrT,
) -> LinenrT {
    let result = diff_get_corresponding_line(from_idx, to_idx, lnum);

    unsafe {
        let to_buf = nvim_get_curtab_diffbuf(to_idx);
        if to_buf.is_null() {
            return result;
        }
        let max_line = nvim_buf_get_ml_line_count(to_buf);
        result.min(max_line).max(1)
    }
}

// =============================================================================
// Diff Block Information
// =============================================================================

/// Information about a diff block for commands.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffBlockInfo {
    /// Handle to the diff block.
    pub handle: DiffBlockHandle,
    /// Line numbers for each buffer.
    pub lnum: [LinenrT; DB_COUNT as usize],
    /// Line counts for each buffer.
    pub count: [LinenrT; DB_COUNT as usize],
}

impl DiffBlockInfo {
    /// Create info from a diff block handle.
    ///
    /// # Safety
    /// The handle must be valid.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub unsafe fn from_handle(dp: DiffBlockHandle) -> Self {
        let mut info = Self {
            handle: dp,
            lnum: [0; DB_COUNT as usize],
            count: [0; DB_COUNT as usize],
        };

        if !dp.is_null() {
            for i in 0..DB_COUNT {
                info.lnum[i as usize] = nvim_diffblock_get_lnum(dp, i);
                info.count[i as usize] = nvim_diffblock_get_count(dp, i);
            }
        }

        info
    }

    /// Create empty info.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            handle: DiffBlockHandle::null(),
            lnum: [0; DB_COUNT as usize],
            count: [0; DB_COUNT as usize],
        }
    }

    /// Get the end line for a buffer index.
    #[must_use]
    pub const fn end_lnum(&self, idx: usize) -> LinenrT {
        if idx < DB_COUNT as usize {
            let count_adj = self.count[idx].saturating_sub(1);
            self.lnum[idx] + if count_adj > 0 { count_adj } else { 0 }
        } else {
            0
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Find diff blocks overlapping a range.
///
/// # Safety
/// `out_first` and `out_last` must be valid pointers if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_find_blocks_in_range(
    buf_idx: c_int,
    first: LinenrT,
    last: LinenrT,
    out_first: *mut DiffBlockHandle,
    out_last: *mut DiffBlockHandle,
) {
    let range = DiffRange::new(first, last);
    let (first_dp, last_dp) = diff_find_blocks_in_range(buf_idx, range);
    if !out_first.is_null() {
        *out_first = first_dp;
    }
    if !out_last.is_null() {
        *out_last = last_dp;
    }
}

/// FFI export: Count blocks in a range.
#[no_mangle]
pub extern "C" fn rs_diff_count_blocks_in_range(
    buf_idx: c_int,
    first: LinenrT,
    last: LinenrT,
) -> c_int {
    let range = DiffRange::new(first, last);
    diff_count_blocks_in_range(buf_idx, range)
}

/// FFI export: Validate a diff block.
#[no_mangle]
pub extern "C" fn rs_diff_validate_block(dp: DiffBlockHandle) -> c_int {
    c_int::from(diff_validate_block(dp))
}

/// FFI export: Find source buffer for diffget.
#[no_mangle]
pub extern "C" fn rs_diff_find_source_buffer(cur_idx: c_int) -> c_int {
    diff_find_source_buffer(cur_idx)
}

/// FFI export: Calculate line adjustment.
#[no_mangle]
pub extern "C" fn rs_diff_calc_line_adjustment(
    dp: DiffBlockHandle,
    idx_from: c_int,
    idx_to: c_int,
) -> LinenrT {
    diff_calc_line_adjustment(dp, idx_from, idx_to)
}

/// FFI export: Get corresponding line in another buffer.
#[no_mangle]
pub extern "C" fn rs_diff_get_corresponding_line(
    from_idx: c_int,
    to_idx: c_int,
    lnum: LinenrT,
) -> LinenrT {
    diff_get_corresponding_line(from_idx, to_idx, lnum)
}

/// FFI export: Get corresponding line, clamped to buffer bounds.
#[no_mangle]
pub extern "C" fn rs_diff_get_corresponding_line_clamped(
    from_idx: c_int,
    to_idx: c_int,
    lnum: LinenrT,
) -> LinenrT {
    diff_get_corresponding_line_clamped(from_idx, to_idx, lnum)
}

/// FFI export: Get diff block info.
#[no_mangle]
pub extern "C" fn rs_diff_get_block_info(dp: DiffBlockHandle) -> DiffBlockInfo {
    unsafe { DiffBlockInfo::from_handle(dp) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_operation_values() {
        assert_eq!(DiffOperation::Get as c_int, 0);
        assert_eq!(DiffOperation::Put as c_int, 1);
    }

    #[test]
    fn test_diff_op_result_fail() {
        let result = DiffOpResult::fail();
        assert_eq!(result.status, FAIL);
        assert_eq!(result.lines_changed, 0);
    }

    #[test]
    fn test_diff_op_result_success() {
        let result = DiffOpResult::success(5, 10, 14);
        assert_eq!(result.status, OK);
        assert_eq!(result.lines_changed, 5);
        assert_eq!(result.first_line, 10);
        assert_eq!(result.last_line, 14);
    }

    #[test]
    fn test_diff_range() {
        let range = DiffRange::new(10, 20);
        assert!(range.is_valid());
        assert_eq!(range.count(), 11);

        let empty = DiffRange::empty();
        assert!(!empty.is_valid());
        assert_eq!(empty.count(), 0);

        let invalid = DiffRange::new(20, 10);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_diff_block_info_empty() {
        let info = DiffBlockInfo::empty();
        assert!(info.handle.is_null());
        for i in 0..DB_COUNT as usize {
            assert_eq!(info.lnum[i], 0);
            assert_eq!(info.count[i], 0);
        }
    }

    #[test]
    fn test_struct_sizes() {
        use std::mem::size_of;

        // DiffOpResult: 4 * 4 = 16 bytes
        assert_eq!(size_of::<DiffOpResult>(), 16);

        // DiffRange: 2 * 4 = 8 bytes
        assert_eq!(size_of::<DiffRange>(), 8);
    }
}
