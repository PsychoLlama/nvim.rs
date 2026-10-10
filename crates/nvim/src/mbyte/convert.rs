//! Converting text between encodings.
//!
//! [`VimConv::setup`] works out *how* to get from one encoding to another and
//! records it in the `VimConv`; [`VimConv::convert`] runs that plan over a
//! string. Four pairs are done here directly — Latin-1 and Latin-9 to and
//! from UTF-8 — because they are the common cases and are pure arithmetic.
//! Everything else goes through iconv, whose descriptor the plan owns
//! ([`Iconv`], in `os/`, which is the only place iconv itself is called).
//!
//! A conversion that cannot represent a character does not fail by default:
//! it substitutes `?` (or `¿` on the way to Latin-1) and carries on, unless
//! `vc_fail` asks for an error instead.

#![forbid(unsafe_code)]

use super::*;
use crate::eval::typval::NumBuf;
use crate::memory::ThinCString;
use crate::os::iconv::Iconv;
use crate::types::Failed;
use ::libc::{E2BIG, EILSEQ, EINVAL};
use core::ffi::{CStr, c_int};

/// iconv's own `errno` values, which a dynamically loaded library may report
/// instead of the host's — both spellings are tested at every site. They are
/// spelled out rather than derived from `libc`'s: on this platform the two
/// happen to agree, but the point of the pair is that elsewhere they need not.
pub const ICONV_E2BIG: c_int = 7;
pub const ICONV_EINVAL: c_int = 22;
pub const ICONV_EILSEQ: c_int = 84;

/// An iconv descriptor converting `from` to `to`, either name past its
/// `8bit-`/`2byte-` prefix, or `None` when iconv cannot convert the pair.
pub(crate) fn open_iconv(to: &CStr, from: &CStr) -> Option<Iconv> {
    fn skip(name: &CStr) -> &CStr {
        cstr::suffix(name, skip_len(name.to_bytes()))
    }
    Iconv::open(skip(to), skip(from))
}

/// Convert `text` through `fd`.
///
/// The output buffer is grown and the conversion resumed whenever iconv runs
/// out of room, which is what the `E2BIG` arm is for: it is not an error.
/// `done` is how much of the result is already final. `None` when iconv
/// stopped on something other than room or, with `fail`, on a character it
/// could not convert.
fn iconv_convert(fd: &Iconv, fail: bool, text: &[u8]) -> Option<Vec<u8>> {
    let mut result: Vec<u8> = Vec::new();
    let mut done = 0;
    let mut from = text;
    let mut errno = 0;
    loop {
        if result.is_empty() || errno == ICONV_E2BIG {
            // Enough for most conversions; on a retry, more than last time.
            let len = result.len() + from.len() * 2 + 40;
            result.resize(len, 0);
        }
        // Two bytes held back: the second `?` an unconvertible wide
        // character can need, and upstream's NUL.
        let limit = result.len() - 2;
        let mut out = &mut result[done..limit];
        let room = out.len();
        let stopped = fd.step(&mut from, &mut out);
        let mut to = done + (room - out.len());
        match stopped {
            Ok(()) => {
                result.truncate(to);
                return Some(result);
            }
            Err(e) => errno = e,
        }

        let incomplete = errno == ICONV_EINVAL || errno == EINVAL;
        let illegal = errno == ICONV_EILSEQ || errno == EILSEQ;
        if !fail && (illegal || incomplete) {
            // Cannot convert: emit `?` and skip one character. This assumes
            // the input is 'encoding'; nothing else would tell us how much
            // to skip.
            result[to] = b'?';
            to += 1;
            if cells_at(from) > 1 {
                result[to] = b'?';
                to += 1;
            }
            from = &from[cluster_len(from)..];
        } else if errno != ICONV_E2BIG && errno != E2BIG {
            return None;
        }
        done = to;
    }
}

/// `iconv({string}, {from}, {to})`.
pub fn f_iconv(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_string(None);

    let str = numbuf.string(&args[0]);
    let mut buf1 = NumBuf::new();
    let from = buf1.string(&args[1]).to_bytes();
    let from = canonize(&from[skip_len(from)..]);
    let mut buf2 = NumBuf::new();
    let to = buf2.string(&args[2]).to_bytes();
    let to = canonize(&to[skip_len(to)..]);

    let mut vimconv = CONV_NONE_INIT;
    let _ = vimconv.setup(Some(from.as_cstr()), Some(to.as_cstr()));
    result.write_string(if vimconv.vc_type == CONV_NONE {
        // Same encoding both ways: hand back a copy unchanged.
        Some(ThinCString::from_cstr(str))
    } else {
        vimconv.convert_cstr(str)
    });
}

impl VimConv {
    /// Plan a conversion from `from` to `to`, replacing whatever this held;
    /// `None` on either side, or an empty name, plans nothing. `Err` when no
    /// conversion between the two is possible.
    pub fn setup(&mut self, from: Option<&CStr>, to: Option<&CStr>) -> Result<(), Failed> {
        self.setup_ext(from, true, to, true)
    }

    /// [`setup`](Self::setup), choosing what "Unicode" means on each side.
    ///
    /// `*_unicode_is_utf8` says whether *any* Unicode encoding on that side
    /// may be treated as UTF-8, or only the one whose properties are exactly
    /// `ENC_UNICODE` — which is `utf-8` itself. A reader of a UTF-16 file
    /// needs the strict answer; a caller that has already decoded to UTF-8
    /// wants the loose one.
    pub(crate) fn setup_ext(
        &mut self,
        from: Option<&CStr>,
        from_unicode_is_utf8: bool,
        to: Option<&CStr>,
        to_unicode_is_utf8: bool,
    ) -> Result<(), Failed> {
        // Closes the descriptor this held, if any.
        *self = CONV_NONE_INIT;

        // Nothing to do: an unnamed side, or the same encoding twice.
        let (Some(from), Some(to)) = (from.filter(|s| !s.is_empty()), to.filter(|s| !s.is_empty()))
        else {
            return Ok(());
        };
        if from == to {
            return Ok(());
        }

        let from_prop = canon_props(from.to_bytes());
        let to_prop = canon_props(to.to_bytes());
        let is_utf8 = |prop: EncProps, loose: bool| {
            if loose {
                prop & ENC_UNICODE != 0
            } else {
                prop == ENC_UNICODE
            }
        };
        let from_is_utf8 = is_utf8(from_prop, from_unicode_is_utf8);
        let to_is_utf8 = is_utf8(to_prop, to_unicode_is_utf8);

        // `vc_factor` is how much the output can grow: a Latin-1 byte becomes
        // at most two UTF-8 bytes, a Latin-9 one at most three, and iconv's
        // worst case is budgeted at four.
        if from_prop & ENC_LATIN1 != 0 && to_is_utf8 {
            self.vc_type = CONV_TO_UTF8;
            self.vc_factor = 2;
        } else if from_prop & ENC_LATIN9 != 0 && to_is_utf8 {
            self.vc_type = CONV_9_TO_UTF8;
            self.vc_factor = 3;
        } else if from_is_utf8 && to_prop & ENC_LATIN1 != 0 {
            self.vc_type = CONV_TO_LATIN1;
        } else if from_is_utf8 && to_prop & ENC_LATIN9 != 0 {
            self.vc_type = CONV_TO_LATIN9;
        } else {
            // A side already known to be UTF-8 is named as such, whatever
            // Unicode spelling it arrived under.
            fn named(is_utf8: bool, name: &CStr) -> &CStr {
                if is_utf8 { c"utf-8" } else { name }
            }
            let to = named(to_is_utf8, to);
            let from = named(from_is_utf8, from);
            if let Some(fd) = open_iconv(to, from) {
                self.vc_fd = Some(fd);
                self.vc_type = CONV_ICONV;
                self.vc_factor = 4;
            }
        }
        if self.vc_type == CONV_NONE {
            Err(Failed)
        } else {
            Ok(())
        }
    }

    /// Run the plan over `text`: the converted bytes, or `None` when the
    /// conversion failed -- an invalid sequence in the input, an
    /// unrepresentable character with `vc_fail` set, or no plan at all.
    pub fn convert(&self, text: &[u8]) -> Option<Vec<u8>> {
        self.convert_sized(text).map(|(bytes, _)| bytes)
    }

    /// [`convert`](Self::convert) of a C string, as one.
    ///
    /// The answer for the four arithmetic conversions is a block of the
    /// conversion's worst-case growth plus a terminator's byte, which is
    /// what upstream allocates (it converts in place into it): the size is
    /// observable -- `typval_list.rs` asserts the malloc sizes a converting
    /// `list_copy` makes -- and it is the bound `vc_factor` promises.
    pub(crate) fn convert_cstr(&self, text: &CStr) -> Option<ThinCString> {
        let (bytes, room) = self.convert_sized(text.to_bytes())?;
        Some(ThinCString::in_block(&bytes, room))
    }

    /// The conversion, and the block size upstream allocates for it.
    fn convert_sized(&self, text: &[u8]) -> Option<(Vec<u8>, usize)> {
        if text.is_empty() {
            return Some((Vec::new(), 1));
        }
        let factor = match self.vc_type {
            CONV_ICONV => {
                let out = iconv_convert(self.vc_fd.as_ref()?, self.vc_fail, text)?;
                let room = out.len() + 1;
                return Some((out, room));
            }
            CONV_TO_UTF8 => 2,
            CONV_9_TO_UTF8 => 3,
            CONV_TO_LATIN1 | CONV_TO_LATIN9 => 1,
            // CONV_NONE, and anything else: nothing was planned.
            _ => return None,
        };
        let out = match self.vc_type {
            CONV_TO_UTF8 => latin1_to_utf8(text),
            CONV_9_TO_UTF8 => latin9_to_utf8(text),
            _ => utf8_to_latin(text, self.vc_type == CONV_TO_LATIN9, self.vc_fail)?,
        };
        let room = text.len() * factor + 1;
        debug_assert!(out.len() < room, "conversion overran vc_factor");
        Some((out, room))
    }
}

/// Latin-1 to UTF-8: every byte is the codepoint of the same number.
fn latin1_to_utf8(src: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len() * 2);
    for &c in src {
        if c < 0x80 {
            out.push(c);
        } else {
            out.push(0xc0 + (c >> 6));
            out.push(0x80 + (c & 0x3f));
        }
    }
    out
}

/// The eight positions where Latin-9 differs from Latin-1, as
/// `(latin9 byte, codepoint)`.
///
/// Latin-9 is Latin-1 with eight characters replaced: the euro sign, the
/// French œ ligature and Ÿ, and the S and Z with caron that Baltic languages
/// need.
const LATIN9_REPLACEMENTS: [(u8, c_int); 8] = [
    (0xa4, 0x20ac), // €
    (0xa6, 0x0160), // Š
    (0xa8, 0x0161), // š
    (0xb4, 0x017d), // Ž
    (0xb8, 0x017e), // ž
    (0xbc, 0x0152), // Œ
    (0xbd, 0x0153), // œ
    (0xbe, 0x0178), // Ÿ
];

/// Latin-9 to UTF-8: Latin-1's mapping, with the eight replacements applied.
fn latin9_to_utf8(src: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len() * 3);
    let mut buf = [0u8; MB_MAXCHAR];
    for &b in src {
        let c = LATIN9_REPLACEMENTS
            .iter()
            .find(|&&(byte, _)| byte == b)
            .map_or(b as c_int, |&(_, code)| code);
        let n = encode_char(c, &mut buf);
        out.extend_from_slice(&buf[..n]);
    }
    out
}

/// UTF-8 to Latin-1 or Latin-9.
///
/// A character with no representation becomes `¿` (plus a `?` when it was
/// drawn two cells wide, so columns line up), unless `fail` asks for an error
/// instead. Composing characters are dropped: the base character is all
/// either target can hold.
///
/// Answers `None` when the input is not UTF-8 at all, or when `fail` is set
/// and a character cannot be represented.
fn utf8_to_latin(src: &[u8], to_latin9: bool, fail: bool) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        let rest = &src[i..];
        let l = promised_char_len(rest);
        if rest[0] == 0 {
            // An embedded NUL, which the length says is part of the string.
            out.push(0);
            i += 1;
            continue;
        }
        if l == 1 {
            if utf8len_tab_zero[usize::from(rest[0])] == 0 {
                return None; // not a lead byte: the input is not UTF-8
            }
            out.push(rest[0]);
            i += 1;
            continue;
        }

        let mut c = char_at(rest);
        if to_latin9 {
            c = to_latin9_byte(c);
        }
        if !utf_iscomposing_legacy(c) {
            if c < 0x100 {
                out.push(c as u8);
            } else if fail {
                return None;
            } else {
                out.push(0xbf); // ¿
                if utf_char2cells(c) > 1 {
                    out.push(b'?');
                }
            }
        }
        i += l;
    }
    Some(out)
}

/// The Latin-9 byte for a codepoint, or a value no byte can hold.
///
/// The eight replaced positions map back to their bytes. The eight
/// *Latin-1* characters those positions displaced map to `0x100`, which is
/// out of range on purpose: they exist in Latin-1 but not in Latin-9, so they
/// must become `¿` rather than silently turning into the wrong glyph.
fn to_latin9_byte(c: c_int) -> c_int {
    if let Some(&(byte, _)) = LATIN9_REPLACEMENTS.iter().find(|&&(_, code)| code == c) {
        return byte as c_int;
    }
    if LATIN9_REPLACEMENTS
        .iter()
        .any(|&(byte, _)| byte as c_int == c)
    {
        return 0x100;
    }
    c
}

/// A conversion from one encoding to another, torn down when dropped.
pub(crate) struct Converter(VimConv);

impl Converter {
    /// The conversion from `from` to `to`, or `None` when none is needed or
    /// possible.
    pub(crate) fn new(from: &CStr, to: &CStr) -> Option<Converter> {
        let mut conv = Converter(CONV_NONE_INIT);
        let _ = conv.0.setup(Some(from), Some(to));
        (conv.0.vc_type != CONV_NONE).then_some(conv)
    }

    /// `text` converted, or `None` when it would not convert.
    pub(crate) fn convert(&self, text: &[u8]) -> Option<Vec<u8>> {
        self.0.convert(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin1_round_trip() {
        let src: Vec<u8> = (0u8..=255).collect();
        let utf8 = latin1_to_utf8(&src);
        // Every byte is the codepoint of the same number.
        assert_eq!(String::from_utf8(utf8).unwrap().chars().count(), 256);
    }

    /// The Latin-9 map has to be a bijection on the eight positions, or a
    /// round trip would lose characters.
    #[test]
    fn latin9_map_is_a_bijection() {
        for &(byte, code) in &LATIN9_REPLACEMENTS {
            assert_eq!(to_latin9_byte(code), byte as c_int);
            // The Latin-1 character this position displaced has no Latin-9
            // spelling, and must be pushed out of byte range.
            assert_eq!(to_latin9_byte(byte as c_int), 0x100);
        }
        // Everything else passes through.
        assert_eq!(to_latin9_byte('a' as c_int), 'a' as c_int);
        assert_eq!(to_latin9_byte(0xe9), 0xe9); // é, same in both
    }

    #[test]
    fn latin9_encodes_the_euro() {
        assert_eq!(latin9_to_utf8(&[0xa4]), "€".as_bytes());
        assert_eq!(latin9_to_utf8(b"a"), b"a");
    }
}
