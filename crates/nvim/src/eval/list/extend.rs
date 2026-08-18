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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_int};

use super::{
    Args, Container, check_lock, copy_tv, cstr_of, cstr_of_chk, err_nr, err_str, frame, number_of,
};
use crate::main::{e_invarg2, e_list_index_out_of_range_nr, e_listblobarg, e_listdictarg};
use crate::types::{
    EvalFuncData, VAR_DICT, VAR_FIXED, VAR_LIST, VAR_UNLOCKED, int64_t, typval_T,
    typval_vval_union, uint8_t,
};

/// `extend()`/`extendnew()` over two Dicts: merge `argvars[1]`'s keys into
/// `argvars[0]` (or into a copy of it) under the policy `argvars[2]` names.
fn extend_dict(mut args: Args<'_>, arg_errmsg: &CStr, is_new: bool, rettv: &mut typval_T) {
    let Container::Dict(mut d1) = Container::of(args.get_mut(0)) else {
        unreachable!("dispatched on VAR_DICT")
    };
    if d1.is_null() {
        // A NULL Dict is `VAR_FIXED`, so this always reports E742.
        let locked = check_lock(VAR_FIXED, arg_errmsg);
        debug_assert!(locked, "locked == true");
        return;
    }
    let Container::Dict(d2) = Container::of(args.get_mut(1)) else {
        unreachable!("dispatched on VAR_DICT")
    };
    if d2.is_null() {
        // Do nothing.
        copy_tv(args.get_mut(0), rettv);
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
    let mut action = c"force";
    if args.has(2) {
        let Some(name) = cstr_of_chk(args.get_mut(2)) else {
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
            err_str(&e_invarg2, name);
            return;
        }
        action = name;
    }

    d1.extend_with(d2, action);

    if is_new {
        *rettv = typval_T {
            v_type: VAR_DICT,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_dict: d1.raw() },
        };
    } else {
        copy_tv(args.get_mut(0), rettv);
    }
}

/// `extend()`/`extendnew()` over two Lists: splice `argvars[1]` into
/// `argvars[0]` (or into a copy of it) before index `argvars[2]`.
fn extend_list(mut args: Args<'_>, arg_errmsg: &CStr, is_new: bool, rettv: &mut typval_T) {
    let mut error = false;
    let Container::List(mut l1) = Container::of(args.get_mut(0)) else {
        unreachable!("dispatched on VAR_LIST")
    };
    let Container::List(l2) = Container::of(args.get_mut(1)) else {
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
        if !args.has(2) {
            break 'find None;
        }
        let idx = number_of(args.get_mut(2), &mut error) as c_int;
        if !error {
            if idx == l1.len() {
                break 'find None;
            }
            match l1.find(idx) {
                Some(item) => break 'find Some(item),
                None => err_nr(&e_list_index_out_of_range_nr, idx as int64_t),
            }
        }
        if is_new {
            l1.unref();
        }
        return;
    };

    l1.extend_with(l2, before);

    if is_new {
        *rettv = typval_T {
            v_type: VAR_LIST,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_list: l1.raw() },
        };
    } else {
        copy_tv(args.get_mut(0), rettv);
    }
}

/// The shared body of `extend()` and `extendnew()`: two Lists or two Dicts,
/// nothing else.
fn extend(mut args: Args<'_>, rettv: &mut typval_T, arg_errmsg: &CStr, is_new: bool) {
    match (
        Container::of(args.get_mut(0)),
        Container::of(args.get_mut(1)),
    ) {
        (Container::List(_), Container::List(_)) => extend_list(args, arg_errmsg, is_new, rettv),
        (Container::Dict(_), Container::Dict(_)) => extend_dict(args, arg_errmsg, is_new, rettv),
        _ => err_str(
            &e_listdictarg,
            if is_new { c"extendnew()" } else { c"extend()" },
        ),
    }
}

/// `extend(list, list [, idx])` / `extend(dict, dict [, action])`: change the
/// first container in place and answer it.
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 2..3, and `rettv`
/// a cleared result.
pub unsafe fn f_extend(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    let (mut args, rettv) = frame!(argvars, rettv);
    extend(args, rettv, c"extend() argument", false);
}

/// `extendnew(list, list [, idx])` / `extendnew(dict, dict [, action])`:
/// [`f_extend`] over a shallow copy, leaving the argument alone.
///
/// # Safety
/// As [`f_extend`].
pub unsafe fn f_extendnew(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    let (mut args, rettv) = frame!(argvars, rettv);
    extend(args, rettv, c"extendnew() argument", true);
}

/// `insert(container, item [, idx])`: put one item into a List, or one byte
/// into a Blob, before `idx`.
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 2..3, and `rettv`
/// a cleared result.
pub unsafe fn f_insert(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract.
    let (mut args, rettv) = frame!(argvars, rettv);
    let mut error = false;
    match Container::of(args.get_mut(0)) {
        Container::Blob(b) => {
            if b.is_null() || check_lock(b.lock(), c"insert() argument") {
                return;
            }
            let len = b.len();
            let mut before = 0;
            if args.has(2) {
                before = number_of(args.get_mut(2), &mut error) as c_int;
                if error {
                    // Type error; errmsg already given.
                    return;
                }
                if before < 0 || before > len {
                    err_str(&e_invarg2, cstr_of(args.get_mut(2)));
                    return;
                }
            }
            let val = number_of(args.get_mut(1), &mut error) as c_int;
            if error {
                return;
            }
            if !(0..=255).contains(&val) {
                err_str(&e_invarg2, cstr_of(args.get_mut(1)));
                return;
            }
            b.insert_byte(before, val as uint8_t);
            copy_tv(args.get_mut(0), rettv);
        }
        Container::List(l) => {
            if check_lock(l.locked(), c"insert() argument") {
                return;
            }
            let mut before: int64_t = 0;
            if args.has(2) {
                before = number_of(args.get_mut(2), &mut error);
            }
            if error {
                // Type error; errmsg already given.
                return;
            }
            let mut item = None;
            if before != l.len() as int64_t {
                item = l.find(before as c_int);
                if item.is_none() {
                    err_nr(&e_list_index_out_of_range_nr, before);
                    return;
                }
            }
            l.insert_tv(args.get_mut(1), item);
            copy_tv(args.get_mut(0), rettv);
        }
        _ => err_str(&e_listblobarg, c"insert()"),
    }
}
