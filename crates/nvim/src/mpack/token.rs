//! The msgpack token: one safe value type, and the byte-level codec over it.
//!
//! Vendored `libmpack` keeps a token in a C struct whose payload is a union
//! of `{ mpack_value_t value; const char *chunk_ptr; int ext_type; }`, so
//! every read of it needs an unchecked union access, and every construction
//! of a `MAP`/`ARRAY` token leaves half of it uninitialised. [`Tok`] is the
//! same information spelled as an ordinary struct: the 64-bit payload is
//! always both halves of `mpack_value_t`, and an ext token's type code is
//! `lo` (which is where the C union puts it too, so the bridge in
//! [`super::mpack_core`] is a field-for-field copy).
//!
//! Everything here is pure: bytes in, [`Tok`] out, or [`Tok`] in and at most
//! [`MAX_TOKEN_LEN`] bytes out. There is no buffering, no cursor and no
//! caller state — [`super::mpack_core`] owns the streaming half. That is what
//! makes the wire format testable without a `lua_State` or a parser.
//!
//! `MPACK_TOKEN_CHUNK` has no [`Tok`] spelling: a chunk is a borrowed slice
//! of the caller's buffer, not a value, and both the reader and the writer
//! handle it before they reach this module.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![forbid(unsafe_code)]

/// The longest encoded token: a one-byte type code plus a 64-bit payload.
pub const MAX_TOKEN_LEN: usize = 9;

/// `mpack_token_type_t`, as a Rust enum. The discriminants are the wire
/// contract: `mpack_read` tests `tok.type > MPACK_TOKEN_MAP` to decide that a
/// token carries trailing bytes, and `lmpack` tests `< MPACK_TOKEN_BIN` for
/// "is a container", so the *order* matters as much as the values.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u32)]
pub enum Kind {
    Nil = 1,
    Boolean = 2,
    Uint = 3,
    Sint = 4,
    Float = 5,
    Chunk = 6,
    Array = 7,
    Map = 8,
    Bin = 9,
    Str = 10,
    Ext = 11,
}

impl Kind {
    /// The inverse of the `as u32` cast, for values arriving from C.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        Some(match raw {
            1 => Self::Nil,
            2 => Self::Boolean,
            3 => Self::Uint,
            4 => Self::Sint,
            5 => Self::Float,
            6 => Self::Chunk,
            7 => Self::Array,
            8 => Self::Map,
            9 => Self::Bin,
            10 => Self::Str,
            11 => Self::Ext,
            _ => return None,
        })
    }
}

/// One token, payload and all.
///
/// `len` is overloaded exactly as the C is: a byte count for
/// `str`/`bin`/`ext`, an item count for `array`/`map`, and for the scalars
/// the *width the value was encoded at* — which is what
/// `test/unit/msgpack_spec.lua` asserts about `mpack_pack_number`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Tok {
    pub kind: Option<Kind>,
    pub len: u32,
    /// Low half of `mpack_value_t`, and an ext token's type code.
    pub lo: u32,
    /// High half of `mpack_value_t`.
    pub hi: u32,
}

impl Tok {
    pub const fn new(kind: Kind, len: u32, lo: u32, hi: u32) -> Self {
        Self {
            kind: Some(kind),
            len,
            lo,
            hi,
        }
    }

    /// The payload read as one 64-bit word, high half first.
    pub const fn word(&self) -> u64 {
        ((self.hi as u64) << 32) | self.lo as u64
    }

    /// A token whose payload is a single byte, `mpack_byte()`'s job.
    const fn byte(kind: Kind, len: u32, b: u8) -> Self {
        Self::new(kind, len, b as u32, 0)
    }
}

/// What [`decode_token`] found at the head of a buffer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Read {
    /// A whole token, and how many bytes it spanned.
    Done { tok: Tok, used: usize },
    /// A type code was read but its payload was short: `need` *more* bytes
    /// are required after it. `mpack_read` buffers `need + 1` bytes (the
    /// type code included) and retries.
    Partial { need: u32 },
    /// Nothing to read at all.
    Empty,
    /// A byte that is not a msgpack type code (only `0xc1`).
    Invalid,
}

/// Decode the token at the head of `buf`.
///
/// The C is `mpack_rtoken`, minus its pointer bumping: the caller advances by
/// `used` instead. `1 << (t - range_start)` is the C's `TLEN` macro, the
/// payload width implied by a type code's position in its run.
pub fn decode_token(buf: &[u8]) -> Read {
    let Some((&t, rest)) = buf.split_first() else {
        return Read::Empty;
    };
    match t {
        // positive fixint
        0x00..=0x7f => Read::Done {
            tok: Tok::byte(Kind::Uint, 1, t),
            used: 1,
        },
        // fixmap / fixarray / fixstr carry their length in the type code
        0x80..=0x8f => fixed(Kind::Map, (t & 0xf) as u32),
        0x90..=0x9f => fixed(Kind::Array, (t & 0xf) as u32),
        0xa0..=0xbf => fixed(Kind::Str, (t & 0x1f) as u32),
        0xc0 => Read::Done {
            tok: Tok::byte(Kind::Nil, 0, 0),
            used: 1,
        },
        0xc2 => Read::Done {
            tok: Tok::byte(Kind::Boolean, 1, 0),
            used: 1,
        },
        0xc3 => Read::Done {
            tok: Tok::byte(Kind::Boolean, 1, 1),
            used: 1,
        },
        0xc4..=0xc6 => blob(Kind::Bin, 1 << (t - 0xc4), rest),
        0xc7..=0xc9 => blob(Kind::Ext, 1 << (t - 0xc7), rest),
        0xca..=0xcb => value(Kind::Float, 1 << (t - 0xc8), rest),
        0xcc..=0xcf => value(Kind::Uint, 1 << (t - 0xcc), rest),
        0xd0..=0xd3 => value(Kind::Sint, 1 << (t - 0xd0), rest),
        // fixext: the payload width is in the type code, but the ext type
        // code still costs one byte.
        0xd4..=0xd8 => match rest.first() {
            None => Read::Partial { need: 1 },
            Some(&ext) => Read::Done {
                tok: Tok::new(Kind::Ext, 1 << (t - 0xd4), ext as u32, 0),
                used: 2,
            },
        },
        0xd9..=0xdb => blob(Kind::Str, 1 << (t - 0xd9), rest),
        0xdc..=0xdd => blob(Kind::Array, 1 << (t - 0xdb), rest),
        0xde..=0xdf => blob(Kind::Map, 1 << (t - 0xdd), rest),
        // negative fixint
        0xe0..=0xff => Read::Done {
            tok: Tok::byte(Kind::Sint, 1, t),
            used: 1,
        },
        _ => Read::Invalid,
    }
}

/// A container or string whose length is part of its type code.
const fn fixed(kind: Kind, len: u32) -> Read {
    Read::Done {
        tok: Tok {
            kind: Some(kind),
            len,
            lo: 0,
            hi: 0,
        },
        used: 1,
    }
}

/// A scalar whose `width` payload bytes follow the type code, big-endian.
///
/// `mpack_rvalue`: the bytes land in `lo` until four remain, at which point
/// what has been read so far shifts up into `hi`. A signed value whose top
/// bit is clear is not negative, and is retyped `Uint` — which is why
/// `mpack_unpack_sint` may assume the sign bit is set.
fn value(kind: Kind, width: u32, rest: &[u8]) -> Read {
    let w = width as usize;
    if rest.len() < w {
        return Read::Partial { need: width };
    }
    let mut tok = Tok::new(kind, width, 0, 0);
    for (i, &b) in rest[..w].iter().enumerate() {
        let remaining = width - 1 - i as u32;
        tok.lo |= (b as u32) << ((remaining % 4) * 8);
        if remaining == 4 {
            tok.hi = tok.lo;
            tok.lo = 0;
        }
    }
    if kind == Kind::Sint && !sign_bit_set(&tok) {
        tok.kind = Some(Kind::Uint);
    }
    Read::Done { tok, used: 1 + w }
}

/// The most significant bit of a `width`-byte signed payload.
const fn sign_bit_set(tok: &Tok) -> bool {
    match tok.len {
        8 => tok.hi >> 31 != 0,
        4 => tok.lo >> 31 != 0,
        2 => tok.lo >> 15 != 0,
        1 => tok.lo >> 7 != 0,
        _ => false,
    }
}

/// A `str`/`bin`/`ext`/`array`/`map` whose count is a `width`-byte
/// big-endian integer after the type code (`mpack_rblob`). `ext` spends one
/// further byte on its type code.
fn blob(kind: Kind, width: u32, rest: &[u8]) -> Read {
    let needs_ext = kind as u32 == Kind::Ext as u32;
    let required = width + u32::from(needs_ext);
    if rest.len() < required as usize {
        return Read::Partial { need: required };
    }
    let Read::Done { tok: count, .. } = value(Kind::Uint, width, rest) else {
        unreachable!("the length check above guarantees `width` bytes");
    };
    let ext = if needs_ext {
        rest[width as usize] as u32
    } else {
        0
    };
    Read::Done {
        tok: Tok {
            kind: Some(kind),
            len: count.lo,
            lo: ext,
            hi: 0,
        },
        used: 1 + required as usize,
    }
}

/// An encoded token: at most [`MAX_TOKEN_LEN`] bytes, on the stack.
#[derive(Clone, Copy, Debug)]
pub struct Encoded {
    buf: [u8; MAX_TOKEN_LEN],
    len: usize,
}

impl Encoded {
    fn new() -> Self {
        Self {
            buf: [0; MAX_TOKEN_LEN],
            len: 0,
        }
    }

    fn push(mut self, byte: u32) -> Self {
        self.buf[self.len] = (byte & 0xff) as u8;
        self.len += 1;
        self
    }

    fn push2(self, v: u32) -> Self {
        self.push(v >> 8).push(v)
    }

    fn push4(self, v: u32) -> Self {
        self.push(v >> 24).push(v >> 16).push(v >> 8).push(v)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

/// Encode `tok`, or `None` for a token no msgpack type covers (`Chunk`, and
/// a `Float` whose width is neither 4 nor 8) — the C's `MPACK_ERROR`.
pub fn encode_token(tok: &Tok) -> Option<Encoded> {
    let out = Encoded::new();
    Some(match tok.kind? {
        Kind::Nil => out.push(0xc0),
        Kind::Boolean => out.push(if tok.lo != 0 { 0xc3 } else { 0xc2 }),
        Kind::Uint => match (tok.hi, tok.lo) {
            (0, lo) if lo <= 0x7f => out.push(lo),
            (0, lo) if lo <= 0xff => out.push(0xcc).push(lo),
            (0, lo) if lo <= 0xffff => out.push(0xcd).push2(lo),
            (0, lo) => out.push(0xce).push4(lo),
            (hi, lo) => out.push(0xcf).push4(hi).push4(lo),
        },
        Kind::Sint => match sint_len(tok.hi, tok.lo) {
            8 => out.push(0xd3).push4(tok.hi).push4(tok.lo),
            4 => out.push(0xd2).push4(tok.lo),
            2 => out.push(0xd1).push2(tok.lo),
            // One byte, either as an `int 8` or as a negative fixint.
            _ if tok.lo < 0xffff_ffe0 => out.push(0xd0).push(tok.lo),
            _ => out.push(0x100u32.wrapping_add(tok.lo)),
        },
        Kind::Float => match tok.len {
            4 => out.push(0xca).push4(tok.lo),
            8 => out.push(0xcb).push4(tok.hi).push4(tok.lo),
            _ => return None,
        },
        Kind::Bin => match tok.len {
            len if len < 0x100 => out.push(0xc4).push(len),
            len if len < 0x10000 => out.push(0xc5).push2(len),
            len => out.push(0xc6).push4(len),
        },
        Kind::Str => match tok.len {
            len if len < 0x20 => out.push(0xa0 | len),
            len if len < 0x100 => out.push(0xd9).push(len),
            len if len < 0x10000 => out.push(0xda).push2(len),
            len => out.push(0xdb).push4(len),
        },
        Kind::Ext => encode_ext(out, tok.lo, tok.len),
        Kind::Array => match tok.len {
            len if len < 0x10 => out.push(0x90 | len),
            len if len < 0x10000 => out.push(0xdc).push2(len),
            len => out.push(0xdd).push4(len),
        },
        Kind::Map => match tok.len {
            len if len < 0x10 => out.push(0x80 | len),
            len if len < 0x10000 => out.push(0xde).push2(len),
            len => out.push(0xdf).push4(len),
        },
        Kind::Chunk => return None,
    })
}

/// The five `fixext` widths get their own type codes; everything else is
/// `ext 8/16/32`, which spell the length *before* the type code.
///
/// Upstream asserts `0 <= type < 0x80` here. It is a C `assert`, so a release
/// build truncates instead, and every caller in this tree already validates
/// (`lmpack` rejects anything outside 0..=127 with a Lua error). Keeping the
/// truncation makes the function total; the `debug_assert` keeps the contract
/// visible.
fn encode_ext(out: Encoded, ext: u32, len: u32) -> Encoded {
    debug_assert!(ext < 0x80, "ext type code out of range");
    match len {
        1 => out.push(0xd4).push(ext),
        2 => out.push(0xd5).push(ext),
        4 => out.push(0xd6).push(ext),
        8 => out.push(0xd7).push(ext),
        16 => out.push(0xd8).push(ext),
        len if len < 0x100 => out.push(0xc7).push(len).push(ext),
        len if len < 0x10000 => out.push(0xc8).push2(len).push(ext),
        len => out.push(0xc9).push4(len).push(ext),
    }
}

/// The narrowest signed width that carries the two's complement in `hi:lo`.
///
/// Every form below eight bytes stores `lo` alone, so `hi` has to *be* `lo`'s
/// sign extension for one of them to be usable. Upstream tests `lo` on its
/// own, both here and when choosing a token's `length`, so a value like
/// `-4294967297` (`hi = 0xfffffffe`, `lo = 0xffffffff`) reads as a negative
/// fixint and encodes as `-1`. See O-B15-10.
const fn sint_len(hi: u32, lo: u32) -> u32 {
    if hi != u32::MAX || lo < 0x8000_0000 {
        8
    } else if lo < 0xffff_8000 {
        4
    } else if lo < 0xffff_ff80 {
        2
    } else {
        1
    }
}

// ---------------------------------------------------------------------------
// Value conversions (`conv.c`)
// ---------------------------------------------------------------------------

/// 2^32, the boundary between `lo` and `hi`.
const POW2_32: f64 = 4294967296.0;
/// 2^64: one past the largest msgpack integer.
const POW2_64: f64 = 18446744073709551616.0;

/// `mpack_pack_uint`.
pub const fn pack_uint(v: u64) -> Tok {
    Tok::new(Kind::Uint, 0, v as u32, (v >> 32) as u32)
}

/// `mpack_pack_sint`: negatives are stored as their two's complement, which
/// on a `u64` is just the bit pattern.
pub const fn pack_sint(v: i64) -> Tok {
    let mut tok = pack_uint(v as u64);
    if v < 0 {
        tok.kind = Some(Kind::Sint);
    }
    tok
}

/// `mpack_unpack_uint`.
pub const fn unpack_uint(tok: &Tok) -> u64 {
    tok.word()
}

/// `mpack_unpack_sint`: undo the two's complement without assuming the host
/// has one, masking to the token's own width first.
///
/// Upstream asserts `length <= sizeof(mpack_sintmax_t)`. Every token that
/// reaches here came off the wire, where the width is one of 1/2/4/8, so the
/// assert never fires; the `min` keeps the shift in range regardless.
pub const fn unpack_sint(tok: &Tok) -> i64 {
    let bits = if tok.len == 8 {
        tok.word()
    } else {
        tok.lo as u64
    };
    let width = if tok.len == 0 {
        1
    } else if tok.len > 8 {
        8
    } else {
        tok.len
    };
    let magnitude = (!bits & ((1u64 << (width * 8 - 1)) - 1)).wrapping_add(1);
    -(magnitude.wrapping_sub(1) as i64) - 1
}

/// `mpack_unpack_boolean`.
pub const fn unpack_boolean(tok: &Tok) -> bool {
    tok.lo != 0 || tok.hi != 0
}

/// `mpack_pack_float_fast`: the IEEE-754 bit pattern, split in halves. The C
/// reaches it through a union and then byte-swaps on a big-endian host; both
/// spellings mean "high half in `hi`".
pub fn pack_float(v: f64) -> Tok {
    if fits_single(v) {
        Tok::new(Kind::Float, 4, (v as f32).to_bits(), 0)
    } else {
        let bits = v.to_bits();
        Tok::new(Kind::Float, 8, bits as u32, (bits >> 32) as u32)
    }
}

/// `mpack_unpack_float_fast`.
pub fn unpack_float(tok: &Tok) -> f64 {
    if tok.len == 4 {
        f32::from_bits(tok.lo) as f64
    } else {
        f64::from_bits(tok.word())
    }
}

/// Whether a `f32` round-trips `v` exactly. False for NaN, which is what
/// sends NaN down the 8-byte arm.
fn fits_single(v: f64) -> bool {
    v as f32 as f64 == v
}

/// `mpack_unpack_number`: the value of an int, uint or float token as a
/// `double`.
///
/// The signed arm cannot use [`unpack_sint`]: the magnitude of `i64::MIN` is
/// not an `i64`. It negates the 32-bit halves instead and rebuilds the
/// `double` from them.
///
/// `hi == 0` distinguishes the two spellings a negative token can have.
/// Off the wire, a value narrower than 8 bytes leaves `hi` untouched and its
/// sign is implied by the token's width, so the complement has to be masked
/// to that width; a full 8-byte value carries the complement in both halves,
/// where `hi` is all ones for any ordinary negative.
///
/// Upstream asserts `length <= 4` in the first arm, which holds for every
/// token that comes off the wire (`mpack_rvalue` retypes a wide signed value
/// with a clear sign bit to `Uint`, so `hi == 0` implies a narrow one).
/// [`pack_number`] can build a token that breaks it, for a magnitude at or
/// past 2^64; clamping the width answers some wrong finite number there
/// instead of aborting, and `pack_number`'s round trip rejects it.
pub fn unpack_number(tok: &Tok) -> f64 {
    let (kind, mut hi, mut lo) = (tok.kind, tok.hi, tok.lo);
    if kind == Some(Kind::Float) {
        return unpack_float(tok);
    }
    let negative = kind == Some(Kind::Sint);
    if negative {
        if hi == 0 {
            lo = !lo & ((1u32 << (tok.len.clamp(1, 4) * 8 - 1)) - 1);
        } else {
            hi = !hi;
            lo = !lo;
        }
        lo = lo.wrapping_add(1);
        if lo == 0 {
            hi = hi.wrapping_add(1);
        }
    }
    let magnitude = lo as f64 + POW2_32 * hi as f64;
    if negative { -magnitude } else { magnitude }
}

/// `mpack_pack_number`: an integer token when `v` is one exactly, a float
/// token otherwise.
///
/// The round trip through [`unpack_number`] is the whole safety net: any `v`
/// the halves cannot represent — a fraction, an infinity, a NaN, anything at
/// or past 2^64 — fails it and falls through to [`pack_float`].
///
/// Upstream opens with `assert(v <= 9007199254740991. && v >= -9007…)`, a
/// double's exact-integer range. It is a C `assert`, dropped from a release
/// build, and the round trip below is what actually decides — so there is no
/// assertion here: the range is not an input contract, it is one of the
/// cases the fallback exists for. See O-B15-3.
pub fn pack_number(v: f64) -> Tok {
    let vabs = if v < 0.0 { -v } else { v };
    // Past the integer types the halves cannot be computed at all: `f64 as
    // u32` saturates, and for `v == 2^64` exactly the saturated pair rounds
    // back to `v`, so the round trip below would accept a token that is one
    // short. Rule the range out first. (NaN is not excluded here — it fails
    // the round trip, since it compares equal to nothing.)
    if vabs >= POW2_64 {
        return pack_float(v);
    }
    let mut tok = Tok::new(
        Kind::Uint,
        0,
        fmod_pow2_32(vabs) as u32,
        (vabs / POW2_32) as u32,
    );

    if v < 0.0 {
        // Two's complement, then the narrowest width that still round-trips.
        tok.kind = Some(Kind::Sint);
        tok.hi = !tok.hi;
        tok.lo = (!tok.lo).wrapping_add(1);
        if tok.lo == 0 {
            tok.hi = tok.hi.wrapping_add(1);
        }
        tok.len = sint_len(tok.hi, tok.lo);
    } else {
        tok.len = match tok.lo {
            _ if tok.hi != 0 => 8,
            lo if lo > 0xffff => 4,
            lo if lo > 0xff => 2,
            _ => 1,
        };
    }

    if unpack_number(&tok) == v {
        tok
    } else {
        pack_float(v)
    }
}

/// `a` modulo 2^32, for an `a` that is already non-negative.
///
/// `f64 as u32` saturates in Rust where C truncates into undefined behaviour,
/// so an out-of-range `a` answers some finite garbage here instead of
/// whatever the host's `cvttsd2si` happened to leave behind. Either way the
/// caller's round-trip check rejects it.
fn fmod_pow2_32(a: f64) -> f64 {
    a - ((a / POW2_32) as u32 as f64 * POW2_32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encode a token and answer its bytes, for terse assertions.
    fn enc(tok: &Tok) -> Vec<u8> {
        encode_token(tok).expect("encodable").as_bytes().to_vec()
    }

    /// Decode a whole token, asserting the buffer was exactly consumed.
    fn dec(bytes: &[u8]) -> Tok {
        match decode_token(bytes) {
            Read::Done { tok, used } => {
                assert_eq!(used, bytes.len(), "trailing bytes");
                tok
            }
            other => panic!("expected a token, got {other:?}"),
        }
    }

    #[test]
    fn integer_widths_match_the_msgpack_spec() {
        assert_eq!(enc(&pack_uint(0)), [0x00]);
        assert_eq!(enc(&pack_uint(0x7f)), [0x7f]);
        assert_eq!(enc(&pack_uint(0x80)), [0xcc, 0x80]);
        assert_eq!(enc(&pack_uint(0x100)), [0xcd, 0x01, 0x00]);
        assert_eq!(enc(&pack_uint(0x10000)), [0xce, 0, 1, 0, 0]);
        assert_eq!(
            enc(&pack_uint(u64::MAX)),
            [0xcf, 255, 255, 255, 255, 255, 255, 255, 255]
        );
        assert_eq!(enc(&pack_sint(-1)), [0xff]);
        assert_eq!(enc(&pack_sint(-32)), [0xe0]);
        assert_eq!(enc(&pack_sint(-33)), [0xd0, 0xdf]);
        assert_eq!(enc(&pack_sint(-128)), [0xd0, 0x80]);
        assert_eq!(enc(&pack_sint(-129)), [0xd1, 0xff, 0x7f]);
        assert_eq!(enc(&pack_sint(i64::MIN)), [0xd3, 0x80, 0, 0, 0, 0, 0, 0, 0]);
    }

    /// The boundaries `test/unit/msgpack_spec.lua` pins, in Rust.
    #[test]
    fn pack_number_picks_the_narrowest_signed_width() {
        for (v, len) in [
            (-1.0, 1),
            (-128.0, 1),
            (-129.0, 2),
            (-32768.0, 2),
            (-32769.0, 4),
            (-2147483648.0, 4),
            (-2147483649.0, 8),
        ] {
            let tok = pack_number(v);
            assert_eq!(tok.kind, Some(Kind::Sint), "{v}");
            assert_eq!(tok.len, len, "{v}");
        }
    }

    /// The unsigned side, where each boundary sits one bit further out
    /// because there is no sign bit to pay for.
    #[test]
    fn pack_number_picks_the_narrowest_unsigned_width() {
        for (v, len) in [
            (0.0, 1),
            (127.0, 1),
            (255.0, 1),
            (256.0, 2),
            (65535.0, 2),
            (65536.0, 4),
            (4294967295.0, 4),
            (4294967296.0, 8),
        ] {
            let tok = pack_number(v);
            assert_eq!(tok.kind, Some(Kind::Uint), "{v}");
            assert_eq!(tok.len, len, "{v}");
        }
    }

    #[test]
    fn containers_and_strings_use_the_short_form_when_they_fit() {
        assert_eq!(enc(&Tok::new(Kind::Str, 31, 0, 0)), [0xbf]);
        assert_eq!(enc(&Tok::new(Kind::Str, 32, 0, 0)), [0xd9, 32]);
        assert_eq!(enc(&Tok::new(Kind::Bin, 1, 0, 0)), [0xc4, 1]);
        assert_eq!(enc(&Tok::new(Kind::Array, 15, 0, 0)), [0x9f]);
        assert_eq!(enc(&Tok::new(Kind::Array, 16, 0, 0)), [0xdc, 0, 16]);
        assert_eq!(enc(&Tok::new(Kind::Map, 15, 0, 0)), [0x8f]);
        assert_eq!(enc(&Tok::new(Kind::Map, 0x10000, 0, 0)), [0xdf, 0, 1, 0, 0]);
        assert_eq!(enc(&Tok::new(Kind::Ext, 4, 7, 0)), [0xd6, 7]);
        assert_eq!(enc(&Tok::new(Kind::Ext, 3, 7, 0)), [0xc7, 3, 7]);
    }

    #[test]
    fn decode_inverts_encode_for_every_scalar_width() {
        for v in [
            0u64,
            1,
            0x7f,
            0x80,
            0xffff,
            0x1_0000,
            u32::MAX as u64,
            u64::MAX,
        ] {
            let tok = dec(&enc(&pack_uint(v)));
            assert_eq!(unpack_uint(&tok), v, "uint {v}");
        }
        for v in [
            -1i64,
            -32,
            -33,
            -128,
            -129,
            -32768,
            -32769,
            i32::MIN as i64,
            i64::MIN,
        ] {
            let tok = dec(&enc(&pack_sint(v)));
            assert_eq!(tok.kind, Some(Kind::Sint), "sint {v}");
            assert_eq!(unpack_sint(&tok), v, "sint {v}");
        }
    }

    #[test]
    fn a_signed_token_with_a_clear_sign_bit_decodes_as_unsigned() {
        // int32 0x7fffffff: encoded signed, read back unsigned.
        let tok = dec(&[0xd2, 0x7f, 0xff, 0xff, 0xff]);
        assert_eq!(tok.kind, Some(Kind::Uint));
        assert_eq!(unpack_uint(&tok), 0x7fff_ffff);
    }

    #[test]
    fn floats_round_trip_through_both_widths() {
        for v in [
            0.5f64,
            -0.5,
            1.0,
            f64::MAX,
            f64::MIN_POSITIVE,
            f64::INFINITY,
        ] {
            let tok = pack_float(v);
            assert_eq!(unpack_float(&dec(&enc(&tok))), v, "{v}");
        }
        assert_eq!(pack_float(0.5).len, 4, "0.5 fits a f32");
        assert_eq!(pack_float(0.1).len, 8, "0.1 does not");
        assert!(unpack_float(&dec(&enc(&pack_float(f64::NAN)))).is_nan());
    }

    #[test]
    fn pack_number_falls_back_to_float_for_a_non_integer() {
        for v in [0.5f64, -0.5, 1.0 / 3.0, -1e-9] {
            assert_eq!(pack_number(v).kind, Some(Kind::Float), "{v}");
            assert_eq!(unpack_number(&pack_number(v)), v, "{v}");
        }
    }

    /// O-B15-3: upstream's `assert()` claimed 2^53 - 1 as the ceiling, but
    /// every double past it is already an integer and the msgpack integer
    /// types run to 2^64 - 1. None of these needs the float fallback.
    #[test]
    fn pack_number_keeps_integers_integral_past_2_53() {
        for v in [
            9007199254740991.0f64,
            9007199254740992.0,
            1e16,
            1e18,
            -1e16,
            -9.2e18,
        ] {
            let tok = pack_number(v);
            assert!(
                matches!(tok.kind, Some(Kind::Uint | Kind::Sint)),
                "{v} should pack as an integer, got {:?}",
                tok.kind
            );
            assert_eq!(unpack_number(&dec(&enc(&tok))), v, "{v} round trip");
        }
        // 1e16 is 0x2386F26FC10000, a uint 64 on the wire.
        assert_eq!(
            enc(&pack_number(1e16)),
            [0xcf, 0, 0x23, 0x86, 0xf2, 0x6f, 0xc1, 0, 0]
        );
    }

    /// Past 2^64, and for the values that are not finite integers at all,
    /// the round trip fails and a float token carries them instead.
    #[test]
    fn pack_number_falls_back_to_float_past_the_integer_types() {
        for v in [
            1e300f64,
            -1e300,
            1.9e19,
            -1.9e19,
            POW2_64,
            -POW2_64,
            f64::INFINITY,
        ] {
            assert_eq!(pack_number(v).kind, Some(Kind::Float), "{v}");
            assert_eq!(unpack_number(&dec(&enc(&pack_number(v)))), v, "{v}");
        }
        assert!(unpack_number(&dec(&enc(&pack_number(f64::NAN)))).is_nan());
    }

    /// O-B15-10: a negative whose magnitude spans both halves may not use a
    /// narrow form, however "small" its low word looks.
    #[test]
    fn a_negative_spanning_both_words_encodes_at_full_width() {
        assert_eq!(
            enc(&pack_number(-4294967297.0)),
            [0xd3, 255, 255, 255, 254, 255, 255, 255, 255]
        );
        assert_eq!(pack_number(-4294967297.0).len, 8);
        for v in [
            -4294967297.0f64,
            -4294967296.0,
            -4294967298.0,
            -1e15,
            -9007199254740991.0,
        ] {
            assert_eq!(unpack_number(&dec(&enc(&pack_number(v)))), v, "{v}");
        }
    }

    #[test]
    fn a_short_buffer_reports_how_much_more_it_needs() {
        assert_eq!(decode_token(&[]), Read::Empty);
        assert_eq!(decode_token(&[0xcf]), Read::Partial { need: 8 });
        assert_eq!(decode_token(&[0xcf, 1, 2]), Read::Partial { need: 8 });
        assert_eq!(decode_token(&[0xc4]), Read::Partial { need: 1 });
        assert_eq!(
            decode_token(&[0xc7]),
            Read::Partial { need: 2 },
            "ext 8: len + type"
        );
        assert_eq!(
            decode_token(&[0xd4]),
            Read::Partial { need: 1 },
            "fixext 1: type only"
        );
        assert_eq!(decode_token(&[0xc1]), Read::Invalid);
    }

    #[test]
    fn a_blob_header_reports_its_length_not_its_body() {
        let Read::Done { tok, used } = decode_token(b"\xa3abc") else {
            panic!()
        };
        assert_eq!((tok.kind, tok.len, used), (Some(Kind::Str), 3, 1));
        let Read::Done { tok, used } = decode_token(&[0xc7, 3, 42, 1, 2, 3]) else {
            panic!()
        };
        assert_eq!(
            (tok.kind, tok.len, tok.lo, used),
            (Some(Kind::Ext), 3, 42, 3)
        );
    }

    #[test]
    fn chunk_tokens_have_no_encoding() {
        assert!(encode_token(&Tok::new(Kind::Chunk, 4, 0, 0)).is_none());
        assert!(encode_token(&Tok::new(Kind::Float, 3, 0, 0)).is_none());
        assert!(encode_token(&Tok::default()).is_none());
    }
}
