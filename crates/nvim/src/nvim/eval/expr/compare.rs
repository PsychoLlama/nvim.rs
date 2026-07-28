//! Comparing two values, for every operator and every pair of types.
//!
//! `typval_compare` is one long else-if chain over the *pair* of types, and
//! the order of its arms is the semantics: a Blob on either side makes the
//! comparison a Blob comparison, then List, then Dict, then Funcref, then
//! Float, then Number, and only what is left compares as a String.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

use crate::src::nvim::eval::typval::{
    tv_blob_equal, tv_clear, tv_dict_equal, tv_equal, tv_get_float, tv_get_number,
    tv_get_string_buf, tv_is_func, tv_list_equal,
};
use crate::src::nvim::eval::{
    EXPR_EQUAL, EXPR_GEQUAL, EXPR_GREATER, EXPR_IS, EXPR_ISNOT, EXPR_MATCH, EXPR_NEQUAL,
    EXPR_NOMATCH, EXPR_SEQUAL, EXPR_SMALLER, FAIL, NUL, OK, VAR_BLOB, VAR_DICT, VAR_FLOAT,
    VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL, e_invalblob, partial_name, pattern_match,
};
use crate::src::nvim::mbyte::mb_strcmp_ic;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{gettext, strcmp};
use crate::src::nvim::types::{dict_T, exprtype_T, float_T, typval_T, varnumber_T};

/// The scratch a Number or Float is rendered into for a String comparison.
const NUMBUFLEN: usize = 65;

/// The name a Funcref or partial calls, with `""` read as "none".
///
/// # Safety
/// `tv` must be a `VAR_FUNC` or `VAR_PARTIAL` typval.
unsafe fn callable_name(tv: *mut typval_T) -> *mut c_char {
    let name = unsafe {
        if (*tv).v_type == VAR_FUNC {
            (*tv).vval.v_string
        } else {
            partial_name((*tv).vval.v_partial)
        }
    };
    if !name.is_null() && unsafe { *name } as c_int == NUL {
        return core::ptr::null_mut();
    }
    name
}

/// Are two Funcrefs — or two partials, or one of each — the same callable?
///
/// Equal names, equal bound dictionaries and equal bound arguments. A name
/// that is present but empty counts as absent, which is how a partial with
/// no function compares against a null Funcref.
///
/// # Safety
/// Both operands must be `VAR_FUNC` or `VAR_PARTIAL` typvals.
pub unsafe fn func_equal(tv1: *mut typval_T, tv2: *mut typval_T, ic: bool) -> bool {
    unsafe {
        let s1 = callable_name(tv1);
        let s2 = callable_name(tv2);
        if s1.is_null() || s2.is_null() {
            if s1 != s2 {
                return false;
            }
        } else if strcmp(s1, s2) != 0 {
            return false;
        }

        // A plain Funcref carries neither a bound dictionary nor arguments.
        let dict_of = |tv: *mut typval_T| -> *mut dict_T {
            if (*tv).v_type == VAR_FUNC {
                core::ptr::null_mut()
            } else {
                (*(*tv).vval.v_partial).pt_dict
            }
        };
        let d1 = dict_of(tv1);
        let d2 = dict_of(tv2);
        if d1.is_null() || d2.is_null() {
            if d1 != d2 {
                return false;
            }
        } else if !tv_dict_equal(d1, d2, ic) {
            return false;
        }

        let argc_of = |tv: *mut typval_T| -> c_int {
            if (*tv).v_type == VAR_FUNC {
                0
            } else {
                (*(*tv).vval.v_partial).pt_argc
            }
        };
        let argc = argc_of(tv1);
        if argc != argc_of(tv2) {
            return false;
        }
        // Reachable only with a non-zero count, which means both sides are
        // partials with an argument vector.
        (0..argc).all(|i| {
            tv_equal(
                (*(*tv1).vval.v_partial).pt_argv.offset(i as isize),
                (*(*tv2).vval.v_partial).pt_argv.offset(i as isize),
                ic,
            )
        })
    }
}

/// The shared shape of the Blob, List and Dict arms.
///
/// They differ only in which pointer identity `is` means, which equality
/// function `==` uses, and which pair of messages an operator they do not
/// support reports. Both tests are closures because upstream reaches them
/// through short circuits: `identical` reads the union member that is only
/// the right one once `same_type` has held.
///
/// Answers `None` after reporting and clearing `typ1`.
///
/// # Safety
/// `typ1` must be a valid typval the caller has given up ownership of.
unsafe fn compare_container(
    typ1: *mut typval_T,
    op: exprtype_T,
    same_type: bool,
    identical: impl FnOnce() -> bool,
    equal: impl FnOnce() -> bool,
    wrong_type: &CStr,
    wrong_op: &CStr,
) -> Option<varnumber_T> {
    if op == EXPR_IS || op == EXPR_ISNOT {
        let same = same_type && identical();
        Some(varnumber_T::from(same == (op == EXPR_IS)))
    } else if !same_type || (op != EXPR_EQUAL && op != EXPR_NEQUAL) {
        unsafe {
            emsg(gettext(
                if !same_type { wrong_type } else { wrong_op }.as_ptr(),
            ));
            tv_clear(typ1);
        }
        None
    } else {
        Some(varnumber_T::from(equal() == (op == EXPR_EQUAL)))
    }
}

/// Turn an already-computed three-way comparison into the operator's answer.
///
/// `EXPR_UNKNOWN` and the two pattern operators answer false; the callers
/// that can see a pattern operator handle it themselves.
fn from_ordering(op: exprtype_T, i: c_int) -> varnumber_T {
    varnumber_T::from(match op {
        EXPR_IS | EXPR_EQUAL => i == 0,
        EXPR_ISNOT | EXPR_NEQUAL => i != 0,
        EXPR_GREATER => i > 0,
        EXPR_GEQUAL => i >= 0,
        EXPR_SMALLER => i < 0,
        EXPR_SEQUAL => i <= 0,
        _ => false,
    })
}

/// Evaluate `typ1 <op> typ2`, leaving the Number answer in `typ1`.
///
/// # Safety
/// Both operands must be valid typvals; `typ1` is cleared either way and
/// receives the result.
pub unsafe fn typval_compare(
    typ1: *mut typval_T,
    typ2: *mut typval_T,
    op: exprtype_T,
    ic: bool,
) -> c_int {
    unsafe {
        let type_is = op == EXPR_IS || op == EXPR_ISNOT;
        let (t1, t2) = ((*typ1).v_type, (*typ2).v_type);
        let same_type = t1 == t2;

        let answer: varnumber_T = if type_is && !same_type {
            // `is` between two different types is simply false.
            varnumber_T::from(op == EXPR_ISNOT)
        } else if t1 == VAR_BLOB || t2 == VAR_BLOB {
            match compare_container(
                typ1,
                op,
                same_type,
                || (*typ1).vval.v_blob == (*typ2).vval.v_blob,
                || tv_blob_equal((*typ1).vval.v_blob, (*typ2).vval.v_blob),
                c"E977: Can only compare Blob with Blob",
                CStr::from_ptr((&raw const e_invalblob).cast()),
            ) {
                Some(n) => n,
                None => return FAIL,
            }
        } else if t1 == VAR_LIST || t2 == VAR_LIST {
            match compare_container(
                typ1,
                op,
                same_type,
                || (*typ1).vval.v_list == (*typ2).vval.v_list,
                || tv_list_equal((*typ1).vval.v_list, (*typ2).vval.v_list, ic),
                c"E691: Can only compare List with List",
                c"E692: Invalid operation for List",
            ) {
                Some(n) => n,
                None => return FAIL,
            }
        } else if t1 == VAR_DICT || t2 == VAR_DICT {
            match compare_container(
                typ1,
                op,
                same_type,
                || (*typ1).vval.v_dict == (*typ2).vval.v_dict,
                || tv_dict_equal((*typ1).vval.v_dict, (*typ2).vval.v_dict, ic),
                c"E735: Can only compare Dictionary with Dictionary",
                c"E736: Invalid operation for Dictionary",
            ) {
                Some(n) => n,
                None => return FAIL,
            }
        } else if tv_is_func(*typ1) || tv_is_func(*typ2) {
            if op != EXPR_EQUAL && op != EXPR_NEQUAL && !type_is {
                emsg(gettext(c"E694: Invalid operation for Funcrefs".as_ptr()));
                tv_clear(typ1);
                return FAIL;
            }
            let equal = if t1 == VAR_PARTIAL && (*typ1).vval.v_partial.is_null()
                || t2 == VAR_PARTIAL && (*typ2).vval.v_partial.is_null()
            {
                // A null partial is only ever equal to another null one, and
                // both union members are pointers.
                (*typ1).vval.v_partial == (*typ2).vval.v_partial
            } else if !type_is || (t1 == VAR_FUNC && t2 == VAR_FUNC) {
                // `is` on two plain Funcrefs falls back to comparing names:
                // there is no object for them to be identical to.
                tv_equal(typ1, typ2, ic)
            } else if t1 == VAR_PARTIAL && t2 == VAR_PARTIAL {
                (*typ1).vval.v_partial == (*typ2).vval.v_partial
            } else {
                false
            };
            varnumber_T::from(equal != (op == EXPR_NEQUAL || op == EXPR_ISNOT))
        } else if (t1 == VAR_FLOAT || t2 == VAR_FLOAT) && op != EXPR_MATCH && op != EXPR_NOMATCH {
            let f1: float_T = tv_get_float(typ1);
            let f2: float_T = tv_get_float(typ2);
            // Not `from_ordering`: NaN is unordered, so every operator has to
            // ask the float comparison itself.
            varnumber_T::from(match op {
                EXPR_IS | EXPR_EQUAL => f1 == f2,
                EXPR_ISNOT | EXPR_NEQUAL => f1 != f2,
                EXPR_GREATER => f1 > f2,
                EXPR_GEQUAL => f1 >= f2,
                EXPR_SMALLER => f1 < f2,
                EXPR_SEQUAL => f1 <= f2,
                _ => false,
            })
        } else if (t1 == VAR_NUMBER || t2 == VAR_NUMBER) && op != EXPR_MATCH && op != EXPR_NOMATCH {
            // Both coercions happen, in this order, whatever the operator —
            // each may report its own error.
            let a = tv_get_number(typ1);
            let b = tv_get_number(typ2);
            from_ordering(op, if a < b { -1 } else { c_int::from(a > b) })
        } else {
            let mut buf1: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
            let mut buf2: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
            let s1 = tv_get_string_buf(typ1, buf1.as_mut_ptr());
            let s2 = tv_get_string_buf(typ2, buf2.as_mut_ptr());
            if op == EXPR_MATCH || op == EXPR_NOMATCH {
                // The pattern is the right-hand side and the subject the left.
                varnumber_T::from((pattern_match(s2, s1, ic) != 0) == (op == EXPR_MATCH))
            } else {
                from_ordering(op, mb_strcmp_ic(ic, s1, s2))
            }
        };

        tv_clear(typ1);
        (*typ1).v_type = VAR_NUMBER;
        (*typ1).vval.v_number = answer;
        OK
    }
}
