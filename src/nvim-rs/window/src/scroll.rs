//! Window scroll position management for 'splitkeep' (Phase 3).
//!
//! This module provides Rust implementations of:
//! - `win_fix_scroll` -- handles 'splitkeep' scroll position preservation
//! - `win_fix_cursor` -- adjusts cursor for 'splitkeep' validity
//! - `may_make_initial_scroll_size_snapshot` -- one-time snapshot initialization

#![allow(clippy::cast_possible_truncation)]

use std::ffi::c_int;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::WinHandle;

// =============================================================================
// Constants
// =============================================================================

/// VALID_WCOL: w_wcol is valid (must match C value 0x02).
const VALID_WCOL: c_int = 0x02;

/// VALID_CROW: w_crow is valid (must match C value 0x10).
const VALID_CROW: c_int = 0x10;

/// FRACTION_MULT: fraction multiplier (must match C value 16384).
const FRACTION_MULT: c_int = 16384;

/// MODE_NORMAL from state_defs.h (0x01).
const MODE_NORMAL: c_int = 0x01;

/// MODE_CMDLINE from state_defs.h (0x08).
const MODE_CMDLINE: c_int = 0x08;

/// MODE_TERMINAL from state_defs.h (0x80).
const MODE_TERMINAL: c_int = 0x80;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get the current window.
    fn nvim_get_curwin() -> WinHandle;

    /// Get the first window in the current tab.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get the `w_next` field from a window.
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    /// Get wp->w_floating.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get *p_spk character: 'c' = cursor, 's' = screen, 't' = topline.
    fn nvim_win_get_p_spk_char() -> c_int;

    /// Set skip_update_topline global.
    fn nvim_set_skip_update_topline(val: c_int);

    /// Get skip_win_fix_cursor global.
    fn nvim_get_skip_win_fix_cursor() -> c_int;

    /// Get wp->w_do_win_fix_cursor.
    fn nvim_win_get_do_win_fix_cursor(wp: WinHandle) -> c_int;

    /// Set wp->w_do_win_fix_cursor.
    fn nvim_win_set_do_win_fix_cursor(wp: WinHandle, val: c_int);

    /// Get wp->w_height.
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;

    /// Get wp->w_prev_height.
    fn nvim_win_get_prev_height(wp: WinHandle) -> c_int;

    /// Set wp->w_prev_height.
    fn nvim_win_set_prev_height(wp: WinHandle, val: c_int);

    /// Get wp->w_winrow.
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;

    /// Get wp->w_prev_winrow.
    fn nvim_win_get_prev_winrow(wp: WinHandle) -> c_int;

    /// Set wp->w_prev_winrow.
    fn nvim_win_set_prev_winrow(wp: WinHandle, val: c_int);

    /// Get wp->w_botline.
    fn nvim_win_get_botline(wp: WinHandle) -> c_int;

    /// Get wp->w_buffer line count (b_ml.ml_line_count).
    fn nvim_win_buf_line_count(wp: WinHandle) -> c_int;

    /// Get wp->w_cursor.lnum.
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;

    /// Set wp->w_cursor.lnum.
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: c_int);

    /// Get wp->w_topline.
    fn nvim_win_get_topline(wp: WinHandle) -> c_int;

    /// Get wp->w_view_height.
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;

    /// Set wp->w_fraction.
    fn nvim_win_set_fraction(wp: WinHandle, val: c_int);

    /// Clear w_valid bits: wp->w_valid &= ~bits.
    fn nvim_win_clear_valid_bits(wp: WinHandle, bits: c_int);

    /// cursor_down_inner(wp, n, skip_conceal).
    fn nvim_cursor_down_inner(wp: WinHandle, n: c_int, skip_conceal: c_int);

    /// cursor_up_inner(wp, n, skip_conceal).
    fn nvim_cursor_up_inner(wp: WinHandle, n: c_int, skip_conceal: c_int);

    /// invalidate_botline(wp).
    fn nvim_invalidate_botline(wp: WinHandle);

    /// validate_botline(wp).
    fn nvim_validate_botline(wp: WinHandle);

    /// scroll_to_fraction(wp, prev_height).
    fn rs_scroll_to_fraction(wp: WinHandle, prev_height: c_int);

    /// rs_get_scrolloff_value(wp).
    fn rs_get_scrolloff_value(wp: WinHandle) -> c_int;

    /// setmark(name) -- saves cursor to jumplist.
    fn nvim_setmark(name: c_int) -> bool;

    /// get_real_state() -- returns current editor mode flags.
    fn nvim_get_real_state() -> c_int;

    /// rs_snapshot_windows_scroll_size() -- take scroll size snapshot.
    fn rs_snapshot_windows_scroll_size();
}

// =============================================================================
// Static: did_initial_scroll_size_snapshot
// =============================================================================

/// Tracks whether the initial scroll size snapshot has been taken.
///
/// This replaces the C static `did_initial_scroll_size_snapshot` in window_shim.c.
static DID_INITIAL_SCROLL_SIZE_SNAPSHOT: AtomicBool = AtomicBool::new(false);

// =============================================================================
// may_make_initial_scroll_size_snapshot
// =============================================================================

/// Take the initial scroll size snapshot if it hasn't been done yet.
///
/// This is the Rust equivalent of `may_make_initial_scroll_size_snapshot()`.
fn may_make_initial_scroll_size_snapshot_impl() {
    // Use compare_exchange to atomically check-and-set.
    if DID_INITIAL_SCROLL_SIZE_SNAPSHOT
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        // SAFETY: rs_snapshot_windows_scroll_size is a Rust FFI export
        unsafe { rs_snapshot_windows_scroll_size() };
    }
}

/// FFI export for `may_make_initial_scroll_size_snapshot`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_may_make_initial_scroll_size_snapshot() {
    may_make_initial_scroll_size_snapshot_impl();
}

/// FFI export: get the did_initial_scroll_size_snapshot flag.
///
/// Used by C's `may_trigger_win_scrolled_resized` in window_shim.c.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_did_initial_scroll_size_snapshot() -> c_int {
    c_int::from(DID_INITIAL_SCROLL_SIZE_SNAPSHOT.load(Ordering::SeqCst))
}

// =============================================================================
// win_fix_scroll
// =============================================================================

/// Handle scroll position depending on 'splitkeep'.
///
/// Replaces the `scroll_to_fraction()` call from `win_new_height()` if
/// 'splitkeep' is "screen" or "topline". Iterates over all windows in the
/// current tabpage and calculates the new scroll position.
///
/// This is the Rust equivalent of `win_fix_scroll()`.
fn win_fix_scroll_impl(resize: bool) {
    // SAFETY: All C accessor calls below are safe field accessors
    unsafe {
        let p_spk = nvim_win_get_p_spk_char();

        // 'splitkeep' is "cursor" -- nothing to do
        if p_spk == c_int::from(b'c') {
            return;
        }

        let curwin = nvim_get_curwin();

        nvim_set_skip_update_topline(1);

        // Iterate all windows in the current tabpage (FOR_ALL_WINDOWS_IN_TAB).
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            let height = nvim_win_get_w_height(wp);
            let prev_height = nvim_win_get_prev_height(wp);
            let floating = nvim_win_get_floating(wp);

            // Skip when window height has not changed or when floating.
            if floating == 0 && height != prev_height {
                // Cursor position in this window may now be invalid.
                nvim_win_set_do_win_fix_cursor(wp, 1);

                // If window has moved, update botline to keep the same screenlines.
                let winrow = nvim_win_get_winrow(wp);
                let prev_winrow = nvim_win_get_prev_winrow(wp);
                let botline = nvim_win_get_botline(wp);
                let line_count = nvim_win_buf_line_count(wp);

                if p_spk == c_int::from(b's')
                    && winrow != prev_winrow
                    && botline - 1 <= line_count
                {
                    let diff = (winrow - prev_winrow) + (height - prev_height);
                    let saved_lnum = nvim_win_get_cursor_lnum(wp);

                    // Set cursor to botline - 1 for scroll calculation
                    nvim_win_set_cursor_lnum(wp, botline - 1);

                    // Add difference in height and row to botline.
                    if diff > 0 {
                        nvim_cursor_down_inner(wp, diff, 0);
                    } else {
                        nvim_cursor_up_inner(wp, -diff, 0);
                    }

                    // Scroll to put the new cursor position at the bottom of screen.
                    nvim_win_set_fraction(wp, FRACTION_MULT);
                    rs_scroll_to_fraction(wp, prev_height);

                    // Restore original cursor position
                    nvim_win_set_cursor_lnum(wp, saved_lnum);
                    nvim_win_clear_valid_bits(wp, VALID_WCOL);
                } else if wp == curwin {
                    nvim_win_clear_valid_bits(wp, VALID_CROW);
                }

                nvim_invalidate_botline(wp);
                nvim_validate_botline(wp);
            }

            // Update splitkeep snapshot values.
            nvim_win_set_prev_height(wp, nvim_win_get_w_height(wp));
            nvim_win_set_prev_winrow(wp, nvim_win_get_winrow(wp));

            wp = nvim_win_get_next(wp);
        }

        nvim_set_skip_update_topline(0);

        // Ensure cursor is valid when not in normal mode or when resized.
        let state = nvim_get_real_state();
        if (state & (MODE_NORMAL | MODE_CMDLINE | MODE_TERMINAL)) == 0 {
            win_fix_cursor_impl(false);
        } else if resize {
            win_fix_cursor_impl(true);
        }
    }
}

/// FFI export for `win_fix_scroll`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_fix_scroll(resize: c_int) {
    win_fix_scroll_impl(resize != 0);
}

// =============================================================================
// win_fix_cursor
// =============================================================================

/// Make sure the cursor position is valid for 'splitkeep'.
///
/// If the cursor is out of valid range:
/// - In normal mode (`normal` = true): save to jumplist and move cursor.
/// - Otherwise: scroll to make it valid.
///
/// This is the Rust equivalent of `win_fix_cursor()`.
fn win_fix_cursor_impl(normal: bool) {
    // SAFETY: All C accessor calls are safe field accessors
    unsafe {
        if nvim_get_skip_win_fix_cursor() != 0 {
            return;
        }

        let wp = nvim_get_curwin();
        if wp.is_null() {
            return;
        }

        if nvim_win_get_do_win_fix_cursor(wp) == 0 {
            return;
        }

        let view_height = nvim_win_get_view_height(wp);
        let line_count = nvim_win_buf_line_count(wp);

        if line_count < view_height {
            return;
        }

        nvim_win_set_do_win_fix_cursor(wp, 0);

        // Determine valid cursor range using scrolloff:
        // so = MIN(w_view_height / 2, rs_get_scrolloff_value(wp))
        let so = rs_get_scrolloff_value(wp).min(view_height / 2);
        let lnum = nvim_win_get_cursor_lnum(wp);

        // Find top boundary: move from topline down by 'so'
        nvim_win_set_cursor_lnum(wp, nvim_win_get_topline(wp));
        nvim_cursor_down_inner(wp, so, 0);
        let top = nvim_win_get_cursor_lnum(wp);

        // Find bottom boundary: move from botline-1 up by 'so'
        nvim_win_set_cursor_lnum(wp, nvim_win_get_botline(wp) - 1);
        nvim_cursor_up_inner(wp, so, 0);
        let bot = nvim_win_get_cursor_lnum(wp);

        // Restore original cursor position
        nvim_win_set_cursor_lnum(wp, lnum);

        // Check if cursor is outside the valid range.
        let botline = nvim_win_get_botline(wp);
        let topline = nvim_win_get_topline(wp);

        let nlnum = if lnum > bot && (botline - line_count) != 1 {
            bot
        } else if lnum < top && topline != 1 {
            // If so hit the half-screen limit, use bot; otherwise use top
            if so == view_height / 2 { bot } else { top }
        } else {
            return; // cursor is in the valid range
        };

        if normal {
            // Save to jumplist and move cursor directly (avoid scrolling).
            nvim_setmark(c_int::from(b'\''));
            nvim_win_set_cursor_lnum(wp, nlnum);
        } else {
            // Scroll to make cursor valid.
            let fraction = if nlnum == bot { FRACTION_MULT } else { 0 };
            nvim_win_set_fraction(wp, fraction);
            let prev_height = nvim_win_get_prev_height(wp);
            rs_scroll_to_fraction(wp, prev_height);
            nvim_validate_botline(wp);
        }
    }
}

/// FFI export for `win_fix_cursor`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_win_fix_cursor(normal: c_int) {
    win_fix_cursor_impl(normal != 0);
}

// =============================================================================
// Static assertions via tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraction_mult() {
        assert_eq!(FRACTION_MULT, 16384);
    }

    #[test]
    fn test_valid_constants() {
        assert_eq!(VALID_WCOL, 0x02);
        assert_eq!(VALID_CROW, 0x10);
    }

    #[test]
    fn test_mode_constants() {
        assert_eq!(MODE_NORMAL, 0x01);
        assert_eq!(MODE_CMDLINE, 0x08);
        assert_eq!(MODE_TERMINAL, 0x80);
    }
}
