//! Writing mappings back out: `:mkexrc`, `:mksession`.
//!
//! [`makemap`] walks the table emitting a `:map` command per entry, splitting
//! one mapblock into up to three commands when its mode set is not one a
//! single command name can express, and [`put_escstr`] writes an LHS or RHS
//! with whatever escaping makes it read back identically.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn makemap(mut fd: *mut FILE, mut buf: *mut buf_T) -> ::core::ffi::c_int {
    unsafe {
        let mut did_cpo: bool = false_0 != 0;
        let mut abbr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while abbr < 2 as ::core::ffi::c_int {
            let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while hash < 256 as ::core::ffi::c_int {
                let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
                if abbr != 0 {
                    if hash > 0 as ::core::ffi::c_int {
                        break;
                    }
                    if !buf.is_null() {
                        mp = (*buf).b_first_abbr;
                    } else {
                        mp = first_abbr.get();
                    }
                } else if !buf.is_null() {
                    mp = (*buf).b_maphash[hash as usize] as *mut mapblock_T;
                } else {
                    mp = (*maphash.ptr())[hash as usize] as *mut mapblock_T;
                }
                while !mp.is_null() {
                    if (*mp).m_noremap != REMAP_SCRIPT as ::core::ffi::c_int {
                        if (*mp).m_luaref == LUA_NOREF {
                            let mut p: *mut ::core::ffi::c_char =
                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                            p = (*mp).m_str;
                            while *p as ::core::ffi::c_int != NUL {
                                if *p.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                    as ::core::ffi::c_int
                                    == K_SPECIAL
                                    && *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                        as ::core::ffi::c_int
                                        == KS_EXTRA
                                    && *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == KE_SNR as ::core::ffi::c_int
                                {
                                    break;
                                }
                                p = p.offset(1);
                            }
                            if *p as ::core::ffi::c_int == NUL {
                                let mut c1: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                                let mut c2: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                                let mut c3: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                                let mut cmd: *mut ::core::ffi::c_char = (if abbr != 0 {
                                    b"abbr\0".as_ptr() as *const ::core::ffi::c_char
                                } else {
                                    b"map\0".as_ptr() as *const ::core::ffi::c_char
                                })
                                    as *mut ::core::ffi::c_char;
                                match (*mp).m_mode {
                                    71 => {}
                                    1 => {
                                        c1 = 'n' as ::core::ffi::c_char;
                                    }
                                    2 => {
                                        c1 = 'x' as ::core::ffi::c_char;
                                    }
                                    64 => {
                                        c1 = 's' as ::core::ffi::c_char;
                                    }
                                    4 => {
                                        c1 = 'o' as ::core::ffi::c_char;
                                    }
                                    3 => {
                                        c1 = 'n' as ::core::ffi::c_char;
                                        c2 = 'x' as ::core::ffi::c_char;
                                    }
                                    65 => {
                                        c1 = 'n' as ::core::ffi::c_char;
                                        c2 = 's' as ::core::ffi::c_char;
                                    }
                                    5 => {
                                        c1 = 'n' as ::core::ffi::c_char;
                                        c2 = 'o' as ::core::ffi::c_char;
                                    }
                                    66 => {
                                        c1 = 'v' as ::core::ffi::c_char;
                                    }
                                    6 => {
                                        c1 = 'x' as ::core::ffi::c_char;
                                        c2 = 'o' as ::core::ffi::c_char;
                                    }
                                    68 => {
                                        c1 = 's' as ::core::ffi::c_char;
                                        c2 = 'o' as ::core::ffi::c_char;
                                    }
                                    67 => {
                                        c1 = 'n' as ::core::ffi::c_char;
                                        c2 = 'v' as ::core::ffi::c_char;
                                    }
                                    7 => {
                                        c1 = 'n' as ::core::ffi::c_char;
                                        c2 = 'x' as ::core::ffi::c_char;
                                        c3 = 'o' as ::core::ffi::c_char;
                                    }
                                    69 => {
                                        c1 = 'n' as ::core::ffi::c_char;
                                        c2 = 's' as ::core::ffi::c_char;
                                        c3 = 'o' as ::core::ffi::c_char;
                                    }
                                    70 => {
                                        c1 = 'v' as ::core::ffi::c_char;
                                        c2 = 'o' as ::core::ffi::c_char;
                                    }
                                    24 => {
                                        if abbr == 0 {
                                            cmd = b"map!\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char;
                                        }
                                    }
                                    8 => {
                                        c1 = 'c' as ::core::ffi::c_char;
                                    }
                                    16 => {
                                        c1 = 'i' as ::core::ffi::c_char;
                                    }
                                    32 => {
                                        c1 = 'l' as ::core::ffi::c_char;
                                    }
                                    128 => {
                                        c1 = 't' as ::core::ffi::c_char;
                                    }
                                    _ => {
                                        iemsg(gettext(b"E228: makemap: Illegal mode\0".as_ptr()
                                            as *const ::core::ffi::c_char));
                                        return FAIL;
                                    }
                                }
                                loop {
                                    if !did_cpo {
                                        if *(*mp).m_str as ::core::ffi::c_int == NUL {
                                            did_cpo = true_0 != 0;
                                        } else {
                                            let specials: [::core::ffi::c_char; 3] = [
                                                K_SPECIAL as uint8_t as ::core::ffi::c_char,
                                                NL as ::core::ffi::c_char,
                                                NUL as ::core::ffi::c_char,
                                            ];
                                            if !strpbrk(
                                                (*mp).m_str,
                                                &raw const specials as *const ::core::ffi::c_char,
                                            )
                                            .is_null()
                                                || !strpbrk(
                                                    (*mp).m_keys,
                                                    &raw const specials
                                                        as *const ::core::ffi::c_char,
                                                )
                                                .is_null()
                                            {
                                                did_cpo = true_0 != 0;
                                            }
                                        }
                                        if did_cpo {
                                            if fprintf(
                                                fd,
                                                b"let s:cpo_save=&cpo\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ) < 0 as ::core::ffi::c_int
                                                || put_eol(fd) < 0 as ::core::ffi::c_int
                                                || fprintf(
                                                    fd,
                                                    b"set cpo&vim\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ) < 0 as ::core::ffi::c_int
                                                || put_eol(fd) < 0 as ::core::ffi::c_int
                                            {
                                                return FAIL;
                                            }
                                        }
                                    }
                                    if c1 as ::core::ffi::c_int != 0
                                        && putc(c1 as ::core::ffi::c_int, fd)
                                            < 0 as ::core::ffi::c_int
                                    {
                                        return FAIL;
                                    }
                                    if (*mp).m_noremap != REMAP_YES as ::core::ffi::c_int
                                        && fprintf(
                                            fd,
                                            b"nore\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) < 0 as ::core::ffi::c_int
                                    {
                                        return FAIL;
                                    }
                                    if fputs(cmd, fd) < 0 as ::core::ffi::c_int {
                                        return FAIL;
                                    }
                                    if !buf.is_null()
                                        && fputs(
                                            b" <buffer>\0".as_ptr() as *const ::core::ffi::c_char,
                                            fd,
                                        ) < 0 as ::core::ffi::c_int
                                    {
                                        return FAIL;
                                    }
                                    if (*mp).m_nowait as ::core::ffi::c_int != 0
                                        && fputs(
                                            b" <nowait>\0".as_ptr() as *const ::core::ffi::c_char,
                                            fd,
                                        ) < 0 as ::core::ffi::c_int
                                    {
                                        return FAIL;
                                    }
                                    if (*mp).m_silent as ::core::ffi::c_int != 0
                                        && fputs(
                                            b" <silent>\0".as_ptr() as *const ::core::ffi::c_char,
                                            fd,
                                        ) < 0 as ::core::ffi::c_int
                                    {
                                        return FAIL;
                                    }
                                    if (*mp).m_expr as ::core::ffi::c_int != 0
                                        && fputs(
                                            b" <expr>\0".as_ptr() as *const ::core::ffi::c_char,
                                            fd,
                                        ) < 0 as ::core::ffi::c_int
                                    {
                                        return FAIL;
                                    }
                                    if putc(' ' as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int
                                        || put_escstr(fd, (*mp).m_keys, 0 as ::core::ffi::c_int)
                                            == FAIL
                                        || putc(' ' as ::core::ffi::c_int, fd)
                                            < 0 as ::core::ffi::c_int
                                        || put_escstr(fd, (*mp).m_str, 1 as ::core::ffi::c_int)
                                            == FAIL
                                        || put_eol(fd) < 0 as ::core::ffi::c_int
                                    {
                                        return FAIL;
                                    }
                                    c1 = c2;
                                    c2 = c3;
                                    c3 = NUL as ::core::ffi::c_char;
                                    if c1 as ::core::ffi::c_int == NUL {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    mp = (*mp).m_next;
                }
                hash += 1;
            }
            abbr += 1;
        }
        if did_cpo {
            if fprintf(
                fd,
                b"let &cpo=s:cpo_save\0".as_ptr() as *const ::core::ffi::c_char,
            ) < 0 as ::core::ffi::c_int
                || put_eol(fd) < 0 as ::core::ffi::c_int
                || fprintf(
                    fd,
                    b"unlet s:cpo_save\0".as_ptr() as *const ::core::ffi::c_char,
                ) < 0 as ::core::ffi::c_int
                || put_eol(fd) < 0 as ::core::ffi::c_int
            {
                return FAIL;
            }
        }
        return OK;
    }
}

pub unsafe extern "C" fn put_escstr(
    mut fd: *mut FILE,
    mut strstart: *const ::core::ffi::c_char,
    mut what: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut str: *mut uint8_t = strstart as *mut uint8_t;
        if *str as ::core::ffi::c_int == NUL && what == 1 as ::core::ffi::c_int {
            if fprintf(fd, b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char)
                < 0 as ::core::ffi::c_int
            {
                return FAIL;
            }
            return OK;
        }
        while *str as ::core::ffi::c_int != NUL {
            let mut p: *const ::core::ffi::c_char =
                mb_unescape(&raw mut str as *mut *const ::core::ffi::c_char);
            's_26: {
                if !p.is_null() {
                    while *p as ::core::ffi::c_int != NUL {
                        let c2rust_fresh19 = p;
                        p = p.offset(1);
                        if fputc(*c2rust_fresh19 as ::core::ffi::c_int, fd)
                            < 0 as ::core::ffi::c_int
                        {
                            return FAIL;
                        }
                    }
                    str = str.offset(-1);
                } else {
                    let mut c: ::core::ffi::c_int = *str as ::core::ffi::c_int;
                    if c == K_SPECIAL && what != 2 as ::core::ffi::c_int {
                        let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == KS_MODIFIER
                        {
                            modifiers =
                                *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
                            str = str.offset(3 as ::core::ffi::c_int as isize);
                            p = mb_unescape(&raw mut str as *mut *const ::core::ffi::c_char);
                            if p.is_null() {
                                c = *str as ::core::ffi::c_int;
                            } else {
                                c = utf_ptr2char(p);
                                str = str.offset(-1);
                            }
                        }
                        if c == K_SPECIAL {
                            c = if *str.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == KS_SPECIAL
                            {
                                K_SPECIAL
                            } else if *str.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == KS_ZERO
                            {
                                K_ZERO
                            } else {
                                -(*str.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    + ((*str.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        << 8 as ::core::ffi::c_int))
                            };
                            str = str.offset(2 as ::core::ffi::c_int as isize);
                        }
                        if c < 0 as ::core::ffi::c_int || modifiers != 0 {
                            if fputs(get_special_key_name(c, modifiers), fd)
                                < 0 as ::core::ffi::c_int
                            {
                                return FAIL;
                            }
                            break 's_26;
                        }
                    }
                    if c == NL {
                        if what == 2 as ::core::ffi::c_int {
                            if fprintf(fd, b"\\\x16\n\0".as_ptr() as *const ::core::ffi::c_char)
                                < 0 as ::core::ffi::c_int
                            {
                                return FAIL;
                            }
                        } else if fprintf(fd, b"<NL>\0".as_ptr() as *const ::core::ffi::c_char)
                            < 0 as ::core::ffi::c_int
                        {
                            return FAIL;
                        }
                    } else {
                        if what == 2 as ::core::ffi::c_int
                            && (ascii_iswhite(c) as ::core::ffi::c_int != 0
                                || c == '"' as ::core::ffi::c_int
                                || c == '\\' as ::core::ffi::c_int)
                        {
                            if putc('\\' as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int {
                                return FAIL;
                            }
                        } else if c < ' ' as ::core::ffi::c_int
                            || c > '~' as ::core::ffi::c_int
                            || c == '|' as ::core::ffi::c_int
                            || what == 0 as ::core::ffi::c_int && c == ' ' as ::core::ffi::c_int
                            || what == 1 as ::core::ffi::c_int
                                && str == strstart as *mut uint8_t
                                && c == ' ' as ::core::ffi::c_int
                            || what != 2 as ::core::ffi::c_int && c == '<' as ::core::ffi::c_int
                        {
                            if putc(Ctrl_V, fd) < 0 as ::core::ffi::c_int {
                                return FAIL;
                            }
                        }
                        if putc(c, fd) < 0 as ::core::ffi::c_int {
                            return FAIL;
                        }
                    }
                }
            }
            str = str.offset(1);
        }
        return OK;
    }
}
