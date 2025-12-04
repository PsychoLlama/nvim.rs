//! Profiling time utilities for Neovim
//!
//! This module provides Rust implementations of the pure timing functions from
//! `src/nvim/profile.c`. These are pure arithmetic functions with no external dependencies.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::os::raw::c_int;

/// Profile time type (nanoseconds since some epoch)
pub type Proftime = u64;

/// Returns the zero time.
#[no_mangle]
pub extern "C" fn rs_profile_zero() -> Proftime {
    0
}

/// Divides time `tm` by `count`.
///
/// Returns 0 if count <= 0, otherwise tm / count (rounded).
#[no_mangle]
pub extern "C" fn rs_profile_divide(tm: Proftime, count: c_int) -> Proftime {
    if count <= 0 {
        return 0;
    }
    // Use floating point for rounding like the C version
    ((tm as f64) / (count as f64)).round() as Proftime
}

/// Adds time `tm2` to `tm1`.
///
/// Returns `tm1` + `tm2`.
#[no_mangle]
pub extern "C" fn rs_profile_add(tm1: Proftime, tm2: Proftime) -> Proftime {
    tm1.wrapping_add(tm2)
}

/// Subtracts time `tm2` from `tm1`.
///
/// Unsigned overflow (wraparound) occurs if `tm2` is greater than `tm1`.
/// Use `rs_profile_signed()` to get the signed integer value.
#[no_mangle]
pub extern "C" fn rs_profile_sub(tm1: Proftime, tm2: Proftime) -> Proftime {
    tm1.wrapping_sub(tm2)
}

/// Adds the `self` time from the total time and the `children` time.
///
/// Returns if `total` <= `children`, then self, otherwise `self` + `total` - `children`.
#[no_mangle]
pub extern "C" fn rs_profile_self(self_time: Proftime, total: Proftime, children: Proftime) -> Proftime {
    // Check that the result won't be negative, which can happen with recursive calls.
    if total <= children {
        return self_time;
    }
    // Add the total time to self and subtract the children's time from self
    rs_profile_sub(rs_profile_add(self_time, total), children)
}

/// Checks if time `tm1` is equal to `tm2`.
#[no_mangle]
pub extern "C" fn rs_profile_equal(tm1: Proftime, tm2: Proftime) -> bool {
    tm1 == tm2
}

/// Converts time duration `tm` (from `profile_sub` result) to a signed integer.
///
/// If tm > INT64_MAX, it's assumed to be a negative duration from unsigned wraparound.
#[no_mangle]
pub extern "C" fn rs_profile_signed(tm: Proftime) -> i64 {
    // (tm > INT64_MAX) is >=150 years, so we can assume it was produced by
    // arithmetic of two proftime_T values. For human-readable representation
    // (and Vim-compat) we want the difference after unsigned wraparound. #10452
    if tm <= i64::MAX as u64 {
        tm as i64
    } else {
        -((u64::MAX - tm) as i64)
    }
}

/// Compares profiling times.
///
/// Times `tm1` and `tm2` must be less than 150 years apart.
///
/// Returns:
/// - <0: `tm2` < `tm1`
/// -  0: `tm2` == `tm1`
/// - >0: `tm2` > `tm1`
#[no_mangle]
pub extern "C" fn rs_profile_cmp(tm1: Proftime, tm2: Proftime) -> c_int {
    if tm1 == tm2 {
        return 0;
    }
    if rs_profile_signed(tm2.wrapping_sub(tm1)) < 0 {
        -1
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_zero() {
        assert_eq!(rs_profile_zero(), 0);
    }

    #[test]
    fn test_profile_add() {
        assert_eq!(rs_profile_add(100, 200), 300);
        assert_eq!(rs_profile_add(0, 0), 0);
        // Test wraparound
        assert_eq!(rs_profile_add(u64::MAX, 1), 0);
    }

    #[test]
    fn test_profile_sub() {
        assert_eq!(rs_profile_sub(300, 100), 200);
        assert_eq!(rs_profile_sub(100, 100), 0);
        // Test wraparound (underflow)
        assert_eq!(rs_profile_sub(100, 200), u64::MAX - 99);
    }

    #[test]
    fn test_profile_divide() {
        assert_eq!(rs_profile_divide(100, 2), 50);
        assert_eq!(rs_profile_divide(100, 3), 33); // rounds
        assert_eq!(rs_profile_divide(100, 0), 0);
        assert_eq!(rs_profile_divide(100, -1), 0);
    }

    #[test]
    fn test_profile_self() {
        // Normal case: self + total - children
        assert_eq!(rs_profile_self(10, 100, 30), 80); // 10 + 100 - 30 = 80

        // Edge case: total <= children (recursive calls)
        assert_eq!(rs_profile_self(10, 30, 30), 10);
        assert_eq!(rs_profile_self(10, 20, 30), 10);
    }

    #[test]
    fn test_profile_equal() {
        assert!(rs_profile_equal(100, 100));
        assert!(!rs_profile_equal(100, 200));
    }

    #[test]
    fn test_profile_signed() {
        // Positive values
        assert_eq!(rs_profile_signed(100), 100);
        assert_eq!(rs_profile_signed(i64::MAX as u64), i64::MAX);

        // Negative values (from wraparound)
        // The C code does: -(int64_t)(UINT64_MAX - tm)
        // When tm = u64::MAX: UINT64_MAX - UINT64_MAX = 0, so result is 0
        // When tm = u64::MAX - 99: UINT64_MAX - (UINT64_MAX - 99) = 99, so result is -99
        assert_eq!(rs_profile_signed(u64::MAX), 0);  // Edge case from C implementation
        assert_eq!(rs_profile_signed(u64::MAX - 99), -99);
    }

    #[test]
    fn test_profile_cmp() {
        // Equal times
        assert_eq!(rs_profile_cmp(100, 100), 0);

        // tm2 > tm1 (positive result)
        assert_eq!(rs_profile_cmp(100, 200), 1);

        // tm2 < tm1 (negative result)
        assert_eq!(rs_profile_cmp(200, 100), -1);
    }
}
