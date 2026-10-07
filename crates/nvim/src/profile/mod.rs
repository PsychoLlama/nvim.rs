//! Profiling: the `ProfTime` arithmetic shared by `:profile`, `reltime()`
//! and regex/search timeouts, the `:profile` command, and the per-line
//! accounting the profiled scripts and functions keep.
//!
//! A `ProfTime` is a `u64` nanosecond reading from `os_hrtime`. Durations
//! are unsigned differences and may wrap when a "later" time is subtracted
//! from an "earlier" one; [`profile_signed`] recovers the signed value
//! (#10452), and everything user-visible funnels through it.
//!
//! | file | what |
//! | --- | --- |
//! | this one | the arithmetic, `:profile`, and the accounting hooks the interpreter calls per line and per call |
//! | [`report`] | what `:profile dump` writes |
//! | [`startuptime`] | the `--startuptime` log |

// No `forbid(unsafe_code)` here: it would reach `startuptime`, whose log
// still writes through the C stdio it shares with the rest of startup.
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

pub mod report;
pub mod startuptime;

// The report and the startuptime log were split out of this file; callers
// name them where they have always been named.
pub use report::profile_dump;
pub use startuptime::{time_finish, time_init, time_msg, time_push, time_start};
pub(crate) use startuptime::{time_msg_at, time_pop};

use crate::charset::skip;
use crate::debugger::ex_breakadd;
use crate::eval::userfunc::{all_funcs, cookie_funccall, current_fc};
use crate::eval::vars::set_vim_var_nr;
use crate::global_cell::GlobalCell;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::os::env::expand_env_save_opt_of;
use crate::os::fs::CFile;
use crate::os::time::os_hrtime;
use crate::runtime::state::current_sctx;
use crate::runtime::{script_count, script_id_valid, with_script_item};
use crate::types::Candidate;
use crate::types::{
    ExArg, Expand, ExpandContext, FuncCall, LineNr, ProfTime, ScriptItem, SnPrl, UserFunc,
    VarNumber, Vv, int64_t,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use std::ffi::CString;
use std::rc::Rc;
pub(crate) static do_profiling: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
/// The `--startuptime` log, open from `time_init` to `time_finish`.
pub(crate) static time_fd: GlobalCell<Option<CFile>> = GlobalCell::new(None);

/// Whether `--startuptime` is logging.
pub(crate) fn startup_timing() -> bool {
    time_fd.with(Option::is_some)
}

/// `do_profiling` states (a `GlobalCell<c_int>` in main).
pub const PROF_NONE: c_int = 0;
pub const PROF_YES: c_int = 1;
pub const PROF_PAUSED: c_int = 2;

/// Accumulated time the user kept the editor waiting (input, `:profile
/// pause`); subtracted from measurements via [`profile_sub_wait`].
static PROF_WAIT_TIME: GlobalCell<ProfTime> = GlobalCell::new(0);
/// Report path from `:profile start {fname}`; `None` when not profiling.
static PROFILE_FNAME: GlobalCell<Option<CString>> = GlobalCell::new(None);

// ---------------------------------------------------------------------------
// Time arithmetic.

/// The current time.
pub fn profile_start() -> ProfTime {
    os_hrtime()
}

/// Elapsed time from `tm` until now.
pub fn profile_end(tm: ProfTime) -> ProfTime {
    profile_sub(profile_start(), tm)
}

/// The zero time.
pub fn profile_zero() -> ProfTime {
    0
}

/// The time `msec` milliseconds into the future, or the zero time ("no
/// limit") when `msec <= 0`.
pub fn profile_setlimit(msec: int64_t) -> ProfTime {
    if msec <= 0 {
        return profile_zero();
    }
    // `msec` is user input -- `search()`, `searchpair()` and `matchfuzzy()`
    // all take a `{timeout}` and nothing on the way here bounds it. Upstream
    // asserts the range, which aborts a debug build and, compiled out,
    // multiplies into an overflow instead. Saturate: an absurd timeout means
    // "as far into the future as a limit can mean". The ceiling is
    // `INT64_MAX` nanoseconds -- ~292 years -- because that is how far apart
    // [`profile_cmp`] can still tell two times, and the wrapping add past it
    // is the arithmetic this module is built on.
    let nsec = msec
        .cast_unsigned()
        .saturating_mul(1_000_000)
        .min(int64_t::MAX.cast_unsigned());
    profile_start().wrapping_add(nsec)
}

/// Whether the current time is past `tm`. False if the limit was never set
/// (`tm` is the zero time).
pub fn profile_passed_limit(tm: ProfTime) -> bool {
    if tm == 0 {
        return false;
    }
    profile_cmp(profile_start(), tm) < 0
}

/// `tm / count` (rounded), or zero when `count <= 0`.
pub fn profile_divide(tm: ProfTime, count: c_int) -> ProfTime {
    if count <= 0 {
        return profile_zero();
    }
    // The quotient is never negative; one past `i64::MAX` nanoseconds (292
    // years) saturates there rather than at `u64::MAX`.
    crate::narrow::float_as_i64((tm as f64 / f64::from(count)).round()).cast_unsigned()
}

pub fn profile_add(tm1: ProfTime, tm2: ProfTime) -> ProfTime {
    tm1.wrapping_add(tm2)
}

/// `tm1 - tm2`, wrapping when `tm2 > tm1`; see [`profile_signed`].
pub fn profile_sub(tm1: ProfTime, tm2: ProfTime) -> ProfTime {
    tm1.wrapping_sub(tm2)
}

/// Self time: `self + total - children`, or `self` unchanged when `total <=
/// children` (possible with recursive calls).
pub fn profile_self(self_: ProfTime, total: ProfTime, children: ProfTime) -> ProfTime {
    if total <= children {
        return self_;
    }
    profile_sub(profile_add(self_, total), children)
}

/// `tma` minus the wait time accumulated since the [`PROF_WAIT_TIME`]
/// snapshot `tm`.
pub(crate) fn profile_sub_wait(tm: ProfTime, tma: ProfTime) -> ProfTime {
    let waited = profile_sub(PROF_WAIT_TIME.get(), tm);
    profile_sub(tma, waited)
}

/// Signed value of a duration produced by [`profile_sub`]. Values above
/// `i64::MAX` (>=150 years) are taken to be wrapped negative differences.
pub fn profile_signed(tm: ProfTime) -> int64_t {
    if tm <= int64_t::MAX.cast_unsigned() {
        tm.cast_signed()
    } else {
        -(ProfTime::MAX - tm).cast_signed()
    }
}

/// Compare two times (which must be less than 150 years apart): negative
/// when `tm2 < tm1`, `0` when equal, positive when `tm2 > tm1`.
pub fn profile_cmp(tm1: ProfTime, tm2: ProfTime) -> c_int {
    if tm1 == tm2 {
        return 0;
    }
    if profile_signed(tm2.wrapping_sub(tm1)) < 0 {
        -1
    } else {
        1
    }
}

/// `tm` as `"%10.6lf"` seconds, the format used throughout the report and
/// by `reltimestr()`.
pub fn profile_msg_str(tm: ProfTime) -> String {
    format!("{:10.6}", profile_signed(tm) as f64 / 1e9)
}

/// C-string flavor of [`profile_msg_str`] for the transpiled callers
/// (syntime report, `reltimestr()`), in its own storage. Upstream answers a
/// static buffer the next call overwrites.
pub(crate) fn profile_msg(tm: ProfTime) -> [c_char; 50] {
    let s = profile_msg_str(tm);
    let mut buf = [0 as c_char; 50];
    let n = s.len().min(buf.len() - 1);
    for (dst, src) in buf.iter_mut().zip(s.as_bytes()[..n].iter()) {
        *dst = c_char::from_ne_bytes([*src]);
    }
    buf[n] = 0;
    buf
}

// ---------------------------------------------------------------------------
// The :profile command.

/// `:profile cmd args`. In the ex_docmd command table.
pub fn ex_profile(excmd: &mut ExArg) {
    /// Time at which `:profile pause` stopped the clock.
    static PAUSE_TIME: GlobalCell<ProfTime> = GlobalCell::new(0);

    let arg = excmd.line.arg;
    let end = excmd.line.skip_to_white(arg);
    let subcmd = excmd.line.slice_at(arg, end - arg);
    let full = excmd.line.arg();
    let e = excmd.line.skip_white(end);

    if subcmd == b"start" && excmd.line.byte_at(e) != 0 {
        let fname = expand_env_save_opt_of(excmd.line.cstr_from(e), true);
        PROFILE_FNAME.set(Some(fname.as_cstr().to_owned()));
        do_profiling.set(PROF_YES);
        PROF_WAIT_TIME.set(profile_zero());
        set_vim_var_nr(Vv::Profiling, 1 as VarNumber);
    } else if do_profiling.get() == PROF_NONE {
        emsg(gettext(c"E750: First use \":profile start {fname}\""));
    } else if full == b"stop" {
        profile_dump();
        do_profiling.set(PROF_NONE);
        set_vim_var_nr(Vv::Profiling, 0 as VarNumber);
        profile_reset();
    } else if full == b"pause" {
        if do_profiling.get() == PROF_YES {
            PAUSE_TIME.set(profile_start());
        }
        do_profiling.set(PROF_PAUSED);
    } else if full == b"continue" {
        if do_profiling.get() == PROF_PAUSED {
            let paused = profile_end(PAUSE_TIME.get());
            PROF_WAIT_TIME.set(profile_add(PROF_WAIT_TIME.get(), paused));
        }
        do_profiling.set(PROF_YES);
    } else if full == b"dump" {
        profile_dump();
    } else {
        // The rest ("func", "file") is parsed like ":breakadd".
        // SAFETY: the caller's ex command.
        ex_breakadd(excmd);
    }
}

/// Forget all profiling information (`:profile stop`).
fn profile_reset() {
    for id in 1..=script_count() {
        with_script_item(id, |si| {
            if si.sn_prof_on {
                si.sn_prof_on = false;
                si.sn_pr_force = false;
                si.sn_pr_child = profile_zero();
                si.sn_pr_nest = 0;
                si.sn_pr_count = 0;
                si.sn_pr_total = profile_zero();
                si.sn_pr_self = profile_zero();
                si.sn_pr_start = profile_zero();
                si.sn_pr_children = profile_zero();
                si.sn_prl_ga = Vec::new();
                si.sn_prl_start = profile_zero();
                si.sn_prl_children = profile_zero();
                si.sn_prl_wait = profile_zero();
                si.sn_prl_idx = -1;
                si.sn_prl_execed = 0;
            }
        });
    }
    for func in profiled_functions() {
        let mut prof = func.prof.borrow_mut();
        prof.profiling = false;
        prof.tm_count = 0;
        prof.tm_total = profile_zero();
        prof.tm_self = profile_zero();
        prof.tm_children = profile_zero();
        prof.tml_count.fill(0);
        prof.tml_total.fill(profile_zero());
        prof.tml_self.fill(profile_zero());
        prof.tml_start = profile_zero();
        prof.tml_children = profile_zero();
        prof.tml_wait = profile_zero();
        prof.tml_idx = -1;
        prof.tml_execed = false;
    }
    PROFILE_FNAME.set(None);
}

const PEXPAND_CMDS: [&[u8]; 7] = [
    b"continue\0",
    b"dump\0",
    b"file\0",
    b"func\0",
    b"pause\0",
    b"start\0",
    b"stop\0",
];

/// expand_generic callback for `:profile` subcommands (fn pointer in the
/// cmdexpand context table).
pub fn get_profile_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    let name = PEXPAND_CMDS.get(idx)?;
    Some(Candidate::Borrowed(
        CStr::from_bytes_with_nul(name).expect("a NUL-terminated literal"),
    ))
}

/// Command-line completion context for `:profile`, whose argument starts at
/// `arg` in the completion's line.
pub fn set_context_in_profile_cmd(expand: &mut Expand, arg: usize) {
    // Default: expand subcommands.
    expand.context = ExpandContext::Profile;
    expand.pattern = arg;

    let line = expand.line_cstr().to_bytes();
    let tail = line.get(arg..).unwrap_or_default();
    let end_subcmd = skip::to_white(tail);
    if end_subcmd == tail.len() {
        return;
    }
    let subcmd = &tail[..end_subcmd];
    let rest = arg + end_subcmd + skip::white(&tail[end_subcmd..]);
    if subcmd == b"start" || subcmd == b"file" {
        expand.context = ExpandContext::Files;
        expand.pattern = rest;
    } else if subcmd == b"func" {
        expand.context = ExpandContext::UserFunc;
        expand.pattern = rest;
    } else {
        expand.context = ExpandContext::Nothing;
    }
}

// ---------------------------------------------------------------------------
// Wait time.

/// When the editor started waiting for the user to type.
static INPUT_WAIT_START: GlobalCell<ProfTime> = GlobalCell::new(0);

/// Called when starting to wait for the user to type a character.
pub fn prof_input_start() {
    INPUT_WAIT_START.set(profile_start());
}

/// Called when finished waiting for the user to type a character.
pub fn prof_input_end() {
    let waited = profile_end(INPUT_WAIT_START.get());
    PROF_WAIT_TIME.set(profile_add(PROF_WAIT_TIME.get(), waited));
}

// ---------------------------------------------------------------------------
// Function profiling.

/// Whether a function defined in the current script should be profiled
/// (the script was targeted by `:profile file` with `!`-forcing).
pub fn prof_def_func() -> bool {
    let sid = current_sctx.get().sc_sid;
    sid > 0 && with_script_item(sid, |si| si.sn_pr_force)
}

/// Start profiling function `func`, sizing its per-line counters to its
/// body on first use.
pub fn func_do_profile(func: &UserFunc) {
    // Avoid zero-length counters.
    let len = func.body().lines.len().max(1);
    let mut prof = func.prof.borrow_mut();
    if !prof.initialized {
        prof.tm_count = 0;
        prof.tm_self = profile_zero();
        prof.tm_total = profile_zero();
        if prof.tml_count.is_empty() {
            prof.tml_count = vec![0; len];
        }
        if prof.tml_total.is_empty() {
            prof.tml_total = vec![profile_zero(); len];
        }
        if prof.tml_self.is_empty() {
            prof.tml_self = vec![profile_zero(); len];
        }
        prof.tml_idx = -1;
        prof.initialized = true;
    }
    prof.profiling = true;
}

/// Prepare for entering a child (another script/function/shell command)
/// whose time should not count towards the current one. Returns the wait
/// time to pass to [`prof_child_exit`].
pub fn prof_child_enter() -> ProfTime {
    if let Some(frame) = profiled_funccal() {
        frame.prof_child.set(profile_start());
    }
    script_prof_save()
}

/// Account the time spent in a child; pairs with [`prof_child_enter`],
/// `wait` being its return value.
pub fn prof_child_exit(wait: ProfTime) {
    if let Some(frame) = profiled_funccal() {
        // Don't count waiting time.
        let child = profile_sub_wait(wait, profile_end(frame.prof_child.get()));
        frame.prof_child.set(child);
        let mut prof = frame.func.prof.borrow_mut();
        prof.tm_children = profile_add(prof.tm_children, child);
        prof.tml_children = profile_add(prof.tml_children, child);
    }
    script_prof_restore(wait);
}

/// The current call frame, when its function is being profiled.
fn profiled_funccal() -> Option<Rc<FuncCall>> {
    current_fc().filter(|frame| frame.func.prof.borrow().profiling)
}

/// Called when starting to read a line of the function `frame` is running;
/// the exestack lnum must be correct. The line may turn out not to execute
/// — the time is stored now, counted only if [`func_line_exec`] follows.
pub(crate) fn func_line_start(frame: &FuncCall) {
    let func = &frame.func;
    let lnum = sourcing_lnum();
    let body = func.body();
    let mut prof = func.prof.borrow_mut();
    if prof.profiling
        && lnum >= 1
        && usize::try_from(lnum).is_ok_and(|lnum| lnum <= body.lines.len())
    {
        let mut idx = usize::try_from(lnum - 1).unwrap_or(0);
        // Skip continuation lines, which the body stores as `None`.
        while idx > 0 && body.lines[idx].is_none() {
            idx -= 1;
        }
        prof.tml_idx = c_int::try_from(idx).unwrap_or(c_int::MAX);
        prof.tml_execed = false;
        prof.tml_start = profile_start();
        prof.tml_children = profile_zero();
        prof.tml_wait = PROF_WAIT_TIME.get();
    }
}

/// Called when actually executing a line of the function `frame` is
/// running.
pub(crate) fn func_line_exec(frame: &FuncCall) {
    let mut prof = frame.func.prof.borrow_mut();
    if prof.profiling && prof.tml_idx >= 0 {
        prof.tml_execed = true;
    }
}

/// Called when done with a line of the function `frame` is running.
pub(crate) fn func_line_end(frame: &FuncCall) {
    let mut prof = frame.func.prof.borrow_mut();
    if prof.profiling && prof.tml_idx >= 0 {
        if prof.tml_execed {
            let i = usize::try_from(prof.tml_idx).unwrap_or(0);
            let spent = profile_sub_wait(prof.tml_wait, profile_end(prof.tml_start));
            prof.tml_start = spent;
            let children = prof.tml_children;
            // `tml_idx` was checked against the body in `func_line_start`,
            // which is what the counters are sized to.
            if let Some(count) = prof.tml_count.get_mut(i) {
                *count += 1;
            }
            if let Some(total) = prof.tml_total.get_mut(i) {
                *total = profile_add(*total, spent);
            }
            if let Some(own) = prof.tml_self.get_mut(i) {
                *own = profile_self(*own, spent, children);
            }
        }
        prof.tml_idx = -1;
    }
}

/// [`func_line_start`] for the function body `cookie` is the
/// `do_cmdline` cookie of.
pub fn func_line_start_cookie(cookie: *mut c_void) {
    func_line_start(&cookie_funccall(cookie));
}

/// [`func_line_exec`] for the function body `cookie` is the `do_cmdline`
/// cookie of.
pub fn func_line_exec_cookie(cookie: *mut c_void) {
    func_line_exec(&cookie_funccall(cookie));
}

/// [`func_line_end`] for the function body `cookie` is the `do_cmdline`
/// cookie of.
pub fn func_line_end_cookie(cookie: *mut c_void) {
    func_line_end(&cookie_funccall(cookie));
}

// ---------------------------------------------------------------------------
// Script profiling.

/// Start profiling script `si` (`:profile file` match on source).
pub fn profile_init(si: &mut ScriptItem) {
    si.sn_pr_count = 0;
    si.sn_pr_total = profile_zero();
    si.sn_pr_self = profile_zero();
    si.sn_prl_ga = Vec::new();
    si.sn_prl_idx = -1;
    si.sn_prof_on = true;
    si.sn_pr_nest = 0;
}

/// Save the wait time when starting to invoke another script or function;
/// returns the snapshot for [`script_prof_restore`].
pub fn script_prof_save() -> ProfTime {
    with_current_script(|si| {
        if si.sn_prof_on {
            let nest = si.sn_pr_nest;
            si.sn_pr_nest += 1;
            if nest == 0 {
                si.sn_pr_child = profile_start();
            }
        }
    });
    PROF_WAIT_TIME.get()
}

/// Count time spent in children after invoking another script or function;
/// `wait` is what [`script_prof_save`] returned.
pub fn script_prof_restore(wait: ProfTime) {
    with_current_script(|si| {
        if !si.sn_prof_on {
            return;
        }
        si.sn_pr_nest -= 1;
        if si.sn_pr_nest == 0 {
            // Don't count wait time.
            let child = profile_sub_wait(wait, profile_end(si.sn_pr_child));
            si.sn_pr_child = child;
            si.sn_pr_children = profile_add(si.sn_pr_children, child);
            si.sn_prl_children = profile_add(si.sn_prl_children, child);
        }
    });
}

/// Called when starting to read a script line; the exestack lnum must be
/// correct. See [`func_line_start`] for the execed dance.
pub fn script_line_start() {
    let lnum = sourcing_lnum();
    with_current_script(|si| {
        if !(si.sn_prof_on && lnum >= 1) {
            return;
        }
        // Grow the array before starting the timer, so that the time spent
        // here isn't counted. Lines that were never reached keep the zero
        // counters this leaves behind.
        let lines = usize::try_from(lnum).unwrap_or(0);
        if si.sn_prl_ga.len() < lines {
            si.sn_prl_ga.resize(lines, SnPrl::default());
        }
        si.sn_prl_idx = lnum - 1;
        si.sn_prl_execed = 0;
        si.sn_prl_start = profile_start();
        si.sn_prl_children = profile_zero();
        si.sn_prl_wait = PROF_WAIT_TIME.get();
    });
}

/// Called when actually executing a script line.
pub fn script_line_exec() {
    with_current_script(|si| {
        if si.sn_prof_on && si.sn_prl_idx >= 0 {
            si.sn_prl_execed = 1;
        }
    });
}

/// Called when done with a script line.
pub fn script_line_end() {
    with_current_script(script_line_done);
}

/// [`script_line_end`] of the current script's item.
fn script_line_done(si: &mut ScriptItem) {
    let idx = usize::try_from(si.sn_prl_idx).ok();
    if si.sn_prof_on && idx.is_some_and(|idx| idx < si.sn_prl_ga.len()) {
        if si.sn_prl_execed != 0 {
            let idx = idx.unwrap_or(0);
            let spent = profile_sub_wait(si.sn_prl_wait, profile_end(si.sn_prl_start));
            si.sn_prl_start = spent;
            let children = si.sn_prl_children;
            let pp = &mut si.sn_prl_ga[idx];
            pp.snp_count += 1;
            pp.sn_prl_total = profile_add(pp.sn_prl_total, spent);
            pp.sn_prl_self = profile_self(pp.sn_prl_self, spent, children);
        }
        si.sn_prl_idx = -1;
    }
}

// ---------------------------------------------------------------------------
// Shared accessors for the editor's script/function tables.

/// Run `f` on the current script's item, if `current_sctx` points at a
/// valid one.
fn with_current_script(f: impl FnOnce(&mut ScriptItem)) {
    let sid = current_sctx.get().sc_sid;
    if script_id_valid(sid) {
        with_script_item(sid, f);
    }
}

/// Line number being sourced/executed: the top of the exestack.
fn sourcing_lnum() -> LineNr {
    crate::runtime::innermost_frame().es_lnum
}

/// All functions in the global function table with profiling data, in the
/// table's slot order.
fn profiled_functions() -> Vec<Rc<UserFunc>> {
    all_funcs()
        .into_iter()
        .filter(|func| func.prof.borrow().initialized)
        .collect()
}
