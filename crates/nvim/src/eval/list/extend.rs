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

use core::ffi::{CStr, c_int};

use super::{Container, check_lock, copy_tv, cstr_of, cstr_of_chk, err_nr, err_str, number_of};
use crate::eval::typval::NumBuf;
use crate::message::{e_invarg2, e_list_index_out_of_range_nr, e_listblobarg, e_listdictarg};
use crate::types::{EvalFuncData, TypVal, VarLock, int64_t, uint8_t};

/// `extend()`/`extendnew()` over two Dicts: merge `argvars[1]`'s keys into
/// `argvars[0]` (or into a copy of it) under the policy `argvars[2]` names.
fn extend_dict(args: &[TypVal], arg_errmsg: &CStr, is_new: bool, result: &mut TypVal) {
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
    if is_new {
        d1 = d1.copy();
        if d1.is_null() {
            return;
        }
    }

    // Check the third argument.
    let mut numbuf = NumBuf::new();
    let mut action = c"force";
    if args.len() > 2 {
        let Some(name) = cstr_of_chk(&args[2], &mut numbuf) else {
            // Type error; error message already given.
            if is_new {
                d1.unref();
            }
            return;
        };
        if !matches!(name.to_bytes(), b"keep" | b"force" | b"error") {
            if is_new {
                d1.unref();
            }
            err_str(e_invarg2, name);
            return;
        }
        action = name;
    }

    d1.extend_with(d2, action);

    if is_new {
        *result = TypVal::Dict(d1.raw());
    } else {
        copy_tv(&args[0], result);
    }
}

/// `extend()`/`extendnew()` over two Lists: splice `argvars[1]` into
/// `argvars[0]` (or into a copy of it) before index `argvars[2]`.
fn extend_list(args: &[TypVal], arg_errmsg: &CStr, is_new: bool, result: &mut TypVal) {
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
    if is_new {
        l1 = l1.copy();
        if l1.is_null() {
            return;
        }
    }

    // The item to splice in before, or None for "at the end".  Every way out
    // of this block that is not an item has to undo the copy above.
    let before = 'find: {
        if args.len() <= 2 {
            break 'find None;
        }
        let idx = number_of(&args[2], &mut error) as c_int;
        if !error {
            if idx == l1.len() {
                break 'find None;
            }
            match l1.find(idx) {
                Some(item) => break 'find Some(item),
                None => err_nr(e_list_index_out_of_range_nr, idx as int64_t),
            }
        }
        if is_new {
            l1.unref();
        }
        return;
    };

    l1.extend_with(l2, before);

    if is_new {
        *result = TypVal::List(l1.raw());
    } else {
        copy_tv(&args[0], result);
    }
}

/// The shared body of `extend()` and `extendnew()`: two Lists or two Dicts,
/// nothing else.
fn extend(args: &[TypVal], result: &mut TypVal, arg_errmsg: &CStr, is_new: bool) {
    match (Container::of(&args[0]), Container::of(&args[1])) {
        (Container::List(_), Container::List(_)) => extend_list(args, arg_errmsg, is_new, result),
        (Container::Dict(_), Container::Dict(_)) => extend_dict(args, arg_errmsg, is_new, result),
        _ => err_str(
            e_listdictarg,
            if is_new { c"extendnew()" } else { c"extend()" },
        ),
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
                before = number_of(&args[2], &mut error) as c_int;
                if error {
                    // Type error; errmsg already given.
                    return;
                }
                if before < 0 || before > len {
                    let mut numbuf = NumBuf::new();
                    err_str(e_invarg2, cstr_of(&args[2], &mut numbuf));
                    return;
                }
            }
            let val = number_of(&args[1], &mut error) as c_int;
            if error {
                return;
            }
            if !(0..=255).contains(&val) {
                let mut numbuf = NumBuf::new();
                err_str(e_invarg2, cstr_of(&args[1], &mut numbuf));
                return;
            }
            b.insert_byte(before, val as uint8_t);
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
            if before != l.len() as int64_t {
                item = l.find(before as c_int);
                if item.is_none() {
                    err_nr(e_list_index_out_of_range_nr, before);
                    return;
                }
            }
            l.insert_tv(&args[1], item);
            copy_tv(&args[0], result);
        }
        _ => err_str(e_listblobarg, c"insert()"),
    }
}
