//! Comparing multibyte text: decomposition and the case- and
//! composing-aware string and character searches.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn mb_decompose(
    mut c: ::core::ffi::c_int,
    mut c1: *mut ::core::ffi::c_int,
    mut c2: *mut ::core::ffi::c_int,
    mut c3: *mut ::core::ffi::c_int,
) {
    let mut d: decomp_T = decomp_T { a: 0, b: 0, c: 0 };
    if c >= 0xfb20 as ::core::ffi::c_int && c <= 0xfb4f as ::core::ffi::c_int {
        d = (*decomp_table.ptr())[(c - 0xfb20 as ::core::ffi::c_int) as usize];
        *c1 = d.a;
        *c2 = d.b;
        *c3 = d.c;
    } else {
        *c1 = c;
        *c2 = 0 as ::core::ffi::c_int;
        *c3 = 0 as ::core::ffi::c_int;
    };
}
pub(crate) unsafe extern "C" fn cstrncmp(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
    mut n: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = 0;
    if !(*rex.ptr()).reg_ic {
        result = strncmp(s1, s2, *n as size_t);
    } else {
        let mut p: *mut ::core::ffi::c_char = s1;
        let mut n2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut n1: ::core::ffi::c_int = *n;
        while n1 > 0 as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL {
            n1 -= utfc_ptr2len(s1);
            p = p.offset(utfc_ptr2len(p) as isize);
            n2 += 1;
        }
        p = s2;
        loop {
            let c2rust_fresh12 = n2;
            n2 = n2 - 1;
            if !(c2rust_fresh12 > 0 as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL) {
                break;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        n2 = p.offset_from(s2) as ::core::ffi::c_int;
        result = utf_strnicmp(s1, s2, *n as size_t, n2 as size_t);
        if result == 0 as ::core::ffi::c_int && n2 < *n {
            *n = n2;
        }
    }
    if result != 0 as ::core::ffi::c_int && (*rex.ptr()).reg_icombine as ::core::ffi::c_int != 0 {
        let mut str1: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut str2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut c1: ::core::ffi::c_int = 0;
        let mut c2: ::core::ffi::c_int = 0;
        let mut c11: ::core::ffi::c_int = 0;
        let mut c12: ::core::ffi::c_int = 0;
        let mut junk: ::core::ffi::c_int = 0;
        str1 = s1;
        str2 = s2;
        c2 = 0 as ::core::ffi::c_int;
        c1 = c2;
        while (str1.offset_from(s1) as ::core::ffi::c_int) < *n {
            c1 = mb_ptr2char_adv(&raw mut str1);
            c2 = mb_ptr2char_adv(&raw mut str2);
            if !(c1 != c2 && (!(*rex.ptr()).reg_ic || utf_fold(c1) != utf_fold(c2))) {
                continue;
            }
            mb_decompose(c1, &raw mut c11, &raw mut junk, &raw mut junk);
            mb_decompose(c2, &raw mut c12, &raw mut junk, &raw mut junk);
            c1 = c11;
            c2 = c12;
            if c11 != c12 && (!(*rex.ptr()).reg_ic || utf_fold(c11) != utf_fold(c12)) {
                break;
            }
        }
        result = c2 - c1;
        if result == 0 as ::core::ffi::c_int {
            *n = str2.offset_from(s2) as ::core::ffi::c_int;
        }
    }
    return result;
}
#[inline(always)]
pub(crate) unsafe extern "C" fn cstrchr(
    s: *const ::core::ffi::c_char,
    c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if !(*rex.ptr()).reg_ic {
        return vim_strchr(s, c);
    }
    let mut cc: ::core::ffi::c_int = 0;
    let mut lc: ::core::ffi::c_int = 0;
    if c > 0x80 as ::core::ffi::c_int {
        cc = utf_fold(c);
        lc = cc;
    } else if c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        cc = if c < 'A' as ::core::ffi::c_int || c > 'Z' as ::core::ffi::c_int {
            c
        } else {
            c + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
        lc = cc;
    } else if c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        cc = if c < 'a' as ::core::ffi::c_int || c > 'z' as ::core::ffi::c_int {
            c
        } else {
            c - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        };
        lc = c;
    } else {
        return vim_strchr(s, c);
    }
    let mut p: *const ::core::ffi::c_char = s;
    while *p as ::core::ffi::c_int != NUL {
        let uc: ::core::ffi::c_int = utf_ptr2char(p);
        if c > 0x80 as ::core::ffi::c_int || uc > 0x80 as ::core::ffi::c_int {
            if (uc < 0x80 as ::core::ffi::c_int || uc != *p as uint8_t as ::core::ffi::c_int)
                && utf_fold(uc) == lc
            {
                return p as *mut ::core::ffi::c_char;
            }
        } else if *p as uint8_t as ::core::ffi::c_int == c
            || *p as uint8_t as ::core::ffi::c_int == cc
        {
            return p as *mut ::core::ffi::c_char;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub(crate) unsafe extern "C" fn do_upper(
    mut d: *mut ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) {
    *d = mb_toupper(c);
}
pub(crate) unsafe extern "C" fn do_lower(
    mut d: *mut ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) {
    *d = mb_tolower(c);
}
