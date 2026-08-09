//! Folding a sequence down to one value: `reduce()`, `max()`, `min()`.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::{
    FAIL, NUL, VARNUMBER_MAX, VARNUMBER_MIN, e_missing_function_argument,
    e_string_list_or_blob_required,
};
use crate::semsg_c;
use crate::src::nvim::eval::typval::{
    tv_blob_get, tv_blob_len, tv_check_for_number_arg, tv_check_for_string_arg, tv_clear, tv_copy,
    tv_dict_len, tv_get_number_chk, tv_get_string, tv_list_first, tv_list_len, tv_list_locked,
    tv_list_set_lock,
};
use crate::src::nvim::eval::{eval_expr_typval, partial_name};
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::main::{
    called_emsg, e_listdictarg, e_reduce_of_an_empty_str_with_no_initial_value,
};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memory::xmemdupz;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::types::{
    EvalFuncData, VAR_BLOB, VAR_DICT, VAR_FIXED, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL,
    VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, blob_T, dictitem_T, typval_T, typval_vval_union,
    varnumber_T,
};
use core::ffi::{c_char, c_int, c_void};

/// A cleared typval.
const EMPTY_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
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
        v_lock: VAR_UNLOCKED,
        // SAFETY: the caller's obligation; `xmemdupz` copies and terminates.
        vval: typval_vval_union {
            v_string: unsafe { xmemdupz(p as *const c_void, len as usize) } as *mut c_char,
        },
    }
}

/// A Number typval.
const fn number_tv(n: varnumber_T) -> typval_T {
    typval_T {
        v_type: VAR_NUMBER,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: n },
    }
}

/// The shared body of `max()` and `min()`.
///
/// # Safety
/// `tv` is a live argument typval and `rettv` the cleared return value.
unsafe fn max_min(tv: *const typval_T, rettv: &mut typval_T, domax: bool) {
    // SAFETY: the caller's obligation; the container is only read, and the
    // dictionary walk is the C's own `TV_DICT_ITER`.
    unsafe {
        let mut error = false;
        rettv.vval.v_number = 0;
        // Seeded at the far end so the first item always wins. An empty
        // container returns the 0 written above instead.
        let mut n: varnumber_T = if domax { VARNUMBER_MIN } else { VARNUMBER_MAX };
        let better = |i: varnumber_T, n: varnumber_T| if domax { i > n } else { i < n };
        let tv = &*tv;
        match tv.v_type {
            VAR_LIST => {
                if tv_list_len(tv.vval.v_list) == 0 {
                    return;
                }
                let mut li = tv_list_first(tv.vval.v_list);
                while !li.is_null() {
                    let i = tv_get_number_chk(&raw const (*li).li_tv, &raw mut error);
                    if error {
                        return;
                    }
                    if better(i, n) {
                        n = i;
                    }
                    li = (*li).li_next;
                }
            }
            VAR_DICT => {
                if tv_dict_len(tv.vval.v_dict) == 0 {
                    return;
                }
                let ht = &raw mut (*tv.vval.v_dict).dv_hashtab;
                let mut todo = (*ht).ht_used;
                let mut hi = (*ht).ht_array;
                while todo != 0 {
                    let key = (*hi).hi_key;
                    if !key.is_null() && key != &raw const hash_removed as *mut c_char {
                        todo -= 1;
                        let di = key.offset(-DI_KEY_OFFSET) as *mut dictitem_T;
                        let i = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error);
                        if error {
                            return;
                        }
                        if better(i, n) {
                            n = i;
                        }
                    }
                    hi = hi.add(1);
                }
            }
            _ => {
                semsg_c!(
                    gettext(e_listdictarg.ptr() as *const c_char),
                    if domax {
                        c"max()".as_ptr()
                    } else {
                        c"min()".as_ptr()
                    },
                );
                return;
            }
        }
        rettv.vval.v_number = n;
    }
}

/// `max({expr})`.
pub unsafe extern "C" fn f_max(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the argument is the frame's.
    unsafe { max_min(args.ptr(0), rettv, true) }
}

/// `min({expr})`.
pub unsafe extern "C" fn f_min(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
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
    // SAFETY: the caller's obligation. `argv` outlives the call, and
    // `rettv`'s old value moves into `argv[0]`.
    unsafe {
        let mut argv = [EMPTY_TV; 3];
        argv[0] = *rettv;
        argv[1] = item;
        if cleanup.blank_rettv {
            rettv.v_type = VAR_UNKNOWN;
        }
        let r = eval_expr_typval(expr, true, argv.as_mut_ptr(), 2, rettv);
        if cleanup.clear_acc {
            tv_clear(&raw mut argv[0]);
        }
        if cleanup.clear_item {
            tv_clear(&raw mut argv[1]);
        }
        r != FAIL && called_emsg.get() == called_emsg_start
    }
}

/// `reduce()` over a List.
///
/// # Safety
/// `args` is the call frame and `rettv` its cleared return value.
unsafe fn reduce_list(args: Args<'_>, expr: *mut typval_T, rettv: &mut typval_T) {
    // SAFETY: the caller's obligation; the list is locked against
    // modification for the whole fold and restored afterwards.
    unsafe {
        let l = args.get(0).vval.v_list;
        let called_emsg_start = called_emsg.get();
        let (initial, mut li) = if args.has(2) {
            (*args.get(2), tv_list_first(l))
        } else {
            if tv_list_len(l) == 0 {
                semsg_c!(
                    gettext(e_reduce_of_an_empty_str_with_no_initial_value.ptr() as *const c_char),
                    c"List".as_ptr(),
                );
                return;
            }
            let first = tv_list_first(l);
            ((*first).li_tv, (*first).li_next)
        };
        tv_copy(&raw const initial, rettv);
        // A null List is `v:_null_list`: nothing to fold, and nothing to
        // lock either.
        if l.is_null() {
            return;
        }
        let prev_locked = tv_list_locked(l);
        tv_list_set_lock(l, VAR_FIXED);
        while !li.is_null() {
            if !fold_step(expr, rettv, (*li).li_tv, LIST_CLEANUP, called_emsg_start) {
                break;
            }
            li = (*li).li_next;
        }
        tv_list_set_lock(l, prev_locked);
    }
}

/// `reduce()` over a String, one composed character at a time.
///
/// # Safety
/// `args` is the call frame and `rettv` its cleared return value.
unsafe fn reduce_string(args: Args<'_>, expr: *mut typval_T, rettv: &mut typval_T) {
    // SAFETY: the caller's obligation. `p` walks a NUL-terminated string
    // owned by the argument, which the fold cannot modify.
    unsafe {
        let mut p = tv_get_string(args.ptr(0));
        let called_emsg_start = called_emsg.get();
        if !args.has(2) {
            if *p as c_int == NUL {
                semsg_c!(
                    gettext(e_reduce_of_an_empty_str_with_no_initial_value.ptr() as *const c_char),
                    c"String".as_ptr(),
                );
                return;
            }
            // With no initial value the first character is it.
            let len = utfc_ptr2len(p);
            *rettv = owned_str(p, len);
            p = p.add(len as usize);
        } else if tv_check_for_string_arg(args.ptr(0), 2) == FAIL {
            return;
        } else {
            tv_copy(args.ptr(2), rettv);
        }
        while *p as c_int != NUL {
            let len = utfc_ptr2len(p);
            if !fold_step(
                expr,
                rettv,
                owned_str(p, len),
                STRING_CLEANUP,
                called_emsg_start,
            ) {
                break;
            }
            p = p.add(len as usize);
        }
    }
}

/// `reduce()` over a Blob, one byte at a time.
///
/// # Safety
/// `args` is the call frame and `rettv` its cleared return value.
unsafe fn reduce_blob(args: Args<'_>, expr: *mut typval_T, rettv: &mut typval_T) {
    // SAFETY: the caller's obligation; the blob is re-measured every pass,
    // as the C does, so a fold that shortens it cannot walk off the end.
    unsafe {
        let b: *const blob_T = args.get(0).vval.v_blob;
        let called_emsg_start = called_emsg.get();
        let (initial, mut i) = if args.has(2) {
            if tv_check_for_number_arg(args.ptr(0), 2) == FAIL {
                return;
            }
            (*args.get(2), 0)
        } else {
            if tv_blob_len(b) == 0 {
                semsg_c!(
                    gettext(e_reduce_of_an_empty_str_with_no_initial_value.ptr() as *const c_char),
                    c"Blob".as_ptr(),
                );
                return;
            }
            (number_tv(tv_blob_get(b, 0) as varnumber_T), 1)
        };
        tv_copy(&raw const initial, rettv);
        while i < tv_blob_len(b) {
            if !fold_step(
                expr,
                rettv,
                number_tv(tv_blob_get(b, i) as varnumber_T),
                BLOB_CLEANUP,
                called_emsg_start,
            ) {
                return;
            }
            i += 1;
        }
    }
}

/// `reduce({object}, {func} [, {initial}])`.
pub unsafe extern "C" fn f_reduce(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: everything read below is the frame's.
    unsafe {
        let ty = args.ty(0);
        if ty != VAR_STRING && ty != VAR_LIST && ty != VAR_BLOB {
            emsg(gettext(
                e_string_list_or_blob_required.ptr() as *const c_char
            ));
            return;
        }
        // The callable is checked for emptiness here rather than by
        // `eval_expr_typval`, so that an empty name reports E1132 instead of
        // an "unknown function" for the empty string.
        let func_name = match args.ty(1) {
            VAR_FUNC => args.get(1).vval.v_string,
            VAR_PARTIAL => partial_name(args.get(1).vval.v_partial),
            _ => tv_get_string(args.ptr(1)),
        };
        if func_name.is_null() || *func_name as c_int == NUL {
            emsg(gettext(e_missing_function_argument.ptr() as *const c_char));
            return;
        }
        let expr = args.ptr(1);
        match ty {
            VAR_LIST => reduce_list(args, expr, rettv),
            VAR_STRING => reduce_string(args, expr, rettv),
            _ => reduce_blob(args, expr, rettv),
        }
    }
}
