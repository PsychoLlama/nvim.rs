//! Characters and character classes: the `magic` toggles, the byte class
//! table the `\d`/`\w`/`\s` atoms test against, and the three bracketed
//! items a `[]` collection can name — `[:alpha:]`, `[=a=]` and `[.a.]`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::cmp::Ordering;
use core::ffi::{CStr, c_char, c_int};

use super::{
    MAGIC_ALL, RF_HASNL, RI_ALPHA, RI_DIGIT, RI_HEAD, RI_HEX, RI_LOWER, RI_OCTAL, RI_UPPER,
    RI_WHITE, RI_WORD, reg_cpo_lit, reg_magic,
};
use crate::global_cell::GlobalCell;
use crate::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::option::cpo_has;
use crate::types::{CpoFlag, regprog_T};

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

/// The backslash an error message has to print in front of a metacharacter
/// for the message to echo what the user typed: none under `\v`, where
/// every metacharacter is bare, one otherwise.
pub(crate) fn magic_prefix() -> &'static str {
    if reg_magic.get() == MAGIC_ALL {
        ""
    } else {
        "\\"
    }
}

/// The control character a `\r`/`\t`/`\e`/`\b` abbreviation stands for.
/// Anything else is returned unchanged.
pub(crate) fn backslash_abbr(c: c_int) -> c_int {
    match c as u8 {
        b'r' => b'\r' as c_int,
        b't' => b'\t' as c_int,
        b'e' => 0x1b,
        b'b' => 0x08,
        _ => c,
    }
}

/// Which of the `RI_*` classes each byte belongs to. Only the ASCII range
/// is meaningful — the engines test `c < 256` before indexing, and
/// everything at or above 0x80 is classless here because a multibyte
/// character is classified by [`crate::mbyte`] instead.
pub(crate) static RI_FLAGS: [i16; 256] = build_ri_flags();

const fn build_ri_flags() -> [i16; 256] {
    let mut tab = [0i16; 256];
    let mut i = 0usize;
    while i < 256 {
        let b = i as u8;
        tab[i] = match b {
            b'0'..=b'7' => RI_DIGIT + RI_HEX + RI_OCTAL + RI_WORD,
            b'8'..=b'9' => RI_DIGIT + RI_HEX + RI_WORD,
            b'a'..=b'f' => RI_HEX + RI_WORD + RI_HEAD + RI_ALPHA + RI_LOWER,
            b'g'..=b'z' => RI_WORD + RI_HEAD + RI_ALPHA + RI_LOWER,
            b'A'..=b'F' => RI_HEX + RI_WORD + RI_HEAD + RI_ALPHA + RI_UPPER,
            b'G'..=b'Z' => RI_WORD + RI_HEAD + RI_ALPHA + RI_UPPER,
            b'_' => RI_WORD + RI_HEAD,
            _ => 0,
        } as i16;
        i += 1;
    }
    tab[b' ' as usize] |= RI_WHITE as i16;
    tab[b'\t' as usize] |= RI_WHITE as i16;
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

/// The entry [`take_char_class`] matched last. Collections repeat a class
/// far more often than they vary it, and the hit skips the search.
static LAST_CLASS: GlobalCell<usize> = GlobalCell::new(0);

/// Recognise a `[:alpha:]`-style class at `*pp`, which points at the `[`.
/// On a hit `*pp` advances past the name — the caller has already consumed
/// the `[`, and the name carries its own `:]` — and the class is returned.
/// Otherwise `None`, with `*pp` untouched.
///
/// # Safety
///
/// `*pp` must point into a NUL-terminated pattern.
pub(crate) unsafe fn take_char_class(pp: &mut *mut c_char) -> Option<CharClass> {
    let p = *pp;
    // Only `[:` followed by at least three lowercase letters is a
    // candidate. That is load-bearing, not just a guard against reading
    // off the end: `[:a:]` is a literal, not a class.
    if unsafe { *p.add(1) } as u8 != b':'
        || !(2..5).all(|off| (unsafe { *p.add(off) } as u8).is_ascii_lowercase())
    {
        return None;
    }
    let name = unsafe { p.add(2) };
    // Order the pattern text against a class name the way the C
    // `strncmp` over the name's length did: byte by byte, stopping at
    // the first difference. A NUL in the pattern therefore just
    // compares low, and no byte past it is read.
    let cmp = |entry: &CStr| {
        for (i, &want) in entry.to_bytes().iter().enumerate() {
            match (unsafe { *name.add(i) } as u8).cmp(&want) {
                Ordering::Equal => {}
                other => return other,
            }
        }
        Ordering::Equal
    };
    let last = LAST_CLASS.get();
    let hit = if cmp(CHAR_CLASS_TAB[last].0).is_eq() {
        Some(last)
    } else {
        // `binary_search_by` orders each entry against the needle;
        // `cmp` reads the other way round.
        CHAR_CLASS_TAB
            .binary_search_by(|(entry, _)| cmp(entry).reverse())
            .ok()
    };
    let i = hit?;
    LAST_CLASS.set(i);
    *pp = unsafe { p.add(2 + CHAR_CLASS_TAB[i].0.to_bytes().len()) };
    Some(CHAR_CLASS_TAB[i].1)
}

pub unsafe fn re_multiline(prog: *const regprog_T) -> bool {
    (unsafe { (*prog).regflags } & RF_HASNL as u32) != 0
}

/// Recognise the one other shape a `[]` collection can bracket: an
/// equivalence class `[=a=]` (`delim` is `=`) or a collation element
/// `[.a.]` (`delim` is `.`). `*pp` points at the `[`; on a hit it advances
/// past the whole item and the character between the delimiters is
/// returned, else 0 with `*pp` untouched.
///
/// Upstream splits this in two and only null-checks `p[0]` in the
/// collation-element half. Both callers hold `p[0] == '['`, so the check
/// never fires either way; it is applied to both here so that a pattern
/// ending mid-item can never walk past its terminator.
///
/// # Safety
///
/// `*pp` must point into a NUL-terminated pattern.
pub(crate) unsafe fn take_bracketed(pp: &mut *mut c_char, delim: u8) -> c_int {
    let p = *pp;
    if unsafe { *p } == 0 || unsafe { *p.add(1) } as u8 != delim || unsafe { *p.add(2) } == 0 {
        return 0;
    }
    let len = unsafe { utfc_ptr2len(p.add(2)) } as usize;
    if unsafe { *p.add(len + 2) } as u8 != delim || unsafe { *p.add(len + 3) } as u8 != b']' {
        return 0;
    }
    *pp = unsafe { p.add(len + 4) };
    unsafe { utf_ptr2char(p.add(2)) }
}

/// Cache whether 'cpoptions' contains `l`, which makes `\r`, `\t` and
/// friends literal inside a `[]` collection. Read once per compile rather
/// than per character.
pub(crate) fn refresh_cpo_flags() {
    let literal = cpo_has(CpoFlag::LITERAL);
    reg_cpo_lit.set(literal as c_int);
}
