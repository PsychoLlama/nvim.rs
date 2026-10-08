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
//! getter for one, or take what `do_one_cmd` left after a `|`. Ordering is
//! load-bearing throughout, and [`Run::keep_going`] is the specification of
//! when a script stops on an error.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::ex_getln::ui_ext_cmdline_block_append;
use crate::message::emsg;
use crate::os::cshim::gettext;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::debugger::{dbg_breakpoint, dbg_find_breakpoint_named, do_debug};

use crate::eval::userfunc::{
    func_breakpoint, func_dbg_tick, func_has_abort, func_has_ended, func_level, func_name,
    get_func_line,
};

use crate::ex_docmd::onecmd::do_one_cmd;
use crate::ex_docmd::source::{
    do_cmdline_end, do_cmdline_start, get_loop_line, getline_cookie, handle_did_throw,
    msg_verbose_cmd, restore_dbg_stuff, save_dbg_stuff, store_loop_line,
};
use crate::ex_docmd::xfree;
use crate::ex_docmd::{DoCmdOpts, sourcing_entry, sourcing_lnum};

use crate::ex_docmd::{LoopCookie, PROF_YES, SavedDebugState, WhileCmd};
use crate::ex_eval::flag::{CSTP_ERROR, CSTP_INTERRUPT, CSTP_THROW};
use crate::ex_eval::{CsFlags, CsLoopFlags};
use crate::ex_eval::{
    OwnedCondStack, PendingAction, PendingValue, aborting, cleanup_conditionals, do_errthrow,
    do_intthrow, has_loop_cmd, pop_msg_list, push_msg_list, report_pending, rewind_conditionals,
};

use crate::ex_getln::{getexline, ui_ext_cmdline_block_leave};

use crate::debugger::state::{debug_break_level, debug_tick};
use crate::ex_docmd::state::{
    did_emsg_syntax, ex_nesting_level, last_cmdline, new_last_cmdline, repeat_cmdline,
};
use crate::ex_eval::state::{
    check_cstack, current_exception, did_endif, did_throw, force_abort, need_rethrow,
    suppress_errthrow, trylevel,
};
use crate::garray::{ga_clear, ga_init};
use crate::getchar::state::{KeyTyped, got_int};
use crate::guard::{Bump, Depth, MsgBump, Suppress};
use crate::message::state::{did_emsg, msg_didany, msg_didout, msg_scroll, need_wait_return};
use crate::message::{e_command_too_recursive, e_endfor, e_endif, e_endtry, e_endwhile};
use crate::option::vars::p_verbose;
use crate::profile::do_profiling;

use crate::message::{msg_start, wait_return};

use crate::os::input::line_breakcheck;
use crate::profile::{
    func_line_end_cookie, func_line_start_cookie, script_line_end, script_line_start,
};
use crate::runtime::{
    getsourceline, set_sourcing_lnum, source_breakpoint, source_dbg_tick, source_level,
};

use crate::memory::XString;
use crate::types::ui::kUICmdline;
use crate::types::{
    CmdLine, CondId, CondSummary, Failed, GArray, LineGetter, LineNr, OptInt, size_t,
};
use crate::ui::ui_has;

/// Free every line a `:while`/`:for` body stored, and the array holding
/// them.
///
/// # Safety
/// `gap` must point at a live growable array, unaliased for the call.
unsafe fn clear_loop_lines(gap: *mut GArray) {
    if !unsafe { (*gap).ga_data }.is_null() {
        for i in 0..unsafe { (*gap).ga_len } {
            let item = unsafe { ((*gap).ga_data as *mut WhileCmd).offset(i as isize) };
            unsafe { xfree((*item).line as *mut c_void) };
        }
    }
    unsafe { ga_clear(gap) };
}

/// Run one Ex command line, as if the user had typed it.
pub fn do_cmdline_cmd(cmd: &CStr) -> Result<(), Failed> {
    // SAFETY: a `CStr` is NUL-terminated, which is the whole contract;
    // `do_cmdline` reads the line and does not write through the pointer.
    unsafe {
        do_cmdline(
            cmd.as_ptr().cast_mut(),
            None,
            ptr::null_mut(),
            DoCmdOpts::VERBOSE | DoCmdOpts::NOWAIT | DoCmdOpts::KEYTYPED,
        )
    }
}

/// Where a `do_cmdline` run gets its lines, and what the debugger needs to
/// ask about them. A record of the caller's promise, not a proof of it:
/// `real_cookie` is `cookie` with the `:while` wrappers peeled off, and the
/// debugger fields are addresses inside whichever of a function or a script
/// this is -- all null when it is neither.
struct Source {
    fgetline: LineGetter,
    cookie: *mut c_void,
    real_cookie: *mut c_void,
    /// The name a breakpoint is looked up by, where it is kept, and the
    /// tick it was read at.
    fname: *mut c_char,
    breakpoint: *mut LineNr,
    dbg_tick: *mut c_int,
}

impl Source {
    /// The body of a function?
    fn is_func(&self) -> bool {
        getline_equal(self.fgetline, self.cookie, Some(get_func_line))
    }

    /// A sourced script?
    fn is_script(&self) -> bool {
        getline_equal(self.fgetline, self.cookie, Some(getsourceline))
    }

    /// A line the user typed at the `:` prompt?
    fn is_typed(&self) -> bool {
        getline_equal(self.fgetline, self.cookie, Some(getexline))
    }

    /// A function this is the body of, aborted by an error?
    fn func_aborted(&self) -> bool {
        self.is_func() && func_has_abort(self.real_cookie) != 0
    }

    /// Breakpoints added or removed since this one was read?
    fn stale_breakpoint(&self) -> bool {
        // SAFETY: the record's promise -- the debugger's own addresses.
        !self.breakpoint.is_null()
            && !self.dbg_tick.is_null()
            && unsafe { *self.dbg_tick } != debug_tick.get()
    }

    /// Is the breakpoint due at or before `lnum`?
    fn breakpoint_due(&self, lnum: LineNr) -> bool {
        // SAFETY: as [`Source::stale_breakpoint`].
        !self.breakpoint.is_null()
            && unsafe { *self.breakpoint } != 0
            && unsafe { *self.breakpoint } <= lnum
    }

    /// Look up the next breakpoint at or after `after`, and stamp it with
    /// the tick it was read at.
    fn read_breakpoint(&self, after: LineNr) {
        if self.breakpoint.is_null() || self.dbg_tick.is_null() {
            return;
        }
        // SAFETY: as [`Source::stale_breakpoint`].
        unsafe {
            *self.breakpoint =
                dbg_find_breakpoint_named(self.is_script(), CStr::from_ptr(self.fname), after);
            *self.dbg_tick = debug_tick.get();
        }
    }
}

/// Take the next line of a `:while`/`:for` body being replayed. `None` when
/// the function returned or the sourced file hit `:finish` while it was.
fn replay_stored_line(source: &Source, lines: &GArray, current_line: c_int) -> Option<Line> {
    // Has the function returned, or (with no try conditional still open)
    // aborted?
    if source.is_func() {
        if do_profiling.get() == PROF_YES {
            func_line_end_cookie(source.real_cookie);
        }
        if func_has_ended(source.real_cookie) != 0 {
            return None;
        }
    } else if do_profiling.get() == PROF_YES && source.is_script() {
        script_line_end();
    }

    // Has the sourced file hit a `:finish`?
    if source_finished(source.fgetline, source.cookie) {
        return None;
    }

    // Breakpoints may have been added or removed since the last look.
    if source.stale_breakpoint() {
        source.read_breakpoint(sourcing_lnum());
    }

    // SAFETY: `current_line` is an index into the stored body, which the
    // caller has just bounds-checked against `ga_len`.
    let stored = unsafe { (lines.ga_data as *mut WhileCmd).offset(current_line as isize) };
    let line = unsafe { (*stored).line };
    set_sourcing_lnum(unsafe { (*stored).lnum });

    if source.breakpoint_due(sourcing_lnum()) {
        // SAFETY: a due breakpoint means this is a function or a script, whose
        // name `fname` is.
        dbg_breakpoint(unsafe { CStr::from_ptr(source.fname) }, sourcing_lnum());
        source.read_breakpoint(sourcing_lnum());
    }
    if do_profiling.get() == PROF_YES {
        if source.is_func() {
            func_line_start_cookie(source.real_cookie);
        } else if source.is_script() {
            script_line_start();
        }
    }
    Some(Line(line))
}

/// Ask the line getter for the next line. `None` at the end of input, which
/// is also how an aborted command line arrives -- the difference is only
/// whether a return has to be waited for.
fn ask_for_line(
    source: &Source,
    indent: c_int,
    count: c_int,
    flags: DoCmdOpts,
    did_block: &mut bool,
) -> Option<Line> {
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

    // SAFETY: the record's promise -- `cookie` is `fgetline`'s payload.
    let line = match source.fgetline {
        Some(get) => unsafe { get(':' as c_int, source.cookie, indent, true) },
        None => ptr::null_mut(),
    };
    if line.is_null() {
        // An aborted command line does not wait for a return. The null
        // that ends a sourced file or a function is not an abort and does
        // not reach here with 'KeyTyped' set.
        if KeyTyped.get() && !flags.has(DoCmdOpts::REPEAT) {
            need_wait_return.set(false);
        }
        return None;
    }

    // Every cmdline_block event but the first goes out immediately:
    // holding them until the commands have run would interleave them
    // wrongly with a nested command line.
    if ui_has(kUICmdline) && count > 0 && source.is_typed() {
        // SAFETY: the line the getter just answered, NUL-terminated.
        ui_ext_cmdline_block_append(indent as size_t, unsafe { cstr::bytes_at(line) });
    }

    // Keep the first typed line for `.` to repeat; forget it as soon as a
    // second one is typed.
    if flags.has(DoCmdOpts::KEEPLINE) {
        // SAFETY: the line the getter just answered, NUL-terminated.
        repeat_cmdline.set((count == 0).then(|| XString::from_cstr(unsafe { cstr::at(line) })));
    }
    Some(Line(line))
}

/// One pass of the loop bookkeeping, after a command has run inside a
/// `:while` or `:for` (outside one it does nothing). `:endwhile`, `:endfor`
/// and `:continue` all land here:
/// commands that ran jump back to the `:while` or `:for`; ones that were
/// skipped have had the loop level decremented already.
fn advance_loop(
    source: &Source,
    cond: CondId,
    lines: &GArray,
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
            if !source.breakpoint.is_null() && lines.ga_len > *current_line {
                // SAFETY: `current_line` is an index into the stored body,
                // just bounds-checked against `ga_len`.
                let body = lines.ga_data as *mut WhileCmd;
                let at = unsafe { (*body.offset(*current_line as isize)).lnum } - 1;
                source.read_breakpoint(at);
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
        && (source.is_script() && !source_finished(source.fgetline, source.cookie)
            || source.is_func() && func_has_ended(source.real_cookie) == 0)
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
    // SAFETY: the record's promise -- the script's own frame.
    let deeper = source.is_script()
        && ex_nesting_level.get() > unsafe { source_level(source.real_cookie) }
        || source.is_func() && ex_nesting_level.get() > func_level(source.real_cookie) + 1;
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

/// A line to run: a newtype, so "no line" is `None` and no `*mut c_char`
/// reaches a return type.
struct Line(*mut c_char);

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
    /// is storing or replaying, which line is next, and the getter a command
    /// reads further ones through.
    cond_stack: OwnedCondStack,
    /// What the stack looked like after the last command settled -- all
    /// that changes it is a command -- so the loop's own questions about it
    /// are reads of a local.
    seen: CondSummary,
    lines: GArray,
    current_line: c_int,
    loop_cookie: LoopCookie,
    /// The line being run, owned: `None` until one has been taken and once
    /// the last command consumed it. A command may replace the buffer under
    /// itself, so it is handed over to `do_one_cmd` and handed back.
    pending: Option<CmdLine>,
    /// How many commands have run, and whether a line came from the getter.
    count: c_int,
    used_getline: bool,
    /// The storing getter is in use, and where in the body this started.
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
        let mut lines: GArray = unsafe { core::mem::zeroed() };
        unsafe { ga_init(&raw mut lines, size_of::<WhileCmd>() as c_int, 10) };
        Run {
            cond_stack: OwnedCondStack::open(),
            seen: CondSummary::EMPTY,
            lines,
            current_line: 0,
            // SAFETY: `LoopCookie` is a `repr(C)` aggregate of scalars and
            // pointers; all-zero is a valid value of every one of them, and
            // every field is written before the cookie is handed out.
            loop_cookie: unsafe { core::mem::zeroed() },
            pending: first,
            count: 0,
            used_getline: false,
            looping: false,
            line_before: 0,
            did_block: false,
            msg_didout_before: false,
            quiet_output: None,
            initial_trylevel: trylevel.get(),
            retval: Ok(()),
        }
    }

    /// Where the next line comes from: replay a stored one, ask the getter,
    /// or take what a `|` left. `Done` at the end of the input.
    fn take_line(&mut self, source: &Source, flags: DoCmdOpts) -> Pass {
        // Replaying a loop body: take the next stored line. Each
        // `|`-separated command was stored separately, so an `:endwhile`
        // can jump back to exactly one of them. A *copy*, because the store
        // keeps its line and the command modifies what it is handed.
        let CondSummary {
            loop_level, idx, ..
        } = self.seen;
        if loop_level > 0 && self.current_line < self.lines.ga_len {
            self.pending = None;
            let Some(Line(line)) = replay_stored_line(source, &self.lines, self.current_line)
            else {
                self.retval = Err(Failed);
                return Pass::Done;
            };
            // SAFETY: the store's line, NUL-terminated.
            self.pending = Some(CmdLine::from_bytes(unsafe { cstr::bytes_at(line) }));
        }

        if self.pending.is_none() {
            let indent = if idx < 0 { 0 } else { (idx + 1) * 2 };
            let Some(Line(line)) =
                ask_for_line(source, indent, self.count, flags, &mut self.did_block)
            else {
                self.retval = Err(Failed);
                return Pass::Done;
            };
            // Taken over: every line getter answers a block its caller owns.
            // SAFETY: as above -- an `xmalloc` block.
            self.pending = Some(CmdLine::from_vec(
                unsafe { XString::from_raw(line) }.into_vec(),
            ));
            self.used_getline = true;
        }
        Pass::Again
    }

    /// The line source one command is handed. Inside a loop -- or on a line
    /// that looks like it opens one -- the line is stored so it can be
    /// replayed and the command gets a getter that stores and replays too,
    /// which is what lets a `:function` be defined inside a `:while`.
    fn command_source(&mut self, source: &Source) {
        let pending = self
            .pending
            .as_mut()
            .expect("`take_line` leaves a line to run");
        self.looping = self.seen.loop_level > 0 || has_loop_cmd(pending.tail(0));
        let line = pending.ptr_at(0);
        self.line_before = 0;
        if !self.looping {
            return;
        }
        self.loop_cookie.lines_gap = &raw mut self.lines;
        self.loop_cookie.current_line = self.current_line;
        self.loop_cookie.lc_getline = source.fgetline;
        self.loop_cookie.cookie = source.cookie;
        self.loop_cookie.repeating = (self.current_line < self.lines.ga_len) as c_int;
        if self.current_line == self.lines.ga_len {
            unsafe { store_loop_line(&raw mut self.lines, line) };
        }
        self.line_before = self.current_line;
    }

    /// Run one `|`-separated command and take what it left. `copy` can move
    /// under the call (`%` and `#` expansion reallocate it) and the answer
    /// is null when nothing followed a `|`.
    fn run_one(&mut self, source: &Source, flags: DoCmdOpts) {
        let (cmd_getline, cmd_cookie) = if self.looping {
            (
                Some(get_loop_line as _),
                (&raw mut self.loop_cookie).cast::<c_void>(),
            )
        } else {
            (source.fgetline, source.cookie)
        };
        let recursing = Depth::of(&RECURSIVE);
        let line = self.pending.take().expect("`take_line` leaves a line");
        let (cond, inactive) = (self.cond_stack.id(), self.seen.in_inactive);
        // SAFETY: the line source is the one `command_source` chose.
        let line = unsafe { do_one_cmd(line, flags, cond, inactive, cmd_getline, cmd_cookie) };
        drop(recursing);

        if self.looping {
            // Defining a function reads further lines through the loop
            // cookie, so take the line number back from it.
            self.current_line = self.loop_cookie.current_line;
        }

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
    fn settle_conditionals(&mut self, source: &Source) {
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
            if self.lines.ga_len > 0 {
                // SAFETY: `lines` is this run's own store, `ga_len` long.
                let body = self.lines.ga_data as *mut WhileCmd;
                let last = unsafe { body.add(self.lines.ga_len as usize - 1) };
                set_sourcing_lnum(unsafe { (*last).lnum });
                unsafe { clear_loop_lines(&raw mut self.lines) };
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
            && self.used_getline
            && source.is_typed();
        let more_to_run = self.pending.is_some() || idx >= 0 || flags.has(DoCmdOpts::REPEAT);
        if aborting_now || typed_error || !more_to_run {
            Pass::Done
        } else {
            Pass::Again
        }
    }

    /// One `|`-separated command, line to verdict.
    fn step(&mut self, source: &Source, flags: DoCmdOpts) -> Pass {
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

        self.command_source(source);
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

        if p_verbose() >= 15 && !sourcing_entry().es_name.is_null() || p_verbose() >= 16 as OptInt {
            let line = self
                .pending
                .as_mut()
                .map_or(ptr::null_mut(), |l| l.ptr_at(0));
            // SAFETY: `line` is this run's NUL-terminated line.
            unsafe { msg_verbose_cmd(sourcing_lnum(), line) };
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
    fn close(&mut self, source: &Source, flags: DoCmdOpts, debug_saved: &mut SavedDebugState) {
        self.pending = None;
        did_emsg_syntax.set(false);
        // SAFETY: `lines` is this run's own store.
        unsafe { clear_loop_lines(&raw mut self.lines) };

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
        if flags.has(DoCmdOpts::EXCRESET) {
            // SAFETY: the caller's own saved state.
            unsafe { restore_dbg_stuff(&raw mut *debug_saved) };
        }
    }
}

/// [`do_cmdline`] of `line`, which must end in a NUL, reading any further
/// lines from wherever the running command `excmd` reads its own -- which is
/// how `:execute` runs what it built.
pub(crate) fn do_cmdline_as(
    excmd: &crate::types::ExArg,
    line: &mut [u8],
    flags: DoCmdOpts,
) -> Result<(), Failed> {
    assert_eq!(line.last(), Some(&0), "a command line is NUL-terminated");
    // SAFETY: `line` is terminated and this call's to lend; a running
    // command's line getter and its cookie are a live pair.
    unsafe {
        do_cmdline(
            line.as_mut_ptr().cast(),
            excmd.ea_getline,
            excmd.cookie,
            flags,
        )
    }
}

/// [`do_cmdline`] of `line`, reading any further lines the way the `:`
/// prompt does: what is typed at the debugger's `>` prompt runs this way.
pub(crate) fn do_cmdline_typed(line: &CStr, flags: DoCmdOpts) -> Result<(), Failed> {
    // SAFETY: `do_cmdline` copies the line before running it, and the typed
    // line's getter takes no cookie.
    unsafe {
        do_cmdline(
            line.as_ptr().cast_mut(),
            Some(getexline as _),
            ptr::null_mut(),
            flags,
        )
    }
}

/// [`do_cmdline`] with no first line, reading every line from `getter`.
///
/// Safe where `do_cmdline` is not: `getter` is a safe function, so no cookie
/// it is handed can make it misbehave -- a function body's getter resolves its
/// cookie as an id and checks it.
pub(crate) fn do_cmdline_getter(
    getter: fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
    cookie: *mut c_void,
    flags: DoCmdOpts,
) -> Result<(), Failed> {
    // SAFETY: no first line, and a safe getter whatever the cookie.
    unsafe { do_cmdline(ptr::null_mut(), Some(getter), cookie, flags) }
}

/// Run Ex commands, from `cmdline` and then from `fgetline`.
///
/// May be called recursively. Answers `Err` when the line could not be run.
///
/// # Safety
/// `cmdline` must point at a NUL-terminated string, unaliased for the call,
/// and `cookie` be `fgetline`'s payload, live for the call.
pub unsafe fn do_cmdline(
    cmdline: *mut c_char,
    fgetline: LineGetter,
    cookie: *mut c_void,
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

    // Where this run's lines come from, and where the debugger keeps this
    // function's or script's breakpoint.
    // SAFETY: the caller's promise, and the accessors read that payload.
    let real_cookie = unsafe { getline_cookie(fgetline, cookie) };
    let (mut fname, mut breakpoint, mut dbg_tick) =
        (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
    if getline_equal(fgetline, cookie, Some(get_func_line)) {
        fname = func_name(real_cookie);
        breakpoint = func_breakpoint(real_cookie);
        dbg_tick = func_dbg_tick(real_cookie);
    } else if getline_equal(fgetline, cookie, Some(getsourceline)) {
        fname = sourcing_entry().es_name;
        breakpoint = unsafe { source_breakpoint(real_cookie) };
        dbg_tick = unsafe { source_dbg_tick(real_cookie) };
    }
    let source = Source {
        fgetline,
        cookie,
        real_cookie,
        fname,
        breakpoint,
        dbg_tick,
    };
    let source = &source;

    // Inside a function, use a higher nesting level.
    if source.is_func() && ex_nesting_level.get() == func_level(source.real_cookie) {
        ex_nesting_level.set(ex_nesting_level.get() + 1);
    }

    if RECURSIVE.get() == 0 {
        force_abort.set(false);
        suppress_errthrow.set(false);
    }

    // SAFETY: `debug_saved` is this frame's own.
    let mut debug_saved = SavedDebugState::new();
    if flags.has(DoCmdOpts::EXCRESET) {
        unsafe { save_dbg_stuff(&raw mut debug_saved) };
    }

    did_throw.set(false);
    // An `emsg` cancels the whole command line and any conditional or
    // loop around it. With 'force_abort' set, everything is cancelled.
    did_emsg.set(0);

    // 'KeyTyped' is only set by `vgetc`; a sourced line never went
    // through it.
    if !flags.has(DoCmdOpts::KEYTYPED) && !source.is_typed() {
        KeyTyped.set(false);
    }

    // A line the caller gave is copied: it is about to be modified in
    // place, and the caller still owns what it handed over.
    // SAFETY: the caller's promise -- a NUL-terminated string, or null.
    let first =
        (!cmdline.is_null()).then(|| CmdLine::from_bytes(unsafe { cstr::bytes_at(cmdline) }));
    let mut run = Run::new(first);
    while run.step(source, flags) == Pass::Again {}
    run.close(source, flags, &mut debug_saved);
    pop_msg_list();
    run.report();

    // In case `do_cmdline` was used recursively.
    did_endif.set(false);
    do_cmdline_end();
    run.retval
}

/// `getline_equal()`, checked.
fn getline_equal(fgetline: LineGetter, cookie: *mut c_void, func: LineGetter) -> bool {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_docmd::source::getline_equal(fgetline, cookie, func) }
}

/// `source_finished()`, checked.
fn source_finished(fgetline: LineGetter, cookie: *mut c_void) -> bool {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::runtime::source_finished(fgetline, cookie) }
}
