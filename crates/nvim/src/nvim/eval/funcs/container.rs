//! Reading a List, Dict or Blob: `get()`, `empty()`, `index()`,
//! `flatten()` and friends.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::{FAIL, NUL, TV_TRANSLATE};
use crate::src::nvim::eval::typval::{
    tv_blob_get, tv_blob_len, tv_check_for_list_or_blob_arg, tv_check_for_opt_bool_arg,
    tv_check_for_opt_dict_arg, tv_check_for_string_or_func_arg, tv_clear, tv_copy,
    tv_dict_add_bool, tv_dict_add_nr, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_number_def,
    tv_dict_len, tv_dict_set_ret, tv_equal, tv_get_bool_chk, tv_get_number_chk, tv_get_string,
    tv_is_func, tv_list_alloc_ret, tv_list_append_tv, tv_list_copy, tv_list_find, tv_list_first,
    tv_list_flatten, tv_list_len, tv_list_locked, tv_list_ref, tv_list_uidx, value_check_lock,
};
use crate::src::nvim::eval::userfunc::{func_ref, get_func_arity, printable_func_name};
use crate::src::nvim::eval::vars::{
    get_vim_var_tv, prepare_vimvar, restore_vimvar, set_vim_var_nr, set_vim_var_type,
};
use crate::src::nvim::eval::{eval_expr_typval, get_copyID, partial_name, var_item_copy};
use crate::src::nvim::main::{
    called_emsg, did_emsg, e_invarg2, e_listarg, e_listblobreq, e_listdictblobarg,
};
use crate::src::nvim::memory::xstrdup;
use crate::src::nvim::message::{emsg, internal_error, semsg};
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::types::{
    BoolVarValue, EvalFuncData, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST,
    VAR_NUMBER, VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_TYPE_BLOB, VAR_TYPE_BOOL, VAR_TYPE_DICT,
    VAR_TYPE_FLOAT, VAR_TYPE_FUNC, VAR_TYPE_LIST, VAR_TYPE_NUMBER, VAR_TYPE_SPECIAL,
    VAR_TYPE_STRING, VAR_UNKNOWN, VAR_UNLOCKED, VV_KEY, VV_VAL, blob_T, kBoolVarTrue,
    kSpecialVarNull, list_T, listitem_T, partial_T, typval_T, typval_vval_union, varnumber_T,
};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// A cleared typval, the shape both dispatchers start every slot from.
const NIL: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// `copy({expr})` — one level deep.
pub unsafe extern "C" fn f_copy(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: `args.ptr(0)` and `rettv` are live typvals.
    unsafe { var_item_copy(ptr::null(), args.ptr(0), rettv, false, 0) };
}

/// `deepcopy({expr} [, {noref}])`.
///
/// Without `noref` the copy is given a copy id, which is what lets it
/// reproduce a self-referential structure rather than recursing forever.
pub unsafe extern "C" fn f_deepcopy(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe {
        if tv_check_for_opt_bool_arg(args.ptr(0), 1) == FAIL {
            return;
        }
        let noref = args.has(1) && tv_get_bool_chk(args.ptr(1), ptr::null_mut()) != 0;
        let copy_id = if noref { 0 } else { get_copyID() };
        var_item_copy(ptr::null(), args.ptr(0), rettv, true, copy_id);
    }
}

/// `empty({expr})` — what "empty" means for each type.
pub unsafe extern "C" fn f_empty(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let tv = args.get(0);
    // SAFETY: every read below is guarded by the type tag that says which
    // union member is live. A String, List, Dict or Blob pointer may still
    // be null, which each reader treats as empty.
    let empty = unsafe {
        match tv.v_type {
            VAR_STRING | VAR_FUNC => {
                tv.vval.v_string.is_null() || *tv.vval.v_string == NUL as c_char
            }
            VAR_PARTIAL => false,
            VAR_NUMBER => tv.vval.v_number == 0,
            VAR_FLOAT => tv.vval.v_float == 0.0,
            VAR_LIST => tv_list_len(tv.vval.v_list) == 0,
            VAR_DICT => tv_dict_len(tv.vval.v_dict) == 0,
            VAR_BLOB => tv_blob_len(tv.vval.v_blob) == 0,
            VAR_SPECIAL => tv.vval.v_special == kSpecialVarNull,
            // A Bool other than the two named values leaves the answer at
            // its "empty" default, as upstream's switch does.
            VAR_BOOL => tv.vval.v_bool != kBoolVarTrue,
            VAR_UNKNOWN => {
                internal_error(c"f_empty(UNKNOWN)".as_ptr());
                true
            }
            _ => true,
        }
    };
    rettv.vval.v_number = empty as varnumber_T;
}

/// `flatten({list} [, {maxdepth}])` — in place.
pub unsafe extern "C" fn f_flatten(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { flatten_common(args, rettv, false) };
}

/// `flattennew({list} [, {maxdepth}])` — into a copy.
pub unsafe extern "C" fn f_flattennew(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { flatten_common(args, rettv, true) };
}

/// The shared body. `make_copy` is what separates `flattennew()` from
/// `flatten()`: the copying form never checks the source for a lock,
/// because it does not write to it.
///
/// # Safety
/// The arguments and `rettv` are live typvals.
unsafe fn flatten_common(args: Args<'_>, rettv: &mut typval_T, make_copy: bool) {
    // SAFETY: the caller's obligation.
    unsafe {
        if args.ty(0) != VAR_LIST {
            semsg(
                gettext(e_listarg.ptr() as *const c_char),
                c"flatten()".as_ptr(),
            );
            return;
        }
        let maxdepth = if !args.has(1) {
            999_999
        } else {
            let mut error = false;
            let depth = tv_get_number_chk(args.ptr(1), &raw mut error) as c_int;
            if error {
                return;
            }
            if depth < 0 {
                emsg(gettext(
                    c"E900: maxdepth must be non-negative number".as_ptr(),
                ));
                return;
            }
            depth
        };

        let mut list = args.get(0).vval.v_list;
        rettv.v_type = VAR_LIST;
        rettv.vval.v_list = list;
        if list.is_null() {
            return;
        }
        if make_copy {
            list = tv_list_copy(ptr::null(), list, false, get_copyID());
            rettv.vval.v_list = list;
            if list.is_null() {
                return;
            }
        } else {
            if value_check_lock(
                tv_list_locked(list),
                c"flatten() argument".as_ptr(),
                TV_TRANSLATE as usize,
            ) {
                return;
            }
            tv_list_ref(list);
        }
        tv_list_flatten(
            list,
            ptr::null_mut(),
            tv_list_len(list) as i64,
            maxdepth as i64,
        );
    }
}

/// `get({container}, {key} [, {default}])` — for a Blob, List, Dict,
/// Funcref or Partial.
pub unsafe extern "C" fn f_get(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals; each union read
    // is guarded by the type tag above it.
    unsafe {
        let found: *mut typval_T = match args.ty(0) {
            VAR_BLOB => get_from_blob(args, rettv),
            VAR_LIST => get_from_list(args),
            VAR_DICT => get_from_dict(args),
            _ if tv_is_func(*args.get(0)) => {
                if !get_from_func(args, rettv) {
                    return;
                }
                // Only the "dict" selector falls through to the default
                // handling below, and only when the Partial had no dict.
                ptr::null_mut()
            }
            _ => {
                semsg(
                    gettext(e_listdictblobarg.ptr() as *const c_char),
                    c"get()".as_ptr(),
                );
                ptr::null_mut()
            }
        };
        if !found.is_null() {
            tv_copy(found, rettv);
        } else if args.has(2) {
            tv_copy(args.ptr(2), rettv);
        }
    }
}

/// # Safety
/// Argument 0 is a live Blob typval.
unsafe fn get_from_blob(args: Args<'_>, rettv: &mut typval_T) -> *mut typval_T {
    // SAFETY: the caller's obligation.
    unsafe {
        let mut error = false;
        let mut idx = tv_get_number_chk(args.ptr(1), &raw mut error) as c_int;
        if error {
            return ptr::null_mut();
        }
        let blob = args.get(0).vval.v_blob;
        rettv.v_type = VAR_NUMBER;
        if idx < 0 {
            idx += tv_blob_len(blob);
        }
        if idx < 0 || idx >= tv_blob_len(blob) {
            // Out of range is -1 rather than the default argument.
            rettv.vval.v_number = -1;
            return ptr::null_mut();
        }
        rettv.vval.v_number = tv_blob_get(blob, idx) as varnumber_T;
        // The value is already in place; copying it onto itself is a no-op
        // and is what upstream does.
        rettv
    }
}

/// # Safety
/// Argument 0 is a live List typval.
unsafe fn get_from_list(args: Args<'_>) -> *mut typval_T {
    // SAFETY: the caller's obligation.
    unsafe {
        let l = args.get(0).vval.v_list;
        if l.is_null() {
            return ptr::null_mut();
        }
        let mut error = false;
        let idx = tv_get_number_chk(args.ptr(1), &raw mut error) as c_int;
        let li = tv_list_find(l, idx);
        if error || li.is_null() {
            return ptr::null_mut();
        }
        &raw mut (*li).li_tv
    }
}

/// # Safety
/// Argument 0 is a live Dict typval.
unsafe fn get_from_dict(args: Args<'_>) -> *mut typval_T {
    // SAFETY: the caller's obligation.
    unsafe {
        let d = args.get(0).vval.v_dict;
        if d.is_null() {
            return ptr::null_mut();
        }
        let di = tv_dict_find(d, tv_get_string(args.ptr(1)), -1);
        if di.is_null() {
            return ptr::null_mut();
        }
        &raw mut (*di).di_tv
    }
}

/// Answer `get()` for a Funcref or Partial. Returns whether the caller
/// should fall through to the default-argument handling, which only the
/// "dict" selector does — and then only when the Partial had no dict.
///
/// # Safety
/// Argument 0 is a live Funcref or Partial typval.
unsafe fn get_from_func(args: Args<'_>, rettv: &mut typval_T) -> bool {
    // SAFETY: the caller's obligation. A plain Funcref is answered through
    // a stack Partial holding just its name, which lives as long as this
    // call and is never stored.
    unsafe {
        let mut fref = partial_T {
            pt_refcount: 0,
            pt_copyID: 0,
            pt_name: ptr::null_mut(),
            pt_func: ptr::null_mut(),
            pt_auto: false,
            pt_argc: 0,
            pt_argv: ptr::null_mut(),
            pt_dict: ptr::null_mut(),
        };
        let pt = if args.ty(0) == VAR_PARTIAL {
            args.get(0).vval.v_partial
        } else {
            fref.pt_name = args.get(0).vval.v_string;
            &raw mut fref
        };
        let what = tv_get_string(args.ptr(1));
        match CStr::from_ptr(what).to_bytes() {
            b"func" | b"name" => {
                let mut name: *const c_char = partial_name(pt);
                // "func" hands back a Funcref, "name" a plain String.
                rettv.v_type = if *what == b'f' as c_char {
                    VAR_FUNC
                } else {
                    VAR_STRING
                };
                debug_assert!(!name.is_null());
                if rettv.v_type == VAR_FUNC {
                    func_ref(name as *mut c_char);
                }
                // A lambda has no name of its own; "name" shows the
                // printable form instead.
                if *what == b'n' as c_char && (*pt).pt_name.is_null() && !(*pt).pt_func.is_null() {
                    name = printable_func_name((*pt).pt_func);
                }
                rettv.vval.v_string = xstrdup(name);
            }
            b"dict" => {
                if !(*pt).pt_dict.is_null() {
                    tv_dict_set_ret(rettv, (*pt).pt_dict);
                }
                // "dict" is the only selector that falls through to the
                // default-argument handling, and it does so whether or not
                // it just found a dict — so a default given alongside it
                // wins. Upstream is the same.
                return true;
            }
            b"args" => {
                rettv.v_type = VAR_LIST;
                let list = tv_list_alloc_ret(rettv, (*pt).pt_argc as isize);
                for i in 0..(*pt).pt_argc {
                    tv_list_append_tv(list, (*pt).pt_argv.offset(i as isize));
                }
            }
            b"arity" => func_arity(pt, rettv),
            _ => {
                // Kept on the variadic message call: `what` is arbitrary
                // user bytes and a Rust format string can only carry UTF-8.
                semsg(gettext(e_invarg2.ptr() as *const c_char), what);
            }
        }
        false
    }
}

/// `get(Funcref, "arity")` — what the function still wants after the
/// Partial's bound arguments are subtracted.
///
/// # Safety
/// `pt` is a live Partial and `rettv` is the cleared return value.
unsafe fn func_arity(pt: *mut partial_T, rettv: &mut typval_T) {
    // SAFETY: the caller's obligation.
    unsafe {
        let (mut required, mut optional, mut varargs) = (0, 0, false);
        get_func_arity(
            partial_name(pt),
            &raw mut required,
            &raw mut optional,
            &raw mut varargs,
        );
        rettv.v_type = VAR_DICT;
        tv_dict_alloc_ret(rettv);
        let dict = rettv.vval.v_dict;
        // The bound arguments cover the required ones first.
        if (*pt).pt_argc >= required + optional {
            optional = 0;
            required = 0;
        } else if (*pt).pt_argc > required {
            optional -= (*pt).pt_argc - required;
            required = 0;
        } else {
            required -= (*pt).pt_argc;
        }
        tv_dict_add_nr(dict, c"required".as_ptr(), 8, required as varnumber_T);
        tv_dict_add_nr(dict, c"optional".as_ptr(), 8, optional as varnumber_T);
        tv_dict_add_bool(dict, c"varargs".as_ptr(), 7, varargs as BoolVarValue);
    }
}

/// `index({object}, {expr} [, {start} [, {ic}]])`.
pub unsafe extern "C" fn f_index(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    // SAFETY: the arguments are live typvals.
    unsafe {
        match args.ty(0) {
            VAR_BLOB => index_blob(args, rettv),
            VAR_LIST => index_list(args, rettv),
            _ => {
                emsg(gettext(e_listblobreq.ptr() as *const c_char));
            }
        }
    }
}

/// # Safety
/// Argument 0 is a live Blob typval.
unsafe fn index_blob(args: Args<'_>, rettv: &mut typval_T) {
    // SAFETY: the caller's obligation.
    unsafe {
        let mut start: c_int = 0;
        if args.has(2) {
            let mut error = false;
            start = tv_get_number_chk(args.ptr(2), &raw mut error) as c_int;
            if error {
                return;
            }
        }
        let b = args.get(0).vval.v_blob;
        if b.is_null() {
            return;
        }
        if start < 0 {
            start = (tv_blob_len(b) + start).max(0);
        }
        for idx in start..tv_blob_len(b) {
            let mut tv = NIL;
            tv.v_type = VAR_NUMBER;
            tv.vval.v_number = tv_blob_get(b, idx) as varnumber_T;
            // The Blob branch never reads argument 3, so a Blob search is
            // always case-sensitive however 'ic' was spelled. Upstream is
            // the same; the flag only reaches the List branch.
            if tv_equal(&raw mut tv, args.ptr(1), false) {
                rettv.vval.v_number = idx as varnumber_T;
                return;
            }
        }
    }
}

/// # Safety
/// Argument 0 is a live List typval.
unsafe fn index_list(args: Args<'_>, rettv: &mut typval_T) {
    // SAFETY: the caller's obligation.
    unsafe {
        let l = args.get(0).vval.v_list;
        if l.is_null() {
            return;
        }
        let mut idx: c_int = 0;
        let mut item = tv_list_first(l);
        let mut ic = false;
        if args.has(2) {
            let mut error = false;
            idx = tv_list_uidx(l, tv_get_number_chk(args.ptr(2), &raw mut error) as c_int);
            if error || idx == -1 {
                item = ptr::null_mut();
            } else {
                item = tv_list_find(l, idx);
                debug_assert!(!item.is_null());
            }
            if args.has(3) {
                ic = tv_get_number_chk(args.ptr(3), &raw mut error) != 0;
                if error {
                    item = ptr::null_mut();
                }
            }
        }
        while !item.is_null() {
            if tv_equal(&raw mut (*item).li_tv, args.ptr(1), ic) {
                rettv.vval.v_number = idx as varnumber_T;
                return;
            }
            item = (*item).li_next;
            idx += 1;
        }
    }
}

/// `indexof({object}, {expr} [, {opts}])` — the first index whose value
/// satisfies `expr`, which sees the item as `v:key` and `v:val`.
pub unsafe extern "C" fn f_indexof(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    // SAFETY: the arguments are live typvals; the two `v:` variables are
    // saved and put back around the search whatever it does.
    unsafe {
        if tv_check_for_list_or_blob_arg(args.ptr(0), 0) == FAIL
            || tv_check_for_string_or_func_arg(args.ptr(0), 1) == FAIL
            || tv_check_for_opt_dict_arg(args.ptr(0), 2) == FAIL
        {
            return;
        }
        // An empty expression matches nothing rather than everything.
        let expr = args.get(1);
        let vacuous = match expr.v_type {
            VAR_STRING => expr.vval.v_string.is_null() || *expr.vval.v_string == NUL as c_char,
            VAR_FUNC => expr.vval.v_partial.is_null(),
            _ => false,
        };
        if vacuous {
            return;
        }
        let startidx = if args.ty(2) == VAR_DICT {
            tv_dict_get_number_def(args.get(2).vval.v_dict, c"startidx".as_ptr(), 0)
        } else {
            0
        };

        let (mut save_val, mut save_key) = (NIL, NIL);
        prepare_vimvar(VV_VAL as c_int, &raw mut save_val);
        prepare_vimvar(VV_KEY as c_int, &raw mut save_key);
        let saved_did_emsg = did_emsg.get();
        did_emsg.set(0);
        rettv.vval.v_number = if args.ty(0) == VAR_BLOB {
            indexof_blob(args.get(0).vval.v_blob, startidx, args.ptr(1))
        } else {
            indexof_list(args.get(0).vval.v_list, startidx, args.ptr(1))
        };
        restore_vimvar(VV_KEY as c_int, &raw mut save_key);
        restore_vimvar(VV_VAL as c_int, &raw mut save_val);
        // As `printf()`: an error raised before this call survives, one
        // raised inside it does not.
        did_emsg.set(did_emsg.get() | saved_did_emsg);
    }
}

/// Evaluate `indexof()`'s predicate against the `v:key`/`v:val` already in
/// place. A failed evaluation, and a result that is not coercible to a
/// Bool, both read as "no match".
///
/// # Safety
/// `expr` is a live String or Funcref typval.
unsafe fn indexof_matches(expr: *mut typval_T) -> bool {
    // SAFETY: the caller's obligation; `argv` and `newtv` are locals that
    // outlive the evaluation, and `newtv` is cleared before returning.
    unsafe {
        let mut argv = [*get_vim_var_tv(VV_KEY), *get_vim_var_tv(VV_VAL), NIL];
        let mut newtv = NIL;
        if eval_expr_typval(expr, false, argv.as_mut_ptr(), 2, &raw mut newtv) == FAIL {
            return false;
        }
        let mut error = false;
        let found = tv_get_bool_chk(&raw mut newtv, &raw mut error);
        tv_clear(&raw mut newtv);
        !error && found != 0
    }
}

/// # Safety
/// `b` is a Blob pointer or null and `expr` is a live predicate typval.
unsafe fn indexof_blob(b: *mut blob_T, startidx: varnumber_T, expr: *mut typval_T) -> varnumber_T {
    if b.is_null() {
        return -1;
    }
    // SAFETY: the caller's obligation.
    unsafe {
        let start = if startidx < 0 {
            (tv_blob_len(b) as varnumber_T + startidx).max(0)
        } else {
            startidx
        };
        set_vim_var_type(VV_KEY, VAR_NUMBER);
        set_vim_var_type(VV_VAL, VAR_NUMBER);
        let called_emsg_start = called_emsg.get();
        for idx in start..tv_blob_len(b) as varnumber_T {
            set_vim_var_nr(VV_KEY, idx);
            set_vim_var_nr(VV_VAL, tv_blob_get(b, idx as c_int) as varnumber_T);
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
}

/// # Safety
/// `l` is a List pointer or null and `expr` is a live predicate typval.
unsafe fn indexof_list(l: *mut list_T, startidx: varnumber_T, expr: *mut typval_T) -> varnumber_T {
    if l.is_null() {
        return -1;
    }
    // SAFETY: the caller's obligation.
    unsafe {
        let mut idx: varnumber_T = 0;
        let mut item: *mut listitem_T;
        // A zero start index is taken literally rather than run through
        // `tv_list_uidx`, so it does not have to be a valid index.
        if startidx == 0 {
            item = tv_list_first(l);
        } else {
            idx = tv_list_uidx(l, startidx as c_int) as varnumber_T;
            if idx == -1 {
                item = ptr::null_mut();
            } else {
                item = tv_list_find(l, idx as c_int);
                debug_assert!(!item.is_null());
            }
        }
        set_vim_var_type(VV_KEY, VAR_NUMBER);
        let called_emsg_start = called_emsg.get();
        while !item.is_null() {
            set_vim_var_nr(VV_KEY, idx);
            tv_copy(&raw mut (*item).li_tv, get_vim_var_tv(VV_VAL));
            let found = indexof_matches(expr);
            tv_clear(get_vim_var_tv(VV_VAL));
            if found {
                return idx;
            }
            if called_emsg.get() != called_emsg_start {
                return -1;
            }
            item = (*item).li_next;
            idx += 1;
        }
        -1
    }
}

/// `len({expr})` — bytes for a String or Number, items otherwise.
pub unsafe extern "C" fn f_len(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    let tv = args.get(0);
    // SAFETY: every union read is guarded by the type tag above it, and a
    // Number is measured through its String spelling.
    unsafe {
        rettv.vval.v_number = match tv.v_type {
            VAR_STRING | VAR_NUMBER => strlen(tv_get_string(args.ptr(0))) as varnumber_T,
            VAR_BLOB => tv_blob_len(tv.vval.v_blob) as varnumber_T,
            VAR_LIST => tv_list_len(tv.vval.v_list) as varnumber_T,
            VAR_DICT => tv_dict_len(tv.vval.v_dict) as varnumber_T,
            // The remaining tags are Unknown, Funcref, Partial, Float,
            // Bool and Special; `VarType` has no twelfth value.
            _ => {
                emsg(gettext(c"E701: Invalid type for len()".as_ptr()));
                return;
            }
        };
    }
}

/// `type({expr})` — the `v:t_*` number for the value's type.
pub unsafe extern "C" fn f_type(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
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
    rettv.vval.v_number = n as varnumber_T;
}
