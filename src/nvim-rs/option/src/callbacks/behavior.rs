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
    fn rs_foldmethodIsDiff(win: WinHandle) -> c_int;
    fn rs_foldUpdateAll(win: WinHandle);

    // Fold functions
    fn rs_foldmethodIsSyntax(win: WinHandle) -> c_int;
    fn rs_foldmethodIsIndent(win: WinHandle) -> c_int;

    // Window functions
    #[link_name = "rs_win_equal"]
    fn win_equal(win: WinHandle, current: c_int, dir: c_int);
    #[link_name = "rs_win_setheight"]
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
    fn nvim_optset_get_newval_number(args: *const c_void) -> i64;
    fn nvim_optset_get_varp(args: *const c_void) -> *mut c_void;

    // Additional window/option accessors for full callbacks
    #[link_name = "rs_win_setwidth"]
    fn win_setwidth(width: c_int);
    fn nvim_option_get_p_wh() -> OptInt;
    fn nvim_option_get_p_wiw() -> OptInt;
    fn nvim_callback_get_curwin_width() -> c_int;

    // Modified/readonly callback accessors
    fn save_file_ff(buf: BufHandle);
    fn nvim_optset_get_newval_boolean(args: *const c_void) -> c_int;
    fn nvim_optset_get_flags(args: *const c_void) -> c_int;
    fn nvim_option_buf_set_modified_was_set(buf: BufHandle, val: c_int);
    fn nvim_option_buf_get_b_p_ro(buf: BufHandle) -> c_int;
    fn nvim_option_buf_set_b_did_warn(buf: BufHandle, val: c_int);
    fn nvim_callback_set_readonlymode(val: c_int);

    // Scrollback callback accessors
    fn nvim_option_buf_get_terminal_ptr(buf: BufHandle) -> *mut c_void;
    fn on_scrollback_option_changed(terminal: *mut c_void);

    // Undolevels callback accessors
    fn nvim_callback_get_p_ul_addr() -> *mut c_void;

    // Binary callback
    fn set_options_bin(oldval: c_int, newval: c_int, opt_flags: c_int);
    fn nvim_option_buf_get_b_p_bin(buf: BufHandle) -> c_int;

    // Buflisted callback
    fn nvim_buf_get_p_bl(buf: BufHandle) -> c_int;
    fn nvim_apply_autocmds_buf_event(event: c_int, buf: BufHandle);

    // Previewwindow callback
    fn nvim_win_get_p_pvw(win: WinHandle) -> c_int;
    fn nvim_win_set_p_pvw(win: WinHandle, val: c_int);
    fn nvim_for_all_windows_in_curtab(
        callback: unsafe extern "C" fn(WinHandle, *mut c_void),
        ud: *mut c_void,
    );
    fn nvim_get_e_preview_window_exists() -> *const std::ffi::c_char;

    // Spell callback
    fn nvim_win_get_p_spell(win: WinHandle) -> c_int;
    fn nvim_parse_spelllang(win: WinHandle) -> CallbackResult;

    // Shiftwidth/tabstop callback
    fn nvim_parse_cino(buf: BufHandle);
    fn nvim_buf_get_b_p_sw_addr(buf: BufHandle) -> *mut c_void;

    // Xhistory callback
    fn nvim_get_p_chi_addr() -> *mut c_void;
    fn nvim_qf_resize_stack(n: c_int);
    fn nvim_ll_resize_stack(win: WinHandle, n: c_int);

    // Shiftwidth buffer-local value
    fn nvim_buf_get_b_p_sw(buf: BufHandle) -> OptInt;

    // fill_culopt_flags accessors
    fn nvim_win_get_p_culopt(win: WinHandle) -> *const std::ffi::c_char;
    fn nvim_win_set_p_culopt_flags(win: WinHandle, flags: u8);

    // set_options_bin global option accessors
    fn nvim_get_p_tw() -> OptInt;
    fn nvim_set_p_tw(v: OptInt);
    fn nvim_get_p_wm() -> OptInt;
    fn nvim_set_p_wm(v: OptInt);
    fn nvim_get_p_ml() -> c_int;
    fn nvim_set_p_ml(v: c_int);
    fn nvim_get_p_et() -> c_int;
    fn nvim_set_p_et(v: c_int);
    fn nvim_set_p_bin(v: c_int);
    fn nvim_get_p_tw_nobin() -> OptInt;
    fn nvim_set_p_tw_nobin(v: OptInt);
    fn nvim_get_p_wm_nobin() -> OptInt;
    fn nvim_set_p_wm_nobin(v: OptInt);
    fn nvim_get_p_ml_nobin() -> c_int;
    fn nvim_set_p_ml_nobin(v: c_int);
    fn nvim_get_p_et_nobin() -> c_int;
    fn nvim_set_p_et_nobin(v: c_int);
    fn nvim_curbuf_get_b_p_tw() -> c_int;
    fn nvim_curbuf_set_b_p_tw(v: OptInt);
    fn nvim_curbuf_get_b_p_wm() -> c_int;
    fn nvim_curbuf_set_b_p_wm(v: OptInt);
    fn nvim_curbuf_get_b_p_ml() -> c_int;
    fn nvim_curbuf_set_b_p_ml(v: c_int);
    fn nvim_curbuf_get_b_p_et() -> c_int;
    fn nvim_curbuf_set_b_p_et(v: c_int);
    fn nvim_curbuf_get_b_p_tw_nobin() -> c_int;
    fn nvim_curbuf_set_b_p_tw_nobin(v: OptInt);
    fn nvim_curbuf_get_b_p_wm_nobin() -> c_int;
    fn nvim_curbuf_set_b_p_wm_nobin(v: OptInt);
    fn nvim_curbuf_get_b_p_ml_nobin() -> c_int;
    fn nvim_curbuf_set_b_p_ml_nobin(v: c_int);
    fn nvim_curbuf_get_b_p_et_nobin() -> c_int;
    fn nvim_curbuf_set_b_p_et_nobin(v: c_int);
    fn nvim_bin_didset_options_sctx(opt_flags: c_int);
}

// =============================================================================
// Constants
// =============================================================================

/// OPT_LOCAL flag from option.h
const OPT_LOCAL: c_int = 0x02;

/// EVENT_BUFADD = 0 (from auevents_enum.generated.h)
const EVENT_BUFADD: c_int = 0;
/// EVENT_BUFDELETE = 2 (from auevents_enum.generated.h)
const EVENT_BUFDELETE: c_int = 2;

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

/// Callback for 'diff' option.
///
/// Adjusts diff buffer list and updates folds if using diff fold method.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_diff(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    rs_diff_buf_adjust(win);
    if rs_foldmethodIsDiff(win) != 0 {
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
    if rs_foldmethodIsSyntax(win) != 0 || rs_foldmethodIsIndent(win) != 0 {
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

/// Callback for 'winheight' option (full replacement).
///
/// Change window height if needed when 'winheight' increases.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_did_set_winheight(_args: *mut c_void) -> CallbackResult {
    if !is_one_window() {
        let p_wh = nvim_option_get_p_wh() as c_int;
        if get_curwin_height() < p_wh {
            win_setheight(p_wh);
        }
    }
    callback_ok()
}

/// Callback for 'winwidth' option (full replacement).
///
/// Change window width if needed when 'winwidth' increases.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_did_set_winwidth(_args: *mut c_void) -> CallbackResult {
    let p_wiw = nvim_option_get_p_wiw() as c_int;
    if !is_one_window() && nvim_callback_get_curwin_width() < p_wiw {
        win_setwidth(p_wiw);
    }
    callback_ok()
}

/// Callback for 'binary' option (full replacement).
///
/// When 'bin' is set, also set some other options and redraw titles.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_binary_full(args: *mut c_void) -> CallbackResult {
    let buf = nvim_optset_get_buf(args);
    let old_val = nvim_optset_get_oldval_boolean(args);
    let new_val = nvim_option_buf_get_b_p_bin(buf);
    set_options_bin(old_val, new_val, nvim_optset_get_flags(args));
    redraw_titles();
    callback_ok()
}

/// Callback for 'modified' option (full replacement).
///
/// When 'modified' is cleared, save file format. Always redraw titles.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_modified(args: *mut c_void) -> CallbackResult {
    let buf = nvim_optset_get_buf(args);
    let newval = nvim_optset_get_newval_boolean(args);
    if newval == 0 {
        save_file_ff(buf);
    }
    redraw_titles();
    nvim_option_buf_set_modified_was_set(buf, newval);
    callback_ok()
}

/// Callback for 'readonly' option (full replacement).
///
/// When 'readonly' is reset globally, also reset readonlymode.
/// When 'readonly' is set, allow W10 warning again.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_readonly(args: *mut c_void) -> CallbackResult {
    let buf = nvim_optset_get_buf(args);
    let flags = nvim_optset_get_flags(args);

    // when 'readonly' is reset globally, also reset readonlymode
    if nvim_option_buf_get_b_p_ro(buf) == 0 && (flags & OPT_LOCAL) == 0 {
        nvim_callback_set_readonlymode(0);
    }

    // when 'readonly' is set may give W10 again
    if nvim_option_buf_get_b_p_ro(buf) != 0 {
        nvim_option_buf_set_b_did_warn(buf, 0);
    }

    redraw_titles();
    callback_ok()
}

/// Callback for 'scrollback' option (full replacement).
///
/// When scrollback decreases, force immediate effect for terminal buffers.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_scrollback(args: *mut c_void) -> CallbackResult {
    let buf = nvim_optset_get_buf(args);
    let old_value = nvim_optset_get_oldval_number(args);
    let new_value = nvim_optset_get_newval_number(args);

    let terminal = nvim_option_buf_get_terminal_ptr(buf);
    if !terminal.is_null() && new_value < old_value {
        on_scrollback_option_changed(terminal);
    }
    callback_ok()
}

/// Callback for 'undolevels' option (full replacement).
///
/// Handles both global and buffer-local undolevels changes.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_undolevels_full(args: *mut c_void) -> CallbackResult {
    let buf = nvim_optset_get_buf(args);
    let varp = nvim_optset_get_varp(args);
    let new_value = nvim_optset_get_newval_number(args);
    let old_value = nvim_optset_get_oldval_number(args);

    let p_ul_addr = nvim_callback_get_p_ul_addr();
    if varp == p_ul_addr {
        // global 'undolevels'
        did_set_global_undolevels(new_value, old_value);
    } else {
        // buffer local 'undolevels'
        did_set_buflocal_undolevels(buf, new_value, old_value);
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

/// Callback for 'buflisted' option.
///
/// When 'buflisted' changes, fire BufAdd or BufDelete autocmd.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_buflisted(args: *mut c_void) -> CallbackResult {
    let buf = nvim_optset_get_buf(args);
    let old_val = nvim_optset_get_oldval_boolean(args);
    let new_val = nvim_buf_get_p_bl(buf);

    if old_val != new_val {
        let event = if new_val != 0 {
            EVENT_BUFADD
        } else {
            EVENT_BUFDELETE
        };
        nvim_apply_autocmds_buf_event(event, buf);
    }
    callback_ok()
}

// State for previewwindow iteration: the window being set, and whether we found a conflict.
static mut PVW_TARGET_WIN: WinHandle = std::ptr::null_mut();
static mut PVW_CONFLICT: bool = false;

/// Per-window callback for 'previewwindow' check.
unsafe extern "C" fn pvw_check_callback(wp: WinHandle, _ud: *mut c_void) {
    let target = PVW_TARGET_WIN;
    if nvim_win_get_p_pvw(wp) != 0 && wp != target {
        // Another window already has pvw set — conflict
        PVW_CONFLICT = true;
    }
}

/// Callback for 'previewwindow' option.
///
/// There can be only one preview window. If another window already has
/// 'previewwindow' set, reset it and return an error.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_previewwindow(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    if nvim_win_get_p_pvw(win) == 0 {
        return callback_ok();
    }

    // Check all windows in current tab for conflicts
    PVW_TARGET_WIN = win;
    PVW_CONFLICT = false;
    nvim_for_all_windows_in_curtab(pvw_check_callback, std::ptr::null_mut());

    if PVW_CONFLICT {
        nvim_win_set_p_pvw(win, 0);
        return nvim_get_e_preview_window_exists();
    }

    callback_ok()
}

/// Callback for 'spell' option.
///
/// When spell is enabled, parse spelllang. Returns error message if parsing fails.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_spell_full(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    if nvim_win_get_p_spell(win) != 0 {
        return nvim_parse_spelllang(win);
    }
    callback_ok()
}

/// Callback for 'shiftwidth' or 'tabstop' option.
///
/// Updates fold if using indent fold method. Reparses cinoptions when
/// shiftwidth changes or shiftwidth is 0 and tabstop changes.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_shiftwidth_tabstop(args: *mut c_void) -> CallbackResult {
    let buf = nvim_optset_get_buf(args);
    let win = nvim_optset_get_win(args);
    let varp = nvim_optset_get_varp(args);
    let sw_addr = nvim_buf_get_b_p_sw_addr(buf);

    if rs_foldmethodIsIndent(win) != 0 {
        rs_foldUpdateAll(win);
    }
    // When 'shiftwidth' changes, or it's zero and 'tabstop' changes: parse 'cinoptions'.
    if varp == sw_addr || nvim_buf_get_b_p_sw(buf) == 0 {
        nvim_parse_cino(buf);
    }
    callback_ok()
}

/// Callback for 'chistory' or 'lhistory' option.
///
/// Resizes the quickfix or location list stack to the new size.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_did_set_xhistory(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    let varp = nvim_optset_get_varp(args);
    let chi_addr = nvim_get_p_chi_addr();

    if varp == chi_addr {
        // 'chistory': resize the global quickfix stack
        let n = *(chi_addr as *const OptInt);
        nvim_qf_resize_stack(n as c_int);
    } else {
        // 'lhistory': resize the location list stack for this window
        let n = *(varp as *const OptInt);
        nvim_ll_resize_stack(win, n as c_int);
    }
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
// Cursorlineopt Flags
// =============================================================================

/// Flag bit for 'cursorlineopt' "line" keyword
const CULOPT_FLAG_LINE: u8 = 0x01;
/// Flag bit for 'cursorlineopt' "screenline" keyword
const CULOPT_FLAG_SCREENLINE: u8 = 0x02;
/// Flag bit for 'cursorlineopt' "number" keyword
const CULOPT_FLAG_NUMBER: u8 = 0x04;

/// OK return value (matches C OK = 1)
const OK: c_int = 1;
/// FAIL return value (matches C FAIL = 0)
const FAIL: c_int = 0;

/// Parse 'cursorlineopt' value into flag bits and store in window.
///
/// If `val` is null, reads the current value from the window.
/// Returns OK on success, FAIL if the value is invalid.
#[no_mangle]
pub unsafe extern "C" fn rs_fill_culopt_flags(
    val: *const std::ffi::c_char,
    wp: WinHandle,
) -> c_int {
    let p: *const u8 = if val.is_null() {
        let culopt = nvim_win_get_p_culopt(wp);
        if culopt.is_null() {
            return FAIL;
        }
        culopt.cast()
    } else {
        val.cast()
    };

    let mut culopt_flags_new: u8 = 0;
    let mut cur = p;

    while *cur != 0 {
        if *cur == b'l'
            && *cur.add(1) == b'i'
            && *cur.add(2) == b'n'
            && *cur.add(3) == b'e'
            && (*cur.add(4) == b',' || *cur.add(4) == 0)
        {
            cur = cur.add(4);
            culopt_flags_new |= CULOPT_FLAG_LINE;
        } else if *cur == b'b'
            && *cur.add(1) == b'o'
            && *cur.add(2) == b't'
            && *cur.add(3) == b'h'
            && (*cur.add(4) == b',' || *cur.add(4) == 0)
        {
            cur = cur.add(4);
            culopt_flags_new |= CULOPT_FLAG_LINE | CULOPT_FLAG_NUMBER;
        } else if *cur == b'n'
            && *cur.add(1) == b'u'
            && *cur.add(2) == b'm'
            && *cur.add(3) == b'b'
            && *cur.add(4) == b'e'
            && *cur.add(5) == b'r'
            && (*cur.add(6) == b',' || *cur.add(6) == 0)
        {
            cur = cur.add(6);
            culopt_flags_new |= CULOPT_FLAG_NUMBER;
        } else if *cur == b's'
            && *cur.add(1) == b'c'
            && *cur.add(2) == b'r'
            && *cur.add(3) == b'e'
            && *cur.add(4) == b'e'
            && *cur.add(5) == b'n'
            && *cur.add(6) == b'l'
            && *cur.add(7) == b'i'
            && *cur.add(8) == b'n'
            && *cur.add(9) == b'e'
            && (*cur.add(10) == b',' || *cur.add(10) == 0)
        {
            cur = cur.add(10);
            culopt_flags_new |= CULOPT_FLAG_SCREENLINE;
        } else {
            return FAIL;
        }

        if *cur != b',' && *cur != 0 {
            return FAIL;
        }
        if *cur == b',' {
            cur = cur.add(1);
        }
    }

    // Can't have both "line" and "screenline".
    if (culopt_flags_new & CULOPT_FLAG_LINE) != 0
        && (culopt_flags_new & CULOPT_FLAG_SCREENLINE) != 0
    {
        return FAIL;
    }

    nvim_win_set_p_culopt_flags(wp, culopt_flags_new);
    OK
}

// =============================================================================
// Binary Option Toggle
// =============================================================================

/// OPT_GLOBAL flag (matches C option.h OPT_GLOBAL = 0x01)
const OPT_GLOBAL_BIN: c_int = 0x01;
/// OPT_LOCAL flag (matches C option.h OPT_LOCAL = 0x02)
const OPT_LOCAL_BIN: c_int = 0x02;

/// Save/restore options when 'binary' ('bin') changes value.
///
/// When `newval` is non-zero (bin turned on):
/// - If previously off, saves current local/global tw, wm, ml, et values
/// - Forces bin-compatible values (no wrap, no modelines, no expandtab)
///
/// When `newval` is zero (bin turned off):
/// - If previously on, restores the saved local/global values
#[no_mangle]
pub unsafe extern "C" fn rs_set_options_bin(oldval: c_int, newval: c_int, opt_flags: c_int) {
    if newval != 0 {
        if oldval == 0 {
            // switched on
            if (opt_flags & OPT_GLOBAL_BIN) == 0 {
                // save local buffer options
                nvim_curbuf_set_b_p_tw_nobin(OptInt::from(nvim_curbuf_get_b_p_tw()));
                nvim_curbuf_set_b_p_wm_nobin(OptInt::from(nvim_curbuf_get_b_p_wm()));
                nvim_curbuf_set_b_p_ml_nobin(nvim_curbuf_get_b_p_ml());
                nvim_curbuf_set_b_p_et_nobin(nvim_curbuf_get_b_p_et());
            }
            if (opt_flags & OPT_LOCAL_BIN) == 0 {
                // save global options
                nvim_set_p_tw_nobin(nvim_get_p_tw());
                nvim_set_p_wm_nobin(nvim_get_p_wm());
                nvim_set_p_ml_nobin(nvim_get_p_ml());
                nvim_set_p_et_nobin(nvim_get_p_et());
            }
        }

        if (opt_flags & OPT_GLOBAL_BIN) == 0 {
            // set bin-compatible local buffer values
            nvim_curbuf_set_b_p_tw(0);
            nvim_curbuf_set_b_p_wm(0);
            nvim_curbuf_set_b_p_ml(0);
            nvim_curbuf_set_b_p_et(0);
        }
        if (opt_flags & OPT_LOCAL_BIN) == 0 {
            // set bin-compatible global values
            nvim_set_p_tw(0);
            nvim_set_p_wm(0);
            nvim_set_p_ml(0);
            nvim_set_p_et(0);
            nvim_set_p_bin(1); // needed when called for the "-b" argument
        }
    } else if oldval != 0 {
        // switched off: restore saved values
        if (opt_flags & OPT_GLOBAL_BIN) == 0 {
            nvim_curbuf_set_b_p_tw(OptInt::from(nvim_curbuf_get_b_p_tw_nobin()));
            nvim_curbuf_set_b_p_wm(OptInt::from(nvim_curbuf_get_b_p_wm_nobin()));
            nvim_curbuf_set_b_p_ml(nvim_curbuf_get_b_p_ml_nobin());
            nvim_curbuf_set_b_p_et(nvim_curbuf_get_b_p_et_nobin());
        }
        if (opt_flags & OPT_LOCAL_BIN) == 0 {
            nvim_set_p_tw(nvim_get_p_tw_nobin());
            nvim_set_p_wm(nvim_get_p_wm_nobin());
            nvim_set_p_ml(nvim_get_p_ml_nobin());
            nvim_set_p_et(nvim_get_p_et_nobin());
        }
    }

    nvim_bin_didset_options_sctx(opt_flags);
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
        // rs_did_set_spell_full now calls parse_spelllang() (extern C, not testable standalone)
        assert!(rs_did_set_termguicolors().is_null());
        assert!(rs_did_set_virtualedit().is_null());
        assert!(rs_did_set_writebackup().is_null());
    }
}
