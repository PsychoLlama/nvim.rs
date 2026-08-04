//! Turning bytes into something displayable.
//!
//! The `msg_outtrans*` half renders unprintable bytes as `<xx>` and multibyte
//! sequences as themselves; the `str2special*` half renders key codes as
//! `<C-X>` notation, which is what mapping listings and `keytrans()` show.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn msg_putchar(mut c: ::core::ffi::c_int) {
    unsafe {
        msg_putchar_hl(c, 0 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn msg_putchar_hl(mut c: ::core::ffi::c_int, mut hl_id: ::core::ffi::c_int) {
    unsafe {
        let mut buf: [::core::ffi::c_char; 7] = [0; 7];
        if c < 0 as ::core::ffi::c_int {
            buf[0 as ::core::ffi::c_int as usize] = K_SPECIAL as ::core::ffi::c_char;
            buf[1 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL {
                KS_SPECIAL
            } else if c == NUL {
                KS_ZERO
            } else {
                -c & 0xff as ::core::ffi::c_int
            }) as ::core::ffi::c_char;
            buf[2 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL || c == NUL {
                KE_FILLER as ::core::ffi::c_uint
            } else {
                -c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint
            }) as ::core::ffi::c_char;
            buf[3 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        } else {
            buf[utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char) as usize] =
                NUL as ::core::ffi::c_char;
        }
        msg_puts_hl(
            &raw mut buf as *mut ::core::ffi::c_char,
            hl_id,
            false_0 != 0,
        );
    }
}

pub unsafe extern "C" fn msg_outnum(mut n: ::core::ffi::c_int) {
    unsafe {
        let mut buf: [::core::ffi::c_char; 20] = [0; 20];
        snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            n,
        );
        msg_puts(&raw mut buf as *mut ::core::ffi::c_char);
    }
}

pub unsafe extern "C" fn msg_home_replace(mut fname: *const ::core::ffi::c_char) {
    unsafe {
        msg_home_replace_hl(fname, 0 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn msg_home_replace_hl(
    mut fname: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
) {
    unsafe {
        let mut name: *mut ::core::ffi::c_char =
            home_replace_save(::core::ptr::null_mut::<buf_T>(), fname);
        msg_outtrans(name, hl_id, false_0 != 0);
        xfree(name as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn msg_outtrans(
    mut str: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) -> ::core::ffi::c_int {
    unsafe {
        return msg_outtrans_len(str, strlen(str) as ::core::ffi::c_int, hl_id, hist);
    }
}

pub unsafe extern "C" fn msg_outtrans_one(
    mut p: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut l: ::core::ffi::c_int = 0;
        l = utfc_ptr2len(p);
        if l > 1 as ::core::ffi::c_int {
            msg_outtrans_len(p, l, hl_id, hist);
            return p.offset(l as isize);
        }
        msg_puts_hl(
            transchar_byte_buf(
                ::core::ptr::null::<buf_T>(),
                *p as uint8_t as ::core::ffi::c_int,
            ),
            hl_id,
            hist,
        );
        return p.offset(1 as ::core::ffi::c_int as isize);
    }
}

pub unsafe extern "C" fn msg_outtrans_len(
    mut msgstr: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut str: *const ::core::ffi::c_char = msgstr;
        let mut plain_start: *const ::core::ffi::c_char = msgstr;
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut c: ::core::ffi::c_int = 0;
        let mut save_got_int: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
        got_int.set(false_0 != 0);
        if hist {
            msg_hist_add(str, len, hl_id);
        }
        if msg_silent.get() == 0 as ::core::ffi::c_int
            && len > 0 as ::core::ffi::c_int
            && msg_row.get() >= cmdline_row.get()
            && msg_col.get() == 0 as ::core::ffi::c_int
        {
            clear_cmdline.set(false_0 != 0);
            mode_displayed.set(false_0 != 0);
        }
        loop {
            len -= 1;
            if !(len >= 0 as ::core::ffi::c_int && !got_int.get()) {
                break;
            }
            let mut mb_l: ::core::ffi::c_int = utfc_ptr2len_len(str, len + 1 as ::core::ffi::c_int);
            if mb_l > 1 as ::core::ffi::c_int {
                c = utf_ptr2char(str);
                if vim_isprintc(c) {
                    retval += utf_ptr2cells(str);
                } else {
                    if str > plain_start {
                        msg_puts_len(plain_start, str.offset_from(plain_start), hl_id, hist);
                    }
                    plain_start = str.offset(mb_l as isize);
                    msg_puts_hl(
                        transchar_buf(::core::ptr::null::<buf_T>(), c),
                        if hl_id == 0 as ::core::ffi::c_int {
                            HLF_8
                        } else {
                            hl_id
                        },
                        false_0 != 0,
                    );
                    retval += char2cells(c);
                }
                len -= mb_l - 1 as ::core::ffi::c_int;
                str = str.offset(mb_l as isize);
            } else {
                s = transchar_byte_buf(
                    ::core::ptr::null::<buf_T>(),
                    *str as uint8_t as ::core::ffi::c_int,
                );
                if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    if str > plain_start {
                        msg_puts_len(plain_start, str.offset_from(plain_start), hl_id, hist);
                    }
                    plain_start = str.offset(1 as ::core::ffi::c_int as isize);
                    msg_puts_hl(
                        s,
                        if hl_id == 0 as ::core::ffi::c_int {
                            HLF_8
                        } else {
                            hl_id
                        },
                        false_0 != 0,
                    );
                    retval += strlen(s) as ::core::ffi::c_int;
                } else {
                    retval += 1;
                }
                str = str.offset(1);
            }
        }
        if (str > plain_start || plain_start == msgstr) && !got_int.get() {
            msg_puts_len(plain_start, str.offset_from(plain_start), hl_id, hist);
        }
        got_int.set(got_int.get() as ::core::ffi::c_int | save_got_int != 0);
        return retval;
    }
}

pub unsafe extern "C" fn msg_make(mut arg: *const ::core::ffi::c_char) {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        static str: GlobalCell<*const ::core::ffi::c_char> =
            GlobalCell::new(b"eeffoc\0".as_ptr() as *const ::core::ffi::c_char);
        static rs: GlobalCell<*const ::core::ffi::c_char> =
            GlobalCell::new(b"Plon#dqg#vxjduB\0".as_ptr() as *const ::core::ffi::c_char);
        arg = skipwhite(arg);
        i = 5 as ::core::ffi::c_int;
        while *arg as ::core::ffi::c_int != 0 && i >= 0 as ::core::ffi::c_int {
            let c2rust_fresh33 = arg;
            arg = arg.offset(1);
            if *c2rust_fresh33 as ::core::ffi::c_int
                != *(*str.ptr()).offset(i as isize) as ::core::ffi::c_int
            {
                break;
            }
            i -= 1;
        }
        if i < 0 as ::core::ffi::c_int {
            msg_putchar('\n' as ::core::ffi::c_int);
            i = 0 as ::core::ffi::c_int;
            while *(*rs.ptr()).offset(i as isize) != 0 {
                msg_putchar(
                    *(*rs.ptr()).offset(i as isize) as ::core::ffi::c_int - 3 as ::core::ffi::c_int,
                );
                i += 1;
            }
        }
    }
}

pub unsafe extern "C" fn msg_outtrans_special(
    mut strstart: *const ::core::ffi::c_char,
    mut from: bool,
    mut maxlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if strstart.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        let mut str: *const ::core::ffi::c_char = strstart;
        let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut hl_id: ::core::ffi::c_int = HLF_8;
        while *str as ::core::ffi::c_int != NUL {
            let mut text: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            if (str == strstart
                || *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
                && *str as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            {
                text = b"<Space>\0".as_ptr() as *const ::core::ffi::c_char;
                str = str.offset(1);
            } else {
                text = str2special(&raw mut str, from, false_0 != 0);
            }
            if *text.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *text.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            {
                text = transchar_byte_buf(
                    ::core::ptr::null::<buf_T>(),
                    *text.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
                );
            }
            let len: ::core::ffi::c_int = vim_strsize(text);
            if maxlen > 0 as ::core::ffi::c_int && retval + len >= maxlen {
                break;
            }
            msg_puts_hl(
                text,
                if len > 1 as ::core::ffi::c_int && utfc_ptr2len(text) <= 1 as ::core::ffi::c_int {
                    hl_id
                } else {
                    0 as ::core::ffi::c_int
                },
                false_0 != 0,
            );
            retval += len;
        }
        return retval;
    }
}

pub unsafe extern "C" fn str2special_save(
    str: *const ::core::ffi::c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            1 as ::core::ffi::c_int,
            40 as ::core::ffi::c_int,
        );
        let mut p: *const ::core::ffi::c_char = str;
        while *p as ::core::ffi::c_int != NUL {
            ga_concat(
                &raw mut ga,
                str2special(&raw mut p, replace_spaces, replace_lt),
            );
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        return ga.ga_data as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn str2special_arena(
    mut str: *const ::core::ffi::c_char,
    mut replace_spaces: bool,
    mut replace_lt: bool,
    mut arena: *mut Arena,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *const ::core::ffi::c_char = str;
        let mut len: size_t = 0 as size_t;
        while *p != 0 {
            len = len.wrapping_add(strlen(str2special(&raw mut p, replace_spaces, replace_lt)));
        }
        let mut buf: *mut ::core::ffi::c_char =
            arena_alloc(arena, len.wrapping_add(1 as size_t), false_0 != 0)
                as *mut ::core::ffi::c_char;
        let mut pos: size_t = 0 as size_t;
        p = str;
        while *p != 0 {
            let mut s: *const ::core::ffi::c_char =
                str2special(&raw mut p, replace_spaces, replace_lt);
            let mut s_len: size_t = strlen(s);
            memcpy(
                buf.offset(pos as isize) as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                s_len,
            );
            pos = pos.wrapping_add(s_len);
        }
        *buf.offset(pos as isize) = NUL as ::core::ffi::c_char;
        return buf;
    }
}

pub unsafe extern "C" fn str2special(
    sp: *mut *const ::core::ffi::c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *const ::core::ffi::c_char {
    unsafe {
        static buf: GlobalCell<[::core::ffi::c_char; 7]> = GlobalCell::new([0; 7]);
        let p: *const ::core::ffi::c_char = mb_unescape(sp);
        if !p.is_null() {
            return p;
        }
        let mut str: *const ::core::ffi::c_char = *sp;
        let mut c: ::core::ffi::c_int = *str as uint8_t as ::core::ffi::c_int;
        let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut special: bool = false_0 != 0;
        if c == K_SPECIAL
            && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            if *str.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                == KS_MODIFIER
            {
                modifiers =
                    *str.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int;
                str = str.offset(3 as ::core::ffi::c_int as isize);
                c = *str as uint8_t as ::core::ffi::c_int;
            }
            if c == K_SPECIAL
                && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                c = if *str.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                    as ::core::ffi::c_int
                    == KS_SPECIAL
                {
                    K_SPECIAL
                } else if *str.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                    as ::core::ffi::c_int
                    == KS_ZERO
                {
                    K_ZERO
                } else {
                    -(*str.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int
                        + ((*str.offset(2 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int)
                            << 8 as ::core::ffi::c_int))
                };
                str = str.offset(2 as ::core::ffi::c_int as isize);
            }
            if c < 0 as ::core::ffi::c_int || modifiers != 0 {
                special = true_0 != 0;
            }
        }
        if !(c < 0 as ::core::ffi::c_int)
            && (*utf8len_tab.ptr())[c as usize] as ::core::ffi::c_int > 1 as ::core::ffi::c_int
        {
            *sp = str;
            let mut p_0: *const ::core::ffi::c_char = mb_unescape(sp);
            if !p_0.is_null() {
                c = utf_ptr2char(p_0);
            } else {
                *sp = str.offset(1 as ::core::ffi::c_int as isize);
            }
        } else {
            *sp = str.offset(
                (if *str as ::core::ffi::c_int == NUL {
                    0 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) as isize,
            );
        }
        if special as ::core::ffi::c_int != 0
            || c < ' ' as ::core::ffi::c_int
            || replace_spaces as ::core::ffi::c_int != 0 && c == ' ' as ::core::ffi::c_int
            || replace_lt as ::core::ffi::c_int != 0 && c == '<' as ::core::ffi::c_int
        {
            return get_special_key_name(c, modifiers);
        }
        (*buf.ptr())[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
        (*buf.ptr())[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        return buf.ptr() as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn msg_outtrans_long(
    mut longstr: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
) {
    unsafe {
        let mut len: ::core::ffi::c_int = strlen(longstr) as ::core::ffi::c_int;
        let mut slen: ::core::ffi::c_int = len;
        let mut room: ::core::ffi::c_int = Columns.get() - msg_col.get();
        if !ui_has(kUIMessages) && len > room && room >= 20 as ::core::ffi::c_int {
            slen = (room - 3 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int;
            msg_outtrans_len(longstr, slen, hl_id, false_0 != 0);
            msg_puts_hl(
                b"...\0".as_ptr() as *const ::core::ffi::c_char,
                HLF_8,
                false_0 != 0,
            );
        }
        msg_outtrans_len(
            longstr.offset(len as isize).offset(-(slen as isize)),
            slen,
            hl_id,
            false_0 != 0,
        );
    }
}
