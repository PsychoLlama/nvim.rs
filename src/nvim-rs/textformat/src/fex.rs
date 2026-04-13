//! formatexpr evaluation for text formatting.
//!
//! This module provides the Rust implementation of `fex_format()`,
//! which evaluates the `'formatexpr'` option.

use std::ffi::{c_char, c_int, c_long, c_void};

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    fn nvim_textfmt_fex_was_set_insecurely() -> bool;
    fn nvim_textfmt_set_vv_lnum(val: i64);
    fn nvim_textfmt_set_vv_count(val: i64);
    fn nvim_textfmt_set_vv_char(c: c_int);
    fn nvim_textfmt_clear_vv_char();
    fn nvim_textfmt_get_curbuf_b_p_fex() -> *mut c_char;
    fn nvim_textfmt_fex_save_sctx();
    fn nvim_textfmt_fex_restore_sctx();
    fn nvim_textfmt_sandbox_inc();
    fn nvim_textfmt_sandbox_dec();
    fn nvim_textfmt_eval_to_number(expr: *mut c_char) -> c_int;
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    #[link_name = "xfree"]
    fn nvim_xfree(ptr: *mut c_void);
}

// =============================================================================
// Implementation
// =============================================================================

/// Evaluate 'formatexpr' and return its result.
///
/// Sets v:lnum, v:count, and v:char before evaluation, and cleans them up after.
///
/// # Safety
/// Accesses global state via C functions.
pub(crate) unsafe fn fex_format_impl(lnum: c_int, count: c_long, c: c_int) -> c_int {
    let use_sandbox = nvim_textfmt_fex_was_set_insecurely();

    // Set v:lnum to the first line number and v:count to the number of lines.
    // Set v:char to the character to be inserted (can be NUL).
    nvim_textfmt_set_vv_lnum(lnum as i64);
    nvim_textfmt_set_vv_count(count);
    nvim_textfmt_set_vv_char(c);

    // Make a copy, the option could be changed while calling it.
    let fex = nvim_xstrdup(nvim_textfmt_get_curbuf_b_p_fex());
    nvim_textfmt_fex_save_sctx();

    // Evaluate the function.
    if use_sandbox {
        nvim_textfmt_sandbox_inc();
    }
    let r = nvim_textfmt_eval_to_number(fex);
    if use_sandbox {
        nvim_textfmt_sandbox_dec();
    }

    nvim_textfmt_clear_vv_char();
    nvim_xfree(fex.cast());
    nvim_textfmt_fex_restore_sctx();

    r
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Evaluate 'formatexpr' for text formatting.
///
/// # Safety
/// Accesses global state via C functions.
#[export_name = "fex_format"]
pub unsafe extern "C" fn rs_fex_format(lnum: c_int, count: c_long, c: c_int) -> c_int {
    fex_format_impl(lnum, count, c)
}
