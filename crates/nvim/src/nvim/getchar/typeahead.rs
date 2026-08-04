//! The typeahead buffer: [`typebuf`], the queue `vgetc` reads from.
//!
//! `typebuf.tb_buf` holds bytes waiting to be interpreted, with a parallel
//! `tb_noremap` array saying how much remapping each byte is still allowed.
//! [`ins_typebuf`] pushes (that is what `feedkeys()` and every mapping
//! expansion do) and [`del_typebuf`] pops; the pair must keep `tb_off`,
//! `tb_len`, `tb_maplen`, `tb_silent` and `tb_no_abbr_cnt` consistent.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn init_typebuf() {
    unsafe {
        if !(*typebuf.ptr()).tb_buf.is_null() {
            return;
        }
        (*typebuf.ptr()).tb_buf = typebuf_init.ptr() as *mut uint8_t;
        (*typebuf.ptr()).tb_noremap = noremapbuf_init.ptr() as *mut uint8_t;
        (*typebuf.ptr()).tb_buflen =
            5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int);
        (*typebuf.ptr()).tb_len = 0 as ::core::ffi::c_int;
        (*typebuf.ptr()).tb_off = MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
        (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn noremap_keys() -> bool {
    return KeyNoremap.get() & (RM_NONE as ::core::ffi::c_int | RM_SCRIPT as ::core::ffi::c_int)
        != 0;
}

pub unsafe extern "C" fn ins_typebuf(
    mut str: *mut ::core::ffi::c_char,
    mut noremap: ::core::ffi::c_int,
    mut offset: ::core::ffi::c_int,
    mut nottyped: bool,
    mut silent: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut val: ::core::ffi::c_int = 0;
        let mut nrm: ::core::ffi::c_int = 0;
        init_typebuf();
        (*typebuf.ptr()).tb_change_cnt += 1;
        if (*typebuf.ptr()).tb_change_cnt == 0 as ::core::ffi::c_int {
            (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
        }
        state_no_longer_safe(b"ins_typebuf()\0".as_ptr() as *const ::core::ffi::c_char);
        let mut addlen: ::core::ffi::c_int = strlen(str) as ::core::ffi::c_int;
        if offset == 0 as ::core::ffi::c_int && addlen <= (*typebuf.ptr()).tb_off {
            (*typebuf.ptr()).tb_off -= addlen;
            memmove(
                (*typebuf.ptr())
                    .tb_buf
                    .offset((*typebuf.ptr()).tb_off as isize)
                    as *mut ::core::ffi::c_void,
                str as *const ::core::ffi::c_void,
                addlen as size_t,
            );
        } else if (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
            && (*typebuf.ptr()).tb_buflen
                >= addlen
                    + 3 as ::core::ffi::c_int
                        * (MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int)
        {
            (*typebuf.ptr()).tb_off = ((*typebuf.ptr()).tb_buflen
                - addlen
                - 3 as ::core::ffi::c_int
                    * (MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int))
                / 2 as ::core::ffi::c_int;
            memmove(
                (*typebuf.ptr())
                    .tb_buf
                    .offset((*typebuf.ptr()).tb_off as isize)
                    as *mut ::core::ffi::c_void,
                str as *const ::core::ffi::c_void,
                addlen as size_t,
            );
        } else {
            let mut newoff: ::core::ffi::c_int =
                MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
            let mut extra: ::core::ffi::c_int = addlen
                + newoff
                + 4 as ::core::ffi::c_int
                    * (MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int);
            if (*typebuf.ptr()).tb_len > INT_MAX - extra {
                emsg(gettext(&raw const e_toocompl as *const ::core::ffi::c_char));
                setcursor();
                return FAIL;
            }
            let mut newlen: ::core::ffi::c_int = (*typebuf.ptr()).tb_len + extra;
            let mut s1: *mut uint8_t = xmalloc(newlen as size_t) as *mut uint8_t;
            let mut s2: *mut uint8_t = xmalloc(newlen as size_t) as *mut uint8_t;
            (*typebuf.ptr()).tb_buflen = newlen;
            memmove(
                s1.offset(newoff as isize) as *mut ::core::ffi::c_void,
                (*typebuf.ptr())
                    .tb_buf
                    .offset((*typebuf.ptr()).tb_off as isize)
                    as *const ::core::ffi::c_void,
                offset as size_t,
            );
            memmove(
                s1.offset(newoff as isize).offset(offset as isize) as *mut ::core::ffi::c_void,
                str as *const ::core::ffi::c_void,
                addlen as size_t,
            );
            let mut bytes: ::core::ffi::c_int =
                (*typebuf.ptr()).tb_len - offset + 1 as ::core::ffi::c_int;
            '_c2rust_label: {
                if bytes > 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"bytes > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        978 as ::core::ffi::c_uint,
                        b"int ins_typebuf(char *, int, int, _Bool, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            memmove(
                s1.offset(newoff as isize)
                    .offset(offset as isize)
                    .offset(addlen as isize) as *mut ::core::ffi::c_void,
                (*typebuf.ptr())
                    .tb_buf
                    .offset((*typebuf.ptr()).tb_off as isize)
                    .offset(offset as isize) as *const ::core::ffi::c_void,
                bytes as size_t,
            );
            if (*typebuf.ptr()).tb_buf != typebuf_init.ptr() as *mut uint8_t {
                xfree((*typebuf.ptr()).tb_buf as *mut ::core::ffi::c_void);
            }
            (*typebuf.ptr()).tb_buf = s1;
            memmove(
                s2.offset(newoff as isize) as *mut ::core::ffi::c_void,
                (*typebuf.ptr())
                    .tb_noremap
                    .offset((*typebuf.ptr()).tb_off as isize)
                    as *const ::core::ffi::c_void,
                offset as size_t,
            );
            memmove(
                s2.offset(newoff as isize)
                    .offset(offset as isize)
                    .offset(addlen as isize) as *mut ::core::ffi::c_void,
                (*typebuf.ptr())
                    .tb_noremap
                    .offset((*typebuf.ptr()).tb_off as isize)
                    .offset(offset as isize) as *const ::core::ffi::c_void,
                ((*typebuf.ptr()).tb_len - offset) as size_t,
            );
            if (*typebuf.ptr()).tb_noremap != noremapbuf_init.ptr() as *mut uint8_t {
                xfree((*typebuf.ptr()).tb_noremap as *mut ::core::ffi::c_void);
            }
            (*typebuf.ptr()).tb_noremap = s2;
            (*typebuf.ptr()).tb_off = newoff;
        }
        (*typebuf.ptr()).tb_len += addlen;
        if noremap == REMAP_SCRIPT as ::core::ffi::c_int {
            val = RM_SCRIPT as ::core::ffi::c_int;
        } else if noremap == REMAP_SKIP as ::core::ffi::c_int {
            val = RM_ABBR as ::core::ffi::c_int;
        } else {
            val = RM_NONE as ::core::ffi::c_int;
        }
        if noremap == REMAP_SKIP as ::core::ffi::c_int {
            nrm = 1 as ::core::ffi::c_int;
        } else if noremap < 0 as ::core::ffi::c_int {
            nrm = addlen;
        } else {
            nrm = noremap;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < addlen {
            nrm -= 1;
            *(*typebuf.ptr())
                .tb_noremap
                .offset(((*typebuf.ptr()).tb_off + i + offset) as isize) =
                (if nrm >= 0 as ::core::ffi::c_int {
                    val
                } else {
                    RM_YES as ::core::ffi::c_int
                }) as uint8_t;
            i += 1;
        }
        if nottyped as ::core::ffi::c_int != 0 || (*typebuf.ptr()).tb_maplen > offset {
            (*typebuf.ptr()).tb_maplen += addlen;
        }
        if silent as ::core::ffi::c_int != 0 || (*typebuf.ptr()).tb_silent > offset {
            (*typebuf.ptr()).tb_silent += addlen;
            cmd_silent.set(true_0 != 0);
        }
        if (*typebuf.ptr()).tb_no_abbr_cnt != 0 && offset == 0 as ::core::ffi::c_int {
            (*typebuf.ptr()).tb_no_abbr_cnt += addlen;
        }
        return OK;
    }
}

pub unsafe extern "C" fn ins_char_typebuf(
    mut c: ::core::ffi::c_int,
    mut modifiers: ::core::ffi::c_int,
    mut on_key_ignore: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: [::core::ffi::c_char; 67] = [0; 67];
        let mut len: ::core::ffi::c_uint = special_to_buf(
            c,
            modifiers,
            true_0 != 0,
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        '_c2rust_label: {
            if (len as usize) < ::core::mem::size_of::<[::core::ffi::c_char; 67]>() {
            } else {
                __assert_fail(
                    b"len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1056 as ::core::ffi::c_uint,
                    b"int ins_char_typebuf(int, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        buf[len as usize] = NUL as ::core::ffi::c_char;
        ins_typebuf(
            &raw mut buf as *mut ::core::ffi::c_char,
            KeyNoremap.get(),
            0 as ::core::ffi::c_int,
            !KeyTyped.get(),
            cmd_silent.get(),
        );
        if KeyTyped.get() as ::core::ffi::c_int != 0 && on_key_ignore as ::core::ffi::c_int != 0 {
            on_key_ignore_len.set((*on_key_ignore_len.ptr()).wrapping_add(len as size_t));
        }
        return len as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn typebuf_changed(mut tb_change_cnt: ::core::ffi::c_int) -> bool {
    unsafe {
        return tb_change_cnt != 0 as ::core::ffi::c_int
            && ((*typebuf.ptr()).tb_change_cnt != tb_change_cnt
                || typebuf_was_filled.get() as ::core::ffi::c_int != 0);
    }
}

pub unsafe extern "C" fn typebuf_typed() -> ::core::ffi::c_int {
    unsafe {
        return ((*typebuf.ptr()).tb_maplen == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn typebuf_maplen() -> ::core::ffi::c_int {
    unsafe {
        return (*typebuf.ptr()).tb_maplen;
    }
}

pub unsafe extern "C" fn del_typebuf(mut len: ::core::ffi::c_int, mut offset: ::core::ffi::c_int) {
    unsafe {
        if len == 0 as ::core::ffi::c_int {
            return;
        }
        (*typebuf.ptr()).tb_len -= len;
        if offset == 0 as ::core::ffi::c_int
            && (*typebuf.ptr()).tb_buflen - ((*typebuf.ptr()).tb_off + len)
                >= 3 as ::core::ffi::c_int * MAXMAPLEN as ::core::ffi::c_int
                    + 3 as ::core::ffi::c_int
        {
            (*typebuf.ptr()).tb_off += len;
        } else {
            let mut i: ::core::ffi::c_int = (*typebuf.ptr()).tb_off + offset;
            if (*typebuf.ptr()).tb_off > MAXMAPLEN as ::core::ffi::c_int {
                memmove(
                    (*typebuf.ptr())
                        .tb_buf
                        .offset(MAXMAPLEN as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    (*typebuf.ptr())
                        .tb_buf
                        .offset((*typebuf.ptr()).tb_off as isize)
                        as *const ::core::ffi::c_void,
                    offset as size_t,
                );
                memmove(
                    (*typebuf.ptr())
                        .tb_noremap
                        .offset(MAXMAPLEN as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    (*typebuf.ptr())
                        .tb_noremap
                        .offset((*typebuf.ptr()).tb_off as isize)
                        as *const ::core::ffi::c_void,
                    offset as size_t,
                );
                (*typebuf.ptr()).tb_off = MAXMAPLEN as ::core::ffi::c_int;
            }
            let mut bytes: ::core::ffi::c_int =
                (*typebuf.ptr()).tb_len - offset + 1 as ::core::ffi::c_int;
            '_c2rust_label: {
                if bytes > 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"bytes > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1122 as ::core::ffi::c_uint,
                        b"void del_typebuf(int, int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            memmove(
                (*typebuf.ptr())
                    .tb_buf
                    .offset((*typebuf.ptr()).tb_off as isize)
                    .offset(offset as isize) as *mut ::core::ffi::c_void,
                (*typebuf.ptr())
                    .tb_buf
                    .offset(i as isize)
                    .offset(len as isize) as *const ::core::ffi::c_void,
                bytes as size_t,
            );
            memmove(
                (*typebuf.ptr())
                    .tb_noremap
                    .offset((*typebuf.ptr()).tb_off as isize)
                    .offset(offset as isize) as *mut ::core::ffi::c_void,
                (*typebuf.ptr())
                    .tb_noremap
                    .offset(i as isize)
                    .offset(len as isize) as *const ::core::ffi::c_void,
                ((*typebuf.ptr()).tb_len - offset) as size_t,
            );
        }
        if (*typebuf.ptr()).tb_maplen > offset {
            if (*typebuf.ptr()).tb_maplen < offset + len {
                (*typebuf.ptr()).tb_maplen = offset;
            } else {
                (*typebuf.ptr()).tb_maplen -= len;
            }
        }
        if (*typebuf.ptr()).tb_silent > offset {
            if (*typebuf.ptr()).tb_silent < offset + len {
                (*typebuf.ptr()).tb_silent = offset;
            } else {
                (*typebuf.ptr()).tb_silent -= len;
            }
        }
        if (*typebuf.ptr()).tb_no_abbr_cnt > offset {
            if (*typebuf.ptr()).tb_no_abbr_cnt < offset + len {
                (*typebuf.ptr()).tb_no_abbr_cnt = offset;
            } else {
                (*typebuf.ptr()).tb_no_abbr_cnt -= len;
            }
        }
        typebuf_was_filled.set(false_0 != 0);
        (*typebuf.ptr()).tb_change_cnt += 1;
        if (*typebuf.ptr()).tb_change_cnt == 0 as ::core::ffi::c_int {
            (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
        }
    }
}

pub unsafe extern "C" fn ungetchars(mut len: ::core::ffi::c_int) {
    unsafe {
        if reg_recording.get() == 0 as ::core::ffi::c_int {
            return;
        }
        delete_buff_tail(recordbuff.ptr(), len);
        last_recorded_len.set((*last_recorded_len.ptr()).wrapping_sub(len as size_t));
    }
}

pub unsafe extern "C" fn may_sync_undo() {
    unsafe {
        if (State.get() & (MODE_INSERT | MODE_CMDLINE) == 0
            || arrow_used.get() as ::core::ffi::c_int != 0)
            && curscript.get() < 0 as ::core::ffi::c_int
        {
            u_sync(false_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn alloc_typebuf() {
    unsafe {
        (*typebuf.ptr()).tb_buf = xmalloc(
            (5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int))
                as size_t,
        ) as *mut uint8_t;
        (*typebuf.ptr()).tb_noremap = xmalloc(
            (5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int))
                as size_t,
        ) as *mut uint8_t;
        (*typebuf.ptr()).tb_buflen =
            5 as ::core::ffi::c_int * (MAXMAPLEN as ::core::ffi::c_int + 3 as ::core::ffi::c_int);
        (*typebuf.ptr()).tb_off = MAXMAPLEN as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
        (*typebuf.ptr()).tb_len = 0 as ::core::ffi::c_int;
        (*typebuf.ptr()).tb_maplen = 0 as ::core::ffi::c_int;
        (*typebuf.ptr()).tb_silent = 0 as ::core::ffi::c_int;
        (*typebuf.ptr()).tb_no_abbr_cnt = 0 as ::core::ffi::c_int;
        (*typebuf.ptr()).tb_change_cnt += 1;
        if (*typebuf.ptr()).tb_change_cnt == 0 as ::core::ffi::c_int {
            (*typebuf.ptr()).tb_change_cnt = 1 as ::core::ffi::c_int;
        }
        typebuf_was_filled.set(false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn free_typebuf() {
    unsafe {
        if (*typebuf.ptr()).tb_buf == typebuf_init.ptr() as *mut uint8_t {
            internal_error(b"Free typebuf 1\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*typebuf.ptr()).tb_buf as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        }
        if (*typebuf.ptr()).tb_noremap == noremapbuf_init.ptr() as *mut uint8_t {
            internal_error(b"Free typebuf 2\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*typebuf.ptr()).tb_noremap as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_0;
            let _ = *ptr__0;
        };
    }
}

pub(crate) unsafe extern "C" fn save_typebuf() {
    unsafe {
        '_c2rust_label: {
            if curscript.get() >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"curscript >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1330 as ::core::ffi::c_uint,
                    b"void save_typebuf(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        init_typebuf();
        (*saved_typebuf.ptr())[curscript.get() as usize] = typebuf.get();
        alloc_typebuf();
    }
}

pub(crate) unsafe extern "C" fn can_get_old_char() -> bool {
    unsafe {
        return old_char.get() != -1 as ::core::ffi::c_int
            && (old_KeyStuffed.get() != 0 || stuff_empty() as ::core::ffi::c_int != 0);
    }
}

pub unsafe extern "C" fn save_typeahead(mut tp: *mut tasave_T) {
    unsafe {
        (*tp).save_typebuf = typebuf.get();
        alloc_typebuf();
        (*tp).typebuf_valid = true_0 != 0;
        (*tp).old_char = old_char.get();
        (*tp).old_mod_mask = old_mod_mask.get();
        old_char.set(-1 as ::core::ffi::c_int);
        (*tp).save_readbuf1 = readbuf1.get();
        (*readbuf1.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
        (*tp).save_readbuf2 = readbuf2.get();
        (*readbuf2.ptr()).bh_first.b_next = ::core::ptr::null_mut::<buffblock>();
    }
}

pub unsafe extern "C" fn restore_typeahead(mut tp: *mut tasave_T) {
    unsafe {
        if (*tp).typebuf_valid {
            free_typebuf();
            typebuf.set((*tp).save_typebuf);
        }
        old_char.set((*tp).old_char);
        old_mod_mask.set((*tp).old_mod_mask);
        free_buff(readbuf1.ptr());
        readbuf1.set((*tp).save_readbuf1);
        free_buff(readbuf2.ptr());
        readbuf2.set((*tp).save_readbuf2);
    }
}
