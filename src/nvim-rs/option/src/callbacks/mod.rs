//! Option callback implementations
//!
//! This module contains Rust implementations of option change callbacks
//! (`did_set_*` functions). These are called after an option value changes
//! to perform side effects like redrawing the screen, updating state, etc.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit

pub mod complex;
pub mod numeric;
pub mod string;

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::OptInt;

// =============================================================================
// Redraw Types
// =============================================================================

/// Update types for `redraw_all_later()`.
/// These correspond to values in `drawscreen.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateType {
    /// No update needed
    Valid = 10,
    /// Some lines need updating
    SomeValid = 20,
    /// Redraw inverted part of current line
    RedrawThis = 25,
    /// Buffer contents need updating
    NotValid = 30,
    /// Current buffer needs update + clear first
    NotValidVirt = 35,
    /// Clear screen and redraw
    Clear = 40,
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
    fn nvim_callback_get_p_hls() -> c_int;
    fn nvim_callback_get_p_titlelen() -> OptInt;
    fn nvim_callback_get_no_hlsearch() -> c_int;

    // State setters
    fn nvim_callback_set_need_maketitle(value: c_int);
    fn nvim_callback_set_redraw_tabline(value: c_int);
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
fn request_redraw_all(typ: UpdateType) {
    unsafe { redraw_all_later(typ as c_int) }
}

/// Request title update.
#[inline]
fn request_maketitle() {
    unsafe { nvim_callback_set_need_maketitle(1) }
}

/// Request tabline redraw.
#[inline]
fn request_redraw_tabline() {
    unsafe { nvim_callback_set_redraw_tabline(1) }
}

/// Check if screen is available for drawing.
#[inline]
fn screen_available() -> bool {
    unsafe { nvim_callback_get_starting() != NO_SCREEN }
}

/// Get 'hlsearch' option value.
#[inline]
fn get_hlsearch() -> bool {
    unsafe { nvim_callback_get_p_hls() != 0 }
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
pub extern "C" fn rs_did_set_hlsearch() -> CallbackResult {
    unsafe { set_no_hlsearch(0) };
    callback_ok()
}

/// Callback for 'ignorecase' option.
/// When 'ignorecase' is set/reset and 'hlsearch' is set, redraw.
#[no_mangle]
pub extern "C" fn rs_did_set_ignorecase() -> CallbackResult {
    if get_hlsearch() {
        request_redraw_all(UpdateType::SomeValid);
    }
    callback_ok()
}

/// Callback for 'title' and 'icon' options.
/// When 'title' or 'icon' changes, may need to update the title.
#[no_mangle]
pub extern "C" fn rs_did_set_title_icon() -> CallbackResult {
    rs_did_set_title();
    callback_ok()
}

/// Internal helper for title-related callbacks.
#[no_mangle]
pub extern "C" fn rs_did_set_title() {
    request_maketitle();
    request_redraw_tabline();
}

/// Callback for 'titlelen' option.
/// If 'titlelen' changed, redraw the title.
#[no_mangle]
pub extern "C" fn rs_did_set_titlelen(old_value: OptInt) -> CallbackResult {
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
pub extern "C" fn rs_did_set_iminsert() -> CallbackResult {
    unsafe {
        showmode();
        status_redraw_curbuf();
    }
    callback_ok()
}

/// Callback for 'langnoremap' option.
/// Reset langmap when 'langnoremap' is set.
#[no_mangle]
pub extern "C" fn rs_did_set_langnoremap(new_value: c_int) -> CallbackResult {
    // When langnoremap is set, langremap should be false and vice versa
    // This logic is handled in C by toggling p_lrm
    let _ = new_value;
    callback_ok()
}

/// Callback for 'langremap' option.
/// Reset langmap when 'langremap' changes.
#[no_mangle]
pub extern "C" fn rs_did_set_langremap(new_value: c_int) -> CallbackResult {
    // When langremap is set, langnoremap should be false and vice versa
    // This logic is handled in C by toggling p_lnr
    let _ = new_value;
    callback_ok()
}

/// Callback for 'paste' option.
/// When 'paste' is set, various options need adjustment.
#[no_mangle]
pub extern "C" fn rs_did_set_paste() -> CallbackResult {
    // The paste option handling is complex and modifies many options.
    // The actual implementation is done in C (paste_option_changed)
    // This callback just signals that it happened.
    callback_ok()
}

/// Callback for 'foldlevel' option.
#[no_mangle]
pub extern "C" fn rs_did_set_foldlevel() -> CallbackResult {
    // newFoldLevel() is called in C to recalculate folds
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

/// Callback for 'textwidth' option.
/// May need to reformat text after textwidth changes.
#[no_mangle]
pub extern "C" fn rs_did_set_textwidth() -> CallbackResult {
    // curbuf->b_p_tw_nopstrte is set in C
    callback_ok()
}

/// Callback for 'pumblend' option.
/// Update popup menu transparency.
#[no_mangle]
pub extern "C" fn rs_did_set_pumblend() -> CallbackResult {
    // hl_invalidate_blends() is called in C
    // pum_recompose() is called if pum_drawn()
    callback_ok()
}

/// Callback for 'winblend' option.
/// Update window transparency.
#[no_mangle]
pub extern "C" fn rs_did_set_winblend() -> CallbackResult {
    // hl_invalidate_blends() is called in C
    // Similar to pumblend, triggers recomposition
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
        assert_eq!(UpdateType::Valid as c_int, 10);
        assert_eq!(UpdateType::SomeValid as c_int, 20);
        assert_eq!(UpdateType::RedrawThis as c_int, 25);
        assert_eq!(UpdateType::NotValid as c_int, 30);
        assert_eq!(UpdateType::NotValidVirt as c_int, 35);
        assert_eq!(UpdateType::Clear as c_int, 40);
    }
}
