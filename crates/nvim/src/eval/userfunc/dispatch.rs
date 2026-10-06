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
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::eval::typval::CallFrame;
use crate::eval::typval::{DictRef, PartialRef};
use crate::types::Failed;
use std::borrow::Cow;

/// What a call is made with besides its name and its arguments, as borrows
/// the caller holds for the length of the call: the safe face of
/// [`FuncExe`], whose raw pointers are taken from these and live no longer.
pub(crate) struct CallWith<'a> {
    /// The range a function with the `range` attribute is handed.
    pub(crate) firstline: LineNr,
    pub(crate) lastline: LineNr,
    /// False to parse the arguments without making the call.
    pub(crate) evaluate: bool,
    /// The partial the callee came out of: bound arguments, a bound `self`.
    pub(crate) partial: Option<&'a PartialRef>,
    /// The dictionary a `dict.Func()` call is a method of.
    pub(crate) selfdict: Option<&'a DictRef>,
    /// The `expr` of `expr->method()`, passed as the first argument.
    pub(crate) basetv: Option<&'a mut TypVal>,
    /// Told whether the function handled the range itself (`:call`).
    pub(crate) doesrange: Option<&'a mut bool>,
    /// The name was found as a variable: a missing function is not then
    /// autoloaded.
    pub(crate) found_var: bool,
}

impl<'a> CallWith<'a> {
    /// A call with nothing bound and no range.
    pub(crate) fn new(evaluate: bool) -> Self {
        CallWith {
            firstline: 0,
            lastline: 0,
            evaluate,
            partial: None,
            selfdict: None,
            basetv: None,
            doesrange: None,
            found_var: false,
        }
    }

    /// A call with nothing bound, over the cursor line.
    pub(crate) fn at_cursor(evaluate: bool) -> Self {
        let lnum = Win::current().w_cursor.lnum;
        CallWith {
            firstline: lnum,
            lastline: lnum,
            ..CallWith::new(evaluate)
        }
    }

    /// The `FuncExe` these borrows describe. Its pointers are only good for
    /// as long as `self` is borrowed.
    fn funcexe(&mut self) -> FuncExe {
        FuncExe {
            fe_firstline: self.firstline,
            fe_lastline: self.lastline,
            fe_evaluate: self.evaluate,
            fe_partial: self.partial.map_or(ptr::null_mut(), PartialRef::as_ptr),
            fe_selfdict: self.selfdict.map_or(ptr::null_mut(), DictRef::as_ptr),
            fe_basetv: self
                .basetv
                .as_deref_mut()
                .map_or(ptr::null_mut(), ptr::from_mut),
            fe_doesrange: self
                .doesrange
                .as_deref_mut()
                .map_or(ptr::null_mut(), ptr::from_mut),
            fe_found_var: self.found_var,
            ..FUNCEXE_INIT
        }
    }
}

/// [`call_func`] of the function `name` names -- its first `len` bytes when
/// that is given -- with `args`.
pub(crate) fn call_func_with(
    name: &CStr,
    len: Option<usize>,
    result: &mut TypVal,
    args: &[TypVal],
    mut with: CallWith<'_>,
) -> Result<(), Failed> {
    let len = len.map_or(-1, |len| c_int::try_from(len).unwrap_or(c_int::MAX));
    let mut funcexe = with.funcexe();
    // SAFETY: `name` is terminated, and holds `len` bytes when that is
    // given; every pointer in `funcexe` is taken from a borrow `with` holds
    // across the call.
    unsafe { call_func(name.as_ptr(), len, result, args, &raw mut funcexe) }
}

/// Evaluate a call written as an expression: read `(a, b)` at the cursor,
/// then make the call.
///
/// `name` is what the C handed on: the callee's name, or with `len` given
/// the text it is the first `len` bytes of -- a `v:lua.` name runs to the
/// cursor, and a message about it quotes the rest of the line.
pub(crate) fn get_func_tv(
    name: &CStr,
    len: Option<usize>,
    result: &mut TypVal,
    cursor: &mut Cursor<'_>,
    with: CallWith<'_>,
) -> Result<(), Failed> {
    let mut argvars = Argv::new();
    let mut argcount = 0;
    let evaluate = with.evaluate;

    let bound = with.partial.map_or(0, |partial| partial.pt_argc);
    let mut ret = get_func_arguments(cursor, evaluate, bound, argvars.room(), &mut argcount);
    // A failed argument leaves whatever it half-built in the slot it was
    // being evaluated into, which upstream leaks and the frame releases.
    argvars.fill(if ret.is_ok() {
        argcount
    } else {
        (argcount + 1).min(MAX_FUNC_ARGS as usize + 1)
    });

    if ret.is_ok() {
        // Prepare for calling `test_garbagecollect_now()`, which needs to
        // know which variables are used on the call stack.
        let pushed = push_func_args(&argvars.args()[..argcount]);
        ret = call_func_with(name, len, result, &argvars.args()[..argcount], with);
        // The nested calls pushed and popped their own; ours are the last.
        pop_func_args(pushed);
    } else if !aborting() && evaluate {
        let message = if argcount == MAX_FUNC_ARGS as usize {
            c"E740: Too many arguments for function %s"
        } else {
            c"E116: Invalid arguments for function %s"
        };
        emsg_funcname(message, name.to_bytes());
    }

    cursor.skip_white();
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
    if !unsafe { callback_call(&*callback, args, &mut rettv) } {
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

/// `bytes` and a NUL after them, in one allocation.
fn terminated(bytes: &[u8]) -> Vec<u8> {
    let mut owned = Vec::with_capacity(bytes.len() + 1);
    owned.extend_from_slice(bytes);
    owned.push(0);
    owned
}

/// The callee's name, copied and terminated: cut at its first NUL, as every
/// C reader of it was, unless it was `measured` to the NUL already.
fn name_copy(name: &[u8], measured: bool) -> Vec<u8> {
    if measured {
        terminated(name)
    } else {
        terminated(&name[..name.iter().position(|&b| b == 0).unwrap_or(name.len())])
    }
}

/// The name `name` is stored under, terminated, when that is not `name`
/// itself; and [`fname_trans_sid`]'s error.
fn stored_name(name: &[u8]) -> (Option<Vec<u8>>, c_int) {
    let (stored, error) = fname_trans_sid(name);
    match stored {
        Cow::Owned(stored) => (Some(terminated(&stored)), error),
        Cow::Borrowed(_) => (None, error),
    }
}

/// A [`terminated`] copy without its NUL.
fn unterminated(text: &[u8]) -> &[u8] {
    &text[..text.len() - 1]
}

/// Make a call: resolve `funcname` to a partial, a `v:lua` reference, a user
/// function (autoloading one if need be) or a builtin, and run it.
///
/// # Safety
/// `funcname` has `len` readable bytes (or is NUL-terminated when `len` is
/// not positive) and `funcexe` describes the call.
pub unsafe fn call_func(
    funcname: *const c_char,
    mut len: c_int,
    result: &mut TypVal,
    args_in: &[TypVal],
    funcexe: *mut FuncExe,
) -> Result<(), Failed> {
    let mut ret = Err(Failed);
    let mut error = FCERR_NONE;
    let mut fp: *mut UserFunc = ptr::null_mut();
    // The name, copied: if it comes from a funcref variable it could be
    // changed or deleted inside the called function. Then the name the
    // function is stored under, when that is not the name itself.
    let mut name: Option<Vec<u8>> = None;
    let mut translated: Option<Vec<u8>> = None;
    // A `v:lua` called directly is reported by that name.
    let mut report_vlua = false;
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

    // A length the caller did not give is the name's own, which then holds
    // no NUL.
    let measured = len <= 0;
    if measured {
        len = unsafe { cstr::bytes_at(funcname) }.len() as c_int;
    }
    if !partial.is_null() {
        fp = unsafe { (*partial).pt_func };
    }
    if fp.is_null() {
        // SAFETY: the caller's promise -- `len` readable bytes. Every
        // reader of the copy stops at its first NUL, as the C's did.
        let copy = unsafe { cstr::slice_at(funcname, len as size_t) };
        let copy = name.insert(name_copy(copy, measured));
        (translated, error) = stored_name(unterminated(copy));
    }
    // The name the function is stored under, with its NUL.
    let fname: &[u8] = translated.as_deref().or(name.as_deref()).unwrap_or(b"\0");
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
            let skip = if fp.is_null() && fname.starts_with(b"g:") {
                2
            } else {
                0
            };
            let rfname = unterminated(&fname[skip..]);
            // The same, as the C string the autocommand and the autoloader
            // read.
            let rfname_c = fname[skip..].as_ptr().cast::<c_char>().cast_mut();

            // the default is number zero
            result.write_number(0);
            error = FCERR_UNKNOWN;

            if is_luafunc(partial) {
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
                    report_vlua = true;
                }
            } else if !fp.is_null() || !builtin_function(rfname) {
                // A user-defined function.
                if fp.is_null() {
                    fp = find_func(rfname);
                }

                // Trigger FuncUndefined, which may load the function.
                let event = AutoEvent::FuncUndefined;
                if fp.is_null()
                    && unsafe { apply_autocmds(event, rfname_c, rfname_c, true, None) }
                    && !aborting()
                {
                    fp = find_func(rfname);
                }
                // Try loading a package.  Reached by every spelling that
                // does *not* go through `deref_func_name` first --
                // `call()`, `nvim_call_function`, `vim.fn` -- because
                // that one's `find_var` has already sourced it.
                if fp.is_null()
                    && unsafe { script_autoload(rfname_c, rfname.len(), true) }
                    && !aborting()
                {
                    fp = find_func(rfname);
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
                    unsafe { call_internal_func(fname.as_ptr().cast(), args, result) }
                } else {
                    unsafe { call_internal_method(fname.as_ptr().cast(), args, result, &mut *base) }
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
        // SAFETY: `funcexe` is the caller's own.
        let found = unsafe { (*funcexe).fe_found_var };
        match &name {
            _ if report_vlua => user_func_error(error, b"v:lua", found),
            Some(name) => user_func_error(error, unterminated(name), found),
            // SAFETY: the caller's terminated name.
            None => user_func_error(error, unsafe { cstr::bytes_at(funcname) }, found),
        }
    }
    ret
}
