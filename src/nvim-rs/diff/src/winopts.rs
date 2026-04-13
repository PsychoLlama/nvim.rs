//! Diff window option management
//!
//! This module provides Rust implementations for setting and restoring
//! window options when entering/exiting diff mode:
//!   - `rs_diff_win_options`: set diff options on a window
//!   - `rs_ex_diffoff`: `:diffoff` command implementation

#![allow(clippy::must_use_candidate)]

use nvim_ex_cmds_types::ExArg;
use std::ffi::c_char;
use std::os::raw::c_int;

use crate::buffer::{win_mut, win_ref, BufHandle, TabpageHandle, WinHandle};

/// Line number type matching linenr_T (i32).
type LinenrT = i32;

/// Result constants.
#[allow(dead_code)]
const OK: c_int = 1;
#[allow(dead_code)]
const FAIL: c_int = 0;

/// DIFF_FOLLOWWRAP flag value (must match C).
const DIFF_FOLLOWWRAP: c_int = 0x800;
/// DIFF_VERTICAL flag value (must match C).
const DIFF_VERTICAL: c_int = 0x080;
/// WSP_VERT constant (must match C window.h).
const WSP_VERT: c_int = 0x02;

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
    fn nvim_win_get_w_p_scb_save(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_scb_save(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_crb_save(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_crb_save(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_wrap_save(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_wrap_save(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_fen_save(wp: WinHandle) -> bool;
    fn nvim_win_set_w_p_fen_save(wp: WinHandle, val: bool);

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

    // Diff-mode compound operations
    fn nvim_diff_get_foldcolumn() -> c_int;
    fn nvim_diff_set_fdm_to_diff(wp: WinHandle);
    fn changed_window_setting(wp: WinHandle);
    fn nvim_diff_sbo_has_hor() -> bool;
    fn do_cmdline_cmd(cmd: *const c_char);

    // Global diff state
    fn nvim_get_curtab() -> TabpageHandle;
    fn nvim_get_diff_flags() -> c_int;
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
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;

    // Exarg handle (opaque pointer, only used to pass through)
    fn nvim_eap_forceit(eap: *const std::ffi::c_void) -> bool;
}

/// Literal C string helper -- returns pointer to static "manual" string.
const unsafe fn c_str_manual() -> *const c_char {
    c"manual".as_ptr()
}

/// Literal C string helper -- returns pointer to static "0" string.
const unsafe fn c_str_zero() -> *const c_char {
    c"0".as_ptr()
}

/// Set fdc to a single-digit string matching diff_foldcolumn.
/// Uses the atomic free+set accessor with a literal string for each possible value 0-9.
unsafe fn set_fdc_to_foldcolumn(wp: WinHandle, n: c_int) {
    // We need to pass a C string with the digit. Use match to select a static literal.
    let s: *const c_char = match n {
        0 => c"0".as_ptr(),
        1 => c"1".as_ptr(),
        3 => c"3".as_ptr(),
        4 => c"4".as_ptr(),
        5 => c"5".as_ptr(),
        6 => c"6".as_ptr(),
        7 => c"7".as_ptr(),
        8 => c"8".as_ptr(),
        9 => c"9".as_ptr(),
        _ => c"2".as_ptr(), // default (covers 2 and out-of-range)
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
#[export_name = "diff_win_options"]
pub unsafe extern "C" fn rs_diff_win_options(wp: WinHandle, addbuf: bool) {
    // Close manually opened folds: temporarily switch curwin to wp.
    // The C code does: curwin = wp; rs_newFoldLevel(); curwin = old_curwin;
    // We call a compound accessor that does this atomically.
    nvim_diff_changed_window_foldlevel_reset(wp);

    let not_yet_in_diff = !win_ref(wp).w_p_diff() != 0;

    // scrollbind
    if not_yet_in_diff {
        nvim_win_set_w_p_scb_save(wp, win_ref(wp).w_p_scb() != 0);
    }
    win_mut(wp).set_w_p_scb(c_int::from(true));

    // cursorbind
    if not_yet_in_diff {
        nvim_win_set_w_p_crb_save(wp, win_ref(wp).w_p_crb() != 0);
    }
    win_mut(wp).set_w_p_crb(c_int::from(true));

    // wrap (only when not DIFF_FOLLOWWRAP)
    let diff_flags = nvim_get_diff_flags();
    if diff_flags & DIFF_FOLLOWWRAP == 0 {
        if not_yet_in_diff {
            nvim_win_set_w_p_wrap_save(wp, win_ref(wp).w_p_wrap() != 0);
        }
        win_mut(wp).set_w_p_wrap(c_int::from(false));
        win_mut(wp).w_skipcol = 0;
    }

    // foldmethod save + set to "diff"
    if not_yet_in_diff {
        // free existing fdm_save if already saved, then save current fdm
        nvim_win_free_and_set_fdm_save(wp, nvim_win_get_w_p_fdm(wp));
    }
    nvim_diff_set_fdm_to_diff(wp);

    // fen_save, fdl_save, fdc_save
    if not_yet_in_diff {
        nvim_win_set_w_p_fen_save(wp, win_ref(wp).w_p_fen() != 0);
        nvim_win_set_w_p_fdl_save(wp, nvim_win_get_w_p_fdl(wp));
        // free existing fdc_save if already saved, then save current fdc
        nvim_win_free_and_set_fdc_save(wp, nvim_win_get_w_p_fdc(wp));
    }

    // Set fdc to diff_foldcolumn value (as C string digit)
    let fc = nvim_diff_get_foldcolumn();
    set_fdc_to_foldcolumn(wp, fc);

    // fen = true, fdl = 0
    win_mut(wp).set_w_p_fen(c_int::from(true));
    nvim_win_set_w_p_fdl(wp, 0);
    rs_foldUpdateAll(wp);

    // make sure topline is not halfway through a fold
    changed_window_setting(wp);

    // add 'hor' to 'sbo' if not present
    if !nvim_diff_sbo_has_hor() {
        do_cmdline_cmd(c"set sbo+=hor".as_ptr());
    }

    // mark options as saved
    nvim_win_set_w_p_diff_saved(wp, true);

    // set w_p_diff = true via the option setter
    rs_set_diff_option(wp, true);

    if addbuf {
        rs_diff_buf_add(nvim_win_get_w_buffer(wp));
    }

    let upd: c_int = 40; // UPD_NOT_VALID
    nvim_redraw_later_win(wp, upd);
}

/// Set options NOT to show diffs.  For the current window or all windows.
/// Only in the current tab page.
///
/// Faithfully reproduces the C `ex_diffoff` logic.
///
/// # Safety
/// Calls C functions that access global Neovim state.
#[export_name = "ex_diffoff"]
pub unsafe extern "C" fn rs_ex_diffoff(eap: *const ExArg) {
    let forceit = nvim_eap_forceit(eap.cast::<std::ffi::c_void>());
    let curtab = nvim_get_curtab();
    let mut diffwin = false;

    // Iterate all windows in current tab
    let mut wp = nvim_tabpage_first_win(curtab);
    while !wp.is_null() {
        let is_curwin = nvim_diff_is_curwin(wp);
        if if forceit {
            win_ref(wp).w_p_diff() != 0
        } else {
            is_curwin
        } {
            // Turn off diff option
            rs_set_diff_option(wp, false);

            if nvim_win_get_w_p_diff_saved(wp) {
                // Restore scrollbind
                if win_ref(wp).w_p_scb() != 0 {
                    win_mut(wp).set_w_p_scb(c_int::from(nvim_win_get_w_p_scb_save(wp)));
                }
                // Restore cursorbind
                if win_ref(wp).w_p_crb() != 0 {
                    win_mut(wp).set_w_p_crb(c_int::from(nvim_win_get_w_p_crb_save(wp)));
                }
                // Restore wrap
                let diff_flags = nvim_get_diff_flags();
                if diff_flags & DIFF_FOLLOWWRAP == 0
                    && !win_ref(wp).w_p_wrap() != 0
                    && nvim_win_get_w_p_wrap_save(wp)
                {
                    win_mut(wp).set_w_p_wrap(c_int::from(true));
                    win_mut(wp).w_leftcol = 0;
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
                if win_ref(wp).w_p_fen() != 0 {
                    let fen_val = if rs_foldmethodIsManual(wp) != 0 {
                        false
                    } else {
                        nvim_win_get_w_p_fen_save(wp)
                    };
                    win_mut(wp).set_w_p_fen(c_int::from(fen_val));
                }
                rs_foldUpdateAll(wp);
            }

            // Remove filler lines
            win_mut(wp).w_topfill = 0;

            // make sure topline is not halfway a fold and cursor is invalidated
            changed_window_setting(wp);

            // Note: 'sbo' is not restored, it's a global option.
            rs_diff_buf_adjust(wp);
        }
        diffwin |= win_ref(wp).w_p_diff() != 0;
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
        do_cmdline_cmd(c"set sbo-=hor".as_ptr());
    }
}

/// `:diffsplit` command -- Rust implementation.
///
/// Splits the window and edits another file with diff options enabled on both windows.
///
/// # Safety
/// Calls C functions that access global Neovim state.
#[export_name = "ex_diffsplit"]
pub unsafe extern "C" fn rs_ex_diffsplit(eap: *mut ExArg) {
    let old_curwin = nvim_diff_get_curwin();

    // Allocate a heap bufref to track old_curbuf across the split
    let old_curbuf_ref = nvim_diff_bufref_alloc();
    nvim_diff_bufref_set_to_curbuf(old_curbuf_ref);

    // Need to compute w_fraction when no redraw happened yet.
    validate_cursor(nvim_diff_get_curwin());
    rs_set_fraction(old_curwin);

    // don't use a new tab page, each tab page has its own diffs
    nvim_diff_set_cmdmod_tab_zero();

    let split_flags = if nvim_get_diff_flags() & DIFF_VERTICAL != 0 {
        WSP_VERT
    } else {
        0
    };
    if rs_win_split(0, split_flags) == FAIL {
        nvim_diff_bufref_free(old_curbuf_ref);
        return;
    }

    // Pretend it was a ":split fname" command
    let cmd_split = nvim_diff_get_CMD_split();
    (*eap).cmdidx = cmd_split;
    nvim_diff_set_curwin_w_p_diff(true);
    nvim_diff_do_exedit_with_old_curwin(eap, old_curwin);

    let curwin = nvim_diff_get_curwin();
    if curwin == old_curwin {
        // split didn't work
        nvim_diff_bufref_free(old_curbuf_ref);
        return;
    }

    // Set 'diff', 'scrollbind' on and 'wrap' off.
    rs_diff_win_options(curwin, true);
    if rs_win_valid(old_curwin) != 0 {
        rs_diff_win_options(old_curwin, true);

        if nvim_diff_bufref_valid(old_curbuf_ref) {
            // Move the cursor position to that of the old window.
            let old_lnum = win_ref(old_curwin).w_cursor.lnum;
            let old_buf = nvim_diff_bufref_get_buf(old_curbuf_ref);
            let new_lnum = rs_diff_get_corresponding_line(old_buf, old_lnum);
            win_mut(curwin).w_cursor.lnum = new_lnum;
        }
    }

    // Now that lines are folded scroll to show the cursor at the same relative position.
    let height = nvim_win_get_w_height(curwin);
    rs_scroll_to_fraction(curwin, height);

    nvim_diff_bufref_free(old_curbuf_ref);
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

    // Phase 2: ex_diffsplit accessors
    fn validate_cursor(wp: WinHandle);
    fn nvim_diff_set_cmdmod_tab_zero();
    fn rs_win_split(size: c_int, flags: c_int) -> c_int; // in window_shim.c
    fn nvim_diff_do_exedit_with_old_curwin(eap: *mut ExArg, old_curwin: WinHandle);
    fn nvim_diff_get_CMD_split() -> c_int;
    fn nvim_diff_set_curwin_w_p_diff(val: bool);
    fn nvim_diff_bufref_alloc() -> *mut std::ffi::c_void;
    fn nvim_diff_bufref_free(r: *mut std::ffi::c_void);
    fn nvim_diff_bufref_set_to_curbuf(r: *mut std::ffi::c_void);
    fn nvim_diff_bufref_valid(r: *const std::ffi::c_void) -> bool;
    fn nvim_diff_bufref_get_buf(r: *const std::ffi::c_void) -> BufHandle;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int; // in window_shim.c
    fn nvim_diff_get_curwin() -> WinHandle; // get global curwin
    fn rs_set_fraction(wp: WinHandle); // in window Rust crate
    fn rs_scroll_to_fraction(wp: WinHandle, prev_height: c_int); // in window Rust crate
    fn rs_diff_get_corresponding_line(buf: BufHandle, lnum: LinenrT) -> LinenrT;
    fn rs_win_valid(wp: WinHandle) -> c_int;
}
