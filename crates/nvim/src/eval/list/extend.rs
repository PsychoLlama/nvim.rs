//! Joining two containers, and inserting into one -- `extend()`,
//! `extendnew()` and `insert()`.
//!
//! [`extend`] is the shared body of `extend()`/`extendnew()`: for Lists it
//! splices the second list in at an index, for Dicts it merges keys under a
//! `"keep"`/`"force"`/`"error"` policy.  `extendnew()` is the same walk over
//! a shallow copy, which is why both halves have to undo that copy on every
//! error path.  `f_insert` is the single-item form.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::narrow::number_as_int;
use core::ffi::CStr;

use super::{Container, DictArg, ListArg, check_lock, copy_tv, cstr_of, cstr_of_chk, number_of};
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::typval::NumBuf;
use crate::message_fmt::msg_cstr;
use crate::os::libc::int64_t;
use crate::semsg;
use crate::types::{TypVal, VarLock};

/// `extend()`/`extendnew()` over two Dicts: merge `argvars[1]`'s keys into
/// `argvars[0]` (or into a copy of it) under the policy `argvars[2]` names.
fn extend_dict(args: &[TypVal], arg_errmsg: &'static CStr, is_new: bool, result: &mut TypVal) {
    let Container::Dict(mut d1) = Container::of(&args[0]) else {
        unreachable!("dispatched on VAR_DICT")
    };
    if d1.is_null() {
        // A NULL Dict is `VarLock::Fixed`, so this always reports E742.
        let locked = check_lock(VarLock::Fixed, arg_errmsg);
        debug_assert!(locked, "locked == true");
        return;
    }
    let Container::Dict(d2) = Container::of(&args[1]) else {
        unreachable!("dispatched on VAR_DICT")
    };
    if d2.is_null() {
        // Do nothing.
        copy_tv(&args[0], result);
        return;
    }

    if !is_new && check_lock(d1.lock(), arg_errmsg) {
        return;
    }
    // `extendnew()` merges into a copy, which this body owns until it
    // becomes the answer.
    let mut held = None;
    if is_new {
        let Some(copy) = d1.copy() else {
            return;
        };
        held = Some(copy);
    }
    if let Some(copy) = &held {
        d1 = DictArg::of(Some(copy));
    }

    // Check the third argument.
    let mut numbuf = NumBuf::new();
    let mut action = c"force";
    if args.len() > 2 {
        let Some(name) = cstr_of_chk(&args[2], &mut numbuf) else {
            // Type error; error message already given. The copy goes with
            // `held`, which is its only reference.
            return;
        };
        if !matches!(name.to_bytes(), b"keep" | b"force" | b"error") {
            let what = msg_cstr(name);
            semsg!("E475: Invalid argument: {what}");
            return;
        }
        action = name;
    }

    d1.extend_with(d2, action);

    if is_new {
        result.write_dict(held);
    } else {
        copy_tv(&args[0], result);
    }
}

/// `extend()`/`extendnew()` over two Lists: splice `argvars[1]` into
/// `argvars[0]` (or into a copy of it) before index `argvars[2]`.
fn extend_list(args: &[TypVal], arg_errmsg: &'static CStr, is_new: bool, result: &mut TypVal) {
    let mut error = false;
    let Container::List(mut l1) = Container::of(&args[0]) else {
        unreachable!("dispatched on VAR_LIST")
    };
    let Container::List(l2) = Container::of(&args[1]) else {
        unreachable!("dispatched on VAR_LIST")
    };

    if !is_new && check_lock(l1.locked(), arg_errmsg) {
        return;
    }
    // `extendnew()` splices into a copy, which this body owns until it
    // becomes the answer: every way out below drops it.
    let mut held = None;
    if is_new {
        let Some(copy) = l1.copy() else {
            return;
        };
        held = Some(copy);
    }
    if let Some(copy) = &held {
        l1 = ListArg::of(Some(copy));
    }

    // The item to splice in before, or None for "at the end".  Every way out
    // of this block that is not an item has to undo the copy above.
    let before = match args.get(2) {
        None => None,
        Some(index) => {
            let idx = number_as_int(number_of(index, &mut error));
            if error {
                return;
            }
            if idx == l1.len() {
                None
            } else {
                let Some(item) = l1.find(idx) else {
                    semsg!("E684: List index out of range: {}", int64_t::from(idx));
                    return;
                };
                Some(item)
            }
        }
    };

    l1.extend_with(l2, before);

    if is_new {
        result.write_list(held);
    } else {
        copy_tv(&args[0], result);
    }
}

/// The shared body of `extend()` and `extendnew()`: two Lists or two Dicts,
/// nothing else.
fn extend(args: &[TypVal], result: &mut TypVal, arg_errmsg: &'static CStr, is_new: bool) {
    match (Container::of(&args[0]), Container::of(&args[1])) {
        (Container::List(_), Container::List(_)) => extend_list(args, arg_errmsg, is_new, result),
        (Container::Dict(_), Container::Dict(_)) => extend_dict(args, arg_errmsg, is_new, result),
        _ => {
            let arg0 = if is_new { "extendnew()" } else { "extend()" };
            semsg!("E712: Argument of {arg0} must be a List or Dictionary");
        }
    }
}

/// `extend(list, list [, idx])` / `extend(dict, dict [, action])`: change the
/// first container in place and answer it.
pub fn f_extend(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    extend(args, result, c"extend() argument", false);
}

/// `extendnew(list, list [, idx])` / `extendnew(dict, dict [, action])`:
/// [`f_extend`] over a shallow copy, leaving the argument alone.
pub fn f_extendnew(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    extend(args, result, c"extendnew() argument", true);
}

/// `insert(container, item [, idx])`: put one item into a List, or one byte
/// into a Blob, before `idx`.
pub fn f_insert(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    let mut error = false;
    match Container::of(&args[0]) {
        Container::Blob(b) => {
            if b.is_null() || check_lock(b.lock(), c"insert() argument") {
                return;
            }
            let len = b.len();
            let mut before = 0;
            if args.len() > 2 {
                before = number_as_int(number_of(&args[2], &mut error));
                if error {
                    // Type error; errmsg already given.
                    return;
                }
                if before < 0 || before > len {
                    let mut numbuf = NumBuf::new();
                    let what = msg_cstr(cstr_of(&args[2], &mut numbuf));
                    semsg!("E475: Invalid argument: {what}");
                    return;
                }
            }
            let val = number_as_int(number_of(&args[1], &mut error));
            if error {
                return;
            }
            if !(0..=255).contains(&val) {
                let mut numbuf = NumBuf::new();
                let what = msg_cstr(cstr_of(&args[1], &mut numbuf));
                semsg!("E475: Invalid argument: {what}");
                return;
            }
            b.insert_byte(before, u8::try_from(val).expect("checked to 0..=255"));
            copy_tv(&args[0], result);
        }
        Container::List(l) => {
            if check_lock(l.locked(), c"insert() argument") {
                return;
            }
            let mut before: int64_t = 0;
            if args.len() > 2 {
                before = number_of(&args[2], &mut error);
            }
            if error {
                // Type error; errmsg already given.
                return;
            }
            let mut item = None;
            if before != int64_t::from(l.len()) {
                item = l.find(number_as_int(before));
                if item.is_none() {
                    semsg!("E684: List index out of range: {before}");
                    return;
                }
            }
            l.insert_tv(&args[1], item);
            copy_tv(&args[0], result);
        }
        _ => {
            let arg0 = "insert()";
            semsg!("E899: Argument of {arg0} must be a List or Blob");
        }
    }
}
