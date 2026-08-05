//! The keys typed while a completion is up, and how one ends.
//!
//! [`ins_compl_prep`] sees every key first and decides whether it belongs to
//! the completion, ends it, or is inserted; [`ins_compl_stop`] is the unwind.
//! [`ins_compl_bs`] and [`ins_compl_addleader`] are the two that edit the
//! leader and re-filter what is shown.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn ins_compl_bs() -> ::core::ffi::c_int {
    unsafe {
        if ins_compl_preinsert_effect() {
            ins_compl_delete(false_0 != 0);
        }
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        let mut p: *mut ::core::ffi::c_char = line.offset((*curwin.get()).w_cursor.col as isize);
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
        let mut p_off: ptrdiff_t = p.offset_from(line);
        if p.offset_from(line) as ::core::ffi::c_int - compl_col.get() < 0 as ::core::ffi::c_int
            || p.offset_from(line) as ::core::ffi::c_int - compl_col.get()
                == 0 as ::core::ffi::c_int
                && !ctrl_x_mode_omni()
            || ctrl_x_mode_eval() as ::core::ffi::c_int != 0
            || !can_bs(BS_START)
                && p.offset_from(line) as ::core::ffi::c_int - compl_col.get() - compl_length.get()
                    < 0 as ::core::ffi::c_int
        {
            return K_BS;
        }
        if (*curwin.get()).w_cursor.col
            <= compl_col.get() as ::core::ffi::c_int + compl_length.get()
            || ins_compl_need_restart() as ::core::ffi::c_int != 0
        {
            ins_compl_restart();
        }
        line = get_cursor_line_ptr();
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*compl_leader.ptr()).size = 0 as size_t;
        compl_leader.set(cbuf_to_string(
            line.offset(compl_col.get() as isize),
            (p_off - compl_col.get() as ptrdiff_t) as size_t,
        ));
        if compl_autocomplete.get() as ::core::ffi::c_int != 0
            && !(*compl_first_match.ptr()).is_null()
            && !ins_compl_has_preinsert()
        {
            compl_shown_match.set(compl_first_match.get());
        }
        ins_compl_new_leader();
        if !(*compl_shown_match.ptr()).is_null() {
            compl_curr_match.set(compl_shown_match.get());
        }
        return NUL;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_new_leader() {
    unsafe {
        ins_compl_del_pum();
        ins_compl_delete(true_0 != 0);
        ins_compl_insert_bytes(
            (*compl_leader.ptr()).data.offset(get_compl_len() as isize),
            -1 as ::core::ffi::c_int,
        );
        compl_used_match.set(false_0 != 0);
        if p_acl.get() > 0 as OptInt {
            pum_undisplay(true_0 != 0);
            redraw_later(curwin.get(), UPD_VALID);
            update_screen();
            ui_flush();
        }
        if compl_started.get() {
            ins_compl_set_original_text((*compl_leader.ptr()).data, (*compl_leader.ptr()).size);
            if is_cpt_func_refresh_always() {
                cpt_compl_refresh();
            }
            if cot_fuzzy() {
                ins_compl_fuzzy_sort();
            }
        } else {
            spell_bad_len.set(0 as size_t);
            compl_restarting.set(true_0 != 0);
            if ins_compl_has_autocomplete() {
                ins_compl_enable_autocomplete();
            } else {
                compl_autocomplete.set(false_0 != 0);
            }
            if ins_complete(Ctrl_N, true_0 != 0) == FAIL {
                compl_cont_status.set(0 as ::core::ffi::c_int);
            }
            compl_restarting.set(false_0 != 0);
        }
        compl_enter_selects
            .set(!compl_used_match.get() && compl_selected_item.get() != -1 as ::core::ffi::c_int);
        ins_compl_show_pum();
        if (*compl_match_array.ptr()).is_null() {
            compl_enter_selects.set(false_0 != 0);
        } else if ins_compl_has_preinsert() as ::core::ffi::c_int != 0
            && (*compl_leader.ptr()).size > 0 as size_t
        {
            ins_compl_insert(true_0 != 0, false_0 != 0);
        } else if compl_started.get() as ::core::ffi::c_int != 0
            && ins_compl_preinsert_longest() as ::core::ffi::c_int != 0
            && (*compl_leader.ptr()).size > 0 as size_t
            && !ins_compl_preinsert_effect()
        {
            ins_compl_insert(true_0 != 0, true_0 != 0);
        }
        if ins_compl_refresh_always() {
            compl_enter_selects.set(false_0 != 0);
        }
    }
}

pub unsafe extern "C" fn ins_compl_addleader(mut c: ::core::ffi::c_int) {
    unsafe {
        let mut cc: ::core::ffi::c_int = 0;
        if ins_compl_preinsert_effect() {
            ins_compl_delete(false_0 != 0);
        }
        if stop_arrow() == FAIL {
            return;
        }
        cc = utf_char2len(c);
        if cc > 1 as ::core::ffi::c_int {
            let mut buf: [::core::ffi::c_char; 7] = [0; 7];
            utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char);
            buf[cc as usize] = NUL as ::core::ffi::c_char;
            ins_char_bytes(&raw mut buf as *mut ::core::ffi::c_char, cc as size_t);
        } else {
            ins_char(c);
        }
        if ins_compl_need_restart() {
            ins_compl_restart();
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*compl_leader.ptr()).size = 0 as size_t;
        compl_leader.set(cbuf_to_string(
            get_cursor_line_ptr().offset(compl_col.get() as isize),
            ((*curwin.get()).w_cursor.col - compl_col.get()) as size_t,
        ));
        ins_compl_new_leader();
    }
}

pub(crate) unsafe extern "C" fn ins_compl_restart() {
    unsafe {
        update_screen();
        ins_compl_free();
        compl_started.set(false_0 != 0);
        compl_matches.set(0 as ::core::ffi::c_int);
        compl_cont_status.set(0 as ::core::ffi::c_int);
        compl_cont_mode.set(0 as ::core::ffi::c_int);
        cpt_sources_clear();
        compl_autocomplete.set(false_0 != 0);
        compl_from_nonkeyword.set(false_0 != 0);
        compl_num_bests.set(0 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn ins_compl_set_original_text(
    mut str: *mut ::core::ffi::c_char,
    mut len: size_t,
) {
    unsafe {
        if match_at_original_text(compl_first_match.get()) {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*compl_first_match.get()).cp_str.data as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            (*compl_first_match.get()).cp_str.size = 0 as size_t;
            (*compl_first_match.get()).cp_str = cbuf_to_string(str, len);
        } else if !(*compl_first_match.get()).cp_prev.is_null()
            && match_at_original_text((*compl_first_match.get()).cp_prev) as ::core::ffi::c_int != 0
        {
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*(*compl_first_match.get()).cp_prev).cp_str.data
                    as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL;
            let _ = *ptr__0;
            (*(*compl_first_match.get()).cp_prev).cp_str.size = 0 as size_t;
            (*(*compl_first_match.get()).cp_prev).cp_str = cbuf_to_string(str, len);
        }
    }
}

pub unsafe extern "C" fn ins_compl_addfrommatch() {
    unsafe {
        let mut len: ::core::ffi::c_int = (*curwin.get()).w_cursor.col - compl_col.get();
        '_c2rust_label: {
            if !(*compl_shown_match.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"compl_shown_match != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2421 as ::core::ffi::c_uint,
                    b"void ins_compl_addfrommatch(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut p: *mut ::core::ffi::c_char = (*compl_shown_match.get()).cp_str.data;
        if (*compl_shown_match.get()).cp_str.size as ::core::ffi::c_int <= len {
            if !match_at_original_text(compl_shown_match.get()) {
                return;
            }
            p = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut plen: size_t = 0 as size_t;
            let mut cp: *mut compl_T = (*compl_shown_match.get()).cp_next;
            while !cp.is_null() && !is_first_match(cp) {
                if (*compl_leader.ptr()).data.is_null()
                    || ins_compl_equal(cp, (*compl_leader.ptr()).data, (*compl_leader.ptr()).size)
                        as ::core::ffi::c_int
                        != 0
                {
                    p = (*cp).cp_str.data;
                    plen = (*cp).cp_str.size;
                    break;
                } else {
                    cp = (*cp).cp_next;
                }
            }
            if p.is_null() || plen as ::core::ffi::c_int <= len {
                return;
            }
        }
        p = p.offset(len as isize);
        let mut c: ::core::ffi::c_int = utf_ptr2char(p);
        ins_compl_addleader(c);
    }
}

pub(crate) unsafe extern "C" fn ins_compl_stop(
    c: ::core::ffi::c_int,
    prev_mode: ::core::ffi::c_int,
    mut retval: bool,
) -> bool {
    unsafe {
        if ins_compl_preinsert_effect() as ::core::ffi::c_int != 0
            && ins_compl_win_active(curwin.get()) as ::core::ffi::c_int != 0
        {
            ins_compl_delete(false_0 != 0);
        }
        if !(*compl_curr_match.ptr()).is_null()
            || !(*compl_leader.ptr()).data.is_null()
            || c == Ctrl_E
        {
            let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if !(*compl_curr_match.ptr()).is_null()
                && compl_used_match.get() as ::core::ffi::c_int != 0
                && c != Ctrl_E
            {
                ptr = (*compl_curr_match.get()).cp_str.data;
            }
            ins_compl_fixRedoBufForLeader(ptr);
        }
        let mut want_cindent: bool =
            get_can_cindent() as ::core::ffi::c_int != 0 && cindent_on() as ::core::ffi::c_int != 0;
        if compl_cont_mode.get() == CTRL_X_WHOLE_LINE {
            if want_cindent {
                do_c_expr_indent();
                want_cindent = false_0 != 0;
            }
        } else if !compl_autocomplete.get() || compl_used_match.get() as ::core::ffi::c_int != 0 {
            let prev_col: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
            if prev_col > 0 as ::core::ffi::c_int {
                dec_cursor();
            }
            if !arrow_used.get() && !ins_need_undo_get() && c != Ctrl_E {
                insertchar(NUL, 0 as ::core::ffi::c_int, -1 as ::core::ffi::c_int);
            }
            if prev_col > 0 as ::core::ffi::c_int
                && *get_cursor_line_ptr().offset((*curwin.get()).w_cursor.col as isize)
                    as ::core::ffi::c_int
                    != NUL
            {
                inc_cursor();
            }
        }
        let mut word: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (c == Ctrl_Y
            || compl_enter_selects.get() as ::core::ffi::c_int != 0
                && (c == CAR || c == K_KENTER || c == NL))
            && pum_visible() as ::core::ffi::c_int != 0
        {
            word = xstrdup((*compl_shown_match.get()).cp_str.data);
            retval = true_0 != 0;
            redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
        }
        if word.is_null()
            && c != Ctrl_E
            && compl_used_match.get() as ::core::ffi::c_int != 0
            && (*compl_match_array.ptr()).is_null()
            && !(*compl_curr_match.ptr()).is_null()
            && !(*compl_curr_match.get()).cp_str.data.is_null()
        {
            word = xstrdup((*compl_curr_match.get()).cp_str.data);
        }
        if c == Ctrl_E {
            ins_compl_delete(false_0 != 0);
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut plen: size_t = 0 as size_t;
            if !(*compl_leader.ptr()).data.is_null() {
                p = (*compl_leader.ptr()).data;
                plen = (*compl_leader.ptr()).size;
            } else if !(*compl_first_match.ptr()).is_null() {
                p = (*compl_orig_text.ptr()).data;
                plen = (*compl_orig_text.ptr()).size;
            }
            if !p.is_null() {
                let compl_len: ::core::ffi::c_int = get_compl_len();
                if plen as ::core::ffi::c_int > compl_len {
                    ins_compl_insert_bytes(
                        p.offset(compl_len as isize),
                        plen as ::core::ffi::c_int - compl_len,
                    );
                }
            }
            restore_orig_extmarks();
            retval = true_0 != 0;
        }
        auto_format(false_0 != 0, true_0 != 0);
        ctrl_x_mode.set(prev_mode);
        ins_apply_autocmds(EVENT_COMPLETEDONEPRE);
        ins_compl_free();
        compl_started.set(false_0 != 0);
        compl_matches.set(0 as ::core::ffi::c_int);
        if !shortmess(SHM_COMPLETIONMENU) {
            msg_clr_cmdline();
        }
        ctrl_x_mode.set(CTRL_X_NORMAL);
        compl_enter_selects.set(false_0 != 0);
        if !(*edit_submode.ptr()).is_null() {
            edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            redraw_mode.set(true_0 != 0);
        }
        compl_autocomplete.set(false_0 != 0);
        compl_from_nonkeyword.set(false_0 != 0);
        compl_num_bests.set(0 as ::core::ffi::c_int);
        compl_ins_end_col.set(0 as ::core::ffi::c_int as colnr_T);
        if c == Ctrl_C && cmdwin_type.get() != 0 as ::core::ffi::c_int {
            update_screen();
        }
        if want_cindent as ::core::ffi::c_int != 0
            && in_cinkeys(
                KEY_COMPLETE,
                ' ' as ::core::ffi::c_int,
                inindent(0 as ::core::ffi::c_int),
            ) as ::core::ffi::c_int
                != 0
        {
            do_c_expr_indent();
        }
        do_autocmd_completedone(c, prev_mode, word);
        xfree(word as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub unsafe extern "C" fn ins_compl_cancel() -> bool {
    unsafe {
        return ins_compl_stop(' ' as ::core::ffi::c_int, ctrl_x_mode.get(), true_0 != 0);
    }
}

pub unsafe extern "C" fn ins_compl_prep(mut c: ::core::ffi::c_int) -> bool {
    unsafe {
        let mut retval: bool = false_0 != 0;
        let prev_mode: ::core::ffi::c_int = ctrl_x_mode.get();
        if c != Ctrl_R && vim_is_ctrl_x_key(c) as ::core::ffi::c_int != 0 {
            edit_submode_extra.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        }
        if c == K_SELECT
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            return retval;
        }
        if ctrl_x_mode.get() == CTRL_X_CMDLINE_CTRL_X && c != Ctrl_X {
            if c == Ctrl_V
                || c == Ctrl_Q
                || c == Ctrl_Z
                || ins_compl_pum_key(c) as ::core::ffi::c_int != 0
                || !vim_is_ctrl_x_key(c)
            {
                ctrl_x_mode.set(CTRL_X_CMDLINE);
                if c == Ctrl_Z {
                    retval = true_0 != 0;
                }
            } else {
                ctrl_x_mode.set(CTRL_X_CMDLINE);
                ins_compl_prep(' ' as ::core::ffi::c_int);
                ctrl_x_mode.set(CTRL_X_NOT_DEFINED_YET);
            }
        }
        if ctrl_x_mode_not_defined_yet() as ::core::ffi::c_int != 0
            || ctrl_x_mode_normal() as ::core::ffi::c_int != 0 && !compl_started.get()
        {
            compl_get_longest.set(
                get_cot_flags() & kOptCotFlagLongest as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0 as ::core::ffi::c_uint,
            );
            compl_used_match.set(true_0 != 0);
        }
        if ctrl_x_mode_not_defined_yet() {
            retval = set_ctrl_x_mode(c);
        } else if ctrl_x_mode_not_default() {
            if !vim_is_ctrl_x_key(c) {
                ctrl_x_mode.set(if ctrl_x_mode_scroll() as ::core::ffi::c_int != 0 {
                    CTRL_X_NORMAL
                } else {
                    CTRL_X_FINISHED
                });
                edit_submode.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            }
            redraw_mode.set(true_0 != 0);
        }
        if compl_started.get() as ::core::ffi::c_int != 0 || ctrl_x_mode.get() == CTRL_X_FINISHED {
            redraw_mode.set(true_0 != 0);
            if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                && c != Ctrl_N
                && c != Ctrl_P
                && c != Ctrl_R
                && !ins_compl_pum_key(c)
                || ctrl_x_mode.get() == CTRL_X_FINISHED
            {
                retval = ins_compl_stop(c, prev_mode, retval);
            }
        } else if ctrl_x_mode.get() == CTRL_X_LOCAL_MSG {
            do_autocmd_completedone(
                c,
                ctrl_x_mode.get(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            );
        }
        may_trigger_modechanged();
        if !vim_is_ctrl_x_key(c) {
            compl_cont_status.set(0 as ::core::ffi::c_int);
            compl_cont_mode.set(0 as ::core::ffi::c_int);
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_fixRedoBufForLeader(
    mut ptr_arg: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut ptr: *mut ::core::ffi::c_char = ptr_arg;
        if ptr.is_null() {
            if !(*compl_leader.ptr()).data.is_null() {
                ptr = (*compl_leader.ptr()).data;
            } else {
                return;
            }
        }
        if !(*compl_orig_text.ptr()).data.is_null() {
            let mut p: *mut ::core::ffi::c_char = (*compl_orig_text.ptr()).data;
            while *p.offset(len as isize) as ::core::ffi::c_int != NUL
                && *p.offset(len as isize) as ::core::ffi::c_int
                    == *ptr.offset(len as isize) as ::core::ffi::c_int
            {
                len += 1;
            }
            if len > 0 as ::core::ffi::c_int {
                len -= utf_head_off(p, p.offset(len as isize));
            }
            p = p.offset(len as isize);
            while *p as ::core::ffi::c_int != NUL {
                AppendCharToRedobuff(K_BS);
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        }
        AppendToRedobuffLit(ptr.offset(len as isize), -1 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn ins_compl_check_keys(
    mut frequency: ::core::ffi::c_int,
    mut in_compl_func: bool,
) {
    unsafe {
        static count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        if !in_compl_func && (using_script() != 0 || ex_normal_busy.get() != 0) {
            return;
        }
        (*count.ptr()) += 1;
        if count.get() < frequency {
            return;
        }
        count.set(0 as ::core::ffi::c_int);
        let mut c: ::core::ffi::c_int = vpeekc_any();
        if c != NUL && !test_disable_char_avail.get() {
            if vim_is_ctrl_x_key(c) as ::core::ffi::c_int != 0 && c != Ctrl_X && c != Ctrl_R {
                c = safe_vgetc();
                compl_shows_dir.set(ins_compl_key2dir(c) as Direction);
                ins_compl_next(
                    false_0 != 0,
                    ins_compl_key2count(c),
                    c != K_UP && c != K_DOWN,
                );
            } else {
                c = safe_vgetc();
                if c != -(253 as ::core::ffi::c_int
                    + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    if c != Ctrl_R && KeyTyped.get() as ::core::ffi::c_int != 0 {
                        compl_interrupted.set(true_0 != 0);
                    }
                    vungetc(c);
                }
            }
        } else {
            let mut normal_mode_strict: bool = ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                && !ctrl_x_mode_line_or_eval()
                && compl_cont_status.get() & CONT_LOCAL == 0
                && !(*cpt_sources_array.ptr()).is_null()
                && cpt_sources_index.get() >= 0 as ::core::ffi::c_int;
            if normal_mode_strict as ::core::ffi::c_int != 0
                && (compl_autocomplete.get() as ::core::ffi::c_int != 0
                    || p_cto.get() > 0 as OptInt)
            {
                check_elapsed_time();
            }
        }
        if compl_pending.get() != 0
            && !got_int.get()
            && cot_flags.get()
                & (kOptCotFlagNoinsert as ::core::ffi::c_int
                    | kOptCotFlagFuzzy as ::core::ffi::c_int)
                    as ::core::ffi::c_uint
                == 0
            && (!compl_autocomplete.get() || ins_compl_has_preinsert() as ::core::ffi::c_int != 0)
        {
            let mut todo: ::core::ffi::c_int = if compl_pending.get() > 0 as ::core::ffi::c_int {
                compl_pending.get()
            } else {
                -compl_pending.get()
            };
            compl_pending.set(0 as ::core::ffi::c_int);
            ins_compl_next(false_0 != 0, todo, true_0 != 0);
        }
    }
}
