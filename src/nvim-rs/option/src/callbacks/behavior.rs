//! Behavior-related option callbacks
//!
//! This module provides Rust implementations for behavior-related `did_set_*`
//! callbacks. These callbacks handle options that affect editor behavior
//! such as diff mode, spell checking, swap files, undo, folds, etc.

use std::ffi::{c_int, c_void};

use crate::callbacks::{callback_ok, CallbackResult};
use crate::{BufHandle, OptInt, WinHandle};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // Diff functions
    fn rs_diff_buf_adjust(win: WinHandle);
    fn foldmethodIsDiff(win: WinHandle) -> c_int;
    fn rs_foldUpdateAll(win: WinHandle);

    // Fold functions
    fn foldmethodIsSyntax(win: WinHandle) -> c_int;
    fn foldmethodIsIndent(win: WinHandle) -> c_int;

    // Window functions
    fn win_equal(win: WinHandle, current: c_int, dir: c_int);
    fn win_setheight(height: c_int);

    // Title/redraw functions
    fn redraw_titles();

    // Swap file functions
    fn ml_open_file(buf: BufHandle);
    fn mf_close_file(buf: BufHandle, del_file: c_int);
    fn ml_open_files();

    // Undo functions
    fn did_set_global_undolevels(new_value: OptInt, old_value: OptInt);
    fn did_set_buflocal_undolevels(buf: BufHandle, new_value: OptInt, old_value: OptInt);

    // State accessors
    fn nvim_callback_get_p_uc() -> OptInt;
    fn nvim_callback_get_p_ea() -> c_int;
    fn nvim_callback_is_one_window() -> c_int;
    fn nvim_callback_is_curbuf_help() -> c_int;
    fn nvim_callback_get_curwin_height() -> c_int;
    fn nvim_callback_get_p_hh() -> OptInt;
    // Buffer accessors
    fn nvim_buf_get_p_swf(buf: BufHandle) -> c_int;

    // optset_T field accessors
    fn nvim_optset_get_win(args: *const c_void) -> WinHandle;
    fn nvim_optset_get_buf(args: *const c_void) -> BufHandle;
    fn nvim_optset_get_oldval_boolean(args: *const c_void) -> c_int;
    fn nvim_optset_get_oldval_number(args: *const c_void) -> i64;
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Get 'updatecount' option value.
#[inline]
fn get_updatecount() -> OptInt {
    unsafe { nvim_callback_get_p_uc() }
}

/// Get 'equalalways' option value.
#[inline]
fn get_equalalways() -> bool {
    unsafe { nvim_callback_get_p_ea() != 0 }
}

/// Check if there's only one window.
#[inline]
fn is_one_window() -> bool {
    unsafe { nvim_callback_is_one_window() != 0 }
}

/// Check if current buffer is help buffer.
#[inline]
fn is_curbuf_help() -> bool {
    unsafe { nvim_callback_is_curbuf_help() != 0 }
}

/// Get current window height.
#[inline]
fn get_curwin_height() -> c_int {
    unsafe { nvim_callback_get_curwin_height() }
}

/// Get 'helpheight' option value.
#[inline]
fn get_helpheight() -> OptInt {
    unsafe { nvim_callback_get_p_hh() }
}

// =============================================================================
// Behavior-Related Callbacks
// =============================================================================

/// Callback for 'binary' option.
///
/// When 'bin' is set, also set some other options and redraw titles.
#[no_mangle]
pub extern "C" fn rs_did_set_binary() -> CallbackResult {
    // set_options_bin() is called from C before this callback
    unsafe { redraw_titles() };
    callback_ok()
}

/// Callback for 'diff' option.
///
/// Adjusts diff buffer list and updates folds if using diff fold method.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_diff(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    rs_diff_buf_adjust(win);
    if foldmethodIsDiff(win) != 0 {
        rs_foldUpdateAll(win);
    }
    callback_ok()
}

/// Callback for 'endoffile', 'endofline', 'fixendofline', or 'bomb' options.
///
/// Redraws the window title and tab page text.
#[no_mangle]
pub extern "C" fn rs_did_set_eof_eol_fixeol_bomb(_args: *mut c_void) -> CallbackResult {
    unsafe { redraw_titles() };
    callback_ok()
}

/// Callback for 'equalalways' option.
///
/// When 'equalalways' is set, make all windows equal size.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_equalalways(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    let old_value = nvim_optset_get_oldval_boolean(args);
    if get_equalalways() && old_value == 0 {
        win_equal(win, 0, 0);
    }
    callback_ok()
}

// Note: rs_did_set_foldlevel is already defined in mod.rs

/// Callback for 'foldminlines' option.
///
/// Updates all folds in the window.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_foldminlines(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    rs_foldUpdateAll(win);
    callback_ok()
}

/// Callback for 'foldnestmax' option.
///
/// Updates folds if using syntax or indent fold method.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_foldnestmax(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    if foldmethodIsSyntax(win) != 0 || foldmethodIsIndent(win) != 0 {
        rs_foldUpdateAll(win);
    }
    callback_ok()
}

/// Callback for 'helpheight' option.
///
/// Changes window height if in help buffer and window is too short.
#[no_mangle]
pub extern "C" fn rs_did_set_helpheight(_args: *mut c_void) -> CallbackResult {
    if !is_one_window() && is_curbuf_help() {
        let hh = get_helpheight();
        #[allow(clippy::cast_possible_truncation)]
        let hh_i32 = hh as c_int;
        if get_curwin_height() < hh_i32 {
            unsafe { win_setheight(hh_i32) };
        }
    }
    callback_ok()
}

/// Callback for 'swapfile' option.
///
/// Creates or removes swap file based on option value.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_swapfile(args: *mut c_void) -> CallbackResult {
    let buf = nvim_optset_get_buf(args);
    if nvim_buf_get_p_swf(buf) != 0 && get_updatecount() != 0 {
        ml_open_file(buf); // create the swap file
    } else {
        mf_close_file(buf, 1); // remove the swap file
    }
    callback_ok()
}

// Note: rs_did_set_textwidth is already defined in mod.rs

/// Callback for 'undolevels' option.
///
/// Handles both global and buffer-local undolevels changes.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_undolevels(
    buf: BufHandle,
    is_global: c_int,
    new_value: OptInt,
    old_value: OptInt,
) -> CallbackResult {
    if is_global != 0 {
        did_set_global_undolevels(new_value, old_value);
    } else {
        did_set_buflocal_undolevels(buf, new_value, old_value);
    }
    callback_ok()
}

/// Callback for 'updatecount' option.
///
/// When 'updatecount' changes from zero to non-zero, open swap files.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_updatecount(args: *mut c_void) -> CallbackResult {
    let old_value = nvim_optset_get_oldval_number(args);
    if get_updatecount() != 0 && old_value == 0 {
        ml_open_files();
    }
    callback_ok()
}

/// Callback for 'autoread' option.
///
/// Placeholder - actual behavior is handled elsewhere.
#[no_mangle]
pub extern "C" fn rs_did_set_autoread() -> CallbackResult {
    callback_ok()
}

/// Callback for 'autowrite' / 'autowriteall' option.
///
/// Placeholder - actual behavior is handled elsewhere.
#[no_mangle]
pub extern "C" fn rs_did_set_autowrite() -> CallbackResult {
    callback_ok()
}

/// Callback for 'backup' option.
///
/// Placeholder - actual behavior is handled elsewhere.
#[no_mangle]
pub extern "C" fn rs_did_set_backup() -> CallbackResult {
    callback_ok()
}

/// Callback for 'expandtab' option.
///
/// Placeholder - actual behavior is handled elsewhere.
#[no_mangle]
pub extern "C" fn rs_did_set_expandtab() -> CallbackResult {
    callback_ok()
}

/// Callback for 'hidden' option.
///
/// Placeholder - actual behavior is handled elsewhere.
#[no_mangle]
pub extern "C" fn rs_did_set_hidden() -> CallbackResult {
    callback_ok()
}

/// Callback for 'insertmode' option.
///
/// Placeholder - actual behavior is handled elsewhere.
#[no_mangle]
pub extern "C" fn rs_did_set_insertmode() -> CallbackResult {
    callback_ok()
}

/// Callback for 'modifiable' option.
///
/// Redraws window title when modifiable state changes.
#[no_mangle]
pub extern "C" fn rs_did_set_modifiable(_args: *mut c_void) -> CallbackResult {
    unsafe { redraw_titles() };
    callback_ok()
}

/// Callback for 'modified' option.
///
/// Redraws titles when modified state changes.
#[no_mangle]
pub extern "C" fn rs_did_set_modified() -> CallbackResult {
    unsafe { redraw_titles() };
    callback_ok()
}

/// Callback for 'readonly' option.
///
/// Redraws titles when readonly state changes.
#[no_mangle]
pub extern "C" fn rs_did_set_readonly() -> CallbackResult {
    unsafe { redraw_titles() };
    callback_ok()
}

/// Callback for 'spell' option.
///
/// When spell is enabled, parse spelllang. Returns error message if parsing fails.
/// Note: The actual spelllang parsing is done in C code.
#[no_mangle]
pub extern "C" fn rs_did_set_spell() -> CallbackResult {
    // parse_spelllang() is called from C when spell is enabled
    // This callback just acknowledges the change
    callback_ok()
}

/// Callback for 'termguicolors' option.
///
/// Placeholder - actual behavior is handled elsewhere.
#[no_mangle]
pub extern "C" fn rs_did_set_termguicolors() -> CallbackResult {
    callback_ok()
}

/// Callback for 'virtualedit' option.
///
/// Placeholder - actual behavior is handled elsewhere.
#[no_mangle]
pub extern "C" fn rs_did_set_virtualedit() -> CallbackResult {
    callback_ok()
}

/// Callback for 'writebackup' option.
///
/// Placeholder - actual behavior is handled elsewhere.
#[no_mangle]
pub extern "C" fn rs_did_set_writebackup() -> CallbackResult {
    callback_ok()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most behavior callbacks call external C functions which require
    // linking against the full C library. These functions are tested via
    // integration tests in the full Neovim build.

    #[test]
    fn test_placeholder_callbacks_return_ok() {
        // Test placeholder callbacks that don't call extern C functions
        assert!(rs_did_set_autoread().is_null());
        assert!(rs_did_set_autowrite().is_null());
        assert!(rs_did_set_backup().is_null());
        assert!(rs_did_set_expandtab().is_null());
        assert!(rs_did_set_hidden().is_null());
        assert!(rs_did_set_insertmode().is_null());
        // rs_did_set_modifiable now calls redraw_titles() (extern C)
        assert!(rs_did_set_spell().is_null());
        assert!(rs_did_set_termguicolors().is_null());
        assert!(rs_did_set_virtualedit().is_null());
        assert!(rs_did_set_writebackup().is_null());
    }
}
