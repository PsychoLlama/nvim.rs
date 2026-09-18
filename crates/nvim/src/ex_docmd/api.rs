//! The API's view of a command line.
//!
//! `nvim_parse_cmd` reaches `parse_cmdline`, which runs every stage of
//! `do_one_cmd`'s parse and runs none of its effects; `nvim_cmd` reaches
//! `execute_cmd`, which starts from an `ExArg` a Dict was decoded into
//! rather than from text. Between them they are the only callers that can
//! present the command machinery with values no command line could spell,
//! which is why the checks here are spelled out again rather than shared
//! with `do_one_cmd`.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::CmdIdx;
use core::ffi::{c_char, c_int};
use core::ptr;
use std::ffi::CString;

use crate::ascii::ascii_iswhite;

use crate::charset::skiptowhite_esc;

use crate::eval::skip_expr;
use crate::ex_docmd::address::{correct_range, find_excmd_after_range, parse_cmd_address};
use crate::ex_docmd::addrtype::{set_cmd_addr_type, set_cmd_dflall_range};
use crate::ex_docmd::scan::skip_colons;

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
use crate::ex_docmd::state::{cmdmod, global_busy};
use crate::ex_docmd::{
    cmdnames, e_ambiguous_use_of_user_defined_command, e_not_an_editor_command, ex_pressedreturn,
};
use crate::ex_getln::{
    cmdpreview_get_bufnr, cmdpreview_get_ns, curbuf_locked, get_text_locked_msg, text_locked,
};
use crate::fold::has_folding;
use crate::guard::Suppress;
use crate::message::state::emsg_silent;
use crate::message::{e_cmdwin, e_command_too_recursive, e_modifiable, e_nobang, e_norange};
use crate::winlayer::graph::cmdwin_type;

use crate::os::cshim::gettext;
use crate::search::{restore_last_search_pattern, save_last_search_pattern};
use crate::types::{
    CmdAddr, CmdLine, CmdParseInfo, CondStack, ExArg, ExArgt, FAIL, Failed, LineNr, NUL, Pos,
};
use crate::usercmd::do_ucmd;
use crate::winlayer::{Buf, Win};

/// Parse one command line into an `ExArg` and a `CmdParseInfo`, running
/// nothing.
///
/// Everything the parse touches that is observable — 'ex_pressedreturn',
/// the cursor (a range may move it) and the last search pattern (`:/pat/`
/// sets it) — is saved and put back, so that a parse has no effect at all.
///
/// On success the caller owns `cmdinfo->cmdmod`'s filter pattern and
/// regexp program, and must free them with `undo_cmdmod` or by running the
/// command through `execute_cmd`.
///
/// `line` is taken over: the parse writes into it, and what it leaves in
/// `excmd` addresses it by offset.
///
/// # Safety
///
/// `excmd` must point at the command's `ExArg`, unaliased for the
/// call. `cmdinfo` must point at the caller's `CmdParseInfo`, unaliased for
/// the call.
pub unsafe fn parse_cmdline(
    line: CmdLine,
    excmd: &mut ExArg,
    cmdinfo: *mut CmdParseInfo,
    errormsg: &mut Option<CString>,
) -> bool {
    let save_ex_pressedreturn = ex_pressedreturn.get();
    let save_cursor: Pos = Win::current().w_cursor;
    save_last_search_pattern();

    let into = cmdinfo.cast::<u8>();
    unsafe { into.write_bytes(0, size_of::<CmdParseInfo>()) };
    *excmd = fresh_exarg();
    excmd.line = line;

    let mut retval = false;
    'end: {
        let orig_cmd = excmd.line.cmd;
        // A modifier that failed to parse is still a modifier: keep
        // going, so that the error is reported against the command
        // rather than against the line.
        let result =
            unsafe { parse_command_modifiers(excmd, errormsg, &mut (*cmdinfo).cmdmod, false) };
        let after_modifier = excmd.line.cmd;
        if result.is_err() && after_modifier == orig_cmd {
            break 'end;
        }

        // The command name says what kind of address the range counts in.
        let Some(mut p) = find_excmd_after_range(excmd) else {
            *errormsg = Some(ex_msg(e_ambiguous_use_of_user_defined_command.as_ptr()));
            break 'end;
        };

        set_cmd_addr_type(excmd, Some(excmd.line.byte_at(excmd.line.skip_white(p))));
        if parse_cmd_address(excmd, errormsg, true) == FAIL {
            break 'end;
        }

        excmd.line.cmd = skip_colons(&excmd.line, excmd.line.cmd, true);
        if excmd.line.byte_at(excmd.line.cmd) == b'"' {
            break 'end;
        }
        // Nothing at all: no command, no range, no modifier.
        if excmd.line.byte_at(excmd.line.cmd) == 0 && excmd.addr_count == 0 && after_modifier == 0 {
            break 'end;
        }

        // A range on its own (`:1`) or a modifier on its own
        // (`:aboveleft`) is a legal thing to parse.
        if excmd.line.byte_at(excmd.line.cmd) == 0 && excmd.cmdidx == CmdIdx::SIZE {
            excmd.line.arg = excmd.line.cmd;
            if excmd.addr_count > 0 {
                excmd.argt = ExArgt::RANGE;
            } else {
                excmd.argt = ExArgt::NONE;
                excmd.addr_type = CmdAddr::NoRange;
            }
            retval = true;
            break 'end;
        }

        if excmd.cmdidx == CmdIdx::SIZE {
            // The modifiers parsed, so the error is in what follows them.
            let msg = ex_msg(e_not_an_editor_command.as_ptr());
            *errormsg = Some(append_command(&msg, excmd.line.rest_of(after_modifier)));
            break 'end;
        }

        excmd.forceit = parse_bang(excmd, &mut p);
        if !is_user_cmd(excmd.cmdidx) {
            excmd.argt = cmdnames[excmd.cmdidx.index()].cmd_argt;
        }
        // `:!` keeps the space: `:!! -l` needs it.
        excmd.line.arg = if excmd.cmdidx == CmdIdx::bang {
            p
        } else {
            excmd.line.skip_white(p)
        };
        // `:r!` is a filter, not a bang.
        if excmd.cmdidx == CmdIdx::read && excmd.forceit {
            excmd.forceit = false;
        }

        if excmd.argt.has(ExArgt::TRLBAR) {
            separate_nextcmd(excmd);
        } else if cmd_has_expr_args(excmd.cmdidx) {
            // A command whose argument is an expression has no
            // `ExArgt::TRLBAR`, because a `|` inside the expression is not a
            // separator. Skipping expression by expression finds the one
            // that is.
            let mut arg = excmd.arg_ptr();
            while byte(arg) != NUL && byte(arg) != '|' as c_int && byte(arg) != '\n' as c_int {
                let start = arg;
                let skipping = Suppress::emsg_skip();
                let _ = unsafe { skip_expr(&raw mut arg, ptr::null_mut()) };
                drop(skipping);
                // Nothing an expression parser recognises: step over one
                // byte, or this loop never ends.
                if arg == start {
                    arg = unsafe { arg.add(1) };
                }
            }
            if byte(arg) == '|' as c_int || byte(arg) == '\n' as c_int {
                excmd.set_nextcmd_ptr(unsafe { check_nextcmd(arg) });
                unsafe { *arg = 0 };
            }
        }

        if !excmd.argt.has(ExArgt::BANG) && excmd.forceit {
            *errormsg = Some(ex_msg(e_nobang.as_ptr()));
            break 'end;
        }
        if !excmd.argt.has(ExArgt::RANGE) && excmd.addr_count > 0 {
            *errormsg = Some(ex_msg(e_norange.as_ptr()));
            break 'end;
        }
        if excmd.argt.has(ExArgt::DFLALL) && excmd.addr_count == 0 {
            set_cmd_dflall_range(excmd);
        }

        parse_register(excmd);
        if parse_count(excmd, errormsg, false).is_err() {
            break 'end;
        }

        if let Some(at) = excmd.line.next {
            excmd.line.next = Some(skip_colons(&excmd.line, at, true));
        }

        // Which characters the caller must escape to have them taken
        // literally when the command is handed back.
        if excmd.argt.has(ExArgt::XFILE) {
            unsafe { (*cmdinfo).magic.file = true };
        }
        if excmd.argt.has(ExArgt::TRLBAR) {
            unsafe { (*cmdinfo).magic.bar = true };
        }
        retval = true;
    }

    if !retval {
        unsafe { undo_cmdmod(&mut (*cmdinfo).cmdmod) };
    }
    ex_pressedreturn.set(save_ex_pressedreturn);
    Win::current().w_cursor = save_cursor;
    restore_last_search_pattern();
    retval
}

/// Expand what is left of the argument and call the command's handler.
///
/// The last stage both `do_one_cmd` and `execute_cmd` share: everything
/// before it is validation, everything after it is error reporting.
///
/// # Safety
///
/// `retv` must point at a writable `int` the caller owns. `excmd` must point
/// at the command's `ExArg`, unaliased for the call.
pub(crate) unsafe fn execute_cmd0(
    retv: *mut c_int,
    excmd: &mut ExArg,
    errormsg: &mut Option<CString>,
    preview: bool,
) -> Result<(), Failed> {
    if excmd.argt.has(ExArgt::XFILE) {
        expand_filename(excmd, errormsg)?;
    }

    // A buffer name may stand in for a buffer number, but not alongside
    // one, and not for a user command.
    if excmd.argt.has(ExArgt::BUFNAME)
        && excmd.line.byte_at(excmd.line.arg) != 0
        && excmd.addr_count == 0
        && !is_user_cmd(excmd.cmdidx)
    {
        if excmd.line.args.is_empty() {
            // `:bdelete`, `:bwipeout` and `:bunload` take several
            // space-separated names, so the first one ends at the first
            // unescaped space; every other command takes one name, so
            // only trailing space is dropped.
            let p = if excmd.cmdidx == CmdIdx::bdelete
                || excmd.cmdidx == CmdIdx::bwipeout
                || excmd.cmdidx == CmdIdx::bunload
            {
                unsafe { skiptowhite_esc(excmd.arg_ptr()) }
            } else {
                let mut at = excmd.line.end_of(excmd.line.arg);
                while at > excmd.line.arg && ascii_iswhite(c_int::from(excmd.line.byte_at(at - 1)))
                {
                    at -= 1;
                }
                excmd.line.ptr_at(at)
            };
            excmd.line2 = buflist_findpat(
                excmd.arg_ptr(),
                p,
                excmd.argt.has(ExArgt::BUFUNL),
                false,
                false,
            ) as LineNr;
            excmd.addr_count = 1;
            excmd.set_arg_ptr(skipwhite(p));
        } else {
            // The API gave the argument positions, so the first argument
            // is the name with no scanning at all.
            let (at, len) = excmd.line.args[0];
            let unlisted = excmd.argt.has(ExArgt::BUFUNL);
            let (start, end) = (excmd.line.ptr_at(at), excmd.line.ptr_at(at + len));
            // SAFETY: both name the first argument, in the command's line.
            excmd.line2 = buflist_findpat(start, end, unlisted, false, false) as LineNr;
            excmd.addr_count = 1;
            shift_cmd_args(excmd);
        }
        if excmd.line2 < 0 {
            return Err(Failed);
        }
    }

    // `:try` saves 'emsg_silent' itself, so `:silent! try` must not
    // still be silencing by the time the body runs.
    let did_esilent = cmdmod.with(|mods| mods.cmod_did_esilent);
    if excmd.cmdidx == CmdIdx::r#try && did_esilent > 0 {
        emsg_silent.set((emsg_silent.get() - did_esilent).max(0));
        cmdmod.with_mut(|mods| mods.cmod_did_esilent = 0);
    }

    if is_user_cmd(excmd.cmdidx) {
        unsafe { *retv = do_ucmd(excmd, preview) };
    } else {
        excmd.errmsg = None;
        if preview {
            unsafe {
                *retv = cmdnames[excmd.cmdidx.index()]
                    .cmd_preview_func
                    .expect("a command with ExArgt::PREVIEW has a preview callback")(
                    excmd,
                    cmdpreview_get_ns(),
                    cmdpreview_get_bufnr(),
                )
            };
        } else {
            cmdnames[excmd.cmdidx.index()]
                .cmd_func
                .expect("every command in the table has a handler")(excmd);
        }
        if excmd.errmsg.is_some() {
            *errormsg = excmd.errmsg.take();
        }
    }

    Ok(())
}

/// Run an `ExArg` the API built, without re-parsing anything.
///
/// The argument checks `do_one_cmd` makes while parsing are *not* repeated
/// here — the caller is trusted to have produced a sane `ExArg` — but the
/// checks about where a command may run (a locked buffer, the command-line
/// window, a non-'modifiable' buffer) are, because they are about the
/// editor's state rather than about the text.
///
/// # Safety
///
/// `excmd` must point at the command's `ExArg`, unaliased for the call.
/// `cmdinfo` must point at the caller's `CmdParseInfo`, unaliased for the
/// call.
pub unsafe fn execute_cmd(excmd: &mut ExArg, cmdinfo: *mut CmdParseInfo, preview: bool) -> c_int {
    let mut retv: c_int = 0;
    if do_cmdline_start().is_err() {
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
        if Buf::current().b_p_ma == 0
            && excmd.argt.has(ExArgt::MODIFY)
            && !(!Buf::current().terminal.is_null()
                && (excmd.cmdidx == CmdIdx::put || excmd.cmdidx == CmdIdx::iput))
        {
            errormsg = Some(ex_msg(e_modifiable.as_ptr()));
            break 'end;
        }
        if !is_user_cmd(excmd.cmdidx) {
            if cmdwin_type.get() != 0 && !excmd.argt.has(ExArgt::CMDWIN) {
                errormsg = Some(ex_msg(e_cmdwin.as_ptr()));
                break 'end;
            }
            if text_locked() && !excmd.argt.has(ExArgt::LOCK_OK) {
                errormsg = Some(ex_msg(get_text_locked_msg().as_ptr()));
                break 'end;
            }
        }
        // `curbuf->b_ro_locked` forbids editing another buffer.
        // `:checktime` is postponed, `:edit` is checked later, and
        // `:file` with no argument only reports.
        if !excmd.argt.has(ExArgt::CMDWIN)
            && excmd.cmdidx != CmdIdx::checktime
            && excmd.cmdidx != CmdIdx::edit
            && !(excmd.cmdidx == CmdIdx::file && excmd.line.byte_at(excmd.line.arg) == 0)
            && !is_user_cmd(excmd.cmdidx)
            && curbuf_locked()
        {
            break 'end;
        }

        correct_range(excmd);
        if excmd.cmdidx == CmdIdx::SIZE && excmd.addr_count > 0 {
            errormsg = ex_range_without_command(excmd);
            break 'end;
        }

        // Put the first line at the start of a closed fold and the last
        // line at its end.
        if (excmd.argt.has(ExArgt::WHOLEFOLD) || excmd.addr_count >= 2)
            && global_busy.get() == 0
            && excmd.addr_type == CmdAddr::Lines
        {
            has_folding(Win::current(), excmd.line1, Some(&mut excmd.line1), None);
            has_folding(Win::current(), excmd.line2, None, Some(&mut excmd.line2));
        }

        if parse_count(excmd, &mut errormsg, true).is_err() {
            break 'end;
        }

        // A conditional stack of its own: `:try` and friends reached
        // this way are not nested inside the caller's.
        let mut cstack: CondStack = unsafe { core::mem::zeroed() };
        cstack.cs_idx = -1;
        excmd.cstack = &raw mut cstack;

        let _ = unsafe { execute_cmd0(&raw mut retv, excmd, &mut errormsg, preview) };
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
#[expect(dead_code, reason = "the last pointer-form caller goes with part B")]
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
    unsafe { c_int::from(*p) }
}
