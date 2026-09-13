//! Folding a sequence down to one value: `reduce()`, `max()`, `min()`.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::wrappers::{arg_copy, arg_string};
use super::{
    VARNUMBER_MAX, VARNUMBER_MIN, e_missing_function_argument, e_string_list_or_blob_required,
};
use crate::eval::typval::CallFrame;
use crate::eval::typval::{
    NumBuf, blob_bytes, dict_len, list_items, list_iter, list_len, list_locked, list_set_lock,
    tv_check_for_number_arg, tv_check_for_string_arg, tv_copy, tv_get_number_chk,
};
use crate::eval::{eval_expr_typval, partial_name};
use crate::mbyte::utfc_ptr2len;
use crate::memory::xmemdupz;
use crate::message::emsg;
use crate::message::state::called_emsg;
use crate::message_fmt::c_str;
use crate::os::cshim::gettext;
use crate::semsg;
use crate::types::{
    EvalFuncData, NUL, TypVal, VAR_BLOB, VAR_DICT, VAR_FUNC, VAR_LIST, VAR_PARTIAL, VAR_STRING,
    VAR_UNKNOWN, VarLock, VarNumber,
};
use core::ffi::{c_char, c_int, c_void};
use core::mem::ManuallyDrop;

/// A one-character String typval owning a copy of `len` bytes at `p`.
///
/// # Safety
/// `p` has at least `len` readable bytes.
unsafe fn owned_str(p: *const c_char, len: c_int) -> TypVal {
    TypVal::String(unsafe { xmemdupz(p as *const c_void, len as usize) } as *mut c_char)
}

/// A Number typval.
const fn number_tv(n: VarNumber) -> TypVal {
    TypVal::Number(n)
}

/// The shared body of `max()` and `min()`.
fn max_min(tv: &TypVal, result: &mut TypVal, domax: bool) {
    // SAFETY throughout: the caller's obligation; the container is only read, and the
    // dictionary walk is the C's own `TV_DICT_ITER`.
    result.write_number(0);
    // Seeded at the far end so the first item always wins. An empty
    // container returns the 0 written above instead.
    let mut n: VarNumber = if domax { VARNUMBER_MIN } else { VARNUMBER_MAX };
    let better = |i: VarNumber, n: VarNumber| if domax { i > n } else { i < n };
    match tv.v_type() {
        VAR_LIST => {
            if list_len(tv.list_ref()) == 0 {
                return;
            }
            for li in list_iter(unsafe { tv.list_or_null().as_ref() }) {
                let Ok(i) = tv_get_number_chk(&li.li_tv) else {
                    return;
                };
                if better(i, n) {
                    n = i;
                }
            }
        }
        VAR_DICT => {
            if dict_len(tv.dict_ref()) == 0 {
                return;
            }
            // SAFETY: the argument's own dictionary, live for the walk.
            let d = unsafe { &*tv.dict_or_null() };
            for item in d.items() {
                let Ok(i) = tv_get_number_chk(&item.di_tv) else {
                    return;
                };
                if better(i, n) {
                    n = i;
                }
            }
        }
        _ => {
            let what = if domax {
                c"max()".as_ptr()
            } else {
                c"min()".as_ptr()
            };
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let what = unsafe { c_str(what) };
            semsg!("E712: Argument of {what} must be a List or Dictionary");
            return;
        }
    }
    result.write_number(n);
}

/// `max({expr})`.
pub fn f_max(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    max_min(&args[0], result, true)
}

/// `min({expr})`.
pub fn f_min(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    max_min(&args[0], result, false)
}

/// What a fold arm owns, which the three arms genuinely disagree about.
///
/// This is a quirk, not a simplification waiting to happen: upstream's
/// three loops differ here and the difference is observable. The List arm
/// blanks the return value's tag before the call and frees the previous
/// accumulator; the String arm frees the accumulator *and* the character it
/// allocated, but does not blank the tag; the Blob arm frees neither, so a
/// fold whose accumulator is a String or a container leaks one value per
/// byte. Left as it is — it is a leak, not a crash, and the answer is the
/// same either way.
struct Cleanup {
    blank_rettv: bool,
    clear_acc: bool,
    clear_item: bool,
}

const LIST_CLEANUP: Cleanup = Cleanup {
    blank_rettv: true,
    clear_acc: true,
    clear_item: false,
};
const STRING_CLEANUP: Cleanup = Cleanup {
    blank_rettv: false,
    clear_acc: true,
    clear_item: true,
};
const BLOB_CLEANUP: Cleanup = Cleanup {
    blank_rettv: false,
    clear_acc: false,
    clear_item: false,
};

/// Call `expr` with the accumulator and the next item, leaving the result in
/// `result`.
///
/// Returns `false` when the fold should stop — the call failed, or it
/// reported an error of its own.
fn fold_step(
    expr: &TypVal,
    result: &mut TypVal,
    item: &TypVal,
    cleanup: Cleanup,
    called_emsg_start: c_int,
) -> bool {
    // SAFETY throughout: the caller's obligation. `argv` outlives the call.
    // The accumulator and the item are *named* by the frame; `cleanup` says
    // which of the two the callee is expected to have taken over, and the
    // caller owns whatever the frame is not told to claim.  Upstream's
    // shape: the List fold blanks `rettv` so that only `argv[0]` holds the
    // old accumulator, the String fold owns the character it just measured,
    // and the Blob fold's accumulator starts as a Number that owns nothing.
    let mut argv = CallFrame::<2>::new();
    argv.push_borrowed(result);
    argv.push_borrowed(item);
    if cleanup.blank_rettv {
        result.write_empty(VAR_UNKNOWN);
    }
    let r = eval_expr_typval(expr, true, argv.args(), result);
    if cleanup.clear_acc {
        argv.own(0);
    }
    if cleanup.clear_item {
        argv.own(1);
    }
    r.is_ok() && called_emsg.get() == called_emsg_start
}

/// `reduce()` over a List.
fn reduce_list(args: &[TypVal], expr: &TypVal, result: &mut TypVal) {
    // SAFETY: the caller's obligation; the list is locked against
    // modification for the whole fold and restored afterwards.
    let l = args[0].list_or_null();
    let called_emsg_start = called_emsg.get();
    // The accumulator starts as a copy of the initial value, or of the
    // first item when the call gave none.
    let mut at = if args.len() > 2 {
        tv_copy(&args[2], result);
        0
    } else {
        // SAFETY: a live list, or NULL, which reads as empty.
        let Some(first) = (list_items(unsafe { l.as_ref() })).first() else {
            semsg!("E998: Reduce of an empty {} with no initial value", "List");
            return;
        };
        tv_copy(&first.li_tv, result);
        1
    };
    // A null List is `v:_null_list`: nothing to fold, and nothing to
    // lock either.
    if l.is_null() {
        return;
    }
    let prev_locked = list_locked(unsafe { l.as_ref() });
    list_set_lock(unsafe { l.as_mut() }, VarLock::Fixed);
    // By index: `expr` is the user's function, and the lock above stops it
    // editing the list but not a `:for` on the same list from doing so.
    // SAFETY: a live list.
    while at < list_items(unsafe { l.as_ref() }).len() {
        let item = &raw const list_items(unsafe { l.as_ref() })[at].li_tv;
        if !unsafe { fold_step(expr, result, &*item, LIST_CLEANUP, called_emsg_start) } {
            break;
        }
        at += 1;
    }
    list_set_lock(unsafe { l.as_mut() }, prev_locked);
}

/// `reduce()` over a String, one composed character at a time.
fn reduce_string(args: &[TypVal], expr: &TypVal, result: &mut TypVal) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the caller's obligation. `p` walks a NUL-terminated string
    // owned by the argument, which the fold cannot modify.
    let mut p = arg_string(&mut numbuf, &args[0]);
    let called_emsg_start = called_emsg.get();
    if args.len() <= 2 {
        if unsafe { *p } as c_int == NUL {
            semsg!(
                "E998: Reduce of an empty {} with no initial value",
                "String"
            );
            return;
        }
        // With no initial value the first character is it.
        let len = unsafe { utfc_ptr2len(p) };
        *result = unsafe { owned_str(p, len) };
        p = unsafe { p.add(len as usize) };
    } else if tv_check_for_string_arg(args, 2).is_err() {
        return;
    } else {
        arg_copy(&args[2], result);
    }
    while unsafe { *p } as c_int != NUL {
        let len = unsafe { utfc_ptr2len(p) };
        // The fold takes the character over -- `STRING_CLEANUP` clears
        // `argv[1]` -- so this must not release it a second time.
        let item = ManuallyDrop::new(unsafe { owned_str(p, len) });
        // SAFETY: `expr` is the caller's callback and `result` the running
        // accumulator; `item` is the character just measured.
        if !fold_step(expr, result, &item, STRING_CLEANUP, called_emsg_start) {
            break;
        }
        p = unsafe { p.add(len as usize) };
    }
}

/// `reduce()` over a Blob, one byte at a time.
fn reduce_blob(args: &[TypVal], expr: &TypVal, result: &mut TypVal) {
    // SAFETY: the caller's obligation; the blob is re-measured every pass,
    // as the C does, so a fold that shortens it cannot walk off the end.
    let bytes = blob_bytes(args[0].blob_ref());
    let called_emsg_start = called_emsg.get();
    let mut at = if args.len() > 2 {
        if tv_check_for_number_arg(args, 2).is_err() {
            return;
        }
        tv_copy(&args[2], result);
        0
    } else {
        let Some(&first) = bytes.first() else {
            semsg!("E998: Reduce of an empty {} with no initial value", "Blob");
            return;
        };
        result.write_number(VarNumber::from(first));
        1
    };
    // By index and re-read each step: the fold runs a user function, which
    // can grow or free the blob under the walk.
    while at < blob_bytes(args[0].blob_ref()).len() {
        let item = number_tv(VarNumber::from(blob_bytes(args[0].blob_ref())[at]));
        if !fold_step(expr, result, &item, BLOB_CLEANUP, called_emsg_start) {
            return;
        }
        at += 1;
    }
}

/// `reduce({object}, {func} [, {initial}])`.
pub fn f_reduce(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: everything read below is the frame's.
    let ty = args[0].v_type();
    if ty != VAR_STRING && ty != VAR_LIST && ty != VAR_BLOB {
        emsg(gettext(e_string_list_or_blob_required));
        return;
    }
    // The callable is checked for emptiness here rather than by
    // `eval_expr_typval`, so that an empty name reports E1132 instead of
    // an "unknown function" for the empty string.
    let func_name = match args[1].v_type() {
        VAR_FUNC => args[1].func_name_or_null(),
        VAR_PARTIAL => unsafe { partial_name(args[1].partial_or_null()) },
        _ => arg_string(&mut numbuf, &args[1]),
    };
    if func_name.is_null() || unsafe { *func_name } as c_int == NUL {
        emsg(gettext(e_missing_function_argument));
        return;
    }
    let expr = &args[1];
    match ty {
        VAR_LIST => reduce_list(args, expr, result),
        VAR_STRING => reduce_string(args, expr, result),
        _ => reduce_blob(args, expr, result),
    }
}
