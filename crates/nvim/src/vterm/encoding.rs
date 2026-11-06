//! Character-set decoders for the terminal emulator's text path.
//!
//! A decoder pulls bytes off the parser's input and appends codepoints to a
//! caller-owned array, keeping any partially decoded sequence in a 16-byte
//! scratch area the caller supplies. The unit specs reach the same decoders
//! through `VTermEncoding` tables of C function pointers, which live with the
//! rest of the C surface in [`super::abi`]; a state machine's slot names its
//! set by which of those tables it holds.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![forbid(unsafe_code)]

use crate::types::{VTermEncoding, VTermEncodingType, uint32_t};
use crate::vterm::abi::{ENCODING_DEC_DRAWING, ENCODING_USASCII, ENCODING_UTF8};
use core::ffi::c_int;
use core::ptr::NonNull;

pub const ENC_UTF8: VTermEncodingType = 0;
pub const ENC_SINGLE_94: VTermEncodingType = 1;

/// Emitted in place of any byte sequence a decoder rejects.
const UNICODE_INVALID: u32 = 0xfffd;

/// The caller's codepoint array plus the count written into it so far.
///
/// A single input byte can append two codepoints — a replacement for an
/// abandoned sequence, then the byte itself — and upstream did that without
/// re-checking the bound, so a full array could be written one element past
/// its end. Pushes here stop at capacity instead.
pub(crate) struct Codepoints<'a> {
    pub(crate) out: &'a mut [uint32_t],
    pub(crate) len: usize,
}

impl Codepoints<'_> {
    fn is_full(&self) -> bool {
        self.len >= self.out.len()
    }

    fn push(&mut self, cp: u32) {
        if let Some(slot) = self.out.get_mut(self.len) {
            *slot = cp;
            self.len += 1;
        }
    }
}

/// A partially decoded UTF-8 sequence, carried between `decode` calls.
///
/// Lives in `VTermEncodingInstance::data`, a 16-byte inline buffer, so the
/// layout is dictated by that.
struct Utf8Decoder {
    /// Continuation bytes still expected for the sequence in progress.
    bytes_remaining: c_int,
    /// Total length of the sequence in progress, for the overlong check.
    bytes_total: c_int,
    /// Codepoint accumulated so far.
    this_cp: c_int,
}

impl Utf8Decoder {
    /// The decoder a slot's scratch holds: three `int`s, as libvterm laid
    /// them out.
    fn load(data: &[u8; 16]) -> Self {
        let int =
            |at: usize| c_int::from_ne_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
        Utf8Decoder {
            bytes_remaining: int(0),
            bytes_total: int(4),
            this_cp: int(8),
        }
    }

    fn store(&self, data: &mut [u8; 16]) {
        data[0..4].copy_from_slice(&self.bytes_remaining.to_ne_bytes());
        data[4..8].copy_from_slice(&self.bytes_total.to_ne_bytes());
        data[8..12].copy_from_slice(&self.this_cp.to_ne_bytes());
    }

    fn reset(&mut self) {
        self.bytes_remaining = 0;
        self.bytes_total = 0;
    }

    /// Begin a `total`-byte sequence whose lead byte contributed `bits`.
    ///
    /// A sequence already in progress is abandoned, and its replacement
    /// character emitted, exactly as upstream did.
    fn begin(&mut self, out: &mut Codepoints, total: c_int, bits: u8) {
        if self.bytes_remaining != 0 {
            out.push(UNICODE_INVALID);
        }
        self.this_cp = c_int::from(bits);
        self.bytes_total = total;
        self.bytes_remaining = total - 1;
    }

    /// The codepoint just completed, or the replacement character if it was
    /// overlong, a surrogate half, or one of the two noncharacters upstream
    /// singled out.
    fn finish(&self) -> u32 {
        let min = match self.bytes_total {
            2 => 0x80,
            3 => 0x800,
            4 => 0x10000,
            5 => 0x200000,
            6 => 0x4000000,
            _ => 0,
        };
        let cp = self.this_cp;
        if cp < min || (0xd800..=0xdfff).contains(&cp) || cp == 0xfffe || cp == 0xffff {
            UNICODE_INVALID
        } else {
            cp.cast_unsigned()
        }
    }

    /// Decode from `bytes[*pos..]` until the output fills, the input runs out,
    /// or a byte the parser owns (C0 or DEL) is reached — that byte is left
    /// unconsumed for the parser to handle.
    fn decode(&mut self, bytes: &[u8], pos: &mut usize, out: &mut Codepoints) {
        while *pos < bytes.len() && !out.is_full() {
            match bytes[*pos] {
                0x00..=0x1f | 0x7f => return,
                c @ 0x20..=0x7e => {
                    if self.bytes_remaining != 0 {
                        out.push(UNICODE_INVALID);
                    }
                    out.push(u32::from(c));
                    self.bytes_remaining = 0;
                }
                c @ 0x80..=0xbf => {
                    if self.bytes_remaining == 0 {
                        out.push(UNICODE_INVALID);
                    } else {
                        self.this_cp = (self.this_cp << 6) | c_int::from(c & 0x3f);
                        self.bytes_remaining -= 1;
                        if self.bytes_remaining == 0 {
                            out.push(self.finish());
                        }
                    }
                }
                c @ 0xc0..=0xdf => self.begin(out, 2, c & 0x1f),
                c @ 0xe0..=0xef => self.begin(out, 3, c & 0x0f),
                c @ 0xf0..=0xf7 => self.begin(out, 4, c & 0x07),
                c @ 0xf8..=0xfb => self.begin(out, 5, c & 0x03),
                c @ 0xfc..=0xfd => self.begin(out, 6, c & 0x01),
                // 0xfe and 0xff can't lead anything. Upstream deliberately
                // leaves a sequence in progress untouched here.
                0xfe..=0xff => out.push(UNICODE_INVALID),
            }
            *pos += 1;
        }
    }
}

/// Decode a single-byte 94-character set.
///
/// The high bit of the *first* byte of the run picks GL or GR, and that
/// choice is then applied to every byte of the run; anything outside the
/// printable range ends the run without being consumed. `table`, when given,
/// remaps printable positions — a zero entry means "unchanged".
fn decode_single_94(
    table: Option<&[uint32_t; 128]>,
    bytes: &[u8],
    pos: &mut usize,
    out: &mut Codepoints,
) {
    let is_gr = bytes.get(*pos).copied().unwrap_or(0) & 0x80;
    while *pos < bytes.len() && !out.is_full() {
        let c = bytes[*pos] ^ is_gr;
        if !(0x20..0x7f).contains(&c) {
            return;
        }
        let mapped = table.map_or(0, |t| t[usize::from(c)]);
        out.push(if mapped != 0 { mapped } else { u32::from(c) });
        *pos += 1;
    }
}

/// The character sets a slot can hold.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Charset {
    Utf8,
    UsAscii,
    /// The DEC Special Graphics set.
    DecDrawing,
}

impl Charset {
    /// The set `enc` is the table of. Every table a slot holds came from
    /// [`lookup`], so it is one of the three.
    pub(crate) fn of(enc: NonNull<VTermEncoding>) -> Self {
        if enc == NonNull::from(&ENCODING_UTF8) {
            Charset::Utf8
        } else if enc == NonNull::from(&ENCODING_DEC_DRAWING) {
            Charset::DecDrawing
        } else {
            Charset::UsAscii
        }
    }

    /// Gives a slot's scratch its starting state.
    pub(crate) fn init(self, data: &mut [u8; 16]) {
        if self == Charset::Utf8 {
            let mut decoder = Utf8Decoder::load(data);
            decoder.reset();
            decoder.store(data);
        }
    }

    /// Decodes from `bytes[*pos..]` into `out`, from its `len` on, until
    /// either runs out or a byte the parser owns is reached.
    pub(crate) fn decode_into(
        self,
        data: &mut [u8; 16],
        bytes: &[u8],
        pos: &mut usize,
        out: &mut Codepoints,
    ) {
        match self {
            Charset::Utf8 => {
                let mut decoder = Utf8Decoder::load(data);
                decoder.decode(bytes, pos, out);
                decoder.store(data);
            }
            Charset::UsAscii => decode_single_94(None, bytes, pos, out),
            Charset::DecDrawing => decode_single_94(Some(&DEC_DRAWING), bytes, pos, out),
        }
    }

    /// Decodes a run of `bytes` into `out`, answering how many codepoints it
    /// wrote and how many bytes it consumed.
    pub(crate) fn decode(
        self,
        data: &mut [u8; 16],
        bytes: &[u8],
        out: &mut [u32],
    ) -> (usize, usize) {
        let mut sink = Codepoints { out, len: 0 };
        let mut pos = 0;
        self.decode_into(data, bytes, &mut pos, &mut sink);
        (sink.len, pos)
    }
}

/// The DEC Special Graphics set: box drawing and a handful of symbols mapped
/// over the printable ASCII positions from `` ` `` onwards.
static DEC_DRAWING: [uint32_t; 128] = dec_drawing_chars();

const fn dec_drawing_chars() -> [uint32_t; 128] {
    let mut chars = [0; 128];
    let mut i = 0;
    // Positions 0x60..=0x7e; DEL stays unmapped.
    let glyphs: [uint32_t; 31] = [
        0x25c6, 0x2592, 0x2409, 0x240c, 0x240d, 0x240a, 0x00b0, 0x00b1, 0x2424, 0x240b, 0x2518,
        0x2510, 0x250c, 0x2514, 0x253c, 0x23ba, 0x23bb, 0x2500, 0x23bc, 0x23bd, 0x251c, 0x2524,
        0x2534, 0x252c, 0x2502, 0x2a7d, 0x2a7e, 0x03c0, 0x2260, 0x00a3, 0x00b7,
    ];
    while i < glyphs.len() {
        chars[0x60 + i] = glyphs[i];
        i += 1;
    }
    chars
}

/// The table of the set `designation` names, among those of kind `type_0`.
///
/// The table is a static nothing writes; C is handed it as the non-`const`
/// pointer libvterm declared.
pub(crate) fn lookup(type_0: VTermEncodingType, designation: u8) -> Option<NonNull<VTermEncoding>> {
    let table = match (type_0, designation) {
        (ENC_UTF8, b'u') => &ENCODING_UTF8,
        (ENC_SINGLE_94, b'0') => &ENCODING_DEC_DRAWING,
        (ENC_SINGLE_94, b'B') => &ENCODING_USASCII,
        _ => return None,
    };
    Some(NonNull::from(table))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decode `input` the way `state.rs` does: one output slot per input byte.
    fn utf8(input: &[u8]) -> Vec<u32> {
        let mut decoder = Utf8Decoder {
            bytes_remaining: 0,
            bytes_total: 0,
            this_cp: 0,
        };
        let mut buf = vec![0; input.len()];
        let mut out = Codepoints {
            out: &mut buf,
            len: 0,
        };
        let mut pos = 0;
        decoder.decode(input, &mut pos, &mut out);
        let len = out.len;
        buf.truncate(len);
        buf
    }

    fn single_94(table: Option<&[uint32_t; 128]>, input: &[u8]) -> Vec<u32> {
        let mut buf = vec![0; input.len()];
        let mut out = Codepoints {
            out: &mut buf,
            len: 0,
        };
        let mut pos = 0;
        decode_single_94(table, input, &mut pos, &mut out);
        let len = out.len;
        buf.truncate(len);
        buf
    }

    #[test]
    fn utf8_decodes_ascii_and_multibyte() {
        assert_eq!(utf8(b"hi"), [0x68, 0x69]);
        assert_eq!(utf8("é".as_bytes()), [0xe9]);
        assert_eq!(utf8("€".as_bytes()), [0x20ac]);
        assert_eq!(utf8("😀".as_bytes()), [0x1f600]);
    }

    #[test]
    fn utf8_stops_at_control_bytes_without_consuming_them() {
        let input = b"ab\x07cd";
        let mut decoder = Utf8Decoder {
            bytes_remaining: 0,
            bytes_total: 0,
            this_cp: 0,
        };
        let mut buf = [0; 8];
        let mut out = Codepoints {
            out: &mut buf,
            len: 0,
        };
        let mut pos = 0;
        decoder.decode(input, &mut pos, &mut out);
        assert_eq!(pos, 2);
        assert_eq!(out.len, 2);
        // DEL ends a run the same way a C0 byte does.
        assert_eq!(utf8(b"a\x7fb"), [0x61]);
    }

    #[test]
    fn utf8_rejects_overlong_surrogate_and_noncharacter() {
        // 0xc0 0x80 encodes U+0000 in two bytes.
        assert_eq!(utf8(b"\xc0\x80"), [UNICODE_INVALID]);
        // 0xed 0xa0 0x80 is the surrogate half U+D800.
        assert_eq!(utf8(b"\xed\xa0\x80"), [UNICODE_INVALID]);
        assert_eq!(utf8(b"\xef\xbf\xbe"), [UNICODE_INVALID]);
        assert_eq!(utf8(b"\xef\xbf\xbf"), [UNICODE_INVALID]);
        // U+FFFD itself round-trips; only the two noncharacters are rejected.
        assert_eq!(utf8(b"\xef\xbf\xbd"), [UNICODE_INVALID]);
    }

    #[test]
    fn utf8_abandons_a_truncated_sequence() {
        // Lead byte then a printable: the sequence is replaced, the printable
        // still arrives.
        assert_eq!(utf8(b"\xc3a"), [UNICODE_INVALID, 0x61]);
        // A stray continuation byte with nothing in progress.
        assert_eq!(utf8(b"\x80"), [UNICODE_INVALID]);
        // 0xfe/0xff lead nothing.
        assert_eq!(utf8(b"\xfe\xff"), [UNICODE_INVALID, UNICODE_INVALID]);
    }

    #[test]
    fn utf8_resumes_across_calls() {
        let mut decoder = Utf8Decoder {
            bytes_remaining: 0,
            bytes_total: 0,
            this_cp: 0,
        };
        let mut buf = [0; 4];
        let mut out = Codepoints {
            out: &mut buf,
            len: 0,
        };
        let mut pos = 0;
        decoder.decode(b"\xe2\x82", &mut pos, &mut out);
        assert_eq!(out.len, 0);
        let mut pos = 0;
        decoder.decode(b"\xac", &mut pos, &mut out);
        assert_eq!(&buf[..1], [0x20ac]);
    }

    #[test]
    fn utf8_never_writes_past_the_output() {
        // A truncated sequence followed by a printable wants two slots for
        // one byte; upstream would have written one past a full array.
        let mut decoder = Utf8Decoder {
            bytes_remaining: 1,
            bytes_total: 2,
            this_cp: 3,
        };
        let mut buf = [0xdead_beef; 1];
        let mut out = Codepoints {
            out: &mut buf,
            len: 0,
        };
        let mut pos = 0;
        decoder.decode(b"a", &mut pos, &mut out);
        assert_eq!(out.len, 1);
        assert_eq!(buf, [UNICODE_INVALID]);
    }

    #[test]
    fn single_94_maps_gl_and_gr_alike() {
        assert_eq!(single_94(None, b"AB"), [0x41, 0x42]);
        // The first byte's high bit selects GR for the whole run.
        assert_eq!(single_94(None, b"\xc1\xc2"), [0x41, 0x42]);
        // ...and a run started in GL stops at the first high byte.
        assert_eq!(single_94(None, b"A\xc2"), [0x41]);
    }

    #[test]
    fn dec_drawing_maps_only_its_own_positions() {
        let table = dec_drawing_chars();
        // `q` is the horizontal line; `A` is outside the mapped range.
        assert_eq!(single_94(Some(&table), b"qA"), [0x2500, 0x41]);
        assert_eq!(table[usize::from(b'`')], 0x25c6);
        assert_eq!(table[usize::from(b'~')], 0x00b7);
        assert_eq!(table[0x7f], 0);
    }

    #[test]
    fn lookup_rejects_unknown_designations() {
        assert_eq!(lookup(ENC_UTF8, b'u').map(Charset::of), Some(Charset::Utf8));
        assert_eq!(
            lookup(ENC_SINGLE_94, b'0').map(Charset::of),
            Some(Charset::DecDrawing)
        );
        assert_eq!(
            lookup(ENC_SINGLE_94, b'B').map(Charset::of),
            Some(Charset::UsAscii)
        );
        assert!(lookup(ENC_UTF8, b'0').is_none());
        assert!(lookup(ENC_SINGLE_94, b'u').is_none());
        assert!(lookup(ENC_SINGLE_94, 0).is_none());
    }
}
