//! Characters and character classes: the `magic` toggles, the byte class
//! table the `\d`/`\w`/`\s` atoms test against, and the three bracketed
//! items a `[]` collection can name — `[:alpha:]`, `[=a=]` and `[.a.]`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::regexp::RegCompiler;
use core::cmp::Ordering;
use core::ffi::{CStr, c_int};

use super::mbyte::{char_at_nul, cluster_len_at};
use super::{ByteClass, MAGIC_ALL, Spelled};

/// A magic metacharacter is held as its byte minus 256, so that the parser
/// can tell `*` (a repeat) from `\*` (a literal star) by sign alone. These
/// two undo and flip that marker.
pub(crate) fn unmagic(c: c_int) -> c_int {
    if c < 0 { c + 256 } else { c }
}

pub(crate) fn toggle_magic(c: c_int) -> c_int {
    if c < 0 { c + 256 } else { c - 256 }
}

/// The marker form of the metacharacter `c`, for matching against what
/// [`super::peekchr`] returns.
pub(crate) const fn magic(c: u8) -> c_int {
    c as c_int - 256
}

/// The metacharacters the two compilers name, in their marker form.
pub(crate) const M_0: c_int = magic(b'0');
pub(crate) const M_1: c_int = magic(b'1');
pub(crate) const M_9: c_int = magic(b'9');
pub(crate) const M_AMP: c_int = magic(b'&');
pub(crate) const M_AT: c_int = magic(b'@');
pub(crate) const M_BAR: c_int = magic(b'|');
pub(crate) const M_BRACE: c_int = magic(b'{');
pub(crate) const M_BRACKET: c_int = magic(b'[');
pub(crate) const M_CARET: c_int = magic(b'^');
pub(crate) const M_C_LOWER: c_int = magic(b'c');
pub(crate) const M_C_UPPER: c_int = magic(b'C');
pub(crate) const M_DOLLAR: c_int = magic(b'$');
pub(crate) const M_EQUAL: c_int = magic(b'=');
pub(crate) const M_GT: c_int = magic(b'>');
pub(crate) const M_LT: c_int = magic(b'<');
pub(crate) const M_M_LOWER: c_int = magic(b'm');
pub(crate) const M_M_UPPER: c_int = magic(b'M');
pub(crate) const M_N: c_int = magic(b'n');
pub(crate) const M_PAREN_CLOSE: c_int = magic(b')');
pub(crate) const M_PAREN_OPEN: c_int = magic(b'(');
pub(crate) const M_PERCENT: c_int = magic(b'%');
pub(crate) const M_PLUS: c_int = magic(b'+');
pub(crate) const M_QUESTION: c_int = magic(b'?');
pub(crate) const M_STAR: c_int = magic(b'*');
pub(crate) const M_TILDE: c_int = magic(b'~');
pub(crate) const M_UNDERSCORE: c_int = magic(b'_');
pub(crate) const M_V_LOWER: c_int = magic(b'v');
pub(crate) const M_V_UPPER: c_int = magic(b'V');
pub(crate) const M_Z: c_int = magic(b'z');
pub(crate) const M_Z_UPPER: c_int = magic(b'Z');

/// The low byte of `c`, as C's store of an `int` in a `char_u`: how a
/// token -- a character, or a metacharacter's negative code -- is matched
/// against the ASCII letter it was spelled with.
#[inline(always)]
pub(crate) const fn low_byte(c: c_int) -> u8 {
    c.to_le_bytes()[0]
}

/// How an error message spells a metacharacter so that it echoes what the
/// user typed: bare under `\v`, where every metacharacter is, escaped
/// otherwise.
pub(crate) fn spelled(rc: &RegCompiler) -> Spelled {
    Spelled::bare_if(rc.magic == MAGIC_ALL)
}

/// The control character a `\r`/`\t`/`\e`/`\b` abbreviation stands for.
/// Anything else is returned unchanged.
pub(crate) fn backslash_abbr(c: c_int) -> c_int {
    match u8::try_from(c) {
        Ok(b'r') => c_int::from(b'\r'),
        Ok(b't') => c_int::from(b'\t'),
        Ok(b'e') => 0x1b,
        Ok(b'b') => 0x08,
        _ => c,
    }
}

/// Which of the `RI_*` classes each byte belongs to. Only the ASCII range
/// is meaningful — the engines test `c < 256` before indexing, and
/// everything at or above 0x80 is classless here because a multibyte
/// character is classified by [`crate::mbyte`] instead.
pub(crate) static RI_FLAGS: [ByteClass; 256] = build_ri_flags();

const fn build_ri_flags() -> [ByteClass; 256] {
    use ByteClass as B;
    let mut tab = [B::NONE; 256];
    let mut i = 0usize;
    while i < 256 {
        let b = i.to_le_bytes()[0];
        tab[i] = match b {
            b'0'..=b'7' => B::DIGIT.or(B::HEX).or(B::OCTAL).or(B::WORD),
            b'8'..=b'9' => B::DIGIT.or(B::HEX).or(B::WORD),
            b'a'..=b'f' => B::HEX.or(B::WORD).or(B::HEAD).or(B::ALPHA).or(B::LOWER),
            b'g'..=b'z' => B::WORD.or(B::HEAD).or(B::ALPHA).or(B::LOWER),
            b'A'..=b'F' => B::HEX.or(B::WORD).or(B::HEAD).or(B::ALPHA).or(B::UPPER),
            b'G'..=b'Z' => B::WORD.or(B::HEAD).or(B::ALPHA).or(B::UPPER),
            b'_' => B::WORD.or(B::HEAD),
            _ => B::NONE,
        };
        i += 1;
    }
    tab[b' ' as usize] = tab[b' ' as usize].or(B::WHITE);
    tab[b'\t' as usize] = tab[b'\t' as usize].or(B::WHITE);
    tab
}

/// One of the POSIX `[:name:]` classes a `[]` collection can name.
///
/// The discriminants are upstream's `CLASS_*` numbers, which both engines
/// index tables by; upstream's `CLASS_NONE` sentinel is [`Option::None`]
/// here.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub(crate) enum CharClass {
    Alnum = 0,
    Alpha = 1,
    Blank = 2,
    Cntrl = 3,
    Digit = 4,
    Graph = 5,
    Lower = 6,
    Print = 7,
    Punct = 8,
    Space = 9,
    Upper = 10,
    Xdigit = 11,
    Tab = 12,
    Return = 13,
    Backspace = 14,
    Escape = 15,
    Ident = 16,
    Keyword = 17,
    Fname = 18,
}

/// The `[:name:]` classes a collection may contain, sorted by name so the
/// lookup below can binary-search them. The trailing `:]` is part of the
/// name: the `[` is already consumed when we get here, and matching the
/// close is what proves the item was a class and not literal text.
static CHAR_CLASS_TAB: [(&CStr, CharClass); 19] = [
    (c"alnum:]", CharClass::Alnum),
    (c"alpha:]", CharClass::Alpha),
    (c"backspace:]", CharClass::Backspace),
    (c"blank:]", CharClass::Blank),
    (c"cntrl:]", CharClass::Cntrl),
    (c"digit:]", CharClass::Digit),
    (c"escape:]", CharClass::Escape),
    (c"fname:]", CharClass::Fname),
    (c"graph:]", CharClass::Graph),
    (c"ident:]", CharClass::Ident),
    (c"keyword:]", CharClass::Keyword),
    (c"lower:]", CharClass::Lower),
    (c"print:]", CharClass::Print),
    (c"punct:]", CharClass::Punct),
    (c"return:]", CharClass::Return),
    (c"space:]", CharClass::Space),
    (c"tab:]", CharClass::Tab),
    (c"upper:]", CharClass::Upper),
    (c"xdigit:]", CharClass::Xdigit),
];

/// Recognise a `[:alpha:]`-style class at `text[*at]`, which is the `[`. On
/// a hit `*at` advances past the name — the caller has already consumed the
/// `[`, and the name carries its own `:]` — and the class is returned.
/// Otherwise `None`, with `*at` untouched.
///
/// `text` is the pattern with its NUL, and reads past it answer NUL.
pub(crate) fn take_char_class(text: &[u8], at: &mut usize) -> Option<CharClass> {
    let p = *at;
    let byte = |off: usize| cstr::byte_at(text, p + off);
    // Only `[:` followed by at least three lowercase letters is a
    // candidate. That is load-bearing, not just a guard against reading
    // off the end: `[:a:]` is a literal, not a class.
    if byte(1) != b':' || !(2..5).all(|off| byte(off).is_ascii_lowercase()) {
        return None;
    }
    // Order the pattern text against a class name the way the C
    // `strncmp` over the name's length did: byte by byte, stopping at
    // the first difference. A NUL in the pattern therefore just
    // compares low, and no byte past it is read.
    let cmp = |entry: &CStr| {
        for (i, &want) in entry.to_bytes().iter().enumerate() {
            match byte(2 + i).cmp(&want) {
                Ordering::Equal => {}
                other => return other,
            }
        }
        Ordering::Equal
    };
    // `binary_search_by` orders each entry against the needle; `cmp` reads
    // the other way round. (Upstream cached the last hit; nineteen entries
    // are five comparisons, and the cache was a global.)
    let i = CHAR_CLASS_TAB
        .binary_search_by(|(entry, _)| cmp(entry).reverse())
        .ok()?;
    *at = p + 2 + CHAR_CLASS_TAB[i].0.to_bytes().len();
    Some(CHAR_CLASS_TAB[i].1)
}

/// Recognise the one other shape a `[]` collection can bracket: an
/// equivalence class `[=a=]` (`delim` is `=`) or a collation element
/// `[.a.]` (`delim` is `.`). `text[*at]` is the `[`; on a hit `*at` advances
/// past the whole item and the character between the delimiters is
/// returned, else 0 with `*at` untouched.
///
/// Upstream splits this in two and only null-checks `p[0]` in the
/// collation-element half. Both callers hold `p[0] == '['`, so the check
/// never fires either way; it is applied to both here so that a pattern
/// ending mid-item can never walk past its terminator.
pub(crate) fn take_bracketed(text: &[u8], at: &mut usize, delim: u8) -> c_int {
    let p = *at;
    let byte = |off: usize| cstr::byte_at(text, p + off);
    if byte(0) == 0 || byte(1) != delim || byte(2) == 0 {
        return 0;
    }
    let len = cluster_len_at(text, p + 2);
    if byte(len + 2) != delim || byte(len + 3) != b']' {
        return 0;
    }
    *at = p + len + 4;
    char_at_nul(text, p + 2)
}
