//! `:return`, `:call`, `:defer`, and the do_cmdline cookie.
//!
//! `ex_return`/`do_return` implement returning -- including the case where
//! a `:finally` is still to run -- and `get_return_cmd` renders a pending
//! return for the debugger.  `get_func_line` and the small accessors below
//! it are the `do_cmdline` cookie interface a function body is executed
//! through.  `:defer` records a call to make on the way out and
//! `invoke_all_defer` makes them.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::ex_eval::CsFlags;
use crate::guard::Suppress;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::CmdIdx;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use std::rc::Rc;

use super::*;
use crate::eval::typval::{DictRef, PartialRef};
use crate::types::{Failed, IOSIZE, Pend};

/// One call recorded by `:defer`, to be made when the function returns.
pub struct Defer {
    /// The callee's name; emptied once the call is under way, so that a
    /// deferred call that throws cannot make this one run twice.
    pub(crate) name: Option<XString>,
    /// The arguments, owned: `:defer` takes its values rather than
    /// borrowing them, and they go with the record.
    pub(crate) args: Vec<TypVal>,
}

/// `:return [expr]`.
pub fn ex_return(excmd: &mut ExArg) {
    let mut rettv = TV_INITIAL_VALUE;
    let mut returning = false;

    if current_fc_id().is_none() {
        emsg(gettext(c"E133: :return not inside a function"));
        return;
    }

    let skipping = (excmd.skip).then(Suppress::emsg_skip);

    excmd.line.next = None;
    let (at, evaluate) = (excmd.line.arg, !excmd.skip);
    if !matches!(excmd.line.byte_at(at), 0 | b'|' | b'\n')
        && eval0_in_cmd(excmd, at, &mut rettv, evaluate).is_ok()
    {
        if !excmd.skip {
            returning = unsafe { do_return(excmd, false, true, (&raw mut rettv).cast::<c_void>()) };
        } else {
            tv_clear(&mut rettv);
        }
    } else if !excmd.skip {
        // It's safer to return also on error.
        update_force_abort();

        // Return unless the expression evaluation was cancelled by an
        // aborting error, an interrupt or an exception.
        if !aborting() {
            returning = unsafe { do_return(excmd, false, true, ptr::null_mut()) };
        }
    }

    // When skipping or the return gets pending, advance to the next
    // command in this line; otherwise the whole line is used.
    if returning {
        excmd.line.next = None;
    } else if excmd.line.next.is_none() {
        excmd.line.next = excmd.line.check_next(excmd.line.arg);
    }

    drop(skipping);
}

/// Make the call `:call` asks for, once per line of its range, with its
/// arguments at `startarg` in the command line. Answers whether it failed
/// and where in the line the last walk stopped.
fn ex_call_inner(
    excmd: &mut ExArg,
    callee: &CStr,
    startarg: usize,
    partial: Option<&PartialRef>,
    selfdict: Option<&DictRef>,
    found_var: bool,
) -> (bool, usize) {
    let mut doesrange = false;
    let mut failed = false;
    let mut stop = startarg;
    let (line1, line2, ranged) = (excmd.line1, excmd.line2, excmd.addr_count > 0);
    // The command line is this command's own: no code a call runs can reach
    // it, so the walk borrows it across the calls.
    let text = excmd.line.rest_of(startarg);

    let mut lnum = line1;
    while lnum <= line2 {
        if ranged {
            // Default is the line number, not the range.
            if lnum > Buf::current().b_ml.ml_line_count {
                emsg(gettext(e_invrange));
                break;
            }
            Win::current().w_cursor.lnum = lnum;
            Win::current().w_cursor.col = 0;
            Win::current().w_cursor.coladd = 0;
        }

        let with = CallWith {
            firstline: line1,
            lastline: line2,
            partial,
            selfdict,
            doesrange: Some(&mut doesrange),
            found_var,
            ..CallWith::new(true)
        };
        let mut rettv = TV_INITIAL_VALUE;
        // The call, then any trailing subscript: `:call f()[1]()`.
        let mut cursor = Cursor::new(text);
        let called = get_func_tv(callee, None, &mut rettv, &mut cursor, with)
            .and_then(|()| handle_subscript(&mut cursor, &mut rettv, true, true));
        stop = startarg + cursor.offset();
        if called.is_err() {
            failed = true;
            break;
        }
        tv_clear(&mut rettv);
        if doesrange || aborting() {
            break;
        }
        lnum += 1;
    }
    (failed, stop)
}

/// `:defer Func(args)`: check the call now and record it for the way out.
/// `text` is the command line from the `(`; answers how much of it the
/// arguments took.
fn ex_defer_inner(
    callee: &XString,
    partial: Option<&PartialRef>,
    text: &[u8],
) -> (Result<(), Failed>, usize) {
    let mut argvars = [TV_INITIAL_VALUE; MAX_FUNC_ARGS as usize + 1];
    let mut partial_argc = 0;
    let mut argcount = 0;

    if current_fc_id().is_none() {
        let arg0 = "defer";
        semsg!("E193: {arg0} not inside a function");
        return (Err(Failed), 0);
    }

    if let Some(partial) = partial {
        if partial.pt_dict.is_some() {
            emsg(gettext(E_CANNOT_USE_PARTIAL_WITH_DICTIONARY_FOR_DEFER));
            return (Err(Failed), 0);
        }
        partial_argc = partial.pt_argv.len();
        for (arg, slot) in partial.pt_argv.iter().zip(&mut argvars) {
            tv_copy(arg, slot);
        }
    }

    // Upstream passes `false` for the partial argument count here; the
    // room already taken is accounted for by the `argvars` offset below.
    let mut cursor = Cursor::new(text);
    let free_slot = &mut argvars[partial_argc..];
    let mut r = get_func_arguments(&mut cursor, true, 0, free_slot, &mut argcount);
    let total = argcount + partial_argc;
    let argcount = c_int::try_from(total).unwrap_or(c_int::MAX);

    if r.is_ok() {
        if builtin_function(callee) {
            r = match find_builtin(callee) {
                None => {
                    emsg_funcname(e_unknown_function_str, callee);
                    Err(Failed)
                }
                Some(fdef) => check_builtin_argcount(fdef, argcount),
            };
        } else {
            if let Some(func) = find_func(callee) {
                let error = check_user_func_argcount(&func, argcount);
                if error != FCERR_UNKNOWN {
                    user_func_error(error, callee, false);
                    r = Err(Failed);
                }
            }
        }
    }

    if r.is_ok() {
        add_defer(callee.as_cstr(), &mut argvars[..total]);
    }
    (r, cursor.offset())
}

/// Whether a `:defer` can be recorded here, i.e. whether a function is
/// running.  Reports the error itself when it cannot.
pub fn can_add_defer() -> bool {
    if current_fc_id().is_none() {
        let arg0 = "defer";
        semsg!("E193: {arg0} not inside a function");
        return false;
    }
    true
}

/// Record a deferred call of `name` on the funccall that is running.  It
/// takes over the values in `args`.
pub(crate) fn add_defer(name: &CStr, args: &mut [TypVal]) {
    let Some(frame) = current_fc() else {
        return;
    };
    let args = args.iter_mut().map(TypVal::take).collect();
    frame.defer.borrow_mut().push(Defer {
        name: Some(XString::from_cstr(name)),
        args,
    });
}

/// Make the calls `:defer` recorded on `frame`, newest first.
pub(crate) fn handle_defer_one(frame: &FuncCall) {
    // The records present now, newest first; one recorded meanwhile -- by a
    // deferred `execute('defer ...')` -- is dropped uncalled, as upstream's
    // walk never reached it.
    let count = frame.defer.borrow().len();
    for idx in (0..count).rev() {
        let taken = {
            let mut records = frame.defer.borrow_mut();
            records
                .get_mut(idx)
                .and_then(|record| Some((record.name.take()?, core::mem::take(&mut record.args))))
        };
        let Some((name, args)) = taken else {
            continue;
        };
        let mut rettv = TV_INITIAL_VALUE;

        // The deferred call runs with a clean exception state, so that it
        // happens even while an exception is in flight.
        let estate = exception_state_save();
        exception_state_clear();
        let _ = call_func_with(name.as_cstr(), None, &mut rettv, &args, CallWith::new(true));
        exception_state_restore(&estate);
        tv_clear(&mut rettv);
        drop(args);
    }
    drop(frame.defer.take());
}

/// Make every deferred call on every funccall, which is what an exit does.
pub fn invoke_all_defer() {
    let stacks = ::core::iter::once(current_fc_id()).chain(set_aside_call_stacks());
    for id in stacks.flat_map(call_chain) {
        handle_defer_one(&id.funccall());
    }
}

/// `:call` and `:defer`.
pub fn ex_call(excmd: &mut ExArg) {
    if excmd.skip {
        // Trailing arguments are still parsed, so that errors in them
        // are reported -- but nothing is called.
        let mut rettv = TV_INITIAL_VALUE;
        let skipping = Suppress::emsg_skip();
        let at = excmd.line.arg;
        if eval0_in_cmd(excmd, at, &mut rettv, false).is_ok() {
            tv_clear(&mut rettv);
        }
        drop(skipping);
        return;
    }

    let arg = excmd.line.arg;
    let FunctionName {
        name: tofree,
        end,
        dict,
        partial,
    } = trans_function_name(excmd.line.rest_of(arg), false, TFN_INT, true);
    if let Some(FuncDict {
        key, new_key: true, ..
    }) = &dict
    {
        // Still need to give an error message for missing key.
        let key = msg_bytes(key);
        semsg!("E716: Key not present in Dictionary: \"{key}\"");
    }
    let Some(tofree) = tofree else {
        return;
    };

    // If it is the name of a variable of type VAR_FUNC or VAR_PARTIAL use
    // its contents; `trans_function_name` skips over "s:" and "g:". The
    // dictionary is held by `dict`, as it could get deleted when evaluating
    // the arguments.
    let dereffed = deref_func_name(&tofree, false);
    let found_var = dereffed.found_var;
    let (name, partial) = match dereffed.name {
        Some(name) => (name, partial.or(dereffed.partial)),
        None => (tofree, partial),
    };
    let selfdict = dict.as_ref().map(|dict| &dict.dict);

    let startarg = excmd.line.skip_white(arg + end);
    if excmd.line.byte_at(startarg) != b'(' {
        let arg = msg_bytes(excmd.line.arg());
        semsg!("E107: Missing parentheses: {arg}");
        return;
    }
    let (failed, stop) = if excmd.cmdidx == CmdIdx::defer {
        let (r, used) = ex_defer_inner(&name, partial.as_ref(), excmd.line.rest_of(startarg));
        (r.is_err(), startarg + used)
    } else {
        let callee = name.as_cstr();
        ex_call_inner(
            excmd,
            callee,
            startarg,
            partial.as_ref(),
            selfdict,
            found_var,
        )
    };

    // When inside a `:try` the trailing text is still checked, so that an
    // error is reported for it rather than swallowed.
    // SAFETY: a command being run has its condition stack.
    let in_try = unsafe { (*excmd.cstack).cs_trylevel } > 0;
    if (!aborting() || did_throw.get()) && (!failed || in_try) {
        if ends_excmd(c_int::from(excmd.line.byte_at(stop))) == 0 {
            if !failed && !aborting() {
                emsg_severe.set(true);
                let rest = msg_bytes(excmd.line.rest_of(stop));
                semsg!("E488: Trailing characters: {rest}");
            }
        } else {
            excmd.line.next = excmd.line.check_next(stop);
        }
    }
}

/// Return from a function, answering whether the return happened now rather
/// than being made pending by a `:finally`.
///
/// # Safety
/// `excmd` is a live command with a condition stack, and `result` is null or a
/// `TypVal`.
pub unsafe fn do_return(
    excmd: &mut ExArg,
    reanimate: bool,
    is_cmd: bool,
    result: *mut c_void,
) -> bool {
    let cstack = excmd.cstack;
    let frame = current_fc().expect(":return inside a function");

    if reanimate {
        // Undo the return.
        frame.returned.set(false);
    }

    // Cleanup (and inactivate) conditionals, but stop when a `:finally`
    // is reached: the return still has to be pending until that has run.
    // SAFETY: the caller's promise -- a command with its condition stack.
    let idx = unsafe { cleanup_conditionals(cstack, CsFlags::NONE, true) };
    if idx >= 0 {
        let at = usize::try_from(idx).expect("a level of the stack");
        // A `:finally` is going to run first; remember the return value.
        let flag = c_char::try_from(CSTP_RETURN).expect("a pending flag fits a char");
        // SAFETY: as above.
        unsafe { (*cstack).cs_pending[at] = flag };

        let pending: *mut c_void = if !is_cmd && !reanimate {
            // A pending return again gets pending: `result` is the boxed
            // value of the original return.
            result
        } else {
            let value = if reanimate {
                // The value is the funccall's; it is not available to the
                // function any more until the `:finally` is done.
                Some(frame.rettv.replace(TypVal::Number(0)))
            } else if result.is_null() {
                None
            } else {
                // SAFETY: the caller's promise -- `result` is a `TypVal`.
                Some(unsafe { (*result.cast::<TypVal>()).take() })
            };
            value.map_or(ptr::null_mut(), |value| {
                Box::into_raw(Box::new(value)).cast::<c_void>()
            })
        };
        // SAFETY: as above; the pending slot owns the box from here.
        unsafe { (*cstack).set_pending_return(at, pending) };
        // SAFETY: the pending value just stored, or null.
        unsafe { report_pending(PendingAction::Made, CSTP_RETURN, Pend::Return(pending)) };
    } else {
        frame.returned.set(true);
        if !reanimate && !result.is_null() {
            // SAFETY: the caller's promise -- `result` is a `TypVal`, the
            // command's own or (when not `is_cmd`) a pending slot's box.
            let value = unsafe { (*result.cast::<TypVal>()).take() };
            drop(frame.rettv.replace(value));
            if !is_cmd {
                // The pending slot's box, emptied just above.
                // SAFETY: as above -- a box `Box::into_raw` made.
                drop(unsafe { Box::from_raw(result.cast::<TypVal>()) });
            }
        }
    }

    idx < 0
}

/// Render `:return <expr>` for the debugger, in allocated memory.
///
/// # Safety
/// `result` is null or a `TypVal`.
pub unsafe fn get_return_cmd(result: *mut c_void) -> *mut c_char {
    // SAFETY: the caller's promise.
    let value = unsafe { result.cast::<TypVal>().as_ref() };
    let rendered = value.map(encode_tv2echo);
    let mut line = b":return ".to_vec();
    if let Some(rendered) = &rendered {
        line.extend_from_slice(rendered.as_bytes());
    }
    // Upstream renders into `IObuff` and marks a cut with "...".
    let limit = IOSIZE as usize - 1;
    if line.len() >= limit {
        line.truncate(limit - 4);
        line.extend_from_slice(b"...");
    }
    XString::from_bytes(&line).into_raw()
}

/// The cookie `do_cmdline` hands [`get_func_line`] for a call: its funccall's
/// id, carried in the bits of the pointer-sized cookie.
pub(crate) fn func_line_cookie(frame: &FuncCall) -> *mut c_void {
    ptr::without_provenance_mut(frame.id.to_bits())
}

/// The funccall a [`func_line_cookie`] names.
///
/// # Panics
/// When the cookie names no funccall that is still in the table.
pub(crate) fn cookie_funccall(cookie: *mut c_void) -> Rc<FuncCall> {
    cookie_id(cookie.addr()).funccall()
}

/// The id a [`func_line_cookie`] carries.
fn cookie_id(bits: usize) -> FcId {
    FcId::from_bits(bits).expect("a function-body cookie names a funccall")
}

/// Run `f` on the funccall a [`func_line_cookie`] names, without taking a
/// reference: the cookie readers run no user code.
fn with_cookie_funccall<R>(bits: usize, f: impl FnOnce(&FuncCall) -> R) -> R {
    cookie_id(bits).with(f)
}

/// The `do_cmdline` line getter a function body is executed through.  It
/// also drives the debugger's breakpoints and the line profiler.
///
/// `getline_equal` compares this function's *address* against the cookie's
/// getter to decide whether a function is running; it coerces to the
/// `LineGetter` type, whose cookie is [`func_line_cookie`]'s.
pub fn get_func_line(
    _c: c_int,
    cookie: *mut c_void,
    _indent: c_int,
    _do_concat: bool,
) -> *mut c_char {
    let frame = cookie_funccall(cookie);
    let func = &frame.func;
    let name = func.name().as_cstr();

    // Check for a breakpoint set after the sourcing started.
    if frame.dbg_tick.get() != debug_tick.get() {
        frame
            .breakpoint
            .set(dbg_find_breakpoint_named(false, name, sourcing_lnum()));
        frame.dbg_tick.set(debug_tick.get());
    }
    if do_profiling.get() == PROF_YES {
        func_line_end(&frame);
    }
    let retval = if (func.has_flag(FuncFlags::ABORT) && did_emsg.get() != 0 && !aborted_in_try())
        || frame.returned.get()
    {
        ptr::null_mut()
    } else {
        let body = func.body();
        let mut linenr = usize::try_from(frame.linenr.get()).unwrap_or(0);
        // Skip NULL lines, they are continuation lines.
        while body.lines.get(linenr).is_some_and(Option::is_none) {
            linenr += 1;
        }
        match body.lines.get(linenr) {
            Some(Some(line)) => {
                linenr += 1;
                frame
                    .linenr
                    .set(c_int::try_from(linenr).unwrap_or(c_int::MAX));
                let dup = XString::from_bytes(line).into_raw();
                crate::runtime::set_sourcing_lnum(LineNr::from(frame.linenr.get()));
                if do_profiling.get() == PROF_YES {
                    func_line_start(&frame);
                }
                dup
            }
            _ => {
                frame
                    .linenr
                    .set(c_int::try_from(linenr).unwrap_or(c_int::MAX));
                ptr::null_mut()
            }
        }
    };

    // Did we encounter a breakpoint?
    let breakpoint = frame.breakpoint.get();
    if breakpoint != 0 && breakpoint <= sourcing_lnum() {
        let at = sourcing_lnum();
        dbg_breakpoint(name, at);
        // Find the next breakpoint.
        frame
            .breakpoint
            .set(dbg_find_breakpoint_named(false, name, at));
        frame.dbg_tick.set(debug_tick.get());
    }

    retval
}

/// Whether the function running under `cookie` has ended.
pub fn func_has_ended(cookie: *mut c_void) -> c_int {
    with_cookie_funccall(cookie.addr(), |frame| {
        c_int::from(
            (frame.func.has_flag(FuncFlags::ABORT) && did_emsg.get() != 0 && !aborted_in_try())
                || frame.returned.get(),
        )
    })
}

/// Whether the function running under `cookie` was declared `abort`.
pub fn func_has_abort(cookie: *mut c_void) -> c_int {
    with_cookie_funccall(cookie.addr(), |frame| {
        c_int::from(frame.func.has_flag(FuncFlags::ABORT))
    })
}

/// The name of the function running under `cookie`, which lives as long as
/// the call does.
pub fn func_name(cookie: *mut c_void) -> *mut c_char {
    with_cookie_funccall(cookie.addr(), |frame| frame.func.name().as_ptr().cast_mut())
}

/// The breakpoint line of the function running under `cookie`, by address:
/// the debugger moves it.
pub fn func_breakpoint(cookie: *mut c_void) -> *mut LineNr {
    with_cookie_funccall(cookie.addr(), |frame| frame.breakpoint.as_ptr())
}

/// The debug tick of the function running under `cookie`, by address.
pub fn func_dbg_tick(cookie: *mut c_void) -> *mut c_int {
    with_cookie_funccall(cookie.addr(), |frame| frame.dbg_tick.as_ptr())
}

/// The `:if`/`:while` nesting level of the function running under `cookie`.
pub fn func_level(cookie: *mut c_void) -> c_int {
    with_cookie_funccall(cookie.addr(), |frame| frame.level)
}

/// Whether the function running has already returned.
pub(crate) fn current_func_returned() -> c_int {
    with_current_fc(|frame| c_int::from(frame.is_some_and(|frame| frame.returned.get())))
}
