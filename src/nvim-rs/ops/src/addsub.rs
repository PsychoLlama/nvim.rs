//! Number increment/decrement operations (Ctrl-A, Ctrl-X)
//!
//! This module implements the logic for incrementing and decrementing
//! numbers and alphabetic characters with Ctrl-A and Ctrl-X.

use std::ffi::c_int;

use crate::types::OpType;

/// Increment or decrement an alphabetic character.
///
/// If the character would wrap past 'Z'/'z' or before 'A'/'a', it clamps
/// to the boundary.
///
/// # Arguments
/// * `c` - The character to modify (must be ASCII alphabetic)
/// * `op_type` - `OpType::NrAdd` for increment, `OpType::NrSub` for decrement
/// * `amount` - The amount to add or subtract
///
/// # Returns
/// The new character value, or the original if not alphabetic
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn addsub_alpha(c: c_int, op_type: OpType, amount: i64) -> c_int {
    // Only handle ASCII range
    if !(0..=127).contains(&c) {
        return c;
    }

    let c_byte = c as u8;

    // Check if it's an ASCII letter
    if !c_byte.is_ascii_alphabetic() {
        return c;
    }

    let is_upper = c_byte.is_ascii_uppercase();
    let base = if is_upper { b'A' } else { b'a' };
    let ord = i64::from(c_byte - base); // 0-25

    let new_ord = if op_type == OpType::NrSub {
        // Decrement - clamp at 0 (A/a)
        if ord < amount {
            0
        } else {
            ord - amount
        }
    } else {
        // Increment - clamp at 25 (Z/z)
        let max_add = 25 - ord;
        if amount > max_add {
            25
        } else {
            ord + amount
        }
    };

    c_int::from(base + new_ord as u8)
}

/// FFI wrapper for addsub_alpha.
#[no_mangle]
pub extern "C" fn rs_addsub_alpha(c: c_int, op_type: c_int, amount: i64) -> c_int {
    let op = OpType::from_raw(op_type).unwrap_or(OpType::NrAdd);
    addsub_alpha(c, op, amount)
}

/// Handle wraparound for unsigned number arithmetic.
///
/// When subtracting causes underflow or adding causes overflow, this
/// computes the proper result accounting for wraparound behavior.
///
/// # Arguments
/// * `n` - The current value
/// * `amount` - The amount to add (positive) or subtract (negative via subtract flag)
/// * `subtract` - true if subtracting, false if adding
///
/// # Returns
/// A tuple of (new_value, is_negative) where is_negative indicates if the
/// result should be displayed as a negative number.
#[must_use]
pub fn handle_wraparound(n: u64, amount: u64, subtract: bool) -> (u64, bool) {
    if subtract {
        if amount > n {
            // Underflow - compute magnitude and flip sign
            let diff = amount - n;
            (diff, true)
        } else {
            (n - amount, false)
        }
    } else {
        let result = n.wrapping_add(amount);
        if result < n {
            // Overflow - compute wrapped value and flip sign
            (!result, true)
        } else {
            (result, false)
        }
    }
}

/// FFI wrapper for handle_wraparound.
///
/// # Safety
/// `out_negative` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_handle_wraparound(
    n: u64,
    amount: u64,
    subtract: c_int,
    out_negative: *mut c_int,
) -> u64 {
    let (result, negative) = handle_wraparound(n, amount, subtract != 0);
    if !out_negative.is_null() {
        *out_negative = c_int::from(negative);
    }
    result
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_addsub_alpha_increment() {
        // Basic increment
        assert_eq!(addsub_alpha(b'a' as c_int, OpType::NrAdd, 1), b'b' as c_int);
        assert_eq!(addsub_alpha(b'A' as c_int, OpType::NrAdd, 1), b'B' as c_int);
        assert_eq!(
            addsub_alpha(b'm' as c_int, OpType::NrAdd, 5),
            b'r' as c_int
        );

        // Clamp at z/Z
        assert_eq!(
            addsub_alpha(b'y' as c_int, OpType::NrAdd, 5),
            b'z' as c_int
        );
        assert_eq!(
            addsub_alpha(b'z' as c_int, OpType::NrAdd, 1),
            b'z' as c_int
        );
        assert_eq!(
            addsub_alpha(b'Z' as c_int, OpType::NrAdd, 100),
            b'Z' as c_int
        );
    }

    #[test]
    fn test_addsub_alpha_decrement() {
        // Basic decrement
        assert_eq!(addsub_alpha(b'b' as c_int, OpType::NrSub, 1), b'a' as c_int);
        assert_eq!(addsub_alpha(b'B' as c_int, OpType::NrSub, 1), b'A' as c_int);
        assert_eq!(
            addsub_alpha(b'n' as c_int, OpType::NrSub, 5),
            b'i' as c_int
        );

        // Clamp at a/A
        assert_eq!(
            addsub_alpha(b'c' as c_int, OpType::NrSub, 5),
            b'a' as c_int
        );
        assert_eq!(
            addsub_alpha(b'a' as c_int, OpType::NrSub, 1),
            b'a' as c_int
        );
        assert_eq!(
            addsub_alpha(b'A' as c_int, OpType::NrSub, 100),
            b'A' as c_int
        );
    }

    #[test]
    fn test_addsub_alpha_non_alpha() {
        // Non-alphabetic characters should be unchanged
        assert_eq!(addsub_alpha(b'0' as c_int, OpType::NrAdd, 1), b'0' as c_int);
        assert_eq!(addsub_alpha(b'!' as c_int, OpType::NrSub, 1), b'!' as c_int);
        assert_eq!(addsub_alpha(b' ' as c_int, OpType::NrAdd, 1), b' ' as c_int);
    }

    #[test]
    fn test_handle_wraparound_no_wrap() {
        // Normal subtraction
        assert_eq!(handle_wraparound(10, 5, true), (5, false));
        // Normal addition
        assert_eq!(handle_wraparound(10, 5, false), (15, false));
    }

    #[test]
    fn test_handle_wraparound_underflow() {
        // Subtracting more than the value
        assert_eq!(handle_wraparound(5, 10, true), (5, true));
        assert_eq!(handle_wraparound(0, 1, true), (1, true));
    }

    #[test]
    fn test_handle_wraparound_overflow() {
        // Adding causes overflow
        let max = u64::MAX;
        let (result, negative) = handle_wraparound(max, 1, false);
        // !0 = max, so result should be max
        assert!(negative);
        assert_eq!(result, max);
    }
}
