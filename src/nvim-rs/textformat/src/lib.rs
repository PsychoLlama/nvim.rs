//! Text formatting utilities for Neovim
//!
//! This crate provides Rust implementations of text formatting functions
//! from `src/nvim/textformat.c`.

#![allow(unsafe_code)] // FFI requires unsafe

mod auto;
mod fex;
mod format_lines;
mod ops;
mod paragraph;
mod textwidth;

use std::ffi::{c_char, c_int};

// Re-export FFI functions from submodules
pub use auto::{rs_auto_format, rs_check_auto_format};
pub use fex::rs_fex_format;
pub use format_lines::rs_format_lines;
pub use ops::{rs_op_format, rs_op_formatexpr};
pub use paragraph::{rs_ends_in_white, rs_fmt_check_par, rs_paragraph_start, rs_same_leader};
pub use textwidth::rs_comp_textwidth;

// C accessor functions
extern "C" {
    fn nvim_get_p_paste() -> c_int;
    fn nvim_get_curbuf_b_p_fo() -> *const c_char;
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
}

/// Return true if format option 'x' is in effect.
/// Take care of no formatting when 'paste' is set.
#[inline]
pub(crate) fn has_format_option_impl(x: c_int) -> bool {
    unsafe {
        // If paste is set, no format options are active
        if nvim_get_p_paste() != 0 {
            return false;
        }

        let fo = nvim_get_curbuf_b_p_fo();
        if fo.is_null() {
            return false;
        }

        // Check if the character x is in the format options string
        !nvim_vim_strchr(fo, x).is_null()
    }
}

/// Return true if format option 'x' is in effect.
/// Take care of no formatting when 'paste' is set.
///
/// # Safety
/// This function accesses global state (p_paste, curbuf->b_p_fo).
#[no_mangle]
pub extern "C" fn rs_has_format_option(x: c_int) -> c_int {
    c_int::from(has_format_option_impl(x))
}

#[cfg(test)]
mod tests {
    // Tests require FFI stubs which aren't available in pure Rust testing.
    // Integration testing is done via the full Neovim build.
}
