//! The API's view of a command line.
//!
//! `nvim_parse_cmd` reaches `parse_cmdline`, which runs every stage of
//! `do_one_cmd`'s parse and runs none of its effects; `nvim_cmd` reaches
//! `execute_cmd`, which starts from an `exarg_T` a Dict was decoded into
//! rather than from text. Between them they are the only callers that can
//! present the command machinery with values no command line could spell,
//! which is why the checks here are spelled out again rather than shared
//! with `do_one_cmd`.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::buffer::buflist_findpat;
use crate::src::nvim::charset::{skiptowhite_esc, skipwhite};
use crate::src::nvim::eval::skip_expr;
use crate::src::nvim::ex_docmd::address::{
    correct_range, find_excmd_after_range, parse_cmd_address, set_cmd_addr_type,
    set_cmd_dflall_range,
};
use crate::src::nvim::ex_docmd::filename::expand_filename;
use crate::src::nvim::ex_docmd::lookup::is_user_cmd;
use crate::src::nvim::ex_docmd::modifier::{
    apply_cmdmod, cmd_has_expr_args, parse_command_modifiers, undo_cmdmod,
};
use crate::src::nvim::ex_docmd::onecmd::{
    append_command, ex_range_without_command, fresh_exarg, shift_cmd_args,
};
use crate::src::nvim::ex_docmd::scan::{
    check_nextcmd, parse_bang, parse_count, parse_register, separate_nextcmd, skip_colon_white,
};
use crate::src::nvim::ex_docmd::source::{do_cmdline_end, do_cmdline_start};
use crate::src::nvim::ex_docmd::{
    ADDR_LINES, ADDR_NONE, CMD_SIZE, CMD_bang, CMD_bdelete, CMD_bunload, CMD_bwipeout,
    CMD_checktime, CMD_edit, CMD_file, CMD_iput, CMD_put, CMD_read, CMD_try, EX_BANG, EX_BUFNAME,
    EX_BUFUNL, EX_CMDWIN, EX_DFLALL, EX_LOCK_OK, EX_MODIFY, EX_RANGE, EX_TRLBAR, EX_WHOLEFOLD,
    EX_XFILE, FAIL, IOSIZE, NUL, OK, cmdmod, cmdnames, e_ambiguous_use_of_user_defined_command,
    e_not_an_editor_command, ex_pressedreturn,
};
use crate::src::nvim::ex_getln::{
    cmdpreview_get_bufnr, cmdpreview_get_ns, curbuf_locked, get_text_locked_msg, text_locked,
};
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::main::{
    IObuff, cmdwin_type, curbuf, curwin, e_cmdwin, e_command_too_recursive, e_modifiable, e_nobang,
    e_norange, emsg_silent, emsg_skip, global_busy,
};
use crate::src::nvim::memory::xstrlcpy;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{gettext, memset, strlen};
use crate::src::nvim::search::{restore_last_search_pattern, save_last_search_pattern};
use crate::src::nvim::types::{
    CmdParseInfo, cmdmod_T, cstack_T, exarg_T, linenr_T, pos_T, size_t, uint32_t,
};
use crate::src::nvim::usercmd::do_ucmd;

/// Parse one command line into an `exarg_T` and a `CmdParseInfo`, running
/// nothing.
///
/// Everything the parse touches that is observable — 'ex_pressedreturn',
/// the cursor (a range may move it) and the last search pattern (`:/pat/`
/// sets it) — is saved and put back, so that a parse has no effect at all.
///
/// On success the caller owns `cmdinfo->cmdmod`'s filter pattern and
/// regexp program, and must free them with `undo_cmdmod` or by running the
/// command through `execute_cmd`.
pub unsafe fn parse_cmdline(
    cmdline: *mut *mut c_char,
    eap: *mut exarg_T,
    cmdinfo: *mut CmdParseInfo,
    errormsg: *mut *const c_char,
) -> bool {
    unsafe {
        let save_ex_pressedreturn = ex_pressedreturn.get();
        let save_cursor: pos_T = (*curwin.get()).w_cursor;
        save_last_search_pattern();

        memset(cmdinfo as *mut c_void, 0, size_of::<CmdParseInfo>());
        *eap = fresh_exarg();
        let ea = &mut *eap;
        ea.cmd = *cmdline;
        ea.cmdlinep = cmdline;

        let mut retval = false;
        'end: {
            let orig_cmd = ea.cmd;
            // A modifier that failed to parse is still a modifier: keep
            // going, so that the error is reported against the command
            // rather than against the line.
            let result = parse_command_modifiers(eap, errormsg, &raw mut (*cmdinfo).cmdmod, false);
            let after_modifier = ea.cmd;
            if result == FAIL && after_modifier == orig_cmd {
                break 'end;
            }

            // The command name says what kind of address the range counts in.
            let mut p = find_excmd_after_range(eap);
            if p.is_null() {
                *errormsg = gettext(
                    (e_ambiguous_use_of_user_defined_command.ptr() as *const _) as *const c_char,
                );
                break 'end;
            }

            set_cmd_addr_type(eap, p);
            if parse_cmd_address(eap, errormsg, true) == FAIL {
                break 'end;
            }

            ea.cmd = skip_colon_white(ea.cmd, true);
            if *ea.cmd as c_int == '"' as c_int {
                break 'end;
            }
            // Nothing at all: no command, no range, no modifier.
            if *ea.cmd as c_int == NUL && ea.addr_count == 0 && after_modifier == *cmdline {
                break 'end;
            }

            // A range on its own (`:1`) or a modifier on its own
            // (`:aboveleft`) is a legal thing to parse.
            if *ea.cmd as c_int == NUL && ea.cmdidx as c_int == CMD_SIZE as c_int {
                ea.arg = ea.cmd;
                if ea.addr_count > 0 {
                    ea.argt = EX_RANGE as uint32_t;
                } else {
                    ea.argt = 0;
                    ea.addr_type = ADDR_NONE;
                }
                retval = true;
                break 'end;
            }

            if ea.cmdidx as c_int == CMD_SIZE as c_int {
                xstrlcpy(
                    IObuff.ptr() as *mut c_char,
                    gettext((e_not_an_editor_command.ptr() as *const _) as *const c_char),
                    IOSIZE as size_t,
                );
                // The modifiers parsed, so the error is in what follows them.
                let cmdname = if after_modifier.is_null() {
                    *cmdline
                } else {
                    after_modifier
                };
                append_command(cmdname);
                *errormsg = IObuff.ptr() as *mut c_char;
                break 'end;
            }

            ea.forceit = parse_bang(eap, &raw mut p) as c_int;
            if !is_user_cmd(ea.cmdidx) {
                ea.argt = (*cmdnames.ptr())[ea.cmdidx as usize].cmd_argt;
            }
            // `:!` keeps the space: `:!! -l` needs it.
            ea.arg = if ea.cmdidx as c_int == CMD_bang as c_int {
                p
            } else {
                skipwhite(p)
            };
            // `:r!` is a filter, not a bang.
            if ea.cmdidx as c_int == CMD_read as c_int && ea.forceit != 0 {
                ea.forceit = 0;
            }

            if ea.argt & EX_TRLBAR as uint32_t != 0 {
                separate_nextcmd(eap);
            } else if cmd_has_expr_args(ea.cmdidx) {
                // A command whose argument is an expression has no
                // `EX_TRLBAR`, because a `|` inside the expression is not a
                // separator. Skipping expression by expression finds the one
                // that is.
                let mut arg = ea.arg;
                while *arg as c_int != NUL
                    && *arg as c_int != '|' as c_int
                    && *arg as c_int != '\n' as c_int
                {
                    let start = arg;
                    *emsg_skip.ptr() += 1;
                    skip_expr(&raw mut arg, ptr::null_mut());
                    *emsg_skip.ptr() -= 1;
                    // Nothing an expression parser recognises: step over one
                    // byte, or this loop never ends.
                    if arg == start {
                        arg = arg.add(1);
                    }
                }
                if *arg as c_int == '|' as c_int || *arg as c_int == '\n' as c_int {
                    ea.nextcmd = check_nextcmd(arg);
                    *arg = NUL as c_char;
                }
            }

            if ea.argt & EX_BANG as uint32_t == 0 && ea.forceit != 0 {
                *errormsg = gettext(&raw const e_nobang as *const c_char);
                break 'end;
            }
            if ea.argt & EX_RANGE as uint32_t == 0 && ea.addr_count > 0 {
                *errormsg = gettext(&raw const e_norange as *const c_char);
                break 'end;
            }
            if ea.argt & EX_DFLALL as uint32_t != 0 && ea.addr_count == 0 {
                set_cmd_dflall_range(eap);
            }

            parse_register(eap);
            if parse_count(eap, errormsg, false) == FAIL {
                break 'end;
            }

            if !ea.nextcmd.is_null() {
                ea.nextcmd = skip_colon_white(ea.nextcmd, true);
            }

            // Which characters the caller must escape to have them taken
            // literally when the command is handed back.
            if ea.argt & EX_XFILE as uint32_t != 0 {
                (*cmdinfo).magic.file = true;
            }
            if ea.argt & EX_TRLBAR as uint32_t != 0 {
                (*cmdinfo).magic.bar = true;
            }
            retval = true;
        }

        if !retval {
            undo_cmdmod(&raw mut (*cmdinfo).cmdmod);
        }
        ex_pressedreturn.set(save_ex_pressedreturn);
        (*curwin.get()).w_cursor = save_cursor;
        restore_last_search_pattern();
        retval
    }
}

/// Expand what is left of the argument and call the command's handler.
///
/// The last stage both `do_one_cmd` and `execute_cmd` share: everything
/// before it is validation, everything after it is error reporting.
pub(crate) unsafe fn execute_cmd0(
    retv: *mut c_int,
    eap: *mut exarg_T,
    errormsg: *mut *const c_char,
    preview: bool,
) -> c_int {
    unsafe {
        let ea = &mut *eap;
        if ea.argt & EX_XFILE as uint32_t != 0
            && expand_filename(eap, ea.cmdlinep, errormsg) == FAIL
        {
            return FAIL;
        }

        // A buffer name may stand in for a buffer number, but not alongside
        // one, and not for a user command.
        if ea.argt & EX_BUFNAME as uint32_t != 0
            && *ea.arg as c_int != NUL
            && ea.addr_count == 0
            && !is_user_cmd(ea.cmdidx)
        {
            if ea.args.is_null() {
                // `:bdelete`, `:bwipeout` and `:bunload` take several
                // space-separated names, so the first one ends at the first
                // unescaped space; every other command takes one name, so
                // only trailing space is dropped.
                let p = if ea.cmdidx as c_int == CMD_bdelete as c_int
                    || ea.cmdidx as c_int == CMD_bwipeout as c_int
                    || ea.cmdidx as c_int == CMD_bunload as c_int
                {
                    skiptowhite_esc(ea.arg)
                } else {
                    let mut p = ea.arg.add(strlen(ea.arg) as usize);
                    while p > ea.arg && ascii_iswhite(*p.sub(1) as c_int) {
                        p = p.sub(1);
                    }
                    p
                };
                ea.line2 = buflist_findpat(
                    ea.arg,
                    p,
                    ea.argt & EX_BUFUNL as uint32_t != 0,
                    false,
                    false,
                ) as linenr_T;
                ea.addr_count = 1;
                ea.arg = skipwhite(p);
            } else {
                // The API gave the argument positions, so the first argument
                // is the name with no scanning at all.
                ea.line2 = buflist_findpat(
                    *ea.args,
                    (*ea.args).add(*ea.arglens as usize),
                    ea.argt & EX_BUFUNL as uint32_t != 0,
                    false,
                    false,
                ) as linenr_T;
                ea.addr_count = 1;
                shift_cmd_args(eap);
            }
            if ea.line2 < 0 {
                return FAIL;
            }
        }

        // `:try` saves 'emsg_silent' itself, so `:silent! try` must not
        // still be silencing by the time the body runs.
        if ea.cmdidx as c_int == CMD_try as c_int && (*cmdmod.ptr()).cmod_did_esilent > 0 {
            *emsg_silent.ptr() -= (*cmdmod.ptr()).cmod_did_esilent;
            emsg_silent.set(emsg_silent.get().max(0));
            (*cmdmod.ptr()).cmod_did_esilent = 0;
        }

        if is_user_cmd(ea.cmdidx) {
            *retv = do_ucmd(eap, preview);
        } else {
            ea.errmsg = ptr::null_mut();
            if preview {
                *retv = (*cmdnames.ptr())[ea.cmdidx as usize]
                    .cmd_preview_func
                    .expect("a command with EX_PREVIEW has a preview callback")(
                    eap,
                    cmdpreview_get_ns(),
                    cmdpreview_get_bufnr(),
                );
            } else {
                (*cmdnames.ptr())[ea.cmdidx as usize]
                    .cmd_func
                    .expect("every command in the table has a handler")(eap);
            }
            if !ea.errmsg.is_null() {
                *errormsg = ea.errmsg;
            }
        }

        OK
    }
}

/// Run an `exarg_T` the API built, without re-parsing anything.
///
/// The argument checks `do_one_cmd` makes while parsing are *not* repeated
/// here — the caller is trusted to have produced a sane `exarg_T` — but the
/// checks about where a command may run (a locked buffer, the command-line
/// window, a non-'modifiable' buffer) are, because they are about the
/// editor's state rather than about the text.
pub unsafe fn execute_cmd(eap: *mut exarg_T, cmdinfo: *mut CmdParseInfo, preview: bool) -> c_int {
    unsafe {
        let ea = &mut *eap;
        let mut retv: c_int = 0;
        if do_cmdline_start() == FAIL {
            emsg(gettext(&raw const e_command_too_recursive as *const c_char));
            return retv;
        }

        let mut errormsg: *const c_char = ptr::null();
        let save_cmdmod: cmdmod_T = cmdmod.get();
        cmdmod.set((*cmdinfo).cmdmod);
        apply_cmdmod(cmdmod.ptr());

        'end: {
            // `:put` is allowed in a terminal buffer, which is not
            // 'modifiable'.
            if (*curbuf.get()).b_p_ma == 0
                && ea.argt & EX_MODIFY as uint32_t != 0
                && !(!(*curbuf.get()).terminal.is_null()
                    && (ea.cmdidx as c_int == CMD_put as c_int
                        || ea.cmdidx as c_int == CMD_iput as c_int))
            {
                errormsg = gettext(&raw const e_modifiable as *const c_char);
                break 'end;
            }
            if !is_user_cmd(ea.cmdidx) {
                if cmdwin_type.get() != 0 && ea.argt & EX_CMDWIN as uint32_t == 0 {
                    errormsg = gettext(&raw const e_cmdwin as *const c_char);
                    break 'end;
                }
                if text_locked() && ea.argt & EX_LOCK_OK as uint32_t == 0 {
                    errormsg = gettext(get_text_locked_msg());
                    break 'end;
                }
            }
            // `curbuf->b_ro_locked` forbids editing another buffer.
            // `:checktime` is postponed, `:edit` is checked later, and
            // `:file` with no argument only reports.
            if ea.argt & EX_CMDWIN as uint32_t == 0
                && ea.cmdidx as c_int != CMD_checktime as c_int
                && ea.cmdidx as c_int != CMD_edit as c_int
                && !(ea.cmdidx as c_int == CMD_file as c_int && *ea.arg as c_int == NUL)
                && !is_user_cmd(ea.cmdidx)
                && curbuf_locked()
            {
                break 'end;
            }

            correct_range(eap);
            if ea.cmdidx as c_int == CMD_SIZE as c_int && ea.addr_count > 0 {
                errormsg = ex_range_without_command(eap);
                break 'end;
            }

            // Put the first line at the start of a closed fold and the last
            // line at its end.
            if (ea.argt & EX_WHOLEFOLD as uint32_t != 0 || ea.addr_count >= 2)
                && global_busy.get() == 0
                && ea.addr_type as c_uint == ADDR_LINES as c_uint
            {
                hasFolding(curwin.get(), ea.line1, &raw mut ea.line1, ptr::null_mut());
                hasFolding(curwin.get(), ea.line2, ptr::null_mut(), &raw mut ea.line2);
            }

            if parse_count(eap, &raw mut errormsg, true) == FAIL {
                break 'end;
            }

            // A conditional stack of its own: `:try` and friends reached
            // this way are not nested inside the caller's.
            let mut cstack: cstack_T = core::mem::zeroed();
            cstack.cs_idx = -1;
            ea.cstack = &raw mut cstack;

            execute_cmd0(&raw mut retv, eap, &raw mut errormsg, preview);
        }

        if !errormsg.is_null() && *errormsg as c_int != NUL {
            emsg(errormsg);
        }
        undo_cmdmod(cmdmod.ptr());
        cmdmod.set(save_cmdmod);
        do_cmdline_end();
        retv
    }
}
