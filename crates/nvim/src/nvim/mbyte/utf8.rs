//! The UTF-8 codec: bytes to a codepoint and back.
//!
//! `utf_ptr2char` decodes; `utf_char2bytes` encodes; `utf_ptr2len` and the
//! `utfc_*` spellings answer how many bytes the character at a pointer occupies,
//! the `c` ones counting the composing characters that follow it as part of the
//! same character.  `utf_ptr2CharInfo_impl` is the decode the header's inlined
//! `utf_ptr2CharInfo` calls once the first byte says the character is multibyte;
//! it is one of the two symbols `unit-fixtures.so` compiles against.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

// The carve of the transpiled module; see each child's docs.
mod tables;

pub use self::tables::*;

static corrections: GlobalCell<[uint32_t; 7]> = GlobalCell::new([0; 7]);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2CharInfo_impl(mut p: *const uint8_t, len: uintptr_t) -> int32_t {
    unsafe {
        let corr: uint32_t = (*corrections.ptr())[len as usize];
        let mut cur: uint8_t = 0;
        cur = *p.offset(1 as ::core::ffi::c_int as isize);
        let mut code_point: uint32_t = ((*p.offset(0 as ::core::ffi::c_int as isize) as uint32_t)
            << 6 as ::core::ffi::c_int)
            .wrapping_add(cur as uint32_t);
        if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
            as ::core::ffi::c_uint
            != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int as ::core::ffi::c_long
            != 0
        {
            return -1 as int32_t;
        }
        if (len as uint32_t) >= 3 as uint32_t {
            cur = *p.offset(2 as ::core::ffi::c_int as isize);
            code_point = (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
            if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
                as ::core::ffi::c_uint
                != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                as ::core::ffi::c_long
                != 0
            {
                return -1 as int32_t;
            }
            if len as uint32_t != 3 as uint32_t {
                cur = *p.offset(3 as ::core::ffi::c_int as isize);
                code_point = (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
                if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
                    as ::core::ffi::c_uint
                    != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                    as ::core::ffi::c_long
                    != 0
                {
                    return -1 as int32_t;
                }
                if len as uint32_t != 4 as uint32_t {
                    cur = *p.offset(4 as ::core::ffi::c_int as isize);
                    code_point =
                        (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
                    if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
                        as ::core::ffi::c_uint
                        != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                        as ::core::ffi::c_long
                        != 0
                    {
                        return -1 as int32_t;
                    }
                    if len as uint32_t != 5 as uint32_t {
                        cur = *p.offset(5 as ::core::ffi::c_int as isize);
                        code_point =
                            (code_point << 6 as ::core::ffi::c_int).wrapping_add(cur as uint32_t);
                        if ((cur as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
                            as ::core::ffi::c_uint
                            != 0x80 as ::core::ffi::c_uint)
                            as ::core::ffi::c_int as ::core::ffi::c_long
                            != 0
                        {
                            return -1 as int32_t;
                        }
                    }
                }
            }
        }
        return code_point.wrapping_add(corr) as int32_t;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut uint8_t = p_in as *mut uint8_t;
        let v0: uint32_t = *p.offset(0 as ::core::ffi::c_int as isize) as uint32_t;
        if (v0 < 0x80 as uint32_t) as ::core::ffi::c_int as ::core::ffi::c_long != 0 {
            return v0 as ::core::ffi::c_int;
        }
        let len: uint8_t = utf8len_tab[v0 as usize];
        if ((len as ::core::ffi::c_int) < 2 as ::core::ffi::c_int) as ::core::ffi::c_int
            as ::core::ffi::c_long
            != 0
        {
            return v0 as ::core::ffi::c_int;
        }
        let v1: uint32_t = *p.offset(1 as ::core::ffi::c_int as isize) as uint32_t;
        if ((v1 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint
            != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int as ::core::ffi::c_long
            != 0
        {
            return v0 as ::core::ffi::c_int;
        }
        if len as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            return (v0 << 6 as ::core::ffi::c_int)
                .wrapping_add(v1)
                .wrapping_sub(
                    ((0xc0 as uint32_t) << 6 as ::core::ffi::c_int).wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                    ),
                ) as ::core::ffi::c_int;
        }
        let v2: uint32_t = *p.offset(2 as ::core::ffi::c_int as isize) as uint32_t;
        if ((v2 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint
            != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int as ::core::ffi::c_long
            != 0
        {
            return v0 as ::core::ffi::c_int;
        }
        if len as ::core::ffi::c_int == 3 as ::core::ffi::c_int {
            return (v0 << 12 as ::core::ffi::c_int)
                .wrapping_add(v1 << 6 as ::core::ffi::c_int)
                .wrapping_add(v2)
                .wrapping_sub(
                    ((0xe0 as uint32_t) << 12 as ::core::ffi::c_int)
                        .wrapping_add(
                            (0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int,
                        )
                        .wrapping_add(
                            (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                        ),
                ) as ::core::ffi::c_int;
        }
        let v3: uint32_t = *p.offset(3 as ::core::ffi::c_int as isize) as uint32_t;
        if ((v3 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint
            != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int as ::core::ffi::c_long
            != 0
        {
            return v0 as ::core::ffi::c_int;
        }
        if len as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
            return (v0 << 18 as ::core::ffi::c_int)
                .wrapping_add(v1 << 12 as ::core::ffi::c_int)
                .wrapping_add(v2 << 6 as ::core::ffi::c_int)
                .wrapping_add(v3)
                .wrapping_sub(
                    ((0xf0 as uint32_t) << 18 as ::core::ffi::c_int)
                        .wrapping_add(
                            (0x80 as ::core::ffi::c_uint as uint32_t) << 12 as ::core::ffi::c_int,
                        )
                        .wrapping_add(
                            (0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int,
                        )
                        .wrapping_add(
                            (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                        ),
                ) as ::core::ffi::c_int;
        }
        let v4: uint32_t = *p.offset(4 as ::core::ffi::c_int as isize) as uint32_t;
        if ((v4 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint
            != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int as ::core::ffi::c_long
            != 0
        {
            return v0 as ::core::ffi::c_int;
        }
        if len as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
            return (v0 << 24 as ::core::ffi::c_int)
                .wrapping_add(v1 << 18 as ::core::ffi::c_int)
                .wrapping_add(v2 << 12 as ::core::ffi::c_int)
                .wrapping_add(v3 << 6 as ::core::ffi::c_int)
                .wrapping_add(v4)
                .wrapping_sub(
                    ((0xf8 as uint32_t) << 24 as ::core::ffi::c_int)
                        .wrapping_add(
                            (0x80 as ::core::ffi::c_uint as uint32_t) << 18 as ::core::ffi::c_int,
                        )
                        .wrapping_add(
                            (0x80 as ::core::ffi::c_uint as uint32_t) << 12 as ::core::ffi::c_int,
                        )
                        .wrapping_add(
                            (0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int,
                        )
                        .wrapping_add(
                            (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                        ),
                ) as ::core::ffi::c_int;
        }
        let v5: uint32_t = *p.offset(5 as ::core::ffi::c_int as isize) as uint32_t;
        if ((v5 & 0xc0 as uint32_t) as uint8_t as ::core::ffi::c_uint
            != 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int as ::core::ffi::c_long
            != 0
        {
            return v0 as ::core::ffi::c_int;
        }
        return (v0 << 30 as ::core::ffi::c_int)
            .wrapping_add(v1 << 24 as ::core::ffi::c_int)
            .wrapping_add(v2 << 18 as ::core::ffi::c_int)
            .wrapping_add(v3 << 12 as ::core::ffi::c_int)
            .wrapping_add(v4 << 6 as ::core::ffi::c_int)
            .wrapping_add(v5)
            .wrapping_sub(
                ((0x80 as ::core::ffi::c_uint as uint32_t) << 24 as ::core::ffi::c_int)
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 18 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 12 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 6 as ::core::ffi::c_int,
                    )
                    .wrapping_add(
                        (0x80 as ::core::ffi::c_uint as uint32_t) << 0 as ::core::ffi::c_int,
                    ),
            ) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn utf_safe_read_char_adv(
    mut s: *mut *const ::core::ffi::c_char,
    mut n: *mut size_t,
) -> ::core::ffi::c_int {
    unsafe {
        if *n == 0 as size_t {
            return 0 as ::core::ffi::c_int;
        }
        let mut k: uint8_t = utf8len_tab_zero[**s as uint8_t as usize];
        if k as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            *n = (*n).wrapping_sub(1);
            let c2rust_fresh0 = *s;
            *s = (*s).offset(1);
            return *c2rust_fresh0 as uint8_t as ::core::ffi::c_int;
        }
        if k as size_t <= *n {
            let mut c: ::core::ffi::c_int = utf_ptr2char(*s);
            if c != **s as uint8_t as ::core::ffi::c_int
                || c == 0xc3 as ::core::ffi::c_int
                    && *(*s).offset(1 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int
                        == 0x83 as ::core::ffi::c_int
            {
                *s = (*s).offset(k as ::core::ffi::c_int as isize);
                *n = (*n).wrapping_sub(k as size_t);
                return c;
            }
        }
        return -1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn mb_ptr2char_adv(
    pp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = utf_ptr2char(*pp);
        *pp = (*pp).offset(utfc_ptr2len(*pp) as isize);
        return c;
    }
}

pub unsafe extern "C" fn mb_cptr2char_adv(
    mut pp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = utf_ptr2char(*pp);
        *pp = (*pp).offset(utf_ptr2len(*pp) as isize);
        return c;
    }
}

pub unsafe extern "C" fn utf_iscomposing_first(mut c: ::core::ffi::c_int) -> bool {
    return c >= 128 as ::core::ffi::c_int
        && !utf8proc_grapheme_break(' ' as utf8proc_int32_t, c as utf8proc_int32_t);
}

pub unsafe extern "C" fn utf_composinglike(
    mut p1: *const ::core::ffi::c_char,
    mut p2: *const ::core::ffi::c_char,
    mut state: *mut GraphemeState,
) -> bool {
    unsafe {
        if (*p2 as uint8_t as ::core::ffi::c_int) < 128 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        let mut first: ::core::ffi::c_int = utf_ptr2char(p1);
        let mut second: ::core::ffi::c_int = utf_ptr2char(p2);
        if !utf8proc_grapheme_break_stateful(
            first as utf8proc_int32_t,
            second as utf8proc_int32_t,
            state.as_mut(),
        ) {
            return true_0 != 0;
        }
        return crate::src::nvim::arabic::arabic_combine(first, second);
    }
}

pub unsafe extern "C" fn utf_iscomposing(
    mut c1: ::core::ffi::c_int,
    mut c2: ::core::ffi::c_int,
    mut state: *mut GraphemeState,
) -> bool {
    unsafe {
        return !utf8proc_grapheme_break_stateful(
            c1 as utf8proc_int32_t,
            c2 as utf8proc_int32_t,
            state.as_mut(),
        ) || crate::src::nvim::arabic::arabic_combine(c1, c2) as ::core::ffi::c_int != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utfc_ptr2schar(
    mut p: *const ::core::ffi::c_char,
    mut firstc: *mut ::core::ffi::c_int,
) -> schar_T {
    unsafe {
        let mut c: ::core::ffi::c_int = utf_ptr2char(p);
        *firstc = c;
        let mut first_compose: bool = utf_iscomposing_first(c);
        let mut maxlen: size_t = (MAX_SCHAR_SIZE
            - 1 as ::core::ffi::c_int
            - first_compose as ::core::ffi::c_int) as size_t;
        let mut len: size_t = utfc_ptr2len_len(p, maxlen as ::core::ffi::c_int) as size_t;
        if len == 1 as size_t && *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
            return 0 as schar_T;
        }
        return schar_from_buf_first(p, len, first_compose);
    }
}

pub unsafe extern "C" fn utfc_ptrlen2schar(
    mut p: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut firstc: *mut ::core::ffi::c_int,
) -> schar_T {
    unsafe {
        if len == 1 as ::core::ffi::c_int
            && *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
            || len == 0 as ::core::ffi::c_int
        {
            *firstc = *p as uint8_t as ::core::ffi::c_int;
            return 0 as schar_T;
        }
        let mut c: ::core::ffi::c_int = utf_ptr2char(p);
        *firstc = c;
        let mut first_compose: bool = utf_iscomposing_first(c);
        let mut maxlen: ::core::ffi::c_int =
            MAX_SCHAR_SIZE - 1 as ::core::ffi::c_int - first_compose as ::core::ffi::c_int;
        if len > maxlen {
            len = utfc_ptr2len_len(p, maxlen);
        }
        return schar_from_buf_first(p, len as size_t, first_compose);
    }
}

unsafe extern "C" fn schar_from_buf_first(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut first_compose: bool,
) -> schar_T {
    unsafe {
        if first_compose {
            let mut cbuf: [::core::ffi::c_char; 32] = [0; 32];
            cbuf[0 as ::core::ffi::c_int as usize] = ' ' as ::core::ffi::c_char;
            memcpy(
                (&raw mut cbuf as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                buf as *const ::core::ffi::c_void,
                len,
            );
            return schar_from_buf(
                &raw mut cbuf as *mut ::core::ffi::c_char,
                len.wrapping_add(1 as size_t),
            );
        } else {
            return schar_from_buf(buf, len);
        };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut uint8_t = p_in as *mut uint8_t;
        if *p as ::core::ffi::c_int == NUL {
            return 0 as ::core::ffi::c_int;
        }
        let len: ::core::ffi::c_int = utf8len_tab[*p as usize] as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i < len {
            if *p.offset(i as isize) as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
                != 0x80 as ::core::ffi::c_int
            {
                return 1 as ::core::ffi::c_int;
            }
            i += 1;
        }
        return len;
    }
}

pub unsafe extern "C" fn utf_byte2len(mut b: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return utf8len_tab[b as usize] as ::core::ffi::c_int;
}

pub unsafe extern "C" fn utf_ptr2len_len(
    mut p: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut m: ::core::ffi::c_int = 0;
        let mut len: ::core::ffi::c_int = utf8len_tab[*p as uint8_t as usize] as ::core::ffi::c_int;
        if len == 1 as ::core::ffi::c_int {
            return 1 as ::core::ffi::c_int;
        }
        if len > size {
            m = size;
        } else {
            m = len;
        }
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i < m {
            if *p.offset(i as isize) as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
                != 0x80 as ::core::ffi::c_int
            {
                return 1 as ::core::ffi::c_int;
            }
            i += 1;
        }
        return len;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut b0: uint8_t = *p as uint8_t;
        if b0 as ::core::ffi::c_int == NUL {
            return 0 as ::core::ffi::c_int;
        }
        if (b0 as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int
            && (*p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int)
                < 0x80 as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        let mut len: ::core::ffi::c_int = utf_ptr2len(p);
        if len == 1 as ::core::ffi::c_int && b0 as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        let mut prevlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
        loop {
            if (*p.offset(len as isize) as uint8_t as ::core::ffi::c_int)
                < 0x80 as ::core::ffi::c_int
                || !utf_composinglike(
                    p.offset(prevlen as isize),
                    p.offset(len as isize),
                    &raw mut state,
                )
            {
                return len;
            }
            prevlen = len;
            len += utf_ptr2len(p.offset(len as isize));
        }
    }
}

pub unsafe extern "C" fn utfc_ptr2len_len(
    mut p: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if size < 1 as ::core::ffi::c_int || *p as ::core::ffi::c_int == NUL {
            return 0 as ::core::ffi::c_int;
        }
        if (*p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int)
            < 0x80 as ::core::ffi::c_int
            && (size == 1 as ::core::ffi::c_int
                || (*p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int)
                    < 0x80 as ::core::ffi::c_int)
        {
            return 1 as ::core::ffi::c_int;
        }
        let mut len: ::core::ffi::c_int = utf_ptr2len_len(p, size);
        if len == 1 as ::core::ffi::c_int
            && *p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                >= 0x80 as ::core::ffi::c_int
            || len > size
        {
            return 1 as ::core::ffi::c_int;
        }
        let mut prevlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
        while len < size {
            if (*p.offset(len as isize) as uint8_t as ::core::ffi::c_int)
                < 0x80 as ::core::ffi::c_int
            {
                break;
            }
            let mut len_next_char: ::core::ffi::c_int =
                utf_ptr2len_len(p.offset(len as isize), size - len);
            if len_next_char > size - len {
                break;
            }
            if !utf_composinglike(
                p.offset(prevlen as isize),
                p.offset(len as isize),
                &raw mut state,
            ) {
                break;
            }
            prevlen = len;
            len += len_next_char;
        }
        return len;
    }
}

pub fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c < 0x80 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    } else if c < 0x800 as ::core::ffi::c_int {
        return 2 as ::core::ffi::c_int;
    } else if c < 0x10000 as ::core::ffi::c_int {
        return 3 as ::core::ffi::c_int;
    } else if c < 0x200000 as ::core::ffi::c_int {
        return 4 as ::core::ffi::c_int;
    } else if c < 0x4000000 as ::core::ffi::c_int {
        return 5 as ::core::ffi::c_int;
    } else {
        return 6 as ::core::ffi::c_int;
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_char2bytes(
    c: ::core::ffi::c_int,
    buf: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if c < 0x80 as ::core::ffi::c_int {
            *buf.offset(0 as ::core::ffi::c_int as isize) = c as ::core::ffi::c_char;
            return 1 as ::core::ffi::c_int;
        } else if c < 0x800 as ::core::ffi::c_int {
            *buf.offset(0 as ::core::ffi::c_int as isize) = (0xc0 as ::core::ffi::c_uint)
                .wrapping_add(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int)
                as ::core::ffi::c_char;
            *buf.offset(1 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
                .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
                as ::core::ffi::c_char;
            return 2 as ::core::ffi::c_int;
        } else if c < 0x10000 as ::core::ffi::c_int {
            *buf.offset(0 as ::core::ffi::c_int as isize) = (0xe0 as ::core::ffi::c_uint)
                .wrapping_add(c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int)
                as ::core::ffi::c_char;
            *buf.offset(1 as ::core::ffi::c_int as isize) =
                (0x80 as ::core::ffi::c_uint).wrapping_add(
                    c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int
                        & 0x3f as ::core::ffi::c_uint,
                ) as ::core::ffi::c_char;
            *buf.offset(2 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
                .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
                as ::core::ffi::c_char;
            return 3 as ::core::ffi::c_int;
        } else if c < 0x200000 as ::core::ffi::c_int {
            *buf.offset(0 as ::core::ffi::c_int as isize) = (0xf0 as ::core::ffi::c_uint)
                .wrapping_add(c as ::core::ffi::c_uint >> 18 as ::core::ffi::c_int)
                as ::core::ffi::c_char;
            *buf.offset(1 as ::core::ffi::c_int as isize) =
                (0x80 as ::core::ffi::c_uint).wrapping_add(
                    c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int
                        & 0x3f as ::core::ffi::c_uint,
                ) as ::core::ffi::c_char;
            *buf.offset(2 as ::core::ffi::c_int as isize) =
                (0x80 as ::core::ffi::c_uint).wrapping_add(
                    c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int
                        & 0x3f as ::core::ffi::c_uint,
                ) as ::core::ffi::c_char;
            *buf.offset(3 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
                .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
                as ::core::ffi::c_char;
            return 4 as ::core::ffi::c_int;
        } else if c < 0x4000000 as ::core::ffi::c_int {
            *buf.offset(0 as ::core::ffi::c_int as isize) = (0xf8 as ::core::ffi::c_uint)
                .wrapping_add(c as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int)
                as ::core::ffi::c_char;
            *buf.offset(1 as ::core::ffi::c_int as isize) =
                (0x80 as ::core::ffi::c_uint).wrapping_add(
                    c as ::core::ffi::c_uint >> 18 as ::core::ffi::c_int
                        & 0x3f as ::core::ffi::c_uint,
                ) as ::core::ffi::c_char;
            *buf.offset(2 as ::core::ffi::c_int as isize) =
                (0x80 as ::core::ffi::c_uint).wrapping_add(
                    c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int
                        & 0x3f as ::core::ffi::c_uint,
                ) as ::core::ffi::c_char;
            *buf.offset(3 as ::core::ffi::c_int as isize) =
                (0x80 as ::core::ffi::c_uint).wrapping_add(
                    c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int
                        & 0x3f as ::core::ffi::c_uint,
                ) as ::core::ffi::c_char;
            *buf.offset(4 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
                .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
                as ::core::ffi::c_char;
            return 5 as ::core::ffi::c_int;
        } else {
            *buf.offset(0 as ::core::ffi::c_int as isize) = (0xfc as ::core::ffi::c_uint)
                .wrapping_add(c as ::core::ffi::c_uint >> 30 as ::core::ffi::c_int)
                as ::core::ffi::c_char;
            *buf.offset(1 as ::core::ffi::c_int as isize) =
                (0x80 as ::core::ffi::c_uint).wrapping_add(
                    c as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int
                        & 0x3f as ::core::ffi::c_uint,
                ) as ::core::ffi::c_char;
            *buf.offset(2 as ::core::ffi::c_int as isize) =
                (0x80 as ::core::ffi::c_uint).wrapping_add(
                    c as ::core::ffi::c_uint >> 18 as ::core::ffi::c_int
                        & 0x3f as ::core::ffi::c_uint,
                ) as ::core::ffi::c_char;
            *buf.offset(3 as ::core::ffi::c_int as isize) =
                (0x80 as ::core::ffi::c_uint).wrapping_add(
                    c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int
                        & 0x3f as ::core::ffi::c_uint,
                ) as ::core::ffi::c_char;
            *buf.offset(4 as ::core::ffi::c_int as isize) =
                (0x80 as ::core::ffi::c_uint).wrapping_add(
                    c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int
                        & 0x3f as ::core::ffi::c_uint,
                ) as ::core::ffi::c_char;
            *buf.offset(5 as ::core::ffi::c_int as isize) = (0x80 as ::core::ffi::c_uint)
                .wrapping_add(c as ::core::ffi::c_uint & 0x3f as ::core::ffi::c_uint)
                as ::core::ffi::c_char;
            return 6 as ::core::ffi::c_int;
        };
    }
}

pub unsafe extern "C" fn utf_iscomposing_legacy(mut c: ::core::ffi::c_int) -> bool {
    unsafe {
        let mut prop: *const utf8proc_property_t = utf8proc_get_property(c as utf8proc_int32_t);
        return (*prop).category as ::core::ffi::c_int
            == UTF8PROC_CATEGORY_MN as ::core::ffi::c_int
            || (*prop).category as ::core::ffi::c_int
                == UTF8PROC_CATEGORY_ME as ::core::ffi::c_int;
    }
}

pub const GRAPHEME_STATE_INIT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

#[inline(always)]
pub(crate) fn utf_is_trail_byte(byte: uint8_t) -> bool {
    return (byte as ::core::ffi::c_uint & 0xc0 as ::core::ffi::c_uint) as uint8_t
        as ::core::ffi::c_uint
        == 0x80 as ::core::ffi::c_uint;
}

/// The codepoint at `p` and the number of bytes it occupies. An invalid
/// sequence reports its first byte negated, with a length of one.
///
/// # Safety
/// `p` must point into a NUL-terminated string.
#[inline(always)]
pub unsafe fn utf_ptr2CharInfo(p_in: *const ::core::ffi::c_char) -> CharInfo {
    unsafe {
        let p = p_in as *const uint8_t;
        let first = *p;
        if first < 0x80 {
            return CharInfo {
                value: first as int32_t,
                len: 1,
            };
        }
        let len = utf8len_tab[first as usize] as ::core::ffi::c_int;
        let code_point = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        CharInfo {
            value: code_point,
            len: if code_point < 0 { 1 } else { len },
        }
    }
}

unsafe extern "C" fn c2rust_run_static_initializers() {
    corrections.set([
        (1 as uint32_t) << 31 as ::core::ffi::c_int,
        (1 as uint32_t) << 31 as ::core::ffi::c_int,
        (0x80 as uint32_t)
            .wrapping_add((0xc0 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0xe0 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_add((0xf0 as uint32_t) << 18 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 18 as ::core::ffi::c_int)
            .wrapping_add((0xf8 as uint32_t) << 24 as ::core::ffi::c_int)
            .wrapping_neg(),
        (0x80 as uint32_t)
            .wrapping_add((0x80 as uint32_t) << 6 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 12 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 18 as ::core::ffi::c_int)
            .wrapping_add((0x80 as uint32_t) << 24 as ::core::ffi::c_int)
            .wrapping_neg(),
    ]);
}

#[used]
#[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
#[cfg_attr(target_os = "windows", unsafe(link_section = ".CRT$XIB"))]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
