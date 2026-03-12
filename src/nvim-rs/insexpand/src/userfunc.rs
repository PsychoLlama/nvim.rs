//! User-defined and omni completion support.
//!
//! This module provides helper functions for user-defined function completion
//! (CTRL-X CTRL-U), omni completion (CTRL-X CTRL-O), and thesaurusfunc.
//! Also provides GC marking for the global completion callbacks.

#![allow(dead_code, unused_imports)]
use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

const OK: c_int = 0;
const FAIL: c_int = -1;
// Sentinel returned by nvim_callback_call_findstart when cursor moved (error emitted).
const CURSOR_MOVED_ERROR: c_int = c_int::MIN;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;

    // Accessors for get_userdefined_compl_info migration (Phase 2)
    fn nvim_get_complete_funcname_empty(ctrl_x_mode: c_int) -> c_int;
    fn nvim_ctrl_x_mode_is_function() -> c_int;
    fn nvim_get_insert_callback_opaque(ctrl_x_mode: c_int) -> *mut c_void;
    fn nvim_callback_call_findstart(cb_opaque: *mut c_void) -> c_int;
    fn nvim_ctrl_x_mode_reset_to_normal();
    fn nvim_emit_completefunc_not_set_error(is_function: c_int);
    fn nvim_clear_compl_opt_refresh_always();
    fn rs_set_compl_globals(startcol: c_int, curs_col: c_int, is_cpt_compl: c_int);
    fn aborting() -> c_int;

    // Compound accessors for Phase 5 (pass 5): callback management
    fn nvim_did_set_completefunc_impl(args: *mut c_void) -> *const c_char;
    fn nvim_did_set_omnifunc_impl(args: *mut c_void) -> *const c_char;
    fn nvim_did_set_thesaurusfunc_impl(args: *mut c_void) -> *const c_char;
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
#[export_name = "did_set_completefunc"]
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
#[export_name = "did_set_omnifunc"]
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
#[export_name = "did_set_thesaurusfunc"]
pub unsafe extern "C" fn rs_did_set_thesaurusfunc(args: *mut std::ffi::c_void) -> *const c_char {
    nvim_did_set_thesaurusfunc_impl(args)
}

/// Get the completion info for user-defined completion ('omnifunc', 'completefunc',
/// 'thesaurusfunc') or a cpt-option function source.
///
/// - `curs_col`: current cursor column
/// - `cb_opaque`: opaque Callback pointer; if null, uses the global mode callback
/// - `startcol_out`: if non-null, receives the start column returned by the function
///
/// Returns OK (0) on success, FAIL (-1) on failure.
///
/// # Safety
/// Requires valid completion state and cursor.
#[no_mangle]
pub unsafe extern "C" fn rs_get_userdefined_compl_info(
    curs_col: c_int,
    cb_opaque: *mut c_void,
    startcol_out: *mut c_int,
) -> c_int {
    let is_cpt_function = !cb_opaque.is_null();
    let cb = if is_cpt_function {
        cb_opaque
    } else {
        // Use the global mode callback; check that it's set.
        let ctrl_x_mode = nvim_get_ctrl_x_mode();
        if nvim_get_complete_funcname_empty(ctrl_x_mode) != 0 {
            nvim_emit_completefunc_not_set_error(nvim_ctrl_x_mode_is_function());
            return FAIL;
        }
        nvim_get_insert_callback_opaque(ctrl_x_mode)
    };

    let col = nvim_callback_call_findstart(cb);

    // Cursor was moved -- error already emitted by the accessor.
    if col == CURSOR_MOVED_ERROR {
        return FAIL;
    }

    // Set startcol unconditionally before checking for special return values
    // so callers can inspect it even when we return FAIL.
    if !startcol_out.is_null() {
        *startcol_out = col;
    }

    // -2: user wants to cancel without error; also cancel if aborting.
    if col == -2 || aborting() != 0 {
        return FAIL;
    }

    // -3: cancel and exit CTRL-X mode (for non-cpt calls only).
    if col == -3 {
        if !is_cpt_function {
            nvim_ctrl_x_mode_reset_to_normal();
        }
        return FAIL;
    }

    // Reset extended completion parameters.
    nvim_clear_compl_opt_refresh_always();

    // Set global completion state (only for the non-cpt path).
    if !is_cpt_function {
        rs_set_compl_globals(col, curs_col, 0);
    }

    OK
}

/// Mark global completion callbacks (completefunc, omnifunc, thesaurusfunc)
/// with copyID so they are not garbage collected.
///
/// # Safety
/// Requires valid GC state.
#[allow(clippy::must_use_candidate)]
#[export_name = "set_ref_in_insexpand_funcs"]
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
