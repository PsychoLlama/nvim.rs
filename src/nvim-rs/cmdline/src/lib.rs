//! Command line utilities for Neovim
//!
//! Provides Rust implementations of command line functions.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

pub mod api;
pub mod cmdwin;
pub mod completion;
pub mod constants;
pub mod context;
pub mod edit;
pub mod entry;
pub mod expand;
pub mod expr;
pub mod fname;
pub mod history;
pub mod keys;
pub mod pattern;
pub mod preview;
pub mod screen;
pub mod search;
pub mod sources;
pub mod state;
pub mod ui;
pub mod usercomplete;
pub mod viewstate;
pub mod wildmenu;

use std::os::raw::c_int;

extern "C" {
    fn nvim_get_ccline_overstrike() -> c_int;
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_get_cmdwin_type() -> c_int;
    fn nvim_get_cmdline_type() -> c_int;
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
    use std::os::raw::c_uint;

    // Flag value from option_vars.generated.h
    const K_OPT_WOP_FLAG_FUZZY: c_uint = 0x01;

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

    #[test]
    fn test_fuzzy_flag_bit_position() {
        // K_OPT_WOP_FLAG_FUZZY should be bit 0 (1 << 0)
        assert_eq!(K_OPT_WOP_FLAG_FUZZY, 1 << 0);
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_nul_is_string_terminator() {
        // NUL should work as C string terminator (NUL is 0, so fits in u8)
        let nul_char = NUL as u8;
        assert_eq!(nul_char, 0);
    }
}
