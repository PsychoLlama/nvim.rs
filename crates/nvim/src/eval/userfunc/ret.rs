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

use crate::cstr;
use crate::ex_eval::CsFlags;
use crate::guard::Suppress;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::CmdIdx;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::eval::typval::{DictRef, PartialRef};
use crate::types::{Failed, IOSIZE, Pend};

/// One call recorded by `:defer`, to be made when the function returns.
pub struct Defer {
    pub dr_name: *mut c_char,
    /// The arguments, **owned**: `:defer` is the one frame in the tree that
    /// takes its values rather than borrowing them, and `handle_defer_one`
    /// clears each one after the call.  The record lives in a `GArray`, so
    /// nothing drops it.
    pub dr_argvars: [TypVal; MAX_FUNC_ARGS as usize + 1],
    pub dr_argcount: c_int,
}

/// `:return [expr]`.
pub fn ex_return(excmd: &mut ExArg) {
    let mut rettv = TV_INITIAL_VALUE;
    let mut returning = false;

    if current_fc().is_null() {
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
            returning = unsafe { do_return(excmd, false, true, (&raw mut rettv) as *mut c_void) };
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

    if current_fc().is_null() {
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
    let argcount = argcount as c_int + partial_argc as c_int;

    if r.is_ok() {
        if builtin_function(callee) {
            r = match find_builtin(callee) {
                None => {
                    emsg_funcname(e_unknown_function_str, callee);
                    Err(Failed)
                }
                // SAFETY: a row of the builtin table.
                Some(fdef) => unsafe { check_internal_func(fdef, argcount) },
            };
        } else {
            let ufunc = find_func(callee);
            if !ufunc.is_null() {
                // SAFETY: the function just found is live.
                let error = unsafe { check_user_func_argcount(ufunc, argcount) };
                if error != FCERR_UNKNOWN {
                    user_func_error(error, callee, false);
                    r = Err(Failed);
                }
            }
        }
    }

    if r.is_ok() {
        // SAFETY: a function is running, and the name is NUL-terminated.
        unsafe {
            add_defer(
                callee.as_ptr().cast_mut(),
                &mut argvars[..argcount as usize],
            )
        };
    }
    (r, cursor.offset())
}

/// Whether a `:defer` can be recorded here, i.e. whether a function is
/// running.  Reports the error itself when it cannot.
pub fn can_add_defer() -> bool {
    if get_current_funccal().is_null() {
        let arg0 = "defer";
        semsg!("E193: {arg0} not inside a function");
        return false;
    }
    true
}

/// Record a deferred call of `name` on the funccall that is running.  It
/// takes over the values in `args`.
///
/// # Safety
/// A function is running, `name` is NUL-terminated and `args` holds
/// `argcount_arg` values.
pub unsafe fn add_defer(name: *mut c_char, args: &mut [TypVal]) {
    let saved_name = unsafe { xstrdup(name) };
    let mut argcount = args.len() as c_int;

    let fc = current_fc();
    if unsafe { (*fc).fc_defer.ga_itemsize } == 0 {
        unsafe { ga_init(&raw mut (*fc).fc_defer, size_of::<Defer>() as c_int, 10) };
    }
    let dr =
        unsafe { ga_append_via_ptr(&raw mut (*fc).fc_defer, size_of::<Defer>()) } as *mut Defer;
    unsafe { (*dr).dr_name = saved_name };
    unsafe { (*dr).dr_argcount = argcount };
    while argcount > 0 {
        argcount -= 1;
        // `ga_append_via_ptr` hands back raw storage, so the value is
        // *written* rather than assigned: there is nothing there to release.
        let slot = unsafe {
            (&raw mut (*dr).dr_argvars)
                .cast::<TypVal>()
                .add(argcount as usize)
        };
        unsafe { slot.write(args[argcount as usize].take()) };
    }
}

/// Make the calls `:defer` recorded on `funccal`, newest first.
///
/// # Safety
/// `funccal` is a live funccall.
pub(crate) unsafe fn handle_defer_one(funccal: *mut FuncCall) {
    // SAFETY: the caller's promise -- `funccal` is a live funccall.
    let frame = unsafe { Fc::new(funccal) };
    let mut idx = frame.fc_defer.ga_len - 1;
    while idx >= 0 {
        let dr = unsafe { (frame.fc_defer.ga_data as *mut Defer).offset(idx as isize) };
        if !unsafe { (*dr).dr_name }.is_null() {
            let mut funcexe = FUNCEXE_INIT;
            funcexe.fe_evaluate = true;
            let mut rettv = TV_INITIAL_VALUE;

            // Clear the name first, so that a deferred call that itself
            // throws cannot make this one run twice.
            let name = unsafe { (*dr).dr_name };
            unsafe { (*dr).dr_name = ptr::null_mut() };

            // The deferred call runs with a clean exception state, so
            // that it happens even while an exception is in flight.
            let estate = exception_state_save();
            exception_state_clear();

            // SAFETY: `dr` is the deferred call's own record, so its
            // argument array holds `dr_argcount` values.
            let argc = unsafe { (*dr).dr_argcount } as usize;
            let argp = unsafe { (&raw mut (*dr).dr_argvars).cast::<TypVal>() };
            let args = unsafe { ::core::slice::from_raw_parts(argp, argc) };
            let exe = &raw mut funcexe;
            let _ = unsafe { call_func(name, -1, &mut rettv, args, exe) };

            exception_state_restore(&estate);
            tv_clear(&mut rettv);
            unsafe { xfree(name as *mut c_void) };
            let mut i = unsafe { (*dr).dr_argcount } - 1;
            while i >= 0 {
                unsafe {
                    tv_clear(&mut *((&raw mut (*dr).dr_argvars) as *mut TypVal).offset(i as isize))
                };
                i -= 1;
            }
        }
        idx -= 1;
    }
    unsafe { ga_clear(&raw mut (*funccal).fc_defer) };
}

/// Make every deferred call on every funccall, which is what an exit does.
pub fn invoke_all_defer() {
    let stacks = ::core::iter::once(current_fc_id()).chain(set_aside_call_stacks());
    for id in stacks.flat_map(call_chain) {
        // SAFETY: every funccall on a call stack is running, so live.
        unsafe { handle_defer_one(id.funccall()) };
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
    // SAFETY: the caller's promise -- `excmd` is the Ex command being run.
    let mut result = result;
    let cstack = excmd.cstack;

    if reanimate {
        // Undo the return.
        unsafe { (*current_fc()).fc_returned = 0 };
    }

    // Cleanup (and inactivate) conditionals, but stop when a `:finally`
    // is reached: the return still has to be pending until that has run.
    let idx = unsafe { cleanup_conditionals(excmd.cstack, CsFlags::NONE, true) };
    // Set when the pending slot took the value out of `result` by bit copy:
    // the source gives it up once `report_pending` has rendered it.
    let mut handed_over = false;
    if idx >= 0 {
        // A `:finally` is going to run first; remember the return value.
        unsafe { (*cstack).cs_pending[idx as usize] = CSTP_RETURN as c_char };

        if !is_cmd && !reanimate {
            // A pending return again gets pending: `result` points to an
            // allocated variable with the value of the original return.
            unsafe { (*cstack).set_pending_return(idx as usize, result) };
        } else {
            if reanimate {
                debug_assert!(!unsafe { (*current_fc()).fc_rettv }.is_null());
                result = unsafe { (*current_fc()).fc_rettv } as *mut c_void;
            }
            if result.is_null() {
                unsafe { (*cstack).set_pending_return(idx as usize, ptr::null_mut()) };
            } else {
                // Store the value of the pending return.  A bit copy, not
                // a take: the pending slot owns the value from here, but
                // `report_pending` below still renders `result` for
                // `:debug`, and blanking it first would leave it a
                // `VAR_UNKNOWN` the echo encoder refuses.  The source gives
                // it up straight after that report instead.
                let copy = unsafe { (*result.cast::<TypVal>()).bit_copy() };
                let saved = Box::into_raw(Box::new(copy)).cast::<c_void>();
                unsafe { (*cstack).set_pending_return(idx as usize, saved) };
                // `reanimate` blanks `fc_rettv` just below, which is its
                // own way of giving the value up.
                handed_over = !reanimate;
            }
            if reanimate {
                // The return value is not available yet.
                unsafe { (*(*current_fc()).fc_rettv).write_number(0) };
            }
        }
        unsafe { report_pending(PendingAction::Made, CSTP_RETURN, Pend::Return(result)) };
        if handed_over {
            // Rendered; the pending slot is the only owner now.
            unsafe { (*result.cast::<TypVal>()).disown() };
        }
    } else {
        unsafe { (*current_fc()).fc_returned = 1 };
        if !reanimate && !result.is_null() {
            unsafe { tv_clear(&mut *(*current_fc()).fc_rettv) };
            unsafe { *(*current_fc()).fc_rettv = (*(result as *mut TypVal)).take() };
            if !is_cmd {
                // The pending slot's box, emptied just above.
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
    // The rendered command. Upstream shares `IObuff`, which the debugger
    // this feeds writes again.
    let mut line = [0 as c_char; IOSIZE as usize];
    let mut s: *mut c_char = ptr::null_mut();
    let mut tofree: *mut c_char = ptr::null_mut();
    let mut slen: size_t = 0;

    if !result.is_null() {
        s = unsafe { encode_tv2echo(&*result.cast::<TypVal>(), ptr::null_mut()) };
        tofree = s;
    }
    if s.is_null() {
        s = c"".as_ptr() as *mut c_char;
    } else {
        slen = unsafe { cstr::bytes_at(s) }.len();
    }

    const PREFIX: &CStr = c":return ";
    let buf = line.as_mut_ptr();
    unsafe { xstrlcpy(buf, PREFIX.as_ptr(), IOSIZE as size_t) };
    // SAFETY: `buf` is `IOSIZE` bytes and the prefix is already in it.
    let after = unsafe { buf.add(PREFIX.count_bytes()) };
    let left = (IOSIZE as size_t) - PREFIX.count_bytes();
    unsafe { xstrlcpy(after, s, left) };
    let mut iobufflen = PREFIX.count_bytes() + slen;
    if iobufflen >= IOSIZE as size_t {
        unsafe { strcpy(buf.offset(IOSIZE as isize - 4), c"...".as_ptr()) };
        iobufflen = IOSIZE as size_t - 1;
    }
    unsafe { xfree(tofree as *mut c_void) };
    unsafe { xstrnsave(buf, iobufflen) }
}

/// The `do_cmdline` line getter a function body is executed through.  It
/// also drives the debugger's breakpoints and the line profiler.
///
/// Keeps the raw signature: `getline_equal` compares this function's *address*
/// against the cookie's getter to decide whether a function is running.
///
/// # Safety
/// `cookie` is the `FuncCall` of the call in progress.
pub unsafe fn get_func_line(
    _c: c_int,
    cookie: *mut c_void,
    _indent: c_int,
    _do_concat: bool,
) -> *mut c_char {
    let fcp = cookie as *mut FuncCall;
    // SAFETY: the caller's promise -- `cookie` is the funccall of the call
    // in progress, so it and its function are live, and its body garray does
    // not move while the function is running.
    let mut frame = unsafe { Fc::new(fcp) };
    let fp = frame.fc_func;
    let f = unsafe { Uf::new(fp) };
    let lines = || ga_strings(unsafe { &(*fp).uf_lines });
    let name = uf_name_ptr(fp);

    // Check for a breakpoint set after the sourcing started.
    if frame.fc_dbg_tick != debug_tick.get() {
        frame.fc_breakpoint = unsafe { dbg_find_breakpoint(false, name, sourcing_lnum()) };
        frame.fc_dbg_tick = debug_tick.get();
    }
    if do_profiling.get() == PROF_YES {
        unsafe { func_line_end(cookie) };
    }
    let retval = if (f.uf_flags.has(FuncFlags::ABORT) && did_emsg.get() != 0 && !aborted_in_try())
        || frame.fc_returned != 0
    {
        ptr::null_mut()
    } else {
        // Skip NULL lines, they are continuation lines.
        while frame.fc_linenr < f.uf_lines.ga_len && lines()[frame.fc_linenr as usize].is_null() {
            frame.fc_linenr += 1;
        }
        if frame.fc_linenr >= f.uf_lines.ga_len {
            ptr::null_mut()
        } else {
            let line = lines()[frame.fc_linenr as usize];
            frame.fc_linenr += 1;
            let dup = unsafe { xstrdup(line) };
            crate::runtime::set_sourcing_lnum(frame.fc_linenr as LineNr);
            if do_profiling.get() == PROF_YES {
                unsafe { func_line_start(cookie) };
            }
            dup
        }
    };

    // Did we encounter a breakpoint?
    if frame.fc_breakpoint != 0 && frame.fc_breakpoint <= sourcing_lnum() {
        let at = sourcing_lnum();
        // SAFETY: the running function's name, live with it.
        dbg_breakpoint(unsafe { CStr::from_ptr(name) }, at);
        // Find the next breakpoint.
        frame.fc_breakpoint = unsafe { dbg_find_breakpoint(false, name, at) };
        frame.fc_dbg_tick = debug_tick.get();
    }

    retval
}

/// Whether the function running under `cookie` has ended.
///
/// # Safety
/// `cookie` is a `FuncCall`.
pub unsafe fn func_has_ended(cookie: *mut c_void) -> c_int {
    let fcp = cookie as *mut FuncCall;
    ((unsafe { (*(*fcp).fc_func).uf_flags }.has(FuncFlags::ABORT)
        && did_emsg.get() != 0
        && !aborted_in_try())
        || unsafe { (*fcp).fc_returned } != 0) as c_int
}

/// Whether the function running under `cookie` was declared `abort`.
///
/// # Safety
/// `cookie` is a `FuncCall`.
pub unsafe fn func_has_abort(cookie: *mut c_void) -> c_int {
    // SAFETY: the caller's promise.
    let flags = unsafe { (*(*(cookie as *mut FuncCall)).fc_func).uf_flags };
    flags.masked(FuncFlags::ABORT).bits()
}

/// The name of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `FuncCall`.
pub unsafe fn func_name(cookie: *mut c_void) -> *mut c_char {
    unsafe { uf_name_ptr((*(cookie as *mut FuncCall)).fc_func) }
}

/// The breakpoint line of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `FuncCall`.
pub unsafe fn func_breakpoint(cookie: *mut c_void) -> *mut LineNr {
    unsafe { &raw mut (*(cookie as *mut FuncCall)).fc_breakpoint }
}

/// The debug tick of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `FuncCall`.
pub unsafe fn func_dbg_tick(cookie: *mut c_void) -> *mut c_int {
    unsafe { &raw mut (*(cookie as *mut FuncCall)).fc_dbg_tick }
}

/// The `:if`/`:while` nesting level of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `FuncCall`.
pub unsafe fn func_level(cookie: *mut c_void) -> c_int {
    unsafe { (*(cookie as *mut FuncCall)).fc_level }
}

/// Whether the function running has already returned.
pub(crate) fn current_func_returned() -> c_int {
    unsafe { (*current_fc()).fc_returned }
}
