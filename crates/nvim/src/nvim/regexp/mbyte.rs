//! Comparing multibyte text under 'ignorecase' and `\Z`: the Hebrew
//! decomposition table, the case- and composing-aware string compare, and
//! the case-folded character search.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::Rex;
use crate::src::nvim::mbyte::{
    mb_ptr2char_adv, utf_fold, utf_ptr2char, utf_strnicmp, utfc_ptr2len,
};
use crate::src::nvim::os::libc::strncmp;
use crate::src::nvim::strings::vim_strchr;

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
    unsafe {
        let mut result = if !rex.reg_ic() {
            strncmp(s1, s2, *n as usize)
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
            while left > 0 && *p != 0 {
                left -= utfc_ptr2len(s1);
                p = p.add(utfc_ptr2len(p) as usize);
                chars += 1;
            }
            p = s2;
            while chars > 0 && *p != 0 {
                chars -= 1;
                p = p.add(utfc_ptr2len(p) as usize);
            }
            let n2 = p.offset_from(s2) as c_int;
            let result = utf_strnicmp(s1, s2, *n as usize, n2 as usize);
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
            while str1.offset_from(s1) < *n as isize {
                c1 = mb_ptr2char_adv(&raw mut str1);
                c2 = mb_ptr2char_adv(&raw mut str2);
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
                *n = str2.offset_from(s2) as c_int;
            }
        }
        result
    }
}

/// `strchr` that honours 'ignorecase': find `c` in `s`, matching either
/// case when the search is case-insensitive.
///
/// # Safety
///
/// `s` must point to a NUL-terminated string.
#[inline(always)]
pub(crate) unsafe fn cstrchr(rex: Rex, s: *const c_char, c: c_int) -> *mut c_char {
    unsafe {
        if !rex.reg_ic() {
            return vim_strchr(s, c);
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
            return vim_strchr(s, c);
        }

        let mut p = s;
        while *p != 0 {
            let uc = utf_ptr2char(p);
            if c > 0x80 || uc > 0x80 {
                // Skip the ASCII case: a multibyte fold must not match a
                // byte that already compared equal below.
                if (uc < 0x80 || uc != *p as u8 as c_int) && utf_fold(uc) == lc {
                    return p as *mut c_char;
                }
            } else if *p as u8 as c_int == c || *p as u8 as c_int == cc {
                return p as *mut c_char;
            }
            p = p.add(utfc_ptr2len(p) as usize);
        }
        core::ptr::null_mut()
    }
}
