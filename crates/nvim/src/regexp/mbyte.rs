//! Comparing multibyte text under 'ignorecase' and `\Z`: the Hebrew
//! decomposition table, the case- and composing-aware string compare, and
//! the case-folded character search.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use core::ffi::{c_char, c_int};

use super::Rex;
use crate::mbyte::{
    c_cluster_len, char_at, char_len, head_off, iscomposing, utf_fold, utf_ptr2char, utf_strnicmp,
    utfc_ptr2len,
};
use crate::strings::vim_strchr;
use crate::types::GraphemeState;

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

/// Compare `*n` bytes of `s1` against `s2` under the current 'ignorecase'
/// and `\Z` settings. On a match `*n` is lowered to the number of bytes of
/// `s2` that were consumed, which can differ from the bytes of `s1` when a
/// case fold or a decomposition changed the encoded length.
///
/// # Safety
///
/// `s1` and `s2` must point to NUL-terminated strings.
pub(crate) unsafe fn cstrncmp(rex: Rex, s1: *mut c_char, s2: *mut c_char, n: &mut c_int) -> c_int {
    let mut result = if !rex.reg_ic() {
        unsafe { cstr::prefix_cmp(s1, s2, *n as usize) as c_int }
    } else {
        // Count the characters `*n` bytes of s1 spans, then measure how
        // many bytes the same count takes in s2. NB: upstream subtracts
        // the length of s1's *first* character every time round rather
        // than the current one's, so a multibyte s1 overcounts the
        // characters; the result feeds only the length hint below, and
        // the quirk is load-bearing for what `*n` comes back as.
        let mut p = s1;
        let mut chars = 0;
        let mut left = *n;
        while left > 0 && unsafe { *p } != 0 {
            left -= unsafe { utfc_ptr2len(s1) };
            p = unsafe { p.add(utfc_ptr2len(p) as usize) };
            chars += 1;
        }
        p = s2;
        while chars > 0 && unsafe { *p } != 0 {
            chars -= 1;
            p = unsafe { p.add(utfc_ptr2len(p) as usize) };
        }
        let n2 = unsafe { p.offset_from(s2) } as c_int;
        let result = unsafe { utf_strnicmp(s1, s2, *n as usize, n2 as usize) };
        if result == 0 && n2 < *n {
            *n = n2;
        }
        result
    };

    // `\Z`: differences that are only in the composing characters, or
    // that decomposition erases, don't count.
    if result != 0 && rex.reg_icombine() {
        let mut str1: *const c_char = s1;
        let mut str2: *const c_char = s2;
        let mut c1 = 0;
        let mut c2 = 0;
        while unsafe { str1.offset_from(s1) } < *n as isize {
            c1 = unsafe { utf_ptr2char(str1) };
            str1 = str1.wrapping_add(unsafe { utfc_ptr2len(str1) } as usize);
            c2 = unsafe { utf_ptr2char(str2) };
            str2 = str2.wrapping_add(unsafe { utfc_ptr2len(str2) } as usize);
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
            *n = unsafe { str2.offset_from(s2) } as c_int;
        }
    }
    result
}

/// `strchr` that honours 'ignorecase': find `c` in `s`, matching either
/// case when the search is case-insensitive.
///
/// # Safety
///
/// `s` must point to a NUL-terminated string.
#[inline(always)]
pub(crate) unsafe fn cstrchr(rex: Rex, s: *const c_char, c: c_int) -> *mut c_char {
    if !rex.reg_ic() {
        return unsafe { vim_strchr(s, c) };
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
        return unsafe { vim_strchr(s, c) };
    }

    let mut p = s;
    while unsafe { *p } != 0 {
        let uc = unsafe { utf_ptr2char(p) };
        if c > 0x80 || uc > 0x80 {
            // Skip the ASCII case: a multibyte fold must not match a
            // byte that already compared equal below.
            if (uc < 0x80 || uc != unsafe { *p } as u8 as c_int) && utf_fold(uc) == lc {
                return p as *mut c_char;
            }
        } else if unsafe { *p } as u8 as c_int == c || unsafe { *p } as u8 as c_int == cc {
            return p as *mut c_char;
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }
    core::ptr::null_mut()
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
    match text.get(i..) {
        Some(rest) => char_at(rest),
        None => 0,
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
        Some(rest) => c_cluster_len(rest),
        None => 0,
    }
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
