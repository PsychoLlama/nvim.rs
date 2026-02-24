//! Diff window option management
//!
//! This module provides Rust implementations for setting and restoring
//! window options when entering/exiting diff mode:
//!   - `rs_diff_win_options`: set diff options on a window
//!   - `rs_ex_diffoff`: `:diffoff` command implementation

#![allow(clippy::must_use_candidate)]

use std::ffi::c_char;
use std::os::raw::c_int;

use crate::buffer::{BufHandle, TabpageHandle, WinHandle};

/// Line number type matching linenr_T (i32).
type LinenrT = i32;

/// Result constants.
#[allow(dead_code)]
const OK: c_int = 1;
#[allow(dead_code)]
const FAIL: c_int = 0;

/// DIFF_FOLLOWWRAP flag value (must match C).
const DIFF_FOLLOWWRAP: c_int = 0x800;

// =============================================================================
// C FFI declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Window iteration
    fn nvim_tabpage_first_win(tp: TabpageHandle) -> WinHandle;
    fn nvim_win_next(wp: WinHandle) -> WinHandle;

    // Window boolean option get/set
    fn nvim_win_get_w_p_diff_saved(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_diff_saved(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_scb(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_scb(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_scb_save(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_scb_save(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_crb(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_crb(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_crb_save(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_crb_save(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_wrap(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_wrap(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_wrap_save(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_wrap_save(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_fen(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_fen(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_fen_save(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_fen_save(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_diff(wp: WinHandle) -> bool;

    // Window integer option get/set
    fn nvim_win_get_w_p_fdl(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_w_p_fdl(wp: WinHandle, val: LinenrT);
    fn nvim_win_get_w_p_fdl_save(wp: WinHandle) -> LinenrT;
    fn nvim_win_set_w_p_fdl_save(wp: WinHandle, val: LinenrT);

    // Window string options (atomic free+set)
    fn nvim_win_free_and_set_fdm(wp: WinHandle, val: *const c_char);
    fn nvim_win_free_and_set_fdc(wp: WinHandle, val: *const c_char);
    fn nvim_win_free_and_set_fdm_save(wp: WinHandle, val: *const c_char);
    fn nvim_win_free_and_set_fdc_save(wp: WinHandle, val: *const c_char);

    // String emptiness checks
    fn nvim_win_get_fdm_save_empty(wp: WinHandle) -> bool;
    fn nvim_win_get_fdc_save_empty(wp: WinHandle) -> bool;

    // String option getters (returns pointer to current string)
    fn nvim_win_get_w_p_fdm(wp: WinHandle) -> *const c_char;
    fn nvim_win_get_w_p_fdc(wp: WinHandle) -> *const c_char; // defined in window_shim.c

    // Window misc setters (using existing window_shim.c names)
    fn nvim_win_set_skipcol(wp: WinHandle, val: LinenrT); // defined in window_shim.c
    fn nvim_win_set_topfill(wp: WinHandle, val: c_int); // defined in window_shim.c
    fn nvim_win_set_leftcol(wp: WinHandle, val: c_int); // defined in window_shim.c

    // Diff-mode compound operations
    fn nvim_diff_get_foldcolumn() -> c_int;
    fn nvim_diff_set_fdm_to_diff(wp: WinHandle);
    fn nvim_diff_changed_window_setting(wp: WinHandle);
    fn nvim_diff_sbo_has_hor() -> bool;
    fn nvim_diff_do_cmdline_cmd(cmd: *const c_char);

    // Global diff state
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_diff_get_diff_flags() -> c_int;
    fn nvim_diff_get_need_update() -> bool;
    fn nvim_diff_set_need_update(val: bool);
    fn nvim_tabpage_set_diff_invalid(tp: TabpageHandle, val: c_int);
    fn nvim_tabpage_set_diff_update(tp: TabpageHandle, val: c_int);

    // Fold operations
    fn rs_newFoldLevel();
    fn rs_foldUpdateAll(wp: WinHandle);
    fn rs_foldmethodIsManual(wp: WinHandle) -> c_int;

    // Diff buffer / option operations
    fn rs_diff_buf_add(buf: BufHandle);
    fn rs_diff_buf_adjust(wp: WinHandle);
    fn rs_diff_buf_clear();
    fn rs_diff_clear(tp: TabpageHandle);
    fn rs_set_diff_option(wp: WinHandle, value: bool);
    fn nvim_redraw_later_win(wp: WinHandle, typ: c_int);
    fn nvim_upd_not_valid() -> c_int;
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_get_p_diff(wp: WinHandle) -> c_int;

    // Exarg handle (opaque pointer, only used to pass through)
    fn nvim_eap_forceit(eap: *const std::ffi::c_void) -> bool;
}

/// Literal C string helper -- returns pointer to static "manual" string.
unsafe fn c_str_manual() -> *const c_char {
    b"manual\0".as_ptr().cast()
}

/// Literal C string helper -- returns pointer to static "0" string.
unsafe fn c_str_zero() -> *const c_char {
    b"0\0".as_ptr().cast()
}

/// Set fdc to a single-digit string matching diff_foldcolumn.
/// Uses the atomic free+set accessor with a literal string for each possible value 0-9.
unsafe fn set_fdc_to_foldcolumn(wp: WinHandle, n: c_int) {
    // We need to pass a C string with the digit. Use match to select a static literal.
    let s: *const c_char = match n {
        0 => b"0\0".as_ptr().cast(),
        1 => b"1\0".as_ptr().cast(),
        2 => b"2\0".as_ptr().cast(),
        3 => b"3\0".as_ptr().cast(),
        4 => b"4\0".as_ptr().cast(),
        5 => b"5\0".as_ptr().cast(),
        6 => b"6\0".as_ptr().cast(),
        7 => b"7\0".as_ptr().cast(),
        8 => b"8\0".as_ptr().cast(),
        9 => b"9\0".as_ptr().cast(),
        _ => b"2\0".as_ptr().cast(),
    };
    nvim_win_free_and_set_fdc(wp, s);
}

/// Set options in window `wp` for diff mode.
///
/// Faithfully reproduces the C `diff_win_options` logic.
/// Called from `rs_ex_diffthis`, `rs_ex_diffsplit`, and the C thin wrapper.
///
/// # Safety
/// Calls C functions that access global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_win_options(wp: WinHandle, addbuf: bool) {
    // Close manually opened folds: temporarily switch curwin to wp.
    // The C code does: curwin = wp; rs_newFoldLevel(); curwin = old_curwin;
    // We call a compound accessor that does this atomically.
    nvim_diff_changed_window_foldlevel_reset(wp);

    let not_yet_in_diff = !nvim_win_get_w_p_diff(wp);

    // scrollbind
    if not_yet_in_diff {
        nvim_win_set_w_p_scb_save(wp, nvim_win_get_w_p_scb(wp));
    }
    nvim_win_set_w_p_scb(wp, true);

    // cursorbind
    if not_yet_in_diff {
        nvim_win_set_w_p_crb_save(wp, nvim_win_get_w_p_crb(wp));
    }
    nvim_win_set_w_p_crb(wp, true);

    // wrap (only when not DIFF_FOLLOWWRAP)
    let diff_flags = nvim_diff_get_diff_flags();
    if diff_flags & DIFF_FOLLOWWRAP == 0 {
        if not_yet_in_diff {
            nvim_win_set_w_p_wrap_save(wp, nvim_win_get_w_p_wrap(wp));
        }
        nvim_win_set_w_p_wrap(wp, false);
        nvim_win_set_skipcol(wp, 0);
    }

    // foldmethod save + set to "diff"
    if not_yet_in_diff {
        // free existing fdm_save if already saved, then save current fdm
        nvim_win_free_and_set_fdm_save(wp, nvim_win_get_w_p_fdm(wp));
    }
    nvim_diff_set_fdm_to_diff(wp);

    // fen_save, fdl_save, fdc_save
    if not_yet_in_diff {
        nvim_win_set_w_p_fen_save(wp, nvim_win_get_w_p_fen(wp));
        nvim_win_set_w_p_fdl_save(wp, nvim_win_get_w_p_fdl(wp));
        // free existing fdc_save if already saved, then save current fdc
        nvim_win_free_and_set_fdc_save(wp, nvim_win_get_w_p_fdc(wp));
    }

    // Set fdc to diff_foldcolumn value (as C string digit)
    let fc = nvim_diff_get_foldcolumn();
    set_fdc_to_foldcolumn(wp, fc);

    // fen = true, fdl = 0
    nvim_win_set_w_p_fen(wp, true);
    nvim_win_set_w_p_fdl(wp, 0);
    rs_foldUpdateAll(wp);

    // make sure topline is not halfway through a fold
    nvim_diff_changed_window_setting(wp);

    // add 'hor' to 'sbo' if not present
    if !nvim_diff_sbo_has_hor() {
        nvim_diff_do_cmdline_cmd(b"set sbo+=hor\0".as_ptr().cast());
    }

    // mark options as saved
    nvim_win_set_w_p_diff_saved(wp, true);

    // set w_p_diff = true via the option setter
    rs_set_diff_option(wp, true);

    if addbuf {
        rs_diff_buf_add(nvim_win_get_w_buffer(wp));
    }

    let upd = nvim_upd_not_valid();
    nvim_redraw_later_win(wp, upd);
}

/// Set options NOT to show diffs.  For the current window or all windows.
/// Only in the current tab page.
///
/// Faithfully reproduces the C `ex_diffoff` logic.
///
/// # Safety
/// Calls C functions that access global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_diffoff(eap: *const std::ffi::c_void) {
    let forceit = nvim_eap_forceit(eap);
    let curtab = nvim_get_curtab();
    let mut diffwin = false;

    // Iterate all windows in current tab
    let mut wp = nvim_tabpage_first_win(curtab);
    while !wp.is_null() {
        let is_curwin = nvim_diff_is_curwin(wp);
        if if forceit {
            nvim_win_get_p_diff(wp) != 0
        } else {
            is_curwin
        } {
            // Turn off diff option
            rs_set_diff_option(wp, false);

            if nvim_win_get_w_p_diff_saved(wp) {
                // Restore scrollbind
                if nvim_win_get_w_p_scb(wp) {
                    nvim_win_set_w_p_scb(wp, nvim_win_get_w_p_scb_save(wp));
                }
                // Restore cursorbind
                if nvim_win_get_w_p_crb(wp) {
                    nvim_win_set_w_p_crb(wp, nvim_win_get_w_p_crb_save(wp));
                }
                // Restore wrap
                let diff_flags = nvim_diff_get_diff_flags();
                if diff_flags & DIFF_FOLLOWWRAP == 0 {
                    if !nvim_win_get_w_p_wrap(wp) && nvim_win_get_w_p_wrap_save(wp) {
                        nvim_win_set_w_p_wrap(wp, true);
                        nvim_win_set_leftcol(wp, 0);
                    }
                }
                // Restore fdm: use fdm_save if non-empty, else "manual"
                if nvim_win_get_fdm_save_empty(wp) {
                    nvim_win_free_and_set_fdm(wp, c_str_manual());
                } else {
                    let fdm_save = nvim_win_get_w_p_fdm_save(wp);
                    nvim_win_free_and_set_fdm(wp, fdm_save);
                }
                // Restore fdc: use fdc_save if non-empty, else "0"
                if nvim_win_get_fdc_save_empty(wp) {
                    nvim_win_free_and_set_fdc(wp, c_str_zero());
                } else {
                    let fdc_save = nvim_win_get_w_p_fdc_save(wp);
                    nvim_win_free_and_set_fdc(wp, fdc_save);
                }
                // Restore fdl
                if nvim_win_get_w_p_fdl(wp) == 0 {
                    nvim_win_set_w_p_fdl(wp, nvim_win_get_w_p_fdl_save(wp));
                }
                // Restore fen (only when foldmethod is not "manual")
                if nvim_win_get_w_p_fen(wp) {
                    let fen_val = if rs_foldmethodIsManual(wp) != 0 {
                        false
                    } else {
                        nvim_win_get_w_p_fen_save(wp)
                    };
                    nvim_win_set_w_p_fen(wp, fen_val);
                }
                rs_foldUpdateAll(wp);
            }

            // Remove filler lines
            nvim_win_set_topfill(wp, 0);

            // make sure topline is not halfway a fold and cursor is invalidated
            nvim_diff_changed_window_setting(wp);

            // Note: 'sbo' is not restored, it's a global option.
            rs_diff_buf_adjust(wp);
        }
        diffwin |= nvim_win_get_p_diff(wp) != 0;
        wp = nvim_win_next(wp);
    }

    // Also remove hidden buffers from the list.
    if forceit {
        rs_diff_buf_clear();
    }

    if !diffwin {
        nvim_diff_set_need_update(false);
        nvim_tabpage_set_diff_invalid(curtab, 0);
        nvim_tabpage_set_diff_update(curtab, 0);
        rs_diff_clear(curtab);
    }

    // Remove "hor" from 'scrollopt' if there are no diff windows left.
    if !diffwin && nvim_diff_sbo_has_hor() {
        nvim_diff_do_cmdline_cmd(b"set sbo-=hor\0".as_ptr().cast());
    }
}

// =============================================================================
// Additional C FFI declarations needed for winopts (not in buffer.rs)
// =============================================================================

#[allow(dead_code)]
extern "C" {
    fn nvim_win_get_w_p_fdm_save(wp: WinHandle) -> *const c_char;
    fn nvim_win_get_w_p_fdc_save(wp: WinHandle) -> *const c_char;
    fn nvim_diff_is_curwin(wp: WinHandle) -> bool;
    fn nvim_diff_changed_window_foldlevel_reset(wp: WinHandle);
}
