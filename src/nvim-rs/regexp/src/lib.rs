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

// --- Magic helpers (matching regexp.c macros) ---
// Magic(x) = (int)(x) - 256; is_Magic(x) = (x) < 0; un_Magic(x) = (x) + 256

/// Multi-type return values
const NOT_MULTI: c_int = 0;
const MULTI_ONE: c_int = 1;
const MULTI_MULT: c_int = 2;

/// Control character constants (matching `ascii_defs.h`)
const BS_CH: c_int = 0o010;
const TAB_CH: c_int = 0o011;
const CAR_CH: c_int = 0o015;
const ESC_CH: c_int = 0o033;

/// Magic('x') = (x as i32) - 256
const fn magic(x: u8) -> c_int {
    (x as c_int) - 256
}

/// If x is Magic (negative), strip the magic to get the plain character.
/// Otherwise return x unchanged.
#[no_mangle]
pub const extern "C" fn rs_no_magic(x: c_int) -> c_int {
    if x < 0 {
        x + 256
    } else {
        x
    }
}

/// If x is Magic (negative), un-magic it. Otherwise make it Magic.
#[no_mangle]
pub const extern "C" fn rs_toggle_magic(x: c_int) -> c_int {
    if x < 0 {
        x + 256
    } else {
        x - 256
    }
}

/// Return `NOT_MULTI` if c is not a "multi" operator.
/// Return `MULTI_ONE` if c is a single "multi" operator.
/// Return `MULTI_MULT` if c is a multi "multi" operator.
#[no_mangle]
pub const extern "C" fn rs_re_multi_type(c: c_int) -> c_int {
    if c == magic(b'@') || c == magic(b'=') || c == magic(b'?') {
        MULTI_ONE
    } else if c == magic(b'*') || c == magic(b'+') || c == magic(b'{') {
        MULTI_MULT
    } else {
        NOT_MULTI
    }
}

/// Translate '\x' to its control character, except "\n", which is Magic.
#[no_mangle]
pub const extern "C" fn rs_backslash_trans(c: c_int) -> c_int {
    match c {
        0x72 => CAR_CH, // 'r'
        0x74 => TAB_CH, // 't'
        0x65 => ESC_CH, // 'e'
        0x62 => BS_CH,  // 'b'
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_magic_positive() {
        // Positive values pass through
        assert_eq!(rs_no_magic(0), 0);
        assert_eq!(rs_no_magic(65), 65); // 'A'
        assert_eq!(rs_no_magic(255), 255);
    }

    #[test]
    fn test_no_magic_negative() {
        // Negative (magic) values get un-magicked: x + 256
        assert_eq!(rs_no_magic(-1), 255);
        assert_eq!(rs_no_magic(-192), 64); // Magic('@') -> '@'
        assert_eq!(rs_no_magic(-256), 0);
    }

    #[test]
    fn test_no_magic_boundary() {
        // At the boundary
        assert_eq!(rs_no_magic(-1), 255);
        assert_eq!(rs_no_magic(0), 0);
    }

    #[test]
    fn test_toggle_magic_positive() {
        // Positive (non-magic) -> subtract 256
        assert_eq!(rs_toggle_magic(65), 65 - 256); // 'A' -> Magic('A')
        assert_eq!(rs_toggle_magic(0), -256);
    }

    #[test]
    fn test_toggle_magic_negative() {
        // Negative (magic) -> add 256
        assert_eq!(rs_toggle_magic(-192), 64); // Magic('@') -> '@'
        assert_eq!(rs_toggle_magic(-1), 255);
    }

    #[test]
    fn test_toggle_magic_roundtrip() {
        // toggle(toggle(x)) == x for values in the valid Magic range.
        // Magic chars are in [-256, 0) and plain chars in [0, 256).
        for x in -256..256 {
            assert_eq!(rs_toggle_magic(rs_toggle_magic(x)), x);
        }
    }

    #[test]
    fn test_re_multi_type_one() {
        assert_eq!(rs_re_multi_type(magic(b'@')), MULTI_ONE);
        assert_eq!(rs_re_multi_type(magic(b'=')), MULTI_ONE);
        assert_eq!(rs_re_multi_type(magic(b'?')), MULTI_ONE);
    }

    #[test]
    fn test_re_multi_type_mult() {
        assert_eq!(rs_re_multi_type(magic(b'*')), MULTI_MULT);
        assert_eq!(rs_re_multi_type(magic(b'+')), MULTI_MULT);
        assert_eq!(rs_re_multi_type(magic(b'{')), MULTI_MULT);
    }

    #[test]
    fn test_re_multi_type_not() {
        assert_eq!(rs_re_multi_type(0), NOT_MULTI);
        assert_eq!(rs_re_multi_type(65), NOT_MULTI); // 'A'
        assert_eq!(rs_re_multi_type(magic(b'a')), NOT_MULTI);
        assert_eq!(rs_re_multi_type(-1), NOT_MULTI);
    }

    #[test]
    fn test_backslash_trans_escapes() {
        assert_eq!(rs_backslash_trans(b'r' as c_int), CAR_CH);
        assert_eq!(rs_backslash_trans(b't' as c_int), TAB_CH);
        assert_eq!(rs_backslash_trans(b'e' as c_int), ESC_CH);
        assert_eq!(rs_backslash_trans(b'b' as c_int), BS_CH);
    }

    #[test]
    fn test_backslash_trans_passthrough() {
        assert_eq!(rs_backslash_trans(b'n' as c_int), b'n' as c_int);
        assert_eq!(rs_backslash_trans(b'a' as c_int), b'a' as c_int);
        assert_eq!(rs_backslash_trans(0), 0);
        assert_eq!(rs_backslash_trans(255), 255);
    }
}
