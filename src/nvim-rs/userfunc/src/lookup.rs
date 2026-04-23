//! Function lookup and cookie accessors for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 6.
//! Covers: func_has_ended, func_has_abort, func_name, func_breakpoint,
//!         func_dbg_tick, func_level, get_func_arity.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void};

/// linenr_T is i32 on all platforms.
type LinenrT = i32;

// =============================================================================
// C Extern Declarations
// =============================================================================

extern "C" {
    // funccall_T field accessors
    fn nvim_fc_get_func(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_get_returned(fc: *mut c_void) -> c_int;
    fn nvim_fc_get_level(fc: *mut c_void) -> c_int;
    fn nvim_fc_get_breakpoint_ptr(fc: *mut c_void) -> *mut LinenrT;
    fn nvim_fc_get_dbg_tick_ptr(fc: *mut c_void) -> *mut c_int;

    // ufunc_T field accessors (use *mut to match listing.rs declarations)
    fn nvim_ufunc_get_name(fp: *mut c_void) -> *const c_char;
    fn nvim_ufunc_get_flags(fp: *mut c_void) -> c_int;

    // ufunc_T arg count accessors (for get_func_arity)
    fn nvim_ufunc_get_args_len(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_get_def_args_len(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_get_varargs(fp: *mut c_void) -> c_int;

    // find_func / find_internal_func
    fn find_func(name: *const c_char) -> *mut c_void;
    fn nvim_get_internal_func_arity(
        name: *const c_char,
        required: *mut c_int,
        optional: *mut c_int,
    ) -> c_int;

    // names.rs already exports rs_fname_trans_sid
    fn rs_fname_trans_sid(
        name: *const c_char,
        fname_buf: *mut c_char,
        tofree: *mut *mut c_char,
        error: *mut c_int,
    ) -> *mut c_char;

    fn xfree(ptr: *mut c_void);

    // globals
    static did_emsg: c_int;

    // error checking
    fn aborted_in_try() -> c_int;
}

// FC_ABORT flag (matches userfunc.h)
const FC_ABORT: c_int = 0x01;

// FCERR_NONE = 0 (matches C definition)
const FCERR_NONE: c_int = 0;
// FLEN_FIXED = 40 (must match C define)
const FLEN_FIXED: usize = 40;

// =============================================================================
// func_has_ended
// =============================================================================

/// Return true if the currently active function should be ended.
/// Used inside a `:while`.
///
/// # Safety
/// `cookie` must be a valid `funccall_T` pointer cast to `*mut c_void`.
#[unsafe(export_name = "func_has_ended")]
pub unsafe extern "C" fn rs_func_has_ended(cookie: *mut c_void) -> c_int {
    let fp = unsafe { nvim_fc_get_func(cookie) };
    let flags = unsafe { nvim_ufunc_get_flags(fp) };
    let fc_returned = unsafe { nvim_fc_get_returned(cookie) };
    let aborted =
        (flags & FC_ABORT != 0) && unsafe { did_emsg } != 0 && unsafe { aborted_in_try() } == 0;
    c_int::from(aborted || fc_returned != 0)
}

// =============================================================================
// func_has_abort
// =============================================================================

/// Return true if cookie indicates a function which "abort"s on errors.
///
/// # Safety
/// `cookie` must be a valid `funccall_T` pointer cast to `*mut c_void`.
#[unsafe(export_name = "func_has_abort")]
pub unsafe extern "C" fn rs_func_has_abort(cookie: *mut c_void) -> c_int {
    let fp = unsafe { nvim_fc_get_func(cookie) };
    let flags = unsafe { nvim_ufunc_get_flags(fp) };
    flags & FC_ABORT
}

// =============================================================================
// func_name
// =============================================================================

/// Return the name of the executed function.
///
/// # Safety
/// `cookie` must be a valid `funccall_T` pointer cast to `*mut c_void`.
#[unsafe(export_name = "func_name")]
pub unsafe extern "C" fn rs_func_name(cookie: *mut c_void) -> *mut c_char {
    let fp = unsafe { nvim_fc_get_func(cookie) };
    unsafe { nvim_ufunc_get_name(fp) }.cast_mut()
}

// =============================================================================
// func_breakpoint
// =============================================================================

/// Return address of next breakpoint line for a funccall cookie.
///
/// # Safety
/// `cookie` must be a valid `funccall_T` pointer cast to `*mut c_void`.
#[unsafe(export_name = "func_breakpoint")]
pub unsafe extern "C" fn rs_func_breakpoint(cookie: *mut c_void) -> *mut LinenrT {
    unsafe { nvim_fc_get_breakpoint_ptr(cookie) }
}

// =============================================================================
// func_dbg_tick
// =============================================================================

/// Return address of the debug tick for a funccall cookie.
///
/// # Safety
/// `cookie` must be a valid `funccall_T` pointer cast to `*mut c_void`.
#[unsafe(export_name = "func_dbg_tick")]
pub unsafe extern "C" fn rs_func_dbg_tick(cookie: *mut c_void) -> *mut c_int {
    unsafe { nvim_fc_get_dbg_tick_ptr(cookie) }
}

// =============================================================================
// func_level
// =============================================================================

/// Return the nesting level for a funccall cookie.
///
/// # Safety
/// `cookie` must be a valid `funccall_T` pointer cast to `*mut c_void`.
#[unsafe(export_name = "func_level")]
pub unsafe extern "C" fn rs_func_level(cookie: *mut c_void) -> c_int {
    unsafe { nvim_fc_get_level(cookie) }
}

// =============================================================================
// get_func_arity
// =============================================================================

/// Get the arity of a function by name.
/// Returns OK (0) on success, FAIL (1) if function not found.
/// On success sets `*required`, `*optional`, `*varargs`.
///
/// # Safety
/// All pointers must be valid and non-null; `name` must be NUL-terminated.
#[unsafe(export_name = "get_func_arity")]
pub unsafe extern "C" fn rs_get_func_arity(
    name: *const c_char,
    required: *mut c_int,
    optional: *mut c_int,
    varargs: *mut bool,
) -> c_int {
    // Try internal (built-in) function first
    let mut req: c_int = 0;
    let mut opt: c_int = 0;
    let found_internal = unsafe {
        nvim_get_internal_func_arity(
            name,
            std::ptr::addr_of_mut!(req),
            std::ptr::addr_of_mut!(opt),
        )
    };
    if found_internal != 0 {
        unsafe {
            *required = req;
            *optional = opt;
            *varargs = false;
        }
        return FCERR_NONE; // OK
    }

    // User-defined function: translate SID prefix
    let mut fname_buf = [0u8; FLEN_FIXED + 1];
    let mut tofree: *mut c_char = std::ptr::null_mut();
    let mut error: c_int = FCERR_NONE;

    let fname = unsafe {
        rs_fname_trans_sid(
            name,
            fname_buf.as_mut_ptr().cast::<c_char>(),
            std::ptr::addr_of_mut!(tofree),
            std::ptr::addr_of_mut!(error),
        )
    };

    let ufunc = if error == FCERR_NONE {
        unsafe { find_func(fname) }
    } else {
        std::ptr::null_mut()
    };

    unsafe { xfree(tofree.cast::<c_void>()) };

    if ufunc.is_null() {
        return 1; // FAIL
    }

    let args_len = unsafe { nvim_ufunc_get_args_len(ufunc) };
    let def_args_len = unsafe { nvim_ufunc_get_def_args_len(ufunc) };
    let is_varargs = unsafe { nvim_ufunc_get_varargs(ufunc) } != 0;

    let min_argcount = args_len - def_args_len;
    unsafe {
        *required = min_argcount;
        *optional = args_len - min_argcount;
        *varargs = is_varargs;
    }

    FCERR_NONE // OK
}
