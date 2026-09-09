//! The `argc()`, `argidx()`, `arglistid()` and `argv()` builtins.
//!
//! Each takes an optional window (and tab page) to ask about; `-1` in the
//! window slot means the global argument list rather than any window's.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::narrow::number_as_int;
use crate::types::{VAR_NUMBER, VAR_UNKNOWN};
use crate::winlayer::Win;

/// The argument list a `{winid}`-style argument selects: the current
/// window's when the argument is missing, the global one for `-1`, and
/// otherwise the named window's — `None` when there is no such window.
///
/// # Safety
///
/// `arg` must be a valid typval.
unsafe fn selected_arglist(arg: *mut TypVal) -> Option<*mut ArgList> {
    // SAFETY: caller contract; `find_win_by_nr_or_id` only reads the typval.
    if unsafe { (*arg).v_type } == VAR_UNKNOWN {
        return Some(win_alist(Win::current()));
    }
    if unsafe { (*arg).v_type } == VAR_NUMBER && unsafe { tv_get_number(arg) } == -1 as VarNumber {
        return Some(global_arglist());
    }
    unsafe { find_win_by_nr_or_id(arg) }.map(win_alist)
}

/// "argc()" function
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe fn f_argc(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract; a window that does not exist answers
    // -1, as it always has.
    let count = unsafe { selected_arglist(args) }.map_or(-1, alist_count);
    unsafe { (*result).write_number(VarNumber::from(count)) };
}

/// "argidx()" function
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe fn f_argidx(_args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract; curwin is valid.
    unsafe { (*result).write_number(VarNumber::from(Win::current().w_arg_idx)) };
}

/// "arglistid()" function
///
/// # Safety
///
/// Standard eval-function contract.
pub unsafe fn f_arglistid(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract -- the caller's argument array, which
    // holds both slots.
    let found = unsafe { find_tabwin(args.offset(0), args.offset(1)) };
    let id = match found {
        Some(wp) => {
            // SAFETY: a window the registry answered with, so it is live,
            // and every window has an argument list.
            let id = unsafe { (*win_alist(wp)).id };
            VarNumber::from(id)
        }
        None => -1 as VarNumber,
    };
    // SAFETY: the caller's return slot.
    unsafe { (*result).write_number(id) };
}

/// Return `count` argument entries as a List of file names. A null
/// `entries` still allocates the (empty) List, which is what `argv(-1)` on a
/// window that does not exist answers.
///
/// # Safety
///
/// `result` must be a valid return-value slot and `entries` hold `count`
/// argument list entries, or be null.
unsafe fn arglist_as_rettv(entries: *mut ArgEntry, count: c_int, result: *mut TypVal) {
    // SAFETY: caller contract; every entry has a name that outlives the copy
    // `tv_list_append_string` takes.
    unsafe { tv_list_alloc_ret(result, count as ptrdiff_t) };
    if entries.is_null() {
        return;
    }
    for idx in 0..count {
        let v_list2 = unsafe { (*result).list_or_null() };
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
pub unsafe fn f_argv(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract; both arguments are optional and are
    // only read once their type says they are present.
    if unsafe { (*args.offset(0)).v_type } == VAR_UNKNOWN {
        // No index: the whole current argument list.
        let (entries, count) = alist_entries(win_alist(Win::current()));
        unsafe { arglist_as_rettv(entries, count, result) };
        return;
    }
    // A window that does not exist leaves no list and a count of -1, so
    // every index is out of range.
    let (entries, count) =
        unsafe { selected_arglist(args.offset(1)) }.map_or((ptr::null_mut(), -1), alist_entries);
    unsafe { (*result).write_string(ptr::null_mut()) };
    let idx = number_as_int(unsafe { tv_get_number_chk(args.offset(0), ptr::null_mut()) });
    if !entries.is_null() && idx >= 0 && idx < count {
        unsafe { (*result).write_string(xstrdup(alist_name(entries.offset(idx as isize)))) };
    } else if idx == -1 {
        unsafe { arglist_as_rettv(entries, count, result) };
    }
}
