//! Character classes, and what is printable.
//!
//! A character's *class* is what `w`, `b`, `iw` and friends compare when they
//! decide where a word ends: two adjacent characters belong to the same word
//! exactly when they answer the same class. Three classes have a fixed
//! meaning — blank, punctuation, word — and a fourth, emoji, exists so a run
//! of emoji is one word. Every other answer is the first codepoint of a
//! *script block*, which gives each script its own class without needing a
//! number per script: Hiragana answers `0x3040`, Katakana `0x30a0`, and all
//! the CJK ideograph blocks answer `0x4e00` so they run together.
//!
//! Latin-1 never reaches the table. It is the range `'iskeyword'` lets a user
//! reconfigure, so it is decided by the buffer's own character table instead.
//!
//! Printability is a separate question, asked by `strtrans()` and by the JSON
//! encoder: whether a codepoint has a glyph at all, or is a formatting
//! control, a surrogate half, or a non-character.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use core::cmp::Ordering;
use core::ffi::{c_char, c_int};

/// The three classes every character can have, plus emoji.
///
/// Any *other* answer from [`utf_class_tab`] is a script block's first
/// codepoint (see the module docs), so callers test `>= CLASS_WORD` rather
/// than `== CLASS_WORD` when they mean "part of a word".
const CLASS_BLANK: c_int = 0;
const CLASS_PUNCT: c_int = 1;
const CLASS_WORD: c_int = 2;
const CLASS_EMOJI: c_int = 3;

/// Every codepoint in `first..=last` has class `class`.
///
/// Sorted by `first` and non-overlapping — [`utf_class_tab`] binary-searches
/// it, and the `const` assertion below is what holds that true. Upstream's
/// comments travel with the rows they annotate.
#[rustfmt::skip]
const CLASSES: [(u32, u32, c_int); 71] = [
    (0x037e, 0x037e, CLASS_PUNCT),    // Greek question mark
    (0x0387, 0x0387, CLASS_PUNCT),    // Greek ano teleia
    (0x055a, 0x055f, CLASS_PUNCT),    // Armenian punctuation
    (0x0589, 0x0589, CLASS_PUNCT),    // Armenian full stop
    (0x05be, 0x05be, CLASS_PUNCT),
    (0x05c0, 0x05c0, CLASS_PUNCT),
    (0x05c3, 0x05c3, CLASS_PUNCT),
    (0x05f3, 0x05f4, CLASS_PUNCT),
    (0x060c, 0x060c, CLASS_PUNCT),
    (0x061b, 0x061b, CLASS_PUNCT),
    (0x061f, 0x061f, CLASS_PUNCT),
    (0x066a, 0x066d, CLASS_PUNCT),
    (0x06d4, 0x06d4, CLASS_PUNCT),
    (0x0700, 0x070d, CLASS_PUNCT),    // Syriac punctuation
    (0x0964, 0x0965, CLASS_PUNCT),
    (0x0970, 0x0970, CLASS_PUNCT),
    (0x0df4, 0x0df4, CLASS_PUNCT),
    (0x0e4f, 0x0e4f, CLASS_PUNCT),
    (0x0e5a, 0x0e5b, CLASS_PUNCT),
    (0x0f04, 0x0f12, CLASS_PUNCT),
    (0x0f3a, 0x0f3d, CLASS_PUNCT),
    (0x0f85, 0x0f85, CLASS_PUNCT),
    (0x104a, 0x104f, CLASS_PUNCT),    // Myanmar punctuation
    (0x10fb, 0x10fb, CLASS_PUNCT),    // Georgian punctuation
    (0x1361, 0x1368, CLASS_PUNCT),    // Ethiopic punctuation
    (0x166d, 0x166e, CLASS_PUNCT),    // Canadian Syl. punctuation
    (0x1680, 0x1680, CLASS_BLANK),
    (0x169b, 0x169c, CLASS_PUNCT),
    (0x16eb, 0x16ed, CLASS_PUNCT),
    (0x1735, 0x1736, CLASS_PUNCT),
    (0x17d4, 0x17dc, CLASS_PUNCT),    // Khmer punctuation
    (0x1800, 0x180a, CLASS_PUNCT),    // Mongolian punctuation
    (0x2000, 0x200b, CLASS_BLANK),    // spaces
    (0x200c, 0x2027, CLASS_PUNCT),    // punctuation and symbols
    (0x2028, 0x2029, CLASS_BLANK),
    (0x202a, 0x202e, CLASS_PUNCT),    // punctuation and symbols
    (0x202f, 0x202f, CLASS_BLANK),
    (0x2030, 0x205e, CLASS_PUNCT),    // punctuation and symbols
    (0x205f, 0x205f, CLASS_BLANK),
    (0x2060, 0x206f, CLASS_PUNCT),    // punctuation and symbols
    (0x2070, 0x207f, 0x2070),         // superscript
    (0x2080, 0x2094, 0x2080),         // subscript
    (0x20a0, 0x27ff, CLASS_PUNCT),    // all kinds of symbols
    (0x2800, 0x28ff, 0x2800),         // braille
    (0x2900, 0x2998, CLASS_PUNCT),    // arrows, brackets, etc.
    (0x29d8, 0x29db, CLASS_PUNCT),
    (0x29fc, 0x29fd, CLASS_PUNCT),
    (0x2e00, 0x2e7f, CLASS_PUNCT),    // supplemental punctuation
    (0x3000, 0x3000, CLASS_BLANK),    // ideographic space
    (0x3001, 0x3020, CLASS_PUNCT),    // ideographic punctuation
    (0x3030, 0x3030, CLASS_PUNCT),
    (0x303d, 0x303d, CLASS_PUNCT),
    (0x3040, 0x309f, 0x3040),         // Hiragana
    (0x30a0, 0x30ff, 0x30a0),         // Katakana
    (0x3300, 0x9fff, 0x4e00),         // CJK Ideographs
    (0xac00, 0xd7a3, 0xac00),         // Hangul Syllables
    (0xf900, 0xfaff, 0x4e00),         // CJK Ideographs
    (0xfd3e, 0xfd3f, CLASS_PUNCT),
    (0xfe30, 0xfe6b, CLASS_PUNCT),    // punctuation forms
    (0xff00, 0xff0f, CLASS_PUNCT),    // half/fullwidth ASCII
    (0xff1a, 0xff20, CLASS_PUNCT),    // half/fullwidth ASCII
    (0xff3b, 0xff40, CLASS_PUNCT),    // half/fullwidth ASCII
    (0xff5b, 0xff65, CLASS_PUNCT),    // half/fullwidth ASCII
    (0x1d000, 0x1d24f, CLASS_PUNCT),  // Musical notation
    (0x1d400, 0x1d7ff, CLASS_PUNCT),  // Mathematical Alphanumeric Symbols
    (0x1f000, 0x1f2ff, CLASS_PUNCT),  // Game pieces; enclosed characters
    (0x1f300, 0x1f9ff, CLASS_PUNCT),  // Many symbol blocks
    (0x20000, 0x2a6df, 0x4e00),       // CJK Ideographs
    (0x2a700, 0x2b73f, 0x4e00),       // CJK Ideographs
    (0x2b740, 0x2b81f, 0x4e00),       // CJK Ideographs
    (0x2f800, 0x2fa1f, 0x4e00),       // CJK Ideographs
];

/// Codepoints with no glyph: formatting controls, the surrogate halves (which
/// exist only inside UTF-16 and are illegal here), the byte-order mark, the
/// interlinear annotation controls and the two non-characters at the end of
/// the BMP. Sorted and non-overlapping, for the same reason [`CLASSES`] is.
const NONPRINTABLE: [(c_int, c_int); 9] = [
    (0x070f, 0x070f),
    (0x180b, 0x180e),
    (0x200b, 0x200f),
    (0x202a, 0x202e),
    (0x2060, 0x206f),
    (0xd800, 0xdfff),
    (0xfeff, 0xfeff),
    (0xfff9, 0xfffb),
    (0xfffe, 0xffff),
];

/// Both tables are binary-searched, which is only correct while they are
/// sorted and their ranges do not touch. Checked at compile time, because
/// the only way to break it is to edit a row.
const fn ranges_are_searchable<const N: usize>(rows: [(u32, u32); N]) -> bool {
    let mut i = 0;
    while i < N {
        if rows[i].0 > rows[i].1 {
            return false;
        }
        if i > 0 && rows[i - 1].1 >= rows[i].0 {
            return false;
        }
        i += 1;
    }
    true
}

const _: () = {
    let mut bounds = [(0u32, 0u32); CLASSES.len()];
    let mut i = 0;
    while i < CLASSES.len() {
        bounds[i] = (CLASSES[i].0, CLASSES[i].1);
        i += 1;
    }
    assert!(
        ranges_are_searchable(bounds),
        "CLASSES must be sorted and disjoint"
    );

    let mut bounds = [(0u32, 0u32); NONPRINTABLE.len()];
    let mut i = 0;
    while i < NONPRINTABLE.len() {
        bounds[i] = (NONPRINTABLE[i].0 as u32, NONPRINTABLE[i].1 as u32);
        i += 1;
    }
    assert!(
        ranges_are_searchable(bounds),
        "NONPRINTABLE must be sorted and disjoint"
    );
};

/// The class [`CLASSES`] gives `c`, or `None` if no range covers it.
fn table_class(c: u32) -> Option<c_int> {
    CLASSES
        .binary_search_by(|&(first, last, _)| {
            if last < c {
                Ordering::Less
            } else if first > c {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
        .ok()
        .map(|i| CLASSES[i].2)
}

/// Is this character an emoji, or a flag's regional-indicator half?
///
/// Both are "extended pictographic" as far as word motions care: a run of
/// them is one word, and a flag is two regional indicators that must not be
/// split. Takes the property by reference, which is what
/// `utf8proc_get_property` hands back, so the whole test is safe code.
pub(crate) fn prop_is_emojilike(prop: &utf8proc_property_t) -> bool {
    prop.boundclass as c_int == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as c_int
        || prop.boundclass as c_int == UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR as c_int
}

/// Does `c` have a visible glyph?
///
/// The inverse of [`NONPRINTABLE`]; everything outside that table's range
/// answers yes, including codepoints Unicode has not assigned.
pub fn utf_printable(c: c_int) -> bool {
    NONPRINTABLE
        .binary_search_by(|&(first, last)| {
            if last < c {
                Ordering::Less
            } else if first > c {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
        .is_err()
}

/// The character class of `c`, deciding Latin-1 with `chartab`.
///
/// See the module docs for what the answer means. Anything below `0x100` is
/// `'iskeyword'`'s to decide; above that the answer is fixed by Unicode, so
/// the table and utf8proc settle it.
///
/// # Safety
///
/// `chartab` must be a buffer's `b_chartab`, the four-word `'iskeyword'`
/// bitmap `vim_iswordc_tab` indexes.
pub unsafe fn utf_class_tab(c: c_int, chartab: *const uint64_t) -> c_int {
    if c < 0x100 {
        // NBSP is a blank here even though 'iskeyword' could claim it.
        if c == ' ' as c_int || c == TAB || c == NUL || c == 0xa0 {
            return CLASS_BLANK;
        }
        // SAFETY: the caller's obligation, forwarded unchanged.
        return if unsafe { vim_iswordc_tab(c, chartab) } {
            CLASS_WORD
        } else {
            CLASS_PUNCT
        };
    }

    if prop_is_emojilike(utf8proc_get_property(c)) {
        return CLASS_EMOJI;
    }

    // `c` is above 0x100 here, so the unsigned comparison the C spells out is
    // the signed one; most characters are in no range at all, and those are
    // word characters.
    table_class(c as u32).unwrap_or(CLASS_WORD)
}

/// [`utf_class_tab`] against the current buffer's `'iskeyword'`.
pub unsafe fn utf_class(c: c_int) -> c_int {
    unsafe { utf_class_tab(c, &raw const (*curbuf.get()).b_chartab as *const uint64_t) }
}

/// The character class of the character `p` points at.
///
/// A single-byte character is settled without decoding: `'iskeyword'` covers
/// the whole of Latin-1, which is every value one byte can hold.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string, and `chartab` is
/// [`utf_class_tab`]'s.
pub unsafe fn mb_get_class_tab(p: *const c_char, chartab: *const uint64_t) -> c_int {
    unsafe {
        let first = *p as u8;
        if utf8len_tab[first as usize] == 1 {
            if first == 0 || ascii_iswhite(first as c_int) {
                return CLASS_BLANK;
            }
            return if vim_iswordc_tab(first as c_int, chartab) {
                CLASS_WORD
            } else {
                CLASS_PUNCT
            };
        }
        utf_class_tab(utf_ptr2char(p), chartab)
    }
}

/// [`mb_get_class_tab`] against the current buffer's `'iskeyword'`.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub unsafe fn mb_get_class(p: *const c_char) -> c_int {
    unsafe { mb_get_class_tab(p, &raw const (*curbuf.get()).b_chartab as *const uint64_t) }
}

/// `charclass({string})` — the class of the string's first character.
pub unsafe fn f_charclass(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        if tv_check_for_string_arg(argvars, 0) == FAIL || (*argvars).vval.v_string.is_null() {
            return;
        }
        (*rettv).vval.v_number = mb_get_class((*argvars).vval.v_string) as varnumber_T;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One representative per fixed class, and one per script block, so a
    /// misplaced row shows up as a class rather than as a moved boundary.
    #[test]
    fn table_classes() {
        assert_eq!(table_class(0x3000), Some(CLASS_BLANK)); // ideographic space
        assert_eq!(table_class(0x037e), Some(CLASS_PUNCT)); // Greek question mark
        assert_eq!(table_class(0x3041), Some(0x3040)); // Hiragana
        assert_eq!(table_class(0x30ff), Some(0x30a0)); // Katakana
        assert_eq!(table_class(0x4e00), Some(0x4e00)); // CJK
        assert_eq!(table_class(0x2f800), Some(0x4e00)); // CJK, last row
        assert_eq!(table_class(0xac00), Some(0xac00)); // Hangul
    }

    /// Every gap between rows, and both ends of the table, answer "no range".
    #[test]
    fn gaps_are_unclassified() {
        assert_eq!(table_class(0x0100), None);
        assert_eq!(table_class(0x037d), None);
        assert_eq!(table_class(0x037f), None);
        assert_eq!(table_class(0x2fa20), None);
        for w in CLASSES.windows(2) {
            let after = w[0].1 + 1;
            if after < w[1].0 {
                assert_eq!(table_class(after), None, "{after:#x}");
            }
        }
    }

    #[test]
    fn printability() {
        // Both ends of every unprintable range, and the characters either
        // side of it.
        for &(first, last) in &NONPRINTABLE {
            assert!(!utf_printable(first), "{first:#x}");
            assert!(!utf_printable(last), "{last:#x}");
            assert!(utf_printable(first - 1), "{:#x}", first - 1);
            if last != 0xffff {
                assert!(utf_printable(last + 1), "{:#x}", last + 1);
            }
        }
        assert!(utf_printable('a' as c_int));
        assert!(utf_printable(0x10000)); // past the table
        assert!(utf_printable(0)); // NUL is "printable"; callers filter it
    }
}
