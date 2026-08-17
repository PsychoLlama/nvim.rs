//! `do_cmdline` — the loop that runs a sequence of Ex command lines.
//!
//! This is the re-entrant heart of the editor: a sourced file, `:execute`, a
//! `:global` body, an autocommand and a mapping all arrive here, nested
//! inside one another, each with its own conditional stack, its own share of
//! the exception state and its own store of the lines a `:while` or `:for`
//! will replay.
//!
//! One pass of the loop runs one `|`-separated command. Where the next one
//! comes from is the first decision it makes, and there are three answers:
//! replay a stored line (inside a loop), ask the line getter for one, or
//! take what `do_one_cmd` left after a `|`.
//!
//! Ordering is load-bearing throughout, and the exit condition at the bottom
//! is the specification of when a script stops on an error.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::debugger::{dbg_breakpoint, dbg_find_breakpoint, do_debug};
use crate::eval::userfunc::{
    func_breakpoint, func_dbg_tick, func_has_abort, func_has_ended, func_level, func_name,
    get_func_line,
};
use crate::ex_docmd::onecmd::do_one_cmd;
use crate::ex_docmd::source::{
    do_cmdline_end, do_cmdline_start, get_loop_line, getline_cookie, getline_equal,
    handle_did_throw, msg_verbose_cmd, restore_dbg_stuff, save_dbg_stuff, store_loop_line,
};
use crate::ex_docmd::{
    CSF_ACTIVE, CSF_FINALLY, CSF_FOR, CSF_TRY, CSF_WHILE, CSL_HAD_CONT, CSL_HAD_ENDLOOP,
    CSL_HAD_FINA, CSL_HAD_LOOP, CSTP_ERROR, CSTP_INTERRUPT, CSTP_THROW, DOCMD_EXCRESET,
    DOCMD_KEEPLINE, DOCMD_KEYTYPED, DOCMD_NOWAIT, DOCMD_REPEAT, DOCMD_VERBOSE, FAIL, OK, PROF_YES,
    dbg_stuff, loop_cookie, wcmd_T,
};
use crate::ex_eval::{
    aborting, cleanup_conditionals, do_errthrow, do_intthrow, has_loop_cmd, report_make_pending,
    rewind_conditionals,
};
use crate::ex_getln::{getexline, ui_ext_cmdline_block_append, ui_ext_cmdline_block_leave};
use crate::garray::{ga_clear, ga_init};
use crate::main::{
    KeyTyped, RedrawingDisabled, check_cstack, current_exception, debug_break_level, debug_tick,
    did_emsg, did_emsg_syntax, did_endif, did_throw, do_profiling, e_command_too_recursive,
    e_endfor, e_endif, e_endtry, e_endwhile, ex_nesting_level, force_abort, got_int, last_cmdline,
    msg_didany, msg_didout, msg_list, msg_scroll, need_rethrow, need_wait_return, new_last_cmdline,
    no_wait_return, p_verbose, repeat_cmdline, suppress_errthrow, trylevel,
};
use crate::memory::{xfree, xstrdup};
use crate::message::{emsg, msg_start, wait_return};
use crate::os::input::line_breakcheck;
use crate::os::libc::{gettext, memmove, memset, strlen};
use crate::profile::{func_line_end, func_line_start, script_line_end, script_line_start};
use crate::runtime::{
    exestack, getsourceline, source_breakpoint, source_dbg_tick, source_finished, source_level,
};
use crate::types::ui::kUICmdline;
use crate::types::{
    LineGetter, OptInt, cstack_T, eslist_T, estack_T, garray_T, linenr_T, msglist_T, size_t,
};
use crate::ui::ui_has;

/// The top of the execution stack: the script or function whose line is
/// running. `SOURCING_LNUM`/`SOURCING_NAME` in the C, where they are macros
/// over `exestack`'s last entry.
pub(crate) unsafe fn sourcing_entry() -> *mut estack_T {
    unsafe {
        let stack = &*exestack.ptr();
        (stack.ga_data as *mut estack_T).offset(stack.ga_len as isize - 1)
    }
}

/// The line number the message and breakpoint machinery reports.
pub(crate) unsafe fn sourcing_lnum() -> linenr_T {
    unsafe { (*sourcing_entry()).es_lnum }
}

/// A zeroed `cstack_T` with the "no conditional open" index the C's
/// `{ .cs_idx = -1 }` sets.
fn empty_cstack() -> cstack_T {
    // SAFETY: `cstack_T` is a `repr(C)` aggregate of scalars, arrays and
    // pointers; all-zero is a valid value of every one of them.
    let mut cstack: cstack_T = unsafe { core::mem::zeroed() };
    cstack.cs_idx = -1;
    cstack
}

/// Free every line a `:while`/`:for` body stored, and the array holding
/// them.
unsafe fn clear_loop_lines(gap: *mut garray_T) {
    unsafe {
        if !(*gap).ga_data.is_null() {
            for i in 0..(*gap).ga_len {
                let item = ((*gap).ga_data as *mut wcmd_T).offset(i as isize);
                xfree((*item).line as *mut c_void);
            }
        }
        ga_clear(gap);
    }
}

/// Run one Ex command line, as if the user had typed it.
pub unsafe fn do_cmdline_cmd(cmd: *const c_char) -> c_int {
    unsafe {
        do_cmdline(
            cmd as *mut c_char,
            None,
            ptr::null_mut(),
            DOCMD_VERBOSE as c_int | DOCMD_NOWAIT as c_int | DOCMD_KEYTYPED as c_int,
        )
    }
}

/// Run Ex commands, from `cmdline` and then from `fgetline`.
///
/// `flags` is a set of `DOCMD_*`:
/// - `VERBOSE` includes the command in any error message;
/// - `NOWAIT` skips `wait_return` and friends;
/// - `REPEAT` keeps asking `fgetline` until it answers null;
/// - `KEYTYPED` leaves 'KeyTyped' alone;
/// - `EXCRESET` saves and restores the exception environment (debugging);
/// - `KEEPLINE` remembers the first typed line, for `.` to repeat.
///
/// May be called recursively. Answers `FAIL` when the line could not be
/// run, `OK` otherwise.
pub unsafe fn do_cmdline(
    cmdline: *mut c_char,
    fgetline: LineGetter,
    cookie: *mut c_void,
    flags: c_int,
) -> c_int {
    unsafe {
        // How deep this call is inside other `do_cmdline` calls. Used to
        // decide whether this is the outermost one, which is the one that
        // owns the "wait for return" bookkeeping.
        static RECURSIVE: crate::global_cell::GlobalCell<c_int> =
            crate::global_cell::GlobalCell::new(0);

        let mut next_cmdline: *mut c_char;
        let mut cmdline_copy: *mut c_char = ptr::null_mut();
        let mut used_getline = false;
        let mut msg_didout_before_start = false;
        let mut count: c_int = 0;
        let mut did_inc = false;
        let mut did_block = false;
        let mut retval = OK;
        let mut cstack = empty_cstack();
        let mut lines_ga: garray_T = core::mem::zeroed();
        let mut current_line: c_int = 0;
        let mut fname: *mut c_char = ptr::null_mut();
        let mut breakpoint: *mut linenr_T = ptr::null_mut();
        let mut dbg_tick: *mut c_int = ptr::null_mut();
        let mut debug_saved: dbg_stuff = core::mem::zeroed();
        let mut cmd_loop_cookie: loop_cookie = core::mem::zeroed();

        // Every do_cmdline/do_one_cmd pair gets its own place to store the
        // error messages an exception may be built from. Without that, the
        // `do_errthrow` in `do_one_cmd` would join an earlier invocation's
        // messages to a later invocation's command name — which is what
        // happens when a BufWritePost autocommand runs after a write error.
        let mut private_msg_list: *mut msglist_T = ptr::null_mut();
        let saved_msg_list = msg_list.get();
        msg_list.set(&raw mut private_msg_list);

        if do_cmdline_start() == FAIL {
            emsg(gettext(&raw const e_command_too_recursive as *const c_char));
            // No command name: this is not an error of any one command.
            do_errthrow(ptr::null_mut(), ptr::null_mut());
            msg_list.set(saved_msg_list);
            return FAIL;
        }

        ga_init(&raw mut lines_ga, size_of::<wcmd_T>() as c_int, 10);

        let real_cookie = getline_cookie(fgetline, cookie);

        // Inside a function, use a higher nesting level.
        let mut getline_is_func = getline_equal(fgetline, cookie, Some(get_func_line));
        if getline_is_func && ex_nesting_level.get() == func_level(real_cookie) {
            *ex_nesting_level.ptr() += 1;
        }

        // Where the next breakpoint line and the debug tick live, for
        // whichever of a function or a script this is.
        if getline_is_func {
            fname = func_name(real_cookie);
            breakpoint = func_breakpoint(real_cookie);
            dbg_tick = func_dbg_tick(real_cookie);
        } else if getline_equal(fgetline, cookie, Some(getsourceline)) {
            fname = (*sourcing_entry()).es_name;
            breakpoint = source_breakpoint(real_cookie);
            dbg_tick = source_dbg_tick(real_cookie);
        }

        if RECURSIVE.get() == 0 {
            force_abort.set(false);
            suppress_errthrow.set(false);
        }

        if flags & DOCMD_EXCRESET as c_int != 0 {
            save_dbg_stuff(&raw mut debug_saved);
        } else {
            memset(
                &raw mut debug_saved as *mut c_void,
                0,
                size_of::<dbg_stuff>(),
            );
        }

        let initial_trylevel = trylevel.get();

        did_throw.set(false);
        // An `emsg` cancels the whole command line and any conditional or
        // loop around it. With 'force_abort' set, everything is cancelled.
        did_emsg.set(0);

        // 'KeyTyped' is only set by `vgetc`; a sourced line never went
        // through it.
        if flags & DOCMD_KEYTYPED as c_int == 0 && !getline_equal(fgetline, cookie, Some(getexline))
        {
            KeyTyped.set(false);
        }

        next_cmdline = cmdline;
        loop {
            getline_is_func = getline_equal(fgetline, cookie, Some(get_func_line));

            // Stop skipping commands after an error once every :endif,
            // :endwhile and :endfor has been passed.
            if next_cmdline.is_null()
                && !force_abort.get()
                && cstack.cs_idx < 0
                && !(getline_is_func && func_has_abort(real_cookie) != 0)
            {
                did_emsg.set(0);
            }

            // 1. Replaying a loop body: take the next stored line. Each
            //    `|`-separated command was stored separately, so that an
            //    `:endwhile` can jump back to exactly one of them.
            if cstack.cs_looplevel > 0 && current_line < lines_ga.ga_len {
                xfree(cmdline_copy as *mut c_void);
                cmdline_copy = ptr::null_mut();

                // Has the function returned, or (with no try conditional
                // still open) aborted?
                if getline_is_func {
                    if do_profiling.get() == PROF_YES {
                        func_line_end(real_cookie);
                    }
                    if func_has_ended(real_cookie) != 0 {
                        retval = FAIL;
                        break;
                    }
                } else if do_profiling.get() == PROF_YES
                    && getline_equal(fgetline, cookie, Some(getsourceline))
                {
                    script_line_end();
                }

                // Has the sourced file hit a `:finish`?
                if source_finished(fgetline, cookie) {
                    retval = FAIL;
                    break;
                }

                // Breakpoints may have been added or removed since the last
                // look.
                if !breakpoint.is_null() && !dbg_tick.is_null() && *dbg_tick != debug_tick.get() {
                    *breakpoint = dbg_find_breakpoint(
                        getline_equal(fgetline, cookie, Some(getsourceline)),
                        fname,
                        sourcing_lnum(),
                    );
                    *dbg_tick = debug_tick.get();
                }

                let stored = (lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize);
                next_cmdline = (*stored).line;
                (*sourcing_entry()).es_lnum = (*stored).lnum;

                if !breakpoint.is_null() && *breakpoint != 0 && *breakpoint <= sourcing_lnum() {
                    dbg_breakpoint(fname, sourcing_lnum());
                    *breakpoint = dbg_find_breakpoint(
                        getline_equal(fgetline, cookie, Some(getsourceline)),
                        fname,
                        sourcing_lnum(),
                    );
                    *dbg_tick = debug_tick.get();
                }
                if do_profiling.get() == PROF_YES {
                    if getline_is_func {
                        func_line_start(real_cookie);
                    } else if getline_equal(fgetline, cookie, Some(getsourceline)) {
                        script_line_start();
                    }
                }
            }

            // 2. No line to hand: ask the line getter for one.
            if next_cmdline.is_null() {
                let indent = if cstack.cs_idx < 0 {
                    0
                } else {
                    (cstack.cs_idx + 1) * 2
                };

                if count == 1 && getline_equal(fgetline, cookie, Some(getexline)) {
                    if ui_has(kUICmdline) {
                        ui_ext_cmdline_block_append(0, last_cmdline.get());
                        did_block = true;
                    }
                    // The first line after an `:if` needs this, or the `:if`
                    // is overwritten.
                    msg_didout.set(true);
                }

                next_cmdline = match fgetline {
                    Some(get) => get(':' as c_int, cookie, indent, true),
                    None => ptr::null_mut(),
                };
                if next_cmdline.is_null() {
                    // An aborted command line does not wait for a return.
                    // The null that ends a sourced file or a function is not
                    // an abort and does not reach here with 'KeyTyped' set.
                    if KeyTyped.get() && flags & DOCMD_REPEAT as c_int == 0 {
                        need_wait_return.set(false);
                    }
                    retval = FAIL;
                    break;
                }
                used_getline = true;

                // Every cmdline_block event but the first goes out
                // immediately: holding them until the commands have run
                // would interleave them wrongly with a nested command line.
                if ui_has(kUICmdline)
                    && count > 0
                    && getline_equal(fgetline, cookie, Some(getexline))
                {
                    ui_ext_cmdline_block_append(indent as size_t, next_cmdline);
                }

                // Keep the first typed line for `.` to repeat; forget it as
                // soon as a second one is typed.
                if flags & DOCMD_KEEPLINE as c_int != 0 {
                    xfree(repeat_cmdline.get() as *mut c_void);
                    repeat_cmdline.set(if count == 0 {
                        xstrdup(next_cmdline)
                    } else {
                        ptr::null_mut()
                    });
                }
            } else if cmdline_copy.is_null() {
                // 3. A line was given: copy it, because it is about to be
                //    modified in place.
                next_cmdline = xstrdup(next_cmdline);
            }
            cmdline_copy = next_cmdline;

            // Inside a loop — or on a line that looks like it opens one —
            // the line is stored so it can be replayed, and `do_one_cmd` is
            // handed a line getter that stores and replays too. That is what
            // lets a `:function` be defined inside a `:while`.
            let mut current_line_before = 0;
            let cmd_getline;
            let cmd_cookie;
            if cstack.cs_looplevel > 0 || has_loop_cmd(next_cmdline) {
                cmd_getline = Some(get_loop_line as _);
                cmd_cookie = &raw mut cmd_loop_cookie as *mut c_void;
                cmd_loop_cookie.lines_gap = &raw mut lines_ga;
                cmd_loop_cookie.current_line = current_line;
                cmd_loop_cookie.lc_getline = fgetline;
                cmd_loop_cookie.cookie = cookie;
                cmd_loop_cookie.repeating = (current_line < lines_ga.ga_len) as c_int;

                if current_line == lines_ga.ga_len {
                    store_loop_line(&raw mut lines_ga, next_cmdline);
                }
                current_line_before = current_line;
            } else {
                cmd_getline = fgetline;
                cmd_cookie = cookie;
            }

            did_endif.set(false);

            if count == 0 {
                // Put all the output below each other without waiting for a
                // return. Not for commands from a script, and not for a
                // recursive call (`:e +command file`).
                if flags & DOCMD_NOWAIT as c_int == 0 && RECURSIVE.get() == 0 {
                    msg_didout_before_start = msg_didout.get();
                    msg_didany.set(false);
                    msg_start();
                    msg_scroll.set(1);
                    *no_wait_return.ptr() += 1;
                    *RedrawingDisabled.ptr() += 1;
                    did_inc = true;
                }
            }
            count += 1;

            if p_verbose.get() >= 15 && !(*sourcing_entry()).es_name.is_null()
                || p_verbose.get() >= 16 as OptInt
            {
                msg_verbose_cmd(sourcing_lnum(), cmdline_copy);
            }

            // Run one `|`-separated command. `cmdline_copy` can change
            // under this call — `%` and `#` expansion reallocate it — and
            // the answer is null when nothing followed a `|`.
            *RECURSIVE.ptr() += 1;
            next_cmdline = do_one_cmd(
                &raw mut cmdline_copy,
                flags,
                &raw mut cstack,
                cmd_getline,
                cmd_cookie,
            );
            *RECURSIVE.ptr() -= 1;

            if cmd_cookie == &raw mut cmd_loop_cookie as *mut c_void {
                // Defining a function reads further lines through the loop
                // cookie, so take the line number back from it.
                current_line = cmd_loop_cookie.current_line;
            }

            if next_cmdline.is_null() {
                xfree(cmdline_copy as *mut c_void);
                cmdline_copy = ptr::null_mut();

                // Remember a typed command for the `:` register — after
                // running it, so that `:@:` works.
                if getline_equal(fgetline, cookie, Some(getexline))
                    && !new_last_cmdline.get().is_null()
                {
                    xfree(last_cmdline.get() as *mut c_void);
                    last_cmdline.set(new_last_cmdline.get());
                    new_last_cmdline.set(ptr::null_mut());
                }
            } else {
                // Move what follows the `|` to the front of the buffer, for
                // the next `do_one_cmd`.
                memmove(
                    cmdline_copy as *mut c_void,
                    next_cmdline as *const c_void,
                    strlen(next_cmdline) + 1,
                );
                next_cmdline = cmdline_copy;
            }

            // A function that an error did not abort keeps going.
            if did_emsg.get() != 0
                && !force_abort.get()
                && getline_equal(fgetline, cookie, Some(get_func_line))
                && func_has_abort(real_cookie) == 0
            {
                did_emsg.set(0);
            }

            if cstack.cs_looplevel > 0 {
                current_line += 1;

                // `:endwhile`, `:endfor` and `:continue` land here. If
                // commands were being executed, jump back to the `:while`
                // or `:for`; if they were being skipped, the loop level has
                // already been decremented.
                if cstack.cs_lflags & (CSL_HAD_CONT as c_int | CSL_HAD_ENDLOOP as c_int) != 0 {
                    cstack.cs_lflags &= !(CSL_HAD_CONT as c_int | CSL_HAD_ENDLOOP as c_int);

                    // Only a `:while` or `:for` entry has a usable
                    // `cs_line`; taking one from any other kind would make
                    // `current_line` point outside the stored lines.
                    let idx = cstack.cs_idx;
                    if did_emsg.get() == 0
                        && !got_int.get()
                        && !did_throw.get()
                        && idx >= 0
                        && cstack.cs_flags[idx as usize] & (CSF_WHILE as c_int | CSF_FOR as c_int)
                            != 0
                        && cstack.cs_line[idx as usize] >= 0
                        && cstack.cs_flags[idx as usize] & CSF_ACTIVE as c_int != 0
                    {
                        current_line = cstack.cs_line[idx as usize];
                        cstack.cs_lflags |= CSL_HAD_LOOP as c_int;
                        line_breakcheck();

                        // The next breakpoint at or after the `:while`.
                        if !breakpoint.is_null() && lines_ga.ga_len > current_line {
                            *breakpoint = dbg_find_breakpoint(
                                getline_equal(fgetline, cookie, Some(getsourceline)),
                                fname,
                                (*(lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize))
                                    .lnum
                                    - 1,
                            );
                            *dbg_tick = debug_tick.get();
                        }
                    } else if idx >= 0 {
                        // Only reachable from `:endwhile` or `:endfor`.
                        rewind_conditionals(
                            &raw mut cstack,
                            idx - 1,
                            CSF_WHILE as c_int | CSF_FOR as c_int,
                            &raw mut cstack.cs_looplevel,
                        );
                    }
                } else if cstack.cs_lflags & CSL_HAD_LOOP as c_int != 0 {
                    // A `:while` or `:for` remembers where its body starts.
                    cstack.cs_lflags &= !(CSL_HAD_LOOP as c_int);
                    cstack.cs_line[cstack.cs_idx as usize] = current_line_before;
                }
            }

            // Outside every loop, the stored lines are of no further use.
            if cstack.cs_looplevel == 0 {
                if lines_ga.ga_len > 0 {
                    (*sourcing_entry()).es_lnum = (*(lines_ga.ga_data as *mut wcmd_T)
                        .offset(lines_ga.ga_len as isize - 1))
                    .lnum;
                    clear_loop_lines(&raw mut lines_ga);
                }
                current_line = 0;
            }

            // A `:finally` makes 'did_emsg', 'got_int' and 'did_throw'
            // pending until the `:endtry`. Reset them here and mark the
            // entry active, so that the finally clause runs at all — which
            // includes the case where the `:finally` itself is what noticed
            // a missing `:endif`, `:endwhile` or `:endfor`.
            if cstack.cs_lflags & CSL_HAD_FINA as c_int != 0 {
                cstack.cs_lflags &= !(CSL_HAD_FINA as c_int);
                report_make_pending(
                    cstack.cs_pending[cstack.cs_idx as usize] as c_int
                        & (CSTP_ERROR as c_int | CSTP_INTERRUPT as c_int | CSTP_THROW as c_int),
                    if did_throw.get() {
                        current_exception.get() as *mut c_void
                    } else {
                        ptr::null_mut()
                    },
                );
                did_emsg.set(0);
                got_int.set(false);
                did_throw.set(false);
                cstack.cs_flags[cstack.cs_idx as usize] |=
                    CSF_ACTIVE as c_int | CSF_FINALLY as c_int;
            }

            // The global `trylevel` is what a *nested* `do_cmdline` reads.
            trylevel.set(initial_trylevel + cstack.cs_trylevel);

            // The outermost try conditional — across function calls and
            // sourced files — aborting cancels everything. Leaving it
            // normally puts the non-exception abort behaviour back for the
            // rest of the script.
            if trylevel.get() == 0 && did_emsg.get() == 0 && !got_int.get() && !did_throw.get() {
                force_abort.set(false);
            }

            do_intthrow(&raw mut cstack);

            // Keep going while:
            // - nothing is aborting, or a try conditional is still open and
            //   has finally clauses to run or an interrupt to catch;
            // - no error was reported against a *typed* line;
            // - and there is something left to run.
            let aborting_now =
                (got_int.get() || did_emsg.get() != 0 && force_abort.get() || did_throw.get())
                    && cstack.cs_trylevel == 0;
            // Inside try/catch an error keeps going, so that it can be dealt
            // with — unless it is a syntax error, which may make the
            // `:endtry` itself be missed.
            let typed_error = did_emsg.get() != 0
                && (cstack.cs_trylevel == 0 || did_emsg_syntax.get())
                && used_getline
                && getline_equal(fgetline, cookie, Some(getexline));
            let more_to_run =
                !next_cmdline.is_null() || cstack.cs_idx >= 0 || flags & DOCMD_REPEAT as c_int != 0;
            if aborting_now || typed_error || !more_to_run {
                break;
            }
        }

        xfree(cmdline_copy as *mut c_void);
        did_emsg_syntax.set(false);
        clear_loop_lines(&raw mut lines_ga);

        if cstack.cs_idx >= 0 {
            // A sourced file or a function that ran to its end with a
            // conditional still open.
            if !got_int.get()
                && !did_throw.get()
                && !aborting()
                && (getline_equal(fgetline, cookie, Some(getsourceline))
                    && !source_finished(fgetline, cookie)
                    || getline_equal(fgetline, cookie, Some(get_func_line))
                        && func_has_ended(real_cookie) == 0)
            {
                let flags_here = cstack.cs_flags[cstack.cs_idx as usize];
                let missing = if flags_here & CSF_TRY as c_int != 0 {
                    &raw const e_endtry as *const c_char
                } else if flags_here & CSF_WHILE as c_int != 0 {
                    &raw const e_endwhile as *const c_char
                } else if flags_here & CSF_FOR as c_int != 0 {
                    &raw const e_endfor as *const c_char
                } else {
                    &raw const e_endif as *const c_char
                };
                emsg(gettext(missing));
            }

            // Put `trylevel` back after a `:finish`, a `:return` or a
            // missing `:endtry`. A try conditional in its finally clause
            // drops anything pending; one in a catch clause finishes the
            // exception it caught. This also frees the `cs_forinfo`s.
            loop {
                let mut idx = cleanup_conditionals(&raw mut cstack, 0, true);
                if idx >= 0 {
                    // Drop a try block that is not in its finally clause.
                    idx -= 1;
                }
                rewind_conditionals(
                    &raw mut cstack,
                    idx,
                    CSF_WHILE as c_int | CSF_FOR as c_int,
                    &raw mut cstack.cs_looplevel,
                );
                if cstack.cs_idx < 0 {
                    break;
                }
            }
            trylevel.set(initial_trylevel);
        }

        // A missing `:endtry`/`:endwhile`/`:endfor`/`:endif` reported above
        // becomes an exception now, after the stack has been rewound.
        do_errthrow(
            &raw mut cstack,
            if getline_equal(fgetline, cookie, Some(get_func_line)) {
                c"endfunction".as_ptr() as *mut c_char
            } else {
                ptr::null_mut()
            },
        );

        if trylevel.get() == 0 {
            if did_throw.get() {
                // An exception thrown out of the outermost try conditional:
                // discard it, stop converting errors and interrupts to
                // exceptions, and run nothing more.
                handle_did_throw();
            } else if got_int.get() || did_emsg.get() != 0 && force_abort.get() {
                // An interrupt, or an aborting error that did not become an
                // exception. Errors stop being converted — which is also
                // what lets the interrupt message through when 'force_abort'
                // is set and 'did_emsg' is not, after an error in a finally
                // clause.
                suppress_errthrow.set(true);
            }
        }

        // This `cstack` is about to go away. An uncaught exception has to be
        // rethrown against the caller's; and a function that has just
        // returned, or a script that has just finished, may leave the
        // caller's stack with finally clauses to run. `do_one_cmd` does
        // both, once it sees these two flags.
        if did_throw.get() {
            need_rethrow.set(true);
        }
        if getline_equal(fgetline, cookie, Some(getsourceline))
            && ex_nesting_level.get() > source_level(real_cookie)
            || getline_equal(fgetline, cookie, Some(get_func_line))
                && ex_nesting_level.get() > func_level(real_cookie) + 1
        {
            if !did_throw.get() {
                check_cstack.set(true);
            }
        } else {
            if getline_equal(fgetline, cookie, Some(get_func_line)) {
                *ex_nesting_level.ptr() -= 1;
            }
            // Single-stepping out of a function drops back into the
            // debugger.
            if (getline_equal(fgetline, cookie, Some(getsourceline))
                || getline_equal(fgetline, cookie, Some(get_func_line)))
                && ex_nesting_level.get() < debug_break_level.get()
            {
                do_debug(gettext(
                    if getline_equal(fgetline, cookie, Some(getsourceline)) {
                        c"End of sourced file".as_ptr()
                    } else {
                        c"End of function".as_ptr()
                    },
                ));
            }
        }

        // After returning from the debugger, not before.
        if flags & DOCMD_EXCRESET as c_int != 0 {
            restore_dbg_stuff(&raw mut debug_saved);
        }

        msg_list.set(saved_msg_list);

        let mut elem: *mut eslist_T = cstack.cs_emsg_silent_list;
        while !elem.is_null() {
            let next = (*elem).next;
            xfree(elem as *mut c_void);
            elem = next;
        }

        // Too much output to fit on the command line: ask for a return
        // before the screen is redrawn. With `:global` this happens once,
        // after the whole command.
        if did_inc {
            *RedrawingDisabled.ptr() -= 1;
            *no_wait_return.ptr() -= 1;
            msg_scroll.set(0);

            if retval == FAIL || did_endif.get() && KeyTyped.get() && did_emsg.get() == 0 {
                // A typed `:if`/`:else` that has just finished, or an error.
                need_wait_return.set(false);
                msg_didany.set(false);
            } else if need_wait_return.get() {
                // `msg_start` above cleared 'msg_didout'; the `wait_return`
                // here must not overwrite whatever was shown before it.
                msg_didout.set(msg_didout.get() || msg_didout_before_start);
                wait_return(0);
            }
        }

        if did_block {
            ui_ext_cmdline_block_leave();
        }

        // In case `do_cmdline` was used recursively.
        did_endif.set(false);

        do_cmdline_end();
        retval
    }
}
