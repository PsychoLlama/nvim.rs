//! The text of the last insert, and repeating it.
//!
//! `last_insert` is a copy of the redo buffer taken when Insert mode was
//! left, with the command that started the insert still on the front.
//! `get_last_insert` hands it out (that is `".` and `i_CTRL-A`),
//! `get_last_insert_save` returns a copy with the trailing `<Esc>` removed,
//! and `stuff_inserted` is `.`/CTRL-A/CTRL-@: push it back into the read
//! buffer so the main loop types it again, `count` times.
//! `set_last_insert` is the single-character case `r` uses.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn set_last_insert(mut c: ::core::ffi::c_int) {
    unsafe {
        xfree((*last_insert.ptr()).data as *mut ::core::ffi::c_void);
        (*last_insert.ptr()).data = xmalloc(
            (MB_MAXBYTES as ::core::ffi::c_int * 3 as ::core::ffi::c_int + 5 as ::core::ffi::c_int)
                as size_t,
        ) as *mut ::core::ffi::c_char;
        let mut s: *mut ::core::ffi::c_char = (*last_insert.ptr()).data;
        if c < ' ' as ::core::ffi::c_int || c == DEL {
            let c2rust_fresh5 = s;
            s = s.offset(1);
            *c2rust_fresh5 = Ctrl_V as ::core::ffi::c_char;
        }
        s = add_char2buf(c, s);
        let c2rust_fresh6 = s;
        s = s.offset(1);
        *c2rust_fresh6 = ESC as ::core::ffi::c_char;
        *s = NUL as ::core::ffi::c_char;
        (*last_insert.ptr()).size = s.offset_from((*last_insert.ptr()).data) as size_t;
        last_insert_skip.set(0 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn stuff_inserted(
    mut c: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut no_esc: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut last: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
        let mut insert: String_0 = get_last_insert();
        if insert.data.is_null() {
            emsg(gettext(
                &raw const e_noinstext as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if c != NUL {
            stuffcharReadbuff(c);
        }
        if insert.size > 0 as size_t {
            let mut p: *mut ::core::ffi::c_char = insert
                .data
                .offset(insert.size as isize)
                .offset(-(1 as ::core::ffi::c_int as isize));
            while p >= insert.data {
                if *p as ::core::ffi::c_int == ESC {
                    insert.size = p.offset_from(insert.data) as size_t;
                    break;
                } else {
                    p = p.offset(-1);
                }
            }
        }
        if insert.size > 0 as size_t {
            let mut p_0: *mut ::core::ffi::c_char = insert
                .data
                .offset(insert.size as isize)
                .offset(-(1 as ::core::ffi::c_int as isize));
            if (*p_0 as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                || *p_0 as ::core::ffi::c_int == '^' as ::core::ffi::c_int)
                && (no_esc != 0
                    || *insert.data as ::core::ffi::c_int == Ctrl_D
                        && count > 1 as ::core::ffi::c_int)
            {
                last = *p_0;
                insert.size = insert.size.wrapping_sub(1);
            }
        }
        loop {
            stuffReadbuffLen(insert.data, insert.size as ptrdiff_t);
            match last as ::core::ffi::c_int {
                48 => {
                    stuffReadbuffLen(
                        b"\x16048\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                            as ptrdiff_t,
                    );
                }
                94 => {
                    stuffReadbuffLen(
                        b"\x16^\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                            as ptrdiff_t,
                    );
                }
                _ => {}
            }
            count -= 1;
            if count <= 0 as ::core::ffi::c_int {
                break;
            }
        }
        if no_esc == 0 {
            stuffcharReadbuff(ESC);
        }
        return OK;
    }
}

pub unsafe extern "C" fn get_last_insert() -> String_0 {
    unsafe {
        return if (*last_insert.ptr()).data.is_null() {
            NULL_STRING
        } else {
            String_0 {
                data: (*last_insert.ptr())
                    .data
                    .offset(last_insert_skip.get() as isize),
                size: (*last_insert.ptr())
                    .size
                    .wrapping_sub(last_insert_skip.get() as size_t),
            }
        };
    }
}

pub unsafe extern "C" fn get_last_insert_save() -> *mut ::core::ffi::c_char {
    unsafe {
        let mut insert: String_0 = get_last_insert();
        if insert.data.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut s: *mut ::core::ffi::c_char =
            xmemdupz(insert.data as *const ::core::ffi::c_void, insert.size)
                as *mut ::core::ffi::c_char;
        if insert.size > 0 as size_t
            && *s.offset(insert.size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == ESC
        {
            insert.size = insert.size.wrapping_sub(1);
            *s.offset(insert.size as isize) = NUL as ::core::ffi::c_char;
        }
        return s;
    }
}
