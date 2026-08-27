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
use crate::winlayer::{Buf, Win};
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
fn pull_insstart_orig_to_cursor() {
    // SAFETY: the caller's contract.
    let cursor = cur_win().w_cursor;
    let orig = Insstart_orig.get();
    if cursor.lnum == orig.lnum && cursor.col < orig.col {
        Insstart_orig.set(orig.with_col(cursor.col));
    }
}

/// `<Del>` in Insert mode: delete forwards.
///
/// At the end of a line that means joining the next one, which needs
/// 'backspace' to contain `eol` just as a backspace over a line break does.
pub(crate) fn ins_del() {
    // SAFETY: every `unsafe` call in this function is an editor-wide routine
    // whose only precondition is the live `curwin`/`curbuf` Insert mode runs
    // with.
    if unsafe { stop_arrow() } == FAIL {
        return;
    }
    if char_at_cursor() == NUL {
        // Delete the newline.
        let temp = cur_win().w_cursor.col;
        if !can_bs(BsFlag::EOL) || unsafe { do_join(2, false, true, false, false) } == FAIL {
            beep_backspace();
        } else {
            cur_win().w_cursor.col = temp;
            // Adjust `orig_line_count` when more lines were deleted than
            // added, so a later `open_line` can still reach every line.
            if State.get() & VREPLACE_FLAG != 0
                && orig_line_count.get() > cur_buf().b_ml.ml_line_count
            {
                orig_line_count.set(cur_buf().b_ml.ml_line_count);
            }
        }
    } else if delete_one_char() == FAIL {
        // Delete the character under the cursor.
        beep_backspace();
    }
    did_ai.set(false);
    did_si.set(false);
    can_si.set(false);
    can_si_back.set(false);
    append_to_redobuff_char(K_DEL);
}

/// Everything a backspace is not allowed to delete.
///
/// Nothing at all in an empty file; never past the first character in the
/// buffer; not past the start of the insert unless 'backspace' has `start`
/// (with a prompt buffer excepted, because the prompt is protected
/// separately); not into an auto-indent without `indent`; and not over a
/// line break without `eol`.  All of it is off in 'revins'.
fn bs_blocked() -> bool {
    // SAFETY: `curbuf` is live for the whole session.
    if unsafe { buf_is_empty(curbuf.get()) } {
        return true;
    }
    if revins_on.get() {
        return false;
    }
    let cursor = cur_win().w_cursor;
    let start = Insstart_orig.get();
    (cursor.lnum == 1 && cursor.col == 0)
        || (!can_bs(BsFlag::START)
            && ((arrow_used.get() && !unsafe { bt_prompt(curbuf.get()) })
                || (cursor.lnum == start.lnum && cursor.col <= start.col)))
        || (!can_bs(BsFlag::INDENT)
            && !arrow_used.get()
            && ai_col.get() > 0
            && cursor.col <= ai_col.get())
        || (!can_bs(BsFlag::EOL) && cursor.col == 0)
}

/// Handle Backspace, delete-word and delete-line in Insert mode.
///
/// `c` is the character that was typed (it goes into the redo buffer),
/// `mode` says which of the three keys it is, and `inserted_space_p` is the
/// caller's "the last thing inserted was a space" flag, which
/// [`bs_one_shiftwidth`] both reads and clears.
///
/// Answers whether a backspace actually happened.
pub(crate) fn ins_bs(c: c_int, mode: Backspace, inserted_space_p: &mut c_int) -> bool {
    if bs_blocked() {
        beep_backspace();
        return false;
    }
    // SAFETY: every `unsafe` call in this function is an editor-wide routine
    // whose only precondition is the live `curwin`/`curbuf` Insert mode runs
    // with.
    if unsafe { stop_arrow() } == FAIL {
        return false;
    }

    let in_indent = unsafe { inindent(0) };
    if in_indent {
        can_cindent.set(false);
    }
    end_comment_pending.set(NUL); // after BS, don't auto-end a comment
    if revins_on.get() {
        cursor_forward(); // put the cursor after the last inserted character
    }

    // In 'virtualedit': BACKSPACE_CHAR eats one virtual space,
    // BACKSPACE_WORD eats all the `coladd`, and BACKSPACE_LINE eats all
    // of it and keeps going.
    if cur_win().w_cursor.coladd > 0 {
        if mode == Backspace::Char {
            cur_win().w_cursor.coladd -= 1;
            return true;
        }
        if mode == Backspace::Word {
            cur_win().w_cursor.coladd = 0;
            return true;
        }
        cur_win().w_cursor.coladd = 0;
    }

    let mut did_backspace = false;
    let mut call_fix_indent = false;

    if cur_win().w_cursor.col == 0 {
        if !bs_join_line() {
            return false;
        }
        did_ai.set(false);
    } else {
        if revins_on.get() {
            cursor_back(); // put the cursor on the last inserted character
        }

        // Keep the indent: CTRL-U stops at the first non-blank if there
        // is one before the cursor.
        let mut mincol: colnr_T = 0;
        if mode == Backspace::Line
            && (cur_buf().b_p_ai != 0 || unsafe { cindent_on() })
            && !revins_on.get()
        {
            let save_col = cur_win().w_cursor.col;
            beginline(BeginlineOpts::WHITE);
            if cur_win().w_cursor.col < save_col {
                mincol = cur_win().w_cursor.col;
                // The indent should now be fixed to match the previous
                // line.
                call_fix_indent = true;
            }
            cur_win().w_cursor.col = save_col;
        }

        // One BS deletes a whole 'shiftwidth' or 'softtabstop' when
        // 'smarttab' says so in the indent, or when the byte before the
        // cursor is white space this insert did not type.
        // A closure, so that the byte before the cursor is only read once
        // 'smarttab' has had its say and the column has been checked.
        let soft_tab = || {
            (unsafe { get_sts_value() } != 0
                || unsafe { tabstop_count(cur_buf().b_p_vsts_array) } != 0)
                && cur_win().w_cursor.col > 0
                && {
                    // SAFETY: the cursor is past column 0, so the byte before
                    // it is a byte of the cursor's own line.
                    let before = unsafe { *get_cursor_pos_ptr().offset(-1) } as c_int;
                    before == TAB
                        || (before == ' ' as c_int && (*inserted_space_p == 0 || arrow_used.get()))
                }
        };
        let one_step = mode == Backspace::Char && ((p_sta.get() != 0 && in_indent) || soft_tab());
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
    if cur_win().w_cursor.col <= 1 {
        did_ai.set(false);
    }
    if call_fix_indent {
        unsafe { fix_indent() };
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
        dollar_vcol.set(cur_win().w_virtcol);
    }

    // After deleting a character the cursor line must never be in a
    // closed fold -- with 'foldmethod' indent, deleting the first
    // non-white character before a TAB can put it in one.
    if did_backspace {
        unsafe { fold_open_cursor() };
    }
    did_backspace
}

/// The cursor is in column 0: delete the line break in front of it.
///
/// Answers false when undo could not be saved, in which case nothing has
/// happened and the caller must give up.
///
/// In Replace mode the line break may have *replaced* characters, which are
/// on the replace stack: first a NUL-terminated run that was deleted after
/// the cursor, then the characters the NL itself replaced.
fn bs_join_line() -> bool {
    let lnum = Insstart.get().lnum;
    if cur_win().w_cursor.lnum == lnum || revins_on.get() {
        // SAFETY: every `unsafe` call in this function is an editor-wide
        // routine whose only precondition is the live `curwin`/`curbuf`
        // Insert mode runs with.
        if u_save(cur_win().w_cursor.lnum - 2, cur_win().w_cursor.lnum + 1) == FAIL {
            return false;
        }
        let lnum = Insstart.get().lnum - 1;
        let len = ml_get_len(lnum);
        Insstart.set(Insstart.get().with_lnum(lnum).with_col(len));
    }

    // In Replace mode: below zero the NL was inserted, so delete it; at
    // or above zero it replaced characters, which go back.
    let mut cc = -1;
    if State.get() & REPLACE_FLAG != 0 {
        cc = replace_pop_if_nul(); // -1 if the NL was inserted
    }

    // In Replace mode, on the line the replacing started on, only the
    // cursor moves.
    if State.get() & REPLACE_FLAG != 0 && cur_win().w_cursor.lnum <= lnum {
        cursor_back();
        return true;
    }

    if State.get() & VREPLACE_FLAG == 0 || cur_win().w_cursor.lnum > orig_line_count.get() {
        let temp = char_at_cursor(); // remember the current character
        cur_win().w_cursor.lnum -= 1;

        // With `aw` in 'formatoptions' the space at the end of the line
        // has to go too, or auto-formatting would break the line again.
        if has_format_option(FoFlag::AUTO) && has_format_option(FoFlag::WHITE_PAR) {
            let ptr = unsafe { ml_get_buf(curbuf.get(), cur_win().w_cursor.lnum) };
            let len = get_cursor_line_len();
            // SAFETY: `ptr` is that line and `len` its length, so its last
            // byte is in bounds, and `xmemdupz` copies that many bytes.
            if len > 0 && unsafe { *ptr.offset((len - 1) as isize) } as c_int == ' ' as c_int {
                let size = (len - 1) as size_t;
                let newp = unsafe { xmemdupz(ptr.cast(), size) } as *mut ::core::ffi::c_char;
                if cur_buf().b_ml.line_is_owned() {
                    unsafe { xfree(cur_buf().b_ml.ml_line_ptr.cast()) };
                }
                cur_buf().b_ml.ml_line_ptr = newp;
                cur_buf().b_ml.ml_line_textlen -= 1;
                cur_buf().b_ml.ml_flags |= MlFlags::LINE_DIRTY;
            }
        }

        unsafe { do_join(2, false, false, false, false) };
        if temp == NUL && char_at_cursor() != NUL {
            cursor_forward();
        }
    } else {
        cursor_back();
    }

    if State.get() & REPLACE_FLAG != 0 {
        // Do the insertions in MODE_NORMAL state, so `ins_char` does not
        // replace characters and does not call `showmatch`.
        let old_state = State.get();
        State.set(MODE_NORMAL);
        // Restore the characters (blanks) that were deleted after the
        // cursor...
        while cc > 0 {
            let save_col = cur_win().w_cursor.col;
            mb_replace_pop_ins();
            cur_win().w_cursor.col = save_col;
            cc = replace_pop_if_nul();
        }
        // ... and then the ones the NL replaced.
        replace_pop_ins();
        State.set(old_state);
    }
    true
}

/// Delete back to the previous 'softtabstop' or 'shiftwidth' boundary.
///
/// The white space around the cursor is *rebuilt* rather than trimmed: the
/// walk finds the last run of blanks that is preceded by something else,
/// deletes back to a boundary at or before the wanted virtual column, and
/// then pads forward with spaces.  `charsize_nowrap` is used throughout so
/// that virtual text and wrapping cannot change the answer.
fn bs_one_shiftwidth(in_indent: bool) {
    let use_ts = cur_win().w_onebuf_opt.wo_list == 0 || cur_win().w_p_lcs_chars.tab1 != 0;
    // SAFETY: the cursor's column is a byte of the cursor's line, so `line`
    // and `cursor_ptr` address that line and the walk below stays inside it.
    let line = get_cursor_line_ptr();
    let cursor_ptr = unsafe { line.offset(cur_win().w_cursor.col as isize) };

    // The cursor's virtual column, and the last white space before it
    // that is preceded by non-white space.
    let mut vcol: colnr_T = 0;
    let mut space_vcol: colnr_T = 0;
    let mut sci: StrCharInfo = unsafe { utf_ptr2str_char_info(line) };
    let mut space_sci = sci;
    let mut prev_space = false;
    while sci.ptr < cursor_ptr {
        let cur_space = ascii_iswhite(sci.chr.value);
        if !prev_space && cur_space {
            space_sci = sci;
            space_vcol = vcol;
        }
        vcol += charsize_at(use_ts, vcol, sci);
        sci = unsafe { utfc_next(sci) };
        prev_space = cur_space;
    }

    // The virtual column to end up at.
    let mut want_vcol = if vcol > 0 { vcol - 1 } else { 0 };
    if p_sta.get() != 0 && in_indent {
        want_vcol -= want_vcol % unsafe { get_sw_value(curbuf.get()) };
    } else {
        let sts = unsafe { get_sts_value() };
        want_vcol = unsafe { tabstop_start(want_vcol, sts, cur_buf().b_p_vsts_array) };
    }

    // Where to stop backspacing.
    loop {
        let size = charsize_at(use_ts, space_vcol, space_sci);
        if space_vcol + size > want_vcol {
            break;
        }
        space_vcol += size;
        space_sci = unsafe { utfc_next(space_sci) };
    }
    // SAFETY: the walk never stepped past `cursor_ptr`, so `space_sci` is
    // still inside the same line as `line`.
    let want_col = unsafe { space_sci.ptr.offset_from(line) } as colnr_T;

    // Delete until at or before `want_col`.
    while cur_win().w_cursor.col > want_col {
        cursor_back();
        if State.get() & REPLACE_FLAG != 0 {
            // Don't delete before the insert point in Replace mode.
            if cur_win().w_cursor.lnum != Insstart.get().lnum
                || cur_win().w_cursor.col >= Insstart.get().col
            {
                replace_do_bs(-1);
            }
        } else {
            delete_one_char();
        }
    }

    // Insert spaces until at `want_vcol`.
    while space_vcol < want_vcol {
        // Remember the first character inserted.
        pull_insstart_orig_to_cursor();

        if State.get() & VREPLACE_FLAG != 0 {
            unsafe { ins_char(' ' as c_int) };
        } else {
            unsafe { ins_str(c" ".as_ptr().cast_mut(), 1) };
            if State.get() & REPLACE_FLAG != 0 {
                unsafe { replace_push_nul() };
            }
        }
        space_vcol += 1;
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
fn bs_delete_chars(mut mode: Backspace, mincol: colnr_T) {
    // What kind of word the deletion started in, so a class change can
    // end it.
    let mut cclass = cursor_char_class();
    // Whether the word being deleted is made of 'iskeyword' characters;
    // only read once `mode` is `WordNotSpace`.
    let mut is_word = 0;
    loop {
        if !revins_on.get() {
            cursor_back(); // put the cursor on the character to delete
        }
        let cc = char_at_cursor();
        let prev_cclass = cclass;
        cclass = cursor_char_class();

        if mode == Backspace::Word && !ascii_isspace(cc) {
            // The start of the word.
            mode = Backspace::WordNotSpace;
            is_word = unsafe { vim_iswordc(cc) } as c_int;
        } else if mode == Backspace::WordNotSpace
            && (ascii_isspace(cc)
                || unsafe { vim_iswordc(cc) } as c_int != is_word
                || prev_cclass != cclass)
        {
            // The end of the word.
            if !revins_on.get() {
                cursor_forward();
            } else if State.get() & REPLACE_FLAG != 0 {
                cursor_back();
            }
            break;
        }

        if State.get() & REPLACE_FLAG != 0 {
            replace_do_bs(-1);
        } else {
            let mut has_composing = false;
            if p_deco.get() != 0 {
                // SAFETY: the cursor is on a character of its line, so the
                // character after it is at most the line's NUL.
                let p0 = get_cursor_pos_ptr();
                let next = unsafe { p0.offset(utf_ptr2len(p0) as isize) };
                has_composing = unsafe { utf_composinglike(p0, next, ::core::ptr::null_mut()) };
            }
            delete_one_char();
            // With combining characters and 'delcombine' set, move the
            // cursor back -- but never before the base character.
            if has_composing {
                cursor_forward();
            }
            if revins_chars.get() != 0 {
                revins_chars.set(revins_chars.get() - 1);
                revins_legal.set(revins_legal.get() + 1);
            }
            if revins_on.get() && char_at_cursor() == NUL {
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
            || (cur_win().w_cursor.col > mincol
                && (can_bs(BsFlag::NOSTOP)
                    || (cur_win().w_cursor.lnum != Insstart_orig.get().lnum
                        || cur_win().w_cursor.col != Insstart_orig.get().col)));
        if !more {
            break;
        }
    }
}

/// Beep, or flash, for a backspace that could not delete anything.
#[inline(always)]
fn beep_backspace() {
    // SAFETY: the bell only reads options.
    unsafe { vim_beep(kOptBoFlagBackspace as ::core::ffi::c_uint) }
}

/// Step the cursor one character forward, over a line break if need be.
#[inline(always)]
fn cursor_forward() -> c_int {
    // SAFETY: `curwin`/`curbuf` are live for the whole session.
    inc_cursor()
}

/// Step the cursor one character back, over a line break if need be.
#[inline(always)]
fn cursor_back() -> c_int {
    // SAFETY: `curwin`/`curbuf` are live for the whole session.
    dec_cursor()
}

/// Delete the character under the cursor, leaving the cursor where it is.
#[inline(always)]
fn delete_one_char() -> c_int {
    // SAFETY: `curwin`/`curbuf` are live for the whole session.
    unsafe { del_char(false) }
}

/// The character under the cursor, `NUL` at the end of the line.
#[inline(always)]
fn char_at_cursor() -> c_int {
    // SAFETY: `curwin`/`curbuf` are live for the whole session.
    gchar_cursor()
}

/// The multi-byte class of the character under the cursor, which is what
/// tells one word from the next for CTRL-W.
#[inline(always)]
fn cursor_char_class() -> c_int {
    // SAFETY: `curwin`/`curbuf` are live for the whole session.
    unsafe { mb_get_class(get_cursor_pos_ptr()) }
}

/// The screen width of the character `sci` names, standing at virtual
/// column `vcol`.  `use_ts` says whether a TAB still advances to a tab stop.
#[inline(always)]
fn charsize_at(use_ts: bool, vcol: colnr_T, sci: StrCharInfo) -> c_int {
    // SAFETY: `sci` names a character of a live line of `curbuf`.
    unsafe { charsize_nowrap(Buf::new(curbuf.get()), sci.ptr, use_ts, vcol, sci.chr.value) }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
