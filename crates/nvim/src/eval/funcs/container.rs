//! Reading a List, Dict or Blob: `get()`, `empty()`, `index()`,
//! `flatten()` and friends.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::TV_TRANSLATE;
use super::wrappers::{arg_copy, arg_number_chk, arg_string, dict_alloc_ret, list_alloc_ret};
use crate::cstr;
use crate::eval::typval::CallFrame;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{
    ListRef, NumBuf, blob_bytes, blob_len, dict_get_number_def, dict_len, list_copy, list_find,
    list_flatten, list_items, list_len, list_locked, list_uidx, tv_check_for_list_or_blob_arg,
    tv_check_for_opt_bool_arg, tv_check_for_opt_dict_arg, tv_check_for_string_or_func_arg,
    tv_clear, tv_copy, tv_dict_set_ret, tv_equal, tv_get_bool_chk, value_check_lock,
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
    Blob, BoolVarValue, EvalFuncData, List, NUL, Partial, Refcount, TypVal, VAR_BLOB, VAR_BOOL,
    VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_SPECIAL, VAR_STRING,
    VAR_TYPE_BLOB, VAR_TYPE_BOOL, VAR_TYPE_DICT, VAR_TYPE_FLOAT, VAR_TYPE_FUNC, VAR_TYPE_LIST,
    VAR_TYPE_NUMBER, VAR_TYPE_SPECIAL, VAR_TYPE_STRING, VAR_UNKNOWN, VarNumber, Vv, kBoolVarTrue,
    kSpecialVarNull,
};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// A cleared typval, the shape both dispatchers start every slot from.
const NIL: TypVal = TV_INITIAL_VALUE;

/// `copy({expr})` — one level deep.
pub fn f_copy(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `&args[0]` and `result` are live typvals.
    let _ = unsafe { var_item_copy(ptr::null(), &args[0], result, false, 0) };
}

/// `deepcopy({expr} [, {noref}])`.
///
/// Without `noref` the copy is given a copy id, which is what lets it
/// reproduce a self-referential structure rather than recursing forever.
pub fn f_deepcopy(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the arguments and `result` are live typvals.
    if tv_check_for_opt_bool_arg(args, 1).is_err() {
        return;
    }
    let noref = args.len() > 1 && tv_get_bool_chk(&args[1]).unwrap_or(-1) != 0;
    let copy_id = if noref { 0 } else { get_copy_id() };
    let _ = unsafe { var_item_copy(ptr::null(), &args[0], result, true, copy_id) };
}

/// `empty({expr})` — what "empty" means for each type.
pub fn f_empty(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let tv = &args[0];
    // SAFETY throughout: every read is guarded by the type tag that says
    // which union member is live. A String, List, Dict or Blob pointer may
    // still be null, which each reader treats as empty.
    let empty = match tv.v_type() {
        VAR_STRING | VAR_FUNC => {
            let s = tv.string_or_func_name();
            s.is_null() || unsafe { *s } == NUL as c_char
        }
        VAR_PARTIAL => false,
        VAR_NUMBER => (tv.number_or_zero()) == 0,
        VAR_FLOAT => (tv.float_or_zero()) == 0.0,
        VAR_LIST => tv.list_ref().is_none_or(List::is_empty),
        VAR_DICT => (dict_len(tv.dict_ref())) == 0,
        VAR_BLOB => tv.blob_ref().is_none_or(Blob::is_empty),
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
    result.write_number(empty as VarNumber);
}

/// `flatten({list} [, {maxdepth}])` — in place.
pub fn f_flatten(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    flatten_common(args, result, false);
}

/// `flattennew({list} [, {maxdepth}])` — into a copy.
pub fn f_flattennew(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    flatten_common(args, result, true);
}

/// The shared body. `make_copy` is what separates `flattennew()` from
/// `flatten()`: the copying form never checks the source for a lock,
/// because it does not write to it.
fn flatten_common(args: &[TypVal], result: &mut TypVal, make_copy: bool) {
    // SAFETY throughout: the tag checked here says which union member is
    // live, and the List it names outlives the call.
    if !args.first().is_some_and(|arg| arg.v_type() == VAR_LIST) {
        let arg0 = "flatten()";
        semsg!("E686: Argument of {arg0} must be a List");
        return;
    }
    let maxdepth = if args.len() <= 1 {
        999_999
    } else {
        let mut error = false;
        let depth = arg_number_chk(&args[1], Some(&mut error)) as c_int;
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

    let mut list = args[0].list_or_null();
    // The answer takes a reference of its own straight away, so that the
    // paths that give up below still leave `result` owning what it names.
    // SAFETY: the argument's list, live for the call.
    result.write_list(unsafe { ListRef::retained(list) });
    if list.is_null() {
        return;
    }
    if make_copy {
        // SAFETY: the argument's live list, which the copy takes a
        // reference to for the walk; no conversion, a fresh copyID.
        let copy = unsafe { list_copy(ptr::null(), ListRef::retained(list), false, get_copy_id()) };
        list = copy.as_ref().map_or(ptr::null_mut(), ListRef::as_ptr);
        // The reference taken above goes back: the answer is the copy.
        drop(result.take_list());
        result.write_list(copy);
        if list.is_null() {
            return;
        }
    } else {
        // SAFETY: `list` is the live List argument 0 named.
        let lock = list_locked(unsafe { list.as_ref() });
        let what = c"flatten() argument".as_ptr();
        if unsafe { value_check_lock(lock, what, TV_TRANSLATE as usize) } {
            return;
        }
    }
    // SAFETY: `list` is the live List argument 0 named.
    let len = list_len(unsafe { list.as_ref() }) as i64;
    list_flatten(unsafe { &mut *list }, 0, len, maxdepth as i64);
}

/// `get({container}, {key} [, {default}])` — for a Blob, List, Dict,
/// Funcref or Partial.
pub fn f_get(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the arguments and `result` are live typvals; each union read
    // is guarded by the type tag above it.
    let found: *mut TypVal = match args[0].v_type() {
        VAR_BLOB => get_from_blob(args, result),
        VAR_LIST => get_from_list(args),
        VAR_DICT => get_from_dict(args),
        _ if args[0].is_func() => {
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
        unsafe { tv_copy(&*found, result) };
    } else if args.len() > 2 {
        arg_copy(&args[2], result);
    }
}

/// `get()` over a Blob. The caller has checked the tag, which is what
/// makes the union read below the right member.
fn get_from_blob(args: &[TypVal], result: &mut TypVal) -> *mut TypVal {
    // SAFETY throughout: the caller has checked the tag, so the union
    // holds the Blob the reads below name.
    let mut error = false;
    let mut idx = arg_number_chk(&args[1], Some(&mut error)) as c_int;
    if error {
        return ptr::null_mut();
    }
    let bytes = blob_bytes(args[0].blob_ref());
    let len = c_int::try_from(bytes.len()).expect("a short blob");
    result.write_empty(VAR_NUMBER);
    if idx < 0 {
        idx += len;
    }
    if idx < 0 || idx >= len {
        // Out of range is -1 rather than the default argument.
        result.write_number(-1);
        return ptr::null_mut();
    }
    result.write_number(VarNumber::from(
        bytes[usize::try_from(idx).expect("a byte of the blob")],
    ));
    // The value is already in place; copying it onto itself is a no-op
    // and is what upstream does.
    result
}

/// `get()` over a List. The caller has checked the tag.
fn get_from_list(args: &[TypVal]) -> *mut TypVal {
    // SAFETY: the caller's obligation.
    let l = args[0].list_or_null();
    if l.is_null() {
        return ptr::null_mut();
    }
    let mut error = false;
    let idx = arg_number_chk(&args[1], Some(&mut error)) as c_int;
    let li = list_find(unsafe { l.as_mut() }, idx);
    if error || li.is_null() {
        return ptr::null_mut();
    }
    unsafe { &raw mut (*li).li_tv }
}

/// `get()` over a Dictionary. The caller has checked the tag.
fn get_from_dict(args: &[TypVal]) -> *mut TypVal {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's obligation.
    let d = args[0].dict_or_null();
    if d.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: the argument's own dictionary and a NUL-terminated key. The
    // pointer form is the answer: the caller writes through the value.
    let di = unsafe { (*d).find_ptr(cstr::bytes_at(arg_string(&mut numbuf, &args[1]))) };
    if di.is_null() {
        return ptr::null_mut();
    }
    unsafe { &raw mut (*di).di_tv }
}

/// Answer `get()` for a Funcref or Partial. Returns whether the caller
/// should fall through to the default-argument handling, which only the
/// "dict" selector does — and then only when the Partial had no dict.
fn get_from_func(args: &[TypVal], result: &mut TypVal) -> bool {
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
    let pt = if args.first().is_some_and(|arg| arg.v_type() == VAR_PARTIAL) {
        args[0].partial_or_null()
    } else {
        fref.pt_name = args[0].func_name_or_null();
        &raw mut fref
    };
    let what = arg_string(&mut numbuf, &args[1]);
    match unsafe { CStr::from_ptr(what) }.to_bytes() {
        b"func" | b"name" => {
            let mut name: *const c_char = unsafe { partial_name(pt) };
            // "func" hands back a Funcref, "name" a plain String.
            let as_funcref = unsafe { *what } == b'f' as c_char;
            debug_assert!(!name.is_null());
            if as_funcref {
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
            let owned = unsafe { xstrdup(name) };
            if as_funcref {
                result.write_func_name(owned);
            } else {
                result.write_string(owned);
            }
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
            result.write_empty(VAR_LIST);
            let list = unsafe { list_alloc_ret(result, (*pt).pt_argc as isize) };
            for i in 0..unsafe { (*pt).pt_argc } {
                unsafe { (*list).push_copy(&*(*pt).pt_argv.offset(i as isize)) };
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
    result.write_empty(VAR_DICT);
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
    let _ = unsafe { (*dict).add_number(b"required", required as VarNumber) };
    let _ = unsafe { (*dict).add_number(b"optional", optional as VarNumber) };
    let _ = unsafe { (*dict).add_bool(b"varargs", varargs as BoolVarValue) };
}

/// `index({object}, {expr} [, {start} [, {ic}]])`.
pub fn f_index(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    // SAFETY throughout: the arguments are live typvals.
    match args[0].v_type() {
        VAR_BLOB => index_blob(args, result),
        VAR_LIST => index_list(args, result),
        _ => {
            emsg(gettext(e_listblobreq));
        }
    }
}

/// `index()` over a Blob. The caller has checked the tag.
fn index_blob(args: &[TypVal], result: &mut TypVal) {
    // SAFETY throughout: the caller has checked the tag, so the union
    // holds the Blob the reads below name.
    let mut start: c_int = 0;
    if args.len() > 2 {
        let mut error = false;
        start = arg_number_chk(&args[2], Some(&mut error)) as c_int;
        if error {
            return;
        }
    }
    let Some(b) = args[0].blob_ref() else {
        return;
    };
    let len = blob_len(Some(b));
    if start < 0 {
        start = (len + start).max(0);
    }
    for idx in start..len {
        let mut tv = NIL;
        tv.write_number(VarNumber::from(b.byte(idx)));
        // The Blob branch never reads argument 3, so a Blob search is
        // always case-sensitive however 'ic' was spelled. Upstream is
        // the same; the flag only reaches the List branch.
        if tv_equal(&tv, &args[1], false) {
            result.write_number(idx as VarNumber);
            return;
        }
    }
}

/// `index()` over a List. The caller has checked the tag.
fn index_list(args: &[TypVal], result: &mut TypVal) {
    // SAFETY: the caller's obligation.
    let l = args[0].list_or_null();
    if l.is_null() {
        return;
    }
    let mut idx: c_int = 0;
    let mut start = Some(0usize);
    let mut ic = false;
    if args.len() > 2 {
        let mut error = false;
        idx = list_uidx(
            unsafe { l.as_ref() },
            arg_number_chk(&args[2], Some(&mut error)) as c_int,
        );
        start = if error {
            None
        } else {
            usize::try_from(idx).ok()
        };
        if args.len() > 3 {
            ic = arg_number_chk(&args[3], Some(&mut error)) != 0;
            if error {
                start = None;
            }
        }
    }
    let Some(start) = start else { return };
    // By index: `tv_equal` compares two values and can re-enter.
    let mut at = start;
    // SAFETY: the caller's obligation: a live list.
    while at < list_items(unsafe { l.as_ref() }).len() {
        let item = &list_items(unsafe { l.as_ref() })[at];
        if tv_equal(&item.li_tv, &args[1], ic) {
            result.write_number(VarNumber::from(idx));
            return;
        }
        at += 1;
        idx += 1;
    }
}

/// `indexof({object}, {expr} [, {opts}])` — the first index whose value
/// satisfies `expr`, which sees the item as `v:key` and `v:val`.
pub fn f_indexof(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    // SAFETY throughout: the arguments are live typvals; the two `v:` variables are
    // saved and put back around the search whatever it does.
    if tv_check_for_list_or_blob_arg(args, 0).is_err()
        || tv_check_for_string_or_func_arg(args, 1).is_err()
        || tv_check_for_opt_dict_arg(args, 2).is_err()
    {
        return;
    }
    // An empty expression matches nothing rather than everything.
    let expr = &args[1];
    let vacuous = match expr.v_type() {
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
    let startidx = if args.get(2).is_some_and(|arg| arg.v_type() == VAR_DICT) {
        dict_get_number_def(args[2].dict_ref(), b"startidx", 0)
    } else {
        0
    };

    let (mut save_val, mut save_key) = (NIL, NIL);
    prepare_vimvar(Vv::Val, &mut save_val);
    prepare_vimvar(Vv::Key, &mut save_key);
    let saved_did_emsg = did_emsg.get();
    did_emsg.set(0);
    result.write_number(if args[0].v_type() == VAR_BLOB {
        indexof_blob(args[0].blob_ref(), startidx, &args[1])
    } else {
        unsafe { indexof_list(args[0].list_or_null(), startidx, &args[1]) }
    });
    restore_vimvar(Vv::Key, &mut save_key);
    restore_vimvar(Vv::Val, &mut save_val);
    // As `printf()`: an error raised before this call survives, one
    // raised inside it does not.
    did_emsg.set(did_emsg.get() | saved_did_emsg);
}

/// Evaluate `indexof()`'s predicate against the `v:key`/`v:val` already in
/// place. A failed evaluation, and a result that is not coercible to a
/// Bool, both read as "no match".
fn indexof_matches(expr: &TypVal) -> bool {
    // SAFETY throughout: the caller's obligation; `argv` and `newtv` are locals that
    // outlive the evaluation, and `newtv` is cleared before returning.
    // A frame naming the two `v:` slots for the length of the call.
    let mut argv = CallFrame::<2>::new();
    argv.push_borrowed(unsafe { &*get_vim_var_tv(Vv::Key) });
    argv.push_borrowed(unsafe { &*get_vim_var_tv(Vv::Val) });
    let mut newtv = NIL;
    if eval_expr_typval(expr, false, argv.args(), &mut newtv).is_err() {
        return false;
    }
    let found = tv_get_bool_chk(&newtv);
    tv_clear(&mut newtv);
    found.is_ok_and(|n| n != 0)
}

/// Walk a Blob's bytes, answering the index of the first `expr` accepts.
fn indexof_blob(b: Option<&Blob>, startidx: VarNumber, expr: &TypVal) -> VarNumber {
    let Some(b) = b else {
        return -1;
    };
    let len = VarNumber::from(blob_len(Some(b)));
    let start = if startidx < 0 {
        (len + startidx).max(0)
    } else {
        startidx
    };
    set_vim_var_type(Vv::Key, VAR_NUMBER);
    set_vim_var_type(Vv::Val, VAR_NUMBER);
    let called_emsg_start = called_emsg.get();
    for idx in start..len {
        set_vim_var_nr(Vv::Key, idx);
        let at = c_int::try_from(idx).expect("a byte of the blob");
        set_vim_var_nr(Vv::Val, VarNumber::from(b.byte(at)));
        if indexof_matches(expr) {
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
unsafe fn indexof_list(l: *mut List, startidx: VarNumber, expr: &TypVal) -> VarNumber {
    if l.is_null() {
        return -1;
    }
    let mut idx: VarNumber = 0;
    // A zero start index is taken literally rather than run through
    // `list_uidx`, so it does not have to be a valid index.
    let start = if startidx == 0 {
        Some(0usize)
    } else {
        idx = VarNumber::from(list_uidx(unsafe { l.as_ref() }, startidx as c_int));
        usize::try_from(idx).ok()
    };
    let Some(start) = start else { return -1 };
    set_vim_var_type(Vv::Key, VAR_NUMBER);
    let called_emsg_start = called_emsg.get();
    // By index: `expr` is the user's, and may edit the list it is testing.
    let mut at = start;
    // SAFETY: the caller's obligation: a live list.
    while at < list_items(unsafe { l.as_ref() }).len() {
        set_vim_var_nr(Vv::Key, idx);
        let item = &list_items(unsafe { l.as_ref() })[at];
        unsafe { tv_copy(&item.li_tv, &mut *get_vim_var_tv(Vv::Val)) };
        let found = indexof_matches(expr);
        unsafe { tv_clear(&mut *get_vim_var_tv(Vv::Val)) };
        if found {
            return idx;
        }
        if called_emsg.get() != called_emsg_start {
            return -1;
        }
        at += 1;
        idx += 1;
    }
    -1
}

/// `len({expr})` — bytes for a String or Number, items otherwise.
pub fn f_len(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let tv = &args[0];
    // SAFETY throughout: every union read is guarded by the type tag above it, and a
    // Number is measured through its String spelling.
    result.write_number(match tv.v_type() {
        VAR_STRING | VAR_NUMBER => {
            let s = arg_string(&mut numbuf, &args[0]);
            unsafe { cstr::bytes_at(s).len() as VarNumber }
        }
        VAR_BLOB => VarNumber::from(blob_len(tv.blob_ref())),
        VAR_LIST => list_len(tv.list_ref()) as VarNumber,
        VAR_DICT => dict_len(tv.dict_ref()) as VarNumber,
        // The remaining tags are Unknown, Funcref, Partial, Float,
        // Bool and Special; `VarType` has no twelfth value.
        _ => {
            emsg(gettext(c"E701: Invalid type for len()"));
            return;
        }
    });
}

/// `type({expr})` — the `v:t_*` number for the value's type.
pub fn f_type(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let n: c_int = match args[0].v_type() {
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
    result.write_number(n as VarNumber);
}
