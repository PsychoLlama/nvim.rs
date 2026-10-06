//! Compound assignment for vimscript values: the `+=`, `-=`, `*=`, `/=`, `%=`
//! and `.=`/`..=` half of `:let`.
//!
//! [`eexe_mod_op`] dispatches on the *left* operand's type; each arm decides
//! for itself whether the operator and the right operand make sense, and
//! answers `FAIL` when they do not so the caller reports one error message.
//!
//! The two operands are a `&mut` and a `&`, so they are never one value: a
//! caller whose right operand may be the left one -- `:let l[0:1] += l[0:1]`
//! in `list_assign_range` -- copies it first. The two may still *share* a
//! List or a Blob, which is what `l += l` is; each arm copies what it reads
//! out of the right operand before it writes.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::typval::{NumBuf, list_extend, tv_clear, tv_get_number};
use crate::eval::{num_divide, num_modulus};
use crate::message_fmt::msg_bytes;
use crate::types::{
    Failed, Float, TypVal, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER,
    VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VarNumber,
};

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
fn float_op(lhs: Float, op: u8, rhs: Float) -> Float {
    match op {
        b'+' => lhs + rhs,
        b'-' => lhs - rhs,
        b'*' => lhs * rhs,
        b'/' => lhs / rhs,
        _ => lhs,
    }
}

/// `blob1 += blob2`.
fn tv_op_blob(lhs: &mut TypVal, rhs: &TypVal, op: u8) -> Result<(), Failed> {
    if op != b'+' || rhs.v_type() != VAR_BLOB {
        return Err(Failed);
    }
    let Some(tail) = rhs.blob_ref() else {
        return Ok(());
    };
    if lhs.blob_ref().is_none() {
        // Appending to an unallocated blob shares the right-hand one
        // rather than copying it.
        lhs.write_blob(rhs.blob_handle());
        return Ok(());
    }
    // Copied out first: `b1 += b1` appends a copy of itself.
    let tail = tail.bytes().to_vec();
    if let Some(blob) = lhs.blob_mut() {
        blob.extend(&tail);
    }
    Ok(())
}

/// `list1 += list2`.
fn tv_op_list(lhs: &mut TypVal, rhs: &TypVal, op: u8) -> Result<(), Failed> {
    if op != b'+' || rhs.v_type() != VAR_LIST {
        return Err(Failed);
    }
    let Some(l2) = rhs.list_shared() else {
        return Ok(());
    };
    match lhs.list_shared() {
        // `l += l` is the one list twice, which the extend answers.
        Some(l1) => list_extend(l1, Some(l2), None),
        // Appending to an unallocated list shares the right-hand one
        // rather than copying it.
        None => lhs.write_list(Some(l2.clone())),
    }
    Ok(())
}

/// `nr += nr`, `nr -= nr`, `nr *= nr`, `nr /= nr`, `nr %= nr`.
///
/// A float on the right promotes the result to a float, except for `%`, which
/// has no float form and fails.
fn tv_op_number(lhs: &mut TypVal, rhs: &TypVal, op: u8) -> Result<(), Failed> {
    let n: VarNumber = tv_get_number(lhs);
    if rhs.v_type() == VAR_FLOAT {
        if op == b'%' {
            return Err(Failed);
        }
        let f = float_op(n as Float, op, rhs.float_or_zero());
        tv_clear(lhs);
        lhs.write_float(f);
    } else {
        // Only the arm that is taken reads the right operand, because
        // `tv_get_number` reports on a value it cannot convert.
        let n = match op {
            b'+' => n.wrapping_add(tv_get_number(rhs)),
            b'-' => n.wrapping_sub(tv_get_number(rhs)),
            b'*' => n.wrapping_mul(tv_get_number(rhs)),
            b'/' => num_divide(n, tv_get_number(rhs)),
            b'%' => num_modulus(n, tv_get_number(rhs)),
            _ => n,
        };
        tv_clear(lhs);
        lhs.write_number(n);
    }
    Ok(())
}

/// `str1 .= str2`.
fn tv_op_string(lhs: &mut TypVal, rhs: &TypVal) -> Result<(), Failed> {
    if rhs.v_type() == VAR_FLOAT {
        return Err(Failed);
    }
    let mut numbuf = NumBuf::new();
    let s2 = numbuf.bytes(rhs);
    // An owned string is extended in place.
    if lhs.append_to_string(s2) {
        return Ok(());
    }
    let mut numbuf1 = NumBuf::new();
    let mut joined = numbuf1.bytes(lhs).to_vec();
    joined.extend_from_slice(s2);
    tv_clear(lhs);
    lhs.write_string(Some(joined.into()));
    Ok(())
}

/// `f1 += f2`, `f1 -= f2`, `f1 *= f2`, `f1 /= f2`.
fn tv_op_float(lhs: &mut TypVal, rhs: &TypVal, op: u8) -> Result<(), Failed> {
    let rhs_type = rhs.v_type();
    if op == b'%'
        || op == b'.'
        || (rhs_type != VAR_FLOAT && rhs_type != VAR_NUMBER && rhs_type != VAR_STRING)
    {
        return Err(Failed);
    }
    let f = if rhs_type == VAR_FLOAT {
        rhs.float_or_zero()
    } else {
        // A string operand goes through the usual "leading number" parse.
        tv_get_number(rhs) as Float
    };
    let result = float_op(lhs.float_or_zero(), op, f);
    lhs.write_float(result);
    Ok(())
}

/// `tv1 += tv2`, `-=`, `*=`, `/=`, `%=`, `.=`, with `op` the operator's one
/// byte (`+`, `-`, `*`, `/`, `%` or `.`). Returns `Ok` or `Err`; on `Err`
/// the "wrong variable type" error has already been reported.
pub fn eexe_mod_op(tv1: &mut TypVal, tv2: &TypVal, op: u8) -> Result<(), Failed> {
    let rhs_type = tv2.v_type();
    // Nothing works with a Funcref or a Dict on the right, and v:true and
    // friends only work with "..=".
    if rhs_type == VAR_FUNC
        || rhs_type == VAR_DICT
        || ((rhs_type == VAR_BOOL || rhs_type == VAR_SPECIAL) && op == b'.')
    {
        report_wrong_type(op);
        return Err(Failed);
    }
    let retval = match tv1.v_type() {
        VAR_BLOB => tv_op_blob(tv1, tv2, op),
        VAR_LIST => tv_op_list(tv1, tv2, op),
        VAR_NUMBER | VAR_STRING => {
            if rhs_type == VAR_LIST {
                Err(Failed)
            } else if is_arithmetic(op) {
                tv_op_number(tv1, tv2, op)
            } else {
                tv_op_string(tv1, tv2)
            }
        }
        VAR_FLOAT => tv_op_float(tv1, tv2, op),
        VAR_UNKNOWN => std::process::abort(),
        // Dict, Funcref, Partial, Bool and Special have no compound form.
        _ => Err(Failed),
    };

    if retval.is_err() {
        report_wrong_type(op);
    }
    retval
}

/// Report `E734` naming the operator that was refused: its one byte, or
/// nothing for an empty one.
fn report_wrong_type(op: u8) {
    let op: &[u8] = if op == 0 { &[] } else { &[op] };
    crate::semsg!("E734: Wrong variable type for {}=", msg_bytes(op));
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
