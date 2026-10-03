//! The Vimscript function bridges, and the garbage collector.
//!
//! [`f_getqflist`]/[`f_setqflist`] and their location-list twins unpack
//! their arguments and call into `getprops`/`setprops`.
//! [`set_ref_in_quickfix`] is the other half: every list's context and
//! every entry's user data is a `TypVal` the collector has to see.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::list::cstr_of_chk;
use crate::eval::typval::NumBuf;
use crate::guard::Depth;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::{
    Refcount, VAR_DICT, VAR_FLOAT, VAR_LIST, VAR_NUMBER, VAR_STRING, kListLenMayKnow,
};
use crate::winlayer::Win;
use core::ffi::c_int;

/// Whether a value can hold a reference at all. Numbers, strings and floats
/// own nothing, so the collector never has to walk into one.
fn holds_references(tv: &TypVal) -> bool {
    !matches!(tv.v_type(), VAR_NUMBER | VAR_STRING | VAR_FLOAT)
}

/// Mark the `user_data` of every entry, and the context value and the
/// `'quickfixtextfunc'` callback, of every list on the stack. Answers
/// whether the walk should be given up, which is what `set_ref_in_item`
/// reports when it finds a cycle it cannot follow.
///
/// Marking runs no user code, so the stack is borrowed for the walk.
fn mark_stack(stack: &QfStack, copy_id: c_int) -> bool {
    let mut aborted = false;
    for list in &stack.lists {
        if aborted {
            break;
        }
        if let Some(ctx) = list.context.as_deref()
            && holds_references(ctx)
        {
            aborted = mark_root(ctx, copy_id);
        }
        aborted = aborted || list.text_func.mark(copy_id);
    }
    if aborted {
        return true;
    }
    for list in &stack.lists {
        if aborted {
            break;
        }
        if !list.has_user_data {
            continue;
        }
        for entry in &list.entries {
            if got_int.get() {
                break;
            }
            // The value is inline in the entry, so it is always there; only
            // its type says whether to walk into it.
            if holds_references(&entry.user_data) {
                aborted = aborted || mark_root(&entry.user_data, copy_id);
            }
        }
    }
    aborted
}

/// Mark everything the quickfix stack and every location list stack hold,
/// so that the garbage collector does not free it.
pub fn set_ref_in_quickfix(copy_id: c_int) -> bool {
    if mark_stack(&Qi::global(), copy_id) || qftf_cb.with(|cb| cb.mark(copy_id)) {
        return true;
    }

    // Every window may own a location list, and a location list window
    // may be the last thing referring to one.
    let aborting = |win: Win| {
        if let Some(own) = win.w_llist
            && mark_stack(&own.stack(), copy_id)
        {
            return true;
        }
        if win.is_location_list_window()
            && let Some(shown) = win.w_llist_ref
        {
            let shown = shown.stack();
            if shown.refcount == Refcount::ONE {
                return mark_stack(&shown, copy_id);
            }
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
    let Some(what_arg) = what_arg else {
        tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
        if (is_qf || window.is_some())
            && let Some(list) = result.list_mut()
        {
            // No list, or an empty one, is an empty answer, not an error.
            let _ = get_errorlist_of(window, list);
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
    if let Some(what) = what_arg.dict_ref()
        && let Some(answer) = result.dict_mut()
    {
        // A request that names nothing readable answers the empty
        // dictionary that is already in `result`.
        let _ = qf_get_properties(window, what, answer);
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
fn set_qf_ll_list(window: Option<Win>, args: &[TypVal], result: &mut TypVal) {
    /// Set while `set_errorlist` runs, because an autocommand it fires may
    /// call `setqflist()` again and the list would be pulled out from under
    /// the outer call.
    static RECURSIVE: GlobalCell<c_int> = GlobalCell::new(0);

    result.write_number(-1);

    let list_arg = &args[0];
    if list_arg.v_type() != VAR_LIST {
        emsg(gettext(e_listreq));
        return;
    }
    if RECURSIVE.get() != 0 {
        emsg(gettext(e_au_recursive));
        return;
    }

    let mut action = b' ';
    let mut title: Option<XString> = None;
    let mut what: Option<&Dict> = None;

    if let Some(action_arg) = args.get(1) {
        if action_arg.v_type() != VAR_STRING {
            emsg(gettext(e_string_required));
            return;
        }
        let mut numbuf = NumBuf::new();
        // Never `None`: the value is a string, which is what
        // `tv_get_string_chk` fails on anything else for.
        let act = cstr_of_chk(action_arg, &mut numbuf).map_or(&b""[..], CStr::to_bytes);
        let known = matches!(act.first(), Some(b'a' | b'r' | b'u' | b' ' | b'f'));
        if !known || act.len() != 1 {
            let act = msg_bytes(act);
            semsg!("E927: Invalid action: '{act}'");
            return;
        }
        action = act[0];

        if let Some(what_arg) = args.get(2) {
            if what_arg.v_type() == VAR_STRING {
                let mut numbuf = NumBuf::new();
                let Some(text) = cstr_of_chk(what_arg, &mut numbuf) else {
                    return;
                };
                title = Some(XString::from_cstr(text));
            } else if what_arg.v_type() == VAR_DICT
                && let Some(d) = what_arg.dict_ref()
            {
                what = Some(d);
            } else {
                emsg(gettext(e_dictreq));
                return;
            }
        }
    }

    let title = title.unwrap_or_else(|| {
        XString::from_cstr(if window.is_none() {
            c":setqflist()"
        } else {
            c":setloclist()"
        })
    });

    let _recursing = Depth::of(&RECURSIVE);
    let list = list_of(list_arg);
    if set_errorlist(window, list, action, title.as_cstr(), what).is_ok() {
        result.write_number(0);
    }
}

/// `setloclist({winnr}, {list} [, {action} [, {what}]])`.
pub fn f_setloclist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    if let Some(win) = find_win_by_nr_or_id(&args[0]) {
        set_qf_ll_list(Some(win), &args[1..], result);
    }
}

/// `setqflist({list} [, {action} [, {what}]])`.
pub fn f_setqflist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    set_qf_ll_list(None, args, result)
}
