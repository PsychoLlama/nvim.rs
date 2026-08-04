//! `:s` and `:g` pattern completion from the buffer's own text.
//!
//! [`expand_pattern_in_buf`] searches the buffer for the pattern being typed
//! and offers what follows each match as a completion, so that `:%s/foo<Tab>`
//! grows into the words that actually occur.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn copy_substring_from_pos(
    mut start: *mut pos_T,
    mut end: *mut pos_T,
    mut match_0: *mut *mut ::core::ffi::c_char,
    mut match_end: *mut pos_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut exacttext: bool = wop_flags.get()
            & kOptWopFlagExacttext as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0;
        if (*start).lnum > (*end).lnum || (*start).lnum == (*end).lnum && (*start).col >= (*end).col
        {
            return FAIL;
        }
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
            128 as ::core::ffi::c_int,
        );
        let mut start_line: *mut ::core::ffi::c_char = ml_get((*start).lnum);
        let mut start_ptr: *mut ::core::ffi::c_char = start_line.offset((*start).col as isize);
        let mut is_single_line: bool = (*start).lnum == (*end).lnum;
        let mut segment_len: ::core::ffi::c_int = if is_single_line as ::core::ffi::c_int != 0 {
            (*end).col - (*start).col
        } else {
            ml_get_len((*start).lnum) - (*start).col
        };
        ga_grow(&raw mut ga, segment_len + 2 as ::core::ffi::c_int);
        ga_concat_len(&raw mut ga, start_ptr, segment_len as size_t);
        if !is_single_line {
            if exacttext {
                ga_concat_len(
                    &raw mut ga,
                    b"\\n\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                );
            } else {
                ga_append(&raw mut ga, '\n' as uint8_t);
            }
        }
        if !is_single_line {
            let mut lnum: linenr_T = (*start).lnum + 1 as linenr_T;
            while lnum < (*end).lnum {
                let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
                let mut linelen: ::core::ffi::c_int = ml_get_len(lnum);
                ga_grow(&raw mut ga, linelen + 2 as ::core::ffi::c_int);
                ga_concat_len(&raw mut ga, line, linelen as size_t);
                if exacttext {
                    ga_concat_len(
                        &raw mut ga,
                        b"\\n\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    ga_append(&raw mut ga, '\n' as uint8_t);
                }
                lnum += 1;
            }
        }
        let mut end_line: *mut ::core::ffi::c_char = ml_get((*end).lnum);
        let mut word_end: *mut ::core::ffi::c_char =
            find_word_end(end_line.offset((*end).col as isize));
        segment_len = word_end.offset_from(end_line) as ::core::ffi::c_int;
        ga_grow(&raw mut ga, segment_len);
        ga_concat_len(
            &raw mut ga,
            end_line.offset(
                (if is_single_line as ::core::ffi::c_int != 0 {
                    (*end).col as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as isize,
            ),
            (segment_len
                - (if is_single_line as ::core::ffi::c_int != 0 {
                    (*end).col as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })) as size_t,
        );
        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
        ga_append(&raw mut ga, NUL as uint8_t);
        *match_0 = ga.ga_data as *mut ::core::ffi::c_char;
        (*match_end).lnum = (*end).lnum;
        (*match_end).col = segment_len as colnr_T;
        return OK;
    }
}

pub(crate) unsafe extern "C" fn is_regex_match(
    mut pat: *mut ::core::ffi::c_char,
    mut str: *mut ::core::ffi::c_char,
) -> bool {
    unsafe {
        if strcmp(pat, str) == 0 as ::core::ffi::c_int {
            return true_0 != 0;
        }
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        (*emsg_off.ptr()) += 1;
        (*msg_silent.ptr()) += 1;
        regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
        (*emsg_off.ptr()) -= 1;
        (*msg_silent.ptr()) -= 1;
        if regmatch.regprog.is_null() {
            return false_0 != 0;
        }
        regmatch.rm_ic = p_ic.get() != 0;
        if p_ic.get() != 0 && p_scs.get() != 0 {
            regmatch.rm_ic = !pat_has_uppercase(pat);
        }
        (*emsg_off.ptr()) += 1;
        (*msg_silent.ptr()) += 1;
        let mut result: bool = vim_regexec_nl(&raw mut regmatch, str, 0 as ::core::ffi::c_int);
        (*emsg_off.ptr()) -= 1;
        (*msg_silent.ptr()) -= 1;
        vim_regfree(regmatch.regprog);
        return result;
    }
}

pub(crate) unsafe extern "C" fn concat_pattern_with_buffer_match(
    mut pat: *mut ::core::ffi::c_char,
    mut pat_len: ::core::ffi::c_int,
    mut end_match_pos: *mut pos_T,
    mut lowercase: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut line: *mut ::core::ffi::c_char = ml_get((*end_match_pos).lnum);
        let mut word_end: *mut ::core::ffi::c_char =
            find_word_end(line.offset((*end_match_pos).col as isize));
        let mut match_len: ::core::ffi::c_int =
            word_end.offset_from(line.offset((*end_match_pos).col as isize)) as ::core::ffi::c_int;
        let mut match_0: *mut ::core::ffi::c_char = xmalloc(
            (match_len as size_t)
                .wrapping_add(pat_len as size_t)
                .wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        memmove(
            match_0 as *mut ::core::ffi::c_void,
            pat as *const ::core::ffi::c_void,
            pat_len as size_t,
        );
        if match_len > 0 as ::core::ffi::c_int {
            if lowercase {
                let mut mword: *mut ::core::ffi::c_char = xstrnsave(
                    line.offset((*end_match_pos).col as isize),
                    match_len as size_t,
                );
                let mut lower: *mut ::core::ffi::c_char = strcase_save(mword, false_0 != 0);
                xfree(mword as *mut ::core::ffi::c_void);
                memmove(
                    match_0.offset(pat_len as isize) as *mut ::core::ffi::c_void,
                    lower as *const ::core::ffi::c_void,
                    match_len as size_t,
                );
                xfree(lower as *mut ::core::ffi::c_void);
            } else {
                memmove(
                    match_0.offset(pat_len as isize) as *mut ::core::ffi::c_void,
                    line.offset((*end_match_pos).col as isize) as *const ::core::ffi::c_void,
                    match_len as size_t,
                );
            }
        }
        *match_0.offset((pat_len + match_len) as isize) = NUL as ::core::ffi::c_char;
        return match_0;
    }
}

pub(crate) unsafe extern "C" fn expand_pattern_in_buf(
    mut pat: *mut ::core::ffi::c_char,
    mut dir: Direction,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut exacttext: bool = wop_flags.get()
            & kOptWopFlagExacttext as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0;
        let mut has_range: bool = search_first_line.get() != 0 as linenr_T;
        *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        *numMatches = 0 as ::core::ffi::c_int;
        if pat.is_null() || *pat as ::core::ffi::c_int == NUL {
            return FAIL;
        }
        let mut pat_len: ::core::ffi::c_int = strlen(pat) as ::core::ffi::c_int;
        let mut cur_match_pos: pos_T = pos_T {
            lnum: 0 as linenr_T,
            col: 0,
            coladd: 0,
        };
        let mut prev_match_pos: pos_T = pos_T {
            lnum: 0 as linenr_T,
            col: 0,
            coladd: 0,
        };
        if has_range {
            cur_match_pos.lnum = search_first_line.get();
        } else {
            cur_match_pos = pre_incsearch_pos.get();
        }
        let mut search_flags: ::core::ffi::c_int = SEARCH_OPT as ::core::ffi::c_int
            | SEARCH_NOOF as ::core::ffi::c_int
            | SEARCH_PEEK as ::core::ffi::c_int
            | SEARCH_NFMSG as ::core::ffi::c_int
            | (if has_range as ::core::ffi::c_int != 0 {
                SEARCH_START as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
        let mut end_match_pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut word_end_pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut looped_around: bool = false_0 != 0;
        let mut compl_started: bool = false_0 != 0;
        let mut match_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut full_match: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        '_cleanup: {
            loop {
                (*emsg_off.ptr()) += 1;
                (*msg_silent.ptr()) += 1;
                let mut found_new_match: ::core::ffi::c_int = searchit(
                    ::core::ptr::null_mut::<win_T>(),
                    curbuf.get(),
                    &raw mut cur_match_pos,
                    &raw mut end_match_pos,
                    dir,
                    pat,
                    pat_len as size_t,
                    1 as ::core::ffi::c_int,
                    search_flags,
                    RE_LAST as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<searchit_arg_T>(),
                );
                (*msg_silent.ptr()) -= 1;
                (*emsg_off.ptr()) -= 1;
                if found_new_match == FAIL {
                    break;
                }
                if has_range as ::core::ffi::c_int != 0
                    && (cur_match_pos.lnum < search_first_line.get()
                        || cur_match_pos.lnum > search_last_line.get())
                {
                    break;
                }
                if compl_started {
                    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                        && ltoreq(cur_match_pos, prev_match_pos) as ::core::ffi::c_int != 0
                        || dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int
                            && ltoreq(prev_match_pos, cur_match_pos) as ::core::ffi::c_int != 0
                    {
                        if looped_around {
                            break;
                        }
                        looped_around = true_0 != 0;
                    }
                }
                compl_started = true_0 != 0;
                prev_match_pos = cur_match_pos;
                if char_avail() as ::core::ffi::c_int != 0
                    || got_int.get() as ::core::ffi::c_int != 0
                {
                    if got_int.get() {
                        vpeekc();
                        got_int.set(false_0 != 0);
                    }
                    break '_cleanup;
                } else if end_match_pos.lnum > (*curbuf.get()).b_ml.ml_line_count {
                    cur_match_pos.lnum = 1 as ::core::ffi::c_int as linenr_T;
                    cur_match_pos.col = 0 as ::core::ffi::c_int as colnr_T;
                    cur_match_pos.coladd = 0 as ::core::ffi::c_int as colnr_T;
                } else {
                    if copy_substring_from_pos(
                        &raw mut cur_match_pos,
                        &raw mut end_match_pos,
                        &raw mut full_match,
                        &raw mut word_end_pos,
                    ) == 0
                    {
                        break;
                    }
                    if exacttext {
                        match_0 = full_match;
                    } else {
                        match_0 = concat_pattern_with_buffer_match(
                            pat,
                            pat_len,
                            &raw mut end_match_pos,
                            false_0 != 0,
                        );
                        if !is_regex_match(match_0, full_match) {
                            xfree(match_0 as *mut ::core::ffi::c_void);
                            match_0 = concat_pattern_with_buffer_match(
                                pat,
                                pat_len,
                                &raw mut end_match_pos,
                                true_0 != 0,
                            );
                            if !is_regex_match(match_0, full_match) {
                                xfree(match_0 as *mut ::core::ffi::c_void);
                                xfree(full_match as *mut ::core::ffi::c_void);
                                continue;
                            }
                        }
                        xfree(full_match as *mut ::core::ffi::c_void);
                    }
                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i < ga.ga_len {
                        if strcmp(
                            match_0,
                            *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                        ) == 0 as ::core::ffi::c_int
                        {
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                &raw mut match_0 as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL;
                            let _ = *ptr_;
                            break;
                        } else {
                            i += 1;
                        }
                    }
                    if !match_0.is_null() {
                        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                        let c2rust_fresh1 = ga.ga_len;
                        ga.ga_len = ga.ga_len + 1;
                        let c2rust_lvalue_ptr = &raw mut *(ga.ga_data
                            as *mut *mut ::core::ffi::c_char)
                            .offset(c2rust_fresh1 as isize);
                        *c2rust_lvalue_ptr = match_0;
                        if ga.ga_len > TAG_MANY as ::core::ffi::c_int {
                            break;
                        }
                    }
                    if has_range {
                        cur_match_pos = word_end_pos;
                    }
                }
            }
            *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
            *numMatches = ga.ga_len;
            return OK;
        }
        ga_clear_strings(&raw mut ga);
        return FAIL;
    }
}
