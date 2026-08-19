//! Scanning buffers, dictionaries, thesauruses and registers for matches.
//!
//! [`ins_compl_dictionaries`] and [`ins_compl_files`] are the `'dictionary'`
//! and `'thesaurus'` file walk; [`get_next_default_completion`] is the
//! keyword search through the buffers `'complete'` names, driven by
//! [`ins_compl_next_buf`]; [`get_register_completion`] is the `CTRL-X
//! CTRL-R` source.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::path::ExpandFlags;
use crate::types::{FAIL, IOSIZE, NUL, OK, ShmFlag};

/// Add every identifier matching `pat` in the `'dictionary'`-style list
/// `dict_start` to the completions.
///
/// `flags` is `DICT_FIRST` and/or `DICT_EXACT`; `thesaurus` selects thesaurus
/// completion.
pub(crate) unsafe fn ins_compl_dictionaries(
    dict_start: *mut c_char,
    pat: *mut c_char,
    flags: c_int,
    thesaurus: bool,
) {
    unsafe {
        let mut dict = dict_start;
        let mut dir = compl_direction.get();

        if *dict as c_int == NUL {
            // When 'dictionary' is empty and spell checking is enabled use
            // "spell".
            if !thesaurus && (*curwin.get()).w_onebuf_opt.wo_spell != 0 {
                dict = c"spell".as_ptr().cast_mut();
            } else {
                return;
            }
        }

        let mut buf = xmalloc(LSIZE as size_t).cast::<c_char>();
        // So that we can leave through 'theend.
        let mut regmatch = regmatch_T {
            regprog: ptr::null_mut(),
            startp: [ptr::null_mut(); 10],
            endp: [ptr::null_mut(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };

        // If 'infercase' is set, don't use 'smartcase' here.
        let save_p_scs = p_scs.get();
        if (*curbuf.get()).b_p_inf != 0 {
            p_scs.set(0);
        }

        // C's `goto theend`, i.e. free and restore below without scanning.
        'theend: {
            // When invoked to match whole lines for CTRL-X CTRL-L adjust the
            // pattern to only match at the start of a line.  Otherwise just
            // match the pattern.  Also need to double backslashes.
            if ctrl_x_mode_line_or_eval() {
                let pat_esc = vim_strsave_escaped(pat, c"\\".as_ptr());
                let len = strlen(pat_esc) + 10;
                let ptr = xmalloc(len).cast::<c_char>();
                vim_snprintf(ptr, len, c"^\\s*\\zs\\V%s".as_ptr(), pat_esc);
                regmatch.regprog = vim_regcomp(ptr, RE_MAGIC);
                xfree(pat_esc.cast::<c_void>());
                xfree(ptr.cast::<c_void>());
            } else {
                regmatch.regprog = vim_regcomp(pat, if magic_isset() { RE_MAGIC } else { 0 });
                if regmatch.regprog.is_null() {
                    break 'theend;
                }
            }

            // Ignore case depends on 'ignorecase', 'smartcase' and "pat".
            regmatch.rm_ic = ignorecase(pat) != 0;
            while *dict as c_int != NUL && !got_int.get() && !compl_interrupted.get() {
                // Copy one dictionary file name into buf.
                // Upstream leaves both uninitialised: every path that reads
                // them either assigns first or is guarded by `count > 0`.
                let mut files: *mut *mut c_char = ptr::null_mut();
                let mut count = 0;
                if flags == DICT_EXACT {
                    count = 1;
                    files = &raw mut dict;
                } else {
                    // Expand wildcards in the dictionary name, but do not allow
                    // backticks (for security, the 'dict' option may have been
                    // set in a modeline).
                    copy_option_part(
                        &raw mut dict,
                        buf,
                        LSIZE as size_t,
                        c",".as_ptr().cast_mut(),
                    );
                    if !thesaurus && strcmp(buf, c"spell".as_ptr()) == 0 {
                        count = -1;
                    } else if !vim_strchr(buf, '`' as c_int).is_null()
                        || expand_wildcards(
                            1,
                            &raw mut buf,
                            &raw mut count,
                            &raw mut files,
                            ExpandFlags::FILE | ExpandFlags::SILENT,
                        ) != OK
                    {
                        count = 0;
                    }
                }

                if count == -1 {
                    // Complete from active spelling.  Skip "\<" in the pattern,
                    // we don't use it as a RE.
                    let word = if *pat as c_int == '\\' as c_int
                        && *pat.offset(1) as c_int == '<' as c_int
                    {
                        pat.offset(2)
                    } else {
                        pat
                    };
                    spell_dump_compl(word, regmatch.rm_ic as c_int, &raw mut dir, 0);
                } else if count > 0 {
                    // Avoid a warning for using "files" uninitialised.
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
                if flags != 0 {
                    break;
                }
            }
        }

        p_scs.set(save_p_scs);
        vim_regfree(regmatch.regprog);
        xfree(buf.cast::<c_void>());
    }
}

/// Add all the words in the line `*buf_arg` from the thesaurus file `fname`,
/// skipping the word at `skip_word`; answers OK on success.
pub(crate) unsafe fn thesaurus_add_words_in_line(
    fname: *mut c_char,
    buf_arg: *mut *mut c_char,
    dir: c_int,
    skip_word: *const c_char,
) -> c_int {
    unsafe {
        let mut status = OK;

        // Add the other matches on the line.
        let mut ptr = *buf_arg;
        while !got_int.get() {
            // Find the start of the next word, skipping white space and
            // punctuation.
            ptr = find_word_start(ptr);
            if *ptr as c_int == NUL || *ptr as c_int == NL {
                break;
            }
            let wstart = ptr;

            // Find the end of the word.  Japanese words may have characters in
            // different classes, so only separate words with single-byte
            // non-word characters.
            while *ptr as c_int != NUL {
                let l = utfc_ptr2len(ptr);
                if l < 2 && !vim_iswordc(*ptr as u8 as c_int) {
                    break;
                }
                ptr = ptr.offset(l as isize);
            }

            // Add the word, skipping the regexp match.
            if wstart != skip_word.cast_mut() {
                status = ins_compl_add_infercase(
                    wstart,
                    ptr.offset_from(wstart) as c_int,
                    p_ic.get() != 0,
                    fname,
                    dir,
                    false,
                    FUZZY_SCORE_NONE,
                );
                if status == FAIL {
                    break;
                }
            }
        }

        *buf_arg = ptr;
        status
    }
}

/// Read `count` dictionary/thesaurus `files` and add the text matching
/// `regmatch`.
pub(crate) unsafe fn ins_compl_files(
    count: c_int,
    files: *mut *mut c_char,
    thesaurus: bool,
    flags: c_int,
    regmatch: *mut regmatch_T,
    buf: *mut c_char,
    dir: *mut Direction,
) {
    unsafe {
        let leader = if cot_fuzzy() {
            ins_compl_leader()
        } else {
            ptr::null_mut()
        };
        let leader_len = if cot_fuzzy() {
            ins_compl_leader_len() as c_int
        } else {
            0
        };

        let mut i = 0;
        while i < count as isize && !got_int.get() && !ins_compl_interrupted() {
            let file = *files.offset(i);
            let fp = os_fopen(file, c"r".as_ptr()); // open dictionary file
            let quiet = shortmess(ShmFlag::COMPLETIONSCAN);
            if flags != DICT_EXACT && !quiet && !compl_autocomplete.get() {
                vim_snprintf(
                    IObuff.ptr().cast::<c_char>(),
                    IOSIZE as size_t,
                    gettext(c"Scanning dictionary: %s".as_ptr()),
                    file,
                );
                msg_progress(
                    IObuff.ptr().cast::<c_char>(),
                    c"completion".as_ptr().cast_mut(),
                    c"running".as_ptr().cast_mut(),
                    HLF_R,
                    false,
                    true,
                );
            }

            if fp.is_null() {
                i += 1;
                continue;
            }

            // Read the dictionary file line by line, checking each for a match.
            while !got_int.get() && !ins_compl_interrupted() && !vim_fgets(buf, LSIZE, fp) {
                let mut ptr = buf;
                if cot_fuzzy() && leader_len > 0 {
                    let line_end = find_line_end(ptr);
                    while ptr < line_end {
                        let mut score = 0;
                        let mut len = 0;
                        if fuzzy_match_str_in_line(
                            &raw mut ptr,
                            leader,
                            &raw mut len,
                            ptr::null_mut(),
                            &raw mut score,
                        ) {
                            let end_ptr = if ctrl_x_mode_line_or_eval() {
                                find_line_end(ptr)
                            } else {
                                find_word_end(ptr)
                            };
                            let add_r = ins_compl_add_infercase(
                                ptr,
                                end_ptr.offset_from(ptr) as c_int,
                                p_ic.get() != 0,
                                file,
                                *dir,
                                false,
                                score,
                            );
                            if add_r == FAIL {
                                break;
                            }
                            ptr = end_ptr; // start from the next word
                            if compl_get_longest.get()
                                && ctrl_x_mode_normal()
                                && !(*compl_first_match.get()).cp_next.is_null()
                                && score == (*(*compl_first_match.get()).cp_next).cp_score
                            {
                                (*compl_num_bests.ptr()) += 1;
                            }
                        }
                    }
                } else if !regmatch.is_null() {
                    while vim_regexec(regmatch, buf, ptr.offset_from(buf) as colnr_T) {
                        let start = (*regmatch).startp[0];
                        ptr = if ctrl_x_mode_line_or_eval() {
                            find_line_end(start)
                        } else {
                            find_word_end(start)
                        };
                        let mut add_r = ins_compl_add_infercase(
                            start,
                            ptr.offset_from(start) as c_int,
                            p_ic.get() != 0,
                            file,
                            *dir,
                            false,
                            FUZZY_SCORE_NONE,
                        );
                        if thesaurus {
                            // For a thesaurus, add all the words in the line.
                            ptr = buf;
                            add_r = thesaurus_add_words_in_line(file, &raw mut ptr, *dir, start);
                        }
                        if add_r == OK {
                            // If dir was BACKWARD then honour it just once.
                            *dir = FORWARD;
                        } else if add_r == FAIL {
                            break;
                        }
                        // Avoid an expensive call to vim_regexec() at the end
                        // of the line.
                        if *ptr as c_int == '\n' as c_int || got_int.get() {
                            break;
                        }
                    }
                }
                line_breakcheck();
                ins_compl_check_keys(50, false);
            }
            fclose(fp);
            i += 1;
        }
    }
}

/// The next window, loaded buffer or non-loaded buffer (depending on `flag`)
/// after `buf` that has not been scanned; `curbuf` when there is none.
///
/// `curbuf` is special: called with `buf == curbuf` this has to be the first
/// call for a given flag/expansion. -- Acevedo
pub(crate) unsafe fn ins_compl_next_buf(mut buf: *mut buf_T, flag: c_int) -> *mut buf_T {
    unsafe {
        static wp: GlobalCell<*mut win_T> = GlobalCell::new(ptr::null_mut());

        if flag == 'w' as c_int {
            // Just windows.
            if buf == curbuf.get() || !win_valid(wp.get()) {
                // First call for this flag/expansion, or the window was closed.
                wp.set(curwin.get());
            }
            debug_assert!(!wp.get().is_null());
            loop {
                // Move to the next window, wrapping to the first at the end.
                wp.set(if !(*wp.get()).w_next.is_null() {
                    (*wp.get()).w_next
                } else {
                    firstwin.get()
                });
                // Stop if we're back at the start, or found an unscanned
                // buffer in a focusable window.
                if wp.get() == curwin.get()
                    || (!(*(*wp.get()).w_buffer).b_scanned && (*wp.get()).w_config.focusable)
                {
                    break;
                }
            }
            buf = (*wp.get()).w_buffer;
        } else {
            // 'b' (just loaded buffers), 'u' (just non-loaded buffers) or 'U'
            // (unlisted buffers).  When completing whole lines skip unloaded
            // buffers.
            loop {
                // Move to the next buffer, wrapping to the first at the end.
                buf = if !(*buf).b_next.is_null() {
                    (*buf).b_next
                } else {
                    firstbuf.get()
                };
                // Stop if we're back at the start buffer.
                if buf == curbuf.get() {
                    break;
                }
                let skip_buffer = if flag == 'U' as c_int {
                    (*buf).b_p_bl != 0
                } else {
                    (*buf).b_p_bl == 0 || (*buf).b_ml.ml_mfp.is_null() != (flag == 'u' as c_int)
                };
                // Stop if we found a buffer that matches our criteria.
                if !skip_buffer && !(*buf).b_scanned {
                    break;
                }
            }
        }
        buf
    }
}

/// The next word or line from `ins_buf` at `cur_match_pos`, with its length in
/// `match_len`; `cont_s_ipos` says the next `CTRL-X <>` sets the initial
/// position.
pub(crate) unsafe fn ins_compl_get_next_word_or_line(
    ins_buf: *mut buf_T,
    cur_match_pos: *mut pos_T,
    match_len: *mut c_int,
    cont_s_ipos: *mut bool,
) -> *mut c_char {
    unsafe {
        *match_len = 0;
        let mut ptr =
            ml_get_buf(ins_buf, (*cur_match_pos).lnum).offset((*cur_match_pos).col as isize);
        let mut len = ml_get_buf_len(ins_buf, (*cur_match_pos).lnum) - (*cur_match_pos).col;
        let iobuff = IObuff.ptr().cast::<c_char>();

        if ctrl_x_mode_line_or_eval() {
            if compl_status_adding() {
                if (*cur_match_pos).lnum >= (*ins_buf).b_ml.ml_line_count {
                    return ptr::null_mut();
                }
                ptr = ml_get_buf(ins_buf, (*cur_match_pos).lnum + 1);
                len = ml_get_buf_len(ins_buf, (*cur_match_pos).lnum + 1);
                if p_paste.get() == 0 {
                    let tmp_ptr = ptr;
                    ptr = skipwhite(tmp_ptr);
                    len -= ptr.offset_from(tmp_ptr) as c_int;
                }
            }
        } else {
            let mut tmp_ptr = ptr;
            if compl_status_adding() && compl_length.get() <= len {
                tmp_ptr = tmp_ptr.offset(compl_length.get() as isize);
                // Skip if already inside a word.
                if vim_iswordp(tmp_ptr) {
                    return ptr::null_mut();
                }
                // Find the start of the next word.
                tmp_ptr = find_word_start(tmp_ptr);
            }
            // Find the end of this word.
            tmp_ptr = find_word_end(tmp_ptr);
            len = tmp_ptr.offset_from(ptr) as c_int;

            if compl_status_adding() && len == compl_length.get() {
                if (*cur_match_pos).lnum < (*ins_buf).b_ml.ml_line_count {
                    // Try the next line, if any: the new word will be "joined"
                    // as if the normal command "J" was used.  IOSIZE is always
                    // greater than compl_length, so the strncpy always works
                    // -- Acevedo
                    strncpy(iobuff, ptr, len as size_t);
                    ptr = skipwhite(ml_get_buf(ins_buf, (*cur_match_pos).lnum + 1));
                    // Find the start and then the end of the next word.
                    tmp_ptr = find_word_end(find_word_start(ptr));
                    if tmp_ptr > ptr {
                        if *ptr as c_int != ')' as c_int
                            && *iobuff.offset((len - 1) as isize) as c_int != TAB
                        {
                            if *iobuff.offset((len - 1) as isize) as c_int != ' ' as c_int {
                                *iobuff.offset(len as isize) = ' ' as c_char;
                                len += 1;
                            }
                            // IObuff =~ "\k.* ", thus len >= 2.
                            if p_js.get() != 0
                                && matches!(
                                    *iobuff.offset((len - 2) as isize) as u8,
                                    b'.' | b'?' | b'!'
                                )
                            {
                                *iobuff.offset(len as isize) = ' ' as c_char;
                                len += 1;
                            }
                        }
                        // Copy as much as possible of the new word.
                        if tmp_ptr.offset_from(ptr) >= (IOSIZE - len) as isize {
                            tmp_ptr = ptr.offset((IOSIZE - len - 1) as isize);
                        }
                        xstrlcpy(iobuff.offset(len as isize), ptr, (IOSIZE - len) as size_t);
                        len += tmp_ptr.offset_from(ptr) as c_int;
                        *cont_s_ipos = true;
                    }
                    *iobuff.offset(len as isize) = NUL as c_char;
                    ptr = iobuff;
                }
                if len == compl_length.get() {
                    return ptr::null_mut();
                }
            }
        }

        *match_len = len;
        ptr
    }
}

/// The next set of words matching `compl_pattern` for default completion —
/// normal `^P`/`^N` and `^X^L`.
///
/// Searches `st->ins_buf` from `start_pos` in the `compl_direction` direction;
/// with `st->set_match_pos` set, `st->first_match_pos` and `st->last_match_pos`
/// are set too. Answers OK if a new match was found, otherwise FAIL.
pub(crate) unsafe fn get_next_default_completion(
    st: *mut ins_compl_next_state_T,
    start_pos: *mut pos_T,
) -> c_int {
    unsafe {
        let mut ptr: *mut c_char = ptr::null_mut();
        let mut len = 0;
        let in_fuzzy_collect = !compl_status_adding() && cot_fuzzy() && compl_length.get() > 0;
        let leader = ins_compl_leader();
        let mut score = FUZZY_SCORE_NONE;
        let in_curbuf = (*st).ins_buf == curbuf.get();

        // If 'infercase' is set, don't use 'smartcase' here.
        let save_p_scs = p_scs.get();
        debug_assert!(!(*st).ins_buf.is_null());
        if (*(*st).ins_buf).b_p_inf != 0 {
            p_scs.set(0);
        }

        // Buffers other than curbuf are scanned from the beginning or the end
        // but never from the middle, thus setting nowrapscan in these buffers
        // is a good idea; on the other hand, we always set wrapscan for curbuf
        // to avoid missing matches -- Acevedo, Webb
        let save_p_ws = p_ws.get();
        if !in_curbuf {
            p_ws.set(0);
        } else if *(*st).e_cpt as c_int == '.' as c_int {
            p_ws.set(1);
        }

        let mut looped_around = false;
        let mut found_new_match;
        loop {
            let mut cont_s_ipos = false;

            (*msg_silent.ptr()) += 1; // Don't want messages for wrapscan.
            if in_fuzzy_collect {
                let hit = search_for_fuzzy_match(
                    (*st).ins_buf,
                    (*st).cur_match_pos,
                    leader,
                    compl_direction.get(),
                    start_pos,
                );
                found_new_match = FAIL;
                if let Some(hit) = hit {
                    (ptr, len) = (hit.ptr, hit.len);
                    score = hit.score.unwrap_or(score);
                    found_new_match = OK;
                }
            } else if ctrl_x_mode_whole_line()
                || ctrl_x_mode_eval()
                || compl_cont_status.get() & CONT_SOL != 0
            {
                // ctrl_x_mode_line_or_eval(), or a word-wise search that has
                // added a word that was at the beginning of the line.
                found_new_match = search_for_exact_line(
                    (*st).ins_buf,
                    (*st).cur_match_pos,
                    compl_direction.get(),
                    (*compl_pattern.ptr()).data,
                );
            } else {
                found_new_match = searchit(
                    ptr::null_mut(),
                    (*st).ins_buf,
                    (*st).cur_match_pos,
                    ptr::null_mut(),
                    compl_direction.get(),
                    (*compl_pattern.ptr()).data,
                    (*compl_pattern.ptr()).size,
                    1,
                    SEARCH_KEEP + SEARCH_NFMSG,
                    RE_LAST,
                    ptr::null_mut(),
                );
            }
            (*msg_silent.ptr()) -= 1;

            let pos = *(*st).cur_match_pos;
            if !compl_started.get() || (*st).set_match_pos {
                // Set "compl_started" even on failure.
                compl_started.set(true);
                (*st).first_match_pos = pos;
                (*st).last_match_pos = pos;
                (*st).set_match_pos = false;
            } else if (*st).first_match_pos.lnum == (*st).last_match_pos.lnum
                && (*st).first_match_pos.col == (*st).last_match_pos.col
            {
                found_new_match = FAIL;
            } else {
                // Passing the previous match going forwards (or backwards) is
                // the wrap-around; the second time round there is nothing new.
                let passed = if compl_dir_forward() {
                    (*st).prev_match_pos.lnum > pos.lnum
                        || ((*st).prev_match_pos.lnum == pos.lnum
                            && (*st).prev_match_pos.col >= pos.col)
                } else {
                    (*st).prev_match_pos.lnum < pos.lnum
                        || ((*st).prev_match_pos.lnum == pos.lnum
                            && (*st).prev_match_pos.col <= pos.col)
                };
                if passed {
                    if looped_around {
                        found_new_match = FAIL;
                    } else {
                        looped_around = true;
                    }
                }
            }
            (*st).prev_match_pos = pos;
            if found_new_match == FAIL {
                break;
            }

            // When ADDING, the text before the cursor matches: skip it.
            if compl_status_adding()
                && in_curbuf
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
            if ptr.is_null() || (ins_compl_has_preinsert() && strcmp(ptr, ins_compl_leader()) == 0)
            {
                continue;
            }

            if is_nearest_active() && in_curbuf {
                score = ((*(*st).cur_match_pos).lnum - (*curwin.get()).w_cursor.lnum) as c_int;
                score = score.abs();
            }

            if ins_compl_add_infercase(
                ptr,
                len,
                p_ic.get() != 0,
                if in_curbuf {
                    ptr::null_mut()
                } else {
                    (*(*st).ins_buf).b_sfname
                },
                kDirectionNotSet,
                cont_s_ipos,
                score,
            ) != NOTDONE
            {
                if in_fuzzy_collect && score == (*(*compl_first_match.get()).cp_next).cp_score {
                    (*compl_num_bests.ptr()) += 1;
                }
                found_new_match = OK;
                break;
            }
        }

        p_scs.set(save_p_scs);
        p_ws.set(save_p_ws);
        found_new_match
    }
}

/// Add completion matches from the contents of every usable register.
pub(crate) unsafe fn get_register_completion() {
    unsafe {
        // Upstream's `!compl_orig_text.data || (p_ic ? STRNICMP : strncmp)(…)`:
        // a candidate counts when there is no original text to compare against,
        // or it starts with it.
        let starts_with_orig = |s: *mut c_char| {
            let orig = *compl_orig_text.ptr();
            orig.data.is_null()
                || if p_ic.get() != 0 {
                    strncasecmp(s, orig.data, orig.size) == 0
                } else {
                    strncmp(s, orig.data, orig.size) == 0
                }
        };

        let mut dir = compl_direction.get();
        let adding_mode = compl_status_adding();

        for i in 0..NUM_REGISTERS {
            let regname = get_register_name(i);
            // Skip an invalid or black hole register.
            if !valid_yank_reg(regname, false) || regname == '_' as c_int {
                continue;
            }

            let reg = copy_register(regname);
            if (*reg).y_array.is_null() || (*reg).y_size == 0 {
                free_register(reg);
                xfree(reg.cast::<c_void>());
                continue;
            }

            for j in 0..(*reg).y_size as isize {
                let str = (*(*reg).y_array.offset(j)).data;
                if str.is_null() {
                    continue;
                }

                if adding_mode {
                    let str_len = strlen(str) as c_int;
                    if str_len == 0 {
                        continue;
                    }
                    if starts_with_orig(str)
                        && ins_compl_add_infercase(
                            str,
                            str_len,
                            p_ic.get() != 0,
                            ptr::null_mut(),
                            dir,
                            false,
                            FUZZY_SCORE_NONE,
                        ) == OK
                    {
                        dir = FORWARD;
                    }
                } else {
                    // The safe end of the string, to avoid NUL byte issues.
                    let str_end = str.add(strlen(str));
                    let mut p = str;
                    while p < str_end && *p as c_int != NUL {
                        let old_p = p;
                        p = find_word_start(p);
                        if p >= str_end || *p as c_int == NUL {
                            break;
                        }

                        let mut word_end = find_word_end(p);
                        if word_end <= p {
                            word_end = p.offset(utfc_ptr2len(p) as isize);
                        }
                        if word_end > str_end {
                            word_end = str_end;
                        }

                        let len = word_end.offset_from(p) as c_int;
                        if len > 0
                            && starts_with_orig(p)
                            && ins_compl_add_infercase(
                                p,
                                len,
                                p_ic.get() != 0,
                                ptr::null_mut(),
                                dir,
                                false,
                                FUZZY_SCORE_NONE,
                            ) == OK
                        {
                            dir = FORWARD;
                        }

                        p = word_end;
                        if p <= old_p {
                            p = old_p.offset(utfc_ptr2len(old_p) as isize);
                        }
                    }
                }
            }

            free_register(reg);
            xfree(reg.cast::<c_void>());
        }
    }
}
