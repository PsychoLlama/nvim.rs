//! Scanning buffers, dictionaries, thesauruses and registers for matches.
//!
//! [`ins_compl_dictionaries`] and [`ins_compl_files`] are the `'dictionary'`
//! and `'thesaurus'` file walk; [`get_next_default_completion`] is the
//! keyword search through the buffers `'complete'` names, driven by
//! [`ins_compl_next_buf`]; [`get_register_completion`] is the `CTRL-X
//! CTRL-R` source.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ins_compl_dictionaries(
    mut dict_start: *mut ::core::ffi::c_char,
    mut pat: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut thesaurus: bool,
) {
    unsafe {
        let mut dict: *mut ::core::ffi::c_char = dict_start;
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        let mut files: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut count: ::core::ffi::c_int = 0;
        let mut dir: Direction = compl_direction.get();
        if *dict as ::core::ffi::c_int == NUL {
            if !thesaurus && (*curwin.get()).w_onebuf_opt.wo_spell != 0 {
                dict =
                    b"spell\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                return;
            }
        }
        let mut buf: *mut ::core::ffi::c_char =
            xmalloc(LSIZE as size_t) as *mut ::core::ffi::c_char;
        regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
        let mut save_p_scs: ::core::ffi::c_int = p_scs.get();
        if (*curbuf.get()).b_p_inf != 0 {
            p_scs.set(false_0);
        }
        '_theend: {
            if ctrl_x_mode_line_or_eval() {
                let mut pat_esc: *mut ::core::ffi::c_char =
                    vim_strsave_escaped(pat, b"\\\0".as_ptr() as *const ::core::ffi::c_char);
                let mut len: size_t = strlen(pat_esc).wrapping_add(10 as size_t);
                ptr = xmalloc(len) as *mut ::core::ffi::c_char;
                vim_snprintf(
                    ptr,
                    len,
                    b"^\\s*\\zs\\V%s\0".as_ptr() as *const ::core::ffi::c_char,
                    pat_esc,
                );
                regmatch.regprog = vim_regcomp(ptr, RE_MAGIC);
                xfree(pat_esc as *mut ::core::ffi::c_void);
                xfree(ptr as *mut ::core::ffi::c_void);
            } else {
                regmatch.regprog = vim_regcomp(
                    pat,
                    if magic_isset() as ::core::ffi::c_int != 0 {
                        RE_MAGIC
                    } else {
                        0 as ::core::ffi::c_int
                    },
                );
                if regmatch.regprog.is_null() {
                    break '_theend;
                }
            }
            regmatch.rm_ic = ignorecase(pat) != 0;
            while *dict as ::core::ffi::c_int != NUL && !got_int.get() && !compl_interrupted.get() {
                if flags == DICT_EXACT {
                    count = 1 as ::core::ffi::c_int;
                    files = &raw mut dict;
                } else {
                    copy_option_part(
                        &raw mut dict,
                        buf,
                        LSIZE as size_t,
                        b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    );
                    if !thesaurus
                        && strcmp(buf, b"spell\0".as_ptr() as *const ::core::ffi::c_char)
                            == 0 as ::core::ffi::c_int
                    {
                        count = -1 as ::core::ffi::c_int;
                    } else if !vim_strchr(buf, '`' as ::core::ffi::c_int).is_null()
                        || expand_wildcards(
                            1 as ::core::ffi::c_int,
                            &raw mut buf,
                            &raw mut count,
                            &raw mut files,
                            EW_FILE | EW_SILENT,
                        ) != OK
                    {
                        count = 0 as ::core::ffi::c_int;
                    }
                }
                if count == -1 as ::core::ffi::c_int {
                    if *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                        && *pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '<' as ::core::ffi::c_int
                    {
                        ptr = pat.offset(2 as ::core::ffi::c_int as isize);
                    } else {
                        ptr = pat;
                    }
                    spell_dump_compl(
                        ptr,
                        regmatch.rm_ic as ::core::ffi::c_int,
                        &raw mut dir,
                        0 as ::core::ffi::c_int,
                    );
                } else if count > 0 as ::core::ffi::c_int {
                    ins_compl_files(
                        count,
                        files,
                        thesaurus,
                        flags,
                        &raw mut regmatch,
                        buf,
                        &raw mut dir,
                    );
                    if flags != DICT_EXACT {
                        FreeWild(count, files);
                    }
                }
                if flags != 0 as ::core::ffi::c_int {
                    break;
                }
            }
        }
        p_scs.set(save_p_scs);
        vim_regfree(regmatch.regprog);
        xfree(buf as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn thesaurus_add_words_in_line(
    mut fname: *mut ::core::ffi::c_char,
    mut buf_arg: *mut *mut ::core::ffi::c_char,
    mut dir: ::core::ffi::c_int,
    mut skip_word: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut status: ::core::ffi::c_int = OK;
        let mut ptr: *mut ::core::ffi::c_char = *buf_arg;
        while !got_int.get() {
            ptr = find_word_start(ptr);
            if *ptr as ::core::ffi::c_int == NUL || *ptr as ::core::ffi::c_int == NL {
                break;
            }
            let mut wstart: *mut ::core::ffi::c_char = ptr;
            while *ptr as ::core::ffi::c_int != NUL {
                let l: ::core::ffi::c_int = utfc_ptr2len(ptr);
                if l < 2 as ::core::ffi::c_int
                    && !vim_iswordc(*ptr as uint8_t as ::core::ffi::c_int)
                {
                    break;
                }
                ptr = ptr.offset(l as isize);
            }
            if wstart == skip_word as *mut ::core::ffi::c_char {
                continue;
            }
            status = ins_compl_add_infercase(
                wstart,
                ptr.offset_from(wstart) as ::core::ffi::c_int,
                p_ic.get() != 0,
                fname,
                dir as Direction,
                false_0 != 0,
                FUZZY_SCORE_NONE,
            );
            if status == FAIL {
                break;
            }
        }
        *buf_arg = ptr;
        return status;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_files(
    mut count: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
    mut thesaurus: bool,
    mut flags: ::core::ffi::c_int,
    mut regmatch: *mut regmatch_T,
    mut buf: *mut ::core::ffi::c_char,
    mut dir: *mut Direction,
) {
    unsafe {
        let mut leader: *mut ::core::ffi::c_char = if cot_fuzzy() as ::core::ffi::c_int != 0 {
            ins_compl_leader()
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
        let mut leader_len: ::core::ffi::c_int = if cot_fuzzy() as ::core::ffi::c_int != 0 {
            ins_compl_leader_len() as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < count && !got_int.get() && !ins_compl_interrupted() {
            let mut fp: *mut FILE = os_fopen(
                *files.offset(i as isize),
                b"r\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if flags != DICT_EXACT && !shortmess(SHM_COMPLETIONSCAN) && !compl_autocomplete.get() {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(b"Scanning dictionary: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    *files.offset(i as isize),
                );
                msg_progress(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    b"completion\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    b"running\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    HLF_R,
                    false_0 != 0,
                    true_0 != 0,
                );
            }
            if !fp.is_null() {
                while !got_int.get() && !ins_compl_interrupted() && !vim_fgets(buf, LSIZE, fp) {
                    let mut ptr: *mut ::core::ffi::c_char = buf;
                    if cot_fuzzy() as ::core::ffi::c_int != 0
                        && leader_len > 0 as ::core::ffi::c_int
                    {
                        let mut line_end: *mut ::core::ffi::c_char = find_line_end(ptr);
                        while ptr < line_end {
                            let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            if !fuzzy_match_str_in_line(
                                &raw mut ptr,
                                leader,
                                &raw mut len,
                                ::core::ptr::null_mut::<pos_T>(),
                                &raw mut score,
                            ) {
                                continue;
                            }
                            let mut end_ptr: *mut ::core::ffi::c_char =
                                if ctrl_x_mode_line_or_eval() as ::core::ffi::c_int != 0 {
                                    find_line_end(ptr)
                                } else {
                                    find_word_end(ptr)
                                };
                            let mut add_r: ::core::ffi::c_int = ins_compl_add_infercase(
                                ptr,
                                end_ptr.offset_from(ptr) as ::core::ffi::c_int,
                                p_ic.get() != 0,
                                *files.offset(i as isize),
                                *dir,
                                false_0 != 0,
                                score,
                            );
                            if add_r == FAIL {
                                break;
                            }
                            ptr = end_ptr;
                            if compl_get_longest.get() as ::core::ffi::c_int != 0
                                && ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                                && !(*compl_first_match.get()).cp_next.is_null()
                                && score == (*(*compl_first_match.get()).cp_next).cp_score
                            {
                                (*compl_num_bests.ptr()) += 1;
                            }
                        }
                    } else if !regmatch.is_null() {
                        while vim_regexec(regmatch, buf, ptr.offset_from(buf) as colnr_T) {
                            ptr = (*regmatch).startp[0 as ::core::ffi::c_int as usize];
                            ptr = if ctrl_x_mode_line_or_eval() as ::core::ffi::c_int != 0 {
                                find_line_end(ptr)
                            } else {
                                find_word_end(ptr)
                            };
                            let mut add_r_0: ::core::ffi::c_int = ins_compl_add_infercase(
                                (*regmatch).startp[0 as ::core::ffi::c_int as usize],
                                ptr.offset_from(
                                    (*regmatch).startp[0 as ::core::ffi::c_int as usize],
                                ) as ::core::ffi::c_int,
                                p_ic.get() != 0,
                                *files.offset(i as isize),
                                *dir,
                                false_0 != 0,
                                FUZZY_SCORE_NONE,
                            );
                            if thesaurus {
                                ptr = buf;
                                add_r_0 = thesaurus_add_words_in_line(
                                    *files.offset(i as isize),
                                    &raw mut ptr,
                                    *dir as ::core::ffi::c_int,
                                    (*regmatch).startp[0 as ::core::ffi::c_int as usize],
                                );
                            }
                            if add_r_0 == OK {
                                *dir = FORWARD;
                            } else if add_r_0 == FAIL {
                                break;
                            }
                            if *ptr as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                                || got_int.get() as ::core::ffi::c_int != 0
                            {
                                break;
                            }
                        }
                    }
                    line_breakcheck();
                    ins_compl_check_keys(50 as ::core::ffi::c_int, false_0 != 0);
                }
                fclose(fp);
            }
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn ins_compl_next_buf(
    mut buf: *mut buf_T,
    mut flag: ::core::ffi::c_int,
) -> *mut buf_T {
    unsafe {
        static wp: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
        if flag == 'w' as ::core::ffi::c_int {
            if buf == curbuf.get() || !win_valid(wp.get()) {
                wp.set(curwin.get());
            }
            '_c2rust_label: {
                if !(*wp.ptr()).is_null() {
                } else {
                    __assert_fail(
                        b"wp\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2872 as ::core::ffi::c_uint,
                        b"buf_T *ins_compl_next_buf(buf_T *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            loop {
                wp.set(if !(*wp.get()).w_next.is_null() {
                    (*wp.get()).w_next
                } else {
                    firstwin.get()
                });
                if wp.get() == curwin.get()
                    || !(*(*wp.get()).w_buffer).b_scanned
                        && (*wp.get()).w_config.focusable as ::core::ffi::c_int != 0
                {
                    break;
                }
            }
            buf = (*wp.get()).w_buffer;
        } else {
            loop {
                buf = if !(*buf).b_next.is_null() {
                    (*buf).b_next
                } else {
                    firstbuf.get()
                };
                if buf == curbuf.get() {
                    break;
                }
                let mut skip_buffer: bool = false;
                if flag == 'U' as ::core::ffi::c_int {
                    skip_buffer = (*buf).b_p_bl != 0;
                } else {
                    skip_buffer = (*buf).b_p_bl == 0
                        || (*buf).b_ml.ml_mfp.is_null() as ::core::ffi::c_int
                            != (flag == 'u' as ::core::ffi::c_int) as ::core::ffi::c_int;
                }
                if !skip_buffer && !(*buf).b_scanned {
                    break;
                }
            }
        }
        return buf;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_get_next_word_or_line(
    mut ins_buf: *mut buf_T,
    mut cur_match_pos: *mut pos_T,
    mut match_len: *mut ::core::ffi::c_int,
    mut cont_s_ipos: *mut bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        *match_len = 0 as ::core::ffi::c_int;
        let mut ptr: *mut ::core::ffi::c_char =
            ml_get_buf(ins_buf, (*cur_match_pos).lnum).offset((*cur_match_pos).col as isize);
        let mut len: ::core::ffi::c_int = ml_get_buf_len(ins_buf, (*cur_match_pos).lnum)
            - (*cur_match_pos).col as ::core::ffi::c_int;
        if ctrl_x_mode_line_or_eval() {
            if compl_status_adding() {
                if (*cur_match_pos).lnum >= (*ins_buf).b_ml.ml_line_count {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                ptr = ml_get_buf(ins_buf, (*cur_match_pos).lnum + 1 as linenr_T);
                len = ml_get_buf_len(ins_buf, (*cur_match_pos).lnum + 1 as linenr_T)
                    as ::core::ffi::c_int;
                if p_paste.get() == 0 {
                    let mut tmp_ptr: *mut ::core::ffi::c_char = ptr;
                    ptr = skipwhite(tmp_ptr);
                    len -= ptr.offset_from(tmp_ptr) as ::core::ffi::c_int;
                }
            }
        } else {
            let mut tmp_ptr_0: *mut ::core::ffi::c_char = ptr;
            if compl_status_adding() as ::core::ffi::c_int != 0 && compl_length.get() <= len {
                tmp_ptr_0 = tmp_ptr_0.offset(compl_length.get() as isize);
                if vim_iswordp(tmp_ptr_0) {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                tmp_ptr_0 = find_word_start(tmp_ptr_0);
            }
            tmp_ptr_0 = find_word_end(tmp_ptr_0);
            len = tmp_ptr_0.offset_from(ptr) as ::core::ffi::c_int;
            if compl_status_adding() as ::core::ffi::c_int != 0 && len == compl_length.get() {
                if (*cur_match_pos).lnum < (*ins_buf).b_ml.ml_line_count {
                    strncpy(IObuff.ptr() as *mut ::core::ffi::c_char, ptr, len as size_t);
                    ptr = ml_get_buf(ins_buf, (*cur_match_pos).lnum + 1 as linenr_T);
                    ptr = skipwhite(ptr);
                    tmp_ptr_0 = ptr;
                    tmp_ptr_0 = find_word_start(tmp_ptr_0);
                    tmp_ptr_0 = find_word_end(tmp_ptr_0);
                    if tmp_ptr_0 > ptr {
                        if *ptr as ::core::ffi::c_int != ')' as ::core::ffi::c_int
                            && (*IObuff.ptr())[(len - 1 as ::core::ffi::c_int) as usize]
                                as ::core::ffi::c_int
                                != TAB
                        {
                            if (*IObuff.ptr())[(len - 1 as ::core::ffi::c_int) as usize]
                                as ::core::ffi::c_int
                                != ' ' as ::core::ffi::c_int
                            {
                                let c2rust_fresh3 = len;
                                len = len + 1;
                                (*IObuff.ptr())[c2rust_fresh3 as usize] =
                                    ' ' as ::core::ffi::c_char;
                            }
                            if p_js.get() != 0
                                && ((*IObuff.ptr())[(len - 2 as ::core::ffi::c_int) as usize]
                                    as ::core::ffi::c_int
                                    == '.' as ::core::ffi::c_int
                                    || (*IObuff.ptr())[(len - 2 as ::core::ffi::c_int) as usize]
                                        as ::core::ffi::c_int
                                        == '?' as ::core::ffi::c_int
                                    || (*IObuff.ptr())[(len - 2 as ::core::ffi::c_int) as usize]
                                        as ::core::ffi::c_int
                                        == '!' as ::core::ffi::c_int)
                            {
                                let c2rust_fresh4 = len;
                                len = len + 1;
                                (*IObuff.ptr())[c2rust_fresh4 as usize] =
                                    ' ' as ::core::ffi::c_char;
                            }
                        }
                        if tmp_ptr_0.offset_from(ptr) >= (IOSIZE - len) as isize {
                            tmp_ptr_0 = ptr
                                .offset(IOSIZE as isize)
                                .offset(-(len as isize))
                                .offset(-(1 as ::core::ffi::c_int as isize));
                        }
                        xstrlcpy(
                            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                            ptr,
                            (IOSIZE - len) as size_t,
                        );
                        len += tmp_ptr_0.offset_from(ptr) as ::core::ffi::c_int;
                        *cont_s_ipos = true_0 != 0;
                    }
                    (*IObuff.ptr())[len as usize] = NUL as ::core::ffi::c_char;
                    ptr = IObuff.ptr() as *mut ::core::ffi::c_char;
                }
                if len == compl_length.get() {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
        }
        *match_len = len;
        return ptr;
    }
}

pub(crate) unsafe extern "C" fn get_next_default_completion(
    mut st: *mut ins_compl_next_state_T,
    mut start_pos: *mut pos_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut in_fuzzy_collect: bool = !compl_status_adding()
            && cot_fuzzy() as ::core::ffi::c_int != 0
            && compl_length.get() > 0 as ::core::ffi::c_int;
        let mut leader: *mut ::core::ffi::c_char = ins_compl_leader();
        let mut score: ::core::ffi::c_int = FUZZY_SCORE_NONE;
        let in_curbuf: bool = (*st).ins_buf == curbuf.get();
        let save_p_scs: ::core::ffi::c_int = p_scs.get();
        '_c2rust_label: {
            if !(*st).ins_buf.is_null() {
            } else {
                __assert_fail(
                    b"st->ins_buf\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4275 as ::core::ffi::c_uint,
                    b"int get_next_default_completion(ins_compl_next_state_T *, pos_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if (*(*st).ins_buf).b_p_inf != 0 {
            p_scs.set(false_0);
        }
        let save_p_ws: ::core::ffi::c_int = p_ws.get();
        if !in_curbuf {
            p_ws.set(false_0);
        } else if *(*st).e_cpt as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            p_ws.set(true_0);
        }
        let mut looped_around: bool = false_0 != 0;
        let mut found_new_match: ::core::ffi::c_int = FAIL;
        loop {
            let mut cont_s_ipos: bool = false_0 != 0;
            (*msg_silent.ptr()) += 1;
            if in_fuzzy_collect {
                let dir = compl_direction.get() as ::core::ffi::c_int;
                let pos = (*st).cur_match_pos;
                let m = search_for_fuzzy_match((*st).ins_buf, pos, leader, dir, start_pos);
                found_new_match = FAIL;
                if let Some(hit) = m {
                    (ptr, len) = (hit.ptr, hit.len);
                    score = hit.score.unwrap_or(score);
                    found_new_match = true_0;
                }
            } else if ctrl_x_mode_whole_line() as ::core::ffi::c_int != 0
                || ctrl_x_mode_eval() as ::core::ffi::c_int != 0
                || compl_cont_status.get() & CONT_SOL != 0
            {
                found_new_match = search_for_exact_line(
                    (*st).ins_buf,
                    (*st).cur_match_pos,
                    compl_direction.get(),
                    (*compl_pattern.ptr()).data,
                );
            } else {
                found_new_match = searchit(
                    ::core::ptr::null_mut::<win_T>(),
                    (*st).ins_buf,
                    (*st).cur_match_pos,
                    ::core::ptr::null_mut::<pos_T>(),
                    compl_direction.get(),
                    (*compl_pattern.ptr()).data,
                    (*compl_pattern.ptr()).size,
                    1 as ::core::ffi::c_int,
                    SEARCH_KEEP as ::core::ffi::c_int + SEARCH_NFMSG as ::core::ffi::c_int,
                    RE_LAST as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<searchit_arg_T>(),
                );
            }
            (*msg_silent.ptr()) -= 1;
            if !compl_started.get() || (*st).set_match_pos as ::core::ffi::c_int != 0 {
                compl_started.set(true_0 != 0);
                (*st).first_match_pos = *(*st).cur_match_pos;
                (*st).last_match_pos = *(*st).cur_match_pos;
                (*st).set_match_pos = false_0 != 0;
            } else if (*st).first_match_pos.lnum == (*st).last_match_pos.lnum
                && (*st).first_match_pos.col == (*st).last_match_pos.col
            {
                found_new_match = FAIL;
            } else if compl_dir_forward() as ::core::ffi::c_int != 0
                && ((*st).prev_match_pos.lnum > (*(*st).cur_match_pos).lnum
                    || (*st).prev_match_pos.lnum == (*(*st).cur_match_pos).lnum
                        && (*st).prev_match_pos.col >= (*(*st).cur_match_pos).col)
            {
                if looped_around {
                    found_new_match = FAIL;
                } else {
                    looped_around = true_0 != 0;
                }
            } else if !compl_dir_forward()
                && ((*st).prev_match_pos.lnum < (*(*st).cur_match_pos).lnum
                    || (*st).prev_match_pos.lnum == (*(*st).cur_match_pos).lnum
                        && (*st).prev_match_pos.col <= (*(*st).cur_match_pos).col)
            {
                if looped_around {
                    found_new_match = FAIL;
                } else {
                    looped_around = true_0 != 0;
                }
            }
            (*st).prev_match_pos = *(*st).cur_match_pos;
            if found_new_match == FAIL {
                break;
            }
            if compl_status_adding() as ::core::ffi::c_int != 0
                && in_curbuf as ::core::ffi::c_int != 0
                && (*start_pos).lnum == (*(*st).cur_match_pos).lnum
                && (*start_pos).col == (*(*st).cur_match_pos).col
            {
                continue;
            }
            if !in_fuzzy_collect {
                ptr = ins_compl_get_next_word_or_line(
                    (*st).ins_buf,
                    (*st).cur_match_pos,
                    &raw mut len,
                    &raw mut cont_s_ipos,
                );
            }
            if ptr.is_null()
                || ins_compl_has_preinsert() as ::core::ffi::c_int != 0
                    && strcmp(ptr, ins_compl_leader()) == 0 as ::core::ffi::c_int
            {
                continue;
            }
            if is_nearest_active() as ::core::ffi::c_int != 0
                && in_curbuf as ::core::ffi::c_int != 0
            {
                score = ((*(*st).cur_match_pos).lnum - (*curwin.get()).w_cursor.lnum)
                    as ::core::ffi::c_int;
                if score < 0 as ::core::ffi::c_int {
                    score = -score;
                }
            }
            if ins_compl_add_infercase(
                ptr,
                len,
                p_ic.get() != 0,
                if in_curbuf as ::core::ffi::c_int != 0 {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    (*(*st).ins_buf).b_sfname
                },
                kDirectionNotSet,
                cont_s_ipos,
                score,
            ) == NOTDONE
            {
                continue;
            }
            if in_fuzzy_collect as ::core::ffi::c_int != 0
                && score == (*(*compl_first_match.get()).cp_next).cp_score
            {
                (*compl_num_bests.ptr()) += 1;
            }
            found_new_match = OK;
            break;
        }
        p_scs.set(save_p_scs);
        p_ws.set(save_p_ws);
        return found_new_match;
    }
}

pub(crate) unsafe extern "C" fn get_register_completion() {
    unsafe {
        let mut dir: Direction = compl_direction.get();
        let mut adding_mode: bool = compl_status_adding();
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < NUM_REGISTERS {
            let mut regname: ::core::ffi::c_int = get_register_name(i);
            if !(!valid_yank_reg(regname, false_0 != 0) || regname == '_' as ::core::ffi::c_int) {
                let mut reg: *mut yankreg_T = copy_register(regname);
                if (*reg).y_array.is_null() || (*reg).y_size == 0 as size_t {
                    free_register(reg);
                    xfree(reg as *mut ::core::ffi::c_void);
                } else {
                    let mut j: size_t = 0 as size_t;
                    while j < (*reg).y_size {
                        let mut str: *mut ::core::ffi::c_char =
                            (*(*reg).y_array.offset(j as isize)).data;
                        if !str.is_null() {
                            if adding_mode {
                                let mut str_len: ::core::ffi::c_int =
                                    strlen(str) as ::core::ffi::c_int;
                                if str_len != 0 as ::core::ffi::c_int {
                                    if (*compl_orig_text.ptr()).data.is_null()
                                        || (if p_ic.get() != 0 {
                                            (strncasecmp(
                                                str,
                                                (*compl_orig_text.ptr()).data,
                                                (*compl_orig_text.ptr()).size,
                                            ) == 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        } else {
                                            (strncmp(
                                                str,
                                                (*compl_orig_text.ptr()).data,
                                                (*compl_orig_text.ptr()).size,
                                            ) == 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        }) != 0
                                    {
                                        if ins_compl_add_infercase(
                                            str,
                                            str_len,
                                            p_ic.get() != 0,
                                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            dir,
                                            false_0 != 0,
                                            FUZZY_SCORE_NONE,
                                        ) == OK
                                        {
                                            dir = FORWARD;
                                        }
                                    }
                                }
                            } else {
                                let mut str_end: *mut ::core::ffi::c_char =
                                    str.offset(strlen(str) as isize);
                                let mut p: *mut ::core::ffi::c_char = str;
                                while p < str_end && *p as ::core::ffi::c_int != NUL {
                                    let mut old_p: *mut ::core::ffi::c_char = p;
                                    p = find_word_start(p);
                                    if p >= str_end || *p as ::core::ffi::c_int == NUL {
                                        break;
                                    }
                                    let mut word_end: *mut ::core::ffi::c_char = find_word_end(p);
                                    if word_end <= p {
                                        word_end = p.offset(utfc_ptr2len(p) as isize);
                                    }
                                    if word_end > str_end {
                                        word_end = str_end;
                                    }
                                    let mut len: ::core::ffi::c_int =
                                        word_end.offset_from(p) as ::core::ffi::c_int;
                                    if len > 0 as ::core::ffi::c_int
                                        && ((*compl_orig_text.ptr()).data.is_null()
                                            || (if p_ic.get() != 0 {
                                                (strncasecmp(
                                                    p,
                                                    (*compl_orig_text.ptr()).data,
                                                    (*compl_orig_text.ptr()).size,
                                                ) == 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                            } else {
                                                (strncmp(
                                                    p,
                                                    (*compl_orig_text.ptr()).data,
                                                    (*compl_orig_text.ptr()).size,
                                                ) == 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                            }) != 0)
                                    {
                                        if ins_compl_add_infercase(
                                            p,
                                            len,
                                            p_ic.get() != 0,
                                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            dir,
                                            false_0 != 0,
                                            FUZZY_SCORE_NONE,
                                        ) == OK
                                        {
                                            dir = FORWARD;
                                        }
                                    }
                                    p = word_end;
                                    if p <= old_p {
                                        p = old_p.offset(utfc_ptr2len(old_p) as isize);
                                    }
                                }
                            }
                        }
                        j = j.wrapping_add(1);
                    }
                    free_register(reg);
                    xfree(reg as *mut ::core::ffi::c_void);
                }
            }
            i += 1;
        }
    }
}
