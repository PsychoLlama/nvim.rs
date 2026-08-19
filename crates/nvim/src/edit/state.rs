//! The Insert-mode state machine: what happens around each key.
//!
//! [`edit`] is the entry point every `i`/`a`/`R`/`gI`/`gr` reaches, and all
//! it does is fill an `InsertState` and hand it to [`insert_enter`], which
//! sets the mode up and then runs the generic loop in `state.rs`.  That loop
//! alternates between two callbacks:
//!
//! - [`insert_check`], run once *before* each key is asked for.  It is the
//!   home of everything that has to happen while no key is available: the
//!   postponed redraw, the one-line scroll that keeps a wrapping line
//!   visible, cursor validation, folds, and the prompt buffer's prompt.
//! - [`insert_execute`], run with the key that arrived.  It pre-processes it
//!   -- the completion machine may claim it, CTRL-\ and CTRL-V take another
//!   key, 'rightleft' mirrors the arrows -- before [`insert_handle_key`]
//!   decides what it means.
//!
//! Either callback answers 0 to leave the mode.  `insert_enter` then asks
//! `ins_esc` whether that really was the end, because a count means the whole
//! insert repeats and the loop runs again.  [`edit`]'s own answer says
//! whether the mode was left by `i_CTRL-O`, which is what tells
//! `do_pending_operator` to run one Normal-mode command and come back.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::guard::Keys;
use crate::keycodes::{K_C_LEFT, K_C_RIGHT, K_EVENT, K_IGNORE, K_NOP};
use crate::r#move::WinValid;
use crate::types::{FAIL, NUL, OK};

/// Set Insert mode up, run the state loop until it really ends, and tear it
/// down again.
///
/// # Safety
/// `s` must point to an `InsertState` whose `state`, `cmdchar`, `startln`
/// and `count` the caller has filled in.
unsafe fn insert_enter(s: *mut InsertState) {
    unsafe {
        (*s).did_backspace = true;
        (*s).old_topfill = -1;
        (*s).replaceState = MODE_REPLACE;
        (*s).cmdchar_todo = (*s).cmdchar;
        (*s).ins_just_started = true;
        // Remember whether editing was restarted after CTRL-O.
        did_restart_edit.set(restart_edit.get());
        // Sleep before redrawing; needed for `CTRL-O :` that ends in an error
        // message.
        msg_check_for_delay(true);
        // Set Insstart_orig to Insstart.
        update_Insstart_orig.set(true);

        ins_compl_clear(); // clear stuff for CTRL-X mode

        // Trigger InsertEnter -- but not for `r<CR>` or `grx`.
        if (*s).cmdchar != 'r' as c_int && (*s).cmdchar != 'v' as c_int {
            trigger_insert_enter((*s).cmdchar);
        }

        if (*where_paste_started.ptr()).lnum != 0 {
            Insstart.set(where_paste_started.get());
        } else {
            Insstart.set((*curwin.get()).w_cursor);
            if (*s).startln != 0 {
                (*Insstart.ptr()).col = 0;
            }
        }
        Insstart_textlen.set(linetabsize_str(get_cursor_line_ptr()) as colnr_T);
        Insstart_blank_vcol.set(MAXCOL as colnr_T);
        if !did_ai.get() {
            ai_col.set(0);
        }

        // Record the command that started the insert, so `.` repeats it.
        if (*s).cmdchar != NUL && restart_edit.get() == 0 {
            ResetRedobuff();
            AppendNumberToRedobuff((*s).count);
            if (*s).cmdchar == 'V' as c_int || (*s).cmdchar == 'v' as c_int {
                // `gR` and `gr`.
                AppendCharToRedobuff('g' as c_int);
                AppendCharToRedobuff(if (*s).cmdchar == 'v' as c_int {
                    'r' as c_int
                } else {
                    'R' as c_int
                });
            } else {
                AppendCharToRedobuff((*s).cmdchar);
                if (*s).cmdchar == 'g' as c_int {
                    AppendCharToRedobuff('I' as c_int); // `gI` means "insert in column 1"
                } else if (*s).cmdchar == 'r' as c_int {
                    (*s).count = 1; // `r<CR>` inserts one <CR> however big the count
                }
            }
        }

        if (*s).cmdchar == 'R' as c_int {
            State.set(MODE_REPLACE);
        } else if (*s).cmdchar == 'V' as c_int || (*s).cmdchar == 'v' as c_int {
            State.set(MODE_VREPLACE);
            (*s).replaceState = MODE_VREPLACE;
            orig_line_count.set((*curbuf.get()).b_ml.ml_line_count);
            vr_lines_changed.set(1);
        } else {
            State.set(MODE_INSERT);
        }
        may_trigger_modechanged();
        stop_insert_mode.set(false);

        // The cursor needs positioning again when it is on a TAB, and when
        // the line carries inline virtual text.
        if gchar_cursor() == TAB || buf_meta_total(curbuf.get(), kMTMetaInline) > 0 {
            (*curwin.get())
                .w_valid
                .clear(WinValid::WROW | WinValid::WCOL | WinValid::VIRTCOL);
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
        revins_chars.set(0);
        revins_legal.set(0);
        revins_scol.set(-1);

        // Handle restarting Insert mode: put the cursor back where CTRL-O
        // took it from.
        if restart_edit.get() != 0 && stuff_empty() {
            arrow_used.set((*where_paste_started.ptr()).lnum == 0);
            restart_edit.set(0);
            validate_virtcol(curwin.get());
            update_curswant();
            restore_ctrl_o_column();
            ins_at_eol.set(false);
        } else {
            arrow_used.set(false);
        }

        need_start_insertmode.set(false);
        ins_need_undo.set(true);
        (*where_paste_started.ptr()).lnum = 0;
        can_cindent.set(true);
        if did_restart_edit.get() == 0 {
            // Open a fold at the cursor line, unless it was already open when
            // CTRL-O left.
            foldOpenCursor();
        }

        // `showmode`'s answer is how many lines the message took, which
        // `change_warning` needs so its own message lands below it.
        (*s).i = 0;
        if p_smd.get() != 0 && msg_silent.get() == 0 {
            (*s).i = showmode();
        }
        if did_restart_edit.get() == 0 {
            change_warning(curbuf.get(), if (*s).i == 0 { 0 } else { (*s).i + 1 });
        }

        ui_cursor_shape();
        do_digraph(-1); // clear digraphs

        // Everything in the redo buffer up to here belongs to the command,
        // not to the text that was typed.
        let inserted = get_inserted();
        new_insert_skip.set(inserted.len() as c_int);
        if !inserted.data().is_null() {
            xfree(inserted.data() as *mut ::core::ffi::c_void);
        }
        old_indent.set(0);

        // The mode ends when `ins_esc` says so: a count means the whole
        // insert is typed again.
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
        if (*s).cmdchar != 'r' as c_int && (*s).cmdchar != 'v' as c_int && (*s).c != Ctrl_C {
            ins_apply_autocmds(EVENT_INSERTLEAVE);
        }
        did_cursorhold.set(false);

        // `ins_redraw` triggers TextChangedI only when the typeahead buffer
        // is empty, so `b_last_changedtick` is reset here when the event was
        // not blocked by `char_avail()` (`:norm!`, say) and did fire.
        if !char_avail()
            && (*curbuf.get()).b_last_changedtick_i == buf_get_changedtick(curbuf.get())
        {
            (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
        }
    }
}

/// Fire `InsertEnter`, and put the cursor back if the autocommand moved it.
///
/// It is allowed to move the cursor deliberately by setting `v:char`; the
/// restore only happens when `v:char` is still empty.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
unsafe fn trigger_insert_enter(cmdchar: c_int) {
    unsafe {
        let save_cursor = (*curwin.get()).w_cursor;

        let mode: *const c_char = if cmdchar == 'R' as c_int {
            c"r".as_ptr()
        } else if cmdchar == 'V' as c_int {
            c"v".as_ptr()
        } else {
            c"i".as_ptr()
        };
        set_vim_var_string(Vv::Insertmode, mode, 1);
        set_vim_var_string(Vv::Char, ::core::ptr::null(), -1);
        ins_apply_autocmds(EVENT_INSERTENTER);

        // Highlighting may have changed, e.g. for ModeMsg.
        if need_highlight_changed.get() {
            highlight_changed();
        }

        // Make sure the cursor did not move.  `check_cursor_col` is still
        // called in case the text was modified; Insert mode has not started
        // yet, so `State` is faked for it.
        if !equalpos((*curwin.get()).w_cursor, save_cursor)
            && *get_vim_var_str(Vv::Char) as c_int == NUL
            && save_cursor.lnum <= (*curbuf.get()).b_ml.ml_line_count
        {
            let save_state = State.get();
            (*curwin.get()).w_cursor = save_cursor;
            State.set(MODE_INSERT);
            check_cursor_col(curwin.get());
            State.set(save_state);
        }
    }
}

/// Restarting after CTRL-O: step the cursor back onto the position past the
/// last character, where the insert was.
///
/// Only when the insert *was* at the end of the line (`ins_at_eol` on the
/// same line) or the wanted column is past the last character.
///
/// # Safety
/// Must run with a live `curwin`.
unsafe fn restore_ctrl_o_column() {
    unsafe {
        let at_eol = ins_at_eol.get() && (*curwin.get()).w_cursor.lnum == o_lnum.get();
        if !at_eol && (*curwin.get()).w_curswant <= (*curwin.get()).w_virtcol {
            return;
        }
        let ptr = get_cursor_line_ptr().offset((*curwin.get()).w_cursor.col as isize);
        if *ptr as c_int == NUL {
            return;
        }
        if *ptr.offset(1) as c_int == NUL {
            (*curwin.get()).w_cursor.col += 1;
        } else {
            let len = utfc_ptr2len(ptr);
            if *ptr.offset(len as isize) as c_int == NUL {
                (*curwin.get()).w_cursor.col += len;
            }
        }
    }
}

/// The state loop's `check` callback: everything that happens before the
/// next key is asked for.
///
/// Answers 0 to leave Insert mode, and 1 to go on.
///
/// # Safety
/// `state` must be an `InsertState`.
unsafe fn insert_check(state: *mut VimState) -> c_int {
    unsafe {
        let s = state as *mut InsertState;

        if revins_legal.get() == 0 {
            revins_scol.set(-1); // reset on an illegal motion
        } else {
            revins_legal.set(0);
        }
        if arrow_used.get() {
            // Don't repeat the insert when an arrow key was used.
            (*s).count = 0;
        }
        if update_Insstart_orig.get() {
            Insstart_orig.set(Insstart.get());
        }

        if !(*curbuf.get()).terminal.is_null() && !stop_insert_mode.get() {
            // Exiting a terminal buffer's Insert mode: a K_NOP is stuffed so
            // the loop wakes up and sees `stop_insert_mode`.
            stop_insert_mode.set(true);
            restart_edit.set('I' as c_int);
            stuffcharReadbuff(K_NOP);
        }
        if stop_insert_mode.get() && !ins_compl_active() {
            // `:stopinsert` was used, or a terminal buffer was entered.
            (*s).count = 0;
            return 0;
        }

        if !arrow_used.get() {
            (*curwin.get()).w_set_curswant = 1;
        }
        if stuff_empty() {
            did_check_timestamps.set(false);
            if need_check_timestamps.get() {
                check_timestamps(0);
            }
        }

        // The mode message is not scrolled away.
        msg_scroll.set(0);
        if fdo_flags.get() & kOptFdoFlagInsert as ::core::ffi::c_uint != 0 {
            foldOpenCursor();
        }
        if !char_avail() {
            foldCheckClose();
        }
        if bt_prompt(curbuf.get()) {
            init_prompt((*s).cmdchar_todo);
            (*s).cmdchar_todo = NUL;
        }

        may_scroll_for_wrap(s);

        if (*s).count <= 1 {
            update_topline(curwin.get());
        }
        (*s).did_backspace = false;
        if (*s).count <= 1 {
            validate_cursor(curwin.get());
        }

        ins_redraw(true);

        if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
            do_check_scrollbind(true);
        }
        if (*curwin.get()).w_onebuf_opt.wo_crb != 0 {
            do_check_cursorbind();
        }
        if (*s).count <= 1 {
            update_curswant();
        }
        (*s).old_topline = (*curwin.get()).w_topline;
        (*s).old_topfill = (*curwin.get()).w_topfill;

        // `lastc` is the previous *real* key: a K_EVENT is not one.
        if (*s).c != K_EVENT {
            (*s).lastc = (*s).c;
        }

        // `i_CTRL-G_U` armed the latch; this is the one key it covers.
        if dont_sync_undo.get() == KeepUndo::Armed {
            dont_sync_undo.set(KeepUndo::Now);
        } else {
            dont_sync_undo.set(KeepUndo::No);
        }

        if (*s).ins_just_started {
            (*s).ins_just_started = false;
            // Autocomplete: with a word character already before the cursor,
            // start completing without waiting for another key.
            if ins_compl_has_autocomplete() && !char_avail() && (*curwin.get()).w_cursor.col > 0 {
                (*s).c = char_before_cursor();
                if vim_isprintc((*s).c) {
                    ins_compl_enable_autocomplete();
                    ins_compl_init_get_longest();
                    insert_do_complete(s);
                    insert_handle_key_post(s);
                    return 1;
                }
            }
        }
        1
    }
}

/// Scroll up one line so the cursor's wrapped line stays visible.
///
/// The case is typing past the right edge of the last visible line: the
/// cursor's screen column went *backwards* by more than a tab stop, which
/// means the line wrapped, and the cursor is on the last row 'scrolloff'
/// allows.  Not with 'smoothscroll', and not while repeating an insert.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn may_scroll_for_wrap(s: *mut InsertState) {
    unsafe {
        if !((*curbuf.get()).b_mod_set
            && (*curwin.get()).w_onebuf_opt.wo_wrap != 0
            && (*curwin.get()).w_onebuf_opt.wo_sms == 0
            && !(*s).did_backspace
            && (*curwin.get()).w_topline == (*s).old_topline
            && (*curwin.get()).w_topfill == (*s).old_topfill
            && (*s).count <= 1)
        {
            return;
        }

        (*s).mincol = (*curwin.get()).w_wcol;
        validate_cursor_col(curwin.get());

        let tabstop = tabstop_at(
            get_nolist_virtcol(),
            (*curbuf.get()).b_p_ts,
            (*curbuf.get()).b_p_vts_array,
            false,
        );
        if (*curwin.get()).w_wcol < (*s).mincol - tabstop
            && (*curwin.get()).w_wrow as int64_t
                == ((*curwin.get()).w_view_height - 1) as int64_t
                    - get_scrolloff_value(curwin.get())
            && ((*curwin.get()).w_cursor.lnum != (*curwin.get()).w_topline
                || (*curwin.get()).w_topfill > 0)
        {
            if (*curwin.get()).w_topfill > 0 {
                (*curwin.get()).w_topfill -= 1;
            } else if hasFolding(
                curwin.get(),
                (*curwin.get()).w_topline,
                ::core::ptr::null_mut(),
                &raw mut (*s).old_topline,
            ) {
                set_topline(curwin.get(), (*s).old_topline + 1);
            } else {
                set_topline(curwin.get(), (*curwin.get()).w_topline + 1);
            }
        }
    }
}

/// The state loop's `execute` callback: pre-process `key`, then hand it to
/// [`insert_handle_key`].
///
/// Answers 0 to leave Insert mode, -1 to ignore the key entirely, and 1 to
/// go on.
///
/// # Safety
/// `state` must be an `InsertState`.
unsafe fn insert_execute(state: *mut VimState, key: c_int) -> c_int {
    unsafe {
        let s = state as *mut InsertState;

        if stop_insert_mode.get() {
            // Insert mode ended while the key was being read; give it back so
            // Normal mode sees it.
            if key != K_IGNORE && key != K_NOP {
                vungetc(key);
            }
            (*s).count = 0;
            (*s).nomove = true;
            ins_compl_prep(ESC);
            return 0;
        }
        if key == K_IGNORE || key == K_NOP {
            return -1; // get another key
        }
        (*s).c = key;

        // Any key but a K_EVENT resets the CursorHold timer.
        if key != K_EVENT {
            did_cursorhold.set(true);
        }

        if compl_takes_key(s) {
            return 1;
        }

        ins_compl_init_get_longest();
        if ins_compl_prep((*s).c) {
            return 1;
        }

        // CTRL-\ CTRL-N goes to Normal mode, CTRL-\ CTRL-G to the "insert
        // mode" the buffer wants, and CTRL-\ CTRL-O is a one-shot CTRL-O.
        // Anything else after CTRL-\ is put back and the CTRL-\ inserted.
        if (*s).c == Ctrl_BSL {
            ins_redraw(false);
            (*s).c = {
                let _raw_key = Keys::unmapped_with_codes();
                plain_vgetc()
            };
            if (*s).c != Ctrl_N && (*s).c != Ctrl_G && (*s).c != Ctrl_O {
                vungetc((*s).c);
                (*s).c = Ctrl_BSL;
            } else {
                if (*s).c == Ctrl_O {
                    ins_ctrl_o();
                    ins_at_eol.set(false); // don't move the cursor afterwards
                    (*s).nomove = true;
                }
                (*s).count = 0;
                return 0;
            }
        }

        if (*s).c != K_EVENT {
            (*s).c = do_digraph((*s).c);
        }

        if ((*s).c == Ctrl_V || (*s).c == Ctrl_Q) && ctrl_x_mode_cmdline() {
            insert_do_complete(s);
            insert_handle_key_post(s);
            return 1;
        }
        if (*s).c == Ctrl_V || (*s).c == Ctrl_Q {
            ins_ctrl_v();
            (*s).c = Ctrl_V;
            return 1;
        }

        // 'cindent' may want to re-indent on this character before it is even
        // inserted.
        if cindent_on() && ctrl_x_mode_none() {
            (*s).line_is_white = inindent(0);
            if in_cinkeys((*s).c, '!' as c_int, (*s).line_is_white) && stop_arrow() == OK {
                do_c_expr_indent();
                return 1; // don't insert the key
            }
            if can_cindent.get()
                && in_cinkeys((*s).c, '*' as c_int, (*s).line_is_white)
                && stop_arrow() == OK
            {
                do_c_expr_indent();
            }
        }

        if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
            (*s).c = mirror_arrow_key((*s).c);
        }

        if ins_start_select((*s).c) {
            return 1;
        }

        insert_handle_key(s)
    }
}

/// In a 'rightleft' window, left and right swap over.
const fn mirror_arrow_key(c: c_int) -> c_int {
    match c {
        K_LEFT => K_RIGHT,
        K_S_LEFT => K_S_RIGHT,
        K_C_LEFT => K_C_RIGHT,
        K_RIGHT => K_LEFT,
        K_S_RIGHT => K_S_LEFT,
        K_C_RIGHT => K_C_LEFT,
        other => other,
    }
}

/// Does the completion machine want this key for itself?
///
/// Only while a completion is showing and the cursor is inside the word
/// being completed.  Backspace shrinks the leader, CTRL-L takes the rest of
/// the match, an ordinary character extends the leader, and CTRL-Y (or
/// `<CR>` under 'completeopt' `noinsert`) accepts.  Answers true when the key
/// was consumed.
///
/// # Safety
/// `s` must point to a live `InsertState`.
unsafe fn compl_takes_key(s: *mut InsertState) -> bool {
    unsafe {
        if !(ins_compl_active()
            && (*curwin.get()).w_cursor.col >= ins_compl_col()
            && ins_compl_has_shown_match()
            && pum_wanted())
        {
            return false;
        }

        // Backspace inside the leader: shrink it rather than deleting text.
        if ((*s).c == K_BS || (*s).c == Ctrl_H) && (*curwin.get()).w_cursor.col > ins_compl_col() {
            (*s).c = ins_compl_bs();
            if (*s).c == NUL {
                return true;
            }
        }

        if ins_compl_used_match() {
            return false;
        }

        // CTRL-L: take the rest of the shown match.
        if (*s).c == Ctrl_L && (!ctrl_x_mode_line_or_eval() || ins_compl_long_shown_match()) {
            ins_compl_addfrommatch();
            return true;
        }

        // An ordinary character extends the leader.  `InsertCharPre` may
        // replace it with a whole string, which goes in a character at a
        // time.
        if ins_compl_accept_char((*s).c) {
            let str = do_insert_char_pre((*s).c);
            if str.is_null() {
                ins_compl_addleader((*s).c);
            } else {
                let mut p = str;
                while *p as c_int != NUL {
                    ins_compl_addleader(utf_ptr2char(p));
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
                xfree(str as *mut ::core::ffi::c_void);
            }
            return true;
        }

        // CTRL-Y accepts the match; so does <CR> under 'completeopt'
        // `noinsert`.
        if ((*s).c == Ctrl_Y
            || (ins_compl_enter_selects() && ((*s).c == CAR || (*s).c == K_KENTER || (*s).c == NL)))
            && stop_arrow() == OK
        {
            ins_compl_delete(false);
            if ins_compl_preinsert_longest() && !ins_compl_is_match_selected() {
                ins_compl_insert(false, true);
                ins_compl_init_get_longest();
                return true;
            }
            ins_compl_insert(false, false);
        } else if ascii_iswhite_nl_or_nul((*s).c) && ins_compl_preinsert_effect() {
            ins_compl_delete(false);
        }
        false
    }
}

/// Run one round of completion for the key in `s->c`.
///
/// # Safety
/// `s` must point to a live `InsertState`.
pub(crate) unsafe fn insert_do_complete(s: *mut InsertState) {
    unsafe {
        compl_busy.set(true);
        // Folds must not be updated while the popup menu is being built.
        (*disable_fold_update.ptr()) += 1;
        if ins_complete((*s).c, true) == FAIL {
            compl_status_clear();
        }
        (*disable_fold_update.ptr()) -= 1;
        compl_busy.set(false);
        can_si.set(may_do_si());
    }
}

/// The tail every key runs after [`insert_handle_key`].
///
/// # Safety
/// `s` must point to a live `InsertState`.
pub(crate) unsafe fn insert_handle_key_post(s: *mut InsertState) {
    unsafe {
        if (*s).c != K_EVENT && ctrl_x_mode_normal() {
            did_cursorhold.set(false);
        }
        // The completion popup belongs to the window it was started in.
        if ins_compl_active() && !ins_compl_win_active(curwin.get()) {
            ins_compl_cancel();
        }
        if arrow_used.get() {
            (*s).inserted_space = 0;
        }
        // 'cindent': re-indent now that the character is in.
        if can_cindent.get()
            && cindent_on()
            && ctrl_x_mode_normal()
            && in_cinkeys((*s).c, ' ' as c_int, (*s).line_is_white)
            && stop_arrow() == OK
        {
            do_c_expr_indent();
        }
    }
}

/// Start inserting text.
///
/// `cmdchar` is the command that started the insert: `i` (insert), `a`
/// (append), `R` (replace), `r` (`r<CR>`, one CR -- the count may be greater
/// than 1 for redo, but still only one CR goes in and `<Esc>` is not used),
/// `g` (`gI`), `V` (`gR`, Virtual Replace) or `v` (`gr`, one character of
/// Virtual Replace).  `startln` inserts at the start of the line.
///
/// Not called recursively: for `i_CTRL-O` it returns and lets the caller
/// handle the Normal-mode command, which is what the answer means.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn edit(cmdchar: c_int, startln: bool, count: c_int) -> bool {
    unsafe {
        if !(*curbuf.get()).terminal.is_null() {
            if ex_normal_busy.get() != 0 {
                // Do not enter terminal mode from `:normal`; ask for Insert
                // mode again after it finishes.
                restart_edit.set('i' as c_int);
                force_restart_edit.set(true);
                return false;
            }
            return terminal_enter();
        }

        // Don't allow inserting in the sandbox, or while textlock is set.
        if sandbox.get() != 0 {
            emsg(gettext(&raw const e_sandbox as *const c_char));
            return false;
        }
        if textlock.get() != 0
            || ins_compl_active()
            || compl_busy.get()
            || pum_visible()
            || expr_map_locked()
        {
            emsg(gettext(&raw const e_textlock as *const c_char));
            return false;
        }

        let mut s = InsertState {
            state: VimState {
                check: Some(insert_check as unsafe fn(*mut VimState) -> c_int),
                execute: Some(insert_execute as unsafe fn(*mut VimState, c_int) -> c_int),
            },
            ca: ::core::ptr::null_mut(),
            mincol: 0,
            cmdchar,
            cmdchar_todo: 0,
            ins_just_started: false,
            startln: startln as c_int,
            count,
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
        };
        insert_enter(&raw mut s);
        s.c == Ctrl_O
    }
}

/// Does the next change still owe a `u_save`?
pub(crate) fn ins_need_undo_get() -> bool {
    ins_need_undo.get()
}
