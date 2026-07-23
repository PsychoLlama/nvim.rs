//! Profiling: the `proftime_T` arithmetic shared by `:profile`, `reltime()`
//! and regex/search timeouts, the `:profile` command with its function- and
//! script-level accounting, the report written by `:profile dump` and on
//! exit, and the `--startuptime` log.
//!
//! A `proftime_T` is a `u64` nanosecond reading from `os_hrtime`. Durations
//! are unsigned differences and may wrap when a "later" time is subtracted
//! from an "earlier" one; [`profile_signed`] recovers the signed value
//! (#10452), and everything user-visible funnels through it.
//!
//! The `--startuptime` log stays on C stdio: `time_fd` (in main) is set up
//! here with a setvbuf full-buffering arrangement so concurrent nvim
//! processes appending to one file each flush their report exactly once, at
//! [`time_finish`]. The `:profile` report has no such constraint and is
//! written with plain Rust I/O.

use crate::src::nvim::charset::{skiptowhite, skipwhite};
use crate::src::nvim::debugger::ex_breakadd;
use crate::src::nvim::eval::userfunc::{func_tbl_get, get_current_funccal};
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::event::libuv::uv_err_name;
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::main::{current_sctx, do_profiling, e_notopen, time_fd};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::env::expand_env_save_opt;
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::libc::{fclose, fopen, fprintf, gettext, setvbuf, stderr};
use crate::src::nvim::os::time::os_hrtime;
use crate::src::nvim::runtime::{exestack, get_scriptname, script_items};
use crate::src::nvim::types::{
    estack_T, exarg_T, expand_T, funccall_T, int64_t, linenr_T, proftime_T, scriptitem_T, ufunc_T,
    varnumber_T, VimVarIndex,
};
use core::ffi::{c_char, c_int, c_void, CStr};
use std::ffi::{CString, OsStr};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::os::unix::ffi::OsStrExt;

/// `do_profiling` states (a `GlobalCell<c_int>` in main).
pub const PROF_NONE: c_int = 0;
pub const PROF_YES: c_int = 1;
pub const PROF_PAUSED: c_int = 2;

const VV_PROFILING: VimVarIndex = 37;
const EXPAND_NOTHING: c_int = 0;
const EXPAND_FILES: c_int = 2;
const EXPAND_USER_FUNC: c_int = 19;
const EXPAND_PROFILE: c_int = 35;
/// First byte of a `<SNR>`-mangled function name.
const K_SPECIAL: u8 = 0x80;
const NL: c_char = b'\n' as c_char;
const IOSIZE: usize = 1024 + 1;
/// Offset of `uf_name` inside `ufunc_T`: hash keys point at the name, this
/// recovers the function (the transpiled `HI2UF`, same constant as
/// eval/userfunc.rs uses).
const UF_NAME_OFFSET: isize = 240;

/// Accumulated time the user kept the editor waiting (input, `:profile
/// pause`); subtracted from measurements via [`profile_sub_wait`].
static PROF_WAIT_TIME: GlobalCell<proftime_T> = GlobalCell::new(0);
/// Report path from `:profile start {fname}`; `None` when not profiling.
static PROFILE_FNAME: GlobalCell<Option<CString>> = GlobalCell::new(None);

/// Per-line counters of a profiled script, the element type of
/// `scriptitem_T.sn_prl_ga`.
#[derive(Copy, Clone)]
#[repr(C)]
struct sn_prl_T {
    snp_count: c_int,
    sn_prl_total: proftime_T,
    sn_prl_self: proftime_T,
}

// ---------------------------------------------------------------------------
// Time arithmetic.

/// The current time.
pub fn profile_start() -> proftime_T {
    // SAFETY: uv_hrtime has no preconditions.
    unsafe { os_hrtime() }
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
    debug_assert!(msec <= (int64_t::MAX / 1_000_000) - 1);
    let nsec = (msec as proftime_T).wrapping_mul(1_000_000);
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
/// (syntime report, `reltimestr()`). Returns a static buffer: copy it
/// before the next call, don't free it.
pub fn profile_msg(tm: proftime_T) -> *const c_char {
    static BUF: GlobalCell<[c_char; 50]> = GlobalCell::new([0; 50]);
    let s = profile_msg_str(tm);
    BUF.with_mut(|buf| {
        let n = s.len().min(buf.len() - 1);
        for (dst, src) in buf.iter_mut().zip(s.as_bytes()[..n].iter()) {
            *dst = *src as c_char;
        }
        buf[n] = 0;
    });
    BUF.as_raw() as *const c_char
}

// ---------------------------------------------------------------------------
// The :profile command.

/// `:profile cmd args`. In the ex_docmd command table.
pub unsafe extern "C" fn ex_profile(eap: *mut exarg_T) {
    /// Time at which `:profile pause` stopped the clock.
    static PAUSE_TIME: GlobalCell<proftime_T> = GlobalCell::new(0);

    let arg = (*eap).arg;
    let end = skiptowhite(arg);
    let len = end.offset_from(arg) as usize;
    let e = skipwhite(end);
    let subcmd = core::slice::from_raw_parts(arg as *const u8, len);
    let full = CStr::from_ptr(arg).to_bytes();

    if subcmd == b"start" && *e != 0 {
        // expand_env_save_opt returns an xmalloc'd C string; the global
        // allocator is malloc-backed, so CString may own (and later free) it.
        PROFILE_FNAME.set(Some(CString::from_raw(expand_env_save_opt(e, true))));
        do_profiling.set(PROF_YES);
        PROF_WAIT_TIME.set(profile_zero());
        set_vim_var_nr(VV_PROFILING, 1 as varnumber_T);
    } else if do_profiling.get() == PROF_NONE {
        emsg(gettext(
            b"E750: First use \":profile start {fname}\"\0".as_ptr() as *const c_char,
        ));
    } else if full == b"stop" {
        profile_dump();
        do_profiling.set(PROF_NONE);
        set_vim_var_nr(VV_PROFILING, 0 as varnumber_T);
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
        ex_breakadd(eap);
    }
}

/// Forget all profiling information (`:profile stop`).
unsafe fn profile_reset() {
    for id in 1..=(*script_items.ptr()).ga_len {
        let si = script_item(id);
        if (*si).sn_prof_on {
            (*si).sn_prof_on = false;
            (*si).sn_pr_force = false;
            (*si).sn_pr_child = profile_zero();
            (*si).sn_pr_nest = 0;
            (*si).sn_pr_count = 0;
            (*si).sn_pr_total = profile_zero();
            (*si).sn_pr_self = profile_zero();
            (*si).sn_pr_start = profile_zero();
            (*si).sn_pr_children = profile_zero();
            ga_clear(&raw mut (*si).sn_prl_ga);
            (*si).sn_prl_start = profile_zero();
            (*si).sn_prl_children = profile_zero();
            (*si).sn_prl_wait = profile_zero();
            (*si).sn_prl_idx = -1;
            (*si).sn_prl_execed = 0;
        }
    }
    for uf in profiled_functions() {
        (*uf).uf_profiling = 0;
        (*uf).uf_tm_count = 0;
        (*uf).uf_tm_total = profile_zero();
        (*uf).uf_tm_self = profile_zero();
        (*uf).uf_tm_children = profile_zero();
        for i in 0..(*uf).uf_lines.ga_len as isize {
            *(*uf).uf_tml_count.offset(i) = 0;
            *(*uf).uf_tml_total.offset(i) = 0;
            *(*uf).uf_tml_self.offset(i) = 0;
        }
        (*uf).uf_tml_start = profile_zero();
        (*uf).uf_tml_children = profile_zero();
        (*uf).uf_tml_wait = profile_zero();
        (*uf).uf_tml_idx = -1;
        (*uf).uf_tml_execed = 0;
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

/// ExpandGeneric callback for `:profile` subcommands (fn pointer in the
/// cmdexpand context table).
pub unsafe extern "C" fn get_profile_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    usize::try_from(idx)
        .ok()
        .and_then(|i| PEXPAND_CMDS.get(i))
        .map_or(core::ptr::null_mut(), |s| s.as_ptr() as *mut c_char)
}

/// Command-line completion context for `:profile`.
pub unsafe fn set_context_in_profile_cmd(xp: *mut expand_T, arg: *const c_char) {
    // Default: expand subcommands.
    (*xp).xp_context = EXPAND_PROFILE;
    (*xp).xp_pattern = arg as *mut c_char;

    let end_subcmd = skiptowhite(arg);
    if *end_subcmd == 0 {
        return;
    }
    let len = end_subcmd.offset_from(arg) as usize;
    let subcmd = core::slice::from_raw_parts(arg as *const u8, len);
    if subcmd == b"start" || subcmd == b"file" {
        (*xp).xp_context = EXPAND_FILES;
        (*xp).xp_pattern = skipwhite(end_subcmd);
    } else if subcmd == b"func" {
        (*xp).xp_context = EXPAND_USER_FUNC;
        (*xp).xp_pattern = skipwhite(end_subcmd);
    } else {
        (*xp).xp_context = EXPAND_NOTHING;
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
pub unsafe fn prof_def_func() -> bool {
    let sid = (*current_sctx.ptr()).sc_sid;
    if sid > 0 {
        return (*script_item(sid)).sn_pr_force;
    }
    false
}

/// Start profiling function `fp`, allocating its per-line counters on
/// first use.
pub unsafe fn func_do_profile(fp: *mut ufunc_T) {
    let mut len = (*fp).uf_lines.ga_len;
    if (*fp).uf_prof_initialized == 0 {
        if len == 0 {
            len = 1; // avoid allocating zero bytes
        }
        (*fp).uf_tm_count = 0;
        (*fp).uf_tm_self = profile_zero();
        (*fp).uf_tm_total = profile_zero();
        if (*fp).uf_tml_count.is_null() {
            (*fp).uf_tml_count = xcalloc(len as usize, core::mem::size_of::<c_int>()) as *mut c_int;
        }
        if (*fp).uf_tml_total.is_null() {
            (*fp).uf_tml_total =
                xcalloc(len as usize, core::mem::size_of::<proftime_T>()) as *mut proftime_T;
        }
        if (*fp).uf_tml_self.is_null() {
            (*fp).uf_tml_self =
                xcalloc(len as usize, core::mem::size_of::<proftime_T>()) as *mut proftime_T;
        }
        (*fp).uf_tml_idx = -1;
        (*fp).uf_prof_initialized = 1;
    }
    (*fp).uf_profiling = 1;
}

/// Prepare for entering a child (another script/function/shell command)
/// whose time should not count towards the current one. Returns the wait
/// time to pass to [`prof_child_exit`].
pub unsafe fn prof_child_enter() -> proftime_T {
    let fc = get_current_funccal();
    if !fc.is_null() && (*(*fc).fc_func).uf_profiling != 0 {
        (*fc).fc_prof_child = profile_start();
    }
    script_prof_save()
}

/// Account the time spent in a child; pairs with [`prof_child_enter`],
/// `wait` being its return value.
pub unsafe fn prof_child_exit(wait: proftime_T) {
    let fc = get_current_funccal();
    if !fc.is_null() && (*(*fc).fc_func).uf_profiling != 0 {
        let mut child = profile_end((*fc).fc_prof_child);
        // Don't count waiting time.
        child = profile_sub_wait(wait, child);
        (*fc).fc_prof_child = child;
        (*(*fc).fc_func).uf_tm_children = profile_add((*(*fc).fc_func).uf_tm_children, child);
        (*(*fc).fc_func).uf_tml_children = profile_add((*(*fc).fc_func).uf_tml_children, child);
    }
    script_prof_restore(wait);
}

/// Called when starting to read a function line; the exestack lnum must be
/// correct. The line may turn out not to execute — the time is stored now,
/// counted only if [`func_line_exec`] follows.
pub unsafe fn func_line_start(cookie: *mut c_void) {
    let fp = (*(cookie as *mut funccall_T)).fc_func;
    let lnum = sourcing_lnum();
    if (*fp).uf_profiling != 0 && lnum >= 1 && lnum <= (*fp).uf_lines.ga_len as linenr_T {
        (*fp).uf_tml_idx = lnum as c_int - 1;
        // Skip continuation lines.
        while (*fp).uf_tml_idx > 0
            && (*((*fp).uf_lines.ga_data as *mut *mut c_char).offset((*fp).uf_tml_idx as isize))
                .is_null()
        {
            (*fp).uf_tml_idx -= 1;
        }
        (*fp).uf_tml_execed = 0;
        (*fp).uf_tml_start = profile_start();
        (*fp).uf_tml_children = profile_zero();
        (*fp).uf_tml_wait = PROF_WAIT_TIME.get();
    }
}

/// Called when actually executing a function line.
pub unsafe fn func_line_exec(cookie: *mut c_void) {
    let fp = (*(cookie as *mut funccall_T)).fc_func;
    if (*fp).uf_profiling != 0 && (*fp).uf_tml_idx >= 0 {
        (*fp).uf_tml_execed = 1;
    }
}

/// Called when done with a function line.
pub unsafe fn func_line_end(cookie: *mut c_void) {
    let fp = (*(cookie as *mut funccall_T)).fc_func;
    if (*fp).uf_profiling != 0 && (*fp).uf_tml_idx >= 0 {
        if (*fp).uf_tml_execed != 0 {
            let i = (*fp).uf_tml_idx as isize;
            *(*fp).uf_tml_count.offset(i) += 1;
            let mut spent = profile_end((*fp).uf_tml_start);
            spent = profile_sub_wait((*fp).uf_tml_wait, spent);
            (*fp).uf_tml_start = spent;
            *(*fp).uf_tml_total.offset(i) = profile_add(*(*fp).uf_tml_total.offset(i), spent);
            *(*fp).uf_tml_self.offset(i) =
                profile_self(*(*fp).uf_tml_self.offset(i), spent, (*fp).uf_tml_children);
        }
        (*fp).uf_tml_idx = -1;
    }
}

// ---------------------------------------------------------------------------
// Script profiling.

/// Start profiling script `si` (`:profile file` match on source).
pub unsafe fn profile_init(si: *mut scriptitem_T) {
    (*si).sn_pr_count = 0;
    (*si).sn_pr_total = profile_zero();
    (*si).sn_pr_self = profile_zero();
    ga_init(
        &raw mut (*si).sn_prl_ga,
        core::mem::size_of::<sn_prl_T>() as c_int,
        100,
    );
    (*si).sn_prl_idx = -1;
    (*si).sn_prof_on = true;
    (*si).sn_pr_nest = 0;
}

/// Save the wait time when starting to invoke another script or function;
/// returns the snapshot for [`script_prof_restore`].
pub unsafe fn script_prof_save() -> proftime_T {
    let sid = (*current_sctx.ptr()).sc_sid;
    if sid > 0 && sid <= (*script_items.ptr()).ga_len {
        let si = script_item(sid);
        if (*si).sn_prof_on {
            let nest = (*si).sn_pr_nest;
            (*si).sn_pr_nest += 1;
            if nest == 0 {
                (*si).sn_pr_child = profile_start();
            }
        }
    }
    PROF_WAIT_TIME.get()
}

/// Count time spent in children after invoking another script or function;
/// `wait` is what [`script_prof_save`] returned.
pub unsafe fn script_prof_restore(wait: proftime_T) {
    let Some(si) = current_script() else { return };
    if !(*si).sn_prof_on {
        return;
    }
    (*si).sn_pr_nest -= 1;
    if (*si).sn_pr_nest == 0 {
        let mut child = profile_end((*si).sn_pr_child);
        // Don't count wait time.
        child = profile_sub_wait(wait, child);
        (*si).sn_pr_child = child;
        (*si).sn_pr_children = profile_add((*si).sn_pr_children, child);
        (*si).sn_prl_children = profile_add((*si).sn_prl_children, child);
    }
}

/// Called when starting to read a script line; the exestack lnum must be
/// correct. See [`func_line_start`] for the execed dance.
pub unsafe fn script_line_start() {
    let Some(si) = current_script() else { return };
    let lnum = sourcing_lnum();
    if (*si).sn_prof_on && lnum >= 1 {
        // Grow the array before starting the timer, so that the time spent
        // here isn't counted.
        ga_grow(
            &raw mut (*si).sn_prl_ga,
            lnum as c_int - (*si).sn_prl_ga.ga_len,
        );
        (*si).sn_prl_idx = lnum - 1;
        while ((*si).sn_prl_ga.ga_len as linenr_T) <= (*si).sn_prl_idx
            && (*si).sn_prl_ga.ga_len < (*si).sn_prl_ga.ga_maxlen
        {
            // Zero counters for a line that was not used before.
            let pp = prl_item(si, (*si).sn_prl_ga.ga_len as isize);
            (*pp).snp_count = 0;
            (*pp).sn_prl_total = profile_zero();
            (*pp).sn_prl_self = profile_zero();
            (*si).sn_prl_ga.ga_len += 1;
        }
        (*si).sn_prl_execed = 0;
        (*si).sn_prl_start = profile_start();
        (*si).sn_prl_children = profile_zero();
        (*si).sn_prl_wait = PROF_WAIT_TIME.get();
    }
}

/// Called when actually executing a script line.
pub unsafe fn script_line_exec() {
    let Some(si) = current_script() else { return };
    if (*si).sn_prof_on && (*si).sn_prl_idx >= 0 {
        (*si).sn_prl_execed = 1;
    }
}

/// Called when done with a script line.
pub unsafe fn script_line_end() {
    let Some(si) = current_script() else { return };
    if (*si).sn_prof_on
        && (*si).sn_prl_idx >= 0
        && (*si).sn_prl_idx < (*si).sn_prl_ga.ga_len as linenr_T
    {
        if (*si).sn_prl_execed != 0 {
            let pp = prl_item(si, (*si).sn_prl_idx as isize);
            (*pp).snp_count += 1;
            let mut spent = profile_end((*si).sn_prl_start);
            spent = profile_sub_wait((*si).sn_prl_wait, spent);
            (*si).sn_prl_start = spent;
            (*pp).sn_prl_total = profile_add((*pp).sn_prl_total, spent);
            (*pp).sn_prl_self = profile_self((*pp).sn_prl_self, spent, (*si).sn_prl_children);
        }
        (*si).sn_prl_idx = -1;
    }
}

// ---------------------------------------------------------------------------
// Shared accessors for the editor's script/function tables.

/// Script item for 1-based script id `sid` (the transpiled `SCRIPT_ITEM`).
unsafe fn script_item(sid: c_int) -> *mut scriptitem_T {
    *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T).offset(sid as isize - 1)
}

/// The current script's item, if `current_sctx` points at a valid one.
unsafe fn current_script() -> Option<*mut scriptitem_T> {
    let sid = (*current_sctx.ptr()).sc_sid;
    if sid <= 0 || sid > (*script_items.ptr()).ga_len {
        return None;
    }
    Some(script_item(sid))
}

/// Line number being sourced/executed: the top of the exestack.
unsafe fn sourcing_lnum() -> linenr_T {
    let es = exestack.ptr();
    (*((*es).ga_data as *mut estack_T).offset((*es).ga_len as isize - 1)).es_lnum
}

/// Per-line counters of script `si` at `idx`.
unsafe fn prl_item(si: *mut scriptitem_T, idx: isize) -> *mut sn_prl_T {
    ((*si).sn_prl_ga.ga_data as *mut sn_prl_T).offset(idx)
}

/// All functions in the global function table with profiling data, in hash
/// table order.
unsafe fn profiled_functions() -> Vec<*mut ufunc_T> {
    let functbl = func_tbl_get();
    let mut found = Vec::new();
    let mut todo = (*functbl).ht_used;
    let mut hi = (*functbl).ht_array;
    while todo > 0 {
        if !(*hi).hi_key.is_null() && (*hi).hi_key != &raw const hash_removed as *mut c_char {
            todo -= 1;
            let fp = (*hi).hi_key.offset(-UF_NAME_OFFSET) as *mut ufunc_T;
            if (*fp).uf_prof_initialized != 0 {
                found.push(fp);
            }
        }
        hi = hi.offset(1);
    }
    found
}

// ---------------------------------------------------------------------------
// The report.

/// Write the profiling report to the `:profile start` file, if set.
pub fn profile_dump() {
    PROFILE_FNAME.with(|fname| {
        let Some(fname) = fname else { return };
        match File::create(OsStr::from_bytes(fname.to_bytes())) {
            Ok(file) => {
                let mut fd = BufWriter::new(file);
                // Like the C fprintf-based writer, I/O errors are ignored.
                // SAFETY: main thread; the tables the dump walks are live.
                let _ = unsafe { script_dump_profile(&mut fd) };
                let _ = unsafe { func_dump_profile(&mut fd) };
            }
            Err(_) => {
                // SAFETY: e_notopen is a NUL-terminated format with one %s.
                unsafe {
                    semsg(gettext(e_notopen.ptr() as *const c_char), fname.as_ptr());
                }
            }
        }
    });
}

/// `"name()"` with a newline, decoding the `<SNR>` mangling.
unsafe fn write_func_name(fd: &mut dyn Write, fp: *mut ufunc_T) -> io::Result<()> {
    let name = CStr::from_ptr(&raw const (*fp).uf_name as *const c_char).to_bytes();
    if name.first() == Some(&K_SPECIAL) {
        write!(fd, "<SNR>")?;
        fd.write_all(name.get(3..).unwrap_or_default())?;
    } else {
        fd.write_all(name)?;
    }
    write!(fd, "()\n")
}

/// One count/total/self report line. With `prefer_self` (function lines),
/// equal totals print only the self time; otherwise only the total.
fn prof_func_line(
    fd: &mut dyn Write,
    count: c_int,
    total: proftime_T,
    self_: proftime_T,
    prefer_self: bool,
) -> io::Result<()> {
    if count > 0 {
        write!(fd, "{count:5} ")?;
        if prefer_self && total == self_ {
            write!(fd, "           ")?;
        } else {
            write!(fd, "{} ", profile_msg_str(total))?;
        }
        if !prefer_self && total == self_ {
            write!(fd, "           ")?;
        } else {
            write!(fd, "{} ", profile_msg_str(self_))?;
        }
    } else {
        write!(fd, "                            ")?;
    }
    Ok(())
}

/// The top-20 list sorted on total or self time.
unsafe fn prof_sort_list(
    fd: &mut dyn Write,
    sorttab: &[*mut ufunc_T],
    title: &str,
    prefer_self: bool,
) -> io::Result<()> {
    write!(fd, "FUNCTIONS SORTED ON {title} TIME\n")?;
    write!(fd, "count  total (s)   self (s)  function\n")?;
    for &fp in sorttab.iter().take(20) {
        prof_func_line(
            fd,
            (*fp).uf_tm_count,
            (*fp).uf_tm_total,
            (*fp).uf_tm_self,
            prefer_self,
        )?;
        write!(fd, " ")?;
        write_func_name(fd, fp)?;
    }
    write!(fd, "\n")
}

/// Per-function sections plus the sorted lists.
unsafe fn func_dump_profile(fd: &mut dyn Write) -> io::Result<()> {
    let mut sorttab = profiled_functions();
    for &fp in &sorttab {
        write!(fd, "FUNCTION  ")?;
        write_func_name(fd, fp)?;
        if (*fp).uf_script_ctx.sc_sid != 0 {
            let mut should_free = false;
            let p = get_scriptname((*fp).uf_script_ctx, &raw mut should_free);
            write!(fd, "    Defined: ")?;
            fd.write_all(CStr::from_ptr(p).to_bytes())?;
            write!(fd, ":{}\n", (*fp).uf_script_ctx.sc_lnum)?;
            if should_free {
                xfree(p as *mut c_void);
            }
        }
        if (*fp).uf_tm_count == 1 {
            write!(fd, "Called 1 time\n")?;
        } else {
            write!(fd, "Called {} times\n", (*fp).uf_tm_count)?;
        }
        write!(fd, "Total time: {}\n", profile_msg_str((*fp).uf_tm_total))?;
        write!(fd, " Self time: {}\n", profile_msg_str((*fp).uf_tm_self))?;
        write!(fd, "\ncount  total (s)   self (s)\n")?;
        for i in 0..(*fp).uf_lines.ga_len as isize {
            let line = *((*fp).uf_lines.ga_data as *mut *mut c_char).offset(i);
            if line.is_null() {
                continue;
            }
            prof_func_line(
                fd,
                *(*fp).uf_tml_count.offset(i),
                *(*fp).uf_tml_total.offset(i),
                *(*fp).uf_tml_self.offset(i),
                true,
            )?;
            fd.write_all(CStr::from_ptr(line).to_bytes())?;
            write!(fd, "\n")?;
        }
        write!(fd, "\n")?;
    }
    if !sorttab.is_empty() {
        sorttab.sort_by(|&a, &b| profile_cmp((*a).uf_tm_total, (*b).uf_tm_total).cmp(&0));
        prof_sort_list(fd, &sorttab, "TOTAL", false)?;
        sorttab.sort_by(|&a, &b| profile_cmp((*a).uf_tm_self, (*b).uf_tm_self).cmp(&0));
        prof_sort_list(fd, &sorttab, "SELF", true)?;
    }
    Ok(())
}

/// Per-script sections: each profiled script's source lines annotated with
/// their counters.
unsafe fn script_dump_profile(fd: &mut dyn Write) -> io::Result<()> {
    for id in 1..=(*script_items.ptr()).ga_len {
        let si = script_item(id);
        if !(*si).sn_prof_on {
            continue;
        }
        write!(fd, "SCRIPT  ")?;
        fd.write_all(CStr::from_ptr((*si).sn_name).to_bytes())?;
        write!(fd, "\n")?;
        if (*si).sn_pr_count == 1 {
            write!(fd, "Sourced 1 time\n")?;
        } else {
            write!(fd, "Sourced {} times\n", (*si).sn_pr_count)?;
        }
        write!(fd, "Total time: {}\n", profile_msg_str((*si).sn_pr_total))?;
        write!(fd, " Self time: {}\n", profile_msg_str((*si).sn_pr_self))?;
        write!(fd, "\ncount  total (s)   self (s)\n")?;

        let sfd = os_fopen((*si).sn_name, b"r\0".as_ptr() as *const c_char);
        if sfd.is_null() {
            write!(fd, "Cannot open file!\n")?;
        } else {
            // Keep going till the end of file, so that trailing
            // continuation lines are listed.
            let mut buf = [0 as c_char; IOSIZE];
            let mut i = 0;
            while !vim_fgets(buf.as_mut_ptr(), IOSIZE as c_int, sfd) {
                // When a line has been truncated, append NL, taking care of
                // multibyte characters.
                if buf[IOSIZE - 2] != 0 && buf[IOSIZE - 2] != NL {
                    let mut n = IOSIZE - 2;
                    // Move back to the first byte of the char.
                    while n > 0 && (buf[n] as u8 & 0xc0) == 0x80 {
                        n -= 1;
                    }
                    buf[n] = NL;
                    buf[n + 1] = 0;
                }
                let pp = if i < (*si).sn_prl_ga.ga_len {
                    prl_item(si, i as isize)
                } else {
                    core::ptr::null_mut()
                };
                if !pp.is_null() && (*pp).snp_count > 0 {
                    write!(fd, "{:5} ", (*pp).snp_count)?;
                    if (*pp).sn_prl_total == (*pp).sn_prl_self {
                        write!(fd, "           ")?;
                    } else {
                        write!(fd, "{} ", profile_msg_str((*pp).sn_prl_total))?;
                    }
                    write!(fd, "{} ", profile_msg_str((*pp).sn_prl_self))?;
                } else {
                    write!(fd, "                            ")?;
                }
                fd.write_all(CStr::from_ptr(buf.as_ptr()).to_bytes())?;
                i += 1;
            }
            fclose(sfd);
        }
        write!(fd, "\n")?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// --startuptime.

/// When `time_start()` was called.
static G_START_TIME: GlobalCell<proftime_T> = GlobalCell::new(0);
/// Time of the previous event line, for the "elapsed" column.
static G_PREV_TIME: GlobalCell<proftime_T> = GlobalCell::new(0);
/// The setvbuf buffer handed to `time_fd`; freed at [`time_finish`].
static STARTUPTIME_BUF: GlobalCell<*mut c_char> = GlobalCell::new(core::ptr::null_mut());

/// Save the previous time before doing something that could nest (sourcing
/// a script from a script). Returns `(rel, start)`: the time elapsed so far
/// (to hand to [`time_pop`]) and the current time.
pub fn time_push() -> (proftime_T, proftime_T) {
    let now = profile_start();
    let rel = profile_sub(now, G_PREV_TIME.get());
    G_PREV_TIME.set(now);
    (rel, now)
}

/// Subtract the nested duration `tp` (from [`time_push`]) from the
/// previous-event time.
pub fn time_pop(tp: proftime_T) {
    G_PREV_TIME.set(G_PREV_TIME.get().wrapping_sub(tp));
}

/// `"%07.3lf"` milliseconds between `then` and `now`.
fn time_diff_str(then: proftime_T, now: proftime_T) -> String {
    format!("{:07.3}", profile_sub(now, then) as f64 / 1e6)
}

/// Append raw bytes to the startuptime log. No-op when `--startuptime` is
/// off or the bytes contain a NUL.
fn write_startup(bytes: &[u8]) {
    let fd = time_fd.get();
    if fd.is_null() {
        return;
    }
    if let Ok(line) = CString::new(bytes) {
        // SAFETY: fd is the open startuptime stream; "%s" consumes the one
        // string argument.
        unsafe { fprintf(fd, b"%s\0".as_ptr() as *const c_char, line.as_ptr()) };
    }
}

/// Write the startuptime report header and the first message. Must be
/// called once before [`time_msg`].
pub unsafe fn time_start(message: *const c_char) {
    if time_fd.get().is_null() {
        return;
    }
    let now = profile_start();
    G_START_TIME.set(now);
    G_PREV_TIME.set(now);
    write_startup(
        b"\ntimes in msec\n clock   self+sourced   self:  sourced script\n clock   elapsed:              other lines\n\n",
    );
    time_msg(message, core::ptr::null());
}

/// One startuptime line: clock, optional self+sourced (when `start` is
/// non-null, only for sourcing), elapsed, and the message.
pub unsafe fn time_msg(mesg: *const c_char, start: *const proftime_T) {
    if time_fd.get().is_null() {
        return;
    }
    let now = profile_start();
    let mut line = time_diff_str(G_START_TIME.get(), now);
    if !start.is_null() {
        line.push_str("  ");
        line.push_str(&time_diff_str(*start, now));
    }
    line.push_str("  ");
    line.push_str(&time_diff_str(G_PREV_TIME.get(), now));
    G_PREV_TIME.set(now);
    line.push_str(": ");
    let mut bytes = line.into_bytes();
    bytes.extend_from_slice(CStr::from_ptr(mesg).to_bytes());
    bytes.push(b'\n');
    write_startup(&bytes);
}

/// Open the `--startuptime` stream. The file is (potentially) written by
/// multiple nvim processes concurrently, so the report accumulates in a
/// full ("controlled") setvbuf buffer and is flushed to disk exactly once,
/// by [`time_finish`].
pub unsafe fn time_init(fname: *const c_char, proc_name: *const c_char) {
    const BUFSIZE: usize = 8192; // Big enough for the entire report.
    const _IOFBF: c_int = 0;
    time_fd.set(fopen(fname, b"a\0".as_ptr() as *const c_char));
    if time_fd.get().is_null() {
        fprintf(stderr, gettext(e_notopen.ptr() as *const c_char), fname);
        return;
    }
    STARTUPTIME_BUF.set(xmalloc(BUFSIZE + 1) as *mut c_char);
    let r = setvbuf(time_fd.get(), STARTUPTIME_BUF.get(), _IOFBF, BUFSIZE + 1);
    if r != 0 {
        xfree(STARTUPTIME_BUF.replace(core::ptr::null_mut()) as *mut c_void);
        fclose(time_fd.get());
        time_fd.set(core::ptr::null_mut());
        fprintf(
            stderr,
            b"time_init: setvbuf failed: %d %s\0".as_ptr() as *const c_char,
            r,
            uv_err_name(r),
        );
        return;
    }
    let mut header = b"--- Startup times for process: ".to_vec();
    header.extend_from_slice(CStr::from_ptr(proc_name).to_bytes());
    header.extend_from_slice(b" ---\n");
    write_startup(&header);
}

/// Flush the startuptime report to disk and close the stream.
pub fn time_finish() {
    if time_fd.get().is_null() {
        return;
    }
    debug_assert!(!STARTUPTIME_BUF.get().is_null());
    // SAFETY: the stream and its buffer were set up by time_init; nothing
    // touches them after the fd is cleared.
    unsafe {
        time_msg(
            b"--- NVIM STARTED ---\n\0".as_ptr() as *const c_char,
            core::ptr::null(),
        );
        fclose(time_fd.get());
        time_fd.set(core::ptr::null_mut());
        xfree(STARTUPTIME_BUF.replace(core::ptr::null_mut()) as *mut c_void);
    }
}
