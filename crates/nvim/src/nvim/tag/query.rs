//! The Vimscript and completion views of a tag.
//!
//! [`get_tags`] is `taglist()`: every match as a dictionary, built up by
//! [`add_tag_field`] and [`get_tag_details`]. [`expand_tags`] is the
//! command-line completion of tag names.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn expand_tags(
    mut tagnames: bool,
    mut pat: *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut name_buf_size: size_t = 100 as size_t;
        let mut ret: ::core::ffi::c_int = 0;
        let mut name_buf: *mut ::core::ffi::c_char =
            xmalloc(name_buf_size) as *mut ::core::ffi::c_char;
        let mut extra_flag: ::core::ffi::c_int = if tagnames as ::core::ffi::c_int != 0 {
            TAG_NAMES as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        if *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '/' as ::core::ffi::c_int
        {
            ret = find_tags(
                pat.offset(1 as ::core::ffi::c_int as isize),
                num_file,
                file,
                TAG_REGEXP as ::core::ffi::c_int
                    | extra_flag
                    | TAG_VERBOSE as ::core::ffi::c_int
                    | TAG_NO_TAGFUNC as ::core::ffi::c_int,
                TAG_MANY as ::core::ffi::c_int,
                (*curbuf.get()).b_ffname,
            );
        } else {
            ret = find_tags(
                pat,
                num_file,
                file,
                TAG_REGEXP as ::core::ffi::c_int
                    | extra_flag
                    | TAG_VERBOSE as ::core::ffi::c_int
                    | TAG_NO_TAGFUNC as ::core::ffi::c_int
                    | TAG_NOIC as ::core::ffi::c_int,
                TAG_MANY as ::core::ffi::c_int,
                (*curbuf.get()).b_ffname,
            );
        }
        if ret == OK && !tagnames {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < *num_file {
                let mut t_p: TagParts = TagParts::default();
                parse_match(*(*file).offset(i as isize), &mut t_p);
                let mut len: size_t = t_p.tagname_end.offset_from(t_p.tagname) as size_t;
                if len > name_buf_size.wrapping_sub(3 as size_t) {
                    name_buf_size = len.wrapping_add(3 as size_t);
                    let mut buf: *mut ::core::ffi::c_char =
                        xrealloc(name_buf as *mut ::core::ffi::c_void, name_buf_size)
                            as *mut ::core::ffi::c_char;
                    name_buf = buf;
                }
                memmove(
                    name_buf as *mut ::core::ffi::c_void,
                    t_p.tagname as *const ::core::ffi::c_void,
                    len,
                );
                let c2rust_fresh14 = len;
                len = len.wrapping_add(1);
                *name_buf.offset(c2rust_fresh14 as isize) = 0 as ::core::ffi::c_char;
                let c2rust_fresh15 = len;
                len = len.wrapping_add(1);
                *name_buf.offset(c2rust_fresh15 as isize) =
                    (if !t_p.tagkind.is_null() && *t_p.tagkind as ::core::ffi::c_int != 0 {
                        *t_p.tagkind as ::core::ffi::c_int
                    } else {
                        'f' as ::core::ffi::c_int
                    }) as ::core::ffi::c_char;
                let c2rust_fresh16 = len;
                len = len.wrapping_add(1);
                *name_buf.offset(c2rust_fresh16 as isize) = 0 as ::core::ffi::c_char;
                memmove(
                    (*(*file).offset(i as isize)).offset(len as isize) as *mut ::core::ffi::c_void,
                    t_p.fname as *const ::core::ffi::c_void,
                    t_p.fname_end.offset_from(t_p.fname) as size_t,
                );
                *(*(*file).offset(i as isize)).offset(
                    len.wrapping_add(t_p.fname_end.offset_from(t_p.fname) as size_t) as isize,
                ) = 0 as ::core::ffi::c_char;
                memmove(
                    *(*file).offset(i as isize) as *mut ::core::ffi::c_void,
                    name_buf as *const ::core::ffi::c_void,
                    len,
                );
                i += 1;
            }
        }
        xfree(name_buf as *mut ::core::ffi::c_void);
        return ret;
    }
}

pub(crate) unsafe extern "C" fn add_tag_field(
    mut dict: *mut dict_T,
    mut field_name: *const ::core::ffi::c_char,
    mut start: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if !tv_dict_find(dict, field_name, -1 as ptrdiff_t).is_null() {
            if p_verbose.get() > 0 as OptInt {
                verbose_enter();
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Duplicate field name: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    field_name,
                );
                verbose_leave();
            }
            return FAIL;
        }
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut buf: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        if !start.is_null() {
            if end.is_null() {
                end = start.offset(strlen(start) as isize);
                while end > start
                    && (*end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\r' as ::core::ffi::c_int
                        || *end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\n' as ::core::ffi::c_int)
                {
                    end = end.offset(-1);
                }
            }
            len = if (end.offset_from(start) as ::core::ffi::c_int)
                < 4096 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            {
                end.offset_from(start) as ::core::ffi::c_int
            } else {
                4096 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            };
            xmemcpyz(
                buf as *mut ::core::ffi::c_void,
                start as *const ::core::ffi::c_void,
                len as size_t,
            );
        }
        *buf.offset(len as isize) = NUL as ::core::ffi::c_char;
        let mut retval: ::core::ffi::c_int =
            tv_dict_add_str(dict, field_name, strlen(field_name), buf);
        xfree(buf as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub unsafe extern "C" fn get_tags(
    mut list: *mut list_T,
    mut pat: *mut ::core::ffi::c_char,
    mut buf_fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut num_matches: ::core::ffi::c_int = 0;
        let mut matches: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut tp: TagParts = TagParts::default();
        let mut ret: ::core::ffi::c_int = find_tags(
            pat,
            &raw mut num_matches,
            &raw mut matches,
            TAG_REGEXP as ::core::ffi::c_int | TAG_NOIC as ::core::ffi::c_int,
            MAXCOL as ::core::ffi::c_int,
            buf_fname,
        );
        if ret != OK || num_matches <= 0 as ::core::ffi::c_int {
            return ret;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_matches {
            if !parse_match(*matches.offset(i as isize), &mut tp) {
                xfree(*matches.offset(i as isize) as *mut ::core::ffi::c_void);
            } else {
                let mut is_static: bool = test_for_static(&mut tp);
                if strncmp(
                    tp.tagname,
                    b"!_TAG_\0".as_ptr() as *const ::core::ffi::c_char,
                    6 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    xfree(*matches.offset(i as isize) as *mut ::core::ffi::c_void);
                } else {
                    let mut dict: *mut dict_T = tv_dict_alloc();
                    tv_list_append_dict(list, dict);
                    let mut full_fname: *mut ::core::ffi::c_char = tag_full_fname(&mut tp);
                    if add_tag_field(
                        dict,
                        b"name\0".as_ptr() as *const ::core::ffi::c_char,
                        tp.tagname,
                        tp.tagname_end,
                    ) == FAIL
                        || add_tag_field(
                            dict,
                            b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                            full_fname,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        ) == FAIL
                        || add_tag_field(
                            dict,
                            b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                            tp.command,
                            tp.command_end,
                        ) == FAIL
                        || add_tag_field(
                            dict,
                            b"kind\0".as_ptr() as *const ::core::ffi::c_char,
                            tp.tagkind,
                            if !tp.tagkind.is_null() {
                                tp.tagkind_end
                            } else {
                                ::core::ptr::null_mut::<::core::ffi::c_char>()
                            },
                        ) == FAIL
                        || tv_dict_add_nr(
                            dict,
                            b"static\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                .wrapping_sub(1 as size_t),
                            is_static as varnumber_T,
                        ) == FAIL
                    {
                        ret = FAIL;
                    }
                    xfree(full_fname as *mut ::core::ffi::c_void);
                    if !tp.command_end.is_null() {
                        let mut p: *mut ::core::ffi::c_char =
                            tp.command_end.offset(3 as ::core::ffi::c_int as isize);
                        while *p as ::core::ffi::c_int != NUL
                            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                            && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                        {
                            if p == tp.tagkind
                                || p.offset(5 as ::core::ffi::c_int as isize) == tp.tagkind
                                    && strncmp(
                                        p,
                                        b"kind:\0".as_ptr() as *const ::core::ffi::c_char,
                                        5 as size_t,
                                    ) == 0 as ::core::ffi::c_int
                            {
                                p = tp.tagkind_end.offset(-(1 as ::core::ffi::c_int as isize));
                            } else if strncmp(
                                p,
                                b"file:\0".as_ptr() as *const ::core::ffi::c_char,
                                5 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            {
                                p = p.offset(4 as ::core::ffi::c_int as isize);
                            } else if !ascii_iswhite(*p as ::core::ffi::c_int) {
                                let mut len: ::core::ffi::c_int = 0;
                                let mut n: *mut ::core::ffi::c_char = p;
                                while *p as ::core::ffi::c_int != NUL
                                    && *p as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
                                    && (*p as ::core::ffi::c_int) < 127 as ::core::ffi::c_int
                                    && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                                {
                                    p = p.offset(1);
                                }
                                len = p.offset_from(n) as ::core::ffi::c_int;
                                if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                                    && len > 0 as ::core::ffi::c_int
                                {
                                    p = p.offset(1);
                                    let mut s: *mut ::core::ffi::c_char = p;
                                    while *p as ::core::ffi::c_int != NUL
                                        && *p as uint8_t as ::core::ffi::c_int
                                            >= ' ' as ::core::ffi::c_int
                                    {
                                        p = p.offset(1);
                                    }
                                    *n.offset(len as isize) = NUL as ::core::ffi::c_char;
                                    if add_tag_field(dict, n, s, p) == FAIL {
                                        ret = FAIL;
                                    }
                                    *n.offset(len as isize) = ':' as ::core::ffi::c_char;
                                } else {
                                    while *p as ::core::ffi::c_int != NUL
                                        && *p as uint8_t as ::core::ffi::c_int
                                            >= ' ' as ::core::ffi::c_int
                                    {
                                        p = p.offset(1);
                                    }
                                }
                                if *p as ::core::ffi::c_int == NUL {
                                    break;
                                }
                            }
                            p = p.offset(utfc_ptr2len(p) as isize);
                        }
                    }
                    xfree(*matches.offset(i as isize) as *mut ::core::ffi::c_void);
                }
            }
            i += 1;
        }
        xfree(matches as *mut ::core::ffi::c_void);
        return ret;
    }
}
