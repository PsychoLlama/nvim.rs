//! The Insert-mode state machine: what happens around each key.
//!
//! `edit()` is the entry point every `i`/`a`/`R`/`gI`/`gr` reaches, and all
//! it does is fill an `InsertState` and hand it to the generic state loop in
//! `state.rs`.  The loop then alternates between `insert_check` -- run once
//! before each key, and the home of everything that has to happen while no
//! key is available (the postponed redraw, 'textwidth' auto-wrap, cursor
//! column fixing, folds) -- and `insert_execute`, which pre-processes the
//! key that arrived (language mapping, CTRL-V, the completion state
//! machine's claim on it) before `insert_handle_key` decides what it means.
//! `insert_handle_key_post` is the tail every key runs afterwards.
//!
//! The mode is left by returning 0 from `insert_check`/`insert_execute`;
//! `edit()`'s answer says whether it was left by `i_CTRL-O`, which is what
//! tells `do_pending_operator` to run one Normal-mode command and come
//! back.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn insert_enter(mut s: *mut InsertState) {
    unsafe {
        (*s).did_backspace = true_0 != 0;
        (*s).old_topfill = -1 as ::core::ffi::c_int;
        (*s).replaceState = MODE_REPLACE;
        (*s).cmdchar_todo = (*s).cmdchar;
        (*s).ins_just_started = true_0 != 0;
        did_restart_edit.set(restart_edit.get());
        msg_check_for_delay(true_0 != 0);
        update_Insstart_orig.set(true_0 != 0);
        ins_compl_clear();
        if (*s).cmdchar != 'r' as ::core::ffi::c_int && (*s).cmdchar != 'v' as ::core::ffi::c_int {
            let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
            let ptr: *const ::core::ffi::c_char = if (*s).cmdchar == 'R' as ::core::ffi::c_int {
                b"r\0".as_ptr() as *const ::core::ffi::c_char
            } else if (*s).cmdchar == 'V' as ::core::ffi::c_int {
                b"v\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"i\0".as_ptr() as *const ::core::ffi::c_char
            };
            set_vim_var_string(VV_INSERTMODE, ptr, 1 as ptrdiff_t);
            set_vim_var_string(
                VV_CHAR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                -1 as ptrdiff_t,
            );
            ins_apply_autocmds(EVENT_INSERTENTER);
            if need_highlight_changed.get() {
                highlight_changed();
            }
            if !equalpos((*curwin.get()).w_cursor, save_cursor)
                && *get_vim_var_str(VV_CHAR) as ::core::ffi::c_int == NUL
                && save_cursor.lnum <= (*curbuf.get()).b_ml.ml_line_count
            {
                let mut save_state: ::core::ffi::c_int = State.get();
                (*curwin.get()).w_cursor = save_cursor;
                State.set(MODE_INSERT);
                check_cursor_col(curwin.get());
                State.set(save_state);
            }
        }
        if (*where_paste_started.ptr()).lnum != 0 as linenr_T {
            Insstart.set(where_paste_started.get());
        } else {
            Insstart.set((*curwin.get()).w_cursor);
            if (*s).startln != 0 {
                (*Insstart.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
            }
        }
        Insstart_textlen.set(linetabsize_str(get_cursor_line_ptr()) as colnr_T);
        Insstart_blank_vcol.set(MAXCOL as ::core::ffi::c_int as colnr_T);
        if !did_ai.get() {
            ai_col.set(0 as ::core::ffi::c_int as colnr_T);
        }
        if (*s).cmdchar != NUL && restart_edit.get() == 0 as ::core::ffi::c_int {
            ResetRedobuff();
            AppendNumberToRedobuff((*s).count);
            if (*s).cmdchar == 'V' as ::core::ffi::c_int
                || (*s).cmdchar == 'v' as ::core::ffi::c_int
            {
                AppendCharToRedobuff('g' as ::core::ffi::c_int);
                AppendCharToRedobuff(if (*s).cmdchar == 'v' as ::core::ffi::c_int {
                    'r' as ::core::ffi::c_int
                } else {
                    'R' as ::core::ffi::c_int
                });
            } else {
                AppendCharToRedobuff((*s).cmdchar);
                if (*s).cmdchar == 'g' as ::core::ffi::c_int {
                    AppendCharToRedobuff('I' as ::core::ffi::c_int);
                } else if (*s).cmdchar == 'r' as ::core::ffi::c_int {
                    (*s).count = 1 as ::core::ffi::c_int;
                }
            }
        }
        if (*s).cmdchar == 'R' as ::core::ffi::c_int {
            State.set(MODE_REPLACE);
        } else if (*s).cmdchar == 'V' as ::core::ffi::c_int
            || (*s).cmdchar == 'v' as ::core::ffi::c_int
        {
            State.set(MODE_VREPLACE);
            (*s).replaceState = MODE_VREPLACE;
            orig_line_count.set((*curbuf.get()).b_ml.ml_line_count);
            vr_lines_changed.set(1 as ::core::ffi::c_int);
        } else {
            State.set(MODE_INSERT);
        }
        may_trigger_modechanged();
        stop_insert_mode.set(false_0 != 0);
        if gchar_cursor() == TAB || buf_meta_total(curbuf.get(), kMTMetaInline) > 0 as uint32_t {
            (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
        }
        if (*curbuf.get()).b_p_iminsert == B_IMODE_LMAP as OptInt {
            (*State.ptr()) |= MODE_LANGMAP;
        }
        setmouse();
        clear_showcmd();
        revins_on.set(State.get() == MODE_INSERT && p_ri.get() != 0);
        if revins_on.get() {
            undisplay_dollar();
        }
        revins_chars.set(0 as ::core::ffi::c_int);
        revins_legal.set(0 as ::core::ffi::c_int);
        revins_scol.set(-1 as ::core::ffi::c_int);
        if restart_edit.get() != 0 as ::core::ffi::c_int && stuff_empty() as ::core::ffi::c_int != 0
        {
            arrow_used.set((*where_paste_started.ptr()).lnum == 0 as linenr_T);
            restart_edit.set(0 as ::core::ffi::c_int);
            validate_virtcol(curwin.get());
            update_curswant();
            let mut ptr_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            if (ins_at_eol.get() as ::core::ffi::c_int != 0
                && (*curwin.get()).w_cursor.lnum == o_lnum.get()
                || (*curwin.get()).w_curswant > (*curwin.get()).w_virtcol)
                && {
                    ptr_0 = get_cursor_line_ptr().offset((*curwin.get()).w_cursor.col as isize);
                    *ptr_0 as ::core::ffi::c_int != NUL
                }
            {
                if *ptr_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                    (*curwin.get()).w_cursor.col += 1;
                } else {
                    (*s).i = utfc_ptr2len(ptr_0);
                    if *ptr_0.offset((*s).i as isize) as ::core::ffi::c_int == NUL {
                        (*curwin.get()).w_cursor.col += (*s).i;
                    }
                }
            }
            ins_at_eol.set(false_0 != 0);
        } else {
            arrow_used.set(false_0 != 0);
        }
        need_start_insertmode.set(false_0 != 0);
        ins_need_undo.set(true_0 != 0);
        (*where_paste_started.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
        can_cindent.set(true_0 != 0);
        if did_restart_edit.get() == 0 as ::core::ffi::c_int {
            foldOpenCursor();
        }
        (*s).i = 0 as ::core::ffi::c_int;
        if p_smd.get() != 0 && msg_silent.get() == 0 as ::core::ffi::c_int {
            (*s).i = showmode();
        }
        if did_restart_edit.get() == 0 as ::core::ffi::c_int {
            change_warning(
                curbuf.get(),
                if (*s).i == 0 as ::core::ffi::c_int {
                    0 as ::core::ffi::c_int
                } else {
                    (*s).i + 1 as ::core::ffi::c_int
                },
            );
        }
        ui_cursor_shape();
        do_digraph(-1 as ::core::ffi::c_int);
        let mut inserted: String_0 = get_inserted();
        new_insert_skip.set(inserted.size as ::core::ffi::c_int);
        if !inserted.data.is_null() {
            xfree(inserted.data as *mut ::core::ffi::c_void);
        }
        old_indent.set(0 as ::core::ffi::c_int);
        loop {
            state_enter(&raw mut (*s).state);
            if ins_esc(&raw mut (*s).count, (*s).cmdchar, (*s).nomove) {
                break;
            }
        }
        if ins_at_eol.get() {
            o_lnum.set((*curwin.get()).w_cursor.lnum);
        }
        pum_check_clear();
        foldUpdateAfterInsert();
        if (*s).cmdchar != 'r' as ::core::ffi::c_int
            && (*s).cmdchar != 'v' as ::core::ffi::c_int
            && (*s).c != Ctrl_C
        {
            ins_apply_autocmds(EVENT_INSERTLEAVE);
        }
        did_cursorhold.set(false_0 != 0);
        if !char_avail()
            && (*curbuf.get()).b_last_changedtick_i == buf_get_changedtick(curbuf.get())
        {
            (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
        }
    }
}

unsafe extern "C" fn insert_check(mut state: *mut VimState) -> ::core::ffi::c_int {
    unsafe {
        let mut s: *mut InsertState = state as *mut InsertState;
        if revins_legal.get() == 0 {
            revins_scol.set(-1 as ::core::ffi::c_int);
        } else {
            revins_legal.set(0 as ::core::ffi::c_int);
        }
        if arrow_used.get() {
            (*s).count = 0 as ::core::ffi::c_int;
        }
        if update_Insstart_orig.get() {
            Insstart_orig.set(Insstart.get());
        }
        if !(*curbuf.get()).terminal.is_null() && !stop_insert_mode.get() {
            stop_insert_mode.set(true_0 != 0);
            restart_edit.set('I' as ::core::ffi::c_int);
            stuffcharReadbuff(
                -(253 as ::core::ffi::c_int
                    + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
            );
        }
        if stop_insert_mode.get() as ::core::ffi::c_int != 0 && !ins_compl_active() {
            (*s).count = 0 as ::core::ffi::c_int;
            return 0 as ::core::ffi::c_int;
        }
        if !arrow_used.get() {
            (*curwin.get()).w_set_curswant = true_0;
        }
        if stuff_empty() {
            did_check_timestamps.set(false_0 != 0);
            if need_check_timestamps.get() {
                check_timestamps(false_0);
            }
        }
        msg_scroll.set(false_0);
        if fdo_flags.get() & kOptFdoFlagInsert as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            foldOpenCursor();
        }
        if !char_avail() {
            foldCheckClose();
        }
        if bt_prompt(curbuf.get()) {
            init_prompt((*s).cmdchar_todo);
            (*s).cmdchar_todo = NUL;
        }
        if (*curbuf.get()).b_mod_set as ::core::ffi::c_int != 0
            && (*curwin.get()).w_onebuf_opt.wo_wrap != 0
            && (*curwin.get()).w_onebuf_opt.wo_sms == 0
            && !(*s).did_backspace
            && (*curwin.get()).w_topline == (*s).old_topline
            && (*curwin.get()).w_topfill == (*s).old_topfill
            && (*s).count <= 1 as ::core::ffi::c_int
        {
            (*s).mincol = (*curwin.get()).w_wcol;
            validate_cursor_col(curwin.get());
            if (*curwin.get()).w_wcol
                < (*s).mincol
                    - tabstop_at(
                        get_nolist_virtcol(),
                        (*curbuf.get()).b_p_ts,
                        (*curbuf.get()).b_p_vts_array,
                        false_0 != 0,
                    )
                && (*curwin.get()).w_wrow as int64_t
                    == ((*curwin.get()).w_view_height - 1 as ::core::ffi::c_int) as int64_t
                        - get_scrolloff_value(curwin.get())
                && ((*curwin.get()).w_cursor.lnum != (*curwin.get()).w_topline
                    || (*curwin.get()).w_topfill > 0 as ::core::ffi::c_int)
            {
                if (*curwin.get()).w_topfill > 0 as ::core::ffi::c_int {
                    (*curwin.get()).w_topfill -= 1;
                } else if hasFolding(
                    curwin.get(),
                    (*curwin.get()).w_topline,
                    ::core::ptr::null_mut::<linenr_T>(),
                    &raw mut (*s).old_topline,
                ) {
                    set_topline(curwin.get(), (*s).old_topline + 1 as linenr_T);
                } else {
                    set_topline(curwin.get(), (*curwin.get()).w_topline + 1 as linenr_T);
                }
            }
        }
        if (*s).count <= 1 as ::core::ffi::c_int {
            update_topline(curwin.get());
        }
        (*s).did_backspace = false_0 != 0;
        if (*s).count <= 1 as ::core::ffi::c_int {
            validate_cursor(curwin.get());
        }
        ins_redraw(true_0 != 0);
        if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
            do_check_scrollbind(true_0 != 0);
        }
        if (*curwin.get()).w_onebuf_opt.wo_crb != 0 {
            do_check_cursorbind();
        }
        if (*s).count <= 1 as ::core::ffi::c_int {
            update_curswant();
        }
        (*s).old_topline = (*curwin.get()).w_topline;
        (*s).old_topfill = (*curwin.get()).w_topfill;
        if (*s).c
            != -(253 as ::core::ffi::c_int
                + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            (*s).lastc = (*s).c;
        }
        if dont_sync_undo.get() as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
            dont_sync_undo.set(kTrue);
        } else {
            dont_sync_undo.set(kFalse);
        }
        if (*s).ins_just_started {
            (*s).ins_just_started = false_0 != 0;
            if ins_compl_has_autocomplete() as ::core::ffi::c_int != 0
                && !char_avail()
                && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
            {
                (*s).c = char_before_cursor();
                if vim_isprintc((*s).c) {
                    ins_compl_enable_autocomplete();
                    ins_compl_init_get_longest();
                    insert_do_complete(s);
                    insert_handle_key_post(s);
                    return 1 as ::core::ffi::c_int;
                }
            }
        }
        return 1 as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn insert_execute(
    mut state: *mut VimState,
    mut key: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let s: *mut InsertState = state as *mut InsertState;
        if stop_insert_mode.get() {
            if key
                != -(253 as ::core::ffi::c_int
                    + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                && key
                    != -(253 as ::core::ffi::c_int
                        + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                vungetc(key);
            }
            (*s).count = 0 as ::core::ffi::c_int;
            (*s).nomove = true_0 != 0;
            ins_compl_prep(ESC);
            return 0 as ::core::ffi::c_int;
        }
        if key
            == -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || key
                == -(253 as ::core::ffi::c_int
                    + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            return -1 as ::core::ffi::c_int;
        }
        (*s).c = key;
        if key
            != -(253 as ::core::ffi::c_int
                + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            did_cursorhold.set(true_0 != 0);
        }
        if ins_compl_active() as ::core::ffi::c_int != 0
            && (*curwin.get()).w_cursor.col >= ins_compl_col()
            && ins_compl_has_shown_match() as ::core::ffi::c_int != 0
            && pum_wanted() as ::core::ffi::c_int != 0
        {
            if ((*s).c == K_BS || (*s).c == Ctrl_H)
                && (*curwin.get()).w_cursor.col > ins_compl_col()
                && {
                    (*s).c = ins_compl_bs();
                    (*s).c == NUL
                }
            {
                return 1 as ::core::ffi::c_int;
            }
            if !ins_compl_used_match() {
                if (*s).c == Ctrl_L
                    && (!ctrl_x_mode_line_or_eval()
                        || ins_compl_long_shown_match() as ::core::ffi::c_int != 0)
                {
                    ins_compl_addfrommatch();
                    return 1 as ::core::ffi::c_int;
                }
                if ins_compl_accept_char((*s).c) {
                    let mut str: *mut ::core::ffi::c_char = do_insert_char_pre((*s).c);
                    if !str.is_null() {
                        let mut p: *mut ::core::ffi::c_char = str;
                        while *p as ::core::ffi::c_int != NUL {
                            ins_compl_addleader(utf_ptr2char(p));
                            p = p.offset(utfc_ptr2len(p) as isize);
                        }
                        xfree(str as *mut ::core::ffi::c_void);
                    } else {
                        ins_compl_addleader((*s).c);
                    }
                    return 1 as ::core::ffi::c_int;
                }
                if ((*s).c == Ctrl_Y
                    || ins_compl_enter_selects() as ::core::ffi::c_int != 0
                        && ((*s).c == CAR || (*s).c == K_KENTER || (*s).c == NL))
                    && stop_arrow() == OK
                {
                    ins_compl_delete(false_0 != 0);
                    if ins_compl_preinsert_longest() as ::core::ffi::c_int != 0
                        && !ins_compl_is_match_selected()
                    {
                        ins_compl_insert(false_0 != 0, true_0 != 0);
                        ins_compl_init_get_longest();
                        return 1 as ::core::ffi::c_int;
                    } else {
                        ins_compl_insert(false_0 != 0, false_0 != 0);
                    }
                } else if ascii_iswhite_nl_or_nul((*s).c) as ::core::ffi::c_int != 0
                    && ins_compl_preinsert_effect() as ::core::ffi::c_int != 0
                {
                    ins_compl_delete(false_0 != 0);
                }
            }
        }
        ins_compl_init_get_longest();
        if ins_compl_prep((*s).c) {
            return 1 as ::core::ffi::c_int;
        }
        if (*s).c == Ctrl_BSL {
            ins_redraw(false_0 != 0);
            (*no_mapping.ptr()) += 1;
            (*allow_keys.ptr()) += 1;
            (*s).c = plain_vgetc();
            (*no_mapping.ptr()) -= 1;
            (*allow_keys.ptr()) -= 1;
            if (*s).c != Ctrl_N && (*s).c != Ctrl_G && (*s).c != Ctrl_O {
                vungetc((*s).c);
                (*s).c = Ctrl_BSL;
            } else {
                if (*s).c == Ctrl_O {
                    ins_ctrl_o();
                    ins_at_eol.set(false_0 != 0);
                    (*s).nomove = true_0 != 0;
                }
                (*s).count = 0 as ::core::ffi::c_int;
                return 0 as ::core::ffi::c_int;
            }
        }
        if (*s).c
            != -(253 as ::core::ffi::c_int
                + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            (*s).c = do_digraph((*s).c);
        }
        if ((*s).c == Ctrl_V || (*s).c == Ctrl_Q)
            && ctrl_x_mode_cmdline() as ::core::ffi::c_int != 0
        {
            insert_do_complete(s);
            insert_handle_key_post(s);
            return 1 as ::core::ffi::c_int;
        }
        if (*s).c == Ctrl_V || (*s).c == Ctrl_Q {
            ins_ctrl_v();
            (*s).c = Ctrl_V;
            return 1 as ::core::ffi::c_int;
        }
        if cindent_on() as ::core::ffi::c_int != 0 && ctrl_x_mode_none() as ::core::ffi::c_int != 0
        {
            (*s).line_is_white = inindent(0 as ::core::ffi::c_int);
            if in_cinkeys((*s).c, '!' as ::core::ffi::c_int, (*s).line_is_white)
                as ::core::ffi::c_int
                != 0
                && stop_arrow() == OK
            {
                do_c_expr_indent();
                return 1 as ::core::ffi::c_int;
            }
            if can_cindent.get() as ::core::ffi::c_int != 0
                && in_cinkeys((*s).c, '*' as ::core::ffi::c_int, (*s).line_is_white)
                    as ::core::ffi::c_int
                    != 0
                && stop_arrow() == OK
            {
                do_c_expr_indent();
            }
        }
        if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
            match (*s).c {
                K_LEFT => {
                    (*s).c = K_RIGHT;
                }
                K_S_LEFT => {
                    (*s).c = K_S_RIGHT;
                }
                -22013 => {
                    (*s).c = -(253 as ::core::ffi::c_int
                        + ((KE_C_RIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                }
                K_RIGHT => {
                    (*s).c = K_LEFT;
                }
                K_S_RIGHT => {
                    (*s).c = K_S_LEFT;
                }
                -22269 => {
                    (*s).c = -(253 as ::core::ffi::c_int
                        + ((KE_C_LEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                }
                _ => {}
            }
        }
        if ins_start_select((*s).c) {
            return 1 as ::core::ffi::c_int;
        }
        return insert_handle_key(s);
    }
}

pub(crate) unsafe extern "C" fn insert_do_complete(mut s: *mut InsertState) {
    unsafe {
        compl_busy.set(true_0 != 0);
        (*disable_fold_update.ptr()) += 1;
        if ins_complete((*s).c, true_0 != 0) == FAIL {
            compl_status_clear();
        }
        (*disable_fold_update.ptr()) -= 1;
        compl_busy.set(false_0 != 0);
        can_si.set(may_do_si());
    }
}

unsafe extern "C" fn insert_do_cindent(mut s: *mut InsertState) {
    unsafe {
        if in_cinkeys((*s).c, ' ' as ::core::ffi::c_int, (*s).line_is_white) {
            if stop_arrow() == OK {
                do_c_expr_indent();
            }
        }
    }
}

pub(crate) unsafe extern "C" fn insert_handle_key_post(mut s: *mut InsertState) {
    unsafe {
        if (*s).c
            != -(253 as ::core::ffi::c_int
                + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            && ctrl_x_mode_normal() as ::core::ffi::c_int != 0
        {
            did_cursorhold.set(false_0 != 0);
        }
        if ins_compl_active() as ::core::ffi::c_int != 0 && !ins_compl_win_active(curwin.get()) {
            ins_compl_cancel();
        }
        if arrow_used.get() {
            (*s).inserted_space = false_0;
        }
        if can_cindent.get() as ::core::ffi::c_int != 0
            && cindent_on() as ::core::ffi::c_int != 0
            && ctrl_x_mode_normal() as ::core::ffi::c_int != 0
        {
            insert_do_cindent(s);
        }
    }
}

pub unsafe extern "C" fn edit(
    mut cmdchar: ::core::ffi::c_int,
    mut startln: bool,
    mut count: ::core::ffi::c_int,
) -> bool {
    unsafe {
        if !(*curbuf.get()).terminal.is_null() {
            if ex_normal_busy.get() != 0 {
                restart_edit.set('i' as ::core::ffi::c_int);
                force_restart_edit.set(true_0 != 0);
                return false_0 != 0;
            }
            return terminal_enter();
        }
        if sandbox.get() != 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_sandbox as *const ::core::ffi::c_char));
            return false_0 != 0;
        }
        if textlock.get() != 0 as ::core::ffi::c_int
            || ins_compl_active() as ::core::ffi::c_int != 0
            || compl_busy.get() as ::core::ffi::c_int != 0
            || pum_visible() as ::core::ffi::c_int != 0
            || expr_map_locked() as ::core::ffi::c_int != 0
        {
            emsg(gettext(&raw const e_textlock as *const ::core::ffi::c_char));
            return false_0 != 0;
        }
        let mut s: [InsertState; 1] = [InsertState {
            state: VimState {
                check: None,
                execute: None,
            },
            ca: ::core::ptr::null_mut::<cmdarg_T>(),
            mincol: 0,
            cmdchar: 0,
            cmdchar_todo: 0,
            ins_just_started: false,
            startln: 0,
            count: 0,
            c: 0,
            lastc: 0,
            i: 0,
            did_backspace: false,
            line_is_white: false,
            old_topline: 0,
            old_topfill: 0,
            inserted_space: 0,
            replaceState: 0,
            did_restart_edit: 0,
            nomove: false,
        }; 1];
        memset(
            &raw mut s as *mut InsertState as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<InsertState>(),
        );
        (*(&raw mut s as *mut InsertState)).state.execute = Some(
            insert_execute
                as unsafe extern "C" fn(*mut VimState, ::core::ffi::c_int) -> ::core::ffi::c_int,
        ) as state_execute_callback;
        (*(&raw mut s as *mut InsertState)).state.check =
            Some(insert_check as unsafe extern "C" fn(*mut VimState) -> ::core::ffi::c_int)
                as state_check_callback;
        (*(&raw mut s as *mut InsertState)).cmdchar = cmdchar;
        (*(&raw mut s as *mut InsertState)).startln = startln as ::core::ffi::c_int;
        (*(&raw mut s as *mut InsertState)).count = count;
        insert_enter(&raw mut s as *mut InsertState);
        return (*(&raw mut s as *mut InsertState)).c == Ctrl_O;
    }
}

pub unsafe extern "C" fn ins_need_undo_get() -> bool {
    return ins_need_undo.get();
}
