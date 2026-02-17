//! Display-related option callbacks
//!
//! This module provides Rust implementations for display-related `did_set_*`
//! callbacks. These callbacks handle options that affect how the screen is
//! drawn, window layouts, status lines, and visual appearance.

use std::ffi::{c_char, c_int, c_void};

use crate::callbacks::{callback_ok, request_redraw_all, CallbackResult, UpdateType};
use crate::{OptInt, WinHandle};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // Screen/window functions
    fn command_height();
    fn win_new_screen_rows();
    fn win_comp_pos();
    fn status_redraw_curbuf();
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

    // State accessors
    fn nvim_callback_get_full_screen() -> c_int;
    fn nvim_callback_get_p_ch() -> OptInt;
    fn nvim_callback_set_p_ch(value: OptInt);
    fn nvim_callback_get_rows() -> c_int;
    fn nvim_callback_get_topframe() -> *mut std::ffi::c_void;
    fn nvim_callback_get_topframe_fr_height() -> c_int;
    fn nvim_callback_tabline_height() -> c_int;
    fn nvim_callback_global_stl_height() -> c_int;
    fn nvim_callback_min_rows_curtab() -> c_int;
    fn nvim_callback_set_clear_cmdline(value: c_int);

    // Window accessors
    fn nvim_option_win_get_stc(win: WinHandle) -> *const c_char;
    fn nvim_option_win_set_nrwidth(win: WinHandle, value: c_int);
    fn nvim_option_win_get_sms(win: WinHandle) -> c_int;
    fn nvim_option_win_set_skipcol(win: WinHandle, value: c_int);
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
    unsafe { nvim_callback_get_p_ch() }
}

/// Set 'cmdheight' value.
#[inline]
fn set_cmdheight(value: OptInt) {
    unsafe { nvim_callback_set_p_ch(value) }
}

/// Get Rows (screen height).
#[inline]
fn get_rows() -> c_int {
    unsafe { nvim_callback_get_rows() }
}

/// Check if full screen is available.
#[inline]
fn full_screen() -> bool {
    unsafe { nvim_callback_get_full_screen() != 0 }
}

/// Get topframe height.
#[inline]
fn get_topframe_height() -> c_int {
    unsafe { nvim_callback_get_topframe_fr_height() }
}

/// Get tabline height.
#[inline]
fn tabline_height() -> c_int {
    unsafe { nvim_callback_tabline_height() }
}

/// Get global status line height.
#[inline]
fn global_stl_height() -> c_int {
    unsafe { nvim_callback_global_stl_height() }
}

/// Get minimum rows for current tab.
#[inline]
fn min_rows_curtab() -> c_int {
    unsafe { nvim_callback_min_rows_curtab() }
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
    if (new_ch != old_value || layout_mismatch) && full_screen() {
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
        nvim_callback_set_clear_cmdline(1);
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
pub unsafe extern "C" fn rs_did_set_number_relativenumber(win: WinHandle) -> CallbackResult {
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
pub unsafe extern "C" fn rs_did_set_numberwidth(win: WinHandle) -> CallbackResult {
    nvim_option_win_set_nrwidth(win, 0);
    callback_ok()
}

/// Callback for 'smoothscroll' option.
///
/// Resets skipcol when smoothscroll is disabled.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_smoothscroll_full(win: WinHandle) -> CallbackResult {
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

/// Callback for 'colorcolumn' option.
///
/// Triggers redraw when color column changes.
#[no_mangle]
pub extern "C" fn rs_did_set_colorcolumn() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// Callback for 'list' option.
///
/// Triggers redraw when list mode changes.
#[no_mangle]
pub extern "C" fn rs_did_set_list() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// Callback for 'wrap' option.
///
/// Triggers redraw when wrap mode changes.
#[no_mangle]
pub extern "C" fn rs_did_set_wrap() -> CallbackResult {
    request_redraw_all(UpdateType::NotValid);
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

/// Callback for 'signcolumn' option.
///
/// Checks and updates sign column display.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_signcolumn(win: WinHandle) -> CallbackResult {
    check_signcolumn(std::ptr::null_mut(), win);
    callback_ok()
}

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

/// Callback for 'concealcursor' option.
///
/// Triggers redraw when conceal cursor mode changes.
#[no_mangle]
pub extern "C" fn rs_did_set_concealcursor() -> CallbackResult {
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
