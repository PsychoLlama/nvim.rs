//! What surrounds a `do_cmdline` call: the exception state it saves and
//! restores for the debugger, the sources it reads its lines from -- the
//! [`LineSource`] arms and the loop line store `:while` and `:for` replay
//! from -- and Ex mode.
#![forbid(unsafe_code)]
use crate::buffer::buf_get_changedtick;
use crate::eval::vars::{set_vim_var_owned, vim_var_string};
use crate::message::emsg;
use crate::os::cshim::gettext;

use crate::getchar::typeahead;
use crate::guard::Suppress;
use crate::memline::MlFlags;
use crate::smsg;
use core::cell::Cell;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use crate::autocmd::getnextac;
use crate::clipboard::{end_batch_changes, start_batch_changes};
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later, update_screen};
use crate::eval::funcs::{ListLines, get_list_line};
use crate::eval::userfunc::get_func_line;
use crate::eval::vars::set_vim_var_string;
use crate::getchar::read_cmd_key_line;
use crate::memory::XString;
use crate::runtime::SourceCookie;
use crate::types::{AutoPatCmd, FuncCall};

use crate::ex_cmds::print_line_no_prefix;
use crate::ex_docmd::cmdline::do_cmdline;
use crate::ex_docmd::sourcing_lnum;

use crate::drawscreen::state::cmdline_row;
use crate::ex_docmd::state::{ex_no_reprint, ex_normal_busy, global_busy};
use crate::ex_docmd::{
    DoCmdOpts, ETYPE_EXCEPT, MSG_BUF_LEN, SavedDebugState, cmdline_call_depth, ex_pressedreturn,
};
use crate::ex_eval::discard_current_exception;
use crate::ex_eval::state::{
    caught_stack, check_cstack, current_exception, did_throw, force_abort, need_rethrow,
    suppress_errthrow, trylevel,
};
use crate::ex_getln::{cmdline_input, getexline};
use crate::getchar::state::{KeyTyped, got_int};
use crate::highlight_group::HLF_E;
use crate::message::e_empty_buffer;
use crate::message::state::{
    did_emsg, emsg_silent, lines_left, msg_col, msg_row, msg_scroll, msg_silent, need_wait_return,
};
use crate::option::vars::p_mfd;
use crate::startup::exiting;
use crate::state::mode::{State, exmode_active};
use crate::ui::state::Rows;

use crate::message::{
    emsg_multiline, msg, msg_clr_eos, msg_scroll_flush, msg_str, verbose_enter_scroll,
    verbose_leave_scroll,
};
use crate::message_fmt::msg_bytes;

use crate::runtime::{estack_pop, estack_push, set_sourcing_lnum};
use crate::state::{MODE_NORMAL, may_trigger_modechanged};

use crate::types::{CmdLine, ExArg, Failed, IOSIZE, LineNr, OptInt, Vv};

use crate::winlayer::{Buf, Win};

/// Take the whole exception environment out of the way, and answer it.
///
/// Used only by the debugger: a `>quit` at a breakpoint must not be
/// swallowed by whatever `:try` the script had open.
pub(crate) fn save_dbg_stuff() -> SavedDebugState {
    SavedDebugState {
        trylevel: trylevel.replace(0),
        force_abort: c_int::from(force_abort.replace(false)),
        caught_stack: caught_stack.take(),
        // Copies, which leave the variables as they are: a `:catch` or a
        // discarded exception in the debugger may replace them, and a
        // pointer to the variable's own string would not survive that.
        vv_exception: vim_var_string(Vv::Exception),
        vv_throwpoint: vim_var_string(Vv::Throwpoint),
        did_emsg: did_emsg.replace(0),
        got_int: c_int::from(got_int.replace(false)),
        did_throw: did_throw.replace(false),
        need_rethrow: c_int::from(need_rethrow.replace(false)),
        check_cstack: c_int::from(check_cstack.replace(false)),
        current_exception: current_exception.take(),
    }
}

/// Put it all back.
pub(crate) fn restore_dbg_stuff(saved: SavedDebugState) {
    suppress_errthrow.set(false);
    trylevel.set(saved.trylevel);
    force_abort.set(saved.force_abort != 0);
    caught_stack.set(saved.caught_stack);
    // A null saved value puts nothing back, as upstream's `v_exception(NULL)`
    // is the read half.
    if let Some(exception) = saved.vv_exception {
        set_vim_var_owned(Vv::Exception, Some(exception));
    }
    if let Some(throwpoint) = saved.vv_throwpoint {
        set_vim_var_owned(Vv::Throwpoint, Some(throwpoint));
    }
    did_emsg.set(saved.did_emsg);
    got_int.set(saved.got_int != 0);
    did_throw.set(saved.did_throw);
    need_rethrow.set(saved.need_rethrow != 0);
    check_cstack.set(saved.check_cstack != 0);
    current_exception.set(saved.current_exception);
}

/// Ex mode: read and run one command line at a time, printing the current
/// line after each one that moved the cursor or changed the buffer.
pub fn do_exmode() {
    exmode_active.set(true);
    State.set(MODE_NORMAL);
    may_trigger_modechanged();

    // `:global` runs Ex mode for each line itself; there is no prompt
    // to give.
    if global_busy.get() != 0 {
        return;
    }

    let save_msg_scroll = msg_scroll.get();
    let redraw_off = Suppress::redraw();
    let no_prompt = Suppress::wait_return();
    msg(
        gettext(c"Entering Ex mode.  Type \"visual\" to go to Normal mode."),
        0,
    );

    while exmode_active.get() {
        // `:normal` that ran out of keys leaves Ex mode rather than
        // waiting for more.
        if ex_normal_busy.get() > 0 && typeahead().is_empty() {
            exmode_active.set(false);
            break;
        }

        msg_scroll.set(1);
        need_wait_return.set(false);
        ex_pressedreturn.set(false);
        ex_no_reprint.set(false);
        let changedtick = buf_get_changedtick(Buf::current());
        let prev_msg_row = msg_row.get();
        let prev_line = Win::current().w_cursor.lnum;
        cmdline_row.set(msg_row.get());

        let _ = do_cmdline(None, Some(&mut LineSource::Typed), DoCmdOpts::NONE);
        lines_left.set(Rows.get() - 1);

        let moved = prev_line != Win::current().w_cursor.lnum
            || changedtick != buf_get_changedtick(Buf::current());
        if moved && !ex_no_reprint.get() {
            if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
                emsg(gettext(e_empty_buffer));
            } else {
                // A bare Return already scrolled; print over that line
                // rather than under it.
                if ex_pressedreturn.get() {
                    msg_scroll_flush();
                    msg_row.set(prev_msg_row);
                    if prev_msg_row == Rows.get() - 1 {
                        msg_row.set(msg_row.get() - 1);
                    }
                }
                msg_col.set(0);
                print_line_no_prefix(Win::current().w_cursor.lnum, false, false);
                msg_clr_eos();
            }
        } else if ex_pressedreturn.get() && !ex_no_reprint.get() {
            // Return on the last line: there is nothing to print.
            if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
                emsg(gettext(e_empty_buffer));
            } else {
                emsg(gettext(c"E501: At end-of-file"));
            }
        }
    }

    drop(redraw_off);
    drop(no_prompt);
    redraw_all_later(UPD_NOT_VALID);
    let _ = update_screen();
    need_wait_return.set(false);
    msg_scroll.set(save_msg_scroll);
}

/// `:verbose` >= 15: report the command about to run, and which line of
/// which script it is.
pub(crate) fn msg_verbose_cmd(lnum: LineNr, cmd: &[u8]) {
    let _no_prompt = Suppress::wait_return();
    verbose_enter_scroll();
    let cmd = msg_bytes(cmd);
    if lnum == 0 {
        smsg!(0, "Executing: {cmd}");
    } else {
        smsg!(0, "line {}: {cmd}", lnum);
    }
    if msg_silent.get() == 0 {
        msg_str(c"\n");
    }
    verbose_leave_scroll();
}

/// Enter a `do_cmdline` call, refusing to nest past 'maxfuncdepth'.
///
/// The limit only bites above a floor of 200: a low 'maxfuncdepth' must
/// still leave room for the editor's own nesting.
pub(crate) fn do_cmdline_start() -> Result<(), Failed> {
    debug_assert!(cmdline_call_depth.get() >= 0);
    if cmdline_call_depth.get() >= 200 && cmdline_call_depth.get() as OptInt >= p_mfd() {
        return Err(Failed);
    }
    cmdline_call_depth.set(cmdline_call_depth.get() + 1);
    // Clipboard writes are batched across the whole command line, so
    // that a `:while` that yanks repeatedly sets the selection once.
    start_batch_changes();
    Ok(())
}

/// Leave it.
pub(crate) fn do_cmdline_end() {
    cmdline_call_depth.set(cmdline_call_depth.get() - 1);
    debug_assert!(cmdline_call_depth.get() >= 0);
    end_batch_changes();
}

/// Report an exception that reached the outermost `:try`, and discard it.
///
/// A user exception (`:throw`) is reported as E605; an error exception
/// replays the messages it was built from, so that the original error text
/// is what the user sees; an interrupt says nothing, because the interrupt
/// message is given elsewhere.
pub fn handle_did_throw() {
    let id = current_exception.get().expect("an exception being thrown");
    // The messages are taken so that discarding the exception does not drop
    // them, and the name goes to the frame pushed below.
    let (kind, value, messages, throw_name, throw_lnum) = id.with(|exception| {
        (
            exception.type_0,
            exception.value.clone(),
            core::mem::take(&mut exception.messages),
            core::mem::take(&mut exception.throw_name),
            exception.throw_lnum,
        )
    });
    // A user exception (ET_USER) is reported as not caught; an error one
    // (ET_ERROR) replays its messages; an interrupt says nothing.
    let reported = (kind == 0).then(|| {
        // Upstream renders it into `IObuff`.
        let template = gettext(c"E605: Exception not caught: %s").to_bytes();
        let at = template
            .windows(2)
            .position(|pair| pair == b"%s")
            .unwrap_or(template.len());
        let mut text = template[..at].to_vec();
        text.extend_from_slice(&value);
        text.extend_from_slice(template.get(at + 2..).unwrap_or_default());
        text.truncate(IOSIZE as usize - 1);
        CString::new(text).expect("no NUL in a message or a value")
    });

    // Report against where the exception was thrown, not where it was
    // caught. The frame names `throw_name` by pointer, and this frame holds
    // the name until the frame is popped.
    estack_push(ETYPE_EXCEPT, throw_name.as_ptr().cast_mut(), throw_lnum);
    discard_current_exception();

    // `:silent!` makes even an uncaught exception non-fatal.
    if emsg_silent.get() == 0 {
        suppress_errthrow.set(true);
        force_abort.set(true);
    }

    if !messages.entries.is_empty() {
        for m in &messages.entries {
            emsg_multiline(m.msg.as_cstr(), Some(c"emsg"), HLF_E, m.multiline);
        }
    } else if let Some(reported) = reported {
        emsg(&reported);
    }

    estack_pop();
    drop(throw_name);
}

// ---------------------------------------------------------------------------
// Where the lines come from.

/// What a line source is, looking through every `:while`/`:for` replaying
/// around it: the three questions upstream asked `getline_equal` (is this a
/// function body, a sourced script, the `:` prompt?) and the two more the
/// command parse asks (an autocommand, or no source at all).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SourceKind {
    /// No reader: a line given whole, which a command that wants more of its
    /// body reads from the command line.
    None,
    /// The `:` prompt.
    Typed,
    /// The command a `<Cmd>` key carries, read back out of the typeahead.
    CmdKey,
    /// `execute()`'s List.
    List,
    /// A function body.
    Function,
    /// A sourced script, file or buffer or string.
    Script,
    /// An autocommand's commands.
    Autocmd,
}

/// Where the debugger keeps the breakpoint of a function or a script being
/// run, and the tick it was looked up at: borrowed from the function's frame
/// or the script's cookie, so a lookup made through it is the frame's own.
pub(crate) struct DebugPos<'s> {
    /// The function's or the script's name, which a breakpoint names.
    pub(crate) name: &'s CStr,
    pub(crate) breakpoint: &'s Cell<LineNr>,
    pub(crate) tick: &'s Cell<c_int>,
}

/// A line source, as a running command and a `:while` around it hold it.
///
/// Implemented by [`LineSource`] alone. It exists because a source has to be
/// wrapped by the loop *inside* the run reading from it: `&mut LineSource<'a>`
/// is invariant in `'a`, so a `:while` could not lend the run's source to its
/// loop arm for any shorter borrow than the source's own. A trait object can:
/// `&'b mut (dyn ReadLines + 'a)` shortens to `+ 'b`. Every method sees
/// through the loops to the source underneath, but [`is_loop`](Self::is_loop).
pub(crate) trait ReadLines {
    /// The next line, or `None` at the end of the input. `c` is the prompt
    /// character, `indent` the indent to show and `do_concat` whether `\`
    /// continuation lines are joined -- each source takes what it needs.
    fn next_line(&mut self, c: c_int, indent: c_int, do_concat: bool) -> Option<XString>;
    /// Which kind of source this is.
    fn kind(&self) -> SourceKind;
    /// Whether this is a `:while`/`:for` replaying, rather than the source
    /// itself.
    fn is_loop(&self) -> bool;
    /// The function this is the body of.
    fn function(&self) -> Option<&FuncCall>;
    /// The script this is reading.
    fn script(&self) -> Option<&SourceCookie>;
    /// The script this is reading, to write to (`:finish`,
    /// `:scriptencoding`).
    fn script_mut(&mut self) -> Option<&mut SourceCookie>;
    /// Where the breakpoint of the function or script this is reading is
    /// kept.
    fn debug(&mut self) -> Option<DebugPos<'_>>;
}

/// Every source a `do_cmdline` run reads its lines from.
pub(crate) enum LineSource<'a> {
    /// No reader: the run has the one line it was given.
    None,
    /// The `:` prompt (`getexline`).
    Typed,
    /// A `<Cmd>` key's command.
    CmdKey,
    /// `execute()`'s List walk.
    List(&'a mut ListLines),
    /// A function body, read from its frame.
    Function(&'a FuncCall),
    /// A script, read through its cookie.
    Script(&'a mut SourceCookie),
    /// An autocommand's commands, one handler per line.
    Autocmd(&'a mut AutoPatCmd),
    /// A `:while`/`:for` body being stored and replayed, around another
    /// source.
    Loop(LoopLines<'a>),
}

/// One line of a `:while`/`:for` body, as it was read, and the line number
/// it was read at.
pub(crate) struct StoredLine {
    pub(crate) line: Vec<u8>,
    pub(crate) lnum: LineNr,
}

/// The source a command inside a `:while` or `:for` is handed.
///
/// Replays a stored line when there is one, and otherwise reads a new line
/// from the source underneath and stores it on the way through. That is
/// what lets a `:function` be defined inside a loop: its body is read once
/// and replayed with everything else. A loop inside a loop wraps it again.
pub(crate) struct LoopLines<'a> {
    /// The body stored so far, and the line last handed out.
    pub(crate) store: &'a mut Vec<StoredLine>,
    pub(crate) current_line: &'a mut c_int,
    /// Replaying: past the end of the store is the end of the body.
    pub(crate) repeating: bool,
    pub(crate) inner: &'a mut dyn ReadLines,
}

impl LoopLines<'_> {
    fn next_line(&mut self, c: c_int, indent: c_int, do_concat: bool) -> Option<XString> {
        let next = usize::try_from(*self.current_line + 1).unwrap_or(0);
        let Some(stored) = self.store.get(next) else {
            // Past the end of what was stored. On a repeat pass that means
            // the loop body is over.
            if self.repeating {
                return None;
            }
            let line = self.inner.next_line(c, indent, do_concat)?;
            self.store.push(StoredLine {
                line: line.as_cstr().to_bytes().to_vec(),
                lnum: sourcing_lnum(),
            });
            *self.current_line += 1;
            return Some(line);
        };
        // A replayed line was not typed. It is handed out as a copy: the
        // store keeps its line, and a command modifies what it is handed.
        KeyTyped.set(false);
        *self.current_line += 1;
        set_sourcing_lnum(stored.lnum);
        Some(XString::from_bytes(&stored.line))
    }
}

impl ReadLines for LineSource<'_> {
    fn next_line(&mut self, c: c_int, indent: c_int, do_concat: bool) -> Option<XString> {
        match self {
            // A command that reads more of its body with no source to read
            // it from asks the command line.
            LineSource::None => cmdline_input(c, indent, do_concat),
            LineSource::Typed => getexline(c, indent, do_concat),
            LineSource::CmdKey => read_cmd_key_line(),
            LineSource::List(walk) => get_list_line(walk),
            LineSource::Function(frame) => get_func_line(frame),
            LineSource::Script(cookie) => cookie.getsourceline(do_concat),
            LineSource::Autocmd(walk) => getnextac(walk),
            LineSource::Loop(lines) => lines.next_line(c, indent, do_concat),
        }
    }

    fn kind(&self) -> SourceKind {
        match self {
            LineSource::None => SourceKind::None,
            LineSource::Typed => SourceKind::Typed,
            LineSource::CmdKey => SourceKind::CmdKey,
            LineSource::List(_) => SourceKind::List,
            LineSource::Function(_) => SourceKind::Function,
            LineSource::Script(_) => SourceKind::Script,
            LineSource::Autocmd(_) => SourceKind::Autocmd,
            LineSource::Loop(lines) => lines.inner.kind(),
        }
    }

    fn is_loop(&self) -> bool {
        matches!(self, LineSource::Loop(_))
    }

    fn function(&self) -> Option<&FuncCall> {
        match self {
            LineSource::Function(frame) => Some(frame),
            LineSource::Loop(lines) => lines.inner.function(),
            _ => None,
        }
    }

    fn script(&self) -> Option<&SourceCookie> {
        match self {
            LineSource::Script(cookie) => Some(cookie),
            LineSource::Loop(lines) => lines.inner.script(),
            _ => None,
        }
    }

    fn script_mut(&mut self) -> Option<&mut SourceCookie> {
        match self {
            LineSource::Script(cookie) => Some(cookie),
            LineSource::Loop(lines) => lines.inner.script_mut(),
            _ => None,
        }
    }

    fn debug(&mut self) -> Option<DebugPos<'_>> {
        match self {
            LineSource::Function(frame) => Some(DebugPos {
                name: frame.func.name().as_cstr(),
                breakpoint: &frame.breakpoint,
                tick: &frame.dbg_tick,
            }),
            LineSource::Script(cookie) => Some(cookie.debug_pos()),
            LineSource::Loop(lines) => lines.inner.debug(),
            _ => None,
        }
    }
}

impl ExArg<'_> {
    /// What the command reads its further lines from; [`SourceKind::None`]
    /// when it was given no source.
    pub(crate) fn source_kind(&self) -> SourceKind {
        self.source
            .as_ref()
            .map_or(SourceKind::None, |source| source.kind())
    }

    /// Where the command reads its further lines from, lent for a nested
    /// `do_cmdline` to read its own through.
    pub(crate) fn source_mut(&mut self) -> Option<&mut dyn ReadLines> {
        self.line_and_source().1
    }

    /// [`source_mut`](Self::source_mut), with the command's own line beside
    /// it: what `:windo` and friends run on each pass.
    pub(crate) fn line_and_source(&mut self) -> (&CmdLine, Option<&mut dyn ReadLines>) {
        let source: Option<&mut dyn ReadLines> = match &mut self.source {
            Some(source) => Some(&mut **source),
            None => None,
        };
        (&self.line, source)
    }

    /// The script the command is running in, when it reads its lines from
    /// one (through any loop).
    pub(crate) fn sourced_script(&mut self) -> Option<&mut SourceCookie> {
        self.source.as_mut()?.script_mut()
    }
}

/// Whether `excmd` reads its lines from a script being `:source`d -- the
/// file itself, or a loop inside it.
pub(crate) fn reads_sourced_lines(excmd: &ExArg) -> bool {
    excmd.source_kind() == SourceKind::Script
}

/// A translated message, copied into an owned Ex-command error message.
///
/// Every producer of an `ExArg::errmsg` — and of the `errormsg`
/// out-parameter the parser threads — answers a buffer of its own, so that
/// raising a second error before the first is reported cannot overwrite it.
/// Upstream shared two static buffers here (`IObuff` and `ex_error_buf`)
/// and `emsg` runs autocommands, so the overwrite was reachable.
pub(crate) fn ex_msg(msg: &'static CStr) -> CString {
    gettext(msg).to_owned()
}

/// [`ex_msg`] for a message with one `%s` in it.
///
/// Cut where upstream's `MSG_BUF_LEN` buffer cut it; an interior NUL in
/// `arg` ends the message, as it did there.
pub(crate) fn ex_errmsg(msg: &'static CStr, arg: &CStr) -> CString {
    let text = crate::formatted!(MSG_BUF_LEN as usize, gettext(msg).to_bytes(), arg);
    CString::new(text).unwrap_or_default()
}

/// Cancel an exit that a QuitPre or ExitPre autocommand called off.
pub fn not_exiting(save_exiting: bool) {
    exiting.set(save_exiting);
    set_vim_var_string(Vv::Exitreason, None);
}

/// The next line of the body a running command reads, from wherever the
/// command reads its own lines: the command line itself when it has no
/// source. `None` at the end of input.
pub(crate) fn read_next_line(
    excmd: &mut ExArg,
    c: c_int,
    indent: c_int,
    do_concat: bool,
) -> Option<XString> {
    match excmd.source.as_mut() {
        None => cmdline_input(c, indent, do_concat),
        Some(source) => source.next_line(c, indent, do_concat),
    }
}
