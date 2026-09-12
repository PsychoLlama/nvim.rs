//! The Vimscript function bridges, and the garbage collector.
//!
//! [`f_getqflist`]/[`f_setqflist`] and their location-list twins unpack
//! their arguments and call into `getprops`/`setprops`.
//! [`set_ref_in_quickfix`] is the other half: every list's context and
//! every entry's user data is a `TypVal` the collector has to see.

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
use crate::eval::typval::NumBuf;
use crate::guard::Depth;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{
    NUL, Refcount, VAR_DICT, VAR_FLOAT, VAR_LIST, VAR_NUMBER, VAR_STRING, kListLenMayKnow,
};
use crate::winlayer::Win;
use core::ffi::{c_char, c_int};
use core::ptr;

/// The parsed global `'quickfixtextfunc'`. A list-local one lives in
/// `QfInfo::qf_qftf_cb`.
///
/// The address, because every operation the tree has on a callback —
/// parsing an option into it, marking it for the collector, copying it,
/// calling it — takes a `*mut Callback`.
pub(super) fn global_qftf() -> *mut Callback {
    qftf_cb.ptr()
}

/// Whether a value can hold a reference at all. Numbers, strings and floats
/// own nothing, so the collector never has to walk into one.
fn holds_references(tv: &TypVal) -> bool {
    // SAFETY: the caller's value.
    !matches!((*tv).v_type(), VAR_NUMBER | VAR_STRING | VAR_FLOAT)
}

/// Mark the `user_data` of every entry of every list on the stack. Answers
/// whether the walk should be given up, which is what `set_ref_in_item`
/// reports when it finds a cycle it cannot follow.
///
/// # Safety
///
/// `qi` must be a live stack.
unsafe fn mark_quickfix_user_data(qi: *mut QfInfo, copy_id: c_int) -> bool {
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
                let (no_ht, no_list) = (ptr::null_mut(), ptr::null_mut());
                if unsafe { holds_references(&*user_data) } {
                    let data = unsafe { &mut *user_data };
                    aborted = aborted || unsafe { set_ref_in_item(data, copy_id, no_ht, no_list) };
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
unsafe fn mark_quickfix_ctx(qi: *mut QfInfo, copy_id: c_int) -> bool {
    let mut aborted = false;
    let mut i = 0;
    while i < unsafe { (*qi).max_count() } && !aborted {
        let ctx = unsafe { (*qf_get_list(qi, i)).qf_ctx };
        if !ctx.is_null() && unsafe { holds_references(&*ctx) } {
            // SAFETY: the list's own context value.
            let ctx = unsafe { &mut *ctx };
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
pub fn set_ref_in_quickfix(copy_id: c_int) -> bool {
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
    find_tab_win(aborting).is_some()
}

/// The body of `getqflist()` and `getloclist()`: with no `what` argument the
/// answer is the list of entries, otherwise the dictionary `what` asks for.
fn get_qf_loc_list(
    is_qf: bool,
    window: Option<Win>,
    what_arg: Option<&TypVal>,
    result: &mut TypVal,
) {
    // SAFETY: forwarded from the caller.
    let Some(what_arg) = what_arg else {
        tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
        if is_qf || window.is_some() {
            // No list, or an empty one, is an empty answer, not an error.
            let _ =
                unsafe { get_errorlist(ptr::null_mut(), window, -1, 0, (*result).list_or_null()) };
        }
        return;
    };

    tv_dict_alloc_ret(result);
    if !is_qf && window.is_none() {
        return;
    }
    if what_arg.v_type() != VAR_DICT {
        emsg(gettext(e_dictreq));
        return;
    }
    let d = what_arg.dict_or_null();
    if !d.is_null() {
        // A request that names nothing readable answers the empty
        // dictionary that is already in `result`.
        let _ = unsafe { qf_get_properties(window, d, (*result).dict_or_null()) };
    }
}

/// `getloclist({winnr} [, {what}])`.
pub fn f_getloclist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_qf_loc_list(false, find_win_by_nr_or_id(&args[0]), args.get(1), result);
}

/// `getqflist([{what}])`.
pub fn f_getqflist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    get_qf_loc_list(true, None, args.first(), result)
}

/// The body of `setqflist()` and `setloclist()`: a list of entries, an
/// optional action character, and an optional title or `what` dictionary.
/// Answers through `result`, which is −1 for every rejection.
///
/// # Safety
///
/// `window` must be null or a live window, and `args` hold three values.
unsafe fn set_qf_ll_list(window: Option<Win>, args: &[TypVal], result: &mut TypVal) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    /// Set while `set_errorlist` runs, because an autocommand it fires may
    /// call `setqflist()` again and the list would be pulled out from under
    /// the outer call.
    static RECURSIVE: GlobalCell<c_int> = GlobalCell::new(0);

    // SAFETY: forwarded from the caller.
    (*result).write_number(-1);

    let list_arg = &args[0];
    if list_arg.v_type() != VAR_LIST {
        emsg(gettext(e_listreq));
        return;
    }
    if RECURSIVE.get() != 0 {
        emsg(gettext(e_au_recursive));
        return;
    }

    let mut action = ' ' as c_char;
    let mut title: *const c_char = ptr::null();
    let mut what: *mut Dict = ptr::null_mut();

    if let Some(action_arg) = args.get(1) {
        if action_arg.v_type() != VAR_STRING {
            emsg(gettext(e_string_required));
            return;
        }
        // Never null: the value is a string, which is what
        // `tv_get_string_chk` fails on anything else for.
        let act = unsafe { numbuf.string_chk(action_arg) };
        let known = matches!(
            unsafe { *act }.cast_unsigned(),
            b'a' | b'r' | b'u' | b' ' | b'f'
        );
        if !known || c_int::from(unsafe { *act.add(1) }) != NUL {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let act = unsafe { c_str(act) };
            semsg!("E927: Invalid action: '{act}'");
            return;
        }
        action = unsafe { *act };

        if let Some(what_arg) = args.get(2) {
            if what_arg.v_type() == VAR_STRING {
                title = unsafe { numbuf2.string_chk(what_arg) };
                if title.is_null() {
                    return;
                }
            } else if what_arg.v_type() == VAR_DICT && !what_arg.dict_or_null().is_null() {
                what = what_arg.dict_or_null();
            } else {
                emsg(gettext(e_dictreq));
                return;
            }
        }
    }

    if title.is_null() {
        title = if window.is_none() {
            c":setqflist()".as_ptr()
        } else {
            c":setloclist()".as_ptr()
        };
    }

    let _recursing = Depth::of(&RECURSIVE);
    let l = list_arg.list_or_null();
    if unsafe { set_errorlist(window, l, c_int::from(action), title.cast_mut(), what) }.is_ok() {
        (*result).write_number(0);
    }
}

/// `setloclist({winnr}, {list} [, {action} [, {what}]])`.
pub fn f_setloclist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's argument array holds at least four values.
    result.write_number(-1);
    if let Some(win) = find_win_by_nr_or_id(&args[0]) {
        unsafe { set_qf_ll_list(Some(win), &args[1..], result) };
    }
}

/// `setqflist({list} [, {action} [, {what}]])`.
pub fn f_setqflist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's argument array holds at least three values.
    unsafe { set_qf_ll_list(None, args, result) }
}
