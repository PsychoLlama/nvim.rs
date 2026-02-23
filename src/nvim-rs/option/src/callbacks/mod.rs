//! Option callback implementations
//!
//! This module contains Rust implementations of option change callbacks
//! (`did_set_*` functions). These are called after an option value changes
//! to perform side effects like redrawing the screen, updating state, etc.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit

pub mod behavior;
pub mod complex;
pub mod display;
pub mod numeric;
pub mod string;
pub mod string_simple;
pub mod winhl;

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::OptInt;

// =============================================================================
// Redraw Types
// =============================================================================

/// Update types for `redraw_all_later()`.
/// These correspond to values in `drawscreen.h`.
/// Values verified against src/nvim/drawscreen.h — add _Static_assert in C if changed.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateType {
    /// UPD_VALID: buffer not changed, or changes marked with b_mod_*
    Valid = 10,
    /// UPD_INVERTED: redisplay inverted part that changed
    Inverted = 20,
    /// UPD_INVERTED_ALL: redisplay whole inverted part
    InvertedAll = 25,
    /// UPD_REDRAW_TOP: display first w_upd_rows screen lines
    RedrawTop = 30,
    /// UPD_SOME_VALID: like UPD_NOT_VALID but may scroll
    SomeValid = 35,
    /// UPD_NOT_VALID: buffer needs complete redraw
    NotValid = 40,
    /// UPD_CLEAR: screen messed up, clear it
    Clear = 50,
}

// =============================================================================
// C Function Declarations (External functions called by callbacks)
// =============================================================================

#[allow(dead_code)] // FFI functions used when linked with C
extern "C" {
    // Screen/redraw functions
    fn redraw_all_later(typ: c_int);
    fn showmode();
    fn status_redraw_all();
    fn status_redraw_curbuf();
    fn maketitle();

    // Search highlight functions
    fn set_no_hlsearch(flag: c_int);

    // State accessors
    fn nvim_callback_get_starting() -> c_int;
    fn nvim_option_get_hls() -> c_int;
    fn nvim_callback_get_p_titlelen() -> OptInt;
    fn nvim_callback_get_no_hlsearch() -> c_int;

    // State setters
    fn nvim_callback_set_need_maketitle(value: c_int);

    // Fold functions
    fn rs_newFoldLevel();

    // Langnoremap/langremap accessors
    fn nvim_callback_get_p_lnr() -> c_int;
    fn nvim_callback_get_p_lrm() -> c_int;
    fn nvim_callback_set_p_lnr(value: c_int);
    fn nvim_callback_set_p_lrm(value: c_int);

    // Pumblend accessors
    fn hl_invalidate_blends();
    fn nvim_callback_get_p_pb() -> OptInt;
    fn nvim_callback_set_pum_grid_blending(value: c_int);
    fn pum_drawn() -> c_int;
    fn pum_redraw();

    // Textwidth helper
    fn check_colorcolumn_win(win: crate::WinHandle);
    fn nvim_callback_for_all_tab_windows(callback: unsafe extern "C" fn(crate::WinHandle));

    // Winblend accessors
    fn nvim_callback_win_clamp_winbl(win: crate::WinHandle);
    fn nvim_callback_win_set_hl_needs_update(win: crate::WinHandle, value: c_int);
    fn check_blending(win: crate::WinHandle);

    // optset_T field accessors
    fn nvim_optset_get_win(args: *const c_void) -> crate::WinHandle;
    fn nvim_optset_get_oldval_number(args: *const c_void) -> i64;
    fn nvim_optset_get_newval_number(args: *const c_void) -> i64;
    fn nvim_optset_get_varp(args: *const c_void) -> *mut c_void;
}

// =============================================================================
// Constants
// =============================================================================

/// `starting` values from globals.h
const NO_SCREEN: c_int = 2;

// =============================================================================
// Callback Result Type
// =============================================================================

/// Result type for option callbacks.
/// Returns NULL on success, or a pointer to an error message on failure.
pub type CallbackResult = *const c_char;

/// Indicates successful callback execution (no error).
#[inline]
pub const fn callback_ok() -> CallbackResult {
    ptr::null()
}

// =============================================================================
// Callback Helper Functions
// =============================================================================

/// Request a redraw of all windows.
#[inline]
pub fn request_redraw_all(typ: UpdateType) {
    unsafe { redraw_all_later(typ as c_int) }
}

/// Request title update.
#[inline]
fn request_maketitle() {
    unsafe { nvim_callback_set_need_maketitle(1) }
}

/// Check if screen is available for drawing.
#[inline]
fn screen_available() -> bool {
    unsafe { nvim_callback_get_starting() != NO_SCREEN }
}

/// Get 'hlsearch' option value.
#[inline]
fn get_hlsearch() -> bool {
    unsafe { nvim_option_get_hls() != 0 }
}

/// Get 'titlelen' option value.
#[inline]
fn get_titlelen() -> OptInt {
    unsafe { nvim_callback_get_p_titlelen() }
}

// =============================================================================
// Simple Boolean Option Callbacks
// =============================================================================

/// Callback for 'hlsearch' option.
/// When 'hlsearch' is set or reset, reset no_hlsearch flag.
#[no_mangle]
pub extern "C" fn rs_did_set_hlsearch(_args: *mut c_void) -> CallbackResult {
    unsafe { set_no_hlsearch(0) };
    callback_ok()
}

/// Callback for 'ignorecase' option.
/// When 'ignorecase' is set/reset and 'hlsearch' is set, redraw.
#[no_mangle]
pub extern "C" fn rs_did_set_ignorecase(_args: *mut c_void) -> CallbackResult {
    if get_hlsearch() {
        request_redraw_all(UpdateType::SomeValid);
    }
    callback_ok()
}

/// Callback for 'title' and 'icon' options.
/// When 'title' or 'icon' changes, may need to update the title.
#[no_mangle]
pub extern "C" fn rs_did_set_title_icon(_args: *mut c_void) -> CallbackResult {
    rs_did_set_title();
    callback_ok()
}

/// Internal helper for title-related callbacks.
/// When changing 'title', 'titlestring', 'icon' or 'iconstring', call
/// maketitle() to create and display it.
#[no_mangle]
pub extern "C" fn rs_did_set_title() {
    if screen_available() {
        unsafe { maketitle() };
    }
}

/// Callback for 'titlelen' option.
/// If 'titlelen' changed, redraw the title.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_titlelen(args: *mut c_void) -> CallbackResult {
    let old_value = nvim_optset_get_oldval_number(args);
    if screen_available() && old_value != get_titlelen() {
        request_maketitle();
    }
    callback_ok()
}

/// Callback for 'laststatus' option.
/// Update status lines based on new laststatus value.
#[no_mangle]
pub extern "C" fn rs_did_set_laststatus() -> CallbackResult {
    // Status line redraw is handled by last_status() in C
    // This is called after last_status() runs
    callback_ok()
}

/// Callback for 'showtabline' option.
/// Update tabline display based on new value.
#[no_mangle]
pub extern "C" fn rs_did_set_showtabline() -> CallbackResult {
    // shell_new_rows() is called in C to handle this
    callback_ok()
}

/// Callback for 'iminsert' option.
/// Show/unshow value of 'keymap' in status lines.
#[no_mangle]
pub extern "C" fn rs_did_set_iminsert(_args: *mut c_void) -> CallbackResult {
    unsafe {
        showmode();
        status_redraw_curbuf();
    }
    callback_ok()
}

/// Callback for 'langnoremap' option.
/// 'langnoremap' -> !'langremap': toggle the paired option.
#[no_mangle]
pub extern "C" fn rs_did_set_langnoremap(_args: *mut c_void) -> CallbackResult {
    // p_lrm = !p_lnr
    let lnr = unsafe { nvim_callback_get_p_lnr() };
    unsafe { nvim_callback_set_p_lrm(c_int::from(lnr == 0)) };
    callback_ok()
}

/// Callback for 'langremap' option.
/// 'langremap' -> !'langnoremap': toggle the paired option.
#[no_mangle]
pub extern "C" fn rs_did_set_langremap(_args: *mut c_void) -> CallbackResult {
    // p_lnr = !p_lrm
    let lrm = unsafe { nvim_callback_get_p_lrm() };
    unsafe { nvim_callback_set_p_lnr(c_int::from(lrm == 0)) };
    callback_ok()
}

/// Callback for 'foldlevel' option.
/// Recalculate fold levels when 'foldlevel' changes.
#[no_mangle]
pub extern "C" fn rs_did_set_foldlevel(_args: *mut c_void) -> CallbackResult {
    unsafe { rs_newFoldLevel() };
    callback_ok()
}

/// Callback for 'smoothscroll' option.
/// Reset the displayed portion when smoothscroll changes.
#[no_mangle]
pub extern "C" fn rs_did_set_smoothscroll() -> CallbackResult {
    // w_skipcol is reset in C
    request_redraw_all(UpdateType::NotValid);
    callback_ok()
}

/// Callback for check_colorcolumn on a single window (used as fn pointer).
unsafe extern "C" fn check_colorcolumn_for_win(win: crate::WinHandle) {
    check_colorcolumn_win(win);
}

/// Callback for 'textwidth' option.
/// Check colorcolumn for all windows when textwidth changes.
#[no_mangle]
pub extern "C" fn rs_did_set_textwidth(_args: *mut c_void) -> CallbackResult {
    unsafe { nvim_callback_for_all_tab_windows(check_colorcolumn_for_win) };
    callback_ok()
}

/// Callback for 'pumblend' option.
/// Update popup menu transparency: invalidate blends, update pum_grid.blending,
/// and redraw popup menu if visible.
#[no_mangle]
pub extern "C" fn rs_did_set_pumblend(_args: *mut c_void) -> CallbackResult {
    unsafe {
        hl_invalidate_blends();
        let pb = nvim_callback_get_p_pb();
        nvim_callback_set_pum_grid_blending(c_int::from(pb > 0));
        if pum_drawn() != 0 {
            pum_redraw();
        }
    }
    callback_ok()
}

/// Callback for 'winblend' option.
/// Clamp value to [0, 100], update highlight blending if changed.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_winblend(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    let old_value = nvim_optset_get_oldval_number(args);
    let new_value = nvim_optset_get_newval_number(args);
    if new_value != old_value {
        nvim_callback_win_clamp_winbl(win);
        nvim_callback_win_set_hl_needs_update(win, 1);
        check_blending(win);
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
    fn test_callback_ok() {
        let result = callback_ok();
        assert!(result.is_null());
    }

    #[test]
    fn test_update_type_values() {
        // Values must match src/nvim/drawscreen.h
        assert_eq!(UpdateType::Valid as c_int, 10);
        assert_eq!(UpdateType::Inverted as c_int, 20);
        assert_eq!(UpdateType::InvertedAll as c_int, 25);
        assert_eq!(UpdateType::RedrawTop as c_int, 30);
        assert_eq!(UpdateType::SomeValid as c_int, 35);
        assert_eq!(UpdateType::NotValid as c_int, 40);
        assert_eq!(UpdateType::Clear as c_int, 50);
    }
}
