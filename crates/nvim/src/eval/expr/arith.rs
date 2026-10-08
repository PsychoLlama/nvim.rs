//! What the arithmetic levels do once both operands are in hand.
//!
//! Vimscript's Number is a 64-bit two's-complement integer and its
//! arithmetic **wraps**; the C original relies on that and reports nothing.
//! Every operator here therefore uses Rust's `wrapping_*`, which is the
//! same answer without the debug-build abort the transpile had.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::typval::TV_INITIAL_VALUE;
use core::ffi::c_int;

use crate::eval::typval::{
    NumBuf, blob_bytes, list_concat_values, tv_blob_alloc, tv_blob_set_ret, tv_clear,
    tv_get_number_chk,
};
use crate::eval::{VARNUMBER_MAX, VARNUMBER_MIN};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::types::{Float, TypVal, VAR_FLOAT, VarNumber};

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
///
/// The answer is a blob of this call's own, so it overlaps neither operand
/// even when the two are the same blob.
pub(crate) fn eval_addblob(tv1: &mut TypVal, tv2: &TypVal) {
    let (b1, b2) = (blob_bytes(tv1.blob_ref()), blob_bytes(tv2.blob_ref()));
    let mut held = tv_blob_alloc();
    let total = b1.len() + b2.len();

    // A result that would not fit a garray is silently dropped: the
    // answer is an empty Blob and nothing is reported.
    if c_int::try_from(total).is_ok() {
        let room = held.claim(total);
        room[..b1.len()].copy_from_slice(b1);
        room[b1.len()..].copy_from_slice(b2);
    }
    tv_clear(tv1);
    tv_blob_set_ret(tv1, Some(held));
}

/// `list + list`. Clears both operands on failure, as every arithmetic
/// helper here does — the caller has already given up ownership.
pub(crate) fn eval_addlist(tv1: &mut TypVal, tv2: &mut TypVal) -> bool {
    let mut joined = TV_INITIAL_VALUE;
    if list_concat_values(tv1, tv2, &mut joined).is_err() {
        tv_clear(tv1);
        tv_clear(tv2);
        return false;
    }
    tv_clear(tv1);
    *tv1 = joined;
    true
}

/// `..` (and `.`): the string concatenation `eval5` performs.
///
/// `eval5` has already checked that `tv1` has a string form, so reading it
/// reports nothing; `tv2` may still be refused.
pub(crate) fn eval_concat_str(tv1: &mut TypVal, tv2: &mut TypVal) -> bool {
    let mut buf2 = NumBuf::new();
    let Some(s2) = buf2.string_chk(tv2) else {
        tv_clear(tv1);
        tv_clear(tv2);
        return false;
    };
    // A String with an allocation grows in place.
    if tv1.append_to_string(s2.to_bytes()) {
        return true;
    }
    let mut buf1 = NumBuf::new();
    let s1 = buf1.string(tv1).to_bytes();
    let mut joined = Vec::with_capacity(s1.len() + s2.to_bytes().len() + 1);
    joined.extend_from_slice(s1);
    joined.extend_from_slice(s2.to_bytes());
    tv_clear(tv1);
    tv1.write_string(Some(joined.into()));
    true
}

/// `+` and `-` over Numbers and Floats.
pub(crate) fn eval_addsub_number(tv1: &mut TypVal, tv2: &mut TypVal, op: u8) -> bool {
    let mut n1: VarNumber = 0;
    let mut n2: VarNumber = 0;
    let mut f1: Float = 0.0;
    let mut f2: Float = 0.0;

    if tv1.v_type() == VAR_FLOAT {
        // The kind says the value holds a Float.
        f1 = tv1.float_or_zero();
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
        if tv2.v_type() == VAR_FLOAT {
            f1 = n1 as Float;
        }
    }
    if tv2.v_type() == VAR_FLOAT {
        // As above, for the right operand.
        f2 = tv2.float_or_zero();
    } else {
        let Ok(right) = tv_get_number_chk(tv2) else {
            tv_clear(tv1);
            tv_clear(tv2);
            return false;
        };
        n2 = right;
        if tv1.v_type() == VAR_FLOAT {
            f2 = n2 as Float;
        }
    }
    // Which arithmetic to do is decided *before* the left operand is
    // cleared.  Upstream reads the tags afterwards, which was the same
    // answer while `tv_clear` left the kind behind and is not now: a
    // cleared value is `Unknown`, so `1.234 - 8` would take the integer
    // branch and answer -8.
    let use_float = tv1.v_type() == VAR_FLOAT || tv2.v_type() == VAR_FLOAT;
    tv_clear(tv1);

    if use_float {
        tv1.write_float(if op == b'+' { f1 + f2 } else { f1 - f2 });
    } else {
        tv1.write_number(if op == b'+' {
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
    let mut use_float = tv1.v_type() == VAR_FLOAT;

    if use_float {
        // The kind says the value holds a Float.
        f1 = tv1.float_or_zero();
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

    if tv2.v_type() == VAR_FLOAT {
        if !use_float {
            f1 = n1 as Float;
            use_float = true;
        }
        // As above, for the right operand.
        f2 = tv2.float_or_zero();
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
        tv1.write_float(result);
    } else {
        tv1.write_number(match op {
            b'*' => n1.wrapping_mul(n2),
            b'/' => num_divide(n1, n2),
            _ => num_modulus(n1, n2),
        });
    }
    true
}
