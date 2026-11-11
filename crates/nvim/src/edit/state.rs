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
//! Either callback answers [`Step::Leave`] to leave the mode.  `insert_enter` then asks
//! `ins_esc` whether that really was the end, because a count means the whole
//! insert repeats and the loop runs again.  [`edit`]'s own answer says
//! whether the mode was left by `i_CTRL-O`, which is what tells
//! `do_pending_operator` to run one Normal-mode command and come back.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr::byte_at;
use crate::keycodes::SpecialKey;
use crate::mbyte::Codepoint;
use crate::mbyte::{c_cluster_len, char_at};
use crate::narrow::len_as_int;
use crate::state::Step;
use crate::types::{Failed, Key};
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::guard::{Keys, Suppress};
use crate::r#move::WinValid;

/// Set Insert mode up, run the state loop until it really ends, and tear it
/// down again.
///
/// `s` is an `InsertState` whose `state`, `cmdchar`, `startln` and `count`
/// the caller has filled in.
fn insert_enter(s: &mut InsertState) {
    s.did_backspace = true;
    s.old_topfill = -1;
    s.replace_state = MODE_REPLACE;
    s.cmdchar_todo = s.cmdchar;
    s.ins_just_started = true;
    // Remember whether editing was restarted after CTRL-O.
    DID_RESTART_EDIT.set(RESTART_EDIT.get());
    // Sleep before redrawing; needed for `CTRL-O :` that ends in an error
    // message.
    msg_check_for_delay(true);
    // Set Insstart_orig to Insstart.
    UPDATE_INSSTART_ORIG.set(true);

    ins_compl_clear(); // clear stuff for CTRL-X mode

    // Trigger InsertEnter -- but not for `r<CR>` or `grx`.
    if !s.cmdchar.is('r') && !s.cmdchar.is('v') {
        trigger_insert_enter(s.cmdchar);
    }

    if WHERE_PASTE_STARTED.get().lnum != LineNr::ZERO {
        INSSTART.set(WHERE_PASTE_STARTED.get());
    } else {
        INSSTART.set(Win::current().w_cursor);
        if s.startln {
            INSSTART.set(INSSTART.get().with_col(ColNr::ZERO));
        }
    }
    let mut lines = Lines::current();
    INSSTART_TEXTLEN.set(ColNr::new(linetabsize_str(
        lines.line(Win::current().w_cursor.lnum),
    )));
    INSSTART_BLANK_VCOL.set(ColNr::MAX);
    if !DID_AI.get() {
        AI_COL.set(ColNr::ZERO);
    }

    // Record the command that started the insert, so `.` repeats it.
    if !s.cmdchar.is_nul() && RESTART_EDIT.get().is_nul() {
        reset_redobuff();
        append_to_redobuff_number(s.count);
        if s.cmdchar.is('V') || s.cmdchar.is('v') {
            // `gR` and `gr`.
            append_to_redobuff_char(Key::ascii(b'g'));
            append_to_redobuff_char(if s.cmdchar.is('v') {
                Key::ascii(b'r')
            } else {
                Key::ascii(b'R')
            });
        } else {
            append_to_redobuff_char(s.cmdchar);
            if s.cmdchar.is('g') {
                append_to_redobuff_char(Key::ascii(b'I')); // `gI` means "insert in column 1"
            } else if s.cmdchar.is('r') {
                s.count = 1; // `r<CR>` inserts one <CR> however big the count
            }
        }
    }

    if s.cmdchar.is('R') {
        STATE.set(MODE_REPLACE);
    } else if s.cmdchar.is('V') || s.cmdchar.is('v') {
        STATE.set(MODE_VREPLACE);
        s.replace_state = MODE_VREPLACE;
        ORIG_LINE_COUNT.set(Buf::current().b_ml.ml_line_count);
        VR_LINES_CHANGED.set(1);
    } else {
        STATE.set(MODE_INSERT);
    }
    may_trigger_modechanged();
    STOP_INSERT_MODE.set(false);

    // The cursor needs positioning again when it is on a TAB, and when
    // the line carries inline virtual text.
    if gchar_cursor() == b'\t' || buf_meta_total(Buf::current(), MT_META_INLINE) > 0 {
        Win::current()
            .w_valid
            .clear(WinValid::WROW | WinValid::WCOL | WinValid::VIRTCOL);
    }
    if Buf::current().b_p_iminsert == B_IMODE_LMAP {
        STATE.set(STATE.get() | MODE_LANGMAP);
    }

    setmouse();
    clear_showcmd();
    REVINS_ON.set(STATE.get() == MODE_INSERT && p_ri());
    if REVINS_ON.get() {
        undisplay_dollar();
    }
    REVINS_CHARS.set(0);
    REVINS_LEGAL.set(0);
    REVINS_SCOL.set(-1);

    // Handle restarting Insert mode: put the cursor back where CTRL-O
    // took it from.
    if !RESTART_EDIT.get().is_nul() && stuff_empty() {
        ARROW_USED.set(WHERE_PASTE_STARTED.get().lnum == LineNr::ZERO);
        RESTART_EDIT.set(Key::NUL);
        validate_virtcol(Win::current());
        update_curswant();
        restore_ctrl_o_column();
        INS_AT_EOL.set(false);
    } else {
        ARROW_USED.set(false);
    }

    NEED_START_INSERTMODE.set(false);
    INS_NEED_UNDO.set(true);
    WHERE_PASTE_STARTED.set(WHERE_PASTE_STARTED.get().with_lnum(LineNr::ZERO));
    CAN_CINDENT.set(true);
    if DID_RESTART_EDIT.get().is_nul() {
        // Open a fold at the cursor line, unless it was already open when
        // CTRL-O left.
        fold_open_cursor();
    }

    // `showmode`'s answer is how many lines the message took, which
    // `change_warning` needs so its own message lands below it.
    s.i = 0;
    if p_smd() && MSG_SILENT.get() == 0 {
        s.i = showmode();
    }
    if DID_RESTART_EDIT.get().is_nul() {
        change_warning(Buf::current(), if s.i == 0 { 0 } else { s.i + 1 });
    }

    ui_cursor_shape();
    reset_digraph(); // clear digraphs

    // Everything in the redo buffer up to here belongs to the command,
    // not to the text that was typed.
    let inserted = get_inserted();
    NEW_INSERT_SKIP.set(len_as_int(inserted.len()));
    OLD_INDENT.set(0);

    // The mode ends when `ins_esc` says so: a count means the whole
    // insert is typed again.
    loop {
        state_enter(ModeState::insert(s));
        if ins_esc(&mut s.count, s.cmdchar, s.nomove) {
            break;
        }
    }

    if INS_AT_EOL.get() {
        O_LNUM.set(Win::current().w_cursor.lnum);
    }
    pum_check_clear();
    fold_update_after_insert();
    if !s.cmdchar.is('r') && !s.cmdchar.is('v') && s.c != Key::CTRL_C {
        ins_apply_autocmds(AutoEvent::InsertLeave);
    }
    DID_CURSORHOLD.set(false);

    // `ins_redraw` triggers TextChangedI only when the typeahead buffer
    // is empty, so `b_last_changedtick` is reset here when the event was
    // not blocked by `char_avail()` (`:norm!`, say) and did fire.
    if !key_available()
        && Buf::current().b_last_changedtick_i == buf_get_changedtick(Buf::current())
    {
        Buf::current().b_last_changedtick = buf_get_changedtick(Buf::current());
    }
}

/// Fire `InsertEnter`, and put the cursor back if the autocommand moved it.
///
/// It is allowed to move the cursor deliberately by setting `v:char`; the
/// restore only happens when `v:char` is still empty.
fn trigger_insert_enter(cmdchar: Key) {
    let save_cursor = Win::current().w_cursor;

    let mode: &[u8] = if cmdchar.is('R') {
        b"r"
    } else if cmdchar.is('V') {
        b"v"
    } else {
        b"i"
    };
    set_vim_var_string(Vv::Insertmode, Some(mode));
    set_vim_var_string(Vv::Char, None);
    ins_apply_autocmds(AutoEvent::InsertEnter);

    // Highlighting may have changed, e.g. for ModeMsg.
    if NEED_HIGHLIGHT_CHANGED.get() {
        highlight_changed();
    }

    // Make sure the cursor did not move.  `check_cursor_col` is still
    // called in case the text was modified; Insert mode has not started
    // yet, so `State` is faked for it.
    if !equalpos(Win::current().w_cursor, save_cursor)
        && with_vim_var_str(Vv::Char, |s| s.is_empty())
        && save_cursor.lnum <= Buf::current().b_ml.ml_line_count
    {
        let save_state = STATE.get();
        Win::current().w_cursor = save_cursor;
        STATE.set(MODE_INSERT);
        check_cursor_col(Win::current());
        STATE.set(save_state);
    }
}

/// Restarting after CTRL-O: step the cursor back onto the position past the
/// last character, where the insert was.
///
/// Only when the insert *was* at the end of the line (`INS_AT_EOL` on the
/// same line) or the wanted column is past the last character.
fn restore_ctrl_o_column() {
    let at_eol = INS_AT_EOL.get() && Win::current().w_cursor.lnum == O_LNUM.get();
    if !at_eol && Win::current().w_curswant <= Win::current().w_virtcol {
        return;
    }
    let cursor = Win::current().w_cursor;
    let step = {
        let mut lines = Lines::current();
        let line = lines.line(cursor.lnum);
        let tail = &line[cursor.col.as_usize().min(line.len())..];
        if tail.is_empty() {
            return;
        }
        if byte_at(tail, 1) == 0 {
            1
        } else {
            let len = c_cluster_len(tail);
            if byte_at(tail, len) == 0 { len } else { 0 }
        }
    };
    Win::current().w_cursor.col += len_as_int(step);
}

/// The state loop's `check` callback: everything that happens before the
/// next key is asked for.
///
/// Answers [`Step::Leave`] to leave Insert mode, and [`Step::Go`] to go on.
pub(crate) fn insert_check(s: &mut InsertState) -> Step {
    if REVINS_LEGAL.get() == 0 {
        REVINS_SCOL.set(-1); // reset on an illegal motion
    } else {
        REVINS_LEGAL.set(0);
    }
    if ARROW_USED.get() {
        // Don't repeat the insert when an arrow key was used.
        s.count = 0;
    }
    if UPDATE_INSSTART_ORIG.get() {
        INSSTART_ORIG.set(INSSTART.get());
    }

    if !Buf::current().terminal.is_null() && !STOP_INSERT_MODE.get() {
        // Exiting a terminal buffer's Insert mode: a K_NOP is stuffed so
        // the loop wakes up and sees `STOP_INSERT_MODE`.
        STOP_INSERT_MODE.set(true);
        RESTART_EDIT.set(Key::ascii(b'I'));
        stuff_readbuf_char(Key::from(SpecialKey::Nop));
    }
    if STOP_INSERT_MODE.get() && !ins_compl_active() {
        // `:stopinsert` was used, or a terminal buffer was entered.
        s.count = 0;
        return Step::Leave;
    }

    if !ARROW_USED.get() {
        Win::current().w_set_curswant = true;
    }
    if stuff_empty() {
        DID_CHECK_TIMESTAMPS.set(false);
        if NEED_CHECK_TIMESTAMPS.get() {
            check_timestamps(false);
        }
    }

    // The mode message is not scrolled away.
    MSG_SCROLL.set(0);
    if FDO_FLAGS.get() & OPT_FDO_FLAG_INSERT as ::core::ffi::c_uint != 0 {
        fold_open_cursor();
    }
    if !key_available() {
        fold_check_close();
    }
    if buf_is_prompt(current_buf()) {
        init_prompt(s.cmdchar_todo);
        s.cmdchar_todo = Key::NUL;
    }

    may_scroll_for_wrap(s);

    if s.count <= 1 {
        update_topline(Win::current());
    }
    s.did_backspace = false;
    if s.count <= 1 {
        validate_cursor(Win::current());
    }

    ins_redraw(true);

    if Win::current().w_onebuf_opt.wo_scb != 0 {
        do_check_scrollbind(true);
    }
    if Win::current().w_onebuf_opt.wo_crb != 0 {
        do_check_cursorbind();
    }
    if s.count <= 1 {
        update_curswant();
    }
    s.old_topline = Win::current().w_topline;
    s.old_topfill = Win::current().w_topfill;

    // `lastc` is the previous *real* key: a K_EVENT is not one.
    if s.c != SpecialKey::Event {
        s.lastc = s.c;
    }

    // `i_CTRL-G_U` armed the latch; this is the one key it covers.
    if DONT_SYNC_UNDO.get() == KeepUndo::Armed {
        DONT_SYNC_UNDO.set(KeepUndo::Now);
    } else {
        DONT_SYNC_UNDO.set(KeepUndo::No);
    }

    if s.ins_just_started {
        s.ins_just_started = false;
        // Autocomplete: with a word character already before the cursor,
        // start completing without waiting for another key.
        if ins_compl_has_autocomplete()
            && !key_available()
            && Win::current().w_cursor.col > ColNr::ZERO
        {
            s.c = Key::from_code(char_before_cursor());
            if vim_isprintc(Codepoint::new(s.c.code())) {
                ins_compl_enable_autocomplete();
                ins_compl_init_get_longest();
                insert_do_complete(s);
                insert_handle_key_post(s);
                return Step::Go;
            }
        }
    }
    Step::Go
}

/// Scroll up one line so the cursor's wrapped line stays visible.
///
/// The case is typing past the right edge of the last visible line: the
/// cursor's screen column went *backwards* by more than a tab stop, which
/// means the line wrapped, and the cursor is on the last row 'scrolloff'
/// allows.  Not with 'smoothscroll', and not while repeating an insert.
fn may_scroll_for_wrap(s: &mut InsertState) {
    if !(Buf::current().b_mod_set
        && Win::current().w_onebuf_opt.wo_wrap != 0
        && Win::current().w_onebuf_opt.wo_sms == 0
        && !s.did_backspace
        && Win::current().w_topline == s.old_topline
        && Win::current().w_topfill == s.old_topfill
        && s.count <= 1)
    {
        return;
    }

    s.mincol = Win::current().w_wcol;
    validate_cursor_col(Win::current());

    let vcol = get_nolist_virtcol();
    let tabstop = Buf::current().tab_width_at(vcol, false);
    if Win::current().w_wcol < s.mincol - tabstop
        && int64_t::from(Win::current().w_wrow)
            == int64_t::from(Win::current().w_view_height - 1) - get_scrolloff_value(Win::current())
        && (Win::current().w_cursor.lnum != Win::current().w_topline
            || Win::current().w_topfill > 0)
    {
        if Win::current().w_topfill > 0 {
            Win::current().w_topfill -= 1;
        } else if Win::current().has_folding(
            Win::current().w_topline,
            None,
            Some(&mut s.old_topline),
        ) {
            set_topline(Win::current(), s.old_topline + 1);
        } else {
            set_topline(Win::current(), Win::current().w_topline + 1);
        }
    }
}

/// The state loop's `execute` callback: pre-process `key`, then hand it to
/// [`insert_handle_key`].
///
/// Answers [`Step::Leave`] to leave Insert mode, [`Step::Again`] to ignore
/// the key entirely, and [`Step::Go`] to go on.
pub(crate) fn insert_execute(s: &mut InsertState, key: Key) -> Step {
    if STOP_INSERT_MODE.get() {
        // Insert mode ended while the key was being read; give it back so
        // Normal mode sees it.
        if key != SpecialKey::Ignore && key != SpecialKey::Nop {
            vungetc(key);
        }
        s.count = 0;
        s.nomove = true;
        ins_compl_prep(Key::ESC);
        return Step::Leave;
    }
    if key == SpecialKey::Ignore || key == SpecialKey::Nop {
        return Step::Again; // get another key
    }
    s.c = key;

    // Any key but a K_EVENT resets the CursorHold timer.
    if key != SpecialKey::Event {
        DID_CURSORHOLD.set(true);
    }

    if compl_takes_key(s) {
        return Step::Go;
    }

    ins_compl_init_get_longest();
    if ins_compl_prep(s.c) {
        return Step::Go;
    }

    // CTRL-\ CTRL-N goes to Normal mode, CTRL-\ CTRL-G to the "insert
    // mode" the buffer wants, and CTRL-\ CTRL-O is a one-shot CTRL-O.
    // Anything else after CTRL-\ is put back and the CTRL-\ inserted.
    if s.c == Key::CTRL_BSL {
        ins_redraw(false);
        s.c = {
            let _raw_key = Keys::unmapped_with_codes();
            plain_vgetc()
        };
        if s.c != Key::CTRL_N && s.c != Key::CTRL_G && s.c != Key::CTRL_O {
            vungetc(s.c);
            s.c = Key::CTRL_BSL;
        } else {
            if s.c == Key::CTRL_O {
                ins_ctrl_o();
                INS_AT_EOL.set(false); // don't move the cursor afterwards
                s.nomove = true;
            }
            s.count = 0;
            return Step::Leave;
        }
    }

    if s.c != SpecialKey::Event {
        s.c = do_digraph(s.c);
    }

    if (s.c == Key::CTRL_V || s.c == Key::CTRL_Q) && ctrl_x_mode_cmdline() {
        insert_do_complete(s);
        insert_handle_key_post(s);
        return Step::Go;
    }
    if s.c == Key::CTRL_V || s.c == Key::CTRL_Q {
        ins_ctrl_v();
        s.c = Key::CTRL_V;
        return Step::Go;
    }

    // 'cindent' may want to re-indent on this character before it is even
    // inserted.
    if cindent_on() && ctrl_x_mode_none() {
        s.line_is_white = inindent(0);
        if in_cinkeys(s.c, b'!', s.line_is_white) && stop_arrow_ok() {
            c_expr_indent();
            return Step::Go; // don't insert the key
        }
        if CAN_CINDENT.get() && in_cinkeys(s.c, b'*', s.line_is_white) && stop_arrow_ok() {
            c_expr_indent();
        }
    }

    if Win::current().w_onebuf_opt.wo_rl != 0 {
        s.c = mirror_arrow_key(s.c);
    }

    if ins_start_select(s.c) {
        return Step::Go;
    }

    insert_handle_key(s)
}

/// In a 'rightleft' window, left and right swap over.
fn mirror_arrow_key(c: Key) -> Key {
    let mirrored = match c.special() {
        Some(SpecialKey::Left) => SpecialKey::Right,
        Some(SpecialKey::SLeft) => SpecialKey::SRight,
        Some(SpecialKey::CLeft) => SpecialKey::CRight,
        Some(SpecialKey::Right) => SpecialKey::Left,
        Some(SpecialKey::SRight) => SpecialKey::SLeft,
        Some(SpecialKey::CRight) => SpecialKey::CLeft,
        _ => return c,
    };
    Key::from(mirrored)
}

/// Does the completion machine want this key for itself?
///
/// Only while a completion is showing and the cursor is inside the word
/// being completed.  Backspace shrinks the leader, CTRL-L takes the rest of
/// the match, an ordinary character extends the leader, and CTRL-Y (or
/// `<CR>` under 'completeopt' `noinsert`) accepts.  Answers true when the key
/// was consumed.
fn compl_takes_key(s: &mut InsertState) -> bool {
    if !(ins_compl_active()
        && Win::current().w_cursor.col >= ins_compl_col()
        && ins_compl_has_shown_match()
        && pum_wanted())
    {
        return false;
    }

    // Backspace inside the leader: shrink it rather than deleting text.
    if (s.c == SpecialKey::Bs || s.c == Key::CTRL_H)
        && Win::current().w_cursor.col > ins_compl_col()
    {
        s.c = ins_compl_bs();
        if s.c.is_nul() {
            return true;
        }
    }

    if ins_compl_used_match() {
        return false;
    }

    // CTRL-L: take the rest of the shown match.
    if s.c == Key::CTRL_L && (!ctrl_x_mode_line_or_eval() || ins_compl_long_shown_match()) {
        ins_compl_addfrommatch();
        return true;
    }

    // An ordinary character extends the leader.  `InsertCharPre` may
    // replace it with a whole string, which goes in a character at a
    // time.
    if ins_compl_accept_char(s.c) {
        match do_insert_char_pre(s.c) {
            None => ins_compl_addleader(s.c),
            Some(text) => {
                let mut at = 0;
                while at < text.len() {
                    ins_compl_addleader(Key::from_code(char_at(&text[at..]).get()));
                    at += c_cluster_len(&text[at..]);
                }
            }
        }
        return true;
    }

    // CTRL-Y accepts the match; so does <CR> under 'completeopt'
    // `noinsert`.
    if (s.c == Key::CTRL_Y
        || (ins_compl_enter_selects()
            && (s.c == Key::CAR || s.c == SpecialKey::Kenter || s.c == Key::NL)))
        && stop_arrow_ok()
    {
        ins_compl_delete(false);
        if ins_compl_preinsert_longest() && !ins_compl_is_match_selected() {
            ins_compl_insert(false, true);
            ins_compl_init_get_longest();
            return true;
        }
        ins_compl_insert(false, false);
    } else if ascii_iswhite_nl_or_nul(s.c.code()) && ins_compl_preinsert_effect() {
        ins_compl_delete(false);
    }
    false
}

/// Run one round of completion for the key in `s->c`.
pub(crate) fn insert_do_complete(s: &mut InsertState) {
    COMPL_BUSY.set(true);
    // Folds must not be updated while the popup menu is being built.
    let folds_frozen = Suppress::fold_update();
    if ins_complete(s.c, true).is_err() {
        compl_status_clear();
    }
    drop(folds_frozen);
    COMPL_BUSY.set(false);
    CAN_SI.set(may_do_si());
}

/// The tail every key runs after [`insert_handle_key`].
pub(crate) fn insert_handle_key_post(s: &mut InsertState) {
    if s.c != SpecialKey::Event && ctrl_x_mode_normal() {
        DID_CURSORHOLD.set(false);
    }
    // The completion popup belongs to the window it was started in.
    if ins_compl_active() && !ins_compl_win_active(Win::current()) {
        ins_compl_cancel();
    }
    if ARROW_USED.get() {
        s.inserted_space = false;
    }
    // 'cindent': re-indent now that the character is in.
    if CAN_CINDENT.get()
        && cindent_on()
        && ctrl_x_mode_normal()
        && in_cinkeys(s.c, b' ', s.line_is_white)
        && stop_arrow_ok()
    {
        c_expr_indent();
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
/// handle the Normal-mode command, which is what `Ok(true)` means.  `Err`
/// when inserting is not allowed here (the sandbox, textlock), which has
/// been reported.
pub(crate) fn edit(cmdchar: Key, startln: bool, count: c_int) -> Result<bool, Failed> {
    if !Buf::current().terminal.is_null() {
        if EX_NORMAL_BUSY.get() != 0 {
            // Do not enter terminal mode from `:normal`; ask for Insert
            // mode again after it finishes.
            RESTART_EDIT.set(Key::ascii(b'i'));
            FORCE_RESTART_EDIT.set(true);
            return Ok(false);
        }
        return Ok(terminal_enter());
    }

    // Don't allow inserting in the sandbox, or while textlock is set.
    if SANDBOX.get() != 0 {
        emsg(gettext(E_SANDBOX));
        return Err(Failed);
    }
    if TEXTLOCK.get() != 0
        || ins_compl_active()
        || COMPL_BUSY.get()
        || pum_visible()
        || expr_map_locked()
    {
        emsg(gettext(E_TEXTLOCK));
        return Err(Failed);
    }

    let mut s = InsertState {
        mincol: 0,
        cmdchar,
        cmdchar_todo: Key::NUL,
        ins_just_started: false,
        startln,
        count,
        c: Key::NUL,
        lastc: Key::NUL,
        i: 0,
        did_backspace: false,
        line_is_white: false,
        old_topline: LineNr::ZERO,
        old_topfill: 0,
        inserted_space: false,
        replace_state: 0,
        nomove: false,
    };
    insert_enter(&mut s);
    Ok(s.c == Key::CTRL_O)
}

/// Does the next change still owe a `u_save`?
pub(crate) fn ins_need_undo_get() -> bool {
    INS_NEED_UNDO.get()
}

/// Is a character available to be read without waiting?
#[inline(always)]
fn key_available() -> bool {
    char_avail()
}

/// Did the arrow-key check leave the insert in a state that can be undone?
#[inline(always)]
fn stop_arrow_ok() -> bool {
    stop_arrow().is_ok()
}

/// Re-indent the current line the way 'cindent'/'indentexpr' wants it.
#[inline(always)]
fn c_expr_indent() {
    do_c_expr_indent()
}

#[cfg(test)]
mod tests {
    //! 'rightleft' swaps the horizontal arrows and nothing else.

    use super::mirror_arrow_key;
    use crate::keycodes::SpecialKey;
    use crate::types::Key;

    #[test]
    fn rightleft_swaps_the_horizontal_arrows() {
        let pairs = [
            (SpecialKey::Left, SpecialKey::Right),
            (SpecialKey::SLeft, SpecialKey::SRight),
            (SpecialKey::CLeft, SpecialKey::CRight),
        ];
        for (a, b) in pairs {
            assert_eq!(mirror_arrow_key(Key::from(a)), b);
            assert_eq!(mirror_arrow_key(Key::from(b)), a);
        }
        for key in [
            Key::from(SpecialKey::Up),
            Key::from(SpecialKey::Home),
            Key::ascii(b'h'),
            Key::ascii(b'l'),
            Key::NUL,
        ] {
            assert_eq!(mirror_arrow_key(key), key);
        }
    }
}
