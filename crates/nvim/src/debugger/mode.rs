//! Debug mode: the `>` prompt, the backtrace, and `:debug`/`:debuggreedy`.
//!
//! [`do_debug`] is entered from [`super::dbg_check_breakpoint`] when a
//! breakpoint was hit or when the last `>` command asked to stop at this
//! nesting level. It takes the screen over, reads `>` commands until one of
//! them resumes execution, and leaves `DEBUG_BREAK_LEVEL` set to whichever
//! depth should stop next.
//!
//! Anything typed at the prompt that is not one of the dozen `>` commands is
//! run as an ordinary Ex command and the prompt comes back, which is why the
//! parser here answers `None` rather than an error.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use super::*;
use crate::ex_docmd::{DoCmdOpts, LineSource, do_cmdline};
use crate::guard::{Allow, Bump, MsgBump, Saved};
use crate::message::msg;
use crate::message::state::MsgField;
use crate::message_fmt::{msg_cstr, report_msg};
use crate::tr_c;

/// The editor state [`do_debug`] takes over while the `>` prompt is up, and
/// puts back on the way out. The prompt has to be *visible*, so silence and
/// redirection are off for its duration whatever the debugged code asked for.
#[derive(Debug)]
struct SavedState {
    msg_scroll: c_int,
    state: c_int,
    did_emsg: c_int,
    cmd_silent: bool,
    emsg_silent: c_int,
    redir_off: bool,
    /// Released at the top of [`SavedState::leave`], before the redraw is
    /// queued — and by dropping the whole state if the prompt panics out.
    redraw_off: Bump,
    no_prompt: MsgBump,
    /// `MSG_SILENT`, put back late with the rest of the message state.
    loud: Saved<MsgField<c_int>>,
}

impl SavedState {
    fn enter() -> Self {
        // Do not redisplay the window, and do not wait for a return.
        let redraw_off = Suppress::redraw();
        let no_prompt = Suppress::wait_return();
        // The prompt has to be visible whatever the debugged code asked for.
        let loud = Allow::messages();
        let saved = Self {
            msg_scroll: MSG_SCROLL.get(),
            state: STATE.get(),
            did_emsg: DID_EMSG.get(),
            cmd_silent: CMD_SILENT.get(),
            emsg_silent: EMSG_SILENT.get(),
            redir_off: REDIR_OFF.get(),
            redraw_off,
            no_prompt,
            loud,
        };
        // An error from the debugged code is not ours.
        DID_EMSG.set(0);
        CMD_SILENT.set(false);
        EMSG_SILENT.set(0);
        // Debug commands are not part of the redirected output.
        REDIR_OFF.set(true);
        STATE.set(MODE_NORMAL);
        DEBUG_MODE.set(true);
        saved
    }

    fn leave(self) {
        drop(self.redraw_off);
        drop(self.no_prompt);
        redraw_all_later(UPD_NOT_VALID);
        NEED_WAIT_RETURN.set(false);
        MSG_SCROLL.set(self.msg_scroll);
        LINES_LEFT.set(ROWS.get() - 1);
        STATE.set(self.state);
        DEBUG_MODE.set(false);
        DID_EMSG.set(self.did_emsg);
        CMD_SILENT.set(self.cmd_silent);
        drop(self.loud);
        EMSG_SILENT.set(self.emsg_silent);
        REDIR_OFF.set(self.redir_off);
        // Print the banner again only after something else has been typed.
        DEBUG_DID_MSG.set(true);
    }
}

/// Debug mode: repeatedly read an Ex command, until told to continue normal
/// execution. `cmd` is the command line about to be executed.
pub fn do_debug(cmd: &CStr) {
    let saved = SavedState::enter();
    show_debug_banner(cmd);
    debug_prompt(cmd);
    saved.leave();
}

/// What is printed on the way in: why we stopped, where, and on which line.
fn show_debug_banner(cmd: &CStr) {
    if !DEBUG_DID_MSG.get() {
        smsg!(0, "Entering Debug mode.  Type \"cont\" to continue.");
    }
    // A watch expression that just changed left both of its values here.
    // They are `typval_tostring` output -- bytes, not necessarily UTF-8 -- so
    // they go through vim's own printf rather than through `format_args!`.
    for (label, cell) in [
        (c"Oldval = \"%s\"", &DEBUG_OLDVAL),
        (c"Newval = \"%s\"", &DEBUG_NEWVAL),
    ] {
        let Some(text) = cell.take() else {
            continue;
        };
        let shown = msg_cstr(text.as_cstr());
        let _: bool = report_msg(0, || tr_c!(label, shown));
    }

    if let Some(sname) = estack_sfile(ESTACK_NONE) {
        msg(sname.as_cstr(), 0);
    }
    show_debug_line(cmd);
}

/// The `line N: <cmd>` / `cmd: <cmd>` line, which both the banner and
/// `>backtrace` end with. The command line is arbitrary bytes, kept as they
/// are.
fn show_debug_line(cmd: &CStr) {
    let lnum = sourcing_lnum();
    let cmd = msg_cstr(cmd);
    if lnum != LineNr::ZERO {
        smsg!(0, "line {}: {cmd}", i64::from(lnum));
    } else {
        smsg!(0, "cmd: {cmd}");
    }
}

/// A command understood at the `>` prompt.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DebugCmd {
    /// `cont`: run on without stopping again.
    Cont,
    /// `next`: stop at the next command in this nesting level.
    Next,
    /// `step`: stop at the very next command, however deep.
    Step,
    /// `finish`: stop when this level returns.
    Finish,
    /// `quit`: interrupt and stop debugging.
    Quit,
    /// `interrupt`: interrupt, but keep stepping afterwards.
    Interrupt,
    /// `backtrace`/`bt`/`where`: print the call stack.
    Backtrace,
    /// `frame [N]`: move to, or print, a stack frame.
    Frame,
    /// `up`/`down`: move one frame.
    Up,
    Down,
}

/// The `>`-prompt command at the head of `line`, and how many bytes of it the
/// name took -- the rest is `>frame`'s argument.
///
/// `None` is "not a debug command", which sends the whole line to
/// `do_cmdline` instead.
fn parse_debug_cmd(line: &[u8]) -> Option<(DebugCmd, usize)> {
    // Each command is matched by its first letter; what follows only has to
    // be a prefix of the full spelling, so `>c`, `>co` and `>cont` are one
    // command. `f` and `b` need a second letter to tell their pairs apart.
    let (cmd, rest): (DebugCmd, &[u8]) = match (*line.first()?, line.get(1)) {
        (b'c', _) => (DebugCmd::Cont, b"ont"),
        (b'n', _) => (DebugCmd::Next, b"ext"),
        (b's', _) => (DebugCmd::Step, b"tep"),
        (b'f', Some(b'r')) => (DebugCmd::Frame, b"rame"),
        (b'f', _) => (DebugCmd::Finish, b"inish"),
        (b'q', _) => (DebugCmd::Quit, b"uit"),
        (b'i', _) => (DebugCmd::Interrupt, b"nterrupt"),
        (b'b', Some(b't')) => (DebugCmd::Backtrace, b"t"),
        (b'b', _) => (DebugCmd::Backtrace, b"acktrace"),
        (b'w', _) => (DebugCmd::Backtrace, b"here"),
        (b'u', _) => (DebugCmd::Up, b"p"),
        (b'd', _) => (DebugCmd::Down, b"own"),
        _ => return None,
    };
    let matched = line[1..]
        .iter()
        .zip(rest)
        .take_while(|(typed, full)| typed == full)
        .count();
    let end = 1 + matched;
    // A letter left over means it was some other word all along -- except
    // after `>frame`, whose level may follow without a space.
    if cmd != DebugCmd::Frame && line.get(end).is_some_and(u8::is_ascii_alphabetic) {
        return None;
    }
    Some((cmd, end))
}

/// Read `>` commands until one of them resumes execution.
///
/// Anything that is not a debug command is run as an Ex command and the
/// prompt comes back.
fn debug_prompt(cmd: &CStr) {
    /// The command last given, reused for a blank line. Static, so `>step`
    /// followed by three empty lines steps four times.
    static LAST_CMD: GlobalCell<Option<DebugCmd>> = GlobalCell::new(None);

    // These three outlive the iteration that sets them, exactly as upstream's
    // do: `:debuggreedy` can be typed at this very prompt, so a pass that
    // does not save the typeahead may still restore what an earlier one did.
    let mut typeaheadbuf = TypeaheadSave::default();
    let mut typeahead_saved = false;
    let mut save_ignore_script = false;

    loop {
        MSG_SCROLL.set(1);
        NEED_WAIT_RETURN.set(false);

        // Read from the user, not from whatever a mapping or a script had
        // queued: swap in an empty typeahead buffer, drop `:normal`'s side
        // effects, and stop reading script input.
        let save_ex_normal_busy = EX_NORMAL_BUSY.get();
        EX_NORMAL_BUSY.set(0);
        if !DEBUG_GREEDY.get() {
            save_typeahead(&mut typeaheadbuf);
            typeahead_saved = true;
            save_ignore_script = IGNORE_SCRIPT.get();
            IGNORE_SCRIPT.set(true);
        }

        // Do not debug whatever reading the line itself runs -- an expression
        // mapping, for instance.
        let outer_level = DEBUG_BREAK_LEVEL.replace(-1);
        let cmdline = getcmdline_bare(b'>');
        DEBUG_BREAK_LEVEL.set(outer_level);

        if typeahead_saved {
            // Paired with the `save_typeahead` above (or an earlier pass's,
            // per the note on the declaration).
            restore_typeahead(&mut typeaheadbuf);
            IGNORE_SCRIPT.set(save_ignore_script);
        }
        EX_NORMAL_BUSY.set(save_ex_normal_busy);

        CMDLINE_ROW.set(MSG_ROW.get());
        msg_starthere();

        if let Some(cmdline) = cmdline {
            let start = skip_white(&cmdline, 0);
            let line = &cmdline[start..];
            // A blank line repeats: only a line with something on it decides
            // what `LAST_CMD` is.
            let mut arg = line;
            if !line.is_empty() {
                match parse_debug_cmd(line) {
                    Some((parsed, end)) => {
                        LAST_CMD.set(Some(parsed));
                        arg = &line[end..];
                    }
                    None => LAST_CMD.set(None),
                }
            }

            if let Some(parsed) = LAST_CMD.get() {
                if run_debug_cmd(parsed, cmd, arg, &LAST_CMD) {
                    continue;
                }
                // On the way out, the backtrace is back at the bottom.
                DEBUG_BACKTRACE_LEVEL.set(0);
                break;
            }

            // Not a debug command, so run it -- but do not debug it.
            let outer_level = DEBUG_BREAK_LEVEL.replace(-1);
            let flags = DoCmdOpts::VERBOSE | DoCmdOpts::EXCRESET;
            let _ = do_cmdline(Some(&cmdline[..]), Some(&mut LineSource::Typed), flags);
            DEBUG_BREAK_LEVEL.set(outer_level);
        }
        LINES_LEFT.set(ROWS.get() - 1);
    }
}

/// Act on one `>` command. True means "ask again" -- the stack-walking
/// commands do not resume execution. `arg` is what followed the command's
/// name in the line just read.
fn run_debug_cmd(
    parsed: DebugCmd,
    cmd: &CStr,
    arg: &[u8],
    last_cmd: &GlobalCell<Option<DebugCmd>>,
) -> bool {
    match parsed {
        DebugCmd::Cont => DEBUG_BREAK_LEVEL.set(-1),
        DebugCmd::Next => DEBUG_BREAK_LEVEL.set(EX_NESTING_LEVEL.get()),
        DebugCmd::Step => DEBUG_BREAK_LEVEL.set(9999),
        DebugCmd::Finish => DEBUG_BREAK_LEVEL.set(EX_NESTING_LEVEL.get() - 1),
        DebugCmd::Quit => {
            GOT_INT.set(true);
            DEBUG_BREAK_LEVEL.set(-1);
        }
        DebugCmd::Interrupt => {
            GOT_INT.set(true);
            DEBUG_BREAK_LEVEL.set(9999);
            // `>interrupt` does not repeat on a blank line; keep stepping.
            last_cmd.set(Some(DebugCmd::Step));
        }
        DebugCmd::Backtrace => {
            do_showbacktrace(cmd);
            return true;
        }
        DebugCmd::Frame => {
            if arg.is_empty() {
                do_showbacktrace(cmd);
            } else {
                do_setdebugtracelevel(&arg[skip_white(arg, 0)..]);
            }
            return true;
        }
        DebugCmd::Up => {
            DEBUG_BACKTRACE_LEVEL.set(DEBUG_BACKTRACE_LEVEL.get() + 1);
            do_checkbacktracelevel();
            return true;
        }
        DebugCmd::Down => {
            DEBUG_BACKTRACE_LEVEL.set(DEBUG_BACKTRACE_LEVEL.get() - 1);
            do_checkbacktracelevel();
            return true;
        }
    }
    false
}

/// How deep the execution stack is, read off `estack_sfile`'s `..`-joined
/// rendering of it.
fn get_maxbacktrace_level(sname: Option<&[u8]>) -> c_int {
    let Some(joined) = sname else {
        return 0;
    };
    // Non-overlapping, the way `strstr` plus `p += 2` counts them: in a name
    // holding `...` that is one separator followed by a dot, not two
    // separators. A `windows(2)` count would answer differently.
    let (mut levels, mut i) = (0, 0);
    while i + 1 < joined.len() {
        if joined[i] == b'.' && joined[i + 1] == b'.' {
            levels += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    levels
}

/// `>frame N`, `>frame +N` and `>frame -N`.
fn do_setdebugtracelevel(arg: &[u8]) {
    let (level, relative) = (atoi(arg), arg.first() == Some(&b'+'));
    if relative || level < 0 {
        DEBUG_BACKTRACE_LEVEL.set(DEBUG_BACKTRACE_LEVEL.get() + level);
    } else {
        DEBUG_BACKTRACE_LEVEL.set(level);
    }
    do_checkbacktracelevel();
}

/// Clamp the requested frame to one that exists, saying so when it moved.
fn do_checkbacktracelevel() {
    if DEBUG_BACKTRACE_LEVEL.get() < 0 {
        DEBUG_BACKTRACE_LEVEL.set(0);
        smsg!(0, "frame is zero");
        return;
    }
    let sname = estack_sfile(ESTACK_NONE);
    let max = get_maxbacktrace_level(sname.as_deref());
    if DEBUG_BACKTRACE_LEVEL.get() > max {
        DEBUG_BACKTRACE_LEVEL.set(max);
        smsg!(0, "frame at highest level: {max}");
    }
}

/// `>backtrace`: the execution stack, innermost last, with `->` on the frame
/// `>up`/`>down` have selected.
fn do_showbacktrace(cmd: &CStr) {
    let sname = estack_sfile(ESTACK_NONE);
    let max = get_maxbacktrace_level(sname.as_deref());
    if let Some(sname) = sname {
        // The frames are one string joined by "..": each is printed up to
        // the next separator.
        let mut i = 0;
        let mut rest = &sname[..];
        while !GOT_INT.get() {
            let next = rest.windows(2).position(|pair| pair == b"..");
            let frame = msg_bytes(&rest[..next.unwrap_or(rest.len())]);
            let at = max - i;
            if i == max - DEBUG_BACKTRACE_LEVEL.get() {
                smsg!(0, "->{at} {frame}");
            } else {
                smsg!(0, "  {at} {frame}");
            }
            i += 1;
            let Some(next) = next else {
                break;
            };
            rest = &rest[next + 2..];
        }
    }
    show_debug_line(cmd);
}

/// `:debug {cmd}`: run one command with the debugger stopping at everything.
pub fn ex_debug(excmd: &mut ExArg) {
    let outer_level = DEBUG_BREAK_LEVEL.replace(9999);
    let _ = do_cmdline_cmd(excmd.line.cstr_from(excmd.line.arg));
    DEBUG_BREAK_LEVEL.set(outer_level);
}

/// `:debuggreedy`, whose `0` argument turns it back off.
pub(crate) fn ex_debuggreedy(excmd: &mut ExArg) {
    DEBUG_GREEDY.set(excmd.addr_count == 0 || excmd.number2 != 0);
}
