//! Encoding names: canonicalising them, and finding the locale's.
//!
//! Users write an encoding name in whatever spelling they know —
//! `ISO_8859-15`, `latin-9`, `microsoft-cp1252`, `ja_JP.EUC` — and every one
//! of them has to reduce to a single name nvim can look up. [`canonize`]
//! is that reduction, in three stages: normalise the spelling (lowercase,
//! `_` becomes `-`), rewrite the four families whose punctuation varies, and
//! then resolve the result through the alias table.
//!
//! [`canon_props`] answers what a canonical name *is* — 8-bit, DBCS or
//! Unicode, and for Unicode which byte order and unit width — which is what
//! decides whether a conversion needs iconv at all.
//!
//! [`enc_locale`] is where the default comes from: the C library's idea of
//! what the user's locale encodes in, put through the same canonicalisation.
//! [`bomb_size`] is the byte-order-mark half of the question.
//!
//! The pointer spellings (`enc_canonize`, `enc_canon_props`, `enc_skip`)
//! live in [`utf8`](super::utf8) as shims over these.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::memory::XString;
use crate::optionstr::OptStringRef;
use crate::os::cshim::ctype_bits;
use crate::os::env::getenv_owned;
use crate::os::lang::{ctype_locale, locale_codeset};
use crate::types::Candidate;
use crate::winlayer::Buf;
use core::ffi::{c_int, c_uint};

// The carve of the transpiled module; see each child's docs.
mod tables;

pub use self::tables::*;

/// `<ctype.h>`'s alphanumeric class bit, as the rest of the tree spells it.
pub const _ISalnum: c_uint = 8;

/// What [`canon_props`] reports about an encoding name.
///
/// `c_int`, which is the type of [`Encoding::prop`] and of every value
/// compared against it; c2rust typed the anonymous enum from what the C
/// compiler picked and cast at all 130 use sites.
pub type EncProps = c_int;

/// A single-byte encoding: every byte is one character.
pub const ENC_8BIT: EncProps = 1;
/// A double-byte character set — a legacy CJK encoding.
pub const ENC_DBCS: EncProps = 2;
/// Some form of Unicode; the flags below say which.
pub const ENC_UNICODE: EncProps = 4;
/// Big-endian.
pub const ENC_ENDIAN_B: EncProps = 16;
/// Little-endian.
pub const ENC_ENDIAN_L: EncProps = 32;
/// Two bytes per unit (UCS-2).
pub const ENC_2BYTE: EncProps = 64;
/// Four bytes per unit (UCS-4).
pub const ENC_4BYTE: EncProps = 128;
/// Two-byte units with surrogate pairs (UTF-16).
pub const ENC_2WORD: EncProps = 256;
/// Latin-1, which converts to and from UTF-8 without iconv.
pub const ENC_LATIN1: EncProps = 512;
/// Latin-9, likewise.
pub const ENC_LATIN9: EncProps = 1024;
/// Mac OS Roman.
pub const ENC_MACROMAN: EncProps = 2048;

/// The index of `name` in [`ENCODINGS`], if it is a canonical name.
fn find_canonical(name: &[u8]) -> Option<usize> {
    ENCODINGS.iter().position(|e| e.name.to_bytes() == name)
}

/// The [`ENCODINGS`] index `name` is an alias for.
fn find_alias(name: &[u8]) -> Option<usize> {
    ENCODING_ALIASES
        .iter()
        .find(|(alias, _)| alias.to_bytes() == name)
        .map(|&(_, idx)| idx)
}

/// What kind of encoding `name` is, or 0 for one that is not recognised.
///
/// The `2byte-`/`8bit-`/`iso-8859-` prefixes answer even when the full name
/// is unknown: they say what the encoding *is* without saying which one.
pub fn canon_props(name: &[u8]) -> EncProps {
    if let Some(i) = find_canonical(name) {
        return ENCODINGS[i].prop;
    }
    if name.starts_with(b"2byte-") {
        return ENC_DBCS;
    }
    if name.starts_with(b"8bit-") || name.starts_with(b"iso-8859-") {
        return ENC_8BIT;
    }
    0
}

/// How many bytes the current buffer's byte-order mark occupies, or 0 when it
/// is not writing one.
pub fn bomb_size() -> c_int {
    let buf = Buf::current();
    if buf.b_p_bomb == 0 || buf.b_p_bin != 0 {
        return 0;
    }
    let fenc = buf.b_p_fenc.bytes();
    if fenc.is_empty() || fenc == b"utf-8" {
        3
    } else if fenc.starts_with(b"ucs-2") || fenc.starts_with(b"utf-16") {
        2
    } else if fenc.starts_with(b"ucs-4") {
        4
    } else {
        0
    }
}

/// How many bytes of `name` are a `2byte-`/`8bit-` prefix.
///
/// The prefix says what an encoding *is* rather than naming it, so the rest
/// is what gets looked up.
pub fn skip_len(name: &[u8]) -> usize {
    if name.starts_with(b"2byte-") {
        6
    } else if name.starts_with(b"8bit-") {
        5
    } else {
        0
    }
}

/// Normalise a spelling: uppercase becomes lowercase and `_` becomes `-`.
fn normalise_spelling(enc: &[u8]) -> Vec<u8> {
    enc.iter()
        .map(|&b| {
            if b == b'_' {
                b'-'
            } else {
                b.to_ascii_lowercase()
            }
        })
        .collect()
}

/// Rewrite the four name families whose punctuation varies between spellings,
/// so that one lookup covers all of them.
fn normalise_punctuation(name: &[u8]) -> Vec<u8> {
    let mut out = name.to_vec();
    if out.starts_with(b"microsoft-cp") {
        out.drain(..10); // "microsoft-cp1252" -> "cp1252"
    }
    if out.starts_with(b"iso8859") {
        out.insert(3, b'-'); // "iso88591" -> "iso-88591"
    }
    if out.starts_with(b"iso-8859") && out.get(8) != Some(&b'-') {
        out.insert(8, b'-'); // "iso-88591" -> "iso-8859-1"
    }
    if out.starts_with(b"latin-") {
        out.remove(5); // "latin-1" -> "latin1"
    }
    out
}

/// Reduce `enc` to a canonical encoding name.
///
/// `"default"` is the one name that does not go through this: it means "the
/// encoding the locale asked for", which was settled at startup.
///
/// A name that is neither canonical nor an alias comes back *normalised but
/// unresolved*, prefix included — an unknown encoding is still a name iconv
/// might accept.
pub fn canonize(enc: &[u8]) -> XString {
    if enc == b"default" {
        return fenc_default.with(|enc| enc.clone().expect("set at startup"));
    }

    let normalised = normalise_spelling(enc);
    let skip = skip_len(&normalised);
    let body = normalise_punctuation(&normalised[skip..]);

    if find_canonical(&body).is_some() {
        // Canonical without the prefix: the prefix said nothing new.
        return XString::from_bytes(&body);
    }
    if let Some(i) = find_alias(&body) {
        return XString::from_cstr(ENCODINGS[i].name);
    }
    // Unrecognised: keep the prefix, since it is all anyone knows about it.
    let mut whole = normalised[..skip].to_vec();
    whole.extend_from_slice(&body);
    XString::from_bytes(&whole)
}

/// The encoding the C library says the user's locale uses, canonicalised.
///
/// A locale name is `language[_territory][.codeset][@modifier]...`, so the
/// codeset is what follows a `.`; without one, the whole name is taken as far
/// as its first non-alphanumeric character. `ja_JP.EUC` and its two siblings
/// are the exception — those name the encoding by the *territory*, so they
/// become `euc-jp`, `euc-cn`, `euc-kr`.
///
/// `None` when the C library names no locale at all.
pub fn enc_locale() -> Option<XString> {
    let name: Vec<u8> = match locale_codeset().or_else(ctype_locale) {
        Some(name) => name,
        None => {
            // Upstream's chain, reproduced as written: each step *replaces*
            // the answer rather than keeping what it just found, so a set
            // `LC_ALL` means `LANG` is what ends up being read.
            let mut value = getenv_owned(c"LC_ALL");
            if value.is_some() {
                value = getenv_owned(c"LC_CTYPE");
                if value.is_some() {
                    value = getenv_owned(c"LANG");
                }
            }
            value?.to_vec()
        }
    };

    let mut copy_from = 0;
    if let Some(dot) = name.iter().position(|&b| b == b'.') {
        if is_territory_euc(&name, dot) {
            // "XY.EUC" is "euc-xy"; a territory character that is not one
            // ends the name there.
            let mut euc = b"euc-".to_vec();
            euc.extend(
                [name[dot - 2], name[dot - 1]]
                    .iter()
                    .map_while(|&b| alnum_lowered(b)),
            );
            return Some(canonize(&euc));
        }
        copy_from = dot + 1;
    }

    // Copy the codeset, lowercased, `_`/`-` unified, stopping at the first
    // character that cannot be part of a name, and at 49 bytes.
    let codeset: Vec<u8> = name[copy_from..]
        .iter()
        .take(49)
        .map_while(|&b| {
            if b == b'_' || b == b'-' {
                Some(b'-')
            } else if b.is_ascii_alphanumeric() {
                Some(b.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect();
    Some(canonize(&codeset))
}

/// Is this locale name one of the `ja_JP.EUC` family, which names its
/// encoding by the territory two characters before the dot at `dot`?
fn is_territory_euc(name: &[u8], dot: usize) -> bool {
    let at = |i: usize| cstr::byte_at(name, i);
    dot > 2
        && name[dot + 1..]
            .get(..3)
            .is_some_and(|euc| euc.eq_ignore_ascii_case(b"EUC"))
        // Nothing may follow the "EUC" but a separator: "EUC-JP" is
        // already a codeset name and goes down the ordinary path.
        && c_uint::from(ctype_bits(at(dot + 4))) & _ISalnum == 0
        && at(dot + 4) != b'-'
        && name[dot - 3] == b'_'
}

/// One territory character, lowercased — or `None` if it is not one, which
/// ends the name there and makes it unresolvable rather than wrong.
fn alnum_lowered(b: u8) -> Option<u8> {
    b.is_ascii_alphanumeric().then(|| b.to_ascii_lowercase())
}

/// `:set fileencoding=<Tab>` completion: the `idx`th canonical name.
pub fn get_encoding_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    ENCODINGS.get(idx).map(|enc| Candidate::Borrowed(enc.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `IDX_*` constants are indices, so each must name its own row.
    #[test]
    fn indices_name_their_rows() {
        assert_eq!(ENCODINGS.len(), IDX_COUNT);
        assert_eq!(ENCODINGS[IDX_LATIN_1].name, c"latin1");
        assert_eq!(ENCODINGS[IDX_ISO_15].name, c"iso-8859-15");
        assert_eq!(ENCODINGS[IDX_UTF8].name, c"utf-8");
        assert_eq!(ENCODINGS[IDX_UCS4LE].name, c"ucs-4le");
        assert_eq!(ENCODINGS[IDX_BIG5].name, c"big5");
        assert_eq!(ENCODINGS[IDX_CP950].name, c"cp950");
        assert_eq!(ENCODINGS[IDX_MACROMAN].name, c"macroman");
        assert_eq!(ENCODINGS[IDX_HPROMAN8].name, c"hp-roman8");
        for (_, idx) in &ENCODING_ALIASES {
            assert!(*idx < IDX_COUNT);
        }
    }

    /// No canonical name may repeat, or the search would answer the wrong
    /// index for it.
    #[test]
    fn canonical_names_are_unique() {
        for (i, e) in ENCODINGS.iter().enumerate() {
            assert_eq!(find_canonical(e.name.to_bytes()), Some(i), "{:?}", e.name);
        }
    }

    /// `"950"` is in the alias table twice; the first row is the answer.
    #[test]
    fn first_alias_wins() {
        assert_eq!(find_alias(b"950"), Some(IDX_CP950));
        assert_eq!(
            ENCODING_ALIASES
                .iter()
                .filter(|(a, _)| a.to_bytes() == b"950")
                .count(),
            2
        );
    }

    #[test]
    fn punctuation_families() {
        assert_eq!(normalise_punctuation(b"microsoft-cp1252"), b"cp1252");
        assert_eq!(normalise_punctuation(b"iso88591"), b"iso-8859-1");
        assert_eq!(normalise_punctuation(b"iso-88591"), b"iso-8859-1");
        assert_eq!(normalise_punctuation(b"iso-8859-1"), b"iso-8859-1");
        assert_eq!(normalise_punctuation(b"latin-1"), b"latin1");
        assert_eq!(normalise_punctuation(b"utf-8"), b"utf-8");
        assert_eq!(normalise_spelling(b"ISO_8859-15"), b"iso-8859-15");
    }

    #[test]
    fn prefixes() {
        assert_eq!(skip_len(b"2byte-euc-jp"), 6);
        assert_eq!(skip_len(b"8bit-latin1"), 5);
        assert_eq!(skip_len(b"latin1"), 0);
    }
}
