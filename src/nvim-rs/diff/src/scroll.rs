//! Diff scroll binding utilities
//!
//! This module provides structures and helpers for synchronizing scroll
//! positions between diff windows. It handles:
//! - Topline synchronization between windows
//! - Filler line calculations for alignment
//! - Topfill adjustments
//! - Botfill handling

use std::cmp::Ordering;
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
// Scroll State
// =============================================================================

/// State for scroll synchronization between diff windows.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffScrollState {
    /// Index of source buffer in diff
    pub from_idx: c_int,
    /// Index of target buffer in diff
    pub to_idx: c_int,
    /// Source window topline
    pub from_topline: LinenrT,
    /// Source window topfill
    pub from_topfill: LinenrT,
    /// Computed target topline
    pub to_topline: LinenrT,
    /// Computed target topfill
    pub to_topfill: LinenrT,
    /// Whether target needs botfill
    pub to_botfill: bool,
    /// Whether the state is valid
    pub valid: bool,
}

impl Default for DiffScrollState {
    fn default() -> Self {
        Self {
            from_idx: -1,
            to_idx: -1,
            from_topline: 1,
            from_topfill: 0,
            to_topline: 1,
            to_topfill: 0,
            to_botfill: false,
            valid: false,
        }
    }
}

/// Initialize scroll state from window indices and topline.
#[no_mangle]
pub extern "C" fn rs_diff_scroll_state_init(
    from_idx: c_int,
    to_idx: c_int,
    from_topline: LinenrT,
    from_topfill: LinenrT,
) -> DiffScrollState {
    if from_idx < 0 || from_idx >= DB_COUNT as c_int || to_idx < 0 || to_idx >= DB_COUNT as c_int {
        return DiffScrollState::default();
    }

    DiffScrollState {
        from_idx,
        to_idx,
        from_topline,
        from_topfill,
        to_topline: 1,
        to_topfill: 0,
        to_botfill: false,
        valid: true,
    }
}

/// Check if scroll state is valid.
#[no_mangle]
pub const extern "C" fn rs_diff_scroll_state_valid(state: &DiffScrollState) -> bool {
    state.valid
}

// =============================================================================
// Block Finding for Scroll
// =============================================================================

/// Find the diff block containing a line for scroll synchronization.
///
/// Returns the block that contains or is after the given line.
///
/// # Safety
/// `first_dp` must be a valid diff block handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_find_block_for_scroll(
    first_dp: DiffBlockHandle,
    from_idx: c_int,
    lnum: LinenrT,
) -> DiffBlockHandle {
    if first_dp.is_null() || from_idx < 0 || from_idx >= DB_COUNT as c_int {
        return DiffBlockHandle::null();
    }

    let mut dp = first_dp;
    while !dp.is_null() {
        let block_lnum = nvim_diffblock_get_lnum(dp, from_idx);
        let block_count = nvim_diffblock_get_count(dp, from_idx);

        // Block includes or is after lnum
        if lnum <= block_lnum + block_count {
            return dp;
        }

        dp = nvim_diffblock_get_next(dp);
    }

    // No block found - after all blocks
    DiffBlockHandle::null()
}

// =============================================================================
// Topline Calculation
// =============================================================================

/// Result of topline calculation for scroll binding.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ToplineResult {
    /// Computed topline for target window
    pub topline: LinenrT,
    /// Computed topfill for target window
    pub topfill: LinenrT,
    /// Whether botfill is needed
    pub botfill: bool,
    /// Whether calculation was valid
    pub valid: bool,
}

/// Calculate topline when no diff block is found (after all changes).
///
/// Computes topline relative to end of file.
#[no_mangle]
pub const extern "C" fn rs_diff_calc_topline_after_changes(
    from_lnum: LinenrT,
    from_line_count: LinenrT,
    to_line_count: LinenrT,
) -> ToplineResult {
    let topline = to_line_count - (from_line_count - from_lnum);

    ToplineResult {
        topline,
        topfill: 0,
        botfill: false,
        valid: true,
    }
}

/// Calculate base topline offset from diff block.
///
/// Returns the topline before filler line adjustment.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_calc_topline_from_block(
    dp: DiffBlockHandle,
    from_idx: c_int,
    to_idx: c_int,
    from_lnum: LinenrT,
) -> LinenrT {
    if dp.is_null()
        || from_idx < 0
        || from_idx >= DB_COUNT as c_int
        || to_idx < 0
        || to_idx >= DB_COUNT as c_int
    {
        return from_lnum;
    }

    let from_block_lnum = nvim_diffblock_get_lnum(dp, from_idx);
    let to_block_lnum = nvim_diffblock_get_lnum(dp, to_idx);

    from_lnum + (to_block_lnum - from_block_lnum)
}

// =============================================================================
// Topfill Calculation
// =============================================================================

/// Result of topfill and adjusted topline calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TopfillResult {
    /// Adjusted topline
    pub topline: LinenrT,
    /// Computed topfill
    pub topfill: LinenrT,
    /// Whether calculation was valid
    pub valid: bool,
}

/// Calculate topfill and adjust topline based on diff block.
///
/// This handles the complex filler line calculation when scrolling
/// within a diff block.
///
/// # Safety
/// `dp` must be a valid diff block handle.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_calc_topfill_for_scroll(
    dp: DiffBlockHandle,
    from_idx: c_int,
    to_idx: c_int,
    from_topline: LinenrT,
    from_topfill: LinenrT,
    base_to_topline: LinenrT,
) -> TopfillResult {
    if dp.is_null()
        || from_idx < 0
        || from_idx >= DB_COUNT as c_int
        || to_idx < 0
        || to_idx >= DB_COUNT as c_int
    {
        return TopfillResult {
            topline: base_to_topline,
            topfill: 0,
            valid: false,
        };
    }

    let from_block_lnum = nvim_diffblock_get_lnum(dp, from_idx);
    let from_block_count = nvim_diffblock_get_count(dp, from_idx);
    let to_block_lnum = nvim_diffblock_get_lnum(dp, to_idx);
    let to_block_count = nvim_diffblock_get_count(dp, to_idx);

    // Offset within the from block
    let from_offset = from_topline - from_block_lnum;

    // Max lines available in each block
    let max_from = from_block_count.max(0);
    let max_to = to_block_count.max(0);

    // Calculate virtual line position (including filler lines)
    // In the from window, we're at line from_offset with from_topfill filler above
    let virtual_pos = if from_offset >= 0 {
        from_offset + from_topfill
    } else {
        from_topfill
    };

    // Translate virtual position to to window
    let (to_topline, to_topfill) = match max_to.cmp(&max_from) {
        Ordering::Greater => {
            // Target has more lines, may need filler at bottom of from
            if virtual_pos < max_to {
                // Within the target's block
                (to_block_lnum + virtual_pos.min(max_to - 1), 0)
            } else {
                // Past the target's block
                (to_block_lnum + max_to.saturating_sub(1).max(0), 0)
            }
        }
        Ordering::Less => {
            // From has more lines, target needs filler lines
            if virtual_pos < max_to {
                // Within the target's real lines
                (to_block_lnum + virtual_pos, 0)
            } else {
                // In filler region of target
                let topline = to_block_lnum + max_to.saturating_sub(1).max(0);
                // Filler lines needed
                let topfill = (virtual_pos - max_to + 1).max(0);
                (topline, topfill)
            }
        }
        Ordering::Equal => {
            // Equal number of lines
            (
                to_block_lnum + from_offset.min(max_to.saturating_sub(1).max(0)),
                from_topfill,
            )
        }
    };

    TopfillResult {
        topline: to_topline,
        topfill: to_topfill,
        valid: true,
    }
}

// =============================================================================
// Safety Bounds Checking
// =============================================================================

/// Result of applying safety bounds to topline.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BoundsResult {
    /// Adjusted topline (clamped to valid range)
    pub topline: LinenrT,
    /// Adjusted topfill
    pub topfill: LinenrT,
    /// Whether botfill is needed (topline was beyond file end)
    pub botfill: bool,
}

/// Apply safety bounds to computed topline values.
#[no_mangle]
pub const extern "C" fn rs_diff_apply_topline_bounds(
    topline: LinenrT,
    topfill: LinenrT,
    line_count: LinenrT,
) -> BoundsResult {
    let mut result = BoundsResult {
        topline,
        topfill,
        botfill: false,
    };

    if result.topline > line_count {
        result.topline = line_count;
        result.botfill = true;
    }

    if result.topline < 1 {
        result.topline = 1;
        result.topfill = 0;
    }

    result
}

// =============================================================================
// Scroll Binding Needed Flag
// =============================================================================

/// State for tracking when scroll binding needs update.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollBindState {
    /// Whether scroll binding update is needed
    pub need_update: bool,
    /// Number of windows that need update
    pub window_count: c_int,
}

/// Initialize scroll bind state.
#[no_mangle]
pub extern "C" fn rs_diff_scrollbind_init() -> ScrollBindState {
    ScrollBindState::default()
}

/// Mark that scroll binding needs update.
#[no_mangle]
pub const extern "C" fn rs_diff_scrollbind_set_needed(state: &mut ScrollBindState) {
    state.need_update = true;
}

/// Clear scroll binding need flag.
#[no_mangle]
pub const extern "C" fn rs_diff_scrollbind_clear(state: &mut ScrollBindState) {
    state.need_update = false;
    state.window_count = 0;
}

/// Check if scroll binding needs update.
#[no_mangle]
pub const extern "C" fn rs_diff_scrollbind_needed(state: &ScrollBindState) -> bool {
    state.need_update
}

// =============================================================================
// Filler Line Position
// =============================================================================

/// Position information for filler lines at a specific line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FillerPosition {
    /// Number of filler lines above this line
    pub above: LinenrT,
    /// Number of filler lines below this line (in same block)
    pub below: LinenrT,
    /// Whether this line is in a diff block
    pub in_block: bool,
}

/// Get filler line position information.
///
/// # Safety
/// `first_dp` must be a valid diff block handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_get_filler_position(
    first_dp: DiffBlockHandle,
    buf_idx: c_int,
    lnum: LinenrT,
) -> FillerPosition {
    if first_dp.is_null() || buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        return FillerPosition::default();
    }

    let mut dp = first_dp;
    while !dp.is_null() {
        let block_lnum = nvim_diffblock_get_lnum(dp, buf_idx);
        let block_count = nvim_diffblock_get_count(dp, buf_idx);

        // Check if lnum is within this block
        if lnum >= block_lnum && lnum < block_lnum + block_count {
            // Find max count across all buffers to calculate filler
            let mut max_count: LinenrT = 0;
            for i in 0..DB_COUNT as c_int {
                let count = nvim_diffblock_get_count(dp, i);
                if count > max_count {
                    max_count = count;
                }
            }

            let lines_in_block = lnum - block_lnum + 1;
            let filler_total = max_count - block_count;

            return FillerPosition {
                above: 0, // Would need more context to calculate exact position
                below: if lines_in_block == block_count {
                    filler_total
                } else {
                    0
                },
                in_block: true,
            };
        }

        // Check if before this block
        if lnum < block_lnum {
            break;
        }

        dp = nvim_diffblock_get_next(dp);
    }

    FillerPosition::default()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_state_default() {
        let state = DiffScrollState::default();
        assert_eq!(state.from_idx, -1);
        assert_eq!(state.to_idx, -1);
        assert!(!state.valid);
    }

    #[test]
    fn test_scroll_state_init_valid() {
        let state = rs_diff_scroll_state_init(0, 1, 10, 2);
        assert!(state.valid);
        assert_eq!(state.from_idx, 0);
        assert_eq!(state.to_idx, 1);
        assert_eq!(state.from_topline, 10);
        assert_eq!(state.from_topfill, 2);
    }

    #[test]
    fn test_scroll_state_init_invalid() {
        let state = rs_diff_scroll_state_init(-1, 0, 10, 2);
        assert!(!state.valid);

        let state = rs_diff_scroll_state_init(0, 10, 10, 2);
        assert!(!state.valid);
    }

    #[test]
    fn test_calc_topline_after_changes() {
        // File with 100 lines, at line 90, target has 80 lines
        let result = rs_diff_calc_topline_after_changes(90, 100, 80);
        assert!(result.valid);
        // 80 - (100 - 90) = 80 - 10 = 70
        assert_eq!(result.topline, 70);
        assert_eq!(result.topfill, 0);
    }

    #[test]
    fn test_apply_topline_bounds_normal() {
        let result = rs_diff_apply_topline_bounds(50, 2, 100);
        assert_eq!(result.topline, 50);
        assert_eq!(result.topfill, 2);
        assert!(!result.botfill);
    }

    #[test]
    fn test_apply_topline_bounds_too_high() {
        let result = rs_diff_apply_topline_bounds(150, 2, 100);
        assert_eq!(result.topline, 100);
        assert!(result.botfill);
    }

    #[test]
    fn test_apply_topline_bounds_too_low() {
        let result = rs_diff_apply_topline_bounds(0, 2, 100);
        assert_eq!(result.topline, 1);
        assert_eq!(result.topfill, 0);
    }

    #[test]
    fn test_scrollbind_state() {
        let mut state = rs_diff_scrollbind_init();
        assert!(!rs_diff_scrollbind_needed(&state));

        rs_diff_scrollbind_set_needed(&mut state);
        assert!(rs_diff_scrollbind_needed(&state));

        rs_diff_scrollbind_clear(&mut state);
        assert!(!rs_diff_scrollbind_needed(&state));
    }

    #[test]
    fn test_filler_position_default() {
        let pos = FillerPosition::default();
        assert_eq!(pos.above, 0);
        assert_eq!(pos.below, 0);
        assert!(!pos.in_block);
    }
}
