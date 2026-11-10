//! The API's view of a command line.
//!
//! `nvim_parse_cmd` reaches `parse_cmdline`, which runs every stage of
//! `do_one_cmd`'s parse and runs none of its effects; `nvim_cmd` reaches
//! `execute_cmd`, which starts from an `ExArg` a Dict was decoded into
//! rather than from text. Between them they are the only callers that can
//! present the command machinery with values no command line could spell,
//! which is why the checks here are spelled out again rather than shared
//! with `do_one_cmd`.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::buffer::buflist_findpat_prefix;
use crate::charset::skip;
use crate::ex_docmd::ex_msg;
use crate::message::emsg;
use crate::types::CmdIdx;
use crate::types::Handle;
use core::ffi::{CStr, c_int};

use crate::ascii::ascii_iswhite;

use crate::eval::skip_expr;
use crate::ex_docmd::address::{correct_range, find_excmd_after_range, parse_cmd_address};
use crate::ex_docmd::addrtype::set_cmd_dflall_range;
use crate::ex_docmd::scan::skip_colons;

use crate::ex_docmd::lookup::is_user_cmd;
use crate::ex_docmd::modifier::{CmdModScope, cmd_has_expr_args, undo_cmdmod};
use crate::ex_docmd::onecmd::{
    append_command, ex_range_without_command, fresh_exarg, shift_cmd_args,
};
use crate::ex_docmd::scan::{parse_bang, parse_register, separate_nextcmd};

use crate::ex_docmd::source::{do_cmdline_end, do_cmdline_start};
use crate::ex_docmd::state::{CMDMOD, GLOBAL_BUSY};
use crate::ex_docmd::{
    CMDNAMES, CmdError, E_AMBIGUOUS_USE_OF_USER_DEFINED_COMMAND, E_NOT_AN_EDITOR_COMMAND,
    EX_PRESSEDRETURN,
};
use crate::ex_getln::{
    cmdpreview_get_bufnr, cmdpreview_get_ns, curbuf_locked, get_text_locked_msg, text_locked,
};
use crate::guard::Suppress;
use crate::message::state::EMSG_SILENT;
use crate::message::{E_CMDWIN, E_COMMAND_TOO_RECURSIVE, E_MODIFIABLE, E_NOBANG, E_NORANGE};
use crate::winlayer::graph::CMDWIN_TYPE;

use crate::search::{restore_last_search_pattern, save_last_search_pattern};
use crate::types::{CmdAddr, CmdLine, CmdParseInfo, ExArg, ExArgt, Pos};
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
/// command through `execute_cmd`. A refusal carries the message the caller
/// reports, if there is one.
///
/// `line` is taken over: the parse writes into it, and what it leaves in
/// `excmd` addresses it by offset.
pub fn parse_cmdline(
    line: CmdLine,
    excmd: &mut ExArg,
    cmdinfo: &mut CmdParseInfo,
) -> Result<(), CmdError> {
    let save_ex_pressedreturn = EX_PRESSEDRETURN.get();
    let save_cursor: Pos = Win::current().w_cursor;
    save_last_search_pattern();

    // The caller's old modifiers go.
    *cmdinfo = crate::ex_getln::CMD_PARSE_INFO_INIT;
    *excmd = fresh_exarg();
    excmd.line = line;

    let parsed = parse_command(excmd, cmdinfo);
    if parsed.is_err() {
        undo_cmdmod(&mut cmdinfo.cmdmod);
    }
    EX_PRESSEDRETURN.set(save_ex_pressedreturn);
    Win::current().w_cursor = save_cursor;
    restore_last_search_pattern();
    parsed
}

/// [`parse_cmdline`]'s parse: the modifiers, then everything after them.
///
/// A modifier that failed to parse is still a modifier: the parse goes on,
/// so that an error is reported against the command rather than against the
/// line, and the modifier's message stands only when nothing after it has
/// one of its own.
fn parse_command(excmd: &mut ExArg, cmdinfo: &mut CmdParseInfo) -> Result<(), CmdError> {
    let orig_cmd = excmd.line.cmd;
    let modifiers = excmd.parse_modifiers(&mut cmdinfo.cmdmod, false);
    let after_modifier = excmd.line.cmd;
    if modifiers.is_err() && after_modifier == orig_cmd {
        return modifiers;
    }
    match (
        parse_after_modifiers(excmd, cmdinfo, after_modifier),
        modifiers,
    ) {
        (Err(CmdError::Quiet), Err(said)) => Err(said),
        (parsed, _) => parsed,
    }
}

/// The range, the command word and its argument, from `after_modifier` on.
fn parse_after_modifiers(
    excmd: &mut ExArg,
    cmdinfo: &mut CmdParseInfo,
    after_modifier: usize,
) -> Result<(), CmdError> {
    // The command name says what kind of address the range counts in.
    let Some(mut p) = find_excmd_after_range(excmd) else {
        return Err(CmdError::Message(ex_msg(
            E_AMBIGUOUS_USE_OF_USER_DEFINED_COMMAND,
        )));
    };

    excmd.set_addr_type(Some(excmd.line.byte_at(excmd.line.skip_white(p))));
    parse_cmd_address(excmd, true)?;

    excmd.line.cmd = skip_colons(&excmd.line, excmd.line.cmd, true);
    if excmd.line.byte_at(excmd.line.cmd) == b'"' {
        return Err(CmdError::Quiet);
    }
    // Nothing at all: no command, no range, no modifier.
    if excmd.line.byte_at(excmd.line.cmd) == 0 && excmd.addr_count == 0 && after_modifier == 0 {
        return Err(CmdError::Quiet);
    }

    // A range on its own (`:1`) or a modifier on its own (`:aboveleft`) is a
    // legal thing to parse.
    if excmd.line.byte_at(excmd.line.cmd) == 0 && excmd.cmdidx == CmdIdx::SIZE {
        excmd.line.arg = excmd.line.cmd;
        if excmd.addr_count > 0 {
            excmd.argt = ExArgt::RANGE;
        } else {
            excmd.argt = ExArgt::NONE;
            excmd.set_addr_kind(CmdAddr::NoRange);
        }
        return Ok(());
    }

    if excmd.cmdidx == CmdIdx::SIZE {
        // The modifiers parsed, so the error is in what follows them.
        let msg = ex_msg(E_NOT_AN_EDITOR_COMMAND);
        return Err(CmdError::Message(append_command(
            &msg,
            excmd.line.rest_of(after_modifier),
        )));
    }

    excmd.forceit = parse_bang(excmd, &mut p);
    if !is_user_cmd(excmd.cmdidx) {
        excmd.argt = CMDNAMES[excmd.cmdidx.index()].cmd_argt;
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
        separate_after_exprs(excmd);
    }

    if !excmd.argt.has(ExArgt::BANG) && excmd.forceit {
        return Err(CmdError::Message(ex_msg(E_NOBANG)));
    }
    if !excmd.argt.has(ExArgt::RANGE) && excmd.addr_count > 0 {
        return Err(CmdError::Message(ex_msg(E_NORANGE)));
    }
    if excmd.argt.has(ExArgt::DFLALL) && excmd.addr_count == 0 {
        set_cmd_dflall_range(excmd);
    }

    parse_register(excmd);
    excmd.parse_count(false)?;

    if let Some(at) = excmd.line.next {
        excmd.line.next = Some(skip_colons(&excmd.line, at, true));
    }

    // Which characters the caller must escape to have them taken literally
    // when the command is handed back.
    if excmd.argt.has(ExArgt::XFILE) {
        cmdinfo.magic.file = true;
    }
    if excmd.argt.has(ExArgt::TRLBAR) {
        cmdinfo.magic.bar = true;
    }
    Ok(())
}

/// Find the next command after an expression argument.
///
/// A command whose argument is an expression has no `ExArgt::TRLBAR`,
/// because a `|` inside the expression is not a separator. Skipping
/// expression by expression finds the one that is.
fn separate_after_exprs(excmd: &mut ExArg) {
    let mut at = excmd.line.arg;
    while !matches!(excmd.line.byte_at(at), 0 | b'|' | b'\n') {
        let skipping = Suppress::emsg_skip();
        let used = skip_expr(excmd.line.rest_of(at));
        drop(skipping);
        // Nothing an expression parser recognises: step over one byte, or
        // this loop never ends.
        at += used.max(1);
    }
    if matches!(excmd.line.byte_at(at), b'|' | b'\n') {
        excmd.line.next = excmd.line.check_next(at);
        excmd.line.terminate_at(at);
    }
}

/// Expand what is left of the argument and call the command's handler.
///
/// The last stage both `do_one_cmd` and `execute_cmd` share: everything
/// before it is validation, everything after it is error reporting.
///
/// Answers the handler's return value (a preview's kind; 0 otherwise). A
/// handler that ran and refused leaves its message in `excmd.errmsg`, which
/// the caller reports.
pub(crate) fn execute_cmd0(excmd: &mut ExArg, preview: bool) -> Result<c_int, CmdError> {
    if excmd.argt.has(ExArgt::XFILE) {
        excmd.expand_filename()?;
    }

    // A buffer name may stand in for a buffer number, but not alongside
    // one, and not for a user command.
    if excmd.argt.has(ExArgt::BUFNAME)
        && excmd.line.byte_at(excmd.line.arg) != 0
        && excmd.addr_count == 0
        && !is_user_cmd(excmd.cmdidx)
    {
        let unlisted = excmd.argt.has(ExArgt::BUFUNL);
        if excmd.line.args.is_empty() {
            // `:bdelete`, `:bwipeout` and `:bunload` take several
            // space-separated names, so the first one ends at the first
            // unescaped space; every other command takes one name, so
            // only trailing space is dropped.
            let arg = excmd.line.arg;
            let end = if excmd.cmdidx == CmdIdx::bdelete
                || excmd.cmdidx == CmdIdx::bwipeout
                || excmd.cmdidx == CmdIdx::bunload
            {
                arg + skip::to_white_esc(excmd.line.rest_of(arg))
            } else {
                let mut at = excmd.line.end_of(arg);
                while at > arg && ascii_iswhite(c_int::from(excmd.line.byte_at(at - 1))) {
                    at -= 1;
                }
                at
            };
            let name = excmd.line.cstr_from(arg);
            let found = buflist_findpat_prefix(name, end - arg, unlisted, false, false)?;
            excmd.number2 = Handle::number_of(found);
            excmd.addr_count = 1;
            excmd.line.arg = excmd.line.skip_white(end);
        } else {
            // The API gave the argument positions, so the first argument
            // is the name with no scanning at all.
            let (at, len) = excmd.line.args[0];
            let name = excmd.line.cstr_from(at);
            let found = buflist_findpat_prefix(name, len, unlisted, false, false)?;
            excmd.number2 = Handle::number_of(found);
            excmd.addr_count = 1;
            shift_cmd_args(excmd);
        }
    }

    // `:try` saves 'emsg_silent' itself, so `:silent! try` must not
    // still be silencing by the time the body runs.
    let did_esilent = CMDMOD.with(|mods| mods.cmod_did_esilent);
    if excmd.cmdidx == CmdIdx::r#try && did_esilent > 0 {
        EMSG_SILENT.set((EMSG_SILENT.get() - did_esilent).max(0));
        CMDMOD.with_mut(|mods| mods.cmod_did_esilent = 0);
    }

    let mut retv = 0;
    if is_user_cmd(excmd.cmdidx) {
        retv = do_ucmd(excmd, preview);
    } else {
        excmd.errmsg = None;
        if preview {
            retv = CMDNAMES[excmd.cmdidx.index()]
                .cmd_preview_func
                .expect("a command with ExArgt::PREVIEW has a preview callback")(
                excmd,
                cmdpreview_get_ns(),
                cmdpreview_get_bufnr(),
            );
        } else {
            CMDNAMES[excmd.cmdidx.index()]
                .cmd_func
                .expect("every command in the table has a handler")(excmd);
        }
    }

    Ok(retv)
}

/// Run an `ExArg` the API built, without re-parsing anything.
///
/// The argument checks `do_one_cmd` makes while parsing are *not* repeated
/// here — the caller is trusted to have produced a sane `ExArg` — but the
/// checks about where a command may run (a locked buffer, the command-line
/// window, a non-'modifiable' buffer) are, because they are about the
/// editor's state rather than about the text.
pub fn execute_cmd(excmd: &mut ExArg, cmdinfo: &CmdParseInfo, preview: bool) -> c_int {
    if do_cmdline_start().is_err() {
        return report(
            excmd,
            Err(CmdError::Message(ex_msg(E_COMMAND_TOO_RECURSIVE))),
        );
    }

    // Shallow both ways: the guard owns what the set it took out points at
    // until it goes back, and the caller keeps owning `cmdinfo`.
    let mods = CmdModScope::enter(cmdinfo.cmdmod.clone());
    let ran = execute_checked(excmd, preview);
    let retv = report(excmd, ran);
    drop(mods);
    do_cmdline_end();
    retv
}

/// Put out what [`execute_cmd`]'s command had to say, and answer its
/// handler's value: a refused command has no preview kind (0).
fn report(excmd: &mut ExArg, ran: Result<c_int, CmdError>) -> c_int {
    let (retv, errormsg) = match ran {
        // A handler that ran may still have refused.
        Ok(retv) => (retv, excmd.errmsg.take()),
        Err(refused) => (0, refused.into_message()),
    };
    if let Some(msg) = &errormsg
        && !msg.is_empty()
    {
        emsg(msg);
    }
    retv
}

/// [`execute_cmd`]'s checks about where the command may run, then the
/// command.
fn execute_checked(excmd: &mut ExArg, preview: bool) -> Result<c_int, CmdError> {
    let refuse = |msg: &'static CStr| Err(CmdError::Message(ex_msg(msg)));
    // `:put` is allowed in a terminal buffer, which is not 'modifiable'.
    if Buf::current().b_p_ma == 0
        && excmd.argt.has(ExArgt::MODIFY)
        && !(!Buf::current().terminal.is_null()
            && (excmd.cmdidx == CmdIdx::put || excmd.cmdidx == CmdIdx::iput))
    {
        return refuse(E_MODIFIABLE);
    }
    if !is_user_cmd(excmd.cmdidx) {
        if CMDWIN_TYPE.get() != 0 && !excmd.argt.has(ExArgt::CMDWIN) {
            return refuse(E_CMDWIN);
        }
        if text_locked() && !excmd.argt.has(ExArgt::LOCK_OK) {
            return refuse(get_text_locked_msg());
        }
    }
    // `CURBUF->b_ro_locked` forbids editing another buffer. `:checktime` is
    // postponed, `:edit` is checked later, and `:file` with no argument only
    // reports.
    if !excmd.argt.has(ExArgt::CMDWIN)
        && excmd.cmdidx != CmdIdx::checktime
        && excmd.cmdidx != CmdIdx::edit
        && !(excmd.cmdidx == CmdIdx::file && excmd.line.byte_at(excmd.line.arg) == 0)
        && !is_user_cmd(excmd.cmdidx)
        && curbuf_locked()
    {
        return Err(CmdError::Quiet);
    }

    correct_range(excmd);
    if excmd.cmdidx == CmdIdx::SIZE && excmd.addr_count > 0 {
        ex_range_without_command(excmd)?;
        return Err(CmdError::Quiet);
    }

    // Put the first line at the start of a closed fold and the last line at
    // its end.
    if (excmd.argt.has(ExArgt::WHOLEFOLD) || excmd.addr_count >= 2)
        && GLOBAL_BUSY.get() == 0
        && excmd.addr_type == CmdAddr::Lines
    {
        Win::current().has_folding(excmd.line1, Some(&mut excmd.line1), None);
        Win::current().has_folding(excmd.line2, None, Some(&mut excmd.line2));
    }

    excmd.parse_count(true)?;

    // A conditional stack of its own: `:try` and friends reached this way
    // are not nested inside the caller's.
    let cond_stack = crate::ex_eval::OwnedCondStack::open();
    excmd.cond_stack = Some(cond_stack.id());

    execute_cmd0(excmd, preview)
}
