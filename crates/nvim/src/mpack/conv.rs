//! `conv.c`: values in and out of tokens.
//!
//! Every function here is a name the rest of the tree calls by; the
//! arithmetic all lives in [`super::token`], where it is pure and tested.
//! What this module adds is upstream's names for them, which `lmpack` and
//! the RPC framer call by.
//!
//! Two of upstream's entry points are gone rather than ported.
//! `mpack_pack_float_compat` / `mpack_unpack_float_compat` reassemble an
//! IEEE-754 double by repeated multiplication, for a host whose `double` is
//! not IEEE-754; `MPACK_USE_CONV` never selected them in this build and no
//! caller referenced them.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_double, c_int, c_uint};

use super::token::{self, Kind, Tok};
use crate::types::{mpack_sintmax_t, mpack_uint32_t, mpack_uintmax_t};

/// A token with no payload: `nil`, or a container/blob whose only datum is
/// its length.
const fn header(kind: Kind, len: mpack_uint32_t) -> Tok {
    Tok::new(kind, len, 0, 0)
}

pub const fn mpack_pack_nil() -> Tok {
    header(Kind::Nil, 0)
}

pub fn mpack_pack_boolean(v: c_uint) -> Tok {
    Tok::new(Kind::Boolean, 0, u32::from(v != 0), 0)
}

pub const fn mpack_pack_str(l: mpack_uint32_t) -> Tok {
    header(Kind::Str, l)
}

pub const fn mpack_pack_bin(l: mpack_uint32_t) -> Tok {
    header(Kind::Bin, l)
}

pub const fn mpack_pack_array(l: mpack_uint32_t) -> Tok {
    header(Kind::Array, l)
}

pub const fn mpack_pack_map(l: mpack_uint32_t) -> Tok {
    header(Kind::Map, l)
}

/// An `ext` header. The type code rides in `lo`.
pub const fn mpack_pack_ext(t: c_int, l: mpack_uint32_t) -> Tok {
    Tok::new(Kind::Ext, l, t.cast_unsigned(), 0)
}

/// A slice of a `str`/`bin`/`ext` body: its length only. The bytes are the
/// writer's to be handed beside it (see [`super::mpack_core::TokBuf::write`]).
pub const fn mpack_pack_chunk(l: mpack_uint32_t) -> Tok {
    header(Kind::Chunk, l)
}

/// The narrowest integer token that represents `v` exactly, or a float token
/// when none does.
pub(crate) fn mpack_pack_number(v: c_double) -> Tok {
    token::pack_number(v)
}

pub fn mpack_pack_float_fast(v: c_double) -> Tok {
    token::pack_float(v)
}

pub const fn mpack_unpack_boolean(t: Tok) -> bool {
    token::unpack_boolean(&t)
}

pub const fn mpack_unpack_uint(t: Tok) -> mpack_uintmax_t {
    token::unpack_uint(&t)
}

pub const fn mpack_unpack_sint(t: Tok) -> mpack_sintmax_t {
    token::unpack_sint(&t)
}

pub fn mpack_unpack_float_fast(t: Tok) -> c_double {
    token::unpack_float(&t)
}

pub fn mpack_unpack_number(t: Tok) -> c_double {
    token::unpack_number(&t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_carry_a_length_and_nothing_else() {
        for tok in [
            mpack_pack_str(7),
            mpack_pack_bin(7),
            mpack_pack_array(7),
            mpack_pack_map(7),
            mpack_pack_chunk(7),
        ] {
            assert_eq!((tok.len, tok.lo, tok.hi), (7, 0, 0));
        }
        assert_eq!(mpack_pack_ext(3, 7).lo, 3);
    }

    #[test]
    fn pack_number_round_trips() {
        for v in [0.0f64, 1.0, -1.0, 255.0, -32769.0, 0.5] {
            assert_eq!(mpack_unpack_number(mpack_pack_number(v)), v, "{v}");
        }
        // The width the token carries is the width the value is written at:
        // a token one byte too wide is a silent wire regression nothing else
        // notices.
        for (v, length) in [(-128.0, 1), (-129.0, 2), (255.0, 1), (256.0, 2)] {
            assert_eq!(mpack_pack_number(v).len, length, "{v}");
        }
        assert!(mpack_unpack_boolean(mpack_pack_boolean(1)));
        assert!(!mpack_unpack_boolean(mpack_pack_boolean(0)));
    }
}
