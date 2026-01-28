//! Additional diff navigation and window options
//!
//! This module provides additional navigation utilities and window option
//! handling for diff mode. It complements the navigate module with:
//! - Window option state for diff mode
//! - Diff mode enter/exit option management
//! - Motion count handling
//! - Cursor positioning helpers

use std::ffi::c_int;

use crate::buffer::{DiffBlockHandle, DB_COUNT};

// Line number type matching linenr_T (i32)
type LinenrT = i32;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    fn nvim_diffblock_get_lnum(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_count(dp: DiffBlockHandle, idx: c_int) -> LinenrT;
    fn nvim_diffblock_get_next(dp: DiffBlockHandle) -> DiffBlockHandle;
}

// =============================================================================
// Window Option State
// =============================================================================

/// Saved window options for diff mode.
///
/// When entering diff mode, certain options are set and the previous
/// values are saved for restoration when exiting.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffWinOptions {
    /// Original 'wrap' setting
    pub saved_wrap: bool,
    /// Original 'scrollbind' setting
    pub saved_scrollbind: bool,
    /// Original 'cursorbind' setting
    pub saved_cursorbind: bool,
    /// Original 'foldmethod' (simplified to bool for ifdiff)
    pub saved_foldmethod_diff: bool,
    /// Original 'foldcolumn'
    pub saved_foldcolumn: c_int,
    /// Whether options have been saved
    pub options_saved: bool,
}

/// Initialize window options state before entering diff mode.
#[no_mangle]
pub const extern "C" fn rs_diff_win_options_save(
    wrap: bool,
    scrollbind: bool,
    cursorbind: bool,
    foldmethod_diff: bool,
    foldcolumn: c_int,
) -> DiffWinOptions {
    DiffWinOptions {
        saved_wrap: wrap,
        saved_scrollbind: scrollbind,
        saved_cursorbind: cursorbind,
        saved_foldmethod_diff: foldmethod_diff,
        saved_foldcolumn: foldcolumn,
        options_saved: true,
    }
}

/// Check if window options have been saved.
#[no_mangle]
pub const extern "C" fn rs_diff_win_options_saved(opts: &DiffWinOptions) -> bool {
    opts.options_saved
}

/// Get recommended diff mode options.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffModeOptions {
    /// Set 'wrap' to false
    pub wrap: bool,
    /// Set 'scrollbind' to true
    pub scrollbind: bool,
    /// Set 'cursorbind' to true
    pub cursorbind: bool,
    /// Set 'foldmethod' to "diff"
    pub foldmethod_diff: bool,
    /// Suggested foldcolumn value
    pub foldcolumn: c_int,
}

impl Default for DiffModeOptions {
    fn default() -> Self {
        Self {
            wrap: false,      // diff mode disables wrap
            scrollbind: true, // enable scroll binding
            cursorbind: true, // enable cursor binding
            foldmethod_diff: true,
            foldcolumn: 2, // typical diff foldcolumn
        }
    }
}

/// Get the options to set when entering diff mode.
#[no_mangle]
pub extern "C" fn rs_diff_get_enter_options() -> DiffModeOptions {
    DiffModeOptions::default()
}

// =============================================================================
// Motion Count Handling
// =============================================================================

/// Result of applying a motion count to diff navigation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MotionResult {
    /// Target line number
    pub lnum: LinenrT,
    /// Number of successful jumps made
    pub count_done: c_int,
    /// Whether target was found
    pub found: bool,
}

/// Apply motion count to find nth hunk in direction.
///
/// # Safety
/// `first_dp` must be a valid diff block handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_motion_count(
    first_dp: DiffBlockHandle,
    buf_idx: c_int,
    start_lnum: LinenrT,
    count: c_int,
    forward: bool,
) -> MotionResult {
    if first_dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int || count <= 0 {
        return MotionResult::default();
    }

    let mut result = MotionResult::default();
    let mut remaining = count;
    let mut current_lnum = start_lnum;

    if forward {
        let mut dp = first_dp;
        while !dp.is_null() && remaining > 0 {
            let block_start = nvim_diffblock_get_lnum(dp, buf_idx);

            // Only count if we're moving to a new position
            if block_start > current_lnum {
                remaining -= 1;
                result.count_done += 1;
                current_lnum = block_start;
                if remaining == 0 {
                    result.lnum = block_start;
                    result.found = true;
                    break;
                }
            }

            dp = nvim_diffblock_get_next(dp);
        }
    } else {
        // For backward motion, we need to collect blocks first
        let mut blocks: Vec<LinenrT> = Vec::new();
        let mut dp = first_dp;
        while !dp.is_null() {
            let block_start = nvim_diffblock_get_lnum(dp, buf_idx);
            if block_start < current_lnum {
                blocks.push(block_start);
            }
            dp = nvim_diffblock_get_next(dp);
        }

        // Process blocks in reverse
        for &block_start in blocks.iter().rev() {
            remaining -= 1;
            result.count_done += 1;
            if remaining == 0 {
                result.lnum = block_start;
                result.found = true;
                break;
            }
        }
    }

    result
}

// =============================================================================
// Cursor Positioning
// =============================================================================

/// Cursor position adjustment result.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CursorAdjustResult {
    /// Adjusted line number
    pub lnum: LinenrT,
    /// Adjusted column
    pub col: c_int,
    /// Whether adjustment was made
    pub adjusted: bool,
}

/// Adjust cursor position after diff operation.
///
/// When lines are added or removed, cursor position may need adjustment.
#[no_mangle]
pub const extern "C" fn rs_diff_adjust_cursor_after_op(
    cursor_lnum: LinenrT,
    cursor_col: c_int,
    op_start: LinenrT,
    lines_added: LinenrT,
    max_line: LinenrT,
) -> CursorAdjustResult {
    let mut result = CursorAdjustResult {
        lnum: cursor_lnum,
        col: cursor_col,
        adjusted: false,
    };

    // If cursor is before operation, no adjustment needed
    if cursor_lnum < op_start {
        return result;
    }

    // Adjust cursor line based on lines added/removed
    let new_lnum = cursor_lnum + lines_added;

    if new_lnum < 1 {
        result.lnum = 1;
        result.col = 0;
        result.adjusted = true;
    } else if new_lnum > max_line {
        result.lnum = max_line;
        result.adjusted = true;
    } else if new_lnum != cursor_lnum {
        result.lnum = new_lnum;
        result.adjusted = true;
    }

    result
}

// =============================================================================
// Diff Block Position
// =============================================================================

/// Position within a diff block.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockPosition {
    /// Before all diff blocks
    #[default]
    BeforeAll = 0,
    /// Inside a diff block
    Inside = 1,
    /// Between diff blocks
    Between = 2,
    /// After all diff blocks
    AfterAll = 3,
}

/// Result of checking cursor position relative to diff blocks.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BlockPositionResult {
    /// Position type
    pub position: BlockPosition,
    /// If inside or at a block, which block handle
    pub block: DiffBlockHandle,
    /// Offset within the block (if inside)
    pub offset: LinenrT,
}

impl Default for BlockPositionResult {
    fn default() -> Self {
        Self {
            position: BlockPosition::default(),
            block: DiffBlockHandle::null(),
            offset: 0,
        }
    }
}

/// Get cursor position relative to diff blocks.
///
/// # Safety
/// `first_dp` must be a valid diff block handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_get_cursor_block_position(
    first_dp: DiffBlockHandle,
    buf_idx: c_int,
    lnum: LinenrT,
) -> BlockPositionResult {
    if first_dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return BlockPositionResult::default();
    }

    let mut prev_end: LinenrT = 0;
    let mut dp = first_dp;

    while !dp.is_null() {
        let block_start = nvim_diffblock_get_lnum(dp, buf_idx);
        let block_count = nvim_diffblock_get_count(dp, buf_idx);
        let block_end = if block_count > 0 {
            block_start + block_count - 1
        } else {
            block_start
        };

        // Before this block?
        if lnum < block_start {
            if prev_end == 0 {
                return BlockPositionResult {
                    position: BlockPosition::BeforeAll,
                    block: DiffBlockHandle::null(),
                    offset: 0,
                };
            }
            return BlockPositionResult {
                position: BlockPosition::Between,
                block: DiffBlockHandle::null(),
                offset: 0,
            };
        }

        // Inside this block?
        if lnum <= block_end {
            return BlockPositionResult {
                position: BlockPosition::Inside,
                block: dp,
                offset: lnum - block_start,
            };
        }

        prev_end = block_end;
        dp = nvim_diffblock_get_next(dp);
    }

    BlockPositionResult {
        position: BlockPosition::AfterAll,
        block: DiffBlockHandle::null(),
        offset: 0,
    }
}

// =============================================================================
// Diff Status Flags
// =============================================================================

/// Check if a window is in diff mode.
///
/// This is a helper that can be used when window handle internals
/// aren't available in Rust.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowDiffStatus {
    /// Whether window is in diff mode
    pub in_diff: bool,
    /// Buffer index in diff (-1 if not in diff)
    pub buf_idx: c_int,
    /// Whether diff is outdated
    pub outdated: bool,
}

/// Create window diff status from C-provided values.
#[no_mangle]
pub const extern "C" fn rs_diff_window_status(
    in_diff: bool,
    buf_idx: c_int,
    outdated: bool,
) -> WindowDiffStatus {
    WindowDiffStatus {
        in_diff,
        buf_idx,
        outdated,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_win_options_save() {
        let opts = rs_diff_win_options_save(true, false, false, false, 0);
        assert!(opts.options_saved);
        assert!(opts.saved_wrap);
        assert!(!opts.saved_scrollbind);
    }

    #[test]
    fn test_diff_mode_options_default() {
        let opts = DiffModeOptions::default();
        assert!(!opts.wrap);
        assert!(opts.scrollbind);
        assert!(opts.cursorbind);
    }

    #[test]
    fn test_cursor_adjust_before_op() {
        let result = rs_diff_adjust_cursor_after_op(10, 5, 20, 3, 100);
        assert!(!result.adjusted);
        assert_eq!(result.lnum, 10);
    }

    #[test]
    fn test_cursor_adjust_after_add() {
        let result = rs_diff_adjust_cursor_after_op(30, 5, 20, 3, 100);
        assert!(result.adjusted);
        assert_eq!(result.lnum, 33);
    }

    #[test]
    fn test_cursor_adjust_after_remove() {
        let result = rs_diff_adjust_cursor_after_op(30, 5, 20, -3, 100);
        assert!(result.adjusted);
        assert_eq!(result.lnum, 27);
    }

    #[test]
    fn test_cursor_adjust_clamp_to_end() {
        let result = rs_diff_adjust_cursor_after_op(95, 5, 20, 10, 100);
        assert!(result.adjusted);
        assert_eq!(result.lnum, 100);
    }

    #[test]
    fn test_cursor_adjust_clamp_to_start() {
        let result = rs_diff_adjust_cursor_after_op(5, 5, 1, -10, 100);
        assert!(result.adjusted);
        assert_eq!(result.lnum, 1);
    }

    #[test]
    fn test_motion_result_default() {
        let result = MotionResult::default();
        assert!(!result.found);
        assert_eq!(result.lnum, 0);
        assert_eq!(result.count_done, 0);
    }

    #[test]
    fn test_block_position_default() {
        let result = BlockPositionResult::default();
        assert_eq!(result.position, BlockPosition::BeforeAll);
        assert!(result.block.is_null());
    }

    #[test]
    fn test_window_diff_status() {
        let status = rs_diff_window_status(true, 2, false);
        assert!(status.in_diff);
        assert_eq!(status.buf_idx, 2);
        assert!(!status.outdated);
    }
}
