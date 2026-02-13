//! Script line profiling.
//!
//! Tracks per-line execution counts and timing for sourced scripts,
//! and handles save/restore of profiling state when entering/leaving
//! child scripts or functions.

use std::os::raw::c_int;

use crate::Proftime;

extern "C" {
    fn nvim_get_current_sctx_sid() -> c_int;
    fn nvim_get_script_items_len() -> c_int;
    fn nvim_get_sourcing_lnum() -> c_int;

    // scriptitem_T profiling fields
    fn nvim_si_get_prof_on(sid: c_int) -> c_int;
    fn nvim_si_get_prl_idx(sid: c_int) -> c_int;
    fn nvim_si_set_prl_idx(sid: c_int, val: c_int);
    fn nvim_si_get_prl_execed(sid: c_int) -> c_int;
    fn nvim_si_set_prl_execed(sid: c_int, val: c_int);
    fn nvim_si_get_prl_start(sid: c_int) -> Proftime;
    fn nvim_si_set_prl_start(sid: c_int, val: Proftime);
    fn nvim_si_get_prl_children(sid: c_int) -> Proftime;
    fn nvim_si_set_prl_children(sid: c_int, val: Proftime);
    fn nvim_si_get_prl_wait(sid: c_int) -> Proftime;
    fn nvim_si_set_prl_wait(sid: c_int, val: Proftime);
    fn nvim_si_get_pr_nest(sid: c_int) -> c_int;
    fn nvim_si_set_pr_nest(sid: c_int, val: c_int);
    fn nvim_si_get_pr_child(sid: c_int) -> Proftime;
    fn nvim_si_set_pr_child(sid: c_int, val: Proftime);
    fn nvim_si_get_pr_children(sid: c_int) -> Proftime;
    fn nvim_si_set_pr_children(sid: c_int, val: Proftime);

    // garray_T ops for sn_prl_ga
    fn nvim_si_prl_ga_len(sid: c_int) -> c_int;
    fn nvim_si_prl_ga_set_len(sid: c_int, len: c_int);
    fn nvim_si_prl_ga_maxlen(sid: c_int) -> c_int;
    fn nvim_si_prl_ga_grow(sid: c_int, n: c_int);

    // PRL_ITEM field accessors
    fn nvim_si_prl_item_set_count(sid: c_int, idx: c_int, val: c_int);
    fn nvim_si_prl_item_set_total(sid: c_int, idx: c_int, val: Proftime);
    fn nvim_si_prl_item_get_self(sid: c_int, idx: c_int) -> Proftime;
    fn nvim_si_prl_item_set_self(sid: c_int, idx: c_int, val: Proftime);
    fn nvim_si_prl_item_get_count(sid: c_int, idx: c_int) -> c_int;
    fn nvim_si_prl_item_get_total(sid: c_int, idx: c_int) -> Proftime;
}

/// Returns true if the current script id is valid (in range).
unsafe fn script_id_valid(sid: c_int) -> bool {
    sid > 0 && sid <= nvim_get_script_items_len()
}

/// Called when starting to read a script line.
/// `sourcing_lnum` must be correct!
///
/// # Safety
///
/// Calls FFI functions to access C globals and script item fields.
#[no_mangle]
pub unsafe extern "C" fn rs_script_line_start() {
    let sid = nvim_get_current_sctx_sid();
    if !script_id_valid(sid) {
        return;
    }

    if nvim_si_get_prof_on(sid) == 0 {
        return;
    }

    let lnum = nvim_get_sourcing_lnum();
    if lnum < 1 {
        return;
    }

    // Grow the array before starting the timer.
    let ga_len = nvim_si_prl_ga_len(sid);
    nvim_si_prl_ga_grow(sid, lnum - ga_len);

    nvim_si_set_prl_idx(sid, lnum - 1);

    // Zero counters for lines that were not used before.
    let mut cur_len = nvim_si_prl_ga_len(sid);
    let prl_idx = nvim_si_get_prl_idx(sid);
    let maxlen = nvim_si_prl_ga_maxlen(sid);
    while cur_len <= prl_idx && cur_len < maxlen {
        nvim_si_prl_item_set_count(sid, cur_len, 0);
        nvim_si_prl_item_set_total(sid, cur_len, crate::rs_profile_zero());
        nvim_si_prl_item_set_self(sid, cur_len, crate::rs_profile_zero());
        cur_len += 1;
        nvim_si_prl_ga_set_len(sid, cur_len);
    }

    nvim_si_set_prl_execed(sid, 0);
    nvim_si_set_prl_start(sid, crate::timing::rs_profile_start());
    nvim_si_set_prl_children(sid, crate::rs_profile_zero());
    nvim_si_set_prl_wait(sid, crate::timing::rs_profile_get_wait_time());
}

/// Called when actually executing a script line.
///
/// # Safety
///
/// Calls FFI functions to access C globals and script item fields.
#[no_mangle]
pub unsafe extern "C" fn rs_script_line_exec() {
    let sid = nvim_get_current_sctx_sid();
    if !script_id_valid(sid) {
        return;
    }

    if nvim_si_get_prof_on(sid) != 0 && nvim_si_get_prl_idx(sid) >= 0 {
        nvim_si_set_prl_execed(sid, 1);
    }
}

/// Called when done with a script line.
///
/// # Safety
///
/// Calls FFI functions to access C globals and script item fields.
#[no_mangle]
pub unsafe extern "C" fn rs_script_line_end() {
    let sid = nvim_get_current_sctx_sid();
    if !script_id_valid(sid) {
        return;
    }

    if nvim_si_get_prof_on(sid) == 0 {
        return;
    }

    let prl_idx = nvim_si_get_prl_idx(sid);
    if prl_idx < 0 || prl_idx >= nvim_si_prl_ga_len(sid) {
        return;
    }

    if nvim_si_get_prl_execed(sid) != 0 {
        let count = nvim_si_prl_item_get_count(sid, prl_idx);
        nvim_si_prl_item_set_count(sid, prl_idx, count + 1);

        let start = crate::timing::rs_profile_end(nvim_si_get_prl_start(sid));
        let start = crate::rs_profile_sub_wait(nvim_si_get_prl_wait(sid), start);

        let total = crate::rs_profile_add(nvim_si_prl_item_get_total(sid, prl_idx), start);
        nvim_si_prl_item_set_total(sid, prl_idx, total);

        let self_time = crate::rs_profile_self(
            nvim_si_prl_item_get_self(sid, prl_idx),
            start,
            nvim_si_get_prl_children(sid),
        );
        nvim_si_prl_item_set_self(sid, prl_idx, self_time);
    }
    nvim_si_set_prl_idx(sid, -1);
}

/// Save time when starting to invoke another script or function.
///
/// # Safety
///
/// `tm` must be a valid pointer to a `proftime_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_script_prof_save(tm: *mut Proftime) {
    let sid = nvim_get_current_sctx_sid();
    if script_id_valid(sid) && nvim_si_get_prof_on(sid) != 0 {
        let nest = nvim_si_get_pr_nest(sid);
        nvim_si_set_pr_nest(sid, nest + 1);
        if nest == 0 {
            nvim_si_set_pr_child(sid, crate::timing::rs_profile_start());
        }
    }
    *tm = crate::timing::rs_profile_get_wait_time();
}

/// Count time spent in children after invoking another script or function.
///
/// # Safety
///
/// `tm` must be a valid pointer to a `proftime_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_script_prof_restore(tm: *const Proftime) {
    let sid = nvim_get_current_sctx_sid();
    if !script_id_valid(sid) {
        return;
    }

    if nvim_si_get_prof_on(sid) == 0 {
        return;
    }

    let nest = nvim_si_get_pr_nest(sid) - 1;
    nvim_si_set_pr_nest(sid, nest);

    if nest == 0 {
        let child = crate::timing::rs_profile_end(nvim_si_get_pr_child(sid));
        // don't count wait time
        let child = crate::rs_profile_sub_wait(*tm, child);
        nvim_si_set_pr_child(sid, child);

        let children = crate::rs_profile_add(nvim_si_get_pr_children(sid), child);
        nvim_si_set_pr_children(sid, children);

        let prl_children = crate::rs_profile_add(nvim_si_get_prl_children(sid), child);
        nvim_si_set_prl_children(sid, prl_children);
    }
}
