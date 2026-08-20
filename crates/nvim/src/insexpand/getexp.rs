//! The collection driver: one pass over `'complete'`, one source at a time.
//!
//! [`ins_compl_get_exp`] is the loop — it asks [`process_next_cpt_value`] for
//! the next `'complete'` entry, calls the `get_next_*_completion` function
//! that entry names, and keeps going until it has enough matches or runs out
//! of sources.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::Expanded;
use crate::path::ExpandFlags;
use crate::types::{FAIL, IOSIZE, NUL, OK, ShmFlag};

/// In large buffers a timeout can miss nearby matches, so the search starts
/// this many lines above the cursor.
const LOOKBACK_LINE_COUNT: linenr_T = 1000;

/// Thesaurus completion goes through a function rather than a word list:
/// `'thesaurusfunc'` is set.
pub(crate) unsafe fn thesaurus_func_complete(type_0: c_int) -> bool {
    unsafe {
        type_0 == CTRL_X_THESAURUS
            && (*(*curbuf.get()).b_p_tsrfu as c_int != NUL || *p_tsrfu.get() as c_int != NUL)
    }
}

/// Is there another `'complete'` entry after `cpt`, so the source index should
/// move on?
pub(crate) unsafe fn may_advance_cpt_index(cpt: *const c_char) -> bool {
    unsafe {
        if cpt_sources_index.get() == -1 {
            return false;
        }
        let mut p = cpt;
        while *p as c_int == ',' as c_int || *p as c_int == ' ' as c_int {
            p = p.offset(1);
        }
        *p as c_int != NUL
    }
}

/// Get the next entry from `'complete'` (`st.e_cpt`) and set up `st` for it.
///
/// Writes the CTRL-X mode the entry stands for to `compl_type_arg` and whether
/// the source index should advance to `advance_cpt_idx`. Returns
/// `INS_COMPL_CPT_OK` when the entry is ready to collect from,
/// `INS_COMPL_CPT_CONT` to skip it, `INS_COMPL_CPT_END` when `'complete'` is
/// exhausted.
pub(crate) unsafe fn process_next_cpt_value(
    st: *mut ins_compl_next_state_T,
    compl_type_arg: *mut c_int,
    start_match_pos: *mut pos_T,
    fuzzy_collect: bool,
    advance_cpt_idx: *mut bool,
) -> c_int {
    unsafe {
        let mut compl_type = -1;
        let mut status = INS_COMPL_CPT_OK;
        let skip_source = compl_autocomplete.get() && compl_from_nonkeyword.get();

        (*st).found_all = false;
        *advance_cpt_idx = false;

        while *(*st).e_cpt as c_int == ',' as c_int || *(*st).e_cpt as c_int == ' ' as c_int {
            (*st).e_cpt = (*st).e_cpt.offset(1);
        }

        'done: {
            if *(*st).e_cpt as c_int == '.' as c_int
                && !(*curbuf.get()).b_scanned
                && !skip_source
                && !compl_time_slice_expired.get()
            {
                (*st).ins_buf = curbuf.get();
                (*st).first_match_pos = *start_match_pos;
                // Move the cursor back one character so that CTRL-N can match
                // the word immediately after the cursor.
                if ctrl_x_mode_normal() && !fuzzy_collect && dec(&raw mut (*st).first_match_pos) < 0
                {
                    // Move to after the last character in the buffer, so that
                    // a word at the start of it is found correctly.
                    (*st).first_match_pos.lnum = (*(*st).ins_buf).b_ml.ml_line_count;
                    (*st).first_match_pos.col = ml_get_len((*st).first_match_pos.lnum);
                }
                (*st).last_match_pos = (*st).first_match_pos;
                compl_type = 0;
                // Remember the first match, so the loop stops when the search
                // wraps and comes back to it a second time.
                (*st).set_match_pos = true;
            } else if !skip_source
                && !compl_time_slice_expired.get()
                && !vim_strchr(c"buwU".as_ptr(), *(*st).e_cpt as uint8_t as c_int).is_null()
                && {
                    (*st).ins_buf = ins_compl_next_buf((*st).ins_buf, *(*st).e_cpt as c_int);
                    (*st).ins_buf != curbuf.get()
                }
            {
                // Scan a buffer, but not the current one.
                if !(*(*st).ins_buf).b_ml.ml_mfp.is_null() {
                    // Loaded buffer.
                    compl_started.set(true);
                    (*st).first_match_pos.col = 0;
                    (*st).last_match_pos.col = 0;
                    (*st).first_match_pos.lnum = (*(*st).ins_buf).b_ml.ml_line_count + 1;
                    (*st).last_match_pos.lnum = 0;
                    compl_type = 0;
                } else {
                    // Unloaded buffer: scan it like a dictionary.
                    (*st).found_all = true;
                    if (*(*st).ins_buf).b_fname.is_null() {
                        status = INS_COMPL_CPT_CONT;
                        break 'done;
                    }
                    compl_type = CTRL_X_DICTIONARY;
                    (*st).dict = (*(*st).ins_buf).b_fname;
                    (*st).dict_f = DICT_EXACT;
                }
                if !shortmess(ShmFlag::COMPLETIONSCAN) && !compl_autocomplete.get() {
                    let buf = (*st).ins_buf;
                    vim_snprintf(
                        IObuff.ptr() as *mut c_char,
                        IOSIZE as size_t,
                        gettext(c"Scanning: %s".as_ptr()),
                        if (*buf).b_fname.is_null() {
                            buf_spname(buf)
                        } else if (*buf).b_sfname.is_null() {
                            (*buf).b_fname
                        } else {
                            (*buf).b_sfname
                        },
                    );
                    msg_progress(
                        IObuff.ptr() as *mut c_char,
                        c"completion".as_ptr().cast_mut(),
                        c"running".as_ptr().cast_mut(),
                        HLF_R,
                        false,
                        true,
                    );
                }
            } else if *(*st).e_cpt as c_int == NUL {
                status = INS_COMPL_CPT_END;
            } else {
                if ctrl_x_mode_line_or_eval() {
                    // compl_type stays -1.
                } else if *(*st).e_cpt as c_int == 'F' as c_int
                    || *(*st).e_cpt as c_int == 'o' as c_int
                {
                    compl_type = CTRL_X_FUNCTION;
                    (*st).func_cb = get_callback_if_cpt_func((*st).e_cpt, cpt_sources_index.get());
                    if (*st).func_cb.is_null() {
                        compl_type = -1;
                    }
                } else if !skip_source {
                    let flag = *(*st).e_cpt as c_int;
                    if flag == 'k' as c_int || flag == 's' as c_int {
                        compl_type = if flag == 'k' as c_int {
                            CTRL_X_DICTIONARY
                        } else {
                            CTRL_X_THESAURUS
                        };
                        // C's `*++st->e_cpt`: a name may follow the flag.
                        (*st).e_cpt = (*st).e_cpt.offset(1);
                        if *(*st).e_cpt as c_int != ',' as c_int && *(*st).e_cpt as c_int != NUL {
                            (*st).dict = (*st).e_cpt;
                            (*st).dict_f = DICT_FIRST;
                        }
                    } else if flag == 'i' as c_int {
                        compl_type = CTRL_X_PATH_PATTERNS;
                    } else if flag == 'd' as c_int {
                        compl_type = CTRL_X_PATH_DEFINES;
                    } else if flag == 'f' as c_int {
                        compl_type = CTRL_X_BUFNAMES;
                    } else if flag == ']' as c_int || flag == 't' as c_int {
                        compl_type = CTRL_X_TAGS;
                        if !shortmess(ShmFlag::COMPLETIONSCAN) && !compl_autocomplete.get() {
                            vim_snprintf(
                                IObuff.ptr() as *mut c_char,
                                IOSIZE as size_t,
                                c"%s".as_ptr(),
                                gettext(c"Scanning tags.".as_ptr()),
                            );
                            msg_progress(
                                IObuff.ptr() as *mut c_char,
                                c"completion".as_ptr().cast_mut(),
                                c"running".as_ptr().cast_mut(),
                                HLF_R,
                                false,
                                true,
                            );
                        }
                    }
                }

                // In any case `e_cpt` advances to the next entry.
                copy_option_part(
                    &raw mut (*st).e_cpt,
                    IObuff.ptr() as *mut c_char,
                    IOSIZE as size_t,
                    c",".as_ptr().cast_mut(),
                );
                *advance_cpt_idx = may_advance_cpt_index((*st).e_cpt);

                (*st).found_all = true;
                if compl_type == -1 {
                    status = INS_COMPL_CPT_CONT;
                }
            }
        }

        *compl_type_arg = compl_type;
        status
    }
}

/// Identifiers (`i`) or defines (`d`) from included files.
pub(crate) unsafe fn get_next_include_file_completion(compl_type: c_int) {
    unsafe {
        let pattern = compl_pattern.get();
        find_pattern_in_path(
            pattern.data(),
            compl_direction.get(),
            pattern.len(),
            false,
            false,
            if compl_type == CTRL_X_PATH_DEFINES && compl_cont_status.get() & CONT_SOL == 0 {
                FIND_DEFINE
            } else {
                FIND_ANY
            },
            1,
            ACTION_EXPAND,
            1,
            MAXLNUM as linenr_T,
            false,
            compl_autocomplete.get(),
        );
    }
}

/// Words from `'dictionary'` (`k`) or `'thesaurus'` (`s`) files.
pub(crate) unsafe fn get_next_dict_tsr_completion(
    compl_type: c_int,
    dict: *mut c_char,
    dict_f: c_int,
) {
    unsafe {
        let pattern = compl_pattern.get().data();
        if thesaurus_func_complete(compl_type) {
            expand_by_function(compl_type, pattern, ptr::null_mut());
            return;
        }
        let files = if !dict.is_null() {
            dict
        } else if compl_type == CTRL_X_THESAURUS {
            if *(*curbuf.get()).b_p_tsr as c_int == NUL {
                p_tsr.get()
            } else {
                (*curbuf.get()).b_p_tsr
            }
        } else if *(*curbuf.get()).b_p_dict as c_int == NUL {
            p_dict.get()
        } else {
            (*curbuf.get()).b_p_dict
        };
        ins_compl_dictionaries(
            files,
            pattern,
            if dict.is_null() { 0 } else { dict_f },
            compl_type == CTRL_X_THESAURUS,
        );
    }
}

/// Tag names matching `compl_pattern`, up to `TAG_MANY` of them.
pub(crate) unsafe fn get_next_tag_completion() {
    unsafe {
        // Set `p_ic` from `p_ic`, `p_scs` and the pattern, for `find_tags`.
        let save_p_ic = p_ic.get();
        p_ic.set(ignorecase(compl_pattern.get().data()));
        g_tag_at_cursor.set(true);

        let mut matches: *mut *mut c_char = ptr::null_mut();
        let mut num_matches = 0;
        // Bounded to TAG_MANY, which is what stops an empty pattern finding
        // an enormous number of matches.
        if find_tags(
            compl_pattern.get().data(),
            &raw mut num_matches,
            &raw mut matches,
            TAG_REGEXP
                | TAG_NAMES
                | TAG_NOIC
                | TAG_INS_COMP
                | if ctrl_x_mode_not_default() {
                    TAG_VERBOSE
                } else {
                    0
                },
            TAG_MANY,
            (*curbuf.get()).b_ffname,
        ) == OK
            && num_matches > 0
        {
            ins_compl_add_matches(num_matches, matches, p_ic.get());
        }

        g_tag_at_cursor.set(false);
        p_ic.set(save_p_ic);
    }
}

/// File names matching `compl_pattern`, fuzzily when `'completeopt'` asks.
pub(crate) unsafe fn get_next_filename_completion() {
    unsafe {
        let mut matches: *mut *mut c_char = ptr::null_mut();
        let mut num_matches = 0;
        let mut leader = ins_compl_leader();
        let mut leader_len = ins_compl_leader_len();
        let mut in_fuzzy_collect = cot_fuzzy() && leader_len > 0;
        let need_collect_bests = in_fuzzy_collect && compl_get_longest.get();
        let mut max_score = 0;
        let mut dir = compl_direction.get();

        // Fuzzy matching is done over the whole directory, so the pattern is
        // widened to a wildcard and the leader keeps only the last component.
        if in_fuzzy_collect {
            let last_sep = strrchr(leader, PATHSEP);
            if last_sep.is_null() {
                // No path separator: match everything in the current dir.
                xfree(compl_pattern.get().data().cast::<c_void>());
                compl_pattern.set(cbuf_to_string(c"*".as_ptr(), 1));
            } else if *last_sep.offset(1) as c_int == NUL {
                // The leader ends in a separator: nothing to fuzzy-match.
                in_fuzzy_collect = false;
            } else {
                let path_len = last_sep.offset_from(leader) as size_t + 1;
                let path_with_wildcard = xmalloc(path_len + 2) as *mut c_char;
                vim_snprintf(
                    path_with_wildcard,
                    path_len + 2,
                    c"%*.*s*".as_ptr(),
                    path_len as c_int,
                    path_len as c_int,
                    leader,
                );
                xfree(compl_pattern.get().data().cast::<c_void>());
                compl_pattern.set(String_0::from_raw_parts(path_with_wildcard, path_len + 1));
                // Restrict the leader to the file-name part.
                leader = last_sep.offset(1);
                leader_len -= path_len;
            }
        }

        if expand_wildcards(
            1,
            (*compl_pattern.ptr()).data_mut(),
            &raw mut num_matches,
            &raw mut matches,
            ExpandFlags::FILE | ExpandFlags::DIR | ExpandFlags::ADDSLASH | ExpandFlags::SILENT,
        ) != OK
        {
            return;
        }

        // Expand `~/` so the completion shows the shortened name.
        tilde_replace(compl_pattern.get().data(), num_matches, matches);

        if in_fuzzy_collect {
            let mut fuzzy_indices = GARRAY_T_INIT;
            ga_init(&raw mut fuzzy_indices, size_of::<c_int>() as c_int, 10);
            compl_fuzzy_scores
                .set(xmalloc(size_of::<c_int>() * num_matches as size_t) as *mut c_int);

            for i in 0..num_matches {
                let score = fuzzy_match_str(*matches.offset(i as isize), leader);
                if score != FUZZY_SCORE_NONE {
                    ga_grow(&raw mut fuzzy_indices, 1);
                    *(fuzzy_indices.ga_data as *mut c_int).offset(fuzzy_indices.ga_len as isize) =
                        i;
                    fuzzy_indices.ga_len += 1;
                    *compl_fuzzy_scores.get().offset(i as isize) = score;
                }
            }

            if fuzzy_indices.ga_len > 0 {
                let indices = fuzzy_indices.ga_data as *mut c_int;
                qsort(
                    indices.cast::<c_void>(),
                    fuzzy_indices.ga_len as size_t,
                    size_of::<c_int>(),
                    Some(compare_scores),
                );
                for i in 0..fuzzy_indices.ga_len as isize {
                    let idx = *indices.offset(i) as isize;
                    let current_score = *compl_fuzzy_scores.get().offset(idx);
                    if ins_compl_add(
                        *matches.offset(idx),
                        -1,
                        ptr::null_mut(),
                        ptr::null(),
                        false,
                        ptr::null_mut(),
                        dir,
                        CP_FAST
                            | if p_fic.get() != 0 || p_wic.get() != 0 {
                                CP_ICASE
                            } else {
                                0
                            },
                        false,
                        ptr::null(),
                        current_score,
                    ) == OK
                    {
                        dir = FORWARD;
                    }
                    if need_collect_bests && (i == 0 || current_score == max_score) {
                        compl_num_bests.set(compl_num_bests.get() + 1);
                        max_score = current_score;
                    }
                }
                free_wild(num_matches, matches);
            } else if leader_len > 0 {
                free_wild(num_matches, matches);
                num_matches = 0;
            }

            xfree(compl_fuzzy_scores.get().cast::<c_void>());
            ga_clear(&raw mut fuzzy_indices);
            if compl_num_bests.get() > 0 && compl_get_longest.get() {
                fuzzy_longest_match();
            }
            return;
        }

        if num_matches > 0 {
            ins_compl_add_matches(
                num_matches,
                matches,
                c_int::from(p_fic.get() != 0 || p_wic.get() != 0),
            );
        }
    }
}

/// Vim command-line completion (`CTRL-X CTRL-V`).
pub(crate) unsafe fn get_next_cmdline_completion() {
    unsafe {
        let mut matches: *mut *mut c_char = ptr::null_mut();
        let mut num_matches = 0;
        let pattern = compl_pattern.get();
        if expand_cmdline(
            compl_xp.ptr(),
            pattern.data(),
            pattern.len() as c_int,
            &raw mut num_matches,
            &raw mut matches,
        ) == Expanded::Ok
        {
            ins_compl_add_matches(num_matches, matches, 0);
        }
    }
}

/// Spelling suggestions for the bad word at `lnum`.
pub(crate) unsafe fn get_next_spell_completion(lnum: linenr_T) {
    unsafe {
        let mut matches: *mut *mut c_char = ptr::null_mut();
        let num_matches = expand_spelling(lnum, compl_pattern.get().data(), &raw mut matches);
        if num_matches > 0 {
            ins_compl_add_matches(num_matches, matches, p_ic.get());
        } else {
            xfree(matches.cast::<c_void>());
        }
    }
}

/// Collect one source's worth of matches for `type_0`.
///
/// Returns true when a new match was found.
pub(crate) unsafe fn get_next_completion_match(
    type_0: c_int,
    st: *mut ins_compl_next_state_T,
    ini: *mut pos_T,
) -> bool {
    unsafe {
        let mut found_new_match = FAIL;
        match type_0 {
            // No source: `process_next_cpt_value` rejected this entry.
            -1 => {}
            CTRL_X_PATH_PATTERNS | CTRL_X_PATH_DEFINES => {
                get_next_include_file_completion(type_0);
            }
            CTRL_X_DICTIONARY | CTRL_X_THESAURUS => {
                get_next_dict_tsr_completion(type_0, (*st).dict, (*st).dict_f);
                (*st).dict = ptr::null_mut();
            }
            CTRL_X_TAGS => get_next_tag_completion(),
            CTRL_X_FILES => get_next_filename_completion(),
            CTRL_X_CMDLINE | CTRL_X_CMDLINE_CTRL_X => get_next_cmdline_completion(),
            CTRL_X_FUNCTION => {
                if ctrl_x_mode_normal() {
                    // Invoked by an `F`/`o` entry in 'complete'.
                    get_cpt_func_completion_matches((*st).func_cb);
                } else {
                    expand_by_function(type_0, compl_pattern.get().data(), ptr::null_mut());
                }
            }
            CTRL_X_OMNI => {
                expand_by_function(type_0, compl_pattern.get().data(), ptr::null_mut());
            }
            CTRL_X_SPELL => get_next_spell_completion((*st).first_match_pos.lnum),
            CTRL_X_BUFNAMES => get_next_bufname_token(),
            CTRL_X_REGISTER => get_register_completion(),
            // Normal CTRL-P/CTRL-N and CTRL-X CTRL-L.
            _ => {
                found_new_match = get_next_default_completion(st, ini);
                if found_new_match == FAIL && (*st).ins_buf == curbuf.get() {
                    (*st).found_all = true;
                }
            }
        }
        if type_0 != 0 && compl_curr_match.get() != compl_old_match.get() {
            found_new_match = OK;
        }
        found_new_match != 0
    }
}

/// Start the per-source time slice, where a timeout is configured at all.
pub(crate) unsafe fn compl_source_start_timer(source_idx: c_int) {
    unsafe {
        if compl_autocomplete.get() || p_cto.get() > 0 {
            (*cpt_sources_array.get().offset(source_idx as isize)).compl_start_tv = os_hrtime();
            compl_time_slice_expired.set(false);
        }
    }
}

/// Collect the next expansions using `compl_pattern`, starting at `ini` and
/// running in `compl_direction`.
///
/// With `compl_started` false the search starts at that position, otherwise it
/// continues where the previous call stopped. May return before every match is
/// found; the answer is the total number of matches, or −1 while that is still
/// unknown. -- Acevedo
pub(crate) unsafe fn ins_compl_get_exp(ini: *mut pos_T) -> c_int {
    unsafe {
        static st: GlobalCell<ins_compl_next_state_T> = GlobalCell::new(INS_COMPL_NEXT_STATE_INIT);
        static st_cleared: GlobalCell<bool> = GlobalCell::new(false);

        let mut found_new_match;
        let mut type_0 = ctrl_x_mode.get();
        let mut may_advance_cpt_idx = false;
        let mut start_pos = *ini;

        debug_assert!(!curbuf.get().is_null());

        if !compl_started.get() {
            let mut buf = firstbuf.get();
            while !buf.is_null() {
                (*buf).b_scanned = false;
                buf = (*buf).b_next;
            }
            if !st_cleared.get() {
                st.set(INS_COMPL_NEXT_STATE_INIT);
                st_cleared.set(true);
            }
            (*st.ptr()).found_all = false;
            (*st.ptr()).ins_buf = curbuf.get();
            xfree((*st.ptr()).e_cpt_copy.cast::<c_void>());
            // Copy 'complete', in case the buffer is wiped out.
            (*st.ptr()).e_cpt_copy = xstrdup(if compl_cont_status.get() & CONT_LOCAL != 0 {
                c".".as_ptr()
            } else {
                (*curbuf.get()).b_p_cpt
            });
            strip_caret_numbers_in_place((*st.ptr()).e_cpt_copy);
            (*st.ptr()).e_cpt = (*st.ptr()).e_cpt_copy;

            if compl_autocomplete.get() && is_nearest_active() {
                start_pos.lnum = (start_pos.lnum - LOOKBACK_LINE_COUNT).max(1);
                start_pos.col = 0;
            }
            (*st.ptr()).first_match_pos = start_pos;
            (*st.ptr()).last_match_pos = start_pos;
        } else if (*st.ptr()).ins_buf != curbuf.get() && !buf_valid((*st.ptr()).ins_buf) {
            // In case the buffer was wiped out.
            (*st.ptr()).ins_buf = curbuf.get();
        }
        debug_assert!(!(*st.ptr()).ins_buf.is_null());

        // Remember the last current match.
        compl_old_match.set(compl_curr_match.get());
        (*st.ptr()).cur_match_pos = if compl_dir_forward() {
            &raw mut (*st.ptr()).last_match_pos
        } else {
            &raw mut (*st.ptr()).first_match_pos
        };

        let normal_mode_strict = ctrl_x_mode_normal()
            && !ctrl_x_mode_line_or_eval()
            && compl_cont_status.get() & CONT_LOCAL == 0
            && !cpt_sources_array.get().is_null();
        if normal_mode_strict {
            cpt_sources_index.set(0);
            if compl_autocomplete.get() || p_cto.get() > 0 {
                compl_source_start_timer(0);
                compl_time_slice_expired.set(false);
                compl_timeout_ms.set(if compl_autocomplete.get() {
                    (COMPL_INITIAL_TIMEOUT_MS as OptInt).max(p_act.get()) as uint64_t
                } else {
                    p_cto.get() as uint64_t
                });
            }
        }

        // For CTRL-N/CTRL-P, loop over all the flags/windows/buffers in
        // 'complete'.
        loop {
            found_new_match = FAIL;
            (*st.ptr()).set_match_pos = false;

            // For CTRL-N/CTRL-P pick a new entry from `e_cpt` when
            // `compl_started` is off, or when `found_all` says this entry is
            // done. For CTRL-X CTRL-L only the entries that look in loaded
            // buffers are used.
            if (ctrl_x_mode_normal() || ctrl_x_mode_line_or_eval())
                && (!compl_started.get() || (*st.ptr()).found_all)
            {
                let status = process_next_cpt_value(
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
                    if may_advance_cpt_idx {
                        if advance_cpt_sources_index_safe() == 0 {
                            break;
                        }
                        compl_source_start_timer(cpt_sources_index.get());
                    }
                    continue;
                }
            }

            // LSP servers may sporadically take over a second to respond (for
            // instance while loading modules) but other sources may already
            // have matches, so keyword completion uses a short timeout and
            // non-keyword completion — where only function sources are active
            // — a longer one.
            let mut compl_timeout_save = 0;
            if normal_mode_strict
                && type_0 == CTRL_X_FUNCTION
                && (compl_autocomplete.get() || p_cto.get() > 0)
            {
                compl_timeout_save = compl_timeout_ms.get();
                compl_timeout_ms.set(if compl_from_nonkeyword.get() {
                    COMPL_FUNC_TIMEOUT_NON_KW_MS as uint64_t
                } else {
                    COMPL_FUNC_TIMEOUT_MS as uint64_t
                });
            }

            found_new_match = c_int::from(get_next_completion_match(
                type_0,
                st.ptr(),
                &raw mut start_pos,
            ));

            // If complete() was called then `compl_pattern` has been reset and
            // the rest of this cannot work; bail out.
            if compl_pattern.get().data().is_null() {
                break;
            }

            if may_advance_cpt_idx {
                if advance_cpt_sources_index_safe() == 0 {
                    break;
                }
                compl_source_start_timer(cpt_sources_index.get());
            }

            // Break out for the specialised modes — 'complete' is only for the
            // generic CTRL_X_NORMAL — or when a new match has been found.
            if (ctrl_x_mode_not_default() && !ctrl_x_mode_line_or_eval()) || found_new_match != FAIL
            {
                if got_int.get() {
                    break;
                }
                // Fill the popup menu as soon as possible.
                if type_0 != -1 {
                    ins_compl_check_keys(0, false);
                }
                if (ctrl_x_mode_not_default() && !ctrl_x_mode_line_or_eval())
                    || compl_interrupted.get()
                {
                    break;
                }
                compl_started.set(!compl_time_slice_expired.get());
            } else {
                // Mark a buffer scanned when it has been scanned completely.
                if buf_valid((*st.ptr()).ins_buf) && (type_0 == 0 || type_0 == CTRL_X_PATH_PATTERNS)
                {
                    debug_assert!(!(*st.ptr()).ins_buf.is_null());
                    (*(*st.ptr()).ins_buf).b_scanned = true;
                }
                compl_started.set(false);
            }

            // Restore the timeout after collecting from a function source.
            // Re-tested rather than remembered: the source just run can be a
            // user function, and that can have changed either operand.
            if normal_mode_strict
                && type_0 == CTRL_X_FUNCTION
                && (compl_autocomplete.get() || p_cto.get() > 0)
            {
                compl_timeout_ms.set(compl_timeout_save);
            }

            // For CTRL-P completion, reset `compl_curr_match` to the head, to
            // avoid mixing matches from different sources.
            if !compl_dir_forward() {
                let mut curr = compl_curr_match.get();
                while !(*curr).cp_prev.is_null() && !match_at_original_text((*curr).cp_prev) {
                    curr = (*curr).cp_prev;
                }
                compl_curr_match.set(curr);
            }
        }

        cpt_sources_index.set(-1);
        compl_started.set(true);

        if (ctrl_x_mode_normal() || ctrl_x_mode_line_or_eval())
            && *(*st.ptr()).e_cpt as c_int == NUL
        {
            // Got to the end of 'complete'.
            found_new_match = FAIL;
        }

        // Total number of matches; −1 while unknown.
        let mut match_count = -1;
        if found_new_match == FAIL || (ctrl_x_mode_not_default() && !ctrl_x_mode_line_or_eval()) {
            match_count = ins_compl_make_cyclic();
        }

        if cot_fuzzy() && compl_get_longest.get() && compl_num_bests.get() > 0 {
            fuzzy_longest_match();
        }

        if !compl_old_match.get().is_null() {
            // If several matches were added (FORWARD), or the search failed
            // and the list has just been made cyclic, `compl_curr_match` has
            // to move to the next or previous entry, if any. -- Acevedo
            let old = compl_old_match.get();
            let next = if compl_dir_forward() {
                (*old).cp_next
            } else {
                (*old).cp_prev
            };
            compl_curr_match.set(if next.is_null() { old } else { next });
        }
        may_trigger_modechanged();

        if match_count > 0 && !ctrl_x_mode_spell() {
            if is_nearest_active() && !ins_compl_has_preinsert() {
                sort_compl_match_list(Some(cp_compare_nearest));
            }
            if cot_fuzzy() && ins_compl_leader_len() > 0 {
                ins_compl_fuzzy_sort();
            }
        }

        match_count
    }
}

/// Expire the current source's time slice, halving the budget each time so a
/// slow source cannot hold up the rest.
pub(crate) unsafe fn check_elapsed_time() {
    unsafe {
        let start_tv = (*cpt_sources_array
            .get()
            .offset(cpt_sources_index.get() as isize))
        .compl_start_tv;
        let elapsed_ms = (os_hrtime() - start_tv) / 1_000_000;
        if elapsed_ms > compl_timeout_ms.get() {
            compl_time_slice_expired.set(true);
            if compl_timeout_ms.get() > COMPL_MIN_TIMEOUT_MS as uint64_t {
                compl_timeout_ms.set(compl_timeout_ms.get() / 2);
            }
        }
    }
}
