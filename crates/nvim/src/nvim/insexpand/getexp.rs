//! The collection driver: one pass over `'complete'`, one source at a time.
//!
//! [`ins_compl_get_exp`] is the loop — it asks [`process_next_cpt_value`] for
//! the next `'complete'` entry, calls the `get_next_*_completion` function
//! that entry names, and keeps going until it has enough matches or runs out
//! of sources.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn thesaurus_func_complete(mut type_0: ::core::ffi::c_int) -> bool {
    unsafe {
        return type_0 == CTRL_X_THESAURUS
            && (*(*curbuf.get()).b_p_tsrfu as ::core::ffi::c_int != NUL
                || *p_tsrfu.get() as ::core::ffi::c_int != NUL);
    }
}

pub(crate) unsafe extern "C" fn may_advance_cpt_index(mut cpt: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut p: *const ::core::ffi::c_char = cpt;
        if cpt_sources_index.get() == -1 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        return *p as ::core::ffi::c_int != NUL;
    }
}

pub(crate) unsafe extern "C" fn process_next_cpt_value(
    mut st: *mut ins_compl_next_state_T,
    mut compl_type_arg: *mut ::core::ffi::c_int,
    mut start_match_pos: *mut pos_T,
    mut fuzzy_collect: bool,
    mut advance_cpt_idx: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut compl_type: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut status: ::core::ffi::c_int = INS_COMPL_CPT_OK;
        let mut skip_source: bool = compl_autocomplete.get() as ::core::ffi::c_int != 0
            && compl_from_nonkeyword.get() as ::core::ffi::c_int != 0;
        (*st).found_all = false_0 != 0;
        *advance_cpt_idx = false_0 != 0;
        while *(*st).e_cpt as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *(*st).e_cpt as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        {
            (*st).e_cpt = (*st).e_cpt.offset(1);
        }
        '_done: {
            if *(*st).e_cpt as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                && !(*curbuf.get()).b_scanned
                && !skip_source
                && !compl_time_slice_expired.get()
            {
                (*st).ins_buf = curbuf.get();
                (*st).first_match_pos = *start_match_pos;
                if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                    && (!fuzzy_collect
                        && dec(&raw mut (*st).first_match_pos) < 0 as ::core::ffi::c_int)
                {
                    (*st).first_match_pos.lnum = (*(*st).ins_buf).b_ml.ml_line_count;
                    (*st).first_match_pos.col = ml_get_len((*st).first_match_pos.lnum);
                }
                (*st).last_match_pos = (*st).first_match_pos;
                compl_type = 0 as ::core::ffi::c_int;
                (*st).set_match_pos = true_0 != 0;
            } else if !skip_source
                && !compl_time_slice_expired.get()
                && !vim_strchr(
                    b"buwU\0".as_ptr() as *const ::core::ffi::c_char,
                    *(*st).e_cpt as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
                && {
                    (*st).ins_buf =
                        ins_compl_next_buf((*st).ins_buf, *(*st).e_cpt as ::core::ffi::c_int);
                    (*st).ins_buf != curbuf.get()
                }
            {
                if !(*(*st).ins_buf).b_ml.ml_mfp.is_null() {
                    compl_started.set(true_0 != 0);
                    (*st).last_match_pos.col = 0 as ::core::ffi::c_int as colnr_T;
                    (*st).first_match_pos.col = (*st).last_match_pos.col;
                    (*st).first_match_pos.lnum =
                        (*(*st).ins_buf).b_ml.ml_line_count + 1 as linenr_T;
                    (*st).last_match_pos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                    compl_type = 0 as ::core::ffi::c_int;
                } else {
                    (*st).found_all = true_0 != 0;
                    if (*(*st).ins_buf).b_fname.is_null() {
                        status = INS_COMPL_CPT_CONT;
                        break '_done;
                    } else {
                        compl_type = CTRL_X_DICTIONARY;
                        (*st).dict = (*(*st).ins_buf).b_fname;
                        (*st).dict_f = DICT_EXACT;
                    }
                }
                if !shortmess(SHM_COMPLETIONSCAN) && !compl_autocomplete.get() {
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        gettext(b"Scanning: %s\0".as_ptr() as *const ::core::ffi::c_char),
                        if (*(*st).ins_buf).b_fname.is_null() {
                            buf_spname((*st).ins_buf)
                        } else if (*(*st).ins_buf).b_sfname.is_null() {
                            (*(*st).ins_buf).b_fname
                        } else {
                            (*(*st).ins_buf).b_sfname
                        },
                    );
                    msg_progress(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        b"completion\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        b"running\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        HLF_R,
                        false_0 != 0,
                        true_0 != 0,
                    );
                }
            } else if *(*st).e_cpt as ::core::ffi::c_int == NUL {
                status = INS_COMPL_CPT_END;
            } else {
                if !ctrl_x_mode_line_or_eval() {
                    if *(*st).e_cpt as ::core::ffi::c_int == 'F' as ::core::ffi::c_int
                        || *(*st).e_cpt as ::core::ffi::c_int == 'o' as ::core::ffi::c_int
                    {
                        compl_type = CTRL_X_FUNCTION;
                        (*st).func_cb =
                            get_callback_if_cpt_func((*st).e_cpt, cpt_sources_index.get());
                        if (*st).func_cb.is_null() {
                            compl_type = -1 as ::core::ffi::c_int;
                        }
                    } else if !skip_source {
                        if *(*st).e_cpt as ::core::ffi::c_int == 'k' as ::core::ffi::c_int
                            || *(*st).e_cpt as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                        {
                            if *(*st).e_cpt as ::core::ffi::c_int == 'k' as ::core::ffi::c_int {
                                compl_type = CTRL_X_DICTIONARY;
                            } else {
                                compl_type = CTRL_X_THESAURUS;
                            }
                            (*st).e_cpt = (*st).e_cpt.offset(1);
                            if *(*st).e_cpt as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                                && *(*st).e_cpt as ::core::ffi::c_int != NUL
                            {
                                (*st).dict = (*st).e_cpt;
                                (*st).dict_f = DICT_FIRST;
                            }
                        } else if *(*st).e_cpt as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
                            compl_type = CTRL_X_PATH_PATTERNS;
                        } else if *(*st).e_cpt as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                            compl_type = CTRL_X_PATH_DEFINES;
                        } else if *(*st).e_cpt as ::core::ffi::c_int == 'f' as ::core::ffi::c_int {
                            compl_type = CTRL_X_BUFNAMES;
                        } else if *(*st).e_cpt as ::core::ffi::c_int == ']' as ::core::ffi::c_int
                            || *(*st).e_cpt as ::core::ffi::c_int == 't' as ::core::ffi::c_int
                        {
                            compl_type = CTRL_X_TAGS;
                            if !shortmess(SHM_COMPLETIONSCAN) && !compl_autocomplete.get() {
                                vim_snprintf(
                                    IObuff.ptr() as *mut ::core::ffi::c_char,
                                    IOSIZE as size_t,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    gettext(
                                        b"Scanning tags.\0".as_ptr() as *const ::core::ffi::c_char
                                    ),
                                );
                                msg_progress(
                                    IObuff.ptr() as *mut ::core::ffi::c_char,
                                    b"completion\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    b"running\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    HLF_R,
                                    false_0 != 0,
                                    true_0 != 0,
                                );
                            }
                        }
                    }
                }
                copy_option_part(
                    &raw mut (*st).e_cpt,
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                *advance_cpt_idx = may_advance_cpt_index((*st).e_cpt);
                (*st).found_all = true_0 != 0;
                if compl_type == -1 as ::core::ffi::c_int {
                    status = INS_COMPL_CPT_CONT;
                }
            }
        }
        *compl_type_arg = compl_type;
        return status;
    }
}

pub(crate) unsafe extern "C" fn get_next_include_file_completion(
    mut compl_type: ::core::ffi::c_int,
) {
    unsafe {
        find_pattern_in_path(
            (*compl_pattern.ptr()).data,
            compl_direction.get(),
            (*compl_pattern.ptr()).size,
            false_0 != 0,
            false_0 != 0,
            if compl_type == CTRL_X_PATH_DEFINES && compl_cont_status.get() & CONT_SOL == 0 {
                FIND_DEFINE
            } else {
                FIND_ANY
            },
            1 as ::core::ffi::c_int,
            ACTION_EXPAND,
            1 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            false_0 != 0,
            compl_autocomplete.get(),
        );
    }
}

pub(crate) unsafe extern "C" fn get_next_dict_tsr_completion(
    mut compl_type: ::core::ffi::c_int,
    mut dict: *mut ::core::ffi::c_char,
    mut dict_f: ::core::ffi::c_int,
) {
    unsafe {
        if thesaurus_func_complete(compl_type) {
            expand_by_function(
                compl_type,
                (*compl_pattern.ptr()).data,
                ::core::ptr::null_mut::<Callback>(),
            );
        } else {
            ins_compl_dictionaries(
                if !dict.is_null() {
                    dict
                } else if compl_type == CTRL_X_THESAURUS {
                    if *(*curbuf.get()).b_p_tsr as ::core::ffi::c_int == NUL {
                        p_tsr.get()
                    } else {
                        (*curbuf.get()).b_p_tsr
                    }
                } else if *(*curbuf.get()).b_p_dict as ::core::ffi::c_int == NUL {
                    p_dict.get()
                } else {
                    (*curbuf.get()).b_p_dict
                },
                (*compl_pattern.ptr()).data,
                if !dict.is_null() {
                    dict_f
                } else {
                    0 as ::core::ffi::c_int
                },
                compl_type == CTRL_X_THESAURUS,
            );
        };
    }
}

pub(crate) unsafe extern "C" fn get_next_tag_completion() {
    unsafe {
        let save_p_ic: ::core::ffi::c_int = p_ic.get();
        p_ic.set(ignorecase((*compl_pattern.ptr()).data));
        g_tag_at_cursor.set(true_0 != 0);
        let mut matches: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut num_matches: ::core::ffi::c_int = 0;
        if find_tags(
            (*compl_pattern.ptr()).data,
            &raw mut num_matches,
            &raw mut matches,
            TAG_REGEXP
                | TAG_NAMES
                | TAG_NOIC
                | TAG_INS_COMP
                | (if ctrl_x_mode_not_default() as ::core::ffi::c_int != 0 {
                    TAG_VERBOSE
                } else {
                    0 as ::core::ffi::c_int
                }),
            TAG_MANY,
            (*curbuf.get()).b_ffname,
        ) == OK
            && num_matches > 0 as ::core::ffi::c_int
        {
            ins_compl_add_matches(num_matches, matches, p_ic.get());
        }
        g_tag_at_cursor.set(false_0 != 0);
        p_ic.set(save_p_ic);
    }
}

pub(crate) unsafe extern "C" fn get_next_filename_completion() {
    unsafe {
        let mut matches: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut num_matches: ::core::ffi::c_int = 0;
        let mut leader: *mut ::core::ffi::c_char = ins_compl_leader();
        let mut leader_len: size_t = ins_compl_leader_len();
        let mut in_fuzzy_collect: bool =
            cot_fuzzy() as ::core::ffi::c_int != 0 && leader_len > 0 as size_t;
        let mut need_collect_bests: bool = in_fuzzy_collect as ::core::ffi::c_int != 0
            && compl_get_longest.get() as ::core::ffi::c_int != 0;
        let mut max_score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut dir: Direction = compl_direction.get();
        let mut pathsep: ::core::ffi::c_char = PATHSEP as ::core::ffi::c_char;
        if in_fuzzy_collect {
            let mut last_sep: *mut ::core::ffi::c_char =
                strrchr(leader, pathsep as ::core::ffi::c_int);
            if last_sep.is_null() {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut (*compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                let _ = *ptr_;
                (*compl_pattern.ptr()).size = 0 as size_t;
                compl_pattern.set(cbuf_to_string(
                    b"*\0".as_ptr() as *const ::core::ffi::c_char,
                    1 as size_t,
                ));
            } else if *last_sep.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == NUL
            {
                in_fuzzy_collect = false_0 != 0;
            } else {
                let mut path_len: size_t =
                    (last_sep.offset_from(leader) as size_t).wrapping_add(1 as size_t);
                let mut path_with_wildcard: *mut ::core::ffi::c_char =
                    xmalloc(path_len.wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
                vim_snprintf(
                    path_with_wildcard,
                    path_len.wrapping_add(2 as size_t),
                    b"%*.*s*\0".as_ptr() as *const ::core::ffi::c_char,
                    path_len as ::core::ffi::c_int,
                    path_len as ::core::ffi::c_int,
                    leader,
                );
                let mut ptr__0: *mut *mut ::core::ffi::c_void =
                    &raw mut (*compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
                xfree(*ptr__0);
                *ptr__0 = NULL;
                let _ = *ptr__0;
                (*compl_pattern.ptr()).size = 0 as size_t;
                (*compl_pattern.ptr()).data = path_with_wildcard;
                (*compl_pattern.ptr()).size = path_len.wrapping_add(1 as size_t);
                leader = last_sep.offset(1 as ::core::ffi::c_int as isize);
                leader_len = leader_len.wrapping_sub(path_len);
            }
        }
        if expand_wildcards(
            1 as ::core::ffi::c_int,
            &raw mut (*compl_pattern.ptr()).data,
            &raw mut num_matches,
            &raw mut matches,
            EW_FILE | EW_DIR | EW_ADDSLASH | EW_SILENT,
        ) != OK
        {
            return;
        }
        tilde_replace((*compl_pattern.ptr()).data, num_matches, matches);
        if in_fuzzy_collect {
            let mut fuzzy_indices: garray_T = garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            };
            ga_init(
                &raw mut fuzzy_indices,
                ::core::mem::size_of::<::core::ffi::c_int>() as ::core::ffi::c_int,
                10 as ::core::ffi::c_int,
            );
            compl_fuzzy_scores.set(xmalloc(
                ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(num_matches as size_t),
            ) as *mut ::core::ffi::c_int);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < num_matches {
                let mut ptr: *mut ::core::ffi::c_char = *matches.offset(i as isize);
                let mut score: ::core::ffi::c_int = fuzzy_match_str(ptr, leader);
                if score != FUZZY_SCORE_NONE {
                    ga_grow(&raw mut fuzzy_indices, 1 as ::core::ffi::c_int);
                    *(fuzzy_indices.ga_data as *mut ::core::ffi::c_int)
                        .offset(fuzzy_indices.ga_len as isize) = i;
                    fuzzy_indices.ga_len += 1;
                    *(*compl_fuzzy_scores.ptr()).offset(i as isize) = score;
                }
                i += 1;
            }
            if fuzzy_indices.ga_len > 0 as ::core::ffi::c_int {
                let mut fuzzy_indices_data: *mut ::core::ffi::c_int =
                    fuzzy_indices.ga_data as *mut ::core::ffi::c_int;
                qsort(
                    fuzzy_indices_data as *mut ::core::ffi::c_void,
                    fuzzy_indices.ga_len as size_t,
                    ::core::mem::size_of::<::core::ffi::c_int>(),
                    Some(
                        compare_scores
                            as unsafe extern "C" fn(
                                *const ::core::ffi::c_void,
                                *const ::core::ffi::c_void,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_0 < fuzzy_indices.ga_len {
                    let mut match_0: *mut ::core::ffi::c_char =
                        *matches.offset(*fuzzy_indices_data.offset(i_0 as isize) as isize);
                    let mut current_score: ::core::ffi::c_int = *(*compl_fuzzy_scores.ptr())
                        .offset(*fuzzy_indices_data.offset(i_0 as isize) as isize);
                    if ins_compl_add(
                        match_0,
                        -1 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null::<*mut ::core::ffi::c_char>(),
                        false_0 != 0,
                        ::core::ptr::null_mut::<typval_T>(),
                        dir,
                        CP_FAST
                            | (if p_fic.get() != 0 || p_wic.get() != 0 {
                                CP_ICASE
                            } else {
                                0 as ::core::ffi::c_int
                            }),
                        false_0 != 0,
                        ::core::ptr::null::<::core::ffi::c_int>(),
                        current_score,
                    ) == OK
                    {
                        dir = FORWARD;
                    }
                    if need_collect_bests {
                        if i_0 == 0 as ::core::ffi::c_int || current_score == max_score {
                            (*compl_num_bests.ptr()) += 1;
                            max_score = current_score;
                        }
                    }
                    i_0 += 1;
                }
                FreeWild(num_matches, matches);
            } else if leader_len > 0 as size_t {
                FreeWild(num_matches, matches);
                num_matches = 0 as ::core::ffi::c_int;
            }
            xfree(compl_fuzzy_scores.get() as *mut ::core::ffi::c_void);
            ga_clear(&raw mut fuzzy_indices);
            if compl_num_bests.get() > 0 as ::core::ffi::c_int
                && compl_get_longest.get() as ::core::ffi::c_int != 0
            {
                fuzzy_longest_match();
            }
            return;
        }
        if num_matches > 0 as ::core::ffi::c_int {
            ins_compl_add_matches(
                num_matches,
                matches,
                (p_fic.get() != 0 || p_wic.get() != 0) as ::core::ffi::c_int,
            );
        }
    }
}

pub(crate) unsafe extern "C" fn get_next_cmdline_completion() {
    unsafe {
        let mut matches: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut num_matches: ::core::ffi::c_int = 0;
        if expand_cmdline(
            compl_xp.ptr(),
            (*compl_pattern.ptr()).data,
            (*compl_pattern.ptr()).size as ::core::ffi::c_int,
            &raw mut num_matches,
            &raw mut matches,
        ) == EXPAND_OK
        {
            ins_compl_add_matches(num_matches, matches, false_0);
        }
    }
}

pub(crate) unsafe extern "C" fn get_next_spell_completion(mut lnum: linenr_T) {
    unsafe {
        let mut matches: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut num_matches: ::core::ffi::c_int =
            expand_spelling(lnum, (*compl_pattern.ptr()).data, &raw mut matches);
        if num_matches > 0 as ::core::ffi::c_int {
            ins_compl_add_matches(num_matches, matches, p_ic.get());
        } else {
            xfree(matches as *mut ::core::ffi::c_void);
        };
    }
}

pub(crate) unsafe extern "C" fn get_next_completion_match(
    mut type_0: ::core::ffi::c_int,
    mut st: *mut ins_compl_next_state_T,
    mut ini: *mut pos_T,
) -> bool {
    unsafe {
        let mut found_new_match: ::core::ffi::c_int = FAIL;
        match type_0 {
            -1 => {}
            262 | 263 => {
                get_next_include_file_completion(type_0);
            }
            265 | 266 => {
                get_next_dict_tsr_completion(type_0, (*st).dict, (*st).dict_f);
                (*st).dict = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            261 => {
                get_next_tag_completion();
            }
            4 => {
                get_next_filename_completion();
            }
            11 | 17 => {
                get_next_cmdline_completion();
            }
            12 => {
                if ctrl_x_mode_normal() {
                    get_cpt_func_completion_matches((*st).func_cb);
                } else {
                    expand_by_function(
                        type_0,
                        (*compl_pattern.ptr()).data,
                        ::core::ptr::null_mut::<Callback>(),
                    );
                }
            }
            13 => {
                expand_by_function(
                    type_0,
                    (*compl_pattern.ptr()).data,
                    ::core::ptr::null_mut::<Callback>(),
                );
            }
            14 => {
                get_next_spell_completion((*st).first_match_pos.lnum);
            }
            18 => {
                get_next_bufname_token();
            }
            19 => {
                get_register_completion();
            }
            _ => {
                found_new_match = get_next_default_completion(st, ini);
                if found_new_match == FAIL && (*st).ins_buf == curbuf.get() {
                    (*st).found_all = true_0 != 0;
                }
            }
        }
        if type_0 != 0 as ::core::ffi::c_int && compl_curr_match.get() != compl_old_match.get() {
            found_new_match = OK;
        }
        return found_new_match != 0;
    }
}

pub(crate) unsafe extern "C" fn compl_source_start_timer(mut source_idx: ::core::ffi::c_int) {
    unsafe {
        if compl_autocomplete.get() as ::core::ffi::c_int != 0 || p_cto.get() > 0 as OptInt {
            (*(*cpt_sources_array.ptr()).offset(source_idx as isize)).compl_start_tv = os_hrtime();
            compl_time_slice_expired.set(false_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn ins_compl_get_exp(mut ini: *mut pos_T) -> ::core::ffi::c_int {
    unsafe {
        static st: GlobalCell<ins_compl_next_state_T> = GlobalCell::new(ins_compl_next_state_T {
            e_cpt_copy: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            e_cpt: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ins_buf: ::core::ptr::null_mut::<buf_T>(),
            cur_match_pos: ::core::ptr::null_mut::<pos_T>(),
            prev_match_pos: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            set_match_pos: false,
            first_match_pos: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            last_match_pos: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            found_all: false,
            dict: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            dict_f: 0,
            func_cb: ::core::ptr::null_mut::<Callback>(),
        });
        static st_cleared: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut found_new_match: ::core::ffi::c_int = 0;
        let mut type_0: ::core::ffi::c_int = ctrl_x_mode.get();
        let mut may_advance_cpt_idx: bool = false_0 != 0;
        let mut start_pos: pos_T = *ini;
        '_c2rust_label: {
            if !(*curbuf.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"curbuf != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4690 as ::core::ffi::c_uint,
                    b"int ins_compl_get_exp(pos_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if !compl_started.get() {
            let mut buf: *mut buf_T = firstbuf.get();
            while !buf.is_null() {
                (*buf).b_scanned = false_0 != 0;
                buf = (*buf).b_next;
            }
            if !st_cleared.get() {
                memset(
                    st.ptr() as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<ins_compl_next_state_T>(),
                );
                st_cleared.set(true_0 != 0);
            }
            (*st.ptr()).found_all = false_0 != 0;
            (*st.ptr()).ins_buf = curbuf.get();
            xfree((*st.ptr()).e_cpt_copy as *mut ::core::ffi::c_void);
            (*st.ptr()).e_cpt_copy = xstrdup(if compl_cont_status.get() & CONT_LOCAL != 0 {
                b".\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                (*curbuf.get()).b_p_cpt as *const ::core::ffi::c_char
            });
            strip_caret_numbers_in_place((*st.ptr()).e_cpt_copy);
            (*st.ptr()).e_cpt = (*st.ptr()).e_cpt_copy;
            if compl_autocomplete.get() as ::core::ffi::c_int != 0
                && is_nearest_active() as ::core::ffi::c_int != 0
            {
                start_pos.lnum = if 1 as linenr_T > start_pos.lnum - 1000 as linenr_T {
                    1 as linenr_T
                } else {
                    start_pos.lnum - 1000 as linenr_T
                };
                start_pos.col = 0 as ::core::ffi::c_int as colnr_T;
            }
            (*st.ptr()).first_match_pos = start_pos;
            (*st.ptr()).last_match_pos = (*st.ptr()).first_match_pos;
        } else if (*st.ptr()).ins_buf != curbuf.get() && !buf_valid((*st.ptr()).ins_buf) {
            (*st.ptr()).ins_buf = curbuf.get();
        }
        '_c2rust_label_0: {
            if !(*st.ptr()).ins_buf.is_null() {
            } else {
                __assert_fail(
                    b"st.ins_buf != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4718 as ::core::ffi::c_uint,
                    b"int ins_compl_get_exp(pos_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        compl_old_match.set(compl_curr_match.get());
        (*st.ptr()).cur_match_pos = if compl_dir_forward() as ::core::ffi::c_int != 0 {
            &raw mut (*st.ptr()).last_match_pos
        } else {
            &raw mut (*st.ptr()).first_match_pos
        };
        let mut normal_mode_strict: bool = ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            && !ctrl_x_mode_line_or_eval()
            && compl_cont_status.get() & CONT_LOCAL == 0
            && !(*cpt_sources_array.ptr()).is_null();
        if normal_mode_strict {
            cpt_sources_index.set(0 as ::core::ffi::c_int);
            if compl_autocomplete.get() as ::core::ffi::c_int != 0 || p_cto.get() > 0 as OptInt {
                compl_source_start_timer(0 as ::core::ffi::c_int);
                compl_time_slice_expired.set(false_0 != 0);
                compl_timeout_ms.set(if compl_autocomplete.get() as ::core::ffi::c_int != 0 {
                    (if 80 as OptInt > p_act.get() {
                        80 as OptInt
                    } else {
                        p_act.get()
                    }) as uint64_t
                } else {
                    p_cto.get() as uint64_t
                });
            }
        }
        loop {
            found_new_match = FAIL;
            (*st.ptr()).set_match_pos = false_0 != 0;
            if (ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                || ctrl_x_mode_line_or_eval() as ::core::ffi::c_int != 0)
                && (!compl_started.get() || (*st.ptr()).found_all as ::core::ffi::c_int != 0)
            {
                let mut status: ::core::ffi::c_int = process_next_cpt_value(
                    st.ptr(),
                    &raw mut type_0,
                    &raw mut start_pos,
                    cot_fuzzy(),
                    &raw mut may_advance_cpt_idx,
                );
                if status == INS_COMPL_CPT_END {
                    break;
                }
                if status == INS_COMPL_CPT_CONT {
                    if !may_advance_cpt_idx {
                        continue;
                    }
                    if advance_cpt_sources_index_safe() == 0 {
                        break;
                    }
                    compl_source_start_timer(cpt_sources_index.get());
                    continue;
                }
            }
            let mut compl_timeout_save: uint64_t = 0 as uint64_t;
            if normal_mode_strict as ::core::ffi::c_int != 0
                && type_0 == CTRL_X_FUNCTION
                && (compl_autocomplete.get() as ::core::ffi::c_int != 0
                    || p_cto.get() > 0 as OptInt)
            {
                compl_timeout_save = compl_timeout_ms.get();
                compl_timeout_ms.set(
                    (if compl_from_nonkeyword.get() as ::core::ffi::c_int != 0 {
                        COMPL_FUNC_TIMEOUT_NON_KW_MS
                    } else {
                        COMPL_FUNC_TIMEOUT_MS
                    }) as uint64_t,
                );
            }
            found_new_match = get_next_completion_match(type_0, st.ptr(), &raw mut start_pos)
                as ::core::ffi::c_int;
            if (*compl_pattern.ptr()).data.is_null() {
                break;
            }
            if may_advance_cpt_idx {
                if advance_cpt_sources_index_safe() == 0 {
                    break;
                }
                compl_source_start_timer(cpt_sources_index.get());
            }
            if ctrl_x_mode_not_default() as ::core::ffi::c_int != 0 && !ctrl_x_mode_line_or_eval()
                || found_new_match != FAIL
            {
                if got_int.get() {
                    break;
                }
                if type_0 != -1 as ::core::ffi::c_int {
                    ins_compl_check_keys(0 as ::core::ffi::c_int, false_0 != 0);
                }
                if ctrl_x_mode_not_default() as ::core::ffi::c_int != 0
                    && !ctrl_x_mode_line_or_eval()
                    || compl_interrupted.get() as ::core::ffi::c_int != 0
                {
                    break;
                }
                compl_started.set(!compl_time_slice_expired.get());
            } else {
                if buf_valid((*st.ptr()).ins_buf) as ::core::ffi::c_int != 0
                    && (type_0 == 0 as ::core::ffi::c_int || type_0 == CTRL_X_PATH_PATTERNS)
                {
                    '_c2rust_label_1: {
                        if !(*st.ptr()).ins_buf.is_null() {
                        } else {
                            __assert_fail(
                                b"st.ins_buf\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                4812 as ::core::ffi::c_uint,
                                b"int ins_compl_get_exp(pos_T *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    (*(*st.ptr()).ins_buf).b_scanned = true_0 != 0;
                }
                compl_started.set(false_0 != 0);
            }
            if normal_mode_strict as ::core::ffi::c_int != 0
                && type_0 == CTRL_X_FUNCTION
                && (compl_autocomplete.get() as ::core::ffi::c_int != 0
                    || p_cto.get() > 0 as OptInt)
            {
                compl_timeout_ms.set(compl_timeout_save);
            }
            if !compl_dir_forward() {
                while !(*compl_curr_match.get()).cp_prev.is_null()
                    && !match_at_original_text((*compl_curr_match.get()).cp_prev)
                {
                    compl_curr_match.set((*compl_curr_match.get()).cp_prev);
                }
            }
        }
        cpt_sources_index.set(-1 as ::core::ffi::c_int);
        compl_started.set(true_0 != 0);
        if (ctrl_x_mode_normal() as ::core::ffi::c_int != 0
            || ctrl_x_mode_line_or_eval() as ::core::ffi::c_int != 0)
            && *(*st.ptr()).e_cpt as ::core::ffi::c_int == NUL
        {
            found_new_match = FAIL;
        }
        let mut match_count: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if found_new_match == FAIL
            || ctrl_x_mode_not_default() as ::core::ffi::c_int != 0 && !ctrl_x_mode_line_or_eval()
        {
            match_count = ins_compl_make_cyclic();
        }
        if cot_fuzzy() as ::core::ffi::c_int != 0
            && compl_get_longest.get() as ::core::ffi::c_int != 0
            && compl_num_bests.get() > 0 as ::core::ffi::c_int
        {
            fuzzy_longest_match();
        }
        if !(*compl_old_match.ptr()).is_null() {
            compl_curr_match.set(if compl_dir_forward() as ::core::ffi::c_int != 0 {
                (*compl_old_match.get()).cp_next
            } else {
                (*compl_old_match.get()).cp_prev
            });
            if (*compl_curr_match.ptr()).is_null() {
                compl_curr_match.set(compl_old_match.get());
            }
        }
        may_trigger_modechanged();
        if match_count > 0 as ::core::ffi::c_int && !ctrl_x_mode_spell() {
            if is_nearest_active() as ::core::ffi::c_int != 0 && !ins_compl_has_preinsert() {
                sort_compl_match_list(Some(
                    cp_compare_nearest
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ));
            }
            if cot_fuzzy() as ::core::ffi::c_int != 0 && ins_compl_leader_len() > 0 as size_t {
                ins_compl_fuzzy_sort();
            }
        }
        return match_count;
    }
}

pub(crate) unsafe extern "C" fn check_elapsed_time() {
    unsafe {
        let mut start_tv: uint64_t =
            (*(*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize)).compl_start_tv;
        let mut elapsed_ms: uint64_t = os_hrtime()
            .wrapping_sub(start_tv)
            .wrapping_div(1000000 as uint64_t);
        if elapsed_ms > compl_timeout_ms.get() {
            compl_time_slice_expired.set(true_0 != 0);
            if compl_timeout_ms.get() > COMPL_MIN_TIMEOUT_MS as uint64_t {
                compl_timeout_ms.set((*compl_timeout_ms.ptr()).wrapping_div(2 as uint64_t));
            }
        }
    }
}
