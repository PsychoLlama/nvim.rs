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

use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

use crate::ascii::ascii_iswhite;

use crate::charset::skiptowhite_esc;

use crate::eval::skip_expr;
use crate::ex_docmd::address::{
    correct_range, find_excmd_after_range, parse_cmd_address, set_cmd_addr_type,
    set_cmd_dflall_range,
};

use crate::ex_docmd::filename::expand_filename;
use crate::ex_docmd::lookup::is_user_cmd;
use crate::ex_docmd::modifier::{
    CmdModScope, cmd_has_expr_args, parse_command_modifiers, undo_cmdmod,
};
use crate::ex_docmd::onecmd::{
    append_command, ex_range_without_command, fresh_exarg, shift_cmd_args,
};
use crate::ex_docmd::scan::{
    check_nextcmd, parse_bang, parse_count, parse_register, separate_nextcmd,
};

use crate::ex_docmd::source::{do_cmdline_end, do_cmdline_start};
use crate::ex_docmd::{
    cmdnames, e_ambiguous_use_of_user_defined_command, e_not_an_editor_command, ex_pressedreturn,
};
use crate::ex_getln::{
    cmdpreview_get_bufnr, cmdpreview_get_ns, curbuf_locked, get_text_locked_msg, text_locked,
};
use crate::fold::has_folding;
use crate::guard::Suppress;
use crate::main::{
    cmdmod, cmdwin_type, e_cmdwin, e_command_too_recursive, e_modifiable, e_nobang, e_norange,
    emsg_silent, global_busy,
};

use crate::os::cshim::gettext;
use crate::search::{restore_last_search_pattern, save_last_search_pattern};
use crate::types::{
    CMD_SIZE, CMD_bang, CMD_bdelete, CMD_bunload, CMD_bwipeout, CMD_checktime, CMD_edit, CMD_file,
    CMD_iput, CMD_put, CMD_read, CMD_try, CmdAddr, CmdParseInfo, ExArgt, FAIL, NUL, OK, cstack_T,
    exarg_T, linenr_T, pos_T,
};
use crate::usercmd::do_ucmd;
use crate::winlayer::{Buf, Ea, Win};
use ::libc::{memset, strlen};

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
    errormsg: &mut Option<CString>,
) -> bool {
    let save_ex_pressedreturn = ex_pressedreturn.get();
    let save_cursor: pos_T = cur_win().w_cursor;
    save_last_search_pattern();

    unsafe { memset(cmdinfo as *mut c_void, 0, size_of::<CmdParseInfo>()) };
    unsafe { *eap = fresh_exarg() };
    let mut ea = unsafe { Ea::new(eap) };
    ea.cmd = unsafe { *cmdline };
    ea.cmdlinep = cmdline;

    let mut retval = false;
    'end: {
        let orig_cmd = ea.cmd;
        // A modifier that failed to parse is still a modifier: keep
        // going, so that the error is reported against the command
        // rather than against the line.
        let result =
            unsafe { parse_command_modifiers(eap, errormsg, &mut (*cmdinfo).cmdmod, false) };
        let after_modifier = ea.cmd;
        if result == FAIL && after_modifier == orig_cmd {
            break 'end;
        }

        // The command name says what kind of address the range counts in.
        let mut p = find_excmd_after_range(ea);
        if p.is_null() {
            *errormsg = Some(ex_msg(e_ambiguous_use_of_user_defined_command.as_ptr()));
            break 'end;
        }

        unsafe { set_cmd_addr_type(eap, p) };
        if unsafe { parse_cmd_address(eap, errormsg, true) } == FAIL {
            break 'end;
        }

        ea.cmd = skip_colon_white(ea.cmd, true);
        if byte(ea.cmd) == '"' as c_int {
            break 'end;
        }
        // Nothing at all: no command, no range, no modifier.
        if byte(ea.cmd) == NUL && ea.addr_count == 0 && after_modifier == unsafe { *cmdline } {
            break 'end;
        }

        // A range on its own (`:1`) or a modifier on its own
        // (`:aboveleft`) is a legal thing to parse.
        if byte(ea.cmd) == NUL && ea.cmdidx as c_int == CMD_SIZE as c_int {
            ea.arg = ea.cmd;
            if ea.addr_count > 0 {
                ea.argt = ExArgt::RANGE;
            } else {
                ea.argt = ExArgt::NONE;
                ea.addr_type = CmdAddr::NoRange;
            }
            retval = true;
            break 'end;
        }

        if ea.cmdidx as c_int == CMD_SIZE as c_int {
            // The modifiers parsed, so the error is in what follows them.
            let cmdname = if after_modifier.is_null() {
                unsafe { *cmdline }
            } else {
                after_modifier
            };
            let msg = ex_msg(e_not_an_editor_command.as_ptr());
            *errormsg = Some(unsafe { append_command(&msg, cmdname) });
            break 'end;
        }

        ea.forceit = unsafe { parse_bang(ea, &raw mut p) } as c_int;
        if !is_user_cmd(ea.cmdidx) {
            ea.argt = cmdnames[ea.cmdidx as usize].cmd_argt;
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

        if ea.argt.has(ExArgt::TRLBAR) {
            unsafe { separate_nextcmd(eap) };
        } else if cmd_has_expr_args(ea.cmdidx) {
            // A command whose argument is an expression has no
            // `ExArgt::TRLBAR`, because a `|` inside the expression is not a
            // separator. Skipping expression by expression finds the one
            // that is.
            let mut arg = ea.arg;
            while byte(arg) != NUL && byte(arg) != '|' as c_int && byte(arg) != '\n' as c_int {
                let start = arg;
                let skipping = Suppress::emsg_skip();
                unsafe { skip_expr(&raw mut arg, ptr::null_mut()) };
                drop(skipping);
                // Nothing an expression parser recognises: step over one
                // byte, or this loop never ends.
                if arg == start {
                    arg = unsafe { arg.add(1) };
                }
            }
            if byte(arg) == '|' as c_int || byte(arg) == '\n' as c_int {
                ea.nextcmd = unsafe { check_nextcmd(arg) };
                unsafe { *arg = NUL as c_char };
            }
        }

        if !ea.argt.has(ExArgt::BANG) && ea.forceit != 0 {
            *errormsg = Some(ex_msg(e_nobang.as_ptr()));
            break 'end;
        }
        if !ea.argt.has(ExArgt::RANGE) && ea.addr_count > 0 {
            *errormsg = Some(ex_msg(e_norange.as_ptr()));
            break 'end;
        }
        if ea.argt.has(ExArgt::DFLALL) && ea.addr_count == 0 {
            unsafe { set_cmd_dflall_range(eap) };
        }

        unsafe { parse_register(eap) };
        if unsafe { parse_count(eap, errormsg, false) } == FAIL {
            break 'end;
        }

        if !ea.nextcmd.is_null() {
            ea.nextcmd = skip_colon_white(ea.nextcmd, true);
        }

        // Which characters the caller must escape to have them taken
        // literally when the command is handed back.
        if ea.argt.has(ExArgt::XFILE) {
            unsafe { (*cmdinfo).magic.file = true };
        }
        if ea.argt.has(ExArgt::TRLBAR) {
            unsafe { (*cmdinfo).magic.bar = true };
        }
        retval = true;
    }

    if !retval {
        unsafe { undo_cmdmod(&mut (*cmdinfo).cmdmod) };
    }
    ex_pressedreturn.set(save_ex_pressedreturn);
    cur_win().w_cursor = save_cursor;
    restore_last_search_pattern();
    retval
}

/// Expand what is left of the argument and call the command's handler.
///
/// The last stage both `do_one_cmd` and `execute_cmd` share: everything
/// before it is validation, everything after it is error reporting.
pub(crate) unsafe fn execute_cmd0(
    retv: *mut c_int,
    eap: *mut exarg_T,
    errormsg: &mut Option<CString>,
    preview: bool,
) -> c_int {
    let mut ea = unsafe { Ea::new(eap) };
    if ea.argt.has(ExArgt::XFILE) && unsafe { expand_filename(eap, ea.cmdlinep, errormsg) } == FAIL
    {
        return FAIL;
    }

    // A buffer name may stand in for a buffer number, but not alongside
    // one, and not for a user command.
    if ea.argt.has(ExArgt::BUFNAME)
        && byte(ea.arg) != NUL
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
                unsafe { skiptowhite_esc(ea.arg) }
            } else {
                let mut p = unsafe { ea.arg.add(strlen(ea.arg) as usize) };
                while p > ea.arg && ascii_iswhite(byte(unsafe { p.sub(1) })) {
                    p = unsafe { p.sub(1) };
                }
                p
            };
            ea.line2 =
                buflist_findpat(ea.arg, p, ea.argt.has(ExArgt::BUFUNL), false, false) as linenr_T;
            ea.addr_count = 1;
            ea.arg = skipwhite(p);
        } else {
            // The API gave the argument positions, so the first argument
            // is the name with no scanning at all.
            ea.line2 = unsafe {
                buflist_findpat(
                    *ea.args,
                    (*ea.args).add(*ea.arglens),
                    ea.argt.has(ExArgt::BUFUNL),
                    false,
                    false,
                )
            } as linenr_T;
            ea.addr_count = 1;
            shift_cmd_args(ea);
        }
        if ea.line2 < 0 {
            return FAIL;
        }
    }

    // `:try` saves 'emsg_silent' itself, so `:silent! try` must not
    // still be silencing by the time the body runs.
    let did_esilent = cmdmod.with(|mods| mods.cmod_did_esilent);
    if ea.cmdidx as c_int == CMD_try as c_int && did_esilent > 0 {
        emsg_silent.set((emsg_silent.get() - did_esilent).max(0));
        cmdmod.with_mut(|mods| mods.cmod_did_esilent = 0);
    }

    if is_user_cmd(ea.cmdidx) {
        unsafe { *retv = do_ucmd(eap, preview) };
    } else {
        ea.errmsg = None;
        if preview {
            unsafe {
                *retv = cmdnames[ea.cmdidx as usize]
                    .cmd_preview_func
                    .expect("a command with ExArgt::PREVIEW has a preview callback")(
                    eap,
                    cmdpreview_get_ns(),
                    cmdpreview_get_bufnr(),
                )
            };
        } else {
            unsafe {
                cmdnames[ea.cmdidx as usize]
                    .cmd_func
                    .expect("every command in the table has a handler")(eap)
            };
        }
        if ea.errmsg.is_some() {
            *errormsg = ea.errmsg.take();
        }
    }

    OK
}

/// Run an `exarg_T` the API built, without re-parsing anything.
///
/// The argument checks `do_one_cmd` makes while parsing are *not* repeated
/// here — the caller is trusted to have produced a sane `exarg_T` — but the
/// checks about where a command may run (a locked buffer, the command-line
/// window, a non-'modifiable' buffer) are, because they are about the
/// editor's state rather than about the text.
pub unsafe fn execute_cmd(eap: *mut exarg_T, cmdinfo: *mut CmdParseInfo, preview: bool) -> c_int {
    let mut ea = unsafe { Ea::new(eap) };
    let mut retv: c_int = 0;
    if do_cmdline_start() == FAIL {
        emsg(gettext(e_command_too_recursive).as_ptr());
        return retv;
    }

    let mut errormsg: Option<CString> = None;
    // Shallow both ways: the guard owns what the set it took out
    // points at until it goes back, and the caller keeps owning
    // `cmdinfo`.
    let mods = unsafe { CmdModScope::enter((*cmdinfo).cmdmod.clone()) };

    'end: {
        // `:put` is allowed in a terminal buffer, which is not
        // 'modifiable'.
        if cur_buf().b_p_ma == 0
            && ea.argt.has(ExArgt::MODIFY)
            && !(!cur_buf().terminal.is_null()
                && (ea.cmdidx as c_int == CMD_put as c_int
                    || ea.cmdidx as c_int == CMD_iput as c_int))
        {
            errormsg = Some(ex_msg(e_modifiable.as_ptr()));
            break 'end;
        }
        if !is_user_cmd(ea.cmdidx) {
            if cmdwin_type.get() != 0 && !ea.argt.has(ExArgt::CMDWIN) {
                errormsg = Some(ex_msg(e_cmdwin.as_ptr()));
                break 'end;
            }
            if unsafe { text_locked() } && !ea.argt.has(ExArgt::LOCK_OK) {
                errormsg = Some(ex_msg(get_text_locked_msg()));
                break 'end;
            }
        }
        // `curbuf->b_ro_locked` forbids editing another buffer.
        // `:checktime` is postponed, `:edit` is checked later, and
        // `:file` with no argument only reports.
        if !ea.argt.has(ExArgt::CMDWIN)
            && ea.cmdidx as c_int != CMD_checktime as c_int
            && ea.cmdidx as c_int != CMD_edit as c_int
            && !(ea.cmdidx as c_int == CMD_file as c_int && byte(ea.arg) == NUL)
            && !is_user_cmd(ea.cmdidx)
            && unsafe { curbuf_locked() }
        {
            break 'end;
        }

        correct_range(ea);
        if ea.cmdidx as c_int == CMD_SIZE as c_int && ea.addr_count > 0 {
            errormsg = unsafe { ex_range_without_command(eap) };
            break 'end;
        }

        // Put the first line at the start of a closed fold and the last
        // line at its end.
        if (ea.argt.has(ExArgt::WHOLEFOLD) || ea.addr_count >= 2)
            && global_busy.get() == 0
            && ea.addr_type == CmdAddr::Lines
        {
            has_folding(cur_win(), ea.line1, Some(&mut ea.line1), None);
            has_folding(cur_win(), ea.line2, None, Some(&mut ea.line2));
        }

        if unsafe { parse_count(eap, &mut errormsg, true) } == FAIL {
            break 'end;
        }

        // A conditional stack of its own: `:try` and friends reached
        // this way are not nested inside the caller's.
        let mut cstack: cstack_T = unsafe { core::mem::zeroed() };
        cstack.cs_idx = -1;
        ea.cstack = &raw mut cstack;

        unsafe { execute_cmd0(&raw mut retv, eap, &mut errormsg, preview) };
    }

    if let Some(msg) = &errormsg
        && !msg.is_empty()
    {
        emsg(msg.as_ptr());
    }
    drop(mods);
    do_cmdline_end();
    retv
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

/// `buflist_findpat()` as checked code.
fn buflist_findpat(
    pattern: *const c_char,
    pattern_end: *const c_char,
    unlisted: bool,
    diffmode: bool,
    curtab_only: bool,
) -> c_int {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::buffer::buflist_findpat(pattern, pattern_end, unlisted, diffmode, curtab_only) }
}

/// `emsg()` as checked code.
fn emsg(s: *const c_char) -> bool {
    // SAFETY: a NUL-terminated message.
    unsafe { crate::message::emsg_ptr(s) }
}

/// `ex_msg()` as checked code.
fn ex_msg(msg: *const c_char) -> CString {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_docmd::ex_msg(msg) }
}

/// `skip_colon_white()` as checked code.
fn skip_colon_white(p: *const c_char, skipleadingwhite: bool) -> *mut c_char {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_docmd::scan::skip_colon_white(p, skipleadingwhite) }
}

/// `skipwhite()` as checked code.
fn skipwhite(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::charset::skipwhite(p) }
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as c_int }
}
