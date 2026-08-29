//! The Vimscript function bridges, and the garbage collector.
//!
//! [`f_getqflist`]/[`f_setqflist`] and their location-list twins unpack
//! their arguments and call into `getprops`/`setprops`.
//! [`set_ref_in_quickfix`] is the other half: every list's context and
//! every entry's user data is a `typval_T` the collector has to see.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::eval::typval::NumBuf;
use crate::semsg_c;
use crate::types::{
    NUL, Refcount, VAR_DICT, VAR_FLOAT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
    kListLenMayKnow,
};
use crate::winlayer::Win;
use core::ffi::{c_char, c_int};
use core::ptr;

/// The parsed global `'quickfixtextfunc'`. A list-local one lives in
/// `qf_info_T::qf_qftf_cb`.
///
/// The address, because every operation the tree has on a callback —
/// parsing an option into it, marking it for the collector, copying it,
/// calling it — takes a `*mut Callback`.
pub(super) fn global_qftf() -> *mut Callback {
    qftf_cb.ptr()
}

/// Whether a value can hold a reference at all. Numbers, strings and floats
/// own nothing, so the collector never has to walk into one.
///
/// # Safety
///
/// `tv` must be a live value.
unsafe fn holds_references(tv: *const typval_T) -> bool {
    // SAFETY: the caller's value.
    unsafe { !matches!((*tv).v_type, VAR_NUMBER | VAR_STRING | VAR_FLOAT) }
}

/// Mark the `user_data` of every entry of every list on the stack. Answers
/// whether the walk should be given up, which is what `set_ref_in_item`
/// reports when it finds a cycle it cannot follow.
///
/// # Safety
///
/// `qi` must be a live stack.
unsafe fn mark_quickfix_user_data(qi: *mut qf_info_T, copy_id: c_int) -> bool {
    // SAFETY: forwarded from the caller.
    let mut aborted = false;
    let mut i = 0;
    while i < unsafe { (*qi).max_count() } && !aborted {
        let qfl = unsafe { qf_get_list(qi, i) };
        if unsafe { (*qfl).qf_has_user_data } {
            let mut qfp = unsafe { (*qfl).qf_start };
            let mut j = 1;
            while !got_int.get() && j <= unsafe { (*qfl).qf_count } && !qfp.is_null() {
                // The value is inline in the entry, so it is always
                // there; only its type says whether to walk into it.
                let user_data = unsafe { &raw mut (*qfp).qf_user_data };
                if unsafe { holds_references(user_data) } {
                    aborted = aborted
                        || unsafe {
                            set_ref_in_item(user_data, copy_id, ptr::null_mut(), ptr::null_mut())
                        };
                }
                j += 1;
                qfp = unsafe { (*qfp).qf_next };
            }
        }
        i += 1;
    }
    aborted
}

/// Mark the context value and the `'quickfixtextfunc'` callback of every
/// list on the stack.
///
/// # Safety
///
/// `qi` must be a live stack.
unsafe fn mark_quickfix_ctx(qi: *mut qf_info_T, copy_id: c_int) -> bool {
    // SAFETY: forwarded from the caller.
    let mut aborted = false;
    let mut i = 0;
    while i < unsafe { (*qi).max_count() } && !aborted {
        let ctx = unsafe { (*qf_get_list(qi, i)).qf_ctx };
        if !ctx.is_null() && unsafe { holds_references(ctx) } {
            aborted = unsafe { set_ref_in_item(ctx, copy_id, ptr::null_mut(), ptr::null_mut()) };
        }
        let cb = unsafe { &raw mut (*qf_get_list(qi, i)).qf_qftf_cb };
        aborted = aborted
            || unsafe { set_ref_in_callback(cb, copy_id, ptr::null_mut(), ptr::null_mut()) };
        i += 1;
    }
    aborted
}

/// Mark everything the quickfix stack and every location list stack hold,
/// so that the garbage collector does not free it.
///
/// # Safety
///
/// The editor must be initialised.
pub unsafe fn set_ref_in_quickfix(copy_id: c_int) -> bool {
    // SAFETY: the stacks and window lists are only read.
    let ql = QfStack::Global.raw();
    if unsafe { mark_quickfix_ctx(ql, copy_id) }
        || unsafe { mark_quickfix_user_data(ql, copy_id) }
        || unsafe { set_ref_in_callback(global_qftf(), copy_id, ptr::null_mut(), ptr::null_mut()) }
    {
        return true;
    }

    // Every window may own a location list, and a location list window
    // may be the last thing referring to one.
    let aborting = |win: Win| {
        let own = win.w_llist;
        if !own.is_null()
            && (unsafe { mark_quickfix_ctx(own, copy_id) }
                || unsafe { mark_quickfix_user_data(own, copy_id) })
        {
            return true;
        }
        let shown = win.w_llist_ref;
        if is_ll_window(win) && unsafe { (*shown).qf_refcount } == Refcount::ONE {
            return unsafe { mark_quickfix_ctx(shown, copy_id) }
                || unsafe { mark_quickfix_user_data(shown, copy_id) };
        }
        false
    };
    unsafe { find_tab_win(aborting) }.is_some()
}

/// The body of `getqflist()` and `getloclist()`: with no `what` argument the
/// answer is the list of entries, otherwise the dictionary `what` asks for.
///
/// # Safety
///
/// The two values must be live.
unsafe fn get_qf_loc_list(
    is_qf: bool,
    wp: Option<Win>,
    what_arg: *mut typval_T,
    rettv: *mut typval_T,
) {
    // SAFETY: forwarded from the caller.
    if unsafe { (*what_arg).v_type } == VAR_UNKNOWN {
        unsafe { tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t) };
        if is_qf || wp.is_some() {
            // No list, or an empty one, is an empty answer, not an error.
            let _ = unsafe { get_errorlist(ptr::null_mut(), wp, -1, 0, (*rettv).vval.v_list) };
        }
        return;
    }

    unsafe { tv_dict_alloc_ret(rettv) };
    if !is_qf && wp.is_none() {
        return;
    }
    if unsafe { (*what_arg).v_type } != VAR_DICT {
        emsg(gettext(e_dictreq));
        return;
    }
    let d = unsafe { (*what_arg).vval.v_dict };
    if !d.is_null() {
        // A request that names nothing readable answers the empty
        // dictionary that is already in `rettv`.
        let _ = unsafe { qf_get_properties(wp, d, (*rettv).vval.v_dict) };
    }
}

/// `getloclist({winnr} [, {what}])`.
///
/// # Safety
///
/// Called through the Vimscript function table with its argument array.
pub unsafe fn f_getloclist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's argument array holds at least two values.
    unsafe { get_qf_loc_list(false, find_win_by_nr_or_id(argvars), argvars.add(1), rettv) };
}

/// `getqflist([{what}])`.
///
/// # Safety
///
/// Called through the Vimscript function table with its argument array.
pub unsafe fn f_getqflist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's argument array holds at least one value.
    unsafe { get_qf_loc_list(true, None, argvars, rettv) }
}

/// The body of `setqflist()` and `setloclist()`: a list of entries, an
/// optional action character, and an optional title or `what` dictionary.
/// Answers through `rettv`, which is −1 for every rejection.
///
/// # Safety
///
/// `wp` must be null or a live window, and `args` hold three values.
unsafe fn set_qf_ll_list(wp: Option<Win>, args: *mut typval_T, rettv: *mut typval_T) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    /// Set while `set_errorlist` runs, because an autocommand it fires may
    /// call `setqflist()` again and the list would be pulled out from under
    /// the outer call.
    static RECURSIVE: GlobalCell<c_int> = GlobalCell::new(0);

    // SAFETY: forwarded from the caller.
    unsafe { (*rettv).vval.v_number = -1 };

    let list_arg = args;
    if unsafe { (*list_arg).v_type } != VAR_LIST {
        emsg(gettext(e_listreq));
        return;
    }
    if RECURSIVE.get() != 0 {
        emsg(gettext(e_au_recursive));
        return;
    }

    let mut action = ' ' as c_char;
    let mut title: *const c_char = ptr::null();
    let mut what: *mut dict_T = ptr::null_mut();

    let action_arg = unsafe { args.add(1) };
    if unsafe { (*action_arg).v_type } != VAR_UNKNOWN {
        if unsafe { (*action_arg).v_type } != VAR_STRING {
            emsg(gettext(e_string_required));
            return;
        }
        // Never null: the value is a string, which is what
        // `tv_get_string_chk` fails on anything else for.
        let act = unsafe { numbuf.string_chk(action_arg) };
        let known = matches!(unsafe { *act } as u8, b'a' | b'r' | b'u' | b' ' | b'f');
        if !known || unsafe { *act.add(1) } as c_int != NUL {
            unsafe { semsg_c!(gettext(c"E927: Invalid action: '%s'"), act) };
            return;
        }
        action = unsafe { *act };

        let what_arg = unsafe { args.add(2) };
        if unsafe { (*what_arg).v_type } == VAR_STRING {
            title = unsafe { numbuf2.string_chk(what_arg) };
            if title.is_null() {
                return;
            }
        } else if unsafe { (*what_arg).v_type } == VAR_DICT
            && !unsafe { (*what_arg).vval.v_dict }.is_null()
        {
            what = unsafe { (*what_arg).vval.v_dict };
        } else if unsafe { (*what_arg).v_type } != VAR_UNKNOWN {
            emsg(gettext(e_dictreq));
            return;
        }
    }

    if title.is_null() {
        title = if wp.is_none() {
            c":setqflist()".as_ptr()
        } else {
            c":setloclist()".as_ptr()
        };
    }

    RECURSIVE.set(RECURSIVE.get() + 1);
    let l = unsafe { (*list_arg).vval.v_list };
    if unsafe { set_errorlist(wp, l, action as c_int, title.cast_mut(), what) }.is_ok() {
        unsafe { (*rettv).vval.v_number = 0 };
    }
    RECURSIVE.set(RECURSIVE.get() - 1);
}

/// `setloclist({winnr}, {list} [, {action} [, {what}]])`.
///
/// # Safety
///
/// Called through the Vimscript function table with its argument array.
pub unsafe fn f_setloclist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's argument array holds at least four values.
    unsafe { (*rettv).vval.v_number = -1 };
    if let Some(win) = unsafe { find_win_by_nr_or_id(argvars) } {
        unsafe { set_qf_ll_list(Some(win), argvars.add(1), rettv) };
    }
}

/// `setqflist({list} [, {action} [, {what}]])`.
///
/// # Safety
///
/// Called through the Vimscript function table with its argument array.
pub unsafe fn f_setqflist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's argument array holds at least three values.
    unsafe { set_qf_ll_list(None, argvars, rettv) }
}
