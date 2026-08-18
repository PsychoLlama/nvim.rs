//! What the arithmetic levels do once both operands are in hand.
//!
//! Vimscript's Number is a 64-bit two's-complement integer and its
//! arithmetic **wraps**; the C original relies on that and reports nothing.
//! Every operator here therefore uses Rust's `wrapping_*`, which is the
//! same answer without the debug-build abort the transpile had.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr::{copy, copy_nonoverlapping};

use crate::eval::typval::{
    tv_blob_alloc, tv_blob_len, tv_blob_set_ret, tv_clear, tv_get_number_chk, tv_get_string_buf,
    tv_get_string_buf_chk, tv_list_concat,
};
use crate::eval::{FAIL, INT_MAX, VARNUMBER_MAX, VARNUMBER_MIN};
use crate::garray::ga_grow;
use crate::memory::xrealloc;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::strings::concat_str;
use crate::types::{
    VAR_FLOAT, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, blob_T, float_T, typval_T,
    typval_vval_union, varnumber_T,
};
use ::libc::strlen;

/// The length of the scratch buffer `tv_get_string_buf` may render a Number
/// or a Float into. `NUMBUFLEN` in the C.
const NUMBUFLEN: usize = 65;

/// `n1 / n2`, with the two cases a machine divide cannot answer.
///
/// Division by zero yields the largest magnitude with the sign of the
/// numerator — and `VARNUMBER_MIN` for `0 / 0`, which upstream describes as
/// "similar to NaN". `VARNUMBER_MIN / -1` would be a positive number that
/// does not fit, and traps on x86; it answers `VARNUMBER_MAX`.
pub fn num_divide(n1: varnumber_T, n2: varnumber_T) -> varnumber_T {
    if n2 == 0 {
        if n1 == 0 {
            VARNUMBER_MIN as varnumber_T
        } else if n1 < 0 {
            -(VARNUMBER_MAX as varnumber_T)
        } else {
            VARNUMBER_MAX as varnumber_T
        }
    } else if n1 == VARNUMBER_MIN as varnumber_T && n2 == -1 {
        VARNUMBER_MAX as varnumber_T
    } else {
        n1 / n2
    }
}

/// `n1 % n2`, answering 0 for a zero divisor rather than reporting anything.
///
/// `VARNUMBER_MIN % -1` is mathematically 0 but traps on x86 and aborts a
/// debug build in Rust, so it goes through `wrapping_rem`. Upstream has no
/// guard for it — `num_divide`'s companion case is guarded and this one is
/// not.
pub fn num_modulus(n1: varnumber_T, n2: varnumber_T) -> varnumber_T {
    if n2 == 0 { 0 } else { n1.wrapping_rem(n2) }
}

/// `blob + blob`.
///
/// # Safety
/// Both operands must be Blobs, which is what `eval5` checked before
/// dispatching here.
pub(crate) unsafe fn eval_addblob(tv1: *mut typval_T, tv2: *mut typval_T) {
    unsafe {
        let b1: *const blob_T = (*tv1).vval.v_blob;
        let b2: *const blob_T = (*tv2).vval.v_blob;
        let b: *mut blob_T = tv_blob_alloc();
        let len1 = tv_blob_len(b1) as i64;
        let len2 = tv_blob_len(b2) as i64;
        let total = len1 + len2;

        // A result that would not fit a garray is silently dropped: the
        // answer is an empty Blob and nothing is reported.
        if (0..=i64::from(INT_MAX)).contains(&total) {
            ga_grow(&raw mut (*b).bv_ga, total as c_int);
            let dest = (*b).bv_ga.ga_data as *mut u8;
            // `b` was allocated a moment ago, so it cannot overlap either
            // source even when the two operands are the same Blob.
            if len1 > 0 {
                copy_nonoverlapping((*b1).bv_ga.ga_data as *const u8, dest, len1 as usize);
            }
            if len2 > 0 {
                copy_nonoverlapping(
                    (*b2).bv_ga.ga_data as *const u8,
                    dest.add(len1 as usize),
                    len2 as usize,
                );
            }
            (*b).bv_ga.ga_len = total as c_int;
        }
        tv_clear(tv1);
        tv_blob_set_ret(tv1, b);
    }
}

/// `list + list`. Clears both operands on failure, as every arithmetic
/// helper here does — the caller has already given up ownership.
///
/// # Safety
/// Both operands must be Lists.
pub(crate) unsafe fn eval_addlist(tv1: *mut typval_T, tv2: *mut typval_T) -> bool {
    unsafe {
        let mut joined = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if tv_list_concat((*tv1).vval.v_list, (*tv2).vval.v_list, &raw mut joined) == FAIL {
            tv_clear(tv1);
            tv_clear(tv2);
            return false;
        }
        tv_clear(tv1);
        *tv1 = joined;
        true
    }
}

/// Append `s2` to a String typval in place, reusing its allocation.
///
/// Answers false — leaving `tv1` alone — for anything that is not a String
/// with an allocation to grow, which is the caller's cue to build a fresh
/// one.
///
/// # Safety
/// `tv1` must be a valid typval and `s2` a NUL-terminated string that does
/// not point into `tv1`'s own allocation.
pub unsafe fn grow_string_tv(tv1: *mut typval_T, s2: *const c_char) -> bool {
    unsafe {
        if (*tv1).v_type != VAR_STRING || (*tv1).vval.v_string.is_null() {
            return false;
        }
        let len1 = strlen((*tv1).vval.v_string);
        let len2 = strlen(s2);
        let grown = xrealloc((*tv1).vval.v_string.cast(), len1 + len2 + 1) as *mut c_char;
        // The terminator moves with the bytes.
        copy(s2, grown.add(len1), len2 + 1);
        (*tv1).vval.v_string = grown;
        true
    }
}

/// `..` (and `.`): the string concatenation `eval5` performs.
///
/// # Safety
/// Both operands must be valid typvals the caller has given up ownership of.
pub(crate) unsafe fn eval_concat_str(tv1: *mut typval_T, tv2: *mut typval_T) -> bool {
    unsafe {
        let mut buf1: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        let mut buf2: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        let s1 = tv_get_string_buf(tv1, buf1.as_mut_ptr());
        let s2 = tv_get_string_buf_chk(tv2, buf2.as_mut_ptr());
        if s2.is_null() {
            tv_clear(tv1);
            tv_clear(tv2);
            return false;
        }
        if grow_string_tv(tv1, s2) {
            return true;
        }
        // `s1` may point into `buf1`, so build the result before clearing.
        let joined = concat_str(s1, s2);
        tv_clear(tv1);
        (*tv1).v_type = VAR_STRING;
        (*tv1).vval.v_string = joined;
        true
    }
}

/// `+` and `-` over Numbers and Floats.
///
/// # Safety
/// Both operands must be valid typvals the caller has given up ownership of.
pub(crate) unsafe fn eval_addsub_number(tv1: *mut typval_T, tv2: *mut typval_T, op: u8) -> bool {
    unsafe {
        let mut error = false;
        let mut n1: varnumber_T = 0;
        let mut n2: varnumber_T = 0;
        let mut f1: float_T = 0.0;
        let mut f2: float_T = 0.0;

        if (*tv1).v_type == VAR_FLOAT {
            f1 = (*tv1).vval.v_float;
        } else {
            n1 = tv_get_number_chk(tv1, &raw mut error);
            if error {
                // Only reachable for "list + non-list" or "blob + non-blob":
                // for anything else the caller returned before evaluating the
                // second operand.
                tv_clear(tv1);
                tv_clear(tv2);
                return false;
            }
            if (*tv2).v_type == VAR_FLOAT {
                f1 = n1 as float_T;
            }
        }
        if (*tv2).v_type == VAR_FLOAT {
            f2 = (*tv2).vval.v_float;
        } else {
            n2 = tv_get_number_chk(tv2, &raw mut error);
            if error {
                tv_clear(tv1);
                tv_clear(tv2);
                return false;
            }
            if (*tv1).v_type == VAR_FLOAT {
                f2 = n2 as float_T;
            }
        }
        tv_clear(tv1);

        // Deliberately read *after* the clear, as upstream does: `tv_clear`
        // leaves a Float's tag alone (there is nothing to free), so this
        // still sees a Float on the left, but a List or Blob that was
        // rejected above has become VAR_UNKNOWN.
        if (*tv1).v_type == VAR_FLOAT || (*tv2).v_type == VAR_FLOAT {
            (*tv1).v_type = VAR_FLOAT;
            (*tv1).vval.v_float = if op == b'+' { f1 + f2 } else { f1 - f2 };
        } else {
            (*tv1).v_type = VAR_NUMBER;
            (*tv1).vval.v_number = if op == b'+' {
                n1.wrapping_add(n2)
            } else {
                n1.wrapping_sub(n2)
            };
        }
        true
    }
}

/// `*`, `/` and `%` over Numbers and Floats.
///
/// # Safety
/// Both operands must be valid typvals the caller has given up ownership of.
pub(crate) unsafe fn eval_multdiv_number(tv1: *mut typval_T, tv2: *mut typval_T, op: u8) -> bool {
    unsafe {
        let mut error = false;
        let mut n1: varnumber_T = 0;
        let mut n2: varnumber_T = 0;
        let mut f1: float_T = 0.0;
        let mut f2: float_T = 0.0;
        let mut use_float = (*tv1).v_type == VAR_FLOAT;

        if use_float {
            f1 = (*tv1).vval.v_float;
        } else {
            n1 = tv_get_number_chk(tv1, &raw mut error);
        }
        // Unlike the additive path this clears the left operand before
        // looking at the error, and clears the right one only on the branch
        // that read it.
        tv_clear(tv1);
        if error {
            tv_clear(tv2);
            return false;
        }

        if (*tv2).v_type == VAR_FLOAT {
            if !use_float {
                f1 = n1 as float_T;
                use_float = true;
            }
            f2 = (*tv2).vval.v_float;
        } else {
            n2 = tv_get_number_chk(tv2, &raw mut error);
            tv_clear(tv2);
            if error {
                return false;
            }
            if use_float {
                f2 = n2 as float_T;
            }
        }

        if use_float {
            let result = match op {
                b'*' => f1 * f2,
                b'/' if f2 == 0.0 => {
                    // A Float divided by zero answers an infinity of the
                    // numerator's sign, and NaN for 0.0 / 0.0.
                    if f1 == 0.0 {
                        float_T::NAN
                    } else if f1 > 0.0 {
                        float_T::INFINITY
                    } else {
                        float_T::NEG_INFINITY
                    }
                }
                b'/' => f1 / f2,
                _ => {
                    emsg(gettext(c"E804: Cannot use '%' with Float".as_ptr()));
                    return false;
                }
            };
            (*tv1).v_type = VAR_FLOAT;
            (*tv1).vval.v_float = result;
        } else {
            (*tv1).v_type = VAR_NUMBER;
            (*tv1).vval.v_number = match op {
                b'*' => n1.wrapping_mul(n2),
                b'/' => num_divide(n1, n2),
                _ => num_modulus(n1, n2),
            };
        }
        true
    }
}
