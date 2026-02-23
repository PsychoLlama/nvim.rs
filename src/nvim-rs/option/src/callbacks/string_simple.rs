//! Phase 1 simple string validation callbacks
//!
//! This module contains Rust implementations of simple `did_set_*` string
//! validation callbacks migrated from `optionstr.c`. These callbacks validate
//! option string values against allowed character sets or fixed value lists.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit

use std::ffi::{c_char, c_int, c_void};

use super::{callback_ok, CallbackResult};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // optset_T field accessors
    fn nvim_optset_get_varp_str(args: *const c_void) -> *const c_char;
    fn nvim_optset_get_errbuf(args: *const c_void) -> *mut c_char;
    fn nvim_optset_get_errbuflen(args: *const c_void) -> usize;
    fn nvim_optset_get_win(args: *const c_void) -> crate::WinHandle;
    fn nvim_optset_get_varp_ptr(args: *const c_void) -> *const c_void;

    // Validation helpers
    fn nvim_illegal_char(errbuf: *mut c_char, errbuflen: usize, c: c_int) -> *const c_char;
    fn nvim_did_set_str_generic(args: *mut c_void) -> *const c_char;

    // Side-effect helpers
    fn nvim_call_init_chartab();
    fn nvim_call_msg_grid_validate();
    fn nvim_call_check_opt_wim() -> c_int;
    fn nvim_call_briopt_check_win(val: *const c_char, win: crate::WinHandle) -> c_int;
    fn nvim_win_get_p_briopt_addr(win: crate::WinHandle) -> *const c_void;
    fn nvim_get_cmdpreview() -> c_int;
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_redraw_curbuf_later(redraw_type: c_int);
    fn nvim_win_get_briopt_list(win: crate::WinHandle) -> c_int;
    fn redraw_all_later(typ: c_int);
}

// =============================================================================
// Constants
// =============================================================================

/// Error: Invalid argument (E474)
const E_INVARG: *const c_char = c"E474: Invalid argument".as_ptr();

/// FAIL return value (matches C FAIL = 0)
const FAIL: c_int = 0;

/// UPD_INVERTED = 20 (from drawscreen.h)
const UPD_INVERTED: c_int = 20;
/// UPD_NOT_VALID = 40 (from drawscreen.h)
const UPD_NOT_VALID: c_int = 40;

// =============================================================================
// Flag character sets for list-flag options
// =============================================================================

/// All valid flags for 'cpoptions' option (vi compatibility)
const CPO_VI: &[u8] = b"aAbBcCdDeEfFiIJKlLmMnoOpPqrRsStuvWxXyZ$!%+>;~_";

/// All valid flags for 'formatoptions' option
const FO_ALL: &[u8] = b"tcro/q2vlb1mMBn,aw]jp";

/// All valid flags for 'mouse' option
const MOUSE_ALL: &[u8] = b"anvichr";

/// All valid flags for 'shortmess' option
const SHM_ALL: &[u8] = b"rwoOstTWIcCqaAFnlxfiS";

/// All valid flags for 'whichwrap' option (comma is also allowed as separator)
const WW_ALL: &[u8] = b"bshl<>[]~,";

// =============================================================================
// Internal helpers
// =============================================================================

/// Validate that a string contains only characters from an allowed set.
/// On failure, formats an E539 "Illegal character" message into errbuf.
/// Returns NULL on success, or a pointer to an error message on failure.
#[inline]
unsafe fn validate_listflag_with_errbuf(
    val: *const c_char,
    allowed: &[u8],
    errbuf: *mut c_char,
    errbuflen: usize,
) -> CallbackResult {
    if val.is_null() {
        return callback_ok();
    }
    let mut p = val;
    while *p != 0 {
        let ch = *p as u8;
        if !allowed.contains(&ch) {
            return nvim_illegal_char(errbuf, errbuflen, c_int::from(ch));
        }
        p = p.add(1);
    }
    callback_ok()
}

// =============================================================================
// List-flag option callbacks
// =============================================================================

/// Callback for 'cpoptions' option.
/// Validates that all characters are valid vi compatibility flags.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_cpoptions(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    validate_listflag_with_errbuf(val, CPO_VI, errbuf, errbuflen)
}

/// Callback for 'formatoptions' option.
/// Validates that all characters are valid format option flags.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_formatoptions(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    validate_listflag_with_errbuf(val, FO_ALL, errbuf, errbuflen)
}

/// Callback for 'mouse' option.
/// Validates that all characters are valid mouse mode flags.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_mouse(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    validate_listflag_with_errbuf(val, MOUSE_ALL, errbuf, errbuflen)
}

/// Callback for 'shortmess' option.
/// Validates that all characters are valid shortmess flags.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_shortmess(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    validate_listflag_with_errbuf(val, SHM_ALL, errbuf, errbuflen)
}

/// Callback for 'whichwrap' option.
/// Validates characters in "bshl<>[]~" (comma-separated list).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_whichwrap(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);
    validate_listflag_with_errbuf(val, WW_ALL, errbuf, errbuflen)
}

// =============================================================================
// String-generic option callbacks (delegate to did_set_str_generic)
// =============================================================================

/// Callback for 'backspace' option.
/// Numeric form only allows "2". Otherwise validates against the allowed values list.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_backspace(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    if val.is_null() {
        return callback_ok();
    }
    let first = *val as u8;
    if first.is_ascii_digit() {
        if first != b'2' {
            return E_INVARG;
        }
        return callback_ok();
    }
    nvim_did_set_str_generic(args)
}

/// Callback for 'bufhidden' option.
/// Validates against "hide,unload,delete,wipe" and empty string.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_bufhidden(args: *mut c_void) -> CallbackResult {
    nvim_did_set_str_generic(args)
}

/// Callback for 'inccommand' option.
/// Disallows change while cmdpreview is active, then validates against allowed values.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_inccommand(args: *mut c_void) -> CallbackResult {
    if nvim_get_cmdpreview() != 0 {
        return E_INVARG;
    }
    nvim_did_set_str_generic(args)
}

/// Callback for 'lispoptions' option.
/// Valid values: empty, "expr:0", or "expr:1".
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_lispoptions(args: *mut c_void) -> CallbackResult {
    let val = nvim_optset_get_varp_str(args);
    if val.is_null() || *val == 0 {
        return callback_ok();
    }
    // Must be exactly "expr:0" or "expr:1" (6 bytes + null = 7 bytes total)
    let bytes = std::slice::from_raw_parts(val.cast::<u8>(), 7);
    let is_valid = bytes[0] == b'e'
        && bytes[1] == b'x'
        && bytes[2] == b'p'
        && bytes[3] == b'r'
        && bytes[4] == b':'
        && (bytes[5] == b'0' || bytes[5] == b'1')
        && bytes[6] == 0;
    if is_valid {
        callback_ok()
    } else {
        E_INVARG
    }
}

// =============================================================================
// Callbacks with side effects
// =============================================================================

/// Callback for 'wildmode' option.
/// Validates via check_opt_wim().
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_wildmode(args: *mut c_void) -> CallbackResult {
    let _ = args;
    if nvim_call_check_opt_wim() == FAIL {
        return E_INVARG;
    }
    callback_ok()
}

/// Callback for 'breakindentopt' option.
/// Validates via briopt_check; applies to window if window-local, triggers redraw if list.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_breakindentopt(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    let varp = nvim_optset_get_varp_ptr(args);
    let briopt_addr = nvim_win_get_p_briopt_addr(win);
    // Pass win only when varp IS the window-local option (mirrors C behavior)
    let win_for_check = if varp == briopt_addr {
        win
    } else {
        std::ptr::null_mut()
    };
    let val = nvim_optset_get_varp_str(args);
    if nvim_call_briopt_check_win(val, win_for_check) == 0 {
        return E_INVARG;
    }
    // List setting requires a redraw when applied to current window
    if varp == briopt_addr && nvim_win_get_briopt_list(win) != 0 {
        redraw_all_later(UPD_NOT_VALID);
    }
    callback_ok()
}

/// Callback for 'display' option.
/// Validates against allowed values, then reinitializes chartab and msg grid.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_display(args: *mut c_void) -> CallbackResult {
    let errmsg = nvim_did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    nvim_call_init_chartab();
    nvim_call_msg_grid_validate();
    callback_ok()
}

/// Callback for 'showcmdloc' option.
/// Validates against allowed values ("last", "statusline", "tabline"),
/// then recomputes column positions.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_showcmdloc(args: *mut c_void) -> CallbackResult {
    let errmsg = nvim_did_set_str_generic(args);
    if errmsg.is_null() {
        // nvim_comp_col is already declared in window_shim.c
        nvim_comp_col();
    }
    errmsg
}

/// Callback for 'selection' option.
/// Validates against allowed values ("old", "inclusive", "exclusive"),
/// then triggers a redraw if Visual mode is active.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_selection(args: *mut c_void) -> CallbackResult {
    let errmsg = nvim_did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }
    if nvim_get_VIsual_active() != 0 {
        nvim_redraw_curbuf_later(UPD_INVERTED);
    }
    callback_ok()
}

// =============================================================================
// Helper: nvim_comp_col (declared in window_shim.c)
// =============================================================================

extern "C" {
    fn nvim_comp_col();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ww_all_contains_comma() {
        // WW_ALL must contain comma since 'whichwrap' is comma-separated
        assert!(WW_ALL.contains(&b','));
    }
}
