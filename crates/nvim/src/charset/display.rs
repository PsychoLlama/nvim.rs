//! Rendering text for display: the width a character occupies on screen and
//! the `^X` / `<xx>` forms that stand in for one that cannot be shown.
//!
//! The `transchar*` family answers out of one shared buffer, so the
//! rendering itself is done as a *value* ([`render::Rendered`], built by
//! safe code in `transchar.rs`) and parked in the buffer at the very last
//! step. Everything that only needs the bytes — `transstr_buf`,
//! `trans_characters` — takes the value and never touches the buffer at
//! all.

#![forbid(unsafe_code)]

use crate::cstr;
use core::ffi::CStr;
use core::ffi::{c_char, c_int};

use crate::mbyte::{
    c_cluster_len, char_at, char_len, cluster_len, encode_char, mb_tolower, utf_char2cells,
    utf_char2len,
};
use crate::memory::XString;
use crate::option::get_fileformat;
use crate::option::vars::dy_flags;
use crate::types::{NUL, uint8_t};
use crate::winlayer::Buf;

use super::{
    CT_CELL_MASK, EOL_MAC, NL, TAB, chartab, chartab_initialized, kOptDyFlagUhex,
    transchar as render, vim_isprintc,
};
use crate::keycodes::K_SPECIAL;
use crate::pos::MAXCOL;

const CAR: c_int = 13;
const KS_ZERO: c_int = 255;
const KS_SPECIAL: c_int = 254;

/// A cursor over the characters of a C string's bytes.
///
/// The text is the string's bytes, and its end or a NUL in it ends the walk,
/// as the terminator did; every step is by the length the multibyte readers
/// measure, which never runs past either.
#[derive(Clone, Copy)]
struct Chars<'a> {
    text: &'a [u8],
    at: usize,
}

impl<'a> Chars<'a> {
    #[inline(always)]
    fn new(text: &'a [u8]) -> Self {
        Chars { text, at: 0 }
    }

    #[inline(always)]
    fn rest(self) -> &'a [u8] {
        self.text.get(self.at..).unwrap_or_default()
    }

    /// The byte under the cursor, a NUL at the end.
    #[inline(always)]
    fn byte(self) -> uint8_t {
        cstr::byte_at(self.text, self.at)
    }

    /// The length of the whole character under the cursor, composing
    /// characters included.
    #[inline(always)]
    fn char_len(self) -> usize {
        c_cluster_len(self.rest())
    }

    /// The length of just the code point under the cursor.
    #[inline(always)]
    fn code_len(self) -> usize {
        if self.byte() == 0 {
            0
        } else {
            char_len(self.rest())
        }
    }

    /// The code point under the cursor.
    #[inline(always)]
    fn code(self) -> c_int {
        char_at(self.rest())
    }

    /// The `n` bytes at the cursor, which the caller has measured.
    #[inline(always)]
    fn bytes(self, n: usize) -> &'a [uint8_t] {
        &self.text[self.at..self.at + n]
    }

    /// The cursor `n` bytes further on, which the caller has measured.
    #[inline(always)]
    fn skip(self, n: usize) -> Self {
        Chars {
            text: self.text,
            at: self.at + n,
        }
    }

    #[inline(always)]
    fn advance(&mut self, n: usize) {
        self.at += n;
    }
}

/// The display width of the byte `b` from the global table.
#[inline(always)]
fn table_cells(b: uint8_t) -> c_int {
    (chartab(b) & CT_CELL_MASK) as c_int
}

/// The display width of a byte, zero for a multibyte lead byte.
#[inline(always)]
fn byte_cells(b: c_int) -> c_int {
    if b >= 0x80 {
        return 0;
    }
    table_cells(b as uint8_t)
}

/// Replace every unprintable byte in `buf` with its display form, in place.
///
/// `bytes` is the whole buffer, the string NUL-terminated inside it. Stops
/// early rather than overflowing when the expansions no longer fit, leaving
/// the tail untranslated.
pub(crate) fn trans_characters_in(bytes: &mut [u8]) {
    let bufsize = bytes.len();
    // Bytes of the string still to be translated, and bytes left over after
    // it. Both are counted from the cursor, so their sum is invariant.
    let mut len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let mut room = bufsize as isize - len as isize;
    let mut at = 0usize;
    while bytes[at] != 0 {
        // The bytes past the terminator are the buffer's, not the string's,
        // and a cluster never crosses a NUL: it is not a continuation byte
        // and never combines.
        let mut step = cluster_len(&bytes[at..]);
        if step > 1 {
            // A multibyte character is left alone.
            len -= step;
        } else {
            let trs = render_byte(Buf::current_or_none(), bytes[at] as c_int);
            step = trs.len;
            if step > 1 {
                room -= step as isize - 1;
                if room <= 0 {
                    return;
                }
                // The tail, terminator included, slides up to make room.
                bytes.copy_within(at + 1..at + 1 + len, at + step);
            }
            bytes[at..at + step].copy_from_slice(&trs.bytes[..step]);
            len -= 1;
        }
        at += step;
    }
}

/// How many bytes [`transstr_into`] would write for `s`, excluding the NUL.
fn transstr_len_in(s: &[u8], untab: bool) -> usize {
    let mut cursor = Chars::new(s);
    let mut len = 0;
    loop {
        let byte = cursor.byte();
        if byte == 0 {
            return len;
        }
        let l = cursor.char_len();
        if l > 1 {
            if vim_isprintc(cursor.code()) {
                len += l;
            } else {
                // An unprintable multibyte character is spelled out one
                // codepoint at a time, composing characters included.
                let mut off = 0;
                while off < l {
                    let at = cursor.skip(off);
                    len += render::hex_form(at.code()).len;
                    off += at.code_len();
                }
            }
            cursor.advance(l);
        } else if byte as c_int == TAB && !untab {
            len += 1;
            cursor.advance(1);
        } else {
            let cells = byte_cells(byte as c_int);
            cursor.advance(1);
            // A zero width means the table has no entry: `<xx>` is four.
            len += if cells > 0 { cells } else { 4 } as usize;
        }
    }
}

/// Copy `s` into `out` with every unprintable character replaced by its
/// display form, NUL-terminated. Answers the number of bytes written.
///
/// `slen` bounds how much of `s` is read, `None` meaning all of it; a
/// character that starts inside the bound is taken whole. The output is
/// truncated at a character boundary rather than overrunning `out`, which
/// must be at least one byte: the last of them is the terminator's.
pub(crate) fn transstr_into(s: &[u8], slen: Option<usize>, out: &mut [u8], untab: bool) -> usize {
    // The last byte of the buffer belongs to the terminator.
    let limit = out.len() - 1;
    let mut cursor = Chars::new(s);
    let mut read = 0;
    let mut written = 0;
    while slen.is_none_or(|slen| read < slen) && cursor.byte() != 0 && written < limit {
        let l = cursor.char_len();
        if l > 1 {
            if written + l > limit {
                break;
            }
            if vim_isprintc(cursor.code()) {
                out[written..written + l].copy_from_slice(cursor.bytes(l));
                written += l;
            } else {
                let mut off = 0;
                while off < l {
                    let at = cursor.skip(off);
                    let hex = render::hex_form(at.code());
                    if written + hex.len > limit {
                        break;
                    }
                    out[written..written + hex.len].copy_from_slice(&hex.bytes[..hex.len]);
                    written += hex.len;
                    off += at.code_len();
                }
            }
            cursor.advance(l);
            read += l;
        } else if cursor.byte() as c_int == TAB && !untab {
            out[written] = cursor.byte();
            written += 1;
            cursor.advance(1);
            read += 1;
        } else {
            let tb = render_byte(Buf::current_or_none(), cursor.byte() as c_int);
            cursor.advance(1);
            read += 1;
            if written + tb.len > limit {
                break;
            }
            out[written..written + tb.len].copy_from_slice(&tb.bytes[..tb.len]);
            written += tb.len;
        }
    }
    debug_assert!(written <= limit, "buf_p <= buf_e");
    out[written] = NUL as uint8_t;
    written
}

/// [`transstr_into`] into a freshly allocated string, sized exactly.
pub(crate) fn transstr_owned(s: &[u8], untab: bool) -> XString {
    let len = transstr_len_in(s, untab);
    let mut out = vec![0u8; len + 1];
    let written = transstr_into(s, None, &mut out, untab);
    out.truncate(written);
    XString::from_bytes(&out)
}

/// `text` folded to lowercase, up to its first NUL.
///
/// With a `room`, the result is what fits a buffer of that many bytes: the
/// text is cut to `room - 1` bytes first, and a character whose lowercase
/// form is longer and no longer fits is left as it was.
pub(crate) fn foldcase(text: &[u8], room: Option<usize>) -> Vec<u8> {
    let mut len = room.map_or(text.len(), |room| text.len().min(room - 1));
    // The folding happens in a copy terminated where the C string was, so a
    // walk over it stops where upstream's did.
    let mut work: Vec<u8> = text[..len].to_vec();
    work.push(0);
    let mut i = 0;
    loop {
        let cursor = Chars::new(&work).skip(i);
        if cursor.byte() == 0 {
            break;
        }
        let c = cursor.code();
        let olen = cursor.code_len();
        let mut lc = mb_tolower(c);
        // Only ASCII and real multibyte characters fold; a lone Latin-1 byte
        // has no case in this encoding.
        if (c < 0x80 || olen > 1) && c != lc {
            let mut nlen = utf_char2len(lc) as usize;
            if nlen > olen && room.is_some_and(|room| len + nlen - olen >= room) {
                // No room to grow: keep the original character.
                lc = c;
                nlen = olen;
            }
            if olen != nlen {
                // The tail, terminator included, moves up or down.
                let tail = work[i + olen..].to_vec();
                work.truncate(i + nlen.min(olen));
                work.resize(i + nlen, 0);
                work.extend_from_slice(&tail);
                len = len + nlen - olen;
            }
            let mut folded = [0u8; 6];
            let n = encode_char(lc, &mut folded);
            work[i..i + n].copy_from_slice(&folded[..n]);
        }
        i += Chars::new(&work).skip(i).char_len();
    }
    work.truncate(work.iter().position(|&b| b == 0).unwrap_or(work.len()));
    work
}

/// A character's display form, NUL-terminated within its own storage.
///
/// Upstream answers a pointer into one shared static buffer, so a second
/// rendering invalidates the first — which the `msg_display` family walks
/// straight into, holding one rendering while it asks for the next.
pub(crate) type CharDisplay = [c_char; CHAR_DISPLAY_LEN];

/// How long a [`CharDisplay`] is.
pub(crate) const CHAR_DISPLAY_LEN: usize = render::MAX_LEN;

/// A rendering as its own NUL-terminated buffer.
#[inline(always)]
fn owned(rendered: &render::Rendered) -> CharDisplay {
    let mut out: CharDisplay = [0; render::MAX_LEN];
    // A rendering and its terminator are never longer than `MAX_LEN`.
    for (slot, &byte) in out.iter_mut().zip(&rendered.bytes[..=rendered.len]) {
        *slot = byte as c_char;
    }
    out
}

/// The display form of the unprintable byte `c`, as a value.
///
#[inline(always)]
fn render_nonprint(buffer: Option<Buf>, c: c_int) -> render::Rendered {
    let c = if c == NL {
        // A NUL is stored as a newline internally.
        NUL
    } else if c == CAR && buffer.is_some_and(|b| get_fileformat(b) == EOL_MAC) {
        NL
    } else {
        c
    };
    debug_assert!(c <= 0xff, "c <= 0xff");
    if dy_flags.get() & kOptDyFlagUhex != 0 || c > 0x7f {
        render::hex_form(c)
    } else {
        render::control_form(c)
    }
}

/// The display form of the character `c` as it would appear in `buffer`, as a
/// value.
#[inline(always)]
fn render_char(buffer: Option<Buf>, c: c_int) -> render::Rendered {
    // A negative code is one of the key-translation escapes; it renders as
    // its byte behind a `~@`.
    let (prefix, c) = if c < 0 {
        let (prefix, byte) = render::negative_form(c);
        (Some(prefix), byte)
    } else {
        (None, c)
    };
    // Before the tables exist, printable ASCII is all that can be trusted.
    let body = if (!chartab_initialized.get() && (' ' as c_int..='~' as c_int).contains(&c))
        || (c <= 0xff && vim_isprintc(c))
    {
        render::Rendered::literal(c as uint8_t)
    } else if c <= 0xff {
        render_nonprint(buffer, c)
    } else {
        render::hex_form(c)
    };
    match prefix {
        Some(prefix) => body.behind(prefix),
        None => body,
    }
}

/// [`render_char`] for a single byte: unlike it, a high byte is never taken
/// for a printable Latin-1 character.
#[inline(always)]
fn render_byte(buffer: Option<Buf>, c: c_int) -> render::Rendered {
    if c >= 0x80 {
        return render_nonprint(buffer, c);
    }
    render_char(buffer, c)
}

/// The display form of character `c`.
pub(crate) fn transchar(c: c_int) -> CharDisplay {
    transchar_buf(Buf::current_or_none(), c)
}

/// The display form of `c` as it would appear in `buffer` (which decides how a
/// carriage return renders).
pub(crate) fn transchar_buf(buffer: Option<Buf>, c: c_int) -> CharDisplay {
    owned(&render_char(buffer, c))
}

/// The display form of the single byte `c`. Unlike [`transchar_buf`] this
/// never treats a high byte as a printable Latin-1 character.
pub(crate) fn transchar_byte_buf(buffer: Option<Buf>, c: c_int) -> CharDisplay {
    owned(&render_byte(buffer, c))
}

/// [`transchar_byte_buf`] for the current buffer.
pub(crate) fn transchar_byte(c: c_int) -> CharDisplay {
    transchar_byte_buf(Buf::current_or_none(), c)
}

/// The display form of the unprintable byte `c`.
pub(crate) fn transchar_nonprint_form(buffer: Option<Buf>, c: c_int) -> CharDisplay {
    owned(&render_nonprint(buffer, c))
}

/// Write `c`'s `<xx>` form into `out`, NUL-terminated. Answers the length,
/// excluding the NUL; nine bytes hold the widest form.
pub fn transchar_hex_into(out: &mut [c_char], c: c_int) -> usize {
    let rendered = render::hex_form(c);
    for (slot, &byte) in out.iter_mut().zip(&rendered.bytes[..=rendered.len]) {
        *slot = byte as c_char;
    }
    rendered.len
}

/// Reverse `bytes` in place. Only correct for single-byte text; the callers
/// use it on generated ASCII.
pub(crate) fn mirror_ascii(bytes: &mut [u8]) {
    bytes.reverse();
}

/// The display width of the byte `b`, or zero for a multibyte lead byte.
pub fn byte2cells(b: c_int) -> c_int {
    byte_cells(b)
}

/// The display width of character `c`. A negative code costs two cells more
/// for its `~@` prefix.
pub fn char2cells(c: c_int) -> c_int {
    if c < 0 {
        let escaped = if c == K_SPECIAL {
            KS_SPECIAL
        } else if c == NUL {
            KS_ZERO
        } else {
            -c & 0xff
        };
        return char2cells(escaped) + 2;
    }
    if c >= 0x80 {
        return utf_char2cells(c);
    }
    table_cells((c & 0xff) as uint8_t)
}

/// [`ptr2cells`] over a slice: the cells the character at the start of
/// `bytes` takes. An empty slice reads as the NUL the pointer form stops at.
pub(crate) fn ptr2cells_at(bytes: &[u8]) -> c_int {
    let first = bytes.first().copied().unwrap_or(0);
    if first >= 0x80 {
        return crate::mbyte::cells_at(bytes);
    }
    table_cells(first)
}

/// The display width of the whole of `s`, the bytes of a C string.
pub(crate) fn strsize(s: &[u8]) -> c_int {
    strnsize(s, MAXCOL)
}

/// [`strsize`] of a string the caller owns.
pub(crate) fn vim_strsize_cstr(s: &CStr) -> c_int {
    strsize(s.to_bytes())
}

/// The display width of the characters starting in the first `len` bytes of
/// `s`, the bytes of a C string: a character that starts inside the bound is
/// counted whole.
pub(crate) fn strnsize(s: &[u8], len: c_int) -> c_int {
    crate::mbyte::c_display_width(s, len)
}
