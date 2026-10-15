//! `do_cmdline` — the loop that runs a sequence of Ex command lines.
//!
//! This is the re-entrant heart of the editor: a sourced file, `:execute`, a
//! `:global` body, an autocommand and a mapping all arrive here, nested
//! inside one another, each with its own conditional stack, its own share of
//! the exception state and its own store of the lines a `:while` replays.
//!
//! One pass of the loop -- [`Run::step`] -- runs one `|`-separated command.
//! Where the next one comes from is the first decision it makes, and there
//! are three answers: replay a stored line (inside a loop), ask the line
//! source ([`LineSource`]) for one, or take what `do_one_cmd` left after a
//! `|`. Ordering is
//! load-bearing throughout, and [`Run::keep_going`] is the specification of
//! when a script stops on an error.
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::ex_getln::ui_ext_cmdline_block_append;
use crate::message::emsg;
use crate::os::cshim::gettext;
use core::ffi::{CStr, c_int};

use crate::debugger::{dbg_breakpoint, dbg_find_breakpoint_named, do_debug};

use crate::ex_docmd::onecmd::do_one_cmd;
use crate::ex_docmd::source::{
    LineSource, LoopLines, ReadLines, SourceKind, StoredLine, do_cmdline_end, do_cmdline_start,
    handle_did_throw, msg_verbose_cmd, restore_dbg_stuff, save_dbg_stuff,
};
use crate::ex_docmd::{DoCmdOpts, sourcing_lnum};

use crate::ex_docmd::{PROF_YES, SavedDebugState};
use crate::ex_eval::flag::{CSTP_ERROR, CSTP_INTERRUPT, CSTP_THROW};
use crate::ex_eval::{CsFlags, CsLoopFlags};
use crate::ex_eval::{
    OwnedCondStack, PendingAction, PendingValue, aborting, cleanup_conditionals, do_errthrow,
    do_intthrow, has_loop_cmd, pop_msg_list, push_msg_list, report_pending, rewind_conditionals,
};

use crate::ex_getln::ui_ext_cmdline_block_leave;

use crate::debugger::state::{debug_break_level, debug_tick};
use crate::ex_docmd::state::{
    did_emsg_syntax, ex_nesting_level, last_cmdline, new_last_cmdline, repeat_cmdline,
};
use crate::ex_eval::state::{
    check_cstack, current_exception, did_endif, did_throw, force_abort, need_rethrow,
    suppress_errthrow, trylevel,
};
use crate::getchar::state::{KeyTyped, got_int};
use crate::guard::{Bump, Depth, MsgBump, Suppress};
use crate::message::state::{did_emsg, msg_didany, msg_didout, msg_scroll, need_wait_return};
use crate::message::{e_command_too_recursive, e_endfor, e_endif, e_endtry, e_endwhile};
use crate::option::vars::p_verbose;
use crate::profile::{do_profiling, func_line_end, func_line_start};

use crate::message::{msg_start, wait_return};

use crate::os::input::line_breakcheck;
use crate::profile::{script_line_end, script_line_start};
use crate::runtime::set_sourcing_lnum;

use crate::memory::XString;
use crate::types::ui::kUICmdline;
use crate::types::{CmdLine, CondId, CondSummary, Failed, LineNr, OptInt};
use crate::ui::ui_has;

/// Run one Ex command line, as if the user had typed it.
pub fn do_cmdline_cmd(cmd: &CStr) -> Result<(), Failed> {
    do_cmdline(
        Some(cmd.to_bytes()),
        None,
        DoCmdOpts::VERBOSE | DoCmdOpts::NOWAIT | DoCmdOpts::KEYTYPED,
    )
}

/// Where a `do_cmdline` run gets its lines, and which kind of source that
/// is -- asked once, since a run's source does not change under it.
struct Source<'s, 'a> {
    lines: &'s mut (dyn ReadLines + 'a),
    kind: SourceKind,
}

impl Source<'_, '_> {
    /// The body of a function?
    fn is_func(&self) -> bool {
        self.kind == SourceKind::Function
    }

    /// A sourced script?
    fn is_script(&self) -> bool {
        self.kind == SourceKind::Script
    }

    /// A line the user typed at the `:` prompt?
    fn is_typed(&self) -> bool {
        self.kind == SourceKind::Typed
    }

    /// A function this is the body of, declared `abort`?
    fn func_aborted(&self) -> bool {
        self.lines.function().is_some_and(|frame| frame.aborts())
    }

    /// The function this is the body of has returned, or (with no try
    /// conditional still open) aborted?
    fn func_ended(&self) -> bool {
        self.lines.function().is_some_and(|frame| frame.has_ended())
    }

    /// The sourced script hit a `:finish`?
    fn finished(&self) -> bool {
        self.lines.script().is_some_and(|cookie| cookie.finished)
    }

    /// The `:if`/`:while` nesting level the function or script started at.
    fn level(&self) -> Option<c_int> {
        match self.kind {
            SourceKind::Function => self.lines.function().map(|frame| frame.level + 1),
            SourceKind::Script => self.lines.script().map(|cookie| cookie.level),
            _ => None,
        }
    }

    /// Breakpoints added or removed since this one was read?
    fn stale_breakpoint(&mut self) -> bool {
        self.lines
            .debug()
            .is_some_and(|pos| pos.tick.get() != debug_tick.get())
    }

    /// Is the breakpoint due at or before `lnum`?
    fn breakpoint_due(&mut self, lnum: LineNr) -> bool {
        self.lines.debug().is_some_and(|pos| {
            let at = pos.breakpoint.get();
            at != 0 && at <= lnum
        })
    }

    /// Look up the next breakpoint at or after `after`, and stamp it with
    /// the tick it was read at.
    fn read_breakpoint(&mut self, after: LineNr) {
        let is_script = self.is_script();
        if let Some(pos) = self.lines.debug() {
            pos.breakpoint
                .set(dbg_find_breakpoint_named(is_script, pos.name, after));
            pos.tick.set(debug_tick.get());
        }
    }

    /// Stop at the breakpoint due at `lnum`, and look up the next one.
    fn stop_at_breakpoint(&mut self, lnum: LineNr) {
        if let Some(pos) = self.lines.debug() {
            dbg_breakpoint(pos.name, lnum);
        }
        self.read_breakpoint(lnum);
    }
}

/// The index of a stored line, `None` before the first.
fn stored_index(at: c_int) -> Option<usize> {
    usize::try_from(at).ok()
}

/// Take the next line of a `:while`/`:for` body being replayed -- a copy,
/// because the store keeps its line and the command modifies what it is
/// handed. `None` when the function returned or the sourced file hit
/// `:finish` while it was.
fn replay_stored_line(source: &mut Source, stored: &StoredLine) -> Option<CmdLine> {
    // Has the function returned, or (with no try conditional still open)
    // aborted?
    if source.is_func() {
        if do_profiling.get() == PROF_YES
            && let Some(frame) = source.lines.function()
        {
            func_line_end(frame);
        }
        if source.func_ended() {
            return None;
        }
    } else if do_profiling.get() == PROF_YES && source.is_script() {
        script_line_end();
    }

    // Has the sourced file hit a `:finish`?
    if source.finished() {
        return None;
    }

    // Breakpoints may have been added or removed since the last look.
    if source.stale_breakpoint() {
        source.read_breakpoint(sourcing_lnum());
    }

    set_sourcing_lnum(stored.lnum);

    if source.breakpoint_due(sourcing_lnum()) {
        source.stop_at_breakpoint(sourcing_lnum());
    }
    if do_profiling.get() == PROF_YES {
        if source.is_func() {
            if let Some(frame) = source.lines.function() {
                func_line_start(frame);
            }
        } else if source.is_script() {
            script_line_start();
        }
    }
    Some(CmdLine::from_bytes(&stored.line))
}

/// Ask the source for the next line. `None` at the end of input, which is
/// also how an aborted command line arrives -- the difference is only
/// whether a return has to be waited for.
fn ask_for_line(
    source: &mut Source,
    indent: c_int,
    count: c_int,
    flags: DoCmdOpts,
    did_block: &mut bool,
) -> Option<CmdLine> {
    if count == 1 && source.is_typed() {
        if ui_has(kUICmdline) {
            let last = last_cmdline.with(Clone::clone).unwrap_or_default();
            ui_ext_cmdline_block_append(0, &last);
            *did_block = true;
        }
        // The first line after an `:if` needs this, or the `:if` is
        // overwritten.
        msg_didout.set(true);
    }

    // A run with no source has only the line it was given.
    let line = match source.kind {
        SourceKind::None => None,
        _ => source.lines.next_line(c_int::from(b':'), indent, true),
    };
    let Some(line) = line else {
        // An aborted command line does not wait for a return. The end of a
        // sourced file or a function is not an abort and does not reach
        // here with 'KeyTyped' set.
        if KeyTyped.get() && !flags.has(DoCmdOpts::REPEAT) {
            need_wait_return.set(false);
        }
        return None;
    };

    // Every cmdline_block event but the first goes out immediately:
    // holding them until the commands have run would interleave them
    // wrongly with a nested command line.
    if ui_has(kUICmdline) && count > 0 && source.is_typed() {
        let shown = line.as_cstr().to_bytes();
        ui_ext_cmdline_block_append(usize::try_from(indent).unwrap_or(0), shown);
    }

    // Keep the first typed line for `.` to repeat; forget it as soon as a
    // second one is typed.
    if flags.has(DoCmdOpts::KEEPLINE) {
        repeat_cmdline.set((count == 0).then(|| XString::from_cstr(line.as_cstr())));
    }
    // Taken over: the source answers a block of its own.
    Some(CmdLine::from_vec(line.into_vec()))
}

/// One pass of the loop bookkeeping, after a command has run inside a
/// `:while` or `:for` (outside one it does nothing). `:endwhile`, `:endfor`
/// and `:continue` all land here:
/// commands that ran jump back to the `:while` or `:for`; ones that were
/// skipped have had the loop level decremented already.
fn advance_loop(
    source: &mut Source,
    cond: CondId,
    lines: &[StoredLine],
    current_line: &mut c_int,
    current_line_before: c_int,
) {
    // What the loop bookkeeping decided: nothing at all outside a loop,
    // `None` to carry on, or whether to jump back to the innermost loop's
    // first line (`Some(Some(line))`) or rewind the loop that just ended
    // (`Some(None)`).
    let decided = cond.with(|cs| {
        if cs.loop_level <= 0 {
            return Err(());
        }
        if !cs
            .loop_flags
            .has(CsLoopFlags::HAD_CONT | CsLoopFlags::HAD_ENDLOOP)
        {
            if cs.loop_flags.has(CsLoopFlags::HAD_LOOP) {
                // A `:while` or `:for` remembers where its body starts.
                cs.loop_flags.clear(CsLoopFlags::HAD_LOOP);
                let top = cs.top().expect("a loop level is open");
                cs.line[top] = current_line_before;
            }
            return Ok(None);
        }
        cs.loop_flags
            .clear(CsLoopFlags::HAD_CONT | CsLoopFlags::HAD_ENDLOOP);

        // Only a `:while` or `:for` level has a usable `line`; taking one
        // from any other kind would make `current_line` point outside the
        // stored lines.
        let Some(top) = cs.top() else {
            return Ok(None);
        };
        let flags = cs.flags[top];
        if did_emsg.get() == 0
            && !got_int.get()
            && !did_throw.get()
            && flags.has(CsFlags::LOOP)
            && cs.line[top] >= 0
            && flags.has(CsFlags::ACTIVE)
        {
            cs.loop_flags |= CsLoopFlags::HAD_LOOP;
            Ok(Some(Some(cs.line[top])))
        } else {
            Ok(Some(None))
        }
    });
    let Ok(decided) = decided else {
        return;
    };
    *current_line += 1;
    match decided {
        None => {}
        Some(Some(start)) => {
            *current_line = start;
            line_breakcheck();

            // The next breakpoint at or after the `:while`.
            if let Some(stored) = stored_index(*current_line).and_then(|at| lines.get(at)) {
                source.read_breakpoint(stored.lnum - 1);
            }
        }
        Some(None) => {
            // Only reachable from `:endwhile` or `:endfor`.
            let below = cond.with(|cs| cs.top().and_then(|top| top.checked_sub(1)));
            rewind_conditionals(cond, below, CsFlags::LOOP);
        }
    }
}

/// Rewind a conditional stack that still has entries when the run ends: a
/// sourced file or a function that finished with an `:if`, `:while`, `:for`
/// or `:try` still open. Reports the missing `:end…` where one really was,
/// then puts `trylevel` back after a `:finish`, a `:return` or that missing
/// `:endtry` -- a try block in its finally clause drops anything pending,
/// one in a catch clause finishes what it caught. Frees the `cs_forinfo`s.
fn unwind_conditionals(source: &Source, cond: CondId, initial_trylevel: c_int) {
    if !got_int.get()
        && !did_throw.get()
        && !aborting()
        && (source.is_script() && !source.finished() || source.is_func() && !source.func_ended())
    {
        let flags_here = cond.with(|cs| cs.top_flags());
        let missing = if flags_here.has(CsFlags::TRY) {
            e_endtry
        } else if flags_here.has(CsFlags::WHILE) {
            e_endwhile
        } else if flags_here.has(CsFlags::FOR) {
            e_endfor
        } else {
            e_endif
        };
        emsg(gettext(missing));
    }

    loop {
        // A try block that is not in its finally clause goes too.
        let keep = cleanup_conditionals(cond, CsFlags::NONE, true).and_then(|at| at.checked_sub(1));
        rewind_conditionals(cond, keep, CsFlags::LOOP);
        if cond.with(|cs| cs.idx < 0) {
            break;
        }
    }
    trylevel.set(initial_trylevel);
}

/// Hand what this run left back to the caller's conditional stack, and drop
/// out of the debugger's nesting level. This run's stack is about to go away:
/// an uncaught exception has to be rethrown against the caller's, and a
/// finished function or script may leave the caller's stack with finally
/// clauses to run -- `do_one_cmd` does both once it sees these flags.
fn leave_nesting(source: &Source) {
    if did_throw.get() {
        need_rethrow.set(true);
    }
    let deeper = source
        .level()
        .is_some_and(|level| ex_nesting_level.get() > level);
    if deeper {
        if !did_throw.get() {
            check_cstack.set(true);
        }
        return;
    }
    if source.is_func() {
        ex_nesting_level.set(ex_nesting_level.get() - 1);
    }
    // Single-stepping out of a function drops back into the debugger.
    if (source.is_script() || source.is_func()) && ex_nesting_level.get() < debug_break_level.get()
    {
        let what = if source.is_script() {
            c"End of sourced file"
        } else {
            c"End of function"
        };
        do_debug(gettext(what));
    }
}

/// How deep the current call is inside other [`do_cmdline`] calls: the
/// outermost one owns the "wait for return" bookkeeping.
static RECURSIVE: crate::global_cell::GlobalCell<c_int> = crate::global_cell::GlobalCell::new(0);

/// What one pass of the loop decided.
#[derive(PartialEq, Eq)]
enum Pass {
    Again,
    /// The run is over; `Run::retval` is its answer.
    Done,
}

/// Everything one [`do_cmdline`] run carries from one `|`-separated command
/// to the next. C kept these as a dozen locals of a 600-line function whose
/// loop body read and wrote all of them; naming the set is what lets a pass be
/// a method.
struct Run {
    /// The `:if`/`:while`/`:try` stack this run opens and closes, the body it
    /// is storing or replaying, and which line of it is next.
    cond_stack: OwnedCondStack,
    /// What the stack looked like after the last command settled -- all
    /// that changes it is a command -- so the loop's own questions about it
    /// are reads of a local.
    seen: CondSummary,
    lines: Vec<StoredLine>,
    current_line: c_int,
    /// Replaying the stored body, as the command being run was handed it.
    repeating: bool,
    /// The line being run, owned: `None` until one has been taken and once
    /// the last command consumed it. A command may replace the buffer under
    /// itself, so it is handed over to `do_one_cmd` and handed back.
    pending: Option<CmdLine>,
    /// How many commands have run, and whether a line came from the source.
    count: c_int,
    read_from_source: bool,
    /// The command runs inside the loop store, and where in the body it
    /// started.
    looping: bool,
    line_before: c_int,
    /// A `cmdline_block` UI event is open; what 'msg_didout' was before the
    /// first `msg_start`; the suppression held from it to the run's end.
    did_block: bool,
    msg_didout_before: bool,
    quiet_output: Option<(MsgBump, Bump)>,
    /// The `trylevel` this run started from, and its answer.
    initial_trylevel: c_int,
    retval: Result<(), Failed>,
}

impl Run {
    /// A run's starting state: no conditionals, no stored loop lines.
    fn new(first: Option<CmdLine>) -> Run {
        Run {
            cond_stack: OwnedCondStack::open(),
            seen: CondSummary::EMPTY,
            lines: Vec::new(),
            current_line: 0,
            repeating: false,
            pending: first,
            count: 0,
            read_from_source: false,
            looping: false,
            line_before: 0,
            did_block: false,
            msg_didout_before: false,
            quiet_output: None,
            initial_trylevel: trylevel.get(),
            retval: Ok(()),
        }
    }

    /// Where the next line comes from: replay a stored one, ask the source,
    /// or take what a `|` left. `Done` at the end of the input.
    fn take_line(&mut self, source: &mut Source, flags: DoCmdOpts) -> Pass {
        // Replaying a loop body: take the next stored line. Each
        // `|`-separated command was stored separately, so an `:endwhile`
        // can jump back to exactly one of them.
        let CondSummary {
            loop_level, idx, ..
        } = self.seen;
        let stored = stored_index(self.current_line).and_then(|at| self.lines.get(at));
        if loop_level > 0
            && let Some(stored) = stored
        {
            self.pending = None;
            let Some(line) = replay_stored_line(source, stored) else {
                self.retval = Err(Failed);
                return Pass::Done;
            };
            self.pending = Some(line);
        }

        if self.pending.is_none() {
            let indent = if idx < 0 { 0 } else { (idx + 1) * 2 };
            let Some(line) = ask_for_line(source, indent, self.count, flags, &mut self.did_block)
            else {
                self.retval = Err(Failed);
                return Pass::Done;
            };
            self.pending = Some(line);
            self.read_from_source = true;
        }
        Pass::Again
    }

    /// The line source one command is handed. Inside a loop -- or on a line
    /// that looks like it opens one -- the line is stored so it can be
    /// replayed and the command gets a source that stores and replays too,
    /// which is what lets a `:function` be defined inside a `:while`.
    fn command_source(&mut self) {
        let pending = self
            .pending
            .as_ref()
            .expect("`take_line` leaves a line to run");
        self.looping = self.seen.loop_level > 0 || has_loop_cmd(pending.tail(0));
        self.line_before = 0;
        if !self.looping {
            return;
        }
        let stored = c_int::try_from(self.lines.len()).unwrap_or(c_int::MAX);
        self.repeating = self.current_line < stored;
        if self.current_line == stored {
            self.lines.push(StoredLine {
                line: pending.rest_of(0).to_vec(),
                lnum: sourcing_lnum(),
            });
        }
        self.line_before = self.current_line;
    }

    /// Run one `|`-separated command and take what it left. The command
    /// can replace the line under itself (`%` and `#` expansion do), and
    /// `next` is `None` when nothing followed a `|`.
    fn run_one(&mut self, source: &mut Source, flags: DoCmdOpts) {
        let recursing = Depth::of(&RECURSIVE);
        let line = self.pending.take().expect("`take_line` leaves a line");
        let (cond, inactive) = (self.cond_stack.id(), self.seen.in_inactive);
        // Defining a function reads further lines through the loop's
        // source, which moves `current_line` on with them.
        let line = if self.looping {
            let mut looped = LineSource::Loop(LoopLines {
                store: &mut self.lines,
                current_line: &mut self.current_line,
                repeating: self.repeating,
                inner: &mut *source.lines,
            });
            do_one_cmd(line, flags, cond, inactive, Some(&mut looped))
        } else if source.kind == SourceKind::None {
            do_one_cmd(line, flags, cond, inactive, None)
        } else {
            do_one_cmd(line, flags, cond, inactive, Some(&mut *source.lines))
        };
        drop(recursing);

        if let Some(at) = line.next {
            // What follows the `|` becomes the whole of the line, in the
            // same allocation, for the next `do_one_cmd`.
            let mut line = line;
            line.restart_at(at);
            self.pending = Some(line);
        } else {
            self.pending = None;

            // Remember a typed command for the `:` register -- after
            // running it, so that `:@:` works.
            if source.is_typed()
                && let Some(line) = new_last_cmdline.take()
            {
                last_cmdline.set(Some(line));
            }
        }
    }

    /// What the conditional stack decides once the command has run.
    fn settle_conditionals(&mut self, source: &mut Source) {
        let (cond, before) = (self.cond_stack.id(), self.line_before);
        advance_loop(source, cond, &self.lines, &mut self.current_line, before);

        // What the stack says now, in one look: what a `:finally` that just
        // ran made pending, and what the loop asks until the next command.
        let (finally, seen) = cond.with(|cs| {
            let finally = cs.loop_flags.has(CsLoopFlags::HAD_FINA);
            cs.loop_flags.clear(CsLoopFlags::HAD_FINA);
            let pending = finally.then(|| cs.pending[cs.top().expect("the :try is open")]);
            (pending, cs.summary())
        });
        let try_level = seen.try_level;
        self.seen = seen;

        // Outside every loop, the stored lines are of no further use.
        if self.seen.loop_level == 0 {
            if let Some(last) = self.lines.last() {
                set_sourcing_lnum(last.lnum);
                self.lines.clear();
            }
            self.current_line = 0;
        }

        // A `:finally` makes 'did_emsg', 'got_int' and 'did_throw' pending
        // until the `:endtry`: reset them and mark the level active, so the
        // clause runs at all -- which includes the case where the
        // `:finally` itself noticed a missing `:endif`/`:endwhile`/`:endfor`.
        if let Some(pending) = finally {
            let carried = match current_exception.get() {
                Some(id) if did_throw.get() => PendingValue::Exception(id),
                _ => PendingValue::None,
            };
            report_pending(
                PendingAction::Made,
                pending & (CSTP_ERROR | CSTP_INTERRUPT | CSTP_THROW),
                carried,
            );
            did_emsg.set(0);
            got_int.set(false);
            did_throw.set(false);
            self.seen.in_inactive = cond.with(|cs| {
                let top = cs.top().expect("the :try is open");
                cs.flags[top] |= CsFlags::ACTIVE | CsFlags::FINALLY;
                cs.innermost_inactive()
            });
        }

        // The global `trylevel` is what a *nested* `do_cmdline` reads.
        trylevel.set(self.initial_trylevel + try_level);

        // The outermost try conditional -- across function calls and
        // sourced files -- aborting cancels everything. Leaving it normally
        // puts the non-exception abort behaviour back for the rest of the
        // script.
        if trylevel.get() == 0 && did_emsg.get() == 0 && !got_int.get() && !did_throw.get() {
            force_abort.set(false);
        }

        // A throw deactivates conditionals.
        if do_intthrow(cond) {
            self.seen.in_inactive = cond.with(|cs| cs.innermost_inactive());
        }
    }

    /// Keep going while nothing is aborting (or a try conditional is still
    /// open with finally clauses to run or an interrupt to catch), no error
    /// was reported against a *typed* line, and something is left to run.
    fn keep_going(&self, source: &Source, flags: DoCmdOpts) -> Pass {
        let CondSummary { try_level, idx, .. } = self.seen;
        let aborting_now =
            (got_int.get() || did_emsg.get() != 0 && force_abort.get() || did_throw.get())
                && try_level == 0;
        // Inside try/catch an error keeps going, so that it can be dealt
        // with -- unless it is a syntax error, which may make the `:endtry`
        // itself be missed.
        let typed_error = did_emsg.get() != 0
            && (try_level == 0 || did_emsg_syntax.get())
            && self.read_from_source
            && source.is_typed();
        let more_to_run = self.pending.is_some() || idx >= 0 || flags.has(DoCmdOpts::REPEAT);
        if aborting_now || typed_error || !more_to_run {
            Pass::Done
        } else {
            Pass::Again
        }
    }

    /// One `|`-separated command, line to verdict.
    fn step(&mut self, source: &mut Source, flags: DoCmdOpts) -> Pass {
        // Stop skipping commands after an error once every `:endif`,
        // `:endwhile` and `:endfor` has been passed.
        if self.pending.is_none()
            && !force_abort.get()
            && self.seen.idx < 0
            && !source.func_aborted()
        {
            did_emsg.set(0);
        }

        if self.take_line(source, flags) == Pass::Done {
            return Pass::Done;
        }

        self.command_source();
        did_endif.set(false);

        // Put all the output below each other without waiting for a return.
        // Not for commands from a script, and not for a recursive call
        // (`:e +command file`).
        if self.count == 0 && !flags.has(DoCmdOpts::NOWAIT) && RECURSIVE.get() == 0 {
            self.msg_didout_before = msg_didout.get();
            msg_didany.set(false);
            msg_start();
            msg_scroll.set(1);
            self.quiet_output = Some((Suppress::wait_return(), Suppress::redraw()));
        }
        self.count += 1;

        if p_verbose() >= 15 && crate::runtime::has_sourcing_name() || p_verbose() >= 16 as OptInt {
            let line = self
                .pending
                .as_ref()
                .map_or(&[][..], |line| line.rest_of(0));
            msg_verbose_cmd(sourcing_lnum(), line);
        }

        self.run_one(source, flags);

        // A function that an error did not abort keeps going.
        if did_emsg.get() != 0 && !force_abort.get() && source.is_func() && !source.func_aborted() {
            did_emsg.set(0);
        }

        self.settle_conditionals(source);
        self.keep_going(source, flags)
    }

    /// Ask for a return when too much output piled up to fit on the command
    /// line (with `:global`, once after the whole command). Runs *after* the
    /// message list is put back, so that what `wait_return` says reaches the
    /// caller's list and not this run's.
    fn report(&mut self) {
        if self.quiet_output.take().is_some() {
            msg_scroll.set(0);

            if self.retval.is_err() || did_endif.get() && KeyTyped.get() && did_emsg.get() == 0 {
                // A typed `:if`/`:else` that has just finished, or an error.
                need_wait_return.set(false);
                msg_didany.set(false);
            } else if need_wait_return.get() {
                // `msg_start` above cleared 'msg_didout'; the `wait_return`
                // here must not overwrite whatever was shown before it.
                msg_didout.set(msg_didout.get() || self.msg_didout_before);
                wait_return(0);
            }
        }

        if self.did_block {
            ui_ext_cmdline_block_leave();
        }
    }

    /// What the run puts back once its last command has run.
    fn close(&mut self, source: &Source, debug_saved: Option<SavedDebugState>) {
        self.pending = None;
        did_emsg_syntax.set(false);
        self.lines.clear();

        let cond = self.cond_stack.id();
        if cond.with(|cs| cs.idx >= 0) {
            unwind_conditionals(source, cond, self.initial_trylevel);
        }

        // A missing `:endtry`/`:endwhile`/`:endfor`/`:endif` reported above
        // becomes an exception now, after the stack has been rewound.
        do_errthrow(Some(cond), || source.is_func().then_some(c"endfunction"));

        if trylevel.get() == 0 {
            if did_throw.get() {
                // An exception thrown out of the outermost try conditional:
                // discard it, stop converting errors and interrupts to
                // exceptions, and run nothing more.
                handle_did_throw();
            } else if got_int.get() || did_emsg.get() != 0 && force_abort.get() {
                // An interrupt, or an aborting error that did not become an
                // exception. Errors stop being converted -- which is also
                // what lets the interrupt message through when 'force_abort'
                // is set and 'did_emsg' is not, after an error in a finally
                // clause.
                suppress_errthrow.set(true);
            }
        }

        leave_nesting(source);

        // After returning from the debugger, not before.
        if let Some(saved) = debug_saved {
            restore_dbg_stuff(saved);
        }
    }
}

/// Run Ex commands: `first` when given, then whatever `source` answers.
///
/// May be called recursively. `first` is the bytes up to its first NUL, if
/// it has one; it is copied before it runs, because a command modifies the
/// line it is handed. With no `source` the run has only `first`, and a
/// command that wants more of its body reads it from the command line.
/// Answers `Err` when the line could not be run.
pub(crate) fn do_cmdline(
    first: Option<&[u8]>,
    source: Option<&mut dyn ReadLines>,
    flags: DoCmdOpts,
) -> Result<(), Failed> {
    // Every do_cmdline/do_one_cmd pair gets its own place to store the
    // error messages an exception may be built from. Without that, the
    // `do_errthrow` in `do_one_cmd` would join an earlier invocation's
    // messages to a later invocation's command name — which is what
    // happens when a BufWritePost autocommand runs after a write error.
    push_msg_list();

    if do_cmdline_start().is_err() {
        emsg(gettext(e_command_too_recursive));
        // No command name: this is not an error of any one command, and
        // the throw goes through an empty stack of its own.
        let none = OwnedCondStack::open();
        do_errthrow(Some(none.id()), || None);
        drop(none);
        pop_msg_list();
        return Err(Failed);
    }

    let mut no_source = LineSource::None;
    let lines: &mut dyn ReadLines = match source {
        Some(lines) => lines,
        None => &mut no_source,
    };
    let kind = lines.kind();
    let mut source = Source { lines, kind };

    // Inside a function, use a higher nesting level.
    if let Some(frame) = source.lines.function()
        && source.is_func()
        && ex_nesting_level.get() == frame.level
    {
        ex_nesting_level.set(ex_nesting_level.get() + 1);
    }

    if RECURSIVE.get() == 0 {
        force_abort.set(false);
        suppress_errthrow.set(false);
    }

    let debug_saved = flags.has(DoCmdOpts::EXCRESET).then(save_dbg_stuff);

    did_throw.set(false);
    // An `emsg` cancels the whole command line and any conditional or
    // loop around it. With 'force_abort' set, everything is cancelled.
    did_emsg.set(0);

    // 'KeyTyped' is only set by `vgetc`; a sourced line never went
    // through it.
    if !flags.has(DoCmdOpts::KEYTYPED) && !source.is_typed() {
        KeyTyped.set(false);
    }

    let first = first.map(|line| {
        let end = line
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(line.len());
        CmdLine::from_bytes(&line[..end])
    });
    let mut run = Run::new(first);
    while run.step(&mut source, flags) == Pass::Again {}
    run.close(&source, debug_saved);
    pop_msg_list();
    run.report();

    // In case `do_cmdline` was used recursively.
    did_endif.set(false);
    do_cmdline_end();
    run.retval
}
