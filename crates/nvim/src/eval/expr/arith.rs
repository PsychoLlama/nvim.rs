//! What the arithmetic levels do once both operands are in hand.
//!
//! Vimscript's Number is a 64-bit two's-complement integer and its
//! arithmetic **wraps**; the C original relies on that and reports nothing.
//! Every operator here therefore uses Rust's `wrapping_*`, which is the
//! same answer without the debug-build abort the transpile had.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::eval::typval::TV_INITIAL_VALUE;
use core::ffi::{c_char, c_int};
use core::ptr::{copy, copy_nonoverlapping};

use crate::eval::typval::{
    NumBuf, tv_blob_alloc, tv_blob_len, tv_blob_set_ret, tv_clear, tv_get_number_chk,
    tv_list_concat,
};
use crate::eval::{INT_MAX, Tv, VARNUMBER_MAX, VARNUMBER_MIN};
use crate::garray::ga_grow;
use crate::memory::xrealloc;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::strings::concat_str;
use crate::types::{Blob, Float, TypVal, VAR_FLOAT, VAR_STRING, VarNumber};

/// `n1 / n2`, with the two cases a machine divide cannot answer.
///
/// Division by zero yields the largest magnitude with the sign of the
/// numerator — and `VARNUMBER_MIN` for `0 / 0`, which upstream describes as
/// "similar to NaN". `VARNUMBER_MIN / -1` would be a positive number that
/// does not fit, and traps on x86; it answers `VARNUMBER_MAX`.
pub(crate) fn num_divide(n1: VarNumber, n2: VarNumber) -> VarNumber {
    if n2 == 0 {
        if n1 == 0 {
            VARNUMBER_MIN as VarNumber
        } else if n1 < 0 {
            -(VARNUMBER_MAX as VarNumber)
        } else {
            VARNUMBER_MAX as VarNumber
        }
    } else if n1 == VARNUMBER_MIN as VarNumber && n2 == -1 {
        VARNUMBER_MAX as VarNumber
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
pub(crate) fn num_modulus(n1: VarNumber, n2: VarNumber) -> VarNumber {
    if n2 == 0 { 0 } else { n1.wrapping_rem(n2) }
}

/// `blob + blob`.
pub(crate) fn eval_addblob(tv1: &mut TypVal, tv2: &mut TypVal) {
    // SAFETY: the caller's promise -- both operands are Blobs, so each
    // union holds a live `Blob`, and `b` is a Blob of this call's own.
    let b1: *const Blob = (*tv1).blob_or_null();
    let b2: *const Blob = (*tv2).blob_or_null();
    let b: *mut Blob = tv_blob_alloc();
    let len1 = unsafe { tv_blob_len(b1) } as i64;
    let len2 = unsafe { tv_blob_len(b2) } as i64;
    let total = len1 + len2;

    // A result that would not fit a garray is silently dropped: the
    // answer is an empty Blob and nothing is reported.
    if (0..=i64::from(INT_MAX)).contains(&total) {
        // SAFETY: as above; `ga_grow` sized `bv_ga` for `total` bytes and
        // `b` was allocated a moment ago, so it cannot overlap either
        // source even when the two operands are the same Blob.
        let dest = unsafe {
            ga_grow(&raw mut (*b).bv_ga, total as c_int);
            (*b).bv_ga.ga_data as *mut u8
        };
        if len1 > 0 {
            let src = unsafe { (*b1).bv_ga.ga_data } as *const u8;
            unsafe { copy_nonoverlapping(src, dest, len1 as usize) };
        }
        if len2 > 0 {
            let src = unsafe { (*b2).bv_ga.ga_data } as *const u8;
            let at = unsafe { dest.add(len1 as usize) };
            unsafe { copy_nonoverlapping(src, at, len2 as usize) };
        }
        unsafe { (*b).bv_ga.ga_len = total as c_int };
    }
    tv_clear(tv1);
    unsafe { tv_blob_set_ret(tv1, b) };
}

/// `list + list`. Clears both operands on failure, as every arithmetic
/// helper here does — the caller has already given up ownership.
pub(crate) fn eval_addlist(tv1: &mut TypVal, tv2: &mut TypVal) -> bool {
    let mut joined = TV_INITIAL_VALUE;
    // SAFETY: the caller's promise -- both operands are Lists, so each
    // union holds a live `List`, and `joined` is this frame's own.
    let (l1, l2) = ((*tv1).list_or_null(), (*tv2).list_or_null());
    if unsafe { tv_list_concat(l1, l2, &mut joined) }.is_err() {
        tv_clear(tv1);
        tv_clear(tv2);
        return false;
    }
    tv_clear(tv1);
    *tv1 = joined;
    true
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
pub(crate) unsafe fn grow_string_tv(tv1: &mut TypVal, s2: *const c_char) -> bool {
    // SAFETY: the caller's promise -- `tv1` is a valid typval.
    let mut one = unsafe { Tv::new(tv1) };
    let old = one.string_or_null();
    if one.v_type() != VAR_STRING || old.is_null() {
        return false;
    }
    // SAFETY: `old` is that String's allocation and `s2` is NUL-terminated
    // and outside it, so the copy does not overlap what `xrealloc` moved.
    let len1 = unsafe { cstr::bytes_at(old) }.len();
    let len2 = unsafe { cstr::bytes_at(s2) }.len();
    let grown = unsafe { xrealloc(old.cast(), len1 + len2 + 1) } as *mut c_char;
    // The terminator moves with the bytes.
    unsafe { copy(s2, grown.add(len1), len2 + 1) };
    one.write_string(grown);
    true
}

/// `..` (and `.`): the string concatenation `eval5` performs.
pub(crate) fn eval_concat_str(tv1: &mut TypVal, tv2: &mut TypVal) -> bool {
    let mut buf1 = NumBuf::new();
    let mut buf2 = NumBuf::new();
    // SAFETY: the caller's promise -- both operands are valid typvals, and
    // the two scratch buffers are this frame's own.
    let mut one = unsafe { Tv::new(tv1) };
    let s1 = buf1.string_ptr(tv1);
    let s2 = buf2.string_ptr_chk(tv2);
    if s2.is_null() {
        tv_clear(tv1);
        tv_clear(tv2);
        return false;
    }
    // `s2` is `buf2` or `tv2`'s own allocation, never `tv1`'s.
    if unsafe { grow_string_tv(tv1, s2) } {
        return true;
    }
    // `s1` may point into `buf1`, so build the result before clearing.
    let joined = unsafe { concat_str(s1, s2) };
    tv_clear(tv1);
    one.write_string(joined);
    true
}

/// `+` and `-` over Numbers and Floats.
pub(crate) fn eval_addsub_number(tv1: &mut TypVal, tv2: &mut TypVal, op: u8) -> bool {
    let mut n1: VarNumber = 0;
    let mut n2: VarNumber = 0;
    let mut f1: Float = 0.0;
    let mut f2: Float = 0.0;

    // SAFETY: the caller's promise -- both operands are valid typvals.
    let (mut one, two) = unsafe { (Tv::new(tv1), Tv::new(tv2)) };

    if one.v_type() == VAR_FLOAT {
        // SAFETY: the kind says the value holds a Float.
        f1 = one.float_or_zero();
    } else {
        let Ok(left) = tv_get_number_chk(tv1) else {
            // Only reachable for "list + non-list" or "blob + non-blob":
            // for anything else the caller returned before evaluating the
            // second operand.
            tv_clear(tv1);
            tv_clear(tv2);
            return false;
        };
        n1 = left;
        if two.v_type() == VAR_FLOAT {
            f1 = n1 as Float;
        }
    }
    if two.v_type() == VAR_FLOAT {
        // SAFETY: as above, for the right operand.
        f2 = two.float_or_zero();
    } else {
        let Ok(right) = tv_get_number_chk(tv2) else {
            tv_clear(tv1);
            tv_clear(tv2);
            return false;
        };
        n2 = right;
        if one.v_type() == VAR_FLOAT {
            f2 = n2 as Float;
        }
    }
    // Which arithmetic to do is decided *before* the left operand is
    // cleared.  Upstream reads the tags afterwards, which was the same
    // answer while `tv_clear` left the kind behind and is not now: a
    // cleared value is `Unknown`, so `1.234 - 8` would take the integer
    // branch and answer -8.
    let use_float = one.v_type() == VAR_FLOAT || two.v_type() == VAR_FLOAT;
    tv_clear(tv1);

    if use_float {
        one.write_float(if op == b'+' { f1 + f2 } else { f1 - f2 });
    } else {
        one.write_number(if op == b'+' {
            n1.wrapping_add(n2)
        } else {
            n1.wrapping_sub(n2)
        });
    }
    true
}

/// `*`, `/` and `%` over Numbers and Floats.
pub(crate) fn eval_multdiv_number(tv1: &mut TypVal, tv2: &mut TypVal, op: u8) -> bool {
    let mut error = false;
    let mut n1: VarNumber = 0;
    let mut n2: VarNumber = 0;
    let mut f1: Float = 0.0;
    let mut f2: Float = 0.0;
    // SAFETY: the caller's promise -- both operands are valid typvals.
    let (mut one, two) = unsafe { (Tv::new(tv1), Tv::new(tv2)) };
    let mut use_float = one.v_type() == VAR_FLOAT;

    if use_float {
        // SAFETY: the kind says the value holds a Float.
        f1 = one.float_or_zero();
    } else {
        n1 = tv_get_number_chk(tv1).unwrap_or_else(|_| {
            error = true;
            0
        });
    }
    // Unlike the additive path this clears the left operand before
    // looking at the error, and clears the right one only on the branch
    // that read it.
    tv_clear(tv1);
    if error {
        tv_clear(tv2);
        return false;
    }

    if two.v_type() == VAR_FLOAT {
        if !use_float {
            f1 = n1 as Float;
            use_float = true;
        }
        // SAFETY: as above, for the right operand.
        f2 = two.float_or_zero();
    } else {
        let read = tv_get_number_chk(tv2);
        tv_clear(tv2);
        let Ok(right) = read else {
            return false;
        };
        n2 = right;
        if use_float {
            f2 = n2 as Float;
        }
    }

    if use_float {
        let result = match op {
            b'*' => f1 * f2,
            b'/' if f2 == 0.0 => {
                // A Float divided by zero answers an infinity of the
                // numerator's sign, and NaN for 0.0 / 0.0.
                if f1 == 0.0 {
                    Float::NAN
                } else if f1 > 0.0 {
                    Float::INFINITY
                } else {
                    Float::NEG_INFINITY
                }
            }
            b'/' => f1 / f2,
            _ => {
                emsg(gettext(c"E804: Cannot use '%' with Float"));
                return false;
            }
        };
        one.write_float(result);
    } else {
        one.write_number(match op {
            b'*' => n1.wrapping_mul(n2),
            b'/' => num_divide(n1, n2),
            _ => num_modulus(n1, n2),
        });
    }
    true
}
