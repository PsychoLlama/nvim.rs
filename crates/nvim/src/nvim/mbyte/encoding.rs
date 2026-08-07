//! Encoding names: canonicalising them, and finding the locale's.
//!
//! `enc_canonize` reduces a user-written `'encoding'`/`'fileencoding'` value to
//! one of the names in the canonical table, resolving the alias table on the way,
//! and `enc_canon_props` reports what that name *is* (8-bit, DBCS, Unicode, and
//! which byte order).  `enc_locale` asks the C library what the user's locale
//! encodes in, for the default.  `bomb_size`/`remove_bom` are the byte-order-mark
//! half of the same question.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

// The carve of the transpiled module; see each child's docs.
mod tables;

pub use self::tables::*;

/// `<ctype.h>`'s class bits, as the rest of the tree already spells them.
pub const _ISalnum: ::core::ffi::c_uint = 8;

/// What `enc_canon_props()` reports about an encoding name.
///
/// `c_int`, which is the type of `enc_canon_table`'s `prop` field and of
/// every value compared against it; c2rust typed the anonymous enum from
/// what the C compiler picked and cast at all 130 use sites.
pub type EncProps = ::core::ffi::c_int;

pub const ENC_MACROMAN: EncProps = 2048;

pub const ENC_LATIN9: EncProps = 1024;

pub const ENC_LATIN1: EncProps = 512;

pub const ENC_2WORD: EncProps = 256;

pub const ENC_4BYTE: EncProps = 128;

pub const ENC_2BYTE: EncProps = 64;

pub const ENC_ENDIAN_L: EncProps = 32;

pub const ENC_ENDIAN_B: EncProps = 16;

pub const ENC_UNICODE: EncProps = 4;

pub const ENC_DBCS: EncProps = 2;

pub const ENC_8BIT: EncProps = 1;

pub const CODESET: nl_item = 14;

pub type nl_item = ::core::ffi::c_int;

unsafe extern "C" fn enc_canon_search(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < IDX_COUNT {
            if strcmp(name, (*enc_canon_table.ptr())[i as usize].name) == 0 as ::core::ffi::c_int {
                return i;
            }
            i += 1;
        }
        return -1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn enc_canon_props(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = enc_canon_search(name);
        if i >= 0 as ::core::ffi::c_int {
            return (*enc_canon_table.ptr())[i as usize].prop;
        } else if strncmp(
            name,
            b"2byte-\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            return ENC_DBCS;
        } else if strncmp(
            name,
            b"8bit-\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                name,
                b"iso-8859-\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            return ENC_8BIT;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn bomb_size() -> ::core::ffi::c_int {
    unsafe {
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*curbuf.get()).b_p_bomb != 0 && (*curbuf.get()).b_p_bin == 0 {
            if *(*curbuf.get()).b_p_fenc as ::core::ffi::c_int == NUL
                || strcmp(
                    (*curbuf.get()).b_p_fenc,
                    b"utf-8\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                n = 3 as ::core::ffi::c_int;
            } else if strncmp(
                (*curbuf.get()).b_p_fenc,
                b"ucs-2\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
                || strncmp(
                    (*curbuf.get()).b_p_fenc,
                    b"utf-16\0".as_ptr() as *const ::core::ffi::c_char,
                    6 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                n = 2 as ::core::ffi::c_int;
            } else if strncmp(
                (*curbuf.get()).b_p_fenc,
                b"ucs-4\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                n = 4 as ::core::ffi::c_int;
            }
        }
        return n;
    }
}

pub unsafe extern "C" fn remove_bom(mut s: *mut ::core::ffi::c_char) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = s;
        loop {
            p = strchr(p, 0xef as ::core::ffi::c_int);
            if p.is_null() {
                break;
            }
            if *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                == 0xbb as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                    == 0xbf as ::core::ffi::c_int
            {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(3 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(p.offset(3 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            } else {
                p = p.offset(1);
            }
        }
    }
}

pub unsafe extern "C" fn enc_skip(mut p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unsafe {
        if strncmp(
            p,
            b"2byte-\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            return p.offset(6 as ::core::ffi::c_int as isize);
        }
        if strncmp(
            p,
            b"8bit-\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            return p.offset(5 as ::core::ffi::c_int as isize);
        }
        return p;
    }
}

pub unsafe extern "C" fn enc_canonize(
    mut enc: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if strcmp(enc, b"default\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            return xstrdup(fenc_default.get());
        }
        let mut r: *mut ::core::ffi::c_char =
            xmalloc(strlen(enc).wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
        let mut p: *mut ::core::ffi::c_char = r;
        let mut s: *mut ::core::ffi::c_char = enc;
        while *s as ::core::ffi::c_int != NUL {
            if *s as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
                let c2rust_fresh15 = p;
                p = p.offset(1);
                *c2rust_fresh15 = '-' as ::core::ffi::c_char;
            } else {
                let c2rust_fresh16 = p;
                p = p.offset(1);
                *c2rust_fresh16 = (if (*s as ::core::ffi::c_int) < 'A' as ::core::ffi::c_int
                    || *s as ::core::ffi::c_int > 'Z' as ::core::ffi::c_int
                {
                    *s as ::core::ffi::c_int
                } else {
                    *s as ::core::ffi::c_int
                        + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) as ::core::ffi::c_char;
            }
            s = s.offset(1);
        }
        *p = NUL as ::core::ffi::c_char;
        p = enc_skip(r);
        if strncmp(
            p,
            b"microsoft-cp\0".as_ptr() as *const ::core::ffi::c_char,
            12 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(10 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(10 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
        }
        if strncmp(
            p,
            b"iso8859\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            memmove(
                p.offset(4 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                p.offset(3 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(3 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
            *p.offset(3 as ::core::ffi::c_int as isize) = '-' as ::core::ffi::c_char;
        }
        if strncmp(
            p,
            b"iso-8859\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
            && *p.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '-' as ::core::ffi::c_int
        {
            memmove(
                p.offset(9 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                p.offset(8 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(8 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
            *p.offset(8 as ::core::ffi::c_int as isize) = '-' as ::core::ffi::c_char;
        }
        if strncmp(
            p,
            b"latin-\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            memmove(
                p.offset(5 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                p.offset(6 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(6 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
        }
        let mut i: ::core::ffi::c_int = 0;
        if enc_canon_search(p) >= 0 as ::core::ffi::c_int {
            if p != r {
                memmove(
                    r as *mut ::core::ffi::c_void,
                    p as *const ::core::ffi::c_void,
                    strlen(p).wrapping_add(1 as size_t),
                );
            }
        } else {
            i = enc_alias_search(p);
            if i >= 0 as ::core::ffi::c_int {
                xfree(r as *mut ::core::ffi::c_void);
                r = xstrdup((*enc_canon_table.ptr())[i as usize].name);
            }
        }
        return r;
    }
}

unsafe extern "C" fn enc_alias_search(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !(*enc_alias_table.ptr())[i as usize].name.is_null() {
            if strcmp(name, (*enc_alias_table.ptr())[i as usize].name) == 0 as ::core::ffi::c_int {
                return (*enc_alias_table.ptr())[i as usize].canon;
            }
            i += 1;
        }
        return -1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn enc_locale() -> *mut ::core::ffi::c_char {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        let mut buf: [::core::ffi::c_char; 50] = [0; 50];
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        s = nl_langinfo(CODESET);
        if s.is_null() || *s as ::core::ffi::c_int == NUL {
            s = setlocale(LC_CTYPE, ::core::ptr::null::<::core::ffi::c_char>());
            if s.is_null() || *s as ::core::ffi::c_int == NUL {
                s = os_getenv_noalloc(b"LC_ALL\0".as_ptr() as *const ::core::ffi::c_char);
                if !s.is_null() {
                    s = os_getenv_noalloc(b"LC_CTYPE\0".as_ptr() as *const ::core::ffi::c_char);
                    if !s.is_null() {
                        s = os_getenv_noalloc(b"LANG\0".as_ptr() as *const ::core::ffi::c_char);
                    }
                }
            }
        }
        if s.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut p: *const ::core::ffi::c_char = vim_strchr(s, '.' as ::core::ffi::c_int);
        's_140: {
            if !p.is_null() {
                if p > s.offset(2 as ::core::ffi::c_int as isize)
                    && strncasecmp(
                        p.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
                        b"EUC\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                        3 as ::core::ffi::c_int as size_t,
                    ) == 0
                    && *(*__ctype_b_loc())
                        .offset(*p.offset(4 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISalnum as ::core::ffi::c_int
                        == 0
                    && *p.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '-' as ::core::ffi::c_int
                    && *p.offset(-3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '_' as ::core::ffi::c_int
                {
                    memmove(
                        &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        b"euc-\0".as_ptr() as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        4 as size_t,
                    );
                    buf[4 as ::core::ffi::c_int as usize] = (if *p
                        .offset(-2 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint
                        || ascii_isdigit(
                            *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        if (*p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                            < 'A' as ::core::ffi::c_int
                            || *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                > 'Z' as ::core::ffi::c_int
                        {
                            *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        } else {
                            *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        }
                    } else {
                        0 as ::core::ffi::c_int
                    })
                        as ::core::ffi::c_char;
                    buf[5 as ::core::ffi::c_int as usize] = (if *p
                        .offset(-1 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint
                        || ascii_isdigit(
                            *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        if (*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                            < 'A' as ::core::ffi::c_int
                            || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                > 'Z' as ::core::ffi::c_int
                        {
                            *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        } else {
                            *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        }
                    } else {
                        0 as ::core::ffi::c_int
                    })
                        as ::core::ffi::c_char;
                    buf[6 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                    break 's_140;
                } else {
                    s = p.offset(1 as ::core::ffi::c_int as isize);
                }
            }
            i = 0 as ::core::ffi::c_int;
            while i < ::core::mem::size_of::<[::core::ffi::c_char; 50]>() as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                && *s.offset(i as isize) as ::core::ffi::c_int != NUL
            {
                if *s.offset(i as isize) as ::core::ffi::c_int == '_' as ::core::ffi::c_int
                    || *s.offset(i as isize) as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                {
                    buf[i as usize] = '-' as ::core::ffi::c_char;
                } else {
                    if !(*s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *s.offset(i as isize) as uint8_t as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint
                        || ascii_isdigit(*s.offset(i as isize) as uint8_t as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0)
                    {
                        break;
                    }
                    buf[i as usize] = (if (*s.offset(i as isize) as ::core::ffi::c_int)
                        < 'A' as ::core::ffi::c_int
                        || *s.offset(i as isize) as ::core::ffi::c_int > 'Z' as ::core::ffi::c_int
                    {
                        *s.offset(i as isize) as ::core::ffi::c_int
                    } else {
                        *s.offset(i as isize) as ::core::ffi::c_int
                            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    }) as ::core::ffi::c_char;
                }
                i += 1;
            }
            buf[i as usize] = NUL as ::core::ffi::c_char;
        }
        return enc_canonize(&raw mut buf as *mut ::core::ffi::c_char);
    }
}

pub unsafe extern "C" fn get_encoding_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if idx
            >= ::core::mem::size_of::<[C2Rust_Unnamed_21; 59]>()
                .wrapping_div(::core::mem::size_of::<C2Rust_Unnamed_21>())
                .wrapping_div(
                    (::core::mem::size_of::<[C2Rust_Unnamed_21; 59]>()
                        .wrapping_rem(::core::mem::size_of::<C2Rust_Unnamed_21>())
                        == 0) as ::core::ffi::c_int as usize,
                ) as ::core::ffi::c_int
        {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return (*enc_canon_table.ptr())[idx as usize].name as *mut ::core::ffi::c_char;
    }
}
