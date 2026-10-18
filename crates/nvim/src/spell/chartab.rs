//! Which characters are letters, and what case they are in.
//!
//! Spell checking cannot use the locale's idea of a letter: a `.spl` file
//! has to mean the same thing wherever it is loaded. So every language
//! carries its own answer, and [`spelltab`] holds the one currently in
//! force — a byte-indexed table of "is a word character", "is upper case",
//! and the folded and upper-cased forms.
//!
//! Only bytes below 256 live in the table. Wider characters are classified
//! by [`utf_class`] instead, through [`spell_mb_isword_class`], and folded
//! and upper-cased by the general [`utf_fold`]/[`mb_toupper`].
//!
//! [`init_spell_chartab`] fills the table from the current encoding at
//! startup; a `.spl` file's `FOL`/`LOW`/`UPP` section replaces it wholesale
//! and sets [`did_set_spelltab`].
//!
//! On top of that sit the case helpers the rest of the subsystem uses:
//! [`captype`] classifies a word's capitalisation into the `WF_*` flags the
//! word tree stores, and [`make_case_word`] applies those flags back to a
//! folded word to reconstruct how it should be spelled.

#![forbid(unsafe_code)]

use crate::cstr::byte_at;
use crate::spell::WordFlags;
use crate::strings::find_char;
use crate::winlayer::Win;
use core::ffi::c_int;

use crate::mbyte::{
    c_cluster_len, char_at, char_len, encode_char, mb_islower, mb_isupper, mb_toupper, utf_class,
    utf_fold,
};
use crate::types::{Failed, MB_MAXBYTES, SpellTab, uint8_t};

use super::{MAXWLEN, did_set_spelltab, spelltab};

/// A table holding the ASCII answers: digits and letters are word
/// characters, `A`-`Z` are upper case, and everything else maps to itself.
///
/// Digits are included even though a word may not *start* with one; that
/// restriction is handled where words are looked up.
pub fn ascii_spell_chartab() -> SpellTab {
    let mut sp = SpellTab {
        st_isw: [false; 256],
        st_isu: [false; 256],
        st_fold: [0; 256],
        st_upper: [0; 256],
    };
    for i in 0..256 {
        sp.st_fold[i] = i as uint8_t;
        sp.st_upper[i] = i as uint8_t;
    }
    for i in b'0'..=b'9' {
        sp.st_isw[i as usize] = true;
    }
    for i in b'A'..=b'Z' {
        sp.st_isw[i as usize] = true;
        sp.st_isu[i as usize] = true;
        sp.st_fold[i as usize] = i + 0x20;
    }
    for i in b'a'..=b'z' {
        sp.st_isw[i as usize] = true;
        sp.st_upper[i as usize] = i - 0x20;
    }
    sp
}

/// The four one-byte questions the spell-check path asks the table in force.
///
/// They read a single element rather than copying the table, which is a
/// kilobyte and is asked about once per character.
///
/// `c` must be a byte value; every caller has already ruled out anything
/// above 255.
#[inline]
pub(crate) fn spelltab_isw(c: usize) -> bool {
    spelltab.get_field(|tab| &tab.st_isw[c])
}

/// Whether byte `c` is upper case. See [`spelltab_isw`].
#[inline]
pub(crate) fn spelltab_isu(c: usize) -> bool {
    spelltab.get_field(|tab| &tab.st_isu[c])
}

/// Byte `c` folded to lower case. See [`spelltab_isw`].
#[inline]
pub(crate) fn spelltab_fold(c: usize) -> uint8_t {
    spelltab.get_field(|tab| &tab.st_fold[c])
}

/// Byte `c` raised to upper case. See [`spelltab_isw`].
#[inline]
pub(crate) fn spelltab_upper(c: usize) -> uint8_t {
    spelltab.get_field(|tab| &tab.st_upper[c])
}

/// Reset [`spelltab`] to what the current encoding says, discarding
/// anything a `.spl` file installed. Called once at startup and again
/// whenever the spell files are reloaded.
pub fn init_spell_chartab() {
    did_set_spelltab.set(false);
    let mut tab = ascii_spell_chartab();
    for i in 128..256 {
        let f = utf_fold(i);
        let u = mb_toupper(i);

        tab.st_isu[i as usize] = mb_isupper(i);
        tab.st_isw[i as usize] = tab.st_isu[i as usize] || mb_islower(i);
        // The folded and upper-cased values of 0xb5 differ between latin1
        // and utf-8, which used to raise E763 for no good reason. Keep the
        // latin1 answer for both.
        tab.st_fold[i as usize] = if f < 256 { f as uint8_t } else { i as uint8_t };
        tab.st_upper[i as usize] = if u < 256 { u as uint8_t } else { i as uint8_t };
    }
    spelltab.set(tab);
}

/// `text` from byte `at` on, or nothing when `at` is at or past its end:
/// what the pointer walks read as the terminator.
#[inline]
pub(crate) fn tail(text: &[u8], at: usize) -> &[u8] {
    text.get(at..).unwrap_or(&[])
}

/// Whether `c` is one of the window's multibyte `MIDWORD` characters.
fn is_midword_mb(window: Win, c: c_int) -> bool {
    find_char(&window.syn_block().b_spell_ismw_mb, c).is_some()
}

/// Whether `p` starts with a word character.
///
/// `p` is the text from the position of interest to the end of its string;
/// its end, or a NUL in it, is the terminator.
///
/// A "midword" character — one listed in the language's `MIDWORD` — counts
/// as a word character when a word character follows it, so that
/// `they're` is one word but `they there` is two. That only works past the
/// first character of a word, which is all the callers need.
pub fn spell_iswordp(p: &[u8], window: Win) -> bool {
    let first = p.first().copied().unwrap_or(0);
    let mut s = p;
    if first != 0 && first < 0x80 && p.get(1).is_none_or(|&next| next < 0x80) {
        // Be quick for ASCII: one byte, and nothing composing on it.
        if window.syn_block().b_spell_ismw[usize::from(first)] {
            s = &p[1..];
        }
    } else {
        // An illegal byte is a cluster of its own and reads as its value,
        // so it lands in the table like any other byte below 256.
        let c = char_at(p);
        let midword = if c < 256 {
            window.syn_block().b_spell_ismw[c as usize]
        } else {
            is_midword_mb(window, c)
        };
        if midword {
            s = tail(p, c_cluster_len(p));
        }
    }

    let c = char_at(s);
    if c > 255 {
        return spell_mb_isword_class(utf_class(c), window);
    }
    spelltab_isw(c as usize)
}

/// Whether `p` starts with a word character, ignoring midword characters.
pub fn spell_iswordp_nmw(p: &[u8], window: Win) -> bool {
    let c = char_at(p);
    if c > 255 {
        return spell_mb_isword_class(utf_class(c), window);
    }
    spelltab_isw(c as usize)
}

/// Whether a character class from [`utf_class`] is a word character.
///
/// Only meaningful above 255. Unicode sub- and superscripts are excluded;
/// with `'spelloptions'` containing `cjk` the East Asian scripts are too.
fn spell_mb_isword_class(cl: c_int, window: Win) -> bool {
    if window.syn_block().b_cjk != 0 {
        return cl == 2 || cl == 0x2800;
    }
    cl >= 2 && cl != 0x2070 && cl != 0x2080 && cl != 3
}

/// Wide-character [`spell_iswordp`]: `w` is the tail of a character array
/// starting at the position of interest, its end or a zero the terminator.
pub(super) fn spell_iswordp_w(w: &[c_int], window: Win) -> bool {
    let first = w.first().copied().unwrap_or(0);
    let midword = if first < 256 {
        window.syn_block().b_spell_ismw[first as usize]
    } else {
        is_midword_mb(window, first)
    };
    let c = if midword {
        w.get(1).copied().unwrap_or(0)
    } else {
        first
    };

    if c > 255 {
        return spell_mb_isword_class(utf_class(c), window);
    }
    spelltab_isw(c as usize)
}

/// Case-fold the first `len` bytes of `text` into `out`, NUL terminated,
/// using the character definitions from the `.spl` file. Folding may change
/// the length.
///
/// `text` runs on past `len` to the end of its string: whether a Greek
/// sigma ends a word depends on what follows it.
///
/// Answers `Err` when the result does not fit in `out`, having still
/// terminated what was written.
pub fn spell_casefold(window: Win, text: &[u8], len: usize, out: &mut [u8]) -> Result<(), Failed> {
    casefold_len(window, text, len, out)
        .map(|_| ())
        .map_err(|_| Failed)
}

/// [`spell_casefold`], answering how many bytes it wrote before the
/// terminator: `Err` with that count when the result did not fit.
pub(crate) fn casefold_len(
    window: Win,
    text: &[u8],
    len: usize,
    out: &mut [u8],
) -> Result<usize, usize> {
    let buflen = out.len();
    if len >= buflen {
        if let Some(first) = out.first_mut() {
            *first = 0;
        }
        return Err(0);
    }

    let mut outi = 0;
    let mut p = 0;
    while p < len {
        if outi + MB_MAXBYTES > buflen {
            out[outi] = 0;
            return Err(outi);
        }
        // ASCII folds through the table, one byte for one; what the table
        // answers may still need two bytes.
        if let Some(&b) = text.get(p)
            && b < 0x80
        {
            p += 1;
            let folded = spelltab_fold(usize::from(b));
            if folded < 0x80 {
                out[outi] = folded;
                outi += 1;
            } else {
                outi += encode_char(c_int::from(folded), &mut out[outi..]);
            }
            continue;
        }
        let rest = tail(text, p);
        let mut c = char_at(rest);
        p += char_len(rest).max(1);
        if c == 0x3a3 || c == 0x3c2 {
            // Greek sigma folds to the final form at the end of a word
            // and to the medial form anywhere else.
            c = if p == len || !spell_iswordp(tail(text, p), window) {
                0x3c2
            } else {
                0x3c3
            };
        } else if c >= 128 {
            c = utf_fold(c);
        } else {
            c = c_int::from(spelltab_fold(c as usize));
        }
        outi += encode_char(c, &mut out[outi..]);
    }
    out[outi] = 0;
    Ok(outi)
}

/// How far the character at `at` in `text` reaches: one byte for ASCII
/// with nothing composing on it, else the cluster -- and one byte at a NUL
/// or the end, so a walk always moves.
#[inline]
pub(crate) fn step(text: &[u8], at: usize) -> usize {
    match text.get(at..at + 2) {
        Some(&[first, next]) if first < 0x80 && next < 0x80 && first != 0 => 1,
        _ => c_cluster_len(tail(text, at)).max(1),
    }
}

/// Classify the capitalisation of `text[..end]` as one of
/// `WordFlags::ONECAP`, `WordFlags::ALLCAP`, `WordFlags::KEEPCAP`, or zero
/// for all lower case. A NUL before `end` ends the word too.
///
/// `text` may run on past `end`: the scan for the first word character
/// reads the character at `end` before it asks whether the word is over,
/// as upstream's does.
///
/// `WordFlags::KEEPCAP` means the pattern is neither of the simple ones —
/// "MacBeth" — so the word tree has to store it spelled out.
pub fn captype(text: &[u8], end: usize) -> WordFlags {
    let at_end = |p: usize| p >= end || byte_at(text, p) == 0;
    let window = Win::current();
    // Skip over any leading non-word characters.
    let mut p = 0;
    while !spell_iswordp_nmw(tail(text, p), window) {
        if at_end(p) {
            return WordFlags::NONE;
        }
        p += step(text, p);
    }

    let c = char_at(tail(text, p));
    p += step(text, p);
    let mut allcap = is_upper(c);
    let firstcap = allcap;
    let mut past_second = false;

    while !at_end(p) {
        let rest = tail(text, p);
        if spell_iswordp_nmw(rest, window) {
            if !is_upper(char_at(rest)) {
                // A lower-case letter after two upper-case ones, or
                // after a mix, cannot be described by a flag.
                if past_second && allcap {
                    return WordFlags::KEEPCAP;
                }
                allcap = false;
            } else if !allcap {
                return WordFlags::KEEPCAP;
            }
            past_second = true;
        }
        p += step(text, p);
    }

    if allcap {
        WordFlags::ALLCAP
    } else if firstcap {
        WordFlags::ONECAP
    } else {
        WordFlags::NONE
    }
}

/// Whether `c` is upper case, by the spell table below 128 and by the
/// general rules above it.
fn is_upper(c: c_int) -> bool {
    if c >= 128 {
        mb_isupper(c)
    } else {
        spelltab_isu(c as usize)
    }
}

/// The upper-case form of `c`, by the spell table below 128 and the
/// general rules above it.
pub(super) fn spell_toupper(c: c_int) -> c_int {
    if c >= 128 {
        mb_toupper(c)
    } else {
        c_int::from(spelltab_upper(c as usize))
    }
}

/// Copy the C string `src` into `out` at `at`, cut so that terminator
/// included it fills no more than `room` bytes there: `xstrlcpy`. Answers
/// where the terminator went.
fn put_cstr(out: &mut [u8], at: usize, src: &[u8], room: usize) -> usize {
    let mut d = at;
    for &b in src.iter().take(room.saturating_sub(1)) {
        if b == 0 {
            break;
        }
        out[d] = b;
        d += 1;
    }
    out[d] = 0;
    d
}

/// Copy `word` into `out`, upper-casing (or folding) the first character,
/// NUL terminated. At most [`MAXWLEN`] bytes of `out` are used, terminator
/// included; the copy stops short rather than overflow. `word` ends at a
/// NUL or at its end.
pub fn onecap_copy(word: &[u8], out: &mut [u8], upper: bool) {
    let mut c = char_at(word);
    let first = if byte_at(word, 0) == 0 {
        0
    } else {
        char_len(word)
    };
    c = if upper {
        spell_toupper(c)
    } else if c >= 128 {
        utf_fold(c)
    } else {
        c_int::from(spelltab_fold(c as usize))
    };
    let l = encode_char(c, out);
    put_cstr(out, l, tail(word, first), MAXWLEN - l);
}

/// Copy `word` into `out` with every character upper-cased, NUL
/// terminated. At most [`MAXWLEN`] bytes of `out` are used; the copy stops
/// short rather than overflow. `word` ends at a NUL or at its end.
pub fn allcap_copy(word: &[u8], out: &mut [u8]) {
    let mut d = 0;
    let mut s = 0;
    while byte_at(word, s) != 0 {
        let rest = &word[s..];
        let mut c = char_at(rest);
        s += char_len(rest);
        if c == 0xdf {
            // German sharp s upper-cases to two characters; the second
            // goes through the usual path below.
            c = 'S' as c_int;
            if d >= MAXWLEN - 1 {
                break;
            }
            out[d] = b'S';
            d += 1;
        } else {
            c = spell_toupper(c);
        }

        if d >= MAXWLEN - MB_MAXBYTES {
            break;
        }
        d += encode_char(c, &mut out[d..]);
    }
    out[d] = 0;
}

/// The length in bytes of the first `flen` bytes' worth of *characters* of
/// `fword`, measured in the unfolded `word`.
///
/// Folding can change how many bytes a character takes, so a length into
/// the folded word has to be converted before it can index the original.
pub fn nofold_len(fword: &[u8], flen: usize, word: &[u8]) -> usize {
    let mut i = 0;
    let mut p = 0;
    while p < flen {
        i += 1;
        p += step(fword, p);
    }

    let mut p = 0;
    while i > 0 {
        i -= 1;
        p += step(word, p);
    }
    p
}

/// Write the folded word `fword` into `out` with the capitalisation that
/// `flags` (as produced by [`captype`]) describes, NUL terminated.
pub fn make_case_word(fword: &[u8], out: &mut [u8], flags: WordFlags) {
    if flags.has(WordFlags::ALLCAP) {
        allcap_copy(fword, out);
    } else if flags.has(WordFlags::ONECAP) {
        onecap_copy(fword, out, true);
    } else {
        put_cstr(out, 0, fword, usize::MAX);
    }
}

/// Whether byte `n` is one of `set`'s, up to a NUL in it. Like `strchr()`
/// but independent of the locale.
pub fn byte_in_str(set: &[u8], n: c_int) -> bool {
    set.iter()
        .take_while(|&&b| b != 0)
        .any(|&b| c_int::from(b) == n)
}
