//! Regexp pattern scanning utilities for Neovim.
//!
//! Provides `rs_skip_regexp` and `rs_skip_regexp_ex` — stateless helpers that
//! skip past a regexp pattern to its closing delimiter, handling magic modes,
//! `[...]` character class ranges, and multibyte characters.

#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_char, c_int};

extern "C" {
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn nvim_regexp_get_char_class(pp: *mut *mut c_char) -> c_int;
    fn nvim_regexp_get_equi_class(pp: *mut *mut c_char) -> c_int;
    fn nvim_regexp_get_coll_element(pp: *mut *mut c_char) -> c_int;
    fn nvim_get_p_cpo() -> *const c_char;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
}

// Characters always special inside [] ranges
const REGEXP_INRANGE: &[u8] = b"]^-n\\";
// Abbreviation characters after '\'
const REGEXP_ABBR: &[u8] = b"nrtebdoxuU";
// CPO_LITERAL flag character
const CPO_LITERAL: c_int = b'l' as c_int;
// CLASS_NONE return value from get_char_class
const CLASS_NONE: c_int = 99;

// Magic modes (matching regexp_defs.h)
#[allow(dead_code)]
const MAGIC_NONE: c_int = 1;
const MAGIC_OFF: c_int = 2;
const MAGIC_ON: c_int = 3;
const MAGIC_ALL: c_int = 4;

/// Skip over a "[]" range. `p` must point to the character after the '['.
/// The returned pointer is on the matching ']', or the terminating NUL.
///
/// Mirrors `skip_anyof` in `regexp.c`.
unsafe fn skip_anyof(mut p: *mut c_char, reg_cpo_lit: bool) -> *mut c_char {
    if *p == b'^' as c_char {
        p = p.add(1);
    }
    if *p == b']' as c_char || *p == b'-' as c_char {
        p = p.add(1);
    }
    while *p != 0 && *p != b']' as c_char {
        let l = utfc_ptr2len(p);
        if l > 1 {
            p = p.add(l as usize);
        } else if *p == b'-' as c_char {
            p = p.add(1);
            if *p != b']' as c_char && *p != 0 {
                // MB_PTR_ADV
                p = p.add(utfc_ptr2len(p) as usize);
            }
        } else if *p == b'\\' as c_char
            && (REGEXP_INRANGE.contains(&(*p.add(1) as u8))
                || (!reg_cpo_lit && REGEXP_ABBR.contains(&(*p.add(1) as u8))))
        {
            p = p.add(2);
        } else if *p == b'[' as c_char {
            if nvim_regexp_get_char_class(&mut p) == CLASS_NONE
                && nvim_regexp_get_equi_class(&mut p) == 0
                && nvim_regexp_get_coll_element(&mut p) == 0
                && *p != 0
            {
                p = p.add(1);
            }
        } else {
            p = p.add(1);
        }
    }
    p
}

/// Skip past regular expression, extended version.
///
/// Stop at end of `startp` or where `dirc` delimiter is found.
/// Handles backslash escapes, `[...]` ranges, `\?` replacement, and
/// `\v`/`\V` magic mode switches.
///
/// When `newp` is not NULL and `dirc` is '?', makes an allocated copy of the
/// expression and changes `\?` to `?`. If `*newp` is not NULL the expression
/// is changed in-place.
/// If a `\?` is changed to `?` then `dropped` is incremented, unless NULL.
/// If `magic_val` is not NULL, returns the effective magicness of the pattern.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_regexp_ex(
    startp: *mut c_char,
    dirc: c_int,
    magic: c_int,
    newp: *mut *mut c_char,
    dropped: *mut c_int,
    magic_val: *mut c_int,
) -> *mut c_char {
    let mut mymagic: c_int = if magic != 0 { MAGIC_ON } else { MAGIC_OFF };
    let reg_cpo_lit = !vim_strchr(nvim_get_p_cpo(), CPO_LITERAL).is_null();

    let mut p = startp;
    let mut startp = startp;
    let mut startplen: usize = 0;

    while *p != 0 {
        if c_int::from(*p) == dirc {
            break;
        }
        if (*p == b'[' as c_char && mymagic >= MAGIC_ON)
            || (*p == b'\\' as c_char && *p.add(1) == b'[' as c_char && mymagic <= MAGIC_OFF)
        {
            p = skip_anyof(p.add(1), reg_cpo_lit);
            if *p == 0 {
                break;
            }
        } else if *p == b'\\' as c_char && *p.add(1) != 0 {
            if dirc == b'?' as c_int && !newp.is_null() && *p.add(1) == b'?' as c_char {
                // change "\?" to "?", make a copy first.
                if startplen == 0 {
                    startplen = libc_strlen(startp);
                }
                if (*newp).is_null() {
                    *newp = xstrnsave(startp, startplen);
                    p = (*newp).add(p.offset_from(startp) as usize);
                    startp = *newp;
                }
                if !dropped.is_null() {
                    *dropped += 1;
                }
                std::ptr::copy(
                    p.add(1),
                    p,
                    startplen - (p.add(1).offset_from(startp) as usize) + 1,
                );
            } else {
                p = p.add(1); // skip next character
            }
            if *p == b'v' as c_char {
                mymagic = MAGIC_ALL;
            } else if *p == b'V' as c_char {
                mymagic = MAGIC_NONE;
            }
        }
        // MB_PTR_ADV
        p = p.add(utfc_ptr2len(p) as usize);
    }
    if !magic_val.is_null() {
        *magic_val = mymagic;
    }
    p
}

/// Skip past regular expression.
///
/// Stop at end of `startp` or where `dirc` is found ('/', '?', etc).
/// Take care of characters with a backslash in front of it.
/// Skip strings inside [ and ].
#[no_mangle]
pub unsafe extern "C" fn rs_skip_regexp(
    startp: *mut c_char,
    dirc: c_int,
    magic: c_int,
) -> *mut c_char {
    rs_skip_regexp_ex(
        startp,
        dirc,
        magic,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    )
}

/// Simple strlen implementation to avoid depending on libc crate.
#[allow(clippy::missing_const_for_fn)]
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

#[cfg(test)]
mod tests {
    // Tests require C FFI functions, so they run via compare_regexp.c
    // and the `just rust-ffi-test` target.
}
