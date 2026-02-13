//! Function line profiling.
//!
//! Tracks per-line execution counts and timing for user-defined functions.

use std::os::raw::c_int;

use crate::types::{FuncCallHandle, UFuncHandle};
use crate::Proftime;

extern "C" {
    fn nvim_fc_get_func(fc: FuncCallHandle) -> UFuncHandle;
    fn nvim_ufunc_get_profiling(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_set_profiling(fp: UFuncHandle, val: c_int);
    fn nvim_ufunc_get_prof_initialized(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_set_prof_initialized(fp: UFuncHandle, val: c_int);
    fn nvim_ufunc_get_lines_len(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_get_tml_idx(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_set_tml_idx(fp: UFuncHandle, val: c_int);
    fn nvim_ufunc_get_tml_execed(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_set_tml_execed(fp: UFuncHandle, val: c_int);
    fn nvim_ufunc_get_tml_start(fp: UFuncHandle) -> Proftime;
    fn nvim_ufunc_set_tml_start(fp: UFuncHandle, val: Proftime);
    fn nvim_ufunc_get_tml_children(fp: UFuncHandle) -> Proftime;
    fn nvim_ufunc_set_tml_children(fp: UFuncHandle, val: Proftime);
    fn nvim_ufunc_get_tml_wait(fp: UFuncHandle) -> Proftime;
    fn nvim_ufunc_set_tml_wait(fp: UFuncHandle, val: Proftime);
    fn nvim_ufunc_set_tm_count(fp: UFuncHandle, val: c_int);
    fn nvim_ufunc_set_tm_total(fp: UFuncHandle, val: Proftime);
    fn nvim_ufunc_set_tm_self(fp: UFuncHandle, val: Proftime);
    fn nvim_ufunc_get_tml_count_i(fp: UFuncHandle, i: c_int) -> c_int;
    fn nvim_ufunc_set_tml_count_i(fp: UFuncHandle, i: c_int, val: c_int);
    fn nvim_ufunc_get_tml_total_i(fp: UFuncHandle, i: c_int) -> Proftime;
    fn nvim_ufunc_set_tml_total_i(fp: UFuncHandle, i: c_int, val: Proftime);
    fn nvim_ufunc_get_tml_self_i(fp: UFuncHandle, i: c_int) -> Proftime;
    fn nvim_ufunc_set_tml_self_i(fp: UFuncHandle, i: c_int, val: Proftime);
    fn nvim_ufunc_tml_count_is_null(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_tml_total_is_null(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_tml_self_is_null(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_alloc_tml_count(fp: UFuncHandle, len: c_int);
    fn nvim_ufunc_alloc_tml_total(fp: UFuncHandle, len: c_int);
    fn nvim_ufunc_alloc_tml_self(fp: UFuncHandle, len: c_int);
    fn nvim_ufunc_funcline_is_null(fp: UFuncHandle, idx: c_int) -> c_int;
    fn nvim_get_sourcing_lnum() -> c_int;
}

/// Called when starting to read a function line.
/// `sourcing_lnum` must be correct!
///
/// # Safety
///
/// `cookie` must be a valid `funccall_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_func_line_start(cookie: FuncCallHandle) {
    let fp = nvim_fc_get_func(cookie);

    if nvim_ufunc_get_profiling(fp) == 0 {
        return;
    }

    let lnum = nvim_get_sourcing_lnum();
    let lines_len = nvim_ufunc_get_lines_len(fp);

    if lnum < 1 || lnum > lines_len {
        return;
    }

    let mut idx = lnum - 1;
    // Skip continuation lines.
    while idx > 0 && nvim_ufunc_funcline_is_null(fp, idx) != 0 {
        idx -= 1;
    }
    nvim_ufunc_set_tml_idx(fp, idx);
    nvim_ufunc_set_tml_execed(fp, 0);
    nvim_ufunc_set_tml_start(fp, crate::timing::rs_profile_start());
    nvim_ufunc_set_tml_children(fp, crate::rs_profile_zero());
    nvim_ufunc_set_tml_wait(fp, crate::timing::rs_profile_get_wait_time());
}

/// Called when actually executing a function line.
///
/// # Safety
///
/// `cookie` must be a valid `funccall_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_func_line_exec(cookie: FuncCallHandle) {
    let fp = nvim_fc_get_func(cookie);

    if nvim_ufunc_get_profiling(fp) != 0 && nvim_ufunc_get_tml_idx(fp) >= 0 {
        nvim_ufunc_set_tml_execed(fp, 1);
    }
}

/// Called when done with a function line.
///
/// # Safety
///
/// `cookie` must be a valid `funccall_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_func_line_end(cookie: FuncCallHandle) {
    let fp = nvim_fc_get_func(cookie);

    if nvim_ufunc_get_profiling(fp) == 0 {
        return;
    }

    let idx = nvim_ufunc_get_tml_idx(fp);
    if idx < 0 {
        return;
    }

    if nvim_ufunc_get_tml_execed(fp) != 0 {
        nvim_ufunc_set_tml_count_i(fp, idx, nvim_ufunc_get_tml_count_i(fp, idx) + 1);

        let start = crate::timing::rs_profile_end(nvim_ufunc_get_tml_start(fp));
        let start = crate::rs_profile_sub_wait(nvim_ufunc_get_tml_wait(fp), start);

        let total = crate::rs_profile_add(nvim_ufunc_get_tml_total_i(fp, idx), start);
        nvim_ufunc_set_tml_total_i(fp, idx, total);

        let self_time = crate::rs_profile_self(
            nvim_ufunc_get_tml_self_i(fp, idx),
            start,
            nvim_ufunc_get_tml_children(fp),
        );
        nvim_ufunc_set_tml_self_i(fp, idx, self_time);
    }
    nvim_ufunc_set_tml_idx(fp, -1);
}

/// Start profiling function `fp`.
///
/// # Safety
///
/// `fp` must be a valid `ufunc_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_func_do_profile(fp: UFuncHandle) {
    let mut len = nvim_ufunc_get_lines_len(fp);

    if nvim_ufunc_get_prof_initialized(fp) == 0 {
        if len == 0 {
            len = 1; // avoid getting error for allocating zero bytes
        }
        nvim_ufunc_set_tm_count(fp, 0);
        nvim_ufunc_set_tm_self(fp, crate::rs_profile_zero());
        nvim_ufunc_set_tm_total(fp, crate::rs_profile_zero());

        if nvim_ufunc_tml_count_is_null(fp) != 0 {
            nvim_ufunc_alloc_tml_count(fp, len);
        }
        if nvim_ufunc_tml_total_is_null(fp) != 0 {
            nvim_ufunc_alloc_tml_total(fp, len);
        }
        if nvim_ufunc_tml_self_is_null(fp) != 0 {
            nvim_ufunc_alloc_tml_self(fp, len);
        }

        nvim_ufunc_set_tml_idx(fp, -1);
        nvim_ufunc_set_prof_initialized(fp, 1);
    }

    nvim_ufunc_set_profiling(fp, 1);
}
