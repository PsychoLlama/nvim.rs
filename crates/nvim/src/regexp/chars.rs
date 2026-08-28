//! Characters and character classes: the `magic` toggles, the byte class
//! table the `\d`/`\w`/`\s` atoms test against, and the three bracketed
//! items a `[]` collection can name — `[:alpha:]`, `[=a=]` and `[.a.]`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::cmp::Ordering;
use core::ffi::{CStr, c_char, c_int};

use super::{
    CLASS_ALNUM, CLASS_ALPHA, CLASS_BACKSPACE, CLASS_BLANK, CLASS_CNTRL, CLASS_DIGIT, CLASS_ESCAPE,
    CLASS_FNAME, CLASS_GRAPH, CLASS_IDENT, CLASS_KEYWORD, CLASS_LOWER, CLASS_NONE, CLASS_PRINT,
    CLASS_PUNCT, CLASS_RETURN, CLASS_SPACE, CLASS_TAB, CLASS_UPPER, CLASS_XDIGIT, MAGIC_ALL,
    RF_HASNL, RI_ALPHA, RI_DIGIT, RI_HEAD, RI_HEX, RI_LOWER, RI_OCTAL, RI_UPPER, RI_WHITE, RI_WORD,
    reg_cpo_lit, reg_magic,
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

/// The `[:name:]` classes a collection may contain, sorted by name so the
/// lookup below can binary-search them. The trailing `:]` is part of the
/// name: the `[` is already consumed when we get here, and matching the
/// close is what proves the item was a class and not literal text.
static CHAR_CLASS_TAB: [(&CStr, c_int); 19] = [
    (c"alnum:]", CLASS_ALNUM as c_int),
    (c"alpha:]", CLASS_ALPHA as c_int),
    (c"backspace:]", CLASS_BACKSPACE as c_int),
    (c"blank:]", CLASS_BLANK as c_int),
    (c"cntrl:]", CLASS_CNTRL as c_int),
    (c"digit:]", CLASS_DIGIT as c_int),
    (c"escape:]", CLASS_ESCAPE as c_int),
    (c"fname:]", CLASS_FNAME as c_int),
    (c"graph:]", CLASS_GRAPH as c_int),
    (c"ident:]", CLASS_IDENT as c_int),
    (c"keyword:]", CLASS_KEYWORD as c_int),
    (c"lower:]", CLASS_LOWER as c_int),
    (c"print:]", CLASS_PRINT as c_int),
    (c"punct:]", CLASS_PUNCT as c_int),
    (c"return:]", CLASS_RETURN as c_int),
    (c"space:]", CLASS_SPACE as c_int),
    (c"tab:]", CLASS_TAB as c_int),
    (c"upper:]", CLASS_UPPER as c_int),
    (c"xdigit:]", CLASS_XDIGIT as c_int),
];

/// The entry [`take_char_class`] matched last. Collections repeat a class
/// far more often than they vary it, and the hit skips the search.
static LAST_CLASS: GlobalCell<usize> = GlobalCell::new(0);

/// Recognise a `[:alpha:]`-style class at `*pp`, which points at the `[`.
/// On a hit `*pp` advances past the name — the caller has already consumed
/// the `[`, and the name carries its own `:]` — and the `CLASS_*` code is
/// returned. Otherwise `CLASS_NONE`, with `*pp` untouched.
///
/// # Safety
///
/// `*pp` must point into a NUL-terminated pattern.
pub(crate) unsafe fn take_char_class(pp: &mut *mut c_char) -> c_int {
    let p = *pp;
    // Only `[:` followed by at least three lowercase letters is a
    // candidate. That is load-bearing, not just a guard against reading
    // off the end: `[:a:]` is a literal, not a class.
    if unsafe { *p.add(1) } as u8 != b':'
        || !(2..5).all(|off| (unsafe { *p.add(off) } as u8).is_ascii_lowercase())
    {
        return CLASS_NONE as c_int;
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
    match hit {
        Some(i) => {
            LAST_CLASS.set(i);
            *pp = unsafe { p.add(2 + CHAR_CLASS_TAB[i].0.to_bytes().len()) };
            CHAR_CLASS_TAB[i].1
        }
        None => CLASS_NONE as c_int,
    }
}

pub unsafe fn re_multiline(prog: *const regprog_T) -> c_int {
    (unsafe { (*prog).regflags } & RF_HASNL as u32) as c_int
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
