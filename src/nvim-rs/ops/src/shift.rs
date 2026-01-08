//! Shift operations (< and >)
//!
//! This module implements indent shifting logic used by the `<` and `>`
//! operators.

use std::ffi::c_int;

/// Calculate new indentation when shifting.
///
/// # Arguments
/// * `left` - true if shifting left (<), false if shifting right (>)
/// * `round` - true if 'shiftround' is set
/// * `amount` - number of shift operations
/// * `sw_val` - shiftwidth value
/// * `current_indent` - current indentation in spaces
///
/// # Returns
/// The new indentation value (always >= 0)
#[must_use]
pub fn calc_new_indent(
    left: bool,
    round: bool,
    amount: i64,
    sw_val: i64,
    current_indent: i64,
) -> i64 {
    if sw_val == 0 {
        return current_indent;
    }

    if round {
        // Round off indent
        let i = current_indent / sw_val; // Number of 'shiftwidth' rounded down
        let j = current_indent % sw_val; // Extra spaces
        let mut amount = amount;

        // First remove extra spaces when shifting left
        if j != 0 && left {
            amount -= 1;
        }

        let new_units = if left {
            (i - amount).max(0)
        } else {
            i + amount
        };

        new_units * sw_val
    } else {
        // Original vi indent
        if left {
            (current_indent - sw_val * amount).max(0)
        } else {
            current_indent + sw_val * amount
        }
    }
}

/// FFI wrapper for calc_new_indent.
#[no_mangle]
pub extern "C" fn rs_calc_new_indent(
    left: c_int,
    round: c_int,
    amount: i64,
    sw_val: i64,
    current_indent: i64,
) -> i64 {
    calc_new_indent(left != 0, round != 0, amount, sw_val, current_indent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_right_no_round() {
        // Shift right without rounding
        assert_eq!(calc_new_indent(false, false, 1, 4, 0), 4);
        assert_eq!(calc_new_indent(false, false, 1, 4, 4), 8);
        assert_eq!(calc_new_indent(false, false, 2, 4, 0), 8);
        assert_eq!(calc_new_indent(false, false, 1, 2, 3), 5);
    }

    #[test]
    fn test_shift_left_no_round() {
        // Shift left without rounding
        assert_eq!(calc_new_indent(true, false, 1, 4, 8), 4);
        assert_eq!(calc_new_indent(true, false, 1, 4, 4), 0);
        assert_eq!(calc_new_indent(true, false, 2, 4, 8), 0);
        // Cannot go negative
        assert_eq!(calc_new_indent(true, false, 1, 4, 2), 0);
        assert_eq!(calc_new_indent(true, false, 1, 4, 0), 0);
    }

    #[test]
    fn test_shift_right_with_round() {
        // Shift right with rounding to shiftwidth boundary
        // When shifting right, extra spaces don't affect the amount
        // i = count / sw_val (integer division)
        // i += amount
        // result = i * sw_val
        assert_eq!(calc_new_indent(false, true, 1, 4, 0), 4); // 0/4=0, 0+1=1, 1*4=4
        assert_eq!(calc_new_indent(false, true, 1, 4, 2), 4); // 2/4=0, 0+1=1, 1*4=4
        assert_eq!(calc_new_indent(false, true, 1, 4, 4), 8); // 4/4=1, 1+1=2, 2*4=8
        assert_eq!(calc_new_indent(false, true, 1, 4, 5), 8); // 5/4=1, 1+1=2, 2*4=8
    }

    #[test]
    fn test_shift_left_with_round() {
        // Shift left with rounding - first removes extra spaces
        assert_eq!(calc_new_indent(true, true, 1, 4, 8), 4);
        assert_eq!(calc_new_indent(true, true, 1, 4, 4), 0);
        // Extra spaces: 6 = 1*4 + 2, so first shift removes the 2 extra -> 4
        assert_eq!(calc_new_indent(true, true, 1, 4, 6), 4);
        // After removing extra, continues shifting
        assert_eq!(calc_new_indent(true, true, 2, 4, 6), 0);
    }

    #[test]
    fn test_shift_zero_shiftwidth() {
        // Zero shiftwidth should return current indent unchanged
        assert_eq!(calc_new_indent(false, false, 1, 0, 4), 4);
        assert_eq!(calc_new_indent(true, true, 1, 0, 4), 4);
    }

    #[test]
    fn test_shift_large_amount() {
        // Large shift amounts
        assert_eq!(calc_new_indent(false, false, 10, 4, 0), 40);
        // Cannot go below 0
        assert_eq!(calc_new_indent(true, false, 10, 4, 8), 0);
    }
}
