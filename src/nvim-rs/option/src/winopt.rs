//! Window option copy/clear/check operations
//!
//! This module provides Rust implementations of the winopt_T manipulation
//! functions: copy_winopt, clear_winopt, check_winopt, didset_window_options,
//! and check_win_options.

use std::ffi::{c_char, c_int, c_void};

// Opaque handle types
type WinoptHandle = *mut c_void;
type WinHandle = *mut c_void;

/// Number of string fields in winopt_T (indices 0..22 in nvim_winopt_string_field_ptr).
/// Must stay in sync with the switch in option_shim.c:nvim_winopt_string_field_ptr.
const WINOPT_STRING_FIELD_COUNT: c_int = 23;

extern "C" {
    // String field accessor: returns &mut *char for field at index [0..22]
    fn nvim_winopt_string_field_ptr(wop: WinoptHandle, idx: c_int) -> *mut *mut c_char;

    // Scalar copy: copies all bool/int/flags fields from->to
    fn nvim_copy_winopt_scalars(from: WinoptHandle, to: WinoptHandle);
    // Conditional xstrdup for wo_fdc_save and wo_fdm_save based on wo_diff_saved
    fn nvim_copy_winopt_save_strs(from: WinoptHandle, to: WinoptHandle);
    // memmove of wo_script_ctx
    fn nvim_copy_winopt_script_ctx(from: WinoptHandle, to: WinoptHandle);

    fn nvim_get_empty_string_option() -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // Per-field string manipulation
    fn clear_string_option(ptr: *mut *mut c_char);
    fn check_string_option(ptr: *mut *mut c_char);

    // didset_window_options sub-calls
    // nvim_win_get_p_wrap, nvim_win_set_leftcol, nvim_win_set_skipcol defined in window_shim.c
    fn nvim_win_get_p_wrap(wp: WinHandle) -> c_int;
    fn nvim_win_set_leftcol(wp: WinHandle, v: c_int);
    fn nvim_win_set_skipcol(wp: WinHandle, v: c_int);
    /// check_colorcolumn(cc, wp) -- pass NULL for cc to use current w_p_cc
    fn check_colorcolumn(cc: *const c_char, wp: WinHandle) -> *const c_char;
    /// briopt_check(briopt, wp) -- pass NULL for briopt to use current w_p_briopt
    fn briopt_check(briopt: *const c_char, wp: WinHandle) -> bool;
    /// fill_culopt_flags(val, wp) -- pass NULL for val to use current w_p_culopt
    fn fill_culopt_flags(val: *mut c_char, wp: WinHandle) -> c_int;
    fn nvim_win_get_opt_field_addr(win: WinHandle, idx: c_int) -> *mut c_void;
    fn set_chars_option(
        wp: WinHandle,
        value: *const c_char,
        what: c_int,
        apply: bool,
        errbuf: *mut c_char,
        errbuflen: usize,
    ) -> *const c_char;
    fn check_blending(wp: WinHandle);
    /// set_winbar_win(wp, make_room, valid_cursor)
    fn set_winbar_win(wp: WinHandle, make_room: bool, valid_cursor: bool) -> c_int;
    /// check_signcolumn(scl, wp) -- pass NULL for scl to use current w_p_scl
    fn check_signcolumn(scl: *const c_char, wp: WinHandle) -> c_int;
    // nvim_win_update_grid_blending defined in option_shim.c (uses w_p_winbl > 0 logic)
    fn nvim_win_update_grid_blending(wp: WinHandle);
}

/// Copy all window options from `from` to `to`.
///
/// Does not free old values in `to` - use `rs_clear_winopt` first.
/// The 'scroll' option is not copied (depends on window height).
///
/// # Safety
/// Both `from` and `to` must be valid non-null winopt_T pointers.
#[export_name = "copy_winopt"]
pub unsafe extern "C" fn rs_copy_winopt(from: WinoptHandle, to: WinoptHandle) {
    // Copy all scalar fields
    nvim_copy_winopt_scalars(from, to);

    // Copy all string fields via copy_option_val (which handles empty_string_option).
    // Skip indices 1 (wo_fdc_save) and 4 (wo_fdm_save) -- handled specially below.
    let n = WINOPT_STRING_FIELD_COUNT;
    for i in 0..n {
        if i == 1 || i == 4 {
            continue; // handled by nvim_copy_winopt_save_strs
        }
        let from_field = nvim_winopt_string_field_ptr(from, i);
        let to_field = nvim_winopt_string_field_ptr(to, i);
        if from_field.is_null() || to_field.is_null() {
            continue;
        }
        *to_field = if *from_field == nvim_get_empty_string_option() {
            nvim_get_empty_string_option()
        } else {
            xstrdup(*from_field)
        };
    }

    // Handle wo_fdc_save and wo_fdm_save with conditional xstrdup (diff-mode save fields)
    nvim_copy_winopt_save_strs(from, to);

    // Copy script context array
    nvim_copy_winopt_script_ctx(from, to);

    // Ensure no NULL pointers remain
    rs_check_winopt(to);
}

/// Free all allocated string fields in a winopt_T.
///
/// # Safety
/// `wop` must be a valid non-null winopt_T pointer.
#[export_name = "clear_winopt"]
pub unsafe extern "C" fn rs_clear_winopt(wop: WinoptHandle) {
    let n = WINOPT_STRING_FIELD_COUNT;
    for i in 0..n {
        let field = nvim_winopt_string_field_ptr(wop, i);
        if !field.is_null() {
            clear_string_option(field);
        }
    }
}

/// Replace NULL string pointers in a winopt_T with `empty_string_option`.
///
/// # Safety
/// `wop` must be a valid non-null winopt_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_check_winopt(wop: WinoptHandle) {
    let n = WINOPT_STRING_FIELD_COUNT;
    for i in 0..n {
        let field = nvim_winopt_string_field_ptr(wop, i);
        if !field.is_null() {
            check_string_option(field);
        }
    }
}

/// Recalculate derived window state after option changes.
///
/// # Safety
/// `wp` must be a valid non-null win_T pointer.
#[export_name = "didset_window_options"]
pub unsafe extern "C" fn rs_didset_window_options(wp: WinHandle, valid_cursor: bool) {
    // Set w_leftcol or w_skipcol to zero based on 'wrap'
    if nvim_win_get_p_wrap(wp) != 0 {
        nvim_win_set_leftcol(wp, 0);
    } else {
        nvim_win_set_skipcol(wp, 0);
    }
    check_colorcolumn(std::ptr::null(), wp);
    briopt_check(std::ptr::null(), wp);
    fill_culopt_flags(std::ptr::null_mut(), wp);
    // kFillchars=0, kListchars=1 (CharsOption enum from optionstr.h)
    let fcs = *(nvim_win_get_opt_field_addr(wp, crate::opt_index::K_OPT_FILLCHARS)
        as *const *const c_char);
    let lcs = *(nvim_win_get_opt_field_addr(wp, crate::opt_index::K_OPT_LISTCHARS)
        as *const *const c_char);
    set_chars_option(wp, fcs, 0, true, std::ptr::null_mut(), 0);
    set_chars_option(wp, lcs, 1, true, std::ptr::null_mut(), 0);
    crate::callbacks::winhl::rs_parse_winhl_opt(std::ptr::null(), wp); // sets w_hl_needs_update also for w_p_winbl
    check_blending(wp);
    set_winbar_win(wp, false, valid_cursor);
    check_signcolumn(std::ptr::null(), wp);
    nvim_win_update_grid_blending(wp);
}
