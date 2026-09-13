//! Choosing what a call *is*, before any of it happens.
//!
//! `call_func` is the one entry point every caller of anything callable
//! reaches: a partial, a `v:lua` reference, a user function, an autoloaded
//! one, or a builtin.  `get_func_tv` is the expression-level wrapper that
//! parses the argument list first, and `func_call` the one that takes the
//! arguments already built as a list.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::eval::typval::CallFrame;
use crate::types::Failed;

/// Evaluate a call written as an expression: read `(a, b)` at `*arg`, then
/// make the call.
///
/// # Safety
/// `name` has `len` readable bytes, `*arg` points at the `(`, and `funcexe`
/// describes the call.
pub unsafe fn get_func_tv(
    name: *const c_char,
    len: c_int,
    result: &mut TypVal,
    arg: *mut *mut c_char,
    evalarg: *mut EvalArg,
    funcexe: *mut FuncExe,
) -> Result<(), Failed> {
    let mut argvars = Argv::new();
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
    let argpp = &raw mut argp;
    let mut ret =
        unsafe { get_func_arguments(argpp, evalarg, bound, argvars.room(), &mut argcount) };
    // A failed argument leaves whatever it half-built in the slot it was
    // being evaluated into, which upstream leaks and the frame releases.
    argvars.fill(if ret.is_ok() {
        argcount
    } else {
        (argcount + 1).min(MAX_FUNC_ARGS as usize + 1)
    });
    debug_assert!(ret.is_ok() || ret.is_err());

    if ret.is_ok() {
        // Prepare for calling `test_garbagecollect_now()`, which needs to
        // know which variables are used on the call stack.
        let pushed = if get_vim_var_nr(Vv::Testing) != 0 {
            funcargs.with_mut(|args| {
                args.extend(
                    argvars.args()[..argcount]
                        .iter()
                        .map(|tv| ptr::from_ref(tv).cast_mut()),
                );
            });
            argcount
        } else {
            0
        };
        // SAFETY: the caller's promise -- `result` is the return value.
        let rv = &mut *result;
        ret = unsafe { call_func(name, len, rv, &argvars.args()[..argcount], funcexe) };
        // The nested calls pushed and popped their own; ours are the last.
        funcargs.with_mut(|args| args.truncate(args.len().saturating_sub(pushed)));
    } else if !aborting() && evaluate {
        if argcount == MAX_FUNC_ARGS as usize {
            unsafe { emsg_funcname(c"E740: Too many arguments for function %s".as_ptr(), name) };
        } else {
            unsafe { emsg_funcname(c"E116: Invalid arguments for function %s".as_ptr(), name) };
        }
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
    args: &TypVal,
    partial: *mut Partial,
    selfdict: *mut Dict,
    result: &mut TypVal,
) -> Result<(), Failed> {
    let mut argv = Argv::new();
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
        for item in list_iter(items) {
            if argc == (MAX_FUNC_ARGS - bound) as usize {
                emsg(gettext(c"E699: Too many arguments"));
                break 'skip_call;
            }
            // Copy each argument, so that `v_lock` can be set to
            // VarLock::Fixed in the copy without changing the original list.
            tv_copy(&item.li_tv, argv.claim());
            argc += 1;
        }

        let mut funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = Win::current().w_cursor.lnum;
        funcexe.fe_lastline = Win::current().w_cursor.lnum;
        funcexe.fe_evaluate = true;
        funcexe.fe_partial = partial;
        funcexe.fe_selfdict = selfdict;
        // SAFETY: the caller's promise -- `result` is the return value.
        let rv = &mut *result;
        r = unsafe { call_func(name, -1, rv, argv.args(), &raw mut funcexe) };
    }
    r
}

/// Call a callback and take its answer as a number; -2 when the call itself
/// failed.
///
/// # Safety
/// `callback` is live.
pub unsafe fn callback_call_retnr(callback: *mut Callback, args: &[TypVal]) -> VarNumber {
    let mut rettv = TV_INITIAL_VALUE;
    if !unsafe { callback_call(callback, args, &mut rettv) } {
        return -2;
    }
    let retval = tv_get_number_chk(&rettv).unwrap_or(-1);
    tv_clear(&mut rettv);
    retval
}

/// The argument frame one call is spliced into: `MAX_FUNC_ARGS` values plus
/// the slot a `base->Method()` base is put in front of them.
type Argv = CallFrame<{ MAX_FUNC_ARGS as usize + 1 }>;

/// The argument list as it stands: the frame, once something has been put in
/// front of the caller's values; the caller's own slice until then.
///
/// The frame is an `Option` so that the ordinary call -- nothing in front of
/// the caller's values -- never builds one: a `CallFrame` has a destructor,
/// so a local of one cannot have its initialisation sunk into the branch
/// that needs it, and 21 slots is `MAX_FUNC_ARGS + 1`.
fn spliced_args<'a>(argv: &'a Option<Argv>, args: &'a [TypVal], n: usize) -> &'a [TypVal] {
    match argv {
        Some(argv) => argv.args(),
        None => &args[..n],
    }
}

/// Put a partial's bound arguments in front of the caller's own.
///
/// The bound values are *copied*: the call may free the partial, and the
/// frame outlives it either way.  Answers `Err` when the two lists together
/// are more arguments than a call can take.
///
/// # Safety
/// `partial` is a live partial with `pt_argc` bound arguments.
unsafe fn splice_bound(
    argv: &mut Argv,
    partial: *const Partial,
    args: &[TypVal],
) -> Result<(), ()> {
    let bound = unsafe { (*partial).pt_argc } as usize;
    if bound + args.len() > MAX_FUNC_ARGS as usize {
        return Err(());
    }
    for i in 0..bound {
        // SAFETY: the caller's promise -- `pt_argv` holds `pt_argc` values.
        argv.push_owned(unsafe { (*(*partial).pt_argv.add(i)).clone() });
    }
    argv.extend_borrowed(args);
    Ok(())
}

/// Put the base of a `base->Method()` call in front of the argument list,
/// moving the caller's own values into the frame when nothing has yet.
///
/// Answers `Err` when there is no room for it.
fn splice_base(argv: &mut Option<Argv>, args: &[TypVal], base: &TypVal) -> Result<(), ()> {
    let argv = argv.get_or_insert_with(|| {
        let mut frame = Argv::new();
        frame.extend_borrowed(args);
        frame
    });
    if argv.is_full() {
        return Err(());
    }
    argv.insert_borrowed_front(base);
    Ok(())
}

/// Make a call: resolve `funcname` to a partial, a `v:lua` reference, a user
/// function (autoloading one if need be) or a builtin, and run it.
///
/// # Safety
/// `funcname` has `len` readable bytes (or is NUL-terminated when `len` is
/// not positive) and `funcexe` describes the call.
pub unsafe fn call_func(
    mut funcname: *const c_char,
    mut len: c_int,
    result: &mut TypVal,
    args_in: &[TypVal],
    funcexe: *mut FuncExe,
) -> Result<(), Failed> {
    let mut ret = Err(Failed);
    let mut error = FCERR_NONE;
    let mut fp: *mut UserFunc = ptr::null_mut();
    let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
    let mut tofree: *mut c_char = ptr::null_mut();
    let mut fname: *mut c_char = ptr::null_mut();
    let mut name: *mut c_char = ptr::null_mut();
    let mut selfdict = unsafe { (*funcexe).fe_selfdict };
    // How much of `args_in` is still the argument list, once an
    // `fe_argv_func` has had its say.
    let mut nargs = args_in.len();
    // Built only when a partial or `fe_basetv` puts arguments in front;
    // until then the caller's own slice *is* the argument list.
    let mut argv: Option<Argv> = None;
    let partial = unsafe { (*funcexe).fe_partial };

    // Initialise rettv so that the caller may `tv_clear` it even when
    // this answers FAIL.
    result.write_empty(VAR_UNKNOWN);

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
                let mut frame = Argv::new();
                // SAFETY: `funcexe`'s partial is live.
                if unsafe { splice_bound(&mut frame, partial, args_in) }.is_err() {
                    error = FCERR_TOOMANY;
                    break 'theend;
                }
                argv = Some(frame);
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

            // the default is number zero
            result.write_number(0);
            error = FCERR_UNKNOWN;

            if unsafe { is_luafunc(partial) } {
                if len > 0 {
                    error = FCERR_NONE;
                    // SAFETY: `funcexe`'s base is null or a live typval.
                    if let Some(base) = unsafe { (*funcexe).fe_basetv.as_ref() }
                        && splice_base(&mut argv, &args_in[..nargs], base).is_err()
                    {
                        error = FCERR_TOOMANY;
                        break 'theend;
                    }
                    let len = len as size_t;
                    let args = spliced_args(&argv, args_in, nargs);
                    unsafe { nlua_typval_call(funcname, len, args, result) };
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
                if fp.is_null()
                    && unsafe { apply_autocmds(event, rfname, rfname, true, None) }
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
                        let filled = argv.as_ref().map_or(0, Argv::len);
                        let skip = filled.saturating_sub(args_in.len());
                        // SAFETY: `fp` is the live function being called.
                        let args = spliced_args(&argv, args_in, nargs);
                        let n = unsafe { argv_func(args, skip, fp) };
                        match &mut argv {
                            Some(argv) => argv.truncate(n),
                            None => nargs = n,
                        }
                    }
                    // SAFETY: as the `v:lua` branch above.
                    if let Some(base) = unsafe { (*funcexe).fe_basetv.as_ref() }
                        && splice_base(&mut argv, &args_in[..nargs], base).is_err()
                    {
                        error = FCERR_TOOMANY;
                        break 'theend;
                    }
                    let args = spliced_args(&argv, args_in, nargs);
                    error = unsafe { call_user_func_check(fp, args, result, funcexe, selfdict) };
                }
            } else {
                // SAFETY: as the two calls above.
                let base = unsafe { (*funcexe).fe_basetv };
                let args = spliced_args(&argv, args_in, nargs);
                error = if base.is_null() {
                    unsafe { call_internal_func(fname, args, result) }
                } else {
                    unsafe { call_internal_method(fname, args, result, &mut *base) }
                };
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

    // The copies made from the partial go with the frame.
    unsafe { xfree(tofree as *mut c_void) };
    unsafe { xfree(name as *mut c_void) };
    ret
}
