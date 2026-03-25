//! Command-line and register completion support.
//!
//! This module provides helper functions for command-line completion (CTRL-X CTRL-V)
//! and register completion (CTRL-X CTRL-R).
//! The core operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// nvim_get_cmdline_compl_info_impl: deleted (Phase 27), inlined below

extern "C" {
    // Helpers for inlined rs_get_cmdline_compl_info (Phase 27)
    fn nvim_compl_xp_get_context() -> c_int;
    fn nvim_compl_xp_get_pattern() -> *const c_char;
    fn nvim_compl_xp_set_cmd_context(len: c_int, col: c_int);
    fn nvim_compl_xp_nlua_expand();
}

// ExpandContext constants (from cmdexpand_defs.h)
const EXPAND_UNSUCCESSFUL: c_int = -2;
const EXPAND_NOTHING: c_int = 0;
const EXPAND_LUA: c_int = 63;
const OK: c_int = 1;

// CTRL-X mode constants
const CTRL_X_CMDLINE: c_int = 11;
const CTRL_X_REGISTER: c_int = 19;

/// Get the pattern, column and length for command-line completion.
///
/// Sets `compl_col`, `compl_length`, and `compl_pattern` globals.
///
/// Rust translation of nvim_get_cmdline_compl_info_impl (Phase 27).
///
/// # Safety
/// `line` must be a valid C string. Requires valid global completion state.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_get_cmdline_compl_info(line: *mut c_char, curs_col: c_int) -> c_int {
    use crate::vars::NvimString;
    extern "C" {
        fn cbuf_to_string(buf: *const c_char, size: usize) -> NvimString;
    }
    crate::vars::compl_pattern = cbuf_to_string(line.cast_const(), curs_col as usize);
    nvim_compl_xp_set_cmd_context(crate::vars::compl_pattern.size as c_int, curs_col);
    if nvim_compl_xp_get_context() == EXPAND_LUA {
        nvim_compl_xp_nlua_expand();
    }
    let ctx = nvim_compl_xp_get_context();
    if ctx == EXPAND_UNSUCCESSFUL || ctx == EXPAND_NOTHING {
        crate::vars::nvim_set_compl_col(curs_col);
    } else {
        let pat_ptr = nvim_compl_xp_get_pattern();
        let base_ptr = crate::vars::compl_pattern.data.cast_const();
        // SAFETY: both pointers are within the same compl_pattern allocation
        let offset = pat_ptr.offset_from(base_ptr) as c_int;
        crate::vars::nvim_set_compl_col(offset);
    }
    crate::vars::nvim_set_compl_length(curs_col - crate::vars::nvim_get_compl_col());
    OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_CMDLINE, 11);
        assert_eq!(CTRL_X_REGISTER, 19);
    }
}
