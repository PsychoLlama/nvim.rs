//! Choosing what a call *is*, before any of it happens.
//!
//! `call_func` is the one entry point every caller of anything callable
//! reaches: a partial, a `v:lua` reference, a user function, an autoloaded
//! one, or a builtin.  `get_func_tv` is the expression-level wrapper that
//! parses the argument list first, and `func_call` the one that takes the
//! arguments already built as a list.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::winlayer::Win;
use core::ffi::c_int;
use core::ptr;
use std::rc::Rc;

use super::*;
use crate::autocmd::fire_autocmds_for;
use crate::eval::funcs::{call_internal_func_named, call_internal_method_named};
use crate::eval::typval::CallFrame;
use crate::eval::typval::{DictRef, PartialRef, index_of};
use crate::lua::executor::typval_call_lua;
use crate::runtime::script_autoload_named;
use crate::types::{ArgvFunc, Failed};
use std::borrow::Cow;

/// What a call is made with besides its name and its arguments, as borrows
/// the caller holds for the length of the call.
pub(crate) struct CallWith<'a> {
    /// Fills in the arguments once the function is known: a `\=`
    /// expression's submatch list goes only to a function that takes it.
    pub(crate) argv_func: Option<ArgvFunc>,
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
            argv_func: None,
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

    let bound = with
        .partial
        .map_or(0, |partial| index_of(partial.pt_argv.len()));
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
/// `call()` does.
pub(crate) fn func_call(
    name: &CStr,
    args: &TypVal,
    partial: Option<&PartialRef>,
    selfdict: Option<&DictRef>,
    result: &mut TypVal,
) -> Result<(), Failed> {
    let mut argv = Argv::new();
    let bound = partial.map_or(0, |partial| index_of(partial.pt_argv.len()));
    for (argc, item) in list_iter(args.list_ref()).enumerate() {
        if argc == usize::try_from(MAX_FUNC_ARGS - bound).unwrap_or(0) {
            emsg(gettext(c"E699: Too many arguments"));
            return Ok(());
        }
        // Copy each argument, so that `v_lock` can be set to VarLock::Fixed
        // in the copy without changing the original list.
        tv_copy(&item.li_tv, argv.claim());
    }

    let with = CallWith {
        partial,
        selfdict,
        ..CallWith::at_cursor(true)
    };
    call_func_with(name, None, result, argv.args(), with)
}

/// Call a callback and take its answer as a number; -2 when the call itself
/// failed.
pub fn callback_call_retnr(callback: &Callback, args: &[TypVal]) -> VarNumber {
    let mut rettv = TV_INITIAL_VALUE;
    if !callback_call(callback, args, &mut rettv) {
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
fn splice_bound(argv: &mut Argv, partial: &Partial, args: &[TypVal]) -> Result<(), ()> {
    let bound = &partial.pt_argv;
    if bound.len() + args.len() > MAX_FUNC_ARGS as usize {
        return Err(());
    }
    for value in bound {
        argv.push_owned(value.clone());
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
/// `funcname` is what the C handed on: the callee's name, or with `len`
/// given the text it is the first `len` bytes of -- a `v:lua.` name runs to
/// the cursor, and a message about it quotes the rest of the line.
pub(crate) fn call_func_with(
    funcname: &CStr,
    len: Option<usize>,
    result: &mut TypVal,
    args_in: &[TypVal],
    mut with: CallWith<'_>,
) -> Result<(), Failed> {
    let mut ret = Err(Failed);
    let mut error = FCERR_NONE;
    let mut func: Option<Rc<UserFunc>> = None;
    // The name, copied: if it comes from a funcref variable it could be
    // changed or deleted inside the called function. Then the name the
    // function is stored under, when that is not the name itself.
    let mut name: Option<Vec<u8>> = None;
    let mut translated: Option<Vec<u8>> = None;
    // A `v:lua` called directly is reported by that name.
    let mut report_vlua = false;
    let mut selfdict = with.selfdict;
    // How much of `args_in` is still the argument list, once an
    // `argv_func` has had its say.
    let mut nargs = args_in.len();
    // Built only when a partial or the base puts arguments in front; until
    // then the caller's own slice *is* the argument list.
    let mut argv: Option<Argv> = None;
    let partial = with.partial;

    // Initialise rettv so that the caller may `tv_clear` it even when this
    // answers FAIL.
    result.write_empty(VAR_UNKNOWN);

    // A length the caller did not give is the name's own, which then holds
    // no NUL.
    let text = funcname.to_bytes();
    let measured = len.is_none();
    let len = len.unwrap_or(text.len());
    if let Some(partial) = partial {
        func = partial.pt_func.clone();
    }
    if func.is_none() {
        // Every reader of the copy stops at its first NUL, as the C's did.
        let copy = name.insert(name_copy(&text[..len.min(text.len())], measured));
        (translated, error) = stored_name(unterminated(copy));
    }
    // The name the function is stored under, with its NUL.
    let fname: &[u8] = translated.as_deref().or(name.as_deref()).unwrap_or(b"\0");
    if let Some(doesrange) = with.doesrange.as_deref_mut() {
        *doesrange = false;
    }

    'theend: {
        if let Some(partial) = partial {
            // When the function has a partial with a dict and there is a
            // dict argument, use the dict argument -- that is backwards
            // compatible.  When the dict was bound explicitly, use the
            // partial's.
            if let Some(dict) = &partial.pt_dict
                && (selfdict.is_none() || !partial.pt_auto)
            {
                selfdict = Some(dict);
            }
            if error == FCERR_NONE && !partial.pt_argv.is_empty() {
                let mut frame = Argv::new();
                if splice_bound(&mut frame, partial, args_in).is_err() {
                    error = FCERR_TOOMANY;
                    break 'theend;
                }
                argv = Some(frame);
            }
        }

        if error == FCERR_NONE && with.evaluate {
            // Skip "g:" before a function name.
            let skip = if func.is_none() && fname.starts_with(b"g:") {
                2
            } else {
                0
            };
            let rfname = unterminated(&fname[skip..]);
            let rfname_c =
                CStr::from_bytes_until_nul(&fname[skip..]).expect("the stored name is terminated");

            // the default is number zero
            result.write_number(0);
            error = FCERR_UNKNOWN;

            if is_luafunc(partial.map_or(ptr::null_mut(), PartialRef::as_ptr)) {
                if len > 0 {
                    error = FCERR_NONE;
                    if let Some(base) = with.basetv.as_deref()
                        && splice_base(&mut argv, &args_in[..nargs], base).is_err()
                    {
                        error = FCERR_TOOMANY;
                        break 'theend;
                    }
                    let args = spliced_args(&argv, args_in, nargs);
                    typval_call_lua(&text[..len.min(text.len())], args, result);
                } else {
                    // v:lua was called directly; show its name in the
                    // message.
                    report_vlua = true;
                }
            } else if func.is_some() || !builtin_function(rfname) {
                // A user-defined function.
                if func.is_none() {
                    func = find_func(rfname);
                }

                // Trigger FuncUndefined, which may load the function.
                let event = AutoEvent::FuncUndefined;
                if func.is_none()
                    && fire_autocmds_for(event, Some(rfname_c), Some(rfname_c), true, None)
                    && !aborting()
                {
                    func = find_func(rfname);
                }
                // Try loading a package.  Reached by every spelling that
                // does *not* go through `deref_func_name` first --
                // `call()`, `nvim_call_function`, `vim.fn` -- because that
                // one's `find_var` has already sourced it.
                if func.is_none() && script_autoload_named(rfname, true) && !aborting() {
                    func = find_func(rfname);
                }

                if let Some(func) = &func {
                    if func.has_flag(FuncFlags::DELETED) {
                        error = FCERR_DELETED;
                    } else {
                        if let Some(argv_func) = with.argv_func {
                            // Postponed filling in the arguments; do it now.
                            let filled = argv.as_ref().map_or(0, Argv::len);
                            let skip = filled.saturating_sub(args_in.len());
                            let args = spliced_args(&argv, args_in, nargs);
                            let n = argv_func(args, skip, func);
                            match &mut argv {
                                Some(argv) => argv.truncate(n),
                                None => nargs = n,
                            }
                        }
                        if let Some(base) = with.basetv.as_deref()
                            && splice_base(&mut argv, &args_in[..nargs], base).is_err()
                        {
                            error = FCERR_TOOMANY;
                            break 'theend;
                        }
                        let args = spliced_args(&argv, args_in, nargs);
                        error = call_user_func_check(func, args, result, &mut with, selfdict);
                    }
                }
            } else {
                let args = spliced_args(&argv, args_in, nargs);
                let builtin = CStr::from_bytes_until_nul(fname).expect("a terminated name");
                error = match with.basetv.as_deref_mut() {
                    None => call_internal_func_named(builtin, args, result),
                    Some(base) => call_internal_method_named(builtin, args, result, base),
                };
            }

            // The call (or the FuncUndefined autocommand sequence) may have
            // been aborted by an error, an interrupt, or an uncaught
            // exception, which `aborting()` reports.  For an error inside an
            // internal function, or for E132 in `call_user_func`, the throw
            // point where `force_abort` is normally updated has not been
            // reached yet, so update it here to make `aborting()` reliable.
            update_force_abort();
        }
        if error == FCERR_NONE {
            ret = Ok(());
        }
    }

    // Report an error unless evaluating the arguments or making the call
    // was cancelled by an aborting error, an interrupt or an exception.
    if !aborting() {
        let found = with.found_var;
        match &name {
            _ if report_vlua => user_func_error(error, b"v:lua", found),
            Some(name) => user_func_error(error, unterminated(name), found),
            None => user_func_error(error, text, found),
        }
    }
    ret
}
