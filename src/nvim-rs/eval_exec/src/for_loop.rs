//! `:for` loop implementation -- eval_for_line, next_for_item, free_for_info.
//!
//! Migrated from `eval_shim.c` Phase 3.
//!
//! The `forinfo_T` C struct is managed as an opaque heap allocation via C
//! accessor functions (nvim_forinfo_*). Rust treats it as `*mut c_void`.

#![allow(clippy::too_many_lines)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::eval::{eval0_impl, EvalargHandle, ExargHandle, TypevalHandle};

// =============================================================================
// Constants (matching C)
// =============================================================================

const OK: c_int = 1;
#[allow(dead_code)]
const FAIL: c_int = 0;
const EVAL_EVALUATE: c_int = 1;

const VAR_STRING: c_int = 2;
const VAR_LIST: c_int = 4;
const VAR_BLOB: c_int = 10;

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // forinfo_T accessors (all take void * to avoid exposing local typedef)
    fn nvim_forinfo_alloc() -> *mut c_void;
    fn nvim_forinfo_free(fi: *mut c_void);
    fn nvim_forinfo_get_varcount(fi: *mut c_void) -> c_int;
    fn nvim_forinfo_set_varcount(fi: *mut c_void, n: c_int);
    fn nvim_forinfo_get_semicolon(fi: *mut c_void) -> c_int;
    fn nvim_forinfo_set_semicolon(fi: *mut c_void, v: c_int);
    fn nvim_forinfo_has_list(fi: *mut c_void) -> bool;
    fn nvim_forinfo_has_blob(fi: *mut c_void) -> bool;
    fn nvim_forinfo_has_string(fi: *mut c_void) -> bool;
    fn nvim_forinfo_get_bi(fi: *mut c_void) -> c_int;
    fn nvim_forinfo_set_bi(fi: *mut c_void, n: c_int);
    fn nvim_forinfo_get_byte_idx(fi: *mut c_void) -> c_int;
    fn nvim_forinfo_set_byte_idx(fi: *mut c_void, n: c_int);
    fn nvim_forinfo_get_string(fi: *mut c_void) -> *mut c_char;
    fn nvim_forinfo_set_string(fi: *mut c_void, s: *mut c_char);
    fn nvim_forinfo_get_list(fi: *mut c_void) -> *mut c_void;
    fn nvim_forinfo_set_list(fi: *mut c_void, l: *mut c_void);
    fn nvim_forinfo_get_blob(fi: *mut c_void) -> *mut c_void;
    fn nvim_forinfo_set_blob(fi: *mut c_void, b: *mut c_void);
    fn nvim_forinfo_get_lw_item(fi: *mut c_void) -> *mut c_void;
    fn nvim_forinfo_set_lw_item(fi: *mut c_void, item: *mut c_void);
    fn nvim_forinfo_list_watch_add(fi: *mut c_void, l: *mut c_void);
    fn nvim_forinfo_list_watch_remove(fi: *mut c_void);

    // list/blob operations
    fn nvim_list_item_next(l: *mut c_void, item: *mut c_void) -> *mut c_void;
    fn nvim_tv_list_first(l: *mut c_void) -> *mut c_void;
    fn nvim_tv_list_unref(l: *mut c_void);
    fn nvim_tv_blob_unref(b: *mut c_void);
    fn nvim_tv_blob_copy(from: *mut c_void, to: TypevalHandle);

    // TV type / value accessors
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;
    fn nvim_tv_get_list(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_get_blob(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_get_vstring(tv: TypevalHandle) -> *mut c_char;
    fn nvim_tv_set_vstring_owned(tv: TypevalHandle, s: *mut c_char);
    fn tv_clear(tv: TypevalHandle);
    fn nvim_blob_len(b: *const c_void) -> c_int;
    fn nvim_blob_get(b: *const c_void, idx: c_int) -> c_int;

    // skip_var_list / ex_let_vars
    fn nvim_skip_var_list(
        arg: *const c_char,
        varcount: *mut c_int,
        semicolon: *mut c_int,
        nested: bool,
    ) -> *const c_char;
    fn nvim_ex_let_vars_number(
        arg: *mut c_char,
        n: i64,
        copy: bool,
        semicolon: c_int,
        varcount: c_int,
    ) -> bool;
    fn nvim_ex_let_vars_string_owned(
        arg: *mut c_char,
        s: *mut c_char,
        semicolon: c_int,
        varcount: c_int,
    ) -> bool;
    fn nvim_ex_let_vars_list_item(
        arg: *mut c_char,
        item: *mut c_void,
        semicolon: c_int,
        varcount: c_int,
    ) -> bool;

    fn evalarg_get_flags(ea: EvalargHandle) -> c_int;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn emsg(s: *const c_char) -> c_int;

    // xmemdupz / xstrdup / xfree / xmalloc
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;

    // utfc_ptr2len
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // emsg_skip
    fn nvim_emsg_skip_inc();
    fn nvim_emsg_skip_dec();

    // ascii_iswhite (declared in normal_shim, takes int)
    fn nvim_ascii_iswhite(c: c_int) -> bool;
}

// Error messages
static E_MISSING_IN: &[u8] = b"E690: Missing \"in\" after :for\0";
static E_STRING_LIST_OR_BLOB: &[u8] = b"E714: List, String, Tuple or Blob required\0";

/// Allocate a temporary typval on heap (64 bytes, zeroed).
unsafe fn alloc_typval() -> TypevalHandle {
    let ptr = xmalloc(64);
    ptr::write_bytes(ptr as *mut u8, 0, 64);
    TypevalHandle::from_ptr(ptr)
}

/// Free a heap-allocated typval.
unsafe fn free_typval(tv: TypevalHandle) {
    if !tv.is_null() {
        xfree(tv.as_ptr());
    }
}

/// Get a byte at a pointer (0 if null).
#[inline]
unsafe fn get_byte(p: *const c_char) -> u8 {
    if p.is_null() {
        0
    } else {
        *p as u8
    }
}

// =============================================================================
// eval_for_line
// =============================================================================

/// Evaluate the expression in `:for var in expr`, set up iteration state.
///
/// Returns an opaque `forinfo_T *` (as `*mut c_void`) to be passed to
/// `rs_next_for_item` and `rs_free_for_info`. Never returns null.
///
/// # Safety
/// - `arg` must be a valid C string
/// - `errp` must be a valid pointer
/// - `evalarg` must be non-null (contains eval_flags)
pub unsafe fn eval_for_line_impl(
    arg: *const c_char,
    errp: *mut bool,
    eap: ExargHandle,
    evalarg: EvalargHandle,
) -> *mut c_void {
    let fi = nvim_forinfo_alloc();
    let skip = (evalarg_get_flags(evalarg) & EVAL_EVALUATE) == 0;

    // Default: there is an error.
    *errp = true;

    let mut varcount: c_int = 0;
    let mut semicolon: c_int = 0;
    let expr = nvim_skip_var_list(arg, &mut varcount, &mut semicolon, false);
    nvim_forinfo_set_varcount(fi, varcount);
    nvim_forinfo_set_semicolon(fi, semicolon);

    if expr.is_null() {
        return fi;
    }

    let expr = skipwhite(expr);

    // Check for "in" keyword: must be "in" followed by NUL or whitespace
    let b0 = get_byte(expr);
    let b1 = get_byte(expr.add(1));
    let b2 = get_byte(expr.add(2));
    if b0 != b'i' || b1 != b'n' || !(b2 == 0 || nvim_ascii_iswhite(c_int::from(b2))) {
        emsg(E_MISSING_IN.as_ptr() as *const c_char);
        return fi;
    }

    if skip {
        nvim_emsg_skip_inc();
    }

    let expr_after_in = skipwhite(expr.add(2));
    let tv = alloc_typval();
    if eval0_impl(expr_after_in as *mut c_char, tv, eap, evalarg) == OK {
        *errp = false;
        if !skip {
            let tv_type = nvim_tv_get_type(tv);
            if tv_type == VAR_LIST {
                let l = nvim_tv_get_list(tv);
                if l.is_null() {
                    // null list is like empty list: do nothing
                    tv_clear(tv);
                } else {
                    // No need to increment refcount, already set for list in tv
                    nvim_forinfo_set_list(fi, l);
                    nvim_forinfo_list_watch_add(fi, l);
                    let first = nvim_tv_list_first(l);
                    nvim_forinfo_set_lw_item(fi, first);
                    // Don't call tv_clear: the list is now owned by fi
                }
            } else if tv_type == VAR_BLOB {
                nvim_forinfo_set_bi(fi, 0);
                let b = nvim_tv_get_blob(tv);
                if !b.is_null() {
                    // Make a copy so iteration still works if blob is changed
                    let btv = alloc_typval();
                    nvim_tv_blob_copy(b, btv);
                    let blob_copy = nvim_tv_get_blob(btv);
                    nvim_forinfo_set_blob(fi, blob_copy);
                    free_typval(btv);
                }
                tv_clear(tv);
            } else if tv_type == VAR_STRING {
                nvim_forinfo_set_byte_idx(fi, 0);
                let s = nvim_tv_get_vstring(tv);
                if s.is_null() {
                    let empty = xstrdup(c"".as_ptr());
                    nvim_forinfo_set_string(fi, empty);
                } else {
                    // Take ownership of the string from tv
                    nvim_forinfo_set_string(fi, s);
                    nvim_tv_set_vstring_owned(tv, ptr::null_mut());
                }
                tv_clear(tv);
            } else {
                emsg(E_STRING_LIST_OR_BLOB.as_ptr() as *const c_char);
                tv_clear(tv);
            }
        } else {
            tv_clear(tv);
        }
    } else {
        tv_clear(tv);
    }

    free_typval(tv);

    if skip {
        nvim_emsg_skip_dec();
    }

    fi
}

/// FFI export for eval_for_line.
///
/// # Safety
/// See `eval_for_line_impl` for safety requirements.
#[export_name = "eval_for_line"]
pub unsafe extern "C" fn rs_eval_for_line(
    arg: *const c_char,
    errp: *mut bool,
    eap: ExargHandle,
    evalarg: EvalargHandle,
) -> *mut c_void {
    eval_for_line_impl(arg, errp, eap, evalarg)
}

// =============================================================================
// next_for_item
// =============================================================================

/// Advance to the next item in a `:for` loop.
///
/// Returns `true` when a valid item was found, `false` when at end or error.
///
/// # Safety
/// - `fi_void` must be a valid `forinfo_T *` returned by `rs_eval_for_line`
/// - `arg` must be a valid mutable C string (variable name(s))
pub unsafe fn next_for_item_impl(fi_void: *mut c_void, arg: *mut c_char) -> bool {
    let semicolon = nvim_forinfo_get_semicolon(fi_void);
    let varcount = nvim_forinfo_get_varcount(fi_void);

    if nvim_forinfo_has_blob(fi_void) {
        let blob = nvim_forinfo_get_blob(fi_void);
        let bi = nvim_forinfo_get_bi(fi_void);
        if bi >= nvim_blob_len(blob as *const c_void) {
            return false;
        }
        let byte_val = i64::from(nvim_blob_get(blob as *const c_void, bi));
        nvim_forinfo_set_bi(fi_void, bi + 1);
        return nvim_ex_let_vars_number(arg, byte_val, true, semicolon, varcount);
    }

    if nvim_forinfo_has_string(fi_void) {
        let s = nvim_forinfo_get_string(fi_void);
        let byte_idx = nvim_forinfo_get_byte_idx(fi_void);
        let len = utfc_ptr2len(s.add(byte_idx as usize));
        if len == 0 {
            return false;
        }
        let dup = xmemdupz(s.add(byte_idx as usize) as *const c_void, len as usize);
        nvim_forinfo_set_byte_idx(fi_void, byte_idx + len);
        return nvim_ex_let_vars_string_owned(arg, dup, semicolon, varcount);
    }

    // List iteration
    let item = nvim_forinfo_get_lw_item(fi_void);
    if item.is_null() {
        return false;
    }
    let list = nvim_forinfo_get_list(fi_void);
    let next = nvim_list_item_next(list, item);
    nvim_forinfo_set_lw_item(fi_void, next);
    nvim_ex_let_vars_list_item(arg, item, semicolon, varcount)
}

/// FFI export for next_for_item.
///
/// # Safety
/// See `next_for_item_impl` for safety requirements.
#[export_name = "next_for_item"]
pub unsafe extern "C" fn rs_next_for_item(fi_void: *mut c_void, arg: *mut c_char) -> bool {
    next_for_item_impl(fi_void, arg)
}

// =============================================================================
// free_for_info
// =============================================================================

/// Free the structure used to store info used by `:for`.
///
/// # Safety
/// - `fi_void` must be a valid `forinfo_T *` or null
pub unsafe fn free_for_info_impl(fi_void: *mut c_void) {
    if fi_void.is_null() {
        return;
    }
    if nvim_forinfo_has_list(fi_void) {
        nvim_forinfo_list_watch_remove(fi_void);
        let l = nvim_forinfo_get_list(fi_void);
        nvim_tv_list_unref(l);
    } else if nvim_forinfo_has_blob(fi_void) {
        let b = nvim_forinfo_get_blob(fi_void);
        nvim_tv_blob_unref(b);
    } else {
        let s = nvim_forinfo_get_string(fi_void);
        if !s.is_null() {
            xfree(s as *mut c_void);
        }
    }
    nvim_forinfo_free(fi_void);
}

/// FFI export for free_for_info.
///
/// # Safety
/// See `free_for_info_impl` for safety requirements.
#[export_name = "free_for_info"]
pub unsafe extern "C" fn rs_free_for_info(fi_void: *mut c_void) {
    free_for_info_impl(fi_void);
}
