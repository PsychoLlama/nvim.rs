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

                if p_spk == c_int::from(b's') && winrow != prev_winrow && botline - 1 <= line_count
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
            if so == view_height / 2 {
                bot
            } else {
                top
            }
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
// Phase 9: may_trigger_win_scrolled_resized migration
// =============================================================================

/// Opaque handle to a C dict_T*.
type DictHandle = *mut std::ffi::c_void;

/// Opaque handle to a C list_T*.
type ListHandle = *mut std::ffi::c_void;

extern "C" {
    // Window snapshot field getters
    fn nvim_win_get_last_topline(wp: WinHandle) -> c_int;
    fn nvim_win_get_last_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_get_last_leftcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_last_skipcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_last_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_last_height(wp: WinHandle) -> c_int;

    // Event ignored/has_event wrappers
    fn nvim_event_ignored_winscrolled(wp: WinHandle) -> c_int;
    fn nvim_event_ignored_winresized(wp: WinHandle) -> c_int;
    fn nvim_has_event_winscrolled() -> c_int;
    fn nvim_has_event_winresized() -> c_int;

    // Window field getters (some already declared in existing extern block above)
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_handle(wp: WinHandle) -> c_int;
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_get_leftcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_skipcol(wp: WinHandle) -> c_int;

    // Init floating window snapshot
    fn nvim_win_init_float_snapshot(wp: WinHandle);

    // Typval compound operations
    fn nvim_tv_dict_alloc_refcount1() -> DictHandle;
    fn nvim_tv_dict_add_number(
        dict: DictHandle,
        key: *const std::ffi::c_char,
        key_len: usize,
        nr: c_int,
    ) -> c_int;
    fn nvim_tv_dict_add_dict_wrapper(
        dict: DictHandle,
        key: *const std::ffi::c_char,
        key_len: usize,
        child: DictHandle,
    ) -> c_int;
    fn nvim_tv_dict_unref_wrapper(dict: DictHandle);
    fn nvim_tv_list_alloc_wrapper(count: c_int) -> ListHandle;
    fn nvim_tv_list_append_number(list: ListHandle, nr: c_int);

    // Compound autocmd/v:event fire operations.
    // These wrappers take full ownership of list/dict and handle the entire
    // get_v_event / set_keys_readonly / apply_autocmds / restore_v_event lifecycle.
    // The win_handle is used to find the buffer (via bufref_valid check).
    fn nvim_fire_winresized(
        list: ListHandle,
        winid_str: *const std::ffi::c_char,
        first_size_win: WinHandle,
        first_size_win_buf_fnum: c_int,
    );
    fn nvim_fire_winscrolled(
        dict: DictHandle,
        winid_str: *const std::ffi::c_char,
        first_scroll_win: WinHandle,
        first_scroll_win_buf_fnum: c_int,
    );

    // Get buf fnum for bufref validity check
    fn nvim_win_get_buf_fnum(wp: WinHandle) -> c_int;
}

/// Scan result from iterating windows for scroll/resize changes.
struct ScrollResizeScan {
    size_count: c_int,
    first_scroll_win: WinHandle,
    first_size_win: WinHandle,
}

/// Scan all windows in current tab for scroll/size changes.
///
/// Returns counts and first-window pointers needed to decide what events to fire.
fn check_window_scroll_resize_scan() -> ScrollResizeScan {
    let mut result = ScrollResizeScan {
        size_count: 0,
        first_scroll_win: WinHandle::null(),
        first_size_win: WinHandle::null(),
    };

    // SAFETY: all accessor calls are safe C functions
    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            // Skip floating windows without a snapshot (init them instead)
            if nvim_win_get_floating(wp) != 0 && nvim_win_get_last_topline(wp) == 0 {
                nvim_win_init_float_snapshot(wp);
                wp = nvim_win_get_next(wp);
                continue;
            }

            let ignore_scroll = nvim_event_ignored_winscrolled(wp) != 0;
            let size_changed = nvim_event_ignored_winresized(wp) == 0
                && (nvim_win_get_last_width(wp) != nvim_win_get_w_width(wp)
                    || nvim_win_get_last_height(wp) != nvim_win_get_w_height(wp));

            if size_changed {
                result.size_count += 1;
                if result.first_size_win.is_null() {
                    result.first_size_win = wp;
                }
                // For WinScrolled: first window with a size change is also used
                // as first_scroll_win even when it didn't scroll (per C original).
                if result.first_scroll_win.is_null() && !ignore_scroll {
                    result.first_scroll_win = wp;
                }
            }

            let scroll_changed = !ignore_scroll
                && (nvim_win_get_last_topline(wp) != nvim_win_get_topline(wp)
                    || nvim_win_get_last_topfill(wp) != nvim_win_get_topfill(wp)
                    || nvim_win_get_last_leftcol(wp) != nvim_win_get_leftcol(wp)
                    || nvim_win_get_last_skipcol(wp) != nvim_win_get_skipcol(wp));

            if scroll_changed && result.first_scroll_win.is_null() {
                result.first_scroll_win = wp;
            }

            wp = nvim_win_get_next(wp);
        }
    }

    result
}

/// Build the list of window handles with size changes for WinResized v:event.
fn check_window_scroll_resize_build_list(list: ListHandle) {
    // SAFETY: all accessor calls are safe C functions
    unsafe {
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_get_floating(wp) != 0 && nvim_win_get_last_topline(wp) == 0 {
                wp = nvim_win_get_next(wp);
                continue;
            }

            let size_changed = nvim_event_ignored_winresized(wp) == 0
                && (nvim_win_get_last_width(wp) != nvim_win_get_w_width(wp)
                    || nvim_win_get_last_height(wp) != nvim_win_get_w_height(wp));

            if size_changed {
                nvim_tv_list_append_number(list, nvim_win_get_handle(wp));
            }

            wp = nvim_win_get_next(wp);
        }
    }
}

/// Allocate a win-info dict with 6 number entries.
///
/// Returns null DictHandle on any allocation failure (frees partial dict).
fn make_win_info_dict_rs(
    width: c_int,
    height: c_int,
    topline: c_int,
    topfill: c_int,
    leftcol: c_int,
    skipcol: c_int,
) -> DictHandle {
    // SAFETY: compound C wrappers manage dict memory
    unsafe {
        let d = nvim_tv_dict_alloc_refcount1();
        if d.is_null() {
            return std::ptr::null_mut();
        }

        // OK == 1 in Neovim's C API (maps to FAIL == 0)
        let mut success = true;

        macro_rules! add_num {
            ($key:expr, $val:expr) => {{
                let k: &[u8] = $key;
                if nvim_tv_dict_add_number(d, k.as_ptr().cast(), k.len() - 1, $val) == 0 {
                    success = false;
                }
            }};
        }

        add_num!(b"width\0", width);
        if success {
            add_num!(b"height\0", height);
        }
        if success {
            add_num!(b"topline\0", topline);
        }
        if success {
            add_num!(b"topfill\0", topfill);
        }
        if success {
            add_num!(b"leftcol\0", leftcol);
        }
        if success {
            add_num!(b"skipcol\0", skipcol);
        }

        if success {
            d
        } else {
            nvim_tv_dict_unref_wrapper(d);
            std::ptr::null_mut()
        }
    }
}

/// Build the scroll dict for WinScrolled v:event.
///
/// For each window with scroll or size changes, adds a per-window sub-dict
/// keyed by window ID string. Also adds an "all" totals dict.
///
/// Returns the dict (caller owns it with refcount=1), or null on failure.
fn check_window_scroll_resize_build_dict() -> DictHandle {
    let mut tot_width: c_int = 0;
    let mut tot_height: c_int = 0;
    let mut tot_topline: c_int = 0;
    let mut tot_topfill: c_int = 0;
    let mut tot_leftcol: c_int = 0;
    let mut tot_skipcol: c_int = 0;

    // SAFETY: all compound C wrappers manage their own memory
    unsafe {
        let scroll_dict = nvim_tv_dict_alloc_refcount1();
        if scroll_dict.is_null() {
            return std::ptr::null_mut();
        }

        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_get_floating(wp) != 0 && nvim_win_get_last_topline(wp) == 0 {
                wp = nvim_win_get_next(wp);
                continue;
            }

            let ignore_scroll = nvim_event_ignored_winscrolled(wp) != 0;
            let size_changed = nvim_event_ignored_winresized(wp) == 0
                && (nvim_win_get_last_width(wp) != nvim_win_get_w_width(wp)
                    || nvim_win_get_last_height(wp) != nvim_win_get_w_height(wp));

            let scroll_changed = !ignore_scroll
                && (nvim_win_get_last_topline(wp) != nvim_win_get_topline(wp)
                    || nvim_win_get_last_topfill(wp) != nvim_win_get_topfill(wp)
                    || nvim_win_get_last_leftcol(wp) != nvim_win_get_leftcol(wp)
                    || nvim_win_get_last_skipcol(wp) != nvim_win_get_skipcol(wp));

            if size_changed || scroll_changed {
                let width = nvim_win_get_w_width(wp) - nvim_win_get_last_width(wp);
                let height = nvim_win_get_w_height(wp) - nvim_win_get_last_height(wp);
                let topline = nvim_win_get_topline(wp) - nvim_win_get_last_topline(wp);
                let topfill = nvim_win_get_topfill(wp) - nvim_win_get_last_topfill(wp);
                let leftcol = nvim_win_get_leftcol(wp) - nvim_win_get_last_leftcol(wp);
                let skipcol = nvim_win_get_skipcol(wp) - nvim_win_get_last_skipcol(wp);

                let d = make_win_info_dict_rs(width, height, topline, topfill, leftcol, skipcol);
                if d.is_null() {
                    nvim_tv_dict_unref_wrapper(scroll_dict);
                    return std::ptr::null_mut();
                }

                // Format window handle as NUL-terminated decimal string
                let handle = nvim_win_get_handle(wp);
                let mut winid_buf = [0u8; 24];
                let key_len = format_int_to_buf(handle, &mut winid_buf);

                if nvim_tv_dict_add_dict_wrapper(scroll_dict, winid_buf.as_ptr().cast(), key_len, d)
                    == 0
                {
                    // Wrapper does not consume d on failure; free it.
                    nvim_tv_dict_unref_wrapper(d);
                    nvim_tv_dict_unref_wrapper(scroll_dict);
                    return std::ptr::null_mut();
                }
                // d ownership transferred (wrapper decrements refcount)

                tot_width += width.abs();
                tot_height += height.abs();
                tot_topline += topline.abs();
                tot_topfill += topfill.abs();
                tot_leftcol += leftcol.abs();
                tot_skipcol += skipcol.abs();
            }

            wp = nvim_win_get_next(wp);
        }

        // Add "all" totals sub-dict (non-fatal if it fails, matching C original)
        let alldict = make_win_info_dict_rs(
            tot_width,
            tot_height,
            tot_topline,
            tot_topfill,
            tot_leftcol,
            tot_skipcol,
        );
        if !alldict.is_null()
            && nvim_tv_dict_add_dict_wrapper(scroll_dict, c"all".as_ptr(), 3, alldict) == 0
        {
            nvim_tv_dict_unref_wrapper(alldict);
            // non-fatal per C original
        }

        scroll_dict
    }
}

/// Format a c_int as decimal ASCII into buf, return length (not including NUL).
///
/// Writes a NUL terminator after the digits.
fn format_int_to_buf(val: c_int, buf: &mut [u8; 24]) -> usize {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(buf.as_mut_slice());
    let _ = write!(cursor, "{val}");
    let pos = cursor.position() as usize;
    if pos < buf.len() {
        buf[pos] = 0; // NUL-terminate
    }
    pos
}

/// Recursive guard for may_trigger_win_scrolled_resized.
///
/// AtomicBool with Relaxed ordering is sufficient since Neovim is single-threaded.
static SCROLLED_RESIZED_RECURSIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Trigger WinScrolled and/or WinResized if any window in the current tab
/// page scrolled or changed size.
///
/// This is the Rust equivalent of `may_trigger_win_scrolled_resized()`.
fn may_trigger_win_scrolled_resized_impl() {
    // SAFETY: compound C wrappers manage typval/autocmd lifecycle
    unsafe {
        let do_resize = nvim_has_event_winresized() != 0;
        let do_scroll = nvim_has_event_winscrolled() != 0;

        if SCROLLED_RESIZED_RECURSIVE.load(Ordering::Relaxed)
            || (!do_scroll && !do_resize)
            || !DID_INITIAL_SCROLL_SIZE_SNAPSHOT.load(Ordering::Relaxed)
        {
            return;
        }

        // Scan for changes
        let scan = check_window_scroll_resize_scan();
        let trigger_resize = do_resize && scan.size_count > 0;
        let trigger_scroll = do_scroll && !scan.first_scroll_win.is_null();

        if !trigger_resize && !trigger_scroll {
            return;
        }

        // Build WinResized list before snapshot
        let windows_list: ListHandle = if trigger_resize {
            let list = nvim_tv_list_alloc_wrapper(scan.size_count);
            if !list.is_null() {
                check_window_scroll_resize_build_list(list);
            }
            list
        } else {
            std::ptr::null_mut()
        };

        // Build WinScrolled dict before snapshot
        let scroll_dict: DictHandle = if trigger_scroll {
            check_window_scroll_resize_build_dict()
        } else {
            std::ptr::null_mut()
        };

        // Take snapshot BEFORE firing autocmds (matching C original)
        rs_snapshot_windows_scroll_size();

        SCROLLED_RESIZED_RECURSIVE.store(true, Ordering::Relaxed);

        // Save winid strings and buf fnums before autocmds (windows can be freed)
        let mut resize_winid = [0u8; 24];
        let resize_buf_fnum: c_int = if trigger_resize && !scan.first_size_win.is_null() {
            format_int_to_buf(nvim_win_get_handle(scan.first_size_win), &mut resize_winid);
            nvim_win_get_buf_fnum(scan.first_size_win)
        } else {
            0
        };

        let mut scroll_winid = [0u8; 24];
        let scroll_buf_fnum: c_int = if trigger_scroll && !scan.first_scroll_win.is_null() {
            format_int_to_buf(
                nvim_win_get_handle(scan.first_scroll_win),
                &mut scroll_winid,
            );
            nvim_win_get_buf_fnum(scan.first_scroll_win)
        } else {
            0
        };

        // Fire WinResized first
        if trigger_resize && !windows_list.is_null() {
            nvim_fire_winresized(
                windows_list,
                resize_winid.as_ptr().cast(),
                scan.first_size_win,
                resize_buf_fnum,
            );
            // windows_list ownership transferred to wrapper
            // If windows_list is null (alloc failed), skip firing but don't crash
        }

        // Fire WinScrolled
        if trigger_scroll && !scroll_dict.is_null() {
            nvim_fire_winscrolled(
                scroll_dict,
                scroll_winid.as_ptr().cast(),
                scan.first_scroll_win,
                scroll_buf_fnum,
            );
            // scroll_dict ownership transferred to wrapper
        }

        SCROLLED_RESIZED_RECURSIVE.store(false, Ordering::Relaxed);
    }
}

/// FFI export for `may_trigger_win_scrolled_resized`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_may_trigger_win_scrolled_resized() {
    may_trigger_win_scrolled_resized_impl();
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
