//! Compound assignment for vimscript values: the `+=`, `-=`, `*=`, `/=`, `%=`
//! and `.=`/`..=` half of `:let`.
//!
//! [`eexe_mod_op`] dispatches on the *left* operand's type; each arm decides
//! for itself whether the operator and the right operand make sense, and
//! answers `FAIL` when they do not so the caller reports one error message.
//!
//! The entry points take raw pointers rather than references because the two
//! operands can be the same object: `:let l[0:1] += l[0:1]` reaches here from
//! `tv_list_assign_range` with `tv1` and `tv2` aliasing, so a `&mut`/`&` pair
//! would be unsound.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::typval::{NumBuf, tv_clear, tv_get_number, tv_list_extend};
use crate::eval::{Tv, grow_string_tv, num_divide, num_modulus};
use crate::garray::ga_grow;
use crate::os::cshim::memmove;
use crate::strings::concat_str;
use crate::types::{
    Failed, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_SPECIAL,
    VAR_STRING, VAR_UNKNOWN, blob_T, float_T, listitem_T, typval_T, uint8_t, varnumber_T,
};
use ::libc::abort;
use core::ffi::{CStr, c_char};

/// Is `op` one of the arithmetic operators, i.e. everything but concatenation?
///
/// Upstream asks `vim_strchr("+-*/%", op)`, which answers NULL for a NUL byte
/// (`vim_strchr` rejects non-positive characters before reaching `strchr`), so
/// an empty operator concatenates.
fn is_arithmetic(op: u8) -> bool {
    matches!(op, b'+' | b'-' | b'*' | b'/' | b'%')
}

/// Fold `rhs` into `lhs` with a float operator. `%` and `.` never reach here;
/// any other unrecognised operator leaves `lhs` alone, as upstream's `switch`
/// with no default did.
fn float_op(lhs: float_T, op: u8, rhs: float_T) -> float_T {
    match op {
        b'+' => lhs + rhs,
        b'-' => lhs - rhs,
        b'*' => lhs * rhs,
        b'/' => lhs / rhs,
        _ => lhs,
    }
}

/// `blob1 += blob2`.
unsafe fn tv_op_blob(tv1: *mut typval_T, tv2: *const typval_T, op: u8) -> Result<(), Failed> {
    // SAFETY: the caller's obligation -- two initialised typvals, which may
    // alias, which is why they are held as pointers and never as references.
    let (mut lhs, rhs) = unsafe { (Tv::new(tv1), Tv::new(tv2.cast_mut())) };
    if op != b'+' || rhs.v_type != VAR_BLOB {
        return Err(Failed);
    }
    // SAFETY: `VAR_BLOB` says `v_blob` is the union's live arm.
    let b2: *mut blob_T = unsafe { rhs.vval.v_blob };
    if b2.is_null() {
        return Ok(());
    }
    // SAFETY: `tv1` reached this helper under `VAR_BLOB` too.
    let b1: *mut blob_T = unsafe { lhs.vval.v_blob };
    if b1.is_null() {
        // Appending to an unallocated blob shares the right-hand one
        // rather than copying it.
        lhs.vval.v_blob = b2;
        // SAFETY: `b2` is the live Blob the right-hand typval holds.
        unsafe { (*b2).bv_refcount.retain() };
        return Ok(());
    }
    // SAFETY: `b2` is live.
    let len = unsafe { (*b2).bv_ga.ga_len };
    if len > 0 {
        // SAFETY (every region below): both Blobs are live, `ga` is the
        // left-hand one's own array, and `ga_grow` has made room for `len`
        // bytes past `ga_len` before the move. `len > 0`, so the narrowing
        // cannot lose a sign.
        let ga = unsafe { &raw mut (*b1).bv_ga };
        unsafe { ga_grow(ga, len) };
        let at = unsafe { (*ga).ga_len } as isize;
        let end = unsafe { (*ga).ga_data.cast::<uint8_t>().offset(at) };
        let n = len.unsigned_abs() as usize;
        unsafe { memmove(end.cast(), (*b2).bv_ga.ga_data, n) };
        unsafe { (*ga).ga_len += len };
    }
    Ok(())
}

/// `list1 += list2`.
unsafe fn tv_op_list(tv1: *mut typval_T, tv2: *const typval_T, op: u8) -> Result<(), Failed> {
    // SAFETY: the caller's obligation -- two initialised typvals, which may
    // alias.
    let (mut lhs, rhs) = unsafe { (Tv::new(tv1), Tv::new(tv2.cast_mut())) };
    if op != b'+' || rhs.v_type != VAR_LIST {
        return Err(Failed);
    }
    // SAFETY: `VAR_LIST` says `v_list` is the union's live arm.
    let l2 = unsafe { rhs.vval.v_list };
    if l2.is_null() {
        return Ok(());
    }
    // SAFETY: `tv1` reached this helper under `VAR_LIST` too.
    let l1 = unsafe { lhs.vval.v_list };
    if l1.is_null() {
        // Appending to an unallocated list shares the right-hand one
        // rather than copying it.
        lhs.vval.v_list = l2;
        // SAFETY: `l2` is the live List the right-hand typval holds.
        unsafe { (*l2).lv_refcount.retain() };
    } else {
        // SAFETY: both Lists are live.
        unsafe { tv_list_extend(l1, l2, ::core::ptr::null_mut::<listitem_T>()) };
    }
    Ok(())
}

/// `nr += nr`, `nr -= nr`, `nr *= nr`, `nr /= nr`, `nr %= nr`.
///
/// A float on the right promotes the result to a float, except for `%`, which
/// has no float form and fails.
unsafe fn tv_op_number(tv1: *mut typval_T, tv2: *const typval_T, op: u8) -> Result<(), Failed> {
    // SAFETY: the caller's obligation -- two initialised typvals, which may
    // alias. Both operands are read out in full before `tv_clear` touches
    // `tv1`, which is what makes the aliasing case safe.
    let (mut lhs, rhs) = unsafe { (Tv::new(tv1), Tv::new(tv2.cast_mut())) };
    // SAFETY: as above.
    let n: varnumber_T = unsafe { tv_get_number(tv1) };
    if rhs.v_type == VAR_FLOAT {
        if op == b'%' {
            return Err(Failed);
        }
        // SAFETY: `VAR_FLOAT` says `v_float` is the union's live arm.
        let f = float_op(n as float_T, op, unsafe { rhs.vval.v_float });
        // SAFETY: `tv1` is the caller's initialised typval.
        unsafe { tv_clear(tv1) };
        lhs.v_type = VAR_FLOAT;
        lhs.vval.v_float = f;
    } else {
        // Only the arm that is taken reads the right operand, because
        // `tv_get_number` reports on a value it cannot convert.
        let n = match op {
            // SAFETY: `tv2` is initialised.
            b'+' => n.wrapping_add(unsafe { tv_get_number(tv2) }),
            b'-' => n.wrapping_sub(unsafe { tv_get_number(tv2) }),
            b'*' => n.wrapping_mul(unsafe { tv_get_number(tv2) }),
            b'/' => num_divide(n, unsafe { tv_get_number(tv2) }),
            b'%' => num_modulus(n, unsafe { tv_get_number(tv2) }),
            _ => n,
        };
        // SAFETY: `tv1` is the caller's initialised typval.
        unsafe { tv_clear(tv1) };
        lhs.v_type = VAR_NUMBER;
        lhs.vval.v_number = n;
    }
    Ok(())
}

/// `str1 .= str2`.
unsafe fn tv_op_string(tv1: *mut typval_T, tv2: *const typval_T) -> Result<(), Failed> {
    // The two operands need a scratch each: `s2` is still live when `tv1`'s
    // own string form is rendered.
    let mut numbuf1 = NumBuf::new();
    // SAFETY: the caller's obligation -- two initialised typvals, which may
    // alias. Both scratches are live locals that outlive the strings
    // formatted into them, and `concat_str` has copied both operands before
    // `tv_clear` runs.
    let (mut lhs, rhs) = unsafe { (Tv::new(tv1), Tv::new(tv2.cast_mut())) };
    if rhs.v_type == VAR_FLOAT {
        return Err(Failed);
    }
    let mut numbuf = NumBuf::new();
    // SAFETY: as above.
    let s2 = unsafe { numbuf.string(tv2) };
    // An owned string with room to spare is extended in place.
    // SAFETY: as above.
    if unsafe { grow_string_tv(tv1, s2) } {
        return Ok(());
    }
    // SAFETY: as above.
    let s = unsafe { concat_str(numbuf1.string(tv1), s2) };
    // SAFETY: both operands have been copied out of `tv1` by now.
    unsafe { tv_clear(tv1) };
    lhs.v_type = VAR_STRING;
    lhs.vval.v_string = s;
    Ok(())
}

/// `f1 += f2`, `f1 -= f2`, `f1 *= f2`, `f1 /= f2`.
unsafe fn tv_op_float(tv1: *mut typval_T, tv2: *const typval_T, op: u8) -> Result<(), Failed> {
    // SAFETY: the caller's obligation -- two initialised typvals, which may
    // alias. The right operand is read before the left is written.
    let (mut lhs, rhs) = unsafe { (Tv::new(tv1), Tv::new(tv2.cast_mut())) };
    let rhs_type = rhs.v_type;
    if op == b'%'
        || op == b'.'
        || (rhs_type != VAR_FLOAT && rhs_type != VAR_NUMBER && rhs_type != VAR_STRING)
    {
        return Err(Failed);
    }
    let f = if rhs_type == VAR_FLOAT {
        // SAFETY: `VAR_FLOAT` says `v_float` is the union's live arm.
        unsafe { rhs.vval.v_float }
    } else {
        // A string operand goes through the usual "leading number" parse.
        // SAFETY: `tv2` is initialised.
        unsafe { tv_get_number(tv2) as float_T }
    };
    // SAFETY: `tv1` reached this helper under `VAR_FLOAT`, so `v_float` is
    // its live arm.
    lhs.vval.v_float = float_op(unsafe { lhs.vval.v_float }, op, f);
    Ok(())
}

/// `tv1 += tv2`, `-=`, `*=`, `/=`, `%=`, `.=`. Returns `OK` or `FAIL`; on
/// `FAIL` the "wrong variable type" error has already been reported.
///
/// # Safety
///
/// `tv1` and `tv2` must point at initialised typvals; `op` must point at a
/// NUL-terminated operator. The two typvals may alias.
pub unsafe fn eexe_mod_op(
    tv1: *mut typval_T,
    tv2: *const typval_T,
    op: *const c_char,
) -> Result<(), Failed> {
    // SAFETY: the caller's obligation -- `op` is NUL-terminated.
    let op = unsafe { CStr::from_ptr(op) };
    let op_byte = op.to_bytes().first().copied().unwrap_or(0);
    // SAFETY: the caller's obligation -- two initialised typvals, which may
    // alias; every helper below restates the same promise.
    let (lhs, rhs) = unsafe { (Tv::new(tv1), Tv::new(tv2.cast_mut())) };
    let rhs_type = rhs.v_type;
    // Nothing works with a Funcref or a Dict on the right, and v:true and
    // friends only work with "..=".
    if rhs_type == VAR_FUNC
        || rhs_type == VAR_DICT
        || ((rhs_type == VAR_BOOL || rhs_type == VAR_SPECIAL) && op_byte == b'.')
    {
        report_wrong_type(op);
        return Err(Failed);
    }
    let retval = match lhs.v_type {
        // SAFETY: as above -- each helper takes the same two pointers.
        VAR_BLOB => unsafe { tv_op_blob(tv1, tv2, op_byte) },
        VAR_LIST => unsafe { tv_op_list(tv1, tv2, op_byte) },
        VAR_NUMBER | VAR_STRING => {
            if rhs_type == VAR_LIST {
                Err(Failed)
            } else if is_arithmetic(op_byte) {
                unsafe { tv_op_number(tv1, tv2, op_byte) }
            } else {
                unsafe { tv_op_string(tv1, tv2) }
            }
        }
        VAR_FLOAT => unsafe { tv_op_float(tv1, tv2, op_byte) },
        VAR_UNKNOWN => unsafe { abort() },
        // Dict, Funcref, Partial, Bool and Special have no compound form.
        _ => Err(Failed),
    };

    if retval.is_err() {
        report_wrong_type(op);
    }
    retval
}

/// Report `E734` naming the operator that was refused.
fn report_wrong_type(op: &CStr) {
    crate::semsg!("E734: Wrong variable type for {}=", op.to_string_lossy());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concatenation_is_the_only_non_arithmetic_operator() {
        for op in *b"+-*/%" {
            assert!(is_arithmetic(op));
        }
        assert!(!is_arithmetic(b'.'));
        // vim_strchr() answers NULL for a NUL byte, so an empty operator
        // concatenates rather than adding.
        assert!(!is_arithmetic(0));
    }

    #[test]
    fn an_unrecognised_float_operator_leaves_the_value_alone() {
        assert_eq!(float_op(1.5, b'+', 2.0), 3.5);
        assert_eq!(float_op(1.5, b'-', 2.0), -0.5);
        assert_eq!(float_op(1.5, b'*', 2.0), 3.0);
        assert_eq!(float_op(3.0, b'/', 2.0), 1.5);
        assert_eq!(float_op(1.5, b'?', 2.0), 1.5);
    }
}
