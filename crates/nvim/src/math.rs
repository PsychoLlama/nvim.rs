//! Numeric helpers shared across the editor.
//!
//! The transpiled module also carried `xfpclassify`/`xisinf`/`xisnan`/
//! `xctz`/`xpopcount`, which only existed because C99 spells these as
//! macros; callers now use the std `f64`/`u64` methods directly.

#![forbid(unsafe_code)]

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

/// splitmix64's finaliser: a bijection on `u64` after which every input bit
/// reaches every output bit. The editor's hashers of small, consecutive keys
/// (ids, block numbers, digests) use it to spread a key over the bits a
/// hash table probes, where the identity hash clusters and SipHash's setup
/// is not worth paying.
#[inline]
pub(crate) const fn splitmix64_mix(x: u64) -> u64 {
    let x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
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
    fn splitmix64_mix_is_the_reference_finaliser() {
        // splitmix64 seeded with 0: its first output is the finaliser of
        // one golden-ratio step (Vigna's reference implementation).
        assert_eq!(splitmix64_mix(0x9e37_79b9_7f4a_7c15), 0xe220_a839_7b1d_cdaf);
        assert_eq!(splitmix64_mix(0), 0);
    }

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
