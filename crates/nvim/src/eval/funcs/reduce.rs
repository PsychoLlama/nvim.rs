//! Folding a sequence down to one value: `reduce()`, `max()`, `min()`.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::wrappers::{arg_copy, arg_string, check_arg};
use super::{
    VARNUMBER_MAX, VARNUMBER_MIN, e_missing_function_argument, e_string_list_or_blob_required,
};
use crate::eval::typval::{
    NumBuf, tv_blob_get, tv_blob_len, tv_check_for_number_arg, tv_check_for_string_arg, tv_clear,
    tv_copy, tv_dict_len, tv_get_number_chk, tv_list_first, tv_list_len, tv_list_locked,
    tv_list_set_lock,
};
use crate::eval::{eval_expr_typval, partial_name};
use crate::hashtab::hash_removed;
use crate::main::called_emsg;
use crate::mbyte::utfc_ptr2len;
use crate::memory::xmemdupz;
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::os::cshim::gettext;
use crate::semsg;
use crate::types::{
    EvalFuncData, NUL, VAR_BLOB, VAR_DICT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_STRING,
    VAR_UNKNOWN, VarLock, blob_T, dictitem_T, typval_T, typval_vval_union, varnumber_T,
};
use core::ffi::{c_char, c_int, c_void};

/// A cleared typval.
const EMPTY_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// The byte offset from a `dictitem_T`'s inline key to the item itself, as
/// the C's `TV_DICT_HI2DI` spells it.
const DI_KEY_OFFSET: isize = 17;

/// A one-character String typval owning a copy of `len` bytes at `p`.
///
/// # Safety
/// `p` has at least `len` readable bytes.
unsafe fn owned_str(p: *const c_char, len: c_int) -> typval_T {
    typval_T {
        v_type: VAR_STRING,
        v_lock: VarLock::Unlocked,
        // SAFETY throughout: the caller's obligation; `xmemdupz` copies and terminates.
        vval: typval_vval_union {
            v_string: unsafe { xmemdupz(p as *const c_void, len as usize) } as *mut c_char,
        },
    }
}

/// A Number typval.
const fn number_tv(n: varnumber_T) -> typval_T {
    typval_T {
        v_type: VAR_NUMBER,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: n },
    }
}

/// The shared body of `max()` and `min()`.
///
/// # Safety
/// `tv` is a live argument typval and `rettv` the cleared return value.
unsafe fn max_min(tv: *const typval_T, rettv: &mut typval_T, domax: bool) {
    // SAFETY throughout: the caller's obligation; the container is only read, and the
    // dictionary walk is the C's own `TV_DICT_ITER`.
    let mut error = false;
    rettv.vval.v_number = 0;
    // Seeded at the far end so the first item always wins. An empty
    // container returns the 0 written above instead.
    let mut n: varnumber_T = if domax { VARNUMBER_MIN } else { VARNUMBER_MAX };
    let better = |i: varnumber_T, n: varnumber_T| if domax { i > n } else { i < n };
    let tv = unsafe { &*tv };
    match tv.v_type {
        VAR_LIST => {
            if unsafe { tv_list_len(tv.vval.v_list) } == 0 {
                return;
            }
            let mut li = unsafe { tv_list_first(tv.vval.v_list) };
            while !li.is_null() {
                let i = unsafe { tv_get_number_chk(&raw const (*li).li_tv, &raw mut error) };
                if error {
                    return;
                }
                if better(i, n) {
                    n = i;
                }
                li = unsafe { (*li).li_next };
            }
        }
        VAR_DICT => {
            if unsafe { tv_dict_len(tv.vval.v_dict) } == 0 {
                return;
            }
            let ht = unsafe { &raw mut (*tv.vval.v_dict).dv_hashtab };
            let mut todo = unsafe { (*ht).ht_used };
            let mut hi = unsafe { (*ht).ht_array };
            while todo != 0 {
                let key = unsafe { (*hi).hi_key };
                if !key.is_null() && !core::ptr::eq(key, &raw const hash_removed) {
                    todo -= 1;
                    let di = unsafe { key.offset(-DI_KEY_OFFSET) } as *mut dictitem_T;
                    let i = unsafe { tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error) };
                    if error {
                        return;
                    }
                    if better(i, n) {
                        n = i;
                    }
                }
                hi = unsafe { hi.add(1) };
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
    rettv.vval.v_number = n;
}

/// `max({expr})`.
pub unsafe fn f_max(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the argument is the frame's.
    unsafe { max_min(args.ptr(0), rettv, true) }
}

/// `min({expr})`.
pub unsafe fn f_min(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the argument is the frame's.
    unsafe { max_min(args.ptr(0), rettv, false) }
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
#[derive(Clone, Copy)]
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
/// `rettv`.
///
/// Returns `false` when the fold should stop — the call failed, or it
/// reported an error of its own.
///
/// # Safety
/// `expr` is a live callable typval and `rettv` the fold's accumulator.
unsafe fn fold_step(
    expr: *mut typval_T,
    rettv: &mut typval_T,
    item: typval_T,
    cleanup: Cleanup,
    called_emsg_start: c_int,
) -> bool {
    // SAFETY throughout: the caller's obligation. `argv` outlives the call, and
    // `rettv`'s old value moves into `argv[0]`.
    let mut argv = [EMPTY_TV; 3];
    argv[0] = *rettv;
    argv[1] = item;
    if cleanup.blank_rettv {
        rettv.v_type = VAR_UNKNOWN;
    }
    let r = unsafe { eval_expr_typval(expr, true, argv.as_mut_ptr(), 2, rettv) };
    if cleanup.clear_acc {
        unsafe { tv_clear(&raw mut argv[0]) };
    }
    if cleanup.clear_item {
        unsafe { tv_clear(&raw mut argv[1]) };
    }
    r.is_ok() && called_emsg.get() == called_emsg_start
}

/// `reduce()` over a List.
///
/// # Safety
/// `args` is the call frame and `rettv` its cleared return value.
unsafe fn reduce_list(args: Args<'_>, expr: *mut typval_T, rettv: &mut typval_T) {
    // SAFETY: the caller's obligation; the list is locked against
    // modification for the whole fold and restored afterwards.
    let l = unsafe { args.get(0).vval.v_list };
    let called_emsg_start = called_emsg.get();
    let (initial, mut li) = if args.has(2) {
        (*args.get(2), unsafe { tv_list_first(l) })
    } else {
        if unsafe { tv_list_len(l) } == 0 {
            semsg!("E998: Reduce of an empty {} with no initial value", "List");
            return;
        }
        let first = unsafe { tv_list_first(l) };
        (unsafe { (*first).li_tv }, unsafe { (*first).li_next })
    };
    unsafe { tv_copy(&raw const initial, rettv) };
    // A null List is `v:_null_list`: nothing to fold, and nothing to
    // lock either.
    if l.is_null() {
        return;
    }
    let prev_locked = unsafe { tv_list_locked(l) };
    unsafe { tv_list_set_lock(l, VarLock::Fixed) };
    while !li.is_null() {
        if !unsafe { fold_step(expr, rettv, (*li).li_tv, LIST_CLEANUP, called_emsg_start) } {
            break;
        }
        li = unsafe { (*li).li_next };
    }
    unsafe { tv_list_set_lock(l, prev_locked) };
}

/// `reduce()` over a String, one composed character at a time.
///
/// # Safety
/// `args` is the call frame and `rettv` its cleared return value.
unsafe fn reduce_string(args: Args<'_>, expr: *mut typval_T, rettv: &mut typval_T) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the caller's obligation. `p` walks a NUL-terminated string
    // owned by the argument, which the fold cannot modify.
    let mut p = arg_string(&mut numbuf, args.get(0));
    let called_emsg_start = called_emsg.get();
    if !args.has(2) {
        if unsafe { *p } as c_int == NUL {
            semsg!(
                "E998: Reduce of an empty {} with no initial value",
                "String"
            );
            return;
        }
        // With no initial value the first character is it.
        let len = unsafe { utfc_ptr2len(p) };
        *rettv = unsafe { owned_str(p, len) };
        p = unsafe { p.add(len as usize) };
    } else if check_arg(args, 2, tv_check_for_string_arg).is_err() {
        return;
    } else {
        arg_copy(args.get(2), rettv);
    }
    while unsafe { *p } as c_int != NUL {
        let len = unsafe { utfc_ptr2len(p) };
        let item = unsafe { owned_str(p, len) };
        // SAFETY: `expr` is the caller's callback and `rettv` the running
        // accumulator; `item` is the character just measured.
        if !unsafe { fold_step(expr, rettv, item, STRING_CLEANUP, called_emsg_start) } {
            break;
        }
        p = unsafe { p.add(len as usize) };
    }
}

/// `reduce()` over a Blob, one byte at a time.
///
/// # Safety
/// `args` is the call frame and `rettv` its cleared return value.
unsafe fn reduce_blob(args: Args<'_>, expr: *mut typval_T, rettv: &mut typval_T) {
    // SAFETY: the caller's obligation; the blob is re-measured every pass,
    // as the C does, so a fold that shortens it cannot walk off the end.
    let b: *const blob_T = unsafe { args.get(0).vval.v_blob };
    let called_emsg_start = called_emsg.get();
    let (initial, mut i) = if args.has(2) {
        if check_arg(args, 2, tv_check_for_number_arg).is_err() {
            return;
        }
        (*args.get(2), 0)
    } else {
        if unsafe { tv_blob_len(b) } == 0 {
            semsg!("E998: Reduce of an empty {} with no initial value", "Blob");
            return;
        }
        (number_tv(unsafe { tv_blob_get(b, 0) } as varnumber_T), 1)
    };
    unsafe { tv_copy(&raw const initial, rettv) };
    while i < unsafe { tv_blob_len(b) } {
        let item = number_tv(unsafe { tv_blob_get(b, i) } as varnumber_T);
        // SAFETY: as the String walk above; `i` is inside the Blob.
        if !unsafe { fold_step(expr, rettv, item, BLOB_CLEANUP, called_emsg_start) } {
            return;
        }
        i += 1;
    }
}

/// `reduce({object}, {func} [, {initial}])`.
pub unsafe fn f_reduce(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: everything read below is the frame's.
    let ty = args.ty(0);
    if ty != VAR_STRING && ty != VAR_LIST && ty != VAR_BLOB {
        emsg(gettext(e_string_list_or_blob_required));
        return;
    }
    // The callable is checked for emptiness here rather than by
    // `eval_expr_typval`, so that an empty name reports E1132 instead of
    // an "unknown function" for the empty string.
    let func_name = match args.ty(1) {
        VAR_FUNC => unsafe { args.get(1).vval.v_string },
        VAR_PARTIAL => unsafe { partial_name(args.get(1).vval.v_partial) },
        _ => arg_string(&mut numbuf, args.get(1)),
    };
    if func_name.is_null() || unsafe { *func_name } as c_int == NUL {
        emsg(gettext(e_missing_function_argument));
        return;
    }
    let expr = args.ptr(1);
    match ty {
        VAR_LIST => unsafe { reduce_list(args, expr, rettv) },
        VAR_STRING => unsafe { reduce_string(args, expr, rettv) },
        _ => unsafe { reduce_blob(args, expr, rettv) },
    }
}
