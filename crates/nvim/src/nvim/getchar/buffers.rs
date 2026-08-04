//! The stuff/read/record buffers: [`buffheader_T`] and its chain.
//!
//! A `buffheader_T` is a linked list of [`buffblock`]s holding a byte string
//! that is appended to at the tail ([`add_buff`]) and consumed from the head
//! ([`read_readbuf`]).  Five of them exist — the two read buffers behind
//! `stuffReadbuff`, the record buffer behind `q`, and the redo pair — and
//! every one of them is filled by the functions here.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn free_buff(mut buf: *mut buffheader_T) {
    unsafe {
        let mut np: *mut buffblock_T = ::core::ptr::null_mut::<buffblock_T>();
        let mut p: *mut buffblock_T = (*buf).bh_first.b_next as *mut buffblock_T;
        while !p.is_null() {
            np = (*p).b_next as *mut buffblock_T;
            xfree(p as *mut ::core::ffi::c_void);
            p = np;
        }
        (*buf).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
        (*buf).bh_curr = ::core::ptr::null_mut::<buffblock_T>();
    }
}

pub(crate) unsafe extern "C" fn get_buffcont(
    mut buffer: *mut buffheader_T,
    mut dozero: ::core::ffi::c_int,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut count: size_t = 0 as size_t;
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut i: size_t = 0 as size_t;
        let mut bp: *const buffblock_T = (*buffer).bh_first.b_next;
        while !bp.is_null() {
            count = count.wrapping_add((*bp).b_strlen);
            bp = (*bp).b_next;
        }
        if count > 0 as size_t || dozero != 0 {
            p = xmalloc(count.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
            let mut p2: *mut ::core::ffi::c_char = p;
            let mut bp_0: *const buffblock_T = (*buffer).bh_first.b_next;
            while !bp_0.is_null() {
                let mut str: *const ::core::ffi::c_char =
                    &raw const (*bp_0).b_str as *const ::core::ffi::c_char;
                while *str != 0 {
                    let c2rust_fresh0 = str;
                    str = str.offset(1);
                    let c2rust_fresh1 = p2;
                    p2 = p2.offset(1);
                    *c2rust_fresh1 = *c2rust_fresh0;
                }
                bp_0 = (*bp_0).b_next;
            }
            *p2 = NUL as ::core::ffi::c_char;
            i = p2.offset_from(p) as size_t;
        }
        if !len.is_null() {
            *len = i;
        }
        return p;
    }
}

pub unsafe extern "C" fn get_recorded() -> *mut ::core::ffi::c_char {
    unsafe {
        let mut len: size_t = 0;
        let mut p: *mut ::core::ffi::c_char = get_buffcont(recordbuff.ptr(), true_0, &raw mut len);
        if p.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        free_buff(recordbuff.ptr());
        if len >= last_recorded_len.get() {
            len = len.wrapping_sub(last_recorded_len.get());
            *p.offset(len as isize) = NUL as ::core::ffi::c_char;
        }
        if len > 0 as size_t
            && restart_edit.get() != 0 as ::core::ffi::c_int
            && *p.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == Ctrl_O
        {
            *p.offset(len.wrapping_sub(1 as size_t) as isize) = NUL as ::core::ffi::c_char;
        }
        return p;
    }
}

pub unsafe extern "C" fn get_inserted() -> String_0 {
    unsafe {
        let mut len: size_t = 0 as size_t;
        let mut str: *mut ::core::ffi::c_char = get_buffcont(redobuff.ptr(), false_0, &raw mut len);
        return String_0 {
            data: str,
            size: len,
        };
    }
}

pub(crate) unsafe extern "C" fn add_buff(
    buf: *mut buffheader_T,
    s: *const ::core::ffi::c_char,
    mut slen: ptrdiff_t,
) {
    unsafe {
        if slen < 0 as ptrdiff_t {
            slen = strlen(s) as ptrdiff_t;
        }
        if slen == 0 as ptrdiff_t {
            return;
        }
        if (*buf).bh_first.b_next.is_null() {
            (*buf).bh_curr = &raw mut (*buf).bh_first;
            (*buf).bh_create_newblock = true_0 != 0;
        } else if (*buf).bh_curr.is_null() {
            iemsg(gettext(
                b"E222: Add to read buffer\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        } else if (*buf).bh_index != 0 as size_t {
            memmove(
                &raw mut (*(*buf).bh_first.b_next).b_str as *mut ::core::ffi::c_char
                    as *mut ::core::ffi::c_void,
                (&raw mut (*(*buf).bh_first.b_next).b_str as *mut ::core::ffi::c_char)
                    .offset((*buf).bh_index as isize) as *const ::core::ffi::c_void,
                (*(*buf).bh_first.b_next)
                    .b_strlen
                    .wrapping_sub((*buf).bh_index)
                    .wrapping_add(1 as size_t),
            );
            (*(*buf).bh_first.b_next).b_strlen = (*(*buf).bh_first.b_next)
                .b_strlen
                .wrapping_sub((*buf).bh_index);
            (*buf).bh_space = (*buf).bh_space.wrapping_add((*buf).bh_index);
        }
        (*buf).bh_index = 0 as size_t;
        if !(*buf).bh_create_newblock && (*buf).bh_space >= slen as size_t {
            xmemcpyz(
                (&raw mut (*(*buf).bh_curr).b_str as *mut ::core::ffi::c_char)
                    .offset((*(*buf).bh_curr).b_strlen as isize)
                    as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                slen as size_t,
            );
            (*(*buf).bh_curr).b_strlen = (*(*buf).bh_curr).b_strlen.wrapping_add(slen as size_t);
            (*buf).bh_space = (*buf).bh_space.wrapping_sub(slen as size_t);
        } else {
            let mut len: size_t = if 20 as size_t > slen as size_t {
                20 as size_t
            } else {
                slen as size_t
            };
            let mut p: *mut buffblock_T =
                xmalloc((16 as size_t).wrapping_add(len).wrapping_add(1 as size_t))
                    as *mut buffblock_T;
            xmemcpyz(
                &raw mut (*p).b_str as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                slen as size_t,
            );
            (*p).b_strlen = slen as size_t;
            (*buf).bh_space = len.wrapping_sub(slen as size_t);
            (*buf).bh_create_newblock = false_0 != 0;
            (*p).b_next = (*(*buf).bh_curr).b_next;
            (*(*buf).bh_curr).b_next = p as *mut buffblock;
            (*buf).bh_curr = p;
        };
    }
}

pub(crate) unsafe extern "C" fn delete_buff_tail(
    mut buf: *mut buffheader_T,
    mut slen: ::core::ffi::c_int,
) {
    unsafe {
        if (*buf).bh_curr.is_null() {
            return;
        }
        if (*(*buf).bh_curr).b_strlen < slen as size_t {
            return;
        }
        *(&raw mut (*(*buf).bh_curr).b_str as *mut ::core::ffi::c_char)
            .offset((*(*buf).bh_curr).b_strlen.wrapping_sub(slen as size_t) as isize) =
            NUL as ::core::ffi::c_char;
        (*(*buf).bh_curr).b_strlen = (*(*buf).bh_curr).b_strlen.wrapping_sub(slen as size_t);
        (*buf).bh_space = (*buf).bh_space.wrapping_add(slen as size_t);
    }
}

pub(crate) unsafe extern "C" fn add_num_buff(
    mut buf: *mut buffheader_T,
    mut n: ::core::ffi::c_int,
) {
    unsafe {
        let mut number: [::core::ffi::c_char; 32] = [0; 32];
        let mut numberlen: ::core::ffi::c_int = snprintf(
            &raw mut number as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            n,
        );
        add_buff(
            buf,
            &raw mut number as *mut ::core::ffi::c_char,
            numberlen as ptrdiff_t,
        );
    }
}

pub(crate) unsafe extern "C" fn add_byte_buff(
    mut buf: *mut buffheader_T,
    mut c: ::core::ffi::c_int,
) {
    unsafe {
        let mut temp: [::core::ffi::c_char; 4] = [0; 4];
        let mut templen: ptrdiff_t = 0;
        if c < 0 as ::core::ffi::c_int || c == K_SPECIAL || c == NUL {
            temp[0 as ::core::ffi::c_int as usize] = K_SPECIAL as ::core::ffi::c_char;
            temp[1 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL {
                KS_SPECIAL
            } else if c == NUL {
                KS_ZERO
            } else {
                -c & 0xff as ::core::ffi::c_int
            }) as ::core::ffi::c_char;
            temp[2 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL || c == NUL {
                KE_FILLER as ::core::ffi::c_uint
            } else {
                -c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint
            }) as ::core::ffi::c_char;
            temp[3 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            templen = 3 as ptrdiff_t;
        } else {
            temp[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
            temp[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            templen = 1 as ptrdiff_t;
        }
        add_buff(buf, &raw mut temp as *mut ::core::ffi::c_char, templen);
    }
}

pub(crate) unsafe extern "C" fn add_char_buff(
    mut buf: *mut buffheader_T,
    mut c: ::core::ffi::c_int,
) {
    unsafe {
        let mut bytes: [uint8_t; 22] = [0; 22];
        let mut len: ::core::ffi::c_int = 0;
        if c < 0 as ::core::ffi::c_int {
            len = 1 as ::core::ffi::c_int;
        } else {
            len = utf_char2bytes(
                c,
                &raw mut bytes as *mut uint8_t as *mut ::core::ffi::c_char,
            );
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < len {
            if !(c < 0 as ::core::ffi::c_int) {
                c = bytes[i as usize] as ::core::ffi::c_int;
            }
            add_byte_buff(buf, c);
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn read_readbuffers(mut advance: bool) -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = read_readbuf(readbuf1.ptr(), advance);
        if c == NUL {
            c = read_readbuf(readbuf2.ptr(), advance);
        }
        return c;
    }
}

pub(crate) unsafe extern "C" fn read_readbuf(
    mut buf: *mut buffheader_T,
    mut advance: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if (*buf).bh_first.b_next.is_null() {
            return NUL;
        }
        let curr: *mut buffblock_T = (*buf).bh_first.b_next as *mut buffblock_T;
        let mut c: uint8_t = *(&raw mut (*curr).b_str as *mut ::core::ffi::c_char)
            .offset((*buf).bh_index as isize) as uint8_t;
        if advance {
            (*buf).bh_index = (*buf).bh_index.wrapping_add(1);
            if *(&raw mut (*curr).b_str as *mut ::core::ffi::c_char)
                .offset((*buf).bh_index as isize) as ::core::ffi::c_int
                == NUL
            {
                (*buf).bh_first.b_next = (*curr).b_next;
                xfree(curr as *mut ::core::ffi::c_void);
                (*buf).bh_index = 0 as size_t;
            }
        }
        return c as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn start_stuff() {
    unsafe {
        if !(*readbuf1.ptr()).bh_first.b_next.is_null() {
            (*readbuf1.ptr()).bh_curr = &raw mut (*readbuf1.ptr()).bh_first;
            (*readbuf1.ptr()).bh_create_newblock = true_0 != 0;
        }
        if !(*readbuf2.ptr()).bh_first.b_next.is_null() {
            (*readbuf2.ptr()).bh_curr = &raw mut (*readbuf2.ptr()).bh_first;
            (*readbuf2.ptr()).bh_create_newblock = true_0 != 0;
        }
    }
}

pub unsafe extern "C" fn stuff_empty() -> bool {
    unsafe {
        return (*readbuf1.ptr()).bh_first.b_next.is_null()
            && (*readbuf2.ptr()).bh_first.b_next.is_null();
    }
}

pub unsafe extern "C" fn readbuf1_empty() -> bool {
    unsafe {
        return (*readbuf1.ptr()).bh_first.b_next.is_null();
    }
}

pub unsafe extern "C" fn typeahead_noflush(mut c: ::core::ffi::c_int) {
    typeahead_char.set(c);
}

pub unsafe extern "C" fn flush_buffers(mut flush_typeahead: flush_buffers_T) {
    unsafe {
        init_typebuf();
        start_stuff();
        while read_readbuffers(true_0 != 0) != NUL {}
        if flush_typeahead as ::core::ffi::c_uint
            == FLUSH_MINIMAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*typebuf.ptr()).tb_off + (*typebuf.ptr()).tb_maplen >= (*typebuf.ptr()).tb_buflen {
                (*typebuf.ptr()).tb_off = MAXMAPLEN as ::core::ffi::c_int;
                (*typebuf.ptr()).tb_len = 0 as ::core::ffi::c_int;
            } else {
                (*typebuf.ptr()).tb_off += (*typebuf.ptr()).tb_maplen;
                (*typebuf.ptr()).tb_len -= (*typebuf.ptr()).tb_maplen;
            }
            if (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int {
                typebuf_was_filled.set(false_0 != 0);
            }
        } else {
            if flush_typeahead as ::core::ffi::c_uint
                == FLUSH_INPUT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                while inchar(
                    (*typebuf.ptr()).tb_buf,
                    (*typebuf.ptr()).tb_buflen - 1 as ::core::ffi::c_int,
                    10 as ::core::ffi::c_long,
                ) != 0 as ::core::ffi::c_int
                {}
            }
            (*typebuf.ptr()).tb_off = MAXMAPLEN as ::core::ffi::c_int;
            (*typebuf.ptr()).tb_len = 0 as ::core::ffi::c_int;
            typebuf_was_filled.set(false_0 != 0);
        }
        (*typebuf.ptr()).tb_maplen = 0 as ::core::ffi::c_int;
        (*typebuf.ptr()).tb_silent = 0 as ::core::ffi::c_int;
        cmd_silent.set(false_0 != 0);
        (*typebuf.ptr()).tb_no_abbr_cnt = 0 as ::core::ffi::c_int;
        (*typebuf.ptr()).tb_change_cnt += 1;
        if (*typebuf.ptr()).tb_change_cnt == 0 as ::core::ffi::c_int {
            (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
        }
    }
}

pub unsafe extern "C" fn beep_flush() {
    unsafe {
        if emsg_silent.get() == 0 as ::core::ffi::c_int {
            flush_buffers(FLUSH_MINIMAL);
            vim_beep(kOptBoFlagError as ::core::ffi::c_int as ::core::ffi::c_uint);
        }
    }
}

pub unsafe extern "C" fn stuffReadbuff(mut s: *const ::core::ffi::c_char) {
    unsafe {
        add_buff(readbuf1.ptr(), s, -1 as ptrdiff_t);
    }
}

pub unsafe extern "C" fn stuffRedoReadbuff(mut s: *const ::core::ffi::c_char) {
    unsafe {
        add_buff(readbuf2.ptr(), s, -1 as ptrdiff_t);
    }
}

pub unsafe extern "C" fn stuffReadbuffLen(mut s: *const ::core::ffi::c_char, mut len: ptrdiff_t) {
    unsafe {
        add_buff(readbuf1.ptr(), s, len);
    }
}

pub unsafe extern "C" fn stuffReadbuffSpec(mut s: *const ::core::ffi::c_char) {
    unsafe {
        while *s as ::core::ffi::c_int != NUL {
            if *s as uint8_t as ::core::ffi::c_int == K_SPECIAL
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                stuffReadbuffLen(s, 3 as ptrdiff_t);
                s = s.offset(3 as ::core::ffi::c_int as isize);
            } else {
                let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
                if c == CAR || c == NL || c == ESC {
                    c = ' ' as ::core::ffi::c_int;
                }
                stuffcharReadbuff(c);
            }
        }
    }
}

pub unsafe extern "C" fn stuffcharReadbuff(mut c: ::core::ffi::c_int) {
    unsafe {
        add_char_buff(readbuf1.ptr(), c);
    }
}

pub unsafe extern "C" fn stuffnumReadbuff(mut n: ::core::ffi::c_int) {
    unsafe {
        add_num_buff(readbuf1.ptr(), n);
    }
}

pub unsafe extern "C" fn stuffescaped(mut arg: *const ::core::ffi::c_char, mut literally: bool) {
    unsafe {
        while *arg as ::core::ffi::c_int != NUL {
            let start: *const ::core::ffi::c_char = arg;
            while *arg as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
                && (*arg as ::core::ffi::c_int) < DEL
                || *arg as uint8_t as ::core::ffi::c_int == K_SPECIAL && !literally
            {
                arg = arg.offset(1);
            }
            if arg > start {
                stuffReadbuffLen(start, arg.offset_from(start));
            }
            if *arg as ::core::ffi::c_int != NUL {
                let c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut arg);
                if literally as ::core::ffi::c_int != 0
                    && (c < ' ' as ::core::ffi::c_int && c != TAB || c == DEL)
                {
                    stuffcharReadbuff(Ctrl_V);
                }
                stuffcharReadbuff(c);
            }
        }
    }
}
