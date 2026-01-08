//! Bitwise functions for VimL.
//!
//! This module implements bitwise functions from `src/nvim/eval/funcs.c`:
//! - `and()`, `or()`, `xor()`, `invert()`

use std::ffi::c_void;

use super::dispatch::{argvar_at, rettv_set_number, tv_get_number_chk, TypevalPtrMut};

// =============================================================================
// Two-argument bitwise operations
// =============================================================================

/// Helper for two-argument bitwise operations.
///
/// Gets numbers from argvars[0] and argvars[1], applies the operation,
/// sets rettv to the result.
#[inline]
fn bitwise_op_double(argvars: *const c_void, rettv: *mut c_void, op: fn(i64, i64) -> i64) {
    let rettv = unsafe { TypevalPtrMut::from_raw(rettv) };
    let arg0 = unsafe { argvar_at(argvars, 0) };
    let arg1 = unsafe { argvar_at(argvars, 1) };

    let (n0, _) = tv_get_number_chk(arg0);
    let (n1, _) = tv_get_number_chk(arg1);
    rettv_set_number(rettv, op(n0, n1));
}

/// "and()" function - bitwise AND
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_and(argvars: *const c_void, rettv: *mut c_void) {
    bitwise_op_double(argvars, rettv, |a, b| a & b);
}

/// "or()" function - bitwise OR
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_or(argvars: *const c_void, rettv: *mut c_void) {
    bitwise_op_double(argvars, rettv, |a, b| a | b);
}

/// "xor()" function - bitwise XOR
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_xor(argvars: *const c_void, rettv: *mut c_void) {
    bitwise_op_double(argvars, rettv, |a, b| a ^ b);
}

// =============================================================================
// Single-argument bitwise operations
// =============================================================================

/// "invert()" function - bitwise NOT
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_invert(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = unsafe { TypevalPtrMut::from_raw(rettv) };
    let arg0 = unsafe { argvar_at(argvars, 0) };

    let (n, _) = tv_get_number_chk(arg0);
    rettv_set_number(rettv, !n);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bitwise_ops() {
        // Test bitwise operations
        assert_eq!(0b1010_i64 & 0b1100_i64, 0b1000);
        assert_eq!(0b1010_i64 | 0b1100_i64, 0b1110);
        assert_eq!(0b1010_i64 ^ 0b1100_i64, 0b0110);
        assert_eq!(!0_i64, -1);
        assert_eq!(!1_i64, -2);
    }
}
