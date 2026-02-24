//! User-defined and omni completion support.
//!
//! This module provides helper functions for user-defined function completion
//! (CTRL-X CTRL-U), omni completion (CTRL-X CTRL-O), and thesaurusfunc.
//! Also provides GC marking for the global completion callbacks.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;

    // Compound accessors for Phase 5 (pass 5): callback management
    fn nvim_did_set_completefunc_impl(args: *mut std::ffi::c_void) -> *const c_char;
    fn nvim_did_set_omnifunc_impl(args: *mut std::ffi::c_void) -> *const c_char;
    fn nvim_did_set_thesaurusfunc_impl(args: *mut std::ffi::c_void) -> *const c_char;
    fn nvim_set_ref_in_insexpand_funcs_impl(copy_id: c_int) -> c_int;
}

// CTRL-X mode constants
const CTRL_X_FUNCTION: c_int = 10;
const CTRL_X_OMNI: c_int = 11;

/// Parse the 'completefunc' option value and set the callback.
///
/// Invoked when the 'completefunc' option is set. Delegates to the C
/// compound accessor which handles buf_T and Callback type interactions.
///
/// # Safety
/// Requires valid optset_T argument.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_completefunc(args: *mut std::ffi::c_void) -> *const c_char {
    nvim_did_set_completefunc_impl(args)
}

/// Parse the 'omnifunc' option value and set the callback.
///
/// Invoked when the 'omnifunc' option is set. Delegates to the C
/// compound accessor which handles buf_T and Callback type interactions.
///
/// # Safety
/// Requires valid optset_T argument.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_omnifunc(args: *mut std::ffi::c_void) -> *const c_char {
    nvim_did_set_omnifunc_impl(args)
}

/// Parse the 'thesaurusfunc' option value and set the callback.
///
/// Invoked when the 'thesaurusfunc' option is set. Handles both local and
/// global option setting. Delegates to the C compound accessor.
///
/// # Safety
/// Requires valid optset_T argument.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_thesaurusfunc(args: *mut std::ffi::c_void) -> *const c_char {
    nvim_did_set_thesaurusfunc_impl(args)
}

/// Mark global completion callbacks (completefunc, omnifunc, thesaurusfunc)
/// with copyID so they are not garbage collected.
///
/// # Safety
/// Requires valid GC state.
#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_insexpand_funcs(copy_id: c_int) -> bool {
    nvim_set_ref_in_insexpand_funcs_impl(copy_id) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(CTRL_X_FUNCTION, 10);
        assert_eq!(CTRL_X_OMNI, 11);
    }
}
