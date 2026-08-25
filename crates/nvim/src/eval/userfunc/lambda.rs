//! Lambdas, closures and partials -- the anonymous half.
//!
//! `get_lambda_tv` parses `{x -> expr}` into a real `ufunc_T` with a
//! generated `<lambda>N` name and, if it captured anything, a reference to
//! the funccall it was made in (`register_closure`).  `make_partial` is the
//! other way a callable carries state: a bound dictionary, bound arguments,
//! or both.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::types::{FAIL, OK};

/// Give `fp` the funccall that is running as its scope, so that the locals it
/// closed over stay alive for as long as it does.
///
/// # Safety
/// `fp` is a live function and a funccall is running.
pub(crate) unsafe fn register_closure(fp: *mut ufunc_T) {
    unsafe {
        if (*fp).uf_scoped == current_funccal.get() {
            return; // no change
        }
        funccal_unref((*fp).uf_scoped, fp, false);
        let fc = current_funccal.get();
        (*fp).uf_scoped = fc;
        (*fc).fc_refcount += 1;
        ga_grow(&raw mut (*fc).fc_ufuncs, 1);
        let ufuncs = &raw mut (*fc).fc_ufuncs;
        *((*ufuncs).ga_data as *mut *mut ufunc_T).offset((*ufuncs).ga_len as isize) = fp;
        (*ufuncs).ga_len += 1;
    }
}

/// `"<lambda>"` plus `NUMBUFLEN`, the widest a `varnumber_T` prints.
const LAMBDA_NAME_LEN: usize = 8 + 65;

/// The name of the next lambda, in `into` — the caller's, so that two
/// names can be alive at once. Upstream answers one static buffer.
///
/// # Safety
/// `into` must outlive the answer.
unsafe fn get_lambda_name(into: &mut [c_char; LAMBDA_NAME_LEN]) -> String_0 {
    unsafe {
        static lambda_no: GlobalCell<c_int> = GlobalCell::new(0);
        lambda_no.set(lambda_no.get() + 1);
        let buf = into.as_mut_ptr();
        let n = snprintf(
            buf,
            LAMBDA_NAME_LEN,
            c"<lambda>%d".as_ptr(),
            lambda_no.get(),
        );
        String_0::from_raw_parts(
            buf,
            if n < 1 {
                0
            } else {
                n.min(LAMBDA_NAME_LEN as c_int - 1) as size_t
            },
        )
    }
}

/// Allocate a `ufunc_T` for a function called `name`, whose name lives in the
/// flexible member at the end of the allocation.
///
/// # Safety
/// `name` has `namelen` readable bytes.
pub(crate) unsafe fn alloc_ufunc(name: *const c_char, namelen: size_t) -> *mut ufunc_T {
    unsafe {
        let fp = xcalloc(1, offset_of!(ufunc_T, uf_name) + namelen + 1) as *mut ufunc_T;
        xmemcpyz(
            uf_name_ptr(fp) as *mut c_void,
            name as *const c_void,
            namelen,
        );
        (*fp).uf_namelen = namelen;

        if *name as u8 as c_int == K_SPECIAL {
            // A script-local name is stored mangled; keep the printable
            // "<SNR>123_name" beside it.
            let len = namelen + 3;
            (*fp).uf_name_exp = xmalloc(len) as *mut c_char;
            snprintf(
                (*fp).uf_name_exp,
                len,
                c"<SNR>%s".as_ptr(),
                uf_name_ptr(fp).add(3),
            );
        }
        fp
    }
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
    unsafe {
        let evaluate = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE != 0;
        let mut newargs = GARRAY_EMPTY;
        let mut varargs = 0;
        let old_eval_lavars = eval_lavars_used.get();
        let mut eval_lavars = false;
        let mut tofree: *mut c_char = ptr::null_mut();

        // First, check whether this is a lambda expression at all: an "->"
        // must follow a well-formed argument list.
        let mut s = skipwhite((*arg).add(1));
        if get_function_args(
            &raw mut s,
            b'-' as c_char,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            true,
        ) == FAIL
            || *s != b'>' as c_char
        {
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
            *arg = skipwhite((*arg).add(1));
            if get_function_args(
                arg,
                b'-' as c_char,
                pnewargs,
                &raw mut varargs,
                ptr::null_mut(),
                false,
            ) == FAIL
                || **arg != b'>' as c_char
            {
                break 'errret false;
            }

            // Set up a flag for checking local variables and arguments.
            if evaluate {
                eval_lavars_used.set(&raw mut eval_lavars);
            }

            // Get the start and the end of the expression.
            *arg = skipwhite((*arg).add(1));
            let start = *arg;
            let ret = skip_expr(arg, evalarg);
            let end = *arg;
            if ret == FAIL {
                break 'errret false;
            }
            if !evalarg.is_null() {
                // Avoid that the expression gets freed when another line
                // break follows.
                tofree = (*evalarg).eval_tofree;
                (*evalarg).eval_tofree = ptr::null_mut();
            }

            *arg = skipwhite(*arg);
            if **arg != b'}' as c_char {
                semsg_c!(gettext(c"E451: Expected }: %s".as_ptr()), *arg);
                break 'errret false;
            }
            *arg = (*arg).add(1);

            if evaluate {
                let mut flags = 0;
                let name = get_lambda_name(&mut lambda_buf);
                let fp = alloc_ufunc(name.data(), name.len());
                let pt = xcalloc(1, size_of::<partial_T>()) as *mut partial_T;

                let mut newlines = GARRAY_EMPTY;
                ga_init(&raw mut newlines, size_of::<*mut c_char>() as c_int, 1);
                ga_grow(&raw mut newlines, 1);

                // The body is the expression with "return " in front of it.
                const RETURN: &CStr = c"return ";
                let body_len = RETURN.count_bytes() + end.offset_from(start) as size_t + 1;
                let p = xmalloc(body_len) as *mut c_char;
                *(newlines.ga_data as *mut *mut c_char) = p;
                newlines.ga_len = 1;
                strcpy(p, RETURN.as_ptr());
                let expr = p.add(RETURN.count_bytes());
                xmemcpyz(
                    expr as *mut c_void,
                    start as *const c_void,
                    end.offset_from(start) as size_t,
                );
                if strstr(expr, c"a:".as_ptr()).is_null() {
                    // No a: variables are used for sure.
                    flags |= FC_NOARGS;
                }

                (*fp).uf_refcount = 1;
                func_table().add(uf_name_ptr(fp));
                (*fp).uf_args = newargs;
                ga_init(
                    &raw mut (*fp).uf_def_args,
                    size_of::<*mut c_char>() as c_int,
                    1,
                );
                (*fp).uf_lines = newlines;
                if !current_funccal.get().is_null() && eval_lavars {
                    flags |= FC_CLOSURE;
                    register_closure(fp);
                } else {
                    (*fp).uf_scoped = ptr::null_mut();
                }

                if prof_def_func() {
                    func_do_profile(fp);
                }
                if sandbox.get() != 0 {
                    flags |= FC_SANDBOX;
                }
                (*fp).uf_varargs = 1;
                (*fp).uf_flags = flags;
                (*fp).uf_calls = 0;
                (*fp).uf_script_ctx = current_sctx.get();
                (*fp).uf_script_ctx.sc_lnum += sourcing_lnum() - newlines.ga_len as linenr_T;

                (*pt).pt_func = fp;
                (*pt).pt_refcount = 1;
                (*rettv).vval.v_partial = pt;
                (*rettv).v_type = VAR_PARTIAL;
            }
            true
        };

        if !parsed {
            ga_clear_strings(&raw mut newargs);
        }
        eval_lavars_used.set(old_eval_lavars);
        if !evalarg.is_null() && (*evalarg).eval_tofree.is_null() {
            (*evalarg).eval_tofree = tofree;
        } else {
            xfree(tofree as *mut c_void);
        }
        if parsed { OK } else { FAIL }
    }
}

/// Turn `dict.Func` into a partial that binds `selfdict`, when `Func` was
/// declared with the `dict` attribute.
///
/// # Safety
/// `rettv` holds the funcref just read and `selfdict` the dictionary it came
/// out of.
pub unsafe fn make_partial(selfdict: *mut dict_T, rettv: *mut typval_T) {
    unsafe {
        let mut fp: *mut ufunc_T = ptr::null_mut();
        let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
        let mut error = 0;

        if (*rettv).v_type == VAR_PARTIAL
            && !(*rettv).vval.v_partial.is_null()
            && !(*(*rettv).vval.v_partial).pt_func.is_null()
        {
            fp = (*(*rettv).vval.v_partial).pt_func;
        } else {
            let mut fname = if (*rettv).v_type == VAR_FUNC || (*rettv).v_type == VAR_STRING {
                (*rettv).vval.v_string
            } else if (*rettv).vval.v_partial.is_null() {
                ptr::null_mut()
            } else {
                (*(*rettv).vval.v_partial).pt_name
            };
            if fname.is_null() {
                // There is no point binding a dict to a NULL function, just
                // create a function reference.
                (*rettv).v_type = VAR_FUNC;
                (*rettv).vval.v_string = ptr::null_mut();
            } else {
                // Translate "s:func" to the stored function name.
                let mut tofree: *mut c_char = ptr::null_mut();
                fname = fname_trans_sid(
                    fname,
                    fname_buf.as_mut_ptr(),
                    &raw mut tofree,
                    &raw mut error,
                );
                fp = find_func(fname);
                xfree(tofree as *mut c_void);
            }
        }

        if fp.is_null() || (*fp).uf_flags & FC_DICT == 0 {
            return;
        }
        let pt = xcalloc(1, size_of::<partial_T>()) as *mut partial_T;
        (*pt).pt_refcount = 1;
        (*pt).pt_dict = selfdict;
        (*selfdict).dv_refcount += 1;
        (*pt).pt_auto = true;
        if (*rettv).v_type == VAR_FUNC || (*rettv).v_type == VAR_STRING {
            // Just a function: take over the function name and use selfdict.
            (*pt).pt_name = (*rettv).vval.v_string;
        } else {
            // Partial: copy the function name, use selfdict and copy the
            // arguments.  Neither can be taken over, because the partial may
            // be referenced elsewhere.
            let ret_pt = (*rettv).vval.v_partial;
            if !(*ret_pt).pt_name.is_null() {
                (*pt).pt_name = xstrdup((*ret_pt).pt_name);
                func_ref((*pt).pt_name);
            } else {
                (*pt).pt_func = (*ret_pt).pt_func;
                func_ptr_ref((*pt).pt_func);
            }
            if (*ret_pt).pt_argc > 0 {
                let arg_size = size_of::<typval_T>().wrapping_mul((*ret_pt).pt_argc as size_t);
                (*pt).pt_argv = xmalloc(arg_size) as *mut typval_T;
                (*pt).pt_argc = (*ret_pt).pt_argc;
                for i in 0..(*pt).pt_argc {
                    tv_copy(
                        (*ret_pt).pt_argv.offset(i as isize),
                        (*pt).pt_argv.offset(i as isize),
                    );
                }
            }
            partial_unref(ret_pt);
        }
        (*rettv).v_type = VAR_PARTIAL;
        (*rettv).vval.v_partial = pt;
    }
}

/// Wrap a Lua reference in a `ufunc_T`, so that Vimscript can call it by
/// name.  Answers that name.
///
/// # Safety
/// `ref_0` is a live Lua reference the new function takes over.
pub unsafe fn register_luafunc(ref_0: LuaRef) -> *mut c_char {
    let mut lambda_buf = [0 as c_char; LAMBDA_NAME_LEN];
    unsafe {
        let name = get_lambda_name(&mut lambda_buf);
        let fp = alloc_ufunc(name.data(), name.len());
        (*fp).uf_refcount = 1;
        (*fp).uf_varargs = 1;
        (*fp).uf_flags = FC_LUAREF;
        (*fp).uf_calls = 0;
        (*fp).uf_script_ctx = current_sctx.get();
        (*fp).uf_luaref = ref_0;

        func_table().add(uf_name_ptr(fp));
        uf_name_ptr(fp)
    }
}
