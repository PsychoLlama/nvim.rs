//! Case folding and case-insensitive comparison.
//!
//! `utf_fold` is the case-*folding* used for `==?` and `'ignorecase'` matching --
//! not the same as lowercasing, which is `mb_tolower`.  `utf_strnicmp` is the
//! comparison built on the fold; it has to decode both sides because the folded
//! forms of two characters can differ in byte length.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_fold(mut a: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if a < 0x80 as ::core::ffi::c_int {
        return if a >= 0x41 as ::core::ffi::c_int && a <= 0x5a as ::core::ffi::c_int {
            a + 32 as ::core::ffi::c_int
        } else {
            a
        };
    }
    if a == 0xdf as ::core::ffi::c_int || a == 0x130 as ::core::ffi::c_int {
        return a;
    }
    let mut result: [utf8proc_int32_t; 1] = [0; 1];
    let res = utf8proc_decompose_char(a as utf8proc_int32_t, &mut result, UTF8PROC_CASEFOLD, None);
    return if res == 1 {
        result[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
    } else {
        a
    };
}

pub unsafe extern "C" fn mb_toupper(mut a: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if a < 128 as ::core::ffi::c_int
            && cmp_flags.get() & kOptCmpFlagKeepascii as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
        {
            return if a < 'a' as ::core::ffi::c_int || a > 'z' as ::core::ffi::c_int {
                a
            } else {
                a - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            };
        }
        if cmp_flags.get() & kOptCmpFlagInternal as ::core::ffi::c_int as ::core::ffi::c_uint == 0 {
            return towupper(a as wint_t) as ::core::ffi::c_int;
        }
        if a < 128 as ::core::ffi::c_int {
            return toupper(a);
        }
        return utf8proc_toupper(a as utf8proc_int32_t) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn mb_islower(mut a: ::core::ffi::c_int) -> bool {
    unsafe {
        return mb_toupper(a) != a;
    }
}

pub unsafe extern "C" fn mb_tolower(mut a: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if a < 128 as ::core::ffi::c_int
            && cmp_flags.get() & kOptCmpFlagKeepascii as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
        {
            return if a < 'A' as ::core::ffi::c_int || a > 'Z' as ::core::ffi::c_int {
                a
            } else {
                a + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            };
        }
        if cmp_flags.get() & kOptCmpFlagInternal as ::core::ffi::c_int as ::core::ffi::c_uint == 0 {
            return towlower(a as wint_t) as ::core::ffi::c_int;
        }
        if a < 128 as ::core::ffi::c_int {
            return tolower(a);
        }
        return utf8proc_tolower(a as utf8proc_int32_t) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn mb_isupper(mut a: ::core::ffi::c_int) -> bool {
    unsafe {
        return mb_tolower(a) != a;
    }
}

pub unsafe extern "C" fn mb_isalpha(mut a: ::core::ffi::c_int) -> bool {
    unsafe {
        return mb_islower(a) as ::core::ffi::c_int != 0
            || mb_isupper(a) as ::core::ffi::c_int != 0;
    }
}

pub unsafe extern "C" fn utf_strnicmp(
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
    mut n1: size_t,
    mut n2: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c1: ::core::ffi::c_int = 0;
        let mut c2: ::core::ffi::c_int = 0;
        let mut buffer: [::core::ffi::c_char; 6] = [0; 6];
        loop {
            c1 = utf_safe_read_char_adv(&raw mut s1, &raw mut n1);
            c2 = utf_safe_read_char_adv(&raw mut s2, &raw mut n2);
            if c1 <= 0 as ::core::ffi::c_int || c2 <= 0 as ::core::ffi::c_int {
                break;
            }
            if c1 == c2 {
                continue;
            }
            let mut cdiff: ::core::ffi::c_int = utf_fold(c1) - utf_fold(c2);
            if cdiff != 0 as ::core::ffi::c_int {
                return cdiff;
            }
        }
        if c1 == 0 as ::core::ffi::c_int || c2 == 0 as ::core::ffi::c_int {
            if c1 == 0 as ::core::ffi::c_int && c2 == 0 as ::core::ffi::c_int {
                return 0 as ::core::ffi::c_int;
            }
            return if c1 == 0 as ::core::ffi::c_int {
                -1 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            };
        }
        if c1 != -1 as ::core::ffi::c_int && c2 == -1 as ::core::ffi::c_int {
            n1 =
                utf_char2bytes(utf_fold(c1), &raw mut buffer as *mut ::core::ffi::c_char) as size_t;
            s1 = &raw mut buffer as *mut ::core::ffi::c_char;
        } else if c2 != -1 as ::core::ffi::c_int && c1 == -1 as ::core::ffi::c_int {
            n2 =
                utf_char2bytes(utf_fold(c2), &raw mut buffer as *mut ::core::ffi::c_char) as size_t;
            s2 = &raw mut buffer as *mut ::core::ffi::c_char;
        }
        while n1 > 0 as size_t
            && n2 > 0 as size_t
            && *s1 as ::core::ffi::c_int != NUL
            && *s2 as ::core::ffi::c_int != NUL
        {
            let mut cdiff_0: ::core::ffi::c_int =
                *s1 as uint8_t as ::core::ffi::c_int - *s2 as uint8_t as ::core::ffi::c_int;
            if cdiff_0 != 0 as ::core::ffi::c_int {
                return cdiff_0;
            }
            s1 = s1.offset(1);
            s2 = s2.offset(1);
            n1 = n1.wrapping_sub(1);
            n2 = n2.wrapping_sub(1);
        }
        if n1 > 0 as size_t && *s1 as ::core::ffi::c_int == NUL {
            n1 = 0 as size_t;
        }
        if n2 > 0 as size_t && *s2 as ::core::ffi::c_int == NUL {
            n2 = 0 as size_t;
        }
        if n1 == 0 as size_t && n2 == 0 as size_t {
            return 0 as ::core::ffi::c_int;
        }
        return if n1 == 0 as size_t {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
}

pub unsafe extern "C" fn mb_strnicmp(
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
    nn: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        return utf_strnicmp(s1, s2, nn, nn);
    }
}

pub unsafe extern "C" fn mb_stricmp(
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return mb_strnicmp(s1, s2, MAXCOL as ::core::ffi::c_int as size_t);
    }
}

pub unsafe extern "C" fn mb_strcmp_ic(
    mut ic: bool,
    mut s1: *const ::core::ffi::c_char,
    mut s2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return if ic as ::core::ffi::c_int != 0 {
            mb_stricmp(s1, s2)
        } else {
            strcmp(s1, s2)
        };
    }
}
