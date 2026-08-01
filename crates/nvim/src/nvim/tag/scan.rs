//! Scanning the tags files for matches.
//!
//! [`find_tags`] is the entry point every tag lookup goes through: it
//! collects the tags files that apply ([`get_tagfname`](super::get_tagfname)),
//! reads each one, and hands back the matching lines sorted by how good a
//! match they are. The state it threads through the readers is
//! `findtags_state_T`; [`prepare_pats`] turns the caller's pattern into the
//! regexp and the plain prefix the readers compare against.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn prepare_pats(mut pats: *mut pat_T, mut has_re: bool) {
    unsafe {
        (*pats).head = (*pats).pat;
        (*pats).headlen = (*pats).len;
        if has_re {
            if *(*pats).pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '^' as ::core::ffi::c_int
            {
                (*pats).head = (*pats).pat.offset(1 as ::core::ffi::c_int as isize);
            } else if *(*pats).pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *(*pats).pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '<' as ::core::ffi::c_int
            {
                (*pats).head = (*pats).pat.offset(2 as ::core::ffi::c_int as isize);
            }
            if (*pats).head == (*pats).pat {
                (*pats).headlen = 0 as ::core::ffi::c_int;
            } else {
                (*pats).headlen = 0 as ::core::ffi::c_int;
                while *(*pats).head.offset((*pats).headlen as isize) as ::core::ffi::c_int != NUL {
                    if !vim_strchr(
                        if magic_isset() as ::core::ffi::c_int != 0 {
                            b".[~*\\$\0".as_ptr() as *const ::core::ffi::c_char
                        } else {
                            b"\\$\0".as_ptr() as *const ::core::ffi::c_char
                        },
                        *(*pats).head.offset((*pats).headlen as isize) as uint8_t
                            as ::core::ffi::c_int,
                    )
                    .is_null()
                    {
                        break;
                    }
                    (*pats).headlen += 1;
                }
            }
            if p_tl.get() != 0 as OptInt && (*pats).headlen as OptInt > p_tl.get() {
                (*pats).headlen = p_tl.get() as ::core::ffi::c_int;
            }
        }
        if has_re {
            (*pats).regmatch.regprog = vim_regcomp(
                (*pats).pat,
                if magic_isset() as ::core::ffi::c_int != 0 {
                    RE_MAGIC
                } else {
                    0 as ::core::ffi::c_int
                },
            );
        } else {
            (*pats).regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
        };
    }
}

pub(crate) unsafe extern "C" fn findtags_state_init(
    mut st: *mut findtags_state_T,
    mut pat: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut mincount: ::core::ffi::c_int,
) {
    unsafe {
        (*st).tag_fname =
            xmalloc((MAXPATHL + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
        (*st).fp = ::core::ptr::null_mut::<FILE>();
        (*st).orgpat = xmalloc(::core::mem::size_of::<pat_T>()) as *mut pat_T;
        (*(*st).orgpat).pat = pat;
        (*(*st).orgpat).len = strlen(pat) as ::core::ffi::c_int;
        (*(*st).orgpat).regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
        (*st).flags = flags;
        (*st).tag_file_sorted = NUL;
        (*st).help_lang_find = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*st).is_txt = false_0 != 0;
        (*st).did_open = false_0 != 0;
        (*st).help_only = flags & TAG_HELP as ::core::ffi::c_int != 0;
        (*st).get_searchpat = false_0 != 0;
        (*st).help_lang[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        (*st).help_pri = 0 as ::core::ffi::c_int;
        (*st).mincount = mincount;
        (*st).lbuf_size = LSIZE as ::core::ffi::c_int;
        (*st).lbuf = xmalloc((*st).lbuf_size as size_t) as *mut ::core::ffi::c_char;
        (*st).match_count = 0 as ::core::ffi::c_int;
        (*st).stop_searching = false_0 != 0;
        let mut mtt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while mtt < MT_COUNT as ::core::ffi::c_int {
            ga_init(
                (&raw mut (*st).ga_match as *mut garray_T).offset(mtt as isize),
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                100 as ::core::ffi::c_int,
            );
            hash_init((&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize));
            mtt += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn findtags_state_free(mut st: *mut findtags_state_T) {
    unsafe {
        xfree((*st).tag_fname as *mut ::core::ffi::c_void);
        xfree((*st).lbuf as *mut ::core::ffi::c_void);
        vim_regfree((*(*st).orgpat).regmatch.regprog);
        xfree((*st).orgpat as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn findtags_in_help_init(mut st: *mut findtags_state_T) -> bool {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        if (*st).is_txt {
            strcpy(
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        } else {
            i = strlen((*st).tag_fname) as ::core::ffi::c_int;
            if i > 3 as ::core::ffi::c_int
                && *(*st)
                    .tag_fname
                    .offset((i - 3 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    == '-' as ::core::ffi::c_int
            {
                xmemcpyz(
                    &raw mut (*st).help_lang as *mut ::core::ffi::c_char
                        as *mut ::core::ffi::c_void,
                    (*st)
                        .tag_fname
                        .offset(i as isize)
                        .offset(-(2 as ::core::ffi::c_int as isize))
                        as *const ::core::ffi::c_void,
                    2 as size_t,
                );
            } else {
                strcpy(
                    &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                    b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            }
        }
        if !(*st).help_lang_find.is_null()
            && strcasecmp(
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                (*st).help_lang_find,
            ) != 0 as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        if (*st).flags & TAG_KEEP_LANG as ::core::ffi::c_int != 0
            && (*st).help_lang_find.is_null()
            && !(*curbuf.get()).b_fname.is_null()
            && {
                i = strlen((*curbuf.get()).b_fname) as ::core::ffi::c_int;
                i > 4 as ::core::ffi::c_int
            }
            && *(*curbuf.get())
                .b_fname
                .offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == 'x' as ::core::ffi::c_int
            && *(*curbuf.get())
                .b_fname
                .offset((i - 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
            && strncasecmp(
                (*curbuf.get())
                    .b_fname
                    .offset(i as isize)
                    .offset(-(3 as ::core::ffi::c_int as isize)),
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            (*st).help_pri = 0 as ::core::ffi::c_int;
        } else {
            (*st).help_pri = 1 as ::core::ffi::c_int;
            let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            s = p_hlg.get();
            while *s as ::core::ffi::c_int != NUL {
                if strncasecmp(
                    s,
                    &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    break;
                }
                (*st).help_pri += 1;
                s = vim_strchr(s, ',' as ::core::ffi::c_int);
                if s.is_null() {
                    break;
                }
                s = s.offset(1);
            }
            if s.is_null() || *s as ::core::ffi::c_int == NUL {
                (*st).help_pri += 1;
                if strcasecmp(
                    &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                    b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ) != 0 as ::core::ffi::c_int
                {
                    (*st).help_pri += 1;
                }
            }
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn findtags_apply_tfu(
    mut st: *mut findtags_state_T,
    mut pat: *mut ::core::ffi::c_char,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let use_tfu: bool =
            (*st).flags & TAG_NO_TAGFUNC as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
        if !use_tfu
            || tfu_in_use.get() as ::core::ffi::c_int != 0
            || *(*curbuf.get()).b_p_tfu as ::core::ffi::c_int == NUL
        {
            return NOTDONE;
        }
        tfu_in_use.set(true_0 != 0);
        let mut retval: ::core::ffi::c_int = find_tagfunc_tags(
            pat,
            &raw mut (*st).ga_match as *mut garray_T,
            &raw mut (*st).match_count,
            (*st).flags,
            buf_ffname,
        );
        tfu_in_use.set(false_0 != 0);
        return retval;
    }
}

pub(crate) unsafe extern "C" fn findtags_get_all_tags(
    mut st: *mut findtags_state_T,
    mut margs: *mut findtags_match_args_T,
    mut buf_ffname: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut tagp: tagptrs_T = tagptrs_T {
            tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tagline: 0,
        };
        let mut search_info: tagsearch_info_T = tagsearch_info_T {
            low_offset: 0,
            high_offset: 0,
            curr_offset: 0,
            curr_offset_used: 0,
            match_offset: 0,
            low_char: 0,
            high_char: 0,
        };
        let mut hash: hash_T = 0 as hash_T;
        memset(
            &raw mut search_info as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<tagsearch_info_T>(),
        );
        let mut retval: ::core::ffi::c_int = 0;
        loop {
            if (*st).state as ::core::ffi::c_uint
                == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*st).state as ::core::ffi::c_uint
                    == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                line_breakcheck();
            } else {
                fast_breakcheck();
            }
            if (*st).flags & TAG_INS_COMP as ::core::ffi::c_int != 0 {
                ins_compl_check_keys(30 as ::core::ffi::c_int, false_0 != 0);
            }
            if got_int.get() as ::core::ffi::c_int != 0
                || ins_compl_interrupted() as ::core::ffi::c_int != 0
            {
                (*st).stop_searching = true_0 != 0;
                break;
            } else if (*st).mincount == TAG_MANY as ::core::ffi::c_int
                && (*st).match_count >= TAG_MANY as ::core::ffi::c_int
            {
                (*st).stop_searching = true_0 != 0;
                break;
            } else {
                if !(*st).get_searchpat {
                    retval = findtags_get_next_line(st, &raw mut search_info) as ::core::ffi::c_int;
                    if retval == TAGS_READ_IGNORE as ::core::ffi::c_int {
                        continue;
                    }
                    if retval == TAGS_READ_EOF as ::core::ffi::c_int {
                        break;
                    }
                }
                if (*st).vimconv.vc_type != CONV_NONE as ::core::ffi::c_int {
                    findtags_string_convert(st);
                }
                if (*st).state as ::core::ffi::c_uint
                    == TS_START as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if !findtags_start_state_handler(
                        st,
                        &raw mut (*margs).sortic,
                        &raw mut search_info,
                    ) {
                        continue;
                    }
                }
                if *(*st)
                    .lbuf
                    .offset(((*st).lbuf_size - 2 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    != NUL
                {
                    (*st).lbuf_size *= 2 as ::core::ffi::c_int;
                    xfree((*st).lbuf as *mut ::core::ffi::c_void);
                    (*st).lbuf = xmalloc((*st).lbuf_size as size_t) as *mut ::core::ffi::c_char;
                    if (*st).state as ::core::ffi::c_uint
                        == TS_STEP_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*st).state as ::core::ffi::c_uint
                            == TS_LINEAR as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        vim_ignored.set(fseeko(
                            (*st).fp,
                            search_info.curr_offset as __off_t,
                            SEEK_SET,
                        ));
                    }
                    search_info.curr_offset = 0 as off_T;
                } else {
                    retval = findtags_parse_line(st, &raw mut tagp, margs, &raw mut search_info)
                        as ::core::ffi::c_int;
                    if retval == TAG_MATCH_NEXT as ::core::ffi::c_int {
                        continue;
                    }
                    if retval == TAG_MATCH_STOP as ::core::ffi::c_int {
                        break;
                    }
                    if retval == TAG_MATCH_FAIL as ::core::ffi::c_int {
                        semsg(
                            gettext(b"E431: Format error in tags file \"%s\"\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            (*st).tag_fname,
                        );
                        semsg(
                            gettext(b"Before byte %ld\0".as_ptr() as *const ::core::ffi::c_char),
                            ftello((*st).fp) as int64_t,
                        );
                        (*st).stop_searching = true_0 != 0;
                        return;
                    }
                    if findtags_match_tag(st, &raw mut tagp, margs) {
                        findtags_add_match(st, &raw mut tagp, margs, buf_ffname, &raw mut hash);
                    }
                }
            }
        }
    }
}

pub(crate) unsafe extern "C" fn findtags_in_file(
    mut st: *mut findtags_state_T,
    mut _flags: ::core::ffi::c_int,
    mut buf_ffname: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut margs: findtags_match_args_T = findtags_match_args_T {
            matchoff: 0,
            match_re: false,
            match_no_ic: false,
            has_re: false,
            sortic: false,
            sort_error: false,
        };
        (*st).vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
        (*st).tag_file_sorted = NUL;
        (*st).fp = ::core::ptr::null_mut::<FILE>();
        findtags_matchargs_init(&raw mut margs, (*st).flags);
        if (*curbuf.get()).b_help {
            if !findtags_in_help_init(st) {
                return;
            }
        }
        (*st).fp = os_fopen(
            (*st).tag_fname,
            b"r\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if (*st).fp.is_null() {
            return;
        }
        if p_verbose.get() >= 5 as OptInt {
            verbose_enter();
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Searching tags file %s\0".as_ptr() as *const ::core::ffi::c_char),
                (*st).tag_fname,
            );
            verbose_leave();
        }
        (*st).did_open = true_0 != 0;
        (*st).state = TS_START;
        findtags_get_all_tags(st, &raw mut margs, buf_ffname);
        if !(*st).fp.is_null() {
            fclose((*st).fp);
            (*st).fp = ::core::ptr::null_mut::<FILE>();
        }
        if (*st).vimconv.vc_type != CONV_NONE as ::core::ffi::c_int {
            convert_setup(
                &raw mut (*st).vimconv,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            );
        }
        if margs.sort_error {
            semsg(
                gettext(b"E432: Tags file not sorted: %s\0".as_ptr() as *const ::core::ffi::c_char),
                (*st).tag_fname,
            );
        }
        if (*st).match_count >= (*st).mincount {
            (*st).stop_searching = true_0 != 0;
        }
    }
}

pub(crate) unsafe extern "C" fn findtags_copy_matches(
    mut st: *mut findtags_state_T,
    mut matchesp: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let name_only: bool = (*st).flags & TAG_NAMES as ::core::ffi::c_int != 0;
        let mut matches: *mut *mut ::core::ffi::c_char =
            (if (*st).match_count > 0 as ::core::ffi::c_int {
                xmalloc(
                    ((*st).match_count as size_t)
                        .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
                )
            } else {
                NULL_0
            }) as *mut *mut ::core::ffi::c_char;
        (*st).match_count = 0 as ::core::ffi::c_int;
        let mut mtt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while mtt < MT_COUNT as ::core::ffi::c_int {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*st).ga_match[mtt as usize].ga_len {
                let mut mfp: *mut ::core::ffi::c_char = *((*st).ga_match[mtt as usize].ga_data
                    as *mut *mut ::core::ffi::c_char)
                    .offset(i as isize);
                if matches.is_null() {
                    xfree(mfp as *mut ::core::ffi::c_void);
                } else {
                    if !name_only {
                        *mfp = (*mfp as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            as ::core::ffi::c_char;
                        let mut p: *mut ::core::ffi::c_char =
                            mfp.offset(1 as ::core::ffi::c_int as isize);
                        while *p as ::core::ffi::c_int != NUL {
                            if *p as ::core::ffi::c_int == TAG_SEP {
                                *p = NUL as ::core::ffi::c_char;
                            }
                            p = p.offset(1);
                        }
                    }
                    let c2rust_fresh4 = (*st).match_count;
                    (*st).match_count = (*st).match_count + 1;
                    let c2rust_lvalue_ptr = &raw mut *matches.offset(c2rust_fresh4 as isize);
                    *c2rust_lvalue_ptr = mfp;
                }
                i += 1;
            }
            ga_clear((&raw mut (*st).ga_match as *mut garray_T).offset(mtt as isize));
            hash_clear((&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize));
            mtt += 1;
        }
        *matchesp = matches;
        return (*st).match_count;
    }
}

pub unsafe extern "C" fn find_tags(
    mut pat: *mut ::core::ffi::c_char,
    mut num_matches: *mut ::core::ffi::c_int,
    mut matchesp: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut mincount: ::core::ffi::c_int,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut st: findtags_state_T = findtags_state_T {
            state: TS_START,
            stop_searching: false,
            orgpat: ::core::ptr::null_mut::<pat_T>(),
            lbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            lbuf_size: 0,
            tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fp: ::core::ptr::null_mut::<FILE>(),
            flags: 0,
            tag_file_sorted: 0,
            get_searchpat: false,
            help_only: false,
            did_open: false,
            mincount: 0,
            linear: false,
            vimconv: vimconv_T {
                vc_type: 0,
                vc_factor: 0,
                vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                vc_fail: false,
            },
            help_lang: [0; 3],
            help_pri: 0,
            help_lang_find: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            is_txt: false,
            match_count: 0,
            ga_match: [garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            }; 16],
            ht_match: [hashtab_T {
                ht_mask: 0,
                ht_used: 0,
                ht_filled: 0,
                ht_changed: 0,
                ht_locked: 0,
                ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                ht_smallarray: [hashitem_T {
                    hi_hash: 0,
                    hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                }; 16],
            }; 16],
        };
        let mut tn: tagname_T = tagname_T {
            tn_tags: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tn_np: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            tn_did_filefind_init: 0,
            tn_hf_idx: 0,
            tn_search_ctx: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut first_file: ::core::ffi::c_int = 0;
        let mut retval: ::core::ffi::c_int = FAIL;
        let mut i: ::core::ffi::c_int = 0;
        let mut saved_pat: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut findall: ::core::ffi::c_int = (mincount == MAXCOL as ::core::ffi::c_int
            || mincount == TAG_MANY as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        let mut has_re: bool = flags & TAG_REGEXP as ::core::ffi::c_int != 0;
        let mut noic: ::core::ffi::c_int = flags & TAG_NOIC as ::core::ffi::c_int;
        let mut verbose: ::core::ffi::c_int = flags & TAG_VERBOSE as ::core::ffi::c_int;
        let mut save_p_ic: ::core::ffi::c_int = p_ic.get();
        match if (*curbuf.get()).b_tc_flags != 0 {
            (*curbuf.get()).b_tc_flags
        } else {
            tc_flags.get()
        } {
            1 => {}
            2 => {
                p_ic.set(true_0);
            }
            4 => {
                p_ic.set(false_0);
            }
            8 => {
                p_ic.set(ignorecase(pat));
            }
            16 => {
                p_ic.set(ignorecase_opt(pat, true_0, true_0));
            }
            _ => {
                abort();
            }
        }
        let mut help_save: ::core::ffi::c_int = (*curbuf.get()).b_help as ::core::ffi::c_int;
        findtags_state_init(&raw mut st, pat, flags, mincount);
        if st.help_only {
            (*curbuf.get()).b_help = true_0 != 0;
        }
        if (*curbuf.get()).b_help {
            if (*st.orgpat).len > 3 as ::core::ffi::c_int
                && *pat.offset(((*st.orgpat).len - 3 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    == '@' as ::core::ffi::c_int
                && (*pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    >= 'A' as ::core::ffi::c_uint
                    && *pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        <= 'Z' as ::core::ffi::c_uint
                    || *pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >= 'a' as ::core::ffi::c_uint
                        && *pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            <= 'z' as ::core::ffi::c_uint)
                && (*pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    >= 'A' as ::core::ffi::c_uint
                    && *pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        <= 'Z' as ::core::ffi::c_uint
                    || *pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >= 'a' as ::core::ffi::c_uint
                        && *pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            <= 'z' as ::core::ffi::c_uint)
            {
                saved_pat = xstrnsave(pat, ((*st.orgpat).len as size_t).wrapping_sub(3 as size_t));
                st.help_lang_find =
                    pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize);
                (*st.orgpat).pat = saved_pat;
                (*st.orgpat).len -= 3 as ::core::ffi::c_int;
            }
        }
        if p_tl.get() != 0 as OptInt && (*st.orgpat).len as OptInt > p_tl.get() {
            (*st.orgpat).len = p_tl.get() as ::core::ffi::c_int;
        }
        let mut save_emsg_off: ::core::ffi::c_int = emsg_off.get();
        emsg_off.set(true_0);
        prepare_pats(st.orgpat, has_re);
        emsg_off.set(save_emsg_off);
        if !(has_re as ::core::ffi::c_int != 0 && (*st.orgpat).regmatch.regprog.is_null()) {
            retval = findtags_apply_tfu(&raw mut st, pat, buf_ffname);
            if retval == NOTDONE {
                retval = FAIL;
                if flags & TAG_KEEP_LANG as ::core::ffi::c_int != 0
                    && st.help_lang_find.is_null()
                    && !(*curbuf.get()).b_fname.is_null()
                    && {
                        i = strlen((*curbuf.get()).b_fname) as ::core::ffi::c_int;
                        i > 4 as ::core::ffi::c_int
                    }
                    && strcasecmp(
                        (*curbuf.get())
                            .b_fname
                            .offset(i as isize)
                            .offset(-(4 as ::core::ffi::c_int as isize)),
                        b".txt\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                {
                    st.is_txt = true_0 != 0;
                }
                (*st.orgpat).regmatch.rm_ic = (p_ic.get() != 0 || noic == 0)
                    && (findall != 0
                        || (*st.orgpat).headlen == 0 as ::core::ffi::c_int
                        || p_tbs.get() == 0);
                let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while round <= 2 as ::core::ffi::c_int {
                    st.linear = (*st.orgpat).headlen == 0 as ::core::ffi::c_int
                        || p_tbs.get() == 0
                        || round == 2 as ::core::ffi::c_int;
                    first_file = true_0;
                    while get_tagfname(&raw mut tn, first_file, st.tag_fname) == OK {
                        findtags_in_file(&raw mut st, flags, buf_ffname);
                        if st.stop_searching {
                            retval = OK;
                            break;
                        } else {
                            first_file = false_0;
                        }
                    }
                    tagname_free(&raw mut tn);
                    if st.stop_searching as ::core::ffi::c_int != 0
                        || st.linear as ::core::ffi::c_int != 0
                        || p_ic.get() == 0 && noic != 0
                        || (*st.orgpat).regmatch.rm_ic as ::core::ffi::c_int != 0
                    {
                        break;
                    }
                    (*st.orgpat).regmatch.rm_ic = true_0 != 0;
                    round += 1;
                }
                if !st.stop_searching {
                    if !st.did_open && verbose != 0 {
                        emsg(gettext(
                            b"E433: No tags file\0".as_ptr() as *const ::core::ffi::c_char
                        ));
                    }
                    retval = OK;
                }
            }
        }
        findtags_state_free(&raw mut st);
        if retval == FAIL {
            st.match_count = 0 as ::core::ffi::c_int;
        }
        *num_matches = findtags_copy_matches(&raw mut st, matchesp);
        (*curbuf.get()).b_help = help_save != 0;
        xfree(saved_pat as *mut ::core::ffi::c_void);
        p_ic.set(save_p_ic);
        return retval;
    }
}
