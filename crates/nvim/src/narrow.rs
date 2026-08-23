#![forbid(unsafe_code)]

//! Deliberate narrowing, named once.
//!
//! The editor's 64-bit numbers -- a Vimscript `Number`, an API `Integer`, both
//! `i64` -- reach C interfaces that take an `int`, and the C narrows them with
//! a plain `(int)` cast at hundreds of sites. That narrowing is **observable**:
//! `winnr(0x1_0000_0000)` truncates to 0, which reads as "the current window",
//! and `nvim_win_set_height(0x1_0000_0002)` resizes to two lines. A `TryFrom`
//! that answered "out of range" instead would be a different editor, and one
//! that panicked would be a denial of service, since the value comes off the
//! RPC wire.
//!
//! So the narrowing stays, and this module is where it is spelled. **This file
//! deliberately does not carry the cast deny** -- it is the one place in the
//! tree allowed to write the cast, which is what lets every module that calls
//! it keep clippy's cast family switched on.
//!
//! Three classes live here, and each one names the C it stands in for:
//!
//! * **Wrapping** -- `number_as_int`, `msgpack_uint_as_u32`. The C truncates,
//!   the truncated value is well defined, and callers can see it.
//! * **Checked** -- `len_as_int`. The C's `(int)strlen(...)` yields a negative
//!   length past 2 GiB and every consumer then treats it as a byte count, so
//!   there is no behaviour to preserve; a panic replaces corruption.
//! * **Saturating** -- `float_as_i64`. The C's float-to-integer conversion is
//!   *undefined* out of range, and Rust's `as` is defined and saturating, so
//!   the port is already the better of the two. Naming it says so on purpose
//!   rather than by accident.
//!
//! Nothing else belongs here: a narrowing with a local answer -- a fallible one
//! the caller reports, a width the code above it just bounded -- wants
//! `TryFrom` at its own site.

use core::ffi::c_int;

/// A 64-bit editor number narrowed to a C `int`, exactly as upstream's
/// `(int)` casts do: the low 32 bits, wrapping.
///
/// The name says `number` because that is what the Vimscript side calls the
/// type; the API's `Integer` and the `varnumber_T` a builtin is handed are the
/// same `i64`, and they narrow the same way.
pub(crate) const fn number_as_int(n: i64) -> c_int {
    n as c_int
}

/// A msgpack unsigned integer narrowed to the 32 bits an RPC envelope carries
/// its message type and request id in, wrapping exactly as upstream's
/// `(uint32_t)` casts do.
///
/// Same argument as `number_as_int`: the value is whatever the peer put on the
/// wire, so rejecting it would be a different protocol and panicking on it
/// would be a denial of service. A message type of `0x1_0000_0000` decodes as
/// 0, i.e. a request, which is what upstream answers.
pub(crate) const fn msgpack_uint_as_u32(n: u64) -> u32 {
    n as u32
}

/// A byte length narrowed to the C `int` that carries it, as upstream's
/// `(int)strlen(...)` casts do -- but checked.
///
/// This is the one narrowing in the tree with nothing to preserve. Upstream
/// writes `int old_len = (int)strlen(ml_get(lnum));` and hands the result to
/// `inserted_bytes`, or stores it in a mark's column; past `INT_MAX` the cast
/// produces a *negative* byte count and every consumer downstream does column
/// arithmetic with it. There is no defined behaviour on the other side of that
/// boundary to be faithful to, so the port panics instead of corrupting a
/// buffer, and says which length overflowed.
///
/// # Panics
/// If `len` does not fit in a C `int` -- i.e. a single string of 2 GiB or more
/// reaching an interface that measures it in `int`.
pub(crate) fn len_as_int(len: usize) -> c_int {
    c_int::try_from(len).unwrap_or_else(|_| panic!("length {len} does not fit in a C int"))
}

/// A floating-point number narrowed to an `i64`, saturating.
///
/// Lua has one number type, so a table key that is used as an array index
/// arrives as an `f64` and upstream assigns it straight into an `int`. That
/// conversion is *undefined* in C once the value is out of the integer's range
/// (C99 6.3.1.4), and it is undefined for a NaN. Rust's `as` is defined for
/// both: it clamps to `i64::MIN`/`i64::MAX` and maps NaN to 0.
///
/// The saturation is therefore deliberate, and it is the reason this is a
/// named helper rather than a bare cast: the port is *stricter* than the C
/// here, and a future reader must not "fix" it into a `TryFrom` that reports an
/// error the C never had, nor into a wrap that reintroduces one.
pub(crate) fn float_as_i64(n: f64) -> i64 {
    n as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::varnumber_T;

    #[test]
    fn a_number_narrows_to_an_int_by_wrapping() {
        assert_eq!(number_as_int(0), 0);
        assert_eq!(number_as_int(1000), 1000);
        assert_eq!(number_as_int(-1), -1);
        // The load-bearing case: `winnr(0x1_0000_0000)` truncates to 0, which
        // the window resolver reads as "the current window". Anything that
        // rejected the value instead -- a `try_from`, a saturating narrow --
        // would answer differently.
        assert_eq!(number_as_int(0x1_0000_0000), 0);
        assert_eq!(number_as_int(0x1_0000_0001), 1);
        // And the sign wraps rather than saturating.
        assert_eq!(number_as_int(varnumber_T::from(c_int::MAX) + 1), c_int::MIN);
        assert_eq!(number_as_int(varnumber_T::MAX), -1);
    }

    #[test]
    fn a_msgpack_uint_narrows_to_32_bits_by_wrapping() {
        assert_eq!(msgpack_uint_as_u32(0), 0);
        assert_eq!(msgpack_uint_as_u32(2), 2);
        assert_eq!(msgpack_uint_as_u32(u64::from(u32::MAX)), u32::MAX);
        // The load-bearing case: a message type past 32 bits wraps into the
        // range the envelope check accepts, so `[0x1_0000_0000, id, m, args]`
        // is a *request*. Rejecting it would be a different protocol.
        assert_eq!(msgpack_uint_as_u32(0x1_0000_0000), 0);
        assert_eq!(msgpack_uint_as_u32(u64::MAX), u32::MAX);
    }

    #[test]
    fn a_length_narrows_to_an_int_when_it_fits() {
        assert_eq!(len_as_int(0), 0);
        assert_eq!(len_as_int(11), 11);
        let max = usize::try_from(c_int::MAX).expect("a 32-bit int fits a usize here");
        assert_eq!(len_as_int(max), c_int::MAX);
    }

    #[test]
    #[should_panic(expected = "does not fit in a C int")]
    fn a_length_past_an_int_panics_rather_than_going_negative() {
        // The whole point of the checked class: `(int)` would answer
        // `i32::MIN` here, and `inserted_bytes` would do column arithmetic
        // with a negative byte count.
        let over = usize::try_from(c_int::MAX).expect("a 32-bit int fits a usize here") + 1;
        let _ = len_as_int(over);
    }

    #[test]
    fn a_float_narrows_to_an_i64_by_saturating() {
        assert_eq!(float_as_i64(0.0), 0);
        assert_eq!(float_as_i64(3.0), 3);
        assert_eq!(float_as_i64(-3.0), -3);
        // Truncation towards zero, as the C's conversion does in range.
        assert_eq!(float_as_i64(2.9), 2);
        assert_eq!(float_as_i64(-2.9), -2);
        // Out of range and NaN are where the C is undefined and this is not.
        assert_eq!(float_as_i64(1e30), i64::MAX);
        assert_eq!(float_as_i64(-1e30), i64::MIN);
        assert_eq!(float_as_i64(f64::INFINITY), i64::MAX);
        assert_eq!(float_as_i64(f64::NAN), 0);
    }
}
