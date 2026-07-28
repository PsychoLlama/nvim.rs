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

use crate::src::nvim::eval::typval::{
    FAIL, OK, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_SPECIAL,
    VAR_STRING, VAR_UNKNOWN, tv_clear, tv_get_number, tv_get_string, tv_get_string_buf,
    tv_list_extend,
};
use crate::src::nvim::eval::{grow_string_tv, num_divide, num_modulus};
use crate::src::nvim::garray::ga_grow;
use crate::src::nvim::os::libc::{abort, memmove};
use crate::src::nvim::strings::concat_str;
use crate::src::nvim::types::{blob_T, float_T, listitem_T, typval_T, uint8_t, varnumber_T};
use core::ffi::{CStr, c_char, c_int};

/// The size of the buffer `tv_get_string_buf` formats a number into.
const NUMBUFLEN: usize = 65;

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
unsafe fn tv_op_blob(tv1: *mut typval_T, tv2: *const typval_T, op: u8) -> c_int {
    if op != b'+' || (*tv2).v_type != VAR_BLOB {
        return FAIL;
    }
    let b2: *mut blob_T = (*tv2).vval.v_blob;
    if b2.is_null() {
        return OK;
    }
    let b1: *mut blob_T = (*tv1).vval.v_blob;
    if b1.is_null() {
        // Appending to an unallocated blob shares the right-hand one rather
        // than copying it.
        (*tv1).vval.v_blob = b2;
        (*b2).bv_refcount += 1;
        return OK;
    }
    let len = (*b2).bv_ga.ga_len;
    if len > 0 {
        ga_grow(&raw mut (*b1).bv_ga, len);
        memmove(
            ((*b1).bv_ga.ga_data as *mut uint8_t)
                .offset((*b1).bv_ga.ga_len as isize)
                .cast(),
            (*b2).bv_ga.ga_data,
            len as usize,
        );
        (*b1).bv_ga.ga_len += len;
    }
    OK
}

/// `list1 += list2`.
unsafe fn tv_op_list(tv1: *mut typval_T, tv2: *const typval_T, op: u8) -> c_int {
    if op != b'+' || (*tv2).v_type != VAR_LIST {
        return FAIL;
    }
    let l2 = (*tv2).vval.v_list;
    if l2.is_null() {
        return OK;
    }
    let l1 = (*tv1).vval.v_list;
    if l1.is_null() {
        // Appending to an unallocated list shares the right-hand one rather
        // than copying it.
        (*tv1).vval.v_list = l2;
        (*l2).lv_refcount += 1;
    } else {
        tv_list_extend(l1, l2, ::core::ptr::null_mut::<listitem_T>());
    }
    OK
}

/// `nr += nr`, `nr -= nr`, `nr *= nr`, `nr /= nr`, `nr %= nr`.
///
/// A float on the right promotes the result to a float, except for `%`, which
/// has no float form and fails.
unsafe fn tv_op_number(tv1: *mut typval_T, tv2: *const typval_T, op: u8) -> c_int {
    let n: varnumber_T = tv_get_number(tv1);
    if (*tv2).v_type == VAR_FLOAT {
        if op == b'%' {
            return FAIL;
        }
        let f = float_op(n as float_T, op, (*tv2).vval.v_float);
        tv_clear(tv1);
        (*tv1).v_type = VAR_FLOAT;
        (*tv1).vval.v_float = f;
    } else {
        let n = match op {
            b'+' => n.wrapping_add(tv_get_number(tv2)),
            b'-' => n.wrapping_sub(tv_get_number(tv2)),
            b'*' => n.wrapping_mul(tv_get_number(tv2)),
            b'/' => num_divide(n, tv_get_number(tv2)),
            b'%' => num_modulus(n, tv_get_number(tv2)),
            _ => n,
        };
        tv_clear(tv1);
        (*tv1).v_type = VAR_NUMBER;
        (*tv1).vval.v_number = n;
    }
    OK
}

/// `str1 .= str2`.
unsafe fn tv_op_string(tv1: *mut typval_T, tv2: *const typval_T) -> c_int {
    if (*tv2).v_type == VAR_FLOAT {
        return FAIL;
    }
    let mut numbuf: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
    let s2 = tv_get_string_buf(tv2, numbuf.as_mut_ptr());
    // An owned string with room to spare is extended in place.
    if grow_string_tv(tv1, s2) {
        return OK;
    }
    let s = concat_str(tv_get_string(tv1), s2);
    tv_clear(tv1);
    (*tv1).v_type = VAR_STRING;
    (*tv1).vval.v_string = s;
    OK
}

/// `f1 += f2`, `f1 -= f2`, `f1 *= f2`, `f1 /= f2`.
unsafe fn tv_op_float(tv1: *mut typval_T, tv2: *const typval_T, op: u8) -> c_int {
    let rhs_type = (*tv2).v_type;
    if op == b'%'
        || op == b'.'
        || (rhs_type != VAR_FLOAT && rhs_type != VAR_NUMBER && rhs_type != VAR_STRING)
    {
        return FAIL;
    }
    let f = if rhs_type == VAR_FLOAT {
        (*tv2).vval.v_float
    } else {
        // A string operand goes through the usual "leading number" parse.
        tv_get_number(tv2) as float_T
    };
    (*tv1).vval.v_float = float_op((*tv1).vval.v_float, op, f);
    OK
}

/// `tv1 += tv2`, `-=`, `*=`, `/=`, `%=`, `.=`. Returns `OK` or `FAIL`; on
/// `FAIL` the "wrong variable type" error has already been reported.
///
/// # Safety
///
/// `tv1` and `tv2` must point at initialised typvals; `op` must point at a
/// NUL-terminated operator. The two typvals may alias.
pub unsafe fn eexe_mod_op(tv1: *mut typval_T, tv2: *const typval_T, op: *const c_char) -> c_int {
    let op = CStr::from_ptr(op);
    let op_byte = op.to_bytes().first().copied().unwrap_or(0);
    let rhs_type = (*tv2).v_type;
    // Nothing works with a Funcref or a Dict on the right, and v:true and
    // friends only work with "..=".
    if rhs_type == VAR_FUNC
        || rhs_type == VAR_DICT
        || ((rhs_type == VAR_BOOL || rhs_type == VAR_SPECIAL) && op_byte == b'.')
    {
        report_wrong_type(op);
        return FAIL;
    }

    let retval = match (*tv1).v_type {
        VAR_BLOB => tv_op_blob(tv1, tv2, op_byte),
        VAR_LIST => tv_op_list(tv1, tv2, op_byte),
        VAR_NUMBER | VAR_STRING => {
            if rhs_type == VAR_LIST {
                FAIL
            } else if is_arithmetic(op_byte) {
                tv_op_number(tv1, tv2, op_byte)
            } else {
                tv_op_string(tv1, tv2)
            }
        }
        VAR_FLOAT => tv_op_float(tv1, tv2, op_byte),
        VAR_UNKNOWN => abort(),
        // Dict, Funcref, Partial, Bool and Special have no compound form.
        _ => FAIL,
    };

    if retval != OK {
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
        for op in [b'+', b'-', b'*', b'/', b'%'] {
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
