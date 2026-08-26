//! The `argc()`, `argidx()`, `arglistid()` and `argv()` builtins.
//!
//! Each takes an optional window (and tab page) to ask about; `-1` in the
//! window slot means the global argument list rather than any window's.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{VAR_NUMBER, VAR_STRING, VAR_UNKNOWN};
use crate::winlayer::Win;

/// The argument list a `{winid}`-style argument selects: the current
/// window's when the argument is missing, the global one for `-1`, and
/// otherwise the named window's — `None` when there is no such window.
///
/// # Safety
///
/// `arg` must be a valid typval.
unsafe fn selected_arglist(arg: *mut typval_T) -> Option<*mut alist_T> {
    // SAFETY: caller contract; `find_win_by_nr_or_id` only reads the typval.
    if unsafe { (*arg).v_type } == VAR_UNKNOWN {
        return Some(win_alist(curwin.get()));
    }
    if unsafe { (*arg).v_type } == VAR_NUMBER && unsafe { tv_get_number(arg) } == -1 as varnumber_T
    {
        return Some(global_arglist());
    }
    unsafe { find_win_by_nr_or_id(arg) }.map(|wp| win_alist(wp.raw()))
}

/// "argc()" function
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe fn f_argc(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract; a window that does not exist answers
    // -1, as it always has.
    let count = unsafe { selected_arglist(argvars) }.map_or(-1, alist_count);
    unsafe { (*rettv).vval.v_number = count as varnumber_T };
}

/// "argidx()" function
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe fn f_argidx(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract; curwin is valid.
    unsafe { (*rettv).vval.v_number = cur_win().w_arg_idx as varnumber_T };
}

/// "arglistid()" function
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe fn f_arglistid(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract -- the caller's argument array, which
    // holds both slots.
    let found = unsafe { find_tabwin(argvars.offset(0), argvars.offset(1)) };
    let id = match found {
        Some(wp) => {
            // SAFETY: a window the registry answered with, so it is live,
            // and every window has an argument list.
            let id = unsafe { (*win_alist(wp.raw())).id };
            id as varnumber_T
        }
        None => -1 as varnumber_T,
    };
    // SAFETY: the caller's return slot.
    unsafe { (*rettv).vval.v_number = id };
}

/// Return `count` argument entries as a List of file names. A null
/// `entries` still allocates the (empty) List, which is what `argv(-1)` on a
/// window that does not exist answers.
///
/// # Safety
///
/// `rettv` must be a valid return-value slot and `entries` hold `count`
/// argument list entries, or be null.
unsafe fn arglist_as_rettv(entries: *mut aentry_T, count: c_int, rettv: *mut typval_T) {
    // SAFETY: caller contract; every entry has a name that outlives the copy
    // `tv_list_append_string` takes.
    unsafe { tv_list_alloc_ret(rettv, count as ptrdiff_t) };
    if entries.is_null() {
        return;
    }
    for idx in 0..count {
        let v_list2 = unsafe { (*rettv).vval.v_list };
        let str = unsafe { alist_name(entries.offset(idx as isize)) };
        let len = -1 as ssize_t;
        unsafe { tv_list_append_string(v_list2, str, len) };
    }
}

/// "argv()" function
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe fn f_argv(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract; both arguments are optional and are
    // only read once their type says they are present.
    if unsafe { (*argvars.offset(0)).v_type } == VAR_UNKNOWN {
        // No index: the whole current argument list.
        let (entries, count) = alist_entries(win_alist(curwin.get()));
        unsafe { arglist_as_rettv(entries, count, rettv) };
        return;
    }
    // A window that does not exist leaves no list and a count of -1, so
    // every index is out of range.
    let (entries, count) =
        unsafe { selected_arglist(argvars.offset(1)) }.map_or((ptr::null_mut(), -1), alist_entries);
    unsafe { (*rettv).v_type = VAR_STRING };
    unsafe { (*rettv).vval.v_string = ptr::null_mut() };
    let idx = unsafe { tv_get_number_chk(argvars.offset(0), ptr::null_mut()) } as c_int;
    if !entries.is_null() && idx >= 0 && idx < count {
        unsafe { (*rettv).vval.v_string = xstrdup(alist_name(entries.offset(idx as isize))) };
    } else if idx == -1 {
        unsafe { arglist_as_rettv(entries, count, rettv) };
    }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
