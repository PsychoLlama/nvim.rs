//! `conv.c`: values in and out of tokens.
//!
//! Every function here is a name the rest of the tree calls by; the
//! arithmetic all lives in [`super::token`], where it is pure and tested.
//! What this module adds is the `mpack_token_t` boundary — building the C
//! struct on the way out, reading it on the way in.
//!
//! Two of upstream's entry points are gone rather than ported.
//! `mpack_pack_float_compat` / `mpack_unpack_float_compat` reassemble an
//! IEEE-754 double by repeated multiplication, for a host whose `double` is
//! not IEEE-754; `MPACK_USE_CONV` never selected them in this build and no
//! caller referenced them.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_double, c_int, c_uint};

use super::mpack_core::{from_tok, to_tok};
use super::token::{self, Kind, Tok};
use crate::types::{
    mpack_sintmax_t, mpack_token_s_data, mpack_token_t, mpack_uint32_t, mpack_uintmax_t,
};

/// A token with no payload: `nil`, or a container/blob whose only datum is
/// its length.
fn header(kind: Kind, len: mpack_uint32_t) -> mpack_token_t {
    from_tok(&Tok::new(kind, len, 0, 0))
}

pub fn mpack_pack_nil() -> mpack_token_t {
    header(Kind::Nil, 0)
}

pub fn mpack_pack_boolean(v: c_uint) -> mpack_token_t {
    from_tok(&Tok::new(Kind::Boolean, 0, u32::from(v != 0), 0))
}

pub fn mpack_pack_str(l: mpack_uint32_t) -> mpack_token_t {
    header(Kind::Str, l)
}

pub fn mpack_pack_bin(l: mpack_uint32_t) -> mpack_token_t {
    header(Kind::Bin, l)
}

pub fn mpack_pack_array(l: mpack_uint32_t) -> mpack_token_t {
    header(Kind::Array, l)
}

pub fn mpack_pack_map(l: mpack_uint32_t) -> mpack_token_t {
    header(Kind::Map, l)
}

/// An `ext` header. The type code shares its four bytes with the value's low
/// half, so this is [`header`] with `lo` set.
pub fn mpack_pack_ext(t: c_int, l: mpack_uint32_t) -> mpack_token_t {
    from_tok(&Tok::new(Kind::Ext, l, t as mpack_uint32_t, 0))
}

/// A slice of a `str`/`bin`/`ext` body, borrowed from the caller.
///
/// The token holds `p` itself — this is the one token whose payload is a
/// pointer, which is why [`to_tok`] refuses to look inside a chunk.
/// `mpack_write` is the only reader, and it copies `l` bytes from it.
pub fn mpack_pack_chunk(p: *const c_char, l: mpack_uint32_t) -> mpack_token_t {
    mpack_token_t {
        type_0: Kind::Chunk as u32,
        length: l,
        data: mpack_token_s_data { chunk_ptr: p },
    }
}

/// The narrowest integer token that represents `v` exactly, or a float token
/// when none does.
///
/// Exported for `test/unit/msgpack_spec.lua`, which asserts the *width* this
/// picks at each signed boundary — the token's `length` is the byte count the
/// value will be encoded at, and picking it one too wide is a silent wire
/// regression that nothing else notices.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn mpack_pack_number(v: c_double) -> mpack_token_t {
    from_tok(&token::pack_number(v))
}

pub fn mpack_pack_float_fast(v: c_double) -> mpack_token_t {
    from_tok(&token::pack_float(v))
}

pub fn mpack_unpack_boolean(t: mpack_token_t) -> bool {
    token::unpack_boolean(&to_tok(&t))
}

pub fn mpack_unpack_uint(t: mpack_token_t) -> mpack_uintmax_t {
    token::unpack_uint(&to_tok(&t))
}

pub fn mpack_unpack_sint(t: mpack_token_t) -> mpack_sintmax_t {
    token::unpack_sint(&to_tok(&t))
}

pub fn mpack_unpack_float_fast(t: mpack_token_t) -> c_double {
    token::unpack_float(&to_tok(&t))
}

pub fn mpack_unpack_number(t: mpack_token_t) -> c_double {
    token::unpack_number(&to_tok(&t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chunk_token_keeps_the_caller_s_pointer() {
        let body = b"hello";
        let tok = mpack_pack_chunk(body.as_ptr().cast(), 5);
        assert_eq!(tok.type_0, Kind::Chunk as u32);
        assert_eq!(tok.length, 5);
        assert_eq!(unsafe { tok.data.chunk_ptr }, body.as_ptr().cast());
        // And `to_tok` must not read that pointer as a value.
        assert_eq!(to_tok(&tok), Tok::new(Kind::Chunk, 5, 0, 0));
    }

    #[test]
    fn headers_carry_a_length_and_nothing_else() {
        for tok in [
            mpack_pack_str(7),
            mpack_pack_bin(7),
            mpack_pack_array(7),
            mpack_pack_map(7),
        ] {
            assert_eq!(tok.length, 7);
            assert_eq!(unsafe { tok.data.value }.lo, 0);
        }
        assert_eq!(unsafe { mpack_pack_ext(3, 7).data.ext_type }, 3);
    }

    #[test]
    fn the_exported_pack_number_round_trips() {
        for v in [0.0f64, 1.0, -1.0, 255.0, -32769.0, 0.5] {
            assert_eq!(mpack_unpack_number(mpack_pack_number(v)), v, "{v}");
        }
        assert!(mpack_unpack_boolean(mpack_pack_boolean(1)));
        assert!(!mpack_unpack_boolean(mpack_pack_boolean(0)));
    }

    #[test]
    fn value_data_fills_both_halves() {
        let data = super::super::mpack_core::value_data(1, 2);
        assert_eq!(unsafe { data.value }.lo, 1);
        assert_eq!(unsafe { data.value }.hi, 2);
    }
}
