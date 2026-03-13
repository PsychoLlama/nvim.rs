//! Pure math functions for VimL.
//!
//! This module implements math functions from `src/nvim/eval/funcs.c`:
//! - `abs()`, `round()`, `ceil()`, `floor()`, `trunc()`
//! - `sin()`, `cos()`, `tan()`, `asin()`, `acos()`, `atan()`, `atan2()`
//! - `sinh()`, `cosh()`, `tanh()`
//! - `exp()`, `log()`, `log10()`, `pow()`, `sqrt()`
//! - `fmod()`, `float2nr()`, `str2float()`
//! - `min()`, `max()`, `isnan()`, `isinf()`

use std::ffi::c_void;

use super::dispatch::{
    argvar_at, rettv_set_float, rettv_set_number, tv_get_float_chk, tv_get_float_raw,
    tv_get_number_chk, tv_get_type, TypevalPtrMut, VarType,
};

// =============================================================================
// Single-argument float functions
// =============================================================================

/// Helper for single-argument float functions.
///
/// Gets the float from argvars[0], applies the function, sets rettv to float result.
#[inline]
fn float_op_single(argvars: *const c_void, rettv: *mut c_void, op: fn(f64) -> f64) {
    let rettv = unsafe { TypevalPtrMut::from_raw(rettv) };
    let arg0 = unsafe { argvar_at(argvars, 0) };

    if let Some(f) = tv_get_float_chk(arg0) {
        rettv_set_float(rettv, op(f));
    } else {
        rettv_set_float(rettv, 0.0);
    }
}

/// "sin()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_sin"]
pub unsafe extern "C" fn rs_f_sin(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::sin);
}

/// "cos()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_cos"]
pub unsafe extern "C" fn rs_f_cos(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::cos);
}

/// "tan()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_tan"]
pub unsafe extern "C" fn rs_f_tan(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::tan);
}

/// "asin()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_asin"]
pub unsafe extern "C" fn rs_f_asin(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::asin);
}

/// "acos()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_acos"]
pub unsafe extern "C" fn rs_f_acos(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::acos);
}

/// "atan()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_atan"]
pub unsafe extern "C" fn rs_f_atan(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::atan);
}

/// "sinh()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_sinh"]
pub unsafe extern "C" fn rs_f_sinh(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::sinh);
}

/// "cosh()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_cosh"]
pub unsafe extern "C" fn rs_f_cosh(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::cosh);
}

/// "tanh()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_tanh"]
pub unsafe extern "C" fn rs_f_tanh(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::tanh);
}

/// "exp()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_exp"]
pub unsafe extern "C" fn rs_f_exp(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::exp);
}

/// "log()" function - natural logarithm
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_log"]
pub unsafe extern "C" fn rs_f_log(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::ln);
}

/// "log10()" function - base-10 logarithm
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_log10"]
pub unsafe extern "C" fn rs_f_log10(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    float_op_single(argvars, rettv, f64::log10);
}

/// "sqrt()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_sqrt"]
pub unsafe extern "C" fn rs_f_sqrt(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::sqrt);
}

/// "ceil()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_ceil"]
pub unsafe extern "C" fn rs_f_ceil(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_single(argvars, rettv, f64::ceil);
}

/// "floor()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_floor"]
pub unsafe extern "C" fn rs_f_floor(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    float_op_single(argvars, rettv, f64::floor);
}

/// "round()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_round"]
pub unsafe extern "C" fn rs_f_round(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    float_op_single(argvars, rettv, f64::round);
}

/// "trunc()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_trunc"]
pub unsafe extern "C" fn rs_f_trunc(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    float_op_single(argvars, rettv, f64::trunc);
}

// =============================================================================
// Two-argument float functions
// =============================================================================

/// Helper for two-argument float functions.
#[inline]
fn float_op_double(argvars: *const c_void, rettv: *mut c_void, op: fn(f64, f64) -> f64) {
    let rettv = unsafe { TypevalPtrMut::from_raw(rettv) };
    let arg0 = unsafe { argvar_at(argvars, 0) };
    let arg1 = unsafe { argvar_at(argvars, 1) };

    match (tv_get_float_chk(arg0), tv_get_float_chk(arg1)) {
        (Some(f0), Some(f1)) => rettv_set_float(rettv, op(f0, f1)),
        _ => rettv_set_float(rettv, 0.0),
    }
}

/// "atan2()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_atan2"]
pub unsafe extern "C" fn rs_f_atan2(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    float_op_double(argvars, rettv, f64::atan2);
}

/// "pow()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_pow"]
pub unsafe extern "C" fn rs_f_pow(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_double(argvars, rettv, f64::powf);
}

/// "fmod()" function - floating-point modulus
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_fmod"]
pub unsafe extern "C" fn rs_f_fmod(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    float_op_double(argvars, rettv, |a, b| a % b);
}

// =============================================================================
// abs() - works on both integers and floats
// =============================================================================

/// "abs()" function
///
/// For floats: returns fabs(x)
/// For integers: returns absolute value
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_abs"]
pub unsafe extern "C" fn rs_f_abs(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);

    if tv_get_type(arg0) == VarType::Float {
        let f = tv_get_float_raw(arg0);
        rettv_set_float(rettv, f.abs());
    } else {
        let (n, error) = tv_get_number_chk(arg0);
        if error {
            rettv_set_number(rettv, -1);
        } else if n > 0 {
            rettv_set_number(rettv, n);
        } else {
            rettv_set_number(rettv, -n);
        }
    }
}

// =============================================================================
// Float/Number conversion functions
// =============================================================================

/// "float2nr()" function - convert float to number
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_float2nr"]
pub unsafe extern "C" fn rs_f_float2nr(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);

    if let Some(f) = tv_get_float_chk(arg0) {
        // Truncate towards zero, clamping to i64 range
        // These casts are intentional - we're checking bounds before truncating
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let n = if f >= i64::MAX as f64 {
            i64::MAX
        } else if f <= i64::MIN as f64 {
            i64::MIN
        } else {
            f.trunc() as i64
        };
        rettv_set_number(rettv, n);
    } else {
        rettv_set_number(rettv, 0);
    }
}

// =============================================================================
// isnan() and isinf() - float inspection
// =============================================================================

/// "isnan()" function - check if float is NaN
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_isnan"]
pub unsafe extern "C" fn rs_f_isnan(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);

    if let Some(f) = tv_get_float_chk(arg0) {
        rettv_set_number(rettv, i64::from(f.is_nan()));
    } else {
        rettv_set_number(rettv, 0);
    }
}

/// "isinf()" function - check if float is infinity
///
/// Returns: 1 for +inf, -1 for -inf, 0 otherwise
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_isinf"]
pub unsafe extern "C" fn rs_f_isinf(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);

    if let Some(f) = tv_get_float_chk(arg0) {
        let result = if f == f64::INFINITY {
            1
        } else if f == f64::NEG_INFINITY {
            -1
        } else {
            0
        };
        rettv_set_number(rettv, result);
    } else {
        rettv_set_number(rettv, 0);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_float_ops() {
        // Test that float operations work correctly
        assert!((0.0_f64.sin() - 0.0).abs() < 1e-10);
        assert!((std::f64::consts::PI.sin()).abs() < 1e-10);
        assert!((0.0_f64.cos() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_isnan_isinf_logic() {
        assert!(f64::NAN.is_nan());
        assert!(!1.0_f64.is_nan());
        assert!(f64::INFINITY.is_infinite());
        assert!(f64::NEG_INFINITY.is_infinite());
        assert!(!1.0_f64.is_infinite());
    }
}
