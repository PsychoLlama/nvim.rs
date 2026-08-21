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
//! it keep clippy's cast family switched on. Nothing else belongs here: a
//! narrowing that is *not* meant to wrap wants `TryFrom` at its own site.

use core::ffi::c_int;

/// A 64-bit editor number narrowed to a C `int`, exactly as upstream's
/// `(int)` casts do: the low 32 bits, wrapping.
///
/// The name says `number` because that is what the Vimscript side calls the
/// type; the API's `Integer` and the `varnumber_T` a builtin is handed are the
/// same `i64`, and they narrow the same way.
pub const fn number_as_int(n: i64) -> c_int {
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
pub const fn msgpack_uint_as_u32(n: u64) -> u32 {
    n as u32
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
}
