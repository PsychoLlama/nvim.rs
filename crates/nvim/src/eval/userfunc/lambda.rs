//! Lambdas, closures and partials -- the anonymous half.
//!
//! `get_lambda_tv` parses `{x -> expr}` into a real `ufunc_T` with a
//! generated `<lambda>N` name and, if it captured anything, a reference to
//! the funccall it was made in (`register_closure`).  `make_partial` is the
//! other way a callable carries state: a bound dictionary, bound arguments,
//! or both.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::types::{FAIL, OK, Refcount};

/// Give `fp` the funccall that is running as its scope, so that the locals it
/// closed over stay alive for as long as it does.
///
/// # Safety
/// `fp` is a live function and a funccall is running.
pub(crate) unsafe fn register_closure(fp: *mut ufunc_T) {
    // SAFETY: the caller's promise -- `fp` is a live function.
    let mut f = unsafe { Uf::new(fp) };
    if f.uf_scoped == current_funccal.get() {
        return; // no change
    }
    unsafe { funccal_unref(f.uf_scoped, fp, false) };
    let fc = current_funccal.get();
    f.uf_scoped = fc;
    unsafe { (*fc).fc_refcount.retain() };
    unsafe { ga_grow(&raw mut (*fc).fc_ufuncs, 1) };
    let ufuncs = unsafe { &raw mut (*fc).fc_ufuncs };
    unsafe { *((*ufuncs).ga_data as *mut *mut ufunc_T).offset((*ufuncs).ga_len as isize) = fp };
    unsafe { (*ufuncs).ga_len += 1 };
}

/// `"<lambda>"` plus `NUMBUFLEN`, the widest a `varnumber_T` prints.
const LAMBDA_NAME_LEN: usize = 8 + 65;

/// The name of the next lambda, in `into` — the caller's, so that two
/// names can be alive at once. Upstream answers one static buffer.
///
/// # Safety
/// `into` must outlive the answer.
unsafe fn get_lambda_name(into: &mut [c_char; LAMBDA_NAME_LEN]) -> String_0 {
    static lambda_no: GlobalCell<c_int> = GlobalCell::new(0);
    lambda_no.set(lambda_no.get() + 1);
    let buf = into.as_mut_ptr();
    let nr = lambda_no.get();
    // SAFETY: `buf` is the caller's array of `LAMBDA_NAME_LEN` bytes.
    let n = unsafe { snprintf(buf, LAMBDA_NAME_LEN, c"<lambda>%d".as_ptr(), nr) };
    String_0::from_raw_parts(
        buf,
        if n < 1 {
            0
        } else {
            n.min(LAMBDA_NAME_LEN as c_int - 1) as size_t
        },
    )
}

/// Allocate a `ufunc_T` for a function called `name`, whose name lives in the
/// flexible member at the end of the allocation.
///
/// # Safety
/// `name` has `namelen` readable bytes.
pub(crate) unsafe fn alloc_ufunc(name: *const c_char, namelen: size_t) -> *mut ufunc_T {
    let fp = unsafe { xcalloc(1, offset_of!(ufunc_T, uf_name) + namelen + 1) } as *mut ufunc_T;
    // SAFETY: the allocation ends in `namelen + 1` bytes for the name.
    let into = uf_name_ptr(fp) as *mut c_void;
    unsafe { xmemcpyz(into, name as *const c_void, namelen) };
    unsafe { (*fp).uf_namelen = namelen };

    if unsafe { *name } as u8 as c_int == K_SPECIAL {
        // A script-local name is stored mangled; keep the printable
        // "<SNR>123_name" beside it.
        let len = namelen + 3;
        // SAFETY: `fp` is the allocation just made, whose inline name has
        // `namelen + 1` bytes; the printable form gets three more.
        let into = unsafe { xmalloc(len) } as *mut c_char;
        unsafe { (*fp).uf_name_exp = into };
        let tail = unsafe { uf_name_ptr(fp).add(3) };
        unsafe { snprintf(into, len, c"<SNR>%s".as_ptr(), tail) };
    }
    fp
}

/// Parse a lambda expression at `*arg` into a partial in `rettv`.
///
/// Answers `NOTDONE` when what is there is a dictionary or a `{expr}` rather
/// than a lambda -- which is decided by whether an `->` follows a legal
/// argument list.
///
/// # Safety
/// `*arg` points at the `{`, and `rettv` is an uninitialised return value.
pub unsafe fn get_lambda_tv(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let mut lambda_buf = [0 as c_char; LAMBDA_NAME_LEN];
    let evaluate = !evalarg.is_null() && unsafe { (*evalarg).eval_flags } & EVAL_EVALUATE != 0;
    let mut newargs = GARRAY_EMPTY;
    let mut varargs = 0;
    let old_eval_lavars = eval_lavars_used.get();
    let mut eval_lavars = false;
    let mut tofree: *mut c_char = ptr::null_mut();

    // First, check whether this is a lambda expression at all: an "->"
    // must follow a well-formed argument list.
    let mut s = unsafe { skipwhite((*arg).add(1)) };
    let (sp, dash) = (&raw mut s, b'-' as c_char);
    // SAFETY: `s` walks the caller's expression; nothing is written back.
    let (no_args, no_var, no_defs) = (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
    let looks_like = unsafe { get_function_args(sp, dash, no_args, no_var, no_defs, true) };
    if looks_like == FAIL || unsafe { *s } != b'>' as c_char {
        return NOTDONE;
    }

    // Neither `fp` nor `pt` escapes the arm that builds them, which is
    // why upstream's `assert(fp == NULL)` at its error label holds.
    let parsed = 'errret: {
        // Parse the arguments again, this time keeping them.
        let pnewargs = if evaluate {
            &raw mut newargs
        } else {
            ptr::null_mut()
        };
        unsafe { *arg = skipwhite((*arg).add(1)) };
        let (dash, varp) = (b'-' as c_char, &raw mut varargs);
        let none = ptr::null_mut();
        // SAFETY: `arg` is the caller's cursor and `varargs` is this
        // frame's local.
        let read = unsafe { get_function_args(arg, dash, pnewargs, varp, none, false) };
        if read == FAIL || unsafe { **arg } != b'>' as c_char {
            break 'errret false;
        }

        // Set up a flag for checking local variables and arguments.
        if evaluate {
            eval_lavars_used.set(&raw mut eval_lavars);
        }

        // Get the start and the end of the expression.
        unsafe { *arg = skipwhite((*arg).add(1)) };
        let start = unsafe { *arg };
        let ret = unsafe { skip_expr(arg, evalarg) };
        let end = unsafe { *arg };
        if ret == FAIL {
            break 'errret false;
        }
        if !evalarg.is_null() {
            // Avoid that the expression gets freed when another line
            // break follows.
            tofree = unsafe { (*evalarg).eval_tofree };
            unsafe { (*evalarg).eval_tofree = ptr::null_mut() };
        }

        unsafe { *arg = skipwhite(*arg) };
        if unsafe { **arg } != b'}' as c_char {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = unsafe { c_str(*arg) };
            semsg!("E451: Expected }}: {arg0}");
            break 'errret false;
        }
        unsafe { *arg = (*arg).add(1) };

        if evaluate {
            let mut flags = 0;
            let name = unsafe { get_lambda_name(&mut lambda_buf) };
            let fp = unsafe { alloc_ufunc(name.data(), name.len()) };
            let pt = unsafe { xcalloc(1, size_of::<partial_T>()) } as *mut partial_T;
            // SAFETY: both are this call's own allocations, and `rettv` is
            // the caller's uninitialised return value.
            let (mut f, mut part) = unsafe { (Uf::new(fp), Live::new(pt)) };
            let mut rv = unsafe { Tv::new(rettv) };

            let mut newlines = GARRAY_EMPTY;
            unsafe { ga_init(&raw mut newlines, size_of::<*mut c_char>() as c_int, 1) };
            unsafe { ga_grow(&raw mut newlines, 1) };

            // The body is the expression with "return " in front of it.
            const RETURN: &CStr = c"return ";
            let body_len = RETURN.count_bytes() + unsafe { end.offset_from(start) } as size_t + 1;
            let p = unsafe { xmalloc(body_len) } as *mut c_char;
            unsafe { *(newlines.ga_data as *mut *mut c_char) = p };
            newlines.ga_len = 1;
            unsafe { strcpy(p, RETURN.as_ptr()) };
            let expr = unsafe { p.add(RETURN.count_bytes()) };
            // SAFETY: `expr` has room for the body, which runs from
            // `start` to `end` inside the caller's expression.
            let len = unsafe { end.offset_from(start) } as size_t;
            unsafe { xmemcpyz(expr as *mut c_void, start as *const c_void, len) };
            if unsafe { strstr(expr, c"a:".as_ptr()) }.is_null() {
                // No a: variables are used for sure.
                flags |= FC_NOARGS;
            }

            f.uf_refcount = Refcount::ONE;
            let _ = unsafe { func_table().add(uf_name_ptr(fp)) };
            f.uf_args = newargs;
            let slot = size_of::<*mut c_char>() as c_int;
            unsafe { ga_init(&raw mut (*fp).uf_def_args, slot, 1) };
            f.uf_lines = newlines;
            if !current_funccal.get().is_null() && eval_lavars {
                flags |= FC_CLOSURE;
                unsafe { register_closure(fp) };
            } else {
                f.uf_scoped = ptr::null_mut();
            }

            if unsafe { prof_def_func() } {
                unsafe { func_do_profile(fp) };
            }
            if sandbox.get() != 0 {
                flags |= FC_SANDBOX;
            }
            f.uf_varargs = 1;
            f.uf_flags = flags;
            f.uf_calls = 0;
            f.uf_script_ctx = current_sctx.get();
            f.uf_script_ctx.sc_lnum += sourcing_lnum() - newlines.ga_len as linenr_T;

            part.pt_func = fp;
            part.pt_refcount = Refcount::ONE;
            rv.vval.v_partial = pt;
            rv.v_type = VAR_PARTIAL;
        }
        true
    };

    if !parsed {
        unsafe { ga_clear_strings(&raw mut newargs) };
    }
    eval_lavars_used.set(old_eval_lavars);
    if !evalarg.is_null() && unsafe { (*evalarg).eval_tofree }.is_null() {
        unsafe { (*evalarg).eval_tofree = tofree };
    } else {
        unsafe { xfree(tofree as *mut c_void) };
    }
    if parsed { OK } else { FAIL }
}

/// Turn `dict.Func` into a partial that binds `selfdict`, when `Func` was
/// declared with the `dict` attribute.
///
/// # Safety
/// `rettv` holds the funcref just read and `selfdict` the dictionary it came
/// out of.
pub unsafe fn make_partial(selfdict: *mut dict_T, rettv: *mut typval_T) {
    // SAFETY: the caller's promise -- `rettv` holds the funcref just read.
    let mut rv = unsafe { Tv::new(rettv) };
    let mut fp: *mut ufunc_T = ptr::null_mut();
    let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
    let mut error = 0;

    // SAFETY: the tag says which union member holds the callable, and a
    // partial in it is null or live.
    let held = unsafe { rv.vval.v_partial };
    if rv.v_type == VAR_PARTIAL && !held.is_null() && !unsafe { (*held).pt_func }.is_null() {
        fp = unsafe { (*held).pt_func };
    } else {
        let mut fname = if rv.v_type == VAR_FUNC || rv.v_type == VAR_STRING {
            unsafe { rv.vval.v_string }
        } else if held.is_null() {
            ptr::null_mut()
        } else {
            unsafe { (*held).pt_name }
        };
        if fname.is_null() {
            // There is no point binding a dict to a NULL function, just
            // create a function reference.
            rv.v_type = VAR_FUNC;
            rv.vval.v_string = ptr::null_mut();
        } else {
            // Translate "s:func" to the stored function name.
            let mut tofree: *mut c_char = ptr::null_mut();
            let buf = fname_buf.as_mut_ptr();
            let (freep, errp) = (&raw mut tofree, &raw mut error);
            fname = unsafe { fname_trans_sid(fname, buf, freep, errp) };
            fp = unsafe { find_func(fname) };
            unsafe { xfree(tofree as *mut c_void) };
        }
    }

    if fp.is_null() || unsafe { (*fp).uf_flags } & FC_DICT == 0 {
        return;
    }
    let pt = unsafe { xcalloc(1, size_of::<partial_T>()) } as *mut partial_T;
    // SAFETY: a fresh partial of this call's own, and `selfdict` is the
    // dictionary the funcref came out of.
    let mut part = unsafe { Live::new(pt) };
    part.pt_refcount = Refcount::ONE;
    part.pt_dict = selfdict;
    unsafe { (*selfdict).dv_refcount.retain() };
    part.pt_auto = true;
    if rv.v_type == VAR_FUNC || rv.v_type == VAR_STRING {
        // Just a function: take over the function name and use selfdict.
        part.pt_name = unsafe { rv.vval.v_string };
    } else {
        // Partial: copy the function name, use selfdict and copy the
        // arguments.  Neither can be taken over, because the partial may
        // be referenced elsewhere.
        // SAFETY: the tag says the union holds a live partial.
        let ret_pt = unsafe { Live::new(rv.vval.v_partial) };
        if !ret_pt.pt_name.is_null() {
            part.pt_name = unsafe { xstrdup(ret_pt.pt_name) };
            unsafe { func_ref(part.pt_name) };
        } else {
            part.pt_func = ret_pt.pt_func;
            unsafe { func_ptr_ref(part.pt_func) };
        }
        if ret_pt.pt_argc > 0 {
            let arg_size = size_of::<typval_T>().wrapping_mul(ret_pt.pt_argc as size_t);
            part.pt_argv = unsafe { xmalloc(arg_size) } as *mut typval_T;
            part.pt_argc = ret_pt.pt_argc;
            let (from, into) = (ret_pt.pt_argv, part.pt_argv);
            for i in 0..part.pt_argc {
                unsafe { tv_copy(from.offset(i as isize), into.offset(i as isize)) };
            }
        }
        unsafe { partial_unref(ret_pt.raw()) };
    }
    rv.v_type = VAR_PARTIAL;
    rv.vval.v_partial = pt;
}

/// Wrap a Lua reference in a `ufunc_T`, so that Vimscript can call it by
/// name.  Answers that name.
///
/// # Safety
/// `ref_0` is a live Lua reference the new function takes over.
pub unsafe fn register_luafunc(ref_0: LuaRef) -> *mut c_char {
    let mut lambda_buf = [0 as c_char; LAMBDA_NAME_LEN];
    let name = unsafe { get_lambda_name(&mut lambda_buf) };
    let fp = unsafe { alloc_ufunc(name.data(), name.len()) };
    // SAFETY: `fp` is the allocation just made.
    let mut f = unsafe { Uf::new(fp) };
    f.uf_refcount = Refcount::ONE;
    f.uf_varargs = 1;
    f.uf_flags = FC_LUAREF;
    f.uf_calls = 0;
    f.uf_script_ctx = current_sctx.get();
    f.uf_luaref = ref_0;

    let _ = unsafe { func_table().add(uf_name_ptr(fp)) };
    uf_name_ptr(fp)
}
