//! make_partial: turn a dict.Func call into a partial bound to the dict.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 10.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void};

// VarType constants (matching C enum)
const VAR_FUNC: c_int = 3;
const VAR_STRING: c_int = 2;
const VAR_PARTIAL: c_int = 9;

// FC_DICT flag (matching userfunc.h)
const FC_DICT: c_int = 0x04;

// sizeof(partial_T) == 48, sizeof(typval_T) == 16 (verified by _Static_assert in C)
const SIZEOF_PARTIAL_T: usize = 48;
const SIZEOF_TYPVAL_T: usize = 16;

// FLEN_FIXED must match C define
const FLEN_FIXED: usize = 40;

extern "C" {
    // typval_T field read accessors
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_get_string_ptr(tv: *const c_void) -> *const c_char;
    fn nvim_tv_get_partial(tv: *const c_void) -> *mut c_void;

    // typval_T field write accessors
    fn nvim_tv_set_type(tv: *mut c_void, v_type: c_int);
    fn nvim_tv_set_partial(tv: *mut c_void, p: *mut c_void);

    // partial_T getters (existing)
    fn nvim_partial_get_func(pt: *const c_void) -> *mut c_void;
    fn nvim_partial_get_argc(pt: *const c_void) -> c_int;

    // partial_T getters (new)
    fn nvim_partial_get_name(pt: *const c_void) -> *mut c_char;
    fn nvim_partial_get_argv(pt: *const c_void) -> *mut c_void;

    // partial_T setters (new)
    fn nvim_partial_set_refcount(pt: *mut c_void, v: c_int);
    fn nvim_partial_set_dict(pt: *mut c_void, dict: *mut c_void);
    fn nvim_partial_set_auto(pt: *mut c_void, v: c_int);
    fn nvim_partial_set_name(pt: *mut c_void, name: *mut c_char);
    fn nvim_partial_set_func(pt: *mut c_void, fp: *mut c_void);
    fn nvim_partial_set_argv(pt: *mut c_void, argv: *mut c_void);
    fn nvim_partial_set_argc(pt: *mut c_void, argc: c_int);

    // ufunc_T getter
    fn nvim_ufunc_get_flags(fp: *mut c_void) -> c_int;

    // Function lookup
    fn find_func(name: *const c_char) -> *mut c_void;

    // Name translation (already Rust, call via C ABI)
    fn rs_fname_trans_sid(
        name: *const c_char,
        fname_buf: *mut c_char,
        tofree: *mut *mut c_char,
        error: *mut c_int,
    ) -> *mut c_char;

    // Memory
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // func_ref and func_ptr_ref are already exported from Rust (refcount.rs),
    // but we call them via their C ABI names.
    fn func_ref(name: *mut c_char);
    fn func_ptr_ref(fp: *mut c_void);

    // typval operations
    fn tv_copy(from: *const c_void, to: *mut c_void);
    fn partial_unref(pt: *mut c_void);

    // dict refcount
    fn nvim_tv_dict_incr_refcount(dict: *mut c_void);
}

// =============================================================================
// make_partial
// =============================================================================

/// Turn a `dict.Func` call result into a partial bound to `dict`.
///
/// If `rettv` holds a function with `FC_DICT` flag and `selfdict` is not null,
/// creates a `partial_T` binding the function to the dict and stores it in `rettv`.
///
/// # Safety
/// `selfdict` and `rettv` must be valid non-null pointers to C `dict_T` and `typval_T`.
#[unsafe(export_name = "make_partial")]
pub unsafe extern "C" fn rs_make_partial(selfdict: *mut c_void, rettv: *mut c_void) {
    let mut tofree: *mut c_char = std::ptr::null_mut();
    let mut fname_buf = [0u8; FLEN_FIXED + 1];
    let mut error: c_int = 0;

    // Determine the ufunc pointer.
    let tv_type = unsafe { nvim_tv_get_type(rettv) };
    let fp: *mut c_void;

    if tv_type == VAR_PARTIAL {
        let ret_pt = unsafe { nvim_tv_get_partial(rettv) };
        let func_from_partial = unsafe { nvim_partial_get_func(ret_pt) };
        if func_from_partial.is_null() {
            // Get name from partial
            let fname_raw = unsafe { nvim_partial_get_name(ret_pt) };
            let fname = unsafe {
                rs_fname_trans_sid(
                    fname_raw,
                    fname_buf.as_mut_ptr().cast::<c_char>(),
                    std::ptr::addr_of_mut!(tofree),
                    std::ptr::addr_of_mut!(error),
                )
            };
            fp = unsafe { find_func(fname) };
            unsafe { xfree(tofree.cast::<c_void>()) };
        } else {
            fp = func_from_partial;
        }
    } else {
        // VAR_FUNC or VAR_STRING: get name from vval.v_string
        let fname_raw = unsafe { nvim_tv_get_string_ptr(rettv) };
        let fname = unsafe {
            rs_fname_trans_sid(
                fname_raw,
                fname_buf.as_mut_ptr().cast::<c_char>(),
                std::ptr::addr_of_mut!(tofree),
                std::ptr::addr_of_mut!(error),
            )
        };
        fp = unsafe { find_func(fname) };
        unsafe { xfree(tofree.cast::<c_void>()) };
    }

    // Only proceed if the function has FC_DICT flag.
    if fp.is_null() {
        return;
    }
    let flags = unsafe { nvim_ufunc_get_flags(fp) };
    if (flags & FC_DICT) == 0 {
        return;
    }

    // Allocate a new partial_T (zeroed).
    let pt = unsafe { xcalloc(1, SIZEOF_PARTIAL_T) };
    unsafe { nvim_partial_set_refcount(pt, 1) };
    unsafe { nvim_partial_set_dict(pt, selfdict) };
    unsafe { nvim_tv_dict_incr_refcount(selfdict) };
    unsafe { nvim_partial_set_auto(pt, 1) };

    let tv_type2 = unsafe { nvim_tv_get_type(rettv) };
    if tv_type2 == VAR_FUNC || tv_type2 == VAR_STRING {
        // Take over the function name from rettv.
        let s = unsafe { nvim_tv_get_string_ptr(rettv) };
        unsafe { nvim_partial_set_name(pt, s.cast_mut()) };
    } else {
        // Partial case: copy name/func and args.
        let ret_pt = unsafe { nvim_tv_get_partial(rettv) };
        let ret_name = unsafe { nvim_partial_get_name(ret_pt) };

        if ret_name.is_null() {
            let ret_func = unsafe { nvim_partial_get_func(ret_pt) };
            unsafe { nvim_partial_set_func(pt, ret_func) };
            unsafe { func_ptr_ref(ret_func) };
        } else {
            let dup = unsafe { xstrdup(ret_name) };
            unsafe { nvim_partial_set_name(pt, dup) };
            unsafe { func_ref(dup) };
        }

        let ret_argc = unsafe { nvim_partial_get_argc(ret_pt) };
        if ret_argc > 0 {
            let arg_size = SIZEOF_TYPVAL_T * ret_argc as usize;
            let new_argv = unsafe { xmalloc(arg_size) };
            unsafe { nvim_partial_set_argv(pt, new_argv) };
            unsafe { nvim_partial_set_argc(pt, ret_argc) };

            let src_argv = unsafe { nvim_partial_get_argv(ret_pt) };
            for i in 0..ret_argc as usize {
                let src = unsafe { src_argv.add(i * SIZEOF_TYPVAL_T) };
                let dst = unsafe { new_argv.add(i * SIZEOF_TYPVAL_T) };
                unsafe { tv_copy(src, dst) };
            }
        }

        unsafe { partial_unref(ret_pt) };
    }

    // Store the new partial in rettv.
    unsafe { nvim_tv_set_type(rettv, VAR_PARTIAL) };
    unsafe { nvim_tv_set_partial(rettv, pt) };
}
