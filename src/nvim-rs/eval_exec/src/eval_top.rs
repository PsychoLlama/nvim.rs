//! Top-level VimL expression evaluation coordination functions.
//!
//! Migrated from `src/nvim/eval_shim.c`:
//! - `eval_to_bool`, `eval_to_number`, `eval_to_string_skip`,
//!   `eval_to_string_eap`, `eval_to_string`, `eval_to_string_safe`
//! - `typval2string` (internal helper)
//! - `skip_expr`, `eval1_emsg` (internal helpers)
//! - `eval_expr_typval`, `eval_expr_to_bool`
//! - `eval_expr_partial`, `eval_expr_func`, `eval_expr_string` (internal helpers)

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)] // Some constants reserved for potential future use

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::eval::{EvalargHandle, ExargHandle, TypevalHandle};

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;
const NOTDONE: c_int = 2;

const VAR_PARTIAL: c_int = 9;
const VAR_FUNC: c_int = 3;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;

// NUMBUFLEN from Neovim - size of buffer for tv_get_string_buf_chk
const NUMBUFLEN: usize = 65;

// =============================================================================
// C External Functions (accessors and helpers)
// =============================================================================

extern "C" {
    // Eval0/eval1 (already in Rust, called via C ABI)
    fn rs_eval0(
        arg: *mut c_char,
        rettv: TypevalHandle,
        eap: ExargHandle,
        evalarg: EvalargHandle,
    ) -> c_int;
    fn rs_eval1(arg: *mut *mut c_char, rettv: TypevalHandle, evalarg: EvalargHandle) -> c_int;

    // Evalarg lifecycle
    fn nvim_evalarg_alloc_from_eap(eap: ExargHandle, skip: bool) -> EvalargHandle;
    fn nvim_evalarg_clear_and_free(ea: EvalargHandle, eap: ExargHandle);
    fn nvim_get_evalarg_evaluate_ptr() -> EvalargHandle;
    fn evalarg_get_flags(ea: EvalargHandle) -> c_int;
    fn evalarg_set_flags(ea: EvalargHandle, flags: c_int);

    // emsg_skip inc/dec
    fn nvim_emsg_skip_inc();
    fn nvim_emsg_skip_dec();

    // emsg_off inc/dec
    fn nvim_eval_emsg_off_inc();
    fn nvim_eval_emsg_off_dec();

    // sandbox/textlock inc/dec
    fn nvim_eval_sandbox_inc();
    fn nvim_eval_sandbox_dec();
    fn nvim_eval_textlock_inc();
    fn nvim_eval_textlock_dec();

    // funccal save/restore
    fn nvim_eval_save_funccal() -> *mut c_void;
    fn nvim_eval_restore_funccal(entry: *mut c_void);

    // may_call_simple_func
    fn nvim_eval_may_call_simple_func(arg: *const c_char, rettv: TypevalHandle) -> c_int;

    // typval operations
    fn tv_clear(tv: TypevalHandle);
    fn tv_get_number_chk(tv: TypevalHandle, error: *mut bool) -> i64;
    fn tv_get_string_buf_chk(tv: TypevalHandle, buf: *mut c_char) -> *const c_char;

    // eval_to_string_safe wrapper (calls through to our rs_eval_to_string)
    fn nvim_eval_tv_get_string(tv: *const c_void) -> *const c_char;
    fn nvim_eval_xstrdup(s: *const c_char) -> *mut c_char;

    // typval2string helpers
    fn nvim_eval_tv_list_join_nl(l: *mut c_void) -> *mut c_char;
    fn nvim_eval_tv_vtype(tv: *const c_void) -> c_int;
    fn nvim_eval_tv_vlist(tv: *const c_void) -> *mut c_void;
    fn nvim_encode_tv2string_wrapper(tv: *mut c_void) -> *mut c_char;

    // eval1_emsg wrapper
    fn nvim_eval1_emsg_wrapper(
        arg: *mut *mut c_char,
        rettv: TypevalHandle,
        eap: ExargHandle,
    ) -> c_int;

    // eval_expr_* helpers
    fn nvim_eval_tv_vpartial(tv: *const c_void) -> *mut c_void; // partial_T*
    fn nvim_eval_tv_vstring_ro(tv: *const c_void) -> *const c_char;
    fn rs_partial_name(pt: *const c_void) -> *mut c_char;
    fn nvim_eval_call_func_partial(
        s: *const c_char,
        partial: *mut c_void,
        argv: TypevalHandle,
        argc: c_int,
        rettv: TypevalHandle,
    ) -> c_int;
    fn nvim_eval_call_func_simple(
        s: *const c_char,
        argv: TypevalHandle,
        argc: c_int,
        rettv: TypevalHandle,
    ) -> c_int;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_semsg_invexpr2(p: *const c_char);
    fn nvim_eap_get_skip_local(eap: ExargHandle) -> c_int;

    // Error globals
    fn aborting() -> c_int;
    fn did_emsg_get() -> c_int;
    fn called_emsg_get() -> c_int;
}

// eval_to_string delegates to rs_eval_to_string (defined here) -- but eval_to_string_safe
// also needs it. We call the C thin wrapper `eval_to_string` which will call us via
// rs_eval_to_string. To avoid circular deps, rs_eval_to_string_safe calls
// rs_eval_to_string_eap directly.

// =============================================================================
// Internal: typval2string
// =============================================================================

/// Convert typval to string.
///
/// When `join_list` is true and tv is a List, joins list items with newlines.
///
/// # Safety
/// `tv` must be a valid pointer to a typval_T.
unsafe fn typval2string_impl(tv: *mut c_void, join_list: bool) -> *mut c_char {
    let vtype = nvim_eval_tv_vtype(tv);
    if join_list && vtype == VAR_LIST {
        let l = nvim_eval_tv_vlist(tv);
        return nvim_eval_tv_list_join_nl(l);
    }
    if vtype == VAR_LIST || vtype == VAR_DICT {
        return nvim_encode_tv2string_wrapper(tv);
    }
    nvim_eval_xstrdup(nvim_eval_tv_get_string(tv))
}

// =============================================================================
// Internal: eval1_emsg
// =============================================================================

/// Call eval1() and give an error message if not done at a lower level.
///
/// # Safety
/// `arg` must be a valid pointer to a mutable C string pointer.
/// `rettv` must be a valid typval handle.
unsafe fn eval1_emsg_impl(arg: *mut *mut c_char, rettv: TypevalHandle, eap: ExargHandle) -> c_int {
    nvim_eval1_emsg_wrapper(arg, rettv, eap)
}

// =============================================================================
// Internal: eval_expr_partial / eval_expr_func / eval_expr_string
// =============================================================================

/// Evaluate a partial expression.
///
/// # Safety
/// `expr` must be a valid pointer to a VAR_PARTIAL typval_T.
unsafe fn eval_expr_partial_impl(
    expr: *const c_void,
    argv: TypevalHandle,
    argc: c_int,
    rettv: TypevalHandle,
) -> c_int {
    let partial = nvim_eval_tv_vpartial(expr);
    if partial.is_null() {
        return FAIL;
    }
    let s = rs_partial_name(partial as *const c_void);
    if s.is_null() || *s == 0 {
        return FAIL;
    }
    nvim_eval_call_func_partial(s, partial, argv, argc, rettv)
}

/// Evaluate a function expression.
///
/// # Safety
/// `expr` must be a valid pointer to a VAR_FUNC or similar typval_T.
unsafe fn eval_expr_func_impl(
    expr: *const c_void,
    argv: TypevalHandle,
    argc: c_int,
    rettv: TypevalHandle,
) -> c_int {
    let mut buf = [0u8; NUMBUFLEN];
    let vtype = nvim_eval_tv_vtype(expr);
    let s = if vtype == VAR_FUNC {
        nvim_eval_tv_vstring_ro(expr)
    } else {
        tv_get_string_buf_chk(
            TypevalHandle::from_ptr(expr as *mut c_void),
            buf.as_mut_ptr() as *mut c_char,
        )
    };
    if s.is_null() || *s == 0 {
        return FAIL;
    }
    nvim_eval_call_func_simple(s, argv, argc, rettv)
}

/// Evaluate a string expression.
///
/// # Safety
/// `expr` must be a valid pointer to a typval_T holding a string.
unsafe fn eval_expr_string_impl(expr: *const c_void, rettv: TypevalHandle) -> c_int {
    let mut buf = [0u8; NUMBUFLEN];
    let raw_s = tv_get_string_buf_chk(
        TypevalHandle::from_ptr(expr as *mut c_void),
        buf.as_mut_ptr() as *mut c_char,
    );
    if raw_s.is_null() {
        return FAIL;
    }
    let mut s = skipwhite(raw_s) as *mut c_char;
    if eval1_emsg_impl(&mut s as *mut *mut c_char, rettv, ExargHandle::null()) == FAIL {
        return FAIL;
    }
    let trail = skipwhite(s);
    if !trail.is_null() && *trail != 0 {
        tv_clear(rettv);
        nvim_semsg_invexpr2(s);
        return FAIL;
    }
    OK
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Top-level bool evaluation.
///
/// Evaluates `arg` as a VimL expression and returns its boolean value.
/// Sets `*error = true` on failure.
///
/// Equivalent to C `eval_to_bool`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string pointer.
/// - `error` must be a valid pointer to bool.
/// - `eap` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_to_bool(
    arg: *mut c_char,
    error: *mut bool,
    eap: ExargHandle,
    skip: bool,
    use_simple_function: bool,
) -> bool {
    let mut tv_storage = [0u64; 8]; // enough space for typval_T
    let tv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);

    let evalarg = nvim_evalarg_alloc_from_eap(eap, skip);
    if skip {
        nvim_emsg_skip_inc();
    }

    let r = if use_simple_function {
        let r_simple = nvim_eval_may_call_simple_func(arg, tv);
        if r_simple == NOTDONE {
            rs_eval0(arg, tv, eap, evalarg)
        } else {
            r_simple
        }
    } else {
        rs_eval0(arg, tv, eap, evalarg)
    };

    let retval;
    if r == FAIL {
        *error = true;
        retval = false;
    } else {
        *error = false;
        if !skip {
            retval = tv_get_number_chk(tv, error) != 0;
            tv_clear(tv);
        } else {
            retval = false;
        }
    }

    if skip {
        nvim_emsg_skip_dec();
    }
    nvim_evalarg_clear_and_free(evalarg, eap);

    retval
}

/// Top-level number evaluation.
///
/// Evaluates `expr` silently and returns the number value, or -1 on error.
///
/// Equivalent to C `eval_to_number`.
///
/// # Safety
/// - `expr` must be a valid null-terminated C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_to_number(expr: *mut c_char, use_simple_function: bool) -> i64 {
    let mut tv_storage = [0u64; 8];
    let tv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);

    let mut p = skipwhite(expr);

    nvim_eval_emsg_off_inc();

    let mut r = NOTDONE;
    if use_simple_function {
        r = nvim_eval_may_call_simple_func(expr, tv);
    }
    if r == NOTDONE {
        let evalarg = nvim_get_evalarg_evaluate_ptr();
        r = rs_eval1(&mut p as *mut *mut c_char, tv, evalarg);
    }

    let retval = if r == FAIL {
        -1i64
    } else {
        let n = tv_get_number_chk(tv, ptr::null_mut());
        tv_clear(tv);
        n
    };

    nvim_eval_emsg_off_dec();
    retval
}

/// Top-level string evaluation (with skip support).
///
/// Equivalent to C `eval_to_string_skip`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string pointer.
/// - `eap` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_to_string_skip(
    arg: *mut c_char,
    eap: ExargHandle,
    skip: bool,
) -> *mut c_char {
    let mut tv_storage = [0u64; 8];
    let tv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);

    let evalarg = nvim_evalarg_alloc_from_eap(eap, skip);
    if skip {
        nvim_emsg_skip_inc();
    }

    let retval = if rs_eval0(arg, tv, eap, evalarg) == FAIL || skip {
        ptr::null_mut()
    } else {
        let s = nvim_eval_tv_get_string(tv.as_ptr() as *const c_void);
        let r = nvim_eval_xstrdup(s);
        tv_clear(tv);
        r
    };

    if skip {
        nvim_emsg_skip_dec();
    }
    nvim_evalarg_clear_and_free(evalarg, eap);

    retval
}

/// Top-level string evaluation with exarg_T and join_list support.
///
/// Equivalent to C `eval_to_string_eap`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string pointer.
/// - `eap` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_to_string_eap(
    arg: *mut c_char,
    join_list: bool,
    eap: ExargHandle,
    use_simple_function: bool,
) -> *mut c_char {
    let mut tv_storage = [0u64; 8];
    let tv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);

    // Determine eap->skip (matches C: eap != NULL && eap->skip)
    let eap_skip = !eap.is_null() && nvim_eap_get_skip_local(eap) != 0;
    let evalarg = nvim_evalarg_alloc_from_eap(eap, eap_skip);

    let r = if use_simple_function {
        let r_simple = nvim_eval_may_call_simple_func(arg, tv);
        if r_simple == NOTDONE {
            rs_eval0(arg, tv, ExargHandle::null(), evalarg)
        } else {
            r_simple
        }
    } else {
        rs_eval0(arg, tv, ExargHandle::null(), evalarg)
    };

    let retval = if r == FAIL {
        ptr::null_mut()
    } else {
        let s = typval2string_impl(tv.as_ptr(), join_list);
        tv_clear(tv);
        s
    };

    nvim_evalarg_clear_and_free(evalarg, ExargHandle::null());
    retval
}

/// Top-level string evaluation.
///
/// Equivalent to C `eval_to_string`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_to_string(
    arg: *mut c_char,
    join_list: bool,
    use_simple_function: bool,
) -> *mut c_char {
    rs_eval_to_string_eap(arg, join_list, ExargHandle::null(), use_simple_function)
}

/// Top-level string evaluation without local variables (sandboxed).
///
/// Equivalent to C `eval_to_string_safe`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_to_string_safe(
    arg: *mut c_char,
    use_sandbox: bool,
    use_simple_function: bool,
) -> *mut c_char {
    let entry = nvim_eval_save_funccal();
    if use_sandbox {
        nvim_eval_sandbox_inc();
    }
    nvim_eval_textlock_inc();

    let retval = rs_eval_to_string(arg, false, use_simple_function);

    if use_sandbox {
        nvim_eval_sandbox_dec();
    }
    nvim_eval_textlock_dec();
    nvim_eval_restore_funccal(entry);

    retval
}

/// Skip over an expression at `*pp`.
///
/// Equivalent to C `skip_expr`.
///
/// # Safety
/// - `pp` must be a valid pointer to a mutable C string pointer.
/// - `evalarg` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_expr(pp: *mut *mut c_char, evalarg: EvalargHandle) -> c_int {
    let save_flags = if evalarg.is_null() {
        0
    } else {
        evalarg_get_flags(evalarg)
    };

    // Don't evaluate the expression.
    if !evalarg.is_null() {
        let flags = evalarg_get_flags(evalarg);
        // Clear EVAL_EVALUATE flag (bit 0)
        evalarg_set_flags(evalarg, flags & !1);
    }

    *pp = skipwhite(*pp);
    let mut tv_storage = [0u64; 8];
    let rettv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);
    let res = rs_eval1(pp, rettv, EvalargHandle::null());

    if !evalarg.is_null() {
        evalarg_set_flags(evalarg, save_flags);
    }

    res
}

/// Call eval1() and give an error message if not done at a lower level.
///
/// Equivalent to C `eval1_emsg`. Real implementation (no longer delegates to C).
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer.
/// - `rettv` must be a valid typval handle.
/// - `eap` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval1_emsg(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    eap: ExargHandle,
) -> c_int {
    let start = *arg;
    let did_emsg_before = did_emsg_get();
    let called_emsg_before = called_emsg_get();

    let skip = !eap.is_null() && nvim_eap_get_skip_local(eap) != 0;
    let evalarg = nvim_evalarg_alloc_from_eap(eap, skip);

    let ret = rs_eval1(arg, rettv, evalarg);
    if ret == FAIL
        && aborting() == 0
        && did_emsg_get() == did_emsg_before
        && called_emsg_get() == called_emsg_before
    {
        nvim_semsg_invexpr2(start);
    }

    nvim_evalarg_clear_and_free(evalarg, eap);
    ret
}

/// Evaluate an expression which can be a function, partial or string.
///
/// Equivalent to C `eval_expr_typval`.
///
/// # Safety
/// - `expr` must be a valid pointer to a typval_T.
/// - `argv` must be a valid typval handle or null (argc=0).
/// - `rettv` must be a valid typval handle.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_expr_typval(
    expr: *const c_void,
    want_func: bool,
    argv: TypevalHandle,
    argc: c_int,
    rettv: TypevalHandle,
) -> c_int {
    let vtype = nvim_eval_tv_vtype(expr);
    if vtype == VAR_PARTIAL {
        return eval_expr_partial_impl(expr, argv, argc, rettv);
    }
    if vtype == VAR_FUNC || want_func {
        return eval_expr_func_impl(expr, argv, argc, rettv);
    }
    eval_expr_string_impl(expr, rettv)
}

/// Like eval_to_bool() but using a typval_T instead of a string.
///
/// Equivalent to C `eval_expr_to_bool`.
///
/// # Safety
/// - `expr` must be a valid pointer to a typval_T.
/// - `error` must be a valid pointer to bool.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_expr_to_bool(expr: *const c_void, error: *mut bool) -> bool {
    let mut argv_storage = [0u64; 8];
    let argv = TypevalHandle::from_ptr(argv_storage.as_mut_ptr() as *mut c_void);
    let mut rettv_storage = [0u64; 8];
    let rettv = TypevalHandle::from_ptr(rettv_storage.as_mut_ptr() as *mut c_void);

    if rs_eval_expr_typval(expr, false, argv, 0, rettv) == FAIL {
        *error = true;
        return false;
    }
    let res = tv_get_number_chk(rettv, error) != 0;
    tv_clear(rettv);
    res
}
