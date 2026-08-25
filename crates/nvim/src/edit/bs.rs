//! Backspace and delete: `i_BS`, `i_CTRL-W`, `i_CTRL-U` and `i_DEL`.
//!
//! [`ins_bs`] is one function for all three backwards forms, told apart by
//! its [`Backspace`] argument.  It is a guard, a setup, one of three deleting
//! phases, and a tail:
//!
//! | phase | when |
//! | --- | --- |
//! | [`bs_blocked`] | the whole set of things backspace may *not* delete |
//! | [`bs_join_line`] | the cursor is in column 0, so the line break goes |
//! | [`bs_one_shiftwidth`] | 'softtabstop'/'smarttab' say one BS eats a whole indent step |
//! | [`bs_delete_chars`] | everything else: delete backwards until a stopping rule fires |
//!
//! The guard is most of what makes the key complicated: 'backspace' decides
//! whether it may cross the start of the insert (`BsFlag::START`), an auto-indent
//! (`BsFlag::INDENT`) or a line break (`BsFlag::EOL`), a prompt buffer's prompt is off
//! limits, and 'revins' turns all of it around.  Replace and Virtual Replace
//! mode never delete at all -- they restore from the replace stack.
//!
//! [`ins_del`] is `<Del>`, the forward case, and is much shorter because
//! none of those rules apply to it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::memline::MlFlags;
use core::ffi::c_int;

use super::*;
use crate::option::cpo_has;
use crate::types::{BsFlag, CpoFlag, FAIL, FoFlag, NUL};

/// Which backwards-delete key is running.
///
/// The word forms change from [`Backspace::Word`] to
/// [`Backspace::WordNotSpace`] *inside* the delete loop, at the first
/// non-blank: that is how "delete the white space, then the word" is
/// spelled.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Backspace {
    /// `i_BS`: one character.
    Char,
    /// `i_CTRL-W`, while still in the white space before the word.
    Word,
    /// `i_CTRL-W`, once the word itself has been reached.
    WordNotSpace,
    /// `i_CTRL-U`: back to the start of the line (or of the indent).
    Line,
}

/// C's `if (curwin->w_cursor.lnum == Insstart_orig.lnum
/// && curwin->w_cursor.col < Insstart_orig.col) Insstart_orig.col =
/// curwin->w_cursor.col;`: a deletion that went back past where the insert
/// started drags that mark along with it.
///
/// # Safety
/// Must run with a live `curwin`.
unsafe fn pull_insstart_orig_to_cursor() {
    // SAFETY: the caller's contract.
    let cursor = unsafe { (*curwin.get()).w_cursor };
    let orig = Insstart_orig.get();
    if cursor.lnum == orig.lnum && cursor.col < orig.col {
        Insstart_orig.set(orig.with_col(cursor.col));
    }
}

/// `<Del>` in Insert mode: delete forwards.
///
/// At the end of a line that means joining the next one, which needs
/// 'backspace' to contain `eol` just as a backspace over a line break does.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn ins_del() {
    unsafe {
        if stop_arrow() == FAIL {
            return;
        }
        if gchar_cursor() == NUL {
            // Delete the newline.
            let temp = (*curwin.get()).w_cursor.col;
            if !can_bs(BsFlag::EOL) || do_join(2, false, true, false, false) == FAIL {
                vim_beep(kOptBoFlagBackspace as ::core::ffi::c_uint);
            } else {
                (*curwin.get()).w_cursor.col = temp;
                // Adjust `orig_line_count` when more lines were deleted than
                // added, so a later `open_line` can still reach every line.
                if State.get() & VREPLACE_FLAG != 0
                    && orig_line_count.get() > (*curbuf.get()).b_ml.ml_line_count
                {
                    orig_line_count.set((*curbuf.get()).b_ml.ml_line_count);
                }
            }
        } else if del_char(false) == FAIL {
            // Delete the character under the cursor.
            vim_beep(kOptBoFlagBackspace as ::core::ffi::c_uint);
        }
        did_ai.set(false);
        did_si.set(false);
        can_si.set(false);
        can_si_back.set(false);
        append_to_redobuff_char(K_DEL);
    }
}

/// Everything a backspace is not allowed to delete.
///
/// Nothing at all in an empty file; never past the first character in the
/// buffer; not past the start of the insert unless 'backspace' has `start`
/// (with a prompt buffer excepted, because the prompt is protected
/// separately); not into an auto-indent without `indent`; and not over a
/// line break without `eol`.  All of it is off in 'revins'.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
unsafe fn bs_blocked() -> bool {
    unsafe {
        if buf_is_empty(curbuf.get()) {
            return true;
        }
        if revins_on.get() {
            return false;
        }
        let cursor = (*curwin.get()).w_cursor;
        let start = Insstart_orig.get();
        (cursor.lnum == 1 && cursor.col == 0)
            || (!can_bs(BsFlag::START)
                && ((arrow_used.get() && !bt_prompt(curbuf.get()))
                    || (cursor.lnum == start.lnum && cursor.col <= start.col)))
            || (!can_bs(BsFlag::INDENT)
                && !arrow_used.get()
                && ai_col.get() > 0
                && cursor.col <= ai_col.get())
            || (!can_bs(BsFlag::EOL) && cursor.col == 0)
    }
}

/// Handle Backspace, delete-word and delete-line in Insert mode.
///
/// `c` is the character that was typed (it goes into the redo buffer),
/// `mode` says which of the three keys it is, and `inserted_space_p` is the
/// caller's "the last thing inserted was a space" flag, which
/// [`bs_one_shiftwidth`] both reads and clears.
///
/// Answers whether a backspace actually happened.
///
/// # Safety
/// `inserted_space_p` must point to a live `c_int`.
pub(crate) unsafe fn ins_bs(c: c_int, mode: Backspace, inserted_space_p: *mut c_int) -> bool {
    unsafe {
        if bs_blocked() {
            vim_beep(kOptBoFlagBackspace as ::core::ffi::c_uint);
            return false;
        }
        if stop_arrow() == FAIL {
            return false;
        }

        let in_indent = inindent(0);
        if in_indent {
            can_cindent.set(false);
        }
        end_comment_pending.set(NUL); // after BS, don't auto-end a comment
        if revins_on.get() {
            inc_cursor(); // put the cursor after the last inserted character
        }

        // In 'virtualedit': BACKSPACE_CHAR eats one virtual space,
        // BACKSPACE_WORD eats all the `coladd`, and BACKSPACE_LINE eats all
        // of it and keeps going.
        if (*curwin.get()).w_cursor.coladd > 0 {
            if mode == Backspace::Char {
                (*curwin.get()).w_cursor.coladd -= 1;
                return true;
            }
            if mode == Backspace::Word {
                (*curwin.get()).w_cursor.coladd = 0;
                return true;
            }
            (*curwin.get()).w_cursor.coladd = 0;
        }

        let mut did_backspace = false;
        let mut call_fix_indent = false;

        if (*curwin.get()).w_cursor.col == 0 {
            if !bs_join_line() {
                return false;
            }
            did_ai.set(false);
        } else {
            if revins_on.get() {
                dec_cursor(); // put the cursor on the last inserted character
            }

            // Keep the indent: CTRL-U stops at the first non-blank if there
            // is one before the cursor.
            let mut mincol: colnr_T = 0;
            if mode == Backspace::Line
                && ((*curbuf.get()).b_p_ai != 0 || cindent_on())
                && !revins_on.get()
            {
                let save_col = (*curwin.get()).w_cursor.col;
                beginline(BeginlineOpts::WHITE);
                if (*curwin.get()).w_cursor.col < save_col {
                    mincol = (*curwin.get()).w_cursor.col;
                    // The indent should now be fixed to match the previous
                    // line.
                    call_fix_indent = true;
                }
                (*curwin.get()).w_cursor.col = save_col;
            }

            // One BS deletes a whole 'shiftwidth' or 'softtabstop' when
            // 'smarttab' says so in the indent, or when the byte before the
            // cursor is white space this insert did not type.
            let one_step = mode == Backspace::Char
                && ((p_sta.get() != 0 && in_indent)
                    || ((get_sts_value() != 0
                        || tabstop_count((*curbuf.get()).b_p_vsts_array) != 0)
                        && (*curwin.get()).w_cursor.col > 0
                        && (*get_cursor_pos_ptr().offset(-1) as c_int == TAB
                            || (*get_cursor_pos_ptr().offset(-1) as c_int == ' ' as c_int
                                && (*inserted_space_p == 0 || arrow_used.get())))));
            if one_step {
                *inserted_space_p = 0;
                bs_one_shiftwidth(in_indent);
            } else {
                bs_delete_chars(mode, mincol);
            }
            did_backspace = true;
        }

        did_si.set(false);
        can_si.set(false);
        can_si_back.set(false);
        if (*curwin.get()).w_cursor.col <= 1 {
            did_ai.set(false);
        }
        if call_fix_indent {
            fix_indent();
        }

        // It is a little strange to put backspaces into the redo buffer, but
        // it makes auto-indent much easier to deal with.
        append_to_redobuff_char(c);

        // If the deletion went before the insertion point, move that too.
        pull_insstart_orig_to_cursor();

        // Vi moves the cursor back but leaves the character on the screen;
        // Vim erases it.  The vi behaviour is emulated by pretending a
        // dollar is displayed even when there is not one.
        //  --pkv Sun Jan 19 01:56:40 EST 2003
        if cpo_has(CpoFlag::BACKSPACE) && dollar_vcol.get() == -1 {
            dollar_vcol.set((*curwin.get()).w_virtcol);
        }

        // After deleting a character the cursor line must never be in a
        // closed fold -- with 'foldmethod' indent, deleting the first
        // non-white character before a TAB can put it in one.
        if did_backspace {
            fold_open_cursor();
        }
        did_backspace
    }
}

/// The cursor is in column 0: delete the line break in front of it.
///
/// Answers false when undo could not be saved, in which case nothing has
/// happened and the caller must give up.
///
/// In Replace mode the line break may have *replaced* characters, which are
/// on the replace stack: first a NUL-terminated run that was deleted after
/// the cursor, then the characters the NL itself replaced.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`, cursor in column 0.
unsafe fn bs_join_line() -> bool {
    unsafe {
        let lnum = Insstart.get().lnum;
        if (*curwin.get()).w_cursor.lnum == lnum || revins_on.get() {
            if u_save(
                (*curwin.get()).w_cursor.lnum - 2,
                (*curwin.get()).w_cursor.lnum + 1,
            ) == FAIL
            {
                return false;
            }
            let lnum = Insstart.get().lnum - 1;
            Insstart.set(Insstart.get().with_lnum(lnum).with_col(ml_get_len(lnum)));
        }

        // In Replace mode: below zero the NL was inserted, so delete it; at
        // or above zero it replaced characters, which go back.
        let mut cc = -1;
        if State.get() & REPLACE_FLAG != 0 {
            cc = replace_pop_if_nul(); // -1 if the NL was inserted
        }

        // In Replace mode, on the line the replacing started on, only the
        // cursor moves.
        if State.get() & REPLACE_FLAG != 0 && (*curwin.get()).w_cursor.lnum <= lnum {
            dec_cursor();
            return true;
        }

        if State.get() & VREPLACE_FLAG == 0 || (*curwin.get()).w_cursor.lnum > orig_line_count.get()
        {
            let temp = gchar_cursor(); // remember the current character
            (*curwin.get()).w_cursor.lnum -= 1;

            // With `aw` in 'formatoptions' the space at the end of the line
            // has to go too, or auto-formatting would break the line again.
            if has_format_option(FoFlag::AUTO) && has_format_option(FoFlag::WHITE_PAR) {
                let ptr = ml_get_buf(curbuf.get(), (*curwin.get()).w_cursor.lnum);
                let len = get_cursor_line_len();
                if len > 0 && *ptr.offset((len - 1) as isize) as c_int == ' ' as c_int {
                    let newp = xmemdupz(ptr as *const ::core::ffi::c_void, (len - 1) as size_t)
                        as *mut ::core::ffi::c_char;
                    if (*curbuf.get()).b_ml.line_is_owned() {
                        xfree((*curbuf.get()).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
                    }
                    (*curbuf.get()).b_ml.ml_line_ptr = newp;
                    (*curbuf.get()).b_ml.ml_line_textlen -= 1;
                    (*curbuf.get()).b_ml.ml_flags |= MlFlags::LINE_DIRTY;
                }
            }

            do_join(2, false, false, false, false);
            if temp == NUL && gchar_cursor() != NUL {
                inc_cursor();
            }
        } else {
            dec_cursor();
        }

        if State.get() & REPLACE_FLAG != 0 {
            // Do the insertions in MODE_NORMAL state, so `ins_char` does not
            // replace characters and does not call `showmatch`.
            let old_state = State.get();
            State.set(MODE_NORMAL);
            // Restore the characters (blanks) that were deleted after the
            // cursor...
            while cc > 0 {
                let save_col = (*curwin.get()).w_cursor.col;
                mb_replace_pop_ins();
                (*curwin.get()).w_cursor.col = save_col;
                cc = replace_pop_if_nul();
            }
            // ... and then the ones the NL replaced.
            replace_pop_ins();
            State.set(old_state);
        }
        true
    }
}

/// Delete back to the previous 'softtabstop' or 'shiftwidth' boundary.
///
/// The white space around the cursor is *rebuilt* rather than trimmed: the
/// walk finds the last run of blanks that is preceded by something else,
/// deletes back to a boundary at or before the wanted virtual column, and
/// then pads forward with spaces.  `charsize_nowrap` is used throughout so
/// that virtual text and wrapping cannot change the answer.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`, cursor past column 0.
unsafe fn bs_one_shiftwidth(in_indent: bool) {
    unsafe {
        let use_ts =
            (*curwin.get()).w_onebuf_opt.wo_list == 0 || (*curwin.get()).w_p_lcs_chars.tab1 != 0;
        let line = get_cursor_line_ptr();
        let cursor_ptr = line.offset((*curwin.get()).w_cursor.col as isize);

        // The cursor's virtual column, and the last white space before it
        // that is preceded by non-white space.
        let mut vcol: colnr_T = 0;
        let mut space_vcol: colnr_T = 0;
        let mut sci: StrCharInfo = utf_ptr2str_char_info(line);
        let mut space_sci = sci;
        let mut prev_space = false;
        while sci.ptr < cursor_ptr {
            let cur_space = ascii_iswhite(sci.chr.value);
            if !prev_space && cur_space {
                space_sci = sci;
                space_vcol = vcol;
            }
            vcol += charsize_nowrap(curbuf.get(), sci.ptr, use_ts, vcol, sci.chr.value);
            sci = utfc_next(sci);
            prev_space = cur_space;
        }

        // The virtual column to end up at.
        let mut want_vcol = if vcol > 0 { vcol - 1 } else { 0 };
        if p_sta.get() != 0 && in_indent {
            want_vcol -= want_vcol % get_sw_value(curbuf.get());
        } else {
            want_vcol = tabstop_start(want_vcol, get_sts_value(), (*curbuf.get()).b_p_vsts_array);
        }

        // Where to stop backspacing.
        loop {
            let size = charsize_nowrap(
                curbuf.get(),
                space_sci.ptr,
                use_ts,
                space_vcol,
                space_sci.chr.value,
            );
            if space_vcol + size > want_vcol {
                break;
            }
            space_vcol += size;
            space_sci = utfc_next(space_sci);
        }
        let want_col = space_sci.ptr.offset_from(line) as colnr_T;

        // Delete until at or before `want_col`.
        while (*curwin.get()).w_cursor.col > want_col {
            dec_cursor();
            if State.get() & REPLACE_FLAG != 0 {
                // Don't delete before the insert point in Replace mode.
                if (*curwin.get()).w_cursor.lnum != Insstart.get().lnum
                    || (*curwin.get()).w_cursor.col >= Insstart.get().col
                {
                    replace_do_bs(-1);
                }
            } else {
                del_char(false);
            }
        }

        // Insert spaces until at `want_vcol`.
        while space_vcol < want_vcol {
            // Remember the first character inserted.
            pull_insstart_orig_to_cursor();

            if State.get() & VREPLACE_FLAG != 0 {
                ins_char(' ' as c_int);
            } else {
                ins_str(c" ".as_ptr().cast_mut(), 1);
                if State.get() & REPLACE_FLAG != 0 {
                    replace_push_nul();
                }
            }
            space_vcol += 1;
        }
    }
}

/// Delete backwards until the starting point, the start of the line, or the
/// previous word.
///
/// `mincol` is where CTRL-U decided the indent begins, and is 0 for the
/// other two keys.  `mode` changes *inside* the loop: CTRL-W eats the white
/// space as [`Backspace::Word`] and then the word itself as
/// [`Backspace::WordNotSpace`], stopping at the first character whose
/// "wordness" or multi-byte class differs from the previous one's.
///
/// # Safety
/// Must run with a live `curwin`, cursor past column 0.
unsafe fn bs_delete_chars(mut mode: Backspace, mincol: colnr_T) {
    unsafe {
        // What kind of word the deletion started in, so a class change can
        // end it.
        let mut cclass = mb_get_class(get_cursor_pos_ptr());
        // Whether the word being deleted is made of 'iskeyword' characters;
        // only read once `mode` is `WordNotSpace`.
        let mut is_word = 0;
        loop {
            if !revins_on.get() {
                dec_cursor(); // put the cursor on the character to delete
            }
            let cc = gchar_cursor();
            let prev_cclass = cclass;
            cclass = mb_get_class(get_cursor_pos_ptr());

            if mode == Backspace::Word && !ascii_isspace(cc) {
                // The start of the word.
                mode = Backspace::WordNotSpace;
                is_word = vim_iswordc(cc) as c_int;
            } else if mode == Backspace::WordNotSpace
                && (ascii_isspace(cc)
                    || vim_iswordc(cc) as c_int != is_word
                    || prev_cclass != cclass)
            {
                // The end of the word.
                if !revins_on.get() {
                    inc_cursor();
                } else if State.get() & REPLACE_FLAG != 0 {
                    dec_cursor();
                }
                break;
            }

            if State.get() & REPLACE_FLAG != 0 {
                replace_do_bs(-1);
            } else {
                let mut has_composing = false;
                if p_deco.get() != 0 {
                    let p0 = get_cursor_pos_ptr();
                    has_composing = utf_composinglike(
                        p0,
                        p0.offset(utf_ptr2len(p0) as isize),
                        ::core::ptr::null_mut(),
                    );
                }
                del_char(false);
                // With combining characters and 'delcombine' set, move the
                // cursor back -- but never before the base character.
                if has_composing {
                    inc_cursor();
                }
                if revins_chars.get() != 0 {
                    revins_chars.set(revins_chars.get() - 1);
                    revins_legal.set(revins_legal.get() + 1);
                }
                if revins_on.get() && gchar_cursor() == NUL {
                    break;
                }
            }

            // Just a single backspace?
            if mode == Backspace::Char {
                break;
            }
            // The `do`-`while` condition: keep going while there is
            // something left this key is allowed to take.
            let more = revins_on.get()
                || ((*curwin.get()).w_cursor.col > mincol
                    && (can_bs(BsFlag::NOSTOP)
                        || ((*curwin.get()).w_cursor.lnum != Insstart_orig.get().lnum
                            || (*curwin.get()).w_cursor.col != Insstart_orig.get().col)));
            if !more {
                break;
            }
        }
    }
}
