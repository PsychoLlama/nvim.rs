//! Folding a sequence down to one value: `reduce()`, `max()`, `min()`.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::wrappers::arg_copy;
use super::{E_MISSING_FUNCTION_ARGUMENT, E_STRING_LIST_OR_BLOB_REQUIRED};
use crate::eval::VARNUMBER_MAX;
use crate::eval::VARNUMBER_MIN;
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::typval::CallFrame;
use crate::eval::typval::ReadScope;
use crate::eval::typval::{
    NumBuf, dict_len, list_iter, list_len, tv_check_for_number_arg, tv_check_for_string_arg,
    tv_copy, tv_get_number_chk,
};
use crate::eval::{eval_expr_typval, partial_name};
use crate::mbyte::cluster_len;
use crate::memory::ThinCString;
use crate::message::emsg;
use crate::message::state::CALLED_EMSG;
use crate::os::cshim::gettext;
use crate::semsg;
use crate::types::{
    TypVal, VAR_BLOB, VAR_DICT, VAR_FUNC, VAR_LIST, VAR_PARTIAL, VAR_STRING, VarLock, VarNumber,
};
use core::ffi::c_int;

/// A Number typval.
const fn number_tv(n: VarNumber) -> TypVal {
    TypVal::Number(n)
}

/// The shared body of `max()` and `min()`.
fn max_min(tv: &TypVal, result: &mut TypVal, domax: bool) {
    // The container is only read: nothing below runs user code. A number
    // that does not convert puts out a message under the read, and the
    // container may be `v:`: the scope holds the message's row.
    let _messages = ReadScope::new();
    result.write_number(0);
    // Seeded at the far end so the first item always wins. An empty
    // container returns the 0 written above instead.
    let mut n: VarNumber = if domax { VARNUMBER_MIN } else { VARNUMBER_MAX };
    let better = |i: VarNumber, n: VarNumber| if domax { i > n } else { i < n };
    match tv.v_type() {
        VAR_LIST => {
            if list_len(tv.list_ref().as_deref()) == 0 {
                return;
            }
            for li in list_iter(tv.list_ref().as_deref()) {
                let Ok(i) = tv_get_number_chk(&li.li_tv) else {
                    return;
                };
                if better(i, n) {
                    n = i;
                }
            }
        }
        VAR_DICT => {
            if dict_len(tv.dict_ref().as_deref()) == 0 {
                return;
            }
            let dict = tv.dict_ref();
            for item in dict.iter().flat_map(|d| d.items()) {
                let Ok(i) = tv_get_number_chk(&item.di_tv) else {
                    return;
                };
                if better(i, n) {
                    n = i;
                }
            }
        }
        _ => {
            let what = if domax { "max()" } else { "min()" };
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

/// What a fold arm does with the accumulator it hands the callee.
///
/// This is a quirk, not a simplification waiting to happen: upstream's
/// three loops differ here. The List and String arms hand the previous
/// accumulator over and free it after the call; the Blob arm hands the call
/// a copy and leaves the accumulator in place, so a fold whose call fails
/// keeps the value it had.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Accumulator {
    /// Moved into the call's frame, which releases it.
    Handed,
    /// Copied into the call's frame; the accumulator keeps its value until
    /// the call writes a new one.
    Kept,
}

/// Call `expr` with the accumulator and the next item, leaving the result in
/// `result`.
///
/// Returns `false` when the fold should stop — the call failed, or it
/// reported an error of its own.
fn fold_step(
    expr: &TypVal,
    result: &mut TypVal,
    item: TypVal,
    accumulator: Accumulator,
    called_emsg_start: c_int,
) -> bool {
    // The frame owns both of its values and releases them after the call:
    // the item is this step's own, and the accumulator is either moved out
    // of `result` or copied from it, so the callee can write `result`
    // while the frame holds the old value.
    let mut argv = CallFrame::<2>::new();
    argv.push_owned(match accumulator {
        Accumulator::Handed => result.take(),
        Accumulator::Kept => result.copy_for_call(),
    });
    argv.push_owned(item);
    let r = eval_expr_typval(expr, true, argv.args(), result);
    drop(argv);
    r.is_ok() && CALLED_EMSG.get() == called_emsg_start
}

/// `reduce()` over a List.
fn reduce_list(args: &[TypVal], expr: &TypVal, result: &mut TypVal) {
    // Locked against modification for the whole fold and restored
    // afterwards.
    let list = args[0].list_shared();
    let called_emsg_start = CALLED_EMSG.get();
    // The accumulator starts as a copy of the initial value, or of the
    // first item when the call gave none.
    let mut at = if args.len() > 2 {
        tv_copy(&args[2], result);
        0
    } else {
        let Some(first) = args[0]
            .list_ref()
            .and_then(|l| l.items().first().map(|li| li.li_tv.clone()))
        else {
            semsg!("E998: Reduce of an empty {} with no initial value", "List");
            return;
        };
        result.overwrite(first);
        1
    };
    // A null List is `v:_null_list`: nothing to fold, and nothing to
    // lock either.
    let Some(list) = list else {
        return;
    };
    let prev_locked = list.read().lock();
    list.edit().set_lock(VarLock::Fixed);
    // By index, through the handle, and with the item copied out: `expr`
    // is the user's function, and the lock above stops it editing the list
    // but not a `:for` on the same list from doing so.
    while let Some(item) = { list.read().items().get(at).map(|li| li.li_tv.clone()) } {
        if !fold_step(expr, result, item, Accumulator::Handed, called_emsg_start) {
            break;
        }
        at += 1;
    }
    list.edit().set_lock(prev_locked);
}

/// `reduce()` over a String, one composed character at a time.
fn reduce_string(args: &[TypVal], expr: &TypVal, result: &mut TypVal) {
    let mut numbuf = NumBuf::new();
    // The argument's own string, which the fold cannot modify.
    let text = numbuf.bytes(&args[0]);
    let mut at = 0;
    let called_emsg_start = CALLED_EMSG.get();
    if args.len() <= 2 {
        if text.is_empty() {
            semsg!(
                "E998: Reduce of an empty {} with no initial value",
                "String"
            );
            return;
        }
        // With no initial value the first character is it.
        at = cluster_len(text);
        *result = TypVal::string_from(&text[..at]);
    } else if tv_check_for_string_arg(args, 2).is_err() {
        return;
    } else {
        arg_copy(&args[2], result);
    }
    while at < text.len() {
        let len = cluster_len(&text[at..]);
        let item = TypVal::string_from(&text[at..at + len]);
        if !fold_step(expr, result, item, Accumulator::Handed, called_emsg_start) {
            break;
        }
        at += len;
    }
}

/// `reduce()` over a Blob, one byte at a time.
fn reduce_blob(args: &[TypVal], expr: &TypVal, result: &mut TypVal) {
    // The blob is re-measured every pass, as the C does, so a fold that
    // shortens it cannot walk off the end.
    let called_emsg_start = CALLED_EMSG.get();
    let mut at = if args.len() > 2 {
        if tv_check_for_number_arg(args, 2).is_err() {
            return;
        }
        tv_copy(&args[2], result);
        0
    } else {
        let Some(&first) = args[0].blob_bytes().first() else {
            semsg!("E998: Reduce of an empty {} with no initial value", "Blob");
            return;
        };
        result.write_number(VarNumber::from(first));
        1
    };
    // By index and re-read each step: the fold runs a user function, which
    // can grow or free the blob under the walk.
    while at < args[0].blob_bytes().len() {
        let item = number_tv(VarNumber::from(args[0].blob_bytes()[at]));
        if !fold_step(expr, result, item, Accumulator::Kept, called_emsg_start) {
            return;
        }
        at += 1;
    }
}

/// `reduce({object}, {func} [, {initial}])`.
pub fn f_reduce(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let ty = args[0].v_type();
    if ty != VAR_STRING && ty != VAR_LIST && ty != VAR_BLOB {
        emsg(gettext(E_STRING_LIST_OR_BLOB_REQUIRED));
        return;
    }
    // The callable is checked for emptiness here rather than by
    // `eval_expr_typval`, so that an empty name reports E1132 instead of
    // an "unknown function" for the empty string.
    let missing = match args[1].v_type() {
        VAR_FUNC => args[1].func_name().is_none_or(ThinCString::is_empty),
        VAR_PARTIAL => args[1].partial_ref().map_or(c"", partial_name).is_empty(),
        _ => numbuf.string(&args[1]).is_empty(),
    };
    if missing {
        emsg(gettext(E_MISSING_FUNCTION_ARGUMENT));
        return;
    }
    let expr = &args[1];
    match ty {
        VAR_LIST => reduce_list(args, expr, result),
        VAR_STRING => reduce_string(args, expr, result),
        _ => reduce_blob(args, expr, result),
    }
}
