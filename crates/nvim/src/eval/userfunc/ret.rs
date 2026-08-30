//! `:return`, `:call`, `:defer`, and the do_cmdline cookie.
//!
//! `ex_return`/`do_return` implement returning -- including the case where
//! a `:finally` is still to run -- and `get_return_cmd` renders a pending
//! return for the debugger.  `get_func_line` and the small accessors below
//! it are the `do_cmdline` cookie interface a function body is executed
//! through.  `:defer` records a call to make on the way out and
//! `invoke_all_defer` makes them.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::guard::Suppress;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::os::cshim::gettext_ptr;
use crate::types::{Failed, IOSIZE, NUL};

/// One call recorded by `:defer`, to be made when the function returns.
#[derive(Copy, Clone)]
pub struct defer_T {
    pub dr_name: *mut c_char,
    pub dr_argvars: [typval_T; MAX_FUNC_ARGS as usize + 1],
    pub dr_argcount: c_int,
}

/// A zeroed `evalarg_T`: no flags, no line getter, nothing to free.
const EVALARG_INIT: evalarg_T = evalarg_T {
    eval_flags: 0,
    eval_getline: None,
    eval_cookie: ptr::null_mut(),
    eval_tofree: ptr::null_mut(),
};

/// `:return [expr]`.
///
/// # Safety
/// `eap` is a live `:return` command.
pub unsafe fn ex_return(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- `eap` is the Ex command being run.
    let mut ea = unsafe { Ea::new(eap) };
    let arg = ea.arg;
    let mut rettv = TV_INITIAL_VALUE;
    let mut returning = false;

    if current_funccal.get().is_null() {
        emsg(gettext(c"E133: :return not inside a function"));
        return;
    }

    let mut evalarg = EVALARG_INIT;
    evalarg.eval_flags = if ea.skip != 0 { 0 } else { EVAL_EVALUATE };

    let skipping = (ea.skip != 0).then(Suppress::emsg_skip);

    ea.nextcmd = ptr::null_mut();
    if unsafe { *arg } != NUL as c_char
        && unsafe { *arg } != b'|' as c_char
        && unsafe { *arg } != b'\n' as c_char
        && unsafe { eval0(arg, &raw mut rettv, eap, &raw mut evalarg) }.is_ok()
    {
        if ea.skip == 0 {
            returning = unsafe { do_return(eap, false, true, (&raw mut rettv) as *mut c_void) };
        } else {
            unsafe { tv_clear(&raw mut rettv) };
        }
    } else if ea.skip == 0 {
        // It's safer to return also on error.
        update_force_abort();

        // Return unless the expression evaluation was cancelled by an
        // aborting error, an interrupt or an exception.
        if !aborting() {
            returning = unsafe { do_return(eap, false, true, ptr::null_mut()) };
        }
    }

    // When skipping or the return gets pending, advance to the next
    // command in this line; otherwise the whole line is used.
    if returning {
        ea.nextcmd = ptr::null_mut();
    } else if ea.nextcmd.is_null() {
        ea.nextcmd = unsafe { check_nextcmd(arg) };
    }

    drop(skipping);
    unsafe { clear_evalarg(&raw mut evalarg, eap) };
}

/// Make the call `:call` asks for, once per line of its range.
///
/// # Safety
/// `eap` is a live `:call`, `name` the resolved function name, and
/// `startarg` the `(` its arguments start at.
unsafe fn ex_call_inner(
    eap: *mut exarg_T,
    name: *mut c_char,
    arg: *mut *mut c_char,
    startarg: *mut c_char,
    funcexe_init: *const funcexe_T,
    evalarg: *mut evalarg_T,
) -> bool {
    // SAFETY: the caller's promise -- `eap` is the Ex command being run.
    let mut ea = unsafe { Ea::new(eap) };
    // The subscript after `:call f()` is evaluated for real whatever the
    // caller's `evalarg` says, so it gets one of its own.
    let mut subscript_evalarg = EVALARG_EVALUATE;
    let mut doesrange = false;
    let mut failed = false;
    let mut lnum = ea.line1;

    while lnum <= ea.line2 {
        if ea.addr_count > 0 {
            // Default is the line number, not the range.
            if lnum > cur_buf().b_ml.ml_line_count {
                emsg(gettext(e_invrange));
                break;
            }
            cur_win().w_cursor.lnum = lnum;
            cur_win().w_cursor.col = 0;
            cur_win().w_cursor.coladd = 0;
        }
        unsafe { *arg = startarg };

        let mut funcexe = unsafe { *funcexe_init };
        funcexe.fe_doesrange = &raw mut doesrange;
        let mut rettv = TV_INITIAL_VALUE;
        if unsafe { get_func_tv(name, -1, &raw mut rettv, arg, evalarg, &raw mut funcexe) }.is_err()
        {
            failed = true;
            break;
        }
        // Handle a trailing subscript, e.g. `:call f()[1]()`.
        let (ret, ev) = (&raw mut rettv, &raw mut subscript_evalarg);
        if unsafe { handle_subscript(arg as *mut *const c_char, ret, ev, true) }.is_err() {
            failed = true;
            break;
        }
        unsafe { tv_clear(&raw mut rettv) };
        if doesrange || aborting() {
            break;
        }
        lnum += 1;
    }
    failed
}

/// `:defer Func(args)`: check the call now and record it for the way out.
///
/// # Safety
/// `name` is the resolved function name and `*arg` its `(`.
unsafe fn ex_defer_inner(
    name: *mut c_char,
    arg: *mut *mut c_char,
    partial: *const partial_T,
    evalarg: *mut evalarg_T,
) -> Result<(), Failed> {
    let mut argvars = [TV_INITIAL_VALUE; MAX_FUNC_ARGS as usize + 1];
    let mut partial_argc = 0;
    let mut argcount = 0;

    if current_funccal.get().is_null() {
        let arg0 = "defer";
        semsg!("E193: {arg0} not inside a function");
        return Err(Failed);
    }

    if !partial.is_null() {
        if !unsafe { (*partial).pt_dict }.is_null() {
            let fmt = E_CANNOT_USE_PARTIAL_WITH_DICTIONARY_FOR_DEFER.as_ptr();
            unsafe { emsg(gettext_ptr(fmt)) };
            return Err(Failed);
        }
        if unsafe { (*partial).pt_argc } > 0 {
            partial_argc = unsafe { (*partial).pt_argc };
            // SAFETY: the partial has `partial_argc` bound arguments and
            // `argvars` has room for them.
            let bound = unsafe { (*partial).pt_argv };
            let into = argvars.as_mut_ptr();
            for i in 0..partial_argc {
                unsafe { tv_copy(bound.offset(i as isize), into.offset(i as isize)) };
            }
        }
    }

    // Upstream passes `false` for the partial argument count here; the
    // room already taken is accounted for by the `argvars` offset below.
    // SAFETY: `argvars` has room past the `partial_argc` slots already
    // taken, and `argcount` is this frame's local.
    let free_slot = unsafe { argvars.as_mut_ptr().offset(partial_argc as isize) };
    let countp = &raw mut argcount;
    let mut r = unsafe { get_func_arguments(arg, evalarg, 0, free_slot, countp) };
    argcount += partial_argc;

    if r.is_ok() {
        if unsafe { builtin_function(name, -1) } {
            let fdef = unsafe { find_internal_func(name) };
            if fdef.is_null() {
                unsafe { emsg_funcname(e_unknown_function_str.as_ptr(), name) };
                r = Err(Failed);
            } else if unsafe { check_internal_func(fdef, argcount) } == -1 {
                r = Err(Failed);
            }
        } else {
            let ufunc = unsafe { find_func(name) };
            if !ufunc.is_null() {
                let error = unsafe { check_user_func_argcount(ufunc, argcount) };
                if error != FCERR_UNKNOWN {
                    unsafe { user_func_error(error, name, false) };
                    r = Err(Failed);
                }
            }
        }
    }

    if r.is_err() {
        while argcount > 0 {
            argcount -= 1;
            unsafe { tv_clear(argvars.as_mut_ptr().offset(argcount as isize)) };
        }
        return Err(Failed);
    }
    unsafe { add_defer(name, argcount, argvars.as_mut_ptr()) };
    Ok(())
}

/// Whether a `:defer` can be recorded here, i.e. whether a function is
/// running.  Reports the error itself when it cannot.
pub unsafe fn can_add_defer() -> bool {
    if unsafe { get_current_funccal() }.is_null() {
        let arg0 = "defer";
        semsg!("E193: {arg0} not inside a function");
        return false;
    }
    true
}

/// Record a deferred call of `name` on the funccall that is running.  It
/// takes over the values in `argvars`.
///
/// # Safety
/// A function is running, `name` is NUL-terminated and `argvars` holds
/// `argcount_arg` values.
pub unsafe fn add_defer(name: *mut c_char, argcount_arg: c_int, argvars: *mut typval_T) {
    let saved_name = unsafe { xstrdup(name) };
    let mut argcount = argcount_arg;

    let fc = current_funccal.get();
    if unsafe { (*fc).fc_defer.ga_itemsize } == 0 {
        unsafe { ga_init(&raw mut (*fc).fc_defer, size_of::<defer_T>() as c_int, 10) };
    }
    let dr =
        unsafe { ga_append_via_ptr(&raw mut (*fc).fc_defer, size_of::<defer_T>()) } as *mut defer_T;
    unsafe { (*dr).dr_name = saved_name };
    unsafe { (*dr).dr_argcount = argcount };
    while argcount > 0 {
        argcount -= 1;
        unsafe { (*dr).dr_argvars[argcount as usize] = *argvars.offset(argcount as isize) };
    }
}

/// Make the calls `:defer` recorded on `funccal`, newest first.
///
/// # Safety
/// `funccal` is a live funccall.
pub(crate) unsafe fn handle_defer_one(funccal: *mut funccall_T) {
    // SAFETY: the caller's promise -- `funccal` is a live funccall.
    let mut frame = unsafe { Fc::new(funccal) };
    let mut idx = frame.fc_defer.ga_len - 1;
    while idx >= 0 {
        let dr = unsafe { (frame.fc_defer.ga_data as *mut defer_T).offset(idx as isize) };
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
            let mut estate: exception_state_T = unsafe { core::mem::zeroed() };
            unsafe { exception_state_save(&raw mut estate) };
            exception_state_clear();

            // SAFETY: `dr` is the deferred call's own record, so its
            // argument array holds `dr_argcount` values.
            let (argc, args) = unsafe { ((*dr).dr_argcount, &raw mut (*dr).dr_argvars) };
            let (ret, exe) = (&raw mut rettv, &raw mut funcexe);
            let _ = unsafe { call_func(name, -1, ret, argc, args as *mut typval_T, exe) };

            unsafe { exception_state_restore(&raw mut estate) };
            unsafe { tv_clear(&raw mut rettv) };
            unsafe { xfree(name as *mut c_void) };
            let mut i = unsafe { (*dr).dr_argcount } - 1;
            while i >= 0 {
                unsafe {
                    tv_clear(((&raw mut (*dr).dr_argvars) as *mut typval_T).offset(i as isize))
                };
                i -= 1;
            }
        }
        idx -= 1;
    }
    unsafe { ga_clear(&raw mut (*funccal).fc_defer) };
}

/// Make every deferred call on every funccall, which is what an exit does.
pub unsafe fn invoke_all_defer() {
    let mut fc = current_funccal.get();
    while !fc.is_null() {
        unsafe { handle_defer_one(fc) };
        fc = unsafe { (*fc).fc_caller };
    }
    let mut fce = funccal_stack.get();
    while !fce.is_null() {
        let mut fc = unsafe { (*fce).top_funccal } as *mut funccall_T;
        while !fc.is_null() {
            unsafe { handle_defer_one(fc) };
            fc = unsafe { (*fc).fc_caller };
        }
        fce = unsafe { (*fce).next };
    }
}

/// `:call` and `:defer`.
///
/// # Safety
/// `eap` is a live `:call`/`:defer` command.
pub unsafe fn ex_call(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- `eap` is the Ex command being run.
    let mut ea = unsafe { Ea::new(eap) };
    let mut arg = ea.arg;
    let mut fudi = FUNCDICT_INIT;
    let mut partial: *mut partial_T = ptr::null_mut();
    let mut evalarg = EVALARG_INIT;
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, eap, ea.skip != 0) };

    if ea.skip != 0 {
        // Trailing arguments are still evaluated, so that errors in them
        // are reported -- but nothing is called.
        let mut rettv = TV_INITIAL_VALUE;
        let skipping = Suppress::emsg_skip();
        if unsafe { eval0(ea.arg, &raw mut rettv, eap, &raw mut evalarg) }.is_ok() {
            unsafe { tv_clear(&raw mut rettv) };
        }
        drop(skipping);
        unsafe { clear_evalarg(&raw mut evalarg, eap) };
        return;
    }

    let (argp, dictp) = (&raw mut arg, &raw mut fudi);
    let partialp = &raw mut partial;
    // SAFETY: `arg` walks the caller's command line and the two
    // out-parameters are this frame's locals.
    let tofree = unsafe { trans_function_name(argp, false, TFN_INT, dictp, partialp) };
    if !fudi.fd_newkey.is_null() {
        // Still need to give an error message for missing key.
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fd_newkey = unsafe { c_str(fudi.fd_newkey) };
        semsg!("E716: Key not present in Dictionary: \"{fd_newkey}\"");
        unsafe { xfree(fudi.fd_newkey as *mut c_void) };
    }
    if tofree.is_null() {
        return;
    }

    // Increase the reference on the dictionary, it could get deleted when
    // evaluating the arguments.
    if !fudi.fd_dict.is_null() {
        unsafe { (*fudi.fd_dict).dv_refcount.retain() };
    }

    // If it is the name of a variable of type VAR_FUNC or VAR_PARTIAL use
    // its contents; `trans_function_name` skips over "s:" and "g:".
    let mut len = unsafe { cstr::bytes_at(tofree) }.len() as c_int;
    let mut found_var = false;
    let want_partial = if partial.is_null() {
        &raw mut partial
    } else {
        ptr::null_mut()
    };
    let (lenp, foundp) = (&raw mut len, &raw mut found_var);
    // SAFETY: `tofree` is the translated name and the out-parameters are
    // this frame's locals.
    let name = unsafe { deref_func_name(tofree, lenp, want_partial, false, foundp) };

    let startarg = unsafe { skipwhite(arg) };
    if unsafe { *startarg } != b'(' as c_char {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(ea.arg) };
        semsg!("E107: Missing parentheses: {arg}");
    } else {
        let failed = if ea.cmdidx == CMD_defer {
            arg = startarg;
            unsafe { ex_defer_inner(name, &raw mut arg, partial, &raw mut evalarg).is_err() }
        } else {
            let mut funcexe = FUNCEXE_INIT;
            funcexe.fe_partial = partial;
            funcexe.fe_selfdict = fudi.fd_dict;
            funcexe.fe_firstline = ea.line1;
            funcexe.fe_lastline = ea.line2;
            funcexe.fe_found_var = found_var;
            funcexe.fe_evaluate = true;
            let (argp, exe) = (&raw mut arg, &raw mut funcexe);
            let ev = &raw mut evalarg;
            unsafe { ex_call_inner(eap, name, argp, startarg, exe, ev) }
        };

        // When inside a `:try` the trailing text is still checked, so
        // that an error is reported for it rather than swallowed.
        if (!aborting() || did_throw.get()) && (!failed || unsafe { (*ea.cstack).cs_trylevel } > 0)
        {
            if ends_excmd(unsafe { *arg } as c_int) == 0 {
                if !failed && !aborting() {
                    emsg_severe.set(true);
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let arg = unsafe { c_str(arg) };
                    semsg!("E488: Trailing characters: {arg}");
                }
            } else {
                ea.nextcmd = unsafe { check_nextcmd(arg) };
            }
        }
        unsafe { clear_evalarg(&raw mut evalarg, eap) };
    }

    unsafe { tv_dict_unref(fudi.fd_dict) };
    unsafe { xfree(tofree as *mut c_void) };
}

/// Return from a function, answering whether the return happened now rather
/// than being made pending by a `:finally`.
///
/// # Safety
/// `eap` is a live command with a condition stack, and `rettv` is null or a
/// `typval_T`.
pub unsafe fn do_return(
    eap: *mut exarg_T,
    reanimate: bool,
    is_cmd: bool,
    rettv: *mut c_void,
) -> bool {
    // SAFETY: the caller's promise -- `eap` is the Ex command being run.
    let mut ea = unsafe { Ea::new(eap) };
    let mut rettv = rettv;
    let cstack = ea.cstack;

    if reanimate {
        // Undo the return.
        unsafe { (*current_funccal.get()).fc_returned = 0 };
    }

    // Cleanup (and inactivate) conditionals, but stop when a `:finally`
    // is reached: the return still has to be pending until that has run.
    let idx = unsafe { cleanup_conditionals(ea.cstack, 0, true) };
    if idx >= 0 {
        // A `:finally` is going to run first; remember the return value.
        unsafe { (*cstack).cs_pending[idx as usize] = CSTP_RETURN as c_char };

        if !is_cmd && !reanimate {
            // A pending return again gets pending: `rettv` points to an
            // allocated variable with the value of the original return.
            unsafe { (*cstack).cs_pend.csp_rv[idx as usize] = rettv };
        } else {
            if reanimate {
                debug_assert!(!unsafe { (*current_funccal.get()).fc_rettv }.is_null());
                rettv = unsafe { (*current_funccal.get()).fc_rettv } as *mut c_void;
            }
            if rettv.is_null() {
                unsafe { (*cstack).cs_pend.csp_rv[idx as usize] = ptr::null_mut() };
            } else {
                // Store the value of the pending return.
                unsafe {
                    (*cstack).cs_pend.csp_rv[idx as usize] = xcalloc(1, size_of::<typval_T>())
                };
                unsafe {
                    *((*cstack).cs_pend.csp_rv[idx as usize] as *mut typval_T) =
                        *(rettv as *mut typval_T)
                };
            }
            if reanimate {
                // The return value is not available yet.
                unsafe { (*(*current_funccal.get()).fc_rettv).v_type = VAR_NUMBER };
                unsafe { (*(*current_funccal.get()).fc_rettv).vval.v_number = 0 };
            }
        }
        unsafe { report_make_pending(CSTP_RETURN, rettv) };
    } else {
        unsafe { (*current_funccal.get()).fc_returned = 1 };
        if !reanimate && !rettv.is_null() {
            unsafe { tv_clear((*current_funccal.get()).fc_rettv) };
            unsafe { *(*current_funccal.get()).fc_rettv = *(rettv as *mut typval_T) };
            if !is_cmd {
                unsafe { xfree(rettv) };
            }
        }
    }

    idx < 0
}

/// Render `:return <expr>` for the debugger, in allocated memory.
///
/// # Safety
/// `rettv` is null or a `typval_T`.
pub unsafe fn get_return_cmd(rettv: *mut c_void) -> *mut c_char {
    // The rendered command. Upstream shares `IObuff`, which the debugger
    // this feeds writes again.
    let mut line = [0 as c_char; IOSIZE as usize];
    let mut s: *mut c_char = ptr::null_mut();
    let mut tofree: *mut c_char = ptr::null_mut();
    let mut slen: size_t = 0;

    if !rettv.is_null() {
        s = unsafe { encode_tv2echo(rettv as *mut typval_T, ptr::null_mut()) };
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
/// `cookie` is the `funccall_T` of the call in progress.
pub unsafe fn get_func_line(
    _c: c_int,
    cookie: *mut c_void,
    _indent: c_int,
    _do_concat: bool,
) -> *mut c_char {
    let fcp = cookie as *mut funccall_T;
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
    let retval = if (f.uf_flags & FC_ABORT != 0 && did_emsg.get() != 0 && !aborted_in_try())
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
            crate::runtime::set_sourcing_lnum(frame.fc_linenr as linenr_T);
            if do_profiling.get() == PROF_YES {
                unsafe { func_line_start(cookie) };
            }
            dup
        }
    };

    // Did we encounter a breakpoint?
    if frame.fc_breakpoint != 0 && frame.fc_breakpoint <= sourcing_lnum() {
        let at = sourcing_lnum();
        dbg_breakpoint(name, at);
        // Find the next breakpoint.
        frame.fc_breakpoint = unsafe { dbg_find_breakpoint(false, name, at) };
        frame.fc_dbg_tick = debug_tick.get();
    }

    retval
}

/// Whether the function running under `cookie` has ended.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe fn func_has_ended(cookie: *mut c_void) -> c_int {
    let fcp = cookie as *mut funccall_T;
    ((unsafe { (*(*fcp).fc_func).uf_flags } & FC_ABORT != 0
        && did_emsg.get() != 0
        && !aborted_in_try())
        || unsafe { (*fcp).fc_returned } != 0) as c_int
}

/// Whether the function running under `cookie` was declared `abort`.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe fn func_has_abort(cookie: *mut c_void) -> c_int {
    unsafe { (*(*(cookie as *mut funccall_T)).fc_func).uf_flags & FC_ABORT }
}

/// The name of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe fn func_name(cookie: *mut c_void) -> *mut c_char {
    unsafe { uf_name_ptr((*(cookie as *mut funccall_T)).fc_func) }
}

/// The breakpoint line of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe fn func_breakpoint(cookie: *mut c_void) -> *mut linenr_T {
    unsafe { &raw mut (*(cookie as *mut funccall_T)).fc_breakpoint }
}

/// The debug tick of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe fn func_dbg_tick(cookie: *mut c_void) -> *mut c_int {
    unsafe { &raw mut (*(cookie as *mut funccall_T)).fc_dbg_tick }
}

/// The `:if`/`:while` nesting level of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe fn func_level(cookie: *mut c_void) -> c_int {
    unsafe { (*(cookie as *mut funccall_T)).fc_level }
}

/// Whether the function running has already returned.
pub unsafe fn current_func_returned() -> c_int {
    unsafe { (*current_funccal.get()).fc_returned }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
