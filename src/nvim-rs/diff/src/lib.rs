//! Diff option checking for Neovim
//!
//! This module provides Rust implementations for checking diff options
//! from the 'diffopt' setting.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]

use std::os::raw::c_int;

// Diff flags (from diff.c)
// These must match the C #define values exactly
const DIFF_FILLER: c_int = 0x001;
const DIFF_HORIZONTAL: c_int = 0x040;
const DIFF_HIDDEN_OFF: c_int = 0x100;
const DIFF_INTERNAL: c_int = 0x200;
const DIFF_CLOSE_OFF: c_int = 0x400;

// C accessor for the static diff_flags variable
extern "C" {
    fn nvim_get_diff_flags() -> c_int;
    fn nvim_is_diffexpr_empty() -> bool;
}

/// Check if 'diffopt' contains "horizontal".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_horizontal() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_HORIZONTAL) != 0)
}

/// Check if 'diffopt' contains "hiddenoff".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_hiddenoff() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_HIDDEN_OFF) != 0)
}

/// Check if 'diffopt' contains "closeoff".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_closeoff() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_CLOSE_OFF) != 0)
}

/// Check if 'diffopt' contains "filler".
#[no_mangle]
pub unsafe extern "C" fn rs_diffopt_filler() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_FILLER) != 0)
}

/// Return true if the options are set to use the internal diff library.
///
/// Note that if the internal diff failed for one of the buffers, the external
/// diff will be used anyway.
#[no_mangle]
pub unsafe extern "C" fn rs_diff_internal() -> c_int {
    c_int::from((nvim_get_diff_flags() & DIFF_INTERNAL) != 0 && nvim_is_diffexpr_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        // Verify the constants match expected values
        assert_eq!(DIFF_FILLER, 0x001);
        assert_eq!(DIFF_HORIZONTAL, 0x040);
        assert_eq!(DIFF_HIDDEN_OFF, 0x100);
        assert_eq!(DIFF_INTERNAL, 0x200);
        assert_eq!(DIFF_CLOSE_OFF, 0x400);
    }
}
