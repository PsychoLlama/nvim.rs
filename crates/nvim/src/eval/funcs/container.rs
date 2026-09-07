//! Reading a List, Dict or Blob: `get()`, `empty()`, `index()`,
//! `flatten()` and friends.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::TV_TRANSLATE;
use super::args::{Args, frame};
use super::wrappers::{
    arg_copy, arg_number_chk, arg_string, check_arg, dict_alloc_ret, list_alloc_ret,
};
use crate::cstr;
use crate::eval::typval::{
    NumBuf, tv_blob_get, tv_blob_len, tv_check_for_list_or_blob_arg, tv_check_for_opt_bool_arg,
    tv_check_for_opt_dict_arg, tv_check_for_string_or_func_arg, tv_clear, tv_copy,
    tv_dict_add_bool, tv_dict_add_nr, tv_dict_find, tv_dict_get_number_def, tv_dict_len,
    tv_dict_set_ret, tv_equal, tv_get_bool_chk, tv_is_func, tv_list_append_tv, tv_list_copy,
    tv_list_find, tv_list_first, tv_list_flatten, tv_list_len, tv_list_locked, tv_list_ref,
    tv_list_uidx, value_check_lock,
};
use crate::eval::userfunc::{func_ref, get_func_arity, printable_func_name};
use crate::eval::vars::{
    get_vim_var_tv, prepare_vimvar, restore_vimvar, set_vim_var_nr, set_vim_var_type,
};
use crate::eval::{eval_expr_typval, get_copy_id, partial_name, var_item_copy};
use crate::memory::xstrdup;
use crate::message::e_listblobreq;
use crate::message::state::{called_emsg, did_emsg};
use crate::message::{emsg, internal_error};
use crate::message_fmt::c_str;
use crate::os::cshim::gettext;
use crate::semsg;
use crate::types::{
    Blob, BoolVarValue, EvalFuncData, List, ListItem, NUL, Partial, Refcount, TypVal, VAR_BLOB,
    VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_SPECIAL,
    VAR_STRING, VAR_TYPE_BLOB, VAR_TYPE_BOOL, VAR_TYPE_DICT, VAR_TYPE_FLOAT, VAR_TYPE_FUNC,
    VAR_TYPE_LIST, VAR_TYPE_NUMBER, VAR_TYPE_SPECIAL, VAR_TYPE_STRING, VAR_UNKNOWN, VarLock,
    VarNumber, Vv, kBoolVarTrue, kSpecialVarNull, typval_vval_union,
};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// A cleared typval, the shape both dispatchers start every slot from.
const NIL: TypVal = TypVal {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// `copy({expr})` — one level deep.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_copy(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: `args.ptr(0)` and `result` are live typvals.
    let _ = unsafe { var_item_copy(ptr::null(), args.ptr(0), result, false, 0) };
}

/// `deepcopy({expr} [, {noref}])`.
///
/// Without `noref` the copy is given a copy id, which is what lets it
/// reproduce a self-referential structure rather than recursing forever.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_deepcopy(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY throughout: the arguments and `result` are live typvals.
    if check_arg(args, 1, tv_check_for_opt_bool_arg).is_err() {
        return;
    }
    let noref = args.has(1) && unsafe { tv_get_bool_chk(args.ptr(1), ptr::null_mut()) } != 0;
    let copy_id = if noref { 0 } else { get_copy_id() };
    let _ = unsafe { var_item_copy(ptr::null(), args.ptr(0), result, true, copy_id) };
}

/// `empty({expr})` — what "empty" means for each type.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_empty(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    let tv = args.get(0);
    // SAFETY throughout: every read is guarded by the type tag that says
    // which union member is live. A String, List, Dict or Blob pointer may
    // still be null, which each reader treats as empty.
    let empty = match tv.v_type {
        VAR_STRING | VAR_FUNC => {
            let s = tv.string_or_func_name();
            s.is_null() || unsafe { *s } == NUL as c_char
        }
        VAR_PARTIAL => false,
        VAR_NUMBER => (tv.number_or_zero()) == 0,
        VAR_FLOAT => (tv.float_or_zero()) == 0.0,
        VAR_LIST => (unsafe { tv_list_len(tv.list_or_null()) }) == 0,
        VAR_DICT => (unsafe { tv_dict_len(tv.dict_or_null()) }) == 0,
        VAR_BLOB => (unsafe { tv_blob_len(tv.blob_or_null()) }) == 0,
        VAR_SPECIAL => tv.as_special() == Some(kSpecialVarNull),
        // A Bool other than the two named values leaves the answer at its
        // "empty" default, as upstream's switch does.
        VAR_BOOL => tv.as_bool() != Some(kBoolVarTrue),
        VAR_UNKNOWN => {
            unsafe { internal_error(c"f_empty(UNKNOWN)".as_ptr()) };
            true
        }
        _ => true,
    };
    result.vval.v_number = empty as VarNumber;
}

/// `flatten({list} [, {maxdepth}])` — in place.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_flatten(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    flatten_common(args, result, false);
}

/// `flattennew({list} [, {maxdepth}])` — into a copy.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_flattennew(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    flatten_common(args, result, true);
}

/// The shared body. `make_copy` is what separates `flattennew()` from
/// `flatten()`: the copying form never checks the source for a lock,
/// because it does not write to it.
fn flatten_common(args: Args<'_>, result: &mut TypVal, make_copy: bool) {
    // SAFETY throughout: the tag checked here says which union member is
    // live, and the List it names outlives the call.
    if args.ty(0) != VAR_LIST {
        let arg0 = "flatten()";
        semsg!("E686: Argument of {arg0} must be a List");
        return;
    }
    let maxdepth = if !args.has(1) {
        999_999
    } else {
        let mut error = false;
        let depth = arg_number_chk(args.get(1), Some(&mut error)) as c_int;
        if error {
            return;
        }
        if depth < 0 {
            let msg = c"E900: maxdepth must be non-negative number";
            emsg(gettext(msg));
            return;
        }
        depth
    };

    let mut list = args.get(0).list_or_null();
    result.v_type = VAR_LIST;
    result.vval.v_list = list;
    if list.is_null() {
        return;
    }
    if make_copy {
        list = unsafe { tv_list_copy(ptr::null(), list, false, get_copy_id()) };
        result.vval.v_list = list;
        if list.is_null() {
            return;
        }
    } else {
        // SAFETY: `list` is the live List argument 0 named.
        let lock = unsafe { tv_list_locked(list) };
        let what = c"flatten() argument".as_ptr();
        if unsafe { value_check_lock(lock, what, TV_TRANSLATE as usize) } {
            return;
        }
        unsafe { tv_list_ref(list) };
    }
    // SAFETY: `list` is the live List argument 0 named.
    let len = unsafe { tv_list_len(list) } as i64;
    unsafe { tv_list_flatten(list, ptr::null_mut(), len, maxdepth as i64) };
}

/// `get({container}, {key} [, {default}])` — for a Blob, List, Dict,
/// Funcref or Partial.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_get(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY throughout: the arguments and `result` are live typvals; each union read
    // is guarded by the type tag above it.
    let found: *mut TypVal = match args.ty(0) {
        VAR_BLOB => get_from_blob(args, result),
        VAR_LIST => get_from_list(args),
        VAR_DICT => get_from_dict(args),
        _ if tv_is_func(*args.get(0)) => {
            if !get_from_func(args, result) {
                return;
            }
            // Only the "dict" selector falls through to the default
            // handling below, and only when the Partial had no dict.
            ptr::null_mut()
        }
        _ => {
            let arg0 = "get()";
            semsg!("E896: Argument of {arg0} must be a List, Dictionary or Blob");
            ptr::null_mut()
        }
    };
    if !found.is_null() {
        unsafe { tv_copy(found, result) };
    } else if args.has(2) {
        arg_copy(args.get(2), result);
    }
}

/// `get()` over a Blob. The caller has checked the tag, which is what
/// makes the union read below the right member.
fn get_from_blob(args: Args<'_>, result: &mut TypVal) -> *mut TypVal {
    // SAFETY throughout: the caller has checked the tag, so the union
    // holds the Blob the reads below name.
    let mut error = false;
    let mut idx = arg_number_chk(args.get(1), Some(&mut error)) as c_int;
    if error {
        return ptr::null_mut();
    }
    let blob = args.get(0).blob_or_null();
    result.v_type = VAR_NUMBER;
    if idx < 0 {
        idx += unsafe { tv_blob_len(blob) };
    }
    if idx < 0 || idx >= unsafe { tv_blob_len(blob) } {
        // Out of range is -1 rather than the default argument.
        result.vval.v_number = -1;
        return ptr::null_mut();
    }
    result.vval.v_number = unsafe { tv_blob_get(blob, idx) } as VarNumber;
    // The value is already in place; copying it onto itself is a no-op
    // and is what upstream does.
    result
}

/// `get()` over a List. The caller has checked the tag.
fn get_from_list(args: Args<'_>) -> *mut TypVal {
    // SAFETY: the caller's obligation.
    let l = args.get(0).list_or_null();
    if l.is_null() {
        return ptr::null_mut();
    }
    let mut error = false;
    let idx = arg_number_chk(args.get(1), Some(&mut error)) as c_int;
    let li = unsafe { tv_list_find(l, idx) };
    if error || li.is_null() {
        return ptr::null_mut();
    }
    unsafe { &raw mut (*li).li_tv }
}

/// `get()` over a Dictionary. The caller has checked the tag.
fn get_from_dict(args: Args<'_>) -> *mut TypVal {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's obligation.
    let d = args.get(0).dict_or_null();
    if d.is_null() {
        return ptr::null_mut();
    }
    let di = unsafe { tv_dict_find(d, arg_string(&mut numbuf, args.get(1)), -1) };
    if di.is_null() {
        return ptr::null_mut();
    }
    unsafe { &raw mut (*di).di_tv }
}

/// Answer `get()` for a Funcref or Partial. Returns whether the caller
/// should fall through to the default-argument handling, which only the
/// "dict" selector does — and then only when the Partial had no dict.
fn get_from_func(args: Args<'_>, result: &mut TypVal) -> bool {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the caller has checked the tag. A plain Funcref is answered through
    // a stack Partial holding just its name, which lives as long as this
    // call and is never stored.
    let mut fref = Partial {
        pt_refcount: Refcount::ZERO,
        pt_copy_id: 0,
        pt_name: ptr::null_mut(),
        pt_func: ptr::null_mut(),
        pt_auto: false,
        pt_argc: 0,
        pt_argv: ptr::null_mut(),
        pt_dict: ptr::null_mut(),
    };
    let pt = if args.ty(0) == VAR_PARTIAL {
        args.get(0).partial_or_null()
    } else {
        fref.pt_name = args.get(0).func_name_or_null();
        &raw mut fref
    };
    let what = arg_string(&mut numbuf, args.get(1));
    match unsafe { CStr::from_ptr(what) }.to_bytes() {
        b"func" | b"name" => {
            let mut name: *const c_char = unsafe { partial_name(pt) };
            // "func" hands back a Funcref, "name" a plain String.
            result.v_type = if unsafe { *what } == b'f' as c_char {
                VAR_FUNC
            } else {
                VAR_STRING
            };
            debug_assert!(!name.is_null());
            if result.v_type == VAR_FUNC {
                unsafe { func_ref(name as *mut c_char) };
            }
            // A lambda has no name of its own; "name" shows the
            // printable form instead.
            if unsafe { *what } == b'n' as c_char
                && unsafe { (*pt).pt_name }.is_null()
                && !unsafe { (*pt).pt_func }.is_null()
            {
                name = unsafe { printable_func_name((*pt).pt_func) };
            }
            result.vval.v_string = unsafe { xstrdup(name) };
        }
        b"dict" => {
            if !unsafe { (*pt).pt_dict }.is_null() {
                unsafe { tv_dict_set_ret(result, (*pt).pt_dict) };
            }
            // "dict" is the only selector that falls through to the
            // default-argument handling, and it does so whether or not
            // it just found a dict — so a default given alongside it
            // wins. Upstream is the same.
            return true;
        }
        b"args" => {
            result.v_type = VAR_LIST;
            let list = unsafe { list_alloc_ret(result, (*pt).pt_argc as isize) };
            for i in 0..unsafe { (*pt).pt_argc } {
                unsafe { tv_list_append_tv(list, (*pt).pt_argv.offset(i as isize)) };
            }
        }
        b"arity" => unsafe { func_arity(pt, result) },
        _ => {
            // Kept on the variadic message call: `what` is arbitrary
            // user bytes and a Rust format string can only carry UTF-8.
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let what = unsafe { c_str(what) };
            semsg!("E475: Invalid argument: {what}");
        }
    }
    false
}

/// `get(Funcref, "arity")` — what the function still wants after the
/// Partial's bound arguments are subtracted.
///
/// # Safety
/// `pt` is a live Partial and `result` is the cleared return value.
unsafe fn func_arity(pt: *mut Partial, result: &mut TypVal) {
    let (mut required, mut optional, mut varargs) = (0, 0, false);
    let name = unsafe { partial_name(pt) };
    let (req, opt, var) = (&raw mut required, &raw mut optional, &raw mut varargs);
    let _ = unsafe { get_func_arity(name, req, opt, var) };
    result.v_type = VAR_DICT;
    dict_alloc_ret(result);
    let dict = result.dict_or_null();
    // The bound arguments cover the required ones first.
    if unsafe { (*pt).pt_argc } >= required + optional {
        optional = 0;
        required = 0;
    } else if unsafe { (*pt).pt_argc } > required {
        optional -= unsafe { (*pt).pt_argc } - required;
        required = 0;
    } else {
        required -= unsafe { (*pt).pt_argc };
    }
    let _ = unsafe { tv_dict_add_nr(dict, c"required".as_ptr(), 8, required as VarNumber) };
    let _ = unsafe { tv_dict_add_nr(dict, c"optional".as_ptr(), 8, optional as VarNumber) };
    let _ = unsafe { tv_dict_add_bool(dict, c"varargs".as_ptr(), 7, varargs as BoolVarValue) };
}

/// `index({object}, {expr} [, {start} [, {ic}]])`.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_index(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    result.vval.v_number = -1;
    // SAFETY throughout: the arguments are live typvals.
    match args.ty(0) {
        VAR_BLOB => index_blob(args, result),
        VAR_LIST => index_list(args, result),
        _ => {
            emsg(gettext(e_listblobreq));
        }
    }
}

/// `index()` over a Blob. The caller has checked the tag.
fn index_blob(args: Args<'_>, result: &mut TypVal) {
    // SAFETY throughout: the caller has checked the tag, so the union
    // holds the Blob the reads below name.
    let mut start: c_int = 0;
    if args.has(2) {
        let mut error = false;
        start = arg_number_chk(args.get(2), Some(&mut error)) as c_int;
        if error {
            return;
        }
    }
    let b = args.get(0).blob_or_null();
    if b.is_null() {
        return;
    }
    if start < 0 {
        start = (unsafe { tv_blob_len(b) } + start).max(0);
    }
    for idx in start..unsafe { tv_blob_len(b) } {
        let mut tv = NIL;
        tv.v_type = VAR_NUMBER;
        tv.vval.v_number = unsafe { tv_blob_get(b, idx) } as VarNumber;
        // The Blob branch never reads argument 3, so a Blob search is
        // always case-sensitive however 'ic' was spelled. Upstream is
        // the same; the flag only reaches the List branch.
        if unsafe { tv_equal(&raw mut tv, args.ptr(1), false) } {
            result.vval.v_number = idx as VarNumber;
            return;
        }
    }
}

/// `index()` over a List. The caller has checked the tag.
fn index_list(args: Args<'_>, result: &mut TypVal) {
    // SAFETY: the caller's obligation.
    let l = args.get(0).list_or_null();
    if l.is_null() {
        return;
    }
    let mut idx: c_int = 0;
    let mut item = unsafe { tv_list_first(l) };
    let mut ic = false;
    if args.has(2) {
        let mut error = false;
        idx = unsafe { tv_list_uidx(l, arg_number_chk(args.get(2), Some(&mut error)) as c_int) };
        if error || idx == -1 {
            item = ptr::null_mut();
        } else {
            item = unsafe { tv_list_find(l, idx) };
            debug_assert!(!item.is_null());
        }
        if args.has(3) {
            ic = arg_number_chk(args.get(3), Some(&mut error)) != 0;
            if error {
                item = ptr::null_mut();
            }
        }
    }
    while !item.is_null() {
        if unsafe { tv_equal(&raw mut (*item).li_tv, args.ptr(1), ic) } {
            result.vval.v_number = idx as VarNumber;
            return;
        }
        item = unsafe { (*item).li_next };
        idx += 1;
    }
}

/// `indexof({object}, {expr} [, {opts}])` — the first index whose value
/// satisfies `expr`, which sees the item as `v:key` and `v:val`.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_indexof(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    result.vval.v_number = -1;
    // SAFETY throughout: the arguments are live typvals; the two `v:` variables are
    // saved and put back around the search whatever it does.
    if check_arg(args, 0, tv_check_for_list_or_blob_arg).is_err()
        || check_arg(args, 1, tv_check_for_string_or_func_arg).is_err()
        || check_arg(args, 2, tv_check_for_opt_dict_arg).is_err()
    {
        return;
    }
    // An empty expression matches nothing rather than everything.
    let expr = args.get(1);
    let vacuous = match expr.v_type {
        VAR_STRING => {
            expr.string_or_null().is_null() || unsafe { *expr.string_or_null() } == NUL as c_char
        }
        // Upstream names `v_partial` here; under `VAR_FUNC` that is the
        // same word as the name, so this asks whether the Funcref has one.
        VAR_FUNC => expr.func_name_or_null().is_null(),
        _ => false,
    };
    if vacuous {
        return;
    }
    let startidx = if args.ty(2) == VAR_DICT {
        unsafe { tv_dict_get_number_def(args.get(2).dict_or_null(), c"startidx".as_ptr(), 0) }
    } else {
        0
    };

    let (mut save_val, mut save_key) = (NIL, NIL);
    unsafe { prepare_vimvar(Vv::Val, &raw mut save_val) };
    unsafe { prepare_vimvar(Vv::Key, &raw mut save_key) };
    let saved_did_emsg = did_emsg.get();
    did_emsg.set(0);
    result.vval.v_number = if args.ty(0) == VAR_BLOB {
        unsafe { indexof_blob(args.get(0).blob_or_null(), startidx, args.ptr(1)) }
    } else {
        unsafe { indexof_list(args.get(0).list_or_null(), startidx, args.ptr(1)) }
    };
    unsafe { restore_vimvar(Vv::Key, &raw mut save_key) };
    unsafe { restore_vimvar(Vv::Val, &raw mut save_val) };
    // As `printf()`: an error raised before this call survives, one
    // raised inside it does not.
    did_emsg.set(did_emsg.get() | saved_did_emsg);
}

/// Evaluate `indexof()`'s predicate against the `v:key`/`v:val` already in
/// place. A failed evaluation, and a result that is not coercible to a
/// Bool, both read as "no match".
///
/// # Safety
/// `expr` is a live String or Funcref typval.
unsafe fn indexof_matches(expr: *mut TypVal) -> bool {
    // SAFETY throughout: the caller's obligation; `argv` and `newtv` are locals that
    // outlive the evaluation, and `newtv` is cleared before returning.
    let mut argv = [
        unsafe { *get_vim_var_tv(Vv::Key) },
        unsafe { *get_vim_var_tv(Vv::Val) },
        NIL,
    ];
    let mut newtv = NIL;
    if unsafe { eval_expr_typval(expr, false, argv.as_mut_ptr(), 2, &raw mut newtv) }.is_err() {
        return false;
    }
    let mut error = false;
    let found = unsafe { tv_get_bool_chk(&raw mut newtv, &raw mut error) };
    unsafe { tv_clear(&raw mut newtv) };
    !error && found != 0
}

/// # Safety
/// `b` is a Blob pointer or null and `expr` is a live predicate typval.
unsafe fn indexof_blob(b: *mut Blob, startidx: VarNumber, expr: *mut TypVal) -> VarNumber {
    if b.is_null() {
        return -1;
    }
    // SAFETY throughout: the caller's obligation.
    let start = if startidx < 0 {
        (unsafe { tv_blob_len(b) } as VarNumber + startidx).max(0)
    } else {
        startidx
    };
    set_vim_var_type(Vv::Key, VAR_NUMBER);
    set_vim_var_type(Vv::Val, VAR_NUMBER);
    let called_emsg_start = called_emsg.get();
    for idx in start..unsafe { tv_blob_len(b) } as VarNumber {
        set_vim_var_nr(Vv::Key, idx);
        unsafe { set_vim_var_nr(Vv::Val, tv_blob_get(b, idx as c_int) as VarNumber) };
        if unsafe { indexof_matches(expr) } {
            return idx;
        }
        // A predicate that reported an error ends the search.
        if called_emsg.get() != called_emsg_start {
            return -1;
        }
    }
    -1
}

/// # Safety
/// `l` is a List pointer or null and `expr` is a live predicate typval.
unsafe fn indexof_list(l: *mut List, startidx: VarNumber, expr: *mut TypVal) -> VarNumber {
    if l.is_null() {
        return -1;
    }
    let mut idx: VarNumber = 0;
    let mut item: *mut ListItem;
    // A zero start index is taken literally rather than run through
    // `tv_list_uidx`, so it does not have to be a valid index.
    if startidx == 0 {
        item = unsafe { tv_list_first(l) };
    } else {
        idx = unsafe { tv_list_uidx(l, startidx as c_int) } as VarNumber;
        if idx == -1 {
            item = ptr::null_mut();
        } else {
            item = unsafe { tv_list_find(l, idx as c_int) };
            debug_assert!(!item.is_null());
        }
    }
    set_vim_var_type(Vv::Key, VAR_NUMBER);
    let called_emsg_start = called_emsg.get();
    while !item.is_null() {
        set_vim_var_nr(Vv::Key, idx);
        unsafe { tv_copy(&raw mut (*item).li_tv, get_vim_var_tv(Vv::Val)) };
        let found = unsafe { indexof_matches(expr) };
        unsafe { tv_clear(get_vim_var_tv(Vv::Val)) };
        if found {
            return idx;
        }
        if called_emsg.get() != called_emsg_start {
            return -1;
        }
        item = unsafe { (*item).li_next };
        idx += 1;
    }
    -1
}

/// `len({expr})` — bytes for a String or Number, items otherwise.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_len(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, result) = frame!(args, result);
    let tv = args.get(0);
    // SAFETY throughout: every union read is guarded by the type tag above it, and a
    // Number is measured through its String spelling.
    result.vval.v_number = match tv.v_type {
        VAR_STRING | VAR_NUMBER => {
            let s = arg_string(&mut numbuf, args.get(0));
            unsafe { cstr::bytes_at(s).len() as VarNumber }
        }
        VAR_BLOB => unsafe { tv_blob_len(tv.blob_or_null()) as VarNumber },
        VAR_LIST => unsafe { tv_list_len(tv.list_or_null()) as VarNumber },
        VAR_DICT => unsafe { tv_dict_len(tv.dict_or_null()) as VarNumber },
        // The remaining tags are Unknown, Funcref, Partial, Float,
        // Bool and Special; `VarType` has no twelfth value.
        _ => {
            emsg(gettext(c"E701: Invalid type for len()"));
            return;
        }
    };
}

/// `type({expr})` — the `v:t_*` number for the value's type.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_type(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    let n: c_int = match args.ty(0) {
        VAR_NUMBER => VAR_TYPE_NUMBER as c_int,
        VAR_STRING => VAR_TYPE_STRING as c_int,
        VAR_PARTIAL | VAR_FUNC => VAR_TYPE_FUNC as c_int,
        VAR_LIST => VAR_TYPE_LIST as c_int,
        VAR_DICT => VAR_TYPE_DICT as c_int,
        VAR_FLOAT => VAR_TYPE_FLOAT as c_int,
        VAR_BOOL => VAR_TYPE_BOOL as c_int,
        VAR_SPECIAL => VAR_TYPE_SPECIAL as c_int,
        VAR_BLOB => VAR_TYPE_BLOB as c_int,
        VAR_UNKNOWN => {
            // SAFETY: a literal message.
            unsafe { internal_error(c"f_type(UNKNOWN)".as_ptr()) };
            -1
        }
        _ => -1,
    };
    result.vval.v_number = n as VarNumber;
}
