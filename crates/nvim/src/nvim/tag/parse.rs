//! Reading a tags file, one line at a time.
//!
//! A sorted tags file is searched by bisection and an unsorted one
//! linearly; [`findtags_start_state_handler`] chooses between them and
//! [`findtags_get_next_line`] is what actually reads, seeks and re-seeks.
//! [`findtags_parse_line`] splits a line into its tag name, file name and
//! search command, and [`findtags_match_tag`] decides whether that name is
//! the one being looked for.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn tag_strnicmp(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        while len > 0 as size_t {
            let mut i: ::core::ffi::c_int =
                (if (*s1 as uint8_t as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                    || *s1 as uint8_t as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
                {
                    *s1 as uint8_t as ::core::ffi::c_int
                } else {
                    *s1 as uint8_t as ::core::ffi::c_int
                        - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                }) - (if (*s2 as uint8_t as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                    || *s2 as uint8_t as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
                {
                    *s2 as uint8_t as ::core::ffi::c_int
                } else {
                    *s2 as uint8_t as ::core::ffi::c_int
                        - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                });
            if i != 0 as ::core::ffi::c_int {
                return i;
            }
            if *s1 as ::core::ffi::c_int == NUL {
                break;
            }
            s1 = s1.offset(1);
            s2 = s2.offset(1);
            len = len.wrapping_sub(1);
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn findtags_get_next_line(
    mut st: *mut findtags_state_T,
    mut sinfo_p: *mut tagsearch_info_T,
) -> tags_read_status_T {
    unsafe {
        let mut eof: bool = false;
        if (*st).state as ::core::ffi::c_uint
            == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut offset: off_T = (*sinfo_p).low_offset
                + ((*sinfo_p).high_offset - (*sinfo_p).low_offset) / 2 as off_T;
            if offset == (*sinfo_p).curr_offset {
                return TAGS_READ_EOF;
            } else {
                (*sinfo_p).curr_offset = offset;
            }
        } else if (*st).state as ::core::ffi::c_uint
            == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*sinfo_p).curr_offset -= ((*st).lbuf_size * 2 as ::core::ffi::c_int) as off_T;
            if (*sinfo_p).curr_offset < 0 as off_T {
                (*sinfo_p).curr_offset = 0 as off_T;
                fseek((*st).fp, 0 as ::core::ffi::c_long, SEEK_SET);
                (*st).state = TS_STEP_FORWARD;
            }
        }
        if (*st).state as ::core::ffi::c_uint
            == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*st).state as ::core::ffi::c_uint
                == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*sinfo_p).curr_offset_used = (*sinfo_p).curr_offset;
            vim_ignored.set(fseeko(
                (*st).fp,
                (*sinfo_p).curr_offset as __off_t,
                SEEK_SET,
            ));
            eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
            if !eof && (*sinfo_p).curr_offset != 0 as off_T {
                (*sinfo_p).curr_offset = ftello((*st).fp) as off_T;
                if (*sinfo_p).curr_offset == (*sinfo_p).high_offset {
                    vim_ignored.set(fseeko((*st).fp, (*sinfo_p).low_offset as __off_t, SEEK_SET));
                    (*sinfo_p).curr_offset = (*sinfo_p).low_offset;
                }
                eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
            }
            while !eof && vim_isblankline((*st).lbuf) as ::core::ffi::c_int != 0 {
                (*sinfo_p).curr_offset = ftello((*st).fp) as off_T;
                eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
            }
            if eof {
                (*st).state = TS_SKIP_BACK;
                (*sinfo_p).match_offset = ftello((*st).fp) as off_T;
                (*sinfo_p).curr_offset = (*sinfo_p).curr_offset_used;
                return TAGS_READ_IGNORE;
            }
        } else {
            loop {
                eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
                if !(!eof && vim_isblankline((*st).lbuf) as ::core::ffi::c_int != 0) {
                    break;
                }
            }
            if eof {
                return TAGS_READ_EOF;
            }
        }
        return TAGS_READ_SUCCESS;
    }
}

pub(crate) unsafe extern "C" fn findtags_hdr_parse(mut st: *mut findtags_state_T) -> bool {
    unsafe {
        if strncmp(
            (*st).lbuf,
            b"!_TAG_\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) != 0 as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
        if strncmp(
            (*st).lbuf,
            b"!_TAG_FILE_SORTED\t\0".as_ptr() as *const ::core::ffi::c_char,
            18 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            (*st).tag_file_sorted = *(*st).lbuf.offset(18 as ::core::ffi::c_int as isize) as uint8_t
                as ::core::ffi::c_int;
        }
        if strncmp(
            (*st).lbuf,
            b"!_TAG_FILE_ENCODING\t\0".as_ptr() as *const ::core::ffi::c_char,
            20 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            p = (*st).lbuf.offset(20 as ::core::ffi::c_int as isize);
            while *p as ::core::ffi::c_int > ' ' as ::core::ffi::c_int
                && (*p as ::core::ffi::c_int) < 127 as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            *p = NUL as ::core::ffi::c_char;
            convert_setup(
                &raw mut (*st).vimconv,
                (*st).lbuf.offset(20 as ::core::ffi::c_int as isize),
                p_enc.get(),
            );
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn findtags_start_state_handler(
    mut st: *mut findtags_state_T,
    mut sortic: *mut bool,
    mut sinfo_p: *mut tagsearch_info_T,
) -> bool {
    unsafe {
        let noic: bool = (*st).flags & TAG_NOIC as ::core::ffi::c_int != 0;
        if strncmp(
            (*st).lbuf,
            b"!_TAG_\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) <= 0 as ::core::ffi::c_int
            || *(*st).lbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '!' as ::core::ffi::c_int
                && (*(*st).lbuf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    >= 'a' as ::core::ffi::c_uint
                    && *(*st).lbuf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                        <= 'z' as ::core::ffi::c_uint)
        {
            return findtags_hdr_parse(st);
        }
        if (*st).linear {
            (*st).state = TS_LINEAR;
        } else if (*st).tag_file_sorted == NUL {
            (*st).state = TS_BINARY;
        } else if (*st).tag_file_sorted == '1' as ::core::ffi::c_int {
            (*st).state = TS_BINARY;
        } else if (*st).tag_file_sorted == '2' as ::core::ffi::c_int {
            (*st).state = TS_BINARY;
            *sortic = true_0 != 0;
            (*(*st).orgpat).regmatch.rm_ic = p_ic.get() != 0 || !noic;
        } else {
            (*st).state = TS_LINEAR;
        }
        if (*st).state as ::core::ffi::c_uint
            == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*(*st).orgpat).regmatch.rm_ic as ::core::ffi::c_int != 0
            && !*sortic
        {
            (*st).linear = true_0 != 0;
            (*st).state = TS_LINEAR;
        }
        if (*st).state as ::core::ffi::c_uint
            == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if fseeko((*st).fp, 0 as __off_t, SEEK_END) != 0 as ::core::ffi::c_int {
                (*st).state = TS_LINEAR;
            } else {
                let filesize: off_T = ftello((*st).fp);
                vim_ignored.set(fseeko((*st).fp, 0 as __off_t, SEEK_SET));
                (*sinfo_p).low_offset = 0 as off_T;
                (*sinfo_p).low_char = 0 as ::core::ffi::c_int;
                (*sinfo_p).high_offset = filesize;
                (*sinfo_p).curr_offset = 0 as off_T;
                (*sinfo_p).high_char = 0xff as ::core::ffi::c_int;
            }
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn findtags_parse_line(
    mut st: *mut findtags_state_T,
    mut tagpp: *mut tagptrs_T,
    mut margs: *mut findtags_match_args_T,
    mut sinfo_p: *mut tagsearch_info_T,
) -> tagmatch_status_T {
    unsafe {
        let mut status: ::core::ffi::c_int = 0;
        if (*(*st).orgpat).headlen != 0 {
            memset(
                tagpp as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<tagptrs_T>(),
            );
            (*tagpp).tagname = (*st).lbuf;
            (*tagpp).tagname_end = vim_strchr((*st).lbuf, TAB);
            if (*tagpp).tagname_end.is_null() {
                return TAG_MATCH_FAIL;
            }
            let mut cmplen: ::core::ffi::c_int =
                (*tagpp).tagname_end.offset_from((*tagpp).tagname) as ::core::ffi::c_int;
            if p_tl.get() != 0 as OptInt && cmplen as OptInt > p_tl.get() {
                cmplen = p_tl.get() as ::core::ffi::c_int;
            }
            if (*st).flags & TAG_REGEXP as ::core::ffi::c_int != 0
                && (*(*st).orgpat).headlen < cmplen
            {
                cmplen = (*(*st).orgpat).headlen;
            } else if (*st).state as ::core::ffi::c_uint
                == TS_LINEAR as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*(*st).orgpat).headlen != cmplen
            {
                return TAG_MATCH_NEXT;
            }
            if (*st).state as ::core::ffi::c_uint
                == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut tagcmp: ::core::ffi::c_int = 0;
                let mut i: ::core::ffi::c_int =
                    *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int;
                if (*margs).sortic {
                    i = if (*(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int)
                        < 'a' as ::core::ffi::c_int
                        || *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            > 'z' as ::core::ffi::c_int
                    {
                        *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                    } else {
                        *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    };
                }
                if i < (*sinfo_p).low_char || i > (*sinfo_p).high_char {
                    (*margs).sort_error = true_0 != 0;
                }
                if (*margs).sortic {
                    tagcmp = tag_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t);
                } else {
                    tagcmp = strncmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t);
                }
                if tagcmp == 0 as ::core::ffi::c_int {
                    if cmplen < (*(*st).orgpat).headlen {
                        tagcmp = -1 as ::core::ffi::c_int;
                    } else if cmplen > (*(*st).orgpat).headlen {
                        tagcmp = 1 as ::core::ffi::c_int;
                    }
                }
                if tagcmp == 0 as ::core::ffi::c_int {
                    (*st).state = TS_SKIP_BACK;
                    (*sinfo_p).match_offset = (*sinfo_p).curr_offset;
                    return TAG_MATCH_NEXT;
                }
                if tagcmp < 0 as ::core::ffi::c_int {
                    (*sinfo_p).curr_offset = ftello((*st).fp) as off_T;
                    if (*sinfo_p).curr_offset < (*sinfo_p).high_offset {
                        (*sinfo_p).low_offset = (*sinfo_p).curr_offset;
                        if (*margs).sortic {
                            (*sinfo_p).low_char =
                                if (*(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    < 'a' as ::core::ffi::c_int
                                    || *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        > 'z' as ::core::ffi::c_int
                                {
                                    *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                } else {
                                    *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                                };
                        } else {
                            (*sinfo_p).low_char =
                                *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as uint8_t
                                    as ::core::ffi::c_int;
                        }
                        return TAG_MATCH_NEXT;
                    }
                }
                if tagcmp > 0 as ::core::ffi::c_int
                    && (*sinfo_p).curr_offset != (*sinfo_p).high_offset
                {
                    (*sinfo_p).high_offset = (*sinfo_p).curr_offset;
                    if (*margs).sortic {
                        (*sinfo_p).high_char =
                            if (*(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int)
                                < 'a' as ::core::ffi::c_int
                                || *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    > 'z' as ::core::ffi::c_int
                            {
                                *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                            } else {
                                *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                            };
                    } else {
                        (*sinfo_p).high_char =
                            *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int;
                    }
                    return TAG_MATCH_NEXT;
                }
                return TAG_MATCH_STOP;
            } else if (*st).state as ::core::ffi::c_uint
                == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                '_c2rust_label: {
                    if cmplen >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                        b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tag.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1797 as ::core::ffi::c_uint,
                        b"tagmatch_status_T findtags_parse_line(findtags_state_T *, tagptrs_T *, findtags_match_args_T *, tagsearch_info_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                if mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t)
                    != 0 as ::core::ffi::c_int
                {
                    (*st).state = TS_STEP_FORWARD;
                } else {
                    (*sinfo_p).curr_offset = (*sinfo_p).curr_offset_used;
                }
                return TAG_MATCH_NEXT;
            } else if (*st).state as ::core::ffi::c_uint
                == TS_STEP_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                '_c2rust_label_0: {
                    if cmplen >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                        b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tag.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1807 as ::core::ffi::c_uint,
                        b"tagmatch_status_T findtags_parse_line(findtags_state_T *, tagptrs_T *, findtags_match_args_T *, tagsearch_info_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                if mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t)
                    != 0 as ::core::ffi::c_int
                {
                    return (if ftello((*st).fp) > (*sinfo_p).match_offset {
                        TAG_MATCH_STOP as ::core::ffi::c_int
                    } else {
                        TAG_MATCH_NEXT as ::core::ffi::c_int
                    }) as tagmatch_status_T;
                }
            } else {
                '_c2rust_label_1: {
                    if cmplen >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                        b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tag.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1815 as ::core::ffi::c_uint,
                        b"tagmatch_status_T findtags_parse_line(findtags_state_T *, tagptrs_T *, findtags_match_args_T *, tagsearch_info_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                if mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t)
                    != 0 as ::core::ffi::c_int
                {
                    return TAG_MATCH_NEXT;
                }
            }
            (*tagpp).fname = (*tagpp)
                .tagname_end
                .offset(1 as ::core::ffi::c_int as isize);
            (*tagpp).fname_end = vim_strchr((*tagpp).fname, TAB);
            if (*tagpp).fname_end.is_null() {
                status = FAIL;
            } else {
                (*tagpp).command = (*tagpp).fname_end.offset(1 as ::core::ffi::c_int as isize);
                status = OK;
            }
        } else {
            status = parse_tag_line((*st).lbuf, tagpp);
        }
        return (if status == FAIL {
            TAG_MATCH_FAIL as ::core::ffi::c_int
        } else {
            TAG_MATCH_SUCCESS as ::core::ffi::c_int
        }) as tagmatch_status_T;
    }
}

pub(crate) unsafe extern "C" fn findtags_match_tag(
    mut st: *mut findtags_state_T,
    mut tagpp: *mut tagptrs_T,
    mut margs: *mut findtags_match_args_T,
) -> bool {
    unsafe {
        let mut match_0: bool = false_0 != 0;
        let mut cmplen: ::core::ffi::c_int =
            (*tagpp).tagname_end.offset_from((*tagpp).tagname) as ::core::ffi::c_int;
        if p_tl.get() != 0 as OptInt && cmplen as OptInt > p_tl.get() {
            cmplen = p_tl.get() as ::core::ffi::c_int;
        }
        if (*(*st).orgpat).len != cmplen {
            match_0 = false_0 != 0;
        } else if (*(*st).orgpat).regmatch.rm_ic {
            '_c2rust_label: {
                if cmplen >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                    b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/tag.rs\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1869 as ::core::ffi::c_uint,
                    b"_Bool findtags_match_tag(findtags_state_T *, tagptrs_T *, findtags_match_args_T *)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
                }
            };
            match_0 = mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).pat, cmplen as size_t)
                == 0 as ::core::ffi::c_int;
            if match_0 {
                (*margs).match_no_ic =
                    strncmp((*tagpp).tagname, (*(*st).orgpat).pat, cmplen as size_t)
                        == 0 as ::core::ffi::c_int;
            }
        } else {
            match_0 = strncmp((*tagpp).tagname, (*(*st).orgpat).pat, cmplen as size_t)
                == 0 as ::core::ffi::c_int;
        }
        (*margs).match_re = false_0 != 0;
        if !match_0 && !(*(*st).orgpat).regmatch.regprog.is_null() {
            let mut cc: ::core::ffi::c_char = *(*tagpp).tagname_end;
            *(*tagpp).tagname_end = NUL as ::core::ffi::c_char;
            match_0 = vim_regexec(
                &raw mut (*(*st).orgpat).regmatch,
                (*tagpp).tagname,
                0 as colnr_T,
            );
            if match_0 {
                (*margs).matchoff =
                    (*(*st).orgpat).regmatch.startp[0 as ::core::ffi::c_int as usize]
                        .offset_from((*tagpp).tagname) as ::core::ffi::c_int;
                if (*(*st).orgpat).regmatch.rm_ic {
                    (*(*st).orgpat).regmatch.rm_ic = false_0 != 0;
                    (*margs).match_no_ic = vim_regexec(
                        &raw mut (*(*st).orgpat).regmatch,
                        (*tagpp).tagname,
                        0 as colnr_T,
                    );
                    (*(*st).orgpat).regmatch.rm_ic = true_0 != 0;
                }
            }
            *(*tagpp).tagname_end = cc;
            (*margs).match_re = true_0 != 0;
        }
        return match_0;
    }
}
