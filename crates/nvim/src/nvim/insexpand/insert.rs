//! Putting the selected match into the buffer, and moving between matches.
//!
//! [`ins_compl_insert`] writes the match at `compl_col` and
//! [`ins_compl_delete`] takes it out again; [`ins_compl_next`] is what
//! CTRL-N / CTRL-P reach, walking to the next match through
//! [`find_next_completion_match`] and asking for more when the list runs
//! out.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ins_compl_insert_bytes(
    mut p: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) {
    unsafe {
        if len == -1 as ::core::ffi::c_int {
            len = strlen(p) as ::core::ffi::c_int;
        }
        '_c2rust_label: {
            if len >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1089 as ::core::ffi::c_uint,
                    b"void ins_compl_insert_bytes(char *, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        ins_bytes_len(p, len as size_t);
        compl_ins_end_col.set((*curwin.get()).w_cursor.col);
    }
}

pub(crate) unsafe extern "C" fn ins_compl_longest_insert(mut prefix: *mut ::core::ffi::c_char) {
    unsafe {
        ins_compl_delete(false_0 != 0);
        ins_compl_insert_bytes(
            prefix.offset(get_compl_len() as isize),
            -1 as ::core::ffi::c_int,
        );
        ins_redraw(false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn fuzzy_longest_match() {
    unsafe {
        if compl_num_bests.get() == 0 as ::core::ffi::c_int {
            return;
        }
        let mut nn_compl: *mut compl_T = (*(*compl_first_match.get()).cp_next).cp_next;
        let mut more_candidates: bool = !nn_compl.is_null() && nn_compl != compl_first_match.get();
        let mut compl: *mut compl_T = if ctrl_x_mode_whole_line() as ::core::ffi::c_int != 0 {
            compl_first_match.get()
        } else {
            (*compl_first_match.get()).cp_next
        };
        if compl_num_bests.get() == 1 as ::core::ffi::c_int {
            if !more_candidates {
                ins_compl_longest_insert((*compl).cp_str.data);
                compl_num_bests.set(0 as ::core::ffi::c_int);
            }
            compl_num_bests.set(0 as ::core::ffi::c_int);
            return;
        }
        if compl_num_bests.get() as size_t
            > (SIZE_MAX as usize).wrapping_div(::core::mem::size_of::<*mut compl_T>())
        {
            return;
        }
        compl_best_matches.set(xmalloc(
            (compl_num_bests.get() as size_t).wrapping_mul(::core::mem::size_of::<*mut compl_T>()),
        ) as *mut *mut compl_T);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !compl.is_null() && i < compl_num_bests.get() {
            *(*compl_best_matches.ptr()).offset(i as isize) = compl;
            compl = (*compl).cp_next;
            i += 1;
        }
        let mut prefix: *mut ::core::ffi::c_char = (**(*compl_best_matches.ptr())
            .offset(0 as ::core::ffi::c_int as isize))
        .cp_str
        .data;
        let mut prefix_len: ::core::ffi::c_int = (**(*compl_best_matches.ptr())
            .offset(0 as ::core::ffi::c_int as isize))
        .cp_str
        .size as ::core::ffi::c_int;
        let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i_0 < compl_num_bests.get() {
            let mut match_str: *mut ::core::ffi::c_char = (**(*compl_best_matches.ptr())
                .offset(i_0 as isize))
            .cp_str
            .data;
            let mut prefix_ptr: *mut ::core::ffi::c_char = prefix;
            let mut match_ptr: *mut ::core::ffi::c_char = match_str;
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < prefix_len
                && *match_ptr as ::core::ffi::c_int != NUL
                && *prefix_ptr as ::core::ffi::c_int != NUL
            {
                if strncmp(prefix_ptr, match_ptr, utfc_ptr2len(prefix_ptr) as size_t)
                    != 0 as ::core::ffi::c_int
                {
                    break;
                }
                prefix_ptr = prefix_ptr.offset(utfc_ptr2len(prefix_ptr) as isize);
                match_ptr = match_ptr.offset(utfc_ptr2len(match_ptr) as isize);
                j += 1;
            }
            if j > 0 as ::core::ffi::c_int {
                prefix_len = j;
            }
            i_0 += 1;
        }
        let mut leader: *mut ::core::ffi::c_char = ins_compl_leader();
        let mut leader_len: size_t = ins_compl_leader_len();
        if !(leader_len > 0 as size_t
            && strncmp(prefix, leader, leader_len) != 0 as ::core::ffi::c_int)
        {
            prefix = xmemdupz(prefix as *const ::core::ffi::c_void, prefix_len as size_t)
                as *mut ::core::ffi::c_char;
            ins_compl_longest_insert(prefix);
            xfree(prefix as *mut ::core::ffi::c_void);
        }
        xfree(compl_best_matches.get() as *mut ::core::ffi::c_void);
        compl_best_matches.set(::core::ptr::null_mut::<*mut compl_T>());
        compl_num_bests.set(0 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn ins_compl_update_shown_match() {
    unsafe {
        get_leader_for_startcol(::core::ptr::null_mut::<compl_T>(), true_0 != 0);
        let mut leader: *mut String_0 =
            get_leader_for_startcol(compl_shown_match.get(), true_0 != 0);
        while !ins_compl_equal(compl_shown_match.get(), (*leader).data, (*leader).size)
            && !(*compl_shown_match.get()).cp_next.is_null()
            && !is_first_match((*compl_shown_match.get()).cp_next)
        {
            compl_shown_match.set((*compl_shown_match.get()).cp_next);
            leader = get_leader_for_startcol(compl_shown_match.get(), true_0 != 0);
        }
        if compl_shows_dir_backward() as ::core::ffi::c_int != 0
            && !ins_compl_equal(compl_shown_match.get(), (*leader).data, (*leader).size)
            && ((*compl_shown_match.get()).cp_next.is_null()
                || is_first_match((*compl_shown_match.get()).cp_next) as ::core::ffi::c_int != 0)
        {
            while !ins_compl_equal(compl_shown_match.get(), (*leader).data, (*leader).size)
                && !(*compl_shown_match.get()).cp_prev.is_null()
                && !is_first_match((*compl_shown_match.get()).cp_prev)
            {
                compl_shown_match.set((*compl_shown_match.get()).cp_prev);
                leader = get_leader_for_startcol(compl_shown_match.get(), true_0 != 0);
            }
        }
    }
}

pub unsafe extern "C" fn ins_compl_delete(mut new_leader: bool) {
    unsafe {
        let mut orig_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if new_leader {
            let mut orig: *mut ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
            let mut leader: *mut ::core::ffi::c_char = ins_compl_leader();
            while *orig as ::core::ffi::c_int != NUL && utf_ptr2char(orig) == utf_ptr2char(leader) {
                leader = leader.offset(utf_ptr2len(leader) as isize);
                orig = orig.offset(utf_ptr2len(orig) as isize);
            }
            orig_col = orig.offset_from((*compl_orig_text.ptr()).data) as ::core::ffi::c_int;
        }
        let mut col: ::core::ffi::c_int = compl_col.get() as ::core::ffi::c_int
            + (if compl_status_adding() as ::core::ffi::c_int != 0 {
                compl_length.get()
            } else {
                orig_col
            });
        if ins_compl_preinsert_effect() {
            col += ins_compl_leader_len() as ::core::ffi::c_int;
            (*curwin.get()).w_cursor.col = compl_ins_end_col.get();
        }
        let mut remaining: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        };
        if (*curwin.get()).w_cursor.lnum > compl_lnum.get() {
            if (*curwin.get()).w_cursor.col < get_cursor_line_len() {
                remaining = cbuf_to_string(get_cursor_pos_ptr(), get_cursor_pos_len() as size_t);
            }
            while (*curwin.get()).w_cursor.lnum > compl_lnum.get() {
                if ml_delete((*curwin.get()).w_cursor.lnum) == FAIL {
                    if !remaining.data.is_null() {
                        xfree(remaining.data as *mut ::core::ffi::c_void);
                    }
                    return;
                }
                deleted_lines_mark((*curwin.get()).w_cursor.lnum, 1 as ::core::ffi::c_int);
                (*curwin.get()).w_cursor.lnum -= 1;
            }
            (*curwin.get()).w_cursor.col = get_cursor_line_len();
        }
        if (*curwin.get()).w_cursor.col > col {
            if stop_arrow() == FAIL {
                if !remaining.data.is_null() {
                    xfree(remaining.data as *mut ::core::ffi::c_void);
                }
                return;
            }
            backspace_until_column(col);
            compl_ins_end_col.set((*curwin.get()).w_cursor.col);
        }
        if !remaining.data.is_null() {
            orig_col = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
            ins_str(remaining.data, remaining.size);
            (*curwin.get()).w_cursor.col = orig_col as colnr_T;
            xfree(remaining.data as *mut ::core::ffi::c_void);
        }
        changed_cline_bef_curs(curwin.get());
        set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED));
    }
}

pub(crate) unsafe extern "C" fn ins_compl_expand_multiple(mut str: *mut ::core::ffi::c_char) {
    unsafe {
        let mut start: *mut ::core::ffi::c_char = str;
        let mut curr: *mut ::core::ffi::c_char = str;
        let mut base_indent: ::core::ffi::c_int = get_indent();
        while *curr as ::core::ffi::c_int != NUL {
            if *curr as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                if curr > start {
                    ins_char_bytes(start, curr.offset_from(start) as size_t);
                }
                open_line(
                    FORWARD as ::core::ffi::c_int,
                    OPENLINE_KEEPTRAIL | OPENLINE_FORCE_INDENT,
                    base_indent,
                    ::core::ptr::null_mut::<bool>(),
                );
                start = curr.offset(1 as ::core::ffi::c_int as isize);
            }
            curr = curr.offset(1);
        }
        if curr > start {
            ins_char_bytes(start, curr.offset_from(start) as size_t);
        }
        compl_ins_end_col.set((*curwin.get()).w_cursor.col);
    }
}

pub unsafe extern "C" fn ins_compl_insert(mut move_cursor: bool, mut insert_prefix: bool) {
    unsafe {
        let mut compl_len: ::core::ffi::c_int = get_compl_len();
        let mut preinsert: bool = ins_compl_has_preinsert();
        let mut cp_str: *mut ::core::ffi::c_char = (*compl_shown_match.get()).cp_str.data;
        let mut cp_str_len: size_t = (*compl_shown_match.get()).cp_str.size;
        let mut leader_len: size_t = ins_compl_leader_len();
        let mut has_multiple: *mut ::core::ffi::c_char = strchr(cp_str, '\n' as ::core::ffi::c_int);
        if insert_prefix {
            cp_str = find_common_prefix(&raw mut cp_str_len, false_0 != 0);
            if cp_str.is_null() {
                cp_str = find_common_prefix(&raw mut cp_str_len, true_0 != 0);
                if cp_str.is_null() {
                    cp_str = (*compl_shown_match.get()).cp_str.data;
                    cp_str_len = (*compl_shown_match.get()).cp_str.size;
                }
            }
        } else if !(*cpt_sources_array.ptr()).is_null() {
            let mut cpt_idx: ::core::ffi::c_int = (*compl_shown_match.get()).cp_cpt_source_idx;
            if cpt_idx >= 0 as ::core::ffi::c_int && compl_col.get() >= 0 as ::core::ffi::c_int {
                let mut startcol: ::core::ffi::c_int =
                    (*(*cpt_sources_array.ptr()).offset(cpt_idx as isize)).cs_startcol;
                if startcol >= 0 as ::core::ffi::c_int && startcol < compl_col.get() {
                    let mut skip: ::core::ffi::c_int = compl_col.get() - startcol;
                    if skip as size_t <= cp_str_len {
                        cp_str_len = cp_str_len.wrapping_sub(skip as size_t);
                        cp_str = cp_str.offset(skip as isize);
                    }
                }
            }
        }
        if compl_len < cp_str_len as ::core::ffi::c_int {
            if !has_multiple.is_null() {
                ins_compl_expand_multiple(cp_str.offset(compl_len as isize));
            } else {
                ins_compl_insert_bytes(
                    cp_str.offset(compl_len as isize),
                    if insert_prefix as ::core::ffi::c_int != 0 {
                        cp_str_len as ::core::ffi::c_int - compl_len
                    } else {
                        -1 as ::core::ffi::c_int
                    },
                );
                if (preinsert as ::core::ffi::c_int != 0
                    || insert_prefix as ::core::ffi::c_int != 0)
                    && move_cursor as ::core::ffi::c_int != 0
                {
                    (*curwin.get()).w_cursor.col -= cp_str_len.wrapping_sub(leader_len) as colnr_T;
                }
            }
        }
        compl_used_match.set(
            !(match_at_original_text(compl_shown_match.get()) as ::core::ffi::c_int != 0
                || preinsert as ::core::ffi::c_int != 0 && !insert_prefix),
        );
        let mut dict: *mut dict_T = ins_compl_dict_alloc(compl_shown_match.get());
        set_vim_var_dict(VV_COMPLETED_ITEM, dict);
        compl_hi_on_autocompl_longest.set(
            insert_prefix as ::core::ffi::c_int != 0 && move_cursor as ::core::ffi::c_int != 0,
        );
    }
}

pub(crate) unsafe extern "C" fn find_next_completion_match(
    mut allow_get_expansion: bool,
    mut todo: ::core::ffi::c_int,
    mut advance: bool,
    mut num_matches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut found_end: bool = false_0 != 0;
        let mut found_compl: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
        let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
        let mut compl_no_select: bool = cur_cot_flags
            & kOptCotFlagNoselect as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint
            || compl_autocomplete.get() as ::core::ffi::c_int != 0 && !ins_compl_has_preinsert();
        loop {
            todo -= 1;
            if todo < 0 as ::core::ffi::c_int {
                break;
            }
            if compl_shows_dir_forward() as ::core::ffi::c_int != 0
                && !(*compl_shown_match.get()).cp_next.is_null()
            {
                if !(*compl_match_array.ptr()).is_null() {
                    compl_shown_match.set(find_next_match_in_menu());
                } else {
                    compl_shown_match.set((*compl_shown_match.get()).cp_next);
                }
                found_end = !(*compl_first_match.ptr()).is_null()
                    && (is_first_match((*compl_shown_match.get()).cp_next) as ::core::ffi::c_int
                        != 0
                        || is_first_match(compl_shown_match.get()) as ::core::ffi::c_int != 0);
            } else if compl_shows_dir_backward() as ::core::ffi::c_int != 0
                && !(*compl_shown_match.get()).cp_prev.is_null()
            {
                found_end = is_first_match(compl_shown_match.get());
                if !(*compl_match_array.ptr()).is_null() {
                    compl_shown_match.set(find_next_match_in_menu());
                } else {
                    compl_shown_match.set((*compl_shown_match.get()).cp_prev);
                }
                found_end = found_end as ::core::ffi::c_int
                    | is_first_match(compl_shown_match.get()) as ::core::ffi::c_int
                    != 0;
            } else {
                if !allow_get_expansion {
                    if advance {
                        if compl_shows_dir_backward() {
                            (*compl_pending.ptr()) -= todo + 1 as ::core::ffi::c_int;
                        } else {
                            (*compl_pending.ptr()) += todo + 1 as ::core::ffi::c_int;
                        }
                    }
                    return -1 as ::core::ffi::c_int;
                }
                if !compl_no_select && advance as ::core::ffi::c_int != 0 {
                    if compl_shows_dir_backward() {
                        (*compl_pending.ptr()) -= 1;
                    } else {
                        (*compl_pending.ptr()) += 1;
                    }
                }
                *num_matches = ins_compl_get_exp(compl_startpos.ptr());
                while compl_pending.get() != 0 as ::core::ffi::c_int
                    && compl_direction.get() as ::core::ffi::c_int
                        == compl_shows_dir.get() as ::core::ffi::c_int
                    && advance as ::core::ffi::c_int != 0
                {
                    if compl_pending.get() > 0 as ::core::ffi::c_int
                        && !(*compl_shown_match.get()).cp_next.is_null()
                    {
                        compl_shown_match.set((*compl_shown_match.get()).cp_next);
                        (*compl_pending.ptr()) -= 1;
                    } else {
                        if !(compl_pending.get() < 0 as ::core::ffi::c_int
                            && !(*compl_shown_match.get()).cp_prev.is_null())
                        {
                            break;
                        }
                        compl_shown_match.set((*compl_shown_match.get()).cp_prev);
                        (*compl_pending.ptr()) += 1;
                    }
                }
                found_end = false_0 != 0;
            }
            let mut leader: *mut String_0 =
                get_leader_for_startcol(compl_shown_match.get(), false_0 != 0);
            if !match_at_original_text(compl_shown_match.get())
                && !(*leader).data.is_null()
                && !ins_compl_equal(compl_shown_match.get(), (*leader).data, (*leader).size)
                && !(cot_fuzzy() as ::core::ffi::c_int != 0
                    && (*compl_shown_match.get()).cp_score != FUZZY_SCORE_NONE)
            {
                todo += 1;
            } else {
                found_compl = compl_shown_match.get();
            }
            if !found_end {
                continue;
            }
            if !found_compl.is_null() {
                compl_shown_match.set(found_compl);
                break;
            } else {
                todo = 1 as ::core::ffi::c_int;
            }
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_next(
    mut allow_get_expansion: bool,
    mut count: ::core::ffi::c_int,
    mut insert_match: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut num_matches: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut todo: ::core::ffi::c_int = count;
        let started: bool = compl_started.get();
        let orig_curbuf: *mut buf_T = curbuf.get();
        let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
        let mut compl_no_insert: bool = cur_cot_flags
            & kOptCotFlagNoinsert as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint
            || compl_autocomplete.get() as ::core::ffi::c_int != 0 && !ins_compl_has_preinsert();
        let mut compl_preinsert: bool = ins_compl_has_preinsert();
        let mut has_autocomplete_delay: bool =
            compl_autocomplete.get() as ::core::ffi::c_int != 0 && p_acl.get() > 0 as OptInt;
        if (*compl_shown_match.ptr()).is_null() {
            return -1 as ::core::ffi::c_int;
        }
        if !(*compl_leader.ptr()).data.is_null()
            && !match_at_original_text(compl_shown_match.get())
            && !cot_fuzzy()
        {
            ins_compl_update_shown_match();
        }
        if allow_get_expansion as ::core::ffi::c_int != 0
            && insert_match as ::core::ffi::c_int != 0
            && (!compl_get_longest.get() || compl_used_match.get() as ::core::ffi::c_int != 0)
        {
            ins_compl_delete(false_0 != 0);
        }
        let mut advance: bool =
            count != 1 as ::core::ffi::c_int || !allow_get_expansion || !compl_get_longest.get();
        if compl_restarting.get() {
            advance = false_0 != 0;
            compl_restarting.set(false_0 != 0);
        }
        if find_next_completion_match(allow_get_expansion, todo, advance, &raw mut num_matches)
            == -1 as ::core::ffi::c_int
        {
            return -1 as ::core::ffi::c_int;
        }
        if curbuf.get() != orig_curbuf {
            return -1 as ::core::ffi::c_int;
        }
        if !started && ins_compl_preinsert_longest() as ::core::ffi::c_int != 0 {
            ins_compl_insert(true_0 != 0, true_0 != 0);
            if has_autocomplete_delay {
                update_screen();
            }
        } else if compl_no_insert as ::core::ffi::c_int != 0 && !started && !compl_preinsert {
            ins_compl_insert_bytes(
                (*compl_orig_text.ptr())
                    .data
                    .offset(get_compl_len() as isize),
                -1 as ::core::ffi::c_int,
            );
            compl_used_match.set(false_0 != 0);
            restore_orig_extmarks();
        } else if insert_match {
            if !compl_get_longest.get() || compl_used_match.get() as ::core::ffi::c_int != 0 {
                let mut preinsert_longest: bool = ins_compl_preinsert_longest()
                    as ::core::ffi::c_int
                    != 0
                    && match_at_original_text(compl_shown_match.get()) as ::core::ffi::c_int != 0;
                ins_compl_insert(
                    compl_preinsert as ::core::ffi::c_int != 0
                        || preinsert_longest as ::core::ffi::c_int != 0,
                    preinsert_longest,
                );
            } else {
                '_c2rust_label: {
                    if !(*compl_leader.ptr()).data.is_null() {
                    } else {
                        __assert_fail(
                            b"compl_leader.data != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            5406 as ::core::ffi::c_uint,
                            b"int ins_compl_next(_Bool, int, _Bool)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                ins_compl_insert_bytes(
                    (*compl_leader.ptr()).data.offset(get_compl_len() as isize),
                    -1 as ::core::ffi::c_int,
                );
            }
            if strequal(
                (*compl_shown_match.get()).cp_str.data,
                (*compl_orig_text.ptr()).data,
            ) {
                restore_orig_extmarks();
            }
        } else {
            compl_used_match.set(false_0 != 0);
        }
        if !allow_get_expansion {
            update_screen();
            if !has_autocomplete_delay {
                ins_compl_show_pum();
            }
            ins_compl_delete(false_0 != 0);
        }
        if compl_no_insert as ::core::ffi::c_int != 0
            && !started
            && !match_at_original_text(compl_shown_match.get())
        {
            compl_enter_selects.set(true_0 != 0);
        } else {
            compl_enter_selects.set(!insert_match && !(*compl_match_array.ptr()).is_null());
        }
        if !(*compl_shown_match.get()).cp_fname.is_null() {
            ins_compl_show_filename();
        }
        return num_matches;
    }
}
