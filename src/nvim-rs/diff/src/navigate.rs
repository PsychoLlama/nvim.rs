//! Diff hunk navigation
//!
//! This module provides Rust implementations for navigating between diff hunks,
//! including finding next/previous hunks and getting hunk boundaries.

#![allow(clippy::must_use_candidate)]

use std::os::raw::c_int;

use crate::buffer::{BufHandle, DiffBlockHandle, DB_COUNT};

/// Line number type matching linenr_T (i32).
type LinenrT = i32;

/// Result constants.
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Direction for navigation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward = 1,
    Backward = -1,
}

// =============================================================================
// External C Functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    fn nvim_get_curtab_diffbuf(idx: c_int) -> BufHandle;
    fn nvim_get_diff_first_block() -> DiffBlockHandle;
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
}

// =============================================================================
// Hunk Navigation
// =============================================================================

/// Hunk boundaries result structure.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DiffHunkBounds {
    /// Start line of the hunk (inclusive).
    pub start_lnum: LinenrT,
    /// End line of the hunk (inclusive).
    pub end_lnum: LinenrT,
    /// Whether a hunk was found.
    pub found: c_int,
}

impl DiffHunkBounds {
    /// Create an empty (not found) result.
    #[must_use]
    pub const fn not_found() -> Self {
        Self {
            start_lnum: 0,
            end_lnum: 0,
            found: 0,
        }
    }

    /// Create a found result with the given bounds.
    #[must_use]
    pub const fn new(start: LinenrT, end: LinenrT) -> Self {
        Self {
            start_lnum: start,
            end_lnum: end,
            found: 1,
        }
    }
}

/// Navigation result with target line number and status.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DiffNavResult {
    /// Target line number (0 if not found).
    pub lnum: LinenrT,
    /// Status: OK if found, FAIL if not.
    pub status: c_int,
}

impl DiffNavResult {
    /// Create a not found result.
    #[must_use]
    pub const fn not_found() -> Self {
        Self {
            lnum: 0,
            status: FAIL,
        }
    }

    /// Create a found result.
    #[must_use]
    pub const fn found(lnum: LinenrT) -> Self {
        Self { lnum, status: OK }
    }
}

/// Find the next diff hunk after a given line number.
///
/// Returns the line number of the next hunk's start, or 0 if not found.
pub fn diff_find_next_hunk(buf_idx: c_int, lnum: LinenrT) -> LinenrT {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return 0;
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // If this block starts after our line, it's the next hunk
            if block_lnum > lnum {
                return block_lnum;
            }

            // If we're inside this block, find the next one
            let block_end = block_lnum + block_count.max(1) - 1;
            if lnum >= block_lnum && lnum <= block_end {
                // We're inside this hunk, find the next one
                let next_dp = nvim_diffblock_get_next(dp);
                if !next_dp.is_null() {
                    return nvim_diffblock_get_lnum(next_dp, buf_idx);
                }
                return 0;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        0
    }
}

/// Find the previous diff hunk before a given line number.
///
/// Returns the line number of the previous hunk's start, or 0 if not found.
pub fn diff_find_prev_hunk(buf_idx: c_int, lnum: LinenrT) -> LinenrT {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return 0;
    }

    unsafe {
        let mut prev_lnum: LinenrT = 0;
        let mut dp = nvim_get_diff_first_block();

        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // If this block starts at or after our line, return the previous one
            if block_lnum >= lnum {
                return prev_lnum;
            }

            // If we're inside this block, return the previous one
            let block_end = block_lnum + block_count.max(1) - 1;
            if lnum <= block_end {
                return prev_lnum;
            }

            prev_lnum = block_lnum;
            dp = nvim_diffblock_get_next(dp);
        }

        // If we've gone through all blocks and lnum is after them, return the last one
        prev_lnum
    }
}

/// Check if a line is inside a diff hunk.
///
/// Returns true if the line is within a diff block, false otherwise.
pub fn diff_lnum_in_hunk(buf_idx: c_int, lnum: LinenrT) -> bool {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return false;
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // If this block is past our line, stop searching
            if block_lnum > lnum {
                return false;
            }

            // Check if we're in this block
            let block_end = block_lnum + block_count.max(1) - 1;
            if lnum >= block_lnum && lnum <= block_end {
                return true;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        false
    }
}

/// Get the start and end lines of the hunk at a given position.
///
/// If the line is not in a hunk, returns a not_found result.
pub fn diff_hunk_start_end(buf_idx: c_int, lnum: LinenrT) -> DiffHunkBounds {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return DiffHunkBounds::not_found();
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // If this block is past our line, stop searching
            if block_lnum > lnum {
                return DiffHunkBounds::not_found();
            }

            // Check if we're in this block
            let block_end = block_lnum + block_count.max(1) - 1;
            if lnum >= block_lnum && lnum <= block_end {
                return DiffHunkBounds::new(block_lnum, block_end);
            }

            dp = nvim_diffblock_get_next(dp);
        }
        DiffHunkBounds::not_found()
    }
}

/// Move to the next or previous diff hunk.
///
/// This implements the navigation logic for `[c` and `]c` commands.
///
/// # Arguments
/// * `buf_idx` - Index of the buffer in the diff list
/// * `lnum` - Current line number
/// * `dir` - Direction to move (Forward or Backward)
/// * `count` - Number of hunks to move
///
/// Returns the target line number or 0 if no hunk found.
pub fn diff_move_to_hunk(buf_idx: c_int, lnum: LinenrT, dir: Direction, count: c_int) -> LinenrT {
    if !(0..DB_COUNT).contains(&buf_idx) || count <= 0 {
        return 0;
    }

    let mut target_lnum = lnum;
    let mut remaining = count;

    while remaining > 0 {
        let next_lnum = match dir {
            Direction::Forward => diff_find_next_hunk(buf_idx, target_lnum),
            Direction::Backward => diff_find_prev_hunk(buf_idx, target_lnum),
        };

        if next_lnum == 0 {
            // No more hunks in this direction
            break;
        }

        target_lnum = next_lnum;
        remaining -= 1;
    }

    if target_lnum == lnum {
        // Didn't move, return 0 to indicate failure
        0
    } else {
        target_lnum
    }
}

/// Get the diff block handle at a given line.
///
/// Returns the diff block containing the line or null if not in a diff block.
pub fn diff_get_block_at_line(buf_idx: c_int, lnum: LinenrT) -> DiffBlockHandle {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return DiffBlockHandle::null();
    }

    unsafe {
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);

            // Check if lnum is within this block
            if lnum >= block_lnum && lnum < block_lnum + block_count.max(1) {
                return dp;
            }

            // If we've passed the line, stop searching
            if block_lnum > lnum {
                break;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        DiffBlockHandle::null()
    }
}

/// Count the total number of diff hunks.
pub fn diff_count_hunks(buf_idx: c_int) -> c_int {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return 0;
    }

    unsafe {
        let mut count = 0;
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            count += 1;
            dp = nvim_diffblock_get_next(dp);
        }
        count
    }
}

/// Get the hunk index at a given line (1-based).
///
/// Returns 0 if not in a hunk.
pub fn diff_get_hunk_index(buf_idx: c_int, lnum: LinenrT) -> c_int {
    if !(0..DB_COUNT).contains(&buf_idx) {
        return 0;
    }

    unsafe {
        let mut index = 0;
        let mut dp = nvim_get_diff_first_block();
        while !dp.is_null() {
            index += 1;
            let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
            let block_count = nvim_diffblock_get_count(dp, buf_idx);
            let block_end = block_lnum + block_count.max(1) - 1;

            if lnum >= block_lnum && lnum <= block_end {
                return index;
            }

            if block_lnum > lnum {
                return 0;
            }

            dp = nvim_diffblock_get_next(dp);
        }
        0
    }
}

// FFI exports removed - the main ones are in lib.rs to maintain existing API compatibility.
// Additional exports that aren't in lib.rs:

/// FFI export: Move to hunk in a direction.
#[no_mangle]
pub extern "C" fn rs_diff_move_to_hunk(
    buf_idx: c_int,
    lnum: LinenrT,
    dir: c_int,
    count: c_int,
) -> LinenrT {
    let direction = if dir > 0 {
        Direction::Forward
    } else {
        Direction::Backward
    };
    diff_move_to_hunk(buf_idx, lnum, direction, count)
}

/// FFI export: Get diff block at line.
#[no_mangle]
pub extern "C" fn rs_diff_get_block_at_line(buf_idx: c_int, lnum: LinenrT) -> DiffBlockHandle {
    diff_get_block_at_line(buf_idx, lnum)
}

/// FFI export: Count total hunks.
#[no_mangle]
pub extern "C" fn rs_diff_count_hunks(buf_idx: c_int) -> c_int {
    diff_count_hunks(buf_idx)
}

/// FFI export: Get hunk index at line.
#[no_mangle]
pub extern "C" fn rs_diff_get_hunk_index(buf_idx: c_int, lnum: LinenrT) -> c_int {
    diff_get_hunk_index(buf_idx, lnum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_hunk_bounds_not_found() {
        let bounds = DiffHunkBounds::not_found();
        assert_eq!(bounds.start_lnum, 0);
        assert_eq!(bounds.end_lnum, 0);
        assert_eq!(bounds.found, 0);
    }

    #[test]
    fn test_diff_hunk_bounds_new() {
        let bounds = DiffHunkBounds::new(10, 20);
        assert_eq!(bounds.start_lnum, 10);
        assert_eq!(bounds.end_lnum, 20);
        assert_eq!(bounds.found, 1);
    }

    #[test]
    fn test_diff_nav_result() {
        let not_found = DiffNavResult::not_found();
        assert_eq!(not_found.lnum, 0);
        assert_eq!(not_found.status, FAIL);

        let found = DiffNavResult::found(42);
        assert_eq!(found.lnum, 42);
        assert_eq!(found.status, OK);
    }

    #[test]
    fn test_direction_values() {
        assert_eq!(Direction::Forward as c_int, 1);
        assert_eq!(Direction::Backward as c_int, -1);
    }

    #[test]
    fn test_hunk_bounds_size() {
        // Should be 3 * 4 = 12 bytes
        assert_eq!(std::mem::size_of::<DiffHunkBounds>(), 12);
    }

    #[test]
    fn test_nav_result_size() {
        // Should be 2 * 4 = 8 bytes
        assert_eq!(std::mem::size_of::<DiffNavResult>(), 8);
    }
}
