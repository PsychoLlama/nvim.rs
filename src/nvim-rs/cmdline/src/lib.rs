//! Command line utilities for Neovim
//!
//! Provides Rust implementations of command line functions.

#![allow(unsafe_code)]

use std::os::raw::c_int;

extern "C" {
    fn nvim_get_ccline_overstrike() -> c_int;
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
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
