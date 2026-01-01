//! Command line utilities for Neovim
//!
//! Provides Rust implementations of command line functions.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::c_char;
use std::os::raw::c_int;
use std::os::raw::c_uint;

// Flag value from option_vars.generated.h
const K_OPT_WOP_FLAG_FUZZY: c_uint = 0x01;

extern "C" {
    fn nvim_get_ccline_overstrike() -> c_int;
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_get_wop_flags() -> c_uint;
    fn nvim_get_cmdwin_type() -> c_int;
    fn nvim_get_cmdline_type() -> c_int;
    fn nvim_get_compl_match_array_not_null() -> c_int;
    fn rs_pum_visible() -> c_int;
    fn nvim_get_cmdpreview_ns() -> c_int;
    fn nvim_get_ccline_cmdfirstc() -> c_int;
}

/// Check if command line is in overstrike mode.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdline_overstrike() -> c_int {
    nvim_get_ccline_overstrike()
}

/// Check if cursor is at the end of the command line.
///
/// Returns true if cmdpos >= cmdlen.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdline_at_end() -> c_int {
    c_int::from(nvim_get_ccline_cmdpos() >= nvim_get_ccline_cmdlen())
}

/// Check if fuzzy completion is enabled and the search string is non-empty.
///
/// Returns true if the 'wildoptions' contains "fuzzy" and the string is
/// not empty.
///
/// # Safety
///
/// `fuzzystr` must be a valid NUL-terminated C string.
/// Calls external C function to access the wop_flags global.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdline_fuzzy_complete(fuzzystr: *const c_char) -> c_int {
    if fuzzystr.is_null() {
        return 0;
    }

    let flags = nvim_get_wop_flags();
    let has_fuzzy_flag = (flags & K_OPT_WOP_FLAG_FUZZY) != 0;
    let is_non_empty = *fuzzystr != 0;

    c_int::from(has_fuzzy_flag && is_non_empty)
}

/// NUL character constant
const NUL: c_int = 0;

/// Check if in the cmdwin, not editing the command line.
///
/// Returns true if `cmdwin_type` != 0 AND `get_cmdline_type()` == NUL.
///
/// # Safety
///
/// Calls external C functions to access global state.
#[no_mangle]
pub unsafe extern "C" fn rs_is_in_cmdwin() -> c_int {
    let cmdwin_type = nvim_get_cmdwin_type();
    let cmdline_type = nvim_get_cmdline_type();

    c_int::from(cmdwin_type != 0 && cmdline_type == NUL)
}

/// Check if the cmdline completion popup menu is being displayed.
///
/// Returns true if `pum_visible()` and `compl_match_array != NULL`.
///
/// # Safety
/// Calls external C functions to access global state.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdline_pum_active() -> c_int {
    c_int::from(rs_pum_visible() != 0 && nvim_get_compl_match_array_not_null() != 0)
}

/// Get the command preview namespace.
///
/// Returns the `cmdpreview_ns` static variable.
///
/// # Safety
/// Calls external C function to access static variable.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdpreview_get_ns() -> c_int {
    nvim_get_cmdpreview_ns()
}

/// Get the first character of the current command line.
///
/// Returns `ccline.cmdfirstc`.
///
/// # Safety
/// Calls external C function to access struct field.
#[no_mangle]
pub unsafe extern "C" fn rs_get_cmdline_firstc() -> c_int {
    nvim_get_ccline_cmdfirstc()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildoption_flag() {
        // K_OPT_WOP_FLAG_FUZZY should be 0x01
        assert_eq!(K_OPT_WOP_FLAG_FUZZY, 0x01);
    }

    #[test]
    fn test_nul_constant() {
        // NUL should be 0
        assert_eq!(NUL, 0);
    }

    #[test]
    fn test_wildoption_flag_is_power_of_two() {
        // Flag should be a single bit (power of 2)
        let flag = K_OPT_WOP_FLAG_FUZZY;
        assert_ne!(flag, 0);
        assert_eq!(flag & (flag - 1), 0, "Flag should be a power of 2");
    }

    #[test]
    fn test_nul_matches_ascii() {
        // NUL constant should match ASCII NUL character
        assert_eq!(NUL, 0);
        assert_eq!(NUL, c_int::from(b'\0'));
    }

}
