//! Diff update orchestration
//!
//! This module provides structures and helpers for coordinating diff updates
//! across windows and buffers. It handles:
//! - Invalidation tracking
//! - Redraw coordination
//! - Filler line updates
//! - Window state synchronization

use std::ffi::c_int;

use crate::buffer::{BufHandle, TabpageHandle, WinHandle, DB_COUNT};

// =============================================================================
// C FFI declarations for Phase 4 (diff_redraw migration)
// =============================================================================

extern "C" {
    // Window iteration
    fn nvim_tabpage_first_win(tp: TabpageHandle) -> WinHandle;
    fn nvim_win_next(wp: WinHandle) -> WinHandle;

    // Window field accessors
    fn nvim_win_get_p_diff(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_get_topline(wp: WinHandle) -> c_int;
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_set_topfill(wp: WinHandle, val: c_int);
    fn nvim_win_get_p_scb(wp: WinHandle) -> bool;

    // Validity checks
    fn buf_valid(buf: BufHandle) -> bool;

    // Redraw
    fn nvim_redraw_later_win(wp: WinHandle, typ: c_int);

    // Fold
    fn rs_foldmethodIsDiff(wp: WinHandle) -> c_int;
    fn rs_foldUpdateAll(wp: WinHandle);

    // Diff-specific
    fn rs_diff_check_fill(wp: WinHandle, lnum: c_int) -> c_int;
    fn nvim_check_topfill(wp: WinHandle, down: c_int);
    fn rs_diff_set_topline(fromwin: WinHandle, towin: WinHandle);

    // Global state accessors
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_set_need_diff_redraw(val: bool);
}

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
}

// =============================================================================
// Update State
// =============================================================================

/// State for tracking diff update requirements.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffUpdateState {
    /// Whether diffs need to be recalculated
    pub invalid: bool,
    /// Whether folds need to be updated
    pub update_folds: bool,
    /// Whether a redraw is needed
    pub need_redraw: bool,
    /// Number of windows that need updating
    pub window_count: c_int,
}

/// Initialize update state from tabpage flags.
#[no_mangle]
pub const extern "C" fn rs_diff_update_state_init(
    diff_invalid: bool,
    diff_update: bool,
) -> DiffUpdateState {
    DiffUpdateState {
        invalid: diff_invalid,
        update_folds: diff_update,
        need_redraw: diff_invalid || diff_update,
        window_count: 0,
    }
}

/// Mark a window as needing update.
#[no_mangle]
pub const extern "C" fn rs_diff_update_add_window(state: &mut DiffUpdateState) {
    state.window_count += 1;
    state.need_redraw = true;
}

/// Check if any updates are pending.
#[no_mangle]
pub const extern "C" fn rs_diff_update_pending(state: &DiffUpdateState) -> bool {
    state.invalid || state.update_folds || state.need_redraw
}

/// Clear update state after processing.
#[no_mangle]
pub const extern "C" fn rs_diff_update_clear(state: &mut DiffUpdateState) {
    state.invalid = false;
    state.update_folds = false;
    state.need_redraw = false;
    state.window_count = 0;
}

// =============================================================================
// Redraw Coordination
// =============================================================================

/// State for coordinating diff window redraws.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DiffRedrawState {
    /// Whether we found another diff window (not curwin)
    pub has_other_window: bool,
    /// Handle to another diff window
    pub other_window: WinHandle,
    /// Whether max fill was used in current window
    pub used_max_fill_curwin: bool,
    /// Whether max fill was used in other window
    pub used_max_fill_other: bool,
    /// Number of diff windows processed
    pub window_count: c_int,
}

impl Default for DiffRedrawState {
    fn default() -> Self {
        Self {
            has_other_window: false,
            other_window: WinHandle::null(),
            used_max_fill_curwin: false,
            used_max_fill_other: false,
            window_count: 0,
        }
    }
}

/// Initialize redraw state.
#[no_mangle]
pub extern "C" fn rs_diff_redraw_state_init() -> DiffRedrawState {
    DiffRedrawState::default()
}

/// Record a diff window during redraw iteration.
///
/// The caller should pass is_diff_window=true only for windows where
/// w_p_diff is set and the buffer is valid. This is called from C
/// where these checks are performed before calling.
#[no_mangle]
pub extern "C" fn rs_diff_redraw_record_window(
    state: &mut DiffRedrawState,
    win: WinHandle,
    curwin: WinHandle,
    is_valid_diff_window: bool,
) -> bool {
    if win.is_null() || !is_valid_diff_window {
        return false;
    }

    state.window_count += 1;

    // Track the "other" window (not current window)
    // Compare raw pointers since WinHandle wraps a pointer
    if win != curwin {
        state.has_other_window = true;
        state.other_window = win;
    }

    true
}

/// Record that max fill was used for a window.
#[no_mangle]
pub const extern "C" fn rs_diff_redraw_set_max_fill(state: &mut DiffRedrawState, is_curwin: bool) {
    if is_curwin {
        state.used_max_fill_curwin = true;
    } else {
        state.used_max_fill_other = true;
    }
}

/// Check if topline adjustment is needed after redraw.
#[no_mangle]
pub const extern "C" fn rs_diff_redraw_need_topline_adjust(state: &DiffRedrawState) -> bool {
    state.has_other_window && (state.used_max_fill_curwin || state.used_max_fill_other)
}

/// Get the window that should be the source for topline adjustment.
///
/// Returns the window whose topline should be used to adjust the other.
/// If used_max_fill_curwin is true, returns other_window (adjust curwin from other).
/// Otherwise returns null (adjust other from curwin).
#[no_mangle]
pub const extern "C" fn rs_diff_redraw_get_topline_source(state: &DiffRedrawState) -> WinHandle {
    if state.used_max_fill_curwin {
        state.other_window
    } else {
        WinHandle::null()
    }
}

// =============================================================================
// Filler Line Updates
// =============================================================================

/// Result of filler line calculation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FillerUpdateResult {
    /// New topfill value
    pub new_topfill: LinenrT,
    /// Whether topfill was changed
    pub changed: bool,
    /// Whether max fill was applied
    pub used_max_fill: bool,
}

/// Calculate filler line update for a window.
///
/// Given the current topfill and the available filler count,
/// determine the new topfill value.
#[no_mangle]
pub extern "C" fn rs_diff_calc_filler_update(
    current_topfill: LinenrT,
    available_fill: LinenrT,
    is_curwin: bool,
) -> FillerUpdateResult {
    let mut result = FillerUpdateResult::default();

    // Non-current windows with positive topfill, or any window with available fill
    let should_update = (!is_curwin && current_topfill > 0) || available_fill > 0;

    if !should_update {
        result.new_topfill = current_topfill;
        return result;
    }

    if current_topfill > available_fill {
        // Reduce topfill to available (but not below 0)
        result.new_topfill = if available_fill > 0 {
            available_fill
        } else {
            0
        };
        result.changed = true;
    } else if available_fill > 0 && available_fill > current_topfill {
        // Increase topfill to available
        result.new_topfill = available_fill;
        result.changed = true;
        result.used_max_fill = true;
    } else {
        result.new_topfill = current_topfill;
    }

    result
}

// =============================================================================
// Invalidation Helpers
// =============================================================================

/// Check if a buffer is part of diff in a tabpage.
///
/// This checks a single tabpage. Call from C in a loop over all tabs.
///
/// # Safety
/// `buf` and `tp` must be valid handles.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_buf_in_tab_diff(buf: BufHandle, tp: TabpageHandle) -> bool {
    if buf.is_null() || tp.is_null() {
        return false;
    }

    for i in 0..DB_COUNT as c_int {
        let diff_buf = nvim_tabpage_get_diffbuf(tp, i);
        if !diff_buf.is_null() && diff_buf == buf {
            return true;
        }
    }

    false
}

// Note: rs_diff_idx_valid is already defined in helpers.rs

/// Result of checking if diff should be invalidated.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ShouldInvalidateResult {
    /// Whether diff should be invalidated
    pub invalidate: bool,
    /// Whether redraw is needed
    pub redraw: bool,
    /// Buffer index in diff (-1 if not found)
    pub idx: c_int,
}

/// Check if buffer modification should invalidate diff.
#[no_mangle]
pub const extern "C" fn rs_diff_should_invalidate(
    buf_idx: c_int,
    is_current_tab: bool,
) -> ShouldInvalidateResult {
    if buf_idx < 0 || buf_idx >= DB_COUNT as c_int {
        // Buffer not in diff
        return ShouldInvalidateResult {
            invalidate: false,
            redraw: false,
            idx: -1,
        };
    }

    ShouldInvalidateResult {
        invalidate: true,
        redraw: is_current_tab,
        idx: buf_idx,
    }
}

// =============================================================================
// Diff Update Line
// =============================================================================

/// State for updating diff at a specific line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffLineUpdateState {
    /// Line number being updated
    pub lnum: LinenrT,
    /// Whether update is in progress
    pub in_progress: bool,
    /// Whether update succeeded
    pub succeeded: bool,
}

/// Initialize line update state.
#[no_mangle]
pub const extern "C" fn rs_diff_line_update_init(lnum: LinenrT) -> DiffLineUpdateState {
    DiffLineUpdateState {
        lnum,
        in_progress: true,
        succeeded: false,
    }
}

/// Mark line update as complete.
#[no_mangle]
pub const extern "C" fn rs_diff_line_update_complete(
    state: &mut DiffLineUpdateState,
    success: bool,
) {
    state.in_progress = false;
    state.succeeded = success;
}

// =============================================================================
// Phase 4 Migration: diff_redraw
// =============================================================================

/// Mark all diff buffers in the current tab page for redraw.
///
/// Rust implementation of the C diff_redraw() function. Iterates over all
/// windows in the current tab, redraws diff windows, recomputes folds,
/// and adjusts topfill/filler lines.
///
/// # Safety
/// Accesses C globals (curtab, curwin, need_diff_redraw).
#[export_name = "diff_redraw"]
pub unsafe extern "C" fn rs_diff_redraw(dofold: bool) {
    let mut wp_other = WinHandle::null();
    let mut used_max_fill_other = false;
    let mut used_max_fill_curwin = false;

    nvim_set_need_diff_redraw(false);

    let curtab = nvim_get_curtab();
    let curwin = nvim_get_curwin();
    let upd_some_valid: c_int = 35; // UPD_SOME_VALID

    // FOR_ALL_WINDOWS_IN_TAB: iterate from firstwin via w_next
    let mut wp = nvim_tabpage_first_win(curtab);
    while !wp.is_null() {
        let buf = nvim_win_get_w_buffer(wp);

        // Skip windows where w_p_diff is not set or buffer is invalid
        if nvim_win_get_p_diff(wp) == 0 || !buf_valid(buf) {
            wp = nvim_win_next(wp);
            continue;
        }

        nvim_redraw_later_win(wp, upd_some_valid);

        if wp != curwin {
            wp_other = wp;
        }

        if dofold && rs_foldmethodIsDiff(wp) != 0 {
            rs_foldUpdateAll(wp);
        }

        // Check if filler lines need updating
        let topline = nvim_win_get_topline(wp);
        let n = rs_diff_check_fill(wp, topline);
        let topfill = nvim_win_get_topfill(wp);

        let should_update = (wp != curwin && topfill > 0) || n > 0;
        if should_update {
            if topfill > n {
                // Reduce topfill to available (but not below 0)
                nvim_win_set_topfill(wp, n.max(0));
            } else if n > 0 && n > topfill {
                // Increase topfill to fill available lines
                nvim_win_set_topfill(wp, n);
                if wp == curwin {
                    used_max_fill_curwin = true;
                } else if !wp_other.is_null() {
                    used_max_fill_other = true;
                }
            }
            nvim_check_topfill(wp, 0);
        }

        wp = nvim_win_next(wp);
    }

    // Handle scroll binding after updating all windows
    if !wp_other.is_null() && nvim_win_get_p_scb(curwin) {
        if used_max_fill_curwin {
            // Current window used max filler lines, may need to reduce them
            rs_diff_set_topline(wp_other, curwin);
        } else if used_max_fill_other {
            // Other window used max filler lines, may need to reduce them
            rs_diff_set_topline(curwin, wp_other);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_state_init() {
        let state = rs_diff_update_state_init(true, false);
        assert!(state.invalid);
        assert!(!state.update_folds);
        assert!(state.need_redraw);

        let state = rs_diff_update_state_init(false, true);
        assert!(!state.invalid);
        assert!(state.update_folds);
        assert!(state.need_redraw);
    }

    #[test]
    fn test_update_state_pending() {
        let mut state = DiffUpdateState::default();
        assert!(!rs_diff_update_pending(&state));

        state.invalid = true;
        assert!(rs_diff_update_pending(&state));

        rs_diff_update_clear(&mut state);
        assert!(!rs_diff_update_pending(&state));
    }

    #[test]
    fn test_redraw_state_default() {
        let state = rs_diff_redraw_state_init();
        assert!(!state.has_other_window);
        assert!(state.other_window.is_null());
        assert!(!state.used_max_fill_curwin);
        assert!(!state.used_max_fill_other);
        assert_eq!(state.window_count, 0);
    }

    #[test]
    fn test_filler_update_no_change() {
        let result = rs_diff_calc_filler_update(0, 0, true);
        assert!(!result.changed);
        assert_eq!(result.new_topfill, 0);
    }

    #[test]
    fn test_filler_update_reduce() {
        let result = rs_diff_calc_filler_update(5, 3, false);
        assert!(result.changed);
        assert_eq!(result.new_topfill, 3);
        assert!(!result.used_max_fill);
    }

    #[test]
    fn test_filler_update_increase() {
        let result = rs_diff_calc_filler_update(2, 5, true);
        assert!(result.changed);
        assert_eq!(result.new_topfill, 5);
        assert!(result.used_max_fill);
    }

    #[test]
    fn test_filler_update_reduce_to_zero() {
        let result = rs_diff_calc_filler_update(5, 0, false);
        assert!(result.changed);
        assert_eq!(result.new_topfill, 0);
    }

    #[test]
    fn test_should_invalidate() {
        let result = rs_diff_should_invalidate(0, true);
        assert!(result.invalidate);
        assert!(result.redraw);
        assert_eq!(result.idx, 0);

        let result = rs_diff_should_invalidate(-1, true);
        assert!(!result.invalidate);
        assert!(!result.redraw);
        assert_eq!(result.idx, -1);

        let result = rs_diff_should_invalidate(2, false);
        assert!(result.invalidate);
        assert!(!result.redraw);
        assert_eq!(result.idx, 2);
    }

    // test_idx_valid is tested in helpers.rs

    #[test]
    fn test_line_update_state() {
        let mut state = rs_diff_line_update_init(100);
        assert_eq!(state.lnum, 100);
        assert!(state.in_progress);
        assert!(!state.succeeded);

        rs_diff_line_update_complete(&mut state, true);
        assert!(!state.in_progress);
        assert!(state.succeeded);
    }
}
