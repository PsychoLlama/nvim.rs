//! Numeric helpers shared across the editor.
//!
//! The transpiled module also carried `xfpclassify`/`xisinf`/`xisnan`/
//! `xctz`/`xpopcount`, which only existed because C99 spells these as
//! macros; callers now use the std `f64`/`u64` methods directly.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

/// Append a decimal digit to `value`; returns false (leaving `value`
/// untouched) if the result would not fit in an `int`.
pub(crate) fn vim_append_digit_int(value: &mut c_int, digit: c_int) -> bool {
    match value.checked_mul(10).and_then(|x| x.checked_add(digit)) {
        Some(x) => {
            *value = x;
            true
        }
        None => false,
    }
}

/// Clamp an `i64` into `int` range.
pub(crate) fn trim_to_int(x: i64) -> c_int {
    // The clamp is the narrowing's proof.
    c_int::try_from(x.clamp(i64::from(c_int::MIN), i64::from(c_int::MAX)))
        .expect("clamped into `c_int` range")
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
