//! The keys typed while a completion is up, and how one ends.
//!
//! [`ins_compl_prep`] sees every key first and decides whether it belongs to
//! the completion, ends it, or is inserted; [`ins_compl_stop`] is the unwind.
//! [`ins_compl_bs`] and [`ins_compl_addleader`] are the two that edit the
//! leader and re-filter what is shown.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{
    Ctrl_C, Ctrl_E, Ctrl_N, Ctrl_P, Ctrl_Q, Ctrl_R, Ctrl_V, Ctrl_X, Ctrl_Y, Ctrl_Z,
};
use crate::types::{BsFlag, FAIL, NUL, ShmFlag};

/// Delete one character before the cursor and show the subset of the matches
/// that match the word now before it.
///
/// Answers the character to use, or NUL when the work is done and another
/// character is to be got from the user.
pub unsafe fn ins_compl_bs() -> c_int {
    unsafe {
        if ins_compl_preinsert_effect() {
            ins_compl_delete(false);
        }

        let mut line = get_cursor_line_ptr();
        let mut p = line.offset((*curwin.get()).w_cursor.col as isize);
        // C's MB_PTR_BACK: step back over one whole character.
        p = p.offset(-((utf_head_off(line, p.offset(-1)) + 1) as isize));
        let p_off = p.offset_from(line);
        let from_start = p_off as c_int - compl_col.get();

        // Stop completion when the whole word was deleted.  For Omni
        // completion allow the word to be deleted, we won't match everything.
        // Respect the 'backspace' option.
        if from_start < 0
            || (from_start == 0 && !ctrl_x_mode_omni())
            || ctrl_x_mode_eval()
            || (!can_bs(BsFlag::START) && from_start - compl_length.get() < 0)
        {
            return K_BS;
        }

        // Deleted more than what was used to find matches, or didn't finish
        // finding all matches: look for matches all over again.
        if (*curwin.get()).w_cursor.col <= compl_col.get() + compl_length.get()
            || ins_compl_need_restart()
        {
            ins_compl_restart();
        }

        // ins_compl_restart() calls update_screen(), which may invalidate the
        // pointer.
        // TODO(bfredl): get rid of random update_screen() calls deep inside
        // completion logic
        line = get_cursor_line_ptr();

        clear_string(&compl_leader);
        compl_leader.set(cbuf_to_string(
            line.offset(compl_col.get() as isize),
            (p_off - compl_col.get() as ptrdiff_t) as size_t,
        ));

        // Clear the selection if a menu item is currently selected in
        // autocompletion.
        if compl_autocomplete.get()
            && !compl_first_match.get().is_null()
            && !ins_compl_has_preinsert()
        {
            compl_shown_match.set(compl_first_match.get());
        }

        ins_compl_new_leader();
        if !compl_shown_match.get().is_null() {
            // Make sure the current match is not a hidden item.
            compl_curr_match.set(compl_shown_match.get());
        }
        NUL
    }
}

/// Called after changing `compl_leader`: show the popup menu with a different
/// set of matches, searching again if the previous search was interrupted.
pub(crate) unsafe fn ins_compl_new_leader() {
    unsafe {
        ins_compl_del_pum();
        ins_compl_delete(true);
        ins_compl_insert_bytes(
            (*compl_leader.ptr())
                .data()
                .offset(get_compl_len() as isize),
            -1,
        );
        compl_used_match.set(false);

        if p_acl.get() > 0 {
            pum_undisplay(true);
            redraw_later(curwin.get(), UPD_VALID);
            update_screen(); // Show char (deletion) immediately
            ui_flush();
        }

        if compl_started.get() {
            ins_compl_set_original_text((*compl_leader.ptr()).data(), (*compl_leader.ptr()).len());
            if is_cpt_func_refresh_always() {
                cpt_compl_refresh();
            }
            if cot_fuzzy() {
                ins_compl_fuzzy_sort();
            }
        } else {
            spell_bad_len.set(0); // need to redetect bad word
            // Matches were cleared, need to search for them now. Set
            // "compl_restarting" to avoid that the first match is inserted.
            compl_restarting.set(true);
            if ins_compl_has_autocomplete() {
                ins_compl_enable_autocomplete();
            } else {
                compl_autocomplete.set(false);
            }
            if ins_complete(Ctrl_N, true) == FAIL {
                compl_cont_status.set(0);
            }
            compl_restarting.set(false);
        }

        compl_enter_selects.set(!compl_used_match.get() && compl_selected_item.get() != -1);

        // Show the popup menu with a different set of matches.
        ins_compl_show_pum();

        // Don't let Enter select the original text when there is no popup menu.
        if (*compl_match_array.ptr()).is_null() {
            compl_enter_selects.set(false);
        } else if ins_compl_has_preinsert() && !(*compl_leader.ptr()).is_empty() {
            ins_compl_insert(true, false);
        } else if compl_started.get()
            && ins_compl_preinsert_longest()
            && !(*compl_leader.ptr()).is_empty()
            && !ins_compl_preinsert_effect()
        {
            ins_compl_insert(true, true);
        }
        // Don't let Enter select when a user function with refresh_always is used.
        if ins_compl_refresh_always() {
            compl_enter_selects.set(false);
        }
    }
}

/// Append one character to the match leader. May reduce the number of matches.
pub unsafe fn ins_compl_addleader(c: c_int) {
    unsafe {
        if ins_compl_preinsert_effect() {
            ins_compl_delete(false);
        }

        if stop_arrow() == FAIL {
            return;
        }
        let cc = utf_char2len(c);
        if cc > 1 {
            let mut buf = [0 as c_char; MB_MAXCHAR + 1];
            utf_char2bytes(c, buf.as_mut_ptr());
            buf[cc as usize] = NUL as c_char;
            ins_char_bytes(buf.as_mut_ptr(), cc as size_t);
        } else {
            ins_char(c);
        }

        // If we didn't complete finding matches we must search again.
        if ins_compl_need_restart() {
            ins_compl_restart();
        }

        clear_string(&compl_leader);
        compl_leader.set(cbuf_to_string(
            get_cursor_line_ptr().offset(compl_col.get() as isize),
            ((*curwin.get()).w_cursor.col - compl_col.get()) as size_t,
        ));
        ins_compl_new_leader();
    }
}

/// Set up for finding completions again without leaving CTRL-X mode, after BS
/// or a key was typed while still searching for matches.
pub(crate) unsafe fn ins_compl_restart() {
    unsafe {
        // Update the screen before restarting, so that if completion is
        // blocked we stay at the last popup menu and reduce flicker.
        update_screen(); // TODO(bfredl): no.
        ins_compl_free();
        compl_started.set(false);
        compl_matches.set(0);
        compl_cont_status.set(0);
        compl_cont_mode.set(0);
        cpt_sources_clear();
        compl_autocomplete.set(false);
        compl_from_nonkeyword.set(false);
        compl_num_bests.set(0);
    }
}

/// Replace the first match — the original text — with `str`.
pub(crate) unsafe fn ins_compl_set_original_text(str: *mut c_char, len: size_t) {
    unsafe {
        let replace = |m: *mut compl_T| {
            xfree((*m).cp_str.data().cast::<c_void>());
            (*m).cp_str = cbuf_to_string(str, len);
        };
        // The CP_ORIGINAL_TEXT flag is at the first item, or possibly at the
        // last one for backward completion.
        let first = compl_first_match.get();
        if match_at_original_text(first) {
            // Safety check.
            replace(first);
        } else if !(*first).cp_prev.is_null() && match_at_original_text((*first).cp_prev) {
            replace((*first).cp_prev);
        }
    }
}

/// Append the next character of the shown match to the leader.
pub unsafe fn ins_compl_addfrommatch() {
    unsafe {
        let shown = compl_shown_match.get();
        let len = (*curwin.get()).w_cursor.col - compl_col.get();
        debug_assert!(!shown.is_null());
        let mut p = (*shown).cp_str.data();
        if (*shown).cp_str.len() as c_int <= len {
            // The match is too short. When still at the original match use the
            // first entry that matches the leader.
            if !match_at_original_text(shown) {
                return;
            }
            p = ptr::null_mut();
            let mut plen: size_t = 0;
            let mut cp = (*shown).cp_next;
            while !cp.is_null() && !is_first_match(cp) {
                let leader = *compl_leader.ptr();
                if leader.data().is_null() || ins_compl_equal(cp, leader.data(), leader.len()) {
                    p = (*cp).cp_str.data();
                    plen = (*cp).cp_str.len();
                    break;
                }
                cp = (*cp).cp_next;
            }
            if p.is_null() || plen as c_int <= len {
                return;
            }
        }
        ins_compl_addleader(utf_ptr2char(p.offset(len as isize)));
    }
}

/// Stop insert completion mode.
pub(crate) unsafe fn ins_compl_stop(c: c_int, prev_mode: c_int, mut retval: bool) -> bool {
    unsafe {
        // Remove pre-inserted text when present.
        if ins_compl_preinsert_effect() && ins_compl_win_active(curwin.get()) {
            ins_compl_delete(false);
        }

        // Get here when we have finished typing a sequence of ^N and ^P or
        // other completion characters in CTRL-X mode.  Free up memory that was
        // used, and make sure we can redo the insert.
        if !compl_curr_match.get().is_null()
            || !(*compl_leader.ptr()).data().is_null()
            || c == Ctrl_E
        {
            // If any of the original typed text has been changed, e.g. when
            // 'ignorecase' is set, we must add back-spaces to the redo buffer.
            // We add as few as necessary to delete just the part of the
            // original text that has changed. When using the longest match,
            // when the match was edited or when CTRL-E was used, don't use the
            // current match.
            let mut ptr: *mut c_char = ptr::null_mut();
            if !compl_curr_match.get().is_null() && compl_used_match.get() && c != Ctrl_E {
                ptr = (*compl_curr_match.get()).cp_str.data();
            }
            ins_compl_fixRedoBufForLeader(ptr);
        }

        let mut want_cindent = get_can_cindent() && cindent_on();

        // When completing whole lines: fix indent for 'cindent'.
        // Otherwise, break the line if it's too long.
        if compl_cont_mode.get() == CTRL_X_WHOLE_LINE {
            // Re-indent the current line.
            if want_cindent {
                do_c_expr_indent();
                want_cindent = false; // don't do it again
            }
        } else if !compl_autocomplete.get() || compl_used_match.get() {
            let prev_col = (*curwin.get()).w_cursor.col;

            // Put the cursor on the last char, for 'tw' formatting.
            if prev_col > 0 {
                dec_cursor();
            }
            // Only format when something was inserted.
            if !arrow_used.get() && !ins_need_undo_get() && c != Ctrl_E {
                insertchar(NUL, 0, -1);
            }
            if prev_col > 0
                && *get_cursor_line_ptr().offset((*curwin.get()).w_cursor.col as isize) as c_int
                    != NUL
            {
                inc_cursor();
            }
        }

        // If the popup menu is displayed, pressing CTRL-Y means accepting the
        // selection without inserting anything.  When compl_enter_selects is
        // set the Enter key does the same.
        let mut word: *mut c_char = ptr::null_mut();
        if (c == Ctrl_Y || (compl_enter_selects.get() && (c == CAR || c == K_KENTER || c == NL)))
            && pum_visible()
        {
            word = xstrdup((*compl_shown_match.get()).cp_str.data());
            retval = true;
            // May need to remove ComplMatchIns highlight.
            redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
        }

        // When a match was inserted but the pum was never displayed (e.g. only
        // one match with 'completeopt' "menu" without "menuone"), the user had
        // no opportunity to explicitly accept or dismiss it, so treat this as
        // an implicit accept (#38160).
        if word.is_null()
            && c != Ctrl_E
            && compl_used_match.get()
            && (*compl_match_array.ptr()).is_null()
            && !compl_curr_match.get().is_null()
            && !(*compl_curr_match.get()).cp_str.data().is_null()
        {
            word = xstrdup((*compl_curr_match.get()).cp_str.data());
        }

        // CTRL-E means completion is Ended: go back to the typed text, but
        // only if the popup is still visible.
        if c == Ctrl_E {
            ins_compl_delete(false);
            let text = if !(*compl_leader.ptr()).data().is_null() {
                *compl_leader.ptr()
            } else if !compl_first_match.get().is_null() {
                *compl_orig_text.ptr()
            } else {
                String_0::NULL
            };
            if !text.data().is_null() {
                let compl_len = get_compl_len();
                if text.len() as c_int > compl_len {
                    ins_compl_insert_bytes(
                        text.data().offset(compl_len as isize),
                        text.len() as c_int - compl_len,
                    );
                }
            }
            restore_orig_extmarks();
            retval = true;
        }

        auto_format(false, true);

        // Trigger the CompleteDonePre event to give scripts a chance to act
        // upon the completion before clearing the info, and restore
        // ctrl_x_mode so that complete_info() can be used.
        ctrl_x_mode.set(prev_mode);
        ins_apply_autocmds(EVENT_COMPLETEDONEPRE);

        ins_compl_free();
        compl_started.set(false);
        compl_matches.set(0);
        if !shortmess(ShmFlag::COMPLETIONMENU) {
            msg_clr_cmdline(); // necessary for "noshowmode"
        }
        ctrl_x_mode.set(CTRL_X_NORMAL);
        compl_enter_selects.set(false);
        if !edit_submode.get().is_null() {
            edit_submode.set(ptr::null_mut());
            redraw_mode.set(true);
        }
        compl_autocomplete.set(false);
        compl_from_nonkeyword.set(false);
        compl_num_bests.set(0);
        compl_ins_end_col.set(0);

        if c == Ctrl_C && cmdwin_type.get() != 0 {
            // Avoid the popup menu remaining displayed when leaving the
            // command line window.
            update_screen();
        }

        // Indent now if a key was typed that is in 'cinkeys'.
        if want_cindent && in_cinkeys(KEY_COMPLETE, ' ' as c_int, inindent(0)) {
            do_c_expr_indent();
        }
        // Trigger the CompleteDone event to give scripts a chance to act upon
        // the end of completion.
        do_autocmd_completedone(c, prev_mode, word);
        xfree(word.cast::<c_void>());

        retval
    }
}

/// Cancel completion.
pub unsafe fn ins_compl_cancel() -> bool {
    unsafe { ins_compl_stop(' ' as c_int, ctrl_x_mode.get(), true) }
}

/// Prepare for Insert mode completion, or stop it; called just after typing a
/// character in Insert mode.
///
/// Answers true when `c` is not to be inserted.
pub unsafe fn ins_compl_prep(c: c_int) -> bool {
    unsafe {
        let mut retval = false;
        let prev_mode = ctrl_x_mode.get();

        // Forget any previous 'special' messages if this is actually a ^X mode
        // key — bar ^R, in which case we wait to see what it gives us.
        if c != Ctrl_R && vim_is_ctrl_x_key(c) {
            edit_submode_extra.set(ptr::null_mut());
        }

        // Ignore end of Select mode mapping and mouse scroll/movement.
        if matches!(
            c,
            K_SELECT
                | K_MOUSEDOWN
                | K_MOUSEUP
                | K_MOUSELEFT
                | K_MOUSERIGHT
                | K_MOUSEMOVE
                | K_EVENT
                | K_COMMAND
                | K_LUA
        ) {
            return retval;
        }

        if ctrl_x_mode.get() == CTRL_X_CMDLINE_CTRL_X && c != Ctrl_X {
            if c == Ctrl_V
                || c == Ctrl_Q
                || c == Ctrl_Z
                || ins_compl_pum_key(c)
                || !vim_is_ctrl_x_key(c)
            {
                // Not starting another completion mode.
                ctrl_x_mode.set(CTRL_X_CMDLINE);
                // CTRL-X CTRL-Z should stop completion without inserting anything.
                if c == Ctrl_Z {
                    retval = true;
                }
            } else {
                ctrl_x_mode.set(CTRL_X_CMDLINE);
                // Other CTRL-X keys first stop completion, then start another
                // completion mode.
                ins_compl_prep(' ' as c_int);
                ctrl_x_mode.set(CTRL_X_NOT_DEFINED_YET);
            }
        }

        // Set "compl_get_longest" when finding the first matches.
        if ctrl_x_mode_not_defined_yet() || (ctrl_x_mode_normal() && !compl_started.get()) {
            compl_get_longest.set(get_cot_flags() & kOptCotFlagLongest as c_uint != 0);
            compl_used_match.set(true);
        }

        if ctrl_x_mode_not_defined_yet() {
            // We have just typed CTRL-X and aren't quite sure which CTRL-X
            // mode it will be yet.  Now we decide.
            retval = set_ctrl_x_mode(c);
        } else if ctrl_x_mode_not_default() {
            // We're already in CTRL-X mode, do we stay in it?
            if !vim_is_ctrl_x_key(c) {
                ctrl_x_mode.set(if ctrl_x_mode_scroll() {
                    CTRL_X_NORMAL
                } else {
                    CTRL_X_FINISHED
                });
                edit_submode.set(ptr::null_mut());
            }
            redraw_mode.set(true);
        }

        if compl_started.get() || ctrl_x_mode.get() == CTRL_X_FINISHED {
            // Show the error message from an attempted keyword completion
            // (probably 'Pattern not found') until another key is hit, then go
            // back to showing what mode we are in.
            redraw_mode.set(true);
            if (ctrl_x_mode_normal()
                && c != Ctrl_N
                && c != Ctrl_P
                && c != Ctrl_R
                && !ins_compl_pum_key(c))
                || ctrl_x_mode.get() == CTRL_X_FINISHED
            {
                retval = ins_compl_stop(c, prev_mode, retval);
            }
        } else if ctrl_x_mode.get() == CTRL_X_LOCAL_MSG {
            // Trigger the CompleteDone event to give scripts a chance to act
            // upon the (possibly failed) completion.
            do_autocmd_completedone(c, ctrl_x_mode.get(), ptr::null_mut());
        }

        may_trigger_modechanged();

        // Reset continue_* if we left expansion mode; if we stay they'll be
        // (re)set properly in ins_complete().
        if !vim_is_ctrl_x_key(c) {
            compl_cont_status.set(0);
            compl_cont_mode.set(0);
        }

        retval
    }
}

/// Fix the redo buffer for the completion leader replacing some of the typed
/// text: insert backspaces and append the changed text.
///
/// `ptr_arg` is the known leader text, or null to use `compl_leader`.
pub(crate) unsafe fn ins_compl_fixRedoBufForLeader(ptr_arg: *mut c_char) {
    unsafe {
        let mut len = 0;
        let mut ptr = ptr_arg;
        if ptr.is_null() {
            if (*compl_leader.ptr()).data().is_null() {
                return; // nothing to do
            }
            ptr = (*compl_leader.ptr()).data();
        }
        if !(*compl_orig_text.ptr()).data().is_null() {
            let mut p = (*compl_orig_text.ptr()).data();
            // Length of the common prefix between the original text and the
            // new completion.
            while *p.offset(len as isize) as c_int != NUL
                && *p.offset(len as isize) == *ptr.offset(len as isize)
            {
                len += 1;
            }
            // Don't break inside a multi-byte character.
            if len > 0 {
                len -= utf_head_off(p, p.offset(len as isize));
            }
            // A backspace for each remaining character of the original text.
            p = p.offset(len as isize);
            while *p as c_int != NUL {
                AppendCharToRedobuff(K_BS);
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        }
        AppendToRedobuffLit(ptr.offset(len as isize), -1);
    }
}

/// While collecting matches, watch for a key that should change what is shown
/// or leave completion; and show a completion as soon as possible while
/// `compl_pending` is non-zero.
///
/// `frequency` says out of how many calls we actually check. `in_compl_func`
/// is true when called from `complete_check()`, where `compl_curr_match` must
/// not be set.
pub unsafe fn ins_compl_check_keys(frequency: c_int, in_compl_func: bool) {
    unsafe {
        static count: GlobalCell<c_int> = GlobalCell::new(0);

        // Don't check when reading keys from a script, :normal or feedkeys().
        // That would break the test scripts.  But do check for keys when
        // called from complete_check().
        if !in_compl_func && (using_script() != 0 || ex_normal_busy.get() != 0) {
            return;
        }

        // Only do this at regular intervals.
        (*count.ptr()) += 1;
        if count.get() < frequency {
            return;
        }
        count.set(0);

        // Check for a typed key.  Do use mappings, otherwise
        // vim_is_ctrl_x_key() can't do its work correctly.
        let mut c = vpeekc_any();
        if c != NUL && !test_disable_char_avail.get() {
            if vim_is_ctrl_x_key(c) && c != Ctrl_X && c != Ctrl_R {
                c = safe_vgetc(); // Eat the character
                compl_shows_dir.set(ins_compl_key2dir(c));
                ins_compl_next(false, ins_compl_key2count(c), c != K_UP && c != K_DOWN);
            } else {
                // Need to get the character to have KeyTyped set.  We'll put it
                // back with vungetc() below.  But skip K_IGNORE.
                c = safe_vgetc();
                if c != K_IGNORE {
                    // Don't interrupt completion when the character wasn't
                    // typed, e.g. when doing @q to replay keys.
                    if c != Ctrl_R && KeyTyped.get() {
                        compl_interrupted.set(true);
                    }
                    vungetc(c);
                }
            }
        } else {
            let normal_mode_strict = ctrl_x_mode_normal()
                && !ctrl_x_mode_line_or_eval()
                && compl_cont_status.get() & CONT_LOCAL == 0
                && !(*cpt_sources_array.ptr()).is_null()
                && cpt_sources_index.get() >= 0;
            if normal_mode_strict && (compl_autocomplete.get() || p_cto.get() > 0) {
                check_elapsed_time();
            }
        }

        if compl_pending.get() != 0
            && !got_int.get()
            && cot_flags.get() & (kOptCotFlagNoinsert as c_uint | kOptCotFlagFuzzy as c_uint) == 0
            && (!compl_autocomplete.get() || ins_compl_has_preinsert())
        {
            // Insert the first match immediately and advance
            // compl_shown_match, before finding other matches.
            let todo = compl_pending.get().abs();
            compl_pending.set(0);
            ins_compl_next(false, todo, true);
        }
    }
}
