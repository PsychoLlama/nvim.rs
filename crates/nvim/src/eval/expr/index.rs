//! `[]`, `[:]` and `.` applied to a value the evaluator already has.
//!
//! Two indexing vocabularies live here and they are not the same. The
//! subscript the *grammar* produces (`s[1]`, `s[1:2]`) counts **bytes** in a
//! String and includes its end; `slice()` counts **characters** and excludes
//! its end. `exclusive` is the flag that tells them apart, and it also
//! switches the String arm onto the character walkers.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{null, null_mut};

use crate::ascii::ascii_iswhite;
use crate::charset::skipwhite;
use crate::eval::typval::{
    tv_blob_slice_or_index, tv_check_str, tv_clear, tv_copy, tv_dict_find, tv_dict_unref,
    tv_get_number, tv_get_string, tv_get_string_chk, tv_is_func, tv_list_slice_or_index,
};
use crate::eval::userfunc::make_partial;
use crate::eval::{
    EVAL_EVALUATE, VARNUMBER_MAX, call_func_rettv, check_luafunc_name, e_cannot_index_a_funcref,
    e_cannot_index_special_variable, e_cannot_slice_dictionary, e_missbrac, eval_isdictc,
    eval_lambda, eval_method, eval1, tv_is_luafunc,
};
use crate::ex_eval::aborting;
use crate::main::{e_dictkey, e_dictkey_len, e_using_float_as_string};
use crate::mbyte::{utf_head_off, utfc_ptr2len};
use crate::memory::xmemdupz;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::types::{
    EvalFuncData, FAIL, OK, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST,
    VAR_NUMBER, VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, dict_T,
    dictitem_T, evalarg_T, ptrdiff_t, size_t, ssize_t, typval_T, typval_vval_union, varnumber_T,
};
use ::libc::strlen;

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
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
) -> c_int {
    unsafe {
        let evaluate = evaluating(evalarg);
        let mut empty1 = false;
        let mut empty2 = false;
        let mut range = false;
        let mut key: *const c_char = null();
        let mut keylen: ptrdiff_t = -1;

        if check_can_index(rettv, evaluate, verbose) == FAIL {
            return FAIL;
        }

        let mut var1 = UNSET_TV;
        let mut var2 = UNSET_TV;
        if **arg == b'.' as c_char {
            // dict.name
            key = (*arg).add(1);
            keylen = 0;
            while eval_isdictc(*key.offset(keylen) as c_int) {
                keylen += 1;
            }
            if keylen == 0 {
                return FAIL;
            }
            *arg = skipwhite(key.offset(keylen));
        } else {
            // The first index, from inside the brackets.
            *arg = skipwhite((*arg).add(1));
            if **arg == b':' as c_char {
                empty1 = true;
            } else if eval1(arg, &raw mut var1, evalarg) == FAIL {
                return FAIL;
            } else if evaluate && !tv_check_str(&raw mut var1) {
                tv_clear(&raw mut var1);
                return FAIL;
            }

            // The second index, from inside the `[ : ]`.
            if **arg == b':' as c_char {
                range = true;
                *arg = skipwhite((*arg).add(1));
                if **arg == b']' as c_char {
                    empty2 = true;
                } else if eval1(arg, &raw mut var2, evalarg) == FAIL {
                    if !empty1 {
                        tv_clear(&raw mut var1);
                    }
                    return FAIL;
                } else if evaluate && !tv_check_str(&raw mut var2) {
                    if !empty1 {
                        tv_clear(&raw mut var1);
                    }
                    tv_clear(&raw mut var2);
                    return FAIL;
                }
            }

            if **arg != b']' as c_char {
                if verbose {
                    emsg(gettext(e_missbrac.as_ptr()));
                }
                // Not guarded by `empty1`: an unread `var1` is still unset.
                tv_clear(&raw mut var1);
                if range {
                    tv_clear(&raw mut var2);
                }
                return FAIL;
            }
            *arg = skipwhite((*arg).add(1));
        }

        if !evaluate {
            return OK;
        }
        let res = eval_index_inner(
            rettv,
            range,
            if empty1 { null_mut() } else { &raw mut var1 },
            if empty2 { null_mut() } else { &raw mut var2 },
            false,
            key,
            keylen,
            verbose,
        );
        if !empty1 {
            tv_clear(&raw mut var1);
        }
        if range {
            tv_clear(&raw mut var2);
        }
        res
    }
}

/// Can `rettv` carry an `[index]` or a `[sli:ce]` at all?
///
/// # Safety
/// `rettv` must be valid.
pub(crate) unsafe fn check_can_index(rettv: *mut typval_T, evaluate: bool, verbose: bool) -> c_int {
    let message = match unsafe { (*rettv).v_type } {
        VAR_FUNC | VAR_PARTIAL => e_cannot_index_a_funcref.as_ptr(),
        VAR_FLOAT => e_using_float_as_string.as_ptr(),
        VAR_BOOL | VAR_SPECIAL => e_cannot_index_special_variable.as_ptr(),
        // Not evaluating: the subscript is only being skipped over, and an
        // unset value is what an unevaluated operand looks like.
        VAR_UNKNOWN if !evaluate => return OK,
        // Reported whether or not the caller asked to be verbose.
        VAR_UNKNOWN => {
            // SAFETY: a message constant is a NUL-terminated literal.
            unsafe { emsg(gettext(e_cannot_index_special_variable.as_ptr())) };
            return FAIL;
        }
        _ => return OK,
    };
    if verbose {
        // SAFETY: as above.
        unsafe { emsg(gettext(message)) };
    }
    FAIL
}

/// `slice()`
///
/// # Safety
/// Called through the builtin table with a terminated argument array.
pub unsafe fn f_slice(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        if check_can_index(argvars, true, false) != OK {
            return;
        }
        tv_copy(argvars, rettv);
        let last = argvars.add(2);
        eval_index_inner(
            rettv,
            true,
            argvars.add(1),
            if (*last).v_type == VAR_UNKNOWN {
                null_mut()
            } else {
                last
            },
            true,
            null(),
            0,
            false,
        );
    }
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
) -> c_int {
    unsafe {
        let mut n1: varnumber_T = 0;
        let mut n2: varnumber_T = 0;
        if !var1.is_null() && (*rettv).v_type != VAR_DICT {
            n1 = tv_get_number(var1);
        }
        if is_range {
            if (*rettv).v_type == VAR_DICT {
                if verbose {
                    emsg(gettext(e_cannot_slice_dictionary.as_ptr()));
                }
                return FAIL;
            }
            n2 = if var2.is_null() {
                VARNUMBER_MAX
            } else {
                tv_get_number(var2)
            };
        }

        match (*rettv).v_type {
            VAR_NUMBER | VAR_STRING => {
                let s = tv_get_string(rettv);
                let len = strlen(s) as c_int as varnumber_T;
                let v = if exclusive {
                    // slice(): character indexes, second one excluded.
                    if is_range {
                        string_slice(s, n1, n2, exclusive)
                    } else {
                        char_from_string(s, n1)
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
                        xmemdupz(s.offset(n1 as isize).cast(), (n2 - n1 + 1) as size_t)
                            as *mut c_char
                    }
                } else if n1 >= len || n1 < 0 {
                    // A one-byte String; too big or negative gives an empty one.
                    null_mut()
                } else {
                    xmemdupz(s.offset(n1 as isize).cast::<c_void>(), 1) as *mut c_char
                };
                tv_clear(rettv);
                (*rettv).v_type = VAR_STRING;
                (*rettv).vval.v_string = v;
            }
            VAR_BLOB => {
                tv_blob_slice_or_index((*rettv).vval.v_blob, is_range, n1, n2, exclusive, rettv);
            }
            VAR_LIST => {
                if var1.is_null() {
                    n1 = 0;
                }
                if var2.is_null() {
                    n2 = VARNUMBER_MAX;
                }
                if tv_list_slice_or_index(
                    (*rettv).vval.v_list,
                    is_range,
                    n1,
                    n2,
                    exclusive,
                    rettv,
                    verbose,
                ) == FAIL
                {
                    return FAIL;
                }
            }
            VAR_DICT => {
                let mut key = key;
                if key.is_null() {
                    key = tv_get_string_chk(var1);
                    if key.is_null() {
                        return FAIL;
                    }
                }
                let item: *mut dictitem_T = tv_dict_find((*rettv).vval.v_dict, key, keylen);
                if item.is_null() && verbose {
                    if keylen > 0 {
                        semsg_c!(gettext(e_dictkey_len.as_ptr()), keylen, key);
                    } else {
                        semsg_c!(gettext(e_dictkey.as_ptr()), key);
                    }
                }
                if item.is_null() || tv_is_luafunc(&raw mut (*item).di_tv) {
                    return FAIL;
                }
                // The copy is taken before `rettv` — which owns the Dict the
                // item lives in — is cleared.
                let mut tmp = UNSET_TV;
                tv_copy(&raw mut (*item).di_tv, &raw mut tmp);
                tv_clear(rettv);
                *rettv = tmp;
            }
            // Not evaluating: skipping over the subscript.
            _ => {}
        }
        OK
    }
}

/// `str[index]` by character index, composing characters included. Answers
/// null when `index` is out of range.
///
/// # Safety
/// `str` must be null or NUL-terminated.
pub(crate) unsafe fn char_from_string(str: *const c_char, index: varnumber_T) -> *mut c_char {
    unsafe {
        if str.is_null() {
            return null_mut();
        }
        let slen = strlen(str);
        let mut nchar = index;

        // As for a List, a negative index counts from the end — but unlike a
        // List, running off the start is an empty string rather than an error.
        if index < 0 {
            let mut clen: c_int = 0;
            let mut nbyte: size_t = 0;
            while nbyte < slen {
                nbyte += utfc_ptr2len(str.add(nbyte as usize)) as size_t;
                clen += 1;
            }
            nchar = clen as varnumber_T + index;
            if nchar < 0 {
                return null_mut();
            }
        }

        let mut nbyte: size_t = 0;
        while nchar > 0 && nbyte < slen {
            nbyte += utfc_ptr2len(str.add(nbyte as usize)) as size_t;
            nchar -= 1;
        }
        if nbyte >= slen {
            return null_mut();
        }
        xmemdupz(
            str.add(nbyte as usize).cast(),
            utfc_ptr2len(str.add(nbyte as usize)) as size_t,
        ) as *mut c_char
    }
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
    unsafe {
        let mut nchar = idx;
        let mut nbyte: size_t = 0;
        if nchar >= 0 {
            while nchar > 0 && nbyte < str_len {
                nbyte += utfc_ptr2len(str.add(nbyte as usize)) as size_t;
                nchar -= 1;
            }
        } else {
            nbyte = str_len;
            while nchar < 0 && nbyte > 0 {
                nbyte -= 1;
                nbyte -= utf_head_off(str, str.add(nbyte as usize)) as size_t;
                nchar += 1;
            }
            if nchar < 0 {
                return -1;
            }
        }
        nbyte as ssize_t
    }
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
    unsafe {
        if str.is_null() {
            return null_mut();
        }
        let slen = strlen(str);
        // A very negative first index starts at zero rather than failing.
        let start_byte = char_idx2byte(str, slen, first).max(0);
        let end_byte = if (last == -1 && !exclusive) || last == VARNUMBER_MAX {
            slen as ssize_t
        } else {
            let mut end = char_idx2byte(str, slen, last);
            if !exclusive && end >= 0 && end < slen as ssize_t {
                // The end index is inclusive here.
                end += utfc_ptr2len(str.add(end as usize)) as ssize_t;
            }
            end
        };

        if start_byte >= slen as ssize_t || end_byte <= start_byte {
            return null_mut();
        }
        xmemdupz(
            str.add(start_byte as usize).cast(),
            (end_byte - start_byte) as size_t,
        ) as *mut c_char
    }
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
pub unsafe fn handle_subscript(
    arg: *mut *const c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> c_int {
    unsafe {
        let evaluate = evaluating(evalarg);
        let mut ret = OK;
        let mut selfdict: *mut dict_T = null_mut();
        let mut lua_funcname: *const c_char = null();

        if tv_is_luafunc(rettv) {
            if !evaluate {
                tv_clear(rettv);
            }
            if **arg != b'.' as c_char {
                tv_clear(rettv);
                ret = FAIL;
            } else {
                *arg = (*arg).add(1);
                lua_funcname = *arg;
                let len = check_luafunc_name(*arg, true);
                if len == 0 {
                    tv_clear(rettv);
                    ret = FAIL;
                }
                *arg = (*arg).offset(len as isize);
            }
        }

        // Whether another subscript follows. The byte *before* the cursor is
        // only read once one of the three opening characters is there, which
        // is what proves it is inside the expression rather than before it.
        let more = || {
            let c = **arg;
            let opens = c == b'[' as c_char
                || (c == b'.' as c_char && (*rettv).v_type == VAR_DICT)
                || (c == b'(' as c_char && (!evaluate || tv_is_func(*rettv)));
            (opens && !ascii_iswhite(*(*arg).offset(-1) as c_int))
                || (c == b'-' as c_char && *(*arg).add(1) == b'>' as c_char)
        };

        while ret == OK && more() {
            if **arg == b'(' as c_char {
                ret = call_func_rettv(
                    arg as *mut *mut c_char,
                    evalarg,
                    rettv,
                    evaluate,
                    selfdict,
                    null_mut(),
                    lua_funcname,
                );
                // Stop evaluating on an immediate abort, an interrupt, or an
                // exception that was thrown and not caught.
                if aborting() {
                    if ret == OK {
                        tv_clear(rettv);
                    }
                    ret = FAIL;
                }
                tv_dict_unref(selfdict);
                selfdict = null_mut();
            } else if **arg == b'-' as c_char {
                ret = if *(*arg).add(2) == b'{' as c_char {
                    // expr->{lambda}()
                    eval_lambda(arg as *mut *mut c_char, rettv, evalarg, verbose)
                } else {
                    // expr->name()
                    eval_method(arg as *mut *mut c_char, rettv, evalarg, verbose)
                };
            } else {
                // `[` or `.`: a Dict being subscripted is the `self` a
                // Funcref found in it would be bound to.
                tv_dict_unref(selfdict);
                selfdict = if (*rettv).v_type == VAR_DICT {
                    let d = (*rettv).vval.v_dict;
                    if !d.is_null() {
                        (*d).dv_refcount += 1;
                    }
                    d
                } else {
                    null_mut()
                };
                if eval_index(arg as *mut *mut c_char, rettv, evalarg, verbose) == FAIL {
                    tv_clear(rettv);
                    ret = FAIL;
                }
            }
        }

        // Turn "dict.Func" into a partial for "Func" bound to "dict".
        if !selfdict.is_null() && tv_is_func(*rettv) {
            set_selfdict(rettv, selfdict);
        }
        tv_dict_unref(selfdict);
        ret
    }
}

/// Bind `selfdict` to the Funcref in `rettv`.
///
/// # Safety
/// `rettv` must be valid and `selfdict` must be a reference this call takes
/// over.
pub unsafe fn set_selfdict(rettv: *mut typval_T, selfdict: *mut dict_T) {
    unsafe {
        // Not for a partial that was bound explicitly (`pt_auto` clear).
        if (*rettv).v_type == VAR_PARTIAL
            && !(*(*rettv).vval.v_partial).pt_auto
            && !(*(*rettv).vval.v_partial).pt_dict.is_null()
        {
            return;
        }
        make_partial(selfdict, rettv);
    }
}
