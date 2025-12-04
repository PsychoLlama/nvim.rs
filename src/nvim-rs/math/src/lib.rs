//! Math utilities for Neovim
//!
//! This module provides Rust implementations of the math functions from
//! `src/nvim/math.c`. These are pure functions with no external dependencies,
//! making them ideal candidates for the initial migration phase.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::os::raw::{c_int, c_uint};

/// Return values matching nvim's OK/FAIL
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;

/// FP classification constants (matching C's math.h)
pub const FP_NORMAL: c_int = 4;
pub const FP_SUBNORMAL: c_int = 3;
pub const FP_ZERO: c_int = 2;
pub const FP_INFINITE: c_int = 1;
pub const FP_NAN: c_int = 0;

/// Classify a floating-point number.
///
/// Returns one of: `FP_NORMAL`, `FP_SUBNORMAL`, `FP_ZERO`, `FP_INFINITE`, `FP_NAN`
///
/// This is a portable implementation that doesn't rely on the system's fpclassify.
#[no_mangle]
pub extern "C" fn rs_xfpclassify(d: f64) -> c_int {
    let bits: u64 = d.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as u32;
    let mantissa = bits & 0xf_ffff_ffff_ffff;

    match exponent {
        0 => {
            if mantissa != 0 {
                FP_SUBNORMAL
            } else {
                FP_ZERO
            }
        }
        0x7ff => {
            if mantissa != 0 {
                FP_NAN
            } else {
                FP_INFINITE
            }
        }
        _ => FP_NORMAL,
    }
}

/// Check if a floating-point number is infinite.
#[no_mangle]
pub extern "C" fn rs_xisinf(d: f64) -> c_int {
    c_int::from(rs_xfpclassify(d) == FP_INFINITE)
}

/// Check if a floating-point number is NaN.
#[no_mangle]
pub extern "C" fn rs_xisnan(d: f64) -> c_int {
    c_int::from(rs_xfpclassify(d) == FP_NAN)
}

/// Count trailing zeroes in a 64-bit value.
///
/// Returns 64 if the input is 0.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)] // trailing_zeros returns at most 64, fits in i32
pub extern "C" fn rs_xctz(x: u64) -> c_int {
    if x == 0 {
        64
    } else {
        x.trailing_zeros() as c_int
    }
}

/// Count the number of set bits (population count) in a 64-bit value.
#[no_mangle]
pub extern "C" fn rs_xpopcount(x: u64) -> c_uint {
    x.count_ones()
}

/// Safely append a digit to an integer value, checking for overflow.
///
/// Returns `OK` (1) on success, `FAIL` (0) on overflow.
/// On success, `*value` is updated to `*value * 10 + digit`.
///
/// # Safety
///
/// `value` must be a valid pointer to a writable `c_int`.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_append_digit_int(value: *mut c_int, digit: c_int) -> c_int {
    if value.is_null() {
        return FAIL;
    }

    let x = unsafe { *value };

    // Check for overflow: x > (INT_MAX - digit) / 10
    // Rearranged to avoid overflow in the check itself
    if let Some(max_before) = c_int::MAX.checked_sub(digit) {
        if x > max_before / 10 {
            return FAIL;
        }
    } else {
        // digit is negative and large enough that INT_MAX - digit overflows
        // This shouldn't happen with valid digit values (0-9)
        return FAIL;
    }

    unsafe {
        *value = x * 10 + digit;
    }
    OK
}

/// Clamp an i64 value to fit in a `c_int` (i32).
///
/// Returns `INT_MAX` if `x > INT_MAX`, `INT_MIN` if `x < INT_MIN`,
/// otherwise returns `x` as a `c_int`.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)] // Truncation is intentional - we clamp first
pub extern "C" fn rs_trim_to_int(x: i64) -> c_int {
    if x > i64::from(c_int::MAX) {
        c_int::MAX
    } else if x < i64::from(c_int::MIN) {
        c_int::MIN
    } else {
        x as c_int
    }
}

/// Check if a number is a power of two.
///
/// Returns 1 if `x` is a power of two, 0 otherwise.
/// Note: 0 is not considered a power of two.
#[no_mangle]
pub extern "C" fn rs_is_power_of_two(x: u64) -> c_int {
    c_int::from(x.is_power_of_two())
}

/// Divide n1 by n2, handling division by zero and overflow.
///
/// Returns:
/// - VARNUMBER_MIN (similar to NaN) if n1 == 0 and n2 == 0
/// - -VARNUMBER_MAX if n1 < 0 and n2 == 0
/// - VARNUMBER_MAX if n1 > 0 and n2 == 0
/// - VARNUMBER_MAX if n1 == VARNUMBER_MIN and n2 == -1 (overflow case)
/// - n1 / n2 otherwise
#[no_mangle]
pub extern "C" fn rs_num_divide(n1: i64, n2: i64) -> i64 {
    if n2 == 0 {
        // Division by zero - give an error message?
        if n1 == 0 {
            i64::MIN // similar to NaN
        } else if n1 < 0 {
            -i64::MAX
        } else {
            i64::MAX
        }
    } else if n1 == i64::MIN && n2 == -1 {
        // Specific case: trying to do VARNUMBER_MIN / -1 results in a positive
        // number that doesn't fit in varnumber_T and causes an FPE
        i64::MAX
    } else {
        n1 / n2
    }
}

/// Compute n1 modulus n2, handling division by zero.
///
/// Returns 0 if n2 == 0, otherwise returns n1 % n2.
#[no_mangle]
pub extern "C" fn rs_num_modulus(n1: i64, n2: i64) -> i64 {
    if n2 == 0 {
        0
    } else {
        n1 % n2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fpclassify() {
        assert_eq!(rs_xfpclassify(0.0), FP_ZERO);
        assert_eq!(rs_xfpclassify(-0.0), FP_ZERO);
        assert_eq!(rs_xfpclassify(1.0), FP_NORMAL);
        assert_eq!(rs_xfpclassify(-1.0), FP_NORMAL);
        assert_eq!(rs_xfpclassify(f64::INFINITY), FP_INFINITE);
        assert_eq!(rs_xfpclassify(f64::NEG_INFINITY), FP_INFINITE);
        assert_eq!(rs_xfpclassify(f64::NAN), FP_NAN);

        // Subnormal: smallest positive subnormal is 2^-1074
        let subnormal = f64::from_bits(1); // Smallest subnormal
        assert_eq!(rs_xfpclassify(subnormal), FP_SUBNORMAL);
    }

    #[test]
    fn test_isinf() {
        assert_eq!(rs_xisinf(0.0), 0);
        assert_eq!(rs_xisinf(1.0), 0);
        assert_eq!(rs_xisinf(f64::INFINITY), 1);
        assert_eq!(rs_xisinf(f64::NEG_INFINITY), 1);
        assert_eq!(rs_xisinf(f64::NAN), 0);
    }

    #[test]
    fn test_isnan() {
        assert_eq!(rs_xisnan(0.0), 0);
        assert_eq!(rs_xisnan(1.0), 0);
        assert_eq!(rs_xisnan(f64::INFINITY), 0);
        assert_eq!(rs_xisnan(f64::NAN), 1);
    }

    #[test]
    fn test_ctz() {
        assert_eq!(rs_xctz(0), 64);
        assert_eq!(rs_xctz(1), 0);
        assert_eq!(rs_xctz(2), 1);
        assert_eq!(rs_xctz(4), 2);
        assert_eq!(rs_xctz(8), 3);
        assert_eq!(rs_xctz(0x8000_0000_0000_0000), 63);
        assert_eq!(rs_xctz(0b1010_0000), 5);
    }

    #[test]
    fn test_popcount() {
        assert_eq!(rs_xpopcount(0), 0);
        assert_eq!(rs_xpopcount(1), 1);
        assert_eq!(rs_xpopcount(0b1111), 4);
        assert_eq!(rs_xpopcount(0xFFFF_FFFF_FFFF_FFFF), 64);
        assert_eq!(rs_xpopcount(0b1010_1010), 4);
    }

    #[test]
    fn test_vim_append_digit_int() {
        let mut value: c_int = 0;
        unsafe {
            assert_eq!(
                rs_vim_append_digit_int(std::ptr::addr_of_mut!(value), 5),
                OK
            );
            assert_eq!(value, 5);

            assert_eq!(
                rs_vim_append_digit_int(std::ptr::addr_of_mut!(value), 3),
                OK
            );
            assert_eq!(value, 53);

            // Test overflow
            value = c_int::MAX / 10;
            assert_eq!(
                rs_vim_append_digit_int(std::ptr::addr_of_mut!(value), 9),
                FAIL
            );
        }
    }

    #[test]
    fn test_trim_to_int() {
        assert_eq!(rs_trim_to_int(0), 0);
        assert_eq!(rs_trim_to_int(100), 100);
        assert_eq!(rs_trim_to_int(-100), -100);
        assert_eq!(rs_trim_to_int(i64::from(c_int::MAX)), c_int::MAX);
        assert_eq!(rs_trim_to_int(i64::from(c_int::MIN)), c_int::MIN);
        assert_eq!(rs_trim_to_int(i64::MAX), c_int::MAX);
        assert_eq!(rs_trim_to_int(i64::MIN), c_int::MIN);
    }

    #[test]
    fn test_is_power_of_two() {
        assert_eq!(rs_is_power_of_two(0), 0);
        assert_eq!(rs_is_power_of_two(1), 1);
        assert_eq!(rs_is_power_of_two(2), 1);
        assert_eq!(rs_is_power_of_two(3), 0);
        assert_eq!(rs_is_power_of_two(4), 1);
        assert_eq!(rs_is_power_of_two(5), 0);
        assert_eq!(rs_is_power_of_two(1 << 63), 1);
    }

    #[test]
    fn test_num_divide() {
        // Normal division
        assert_eq!(rs_num_divide(10, 2), 5);
        assert_eq!(rs_num_divide(10, 3), 3);
        assert_eq!(rs_num_divide(-10, 2), -5);
        assert_eq!(rs_num_divide(10, -2), -5);
        assert_eq!(rs_num_divide(-10, -2), 5);

        // Division by zero
        assert_eq!(rs_num_divide(0, 0), i64::MIN); // NaN-like
        assert_eq!(rs_num_divide(1, 0), i64::MAX);
        assert_eq!(rs_num_divide(-1, 0), -i64::MAX);
        assert_eq!(rs_num_divide(100, 0), i64::MAX);
        assert_eq!(rs_num_divide(-100, 0), -i64::MAX);

        // Overflow case: MIN / -1 would overflow
        assert_eq!(rs_num_divide(i64::MIN, -1), i64::MAX);

        // Edge cases
        assert_eq!(rs_num_divide(i64::MAX, 1), i64::MAX);
        assert_eq!(rs_num_divide(i64::MIN, 1), i64::MIN);
    }

    #[test]
    fn test_num_modulus() {
        // Normal modulus
        assert_eq!(rs_num_modulus(10, 3), 1);
        assert_eq!(rs_num_modulus(10, 5), 0);
        assert_eq!(rs_num_modulus(-10, 3), -1);
        assert_eq!(rs_num_modulus(10, -3), 1);

        // Modulus by zero
        assert_eq!(rs_num_modulus(10, 0), 0);
        assert_eq!(rs_num_modulus(-10, 0), 0);
        assert_eq!(rs_num_modulus(0, 0), 0);
    }
}
