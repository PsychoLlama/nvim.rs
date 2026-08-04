//! `'langmap'`: a second keyboard layout for command keys.
//!
//! `langmap_mapchar` maps the 256 single-byte characters and
//! `langmap_mapga` is a sorted table doing the same for everything above;
//! [`did_set_langmap`] parses the option into both, and
//! [`langmap_adjust_mb`] is the multi-byte lookup.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn langmap_set_entry(
    mut from: ::core::ffi::c_int,
    mut to: ::core::ffi::c_int,
) {
    unsafe {
        let mut entries: *mut langmap_entry_T =
            (*langmap_mapga.ptr()).ga_data as *mut langmap_entry_T;
        let mut a: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
        '_c2rust_label: {
            if (*langmap_mapga.ptr()).ga_len >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"langmap_mapga.ga_len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/mapping.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2496 as ::core::ffi::c_uint,
                    b"void langmap_set_entry(int, int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut b: ::core::ffi::c_uint = (*langmap_mapga.ptr()).ga_len as ::core::ffi::c_uint;
        while a != b {
            let mut i: ::core::ffi::c_uint =
                a.wrapping_add(b).wrapping_div(2 as ::core::ffi::c_uint);
            let mut d: ::core::ffi::c_int = (*entries.offset(i as isize)).from - from;
            if d == 0 as ::core::ffi::c_int {
                (*entries.offset(i as isize)).to = to;
                return;
            }
            if d < 0 as ::core::ffi::c_int {
                a = i.wrapping_add(1 as ::core::ffi::c_uint);
            } else {
                b = i;
            }
        }
        ga_grow(langmap_mapga.ptr(), 1 as ::core::ffi::c_int);
        entries = ((*langmap_mapga.ptr()).ga_data as *mut langmap_entry_T).offset(a as isize);
        memmove(
            entries.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            entries as *const ::core::ffi::c_void,
            (((*langmap_mapga.ptr()).ga_len as ::core::ffi::c_uint).wrapping_sub(a) as size_t)
                .wrapping_mul(::core::mem::size_of::<langmap_entry_T>()),
        );
        (*langmap_mapga.ptr()).ga_len += 1;
        (*entries.offset(0 as ::core::ffi::c_int as isize)).from = from;
        (*entries.offset(0 as ::core::ffi::c_int as isize)).to = to;
    }
}

pub unsafe extern "C" fn langmap_adjust_mb(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut entries: *mut langmap_entry_T =
            (*langmap_mapga.ptr()).ga_data as *mut langmap_entry_T;
        let mut a: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut b: ::core::ffi::c_int = (*langmap_mapga.ptr()).ga_len;
        while a != b {
            let mut i: ::core::ffi::c_int = (a + b) / 2 as ::core::ffi::c_int;
            let mut d: ::core::ffi::c_int = (*entries.offset(i as isize)).from - c;
            if d == 0 as ::core::ffi::c_int {
                return (*entries.offset(i as isize)).to;
            }
            if d < 0 as ::core::ffi::c_int {
                a = i + 1 as ::core::ffi::c_int;
            } else {
                b = i;
            }
        }
        return c;
    }
}

pub unsafe extern "C" fn langmap_init() {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 256 as ::core::ffi::c_int {
            (*langmap_mapchar.ptr())[i as usize] = i as uint8_t;
            i += 1;
        }
        ga_init(
            langmap_mapga.ptr(),
            ::core::mem::size_of::<langmap_entry_T>() as ::core::ffi::c_int,
            8 as ::core::ffi::c_int,
        );
    }
}

pub unsafe extern "C" fn did_set_langmap(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    unsafe {
        ga_clear(langmap_mapga.ptr());
        langmap_init();
        let mut p: *mut ::core::ffi::c_char = p_langmap.get();
        while *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
            let mut p2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            p2 = p;
            while *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ',' as ::core::ffi::c_int
                && *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ';' as ::core::ffi::c_int
            {
                if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *p2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    p2 = p2.offset(1);
                }
                p2 = p2.offset(utfc_ptr2len(p2) as isize);
            }
            if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ';' as ::core::ffi::c_int
            {
                p2 = p2.offset(1);
            } else {
                p2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            while *p.offset(0 as ::core::ffi::c_int as isize) != 0 {
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ',' as ::core::ffi::c_int
                {
                    p = p.offset(1);
                    break;
                } else {
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                    {
                        p = p.offset(1);
                    }
                    let mut from: ::core::ffi::c_int = utf_ptr2char(p);
                    let from_ptr: *const ::core::ffi::c_char = p;
                    let mut to: ::core::ffi::c_int = NUL;
                    let mut to_ptr: *const ::core::ffi::c_char =
                        b"\0".as_ptr() as *const ::core::ffi::c_char;
                    if p2.is_null() {
                        p = p.offset(utfc_ptr2len(p) as isize);
                        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != ',' as ::core::ffi::c_int
                        {
                            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '\\' as ::core::ffi::c_int
                            {
                                p = p.offset(1);
                            }
                            to_ptr = p;
                            to = utf_ptr2char(to_ptr);
                        }
                    } else if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ',' as ::core::ffi::c_int
                    {
                        if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        {
                            p2 = p2.offset(1);
                        }
                        to_ptr = p2;
                        to = utf_ptr2char(to_ptr);
                    }
                    if to == NUL {
                        snprintf(
                            (*args).os_errbuf,
                            (*args).os_errbuflen,
                            gettext(
                                b"E357: 'langmap': Matching character missing for %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ),
                            transchar(from),
                        );
                        return (*args).os_errbuf;
                    }
                    if from >= 256 as ::core::ffi::c_int {
                        langmap_set_entry(from, to);
                    } else {
                        if to > UCHAR_MAX {
                            swmsg(
                                true_0 != 0,
                                b"'langmap': Mapping from %.*s to %.*s will not work properly\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                                utf_ptr2len(from_ptr),
                                from_ptr,
                                utf_ptr2len(to_ptr),
                                to_ptr,
                            );
                        }
                        (*langmap_mapchar.ptr())[(from & 255 as ::core::ffi::c_int) as usize] =
                            to as uint8_t;
                    }
                    p = p.offset(utfc_ptr2len(p) as isize);
                    if p2.is_null() {
                        continue;
                    }
                    p2 = p2.offset(utfc_ptr2len(p2) as isize);
                    if *p as ::core::ffi::c_int != ';' as ::core::ffi::c_int {
                        continue;
                    }
                    p = p2;
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != ',' as ::core::ffi::c_int
                        {
                            snprintf(
                                (*args).os_errbuf,
                                (*args).os_errbuflen,
                                gettext(
                                    b"E358: 'langmap': Extra characters after semicolon: %s\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                ),
                                p,
                            );
                            return (*args).os_errbuf;
                        }
                        p = p.offset(1);
                    }
                    break;
                }
            }
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}
