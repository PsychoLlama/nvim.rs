//! Finding a line whose syntax state is known — `:syntax sync`.
//!
//! [`syn_sync`] answers "where can parsing safely start for line N", which is
//! what keeps highlighting a line in the middle of a big file from costing a
//! parse of the whole file. Every strategy `:syntax sync` offers lives here:
//! a fixed number of lines back, `fromstart`, a C comment scan, line
//! continuations, and the `grouphere`/`groupthere` sync patterns.
//! [`syn_cmd_sync`] is the command that configures them.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn syn_sync(
    mut wp: *mut win_T,
    mut start_lnum: linenr_T,
    mut last_valid: *mut synstate_T,
) {
    unsafe {
        let mut cursor_save: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut lnum: linenr_T = 0;
        let mut break_lnum: linenr_T = 0;
        let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
        let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
        let mut found_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut found_match_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut found_current_lnum: linenr_T = 0 as linenr_T;
        let mut found_current_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut found_m_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
        invalidate_current_state();
        if (*syn_block.get()).b_syn_sync_minlines > start_lnum {
            start_lnum = 1 as ::core::ffi::c_int as linenr_T;
        } else {
            if (*syn_block.get()).b_syn_sync_minlines == 1 as linenr_T {
                lnum = 1 as ::core::ffi::c_int as linenr_T;
            } else if (*syn_block.get()).b_syn_sync_minlines < 10 as linenr_T {
                lnum = (*syn_block.get()).b_syn_sync_minlines * 2 as linenr_T;
            } else {
                lnum = (*syn_block.get()).b_syn_sync_minlines * 3 as linenr_T / 2 as linenr_T;
            }
            if (*syn_block.get()).b_syn_sync_maxlines != 0 as linenr_T
                && lnum > (*syn_block.get()).b_syn_sync_maxlines
            {
                lnum = (*syn_block.get()).b_syn_sync_maxlines;
            }
            if lnum >= start_lnum {
                start_lnum = 1 as ::core::ffi::c_int as linenr_T;
            } else {
                start_lnum -= lnum;
            }
        }
        current_lnum.set(start_lnum);
        if (*syn_block.get()).b_syn_sync_flags & SF_CCOMMENT != 0 {
            let mut curwin_save: *mut win_T = curwin.get();
            curwin.set(wp);
            let mut curbuf_save: *mut buf_T = curbuf.get();
            curbuf.set(syn_buf.get());
            while start_lnum > 1 as linenr_T {
                let mut l: *mut ::core::ffi::c_char = ml_get(start_lnum - 1 as linenr_T);
                if *l as ::core::ffi::c_int == NUL
                    || *l
                        .offset(ml_get_len(start_lnum - 1 as linenr_T) as isize)
                        .offset(-(1 as ::core::ffi::c_int as isize))
                        as ::core::ffi::c_int
                        != '\\' as ::core::ffi::c_int
                {
                    break;
                }
                start_lnum -= 1;
            }
            current_lnum.set(start_lnum);
            cursor_save = (*wp).w_cursor;
            (*wp).w_cursor.lnum = start_lnum;
            (*wp).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            if !find_start_comment((*syn_block.get()).b_syn_sync_maxlines as ::core::ffi::c_int)
                .is_null()
            {
                let mut idx: ::core::ffi::c_int = (*syn_block.get()).b_syn_patterns.ga_len;
                loop {
                    idx -= 1;
                    if idx < 0 as ::core::ffi::c_int {
                        break;
                    }
                    if !((*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset(idx as isize))
                    .sp_syn
                    .id as ::core::ffi::c_int
                        == (*syn_block.get()).b_syn_sync_id as ::core::ffi::c_int
                        && (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                            .offset(idx as isize))
                        .sp_type as ::core::ffi::c_int
                            == SPTYPE_START)
                    {
                        continue;
                    }
                    validate_current_state();
                    push_current_state(idx);
                    update_si_attr((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
                    break;
                }
            }
            (*wp).w_cursor = cursor_save;
            curwin.set(curwin_save);
            curbuf.set(curbuf_save);
        } else if (*syn_block.get()).b_syn_sync_flags & SF_MATCH != 0 {
            if (*syn_block.get()).b_syn_sync_maxlines != 0 as linenr_T
                && start_lnum > (*syn_block.get()).b_syn_sync_maxlines
            {
                break_lnum = start_lnum - (*syn_block.get()).b_syn_sync_maxlines;
            } else {
                break_lnum = 0 as ::core::ffi::c_int as linenr_T;
            }
            found_m_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
            found_m_endpos.col = 0 as ::core::ffi::c_int as colnr_T;
            let mut end_lnum: linenr_T = start_lnum;
            lnum = start_lnum;
            loop {
                lnum -= 1;
                if lnum <= break_lnum {
                    break;
                }
                line_breakcheck();
                if got_int.get() {
                    invalidate_current_state();
                    current_lnum.set(start_lnum);
                    break;
                } else if !last_valid.is_null() && lnum == (*last_valid).sst_lnum {
                    load_current_state(last_valid);
                    break;
                } else {
                    if lnum > 1 as linenr_T && syn_match_linecont(lnum - 1 as linenr_T) != 0 {
                        continue;
                    }
                    validate_current_state();
                    current_lnum.set(lnum);
                    while current_lnum.get() < end_lnum {
                        syn_start_line();
                        loop {
                            let mut had_sync_point: bool = syn_finish_line(true_0 != 0);
                            if !(had_sync_point as ::core::ffi::c_int != 0
                                && (*current_state.ptr()).ga_len != 0)
                            {
                                break;
                            }
                            cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(
                                ((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize,
                            );
                            if (*cur_si).si_m_endpos.lnum > start_lnum {
                                current_lnum.set(end_lnum);
                                break;
                            } else {
                                if (*cur_si).si_idx < 0 as ::core::ffi::c_int {
                                    found_flags = 0 as ::core::ffi::c_int;
                                    found_match_idx = KEYWORD_IDX;
                                } else {
                                    spp = ((*syn_block.get()).b_syn_patterns.ga_data
                                        as *mut synpat_T)
                                        .offset((*cur_si).si_idx as isize);
                                    found_flags = (*spp).sp_flags;
                                    found_match_idx = (*spp).sp_sync_idx;
                                }
                                found_current_lnum = current_lnum.get();
                                found_current_col = current_col.get() as ::core::ffi::c_int;
                                found_m_endpos = (*cur_si).si_m_endpos;
                                if found_m_endpos.lnum > current_lnum.get() {
                                    current_lnum.set(found_m_endpos.lnum);
                                    current_col.set(found_m_endpos.col);
                                    if current_lnum.get() >= end_lnum {
                                        break;
                                    }
                                } else if found_m_endpos.col > current_col.get() {
                                    current_col.set(found_m_endpos.col);
                                } else {
                                    (*current_col.ptr()) += 1;
                                }
                                let mut prev_current_col: colnr_T = current_col.get();
                                if *syn_getcurline().offset(current_col.get() as isize)
                                    as ::core::ffi::c_int
                                    != NUL
                                {
                                    (*current_col.ptr()) += 1;
                                }
                                check_state_ends();
                                current_col.set(prev_current_col);
                            }
                        }
                        (*current_lnum.ptr()) += 1;
                    }
                    if found_flags != 0 {
                        clear_current_state();
                        if found_match_idx >= 0 as ::core::ffi::c_int {
                            push_current_state(found_match_idx);
                            update_si_attr((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
                        }
                        if found_flags & HL_SYNC_HERE != 0 {
                            current_lnum.set(found_m_endpos.lnum);
                            current_col.set(found_m_endpos.col);
                            if !((*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int) {
                                cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
                                    .offset(
                                        ((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int)
                                            as isize,
                                    );
                                (*cur_si).si_h_startpos.lnum = found_current_lnum;
                                (*cur_si).si_h_startpos.col = found_current_col as colnr_T;
                                update_si_end(cur_si, current_col.get(), true_0 != 0);
                                check_keepend();
                            }
                            syn_finish_line(false_0 != 0);
                            (*current_lnum.ptr()) += 1;
                        } else {
                            current_lnum.set(start_lnum);
                        }
                        break;
                    } else {
                        end_lnum = lnum;
                        invalidate_current_state();
                    }
                }
            }
            if lnum <= break_lnum {
                invalidate_current_state();
                current_lnum.set(break_lnum + 1 as linenr_T);
            }
        }
        validate_current_state();
    }
}

pub(crate) unsafe extern "C" fn save_chartab(mut chartab: *mut ::core::ffi::c_char) {
    unsafe {
        if (*syn_block.get()).b_syn_isk == empty_string_option.ptr() as *mut ::core::ffi::c_char {
            return;
        }
        memmove(
            chartab as *mut ::core::ffi::c_void,
            &raw mut (*syn_buf.get()).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
        memmove(
            &raw mut (*syn_buf.get()).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
            &raw mut (*(*syn_win.get()).w_s).b_syn_chartab as *mut uint8_t
                as *const ::core::ffi::c_void,
            32 as ::core::ffi::c_int as size_t,
        );
    }
}

pub(crate) unsafe extern "C" fn restore_chartab(mut chartab: *mut ::core::ffi::c_char) {
    unsafe {
        if (*(*syn_win.get()).w_s).b_syn_isk
            != empty_string_option.ptr() as *mut ::core::ffi::c_char
        {
            memmove(
                &raw mut (*syn_buf.get()).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
                chartab as *const ::core::ffi::c_void,
                32 as ::core::ffi::c_int as size_t,
            );
        }
    }
}

pub(crate) unsafe extern "C" fn syn_match_linecont(mut lnum: linenr_T) -> ::core::ffi::c_int {
    unsafe {
        if (*syn_block.get()).b_syn_linecont_prog.is_null() {
            return false_0;
        }
        let mut regmatch: regmmatch_T = regmmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startpos: [lpos_T { lnum: 0, col: 0 }; 10],
            endpos: [lpos_T { lnum: 0, col: 0 }; 10],
            rmm_matchcol: 0,
            rmm_ic: 0,
            rmm_maxcol: 0,
        };
        let mut buf_chartab: [::core::ffi::c_char; 32] = [0; 32];
        save_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
        regmatch.rmm_ic = (*syn_block.get()).b_syn_linecont_ic;
        regmatch.regprog = (*syn_block.get()).b_syn_linecont_prog;
        let mut r: ::core::ffi::c_int = syn_regexec(
            &raw mut regmatch,
            lnum,
            0 as colnr_T,
            &raw mut (*syn_block.get()).b_syn_linecont_time,
        ) as ::core::ffi::c_int;
        (*syn_block.get()).b_syn_linecont_prog = regmatch.regprog;
        restore_chartab(&raw mut buf_chartab as *mut ::core::ffi::c_char);
        return r;
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_sync(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg_start: *mut ::core::ffi::c_char = (*eap).arg;
        let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut illegal: bool = false_0 != 0;
        let mut finished: bool = false_0 != 0;
        if ends_excmd(*arg_start as ::core::ffi::c_int) != 0 {
            syn_cmd_list(eap, true_0);
            return;
        }
        while ends_excmd(*arg_start as ::core::ffi::c_int) == 0 {
            let mut arg_end: *mut ::core::ffi::c_char = skiptowhite(arg_start);
            let mut next_arg: *mut ::core::ffi::c_char = skipwhite(arg_end);
            xfree(key as *mut ::core::ffi::c_void);
            key = vim_strnsave_up(arg_start, arg_end.offset_from(arg_start) as size_t);
            if strcmp(key, b"CCOMMENT\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                if (*eap).skip == 0 {
                    (*(*curwin.get()).w_s).b_syn_sync_flags |= SF_CCOMMENT;
                }
                if ends_excmd(*next_arg as ::core::ffi::c_int) == 0 {
                    arg_end = skiptowhite(next_arg);
                    if (*eap).skip == 0 {
                        (*(*curwin.get()).w_s).b_syn_sync_id =
                            syn_check_group(next_arg, arg_end.offset_from(next_arg) as size_t)
                                as int16_t;
                    }
                    next_arg = skipwhite(arg_end);
                } else if (*eap).skip == 0 {
                    (*(*curwin.get()).w_s).b_syn_sync_id =
                        syn_name2id(b"Comment\0".as_ptr() as *const ::core::ffi::c_char) as int16_t;
                }
            } else if strncmp(
                key,
                b"LINES\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
                || strncmp(
                    key,
                    b"MINLINES\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
                || strncmp(
                    key,
                    b"MAXLINES\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
                || strncmp(
                    key,
                    b"LINEBREAKS\0".as_ptr() as *const ::core::ffi::c_char,
                    10 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                if *key.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'S' as ::core::ffi::c_int
                {
                    arg_end = key.offset(6 as ::core::ffi::c_int as isize);
                } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'L' as ::core::ffi::c_int
                {
                    arg_end = key.offset(11 as ::core::ffi::c_int as isize);
                } else {
                    arg_end = key.offset(9 as ::core::ffi::c_int as isize);
                }
                if *arg_end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '=' as ::core::ffi::c_int
                    || !ascii_isdigit(*arg_end as ::core::ffi::c_int)
                {
                    illegal = true_0 != 0;
                    break;
                } else {
                    let mut n: linenr_T =
                        getdigits_int32(&raw mut arg_end, false_0 != 0, 0 as int32_t);
                    if (*eap).skip == 0 {
                        if *key.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'B' as ::core::ffi::c_int
                        {
                            (*(*curwin.get()).w_s).b_syn_sync_linebreaks = n;
                        } else if *key.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'A' as ::core::ffi::c_int
                        {
                            (*(*curwin.get()).w_s).b_syn_sync_maxlines = n;
                        } else {
                            (*(*curwin.get()).w_s).b_syn_sync_minlines = n;
                        }
                    }
                }
            } else if strcmp(key, b"FROMSTART\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                if (*eap).skip == 0 {
                    (*(*curwin.get()).w_s).b_syn_sync_minlines =
                        MAXLNUM as ::core::ffi::c_int as linenr_T;
                    (*(*curwin.get()).w_s).b_syn_sync_maxlines =
                        0 as ::core::ffi::c_int as linenr_T;
                }
            } else if strcmp(key, b"LINECONT\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                if *next_arg as ::core::ffi::c_int == NUL {
                    illegal = true_0 != 0;
                    break;
                } else if !(*(*curwin.get()).w_s).b_syn_linecont_pat.is_null() {
                    emsg(gettext(
                        b"E403: syntax sync: line continuations pattern specified twice\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    finished = true_0 != 0;
                    break;
                } else {
                    arg_end = skip_regexp(
                        next_arg.offset(1 as ::core::ffi::c_int as isize),
                        *next_arg as ::core::ffi::c_int,
                        true_0,
                    );
                    if *arg_end as ::core::ffi::c_int != *next_arg as ::core::ffi::c_int {
                        illegal = true_0 != 0;
                        break;
                    } else {
                        if (*eap).skip == 0 {
                            (*(*curwin.get()).w_s).b_syn_linecont_pat = xstrnsave(
                                next_arg.offset(1 as ::core::ffi::c_int as isize),
                                (arg_end.offset_from(next_arg) as size_t).wrapping_sub(1 as size_t),
                            );
                            (*(*curwin.get()).w_s).b_syn_linecont_ic =
                                (*(*curwin.get()).w_s).b_syn_ic;
                            let mut cpo_save: *mut ::core::ffi::c_char = p_cpo.get();
                            p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
                            (*(*curwin.get()).w_s).b_syn_linecont_prog =
                                vim_regcomp((*(*curwin.get()).w_s).b_syn_linecont_pat, RE_MAGIC);
                            p_cpo.set(cpo_save);
                            syn_clear_time(&raw mut (*(*curwin.get()).w_s).b_syn_linecont_time);
                            if (*(*curwin.get()).w_s).b_syn_linecont_prog.is_null() {
                                let mut ptr_: *mut *mut ::core::ffi::c_void =
                                    &raw mut (*(*curwin.get()).w_s).b_syn_linecont_pat
                                        as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr_);
                                *ptr_ = NULL;
                                let _ = *ptr_;
                                finished = true_0 != 0;
                                break;
                            }
                        }
                        next_arg = skipwhite(arg_end.offset(1 as ::core::ffi::c_int as isize));
                    }
                }
            } else {
                (*eap).arg = next_arg;
                if strcmp(key, b"MATCH\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    syn_cmd_match(eap, true_0);
                } else if strcmp(key, b"REGION\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    syn_cmd_region(eap, true_0);
                } else if strcmp(key, b"CLEAR\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    syn_cmd_clear(eap, true_0);
                } else {
                    illegal = true_0 != 0;
                }
                finished = true_0 != 0;
                break;
            }
            arg_start = next_arg;
        }
        xfree(key as *mut ::core::ffi::c_void);
        if illegal {
            semsg(
                gettext(b"E404: Illegal arguments: %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg_start,
            );
        } else if !finished {
            (*eap).nextcmd = check_nextcmd(arg_start);
            redraw_curbuf_later(UPD_SOME_VALID);
            syn_stack_free_all((*curwin.get()).w_s);
        }
    }
}
