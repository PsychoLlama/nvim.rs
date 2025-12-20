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
