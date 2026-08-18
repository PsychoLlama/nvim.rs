//! `do_one_cmd` — parse and run one command from a command line.
//!
//! The order here is the order the C's numbered comments describe, and it
//! is load-bearing: the command *name* has to be found before the range can
//! be parsed, because the name is what says which kind of address the range
//! counts in. So the line is walked twice — `find_excmd_after_range` skips
//! a range it does not yet understand to reach the name, then
//! `parse_cmd_address` goes back and reads the range properly.
//!
//! Every exit runs the same epilogue (the C's `doend:` label), which
//! reports the error, rethrows it as an exception if something is catching,
//! and unwinds the command modifiers. That is what the `'doend` block is.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use crate::autocmd::{EVENT_CMDUNDEFINED, apply_autocmds, getnextac, has_event};
use crate::charset::skipwhite;
use crate::debugger::dbg_check_breakpoint;
use crate::edit::beginline;
use crate::eval::userfunc::{current_func_returned, do_return, get_func_line};
use crate::ex_docmd::address::{
    correct_range, find_excmd_after_range, invalid_range, parse_cmd_address, set_cmd_addr_type,
    set_cmd_dflall_range,
};
use crate::ex_docmd::api::execute_cmd0;
use crate::ex_docmd::argopt::{getargcmd, getargopt};
use crate::ex_docmd::edit::ex_print;
use crate::ex_docmd::filename::replace_makeprg;
use crate::ex_docmd::lookup::{find_ex_command, is_user_cmd};
use crate::ex_docmd::modifier::{apply_cmdmod, parse_command_modifiers, undo_cmdmod};
use crate::ex_docmd::scan::{
    check_nextcmd, get_flags, parse_bang, parse_count, parse_register, separate_nextcmd,
    skip_colon_white,
};
use crate::ex_docmd::source::{ex_errmsg, getline_cookie, getline_equal};
use crate::ex_docmd::verify::verify_command;
use crate::ex_docmd::{
    ADDR_LINES, ADDR_OTHER, BL_FIX, BL_SOL, CSF_ACTIVE, CSF_CAUGHT, CSF_THROWN, CSF_TRUE,
    DOCMD_VERBOSE, EX_ARGOPT, EX_BANG, EX_CMDARG, EX_CMDWIN, EX_COUNT, EX_DFLALL, EX_EXTRA,
    EX_FLAGS, EX_LOCK_OK, EX_MODIFY, EX_NEEDARG, EX_RANGE, EX_SBOXOK, EX_TRLBAR, EX_WHOLEFOLD,
    FAIL, IOSIZE, NUL, PROF_YES, cmdnames, e_ambiguous_use_of_user_defined_command,
    e_not_an_editor_command, ex_func_T, exmode_plus, quitmore,
};
use crate::ex_eval::{aborting, do_errthrow, do_intthrow, do_throw};
use crate::ex_getln::{curbuf_locked, get_text_locked_msg, script_get, text_locked};
use crate::fold::hasFolding;
use crate::input::ask_yesno;
use crate::main::{
    IObuff, check_cstack, cmdmod, cmdwin_type, curbuf, curwin, did_emsg, did_emsg_syntax,
    did_throw, do_profiling, e_argreq, e_cmdwin, e_invarg, e_invrange, e_modifiable, e_nobang,
    e_norange, e_sandbox, e_trailing_arg, ex_nesting_level, exiting, exmode_active, global_busy,
    got_int, msg_silent, need_rethrow, pending_end_reg_executing, reg_executing, sandbox,
};
use crate::mbyte::{mb_copy_char, utf_head_off, utfc_ptr2len};
use crate::memory::{xcalloc, xfree, xmemdupz, xstrlcat, xstrlcpy};
use crate::message::emsg;
use crate::os::cshim::{gettext, memmove};
use crate::profile::{func_line_exec, script_line_exec};
use crate::runtime::{do_finish, getsourceline, source_finished};
use crate::types::{
    CMD_SIZE, CMD_aboveleft, CMD_and, CMD_bang, CMD_belowright, CMD_botright, CMD_browse, CMD_call,
    CMD_catch, CMD_checktime, CMD_confirm, CMD_const, CMD_delfunction, CMD_djump, CMD_dlist,
    CMD_dsearch, CMD_dsplit, CMD_echo, CMD_echoerr, CMD_echomsg, CMD_echon, CMD_edit, CMD_else,
    CMD_elseif, CMD_endfor, CMD_endif, CMD_endtry, CMD_endwhile, CMD_eval, CMD_execute, CMD_file,
    CMD_filter, CMD_finally, CMD_for, CMD_function, CMD_global, CMD_help, CMD_hide, CMD_horizontal,
    CMD_if, CMD_ijump, CMD_ilist, CMD_index, CMD_iput, CMD_isearch, CMD_isplit, CMD_keepalt,
    CMD_keepjumps, CMD_keepmarks, CMD_keeppatterns, CMD_leftabove, CMD_let, CMD_lockmarks,
    CMD_lockvar, CMD_lshift, CMD_lua, CMD_match, CMD_mzscheme, CMD_noautocmd, CMD_noswapfile,
    CMD_perl, CMD_print, CMD_psearch, CMD_put, CMD_py3, CMD_python, CMD_python3, CMD_pythonx,
    CMD_pyx, CMD_read, CMD_return, CMD_rightbelow, CMD_rshift, CMD_ruby, CMD_silent, CMD_smagic,
    CMD_snomagic, CMD_substitute, CMD_syntax, CMD_tab, CMD_tcl, CMD_terminal, CMD_throw, CMD_tilde,
    CMD_topleft, CMD_try, CMD_unlet, CMD_unlockvar, CMD_update, CMD_verbose, CMD_vertical,
    CMD_vglobal, CMD_while, CMD_wincmd, CMD_write, LineGetter, cmdidx_T, cmdmod_T, cstack_T,
    exarg_T, size_t, uint8_t, uint32_t,
};
use ::libc::{strcpy, strlen};

/// A zeroed `exarg_T` with the empty range the parsers start from.
///
/// `CMD_append` and `ADDR_LINES` are both zero, so the only fields the C's
/// `(exarg_T){ .line1 = 1, .line2 = 1 }` sets to anything else are the two
/// line numbers.
pub(crate) fn fresh_exarg() -> exarg_T {
    // SAFETY: `exarg_T` is a `repr(C)` aggregate of scalars, pointers and
    // `Option<fn>`; all-zero is a valid value of every one of them.
    let mut ea: exarg_T = unsafe { core::mem::zeroed() };
    ea.line1 = 1;
    ea.line2 = 1;
    ea
}

/// Is `func` this exact Ex-command handler?
///
/// Ex-command callbacks are identified by address, as the C code did; the
/// comparison is spelled out so the intent survives the
/// `unpredictable_function_pointer_comparisons` lint.
pub(crate) fn ex_func_is(func: ex_func_T, f: unsafe fn(*mut exarg_T)) -> bool {
    func.is_some_and(|g| ptr::fn_addr_eq(g, f))
}

/// Is this a command the build knows the name of but cannot run?
pub unsafe fn is_cmd_ni(cmdidx: cmdidx_T) -> bool {
    unsafe {
        !is_user_cmd(cmdidx)
            && (ex_func_is((*cmdnames.ptr())[cmdidx as usize].cmd_func, ex_ni)
                || ex_func_is((*cmdnames.ptr())[cmdidx as usize].cmd_func, ex_script_ni))
    }
}

/// Drop the first of an API-supplied argument list, and point `eap->arg` at
/// what is left.
///
/// With no arguments left, `eap->arg` answers the end of the *old* first
/// argument rather than null — a command that reads `eap->arg` as a string
/// then sees an empty one.
pub(crate) unsafe fn shift_cmd_args(eap: *mut exarg_T) {
    unsafe {
        let ea = &mut *eap;
        debug_assert!(!ea.args.is_null() && ea.argc > 0);
        let oldargs = ea.args;
        let oldarglens = ea.arglens;

        ea.argc -= 1;
        ea.args = if ea.argc > 0 {
            xcalloc(ea.argc, size_of::<*mut c_char>()) as *mut *mut c_char
        } else {
            ptr::null_mut()
        };
        ea.arglens = if ea.argc > 0 {
            xcalloc(ea.argc, size_of::<size_t>()) as *mut size_t
        } else {
            ptr::null_mut()
        };
        for i in 0..ea.argc {
            *ea.args.add(i) = *oldargs.add(i + 1);
            *ea.arglens.add(i) = *oldarglens.add(i + 1);
        }
        ea.arg = if ea.argc > 0 {
            *ea.args
        } else {
            (*oldargs).add(*oldarglens)
        };

        xfree(oldargs as *mut c_void);
        xfree(oldarglens as *mut c_void);
    }
}

/// The commands that still run when `do_one_cmd` is skipping — inside an
/// inactive `:if` branch, or after an error.
///
/// Two groups: the control-flow commands, which have to run to find the end
/// of the construct at all, and the commands that consume the rest of the
/// line themselves. Upstream's rule for the second group is that a command
/// must either carry `EX_TRLBAR`, appear here, or appear in the list at
/// `:help :bar`.
#[rustfmt::skip]
const RUN_WHILE_SKIPPING: &[CMD_index] = &[
    // Commands that need evaluation.
    CMD_while, CMD_endwhile, CMD_for, CMD_endfor, CMD_if, CMD_elseif, CMD_else, CMD_endif,
    CMD_try, CMD_catch, CMD_finally, CMD_endtry, CMD_function,
    // Commands that handle '|' themselves.
    CMD_aboveleft, CMD_and, CMD_belowright, CMD_botright, CMD_browse, CMD_call, CMD_confirm,
    CMD_const, CMD_delfunction, CMD_djump, CMD_dlist, CMD_dsearch, CMD_dsplit, CMD_echo,
    CMD_echoerr, CMD_echomsg, CMD_echon, CMD_eval, CMD_execute, CMD_filter, CMD_help, CMD_hide,
    CMD_horizontal, CMD_ijump, CMD_ilist, CMD_isearch, CMD_isplit, CMD_keepalt, CMD_keepjumps,
    CMD_keepmarks, CMD_keeppatterns, CMD_leftabove, CMD_let, CMD_lockmarks, CMD_lockvar, CMD_lua,
    CMD_match, CMD_mzscheme, CMD_noautocmd, CMD_noswapfile, CMD_perl, CMD_psearch, CMD_python,
    CMD_py3, CMD_python3, CMD_pythonx, CMD_pyx, CMD_return, CMD_rightbelow, CMD_ruby, CMD_silent,
    CMD_smagic, CMD_snomagic, CMD_substitute, CMD_syntax, CMD_tab, CMD_tcl, CMD_throw, CMD_tilde,
    CMD_topleft, CMD_unlet, CMD_unlockvar, CMD_verbose, CMD_vertical, CMD_wincmd,
];

/// Should this command be passed over rather than run?
pub(crate) unsafe fn skip_cmd(eap: *const exarg_T) -> bool {
    unsafe { (*eap).skip != 0 && !RUN_WHILE_SKIPPING.contains(&((*eap).cmdidx as CMD_index)) }
}

/// Parse and execute one Ex command, and answer where the next one starts.
///
/// `fgetline`/`cookie` are the line source the command may read further
/// lines from (`:append`, a `:function` body, a sourced file); either may be
/// null. Re-entrant: a command that calls `do_cmdline` lands back here.
pub(crate) unsafe fn do_one_cmd(
    cmdlinep: *mut *mut c_char,
    flags: c_int,
    cstack: *mut cstack_T,
    fgetline: LineGetter,
    cookie: *mut c_void,
) -> *mut c_char {
    unsafe {
        let mut errormsg: *const c_char = ptr::null();
        let save_reg_executing = reg_executing.get();
        let save_pending_end_reg_executing = pending_end_reg_executing.get();
        let mut ea = fresh_exarg();
        *ex_nesting_level.ptr() += 1;

        // When the last file has not been edited `:q` has to be typed twice.
        // A `'statusline'` function call and an autocommand (QuitPre) both
        // reach here without the user having typed anything, so neither
        // spends the second `:q`.
        if quitmore_is_pending(fgetline, cookie) {
            *quitmore.ptr() -= 1;
        }

        // Modifiers are restored on the way out, for recursive calls.
        let save_cmdmod: cmdmod_T = cmdmod.get();
        let mut after_modifier: *mut c_char = ptr::null_mut();

        'doend: {
            // "#!anything" is a comment, so that a script can carry a
            // shebang line.
            if *(*cmdlinep).add(0) as c_int == '#' as c_int
                && *(*cmdlinep).add(1) as c_int == '!' as c_int
            {
                break 'doend;
            }

            ea.cmd = *cmdlinep;
            ea.cmdlinep = cmdlinep;
            ea.ea_getline = fgetline;
            ea.cookie = cookie;
            ea.cstack = cstack;

            if parse_command_modifiers(&raw mut ea, &raw mut errormsg, cmdmod.ptr(), false) == FAIL
            {
                break 'doend;
            }
            apply_cmdmod(cmdmod.ptr());
            after_modifier = ea.cmd;

            ea.skip = (did_emsg.get() != 0
                || got_int.get()
                || did_throw.get()
                || ((*cstack).cs_idx >= 0
                    && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_ACTIVE as c_int == 0))
                as c_int;

            // The command name is needed before the range can be read: it is
            // what says whether an address counts lines, windows, buffers or
            // tab pages.
            let mut p = find_excmd_after_range(&raw mut ea);
            profile_cmd(&raw mut ea, cstack, fgetline, cookie);

            if !exiting.get() {
                // May go to debug mode. If the `>quit` debug command is used
                // there, an interrupt exception is thrown and this command
                // is skipped.
                dbg_check_breakpoint(&raw mut ea);
            }
            if ea.skip == 0 && got_int.get() {
                ea.skip = 1;
                do_intthrow(cstack);
            }

            set_cmd_addr_type(&raw mut ea, p);
            if parse_cmd_address(&raw mut ea, &raw mut errormsg, false) == FAIL {
                break 'doend;
            }

            ea.cmd = skip_colon_white(ea.cmd, true);

            // A range with no command after it. Vi's behaviour, preserved:
            // `:3` jumps to line 3, `:3|…` *prints* line 3, and `:|` prints
            // the current line.
            if *ea.cmd as c_int == NUL || *ea.cmd as c_int == '"' as c_int || {
                ea.nextcmd = check_nextcmd(ea.cmd);
                !ea.nextcmd.is_null()
            } {
                if ea.skip == 0 {
                    debug_assert!(errormsg.is_null());
                    errormsg = ex_range_without_command(&raw mut ea);
                }
                break 'doend;
            }

            // An unknown command spelled like a user command, with a
            // CmdUndefined autocommand waiting to define it.
            if !p.is_null()
                && ea.cmdidx as c_int == CMD_SIZE as c_int
                && ea.skip == 0
                && (*ea.cmd as u8).is_ascii_uppercase()
                && has_event(EVENT_CMDUNDEFINED)
            {
                let mut end = ea.cmd;
                while (*end as u8).is_ascii_alphanumeric() {
                    end = end.add(1);
                }
                let cmdname = xmemdupz(ea.cmd as *const c_void, end.offset_from(ea.cmd) as size_t)
                    as *mut c_char;
                let ret =
                    apply_autocmds(EVENT_CMDUNDEFINED, cmdname, cmdname, true, ptr::null_mut());
                xfree(cmdname as *mut c_void);
                // Look again only if the autocommands did something and did
                // not fail.
                p = if ret && !aborting() {
                    find_ex_command(&raw mut ea, ptr::null_mut())
                } else {
                    ea.cmd
                };
            }

            if p.is_null() {
                if ea.skip == 0 {
                    errormsg = gettext(e_ambiguous_use_of_user_defined_command.as_ptr());
                }
                break 'doend;
            }

            if ea.cmdidx as c_int == CMD_SIZE as c_int {
                if ea.skip == 0 {
                    xstrlcpy(
                        IObuff.ptr() as *mut c_char,
                        gettext(e_not_an_editor_command.as_ptr()),
                        IOSIZE as size_t,
                    );
                    // The modifiers parsed, so the error is in what follows
                    // them.
                    let cmdname = if after_modifier.is_null() {
                        *cmdlinep
                    } else {
                        after_modifier
                    };
                    if flags & DOCMD_VERBOSE as c_int == 0 {
                        append_command(cmdname);
                    }
                    errormsg = IObuff.ptr() as *mut c_char;
                    did_emsg_syntax.set(true);
                    verify_command(cmdname);
                }
                break 'doend;
            }

            // Not implemented in this build: the argument checks below are
            // relaxed, because there is nothing to check them against.
            let ni = is_cmd_ni(ea.cmdidx);

            ea.forceit = parse_bang(&raw mut ea, &raw mut p) as c_int;

            if !is_user_cmd(ea.cmdidx) {
                ea.argt = (*cmdnames.ptr())[ea.cmdidx as usize].cmd_argt;
            }

            if ea.skip == 0 {
                if let Some(msg) = refuses_here(&ea) {
                    errormsg = msg;
                    break 'doend;
                }
                // `curbuf->b_ro_locked` forbids editing another buffer.
                // `:checktime` is postponed rather than refused, and `:edit`
                // and `:file` are checked again once their argument is known.
                if ea.argt & EX_CMDWIN as uint32_t == 0
                    && ea.cmdidx as c_int != CMD_checktime as c_int
                    && ea.cmdidx as c_int != CMD_edit as c_int
                    && ea.cmdidx as c_int != CMD_file as c_int
                    && !is_user_cmd(ea.cmdidx)
                    && curbuf_locked()
                {
                    break 'doend;
                }
                if !ni && ea.argt & EX_RANGE as uint32_t == 0 && ea.addr_count > 0 {
                    errormsg = gettext(&raw const e_norange as *const c_char);
                    break 'doend;
                }
            }

            if !ni && ea.argt & EX_BANG as uint32_t == 0 && ea.forceit != 0 {
                errormsg = gettext(&raw const e_nobang as *const c_char);
                break 'doend;
            }

            // A range that is not used is not complained about, which can
            // happen when a line count is accidentally zero.
            if ea.skip == 0 && !ni && ea.argt & EX_RANGE as uint32_t != 0 {
                // A backwards range is offered for swapping. `:global` is
                // busy running a command per line and would fail below
                // anyway, so it is not asked.
                if global_busy.get() == 0 && ea.line1 > ea.line2 {
                    if msg_silent.get() == 0 {
                        if flags & DOCMD_VERBOSE as c_int != 0 || exmode_active.get() {
                            errormsg = gettext(c"E493: Backwards range given".as_ptr());
                            break 'doend;
                        }
                        if ask_yesno(gettext(c"Backwards range given, OK to swap".as_ptr()))
                            != 'y' as c_int
                        {
                            break 'doend;
                        }
                    }
                    core::mem::swap(&mut ea.line1, &mut ea.line2);
                }
                errormsg = invalid_range(&raw mut ea);
                if !errormsg.is_null() {
                    break 'doend;
                }
            }

            // `ADDR_OTHER` counts from 1 rather than from the cursor.
            if ea.addr_type as c_uint == ADDR_OTHER as c_uint && ea.addr_count == 0 {
                ea.line2 = 1;
            }

            correct_range(&raw mut ea);

            // Put the first line at the start of a closed fold and the last
            // line at its end.
            if (ea.argt & EX_WHOLEFOLD as uint32_t != 0 || ea.addr_count >= 2)
                && global_busy.get() == 0
                && ea.addr_type as c_uint == ADDR_LINES as c_uint
            {
                hasFolding(curwin.get(), ea.line1, &raw mut ea.line1, ptr::null_mut());
                hasFolding(curwin.get(), ea.line2, ptr::null_mut(), &raw mut ea.line2);
            }

            // `:make` and `:grep` splice 'makeprg'/'grepprg' into the line
            // here, so that `%` and friends expand inside it.
            p = replace_makeprg(&raw mut ea, p, cmdlinep);
            if p.is_null() {
                break 'doend;
            }

            // `:!` keeps the space: `:!! -l` needs it.
            ea.arg = if ea.cmdidx as c_int == CMD_bang as c_int {
                p
            } else {
                skipwhite(p)
            };

            if ea.cmdidx as c_int == CMD_file as c_int && *ea.arg as c_int != NUL && curbuf_locked()
            {
                break 'doend;
            }

            // `++opt=val` first, so that `:w ++enc=utf8 !cmd` works.
            if ea.argt & EX_ARGOPT as uint32_t != 0 {
                while *ea.arg.add(0) as c_int == '+' as c_int
                    && *ea.arg.add(1) as c_int == '+' as c_int
                {
                    if getargopt(&raw mut ea) == FAIL && !ni {
                        errormsg = gettext(&raw const e_invarg as *const c_char);
                        break 'doend;
                    }
                }
            }

            if ea.cmdidx as c_int == CMD_write as c_int || ea.cmdidx as c_int == CMD_update as c_int
            {
                if *ea.arg as c_int == '>' as c_int {
                    ea.arg = ea.arg.add(1);
                    if *ea.arg as c_int != '>' as c_int {
                        errormsg = gettext(c"E494: Use w or w>>".as_ptr());
                        break 'doend;
                    }
                    ea.arg = skipwhite(ea.arg.add(1));
                    ea.append = 1;
                } else if *ea.arg as c_int == '!' as c_int
                    && ea.cmdidx as c_int == CMD_write as c_int
                {
                    // `:w !filter`
                    ea.arg = ea.arg.add(1);
                    ea.usefilter = 1;
                }
            } else if ea.cmdidx as c_int == CMD_read as c_int {
                if ea.forceit != 0 {
                    // `:r!filter`
                    ea.usefilter = 1;
                    ea.forceit = 0;
                } else if *ea.arg as c_int == '!' as c_int {
                    // `:r !filter`
                    ea.arg = ea.arg.add(1);
                    ea.usefilter = 1;
                }
            } else if ea.cmdidx as c_int == CMD_lshift as c_int
                || ea.cmdidx as c_int == CMD_rshift as c_int
            {
                // How far to shift is how many `<` or `>` were typed.
                ea.amount = 1;
                while *ea.arg as c_int == *ea.cmd as c_int {
                    ea.arg = ea.arg.add(1);
                    ea.amount += 1;
                }
                ea.arg = skipwhite(ea.arg);
            }

            // `+command`, before the next command is looked for. Not for
            // `:read !cmd` and `:write !cmd`.
            if ea.argt & EX_CMDARG as uint32_t != 0 && ea.usefilter == 0 {
                ea.do_ecmd_cmd = getargcmd(&raw mut ea.arg);
            }

            if ea.argt & EX_TRLBAR as uint32_t != 0 && ea.usefilter == 0 {
                separate_nextcmd(&raw mut ea);
            } else if ea.cmdidx as c_int == CMD_bang as c_int
                || ea.cmdidx as c_int == CMD_terminal as c_int
                || ea.cmdidx as c_int == CMD_global as c_int
                || ea.cmdidx as c_int == CMD_vglobal as c_int
                || ea.usefilter != 0
            {
                // A shell command ends at a newline instead, and one
                // backslash before that newline is removed.
                let mut s = ea.arg;
                while *s != 0 {
                    if *s as c_int == '\\' as c_int && *s.add(1) as c_int == '\n' as c_int {
                        memmove(
                            s as *mut c_void,
                            s.add(1) as *const c_void,
                            strlen(s.add(1)) + 1,
                        );
                    } else if *s as c_int == '\n' as c_int {
                        ea.nextcmd = s.add(1);
                        *s = NUL as c_char;
                        break;
                    }
                    s = s.add(1);
                }
            }

            if ea.argt & EX_DFLALL as uint32_t != 0 && ea.addr_count == 0 {
                set_cmd_dflall_range(&raw mut ea);
            }

            parse_register(&raw mut ea);
            if parse_count(&raw mut ea, &raw mut errormsg, true) == FAIL {
                break 'doend;
            }

            if ea.argt & EX_FLAGS as uint32_t != 0 {
                get_flags(&raw mut ea);
            }
            if !ni
                && ea.argt & EX_EXTRA as uint32_t == 0
                && *ea.arg as c_int != NUL
                && *ea.arg as c_int != '"' as c_int
                && (*ea.arg as c_int != '|' as c_int || ea.argt & EX_TRLBAR as uint32_t == 0)
            {
                errormsg = ex_errmsg(&raw const e_trailing_arg as *const c_char, ea.arg);
                break 'doend;
            }
            if !ni && ea.argt & EX_NEEDARG as uint32_t != 0 && *ea.arg as c_int == NUL {
                errormsg = gettext(&raw const e_argreq as *const c_char);
                break 'doend;
            }

            if skip_cmd(&raw mut ea) {
                break 'doend;
            }

            let mut retv: c_int = 0;
            if execute_cmd0(&raw mut retv, &raw mut ea, &raw mut errormsg, false) == FAIL {
                break 'doend;
            }

            // A command that called `do_cmdline` may have left a throw, a
            // `:return` or a `:finish` that the *outer* conditional stack
            // still has to see. Re-raise it here.
            if need_rethrow.get() {
                do_throw(cstack);
            } else if check_cstack.get() {
                if source_finished(fgetline, cookie) {
                    do_finish(&raw mut ea, true);
                } else if getline_equal(fgetline, cookie, Some(get_func_line))
                    && current_func_returned() != 0
                {
                    do_return(&raw mut ea, true, false, ptr::null_mut());
                }
            }
            check_cstack.set(false);
            need_rethrow.set(false);
        }

        // Can happen with a zero line number.
        if (*curwin.get()).w_cursor.lnum == 0 {
            (*curwin.get()).w_cursor.lnum = 1;
            (*curwin.get()).w_cursor.col = 0;
        }

        if !errormsg.is_null() && *errormsg as c_int != NUL && did_emsg.get() == 0 {
            if flags & DOCMD_VERBOSE as c_int != 0 {
                if errormsg != IObuff.ptr() as *const c_char {
                    xstrlcpy(IObuff.ptr() as *mut c_char, errormsg, IOSIZE as size_t);
                    errormsg = IObuff.ptr() as *mut c_char;
                }
                append_command(*ea.cmdlinep);
            }
            emsg(errormsg);
        }
        do_errthrow(
            cstack,
            if ea.cmdidx as c_int != CMD_SIZE as c_int && !is_user_cmd(ea.cmdidx) {
                (*cmdnames.ptr())[ea.cmdidx as usize].cmd_name
            } else {
                ptr::null_mut()
            },
        );

        undo_cmdmod(cmdmod.ptr());
        cmdmod.set(save_cmdmod);
        reg_executing.set(save_reg_executing);
        pending_end_reg_executing.set(save_pending_end_reg_executing);

        // A trailing bar with nothing after it is not really a next command.
        if !ea.nextcmd.is_null() && *ea.nextcmd as c_int == NUL {
            ea.nextcmd = ptr::null_mut();
        }

        *ex_nesting_level.ptr() -= 1;
        xfree(ea.cmdline_tofree as *mut c_void);

        ea.nextcmd
    }
}

/// Does the "type `:q` twice" counter belong to a command the *user* typed?
fn quitmore_is_pending(fgetline: LineGetter, cookie: *mut c_void) -> bool {
    // SAFETY: `getline_equal` only compares `fgetline` against a known line
    // getter, walking `cookie` as a `loop_cookie` chain the caller owns.
    unsafe {
        quitmore.get() != 0
            && !getline_equal(fgetline, cookie, Some(get_func_line))
            && !getline_equal(fgetline, cookie, Some(getnextac))
    }
}

/// Count this line for `:profile`, if profiling is on and the line is one
/// that will really run.
///
/// The `skip` this recomputes is not `eap->skip`: a `:catch` that is about
/// to be entered, an `:else` whose branch is about to be taken and a
/// `:finally` all execute even though the surrounding construct is
/// inactive, and each is worth a profile sample.
pub(crate) unsafe fn profile_cmd(
    eap: *const exarg_T,
    cstack: *mut cstack_T,
    fgetline: LineGetter,
    cookie: *mut c_void,
) {
    unsafe {
        let cs = &*cstack;
        if do_profiling.get() != PROF_YES
            || !((*eap).skip == 0
                || cs.cs_idx == 0
                || (cs.cs_idx > 0
                    && cs.cs_flags[cs.cs_idx as usize - 1] & CSF_ACTIVE as c_int != 0))
        {
            return;
        }
        let mut skip = did_emsg.get() != 0 || got_int.get() || did_throw.get();
        let idx = cs.cs_idx;
        match (*eap).cmdidx as c_int {
            c if c == CMD_catch as c_int => {
                skip = !skip
                    && !(idx >= 0
                        && cs.cs_flags[idx as usize] & CSF_THROWN as c_int != 0
                        && cs.cs_flags[idx as usize] & CSF_CAUGHT as c_int == 0);
            }
            c if c == CMD_else as c_int || c == CMD_elseif as c_int => {
                skip = skip
                    || !(idx >= 0
                        && cs.cs_flags[idx as usize] & (CSF_ACTIVE as c_int | CSF_TRUE as c_int)
                            == 0);
            }
            c if c == CMD_finally as c_int => skip = false,
            c if c != CMD_endif as c_int
                && c != CMD_endfor as c_int
                && c != CMD_endtry as c_int
                && c != CMD_endwhile as c_int =>
            {
                skip = (*eap).skip != 0;
            }
            _ => {}
        }
        if skip {
            return;
        }
        if getline_equal(fgetline, cookie, Some(get_func_line)) {
            func_line_exec(getline_cookie(fgetline, cookie));
        } else if getline_equal(fgetline, cookie, Some(getsourceline)) {
            script_line_exec();
        }
    }
}

/// The three "this command is not allowed here" checks that share an exit.
///
/// Answers the message to report, or `None` when the command may run.
unsafe fn refuses_here(ea: &exarg_T) -> Option<*const c_char> {
    unsafe {
        if sandbox.get() != 0 && ea.argt & EX_SBOXOK as uint32_t == 0 {
            return Some(gettext(&raw const e_sandbox as *const c_char));
        }
        // `:put` is allowed in a terminal buffer, which is not 'modifiable'.
        if (*curbuf.get()).b_p_ma == 0
            && ea.argt & EX_MODIFY as uint32_t != 0
            && !(!(*curbuf.get()).terminal.is_null()
                && (ea.cmdidx as c_int == CMD_put as c_int
                    || ea.cmdidx as c_int == CMD_iput as c_int))
        {
            return Some(gettext(&raw const e_modifiable as *const c_char));
        }
        if !is_user_cmd(ea.cmdidx) {
            if cmdwin_type.get() != 0 && ea.argt & EX_CMDWIN as uint32_t == 0 {
                return Some(gettext(&raw const e_cmdwin as *const c_char));
            }
            if text_locked() && ea.argt & EX_LOCK_OK as uint32_t == 0 {
                return Some(gettext(get_text_locked_msg()));
            }
        }
        None
    }
}

/// A range with no command after it: print the lines, or move the cursor to
/// the last of them.
///
/// Which of the two it is depends on how the line ended — a `|` after the
/// range, or Ex mode, means print. `exmode_plus + 1` is the empty string Ex
/// mode substitutes for a bare `+`; it is recognised by *address*, not by
/// content.
pub(crate) unsafe fn ex_range_without_command(eap: *mut exarg_T) -> *mut c_char {
    unsafe {
        let ea = &mut *eap;
        let mut errormsg: *mut c_char = ptr::null_mut();
        if *ea.cmd as c_int == '|' as c_int
            || (exmode_active.get() && ea.cmd != (exmode_plus.ptr() as *mut c_char).add(1))
        {
            ea.cmdidx = CMD_print;
            ea.argt = (EX_RANGE | EX_COUNT | EX_TRLBAR) as uint32_t;
            errormsg = invalid_range(eap);
            if errormsg.is_null() {
                correct_range(eap);
                ex_print(eap);
            }
        } else if ea.addr_count != 0 {
            ea.line2 = ea.line2.min((*curbuf.get()).b_ml.ml_line_count);
            if ea.line2 < 0 {
                errormsg = gettext(&raw const e_invrange as *const c_char);
            } else {
                // Line 0 is not a position; the cursor goes to line 1.
                (*curwin.get()).w_cursor.lnum = if ea.line2 == 0 { 1 } else { ea.line2 };
                beginline(BL_SOL as c_int | BL_FIX as c_int);
            }
        }
        errormsg
    }
}

/// Append `cmd` to the error message already in `IObuff`.
///
/// Truncates to fit, and spells U+00A0 as `<a0>` — it is white space that
/// would otherwise be invisible in the report, and it is a common paste
/// accident.
pub(crate) unsafe fn append_command(cmd: *const c_char) {
    unsafe {
        let iobuff = IObuff.ptr() as *mut c_char;
        let len = strlen(iobuff);
        if len > (IOSIZE - 100) as size_t {
            let mut d = iobuff.add(IOSIZE as usize - 100);
            d = d.sub(utf_head_off(iobuff, d) as usize);
            strcpy(d, c"...".as_ptr() as *mut c_char);
        }
        xstrlcat(iobuff, c": ".as_ptr(), IOSIZE as size_t);

        let mut s = cmd;
        let mut d = iobuff.add(strlen(iobuff) as usize);
        while *s as c_int != NUL && d.offset_from(iobuff) + 5 < IOSIZE as isize {
            if *s.add(0) as uint8_t == 0xc2 && *s.add(1) as uint8_t == 0xa0 {
                s = s.add(2);
                strcpy(d, c"<a0>".as_ptr() as *mut c_char);
                d = d.add(4);
            } else {
                if d.offset_from(iobuff) + utfc_ptr2len(s) as isize + 1 >= IOSIZE as isize {
                    break;
                }
                mb_copy_char(&raw mut s, &raw mut d);
            }
        }
        *d = NUL as c_char;
    }
}

/// The handler every command this build does not implement runs.
///
/// Keeps `extern "C"`: it is a `cmd_func` in the command table, and
/// `is_cmd_ni` recognises a command by comparing against its address.
pub unsafe fn ex_ni(eap: *mut exarg_T) {
    unsafe {
        if (*eap).skip == 0 {
            (*eap).errmsg = gettext(c"E319: The command is not available in this version".as_ptr());
        }
    }
}

/// The same, for a command whose argument may be a here-document
/// (`:perl <<EOF`) — the body has to be consumed even when the command
/// cannot run, or its lines would be read as commands.
pub(crate) unsafe fn ex_script_ni(eap: *mut exarg_T) {
    unsafe {
        if (*eap).skip == 0 {
            ex_ni(eap);
        } else {
            let mut len: size_t = 0;
            xfree(script_get(eap, &raw mut len) as *mut c_void);
        }
    }
}
