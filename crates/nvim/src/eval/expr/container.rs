//! List and dict literals, including the `#{}` form.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int};
use core::ptr::null_mut;

use crate::ascii::ascii_isdigit;
use crate::charset::skipwhite;
use crate::eval::typval::{
    tv_clear, tv_dict_add, tv_dict_alloc, tv_dict_find, tv_dict_free, tv_dict_item_alloc,
    tv_dict_item_free, tv_dict_set_ret, tv_get_string_buf_chk, tv_list_alloc,
    tv_list_append_owned_tv, tv_list_free, tv_list_set_ret,
};
use crate::eval::{EVAL_EVALUATE, NOTDONE, e_list_end, eval1};
use crate::memory::xmemdupz;
use crate::os::cshim::gettext;
use crate::types::{
    FAIL, NUL, OK, VAR_STRING, VAR_UNKNOWN, VarLock, dict_T, dictitem_T, evalarg_T,
    kListLenShouldKnow, list_T, ptrdiff_t, size_t, typval_T, typval_vval_union,
};

/// The scratch a non-String dict key is rendered into.
const NUMBUFLEN: usize = 65;

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

/// `[a, b, c]`, with the cursor on the `[`.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression.
pub(crate) unsafe fn eval_list(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let evaluate = unsafe { evaluating(evalarg) };
    let list: *mut list_T = if evaluate {
        unsafe { tv_list_alloc(kListLenShouldKnow as ptrdiff_t) }
    } else {
        null_mut()
    };
    unsafe { *arg  = skipwhite((*arg).add(1)) };

    let ok = 'items: {
        while unsafe { **arg } != b']' as c_char && unsafe { **arg } as c_int != NUL {
            let mut tv = UNSET_TV;
            if unsafe { eval1(arg, &raw mut tv, evalarg) } == FAIL {
                break 'items false;
            }
            if evaluate {
                tv.v_lock = VarLock::Unlocked;
                unsafe { tv_list_append_owned_tv(list, tv) };
            }
            let had_comma = unsafe { **arg } == b',' as c_char;
            if had_comma {
                unsafe { *arg  = skipwhite((*arg).add(1)) };
            }
            if unsafe { **arg } == b']' as c_char {
                break;
            }
            // A trailing comma is allowed; a missing one is not.
            if had_comma {
                continue;
            }
            semsg_c!(unsafe { gettext(c"E696: Missing comma in List: %s".as_ptr()) }, unsafe { *arg });
            break 'items false;
        }
        if unsafe { **arg } != b']' as c_char {
            semsg_c!(unsafe { gettext(e_list_end.as_ptr()) }, unsafe { *arg });
            break 'items false;
        }
        unsafe { *arg  = skipwhite((*arg).add(1)) };
        if evaluate {
            unsafe { tv_list_set_ret(rettv, list) };
        }
        true
    };

    if ok {
        return OK;
    }
    if evaluate {
        unsafe { tv_list_free(list) };
    }
    FAIL
}

/// The bare word a `#{}` literal uses as a key.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression.
pub(crate) unsafe fn get_literal_key(arg: *mut *mut c_char, tv: *mut typval_T) -> c_int {
    /// Letters, digits, `_` and `-`: what a literal key may contain.
    fn is_key_char(c: c_char) -> bool {
        let b = c as u8;
        b.is_ascii_alphabetic() || ascii_isdigit(c as c_int) || b == b'_' || b == b'-'
    }

    if !is_key_char(unsafe { **arg }) {
        return FAIL;
    }
    let mut p = unsafe { *arg };
    while is_key_char(unsafe { *p }) {
        p = unsafe { p.add(1) };
    }
    unsafe { (*tv) .v_type  = VAR_STRING };
    unsafe { (*tv) .vval.v_string  = xmemdupz((*arg).cast(), p.offset_from(*arg) as size_t) as *mut c_char };
    unsafe { *arg  = skipwhite(p) };
    OK
}

/// `{k: v}` and, with `literal` set, `#{k: v}`.
///
/// Answers `NOTDONE` when the `{` opened a curly-braces name rather than a
/// dictionary, which the caller then re-reads as a name.
///
/// # Safety
/// `arg` must point at the cursor, on the `{`.
pub(crate) unsafe fn eval_dict(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    literal: bool,
) -> c_int {
    let evaluate = unsafe { evaluating(evalarg) };
    let mut tv = UNSET_TV;
    let mut buf: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];

    // Is this `{expr}` rather than a Dict? It has to be decided without
    // evaluating, or a function in it would be called twice — which is
    // also why `eval1` is handed no `evalarg`. `{}` is an empty Dict and
    // `#{abc}` is never a curly-braces name.
    let mut curly_expr = unsafe { skipwhite((*arg).add(1)) };
    if unsafe { *curly_expr } != b'}' as c_char
        && !literal
        && unsafe { eval1(&raw mut curly_expr, &raw mut tv, null_mut()) } == OK
        && unsafe { *skipwhite(curly_expr) } == b'}' as c_char
    {
        return NOTDONE;
    }

    let dict: *mut dict_T = if evaluate {
        unsafe { tv_dict_alloc() }
    } else {
        null_mut()
    };
    let mut tvkey = UNSET_TV;
    tv = UNSET_TV;
    unsafe { *arg  = skipwhite((*arg).add(1)) };

    let ok = 'items: {
        while unsafe { **arg } != b'}' as c_char && unsafe { **arg } as c_int != NUL {
            let read_key = if literal {
                unsafe { get_literal_key(arg, &raw mut tvkey) }
            } else {
                unsafe { eval1(arg, &raw mut tvkey, evalarg) }
            };
            if read_key == FAIL {
                break 'items false;
            }
            if unsafe { **arg } != b':' as c_char {
                semsg_c!(
                    unsafe { gettext(c"E720: Missing colon in Dictionary: %s".as_ptr()) },
                    unsafe { *arg },
                );
                unsafe { tv_clear(&raw mut tvkey) };
                break 'items false;
            }

            // The key borrows `buf`, so it must not outlive this pass.
            let mut key: *mut c_char = null_mut();
            if evaluate {
                key = unsafe { tv_get_string_buf_chk(&raw mut tvkey, buf.as_mut_ptr()) } as *mut c_char;
                if key.is_null() {
                    unsafe { tv_clear(&raw mut tvkey) };
                    break 'items false;
                }
            }
            unsafe { *arg  = skipwhite((*arg).add(1)) };
            if unsafe { eval1(arg, &raw mut tv, evalarg) } == FAIL {
                unsafe { tv_clear(&raw mut tvkey) };
                break 'items false;
            }
            if evaluate {
                if !unsafe { tv_dict_find(dict, key, -1 as ptrdiff_t) }.is_null() {
                    semsg_c!(
                        unsafe { gettext(c"E721: Duplicate key in Dictionary: \"%s\"".as_ptr()) },
                        key,
                    );
                    unsafe { tv_clear(&raw mut tvkey) };
                    unsafe { tv_clear(&raw mut tv) };
                    break 'items false;
                }
                let item: *mut dictitem_T = unsafe { tv_dict_item_alloc(key) };
                unsafe { (*item) .di_tv  = tv };
                unsafe { (*item) .di_tv.v_lock  = VarLock::Unlocked };
                if unsafe { tv_dict_add(dict, item) } == FAIL {
                    unsafe { tv_dict_item_free(item) };
                }
            }
            unsafe { tv_clear(&raw mut tvkey) };

            let had_comma = unsafe { **arg } == b',' as c_char;
            if had_comma {
                unsafe { *arg  = skipwhite((*arg).add(1)) };
            }
            if unsafe { **arg } == b'}' as c_char {
                break;
            }
            if had_comma {
                continue;
            }
            semsg_c!(
                unsafe { gettext(c"E722: Missing comma in Dictionary: %s".as_ptr()) },
                unsafe { *arg },
            );
            break 'items false;
        }
        if unsafe { **arg } != b'}' as c_char {
            semsg_c!(
                unsafe { gettext(c"E723: Missing end of Dictionary '}': %s".as_ptr()) },
                unsafe { *arg },
            );
            break 'items false;
        }
        unsafe { *arg  = skipwhite((*arg).add(1)) };
        if evaluate {
            unsafe { tv_dict_set_ret(rettv, dict) };
        }
        true
    };

    if ok {
        return OK;
    }
    if !dict.is_null() {
        unsafe { tv_dict_free(dict) };
    }
    FAIL
}

/// `#{...}`, with the cursor on the `#`.
///
/// # Safety
/// As `eval_dict`.
pub(crate) unsafe fn eval_lit_dict(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    if unsafe { *(*arg).add(1) } != b'{' as c_char {
        return NOTDONE;
    }
    unsafe { *arg  = (*arg).add(1) };
    unsafe { eval_dict(arg, rettv, evalarg, true) }
}
