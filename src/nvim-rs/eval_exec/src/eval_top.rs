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

    // Phase 5: fill_evalarg_from_eap / clear_evalarg accessors
    fn nvim_evalarg_init_skip(evalarg: EvalargHandle, skip: bool);
    fn nvim_sourcing_a_script(eap: ExargHandle) -> bool;
    fn nvim_evalarg_copy_getline_from_eap(evalarg: EvalargHandle, eap: ExargHandle);
    fn nvim_evalarg_get_tofree(evalarg: EvalargHandle) -> *mut c_char;
    fn nvim_evalarg_set_tofree(evalarg: EvalargHandle, val: *mut c_char);
    fn nvim_eap_get_cmdline_tofree(eap: ExargHandle) -> *mut c_char;
    fn nvim_eap_set_cmdline_tofree(eap: ExargHandle, val: *mut c_char);
    fn nvim_eap_get_cmdlinep_deref(eap: ExargHandle) -> *mut c_char;
    fn nvim_eap_set_cmdlinep_deref(eap: ExargHandle, val: *mut c_char);
    fn xfree(ptr: *mut c_void);

    // Phase 5: may_call_simple_func / eval_expr_ext accessors
    fn nvim_call_simple_luafunc(name: *const c_char, len: usize, rettv: TypevalHandle) -> c_int;
    fn nvim_call_simple_func(name: *const c_char, len: usize, rettv: TypevalHandle) -> c_int;
    // These are already Rust exports (rs_skip_luafunc_name, rs_to_name_end) called via C ABI
    fn rs_skip_luafunc_name(p: *const c_char) -> *const c_char;
    fn rs_to_name_end(p: *const c_char, use_namespace: bool) -> *const c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn skipdigits(p: *const c_char) -> *mut c_char;
    /// Allocate exactly sizeof(typval_T) bytes, zeroed, for a heap typval.
    fn nvim_alloc_typval() -> *mut c_void;
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

// =============================================================================
// Phase 5 (eval_shim pass 4): fill_evalarg_from_eap, clear_evalarg,
//   may_call_simple_func, eval0_simple_funccal, eval_expr_ext
// =============================================================================

/// Initialize evalarg_T from exarg_T.
///
/// Equivalent to C `fill_evalarg_from_eap`.
///
/// # Safety
/// - `evalarg` must be a valid pointer to an evalarg_T (writable).
/// - `eap` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_fill_evalarg_from_eap(
    evalarg: EvalargHandle,
    eap: ExargHandle,
    skip: bool,
) {
    // Zero-init and set eval_flags based on skip.
    nvim_evalarg_init_skip(evalarg, skip);
    if eap.is_null() {
        return;
    }
    if nvim_sourcing_a_script(eap) {
        // Copy the getline function pointer and cookie from eap.
        nvim_evalarg_copy_getline_from_eap(evalarg, eap);
    }
}

/// Free evalarg resources and potentially update eap cmdline ownership.
///
/// Equivalent to C `clear_evalarg`.
///
/// # Safety
/// - `evalarg` may be null (no-op).
/// - `eap` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_evalarg(evalarg: EvalargHandle, eap: ExargHandle) {
    if evalarg.is_null() {
        return;
    }
    let tofree = nvim_evalarg_get_tofree(evalarg);
    if tofree.is_null() {
        return;
    }
    if !eap.is_null() {
        // Keep both the old and new cmdline; nextcmd may point into the new one.
        let old_cmdline_tofree = nvim_eap_get_cmdline_tofree(eap);
        xfree(old_cmdline_tofree as *mut c_void);
        let current_cmdlinep = nvim_eap_get_cmdlinep_deref(eap);
        nvim_eap_set_cmdline_tofree(eap, current_cmdlinep);
        nvim_eap_set_cmdlinep_deref(eap, tofree);
    } else {
        xfree(tofree as *mut c_void);
    }
    nvim_evalarg_set_tofree(evalarg, std::ptr::null_mut());
}

/// Optimization: if arg is "FuncName()" with no other args, call it directly.
///
/// Returns NOTDONE if the optimization doesn't apply, OK or FAIL otherwise.
///
/// Equivalent to C `may_call_simple_func`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string.
/// - `rettv` must be a valid typval handle.
#[no_mangle]
pub unsafe extern "C" fn rs_may_call_simple_func(
    arg: *const c_char,
    rettv: TypevalHandle,
) -> c_int {
    // Look for "()" in the argument string.
    let parens_needle = b"()\0";
    let parens = strstr(arg, parens_needle.as_ptr() as *const c_char);
    if parens.is_null() {
        return NOTDONE;
    }

    // After "()" there should only be whitespace then NUL.
    let after = skipwhite(parens.add(2));
    if *after != 0 {
        return NOTDONE;
    }

    // Check for "v:lua.FuncName()"
    let vlua_prefix = b"v:lua.\0";
    if strncmp(arg, vlua_prefix.as_ptr() as *const c_char, 6) == 0 {
        let p = arg.add(6);
        if !std::ptr::eq(p, parens) {
            let name_end = rs_skip_luafunc_name(p);
            if std::ptr::eq(name_end, parens) {
                return nvim_call_simple_luafunc(p, parens.offset_from(p) as usize, rettv);
            }
        }
        return NOTDONE;
    }

    // Check for "<SNR>NNN_FuncName()" or plain "FuncName()"
    let snr_prefix = b"<SNR>\0";
    let p: *const c_char = if strncmp(arg, snr_prefix.as_ptr() as *const c_char, 5) == 0 {
        skipdigits(arg.add(5))
    } else {
        arg
    };
    let name_end = rs_to_name_end(p, true);
    if std::ptr::eq(name_end, parens as *const c_char) {
        return nvim_call_simple_func(arg, parens.offset_from(arg) as usize, rettv);
    }

    NOTDONE
}

/// Handle zero-level expression with optimization for a simple function call.
///
/// Equivalent to C `eval0_simple_funccal` (static -- not exported, used internally).
///
/// # Safety
/// All pointers must be valid per eval0 contract.
unsafe fn eval0_simple_funccal_impl(
    arg: *mut c_char,
    rettv: TypevalHandle,
    eap: ExargHandle,
    evalarg: EvalargHandle,
) -> c_int {
    let r = rs_may_call_simple_func(arg, rettv);
    if r == NOTDONE {
        rs_eval0(arg, rettv, eap, evalarg)
    } else {
        r
    }
}

/// Allocate a typval_T, evaluate arg into it, and return it.
///
/// Returns NULL if evaluation fails (FAIL result from eval0).
///
/// Equivalent to C `eval_expr_ext`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string.
/// - `eap` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_expr_ext(
    arg: *mut c_char,
    eap: ExargHandle,
    use_simple_function: bool,
) -> *mut c_void {
    let tv = nvim_alloc_typval();
    let tv_handle = TypevalHandle::from_ptr(tv);

    let eap_skip = !eap.is_null() && nvim_eap_get_skip_local(eap) != 0;

    let evalarg = nvim_evalarg_alloc_from_eap(eap, eap_skip);

    let r = if use_simple_function {
        eval0_simple_funccal_impl(arg, tv_handle, eap, evalarg)
    } else {
        rs_eval0(arg, tv_handle, eap, evalarg)
    };

    if r == FAIL {
        xfree(tv);
        nvim_evalarg_clear_and_free(evalarg, eap);
        return std::ptr::null_mut();
    }

    nvim_evalarg_clear_and_free(evalarg, eap);
    tv
}

// =============================================================================
// Phase 1 (eval_shim pass 5): call_vim_function family + small utilities
// =============================================================================

extern "C" {
    // Phase 1: call_vim_function accessors
    fn nvim_call_func_with_partial(
        func: *const c_char,
        len: c_int,
        rettv: TypevalHandle,
        argc: c_int,
        argv: TypevalHandle,
        partial: *mut c_void,
    ) -> c_int;
    fn nvim_get_vv_lua_partial_p1() -> *mut c_void; // partial_T*
    fn rs_check_luafunc_name(s: *const c_char, paren: bool) -> c_int;

    // Phase 1: set_argv_var accessors
    fn nvim_eval_tv_list_alloc(len: isize) -> *mut c_void; // list_T*
    fn nvim_tv_list_set_lock(l: *mut c_void, lock: c_int);
    fn nvim_tv_list_append_string(l: *mut c_void, s: *const c_char, len: isize);
    fn nvim_tv_list_last_fix_lock(l: *mut c_void);
    fn nvim_set_vim_var_argv_list(list: *mut c_void);

    // Phase 1: var_set_global accessors
    fn nvim_set_var_wrapper(name: *const c_char, name_len: usize, tv: TypevalHandle);

    // Phase 1: eval_fmt_source_name_line accessors
    fn nvim_sourcing_name_get() -> *const c_char;
    fn nvim_sourcing_lnum_get() -> i64; // linenr_T
    fn nvim_snprintf_source_line(buf: *mut c_char, bufsize: usize, name: *const c_char, lnum: i64);
    fn nvim_snprintf_question(buf: *mut c_char, bufsize: usize);

    // Phase 1: find_option_var_end accessors
    fn nvim_find_option_end_wrapper(p: *const c_char, opt_idxp: *mut c_int) -> *const c_char;

    // Phase 1: call_func_retstr helper
    fn nvim_shim_tv_get_string(tv: TypevalHandle) -> *const c_char;
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_tv_set_type(tv: TypevalHandle, vtype: c_int);
}

// VAR_FIXED lock constant (from typval_defs.h: VAR_FIXED = 2)
const VAR_FIXED: c_int = 2;

/// Call a VimL function by name and place the result in `rettv`.
///
/// Equivalent to C `call_vim_function`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_call_vim_function(
    func: *const c_char,
    argc: c_int,
    argv: TypevalHandle,
    rettv: TypevalHandle,
) -> c_int {
    let len = libc_strlen(func) as c_int;

    // Check for "v:lua.FuncName" prefix
    let vlua_prefix = b"v:lua.\0";
    let mut actual_func = func;
    let mut actual_len = len;
    let mut partial: *mut c_void = std::ptr::null_mut();

    if len >= 6 && strncmp(func, vlua_prefix.as_ptr() as *const c_char, 6) == 0 {
        let lua_name = func.add(6);
        let lua_len = rs_check_luafunc_name(lua_name, false);
        if lua_len == 0 {
            tv_clear(rettv);
            return FAIL;
        }
        actual_func = lua_name;
        actual_len = lua_len;
        partial = nvim_get_vv_lua_partial_p1();
    }

    // Initialize rettv: set v_type = VAR_UNKNOWN (0) so tv_clear works on failure
    nvim_tv_set_type(rettv, 0); // VAR_UNKNOWN = 0

    let ret = nvim_call_func_with_partial(actual_func, actual_len, rettv, argc, argv, partial);

    if ret == FAIL {
        tv_clear(rettv);
    }
    ret
}

/// Call a VimL function and return the result as an allocated string.
///
/// Equivalent to C `call_func_retstr`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_call_func_retstr(
    func: *const c_char,
    argc: c_int,
    argv: TypevalHandle,
) -> *mut c_char {
    let mut tv_storage = [0u64; 8];
    let rettv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);

    if rs_call_vim_function(func, argc, argv, rettv) == FAIL {
        return std::ptr::null_mut();
    }

    let s = nvim_shim_tv_get_string(rettv);
    let result = nvim_xstrdup(s);
    tv_clear(rettv);
    result
}

/// Call a VimL function and return the result as a list_T.
///
/// Equivalent to C `call_func_retlist`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_call_func_retlist(
    func: *const c_char,
    argc: c_int,
    argv: TypevalHandle,
) -> *mut c_void {
    let mut tv_storage = [0u64; 8];
    let rettv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);

    if rs_call_vim_function(func, argc, argv, rettv) == FAIL {
        return std::ptr::null_mut();
    }

    let vtype = nvim_eval_tv_vtype(rettv.as_ptr() as *const c_void);
    if vtype != VAR_LIST {
        tv_clear(rettv);
        return std::ptr::null_mut();
    }

    nvim_eval_tv_vlist(rettv.as_ptr() as *const c_void)
}

/// Set the v:argv list from argc/argv.
///
/// Equivalent to C `set_argv_var`.
///
/// # Safety
/// `argv` must be an array of `argc` valid C string pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_set_argv_var(argv: *mut *mut c_char, argc: c_int) {
    let l = nvim_eval_tv_list_alloc(argc as isize);
    nvim_tv_list_set_lock(l, VAR_FIXED);
    for i in 0..argc {
        let s = *argv.add(i as usize);
        nvim_tv_list_append_string(l, s, -1);
        nvim_tv_list_last_fix_lock(l);
    }
    nvim_set_vim_var_argv_list(l);
}

/// Set a global variable via save_funccal/set_var/restore_funccal.
///
/// Equivalent to C `var_set_global`.
///
/// # Safety
/// `name` must be a valid C string. `vartv` must be a valid typval_T.
#[no_mangle]
pub unsafe extern "C" fn rs_var_set_global(name: *const c_char, vartv: TypevalHandle) {
    let entry = nvim_eval_save_funccal();
    let name_len = libc_strlen(name);
    nvim_set_var_wrapper(name, name_len, vartv);
    nvim_eval_restore_funccal(entry);
}

/// Write "<sourcing_name>:<sourcing_lnum>" to buf[bufsize].
///
/// Equivalent to C `eval_fmt_source_name_line`.
///
/// # Safety
/// `buf` must be a valid writable buffer of at least `bufsize` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_fmt_source_name_line(buf: *mut c_char, bufsize: usize) {
    let name = nvim_sourcing_name_get();
    if !name.is_null() {
        let lnum = nvim_sourcing_lnum_get();
        nvim_snprintf_source_line(buf, bufsize, name, lnum);
    } else {
        nvim_snprintf_question(buf, bufsize);
    }
}

/// Skip over the name of an option variable: "&option", "&g:option" or "&l:option".
///
/// Equivalent to C `find_option_var_end`.
///
/// # Safety
/// `arg` must be a valid pointer to a mutable C string pointer.
/// `opt_idxp` and `opt_flags` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_find_option_var_end(
    arg: *mut *const c_char,
    opt_idxp: *mut c_int,
    opt_flags: *mut c_int,
) -> *const c_char {
    let mut p = *arg;

    // Skip past the leading '&' or '+'
    p = p.add(1);

    // Check for g: or l: scope prefix
    if *p == b'g' as c_char && *p.add(1) == b':' as c_char {
        *opt_flags = 1; // OPT_GLOBAL = 0x01
        p = p.add(2);
    } else if *p == b'l' as c_char && *p.add(1) == b':' as c_char {
        *opt_flags = 2; // OPT_LOCAL = 0x02
        p = p.add(2);
    } else {
        *opt_flags = 0;
    }

    let end = nvim_find_option_end_wrapper(p, opt_idxp);
    if end.is_null() {
        // Leave *arg unchanged on failure
    } else {
        *arg = p;
    }
    end
}

// Helper: compute strlen of a C string without linking libc explicitly.
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

// =============================================================================
// Phase 2 (eval_shim pass 5): prompt functions
// =============================================================================

extern "C" {
    // Phase 2: prompt_get_input accessors
    fn nvim_mark_bt_prompt(buf: *mut c_void) -> c_int; // bt_prompt(buf)
    fn nvim_buf_get_prompt_start_lnum(buf: *mut c_void) -> i32; // linenr_T = i32
    fn nvim_buf_get_b_ml_ml_line_count(buf: *mut c_void) -> i32; // linenr_T = i32
    fn nvim_ml_get_buf_wrapper(buf: *mut c_void, lnum: i32) -> *mut c_char;
    fn nvim_prompt_text() -> *const c_char;
    fn nvim_concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;
    // nvim_xstrdup declared in Phase 1 extern block above

    // Phase 2: prompt_invoke_callback accessors
    fn nvim_get_curbuf_ptr() -> *mut c_void; // buf_T*
    fn nvim_curbuf_get_ml_line_count_lnr() -> i32; // linenr_T = i32
    fn nvim_ml_append(lnum: i32, line: *const c_char, len: i32, newfile: bool) -> c_int;
    fn nvim_appended_lines_mark(lnum: i32, count: c_int);
    fn nvim_set_cursor_lnum(lnum: i32); // linenr_T
    fn nvim_set_cursor_col(col: c_int);
    fn nvim_curbuf_set_prompt_start_lnum(lnum: i32);
    fn nvim_eval_cb_get_type(cb: *const c_void) -> c_int;
    fn nvim_curbuf_get_prompt_callback() -> *mut c_void; // Callback*
    fn nvim_curbuf_prompt_callback_call(user_input: *mut c_char) -> bool;
    fn nvim_curbuf_u_clearallandblockfree();

    // Phase 2: invoke_prompt_interrupt accessors
    fn nvim_curbuf_get_prompt_interrupt() -> *mut c_void; // Callback*
    fn nvim_excmds_clear_got_int();
    fn nvim_curbuf_prompt_interrupt_call() -> c_int;
}

// kCallbackNone = 0
const K_CALLBACK_NONE: c_int = 0;

/// Get the current user-input text from a prompt buffer.
///
/// Returns NULL if `buf` is not a prompt buffer.
/// Returns an allocated string that the caller must free.
///
/// Equivalent to C `prompt_get_input`.
///
/// # Safety
/// `buf` must be a valid `buf_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_prompt_get_input(buf: *mut c_void) -> *mut c_char {
    if nvim_mark_bt_prompt(buf) == 0 {
        return std::ptr::null_mut();
    }

    let lnum_start: i32 = nvim_buf_get_prompt_start_lnum(buf);
    let lnum_last: i32 = nvim_buf_get_b_ml_ml_line_count(buf);

    // Get the first line and skip past the prompt prefix
    let text_raw = nvim_ml_get_buf_wrapper(buf, lnum_start);
    let prompt = nvim_prompt_text();
    let prompt_len = libc_strlen(prompt);
    let text_len = libc_strlen(text_raw);
    let text = if text_len >= prompt_len {
        text_raw.add(prompt_len)
    } else {
        text_raw
    };

    let mut full_text = nvim_xstrdup(text);

    // Append subsequent lines with "\n" separator
    let mut i: i32 = lnum_start + 1;
    while i <= lnum_last {
        let newline = b"\n\0";
        let half_text = nvim_concat_str(full_text, newline.as_ptr() as *const c_char);
        xfree(full_text as *mut c_void);
        let line = nvim_ml_get_buf_wrapper(buf, i);
        full_text = nvim_concat_str(half_text, line);
        xfree(half_text as *mut c_void);
        i += 1;
    }

    full_text
}

/// Invoke the user-defined callback for the current prompt buffer.
///
/// Equivalent to C `prompt_invoke_callback`.
///
/// # Safety
/// Uses global `curbuf` and `curwin`. Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_prompt_invoke_callback() {
    let curbuf = nvim_get_curbuf_ptr();
    let lnum: i32 = nvim_curbuf_get_ml_line_count_lnr();

    let user_input = rs_prompt_get_input(curbuf);
    if user_input.is_null() {
        return;
    }

    // Add a new line for the prompt before invoking callback
    let empty = b"\0";
    nvim_ml_append(lnum, empty.as_ptr() as *const c_char, 0, false);
    nvim_appended_lines_mark(lnum, 1);
    nvim_set_cursor_lnum(lnum + 1);
    nvim_set_cursor_col(0);
    nvim_curbuf_set_prompt_start_lnum(lnum + 1);

    let callback = nvim_curbuf_get_prompt_callback();
    let cb_type = nvim_eval_cb_get_type(callback);
    if cb_type == K_CALLBACK_NONE {
        xfree(user_input as *mut c_void);
    } else {
        // user_input ownership transferred to callback (freed by tv_clear)
        nvim_curbuf_prompt_callback_call(user_input);
    }

    // clear undo history on submit
    nvim_curbuf_u_clearallandblockfree();
    let new_lnum: i32 = nvim_curbuf_get_ml_line_count_lnr();
    nvim_curbuf_set_prompt_start_lnum(new_lnum);
}

/// Invoke the prompt interrupt callback.
///
/// Equivalent to C `invoke_prompt_interrupt`.
///
/// # Safety
/// Uses global `curbuf`. Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_invoke_prompt_interrupt() -> bool {
    let callback = nvim_curbuf_get_prompt_interrupt();
    let cb_type = nvim_eval_cb_get_type(callback);
    if cb_type == K_CALLBACK_NONE {
        return false;
    }

    nvim_excmds_clear_got_int(); // got_int = false
    let ret = nvim_curbuf_prompt_interrupt_call();
    ret != FAIL
}

// =============================================================================
// Phase 3 (eval_shim pass 5): eval_foldexpr and eval_foldtext
// =============================================================================

extern "C" {
    // Phase 3: foldexpr accessors
    fn nvim_win_was_set_insecurely_foldexpr(wp: *mut c_void) -> bool;
    fn nvim_win_was_set_insecurely_foldtext(wp: *mut c_void) -> bool;
    fn nvim_win_get_foldexpr(wp: *mut c_void) -> *mut c_char;
    fn nvim_win_get_foldtext(wp: *mut c_void) -> *mut c_char;
    fn nvim_win_set_current_sctx_foldexpr(wp: *mut c_void);
    fn nvim_save_current_sctx() -> *mut c_void; // returns sctx_T*
    fn nvim_restore_current_sctx(saved: *mut c_void); // frees saved sctx_T*

    // Phase 3: typval field accessors
    fn nvim_eval_tv_get_vnumber(tv: *const c_void) -> i64;
    fn nvim_eval_tv_get_vstring(tv: TypevalHandle) -> *mut c_char;

    // Phase 3: foldtext Object construction helpers
    fn nvim_foldtext_make_nil_obj(out: *mut c_void);
    fn nvim_foldtext_make_string_obj(tv: TypevalHandle, out: *mut c_void);
    fn nvim_foldtext_make_array_obj(tv: TypevalHandle, out: *mut c_void);
}

// typval type constants for fold functions
const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;

/// Evaluate 'foldexpr' for a window. Returns the fold level; sets *cp to any
/// prefix character (e.g., '>' or '<').
///
/// Equivalent to C `eval_foldexpr`.
///
/// # Safety
/// - `wp` must be a valid win_T pointer.
/// - `cp` must be a valid writable int pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_foldexpr(wp: *mut c_void, cp: *mut c_int) -> c_int {
    let saved_sctx = nvim_save_current_sctx();
    let use_sandbox = nvim_win_was_set_insecurely_foldexpr(wp);

    let arg = nvim_win_get_foldexpr(wp);
    nvim_win_set_current_sctx_foldexpr(wp);

    nvim_eval_emsg_off_inc();
    if use_sandbox {
        nvim_eval_sandbox_inc();
    }
    nvim_eval_textlock_inc();
    *cp = 0; // NUL

    let tv = nvim_alloc_typval();
    let tv_handle = TypevalHandle::from_ptr(tv);

    let evalarg = nvim_get_evalarg_evaluate_ptr();
    let retval: i64 = if eval0_simple_funccal_impl(arg, tv_handle, ExargHandle::null(), evalarg)
        == FAIL
    {
        0
    } else {
        let vtype = nvim_eval_tv_vtype(tv);
        let result = if vtype == VAR_NUMBER {
            nvim_eval_tv_get_vnumber(tv)
        } else if vtype != VAR_STRING {
            0
        } else {
            let s = nvim_eval_tv_get_vstring(TypevalHandle::from_ptr(tv));
            if s.is_null() {
                0
            } else {
                // If string starts with non-digit, non-minus: that char is the prefix
                let first = *s as u8;
                let s_num = if first != 0
                    && !(first >= b'0' && first <= b'9')
                    && first != b'-'
                {
                    *cp = first as c_int;
                    s.add(1)
                } else {
                    s
                };
                // atol equivalent: parse decimal integer from s_num
                let mut n: i64 = 0;
                let mut neg = false;
                let mut p = s_num;
                if *p == b'-' as i8 {
                    neg = true;
                    p = p.add(1);
                }
                while *p != 0 {
                    let c = *p as u8;
                    if c >= b'0' && c <= b'9' {
                        n = n * 10 + (c - b'0') as i64;
                        p = p.add(1);
                    } else {
                        break;
                    }
                }
                if neg { -n } else { n }
            }
        };
        tv_clear(tv_handle);
        result
    };

    nvim_eval_emsg_off_dec();
    if use_sandbox {
        nvim_eval_sandbox_dec();
    }
    nvim_eval_textlock_dec();
    rs_clear_evalarg(evalarg, ExargHandle::null());
    nvim_restore_current_sctx(saved_sctx);
    xfree(tv);

    retval as c_int
}

/// Evaluate 'foldtext' for a window. Writes result into *out (an Object).
///
/// Equivalent to C `eval_foldtext`.
///
/// # Safety
/// - `wp` must be a valid win_T pointer.
/// - `out` must be a valid pointer to an Object (at least sizeof(Object) bytes).
#[no_mangle]
pub unsafe extern "C" fn rs_eval_foldtext(wp: *mut c_void, out: *mut c_void) {
    let use_sandbox = nvim_win_was_set_insecurely_foldtext(wp);
    let arg = nvim_win_get_foldtext(wp);

    let funccal = nvim_eval_save_funccal();
    if use_sandbox {
        nvim_eval_sandbox_inc();
    }
    nvim_eval_textlock_inc();

    let tv = nvim_alloc_typval();
    let tv_handle = TypevalHandle::from_ptr(tv);

    let evalarg = nvim_get_evalarg_evaluate_ptr();
    if eval0_simple_funccal_impl(arg, tv_handle, ExargHandle::null(), evalarg) == FAIL {
        nvim_foldtext_make_nil_obj(out);
    } else {
        let vtype = nvim_eval_tv_vtype(tv);
        if vtype == VAR_LIST {
            nvim_foldtext_make_array_obj(tv_handle, out);
        } else {
            nvim_foldtext_make_string_obj(tv_handle, out);
        }
        tv_clear(tv_handle);
    }

    rs_clear_evalarg(evalarg, ExargHandle::null());

    if use_sandbox {
        nvim_eval_sandbox_dec();
    }
    nvim_eval_textlock_dec();
    nvim_eval_restore_funccal(funccal);
    xfree(tv);
}
