//! `[]`, `[:]` and `.` applied to a value the evaluator already has.
//!
//! Two indexing vocabularies live here and they are not the same. The
//! subscript the *grammar* produces (`s[1]`, `s[1:2]`) counts **bytes** in a
//! String and includes its end; `slice()` counts **characters** and excludes
//! its end. `exclusive` is the flag that tells them apart, and it also
//! switches the String arm onto the character walkers.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::semsg;
use crate::winlayer::Live;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{null, null_mut};

use crate::ascii::ascii_iswhite;
use crate::eval::typval::{
    NumBuf, tv_blob_slice_or_index, tv_check_str, tv_clear, tv_copy, tv_dict_find, tv_dict_unref,
    tv_get_number, tv_is_func, tv_list_slice_or_index,
};
use crate::eval::userfunc::make_partial;
use crate::eval::{
    Cur, EVAL_EVALUATE, Tv, VARNUMBER_MAX, call_func_rettv, check_luafunc_name,
    e_cannot_index_a_funcref, e_cannot_index_special_variable, e_cannot_slice_dictionary,
    e_missbrac, eval_isdictc, eval_lambda, eval_method, eval1, tv_is_luafunc,
};
use crate::ex_eval::aborting;
use crate::main::e_using_float_as_string;
use crate::mbyte::{utf_head_off, utfc_ptr2len};
use crate::memory::xmemdupz;
use crate::message::emsg;
use crate::message_fmt::{c_str, c_str_len};
use crate::os::cshim::{gettext, gettext_ptr};
use crate::types::{
    EvalFuncData, Failed, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER,
    VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VarLock, dict_T, dictitem_T, evalarg_T,
    ptrdiff_t, size_t, ssize_t, typval_T, typval_vval_union, varnumber_T,
};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// Is this `evalarg` asking for the expression to actually be evaluated?
///
/// # Safety
/// `evalarg` must be null or valid.
unsafe fn evaluating(evalarg: *const evalarg_T) -> bool {
    !evalarg.is_null() && unsafe { (*evalarg).eval_flags } & EVAL_EVALUATE as c_int != 0
}

/// `expr[idx]`, `expr[first : last]` or `dict.key`, with the cursor on the
/// `[` or the `.`. Leaves the cursor after the `]` or the key.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression, `rettv`
/// at the value being subscripted, and `evalarg` must be null or valid.
pub(crate) unsafe fn eval_index(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression, `rettv` is the value being subscripted and `evalarg` is
    // null or valid. All three hold for every call below.
    let cur = unsafe { Cur::new(arg) };
    let evaluate = unsafe { evaluating(evalarg) };
    let mut empty1 = false;
    let mut empty2 = false;
    let mut range = false;
    let mut key: *const c_char = null();
    let mut keylen: ptrdiff_t = -1;

    unsafe { check_can_index(rettv, evaluate, verbose) }?;

    let mut var1 = UNSET_TV;
    let mut var2 = UNSET_TV;
    if cur.byte() == b'.' {
        // dict.name
        key = cur.get().wrapping_add(1);
        keylen = 0;
        // SAFETY: the key runs to the first byte that cannot be in one,
        // which the terminating NUL is not.
        while eval_isdictc(unsafe { *key.offset(keylen) } as c_int) {
            keylen += 1;
        }
        if keylen == 0 {
            return Err(Failed);
        }
        cur.skip(1 + keylen as usize);
    } else {
        // The first index, from inside the brackets.
        cur.skip(1);
        if cur.byte() == b':' {
            empty1 = true;
        } else if unsafe { eval1(arg, &raw mut var1, evalarg) }.is_err() {
            return Err(Failed);
        } else if evaluate && !unsafe { tv_check_str(&raw mut var1) } {
            unsafe { tv_clear(&raw mut var1) };
            return Err(Failed);
        }

        // The second index, from inside the `[ : ]`.
        if cur.byte() == b':' {
            range = true;
            cur.skip(1);
            if cur.byte() == b']' {
                empty2 = true;
            } else if unsafe { eval1(arg, &raw mut var2, evalarg) }.is_err() {
                if !empty1 {
                    unsafe { tv_clear(&raw mut var1) };
                }
                return Err(Failed);
            } else if evaluate && !unsafe { tv_check_str(&raw mut var2) } {
                if !empty1 {
                    unsafe { tv_clear(&raw mut var1) };
                }
                unsafe { tv_clear(&raw mut var2) };
                return Err(Failed);
            }
        }

        if cur.byte() != b']' {
            if verbose {
                emsg(gettext(e_missbrac));
            }
            // Not guarded by `empty1`: an unread `var1` is still unset.
            unsafe { tv_clear(&raw mut var1) };
            if range {
                unsafe { tv_clear(&raw mut var2) };
            }
            return Err(Failed);
        }
        cur.skip(1);
    }

    if !evaluate {
        return Ok(());
    }
    let one = if empty1 { null_mut() } else { &raw mut var1 };
    let two = if empty2 { null_mut() } else { &raw mut var2 };
    let res = unsafe { eval_index_inner(rettv, range, one, two, false, key, keylen, verbose) };
    if !empty1 {
        unsafe { tv_clear(&raw mut var1) };
    }
    if range {
        unsafe { tv_clear(&raw mut var2) };
    }
    res
}

/// Can `rettv` carry an `[index]` or a `[sli:ce]` at all?
///
/// # Safety
/// `rettv` must be valid.
pub(crate) unsafe fn check_can_index(
    rettv: *mut typval_T,
    evaluate: bool,
    verbose: bool,
) -> Result<(), Failed> {
    let message = match unsafe { (*rettv).v_type } {
        VAR_FUNC | VAR_PARTIAL => e_cannot_index_a_funcref.as_ptr(),
        VAR_FLOAT => e_using_float_as_string.as_ptr(),
        VAR_BOOL | VAR_SPECIAL => e_cannot_index_special_variable.as_ptr(),
        // Not evaluating: the subscript is only being skipped over, and an
        // unset value is what an unevaluated operand looks like.
        VAR_UNKNOWN if !evaluate => return Ok(()),
        // Reported whether or not the caller asked to be verbose.
        VAR_UNKNOWN => {
            emsg(gettext(e_cannot_index_special_variable));
            return Err(Failed);
        }
        _ => return Ok(()),
    };
    if verbose {
        // SAFETY: as above.
        unsafe { emsg(gettext_ptr(message)) };
    }
    Err(Failed)
}

/// `slice()`
///
/// # Safety
/// Called through the builtin table with a terminated argument array.
pub(crate) unsafe fn f_slice(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    if unsafe { check_can_index(argvars, true, false) }.is_err() {
        return;
    }
    unsafe { tv_copy(argvars, rettv) };
    // SAFETY: the builtin table hands in three argument slots, terminated by
    // a `VAR_UNKNOWN` when the third was not given.
    let (first, last) = unsafe { (argvars.add(1), argvars.add(2)) };
    let end = if unsafe { (*last).v_type } == VAR_UNKNOWN {
        null_mut()
    } else {
        last
    };
    let _ = unsafe { eval_index_inner(rettv, true, first, end, true, null(), 0, false) };
}

/// Apply an index or a range to `rettv`, in place.
///
/// `var1` is the first index and is null for `[:expr]`; `var2` is the second
/// and is null for `[expr]` and `[expr:]`. `exclusive` is `slice()`'s: the
/// second index is excluded and a String is indexed by character. When `key`
/// is non-null it is the Dict index instead of `var1`.
///
/// # Safety
/// `rettv` must be valid; `var1`/`var2` null or valid; `key` null or
/// `keylen` readable bytes (or NUL-terminated when `keylen` is negative).
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn eval_index_inner(
    rettv: *mut typval_T,
    is_range: bool,
    var1: *mut typval_T,
    var2: *mut typval_T,
    exclusive: bool,
    key: *const c_char,
    keylen: ptrdiff_t,
    verbose: bool,
) -> Result<(), Failed> {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut n1: varnumber_T = 0;
    let mut n2: varnumber_T = 0;
    // SAFETY: the caller's promise -- `rettv` is the value being indexed,
    // and `var1`/`var2` are null or valid typvals.
    let mut rv = unsafe { Tv::new(rettv) };
    if !var1.is_null() && rv.v_type != VAR_DICT {
        n1 = unsafe { tv_get_number(var1) };
    }
    if is_range {
        if rv.v_type == VAR_DICT {
            if verbose {
                emsg(gettext(e_cannot_slice_dictionary));
            }
            return Err(Failed);
        }
        n2 = if var2.is_null() {
            VARNUMBER_MAX
        } else {
            unsafe { tv_get_number(var2) }
        };
    }

    match rv.v_type {
        VAR_NUMBER | VAR_STRING => {
            // SAFETY: `numbuf` is this frame's own scratch, and the String
            // it answers is NUL-terminated with `n1`/`n2` inside it.
            let s = unsafe { numbuf.string(rettv) };
            let len = unsafe { cstr::bytes_at(s) }.len() as c_int as varnumber_T;
            let v = if exclusive {
                // slice(): character indexes, second one excluded.
                if is_range {
                    unsafe { string_slice(s, n1, n2, exclusive) }
                } else {
                    unsafe { char_from_string(s, n1) }
                }
            } else if is_range {
                // A substring. Out-of-range indexes give an empty result.
                if n1 < 0 {
                    n1 = (len + n1).max(0);
                }
                if n2 < 0 {
                    n2 += len;
                } else if n2 >= len {
                    n2 = len;
                }
                if n1 >= len || n2 < 0 || n1 > n2 {
                    null_mut()
                } else {
                    let at = s.wrapping_offset(n1 as isize).cast();
                    unsafe { xmemdupz(at, (n2 - n1 + 1) as size_t) as *mut c_char }
                }
            } else if n1 >= len || n1 < 0 {
                // A one-byte String; too big or negative gives an empty one.
                null_mut()
            } else {
                let at = s.wrapping_offset(n1 as isize).cast::<c_void>();
                unsafe { xmemdupz(at, 1) as *mut c_char }
            };
            unsafe { tv_clear(rettv) };
            rv.v_type = VAR_STRING;
            rv.vval.v_string = v;
        }
        VAR_BLOB => {
            // SAFETY: the tag says the union holds a Blob.
            let blob = rv.blob_or_null();
            let _ = unsafe { tv_blob_slice_or_index(blob, is_range, n1, n2, exclusive, rettv) };
        }
        VAR_LIST => {
            if var1.is_null() {
                n1 = 0;
            }
            if var2.is_null() {
                n2 = VARNUMBER_MAX;
            }
            // SAFETY: the tag says the union holds a List.
            let list = rv.list_or_null();
            let sliced = unsafe {
                tv_list_slice_or_index(list, is_range, n1, n2, exclusive, rettv, verbose)
            };
            sliced?;
        }
        VAR_DICT => {
            let mut key = key;
            if key.is_null() {
                // SAFETY: `numbuf2` is this frame's own scratch.
                key = unsafe { numbuf2.string_chk(var1) };
                if key.is_null() {
                    return Err(Failed);
                }
            }
            // SAFETY: the tag says the union holds a Dict, and `key` is the
            // caller's own of `keylen` bytes.
            let dict = rv.dict_or_null();
            let item: *mut dictitem_T = unsafe { tv_dict_find(dict, key, keylen) };
            if item.is_null() && verbose {
                if keylen > 0 {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let key = unsafe { c_str_len(key, keylen as usize) };
                    semsg!("E716: Key not present in Dictionary: \"{key}\"");
                } else {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let key = unsafe { c_str(key) };
                    semsg!("E716: Key not present in Dictionary: \"{key}\"");
                }
            }
            if item.is_null() || unsafe { tv_is_luafunc(&raw mut (*item).di_tv) } {
                return Err(Failed);
            }
            // The copy is taken before `rettv` — which owns the Dict the
            // item lives in — is cleared.
            let mut tmp = UNSET_TV;
            unsafe { tv_copy(&raw mut (*item).di_tv, &raw mut tmp) };
            unsafe { tv_clear(rettv) };
            *rv = tmp;
        }
        // Not evaluating: skipping over the subscript.
        _ => {}
    }
    Ok(())
}

/// `str[index]` by character index, composing characters included. Answers
/// null when `index` is out of range.
///
/// # Safety
/// `str` must be null or NUL-terminated.
pub(crate) unsafe fn char_from_string(str: *const c_char, index: varnumber_T) -> *mut c_char {
    if str.is_null() {
        return null_mut();
    }
    let slen = unsafe { cstr::bytes_at(str) }.len();
    let mut nchar = index;

    // As for a List, a negative index counts from the end — but unlike a
    // List, running off the start is an empty string rather than an error.
    if index < 0 {
        let mut clen: c_int = 0;
        let mut nbyte: size_t = 0;
        while nbyte < slen {
            nbyte += unsafe { utfc_ptr2len(str.add(nbyte as usize)) } as size_t;
            clen += 1;
        }
        nchar = clen as varnumber_T + index;
        if nchar < 0 {
            return null_mut();
        }
    }

    let mut nbyte: size_t = 0;
    while nchar > 0 && nbyte < slen {
        nbyte += unsafe { utfc_ptr2len(str.add(nbyte as usize)) } as size_t;
        nchar -= 1;
    }
    if nbyte >= slen {
        return null_mut();
    }
    // SAFETY: the caller's promise -- `str` is NUL-terminated, and `nbyte`
    // is the start of a character inside it.
    let at = str.wrapping_add(nbyte as usize);
    let len = unsafe { utfc_ptr2len(at) } as size_t;
    unsafe { xmemdupz(at.cast(), len) as *mut c_char }
}

/// The byte index of character index `idx` in `str`, composing characters
/// included. Answers `str_len` for an index past the end and -1 for one
/// before the start.
///
/// # Safety
/// `str` must hold `str_len` readable bytes.
pub(crate) unsafe fn char_idx2byte(
    str: *const c_char,
    str_len: size_t,
    idx: varnumber_T,
) -> ssize_t {
    let mut nchar = idx;
    let mut nbyte: size_t = 0;
    if nchar >= 0 {
        while nchar > 0 && nbyte < str_len {
            nbyte += unsafe { utfc_ptr2len(str.add(nbyte as usize)) } as size_t;
            nchar -= 1;
        }
    } else {
        nbyte = str_len;
        while nchar < 0 && nbyte > 0 {
            nbyte -= 1;
            nbyte -= unsafe { utf_head_off(str, str.add(nbyte as usize)) } as size_t;
            nchar += 1;
        }
        if nchar < 0 {
            return -1;
        }
    }
    nbyte as ssize_t
}

/// `str[first : last]` by character index, composing characters included.
/// `exclusive` is `slice()`'s. Answers null when the result is empty.
///
/// # Safety
/// `str` must be null or NUL-terminated.
pub(crate) unsafe fn string_slice(
    str: *const c_char,
    first: varnumber_T,
    last: varnumber_T,
    exclusive: bool,
) -> *mut c_char {
    if str.is_null() {
        return null_mut();
    }
    let slen = unsafe { cstr::bytes_at(str) }.len();
    // A very negative first index starts at zero rather than failing.
    let start_byte = unsafe { char_idx2byte(str, slen, first) }.max(0);
    let end_byte = if (last == -1 && !exclusive) || last == VARNUMBER_MAX {
        slen as ssize_t
    } else {
        let mut end = unsafe { char_idx2byte(str, slen, last) };
        if !exclusive && end >= 0 && end < slen as ssize_t {
            // The end index is inclusive here.
            end += unsafe { utfc_ptr2len(str.add(end as usize)) } as ssize_t;
        }
        end
    };

    if start_byte >= slen as ssize_t || end_byte <= start_byte {
        return null_mut();
    }
    // SAFETY: as above -- both byte offsets are inside `str`.
    let at = str.wrapping_add(start_byte as usize).cast();
    unsafe { xmemdupz(at, (end_byte - start_byte) as size_t) as *mut c_char }
}

/// Everything that can follow a completed operand, in any order:
/// `expr[idx]`, `expr[a:b]`, `.name`, a call through a Funcref, and
/// `expr->method()`. `dict.func(expr)[idx]['func'](expr)->len()` is one run
/// of this loop.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression whose
/// preceding byte is readable; `rettv` must be valid; `evalarg` null or
/// valid.
pub(crate) unsafe fn handle_subscript(
    arg: *mut *const c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression, `rettv` is the operand it follows and `evalarg` is null or
    // valid. All three hold for every call below.
    let (cur, rv) = unsafe { (Cur::new(arg.cast()), Tv::new(rettv)) };
    let evaluate = unsafe { evaluating(evalarg) };
    let mut ret = Ok(());
    let mut selfdict: *mut dict_T = null_mut();
    let mut lua_funcname: *const c_char = null();

    if unsafe { tv_is_luafunc(rettv) } {
        if !evaluate {
            unsafe { tv_clear(rettv) };
        }
        if cur.byte() != b'.' {
            unsafe { tv_clear(rettv) };
            ret = Err(Failed);
        } else {
            cur.bump(1);
            lua_funcname = cur.get();
            let len = unsafe { check_luafunc_name(cur.get(), true) };
            if len == 0 {
                unsafe { tv_clear(rettv) };
                ret = Err(Failed);
            }
            cur.bump(len as usize);
        }
    }

    // Whether another subscript follows. The byte *before* the cursor is
    // only read once one of the three opening characters is there, which
    // is what proves it is inside the expression rather than before it.
    let more = || {
        let c = cur.byte();
        let opens = c == b'['
            || (c == b'.' && rv.v_type == VAR_DICT)
            || (c == b'(' && (!evaluate || tv_is_func(*rv)));
        // SAFETY: the caller's promise -- the byte before the cursor is
        // readable, and only an opening character asks for it.
        (opens && !ascii_iswhite(unsafe { *cur.get().offset(-1) } as c_int))
            || (c == b'-' && cur.at(1) == b'>')
    };

    while ret.is_ok() && more() {
        if cur.byte() == b'(' {
            let (raw, lua) = (cur.raw(), lua_funcname);
            ret = unsafe {
                call_func_rettv(raw, evalarg, rettv, evaluate, selfdict, null_mut(), lua)
            };
            // Stop evaluating on an immediate abort, an interrupt, or an
            // exception that was thrown and not caught.
            if aborting() {
                if ret.is_ok() {
                    unsafe { tv_clear(rettv) };
                }
                ret = Err(Failed);
            }
            unsafe { tv_dict_unref(selfdict) };
            selfdict = null_mut();
        } else if cur.byte() == b'-' {
            ret = if cur.at(2) == b'{' {
                // expr->{lambda}()
                unsafe { eval_lambda(cur.raw(), rettv, evalarg, verbose) }
            } else {
                // expr->name()
                unsafe { eval_method(cur.raw(), rettv, evalarg, verbose) }
            };
        } else {
            // `[` or `.`: a Dict being subscripted is the `self` a
            // Funcref found in it would be bound to.
            unsafe { tv_dict_unref(selfdict) };
            selfdict = if rv.v_type == VAR_DICT {
                // SAFETY: the tag says the union holds a Dict.
                let d = rv.dict_or_null();
                if !d.is_null() {
                    unsafe { (*d).dv_refcount.retain() };
                }
                d
            } else {
                null_mut()
            };
            if unsafe { eval_index(cur.raw(), rettv, evalarg, verbose) }.is_err() {
                unsafe { tv_clear(rettv) };
                ret = Err(Failed);
            }
        }
    }

    // Turn "dict.Func" into a partial for "Func" bound to "dict".
    if !selfdict.is_null() && tv_is_func(*rv) {
        unsafe { set_selfdict(rettv, selfdict) };
    }
    unsafe { tv_dict_unref(selfdict) };
    ret
}

/// Bind `selfdict` to the Funcref in `rettv`.
///
/// # Safety
/// `rettv` must be valid and `selfdict` must be a reference this call takes
/// over.
pub(crate) unsafe fn set_selfdict(rettv: *mut typval_T, selfdict: *mut dict_T) {
    // Not for a partial that was bound explicitly (`pt_auto` clear).
    // SAFETY: the caller's promise -- `rettv` is valid, and the tag says
    // whether the union holds a live partial.
    let rv = unsafe { Tv::new(rettv) };
    if rv.v_type == VAR_PARTIAL {
        let pt = unsafe { Live::new(rv.partial_or_null()) };
        if !pt.pt_auto && !pt.pt_dict.is_null() {
            return;
        }
    }
    unsafe { make_partial(selfdict, rettv) };
}
