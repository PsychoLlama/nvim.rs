//! Where one undoable insert ends and the next begins.
//!
//! Insert mode is one undo block from the first character typed to `<Esc>`
//! -- unless the cursor is *moved* in between, which starts a new one.
//! `start_arrow` is that: called by every motion key, it closes the current
//! block and remembers where the insert ended.  `stop_arrow` is the mirror,
//! called before any change, and is where the `u_save` for the block
//! actually happens (`ins_need_undo`).  `stop_insert` is the whole of
//! leaving the mode: fix the cursor column, trim the auto-indent the user
//! never typed on, set `'^` and `'.`, and record the text for `.`.
//!
//! `ins_apply_autocmds` is here because an autocommand may change the
//! buffer, and the undo state has to survive that.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn start_arrow(mut end_insert_pos: *mut pos_T) {
    unsafe {
        start_arrow_common(end_insert_pos, true_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn start_arrow_with_change(
    mut end_insert_pos: *mut pos_T,
    mut end_change: bool,
) {
    unsafe {
        start_arrow_common(end_insert_pos, end_change);
        if !end_change {
            AppendCharToRedobuff(Ctrl_G);
            AppendCharToRedobuff('U' as ::core::ffi::c_int);
        }
    }
}

unsafe extern "C" fn start_arrow_common(mut end_insert_pos: *mut pos_T, mut end_change: bool) {
    unsafe {
        if !arrow_used.get() && end_change as ::core::ffi::c_int != 0 {
            AppendToRedobuff(ESC_STR.as_ptr());
            stop_insert(end_insert_pos, false_0, false_0);
            arrow_used.set(true_0 != 0);
        }
        check_spell_redraw();
    }
}

pub(crate) unsafe extern "C" fn check_spell_redraw() {
    unsafe {
        if spell_redraw_lnum.get() != 0 as linenr_T {
            let mut lnum: linenr_T = spell_redraw_lnum.get();
            spell_redraw_lnum.set(0 as ::core::ffi::c_int as linenr_T);
            redrawWinline(curwin.get(), lnum);
        }
    }
}

pub unsafe extern "C" fn stop_arrow() -> ::core::ffi::c_int {
    unsafe {
        if arrow_used.get() {
            Insstart.set((*curwin.get()).w_cursor);
            if (*Insstart.ptr()).col > (*Insstart_orig.ptr()).col && !ins_need_undo.get() {
                update_Insstart_orig.set(false_0 != 0);
            }
            Insstart_textlen.set(linetabsize_str(get_cursor_line_ptr()) as colnr_T);
            if u_save_cursor() == OK {
                arrow_used.set(false_0 != 0);
                ins_need_undo.set(false_0 != 0);
            }
            ai_col.set(0 as ::core::ffi::c_int as colnr_T);
            if State.get() & VREPLACE_FLAG != 0 {
                orig_line_count.set((*curbuf.get()).b_ml.ml_line_count);
                vr_lines_changed.set(1 as ::core::ffi::c_int);
            }
            ResetRedobuff();
            AppendToRedobuff(b"1i\0".as_ptr() as *const ::core::ffi::c_char);
            new_insert_skip.set(2 as ::core::ffi::c_int);
        } else if ins_need_undo.get() {
            if u_save_cursor() == OK {
                ins_need_undo.set(false_0 != 0);
            }
        }
        foldOpenCursor();
        return if arrow_used.get() as ::core::ffi::c_int != 0
            || ins_need_undo.get() as ::core::ffi::c_int != 0
        {
            FAIL
        } else {
            OK
        };
    }
}

pub(crate) unsafe extern "C" fn stop_insert(
    mut end_insert_pos: *mut pos_T,
    mut esc: ::core::ffi::c_int,
    mut nomove: ::core::ffi::c_int,
) {
    unsafe {
        stop_redo_ins();
        xfree((*replace_stack.ptr()).items as *mut ::core::ffi::c_void);
        (*replace_stack.ptr()).capacity = 0 as size_t;
        (*replace_stack.ptr()).size = (*replace_stack.ptr()).capacity;
        (*replace_stack.ptr()).items = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut inserted: String_0 = get_inserted();
        let mut added: ::core::ffi::c_int = if inserted.data.is_null() {
            0 as ::core::ffi::c_int
        } else {
            inserted.size as ::core::ffi::c_int - new_insert_skip.get()
        };
        if did_restart_edit.get() == 0 as ::core::ffi::c_int || added > 0 as ::core::ffi::c_int {
            xfree((*last_insert.ptr()).data as *mut ::core::ffi::c_void);
            last_insert.set(inserted);
            last_insert_skip.set(if added < 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                new_insert_skip.get()
            });
        } else {
            xfree(inserted.data as *mut ::core::ffi::c_void);
        }
        if !arrow_used.get() && !end_insert_pos.is_null() {
            let mut cc: ::core::ffi::c_int = 0;
            if !ins_need_undo.get() && has_format_option(FO_AUTO) as ::core::ffi::c_int != 0 {
                let mut tpos: pos_T = (*curwin.get()).w_cursor;
                cc = 'x' as ::core::ffi::c_int;
                if (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int && gchar_cursor() == NUL {
                    dec_cursor();
                    cc = gchar_cursor();
                    if !ascii_iswhite(cc) {
                        (*curwin.get()).w_cursor = tpos;
                    }
                }
                auto_format(true_0 != 0, false_0 != 0);
                if ascii_iswhite(cc) {
                    if gchar_cursor() != NUL {
                        inc_cursor();
                    }
                    if gchar_cursor() == NUL
                        && (*curwin.get()).w_cursor.lnum == tpos.lnum
                        && (*curwin.get()).w_cursor.col == tpos.col
                    {
                        (*curwin.get()).w_cursor.coladd = tpos.coladd;
                    }
                }
            }
            check_auto_format(true_0 != 0);
            if nomove == 0
                && did_ai.get() as ::core::ffi::c_int != 0
                && (esc != 0
                    || vim_strchr(p_cpo.get(), CPO_INDENT).is_null()
                        && (*curwin.get()).w_cursor.lnum != (*end_insert_pos).lnum)
                && (*end_insert_pos).lnum <= (*curbuf.get()).b_ml.ml_line_count
            {
                let mut tpos_0: pos_T = (*curwin.get()).w_cursor;
                let mut prev_col: colnr_T = (*end_insert_pos).col;
                (*curwin.get()).w_cursor = *end_insert_pos;
                check_cursor_col(curwin.get());
                loop {
                    if gchar_cursor() == NUL
                        && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
                    {
                        (*curwin.get()).w_cursor.col -= 1;
                    }
                    cc = gchar_cursor();
                    if !ascii_iswhite(cc) {
                        break;
                    }
                    if del_char(true_0 != 0) == FAIL {
                        break;
                    }
                }
                if (*curwin.get()).w_cursor.lnum != tpos_0.lnum {
                    (*curwin.get()).w_cursor = tpos_0;
                } else if (*curwin.get()).w_cursor.col < prev_col {
                    tpos_0 = (*curwin.get()).w_cursor;
                    tpos_0.col += 1;
                    if cc != NUL && gchar_pos(&raw mut tpos_0) == NUL {
                        (*curwin.get()).w_cursor.col += 1;
                    }
                }
                if VIsual_active.get() {
                    check_visual_pos();
                }
            }
        }
        did_ai.set(false_0 != 0);
        did_si.set(false_0 != 0);
        can_si.set(false_0 != 0);
        can_si_back.set(false_0 != 0);
        if !end_insert_pos.is_null() {
            (*curbuf.get()).b_op_start = Insstart.get();
            (*curbuf.get()).b_op_start_orig = Insstart_orig.get();
            (*curbuf.get()).b_op_end = *end_insert_pos;
        }
    }
}

pub unsafe extern "C" fn ins_apply_autocmds(mut event: event_T) -> ::core::ffi::c_int {
    unsafe {
        let mut tick: varnumber_T = buf_get_changedtick(curbuf.get());
        let mut r: ::core::ffi::c_int = apply_autocmds(
            event,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        ) as ::core::ffi::c_int;
        if event as ::core::ffi::c_uint
            != EVENT_INSERTLEAVE as ::core::ffi::c_int as ::core::ffi::c_uint
            && tick != buf_get_changedtick(curbuf.get())
        {
            u_save(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            );
        }
        return r;
    }
}
