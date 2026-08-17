//! Encoding names: canonicalising them, and finding the locale's.
//!
//! Users write an encoding name in whatever spelling they know —
//! `ISO_8859-15`, `latin-9`, `microsoft-cp1252`, `ja_JP.EUC` — and every one
//! of them has to reduce to a single name nvim can look up. [`enc_canonize`]
//! is that reduction, in three stages: normalise the spelling (lowercase,
//! `_` becomes `-`), rewrite the four families whose punctuation varies, and
//! then resolve the result through the alias table.
//!
//! [`enc_canon_props`] answers what a canonical name *is* — 8-bit, DBCS or
//! Unicode, and for Unicode which byte order and unit width — which is what
//! decides whether a conversion needs iconv at all.
//!
//! [`enc_locale`] is where the default comes from: the C library's idea of
//! what the user's locale encodes in, put through the same canonicalisation.
//! `bomb_size`/`remove_bom` are the byte-order-mark half of the question.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

// The carve of the transpiled module; see each child's docs.
mod tables;

pub use self::tables::*;

/// `<ctype.h>`'s alphanumeric class bit, as the rest of the tree spells it.
pub const _ISalnum: c_uint = 8;

/// What [`enc_canon_props`] reports about an encoding name.
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

pub type nl_item = c_int;

/// `nl_langinfo`'s "what does this locale encode in" item.
pub const CODESET: nl_item = 14;

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
///
/// # Safety
///
/// `name` must be a NUL-terminated string.
pub unsafe fn enc_canon_props(name: *const c_char) -> EncProps {
    // SAFETY: the caller's obligation.
    let name = unsafe { CStr::from_ptr(name) }.to_bytes();
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
///
/// # Safety
///
/// The editor's globals must be live.
pub unsafe fn bomb_size() -> c_int {
    unsafe {
        let buf = curbuf.get();
        if (*buf).b_p_bomb == 0 || (*buf).b_p_bin != 0 {
            return 0;
        }
        let fenc = CStr::from_ptr((*buf).b_p_fenc).to_bytes();
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
}

/// Delete every UTF-8 byte-order mark from `s`, in place.
///
/// # Safety
///
/// `s` must be a writable NUL-terminated string.
pub unsafe fn remove_bom(s: *mut c_char) {
    unsafe {
        let mut p = s;
        loop {
            p = strchr(p, 0xef);
            if p.is_null() {
                return;
            }
            if *p.offset(1) as u8 == 0xbb && *p.offset(2) as u8 == 0xbf {
                // Move the rest of the string down over the mark, NUL included.
                let rest = p.offset(3);
                memmove(p as *mut c_void, rest as *const c_void, strlen(rest) + 1);
            } else {
                p = p.offset(1);
            }
        }
    }
}

/// How many bytes of `name` are a `2byte-`/`8bit-` prefix.
///
/// The prefix says what an encoding *is* rather than naming it, so the rest
/// is what gets looked up.
fn skip_len(name: &[u8]) -> usize {
    if name.starts_with(b"2byte-") {
        6
    } else if name.starts_with(b"8bit-") {
        5
    } else {
        0
    }
}

/// Past a `2byte-`/`8bit-` prefix, if there is one.
///
/// # Safety
///
/// `p` must be a NUL-terminated string.
pub unsafe fn enc_skip(p: *mut c_char) -> *mut c_char {
    // SAFETY: the caller's obligation.
    let len = skip_len(unsafe { CStr::from_ptr(p) }.to_bytes());
    // SAFETY: `len` is at most the string's length.
    unsafe { p.add(len) }
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

/// Copy `bytes` into an `xmalloc`'d NUL-terminated string, as every caller of
/// [`enc_canonize`] expects (they `xfree` the result).
fn to_owned_cstring(bytes: &[u8]) -> *mut c_char {
    // SAFETY: `bytes` is a live slice; `xmemdupz` appends the NUL.
    unsafe { xmemdupz(bytes.as_ptr() as *const c_void, bytes.len()) as *mut c_char }
}

/// Reduce `enc` to a canonical encoding name, in a freshly allocated string.
///
/// `"default"` is the one name that does not go through this: it means "the
/// encoding the locale asked for", which was settled at startup.
///
/// A name that is neither canonical nor an alias comes back *normalised but
/// unresolved*, prefix included — an unknown encoding is still a name iconv
/// might accept.
///
/// # Safety
///
/// `enc` must be a NUL-terminated string. The result is `xmalloc`'d and the
/// caller owns it.
pub unsafe fn enc_canonize(enc: *mut c_char) -> *mut c_char {
    unsafe {
        let enc = CStr::from_ptr(enc).to_bytes();
        if enc == b"default" {
            return xstrdup(fenc_default.get());
        }

        let normalised = normalise_spelling(enc);
        let skip = skip_len(&normalised);
        let body = normalise_punctuation(&normalised[skip..]);

        if find_canonical(&body).is_some() {
            // Canonical without the prefix: the prefix said nothing new.
            return to_owned_cstring(&body);
        }
        if let Some(i) = find_alias(&body) {
            return to_owned_cstring(ENCODINGS[i].name.to_bytes());
        }
        // Unrecognised: keep the prefix, since it is all anyone knows about it.
        let mut whole = normalised[..skip].to_vec();
        whole.extend_from_slice(&body);
        to_owned_cstring(&whole)
    }
}

/// The encoding the C library says the user's locale uses, canonicalised.
///
/// A locale name is `language[_territory][.codeset][@modifier]...`, so the
/// codeset is what follows a `.`; without one, the whole name is taken as far
/// as its first non-alphanumeric character. `ja_JP.EUC` and its two siblings
/// are the exception — those name the encoding by the *territory*, so they
/// become `euc-jp`, `euc-cn`, `euc-kr`.
///
/// # Safety
///
/// The editor's globals must be live. The result is `xmalloc`'d.
pub unsafe fn enc_locale() -> *mut c_char {
    unsafe {
        let mut s = nl_langinfo(CODESET);
        if s.is_null() || *s == 0 {
            s = setlocale(LC_CTYPE, core::ptr::null());
            if s.is_null() || *s == 0 {
                // Upstream's chain, reproduced as written: each step
                // *replaces* `s` rather than keeping what it just found, so
                // a set `LC_ALL` means `LANG` is what ends up being read.
                s = os_getenv_noalloc(c"LC_ALL".as_ptr());
                if !s.is_null() {
                    s = os_getenv_noalloc(c"LC_CTYPE".as_ptr());
                    if !s.is_null() {
                        s = os_getenv_noalloc(c"LANG".as_ptr());
                    }
                }
            }
        }
        if s.is_null() {
            return core::ptr::null_mut();
        }

        let mut buf = [0 as c_char; 50];
        let dot = vim_strchr(s, '.' as c_int);
        let mut copy_from = s;
        if !dot.is_null() {
            if is_territory_euc(s, dot) {
                // "XY.EUC" is "euc-xy".
                buf[..4].copy_from_slice(&[
                    b'e' as c_char,
                    b'u' as c_char,
                    b'c' as c_char,
                    b'-' as c_char,
                ]);
                buf[4] = alnum_lowered(*dot.offset(-2));
                buf[5] = alnum_lowered(*dot.offset(-1));
                buf[6] = 0;
                return enc_canonize(buf.as_mut_ptr());
            }
            copy_from = dot.offset(1);
        }

        // Copy the codeset, lowercased, `_`/`-` unified, stopping at the
        // first character that cannot be part of a name.
        let mut i = 0;
        while i < buf.len() - 1 && *copy_from.add(i) != 0 {
            let b = *copy_from.add(i) as u8;
            buf[i] = if b == b'_' || b == b'-' {
                b'-' as c_char
            } else if b.is_ascii_alphanumeric() {
                b.to_ascii_lowercase() as c_char
            } else {
                break;
            };
            i += 1;
        }
        buf[i] = 0;
        enc_canonize(buf.as_mut_ptr())
    }
}

/// Is this locale name one of the `ja_JP.EUC` family, which names its
/// encoding by the territory two characters before the dot?
///
/// # Safety
///
/// `dot` must point at a `.` inside the NUL-terminated string `s`.
unsafe fn is_territory_euc(s: *const c_char, dot: *const c_char) -> bool {
    unsafe {
        dot > s.offset(2)
            && strncasecmp(dot.offset(1) as *mut c_char, c"EUC".as_ptr() as *mut c_char, 3) == 0
            // Nothing may follow the "EUC" but a separator: "EUC-JP" is
            // already a codeset name and goes down the ordinary path.
            && *(*__ctype_b_loc()).offset(*dot.offset(4) as u8 as c_int as isize) as c_int
                & _ISalnum as c_int
                == 0
            && *dot.offset(4) as c_int != '-' as c_int
            && *dot.offset(-3) as c_int == '_' as c_int
    }
}

/// One territory character, lowercased — or NUL if it is not one, which makes
/// the name unresolvable rather than wrong.
fn alnum_lowered(c: c_char) -> c_char {
    let b = c as u8;
    if b.is_ascii_alphanumeric() {
        b.to_ascii_lowercase() as c_char
    } else {
        0
    }
}

/// `:set fileencoding=<Tab>` completion: the `idx`th canonical name.
///
/// # Safety
///
/// The returned pointer is into a `'static` table; the caller must not free
/// it.
pub unsafe extern "C" fn get_encoding_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    match usize::try_from(idx) {
        Ok(i) if i < IDX_COUNT => ENCODINGS[i].name.as_ptr() as *mut c_char,
        _ => core::ptr::null_mut(),
    }
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
