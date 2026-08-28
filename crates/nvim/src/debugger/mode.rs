//! Debug mode: the `>` prompt, the backtrace, and `:debug`/`:debuggreedy`.
//!
//! [`do_debug`] is entered from [`super::dbg_check_breakpoint`] when a
//! breakpoint was hit or when the last `>` command asked to stop at this
//! nesting level. It takes the screen over, reads `>` commands until one of
//! them resumes execution, and leaves `debug_break_level` set to whichever
//! depth should stop next.
//!
//! Anything typed at the prompt that is not one of the dozen `>` commands is
//! run as an ordinary Ex command and the prompt comes back, which is why the
//! parser here answers `None` rather than an error.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ex_docmd::DoCmdOpts;
use crate::guard::{Allow, Bump, Saved, Suppress};
use crate::types::{ExpandContext, NUL};

/// The editor state [`do_debug`] takes over while the `>` prompt is up, and
/// puts back on the way out. The prompt has to be *visible*, so silence and
/// redirection are off for its duration whatever the debugged code asked for.
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
    no_prompt: Bump,
    /// `msg_silent`, put back late with the rest of the message state.
    loud: Saved,
}

impl SavedState {
    fn enter() -> Self {
        // Do not redisplay the window, and do not wait for a return.
        let redraw_off = Suppress::redraw();
        let no_prompt = Suppress::wait_return();
        // The prompt has to be visible whatever the debugged code asked for.
        let loud = Allow::messages();
        let saved = Self {
            msg_scroll: msg_scroll.get(),
            state: State.get(),
            did_emsg: did_emsg.get(),
            cmd_silent: cmd_silent.get(),
            emsg_silent: emsg_silent.get(),
            redir_off: redir_off.get(),
            redraw_off,
            no_prompt,
            loud,
        };
        // An error from the debugged code is not ours.
        did_emsg.set(0);
        cmd_silent.set(false);
        emsg_silent.set(0);
        // Debug commands are not part of the redirected output.
        redir_off.set(true);
        State.set(MODE_NORMAL);
        debug_mode.set(true);
        saved
    }

    fn leave(self) {
        drop(self.redraw_off);
        drop(self.no_prompt);
        // SAFETY: no arguments; it only marks the grid dirty.
        unsafe { redraw_all_later(UPD_NOT_VALID) };
        need_wait_return.set(false);
        msg_scroll.set(self.msg_scroll);
        lines_left.set(Rows.get() - 1);
        State.set(self.state);
        debug_mode.set(false);
        did_emsg.set(self.did_emsg);
        cmd_silent.set(self.cmd_silent);
        drop(self.loud);
        emsg_silent.set(self.emsg_silent);
        redir_off.set(self.redir_off);
        // Print the banner again only after something else has been typed.
        debug_did_msg.set(true);
    }
}

/// Debug mode: repeatedly read an Ex command, until told to continue normal
/// execution.
///
/// # Safety
/// `cmd` must be the NUL-terminated command line about to be executed.
pub unsafe fn do_debug(cmd: *mut c_char) {
    let saved = SavedState::enter();
    // SAFETY: caller contract.
    unsafe { show_debug_banner(cmd) };
    unsafe { debug_prompt(cmd) };
    saved.leave();
}

/// What is printed on the way in: why we stopped, where, and on which line.
///
/// # Safety
/// As [`do_debug`].
unsafe fn show_debug_banner(cmd: *mut c_char) {
    if !debug_did_msg.get() {
        smsg!(0, "Entering Debug mode.  Type \"cont\" to continue.");
    }
    // A watch expression that just changed left both of its values here.
    // They are `typval_tostring` output -- bytes, not necessarily UTF-8 -- so
    // they go through vim's own printf rather than through `format_args!`.
    for (label, cell) in [
        (c"Oldval = \"%s\"", &debug_oldval),
        (c"Newval = \"%s\"", &debug_newval),
    ] {
        let text = cell.get();
        if text.is_null() {
            continue;
        }
        // SAFETY: `text` is the NUL-terminated string the cell owns, and the
        // message copies it before it is freed.
        unsafe { smsg_c!(0, label.as_ptr(), text) };
        unsafe { xfree(text.cast()) };
        cell.set(ptr::null_mut());
    }

    // SAFETY: `estack_sfile` hands back an owned NUL-terminated name or null,
    // and `msg` copies what it keeps.
    let sname = unsafe { estack_sfile(ESTACK_NONE) };
    if !sname.is_null() {
        unsafe { msg(sname, 0) };
    }
    unsafe { xfree(sname.cast()) };
    unsafe { show_debug_line(cmd) };
}

/// The `line N: <cmd>` / `cmd: <cmd>` line, which both the banner and
/// `>backtrace` end with.
///
/// # Safety
/// As [`do_debug`].
unsafe fn show_debug_line(cmd: *mut c_char) {
    let lnum = sourcing_lnum();
    // SAFETY: caller contract; the command line is arbitrary bytes, so it
    // goes through vim's printf verbatim rather than through `format_args!`.
    if lnum != 0 {
        unsafe { smsg_c!(0, c"line %ld: %s".as_ptr(), lnum as int64_t, cmd) };
    } else {
        unsafe { smsg_c!(0, c"cmd: %s".as_ptr(), cmd) };
    }
}

/// A command understood at the `>` prompt.
#[derive(Clone, Copy, PartialEq, Eq)]
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
///
/// # Safety
/// As [`do_debug`].
unsafe fn debug_prompt(cmd: *mut c_char) {
    /// The command last given, reused for a blank line. Static, so `>step`
    /// followed by three empty lines steps four times.
    static last_cmd: GlobalCell<Option<DebugCmd>> = GlobalCell::new(None);

    // These three outlive the iteration that sets them, exactly as upstream's
    // do: `:debuggreedy` can be typed at this very prompt, so a pass that
    // does not save the typeahead may still restore what an earlier one did.
    let mut typeaheadbuf = tasave_T::default();
    let mut typeahead_saved = false;
    let mut save_ignore_script = false;
    let mut cmdline: *mut c_char = ptr::null_mut();

    loop {
        msg_scroll.set(1);
        need_wait_return.set(false);

        // Read from the user, not from whatever a mapping or a script had
        // queued: swap in an empty typeahead buffer, drop `:normal`'s side
        // effects, and stop reading script input.
        let save_ex_normal_busy = ex_normal_busy.get();
        ex_normal_busy.set(0);
        if !debug_greedy.get() {
            // SAFETY: `typeaheadbuf` outlives the restore below.
            unsafe { save_typeahead(&raw mut typeaheadbuf) };
            typeahead_saved = true;
            save_ignore_script = ignore_script.get();
            ignore_script.set(true);
        }

        // Do not debug whatever reading the line itself runs -- an expression
        // mapping, for instance.
        let outer_level = debug_break_level.replace(-1);
        // SAFETY: the previous line is ours to free, and
        // `getcmdline_prompt` hands back an owned line or null.
        unsafe { xfree(cmdline.cast()) };
        cmdline = unsafe {
            getcmdline_prompt(
                '>' as c_int,
                ptr::null(),
                0,
                ExpandContext::Nothing,
                ptr::null(),
                Callback {
                    data: Callback_data {
                        funcref: ptr::null_mut(),
                    },
                    type_0: kCallbackNone,
                },
                false,
                ptr::null_mut(),
            )
        };
        debug_break_level.set(outer_level);

        if typeahead_saved {
            // SAFETY: paired with the `save_typeahead` above (or an earlier
            // pass's, per the note on the declaration).
            unsafe { restore_typeahead(&raw mut typeaheadbuf) };
            ignore_script.set(save_ignore_script);
        }
        ex_normal_busy.set(save_ex_normal_busy);

        cmdline_row.set(msg_row.get());
        // SAFETY: no argument.
        unsafe { msg_starthere() };

        if !cmdline.is_null() {
            // SAFETY: `cmdline` is the NUL-terminated line just read, and
            // `arg` stays inside it.
            let (line, arg) = unsafe {
                let head = skipwhite(cmdline);
                (CStr::from_ptr(head).to_bytes(), head)
            };
            // A blank line repeats: only a line with something on it decides
            // what `last_cmd` is.
            let mut arg = arg;
            if !line.is_empty() {
                match parse_debug_cmd(line) {
                    Some((parsed, end)) => {
                        last_cmd.set(Some(parsed));
                        // SAFETY: `end` is within `line`.
                        arg = unsafe { arg.add(end) };
                    }
                    None => last_cmd.set(None),
                }
            }

            if let Some(parsed) = last_cmd.get() {
                // SAFETY: `cmd` is the caller's, `arg` inside `cmdline`.
                if unsafe { run_debug_cmd(parsed, cmd, arg, &last_cmd) } {
                    continue;
                }
                // On the way out, the backtrace is back at the bottom.
                debug_backtrace_level.set(0);
                break;
            }

            // Not a debug command, so run it -- but do not debug it.
            let outer_level = debug_break_level.replace(-1);
            // SAFETY: `cmdline` is a NUL-terminated Ex command line.
            unsafe {
                do_cmdline(
                    cmdline,
                    Some(getexline as _),
                    NULL,
                    DoCmdOpts::VERBOSE | DoCmdOpts::EXCRESET,
                )
            };
            debug_break_level.set(outer_level);
        }
        lines_left.set(Rows.get() - 1);
    }

    // SAFETY: the last line read is ours.
    unsafe { xfree(cmdline.cast()) };
}

/// Act on one `>` command. True means "ask again" -- the stack-walking
/// commands do not resume execution.
///
/// # Safety
/// `cmd` is the debugged command line and `arg` points into the line just
/// read, both NUL-terminated.
unsafe fn run_debug_cmd(
    parsed: DebugCmd,
    cmd: *mut c_char,
    arg: *mut c_char,
    last_cmd: &GlobalCell<Option<DebugCmd>>,
) -> bool {
    match parsed {
        DebugCmd::Cont => debug_break_level.set(-1),
        DebugCmd::Next => debug_break_level.set(ex_nesting_level.get()),
        DebugCmd::Step => debug_break_level.set(9999),
        DebugCmd::Finish => debug_break_level.set(ex_nesting_level.get() - 1),
        DebugCmd::Quit => {
            got_int.set(true);
            debug_break_level.set(-1);
        }
        DebugCmd::Interrupt => {
            got_int.set(true);
            debug_break_level.set(9999);
            // `>interrupt` does not repeat on a blank line; keep stepping.
            last_cmd.set(Some(DebugCmd::Step));
        }
        DebugCmd::Backtrace => {
            // SAFETY: caller contract.
            unsafe { do_showbacktrace(cmd) };
            return true;
        }
        DebugCmd::Frame => {
            // SAFETY: caller contract.
            if unsafe { *arg } as c_int == NUL {
                unsafe { do_showbacktrace(cmd) };
            } else {
                unsafe { do_setdebugtracelevel(skipwhite(arg)) };
            }
            return true;
        }
        DebugCmd::Up => {
            debug_backtrace_level.set(debug_backtrace_level.get() + 1);
            do_checkbacktracelevel();
            return true;
        }
        DebugCmd::Down => {
            debug_backtrace_level.set(debug_backtrace_level.get() - 1);
            do_checkbacktracelevel();
            return true;
        }
    }
    false
}

/// How deep the execution stack is, read off `estack_sfile`'s `..`-joined
/// rendering of it.
///
/// # Safety
/// `sname` must be null or NUL-terminated.
unsafe fn get_maxbacktrace_level(sname: *mut c_char) -> c_int {
    if sname.is_null() {
        return 0;
    }
    // SAFETY: caller contract.
    let joined = unsafe { CStr::from_ptr(sname) }.to_bytes();
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
///
/// # Safety
/// `arg` must be NUL-terminated.
unsafe fn do_setdebugtracelevel(arg: *mut c_char) {
    // SAFETY: caller contract.
    let (level, relative) = unsafe { (atoi(arg), *arg as c_int == '+' as c_int) };
    if relative || level < 0 {
        debug_backtrace_level.set(debug_backtrace_level.get() + level);
    } else {
        debug_backtrace_level.set(level);
    }
    do_checkbacktracelevel();
}

/// Clamp the requested frame to one that exists, saying so when it moved.
fn do_checkbacktracelevel() {
    if debug_backtrace_level.get() < 0 {
        debug_backtrace_level.set(0);
        smsg!(0, "frame is zero");
        return;
    }
    // SAFETY: `estack_sfile` hands back an owned name or null.
    let max = unsafe {
        let sname = estack_sfile(ESTACK_NONE);
        let max = get_maxbacktrace_level(sname);
        xfree(sname.cast());
        max
    };
    if debug_backtrace_level.get() > max {
        debug_backtrace_level.set(max);
        smsg!(0, "frame at highest level: {max}");
    }
}

/// `>backtrace`: the execution stack, innermost last, with `->` on the frame
/// `>up`/`>down` have selected.
///
/// # Safety
/// As [`do_debug`].
unsafe fn do_showbacktrace(cmd: *mut c_char) {
    // SAFETY: `estack_sfile` hands back an owned NUL-terminated name or null.
    let sname = unsafe { estack_sfile(ESTACK_NONE) };
    let max = unsafe { get_maxbacktrace_level(sname) };
    if !sname.is_null() {
        // The frames are one string joined by "..", split in place: each
        // separator is blanked to print the frame, then put back.
        let mut i = 0;
        let mut cur = sname;
        while !got_int.get() {
            let next = unsafe { strstr(cur, c"..".as_ptr()) };
            if !next.is_null() {
                unsafe { *next = NUL as c_char };
            }
            let marker = if i == max - debug_backtrace_level.get() {
                c"->%d %s"
            } else {
                c"  %d %s"
            };
            unsafe { smsg_c!(0, marker.as_ptr(), max - i, cur) };
            i += 1;
            if next.is_null() {
                break;
            }
            unsafe { *next = '.' as c_char };
            cur = unsafe { next.offset(2) };
        }
        unsafe { xfree(sname.cast()) };
    }
    unsafe { show_debug_line(cmd) };
}

/// `:debug {cmd}`: run one command with the debugger stopping at everything.
///
/// # Safety
/// `eap` must be the live `exarg_T`.
pub unsafe fn ex_debug(eap: *mut exarg_T) {
    let outer_level = debug_break_level.replace(9999);
    // SAFETY: caller contract; `eap.arg` is the NUL-terminated argument.
    unsafe { do_cmdline_cmd((*eap).arg) };
    debug_break_level.set(outer_level);
}

/// `:debuggreedy`, whose `0` argument turns it back off.
///
/// # Safety
/// `eap` must be the live `exarg_T`.
pub unsafe fn ex_debuggreedy(eap: *mut exarg_T) {
    // SAFETY: caller contract.
    let (addr_count, line2) = unsafe { ((*eap).addr_count, (*eap).line2) };
    debug_greedy.set(addr_count == 0 || line2 != 0 as linenr_T);
}
