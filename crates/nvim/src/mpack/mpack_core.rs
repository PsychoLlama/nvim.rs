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
//! Everything here is over slices: [`TokBuf::read`] advances a `&[u8]`
//! cursor and hands a chunk back as a borrow of it, [`TokBuf::write`] fills
//! a `&mut [u8]`. A chunk being *written* is the one token whose bytes live
//! elsewhere, so the writer is handed them beside the token, again on every
//! call that resumes it.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![forbid(unsafe_code)]

use core::ffi::{c_int, c_uint};

use super::token::{self, Kind, MAX_TOKEN_LEN, Read, Tok};
use crate::types::mpack_token_type_t;

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
// The streaming state
// ---------------------------------------------------------------------------

/// `mpack_tokbuf_t`: a partial token, or the count of body bytes still owed.
/// See the module docs for which field means what in which direction.
#[derive(Clone, Copy, Debug)]
pub struct TokBuf {
    pending: [u8; MAX_TOKEN_LEN],
    pending_tok: Tok,
    ppos: usize,
    plen: usize,
    passthrough: u32,
}

impl Default for TokBuf {
    fn default() -> Self {
        Self::new()
    }
}

/// What [`read_step`] produced.
pub(crate) enum Step {
    /// A complete token.
    Token(Tok),
    /// The bytes consumed are a slice of a `str`/`bin`/`ext` body.
    Chunk,
    /// Not enough input; what there was has been buffered.
    Eof,
    /// The input is not msgpack.
    Error,
}

/// What [`TokBuf::read`] produced.
#[derive(Clone, Copy, Debug)]
pub enum Next<'a> {
    /// A complete token.
    Token(Tok),
    /// A slice of a `str`/`bin`/`ext` body, borrowed from the input.
    Chunk(&'a [u8]),
    /// Not enough input; what there was has been buffered.
    Eof,
    /// The input is not msgpack; nothing was consumed.
    Error,
}

impl TokBuf {
    /// A tokbuf with no partial token and no body outstanding.
    pub const fn new() -> Self {
        Self {
            pending: [0; MAX_TOKEN_LEN],
            pending_tok: Tok {
                kind: None,
                len: 0,
                lo: 0,
                hi: 0,
            },
            ppos: 0,
            plen: 0,
            passthrough: 0,
        }
    }

    /// Whether a token is parked half-written, so the next [`write`] resumes
    /// it whatever it is handed.
    ///
    /// [`write`]: Self::write
    pub fn is_parked(&self) -> bool {
        self.plen != 0
    }

    /// Read one token off the front of `input`, advancing it past what was
    /// consumed (`mpack_read`).
    ///
    /// [`Next::Eof`] means the input ran out mid-token: the remainder is
    /// buffered, so the next call resumes. [`Next::Error`] consumes nothing.
    #[inline]
    pub fn read<'a>(&mut self, input: &mut &'a [u8]) -> Next<'a> {
        let (step, consumed) = read_step(self, input);
        let (taken, rest) = input.split_at(consumed);
        *input = rest;
        match step {
            Step::Token(tok) => Next::Token(tok),
            Step::Chunk => Next::Chunk(taken),
            Step::Eof => Next::Eof,
            Step::Error => Next::Error,
        }
    }

    /// Write one token into `out` (`mpack_write`): answers the status and how
    /// many bytes went.
    ///
    /// `MPACK_EOF` means `out` filled first; the caller drains it and calls
    /// again, and the parked token wins over whatever it passes. A chunk
    /// token's body is `chunk`, which a resuming caller hands over again.
    pub fn write(&mut self, out: &mut [u8], tok: &Tok, chunk: &[u8]) -> (c_uint, usize) {
        debug_assert!(!out.is_empty());
        let tok = if self.plen != 0 {
            self.pending_tok
        } else {
            *tok
        };
        if tok.kind == Some(Kind::Chunk) {
            write_chunk(self, &tok, chunk, out)
        } else if self.plen != 0 {
            drain_pending(self, out)
        } else {
            write_step(self, &tok, out)
        }
    }
}

/// A tokbuf's pending bytes.
fn pending_bytes(tb: &TokBuf) -> [u8; MAX_TOKEN_LEN] {
    tb.pending
}

/// Copy as much of `input` into `pending` as the partial token still needs
/// (`mpack_rpending`). Answers how many bytes were taken, and whether the
/// token is now whole.
fn fill_pending(tb: &mut TokBuf, input: &[u8]) -> (usize, bool) {
    debug_assert!(tb.ppos < tb.plen, "nothing pending");
    let want = tb.plen.min(MAX_TOKEN_LEN) - tb.ppos;
    let count = want.min(input.len());
    tb.pending[tb.ppos..tb.ppos + count].copy_from_slice(&input[..count]);
    tb.ppos += count;
    (count, tb.ppos >= tb.plen)
}

/// One step of `mpack_read`, over a slice.
///
/// Answers the step and how many bytes of `input` it consumed. The C bumps
/// the caller's pointer in three different places for this; here every path
/// reports a count and [`TokBuf::read`] does it once.
#[inline]
pub(crate) fn read_step(tb: &mut TokBuf, input: &[u8]) -> (Step, usize) {
    if input.is_empty() {
        return (Step::Eof, 0);
    }

    if tb.passthrough != 0 {
        let len = tb
            .passthrough
            .min(u32::try_from(input.len()).unwrap_or(u32::MAX));
        tb.passthrough -= len;
        return (Step::Chunk, len as usize);
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
/// parked in `pending` and [`drain_pending`] hands over the rest.
fn write_step(tb: &mut TokBuf, tok: &Tok, out: &mut [u8]) -> (c_uint, usize) {
    let Some(encoded) = token::encode_token(tok) else {
        return (MPACK_ERROR, 0);
    };
    let bytes = encoded.as_bytes();
    tb.pending[..bytes.len()].copy_from_slice(bytes);
    let count = bytes.len().min(out.len());
    out[..count].copy_from_slice(&bytes[..count]);
    if count == bytes.len() {
        (MPACK_OK, count)
    } else {
        tb.plen = bytes.len();
        tb.ppos = count;
        tb.pending_tok = *tok;
        (MPACK_EOF, count)
    }
}

/// Hand over the rest of a parked encoding (`mpack_wpending`).
fn drain_pending(tb: &mut TokBuf, out: &mut [u8]) -> (c_uint, usize) {
    debug_assert!(tb.ppos < tb.plen, "nothing pending");
    let count = (tb.plen - tb.ppos).min(out.len());
    out[..count].copy_from_slice(&tb.pending[tb.ppos..tb.ppos + count]);
    tb.ppos += count;
    if tb.ppos == tb.plen {
        tb.plen = 0;
        (MPACK_OK, count)
    } else {
        (MPACK_EOF, count)
    }
}

/// Copy the next slice of a `str`/`bin`/`ext` body out of `body`, which is
/// the whole of it. `ppos` counts how much of *this* chunk has gone, and
/// `plen` holds the chunk's length while it is unfinished.
fn write_chunk(tb: &mut TokBuf, tok: &Tok, body: &[u8], out: &mut [u8]) -> (c_uint, usize) {
    if tb.plen == 0 {
        tb.ppos = 0;
    }
    let written = tb.ppos;
    let outstanding = tok.len as usize - written;
    let count = outstanding.min(out.len());
    out[..count].copy_from_slice(&body[written..written + count]);
    tb.ppos += count;
    if count == outstanding {
        tb.plen = 0;
        (MPACK_OK, count)
    } else {
        tb.plen = tok.len as usize;
        tb.pending_tok = *tok;
        (MPACK_EOF, count)
    }
}

/// Decode a single token with no buffering (`mpack_rtoken`): the whole token
/// must be present. Advances `input` past what was used and answers the
/// status beside the token.
///
/// The RPC framer uses this to peek at a redraw batch and an extension's
/// payload, where the input is a complete buffer by construction. A short
/// token answers `MPACK_EOF` with its type code consumed, as the C did; an
/// invalid byte answers `MPACK_ERROR`, consumed too.
#[inline]
pub fn read_token(input: &mut &[u8]) -> (c_uint, Tok) {
    match token::decode_token(input) {
        Read::Done { tok, used } => {
            *input = &input[used..];
            (MPACK_OK, tok)
        }
        Read::Partial { .. } => {
            *input = &input[1..];
            (MPACK_EOF, Tok::default())
        }
        Read::Empty => (MPACK_EOF, Tok::default()),
        Read::Invalid => {
            *input = &input[1..];
            (MPACK_ERROR, Tok::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokbuf() -> TokBuf {
        TokBuf::new()
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
                    Step::Chunk => out.push(Tok::new(Kind::Chunk, used as u32, 0, 0)),
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
                    Step::Chunk => out.push(Tok::new(Kind::Chunk, used as u32, 0, 0)),
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
        assert!(matches!(step, Step::Chunk));
        assert_eq!((used, tb.passthrough), (3, 2));
        let (step, used) = read_step(&mut tb, b"lo");
        assert!(matches!(step, Step::Chunk));
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
    fn a_chunk_is_written_from_the_body_it_is_handed_again() {
        let mut tb = tokbuf();
        let body = b"hello";
        let tok = Tok::new(Kind::Chunk, 5, 0, 0);
        let mut out = [0u8; 3];
        assert_eq!(tb.write(&mut out, &tok, body), (MPACK_EOF, 3));
        assert!(tb.is_parked());
        // The parked token wins over the argument.
        let mut rest = [0u8; 8];
        assert_eq!(tb.write(&mut rest, &Tok::default(), body), (MPACK_OK, 2));
        assert_eq!((&out[..], &rest[..2]), (&b"hel"[..], &b"lo"[..]));
        assert!(!tb.is_parked());
    }

    #[test]
    fn read_hands_a_body_back_as_a_borrow_of_the_input() {
        let mut tb = tokbuf();
        let mut input: &[u8] = b"\xa3abc\x01";
        assert!(matches!(tb.read(&mut input), Next::Token(t) if t.kind == Some(Kind::Str)));
        assert!(matches!(tb.read(&mut input), Next::Chunk(b"abc")));
        assert!(matches!(tb.read(&mut input), Next::Token(t) if t.lo == 1));
        assert!(input.is_empty());
        assert!(matches!(tb.read(&mut input), Next::Eof));
        let mut bad: &[u8] = b"\xc1";
        assert!(matches!(tb.read(&mut bad), Next::Error));
        assert_eq!(bad.len(), 1);
    }

    #[test]
    fn read_token_consumes_the_type_code_of_a_short_token() {
        let mut input: &[u8] = b"\xcf\x00";
        assert_eq!(read_token(&mut input).0, MPACK_EOF);
        assert_eq!(input, b"\x00");
        let mut input: &[u8] = b"\x07\x08";
        let (status, tok) = read_token(&mut input);
        assert_eq!((status, tok.lo, input), (MPACK_OK, 7, &b"\x08"[..]));
    }
}
