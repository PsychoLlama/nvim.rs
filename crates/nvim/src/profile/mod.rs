//! Profiling: the `proftime_T` arithmetic shared by `:profile`, `reltime()`
//! and regex/search timeouts, the `:profile` command, and the per-line
//! accounting the profiled scripts and functions keep.
//!
//! A `proftime_T` is a `u64` nanosecond reading from `os_hrtime`. Durations
//! are unsigned differences and may wrap when a "later" time is subtracted
//! from an "earlier" one; [`profile_signed`] recovers the signed value
//! (#10452), and everything user-visible funnels through it.
//!
//! | file | what |
//! | --- | --- |
//! | this one | the arithmetic, `:profile`, and the accounting hooks the interpreter calls per line and per call |
//! | [`report`] | what `:profile dump` writes |
//! | [`startuptime`] | the `--startuptime` log |
//!
//! Every `unsafe fn` here has the same contract unless it says otherwise: a
//! main-thread editor call, with the script table, the function table and
//! the exestack live. The pointer-taking ones additionally want a live item
//! of the table they name.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod report;
pub mod startuptime;

// The report and the startuptime log were split out of this file; callers
// name them where they have always been named.
pub use report::profile_dump;
pub use startuptime::{time_finish, time_init, time_msg, time_pop, time_push, time_start};

use crate::charset::{skiptowhite, skipwhite};
use crate::debugger::ex_breakadd;
use crate::eval::userfunc::{func_tbl_get, get_current_funccal};
use crate::eval::vars::set_vim_var_nr;
use crate::garray::{ga_clear, ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::hashtab::hash_removed;
use crate::main::{current_sctx, do_profiling};
use crate::memory::xcalloc;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::os::env::expand_env_save_opt;
use crate::os::time::os_hrtime;
use crate::runtime::{script_count, script_id_valid, script_item};
use crate::types::{
    ExpandContext, Vv, exarg_T, expand_T, funccall_T, int64_t, linenr_T, proftime_T, scriptitem_T,
    ufunc_T, varnumber_T,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use std::ffi::CString;

/// `do_profiling` states (a `GlobalCell<c_int>` in main).
pub const PROF_NONE: c_int = 0;
pub const PROF_YES: c_int = 1;
pub const PROF_PAUSED: c_int = 2;

/// First byte of a `<SNR>`-mangled function name.
const NL: c_char = b'\n' as c_char;
/// Offset of `uf_name` inside `ufunc_T`: hash keys point at the name, this
/// recovers the function (the transpiled `HI2UF`, same constant as
/// eval/userfunc/ uses).
const UF_NAME_OFFSET: isize = 240;

/// Accumulated time the user kept the editor waiting (input, `:profile
/// pause`); subtracted from measurements via [`profile_sub_wait`].
static PROF_WAIT_TIME: GlobalCell<proftime_T> = GlobalCell::new(0);
/// Report path from `:profile start {fname}`; `None` when not profiling.
static PROFILE_FNAME: GlobalCell<Option<CString>> = GlobalCell::new(None);

/// Per-line counters of a profiled script, the element type of
/// `scriptitem_T.sn_prl_ga`.
#[derive(Copy, Clone)]
struct sn_prl_T {
    snp_count: c_int,
    sn_prl_total: proftime_T,
    sn_prl_self: proftime_T,
}

// ---------------------------------------------------------------------------
// Time arithmetic.

/// The current time.
pub fn profile_start() -> proftime_T {
    os_hrtime()
}

/// Elapsed time from `tm` until now.
pub fn profile_end(tm: proftime_T) -> proftime_T {
    profile_sub(profile_start(), tm)
}

/// The zero time.
pub fn profile_zero() -> proftime_T {
    0
}

/// The time `msec` milliseconds into the future, or the zero time ("no
/// limit") when `msec <= 0`.
pub fn profile_setlimit(msec: int64_t) -> proftime_T {
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
    let nsec = (msec as proftime_T)
        .saturating_mul(1_000_000)
        .min(int64_t::MAX as proftime_T);
    profile_start().wrapping_add(nsec)
}

/// Whether the current time is past `tm`. False if the limit was never set
/// (`tm` is the zero time).
pub fn profile_passed_limit(tm: proftime_T) -> bool {
    if tm == 0 {
        return false;
    }
    profile_cmp(profile_start(), tm) < 0
}

/// `tm / count` (rounded), or zero when `count <= 0`.
pub fn profile_divide(tm: proftime_T, count: c_int) -> proftime_T {
    if count <= 0 {
        return profile_zero();
    }
    (tm as f64 / count as f64).round() as proftime_T
}

pub fn profile_add(tm1: proftime_T, tm2: proftime_T) -> proftime_T {
    tm1.wrapping_add(tm2)
}

/// `tm1 - tm2`, wrapping when `tm2 > tm1`; see [`profile_signed`].
pub fn profile_sub(tm1: proftime_T, tm2: proftime_T) -> proftime_T {
    tm1.wrapping_sub(tm2)
}

/// Self time: `self + total - children`, or `self` unchanged when `total <=
/// children` (possible with recursive calls).
pub fn profile_self(self_: proftime_T, total: proftime_T, children: proftime_T) -> proftime_T {
    if total <= children {
        return self_;
    }
    profile_sub(profile_add(self_, total), children)
}

/// `tma` minus the wait time accumulated since the [`PROF_WAIT_TIME`]
/// snapshot `tm`.
pub fn profile_sub_wait(tm: proftime_T, tma: proftime_T) -> proftime_T {
    let waited = profile_sub(PROF_WAIT_TIME.get(), tm);
    profile_sub(tma, waited)
}

/// Signed value of a duration produced by [`profile_sub`]. Values above
/// `i64::MAX` (>=150 years) are taken to be wrapped negative differences.
pub fn profile_signed(tm: proftime_T) -> int64_t {
    if tm <= int64_t::MAX as proftime_T {
        tm as int64_t
    } else {
        -((proftime_T::MAX - tm) as int64_t)
    }
}

/// Compare two times (which must be less than 150 years apart): negative
/// when `tm2 < tm1`, `0` when equal, positive when `tm2 > tm1`.
pub fn profile_cmp(tm1: proftime_T, tm2: proftime_T) -> c_int {
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
pub fn profile_msg_str(tm: proftime_T) -> String {
    format!("{:10.6}", profile_signed(tm) as f64 / 1e9)
}

/// C-string flavor of [`profile_msg_str`] for the transpiled callers
/// (syntime report, `reltimestr()`), in its own storage. Upstream answers a
/// static buffer the next call overwrites.
pub(crate) fn profile_msg(tm: proftime_T) -> [c_char; 50] {
    let s = profile_msg_str(tm);
    let mut buf = [0 as c_char; 50];
    let n = s.len().min(buf.len() - 1);
    for (dst, src) in buf.iter_mut().zip(s.as_bytes()[..n].iter()) {
        *dst = *src as c_char;
    }
    buf[n] = 0;
    buf
}

// ---------------------------------------------------------------------------
// The :profile command.

/// `:profile cmd args`. In the ex_docmd command table.
///
/// # Safety
/// `eap` is the live ex command being executed.
pub unsafe fn ex_profile(eap: *mut exarg_T) {
    /// Time at which `:profile pause` stopped the clock.
    static PAUSE_TIME: GlobalCell<proftime_T> = GlobalCell::new(0);

    // SAFETY: `eap.arg` is the command's NUL-terminated argument, so both
    // walkers stay inside it and the two views borrow from it for the length
    // of this call.
    let (subcmd, full, e) = unsafe {
        let arg = (*eap).arg;
        let end = skiptowhite(arg);
        let len = end.offset_from(arg) as usize;
        (
            core::slice::from_raw_parts(arg as *const u8, len),
            CStr::from_ptr(arg).to_bytes(),
            skipwhite(end),
        )
    };

    if subcmd == b"start" && unsafe { *e } != 0 {
        // SAFETY: `e` points into the argument; expand_env_save_opt returns
        // an xmalloc'd C string, and the global allocator is malloc-backed,
        // so CString may own (and later free) it.
        let fname = unsafe { CString::from_raw(expand_env_save_opt(e, true)) };
        PROFILE_FNAME.set(Some(fname));
        do_profiling.set(PROF_YES);
        PROF_WAIT_TIME.set(profile_zero());
        // SAFETY: a v: variable set to a number.
        unsafe { set_vim_var_nr(Vv::Profiling, 1 as varnumber_T) };
    } else if do_profiling.get() == PROF_NONE {
        emsg(gettext(c"E750: First use \":profile start {fname}\""));
    } else if full == b"stop" {
        profile_dump();
        do_profiling.set(PROF_NONE);
        // SAFETY: a v: variable set to a number, then the profiling tables,
        // which are live for as long as the editor is.
        unsafe { set_vim_var_nr(Vv::Profiling, 0 as varnumber_T) };
        unsafe { profile_reset() };
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
        unsafe { ex_breakadd(eap) };
    }
}

/// Forget all profiling information (`:profile stop`).
///
/// # Safety
/// Main-thread editor call; the script and function tables are live.
unsafe fn profile_reset() {
    for id in 1..=script_count() {
        // SAFETY: `1..=ga_len` are the live script ids.
        let si = unsafe { &mut *script_item(id) };
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
            // SAFETY: the per-line array belongs to this script item.
            unsafe { ga_clear(&raw mut si.sn_prl_ga) };
            si.sn_prl_start = profile_zero();
            si.sn_prl_children = profile_zero();
            si.sn_prl_wait = profile_zero();
            si.sn_prl_idx = -1;
            si.sn_prl_execed = 0;
        }
    }
    // SAFETY: the function table is live and its entries outlive this walk.
    for uf in unsafe { profiled_functions() } {
        // SAFETY: an entry of the function table.
        let uf = unsafe { &mut *uf };
        uf.uf_profiling = 0;
        uf.uf_tm_count = 0;
        uf.uf_tm_total = profile_zero();
        uf.uf_tm_self = profile_zero();
        uf.uf_tm_children = profile_zero();
        for i in 0..uf.uf_lines.ga_len as isize {
            // SAFETY: `func_do_profile` sized all three per-line arrays to
            // `uf_lines`, which is what this walks.
            unsafe { *uf.uf_tml_count.offset(i) = 0 };
            unsafe { *uf.uf_tml_total.offset(i) = 0 };
            unsafe { *uf.uf_tml_self.offset(i) = 0 };
        }
        uf.uf_tml_start = profile_zero();
        uf.uf_tml_children = profile_zero();
        uf.uf_tml_wait = profile_zero();
        uf.uf_tml_idx = -1;
        uf.uf_tml_execed = 0;
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
pub fn get_profile_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    usize::try_from(idx)
        .ok()
        .and_then(|i| PEXPAND_CMDS.get(i))
        .map_or(core::ptr::null_mut(), |s| s.as_ptr() as *mut c_char)
}

/// Command-line completion context for `:profile`.
///
/// # Safety
/// `xp` is the live expansion context; `arg` is NUL-terminated and outlives
/// it (it is stored in `xp_pattern`).
pub unsafe fn set_context_in_profile_cmd(xp: *mut expand_T, arg: *const c_char) {
    // SAFETY: the caller's context.
    let xp = unsafe { &mut *xp };
    // Default: expand subcommands.
    xp.xp_context = ExpandContext::Profile;
    xp.xp_pattern = arg as *mut c_char;

    // SAFETY: `arg` is NUL-terminated, so the walk stays inside it and
    // `subcmd` borrows from it.
    let (subcmd, rest) = unsafe {
        let end_subcmd = skiptowhite(arg);
        if *end_subcmd == 0 {
            return;
        }
        let len = end_subcmd.offset_from(arg) as usize;
        (
            core::slice::from_raw_parts(arg as *const u8, len),
            skipwhite(end_subcmd),
        )
    };
    if subcmd == b"start" || subcmd == b"file" {
        xp.xp_context = ExpandContext::Files;
        xp.xp_pattern = rest;
    } else if subcmd == b"func" {
        xp.xp_context = ExpandContext::UserFunc;
        xp.xp_pattern = rest;
    } else {
        xp.xp_context = ExpandContext::Nothing;
    }
}

// ---------------------------------------------------------------------------
// Wait time.

/// When the editor started waiting for the user to type.
static INPUT_WAIT_START: GlobalCell<proftime_T> = GlobalCell::new(0);

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
///
/// # Safety
/// Main-thread editor call; the script table is live.
pub unsafe fn prof_def_func() -> bool {
    let sid = current_sctx.get().sc_sid;
    // SAFETY: a positive `sc_sid` is a live script id.
    sid > 0 && unsafe { (*script_item(sid)).sn_pr_force }
}

/// Start profiling function `fp`, allocating its per-line counters on
/// first use.
///
/// # Safety
/// `fp` is a live function-table entry.
pub unsafe fn func_do_profile(fp: *mut ufunc_T) {
    // SAFETY: the caller's function.
    let fp = unsafe { &mut *fp };
    // Avoid allocating zero bytes.
    let len = (fp.uf_lines.ga_len as usize).max(1);
    if fp.uf_prof_initialized == 0 {
        fp.uf_tm_count = 0;
        fp.uf_tm_self = profile_zero();
        fp.uf_tm_total = profile_zero();
        // SAFETY: `xcalloc` returns an owned zeroed array of `len` elements,
        // which is what the three per-line counters are read as everywhere.
        if fp.uf_tml_count.is_null() {
            fp.uf_tml_count = unsafe { xcalloc(len, size_of::<c_int>()) } as *mut c_int;
        }
        if fp.uf_tml_total.is_null() {
            fp.uf_tml_total = unsafe { xcalloc(len, size_of::<proftime_T>()) } as *mut proftime_T;
        }
        if fp.uf_tml_self.is_null() {
            fp.uf_tml_self = unsafe { xcalloc(len, size_of::<proftime_T>()) } as *mut proftime_T;
        }
        fp.uf_tml_idx = -1;
        fp.uf_prof_initialized = 1;
    }
    fp.uf_profiling = 1;
}

/// Prepare for entering a child (another script/function/shell command)
/// whose time should not count towards the current one. Returns the wait
/// time to pass to [`prof_child_exit`].
///
/// # Safety
/// Main-thread editor call; the call stack and script table are live.
pub unsafe fn prof_child_enter() -> proftime_T {
    // SAFETY: `get_current_funccal` answers with the live call frame or null,
    // and a frame's `fc_func` is the function being executed.
    if let Some(fc) = unsafe { profiled_funccal() } {
        unsafe { (*fc).fc_prof_child = profile_start() };
    }
    unsafe { script_prof_save() }
}

/// Account the time spent in a child; pairs with [`prof_child_enter`],
/// `wait` being its return value.
///
/// # Safety
/// Main-thread editor call; the call stack and script table are live.
pub unsafe fn prof_child_exit(wait: proftime_T) {
    // SAFETY: as [`prof_child_enter`].
    if let Some(fc) = unsafe { profiled_funccal() } {
        let fc = unsafe { &mut *fc };
        // Don't count waiting time.
        let child = profile_sub_wait(wait, profile_end(fc.fc_prof_child));
        fc.fc_prof_child = child;
        let func = unsafe { &mut *fc.fc_func };
        func.uf_tm_children = profile_add(func.uf_tm_children, child);
        func.uf_tml_children = profile_add(func.uf_tml_children, child);
    }
    unsafe { script_prof_restore(wait) };
}

/// The current call frame, when its function is being profiled.
///
/// # Safety
/// Main-thread editor call; the call stack is live.
unsafe fn profiled_funccal() -> Option<*mut funccall_T> {
    // SAFETY: the caller's contract.
    let fc = unsafe { get_current_funccal() };
    (!fc.is_null() && unsafe { (*(*fc).fc_func).uf_profiling } != 0).then_some(fc)
}

/// Called when starting to read a function line; the exestack lnum must be
/// correct. The line may turn out not to execute — the time is stored now,
/// counted only if [`func_line_exec`] follows.
///
/// # Safety
/// `cookie` is the live `funccall_T` of the function being executed.
pub unsafe fn func_line_start(cookie: *mut c_void) {
    // SAFETY: the caller's call frame and its function.
    let fp = unsafe { &mut *(*(cookie as *mut funccall_T)).fc_func };
    let lnum = sourcing_lnum();
    if fp.uf_profiling != 0 && lnum >= 1 && lnum <= fp.uf_lines.ga_len as linenr_T {
        fp.uf_tml_idx = lnum as c_int - 1;
        // Skip continuation lines, which the line array stores as nulls.
        while fp.uf_tml_idx > 0 && unsafe { func_line(fp, fp.uf_tml_idx as isize) }.is_null() {
            fp.uf_tml_idx -= 1;
        }
        fp.uf_tml_execed = 0;
        fp.uf_tml_start = profile_start();
        fp.uf_tml_children = profile_zero();
        fp.uf_tml_wait = PROF_WAIT_TIME.get();
    }
}

/// The `idx`'th source line of `fp`, or null for a continuation line.
///
/// # Safety
/// `idx` is below `fp.uf_lines.ga_len`.
unsafe fn func_line(fp: &ufunc_T, idx: isize) -> *mut c_char {
    // SAFETY: the caller's bound; the array holds `ga_len` line pointers.
    unsafe { *(fp.uf_lines.ga_data as *mut *mut c_char).offset(idx) }
}

/// Called when actually executing a function line.
///
/// # Safety
/// `cookie` is the live `funccall_T` of the function being executed.
pub unsafe fn func_line_exec(cookie: *mut c_void) {
    // SAFETY: the caller's call frame and its function.
    let fp = unsafe { &mut *(*(cookie as *mut funccall_T)).fc_func };
    if fp.uf_profiling != 0 && fp.uf_tml_idx >= 0 {
        fp.uf_tml_execed = 1;
    }
}

/// Called when done with a function line.
///
/// # Safety
/// `cookie` is the live `funccall_T` of the function being executed.
pub unsafe fn func_line_end(cookie: *mut c_void) {
    // SAFETY: the caller's call frame and its function.
    let fp = unsafe { &mut *(*(cookie as *mut funccall_T)).fc_func };
    if fp.uf_profiling != 0 && fp.uf_tml_idx >= 0 {
        if fp.uf_tml_execed != 0 {
            let i = fp.uf_tml_idx as isize;
            // SAFETY: `uf_tml_idx` was checked against `uf_lines.ga_len` in
            // `func_line_start`, which is what the three arrays are sized to.
            unsafe { *fp.uf_tml_count.offset(i) += 1 };
            let spent = profile_sub_wait(fp.uf_tml_wait, profile_end(fp.uf_tml_start));
            fp.uf_tml_start = spent;
            let children = fp.uf_tml_children;
            // SAFETY: as above.
            unsafe { *fp.uf_tml_total.offset(i) = profile_add(*fp.uf_tml_total.offset(i), spent) };
            unsafe {
                *fp.uf_tml_self.offset(i) = profile_self(*fp.uf_tml_self.offset(i), spent, children)
            };
        }
        fp.uf_tml_idx = -1;
    }
}

// ---------------------------------------------------------------------------
// Script profiling.

/// Start profiling script `si` (`:profile file` match on source).
///
/// # Safety
/// `si` is a live script item.
pub unsafe fn profile_init(si: *mut scriptitem_T) {
    // SAFETY: the caller's script item.
    let si = unsafe { &mut *si };
    si.sn_pr_count = 0;
    si.sn_pr_total = profile_zero();
    si.sn_pr_self = profile_zero();
    // SAFETY: the per-line array belongs to this item and is uninitialised
    // until now.
    unsafe { ga_init(&raw mut si.sn_prl_ga, size_of::<sn_prl_T>() as c_int, 100) };
    si.sn_prl_idx = -1;
    si.sn_prof_on = true;
    si.sn_pr_nest = 0;
}

/// Save the wait time when starting to invoke another script or function;
/// returns the snapshot for [`script_prof_restore`].
///
/// # Safety
/// Main-thread editor call; the script table is live.
pub unsafe fn script_prof_save() -> proftime_T {
    if let Some(si) = current_script() {
        let si = unsafe { &mut *si };
        if si.sn_prof_on {
            let nest = si.sn_pr_nest;
            si.sn_pr_nest += 1;
            if nest == 0 {
                si.sn_pr_child = profile_start();
            }
        }
    }
    PROF_WAIT_TIME.get()
}

/// Count time spent in children after invoking another script or function;
/// `wait` is what [`script_prof_save`] returned.
///
/// # Safety
/// Main-thread editor call; the script table is live.
pub unsafe fn script_prof_restore(wait: proftime_T) {
    let Some(si) = current_script() else {
        return;
    };
    let si = unsafe { &mut *si };
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
}

/// Called when starting to read a script line; the exestack lnum must be
/// correct. See [`func_line_start`] for the execed dance.
///
/// # Safety
/// Main-thread editor call; the script table and exestack are live.
pub unsafe fn script_line_start() {
    // SAFETY: `current_script` only answers with a live script item, and the
    // exestack is live while a script line is being read.
    let (si, lnum) = unsafe {
        let Some(si) = current_script() else { return };
        (&mut *si, sourcing_lnum())
    };
    if si.sn_prof_on && lnum >= 1 {
        // Grow the array before starting the timer, so that the time spent
        // here isn't counted.
        // SAFETY: the per-line array belongs to this item.
        unsafe { ga_grow(&raw mut si.sn_prl_ga, lnum as c_int - si.sn_prl_ga.ga_len) };
        si.sn_prl_idx = lnum - 1;
        while (si.sn_prl_ga.ga_len as linenr_T) <= si.sn_prl_idx
            && si.sn_prl_ga.ga_len < si.sn_prl_ga.ga_maxlen
        {
            // Zero counters for a line that was not used before.
            // SAFETY: `ga_len` is below `ga_maxlen`, which is what the array
            // holds room for.
            let pp = unsafe { &mut *prl_item(si, si.sn_prl_ga.ga_len as isize) };
            pp.snp_count = 0;
            pp.sn_prl_total = profile_zero();
            pp.sn_prl_self = profile_zero();
            si.sn_prl_ga.ga_len += 1;
        }
        si.sn_prl_execed = 0;
        si.sn_prl_start = profile_start();
        si.sn_prl_children = profile_zero();
        si.sn_prl_wait = PROF_WAIT_TIME.get();
    }
}

/// Called when actually executing a script line.
///
/// # Safety
/// Main-thread editor call; the script table is live.
pub unsafe fn script_line_exec() {
    let Some(si) = current_script() else {
        return;
    };
    let si = unsafe { &mut *si };
    if si.sn_prof_on && si.sn_prl_idx >= 0 {
        si.sn_prl_execed = 1;
    }
}

/// Called when done with a script line.
///
/// # Safety
/// Main-thread editor call; the script table is live.
pub unsafe fn script_line_end() {
    let Some(si) = current_script() else {
        return;
    };
    let si = unsafe { &mut *si };
    if si.sn_prof_on && si.sn_prl_idx >= 0 && si.sn_prl_idx < si.sn_prl_ga.ga_len as linenr_T {
        if si.sn_prl_execed != 0 {
            // SAFETY: `sn_prl_idx` was just checked against `ga_len`.
            let pp = unsafe { &mut *prl_item(si, si.sn_prl_idx as isize) };
            pp.snp_count += 1;
            let spent = profile_sub_wait(si.sn_prl_wait, profile_end(si.sn_prl_start));
            si.sn_prl_start = spent;
            pp.sn_prl_total = profile_add(pp.sn_prl_total, spent);
            pp.sn_prl_self = profile_self(pp.sn_prl_self, spent, si.sn_prl_children);
        }
        si.sn_prl_idx = -1;
    }
}

// ---------------------------------------------------------------------------
// Shared accessors for the editor's script/function tables.

/// The current script's item, if `current_sctx` points at a valid one.
fn current_script() -> Option<*mut scriptitem_T> {
    let sid = current_sctx.get().sc_sid;
    script_id_valid(sid).then(|| script_item(sid))
}

/// Line number being sourced/executed: the top of the exestack.
fn sourcing_lnum() -> linenr_T {
    crate::runtime::innermost_frame().es_lnum
}

/// Per-line counters of script `si` at `idx`.
///
/// # Safety
/// `idx` is below `si.sn_prl_ga.ga_maxlen`.
unsafe fn prl_item(si: &scriptitem_T, idx: isize) -> *mut sn_prl_T {
    // SAFETY: the caller's bound.
    unsafe { (si.sn_prl_ga.ga_data as *mut sn_prl_T).offset(idx) }
}

/// All functions in the global function table with profiling data, in hash
/// table order.
///
/// # Safety
/// Main-thread editor call; the function table is live.
unsafe fn profiled_functions() -> Vec<*mut ufunc_T> {
    let mut found = Vec::new();
    // SAFETY: the caller's contract. A hash item's key points at the
    // `uf_name` field of its `ufunc_T`, which is what the offset undoes;
    // `ht_used` bounds how many occupied slots the walk will find, and the
    // array is NUL/removed-padded up to that many.
    let functbl = func_tbl_get();
    let mut todo = unsafe { (*functbl).ht_used };
    let mut hi = unsafe { (*functbl).ht_array };
    while todo > 0 {
        if !unsafe { (*hi).hi_key }.is_null()
            && !core::ptr::eq(unsafe { (*hi).hi_key }, &raw const hash_removed)
        {
            todo -= 1;
            let fp = unsafe { (*hi).hi_key.offset(-UF_NAME_OFFSET) } as *mut ufunc_T;
            if unsafe { (*fp).uf_prof_initialized } != 0 {
                found.push(fp);
            }
        }
        hi = unsafe { hi.offset(1) };
    }
    found
}
