//! Choosing what a call *is*, before any of it happens.
//!
//! `call_func` is the one entry point every caller of anything callable
//! reaches: a partial, a `v:lua` reference, a user function, an autoloaded
//! one, or a builtin.  `get_func_tv` is the expression-level wrapper that
//! parses the argument list first, and `func_call` the one that takes the
//! arguments already built as a list.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::types::Failed;

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
) -> Result<(), Failed> {
    let mut argvars = ARGV_INIT;
    let mut argcount = 0;
    let evaluate = !evalarg.is_null() && unsafe { (*evalarg).eval_flags } & EVAL_EVALUATE != 0;

    // Get the arguments.
    let mut argp = unsafe { *arg };
    // SAFETY: the caller's promise -- `funcexe` describes the call, so its
    // partial is null or live.
    let bound = unsafe {
        if (*funcexe).fe_partial.is_null() {
            0
        } else {
            (*(*funcexe).fe_partial).pt_argc
        }
    };
    let (argpp, args, countp) = (&raw mut argp, argvars.as_mut_ptr(), &raw mut argcount);
    let mut ret = unsafe { get_func_arguments(argpp, evalarg, bound, args, countp) };
    debug_assert!(ret.is_ok() || ret.is_err());

    if ret.is_ok() {
        // Prepare for calling `test_garbagecollect_now()`, which needs to
        // know which variables are used on the call stack.
        let pushed = if unsafe { get_vim_var_nr(Vv::Testing) } != 0 {
            funcargs.with_mut(|args| {
                args.extend(
                    (0..argcount).map(|i| unsafe { argvars.as_mut_ptr().offset(i as isize) }),
                );
            });
            argcount as usize
        } else {
            0
        };
        ret = unsafe { call_func(name, len, rettv, argcount, argvars.as_mut_ptr(), funcexe) };
        // The nested calls pushed and popped their own; ours are the last.
        funcargs.with_mut(|args| args.truncate(args.len().saturating_sub(pushed)));
    } else if !aborting() && evaluate {
        if argcount == MAX_FUNC_ARGS {
            unsafe { emsg_funcname(c"E740: Too many arguments for function %s".as_ptr(), name) };
        } else {
            unsafe { emsg_funcname(c"E116: Invalid arguments for function %s".as_ptr(), name) };
        }
    }

    while argcount > 0 {
        argcount -= 1;
        unsafe { tv_clear(argvars.as_mut_ptr().offset(argcount as isize)) };
    }

    unsafe { *arg = skipwhite(argp) };
    ret
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
) -> Result<(), Failed> {
    let mut argv = ARGV_INIT;
    let mut argc = 0;
    let mut r = Ok(());

    'skip_call: {
        let bound = if partial.is_null() {
            0
        } else {
            unsafe { (*partial).pt_argc }
        };
        // SAFETY: the caller's promise -- `args` holds a List or nothing.
        let items = unsafe { (*args).list_or_null().as_ref() };
        for item in tv_list_iter(items) {
            if argc == MAX_FUNC_ARGS - bound {
                emsg(gettext(c"E699: Too many arguments"));
                break 'skip_call;
            }
            // Copy each argument, so that `v_lock` can be set to
            // VarLock::Fixed in the copy without changing the original list.
            let (from, into) = unsafe { (&raw mut (*item).li_tv, argv.as_mut_ptr()) };
            unsafe { tv_copy(from, into.offset(argc as isize)) };
            argc += 1;
        }

        let mut funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = cur_win().w_cursor.lnum;
        funcexe.fe_lastline = cur_win().w_cursor.lnum;
        funcexe.fe_evaluate = true;
        funcexe.fe_partial = partial;
        funcexe.fe_selfdict = selfdict;
        r = unsafe { call_func(name, -1, rettv, argc, argv.as_mut_ptr(), &raw mut funcexe) };
    }

    while argc > 0 {
        argc -= 1;
        unsafe { tv_clear(argv.as_mut_ptr().offset(argc as isize)) };
    }
    r
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
    let mut rettv = TV_INITIAL_VALUE;
    if !unsafe { callback_call(callback, argcount, argvars, &raw mut rettv) } {
        return -2;
    }
    let retval = unsafe { tv_get_number_chk(&raw mut rettv, ptr::null_mut()) };
    unsafe { tv_clear(&raw mut rettv) };
    retval
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
) -> Result<(), Failed> {
    let mut ret = Err(Failed);
    let mut error = FCERR_NONE;
    let mut fp: *mut ufunc_T = ptr::null_mut();
    let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
    let mut tofree: *mut c_char = ptr::null_mut();
    let mut fname: *mut c_char = ptr::null_mut();
    let mut name: *mut c_char = ptr::null_mut();
    let mut argcount = argcount_in;
    let mut argvars = argvars_in;
    let mut selfdict = unsafe { (*funcexe).fe_selfdict };
    // Used when a partial or `fe_basetv` puts arguments in front.
    let mut argv = ARGV_INIT;
    let mut argv_clear = 0;
    let mut argv_base = 0;
    let partial = unsafe { (*funcexe).fe_partial };

    // Initialise rettv so that the caller may `tv_clear` it even when
    // this answers FAIL.
    unsafe { (*rettv).v_type = VAR_UNKNOWN };

    if len <= 0 {
        len = unsafe { cstr::bytes_at(funcname) }.len() as c_int;
    }
    if !partial.is_null() {
        fp = unsafe { (*partial).pt_func };
    }
    if fp.is_null() {
        // Copy the name: if it comes from a funcref variable it could be
        // changed or deleted inside the called function.
        name = unsafe { xmemdupz(funcname as *const c_void, len as size_t) } as *mut c_char;
        let buf = fname_buf.as_mut_ptr();
        let (freep, errp) = (&raw mut tofree, &raw mut error);
        fname = unsafe { fname_trans_sid(name, buf, freep, errp) };
    }
    if !unsafe { (*funcexe).fe_doesrange }.is_null() {
        unsafe { *(*funcexe).fe_doesrange = false };
    }

    'theend: {
        if !partial.is_null() {
            // When the function has a partial with a dict and there is a
            // dict argument, use the dict argument -- that is backwards
            // compatible.  When the dict was bound explicitly, use the
            // partial's.
            if !unsafe { (*partial).pt_dict }.is_null()
                && (selfdict.is_null() || !unsafe { (*partial).pt_auto })
            {
                selfdict = unsafe { (*partial).pt_dict };
            }
            if error == FCERR_NONE && unsafe { (*partial).pt_argc } > 0 {
                while argv_clear < unsafe { (*partial).pt_argc } {
                    if argv_clear + argcount_in >= MAX_FUNC_ARGS {
                        error = FCERR_TOOMANY;
                        break 'theend;
                    }
                    let bound = unsafe { (*partial).pt_argv };
                    let at = argv_clear as isize;
                    unsafe { tv_copy(bound.offset(at), argv.as_mut_ptr().offset(at)) };
                    argv_clear += 1;
                }
                for i in 0..argcount_in {
                    argv[(i + argv_clear) as usize] = unsafe { *argvars_in.offset(i as isize) };
                }
                argvars = argv.as_mut_ptr();
                argcount = unsafe { (*partial).pt_argc } + argcount_in;
            }
        }

        if error == FCERR_NONE && unsafe { (*funcexe).fe_evaluate } {
            // Skip "g:" before a function name.
            let is_global = fp.is_null()
                && unsafe { *fname } == b'g' as c_char
                && unsafe { *fname.add(1) } == b':' as c_char;
            let rfname = if is_global {
                unsafe { fname.add(2) }
            } else {
                fname
            };

            unsafe { (*rettv).v_type = VAR_NUMBER }; // the default is number zero
            unsafe { (*rettv).vval.v_number = 0 };
            error = FCERR_UNKNOWN;

            if unsafe { is_luafunc(partial) } {
                if len > 0 {
                    error = FCERR_NONE;
                    // SAFETY: `funcexe`'s base is null or valid and the
                    // three out-parameters are this frame's locals.
                    let base = unsafe { (*funcexe).fe_basetv };
                    let (argsp, countp) = (&raw mut argvars, &raw mut argcount);
                    let (into, basep) = (argv.as_mut_ptr(), &raw mut argv_base);
                    unsafe { argv_add_base(base, argsp, countp, into, basep) };
                    unsafe { nlua_typval_call(funcname, len as size_t, argvars, argcount, rettv) };
                } else {
                    // v:lua was called directly; show its name in the
                    // message.
                    unsafe { xfree(name as *mut c_void) };
                    name = ptr::null_mut();
                    funcname = c"v:lua".as_ptr();
                }
            } else if !fp.is_null() || !unsafe { builtin_function(rfname, -1) } {
                // A user-defined function.
                if fp.is_null() {
                    fp = unsafe { find_func(rfname) };
                }

                // Trigger FuncUndefined, which may load the function.
                let event = AutoEvent::FuncUndefined;
                let no_buf = ptr::null_mut();
                if fp.is_null()
                    && unsafe { apply_autocmds(event, rfname, rfname, true, no_buf) }
                    && !aborting()
                {
                    fp = unsafe { find_func(rfname) };
                }
                // Try loading a package.  Reached by every spelling that
                // does *not* go through `deref_func_name` first --
                // `call()`, `nvim_call_function`, `vim.fn` -- because
                // that one's `find_var` has already sourced it.
                if fp.is_null()
                    && unsafe { script_autoload(rfname, cstr::bytes_at(rfname).len(), true) }
                    && !aborting()
                {
                    fp = unsafe { find_func(rfname) };
                }

                if !fp.is_null() && unsafe { (*fp).uf_flags }.has(FuncFlags::DELETED) {
                    error = FCERR_DELETED;
                } else if !fp.is_null() {
                    if let Some(argv_func) = unsafe { (*funcexe).fe_argv_func } {
                        // Postponed filling in the arguments; do it now.
                        argcount = unsafe { argv_func(argcount, argvars, argv_clear, fp) };
                    }
                    // SAFETY: as the `v:lua` branch above.
                    let base = unsafe { (*funcexe).fe_basetv };
                    let (argsp, countp) = (&raw mut argvars, &raw mut argcount);
                    let (into, basep) = (argv.as_mut_ptr(), &raw mut argv_base);
                    unsafe { argv_add_base(base, argsp, countp, into, basep) };
                    let args = argvars;
                    error = unsafe {
                        call_user_func_check(fp, argcount, args, rettv, funcexe, selfdict)
                    };
                }
            } else if !unsafe { (*funcexe).fe_basetv }.is_null() {
                // expr->method(): find the method name in the table and
                // call it with the base as one of the arguments.
                error = unsafe {
                    call_internal_method(fname, argcount, argvars, rettv, (*funcexe).fe_basetv)
                };
            } else {
                // Find the function name in the table and call it.
                error = unsafe { call_internal_func(fname, argcount, argvars, rettv) };
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
            ret = Ok(());
        }
    }

    // Report an error unless evaluating the arguments or making the call
    // was cancelled by an aborting error, an interrupt or an exception.
    if !aborting() {
        let what = if name.is_null() { funcname } else { name };
        // SAFETY: `funcexe` is the caller's own.
        let found = unsafe { (*funcexe).fe_found_var };
        unsafe { user_func_error(error, what, found) };
    }

    // Clear the copies made from the partial.
    while argv_clear > 0 {
        argv_clear -= 1;
        unsafe { tv_clear(argv.as_mut_ptr().offset((argv_clear + argv_base) as isize)) };
    }

    unsafe { xfree(tofree as *mut c_void) };
    unsafe { xfree(name as *mut c_void) };
    ret
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
