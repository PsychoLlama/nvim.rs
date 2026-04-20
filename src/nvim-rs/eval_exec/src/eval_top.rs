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

use nvim_collections::garray::GArray;
use nvim_eval::typval::TypvalT as TypvalTRepr;

use crate::callback::CallbackT;
use crate::eval::{EvalargHandle, EvalargT, ExargHandle, LineGetter, TypevalHandle};
use crate::funcexe::FuncExeT;

/// Return type of rs_find_option_end (mirrors C struct in option crate).
#[repr(C)]
struct FindOptionEndResult {
    end: *const c_char,
    opt_idx: c_int,
}

// =============================================================================
// funccal_entry_T layout (Phase 4)
// =============================================================================

/// Rust mirror of C `funccal_entry_T`.
#[repr(C)]
struct FunccalEntryT {
    top_funccal: *mut c_void,
    next: *mut c_void,
}

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
    fn eval0(
        arg: *mut c_char,
        rettv: TypevalHandle,
        eap: ExargHandle,
        evalarg: EvalargHandle,
    ) -> c_int;
    fn eval1(arg: *mut *mut c_char, rettv: TypevalHandle, evalarg: EvalargHandle) -> c_int;

    // Evalarg lifecycle
    fn nvim_get_evalarg_evaluate_ptr() -> EvalargHandle;

    // Phase 12: emsg_skip accessed directly as a global
    static mut emsg_skip: c_int;

    // Phase 16: emsg_off, sandbox, textlock accessed directly as globals
    static mut emsg_off: c_int;
    static mut sandbox: c_int;
    static mut textlock: c_int;

    // funccal save/restore
    #[link_name = "save_funccal"]
    fn nvim_save_funccal_inner(entry: *mut c_void);
    #[link_name = "restore_funccal"]
    fn nvim_restore_funccal_inner();
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);

    // may_call_simple_func
    #[link_name = "may_call_simple_func"]
    fn nvim_eval_may_call_simple_func(arg: *const c_char, rettv: TypevalHandle) -> c_int;

    // typval operations
    fn tv_clear(tv: TypevalHandle);
    fn tv_get_number_chk(tv: TypevalHandle, error: *mut bool) -> i64;
    fn tv_get_string_buf_chk(tv: TypevalHandle, buf: *mut c_char) -> *const c_char;

    // eval_to_string_safe wrapper (calls through to our rs_eval_to_string)
    #[link_name = "tv_get_string"]
    fn nvim_eval_tv_get_str(tv: TypevalHandle) -> *const c_char;
    #[link_name = "xstrdup"]
    fn nvim_eval_xstrdup(s: *const c_char) -> *mut c_char;

    // typval2string helpers
    #[link_name = "encode_tv2string"]
    fn nvim_encode_tv2string_wrapper(tv: *mut c_void, len: *mut usize) -> *mut c_char;
    // ga helpers for tv_list_join_nl inlining
    fn tv_list_join(ga: *mut GArray, l: *mut c_void, sep: *const c_char);
    fn nvim_tv_list_len(l: *const c_void) -> c_int;

    // eval_expr_* helpers
    fn rs_partial_name(pt: *const c_void) -> *mut c_char;
    // Direct call_func for eval_expr_partial/eval_expr_func/call_vim_function
    fn call_func(
        funcname: *const c_char,
        len: c_int,
        rettv: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        funcexe: *mut FuncExeT,
    ) -> c_int;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_eap_get_skip_local(eap: ExargHandle) -> c_int;

    // Error globals
    fn aborting() -> c_int;
    static did_emsg: c_int;
    static called_emsg: c_int;

    // Phase 5: fill_evalarg_from_eap / clear_evalarg accessors
    fn sourcing_a_script(eap: ExargHandle) -> c_int;
    fn nvim_eap_get_getline(eap: ExargHandle) -> LineGetter;
    fn nvim_eap_get_cookie(eap: ExargHandle) -> *mut c_void;
    fn nvim_eap_get_cmdline_tofree(eap: ExargHandle) -> *mut c_char;
    fn nvim_eap_set_cmdline_tofree(eap: ExargHandle, val: *mut c_char);
    fn nvim_eap_get_cmdlinep_deref(eap: ExargHandle) -> *mut c_char;
    fn nvim_eap_set_cmdlinep_deref(eap: ExargHandle, val: *mut c_char);

    // Phase 5: may_call_simple_func / eval_expr_ext accessors
    #[link_name = "call_simple_luafunc"]
    fn nvim_call_simple_luafunc(name: *const c_char, len: usize, rettv: TypevalHandle) -> c_int;
    #[link_name = "call_simple_func"]
    fn nvim_call_simple_func(name: *const c_char, len: usize, rettv: TypevalHandle) -> c_int;
    // These are already Rust exports (rs_skip_luafunc_name, rs_to_name_end) called via C ABI
    fn rs_skip_luafunc_name(p: *const c_char) -> *const c_char;
    fn rs_to_name_end(p: *const c_char, use_namespace: bool) -> *const c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn skipdigits(p: *const c_char) -> *mut c_char;
    fn xmalloc(size: usize) -> *mut c_void;
}

// eval_to_string delegates to rs_eval_to_string (defined here) -- but eval_to_string_safe
// also needs it. We call the C thin wrapper `eval_to_string` which will call us via
// rs_eval_to_string. To avoid circular deps, rs_eval_to_string_safe calls
// rs_eval_to_string_eap directly.

// =============================================================================
// Internal: evalarg allocation helpers
// =============================================================================

/// Allocate a heap `EvalargT`, initialize via `fill_evalarg_from_eap`, and return a handle.
///
/// Caller must call `free_evalarg(evalarg, eap)` when done.
///
/// # Safety
/// All C contracts for `fill_evalarg_from_eap` apply.
unsafe fn alloc_evalarg(eap: ExargHandle, skip: bool) -> EvalargHandle {
    let mut ea = Box::new(EvalargT::new_skip());
    fill_evalarg_from_eap(EvalargHandle(ea.as_mut()), eap, skip);
    EvalargHandle(Box::into_raw(ea))
}

/// Clear evalarg resources and free the heap `EvalargT`.
///
/// # Safety
/// `evalarg` must have been returned by `alloc_evalarg`.
unsafe fn free_evalarg(evalarg: EvalargHandle, eap: ExargHandle) {
    clear_evalarg(evalarg, eap);
    if !evalarg.is_null() {
        drop(Box::from_raw(evalarg.as_ptr()));
    }
}

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
    let tv_h = TypevalHandle::from_ptr(tv);
    let vtype = (*tv_h.as_ptr().cast::<TypvalTRepr>()).v_type;
    if join_list && vtype == VAR_LIST {
        let l = (*tv.cast::<TypvalTRepr>()).vval.v_list;
        // Inlined nvim_eval_tv_list_join_nl: ga_init + tv_list_join + rs_ga_append(NUL)
        let mut ga = GArray::default();
        nvim_collections::garray::rs_ga_init(&mut ga, 1, 80);
        if !l.is_null() {
            tv_list_join(&mut ga, l, c"\n".as_ptr());
            if nvim_tv_list_len(l) > 0 {
                nvim_collections::garray::rs_ga_append(&mut ga, b'\n');
            }
        }
        nvim_collections::garray::rs_ga_append(&mut ga, 0);
        return ga.ga_data.cast::<c_char>();
    }
    if vtype == VAR_LIST || vtype == VAR_DICT {
        return nvim_encode_tv2string_wrapper(tv, std::ptr::null_mut());
    }
    nvim_eval_xstrdup(nvim_eval_tv_get_str(tv_h))
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
    rs_eval1_emsg(arg, rettv, eap)
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
    let partial = (*expr.cast::<TypvalTRepr>()).vval.v_partial;
    if partial.is_null() {
        return FAIL;
    }
    let s = rs_partial_name(partial as *const c_void);
    if s.is_null() || *s == 0 {
        return FAIL;
    }
    let mut funcexe = FuncExeT::new();
    funcexe.fe_evaluate = true;
    funcexe.fe_partial = partial;
    call_func(s, -1, rettv.as_ptr(), argc, argv.as_ptr(), &mut funcexe)
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
    let expr_h = TypevalHandle::from_ptr(expr as *mut c_void);
    let vtype = (*expr_h.as_ptr().cast::<TypvalTRepr>()).v_type;
    let s: *const c_char = if vtype == VAR_FUNC {
        expr_h.get_vstring() as *const c_char
    } else {
        tv_get_string_buf_chk(expr_h, buf.as_mut_ptr() as *mut c_char)
    };
    if s.is_null() || *s == 0 {
        return FAIL;
    }
    let mut funcexe = FuncExeT::new();
    funcexe.fe_evaluate = true;
    call_func(s, -1, rettv.as_ptr(), argc, argv.as_ptr(), &mut funcexe)
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
        nvim_eval::errors::semsg_invexpr2(s);
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
#[export_name = "eval_to_bool"]
pub unsafe extern "C" fn rs_eval_to_bool(
    arg: *mut c_char,
    error: *mut bool,
    eap: ExargHandle,
    skip: bool,
    use_simple_function: bool,
) -> bool {
    let mut tv_storage = [0u64; 8]; // enough space for typval_T
    let tv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);

    let evalarg = alloc_evalarg(eap, skip);
    if skip {
        emsg_skip += 1;
    }

    let r = if use_simple_function {
        let r_simple = nvim_eval_may_call_simple_func(arg, tv);
        if r_simple == NOTDONE {
            eval0(arg, tv, eap, evalarg)
        } else {
            r_simple
        }
    } else {
        eval0(arg, tv, eap, evalarg)
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
        emsg_skip -= 1;
    }
    free_evalarg(evalarg, eap);

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
#[export_name = "eval_to_number"]
pub unsafe extern "C" fn rs_eval_to_number(expr: *mut c_char, use_simple_function: bool) -> i64 {
    let mut tv_storage = [0u64; 8];
    let tv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);

    let mut p = skipwhite(expr);

    emsg_off += 1;

    let mut r = NOTDONE;
    if use_simple_function {
        r = nvim_eval_may_call_simple_func(expr, tv);
    }
    if r == NOTDONE {
        let evalarg = nvim_get_evalarg_evaluate_ptr();
        r = eval1(&mut p as *mut *mut c_char, tv, evalarg);
    }

    let retval = if r == FAIL {
        -1i64
    } else {
        let n = tv_get_number_chk(tv, ptr::null_mut());
        tv_clear(tv);
        n
    };

    emsg_off -= 1;
    retval
}

/// Top-level string evaluation (with skip support).
///
/// Equivalent to C `eval_to_string_skip`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string pointer.
/// - `eap` may be null.
#[export_name = "eval_to_string_skip"]
pub unsafe extern "C" fn rs_eval_to_string_skip(
    arg: *mut c_char,
    eap: ExargHandle,
    skip: bool,
) -> *mut c_char {
    let mut tv_storage = [0u64; 8];
    let tv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);

    let evalarg = alloc_evalarg(eap, skip);
    if skip {
        emsg_skip += 1;
    }

    let retval = if eval0(arg, tv, eap, evalarg) == FAIL || skip {
        ptr::null_mut()
    } else {
        let s = nvim_eval_tv_get_str(tv);
        let r = nvim_eval_xstrdup(s);
        tv_clear(tv);
        r
    };

    if skip {
        emsg_skip -= 1;
    }
    free_evalarg(evalarg, eap);

    retval
}

/// Top-level string evaluation with exarg_T and join_list support.
///
/// Equivalent to C `eval_to_string_eap`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string pointer.
/// - `eap` may be null.
#[export_name = "eval_to_string_eap"]
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
    let evalarg = alloc_evalarg(eap, eap_skip);

    let r = if use_simple_function {
        let r_simple = nvim_eval_may_call_simple_func(arg, tv);
        if r_simple == NOTDONE {
            eval0(arg, tv, ExargHandle::null(), evalarg)
        } else {
            r_simple
        }
    } else {
        eval0(arg, tv, ExargHandle::null(), evalarg)
    };

    let retval = if r == FAIL {
        ptr::null_mut()
    } else {
        let s = typval2string_impl(tv.as_ptr(), join_list);
        tv_clear(tv);
        s
    };

    free_evalarg(evalarg, ExargHandle::null());
    retval
}

/// Top-level string evaluation.
///
/// Equivalent to C `eval_to_string`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string pointer.
#[export_name = "eval_to_string"]
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
#[export_name = "eval_to_string_safe"]
pub unsafe extern "C" fn rs_eval_to_string_safe(
    arg: *mut c_char,
    use_sandbox: bool,
    use_simple_function: bool,
) -> *mut c_char {
    let entry = xcalloc(1, std::mem::size_of::<FunccalEntryT>());
    nvim_save_funccal_inner(entry);
    if use_sandbox {
        sandbox += 1;
    }
    textlock += 1;

    let retval = rs_eval_to_string(arg, false, use_simple_function);

    if use_sandbox {
        sandbox -= 1;
    }
    textlock -= 1;
    nvim_restore_funccal_inner();
    xfree(entry);

    retval
}

/// Skip over an expression at `*pp`.
///
/// Equivalent to C `skip_expr`.
///
/// # Safety
/// - `pp` must be a valid pointer to a mutable C string pointer.
/// - `evalarg` may be null.
#[export_name = "skip_expr"]
pub unsafe extern "C" fn rs_skip_expr(pp: *mut *mut c_char, evalarg: EvalargHandle) -> c_int {
    // EvalargHandle::flags() returns 0 if null
    let save_flags = evalarg.flags();

    // Don't evaluate the expression -- clear EVAL_EVALUATE flag (bit 0).
    if !evalarg.is_null() {
        let flags = (*evalarg.as_ptr()).eval_flags;
        evalarg.set_flags(flags & !1);
    }

    *pp = skipwhite(*pp);
    let mut tv_storage = [0u64; 8];
    let rettv = TypevalHandle::from_ptr(tv_storage.as_mut_ptr() as *mut c_void);
    let res = eval1(pp, rettv, EvalargHandle::null());

    evalarg.set_flags(save_flags);

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
    let did_emsg_before = did_emsg;
    let called_emsg_before = called_emsg;

    let skip = !eap.is_null() && nvim_eap_get_skip_local(eap) != 0;
    let evalarg = alloc_evalarg(eap, skip);

    let ret = eval1(arg, rettv, evalarg);
    if ret == FAIL
        && aborting() == 0
        && did_emsg == did_emsg_before
        && called_emsg == called_emsg_before
    {
        nvim_eval::errors::semsg_invexpr2(start);
    }

    free_evalarg(evalarg, eap);
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
#[export_name = "eval_expr_typval"]
pub unsafe extern "C" fn rs_eval_expr_typval(
    expr: *const c_void,
    want_func: bool,
    argv: TypevalHandle,
    argc: c_int,
    rettv: TypevalHandle,
) -> c_int {
    let vtype = (*TypevalHandle::from_ptr(expr as *mut c_void)
        .as_ptr()
        .cast::<TypvalTRepr>())
    .v_type;
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
#[export_name = "eval_expr_to_bool"]
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
pub unsafe extern "C" fn fill_evalarg_from_eap(
    evalarg: EvalargHandle,
    eap: ExargHandle,
    skip: bool,
) {
    // Zero-init and set eval_flags based on skip.
    *evalarg.as_ptr() = if skip {
        EvalargT::new_skip()
    } else {
        EvalargT::new_evaluate()
    };
    if eap.is_null() {
        return;
    }
    if sourcing_a_script(eap) != 0 {
        // Copy the getline function pointer and cookie from eap.
        (*evalarg.as_ptr()).eval_getline = nvim_eap_get_getline(eap);
        (*evalarg.as_ptr()).eval_cookie = nvim_eap_get_cookie(eap);
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
pub unsafe extern "C" fn clear_evalarg(evalarg: EvalargHandle, eap: ExargHandle) {
    if evalarg.is_null() {
        return;
    }
    let tofree = (*evalarg.as_ptr()).eval_tofree;
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
    (*evalarg.as_ptr()).eval_tofree = std::ptr::null_mut();
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
pub unsafe extern "C" fn may_call_simple_func(arg: *const c_char, rettv: TypevalHandle) -> c_int {
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
    let r = may_call_simple_func(arg, rettv);
    if r == NOTDONE {
        eval0(arg, rettv, eap, evalarg)
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
#[export_name = "eval_expr_ext"]
pub unsafe extern "C" fn rs_eval_expr_ext(
    arg: *mut c_char,
    eap: ExargHandle,
    use_simple_function: bool,
) -> *mut c_void {
    let tv = xmalloc(16); // sizeof(typval_T) = 16 bytes
    let tv_handle = TypevalHandle::from_ptr(tv);

    let eap_skip = !eap.is_null() && nvim_eap_get_skip_local(eap) != 0;

    let evalarg = alloc_evalarg(eap, eap_skip);

    let r = if use_simple_function {
        eval0_simple_funccal_impl(arg, tv_handle, eap, evalarg)
    } else {
        eval0(arg, tv_handle, eap, evalarg)
    };

    if r == FAIL {
        xfree(tv);
        free_evalarg(evalarg, eap);
        return std::ptr::null_mut();
    }

    free_evalarg(evalarg, eap);
    tv
}

/// Equivalent to C `eval_expr`: evaluate expression, return allocated typval_T or NULL.
///
/// This is a thin wrapper around `rs_eval_expr_ext` with `use_simple_function = false`.
///
/// # Safety
/// - `arg` must be a valid null-terminated C string.
/// - `eap` may be null.
#[export_name = "eval_expr"]
pub unsafe extern "C" fn rs_eval_expr(arg: *mut c_char, eap: ExargHandle) -> *mut c_void {
    rs_eval_expr_ext(arg, eap, false)
}

// =============================================================================
// Phase 1 (eval_shim pass 5): call_vim_function family + small utilities
// =============================================================================

extern "C" {
    // Phase 1: call_vim_function accessors
    // call_func is already declared in eval_expr block above (re-use via funcexe::FuncExeT)
    fn nvim_get_vlua_partial() -> *mut c_void; // partial_T*
    fn nvim_curwin_get_cursor_lnum() -> i32;
    fn rs_check_luafunc_name(s: *const c_char, paren: bool) -> c_int;

    // Phase 1: set_argv_var accessors
    #[link_name = "tv_list_alloc"]
    fn nvim_eval_tv_list_alloc(len: isize) -> *mut c_void; // list_T*
    fn nvim_tv_list_set_lock(l: *mut c_void, lock: c_int);
    #[link_name = "tv_list_append_string"]
    fn nvim_tv_list_append_string(l: *mut c_void, s: *const c_char, len: isize);
    fn nvim_tv_list_last_fix_lock(l: *mut c_void);
    fn nvim_set_vim_var_argv_list(list: *mut c_void);

    // Phase 1: var_set_global accessors
    fn nvim_set_var_wrapper(name: *const c_char, name_len: usize, tv: TypevalHandle);

    // Phase 1: eval_fmt_source_name_line accessors
    fn nvim_sourcing_name_get() -> *const c_char;
    fn nvim_sourcing_lnum_get() -> i64; // linenr_T

    // Direct call to rs_find_option_end (Rust, option crate) - eliminates round-trip via C shim
    fn rs_find_option_end(arg: *const c_char) -> FindOptionEndResult;

    // Phase 1: call_func_retstr helper
    #[link_name = "xstrdup"]
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
}

// VAR_FIXED lock constant (from typval_defs.h: VAR_FIXED = 2)
const VAR_FIXED: c_int = 2;

/// Call a VimL function by name and place the result in `rettv`.
///
/// Equivalent to C `call_vim_function`.
///
/// # Safety
/// All pointers must be valid.
#[export_name = "call_vim_function"]
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
        partial = nvim_get_vlua_partial();
    }

    // Initialize rettv: set v_type = VAR_UNKNOWN (0) so tv_clear works on failure
    rettv.set_type(0); // VAR_UNKNOWN = 0

    let lnum = nvim_curwin_get_cursor_lnum();
    let mut funcexe = FuncExeT::new();
    funcexe.fe_firstline = lnum;
    funcexe.fe_lastline = lnum;
    funcexe.fe_evaluate = true;
    funcexe.fe_partial = partial;
    let ret = call_func(
        actual_func,
        actual_len,
        rettv.as_ptr(),
        argc,
        argv.as_ptr(),
        &mut funcexe,
    );

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
#[export_name = "call_func_retstr"]
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

    let s = nvim_eval_tv_get_str(rettv);
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
#[export_name = "call_func_retlist"]
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

    let vtype = (*rettv.as_ptr().cast::<TypvalTRepr>()).v_type;
    if vtype != VAR_LIST {
        tv_clear(rettv);
        return std::ptr::null_mut();
    }

    (*rettv.as_ptr().cast::<TypvalTRepr>()).vval.v_list
}

/// Set the v:argv list from argc/argv.
///
/// Equivalent to C `set_argv_var`.
///
/// # Safety
/// `argv` must be an array of `argc` valid C string pointers.
#[export_name = "set_argv_var"]
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
#[export_name = "var_set_global"]
pub unsafe extern "C" fn rs_var_set_global(name: *const c_char, vartv: TypevalHandle) {
    let entry = xcalloc(1, std::mem::size_of::<FunccalEntryT>());
    nvim_save_funccal_inner(entry);
    let name_len = libc_strlen(name);
    nvim_set_var_wrapper(name, name_len, vartv);
    nvim_restore_funccal_inner();
    xfree(entry);
}

/// Write "<sourcing_name>:<sourcing_lnum>" to buf[bufsize].
///
/// Equivalent to C `eval_fmt_source_name_line`.
///
/// # Safety
/// `buf` must be a valid writable buffer of at least `bufsize` bytes.
#[export_name = "eval_fmt_source_name_line"]
pub unsafe extern "C" fn rs_eval_fmt_source_name_line(buf: *mut c_char, bufsize: usize) {
    let name = nvim_sourcing_name_get();
    if !name.is_null() {
        let lnum = nvim_sourcing_lnum_get();
        // Format "%s:%" PRIdLINENR into buf -- equivalent to snprintf(buf, bufsize, "%s:%ld", name, lnum)
        let buf_slice = std::slice::from_raw_parts_mut(buf as *mut u8, bufsize);
        let mut cursor = std::io::Cursor::new(buf_slice);
        let name_cstr = std::ffi::CStr::from_ptr(name);
        let name_str = name_cstr.to_bytes();
        use std::io::Write;
        let _ = write!(cursor, "{}:{}\0", String::from_utf8_lossy(name_str), lnum);
        // Ensure NUL termination in case of truncation
        let pos = cursor.position() as usize;
        let buf_slice2 = std::slice::from_raw_parts_mut(buf as *mut u8, bufsize);
        if pos >= bufsize {
            buf_slice2[bufsize - 1] = 0;
        }
    } else {
        // Write "?" to buffer
        if bufsize >= 2 {
            *buf = b'?' as c_char;
            *buf.add(1) = 0;
        } else if bufsize == 1 {
            *buf = 0;
        }
    }
}

/// Skip over the name of an option variable: "&option", "&g:option" or "&l:option".
///
/// Exported as `find_option_var_end` (replaces C wrapper in eval_shim.c, Phase 12).
/// Accepts `OptIndex*` as `*mut c_int` (compatible since OptIndex is int-sized).
///
/// # Safety
/// `arg` must be a valid pointer to a mutable C string pointer.
/// `opt_idxp` and `opt_flags` must be valid pointers.
#[export_name = "find_option_var_end"]
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

    let r = rs_find_option_end(p);
    *opt_idxp = r.opt_idx;
    if r.end.is_null() {
        // Leave *arg unchanged on failure
    } else {
        *arg = p;
    }
    r.end
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
// Phase 16: consolidated to NvimPromptState bulk struct.
// =============================================================================

/// Bulk prompt state read from curbuf.
/// Must match NvimPromptState typedef in eval.h exactly.
#[repr(C)]
struct NvimPromptState {
    curbuf: *mut c_void,    // buf_T*
    ml_line_count: i32,     // linenr_T = int32_t
    prompt_start_lnum: i32, // linenr_T = int32_t
    prompt_callback: *mut CallbackT,
    prompt_interrupt: *mut CallbackT,
}

extern "C" {
    // Phase 2: prompt_get_input accessors
    fn nvim_mark_bt_prompt(buf: *mut c_void) -> c_int; // static inline, needs wrapper
    fn nvim_buf_get_prompt_start_lnum(buf: *mut c_void) -> i32; // linenr_T = i32
    fn nvim_buf_get_b_ml_ml_line_count(buf: *mut c_void) -> i32; // linenr_T = i32
    fn ml_get_buf(buf: *mut c_void, lnum: i32) -> *mut c_char;
    #[link_name = "prompt_text"]
    fn nvim_prompt_text() -> *const c_char;
    #[link_name = "concat_str"]
    fn nvim_concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;
    // nvim_xstrdup declared in Phase 1 extern block above

    // Phase 16: bulk prompt state reader + writer
    fn nvim_read_prompt_state(out: *mut NvimPromptState);
    fn nvim_write_prompt_start_lnum(lnum: i32);

    // Phase 2: prompt_invoke_callback accessors (kept)
    #[link_name = "ml_append"]
    fn nvim_ml_append(lnum: i32, line: *const c_char, len: i32, newfile: bool) -> c_int;
    #[link_name = "appended_lines_mark"]
    fn nvim_appended_lines_mark(lnum: i32, count: c_int);
    fn nvim_set_cursor_lnum(lnum: i32); // linenr_T
    fn nvim_set_cursor_col(col: c_int);
    fn nvim_curbuf_u_clearallandblockfree();

    // Direct C global
    pub static mut got_int: bool;
}

// kCallbackNone = 0
const K_CALLBACK_NONE: c_int = 0;

// TYPVAL_SIZE: sizeof(typval_T) == 16 (validated by _Static_assert in eval_shim.c)
const TYPVAL_SIZE: usize = 16;
// VAR_STRING (v_type = 2) for argv setup in prompt callbacks
const VAR_STRING_TYPE: i32 = 2;

/// Get the current user-input text from a prompt buffer.
///
/// Returns NULL if `buf` is not a prompt buffer.
/// Returns an allocated string that the caller must free.
///
/// Equivalent to C `prompt_get_input`.
///
/// # Safety
/// `buf` must be a valid `buf_T *`.
#[export_name = "prompt_get_input"]
pub unsafe extern "C" fn rs_prompt_get_input(buf: *mut c_void) -> *mut c_char {
    if nvim_mark_bt_prompt(buf) == 0 {
        return std::ptr::null_mut();
    }

    let lnum_start: i32 = nvim_buf_get_prompt_start_lnum(buf);
    let lnum_last: i32 = nvim_buf_get_b_ml_ml_line_count(buf);

    // Get the first line and skip past the prompt prefix
    let text_raw = ml_get_buf(buf, lnum_start);
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
        let line = ml_get_buf(buf, i);
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
#[export_name = "prompt_invoke_callback"]
pub unsafe extern "C" fn rs_prompt_invoke_callback() {
    let mut ps = NvimPromptState {
        curbuf: ptr::null_mut(),
        ml_line_count: 0,
        prompt_start_lnum: 0,
        prompt_callback: ptr::null_mut(),
        prompt_interrupt: ptr::null_mut(),
    };
    nvim_read_prompt_state(&mut ps);
    let lnum: i32 = ps.ml_line_count;

    let user_input = rs_prompt_get_input(ps.curbuf);
    if user_input.is_null() {
        return;
    }

    // Add a new line for the prompt before invoking callback
    let empty = b"\0";
    nvim_ml_append(lnum, empty.as_ptr() as *const c_char, 0, false);
    nvim_appended_lines_mark(lnum, 1);
    nvim_set_cursor_lnum(lnum + 1);
    nvim_set_cursor_col(0);
    nvim_write_prompt_start_lnum(lnum + 1);

    let callback = ps.prompt_callback;
    // Direct field access: replaces nvim_eval_cb_get_type
    if (*callback).cb_type == K_CALLBACK_NONE {
        xfree(user_input as *mut c_void);
    } else {
        // Build argv[2] on the stack: [VAR_STRING(user_input), VAR_UNKNOWN]
        // typval_T layout: v_type (int/4B at offset 0) + v_lock (int/4B at offset 4) + vval (8B at offset 8) = 16B
        let mut argv_buf = [0u8; TYPVAL_SIZE * 2];
        // argv[0].v_type = VAR_STRING (= 2) at offset 0 (4-byte int, LE)
        let argv0_vtype = argv_buf.as_mut_ptr() as *mut i32;
        *argv0_vtype = VAR_STRING_TYPE;
        // argv[0].vval.v_string at offset 8 (pointer size)
        let argv0_vstring = argv_buf.as_mut_ptr().add(8) as *mut *mut c_char;
        *argv0_vstring = user_input;
        // argv[1].v_type = VAR_UNKNOWN (= 0, already zeroed)

        let mut rettv_buf = [0u8; TYPVAL_SIZE];
        let rettv = TypevalHandle::from_ptr(rettv_buf.as_mut_ptr() as *mut c_void);
        crate::callback::callback_call_impl(
            callback,
            1,
            argv_buf.as_mut_ptr() as *mut c_void,
            rettv,
        );
        // Free argv[0] (clears the VAR_STRING user_input) and rettv
        tv_clear(TypevalHandle::from_ptr(argv_buf.as_mut_ptr() as *mut c_void));
        tv_clear(rettv);
    }

    // clear undo history on submit
    nvim_curbuf_u_clearallandblockfree();
    // Re-read ml_line_count after mutation
    nvim_read_prompt_state(&mut ps);
    nvim_write_prompt_start_lnum(ps.ml_line_count);
}

/// Invoke the prompt interrupt callback.
///
/// Equivalent to C `invoke_prompt_interrupt`.
///
/// # Safety
/// Uses global `curbuf`. Must be called from main thread.
#[export_name = "invoke_prompt_interrupt"]
pub unsafe extern "C" fn rs_invoke_prompt_interrupt() -> bool {
    let mut ps = NvimPromptState {
        curbuf: ptr::null_mut(),
        ml_line_count: 0,
        prompt_start_lnum: 0,
        prompt_callback: ptr::null_mut(),
        prompt_interrupt: ptr::null_mut(),
    };
    nvim_read_prompt_state(&mut ps);
    let callback = ps.prompt_interrupt;
    // Direct field access: replaces nvim_eval_cb_get_type
    if (*callback).cb_type == K_CALLBACK_NONE {
        return false;
    }

    unsafe {
        got_int = false;
    }

    // Build argv[1] on the stack: [VAR_UNKNOWN]
    let mut argv_buf = [0u8; TYPVAL_SIZE]; // VAR_UNKNOWN = 0, already zeroed
    let mut rettv_buf = [0u8; TYPVAL_SIZE];
    let rettv = TypevalHandle::from_ptr(rettv_buf.as_mut_ptr() as *mut c_void);
    let ret = crate::callback::callback_call_impl(
        callback,
        0,
        argv_buf.as_mut_ptr() as *mut c_void,
        rettv,
    );
    tv_clear(rettv);
    ret
}

// =============================================================================
// Phase 3 (eval_shim pass 5): eval_foldexpr and eval_foldtext
// Phase 16: consolidated into NvimFoldEvalState bulk struct.
// =============================================================================

/// Bulk fold eval state read from a window.
/// Must match NvimFoldEvalState typedef in eval.h exactly.
/// Layout: bool(1) + bool(1) + pad(6) + *char(8) + *char(8) = 24 bytes.
#[repr(C)]
struct NvimFoldEvalState {
    insecure_foldexpr: bool,
    insecure_foldtext: bool,
    // 6 bytes padding (implicit, C layout with 8-byte pointer alignment)
    _pad: [u8; 6],
    foldexpr: *mut c_char,
    foldtext: *mut c_char,
}

extern "C" {
    // Phase 16: bulk fold eval state reader
    fn nvim_read_fold_eval_state(wp: *mut c_void, out: *mut NvimFoldEvalState);
    // Phase 16: consolidated sctx save+set; restore kept separate
    fn nvim_fold_sctx_save_and_set(wp: *mut c_void) -> *mut c_void; // returns sctx_T*
    fn nvim_restore_current_sctx(saved: *mut c_void); // frees saved sctx_T*

    // Phase 16: unified foldtext object maker
    fn nvim_foldtext_make_obj(tv: *mut c_void, tv_type: c_int, out: *mut c_void);
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
#[export_name = "eval_foldexpr"]
pub unsafe extern "C" fn rs_eval_foldexpr(wp: *mut c_void, cp: *mut c_int) -> c_int {
    let mut fs = NvimFoldEvalState {
        insecure_foldexpr: false,
        insecure_foldtext: false,
        _pad: [0u8; 6],
        foldexpr: ptr::null_mut(),
        foldtext: ptr::null_mut(),
    };
    nvim_read_fold_eval_state(wp, &mut fs);
    let use_sandbox = fs.insecure_foldexpr;
    let arg = fs.foldexpr;

    // Save current_sctx and set it from the window's foldexpr script context
    let saved_sctx = nvim_fold_sctx_save_and_set(wp);

    emsg_off += 1;
    if use_sandbox {
        sandbox += 1;
    }
    textlock += 1;
    *cp = 0; // NUL

    let tv = xmalloc(16); // sizeof(typval_T) = 16 bytes
    let tv_handle = TypevalHandle::from_ptr(tv);

    let evalarg = nvim_get_evalarg_evaluate_ptr();
    let retval: i64 =
        if eval0_simple_funccal_impl(arg, tv_handle, ExargHandle::null(), evalarg) == FAIL {
            0
        } else {
            let vtype = (*TypevalHandle::from_ptr(tv).as_ptr().cast::<TypvalTRepr>()).v_type;
            let result = if vtype == VAR_NUMBER {
                (*tv.cast::<TypvalTRepr>()).vval.v_number
            } else if vtype != VAR_STRING {
                0
            } else {
                let s = TypevalHandle::from_ptr(tv).get_vstring();
                if s.is_null() {
                    0
                } else {
                    // If string starts with non-digit, non-minus: that char is the prefix
                    let first = *s as u8;
                    let s_num = if first != 0 && !first.is_ascii_digit() && first != b'-' {
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
                        if c.is_ascii_digit() {
                            n = n * 10 + (c - b'0') as i64;
                            p = p.add(1);
                        } else {
                            break;
                        }
                    }
                    if neg {
                        -n
                    } else {
                        n
                    }
                }
            };
            tv_clear(tv_handle);
            result
        };

    emsg_off -= 1;
    if use_sandbox {
        sandbox -= 1;
    }
    textlock -= 1;
    clear_evalarg(evalarg, ExargHandle::null());
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
    let mut fs = NvimFoldEvalState {
        insecure_foldexpr: false,
        insecure_foldtext: false,
        _pad: [0u8; 6],
        foldexpr: ptr::null_mut(),
        foldtext: ptr::null_mut(),
    };
    nvim_read_fold_eval_state(wp, &mut fs);
    let use_sandbox = fs.insecure_foldtext;
    let arg = fs.foldtext;

    let funccal = xcalloc(1, std::mem::size_of::<FunccalEntryT>());
    nvim_save_funccal_inner(funccal);
    if use_sandbox {
        sandbox += 1;
    }
    textlock += 1;

    let tv = xmalloc(16); // sizeof(typval_T) = 16 bytes
    let tv_handle = TypevalHandle::from_ptr(tv);

    let evalarg = nvim_get_evalarg_evaluate_ptr();
    if eval0_simple_funccal_impl(arg, tv_handle, ExargHandle::null(), evalarg) == FAIL {
        nvim_foldtext_make_obj(ptr::null_mut(), 0, out);
    } else {
        let vtype = (*tv_handle.as_ptr().cast::<TypvalTRepr>()).v_type;
        nvim_foldtext_make_obj(tv, vtype, out);
        tv_clear(tv_handle);
    }

    clear_evalarg(evalarg, ExargHandle::null());

    if use_sandbox {
        sandbox -= 1;
    }
    textlock -= 1;
    nvim_restore_funccal_inner();
    xfree(funccal);
    xfree(tv);
}
