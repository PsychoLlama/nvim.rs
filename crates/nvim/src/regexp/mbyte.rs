//! Comparing multibyte text under 'ignorecase' and `\Z`: the Hebrew
//! decomposition table, the case- and composing-aware string compare, and
//! the case-folded character search.

#![forbid(unsafe_code)]

use crate::cstr;
use core::ffi::c_int;

use super::Rex;
use crate::charset::{is_word_at, vim_iswordc_buf};
use crate::mbyte::{
    c_cluster_len, char_at, char_len, head_off, iscomposing, strnicmp_in, utf_fold,
    utf_is_trail_byte, utf8len_tab,
};
use crate::memory::{memchr_in, strchr_in, strlen_in};
use crate::strings::find_char;
use crate::types::GraphemeState;
use crate::winlayer::Buf;

/// The base characters of U+FB20..U+FB4F, the Hebrew presentation forms, so
/// that `\Z` can treat a precomposed form and its parts as equal. Entries
/// are `[base, point, dagesh]` with unused slots left 0; a few forms
/// decompose to another presentation form rather than to base letters.
static DECOMP_TABLE: [[c_int; 3]; 48] = [
    [0x5e2, 0, 0],
    [0x5d0, 0, 0],
    [0x5d3, 0, 0],
    [0x5d4, 0, 0],
    [0x5db, 0, 0],
    [0x5dc, 0, 0],
    [0x5dd, 0, 0],
    [0x5e8, 0, 0],
    [0x5ea, 0, 0],
    [b'+' as c_int, 0, 0],
    [0x5e9, 0x5c1, 0],
    [0x5e9, 0x5c2, 0],
    [0x5e9, 0x5c1, 0x5bc],
    [0x5e9, 0x5c2, 0x5bc],
    [0x5d0, 0x5b7, 0],
    [0x5d0, 0x5b8, 0],
    [0x5d0, 0x5b4, 0],
    [0x5d1, 0x5bc, 0],
    [0x5d2, 0x5bc, 0],
    [0x5d3, 0x5bc, 0],
    [0x5d4, 0x5bc, 0],
    [0x5d5, 0x5bc, 0],
    [0x5d6, 0x5bc, 0],
    [0xfb37, 0, 0],
    [0x5d8, 0x5bc, 0],
    [0x5d9, 0x5bc, 0],
    [0x5da, 0x5bc, 0],
    [0x5db, 0x5bc, 0],
    [0x5dc, 0x5bc, 0],
    [0xfb3d, 0, 0],
    [0x5de, 0x5bc, 0],
    [0xfb3f, 0, 0],
    [0x5e0, 0x5bc, 0],
    [0x5e1, 0x5bc, 0],
    [0xfb42, 0, 0],
    [0x5e3, 0x5bc, 0],
    [0x5e4, 0x5bc, 0],
    [0xfb45, 0, 0],
    [0x5e6, 0x5bc, 0],
    [0x5e7, 0x5bc, 0],
    [0x5e8, 0x5bc, 0],
    [0x5e9, 0x5bc, 0],
    [0x5ea, 0x5bc, 0],
    [0x5d5, 0x5b9, 0],
    [0x5d1, 0x5bf, 0],
    [0x5db, 0x5bf, 0],
    [0x5e4, 0x5bf, 0],
    [0x5d0, 0x5dc, 0],
];

/// Split a Hebrew presentation form into its parts. Anything else is its
/// own first part, with the other two 0.
pub(crate) fn decompose(c: c_int) -> [c_int; 3] {
    match usize::try_from(c - 0xfb20) {
        Ok(i) if i < DECOMP_TABLE.len() => DECOMP_TABLE[i],
        _ => [c, 0, 0],
    }
}

/// `strncmp` over two NUL-terminated slices: the first difference within
/// `n` bytes, stopping at a terminator.
#[inline]
fn compare_prefix(s1: &[u8], s2: &[u8], n: usize) -> c_int {
    for i in 0..n {
        let (a, b) = (cstr::byte_at(s1, i), cstr::byte_at(s2, i));
        if a != b {
            return if a < b { -1 } else { 1 };
        }
        if a == 0 {
            break;
        }
    }
    0
}

/// Compare `*n` bytes of `s1` against `s2` under the current 'ignorecase'
/// and `\Z` settings. On a match `*n` is lowered to the number of bytes of
/// `s2` that were consumed, which can differ from the bytes of `s1` when a
/// case fold or a decomposition changed the encoded length.
///
/// Both are NUL-terminated slices; nothing past a terminator is read.
pub(crate) fn cstrncmp(rex: Rex, s1: &[u8], s2: &[u8], n: &mut c_int) -> c_int {
    // A negative count was a huge `size_t` upstream: no bound but the NUL.
    let bound = |n: c_int| usize::try_from(n).unwrap_or(usize::MAX);
    let mut result = if !rex.reg_ic() {
        compare_prefix(s1, s2, bound(*n))
    } else {
        // Count the characters `*n` bytes of s1 spans, then measure how
        // many bytes the same count takes in s2. NB: upstream subtracts
        // the length of s1's *first* character every time round rather
        // than the current one's, so a multibyte s1 overcounts the
        // characters; the result feeds only the length hint below, and
        // the quirk is load-bearing for what `*n` comes back as.
        let first = cluster_len_at(s1, 0) as c_int;
        let mut p = 0;
        let mut chars = 0;
        let mut left = *n;
        while left > 0 && cstr::byte_at(s1, p) != 0 {
            left -= first;
            p += cluster_len_at(s1, p);
            chars += 1;
        }
        p = 0;
        while chars > 0 && cstr::byte_at(s2, p) != 0 {
            chars -= 1;
            p += cluster_len_at(s2, p);
        }
        let n2 = p as c_int;
        // `strnicmp_in` ends either side at a NUL itself.
        let result = strnicmp_in(head(s1, bound(*n)), head(s2, bound(n2)));
        if result == 0 && n2 < *n {
            *n = n2;
        }
        result
    };

    // `\Z`: differences that are only in the composing characters, or
    // that decomposition erases, don't count.
    if result != 0 && rex.reg_icombine() {
        let (mut str1, mut str2) = (0usize, 0usize);
        let mut c1 = 0;
        let mut c2 = 0;
        while (str1 as isize) < *n as isize {
            c1 = char_at_nul(s1, str1);
            str1 += cluster_len_at(s1, str1);
            c2 = char_at_nul(s2, str2);
            str2 += cluster_len_at(s2, str2);
            if c1 == c2 || (rex.reg_ic() && utf_fold(c1) == utf_fold(c2)) {
                continue;
            }
            c1 = decompose(c1)[0];
            c2 = decompose(c2)[0];
            if c1 != c2 && (!rex.reg_ic() || utf_fold(c1) != utf_fold(c2)) {
                break;
            }
        }
        result = c2 - c1;
        if result == 0 {
            *n = str2 as c_int;
        }
    }
    result
}

/// `strchr` that honours 'ignorecase': where `c` is in the string `s`
/// holds, matching either case when the search is case-insensitive.
///
/// `s` ends in a NUL, and the string is what comes before its first one —
/// a line from the cursor on, or a `[]` operand inside the program (which
/// ends in the `END` node's NUL offset). That is what lets the plain search
/// be the C library's `strchr`, as upstream's was: lines are long and core's
/// search costs several times as much per byte.
///
/// # Panics
/// If `s` does not end in a NUL.
#[inline(always)]
pub(crate) fn cstrchr(rex: Rex, s: &[u8], c: c_int) -> Option<usize> {
    if !rex.reg_ic() && (0..0x80).contains(&c) {
        return u8::try_from(c).ok().and_then(|c| strchr_in(s, c));
    }
    cstrchr_in(rex, &s[..strlen_in(s)], c)
}

/// [`cstrchr`] over a line from the cursor on, whose only NUL is its last
/// byte: the string is all of it but that, so no search for the end is
/// needed and the plain search is the C library's `memchr`.
#[inline(always)]
pub(crate) fn cstrchr_line(rex: Rex, rest: &[u8], c: c_int) -> Option<usize> {
    cstrchr_in(rex, &rest[..rest.len() - 1], c)
}

/// [`cstrchr`] over the string itself, its terminator left off.
#[inline(always)]
fn cstrchr_in(rex: Rex, text: &[u8], c: c_int) -> Option<usize> {
    if !rex.reg_ic() {
        return find_char_in(text, c);
    }
    // `cc` is the other case of `c`, `lc` the folded form to compare
    // non-ASCII against. Characters with no other case take the plain
    // search.
    let (cc, lc);
    if c > 0x80 {
        cc = utf_fold(c);
        lc = cc;
    } else if (b'A' as c_int..=b'Z' as c_int).contains(&c) {
        cc = c + (b'a' - b'A') as c_int;
        lc = cc;
    } else if (b'a' as c_int..=b'z' as c_int).contains(&c) {
        cc = c - (b'a' - b'A') as c_int;
        lc = c;
    } else {
        return find_char_in(text, c);
    }

    let mut p = 0;
    while p < text.len() {
        let byte = text[p];
        let uc = char_at_nul(text, p);
        if c > 0x80 || uc > 0x80 {
            // Skip the ASCII case: a multibyte fold must not match a
            // byte that already compared equal below.
            if (uc < 0x80 || uc != c_int::from(byte)) && utf_fold(uc) == lc {
                return Some(p);
            }
        } else if c_int::from(byte) == c || c_int::from(byte) == cc {
            return Some(p);
        }
        p += cluster_len_at(text, p);
    }
    None
}

/// `vim_strchr` over a string held without its terminator.
#[inline(always)]
fn find_char_in(text: &[u8], c: c_int) -> Option<usize> {
    if c <= 0 {
        return None;
    }
    if c < 0x80 {
        return memchr_in(text, c as u8);
    }
    find_char(text, c)
}

/// `strlen` over a NUL-terminated operand or line: the C library's.
#[inline(always)]
pub(crate) fn nul_terminated_len(s: &[u8]) -> usize {
    strlen_in(s)
}

// ------------------------------------------------- the line, as bytes
//
// Both engines and the pattern parser read NUL-terminated text: a line of the
// buffer, the string a match runs over, the pattern itself. They hold it as a
// slice that *keeps* its terminator, because the terminator is a position they
// test for rather than a bound they stop short of (`$` matches there, `\n` is
// looked for there). These are the codec's readers over such a slice at an
// offset, answering what the pointer forms answer at the same byte -- in
// particular 0 at the terminator, where the slice forms count a NUL as a
// character of its own.

/// The character at `text[i]`; 0 at the terminator or past the end.
#[inline(always)]
pub(crate) fn char_at_nul(text: &[u8], i: usize) -> i32 {
    match text.get(i) {
        None => 0,
        // ASCII, the NUL included, is itself: the hot answer.
        Some(&byte) if byte < 0x80 => i32::from(byte),
        Some(_) => char_at(&text[i..]),
    }
}

/// How many bytes the character at `text[i]` takes; 0 at the terminator.
#[inline(always)]
pub(crate) fn char_len_at(text: &[u8], i: usize) -> usize {
    match text.get(i) {
        None | Some(0) => 0,
        Some(_) => char_len(&text[i..]),
    }
}

/// How many bytes the character at `text[i]` takes with its combining
/// characters; 0 at the terminator.
#[inline(always)]
pub(crate) fn cluster_len_at(text: &[u8], i: usize) -> usize {
    match text.get(i..) {
        None | Some([] | [0, ..]) => 0,
        // An ASCII byte followed by another cannot be combined with
        // anything: the hot answer, without decoding.
        Some(&[first, next, ..]) if first < 0x80 && next < 0x80 => 1,
        Some(rest) => cluster_len_slow(rest),
    }
}

/// [`cluster_len_at`] past its ASCII answer, out of line so that every
/// caller inlines only that.
///
/// A whole multibyte character followed by ASCII is a cluster of its own:
/// the answer the full walk reaches on its first look past the character,
/// without its per-byte reads.
#[inline(never)]
fn cluster_len_slow(rest: &[u8]) -> usize {
    let first = rest[0];
    if first >= 0x80 {
        let len = usize::from(utf8len_tab[usize::from(first)]);
        let whole = len >= 2
            && rest
                .get(1..len)
                .is_some_and(|tail| tail.iter().all(|&byte| utf_is_trail_byte(byte)));
        if whole && rest.get(len).is_none_or(|&byte| byte < 0x80) {
            return len;
        }
    }
    c_cluster_len(rest)
}

/// How far back from `text[i]` the character covering it starts.
#[inline(always)]
pub(crate) fn head_off_at(text: &[u8], i: usize) -> usize {
    head_off(text, i)
}

/// Does the character at `text[next]` compose onto the one at `text[cur]`?
#[inline(always)]
pub(crate) fn composes_at(
    text: &[u8],
    cur: usize,
    next: usize,
    state: Option<&mut GraphemeState>,
) -> bool {
    // ASCII never combines, and this is the hot answer.
    cstr::byte_at(text, next) >= 0x80
        && iscomposing(char_at_nul(text, cur), char_at_nul(text, next), state)
}

/// At most the first `n` bytes of `text`.
#[inline(always)]
fn head(text: &[u8], n: usize) -> &[u8] {
    &text[..n.min(text.len())]
}

/// Where the first NUL in `text` is, or its length: `strlen` over a slice,
/// through the C library's `memchr`, which core's search costs several times
/// as much as.
#[inline(always)]
pub(crate) fn nul_at(text: &[u8]) -> usize {
    memchr_in(text, 0).unwrap_or(text.len())
}

/// `text[i..]` up to its terminator, at most `n` bytes: what a pointer form
/// with a length looks at.
#[inline(always)]
pub(crate) fn prefix_at(text: &[u8], i: usize, n: usize) -> &[u8] {
    let rest = text.get(i..).unwrap_or(&[]);
    let rest = &rest[..n.min(rest.len())];
    &rest[..nul_at(rest)]
}

/// Is the character `text` starts with a keyword character of `buffer`'s?
/// [`is_word_at`] with ASCII answered first, without decoding.
#[inline(always)]
pub(crate) fn word_char_at(text: &[u8], buffer: Buf) -> bool {
    match text.first() {
        Some(&byte) if byte < 0x80 => vim_iswordc_buf(c_int::from(byte), buffer),
        _ => is_word_at(text, buffer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fast paths in [`cluster_len_at`] and [`char_at_nul`] answer what
    /// the readers they short-cut answer, over every lead byte followed by a
    /// spread of the bytes that decide anything: NUL, ASCII, continuation
    /// bytes, lead bytes, and the first byte of a combining mark.
    #[test]
    #[cfg_attr(miri, ignore = "exhaustive: two million reads, minutes under Miri")]
    fn the_fast_paths_answer_what_the_full_readers_answer() {
        let probes = [
            0u8, 0x41, 0x80, 0x81, 0xa9, 0xbf, 0xc3, 0xcc, 0xcd, 0xe2, 0xf0, 0xff,
        ];
        for a in 0..=255u8 {
            for &b in &probes {
                for &c in &probes {
                    for &d in &probes {
                        let text = [a, b, c, d, 0];
                        for i in 0..text.len() {
                            assert_eq!(
                                cluster_len_at(&text, i),
                                c_cluster_len(&text[i..]),
                                "cluster at {i} of {text:x?}"
                            );
                            let want = match text[i..].first() {
                                Some(_) => char_at(&text[i..]),
                                None => 0,
                            };
                            assert_eq!(char_at_nul(&text, i), want, "char at {i} of {text:x?}");
                        }
                    }
                }
            }
        }
    }

    /// A base character with combining marks after it is one cluster, and
    /// the fast path hands it to the full walk.
    #[test]
    fn a_character_with_combining_marks_is_one_cluster() {
        // é as e + U+0301, then x.
        let text = b"e\xcc\x81x\0";
        assert_eq!(cluster_len_at(text, 0), 3);
        // ü (two bytes) + U+0308, then NUL.
        let text = b"\xc3\xbc\xcc\x88\0";
        assert_eq!(cluster_len_at(text, 0), 4);
        assert_eq!(cluster_len_at(text, 4), 0);
    }
}
