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
use crate::eval::{EVAL_EVALUATE, FAIL, NOTDONE, NUL, OK, e_list_end, eval1};
use crate::memory::xmemdupz;
use crate::os::cshim::gettext;
use crate::types::{
    VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, dict_T, dictitem_T, evalarg_T, kListLenShouldKnow,
    list_T, ptrdiff_t, size_t, typval_T, typval_vval_union,
};

/// The scratch a non-String dict key is rendered into.
const NUMBUFLEN: usize = 65;

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

/// `[a, b, c]`, with the cursor on the `[`.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression.
pub(crate) unsafe fn eval_list(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    unsafe {
        let evaluate = evaluating(evalarg);
        let list: *mut list_T = if evaluate {
            tv_list_alloc(kListLenShouldKnow as ptrdiff_t)
        } else {
            null_mut()
        };
        *arg = skipwhite((*arg).add(1));

        let ok = 'items: {
            while **arg != b']' as c_char && **arg as c_int != NUL {
                let mut tv = UNSET_TV;
                if eval1(arg, &raw mut tv, evalarg) == FAIL {
                    break 'items false;
                }
                if evaluate {
                    tv.v_lock = VAR_UNLOCKED;
                    tv_list_append_owned_tv(list, tv);
                }
                let had_comma = **arg == b',' as c_char;
                if had_comma {
                    *arg = skipwhite((*arg).add(1));
                }
                if **arg == b']' as c_char {
                    break;
                }
                // A trailing comma is allowed; a missing one is not.
                if had_comma {
                    continue;
                }
                semsg_c!(gettext(c"E696: Missing comma in List: %s".as_ptr()), *arg);
                break 'items false;
            }
            if **arg != b']' as c_char {
                semsg_c!(gettext(e_list_end.as_ptr()), *arg);
                break 'items false;
            }
            *arg = skipwhite((*arg).add(1));
            if evaluate {
                tv_list_set_ret(rettv, list);
            }
            true
        };

        if ok {
            return OK;
        }
        if evaluate {
            tv_list_free(list);
        }
        FAIL
    }
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

    unsafe {
        if !is_key_char(**arg) {
            return FAIL;
        }
        let mut p = *arg;
        while is_key_char(*p) {
            p = p.add(1);
        }
        (*tv).v_type = VAR_STRING;
        (*tv).vval.v_string = xmemdupz((*arg).cast(), p.offset_from(*arg) as size_t) as *mut c_char;
        *arg = skipwhite(p);
        OK
    }
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
    unsafe {
        let evaluate = evaluating(evalarg);
        let mut tv = UNSET_TV;
        let mut buf: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];

        // Is this `{expr}` rather than a Dict? It has to be decided without
        // evaluating, or a function in it would be called twice — which is
        // also why `eval1` is handed no `evalarg`. `{}` is an empty Dict and
        // `#{abc}` is never a curly-braces name.
        let mut curly_expr = skipwhite((*arg).add(1));
        if *curly_expr != b'}' as c_char
            && !literal
            && eval1(&raw mut curly_expr, &raw mut tv, null_mut()) == OK
            && *skipwhite(curly_expr) == b'}' as c_char
        {
            return NOTDONE;
        }

        let dict: *mut dict_T = if evaluate {
            tv_dict_alloc()
        } else {
            null_mut()
        };
        let mut tvkey = UNSET_TV;
        tv = UNSET_TV;
        *arg = skipwhite((*arg).add(1));

        let ok = 'items: {
            while **arg != b'}' as c_char && **arg as c_int != NUL {
                let read_key = if literal {
                    get_literal_key(arg, &raw mut tvkey)
                } else {
                    eval1(arg, &raw mut tvkey, evalarg)
                };
                if read_key == FAIL {
                    break 'items false;
                }
                if **arg != b':' as c_char {
                    semsg_c!(
                        gettext(c"E720: Missing colon in Dictionary: %s".as_ptr()),
                        *arg,
                    );
                    tv_clear(&raw mut tvkey);
                    break 'items false;
                }

                // The key borrows `buf`, so it must not outlive this pass.
                let mut key: *mut c_char = null_mut();
                if evaluate {
                    key = tv_get_string_buf_chk(&raw mut tvkey, buf.as_mut_ptr()) as *mut c_char;
                    if key.is_null() {
                        tv_clear(&raw mut tvkey);
                        break 'items false;
                    }
                }
                *arg = skipwhite((*arg).add(1));
                if eval1(arg, &raw mut tv, evalarg) == FAIL {
                    tv_clear(&raw mut tvkey);
                    break 'items false;
                }
                if evaluate {
                    if !tv_dict_find(dict, key, -1 as ptrdiff_t).is_null() {
                        semsg_c!(
                            gettext(c"E721: Duplicate key in Dictionary: \"%s\"".as_ptr()),
                            key,
                        );
                        tv_clear(&raw mut tvkey);
                        tv_clear(&raw mut tv);
                        break 'items false;
                    }
                    let item: *mut dictitem_T = tv_dict_item_alloc(key);
                    (*item).di_tv = tv;
                    (*item).di_tv.v_lock = VAR_UNLOCKED;
                    if tv_dict_add(dict, item) == FAIL {
                        tv_dict_item_free(item);
                    }
                }
                tv_clear(&raw mut tvkey);

                let had_comma = **arg == b',' as c_char;
                if had_comma {
                    *arg = skipwhite((*arg).add(1));
                }
                if **arg == b'}' as c_char {
                    break;
                }
                if had_comma {
                    continue;
                }
                semsg_c!(
                    gettext(c"E722: Missing comma in Dictionary: %s".as_ptr()),
                    *arg,
                );
                break 'items false;
            }
            if **arg != b'}' as c_char {
                semsg_c!(
                    gettext(c"E723: Missing end of Dictionary '}': %s".as_ptr()),
                    *arg,
                );
                break 'items false;
            }
            *arg = skipwhite((*arg).add(1));
            if evaluate {
                tv_dict_set_ret(rettv, dict);
            }
            true
        };

        if ok {
            return OK;
        }
        if !dict.is_null() {
            tv_dict_free(dict);
        }
        FAIL
    }
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
    unsafe {
        if *(*arg).add(1) != b'{' as c_char {
            return NOTDONE;
        }
        *arg = (*arg).add(1);
        eval_dict(arg, rettv, evalarg, true)
    }
}
