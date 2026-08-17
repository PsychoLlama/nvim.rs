//! Choosing what a call *is*, before any of it happens.
//!
//! `call_func` is the one entry point every caller of anything callable
//! reaches: a partial, a `v:lua` reference, a user function, an autoloaded
//! one, or a builtin.  `get_func_tv` is the expression-level wrapper that
//! parses the argument list first, and `func_call` the one that takes the
//! arguments already built as a list.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

#[allow(unused_imports)]
use super::*;

/// An argument array for one call: `MAX_FUNC_ARGS` values plus the slot a
/// `base->Method()` base is put in front of them.
const ARGV_INIT: [typval_T; MAX_FUNC_ARGS as usize + 1] =
    [TV_INITIAL_VALUE; MAX_FUNC_ARGS as usize + 1];

/// Evaluate a call written as an expression: read `(a, b)` at `*arg`, then
/// make the call.
///
/// # Safety
/// `name` has `len` readable bytes, `*arg` points at the `(`, and `funcexe`
/// describes the call.
pub unsafe fn get_func_tv(
    name: *const c_char,
    len: c_int,
    rettv: *mut typval_T,
    arg: *mut *mut c_char,
    evalarg: *mut evalarg_T,
    funcexe: *mut funcexe_T,
) -> c_int {
    unsafe {
        let mut argvars = ARGV_INIT;
        let mut argcount = 0;
        let evaluate = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE != 0;

        // Get the arguments.
        let mut argp = *arg;
        let mut ret = get_func_arguments(
            &raw mut argp,
            evalarg,
            if (*funcexe).fe_partial.is_null() {
                0
            } else {
                (*(*funcexe).fe_partial).pt_argc
            },
            argvars.as_mut_ptr(),
            &raw mut argcount,
        );
        debug_assert!(ret == OK || ret == FAIL);

        if ret == OK {
            let mut i = 0;
            if get_vim_var_nr(VV_TESTING) != 0 {
                // Prepare for calling `test_garbagecollect_now()`, which
                // needs to know which variables are used on the call stack.
                if (*funcargs.ptr()).ga_itemsize == 0 {
                    ga_init(funcargs.ptr(), size_of::<*mut typval_T>() as c_int, 50);
                }
                while i < argcount {
                    ga_grow(funcargs.ptr(), 1);
                    let ga = funcargs.ptr();
                    *((*ga).ga_data as *mut *mut typval_T).offset((*ga).ga_len as isize) =
                        argvars.as_mut_ptr().offset(i as isize);
                    (*ga).ga_len += 1;
                    i += 1;
                }
            }
            ret = call_func(name, len, rettv, argcount, argvars.as_mut_ptr(), funcexe);
            (*funcargs.ptr()).ga_len -= i;
        } else if !aborting() && evaluate {
            if argcount == MAX_FUNC_ARGS {
                emsg_funcname(c"E740: Too many arguments for function %s".as_ptr(), name);
            } else {
                emsg_funcname(c"E116: Invalid arguments for function %s".as_ptr(), name);
            }
        }

        while argcount > 0 {
            argcount -= 1;
            tv_clear(argvars.as_mut_ptr().offset(argcount as isize));
        }

        *arg = skipwhite(argp);
        ret
    }
}

/// Call `name` with the arguments already built as a list, which is what
/// `call()` and the callbacks do.
///
/// # Safety
/// `name` is NUL-terminated and `args` holds a list (or nothing).
pub unsafe fn func_call(
    name: *mut c_char,
    args: *mut typval_T,
    partial: *mut partial_T,
    selfdict: *mut dict_T,
    rettv: *mut typval_T,
) -> c_int {
    unsafe {
        let mut argv = ARGV_INIT;
        let mut argc = 0;
        let mut r = 0;

        'skip_call: {
            let bound = if partial.is_null() {
                0
            } else {
                (*partial).pt_argc
            };
            for item in tv_list_iter((*args).vval.v_list.as_ref()) {
                if argc == MAX_FUNC_ARGS - bound {
                    emsg(gettext(c"E699: Too many arguments".as_ptr()));
                    break 'skip_call;
                }
                // Copy each argument, so that `v_lock` can be set to
                // VAR_FIXED in the copy without changing the original list.
                tv_copy(
                    &raw mut (*item).li_tv,
                    argv.as_mut_ptr().offset(argc as isize),
                );
                argc += 1;
            }

            let mut funcexe = FUNCEXE_INIT;
            funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_evaluate = true;
            funcexe.fe_partial = partial;
            funcexe.fe_selfdict = selfdict;
            r = call_func(name, -1, rettv, argc, argv.as_mut_ptr(), &raw mut funcexe);
        }

        while argc > 0 {
            argc -= 1;
            tv_clear(argv.as_mut_ptr().offset(argc as isize));
        }
        r
    }
}

/// Call a callback and take its answer as a number; -2 when the call itself
/// failed.
///
/// # Safety
/// `callback` is live and `argvars` holds `argcount` values.
pub unsafe fn callback_call_retnr(
    callback: *mut Callback,
    argcount: c_int,
    argvars: *mut typval_T,
) -> varnumber_T {
    unsafe {
        let mut rettv = TV_INITIAL_VALUE;
        if !callback_call(callback, argcount, argvars, &raw mut rettv) {
            return -2;
        }
        let retval = tv_get_number_chk(&raw mut rettv, ptr::null_mut());
        tv_clear(&raw mut rettv);
        retval
    }
}

/// Make a call: resolve `funcname` to a partial, a `v:lua` reference, a user
/// function (autoloading one if need be) or a builtin, and run it.
///
/// # Safety
/// `funcname` has `len` readable bytes (or is NUL-terminated when `len` is
/// not positive), `argvars_in` holds `argcount_in` values, and `funcexe`
/// describes the call.
pub unsafe fn call_func(
    mut funcname: *const c_char,
    mut len: c_int,
    rettv: *mut typval_T,
    argcount_in: c_int,
    argvars_in: *mut typval_T,
    funcexe: *mut funcexe_T,
) -> c_int {
    unsafe {
        let mut ret = FAIL;
        let mut error = FCERR_NONE;
        let mut fp: *mut ufunc_T = ptr::null_mut();
        let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
        let mut tofree: *mut c_char = ptr::null_mut();
        let mut fname: *mut c_char = ptr::null_mut();
        let mut name: *mut c_char = ptr::null_mut();
        let mut argcount = argcount_in;
        let mut argvars = argvars_in;
        let mut selfdict = (*funcexe).fe_selfdict;
        // Used when a partial or `fe_basetv` puts arguments in front.
        let mut argv = ARGV_INIT;
        let mut argv_clear = 0;
        let mut argv_base = 0;
        let partial = (*funcexe).fe_partial;

        // Initialise rettv so that the caller may `tv_clear` it even when
        // this answers FAIL.
        (*rettv).v_type = VAR_UNKNOWN;

        if len <= 0 {
            len = strlen(funcname) as c_int;
        }
        if !partial.is_null() {
            fp = (*partial).pt_func;
        }
        if fp.is_null() {
            // Copy the name: if it comes from a funcref variable it could be
            // changed or deleted inside the called function.
            name = xmemdupz(funcname as *const c_void, len as size_t) as *mut c_char;
            fname = fname_trans_sid(
                name,
                fname_buf.as_mut_ptr(),
                &raw mut tofree,
                &raw mut error,
            );
        }
        if !(*funcexe).fe_doesrange.is_null() {
            *(*funcexe).fe_doesrange = false;
        }

        'theend: {
            if !partial.is_null() {
                // When the function has a partial with a dict and there is a
                // dict argument, use the dict argument -- that is backwards
                // compatible.  When the dict was bound explicitly, use the
                // partial's.
                if !(*partial).pt_dict.is_null() && (selfdict.is_null() || !(*partial).pt_auto) {
                    selfdict = (*partial).pt_dict;
                }
                if error == FCERR_NONE && (*partial).pt_argc > 0 {
                    while argv_clear < (*partial).pt_argc {
                        if argv_clear + argcount_in >= MAX_FUNC_ARGS {
                            error = FCERR_TOOMANY;
                            break 'theend;
                        }
                        tv_copy(
                            (*partial).pt_argv.offset(argv_clear as isize),
                            argv.as_mut_ptr().offset(argv_clear as isize),
                        );
                        argv_clear += 1;
                    }
                    for i in 0..argcount_in {
                        argv[(i + argv_clear) as usize] = *argvars_in.offset(i as isize);
                    }
                    argvars = argv.as_mut_ptr();
                    argcount = (*partial).pt_argc + argcount_in;
                }
            }

            if error == FCERR_NONE && (*funcexe).fe_evaluate {
                // Skip "g:" before a function name.
                let is_global =
                    fp.is_null() && *fname == b'g' as c_char && *fname.add(1) == b':' as c_char;
                let rfname = if is_global { fname.add(2) } else { fname };

                (*rettv).v_type = VAR_NUMBER; // the default is number zero
                (*rettv).vval.v_number = 0;
                error = FCERR_UNKNOWN;

                if is_luafunc(partial) {
                    if len > 0 {
                        error = FCERR_NONE;
                        argv_add_base(
                            (*funcexe).fe_basetv,
                            &raw mut argvars,
                            &raw mut argcount,
                            argv.as_mut_ptr(),
                            &raw mut argv_base,
                        );
                        nlua_typval_call(funcname, len as size_t, argvars, argcount, rettv);
                    } else {
                        // v:lua was called directly; show its name in the
                        // message.
                        xfree(name as *mut c_void);
                        name = ptr::null_mut();
                        funcname = c"v:lua".as_ptr();
                    }
                } else if !fp.is_null() || !builtin_function(rfname, -1) {
                    // A user-defined function.
                    if fp.is_null() {
                        fp = find_func(rfname);
                    }

                    // Trigger FuncUndefined, which may load the function.
                    if fp.is_null()
                        && apply_autocmds(
                            EVENT_FUNCUNDEFINED,
                            rfname,
                            rfname,
                            true,
                            ptr::null_mut(),
                        )
                        && !aborting()
                    {
                        fp = find_func(rfname);
                    }
                    // Try loading a package.  Reached by every spelling that
                    // does *not* go through `deref_func_name` first --
                    // `call()`, `nvim_call_function`, `vim.fn` -- because
                    // that one's `find_var` has already sourced it.
                    if fp.is_null() && script_autoload(rfname, strlen(rfname), true) && !aborting()
                    {
                        fp = find_func(rfname);
                    }

                    if !fp.is_null() && (*fp).uf_flags & FC_DELETED != 0 {
                        error = FCERR_DELETED;
                    } else if !fp.is_null() {
                        if let Some(argv_func) = (*funcexe).fe_argv_func {
                            // Postponed filling in the arguments; do it now.
                            argcount = argv_func(argcount, argvars, argv_clear, fp);
                        }
                        argv_add_base(
                            (*funcexe).fe_basetv,
                            &raw mut argvars,
                            &raw mut argcount,
                            argv.as_mut_ptr(),
                            &raw mut argv_base,
                        );
                        error =
                            call_user_func_check(fp, argcount, argvars, rettv, funcexe, selfdict);
                    }
                } else if !(*funcexe).fe_basetv.is_null() {
                    // expr->method(): find the method name in the table and
                    // call it with the base as one of the arguments.
                    error =
                        call_internal_method(fname, argcount, argvars, rettv, (*funcexe).fe_basetv);
                } else {
                    // Find the function name in the table and call it.
                    error = call_internal_func(fname, argcount, argvars, rettv);
                }

                // The call (or the FuncUndefined autocommand sequence) may
                // have been aborted by an error, an interrupt, or an
                // uncaught exception, which `aborting()` reports.  For an
                // error inside an internal function, or for E132 in
                // `call_user_func`, the throw point where `force_abort` is
                // normally updated has not been reached yet, so update it
                // here to make `aborting()` reliable.
                update_force_abort();
            }
            if error == FCERR_NONE {
                ret = OK;
            }
        }

        // Report an error unless evaluating the arguments or making the call
        // was cancelled by an aborting error, an interrupt or an exception.
        if !aborting() {
            user_func_error(
                error,
                if name.is_null() { funcname } else { name },
                (*funcexe).fe_found_var,
            );
        }

        // Clear the copies made from the partial.
        while argv_clear > 0 {
            argv_clear -= 1;
            tv_clear(argv.as_mut_ptr().offset((argv_clear + argv_base) as isize));
        }

        xfree(tofree as *mut c_void);
        xfree(name as *mut c_void);
        ret
    }
}
