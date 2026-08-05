//! `:return`, `:call`, `:defer`, and the do_cmdline cookie.
//!
//! `ex_return`/`do_return` implement returning -- including the case where
//! a `:finally` is still to run -- and `get_return_cmd` renders a pending
//! return for the debugger.  `get_func_line` and the small accessors below
//! it are the `do_cmdline` cookie interface a function body is executed
//! through.  `:defer` records a call to make on the way out and
//! `invoke_all_defer` makes them.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

#[allow(unused_imports)]
use super::*;

/// One call recorded by `:defer`, to be made when the function returns.
#[derive(Copy, Clone)]
#[repr(C)]
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
    unsafe {
        let arg = (*eap).arg;
        let mut rettv = TV_INITIAL_VALUE;
        let mut returning = false;

        if current_funccal.get().is_null() {
            emsg(gettext(c"E133: :return not inside a function".as_ptr()));
            return;
        }

        let mut evalarg = EVALARG_INIT;
        evalarg.eval_flags = if (*eap).skip != 0 { 0 } else { EVAL_EVALUATE };

        if (*eap).skip != 0 {
            *emsg_skip.ptr() += 1;
        }

        (*eap).nextcmd = ptr::null_mut();
        if *arg != NUL as c_char
            && *arg != b'|' as c_char
            && *arg != b'\n' as c_char
            && eval0(arg, &raw mut rettv, eap, &raw mut evalarg) != FAIL
        {
            if (*eap).skip == 0 {
                returning = do_return(eap, false, true, (&raw mut rettv) as *mut c_void);
            } else {
                tv_clear(&raw mut rettv);
            }
        } else if (*eap).skip == 0 {
            // It's safer to return also on error.
            update_force_abort();

            // Return unless the expression evaluation was cancelled by an
            // aborting error, an interrupt or an exception.
            if !aborting() {
                returning = do_return(eap, false, true, ptr::null_mut());
            }
        }

        // When skipping or the return gets pending, advance to the next
        // command in this line; otherwise the whole line is used.
        if returning {
            (*eap).nextcmd = ptr::null_mut();
        } else if (*eap).nextcmd.is_null() {
            (*eap).nextcmd = check_nextcmd(arg);
        }

        if (*eap).skip != 0 {
            *emsg_skip.ptr() -= 1;
        }
        clear_evalarg(&raw mut evalarg, eap);
    }
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
    unsafe {
        let mut doesrange = false;
        let mut failed = false;
        let mut lnum = (*eap).line1;

        while lnum <= (*eap).line2 {
            if (*eap).addr_count > 0 {
                // Default is the line number, not the range.
                if lnum > (*curbuf.get()).b_ml.ml_line_count {
                    emsg(gettext(&raw const e_invrange as *const c_char));
                    break;
                }
                (*curwin.get()).w_cursor.lnum = lnum;
                (*curwin.get()).w_cursor.col = 0;
                (*curwin.get()).w_cursor.coladd = 0;
            }
            *arg = startarg;

            let mut funcexe = *funcexe_init;
            funcexe.fe_doesrange = &raw mut doesrange;
            let mut rettv = TV_INITIAL_VALUE;
            if get_func_tv(name, -1, &raw mut rettv, arg, evalarg, &raw mut funcexe) == FAIL {
                failed = true;
                break;
            }
            // Handle a trailing subscript, e.g. `:call f()[1]()`.
            if handle_subscript(
                arg as *mut *const c_char,
                &raw mut rettv,
                EVALARG_EVALUATE.ptr(),
                true,
            ) == FAIL
            {
                failed = true;
                break;
            }
            tv_clear(&raw mut rettv);
            if doesrange || aborting() {
                break;
            }
            lnum += 1;
        }
        failed
    }
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
) -> c_int {
    unsafe {
        let mut argvars = [TV_INITIAL_VALUE; MAX_FUNC_ARGS as usize + 1];
        let mut partial_argc = 0;
        let mut argcount = 0;

        if current_funccal.get().is_null() {
            semsg(
                gettext(&raw const e_str_not_inside_function as *const c_char),
                c"defer".as_ptr(),
            );
            return FAIL;
        }

        if !partial.is_null() {
            if !(*partial).pt_dict.is_null() {
                emsg(gettext(
                    E_CANNOT_USE_PARTIAL_WITH_DICTIONARY_FOR_DEFER.as_ptr(),
                ));
                return FAIL;
            }
            if (*partial).pt_argc > 0 {
                partial_argc = (*partial).pt_argc;
                for i in 0..partial_argc {
                    tv_copy(
                        (*partial).pt_argv.offset(i as isize),
                        argvars.as_mut_ptr().offset(i as isize),
                    );
                }
            }
        }

        // Upstream passes `false` for the partial argument count here; the
        // room already taken is accounted for by the `argvars` offset below.
        let mut r = get_func_arguments(
            arg,
            evalarg,
            0,
            argvars.as_mut_ptr().offset(partial_argc as isize),
            &raw mut argcount,
        );
        argcount += partial_argc;

        if r == OK {
            if builtin_function(name, -1) {
                let fdef = find_internal_func(name);
                if fdef.is_null() {
                    emsg_funcname(&raw const e_unknown_function_str as *const c_char, name);
                    r = FAIL;
                } else if check_internal_func(fdef, argcount) == -1 {
                    r = FAIL;
                }
            } else {
                let ufunc = find_func(name);
                if !ufunc.is_null() {
                    let error = check_user_func_argcount(ufunc, argcount);
                    if error != FCERR_UNKNOWN {
                        user_func_error(error, name, false);
                        r = FAIL;
                    }
                }
            }
        }

        if r == FAIL {
            while argcount > 0 {
                argcount -= 1;
                tv_clear(argvars.as_mut_ptr().offset(argcount as isize));
            }
            return FAIL;
        }
        add_defer(name, argcount, argvars.as_mut_ptr());
        OK
    }
}

/// Whether a `:defer` can be recorded here, i.e. whether a function is
/// running.  Reports the error itself when it cannot.
pub unsafe fn can_add_defer() -> bool {
    unsafe {
        if get_current_funccal().is_null() {
            semsg(
                gettext(&raw const e_str_not_inside_function as *const c_char),
                c"defer".as_ptr(),
            );
            return false;
        }
        true
    }
}

/// Record a deferred call of `name` on the funccall that is running.  It
/// takes over the values in `argvars`.
///
/// # Safety
/// A function is running, `name` is NUL-terminated and `argvars` holds
/// `argcount_arg` values.
pub unsafe fn add_defer(name: *mut c_char, argcount_arg: c_int, argvars: *mut typval_T) {
    unsafe {
        let saved_name = xstrdup(name);
        let mut argcount = argcount_arg;

        let fc = current_funccal.get();
        if (*fc).fc_defer.ga_itemsize == 0 {
            ga_init(&raw mut (*fc).fc_defer, size_of::<defer_T>() as c_int, 10);
        }
        let dr = ga_append_via_ptr(&raw mut (*fc).fc_defer, size_of::<defer_T>()) as *mut defer_T;
        (*dr).dr_name = saved_name;
        (*dr).dr_argcount = argcount;
        while argcount > 0 {
            argcount -= 1;
            (*dr).dr_argvars[argcount as usize] = *argvars.offset(argcount as isize);
        }
    }
}

/// Make the calls `:defer` recorded on `funccal`, newest first.
///
/// # Safety
/// `funccal` is a live funccall.
pub(crate) unsafe fn handle_defer_one(funccal: *mut funccall_T) {
    unsafe {
        let mut idx = (*funccal).fc_defer.ga_len - 1;
        while idx >= 0 {
            let dr = ((*funccal).fc_defer.ga_data as *mut defer_T).offset(idx as isize);
            if !(*dr).dr_name.is_null() {
                let mut funcexe = FUNCEXE_INIT;
                funcexe.fe_evaluate = true;
                let mut rettv = TV_INITIAL_VALUE;

                // Clear the name first, so that a deferred call that itself
                // throws cannot make this one run twice.
                let name = (*dr).dr_name;
                (*dr).dr_name = ptr::null_mut();

                // The deferred call runs with a clean exception state, so
                // that it happens even while an exception is in flight.
                let mut estate: exception_state_T = core::mem::zeroed();
                exception_state_save(&raw mut estate);
                exception_state_clear();

                call_func(
                    name,
                    -1,
                    &raw mut rettv,
                    (*dr).dr_argcount,
                    (&raw mut (*dr).dr_argvars) as *mut typval_T,
                    &raw mut funcexe,
                );

                exception_state_restore(&raw mut estate);
                tv_clear(&raw mut rettv);
                xfree(name as *mut c_void);
                let mut i = (*dr).dr_argcount - 1;
                while i >= 0 {
                    tv_clear(((&raw mut (*dr).dr_argvars) as *mut typval_T).offset(i as isize));
                    i -= 1;
                }
            }
            idx -= 1;
        }
        ga_clear(&raw mut (*funccal).fc_defer);
    }
}

/// Make every deferred call on every funccall, which is what an exit does.
pub unsafe fn invoke_all_defer() {
    unsafe {
        let mut fc = current_funccal.get();
        while !fc.is_null() {
            handle_defer_one(fc);
            fc = (*fc).fc_caller;
        }
        let mut fce = funccal_stack.get();
        while !fce.is_null() {
            let mut fc = (*fce).top_funccal as *mut funccall_T;
            while !fc.is_null() {
                handle_defer_one(fc);
                fc = (*fc).fc_caller;
            }
            fce = (*fce).next;
        }
    }
}

/// `:call` and `:defer`.
///
/// # Safety
/// `eap` is a live `:call`/`:defer` command.
pub unsafe fn ex_call(eap: *mut exarg_T) {
    unsafe {
        let mut arg = (*eap).arg;
        let mut fudi = FUNCDICT_INIT;
        let mut partial: *mut partial_T = ptr::null_mut();
        let mut evalarg = EVALARG_INIT;
        fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);

        if (*eap).skip != 0 {
            // Trailing arguments are still evaluated, so that errors in them
            // are reported -- but nothing is called.
            let mut rettv = TV_INITIAL_VALUE;
            *emsg_skip.ptr() += 1;
            if eval0((*eap).arg, &raw mut rettv, eap, &raw mut evalarg) != FAIL {
                tv_clear(&raw mut rettv);
            }
            *emsg_skip.ptr() -= 1;
            clear_evalarg(&raw mut evalarg, eap);
            return;
        }

        let tofree = trans_function_name(
            &raw mut arg,
            false,
            TFN_INT,
            &raw mut fudi,
            &raw mut partial,
        );
        if !fudi.fd_newkey.is_null() {
            // Still need to give an error message for missing key.
            semsg(
                gettext(&raw const e_dictkey as *const c_char),
                fudi.fd_newkey,
            );
            xfree(fudi.fd_newkey as *mut c_void);
        }
        if tofree.is_null() {
            return;
        }

        // Increase the reference on the dictionary, it could get deleted when
        // evaluating the arguments.
        if !fudi.fd_dict.is_null() {
            (*fudi.fd_dict).dv_refcount += 1;
        }

        // If it is the name of a variable of type VAR_FUNC or VAR_PARTIAL use
        // its contents; `trans_function_name` skips over "s:" and "g:".
        let mut len = strlen(tofree) as c_int;
        let mut found_var = false;
        let name = deref_func_name(
            tofree,
            &raw mut len,
            if partial.is_null() {
                &raw mut partial
            } else {
                ptr::null_mut()
            },
            false,
            &raw mut found_var,
        );

        let startarg = skipwhite(arg);
        if *startarg != b'(' as c_char {
            semsg(
                gettext(&raw const e_missingparen as *const c_char),
                (*eap).arg,
            );
        } else {
            let failed = if (*eap).cmdidx == CMD_defer {
                arg = startarg;
                ex_defer_inner(name, &raw mut arg, partial, &raw mut evalarg) == FAIL
            } else {
                let mut funcexe = FUNCEXE_INIT;
                funcexe.fe_partial = partial;
                funcexe.fe_selfdict = fudi.fd_dict;
                funcexe.fe_firstline = (*eap).line1;
                funcexe.fe_lastline = (*eap).line2;
                funcexe.fe_found_var = found_var;
                funcexe.fe_evaluate = true;
                ex_call_inner(
                    eap,
                    name,
                    &raw mut arg,
                    startarg,
                    &raw mut funcexe,
                    &raw mut evalarg,
                )
            };

            // When inside a `:try` the trailing text is still checked, so
            // that an error is reported for it rather than swallowed.
            if (!aborting() || did_throw.get()) && (!failed || (*(*eap).cstack).cs_trylevel > 0) {
                if ends_excmd(*arg as c_int) == 0 {
                    if !failed && !aborting() {
                        emsg_severe.set(true);
                        semsg(gettext(&raw const e_trailing_arg as *const c_char), arg);
                    }
                } else {
                    (*eap).nextcmd = check_nextcmd(arg);
                }
            }
            clear_evalarg(&raw mut evalarg, eap);
        }

        tv_dict_unref(fudi.fd_dict);
        xfree(tofree as *mut c_void);
    }
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
    unsafe {
        let mut rettv = rettv;
        let cstack = (*eap).cstack;

        if reanimate {
            // Undo the return.
            (*current_funccal.get()).fc_returned = 0;
        }

        // Cleanup (and inactivate) conditionals, but stop when a `:finally`
        // is reached: the return still has to be pending until that has run.
        let idx = cleanup_conditionals((*eap).cstack, 0, 1);
        if idx >= 0 {
            // A `:finally` is going to run first; remember the return value.
            (*cstack).cs_pending[idx as usize] = CSTP_RETURN as c_char;

            if !is_cmd && !reanimate {
                // A pending return again gets pending: `rettv` points to an
                // allocated variable with the value of the original return.
                (*cstack).cs_pend.csp_rv[idx as usize] = rettv;
            } else {
                if reanimate {
                    debug_assert!(!(*current_funccal.get()).fc_rettv.is_null());
                    rettv = (*current_funccal.get()).fc_rettv as *mut c_void;
                }
                if rettv.is_null() {
                    (*cstack).cs_pend.csp_rv[idx as usize] = ptr::null_mut();
                } else {
                    // Store the value of the pending return.
                    (*cstack).cs_pend.csp_rv[idx as usize] = xcalloc(1, size_of::<typval_T>());
                    *((*cstack).cs_pend.csp_rv[idx as usize] as *mut typval_T) =
                        *(rettv as *mut typval_T);
                }
                if reanimate {
                    // The return value is not available yet.
                    (*(*current_funccal.get()).fc_rettv).v_type = VAR_NUMBER;
                    (*(*current_funccal.get()).fc_rettv).vval.v_number = 0;
                }
            }
            report_make_pending(CSTP_RETURN, rettv);
        } else {
            (*current_funccal.get()).fc_returned = 1;
            if !reanimate && !rettv.is_null() {
                tv_clear((*current_funccal.get()).fc_rettv);
                *(*current_funccal.get()).fc_rettv = *(rettv as *mut typval_T);
                if !is_cmd {
                    xfree(rettv);
                }
            }
        }

        idx < 0
    }
}

/// Render `:return <expr>` for the debugger, in allocated memory.
///
/// # Safety
/// `rettv` is null or a `typval_T`.
pub unsafe fn get_return_cmd(rettv: *mut c_void) -> *mut c_char {
    unsafe {
        let mut s: *mut c_char = ptr::null_mut();
        let mut tofree: *mut c_char = ptr::null_mut();
        let mut slen: size_t = 0;

        if !rettv.is_null() {
            s = encode_tv2echo(rettv as *mut typval_T, ptr::null_mut());
            tofree = s;
        }
        if s.is_null() {
            s = c"".as_ptr() as *mut c_char;
        } else {
            slen = strlen(s);
        }

        const PREFIX: &CStr = c":return ";
        let buf = IObuff.ptr() as *mut c_char;
        xstrlcpy(buf, PREFIX.as_ptr(), IOSIZE as size_t);
        xstrlcpy(
            buf.add(PREFIX.count_bytes()),
            s,
            (IOSIZE as size_t) - PREFIX.count_bytes(),
        );
        let mut iobufflen = PREFIX.count_bytes() + slen;
        if iobufflen >= IOSIZE as size_t {
            strcpy(buf.offset(IOSIZE as isize - 4), c"...".as_ptr());
            iobufflen = IOSIZE as size_t - 1;
        }
        xfree(tofree as *mut c_void);
        xstrnsave(buf, iobufflen)
    }
}

/// The `do_cmdline` line getter a function body is executed through.  It
/// also drives the debugger's breakpoints and the line profiler.
///
/// Stays `extern "C"`: `getline_equal` compares this function's *address*
/// against the cookie's getter to decide whether a function is running.
///
/// # Safety
/// `cookie` is the `funccall_T` of the call in progress.
pub unsafe extern "C" fn get_func_line(
    _c: c_int,
    cookie: *mut c_void,
    _indent: c_int,
    _do_concat: bool,
) -> *mut c_char {
    unsafe {
        let fcp = cookie as *mut funccall_T;
        let fp = (*fcp).fc_func;

        // Check for a breakpoint set after the sourcing started.
        if (*fcp).fc_dbg_tick != debug_tick.get() {
            (*fcp).fc_breakpoint = dbg_find_breakpoint(false, uf_name_ptr(fp), sourcing_lnum());
            (*fcp).fc_dbg_tick = debug_tick.get();
        }
        if do_profiling.get() == PROF_YES {
            func_line_end(cookie);
        }

        let gap = &raw mut (*fp).uf_lines;
        let retval = if ((*fp).uf_flags & FC_ABORT != 0 && did_emsg.get() != 0 && !aborted_in_try())
            || (*fcp).fc_returned != 0
        {
            ptr::null_mut()
        } else {
            // Skip NULL lines, they are continuation lines.
            while (*fcp).fc_linenr < (*gap).ga_len
                && ga_strings(&*gap)[(*fcp).fc_linenr as usize].is_null()
            {
                (*fcp).fc_linenr += 1;
            }
            if (*fcp).fc_linenr >= (*gap).ga_len {
                ptr::null_mut()
            } else {
                let line = ga_strings(&*gap)[(*fcp).fc_linenr as usize];
                (*fcp).fc_linenr += 1;
                let dup = xstrdup(line);
                (*sourcing_entry()).es_lnum = (*fcp).fc_linenr as linenr_T;
                if do_profiling.get() == PROF_YES {
                    func_line_start(cookie);
                }
                dup
            }
        };

        // Did we encounter a breakpoint?
        if (*fcp).fc_breakpoint != 0 && (*fcp).fc_breakpoint <= sourcing_lnum() {
            dbg_breakpoint(uf_name_ptr(fp), sourcing_lnum());
            // Find the next breakpoint.
            (*fcp).fc_breakpoint = dbg_find_breakpoint(false, uf_name_ptr(fp), sourcing_lnum());
            (*fcp).fc_dbg_tick = debug_tick.get();
        }

        retval
    }
}

/// Whether the function running under `cookie` has ended.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe extern "C" fn func_has_ended(cookie: *mut c_void) -> c_int {
    unsafe {
        let fcp = cookie as *mut funccall_T;
        (((*(*fcp).fc_func).uf_flags & FC_ABORT != 0 && did_emsg.get() != 0 && !aborted_in_try())
            || (*fcp).fc_returned != 0) as c_int
    }
}

/// Whether the function running under `cookie` was declared `abort`.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe extern "C" fn func_has_abort(cookie: *mut c_void) -> c_int {
    unsafe { (*(*(cookie as *mut funccall_T)).fc_func).uf_flags & FC_ABORT }
}

/// The name of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe extern "C" fn func_name(cookie: *mut c_void) -> *mut c_char {
    unsafe { uf_name_ptr((*(cookie as *mut funccall_T)).fc_func) }
}

/// The breakpoint line of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe extern "C" fn func_breakpoint(cookie: *mut c_void) -> *mut linenr_T {
    unsafe { &raw mut (*(cookie as *mut funccall_T)).fc_breakpoint }
}

/// The debug tick of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe extern "C" fn func_dbg_tick(cookie: *mut c_void) -> *mut c_int {
    unsafe { &raw mut (*(cookie as *mut funccall_T)).fc_dbg_tick }
}

/// The `:if`/`:while` nesting level of the function running under `cookie`.
///
/// # Safety
/// `cookie` is a `funccall_T`.
pub unsafe extern "C" fn func_level(cookie: *mut c_void) -> c_int {
    unsafe { (*(cookie as *mut funccall_T)).fc_level }
}

/// Whether the function running has already returned.
pub unsafe fn current_func_returned() -> c_int {
    unsafe { (*current_funccal.get()).fc_returned }
}
