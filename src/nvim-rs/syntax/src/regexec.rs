//! Regex execution for syntax highlighting.
//!
//! This module owns the logic previously in:
//! - `syn_regexec` (profiling, timeout, b_syn_slow)
//! - `nvim_syn_regexec_pat` (execute regex on a pattern by index, used by region.rs)
//! - `nvim_syn_regexec_by_idx` (same, slightly different API, used by current_attr.rs)
//! - `nvim_syn_exec_linecont` (execute linecont regex, used by line_init.rs)
//!
//! The `regmmatch_T` struct stays opaque in C. A thin C helper
//! `nvim_syn_do_regexec` handles struct setup and `vim_regexec_multi` call.

use std::ffi::{c_int, c_void};

use crate::synblock_struct::{synblock_mut, synblock_ref};
use crate::types::{SynBlockHandle, SynPatHandle, WinHandle};

// ============================================================================
// FFI declarations
// ============================================================================

extern "C" {
    // Thin C helper: set up regmmatch_T, call vim_regexec_multi, extract results.
    // Returns 1 on match, 0 on no match, -1 if regprog is NULL.
    fn nvim_syn_do_regexec(
        regprog: *mut c_void,
        ic: c_int,
        lnum: c_int,
        col: c_int,
        out_s_lnum: *mut c_int,
        out_s_col: *mut c_int,
        out_e_lnum: *mut c_int,
        out_e_col: *mut c_int,
        out_regprog: *mut *mut c_void,
        out_timed_out: *mut c_int,
    ) -> c_int;

    // syn_time_on flag

    // syn_win access for b_syn_slow
    fn nvim_syn_get_win() -> WinHandle;
    fn nvim_win_get_synblock(win: WinHandle) -> SynBlockHandle;

    // Profiling (from profile crate, exported via #[export_name])
    fn profile_start() -> u64;
    fn profile_end(tm: u64) -> u64;

    // Profile arithmetic (from profile crate)
    fn profile_add(tm1: u64, tm2: u64) -> u64;
    fn profile_cmp(tm1: u64, tm2: u64) -> c_int;

    // (synpat_T setters removed -- use direct repr(C) field access)

    // Current synblock pattern access
    fn nvim_syn_get_syn_block() -> SynBlockHandle;
    fn nvim_synblock_get_pattern(block: SynBlockHandle, idx: c_int) -> SynPatHandle;

    // linecont pattern accessors
    fn nvim_syn_block_get_linecont_prog() -> *mut c_void;
    fn nvim_syn_block_set_linecont_prog(prog: *mut c_void);
    fn nvim_syn_block_get_linecont_ic() -> c_int;
    fn nvim_syn_block_get_linecont_time_ptr() -> *mut c_void;

    // (nvim_syn_save_chartab/restore_chartab deleted: call Rust directly)

    // message for b_syn_slow notification
    fn msg(s: *const i8, hl_id: c_int) -> c_int;
}

static MSG_REDRAWTIME_EXCEEDED: &[u8] = b"'redrawtime' exceeded, syntax highlighting disabled\0";

// ============================================================================
// Core regex execution with profiling and timeout handling
// ============================================================================

/// Core helper: execute regex via C helper, handle profiling and b_syn_slow.
///
/// `regprog`    - the regprog pointer from the synpat (may be NULL)
/// `ic`         - ignore-case flag
/// `lnum`, `col` - search position
/// `st_ptr`     - pointer to syn_time_T (NULL if not timing)
/// `out_regprog` - receives the (potentially updated) regprog after execution
///
/// Returns `Some((s_lnum, s_col, e_lnum, e_col))` on match, `None` otherwise.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
unsafe fn syn_regexec_impl(
    regprog: *mut c_void,
    ic: c_int,
    lnum: c_int,
    col: c_int,
    st_ptr: *mut c_void,
    out_regprog: &mut *mut c_void,
) -> Option<(i32, i32, i32, i32)> {
    let syn_time_on = crate::statics::SYN_TIME_ON != 0;

    let pt = if syn_time_on { profile_start() } else { 0 };

    let mut s_lnum: c_int = 0;
    let mut s_col: c_int = 0;
    let mut e_lnum: c_int = 0;
    let mut e_col: c_int = 0;
    let mut new_regprog: *mut c_void = std::ptr::null_mut();
    let mut timed_out: c_int = 0;

    let r = nvim_syn_do_regexec(
        regprog,
        ic,
        lnum,
        col,
        &mut s_lnum,
        &mut s_col,
        &mut e_lnum,
        &mut e_col,
        &mut new_regprog,
        &mut timed_out,
    );

    *out_regprog = new_regprog;

    if syn_time_on && !st_ptr.is_null() {
        let elapsed = profile_end(pt);
        let st = &mut *(st_ptr as *mut crate::ffi_types::SynTime);
        st.total = profile_add(st.total, elapsed);
        if profile_cmp(elapsed, st.slowest) < 0 {
            st.slowest = elapsed;
        }
        st.count += 1;
        if r > 0 {
            st.match_ += 1;
        }
    }

    let syn_win = nvim_syn_get_win();
    let syn_win_block = if syn_win.is_null() {
        SynBlockHandle(std::ptr::null_mut())
    } else {
        nvim_win_get_synblock(syn_win)
    };
    let b_syn_slow = if syn_win_block.is_null() {
        0
    } else {
        synblock_ref(syn_win_block).b_syn_slow as c_int
    };
    if timed_out != 0 && b_syn_slow == 0 {
        if !syn_win_block.is_null() {
            synblock_mut(syn_win_block).b_syn_slow = true;
        }
        msg(MSG_REDRAWTIME_EXCEEDED.as_ptr().cast(), 0);
    }

    if r == 1 {
        Some((s_lnum, s_col, e_lnum, e_col))
    } else {
        None
    }
}

// ============================================================================
// Public functions replacing the deleted C functions
// ============================================================================

/// Execute regex on a synblock pattern by index.
/// Returns 1 if matched, 0 if not. Fills out-params with match positions.
///
/// Replaces C `nvim_syn_regexec_pat` (called from region.rs).
///
/// # Safety
/// Accesses C global state.
pub unsafe fn syn_regexec_pat(
    idx: i32,
    lnum: i32,
    col: i32,
    start_lnum: *mut c_int,
    start_col: *mut c_int,
    end_lnum: *mut c_int,
    end_col: *mut c_int,
) -> c_int {
    let block = nvim_syn_get_syn_block();
    let pat_count = if block.is_null() {
        0
    } else {
        synblock_ref(block).b_syn_patterns.ga_len
    };
    if block.is_null() || idx < 0 || idx >= pat_count {
        return 0;
    }

    let pat = nvim_synblock_get_pattern(block, idx);
    if pat.is_null() {
        return 0;
    }

    let ic = (*pat.as_ptr()).sp_ic;
    let regprog = (*pat.as_ptr()).sp_prog;
    let st_ptr = &raw mut (*pat.as_ptr()).sp_time as *mut c_void;
    let mut new_regprog: *mut c_void = std::ptr::null_mut();

    let result = syn_regexec_impl(regprog, ic, lnum, col, st_ptr, &mut new_regprog);
    (*pat.as_ptr()).sp_prog = new_regprog;

    if let Some((sl, sc, el, ec)) = result {
        if !start_lnum.is_null() {
            *start_lnum = sl;
        }
        if !start_col.is_null() {
            *start_col = sc;
        }
        if !end_lnum.is_null() {
            *end_lnum = el;
        }
        if !end_col.is_null() {
            *end_col = ec;
        }
        1
    } else {
        0
    }
}

/// Execute regex on a synblock pattern by index.
/// Returns 1 if matched (with positions set) or 0 if not.
///
/// Replaces C `nvim_syn_regexec_by_idx` (called from current_attr.rs).
///
/// # Safety
/// Accesses C global state.
pub unsafe fn syn_regexec_by_idx(
    idx: i32,
    lnum: i32,
    col: i32,
    s_lnum: *mut c_int,
    s_col: *mut c_int,
    e_lnum: *mut c_int,
    e_col: *mut c_int,
) -> c_int {
    let block = nvim_syn_get_syn_block();
    if block.is_null() {
        return 0;
    }

    let pat = nvim_synblock_get_pattern(block, idx);
    if pat.is_null() {
        return 0;
    }

    let ic = (*pat.as_ptr()).sp_ic;
    let regprog = (*pat.as_ptr()).sp_prog;
    let st_ptr = &raw mut (*pat.as_ptr()).sp_time as *mut c_void;
    let mut new_regprog: *mut c_void = std::ptr::null_mut();

    let result = syn_regexec_impl(regprog, ic, lnum, col, st_ptr, &mut new_regprog);
    (*pat.as_ptr()).sp_prog = new_regprog;

    if let Some((sl, sc, el, ec)) = result {
        *s_lnum = sl;
        *s_col = sc;
        *e_lnum = el;
        *e_col = ec;
        1
    } else {
        0
    }
}

/// Execute regex for the linecont pattern at the given lnum.
/// Returns nonzero on match.
///
/// Replaces C `nvim_syn_exec_linecont` (called from line_init.rs).
///
/// # Safety
/// Accesses C global state.
pub unsafe fn syn_exec_linecont(lnum: i32) -> c_int {
    let regprog = nvim_syn_block_get_linecont_prog();
    if regprog.is_null() {
        return 0;
    }

    let ic = nvim_syn_block_get_linecont_ic();
    let st_ptr = nvim_syn_block_get_linecont_time_ptr();

    let mut buf_chartab = [0i8; 32];
    crate::line_init::rs_save_chartab(buf_chartab.as_mut_ptr());

    let mut new_regprog: *mut c_void = std::ptr::null_mut();
    let result = syn_regexec_impl(regprog, ic, lnum, 0, st_ptr, &mut new_regprog);
    nvim_syn_block_set_linecont_prog(new_regprog);

    crate::line_init::rs_restore_chartab(buf_chartab.as_mut_ptr());

    if result.is_some() {
        1
    } else {
        0
    }
}
