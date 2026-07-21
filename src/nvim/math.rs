//! Numeric helpers shared across the editor.
//!
//! The transpiled module also carried `xfpclassify`/`xisinf`/`xisnan`/
//! `xctz`/`xpopcount`, which only existed because C99 spells these as
//! macros; callers now use the std `f64`/`u64` methods directly.

use core::ffi::c_int;

/// Append a decimal digit to `value`; returns false (leaving `value`
/// untouched) if the result would not fit in an `int`.
pub fn vim_append_digit_int(value: &mut c_int, digit: c_int) -> bool {
    match value.checked_mul(10).and_then(|x| x.checked_add(digit)) {
        Some(x) => {
            *value = x;
            true
        }
        None => false,
    }
}

/// Clamp an `i64` into `int` range.
pub fn trim_to_int(x: i64) -> c_int {
    x.clamp(c_int::MIN as i64, c_int::MAX as i64) as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_digit_stops_at_int_max() {
        let mut v = c_int::MAX / 10;
        assert!(vim_append_digit_int(&mut v, 7));
        assert_eq!(v, c_int::MAX);
        let mut v = c_int::MAX / 10;
        assert!(!vim_append_digit_int(&mut v, 8));
        assert_eq!(v, c_int::MAX / 10);
    }

    #[test]
    fn trim_to_int_clamps() {
        assert_eq!(trim_to_int(42), 42);
        assert_eq!(trim_to_int(i64::MAX), c_int::MAX);
        assert_eq!(trim_to_int(i64::MIN), c_int::MIN);
    }
}
