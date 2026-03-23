//! Display-related option callbacks
//!
//! This module provides Rust implementations for display-related `did_set_*`
//! callbacks. These callbacks handle options that affect how the screen is
//! drawn, window layouts, status lines, and visual appearance.

use std::ffi::{c_char, c_int, c_void};

use crate::callbacks::{callback_ok, request_redraw_all, CallbackResult, UpdateType};
use crate::{OptInt, WinHandle};

// Direct C globals.
extern "C" {
    static updating_screen: bool;
    static mut Rows: c_int;
    static mut Columns: c_int;
    static mut full_screen: bool;
}

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // Screen/window functions
    fn command_height();
    fn win_new_screen_rows();
    #[link_name = "rs_win_comp_pos"]
    fn win_comp_pos();
    fn status_redraw_curbuf();
    #[link_name = "rs_last_status"]
    fn last_status(morestrict: c_int);
    fn win_float_update_statusline();
    fn frame_new_height(
        topfrp: *mut std::ffi::c_void,
        height: c_int,
        topfirst: c_int,
        wfh: c_int,
        trigger_resize_aucmds: c_int,
    );
    fn check_signcolumn(buf: *mut std::ffi::c_void, win: WinHandle);

    // Direct C globals
    static mut p_window: crate::OptInt;
    fn nvim_callback_get_topframe() -> *mut std::ffi::c_void;
    fn nvim_callback_get_topframe_fr_height() -> c_int;
    fn rs_tabline_height() -> c_int;
    fn rs_global_stl_height() -> c_int;
    fn nvim_get_curtab() -> *mut std::ffi::c_void;
    fn rs_min_rows(tp: *mut std::ffi::c_void) -> c_int;
    static mut clear_cmdline: bool;

    // Window accessors
    fn nvim_option_win_get_stc(win: WinHandle) -> *const c_char;
    fn nvim_option_win_set_nrwidth(win: WinHandle, value: c_int);
    fn nvim_option_win_get_sms(win: WinHandle) -> c_int;
    fn nvim_option_win_set_skipcol(win: WinHandle, value: c_int);

    // optset_T field accessors
    fn nvim_optset_get_win(args: *const c_void) -> WinHandle;
    fn nvim_optset_restore_oldval_number(args: *const c_void);

    // Window field accessors for wrap callback
    fn nvim_win_get_p_wrap(wp: WinHandle) -> c_int;
    fn nvim_win_set_leftcol(wp: WinHandle, val: c_int);

    // Phase 1: string_simple accessors used by concealcursor
    fn nvim_optset_get_varp_str(args: *const c_void) -> *const c_char;
    fn nvim_optset_get_errbuf(args: *const c_void) -> *mut c_char;
    fn nvim_optset_get_errbuflen(args: *const c_void) -> usize;
    fn nvim_illegal_char(errbuf: *mut c_char, errbuflen: usize, c: c_int) -> *const c_char;

    // Lines/columns callback accessors
    fn nvim_get_full_screen() -> bool;
    fn screen_resize(width: c_int, height: c_int);
    fn check_screensize();
    fn nvim_get_cmdline_row() -> c_int;
    fn nvim_set_cmdline_row(val: c_int);
    fn nvim_get_p_ch() -> i64;
    fn nvim_get_p_sj() -> OptInt;
    fn nvim_set_p_sj(val: OptInt);
    fn nvim_option_was_set_window() -> c_int;

    // Phase 107: colorcolumn / background / fileformat - call C directly
    fn did_set_colorcolumn(args: *mut c_void) -> CallbackResult;
    fn did_set_background(args: *mut c_void) -> CallbackResult;
    fn did_set_fileformat(args: *mut c_void) -> CallbackResult;
}

// =============================================================================
// Constants
// =============================================================================

/// Status line height
const STATUS_HEIGHT: c_int = 1;

// =============================================================================
// Helper Functions
// =============================================================================

/// Get 'cmdheight' value.
#[inline]
fn get_cmdheight() -> OptInt {
    unsafe { crate::p_ch }
}

/// Set 'cmdheight' value.
#[inline]
fn set_cmdheight(value: OptInt) {
    unsafe { crate::p_ch = value }
}

/// Get Rows (screen height).
#[inline]
fn get_rows() -> c_int {
    unsafe { Rows }
}

/// Check if full screen is available.
#[inline]
fn get_full_screen() -> bool {
    unsafe { full_screen }
}

/// Get topframe height.
#[inline]
fn get_topframe_height() -> c_int {
    unsafe { nvim_callback_get_topframe_fr_height() }
}

/// Get tabline height.
#[inline]
fn tabline_height() -> c_int {
    unsafe { rs_tabline_height() }
}

/// Get global status line height.
#[inline]
fn global_stl_height() -> c_int {
    unsafe { rs_global_stl_height() }
}

/// Get minimum rows for current tab.
#[inline]
fn min_rows_curtab() -> c_int {
    unsafe { rs_min_rows(nvim_get_curtab()) }
}

// =============================================================================
// Display-Related Callbacks
// =============================================================================

/// Callback for 'cmdheight' option.
///
/// Adjusts command line height based on new value.
/// Ensures cmdheight doesn't exceed available space.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_cmdheight(old_value: OptInt) -> CallbackResult {
    let rows = get_rows();
    let min_rows = min_rows_curtab();

    // Ensure cmdheight doesn't exceed available space
    let max_ch = OptInt::from(rows - min_rows + 1);
    if get_cmdheight() > max_ch {
        set_cmdheight(max_ch);
    }

    // If cmdheight changed and full screen is available, update layout
    let new_ch = get_cmdheight();
    #[allow(clippy::cast_possible_truncation)]
    let new_ch_i32 = new_ch as c_int;
    let layout_mismatch =
        tabline_height() + global_stl_height() + get_topframe_height() != rows - new_ch_i32;
    if (new_ch != old_value || layout_mismatch) && get_full_screen() {
        command_height();
    }

    callback_ok()
}

/// Callback for 'laststatus' option.
///
/// Handles transitions to/from global statusline (value 3).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_laststatus_full(
    old_value: OptInt,
    new_value: OptInt,
) -> CallbackResult {
    let topframe = nvim_callback_get_topframe();

    // When switching to global statusline, decrease topframe height
    if new_value == 3 && old_value != 3 {
        let new_height = get_topframe_height() - STATUS_HEIGHT;
        frame_new_height(topframe, new_height, 0, 0, 0);
        win_comp_pos();
        clear_cmdline = true;
    }

    // When switching from global statusline, increase topframe height
    if old_value == 3 && new_value != 3 {
        let new_height = get_topframe_height() + STATUS_HEIGHT;
        frame_new_height(topframe, new_height, 0, 0, 0);
        win_comp_pos();
    }

    status_redraw_curbuf();
    last_status(0); // (re)set last window status line
    win_float_update_statusline();

    callback_ok()
}

/// Callback for 'showtabline' option.
///
/// Recomputes window positions and heights when tabline visibility changes.
#[no_mangle]
pub extern "C" fn rs_did_set_showtabline_full(_args: *mut c_void) -> CallbackResult {
    unsafe {
        win_new_screen_rows();
    }
    callback_ok()
}

/// Callback for 'number' or 'relativenumber' option.
///
/// When these options change and 'statuscolumn' is set, reset width.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_number_relativenumber(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    let stc = nvim_option_win_get_stc(win);
    if !stc.is_null() && *stc != 0 {
        nvim_option_win_set_nrwidth(win, 0);
    }
    check_signcolumn(std::ptr::null_mut(), win);
    callback_ok()
}

/// Callback for 'numberwidth' option.
///
/// Triggers a redraw by resetting the line count used for number width calculation.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_numberwidth(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    nvim_option_win_set_nrwidth(win, 0);
    callback_ok()
}

/// Callback for 'smoothscroll' option.
///
/// Resets skipcol when smoothscroll is disabled.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_smoothscroll_full(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    if nvim_option_win_get_sms(win) == 0 {
        nvim_option_win_set_skipcol(win, 0);
    }
    callback_ok()
}

/// Callback for 'ruler' option.
///
/// Triggers redraw of ruler/statusline areas.
#[no_mangle]
pub extern "C" fn rs_did_set_ruler() -> CallbackResult {
    // comp_col() is called by post-set processing
    // Status line redraw is handled by check_redraw
    callback_ok()
}

/// Callback for 'showcmd' option.
///
/// Triggers redraw of showcase area.
#[no_mangle]
pub extern "C" fn rs_did_set_showcmd() -> CallbackResult {
    // comp_col() is called by post-set processing
    callback_ok()
}

/// Callback for 'cursorline' option.
///
/// Triggers redraw of current line highlighting.
#[no_mangle]
pub extern "C" fn rs_did_set_cursorline() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// Callback for 'cursorcolumn' option.
///
/// Triggers redraw of current column highlighting.
#[no_mangle]
pub extern "C" fn rs_did_set_cursorcolumn() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// Callback for 'colorcolumn' option (Phase 107).
///
/// Validates the colorcolumn format via check_colorcolumn.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_colorcolumn(args: *mut c_void) -> CallbackResult {
    did_set_colorcolumn(args)
}

/// Callback for 'background' option (Phase 107).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_background(args: *mut c_void) -> CallbackResult {
    did_set_background(args)
}

/// Callback for 'fileformat' option (Phase 107).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_fileformat(args: *mut c_void) -> CallbackResult {
    did_set_fileformat(args)
}

/// Callback for 'list' option.
///
/// Triggers redraw when list mode changes.
#[no_mangle]
pub extern "C" fn rs_did_set_list() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// Callback for 'wrap' option (full replacement).
///
/// When wrap is enabled, reset leftcol. When disabled, reset skipcol.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_wrap(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    if nvim_win_get_p_wrap(win) != 0 {
        nvim_win_set_leftcol(win, 0);
    } else {
        nvim_option_win_set_skipcol(win, 0);
    }
    callback_ok()
}

/// Callback for 'linebreak' option.
///
/// Triggers redraw when linebreak mode changes.
#[no_mangle]
pub extern "C" fn rs_did_set_linebreak() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// Callback for 'breakindent' option.
///
/// Triggers redraw when breakindent mode changes.
#[no_mangle]
pub extern "C" fn rs_did_set_breakindent() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

// rs_did_set_signcolumn is now in behavior.rs (Phase 103) with full optset_T implementation

/// Callback for 'foldcolumn' option.
///
/// Triggers redraw when fold column width changes.
#[no_mangle]
pub extern "C" fn rs_did_set_foldcolumn() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// Callback for 'conceallevel' option.
///
/// Triggers redraw when conceal level changes.
#[no_mangle]
pub extern "C" fn rs_did_set_conceallevel() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// All valid flags for 'concealcursor' option: n, v, i, c
const COCU_ALL: &[u8] = b"nvic";

/// Callback for 'concealcursor' option.
///
/// Validates that all characters are in "nvic", then triggers redraw.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_concealcursor(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    if !val.is_null() {
        let mut p = val;
        while *p != 0 {
            let ch = *p as u8;
            if !COCU_ALL.contains(&ch) {
                return nvim_illegal_char(errbuf, errbuflen, c_int::from(ch));
            }
            p = p.add(1);
        }
    }
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// Callback for 'fillchars' option.
///
/// Triggers redraw when fill characters change.
#[no_mangle]
pub extern "C" fn rs_did_set_fillchars() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// Callback for 'listchars' option.
///
/// Triggers redraw when list characters change.
#[no_mangle]
pub extern "C" fn rs_did_set_listchars() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

// =============================================================================
// Lines / Columns Option Callback
// =============================================================================

/// Callback for 'lines' and 'columns' options.
///
/// Handles screen resize when 'lines' or 'columns' changes. Adjusts Rows/Columns
/// globals, calls screen_resize if appropriate, and updates dependent options.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_did_set_lines_or_columns(args: *mut c_void) -> CallbackResult {
    let p_lines = crate::p_lines;
    let p_columns = crate::p_columns;
    let rows = Rows;
    let columns = Columns;

    if p_lines != OptInt::from(rows) || p_columns != OptInt::from(columns) {
        if updating_screen {
            // Cannot resize while updating the screen; restore old value.
            nvim_optset_restore_oldval_number(args);
        } else if nvim_get_full_screen() {
            screen_resize(p_columns as c_int, p_lines as c_int);
        } else {
            // Postpone the resizing.
            Rows = p_lines as c_int;
            Columns = p_columns as c_int;
            check_screensize();
            let p_ch = nvim_get_p_ch();
            let new_row = Rows - std::cmp::max(p_ch as c_int, 1);
            let cmdline_row = nvim_get_cmdline_row();
            if cmdline_row > new_row && Rows > p_ch as c_int {
                nvim_set_cmdline_row(new_row);
            }
        }
        let window = p_window;
        if window >= OptInt::from(Rows) || nvim_option_was_set_window() == 0 {
            p_window = OptInt::from(Rows) - 1;
        }
    }

    // Adjust 'scrolljump' if needed.
    if nvim_get_p_sj() >= OptInt::from(Rows) && nvim_get_full_screen() {
        nvim_set_p_sj(OptInt::from(Rows) / 2);
    }

    callback_ok()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        // Test that constants have expected values
        assert_eq!(STATUS_HEIGHT, 1);
    }

    // Note: Most display callbacks call external C functions (request_redraw_all,
    // command_height, etc.) which require linking against the full C library.
    // These functions are tested via integration tests in the full Neovim build.
}
