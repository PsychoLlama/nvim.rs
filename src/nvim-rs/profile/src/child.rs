//! Child profiling.
//!
//! Handles profiling state when entering/exiting child scripts or functions,
//! ensuring that time spent in children is correctly attributed.

use std::os::raw::c_int;

use crate::types::{FuncCallHandle, UFuncHandle};
use crate::Proftime;

extern "C" {
    fn nvim_get_current_funccal() -> FuncCallHandle;
    fn nvim_fc_get_func(fc: FuncCallHandle) -> UFuncHandle;
    fn nvim_ufunc_get_profiling(fp: UFuncHandle) -> c_int;
    fn nvim_fc_get_prof_child(fc: FuncCallHandle) -> Proftime;
    fn nvim_fc_set_prof_child(fc: FuncCallHandle, val: Proftime);
    fn nvim_ufunc_get_tm_children(fp: UFuncHandle) -> Proftime;
    fn nvim_ufunc_set_tm_children(fp: UFuncHandle, val: Proftime);
    fn nvim_ufunc_get_tml_children(fp: UFuncHandle) -> Proftime;
    fn nvim_ufunc_set_tml_children(fp: UFuncHandle, val: Proftime);
}

/// Prepare profiling for entering a child or something else that is not
/// counted for the script/function itself.
/// Should always be called in pair with `rs_prof_child_exit`.
///
/// # Safety
///
/// `tm` must be a valid pointer to a `proftime_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_prof_child_enter(tm: *mut Proftime) {
    let fc = nvim_get_current_funccal();

    if !fc.is_null() {
        let fp = nvim_fc_get_func(fc);
        if nvim_ufunc_get_profiling(fp) != 0 {
            nvim_fc_set_prof_child(fc, crate::timing::rs_profile_start());
        }
    }

    crate::script_line::rs_script_prof_save(tm);
}

/// Take care of time spent in a child.
/// Should always be called after `rs_prof_child_enter`.
///
/// # Safety
///
/// `tm` must be a valid pointer to a `proftime_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_prof_child_exit(tm: *mut Proftime) {
    let fc = nvim_get_current_funccal();

    if !fc.is_null() {
        let fp = nvim_fc_get_func(fc);
        if nvim_ufunc_get_profiling(fp) != 0 {
            let child = crate::timing::rs_profile_end(nvim_fc_get_prof_child(fc));
            // don't count waiting time
            let child = crate::rs_profile_sub_wait(*tm, child);
            nvim_fc_set_prof_child(fc, child);

            let tm_children = crate::rs_profile_add(nvim_ufunc_get_tm_children(fp), child);
            nvim_ufunc_set_tm_children(fp, tm_children);

            let tml_children = crate::rs_profile_add(nvim_ufunc_get_tml_children(fp), child);
            nvim_ufunc_set_tml_children(fp, tml_children);
        }
    }

    crate::script_line::rs_script_prof_restore(tm);
}
