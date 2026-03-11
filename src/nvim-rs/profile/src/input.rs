//! Input/wait profiling.
//!
//! Tracks time spent waiting for user input, so it can be excluded from
//! profiling totals.

use std::os::raw::c_int;

use crate::Proftime;

extern "C" {
    fn nvim_get_current_sctx_sid() -> c_int;
    fn nvim_get_script_items_len() -> c_int;
    fn nvim_script_item_get_pr_force(sid: c_int) -> c_int;
}

/// Time spent waiting for user input (to subtract from profiling).
/// Replaces `static proftime_T wait_time` in profile.c.
static mut WAIT_TIME: Proftime = 0;

/// Called when starting to wait for the user to type a character.
///
/// # Safety
///
/// Calls FFI functions and accesses mutable static.
#[export_name = "prof_input_start"]
pub unsafe extern "C" fn rs_prof_input_start() {
    let ptr = std::ptr::addr_of_mut!(WAIT_TIME);
    ptr.write(crate::timing::rs_profile_start());
}

/// Called when finished waiting for the user to type a character.
///
/// # Safety
///
/// Calls FFI functions and accesses mutable static.
#[export_name = "prof_input_end"]
pub unsafe extern "C" fn rs_prof_input_end() {
    let ptr = std::ptr::addr_of_mut!(WAIT_TIME);
    let wt = crate::timing::rs_profile_end(ptr.read());
    ptr.write(wt);
    let new_wait = crate::rs_profile_add(crate::timing::rs_profile_get_wait_time(), wt);
    crate::timing::rs_profile_set_wait(new_wait);
}

/// Returns true when a function defined in the current script should be
/// profiled.
///
/// # Safety
///
/// Calls FFI functions to access C globals.
#[export_name = "prof_def_func"]
pub unsafe extern "C" fn rs_prof_def_func() -> bool {
    let sid = nvim_get_current_sctx_sid();
    if sid > 0 && sid <= nvim_get_script_items_len() {
        return nvim_script_item_get_pr_force(sid) != 0;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wait_time_default() {
        unsafe {
            std::ptr::addr_of_mut!(WAIT_TIME).write(0);
            assert_eq!(std::ptr::addr_of!(WAIT_TIME).read(), 0);
        }
    }
}
