//! Lambda / alloc_ufunc / register_luafunc migration.
//!
//! Phase 2 (plan db85cc6b) from `src/nvim/eval/userfunc.c`:
//! - `get_lambda_name` (Rust: internal `get_lambda_name_str`)
//! - `alloc_ufunc`     (Rust: rs_alloc_ufunc)
//! - `register_luafunc` (Rust: rs_register_luafunc, keeps C linkage name)
//!
//! Phase 5 (plan db85cc6b):
//! - `get_lambda_tv`   (Rust: rs_get_lambda_tv, keeps C linkage name)

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicI32, Ordering};

use super::parsing::GarrayT;

// NUMBUFLEN = 65 (matches C define in vim_defs.h)
const NUMBUFLEN: usize = 65;
// lambda name buffer: "<lambda>" (8) + NUMBUFLEN for the integer
const LAMBDA_BUF_SIZE: usize = 8 + NUMBUFLEN;

// Static counter and buffer for lambda names (mirrors C static state)
static LAMBDA_NO: AtomicI32 = AtomicI32::new(0);

// Return codes (matching C defines in vim_defs.h)
const OK: c_int = 1;
const FAIL: c_int = 0;
const NOTDONE: c_int = 2;

// FC_* flags for uf_flags (matching userfunc.h)
const FC_CLOSURE: c_int = 0x08;
const FC_SANDBOX: c_int = 0x40;
const FC_NOARGS: c_int = 0x200;

// VAR_PARTIAL variant index (matches VarType enum in typval_defs.h)
const VAR_PARTIAL: c_int = 9;

extern "C" {
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GarrayT, n: c_int);
    fn nvim_ga_clear_strings_wrapper(ga: *mut c_void);
    fn nvim_sizeof_ufunc_header() -> usize;
    fn nvim_sizeof_partial() -> usize;
    fn nvim_ufunc_init_name(fp: *mut c_void, name: *const c_char, namelen: usize);
    fn nvim_ufunc_init_luaref_fields(fp: *mut c_void, lua_ref: c_int);
    fn nvim_func_ht_add_fp(fp: *mut c_void);
    fn nvim_ufunc_get_name_ptr(fp: *mut c_void) -> *mut c_char;
    fn nvim_ufunc_set_refcount(fp: *mut c_void, v: c_int);
    fn nvim_ufunc_set_varargs(fp: *mut c_void, v: c_int);
    fn nvim_ufunc_set_flags(fp: *mut c_void, v: c_int);
    fn nvim_ufunc_set_calls(fp: *mut c_void, v: c_int);
    fn nvim_ufunc_set_scoped(fp: *mut c_void, fc: *mut c_void);
    fn nvim_ufunc_move_args_ga(fp: *mut c_void, src: *const GarrayT);
    fn nvim_ufunc_move_lines_ga(fp: *mut c_void, src: *const GarrayT);
    fn nvim_ufunc_get_def_args_ga(fp: *mut c_void) -> *mut c_void;
    fn nvim_ufunc_finalize_script_ctx(fp: *mut c_void, newlines_len: c_int);
    fn nvim_partial_set_func(pt: *mut c_void, fp: *mut c_void);
    fn nvim_partial_set_refcount(pt: *mut c_void, v: c_int);
    fn nvim_tv_set_type(tv: *mut c_void, v_type: c_int);
    fn nvim_tv_set_partial(tv: *mut c_void, pt: *mut c_void);
    fn nvim_vars_get_eval_lavars_used() -> *mut bool;
    fn nvim_vars_set_eval_lavars_used(ptr: *mut bool);
    fn nvim_get_current_funccal() -> *mut c_void;
    fn nvim_get_sandbox() -> c_int;
    fn prof_def_func() -> bool;
    fn func_do_profile(fp: *mut c_void);
    fn nvim_skip_expr(arg: *mut *mut c_char, evalarg: *mut c_void) -> c_int;
    fn nvim_evalarg_get_tofree(ea: *mut c_void) -> *mut c_char;
    fn nvim_evalarg_set_tofree(ea: *mut c_void, v: *mut c_char);
    fn nvim_evalarg_should_evaluate(ea: *const c_void) -> bool;
    fn strstr(s: *const c_char, needle: *const c_char) -> *mut c_char;
    fn rs_register_closure(fp: *mut c_void);
    fn nvim_semsg_e451_expected_cbrace(p: *const c_char);
}

// =============================================================================
// get_lambda_name (internal)
// =============================================================================

/// Generate the next lambda name.
///
/// Returns a stack-allocated buffer containing "<lambda>N\0".
/// Caller gets a slice up to (but not including) the NUL.
fn get_lambda_name_bytes() -> ([u8; LAMBDA_BUF_SIZE], usize) {
    let n = LAMBDA_NO.fetch_add(1, Ordering::Relaxed) + 1;
    let mut buf = [0u8; LAMBDA_BUF_SIZE];
    let prefix = b"<lambda>";
    buf[..prefix.len()].copy_from_slice(prefix);
    // Format the integer manually to avoid formatting machinery in no_std-like context
    let n_str = n.to_string();
    let n_bytes = n_str.as_bytes();
    let len = prefix.len() + n_bytes.len();
    buf[prefix.len()..len].copy_from_slice(n_bytes);
    buf[len] = 0; // NUL terminate
    (buf, len)
}

// =============================================================================
// rs_alloc_ufunc
// =============================================================================

/// Allocate a ufunc_T for a function with the given name and length.
/// Returns a pointer to the newly allocated (xcalloc'd) ufunc_T.
/// The caller is responsible for further initialization.
///
/// # Safety
/// `name` must be a valid pointer to at least `namelen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_alloc_ufunc(name: *const c_char, namelen: usize) -> *mut c_void {
    let header_size = unsafe { nvim_sizeof_ufunc_header() };
    let total_size = header_size + namelen + 1;
    let fp = unsafe { xcalloc(1, total_size) };
    if !fp.is_null() {
        unsafe { nvim_ufunc_init_name(fp, name, namelen) };
    }
    fp
}

// =============================================================================
// rs_get_lambda_name (exported for C callers in get_lambda_tv, register_luafunc)
// =============================================================================

/// Write a new lambda name into `buf` (capacity `bufsize`).
/// Returns the number of bytes written (not counting NUL), or 0 on overflow.
///
/// # Safety
/// `buf` must point to at least `bufsize` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_get_lambda_name(buf: *mut c_char, bufsize: usize) -> usize {
    if buf.is_null() || bufsize == 0 {
        return 0;
    }
    let (bytes, len) = get_lambda_name_bytes();
    let copy_len = len.min(bufsize - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buf, copy_len + 1);
    }
    copy_len
}

// =============================================================================
// rs_register_luafunc
// =============================================================================

/// Register a Lua callback as a named lambda ufunc.
/// Returns `fp->uf_name` (the function name string).
///
/// Mirrors C `register_luafunc(ref)`.
///
/// # Safety
/// `lua_ref` must be a valid LuaRef (int).
#[unsafe(export_name = "register_luafunc")]
pub unsafe extern "C" fn rs_register_luafunc(lua_ref: c_int) -> *mut c_char {
    let (name_bytes, name_len) = get_lambda_name_bytes();
    let fp = unsafe { rs_alloc_ufunc(name_bytes.as_ptr().cast::<c_char>(), name_len) };
    if fp.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { nvim_ufunc_init_luaref_fields(fp, lua_ref) };
    unsafe { nvim_func_ht_add_fp(fp) };
    // coverity[leaked_storage] - intentional: fp lives until func_free
    unsafe { nvim_ufunc_get_name_ptr(fp) }
}

// =============================================================================
// rs_get_lambda_tv
// =============================================================================

/// Helper: restore eval_lavars_used and eval_tofree on success or error.
unsafe fn restore_lambda_state(
    old_eval_lavars: *mut bool,
    tofree: *mut c_char,
    evalarg: *mut c_void,
) {
    nvim_vars_set_eval_lavars_used(old_eval_lavars);
    let can_restore = !evalarg.is_null() && nvim_evalarg_get_tofree(evalarg).is_null();
    if can_restore {
        nvim_evalarg_set_tofree(evalarg, tofree);
    } else {
        xfree(tofree.cast::<c_void>());
    }
}

/// Parse a lambda expression `{args -> expr}` from `*arg` and store a Funcref
/// in `rettv`.
///
/// Returns `OK` (1), `FAIL` (0), or `NOTDONE` (2) for non-lambda `{...}`.
///
/// # Safety
/// - `arg` must be a valid `char **` pointing into a mutable C string.
/// - `rettv` must be a valid `typval_T *`.
/// - `evalarg` may be NULL or a valid `evalarg_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_lambda_tv(
    arg: *mut *mut c_char,
    rettv: *mut c_void,
    evalarg: *mut c_void,
) -> c_int {
    let evaluate = !evalarg.is_null() && nvim_evalarg_should_evaluate(evalarg);

    // GA_EMPTY_INIT_VALUE: all zeros
    let mut newargs = GarrayT {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: std::ptr::null_mut(),
    };
    let mut varargs: c_int = 0;
    let old_eval_lavars = nvim_vars_get_eval_lavars_used();

    // Stack-allocated bool for eval_lavars; its address is passed to C.
    let mut eval_lavars: bool = false;

    let mut tofree: *mut c_char = std::ptr::null_mut();

    // First pass: check that this looks like a lambda (must find "->").
    let mut s = skipwhite((*arg).add(1));
    let ret = super::parsing::rs_get_function_args(
        std::ptr::addr_of_mut!(s),
        b'-' as c_char,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        true,
    );
    if ret == FAIL || *s != b'>' as c_char {
        return NOTDONE;
    }

    // Second pass: parse arguments for real.
    let pnewargs = if evaluate {
        std::ptr::addr_of_mut!(newargs)
    } else {
        std::ptr::null_mut()
    };
    *arg = skipwhite((*arg).add(1));
    let ret = super::parsing::rs_get_function_args(
        arg,
        b'-' as c_char,
        pnewargs,
        std::ptr::addr_of_mut!(varargs),
        std::ptr::null_mut(),
        false,
    );
    if ret == FAIL || **arg != b'>' as c_char {
        // errret: clear newargs (if allocated), restore state.
        nvim_ga_clear_strings_wrapper(std::ptr::addr_of_mut!(newargs).cast::<c_void>());
        // fp is NULL at this point (not yet allocated)
        restore_lambda_state(old_eval_lavars, tofree, evalarg);
        return FAIL;
    }

    // Set up eval_lavars tracking.
    if evaluate {
        nvim_vars_set_eval_lavars_used(std::ptr::addr_of_mut!(eval_lavars));
    }

    // Skip "->" and parse the expression.
    *arg = skipwhite((*arg).add(1));
    let start: *const c_char = *arg;
    let ret = nvim_skip_expr(arg, evalarg);
    let end: *const c_char = *arg;

    if ret == FAIL {
        nvim_ga_clear_strings_wrapper(std::ptr::addr_of_mut!(newargs).cast::<c_void>());
        restore_lambda_state(old_eval_lavars, tofree, evalarg);
        return FAIL;
    }

    // Grab evalarg->eval_tofree to prevent it from being freed prematurely.
    if !evalarg.is_null() {
        tofree = nvim_evalarg_get_tofree(evalarg);
        nvim_evalarg_set_tofree(evalarg, std::ptr::null_mut());
    }

    *arg = skipwhite(*arg);
    if **arg != b'}' as c_char {
        nvim_semsg_e451_expected_cbrace(*arg);
        nvim_ga_clear_strings_wrapper(std::ptr::addr_of_mut!(newargs).cast::<c_void>());
        restore_lambda_state(old_eval_lavars, tofree, evalarg);
        return FAIL;
    }
    *arg = (*arg).add(1); // skip '}'

    if evaluate {
        let mut flags: c_int = 0;

        let (name_bytes, name_len) = get_lambda_name_bytes();
        let fp = rs_alloc_ufunc(name_bytes.as_ptr().cast::<c_char>(), name_len);
        if fp.is_null() {
            nvim_ga_clear_strings_wrapper(std::ptr::addr_of_mut!(newargs).cast::<c_void>());
            restore_lambda_state(old_eval_lavars, tofree, evalarg);
            return FAIL;
        }
        let pt = xcalloc(1, nvim_sizeof_partial());
        if pt.is_null() {
            xfree(fp);
            nvim_ga_clear_strings_wrapper(std::ptr::addr_of_mut!(newargs).cast::<c_void>());
            restore_lambda_state(old_eval_lavars, tofree, evalarg);
            return FAIL;
        }

        // Build newlines garray with "return <expr>"
        let mut newlines = GarrayT {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: std::ptr::null_mut(),
        };
        ga_init(
            std::ptr::addr_of_mut!(newlines),
            std::mem::size_of::<*mut c_char>() as c_int,
            1,
        );
        ga_grow(std::ptr::addr_of_mut!(newlines), 1);

        let expr_len = (end as usize).wrapping_sub(start as usize);
        // "return " (7) + expr + NUL
        let line_buf_len = 7 + expr_len + 1;
        let p = xmalloc(line_buf_len).cast::<u8>();
        // Write "return "
        std::ptr::copy_nonoverlapping(b"return ".as_ptr(), p, 7);
        // Write expr
        std::ptr::copy_nonoverlapping(start.cast::<u8>(), p.add(7), expr_len);
        // NUL-terminate
        *p.add(7 + expr_len) = 0;

        // Push the line into newlines
        let line_slot = newlines
            .ga_data
            .cast::<*mut c_void>()
            .add(newlines.ga_len as usize);
        *line_slot = p.cast::<c_void>();
        newlines.ga_len += 1;

        // Check if any a: variable is used in the expression.
        if strstr(p.add(7).cast::<c_char>(), c"a:".as_ptr()).is_null() {
            flags |= FC_NOARGS;
        }

        nvim_ufunc_set_refcount(fp, 1);
        nvim_func_ht_add_fp(fp);
        nvim_ufunc_move_args_ga(fp, std::ptr::addr_of!(newargs));
        // ga_init fp->uf_def_args
        let def_args_ga = nvim_ufunc_get_def_args_ga(fp).cast::<GarrayT>();
        if !def_args_ga.is_null() {
            ga_init(def_args_ga, std::mem::size_of::<*mut c_char>() as c_int, 1);
        }
        nvim_ufunc_move_lines_ga(fp, std::ptr::addr_of!(newlines));

        if !nvim_get_current_funccal().is_null() && eval_lavars {
            flags |= FC_CLOSURE;
            rs_register_closure(fp);
        } else {
            nvim_ufunc_set_scoped(fp, std::ptr::null_mut());
        }

        if prof_def_func() {
            func_do_profile(fp);
        }
        if nvim_get_sandbox() != 0 {
            flags |= FC_SANDBOX;
        }

        nvim_ufunc_set_varargs(fp, 1);
        nvim_ufunc_set_flags(fp, flags);
        nvim_ufunc_set_calls(fp, 0);
        nvim_ufunc_finalize_script_ctx(fp, newlines.ga_len);

        nvim_partial_set_func(pt, fp);
        nvim_partial_set_refcount(pt, 1);
        nvim_tv_set_partial(rettv, pt);
        nvim_tv_set_type(rettv, VAR_PARTIAL);
    }

    restore_lambda_state(old_eval_lavars, tofree, evalarg);
    OK
}
