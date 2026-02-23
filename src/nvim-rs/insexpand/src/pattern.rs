//! Completion pattern helpers.
//!
//! This module provides Rust implementations for computing the pattern,
//! column, and length for various completion modes (normal, whole-line,
//! filename, spell). The heavy C string manipulation is done via compound
//! C accessors; Rust provides the dispatch and extern declarations.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// Compound C accessors that implement the core pattern-building logic.
// Each calls the original C subsystem functions internally.
extern "C" {
    fn nvim_get_normal_compl_info_impl(
        line: *mut c_char,
        startcol: c_int,
        curs_col: c_int,
    ) -> c_int;
    fn nvim_get_wholeline_compl_info_impl(line: *mut c_char, curs_col: c_int) -> c_int;
    fn nvim_get_filename_compl_info_impl(
        line: *mut c_char,
        startcol: c_int,
        curs_col: c_int,
    ) -> c_int;
    fn nvim_get_spell_compl_info_impl(startcol: c_int, curs_col: c_int) -> c_int;
}

/// Get the pattern, column and length for normal (keyword) completion.
///
/// Sets compl_col, compl_length, compl_pattern, and compl_from_nonkeyword.
/// Also calls setup_cpt_sources/prepare_cpt_compl_funcs for normal CTRL-N.
///
/// # Safety
/// Requires valid global state; line must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_normal_compl_info(
    line: *mut c_char,
    startcol: c_int,
    curs_col: c_int,
) -> c_int {
    nvim_get_normal_compl_info_impl(line, startcol, curs_col)
}

/// Get the pattern, column and length for whole-line completion.
///
/// Sets compl_col, compl_length, compl_pattern.
///
/// # Safety
/// Requires valid global state; line must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_wholeline_compl_info(line: *mut c_char, curs_col: c_int) -> c_int {
    nvim_get_wholeline_compl_info_impl(line, curs_col)
}

/// Get the pattern, column and length for filename completion.
///
/// Sets compl_col, compl_length, compl_pattern.
///
/// # Safety
/// Requires valid global state; line must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_filename_compl_info(
    line: *mut c_char,
    startcol: c_int,
    curs_col: c_int,
) -> c_int {
    nvim_get_filename_compl_info_impl(line, startcol, curs_col)
}

/// Get the pattern, column and length for spell completion.
///
/// Sets compl_col, compl_length, compl_pattern.
///
/// # Safety
/// Requires valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_spell_compl_info(startcol: c_int, curs_col: c_int) -> c_int {
    nvim_get_spell_compl_info_impl(startcol, curs_col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_declarations_exist() {
        // Verify the module compiles and FFI declarations are present.
        // Actual function calls require a running Neovim session.
        let _: unsafe extern "C" fn(*mut c_char, c_int, c_int) -> c_int =
            nvim_get_normal_compl_info_impl;
    }
}
