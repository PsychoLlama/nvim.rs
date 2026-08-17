//! The Vimscript function bridges, and the garbage collector.
//!
//! [`f_getqflist`]/[`f_setqflist`] and their location-list twins unpack
//! their arguments and call into `getprops`/`setprops`.
//! [`set_ref_in_quickfix`] is the other half: every list's context and
//! every entry's user data is a `typval_T` the collector has to see.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{
    VAR_DICT, VAR_FLOAT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, kListLenMayKnow,
};
use core::ffi::{c_char, c_int};
use core::ptr;

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
    unsafe {
        let mut aborted = false;
        let mut i = 0;
        while i < (*qi).max_count() && !aborted {
            let qfl = qf_get_list(qi, i);
            if (*qfl).qf_has_user_data {
                let mut qfp = (*qfl).qf_start;
                let mut j = 1;
                while !got_int.get() && j <= (*qfl).qf_count && !qfp.is_null() {
                    // The value is inline in the entry, so it is always
                    // there; only its type says whether to walk into it.
                    let user_data = &raw mut (*qfp).qf_user_data;
                    if holds_references(user_data) {
                        aborted = aborted
                            || set_ref_in_item(
                                user_data,
                                copy_id,
                                ptr::null_mut(),
                                ptr::null_mut(),
                            );
                    }
                    j += 1;
                    qfp = (*qfp).qf_next;
                }
            }
            i += 1;
        }
        aborted
    }
}

/// Mark the context value and the `'quickfixtextfunc'` callback of every
/// list on the stack.
///
/// # Safety
///
/// `qi` must be a live stack.
unsafe fn mark_quickfix_ctx(qi: *mut qf_info_T, copy_id: c_int) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut aborted = false;
        let mut i = 0;
        while i < (*qi).max_count() && !aborted {
            let ctx = (*qf_get_list(qi, i)).qf_ctx;
            if !ctx.is_null() && holds_references(ctx) {
                aborted = set_ref_in_item(ctx, copy_id, ptr::null_mut(), ptr::null_mut());
            }
            let cb = &raw mut (*qf_get_list(qi, i)).qf_qftf_cb;
            aborted = aborted || set_ref_in_callback(cb, copy_id, ptr::null_mut(), ptr::null_mut());
            i += 1;
        }
        aborted
    }
}

/// Mark everything the quickfix stack and every location list stack hold,
/// so that the garbage collector does not free it.
///
/// # Safety
///
/// The editor must be initialised.
pub unsafe fn set_ref_in_quickfix(copy_id: c_int) -> bool {
    // SAFETY: the stacks and window lists are only read.
    unsafe {
        debug_assert!(!ql_info.get().is_null());
        if mark_quickfix_ctx(ql_info.get(), copy_id)
            || mark_quickfix_user_data(ql_info.get(), copy_id)
            || set_ref_in_callback(qftf_cb.ptr(), copy_id, ptr::null_mut(), ptr::null_mut())
        {
            return true;
        }

        // Every window may own a location list, and a location list window
        // may be the last thing referring to one.
        let aborting = |win: *mut win_T| {
            let own = (*win).w_llist;
            if !own.is_null()
                && (mark_quickfix_ctx(own, copy_id) || mark_quickfix_user_data(own, copy_id))
            {
                return true;
            }
            let shown = (*win).w_llist_ref;
            if is_ll_window(win) && (*shown).qf_refcount == 1 {
                return mark_quickfix_ctx(shown, copy_id)
                    || mark_quickfix_user_data(shown, copy_id);
            }
            false
        };
        !find_tab_win(aborting).is_null()
    }
}

/// The body of `getqflist()` and `getloclist()`: with no `what` argument the
/// answer is the list of entries, otherwise the dictionary `what` asks for.
///
/// # Safety
///
/// `wp` must be null or a live window, and the two values live.
unsafe fn get_qf_loc_list(
    is_qf: bool,
    wp: *mut win_T,
    what_arg: *mut typval_T,
    rettv: *mut typval_T,
) {
    // SAFETY: forwarded from the caller.
    unsafe {
        if (*what_arg).v_type == VAR_UNKNOWN {
            tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
            if is_qf || !wp.is_null() {
                get_errorlist(ptr::null_mut(), wp, -1, 0, (*rettv).vval.v_list);
            }
            return;
        }

        tv_dict_alloc_ret(rettv);
        if !is_qf && wp.is_null() {
            return;
        }
        if (*what_arg).v_type != VAR_DICT {
            emsg(gettext(&raw const e_dictreq as *const c_char));
            return;
        }
        let d = (*what_arg).vval.v_dict;
        if !d.is_null() {
            qf_get_properties(wp, d, (*rettv).vval.v_dict);
        }
    }
}

/// `getloclist({winnr} [, {what}])`.
///
/// # Safety
///
/// Called through the Vimscript function table with its argument array.
pub unsafe extern "C" fn f_getloclist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the caller's argument array holds at least two values.
    unsafe {
        let wp = find_win_by_nr_or_id(argvars);
        get_qf_loc_list(false, wp, argvars.add(1), rettv);
    }
}

/// `getqflist([{what}])`.
///
/// # Safety
///
/// Called through the Vimscript function table with its argument array.
pub unsafe extern "C" fn f_getqflist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the caller's argument array holds at least one value.
    unsafe { get_qf_loc_list(true, ptr::null_mut(), argvars, rettv) }
}

/// The body of `setqflist()` and `setloclist()`: a list of entries, an
/// optional action character, and an optional title or `what` dictionary.
/// Answers through `rettv`, which is −1 for every rejection.
///
/// # Safety
///
/// `wp` must be null or a live window, and `args` hold three values.
unsafe fn set_qf_ll_list(wp: *mut win_T, args: *mut typval_T, rettv: *mut typval_T) {
    /// Set while `set_errorlist` runs, because an autocommand it fires may
    /// call `setqflist()` again and the list would be pulled out from under
    /// the outer call.
    static RECURSIVE: GlobalCell<c_int> = GlobalCell::new(0);

    // SAFETY: forwarded from the caller.
    unsafe {
        (*rettv).vval.v_number = -1;

        let list_arg = args;
        if (*list_arg).v_type != VAR_LIST {
            emsg(gettext(&raw const e_listreq as *const c_char));
            return;
        }
        if RECURSIVE.get() != 0 {
            emsg(gettext(&raw const e_au_recursive as *const c_char));
            return;
        }

        let mut action = ' ' as c_char;
        let mut title: *const c_char = ptr::null();
        let mut what: *mut dict_T = ptr::null_mut();

        let action_arg = args.add(1);
        if (*action_arg).v_type != VAR_UNKNOWN {
            if (*action_arg).v_type != VAR_STRING {
                emsg(gettext(&raw const e_string_required as *const c_char));
                return;
            }
            // Never null: the value is a string, which is what
            // `tv_get_string_chk` fails on anything else for.
            let act = tv_get_string_chk(action_arg);
            let known = matches!(*act as u8, b'a' | b'r' | b'u' | b' ' | b'f');
            if !known || *act.add(1) as c_int != NUL {
                semsg_c!(gettext(c"E927: Invalid action: '%s'".as_ptr()), act);
                return;
            }
            action = *act;

            let what_arg = args.add(2);
            if (*what_arg).v_type == VAR_STRING {
                title = tv_get_string_chk(what_arg);
                if title.is_null() {
                    return;
                }
            } else if (*what_arg).v_type == VAR_DICT && !(*what_arg).vval.v_dict.is_null() {
                what = (*what_arg).vval.v_dict;
            } else if (*what_arg).v_type != VAR_UNKNOWN {
                emsg(gettext(&raw const e_dictreq as *const c_char));
                return;
            }
        }

        if title.is_null() {
            title = if wp.is_null() {
                c":setqflist()".as_ptr()
            } else {
                c":setloclist()".as_ptr()
            };
        }

        RECURSIVE.set(RECURSIVE.get() + 1);
        let l = (*list_arg).vval.v_list;
        if set_errorlist(wp, l, action as c_int, title.cast_mut(), what) == OK {
            (*rettv).vval.v_number = 0;
        }
        RECURSIVE.set(RECURSIVE.get() - 1);
    }
}

/// `setloclist({winnr}, {list} [, {action} [, {what}]])`.
///
/// # Safety
///
/// Called through the Vimscript function table with its argument array.
pub unsafe extern "C" fn f_setloclist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the caller's argument array holds at least four values.
    unsafe {
        (*rettv).vval.v_number = -1;
        let win = find_win_by_nr_or_id(argvars);
        if !win.is_null() {
            set_qf_ll_list(win, argvars.add(1), rettv);
        }
    }
}

/// `setqflist({list} [, {action} [, {what}]])`.
///
/// # Safety
///
/// Called through the Vimscript function table with its argument array.
pub unsafe extern "C" fn f_setqflist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the caller's argument array holds at least three values.
    unsafe { set_qf_ll_list(ptr::null_mut(), argvars, rettv) }
}
