//! `mpack_core.c`: the streaming half of the token codec.
//!
//! [`super::token`] turns bytes into a [`Tok`] and back with no state at all.
//! This module adds the state: a `mpack_tokbuf_t` that can be handed a byte
//! at a time and still produce whole tokens, which is what makes the RPC
//! framer work over a socket that splits wherever it likes.
//!
//! Two pieces of state, never both live:
//!
//! * `pending`/`ppos`/`plen` — a partial token. Reading, it holds the bytes
//!   of a token seen so far and `plen` is how many it needs; writing, it
//!   holds an encoded token and `plen` is how many bytes are left to hand
//!   over.
//! * `passthrough` — how many bytes of a `str`/`bin`/`ext` *body* are still
//!   owed. While it is non-zero every read answers a `MPACK_TOKEN_CHUNK`
//!   borrowing the caller's own buffer, so bodies are never copied.
//!
//! The exported functions keep the C signatures: `nvim/msgpack_rpc/unpacker`
//! and `nvim/eval/decode/msgpack` drive them from raw pointers they own, and
//! `test/unit/msgpack_spec.lua` sizes `Unpacker` (which embeds a tokbuf)
//! through the FFI. Each is a thin shim over a safe core below.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};

use super::token::{self, Kind, MAX_TOKEN_LEN, Read, Tok};
use crate::types::{
    mpack_tokbuf_t, mpack_token_s_data, mpack_token_t, mpack_token_type_t, mpack_uint32_t,
    mpack_value_t, size_t,
};

/// `mpack_read`/`mpack_write` status codes.
pub const MPACK_OK: c_uint = 0;
pub const MPACK_EOF: c_uint = 1;
pub const MPACK_ERROR: c_uint = 2;

pub const MPACK_TOKEN_NIL: mpack_token_type_t = Kind::Nil as mpack_token_type_t;
pub const MPACK_TOKEN_BOOLEAN: mpack_token_type_t = Kind::Boolean as mpack_token_type_t;
pub const MPACK_TOKEN_UINT: mpack_token_type_t = Kind::Uint as mpack_token_type_t;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = Kind::Sint as mpack_token_type_t;
pub const MPACK_TOKEN_FLOAT: mpack_token_type_t = Kind::Float as mpack_token_type_t;
pub const MPACK_TOKEN_CHUNK: mpack_token_type_t = Kind::Chunk as mpack_token_type_t;
pub const MPACK_TOKEN_ARRAY: mpack_token_type_t = Kind::Array as mpack_token_type_t;
pub const MPACK_TOKEN_MAP: mpack_token_type_t = Kind::Map as mpack_token_type_t;
pub const MPACK_TOKEN_BIN: mpack_token_type_t = Kind::Bin as mpack_token_type_t;
pub const MPACK_TOKEN_STR: mpack_token_type_t = Kind::Str as mpack_token_type_t;
pub const MPACK_TOKEN_EXT: mpack_token_type_t = Kind::Ext as mpack_token_type_t;

pub const MPACK_MAX_TOKEN_LEN: c_int = MAX_TOKEN_LEN as c_int;

// ---------------------------------------------------------------------------
// The C token, and the bridge to the safe one
// ---------------------------------------------------------------------------

/// A `mpack_token_t` payload built from the whole 64-bit value.
///
/// Writing both halves matters: the C's `mpack_blob` sets only `ext_type`,
/// which leaves four bytes of the union uninitialised in every `map`,
/// `array`, `str` and `bin` token it makes. `ext_type` is `lo`'s four bytes,
/// so filling `hi` as well costs nothing and makes the token a value the
/// whole tree can copy.
pub const fn value_data(lo: mpack_uint32_t, hi: mpack_uint32_t) -> mpack_token_s_data {
    mpack_token_s_data {
        value: mpack_value_t { lo, hi },
    }
}

/// A zeroed token, for a caller that needs one before it has one.
pub const fn empty_token() -> mpack_token_t {
    mpack_token_t {
        type_0: 0,
        length: 0,
        data: value_data(0, 0),
    }
}

/// The C token as a [`Tok`].
///
/// A `MPACK_TOKEN_CHUNK` has a pointer in that union, not a value, so it
/// answers a payload-less token rather than reading the pointer as integers.
/// No caller of this needs a chunk's payload: both directions handle chunks
/// before they reach the codec.
pub fn to_tok(c: &mpack_token_t) -> Tok {
    let kind = Kind::from_raw(c.type_0);
    let (lo, hi) = match kind {
        Some(Kind::Chunk) => (0, 0),
        // SAFETY: every non-chunk token in this tree is built through
        // `value_data`, so the union's value arm is fully initialised.
        _ => unsafe { (c.data.value.lo, c.data.value.hi) },
    };
    Tok {
        kind,
        len: c.length,
        lo,
        hi,
    }
}

/// The safe token as the C one.
pub const fn from_tok(t: &Tok) -> mpack_token_t {
    let type_0 = match t.kind {
        Some(kind) => kind as mpack_token_type_t,
        None => 0,
    };
    mpack_token_t {
        type_0,
        length: t.len,
        data: value_data(t.lo, t.hi),
    }
}

// ---------------------------------------------------------------------------
// The safe streaming core
// ---------------------------------------------------------------------------

/// What one [`read_step`] produced.
enum Step {
    /// A complete token.
    Token(Tok),
    /// The next `len` bytes of the input are a `str`/`bin`/`ext` body.
    Chunk(u32),
    /// Not enough input; what there was has been buffered.
    Eof,
    /// The input is not msgpack.
    Error,
}

/// A tokbuf's pending bytes. `pending` is a `[c_char; 9]`, which is signed
/// on every platform this builds for; nine bytes is cheap enough to copy
/// rather than reach for a cast.
fn pending_bytes(tb: &mpack_tokbuf_t) -> [u8; MAX_TOKEN_LEN] {
    let mut out = [0u8; MAX_TOKEN_LEN];
    for (slot, &byte) in out.iter_mut().zip(&tb.pending) {
        *slot = byte as u8;
    }
    out
}

/// Copy as much of `input` into `pending` as the partial token still needs
/// (`mpack_rpending`). Answers how many bytes were taken, and whether the
/// token is now whole.
fn fill_pending(tb: &mut mpack_tokbuf_t, input: &[u8]) -> (usize, bool) {
    debug_assert!(tb.ppos < tb.plen, "nothing pending");
    let want = tb.plen.min(MAX_TOKEN_LEN) - tb.ppos;
    let count = want.min(input.len());
    for (slot, &byte) in tb.pending[tb.ppos..].iter_mut().zip(&input[..count]) {
        *slot = byte as c_char;
    }
    tb.ppos += count;
    (count, tb.ppos >= tb.plen)
}

/// One step of `mpack_read`, over a slice.
///
/// Answers the step and how many bytes of `input` it consumed. The C bumps
/// the caller's pointer in three different places for this; here every path
/// reports a count and the shim does it once.
fn read_step(tb: &mut mpack_tokbuf_t, input: &[u8]) -> (Step, usize) {
    if input.is_empty() {
        return (Step::Eof, 0);
    }

    if tb.passthrough != 0 {
        let len = (tb.passthrough as usize).min(input.len()) as mpack_uint32_t;
        tb.passthrough -= len;
        return (Step::Chunk(len), len as usize);
    }

    // A token already half-buffered is completed from `pending`; a fresh one
    // is parsed out of the caller's buffer directly.
    let (decoded, buffered) = if tb.plen != 0 {
        let (taken, whole) = fill_pending(tb, input);
        if !whole {
            // Everything went into the buffer and it is still short.
            return (Step::Eof, input.len());
        }
        (
            token::decode_token(&pending_bytes(tb)[..tb.ppos]),
            tb.ppos - taken,
        )
    } else {
        (token::decode_token(input), 0)
    };

    match decoded {
        Read::Done { tok, used } => {
            tb.ppos = 0;
            tb.plen = 0;
            if tok.kind > Some(Kind::Map) {
                tb.passthrough = tok.len;
            }
            (Step::Token(tok), used - buffered)
        }
        Read::Partial { need } => {
            debug_assert_eq!(tb.plen, 0, "a buffered token cannot ask for more");
            tb.plen = need as usize + 1;
            tb.ppos = 0;
            let (taken, whole) = fill_pending(tb, input);
            debug_assert!(!whole, "a token that needed more should still need more");
            (Step::Eof, if whole { taken } else { input.len() })
        }
        // Unreachable: the empty input and the buffered-token cases are both
        // handled above, so the decoder always has at least one byte.
        Read::Empty => (Step::Eof, 0),
        Read::Invalid => (Step::Error, 0),
    }
}

/// One step of `mpack_write`, over a slice: encode `tok` into `out`, or as
/// much of it as fits.
///
/// Answers `(status, written)`. On a short buffer the whole encoding is
/// parked in `pending` and `mpack_wpending` hands over the rest.
fn write_step(tb: &mut mpack_tokbuf_t, tok: &Tok, out: &mut [u8]) -> (c_uint, usize) {
    let Some(encoded) = token::encode_token(tok) else {
        return (MPACK_ERROR, 0);
    };
    let bytes = encoded.as_bytes();
    for (slot, &byte) in tb.pending.iter_mut().zip(bytes) {
        *slot = byte as c_char;
    }
    let count = bytes.len().min(out.len());
    out[..count].copy_from_slice(&bytes[..count]);
    if count == bytes.len() {
        (MPACK_OK, count)
    } else {
        tb.plen = bytes.len();
        tb.ppos = count;
        tb.pending_tok = from_tok(tok);
        (MPACK_EOF, count)
    }
}

/// Hand over the rest of a parked encoding (`mpack_wpending`).
fn drain_pending(tb: &mut mpack_tokbuf_t, out: &mut [u8]) -> (c_uint, usize) {
    debug_assert!(tb.ppos < tb.plen, "nothing pending");
    let count = (tb.plen - tb.ppos).min(out.len());
    out[..count].copy_from_slice(&pending_bytes(tb)[tb.ppos..tb.ppos + count]);
    tb.ppos += count;
    if tb.ppos == tb.plen {
        tb.plen = 0;
        (MPACK_OK, count)
    } else {
        (MPACK_EOF, count)
    }
}

// ---------------------------------------------------------------------------
// The C entry points
// ---------------------------------------------------------------------------

/// Reset a tokbuf to "no partial token, no body outstanding".
///
/// # Safety
/// `tokbuf` must point at a writable `mpack_tokbuf_t`.
pub unsafe fn mpack_tokbuf_init(tokbuf: *mut mpack_tokbuf_t) {
    // The C leaves `pending`/`pending_tok` alone, on the grounds that
    // `ppos`/`plen` say nothing in them is meaningful yet. That makes the
    // struct un-copyable without reading uninitialised bytes — which
    // `mpack_parser_copy` and `mpack_rpc_session_copy` both do — so it is
    // written out here instead. Twenty-five bytes, once per session.
    unsafe {
        tokbuf.write(mpack_tokbuf_t {
            pending: [0; MAX_TOKEN_LEN],
            pending_tok: empty_token(),
            ppos: 0,
            plen: 0,
            passthrough: 0,
        });
    }
}

/// Read one token out of `*buf`, advancing it past what was consumed.
///
/// Answers `MPACK_OK` with `*tok` filled, `MPACK_EOF` if the input ran out
/// mid-token (the remainder is buffered, so the next call resumes), or
/// `MPACK_ERROR` for a byte that is not msgpack.
///
/// # Safety
/// `buf`/`buflen` must describe a readable slice, and `tokbuf`/`tok` must
/// point at writable objects.
pub unsafe fn mpack_read(
    tokbuf: *mut mpack_tokbuf_t,
    buf: *mut *const c_char,
    buflen: *mut size_t,
    tok: *mut mpack_token_t,
) -> c_int {
    let tokbuf = unsafe { &mut *tokbuf };
    let start = unsafe { *buf };
    let input = unsafe { core::slice::from_raw_parts(start.cast::<u8>(), *buflen) };
    debug_assert!(!start.is_null());

    let (step, consumed) = read_step(tokbuf, input);
    unsafe {
        *buf = start.add(consumed);
        *buflen -= consumed;
    }
    match step {
        Step::Token(parsed) => {
            unsafe { *tok = from_tok(&parsed) };
            MPACK_OK as c_int
        }
        Step::Chunk(len) => {
            unsafe {
                *tok = mpack_token_t {
                    type_0: MPACK_TOKEN_CHUNK,
                    length: len,
                    data: mpack_token_s_data { chunk_ptr: start },
                };
            }
            MPACK_OK as c_int
        }
        Step::Eof => MPACK_EOF as c_int,
        Step::Error => MPACK_ERROR as c_int,
    }
}

/// Write one token into `*buf`, advancing it past what was written.
///
/// Answers `MPACK_EOF` when the buffer filled first; the caller drains and
/// calls again with the same token, which the tokbuf remembers.
///
/// # Safety
/// `buf`/`buflen` must describe a writable slice, `tokbuf` a writable
/// tokbuf, and `t` a readable token whose chunk pointer (if it is a chunk)
/// spans `t->length` bytes.
pub unsafe fn mpack_write(
    tokbuf: *mut mpack_tokbuf_t,
    buf: *mut *mut c_char,
    buflen: *mut size_t,
    t: *const mpack_token_t,
) -> c_int {
    let tokbuf = unsafe { &mut *tokbuf };
    let start = unsafe { *buf };
    debug_assert!(!start.is_null() && unsafe { *buflen } != 0);
    let out = unsafe { core::slice::from_raw_parts_mut(start.cast::<u8>(), *buflen) };

    // A parked token wins over the argument: the caller is resuming.
    let c_tok = if tokbuf.plen != 0 {
        tokbuf.pending_tok
    } else {
        unsafe { *t }
    };
    let (status, written) = if c_tok.type_0 == MPACK_TOKEN_CHUNK {
        unsafe { write_chunk(tokbuf, &c_tok, out) }
    } else if tokbuf.plen != 0 {
        drain_pending(tokbuf, out)
    } else {
        write_step(tokbuf, &to_tok(&c_tok), out)
    };

    unsafe {
        *buf = start.add(written);
        *buflen -= written;
    }
    status as c_int
}

/// Copy the next slice of a `str`/`bin`/`ext` body out of the caller's own
/// storage. `ppos` counts how much of *this* chunk has gone, and `plen`
/// holds the chunk's length while it is unfinished.
///
/// # Safety
/// `tok` must be a chunk token whose `chunk_ptr` spans `tok->length` bytes.
unsafe fn write_chunk(
    tb: &mut mpack_tokbuf_t,
    tok: &mpack_token_t,
    out: &mut [u8],
) -> (c_uint, usize) {
    if tb.plen == 0 {
        tb.ppos = 0;
    }
    let written = tb.ppos;
    let outstanding = tok.length as usize - written;
    let count = outstanding.min(out.len());
    // SAFETY: the caller promises `chunk_ptr` spans the token's length.
    let body = unsafe {
        core::slice::from_raw_parts(tok.data.chunk_ptr.cast::<u8>(), tok.length as usize)
    };
    out[..count].copy_from_slice(&body[written..written + count]);
    tb.ppos += count;
    if count == outstanding {
        tb.plen = 0;
        (MPACK_OK, count)
    } else {
        tb.plen = tok.length as usize;
        tb.pending_tok = *tok;
        (MPACK_EOF, count)
    }
}

/// Decode a single token with no buffering: the whole token must be present.
///
/// `nvim/msgpack_rpc/unpacker` uses this to peek at a message's header
/// fields, where the input is a complete buffer by construction.
///
/// # Safety
/// `buf`/`buflen` must describe a readable slice and `tok` a writable token.
pub unsafe fn mpack_rtoken(
    buf: *mut *const c_char,
    buflen: *mut size_t,
    tok: *mut mpack_token_t,
) -> c_int {
    let start = unsafe { *buf };
    let input = unsafe { core::slice::from_raw_parts(start.cast::<u8>(), *buflen) };
    match token::decode_token(input) {
        Read::Done { tok: parsed, used } => {
            unsafe {
                *tok = from_tok(&parsed);
                *buf = start.add(used);
                *buflen -= used;
            }
            MPACK_OK as c_int
        }
        Read::Partial { need } => {
            // The C leaves the type code consumed and reports the shortfall
            // through `tok->length`; `mpack_read` is the only caller that
            // reads it back.
            unsafe {
                (*tok).length = need;
                *buf = start.add(1);
                *buflen -= 1;
            }
            MPACK_EOF as c_int
        }
        Read::Empty => MPACK_EOF as c_int,
        Read::Invalid => {
            unsafe {
                *buf = start.add(1);
                *buflen -= 1;
            }
            MPACK_ERROR as c_int
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokbuf() -> mpack_tokbuf_t {
        mpack_tokbuf_t {
            pending: [0; MAX_TOKEN_LEN],
            pending_tok: empty_token(),
            ppos: 0,
            plen: 0,
            passthrough: 0,
        }
    }

    /// Feed `input` a byte at a time and collect every token that emerges,
    /// which is the contract the RPC framer depends on.
    fn dribble(input: &[u8]) -> Vec<Tok> {
        let mut tb = tokbuf();
        let mut out = Vec::new();
        for i in 0..input.len() {
            let mut rest = &input[i..i + 1];
            while !rest.is_empty() {
                let (step, used) = read_step(&mut tb, rest);
                rest = &rest[used..];
                match step {
                    Step::Token(tok) => out.push(tok),
                    Step::Chunk(len) => out.push(Tok::new(Kind::Chunk, len, 0, 0)),
                    Step::Eof => break,
                    Step::Error => panic!("unexpected error"),
                }
            }
        }
        out
    }

    #[test]
    fn a_token_split_across_calls_is_reassembled() {
        // [1, "ab", 4294967296]
        let msg = b"\x93\x01\xa2ab\xcf\x00\x00\x00\x01\x00\x00\x00\x00";
        let whole = {
            let mut tb = tokbuf();
            let mut out = Vec::new();
            let mut rest = &msg[..];
            while !rest.is_empty() {
                let (step, used) = read_step(&mut tb, rest);
                rest = &rest[used..];
                match step {
                    Step::Token(tok) => out.push(tok),
                    Step::Chunk(len) => out.push(Tok::new(Kind::Chunk, len, 0, 0)),
                    _ => panic!("unexpected"),
                }
            }
            out
        };
        assert_eq!(whole[0], Tok::new(Kind::Array, 3, 0, 0));
        assert_eq!(whole[2], Tok::new(Kind::Str, 2, 0, 0));
        assert_eq!(token::unpack_uint(&whole[4]), 1 << 32);

        // Byte at a time, the string body arrives as two one-byte chunks
        // instead of one two-byte chunk; everything else must match.
        let dribbled = dribble(msg);
        assert_eq!(dribbled.len(), whole.len() + 1);
        assert_eq!(dribbled[0], whole[0]);
        assert_eq!(dribbled[2], whole[2]);
        assert_eq!(token::unpack_uint(dribbled.last().unwrap()), 1 << 32);
    }

    #[test]
    fn a_body_is_handed_back_as_chunks_of_whatever_is_available() {
        let mut tb = tokbuf();
        let (step, used) = read_step(&mut tb, b"\xa5hello");
        assert!(matches!(step, Step::Token(t) if t == Tok::new(Kind::Str, 5, 0, 0)));
        assert_eq!((used, tb.passthrough), (1, 5));
        let (step, used) = read_step(&mut tb, b"hel");
        assert!(matches!(step, Step::Chunk(3)));
        assert_eq!((used, tb.passthrough), (3, 2));
        let (step, used) = read_step(&mut tb, b"lo");
        assert!(matches!(step, Step::Chunk(2)));
        assert_eq!((used, tb.passthrough), (2, 0));
    }

    #[test]
    fn a_token_wider_than_the_output_is_parked_and_resumed() {
        let mut tb = tokbuf();
        let tok = token::pack_uint(u64::MAX);
        let mut out = [0u8; 4];
        let (status, written) = write_step(&mut tb, &tok, &mut out);
        assert_eq!((status, written), (MPACK_EOF, 4));
        assert_eq!(out, [0xcf, 0xff, 0xff, 0xff]);
        let mut rest = [0u8; 16];
        let (status, written) = drain_pending(&mut tb, &mut rest);
        assert_eq!((status, written), (MPACK_OK, 5));
        assert_eq!(&rest[..5], &[0xff, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(tb.plen, 0);
    }

    #[test]
    fn an_unencodable_token_is_an_error_not_a_write() {
        let mut tb = tokbuf();
        let mut out = [0u8; 16];
        assert_eq!(
            write_step(&mut tb, &Tok::default(), &mut out),
            (MPACK_ERROR, 0)
        );
    }

    #[test]
    fn the_c_token_and_the_safe_one_agree() {
        for tok in [
            Tok::new(Kind::Nil, 0, 0, 0),
            Tok::new(Kind::Ext, 16, 42, 0),
            token::pack_sint(i64::MIN),
            token::pack_float(0.1),
        ] {
            assert_eq!(to_tok(&from_tok(&tok)), tok);
        }
        // An ext token's type code shares its four bytes with `lo`, which is
        // what lets `lmpack` read `data.ext_type` off a token this built.
        let c = from_tok(&Tok::new(Kind::Ext, 1, 42, 0));
        assert_eq!(unsafe { c.data.ext_type }, 42);
    }
}
