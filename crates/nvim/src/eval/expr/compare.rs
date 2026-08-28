//! Comparing two values, for every operator and every pair of types.
//!
//! `typval_compare` is one long else-if chain over the *pair* of types, and
//! the order of its arms is the semantics: a Blob on either side makes the
//! comparison a Blob comparison, then List, then Dict, then Funcref, then
//! Float, then Number, and only what is left compares as a String.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_ushort};

use crate::eval::typval::{
    tv_blob_equal, tv_clear, tv_dict_equal, tv_equal, tv_get_float, tv_get_number,
    tv_get_string_buf, tv_is_func, tv_list_equal,
};
use crate::eval::{
    _ISalnum, Cur, EXPR_EQUAL, EXPR_GEQUAL, EXPR_GREATER, EXPR_IS, EXPR_ISNOT, EXPR_MATCH,
    EXPR_NEQUAL, EXPR_NOMATCH, EXPR_SEQUAL, EXPR_SMALLER, EXPR_UNKNOWN, Tv, e_invalblob,
    partial_name, pattern_match,
};
use crate::mbyte::mb_strcmp_ic;
use crate::message::emsg;
use crate::os::cshim::{__ctype_b_loc, gettext};
use crate::types::{
    FAIL, NUL, OK, VAR_BLOB, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL,
    dict_T, exprtype_T, float_T, typval_T, varnumber_T,
};
use ::libc::strcmp;

/// The scratch a Number or Float is rendered into for a String comparison.
const NUMBUFLEN: usize = 65;

/// `isalnum` in the process locale, which is what decides whether `is` and
/// `isnot` stand as whole words. nvim calls `setlocale(LC_ALL, "")` at
/// startup, so this is deliberately not the ASCII test.
fn isalnum_locale(c: u8) -> bool {
    // SAFETY: `__ctype_b_loc` yields a table valid over the whole byte range.
    unsafe { *(*__ctype_b_loc()).offset(c as isize) & _ISalnum as c_ushort != 0 }
}

/// Recognise a comparison operator, answering it and how many bytes it took.
///
/// The second byte is read *inside* each arm, never before the `match`:
/// `eval5` may well have left the cursor on the terminating NUL, and the
/// first byte matching an operator character is the only thing that proves
/// there is a second one.
///
/// Safe because [`Cur`] carries the promise that its bytes are readable.
pub(crate) fn comparison_at(cur: Cur) -> (exprtype_T, c_int) {
    let next = || cur.at(1);
    match cur.byte() {
        b'=' => match next() {
            b'=' => (EXPR_EQUAL, 2),
            b'~' => (EXPR_MATCH, 2),
            _ => (EXPR_UNKNOWN, 2),
        },
        b'!' => match next() {
            b'=' => (EXPR_NEQUAL, 2),
            b'~' => (EXPR_NOMATCH, 2),
            _ => (EXPR_UNKNOWN, 2),
        },
        b'>' if next() == b'=' => (EXPR_GEQUAL, 2),
        b'>' => (EXPR_GREATER, 1),
        b'<' if next() == b'=' => (EXPR_SEQUAL, 2),
        b'<' => (EXPR_SMALLER, 1),
        b'i' if next() == b's' => {
            let len = if cur.at(2) == b'n' && cur.at(3) == b'o' && cur.at(4) == b't' {
                5
            } else {
                2
            };
            // `isnothing` is a name, not `isnot` followed by `hing`.
            let after = cur.at(len as usize);
            if !isalnum_locale(after) && after != b'_' {
                (if len == 2 { EXPR_IS } else { EXPR_ISNOT }, len)
            } else {
                (EXPR_UNKNOWN, 2)
            }
        }
        _ => (EXPR_UNKNOWN, 2),
    }
}

/// The name a Funcref or partial calls, with `""` read as "none".
///
/// # Safety
/// `tv` must be a `VAR_FUNC` or `VAR_PARTIAL` typval.
unsafe fn callable_name(tv: *mut typval_T) -> *mut c_char {
    // SAFETY: the caller's promise -- the tag says which union member holds
    // the callable, and a partial is null or live.
    let name = if unsafe { (*tv).v_type } == VAR_FUNC {
        unsafe { (*tv).vval.v_string }
    } else {
        unsafe { partial_name((*tv).vval.v_partial) }
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
pub(crate) unsafe fn func_equal(tv1: *mut typval_T, tv2: *mut typval_T, ic: bool) -> bool {
    let s1 = unsafe { callable_name(tv1) };
    let s2 = unsafe { callable_name(tv2) };
    if s1.is_null() || s2.is_null() {
        if s1 != s2 {
            return false;
        }
    } else if unsafe { strcmp(s1, s2) } != 0 {
        return false;
    }

    // A plain Funcref carries neither a bound dictionary nor arguments.
    let dict_of = |tv: *mut typval_T| -> *mut dict_T {
        if unsafe { (*tv).v_type } == VAR_FUNC {
            core::ptr::null_mut()
        } else {
            unsafe { (*(*tv).vval.v_partial).pt_dict }
        }
    };
    let d1 = dict_of(tv1);
    let d2 = dict_of(tv2);
    if d1.is_null() || d2.is_null() {
        if d1 != d2 {
            return false;
        }
    } else if !unsafe { tv_dict_equal(d1, d2, ic) } {
        return false;
    }

    let argc_of = |tv: *mut typval_T| -> c_int {
        if unsafe { (*tv).v_type } == VAR_FUNC {
            0
        } else {
            unsafe { (*(*tv).vval.v_partial).pt_argc }
        }
    };
    let argc = argc_of(tv1);
    if argc != argc_of(tv2) {
        return false;
    }
    if argc == 0 {
        // Neither side has an argument vector to compare -- and a plain
        // Funcref's union holds its *name*, so `v_partial` must not be read
        // at all here. Upstream reaches the reads only from inside the loop
        // body, which a zero count never enters.
        return true;
    }
    // SAFETY: the count is non-zero, so both unions hold a partial with an
    // argument vector of `argc` values.
    let (p1, p2) = unsafe { ((*tv1).vval.v_partial, (*tv2).vval.v_partial) };
    let (a1, a2) = unsafe { ((*p1).pt_argv, (*p2).pt_argv) };
    (0..argc).all(|i| unsafe { tv_equal(a1.offset(i as isize), a2.offset(i as isize), ic) })
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
        let message = if !same_type { wrong_type } else { wrong_op };
        // SAFETY: a message constant is a NUL-terminated literal.
        unsafe { emsg(gettext(message.as_ptr())) };
        unsafe { tv_clear(typ1) };
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
pub(crate) unsafe fn typval_compare(
    typ1: *mut typval_T,
    typ2: *mut typval_T,
    op: exprtype_T,
    ic: bool,
) -> c_int {
    let type_is = op == EXPR_IS || op == EXPR_ISNOT;
    let (t1, t2) = (unsafe { (*typ1).v_type }, unsafe { (*typ2).v_type });
    let same_type = t1 == t2;

    let answer: varnumber_T = if type_is && !same_type {
        // `is` between two different types is simply false.
        varnumber_T::from(op == EXPR_ISNOT)
    } else if t1 == VAR_BLOB || t2 == VAR_BLOB {
        // SAFETY: `same_type` has held before either closure runs, so the
        // union member each reads is the one the tag names.
        let same = || unsafe { (*typ1).vval.v_blob == (*typ2).vval.v_blob };
        let eq = || unsafe { tv_blob_equal((*typ1).vval.v_blob, (*typ2).vval.v_blob) };
        let wrong_type = c"E977: Can only compare Blob with Blob";
        // SAFETY: a message constant is a NUL-terminated literal.
        let wrong_op = unsafe { CStr::from_ptr((&raw const e_invalblob).cast()) };
        let cmp = unsafe { compare_container(typ1, op, same_type, same, eq, wrong_type, wrong_op) };
        match cmp {
            Some(n) => n,
            None => return FAIL,
        }
    } else if t1 == VAR_LIST || t2 == VAR_LIST {
        // SAFETY: as the Blob arm.
        let same = || unsafe { (*typ1).vval.v_list == (*typ2).vval.v_list };
        let eq = || unsafe { tv_list_equal((*typ1).vval.v_list, (*typ2).vval.v_list, ic) };
        let wrong_type = c"E691: Can only compare List with List";
        let wrong_op = c"E692: Invalid operation for List";
        let cmp = unsafe { compare_container(typ1, op, same_type, same, eq, wrong_type, wrong_op) };
        match cmp {
            Some(n) => n,
            None => return FAIL,
        }
    } else if t1 == VAR_DICT || t2 == VAR_DICT {
        // SAFETY: as the Blob arm.
        let same = || unsafe { (*typ1).vval.v_dict == (*typ2).vval.v_dict };
        let eq = || unsafe { tv_dict_equal((*typ1).vval.v_dict, (*typ2).vval.v_dict, ic) };
        let wrong_type = c"E735: Can only compare Dictionary with Dictionary";
        let wrong_op = c"E736: Invalid operation for Dictionary";
        let cmp = unsafe { compare_container(typ1, op, same_type, same, eq, wrong_type, wrong_op) };
        match cmp {
            Some(n) => n,
            None => return FAIL,
        }
    } else if tv_is_func(unsafe { *typ1 }) || tv_is_func(unsafe { *typ2 }) {
        if op != EXPR_EQUAL && op != EXPR_NEQUAL && !type_is {
            unsafe { emsg(gettext(c"E694: Invalid operation for Funcrefs".as_ptr())) };
            unsafe { tv_clear(typ1) };
            return FAIL;
        }
        let equal = if t1 == VAR_PARTIAL && unsafe { (*typ1).vval.v_partial }.is_null()
            || t2 == VAR_PARTIAL && unsafe { (*typ2).vval.v_partial }.is_null()
        {
            // A null partial is only ever equal to another null one, and
            // both union members are pointers.
            unsafe { (*typ1).vval.v_partial == (*typ2).vval.v_partial }
        } else if !type_is || (t1 == VAR_FUNC && t2 == VAR_FUNC) {
            // `is` on two plain Funcrefs falls back to comparing names:
            // there is no object for them to be identical to.
            unsafe { tv_equal(typ1, typ2, ic) }
        } else if t1 == VAR_PARTIAL && t2 == VAR_PARTIAL {
            unsafe { (*typ1).vval.v_partial == (*typ2).vval.v_partial }
        } else {
            false
        };
        varnumber_T::from(equal != (op == EXPR_NEQUAL || op == EXPR_ISNOT))
    } else if (t1 == VAR_FLOAT || t2 == VAR_FLOAT) && op != EXPR_MATCH && op != EXPR_NOMATCH {
        let f1: float_T = unsafe { tv_get_float(typ1) };
        let f2: float_T = unsafe { tv_get_float(typ2) };
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
        let a = unsafe { tv_get_number(typ1) };
        let b = unsafe { tv_get_number(typ2) };
        from_ordering(op, if a < b { -1 } else { c_int::from(a > b) })
    } else {
        let mut buf1: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        let mut buf2: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        let s1 = unsafe { tv_get_string_buf(typ1, buf1.as_mut_ptr()) };
        let s2 = unsafe { tv_get_string_buf(typ2, buf2.as_mut_ptr()) };
        if op == EXPR_MATCH || op == EXPR_NOMATCH {
            // The pattern is the right-hand side and the subject the left.
            varnumber_T::from(unsafe { pattern_match(s2, s1, ic) } == (op == EXPR_MATCH))
        } else {
            from_ordering(op, unsafe { mb_strcmp_ic(ic, s1, s2) })
        }
    };

    // SAFETY: the caller's promise -- `typ1` is a valid typval.
    let mut one = unsafe { Tv::new(typ1) };
    unsafe { tv_clear(typ1) };
    one.v_type = VAR_NUMBER;
    one.vval.v_number = answer;
    OK
}
