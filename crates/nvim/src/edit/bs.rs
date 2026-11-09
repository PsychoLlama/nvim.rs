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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::cstr::byte_at;
use crate::indent::StopList;
use crate::keycodes::SpecialKey;
use crate::mbyte::CharInfo;
use crate::mbyte::{c_char_info, c_next_cluster, char_at, char_len, class_in, iscomposing};
use crate::types::Key;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::ascii::NUL;
use crate::memline::Moved;
use crate::option::cpo_has;
use crate::option::defs::{BsFlag, CpoFlag, FoFlag};
use crate::types::Failed;

/// Which backwards-delete key is running.
///
/// The word forms change from [`Backspace::Word`] to
/// [`Backspace::WordNotSpace`] *inside* the delete loop, at the first
/// non-blank: that is how "delete the white space, then the word" is
/// spelled.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
    let cursor = Win::current().w_cursor;
    let orig = INSSTART_ORIG.get();
    if cursor.lnum == orig.lnum && cursor.col < orig.col {
        INSSTART_ORIG.set(orig.with_col(cursor.col));
    }
}

/// `<Del>` in Insert mode: delete forwards.
///
/// At the end of a line that means joining the next one, which needs
/// 'backspace' to contain `eol` just as a backspace over a line break does.
pub(crate) fn ins_del() {
    if stop_arrow().is_err() {
        return;
    }
    if char_at_cursor() == NUL {
        // Delete the newline.
        let temp = Win::current().w_cursor.col;
        if !can_bs(BsFlag::EOL) || do_join(2, false, true, false, false).is_err() {
            beep_backspace();
        } else {
            Win::current().w_cursor.col = temp;
            // Adjust `ORIG_LINE_COUNT` when more lines were deleted than
            // added, so a later `open_line` can still reach every line.
            if STATE.get() & VREPLACE_FLAG != 0
                && ORIG_LINE_COUNT.get() > Buf::current().b_ml.ml_line_count
            {
                ORIG_LINE_COUNT.set(Buf::current().b_ml.ml_line_count);
            }
        }
    } else if delete_one_char().is_err() {
        // Delete the character under the cursor.
        beep_backspace();
    }
    DID_AI.set(false);
    DID_SI.set(false);
    CAN_SI.set(false);
    CAN_SI_BACK.set(false);
    append_to_redobuff_char(Key::from(SpecialKey::Del));
}

/// Everything a backspace is not allowed to delete.
///
/// Nothing at all in an empty file; never past the first character in the
/// buffer; not past the start of the insert unless 'backspace' has `start`
/// (with a prompt buffer excepted, because the prompt is protected
/// separately); not into an auto-indent without `indent`; and not over a
/// line break without `eol`.  All of it is off in 'revins'.
fn bs_blocked() -> bool {
    if buf_is_empty(Buf::current()) {
        return true;
    }
    if REVINS_ON.get() {
        return false;
    }
    let cursor = Win::current().w_cursor;
    let start = INSSTART_ORIG.get();
    (cursor.lnum == LineNr::ONE && cursor.col == ColNr::ZERO)
        || (!can_bs(BsFlag::START)
            && ((ARROW_USED.get() && !buf_is_prompt(current_buf()))
                || (cursor.lnum == start.lnum && cursor.col <= start.col)))
        || (!can_bs(BsFlag::INDENT)
            && !ARROW_USED.get()
            && AI_COL.get() > ColNr::ZERO
            && cursor.col <= AI_COL.get())
        || (!can_bs(BsFlag::EOL) && cursor.col == ColNr::ZERO)
}

/// Handle Backspace, delete-word and delete-line in Insert mode.
///
/// `c` is the character that was typed (it goes into the redo buffer),
/// `mode` says which of the three keys it is, and `inserted_space` is the
/// caller's "the last thing inserted was a space" flag, which
/// [`bs_one_shiftwidth`] both reads and clears.
///
/// Answers whether a backspace actually happened.
pub(crate) fn ins_bs(c: Key, mode: Backspace, inserted_space: &mut bool) -> bool {
    if bs_blocked() {
        beep_backspace();
        return false;
    }
    if stop_arrow().is_err() {
        return false;
    }

    let in_indent = inindent(0);
    if in_indent {
        CAN_CINDENT.set(false);
    }
    END_COMMENT_PENDING.set(None); // after BS, don't auto-end a comment
    if REVINS_ON.get() {
        cursor_forward(); // put the cursor after the last inserted character
    }

    // In 'virtualedit': BACKSPACE_CHAR eats one virtual space,
    // BACKSPACE_WORD eats all the `coladd`, and BACKSPACE_LINE eats all
    // of it and keeps going.
    if Win::current().w_cursor.coladd > 0 {
        if mode == Backspace::Char {
            Win::current().w_cursor.coladd -= 1;
            return true;
        }
        if mode == Backspace::Word {
            Win::current().w_cursor.coladd = 0;
            return true;
        }
        Win::current().w_cursor.coladd = 0;
    }

    let mut did_backspace = false;
    let mut call_fix_indent = false;

    if Win::current().w_cursor.col == ColNr::ZERO {
        if !bs_join_line() {
            return false;
        }
        DID_AI.set(false);
    } else {
        if REVINS_ON.get() {
            cursor_back(); // put the cursor on the last inserted character
        }

        // Keep the indent: CTRL-U stops at the first non-blank if there
        // is one before the cursor.
        let mut mincol: ColNr = ColNr::ZERO;
        if mode == Backspace::Line
            && (Buf::current().b_p_ai != 0 || cindent_on())
            && !REVINS_ON.get()
        {
            let save_col = Win::current().w_cursor.col;
            beginline(BeginlineOpts::WHITE);
            if Win::current().w_cursor.col < save_col {
                mincol = Win::current().w_cursor.col;
                // The indent should now be fixed to match the previous
                // line.
                call_fix_indent = true;
            }
            Win::current().w_cursor.col = save_col;
        }

        // One BS deletes a whole 'shiftwidth' or 'softtabstop' when
        // 'smarttab' says so in the indent, or when the byte before the
        // cursor is white space this insert did not type.
        // A closure, so that the byte before the cursor is only read once
        // 'smarttab' has had its say and the column has been checked.
        let soft_tab = || {
            (get_sts_value() != 0 || Buf::current().stop_count(StopList::SoftTab) != 0)
                && Win::current().w_cursor.col > ColNr::ZERO
                && {
                    // The cursor is past column 0, so the byte before it is
                    // a byte of the cursor's own line.
                    let before = with_cursor_line(|line, col| byte_at(line, col - 1));
                    before == b'\t' || (before == b' ' && (!*inserted_space || ARROW_USED.get()))
                }
        };
        let one_step = mode == Backspace::Char && ((p_sta() && in_indent) || soft_tab());
        if one_step {
            *inserted_space = false;
            bs_one_shiftwidth(in_indent);
        } else {
            bs_delete_chars(mode, mincol);
        }
        did_backspace = true;
    }

    DID_SI.set(false);
    CAN_SI.set(false);
    CAN_SI_BACK.set(false);
    if Win::current().w_cursor.col <= ColNr::new(1) {
        DID_AI.set(false);
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
    if cpo_has(CpoFlag::BACKSPACE) && DOLLAR_VCOL.get() == ColNr::NONE {
        DOLLAR_VCOL.set(Win::current().w_virtcol);
    }

    // After deleting a character the cursor line must never be in a
    // closed fold -- with 'foldmethod' indent, deleting the first
    // non-white character before a TAB can put it in one.
    if did_backspace {
        fold_open_cursor();
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
    let lnum = INSSTART.get().lnum;
    if Win::current().w_cursor.lnum == lnum || REVINS_ON.get() {
        if u_save(
            Win::current().w_cursor.lnum - 2,
            Win::current().w_cursor.lnum + 1,
        )
        .is_err()
        {
            return false;
        }
        let lnum = INSSTART.get().lnum - 1;
        let len = ml_get_len(lnum);
        INSSTART.set(INSSTART.get().with_lnum(lnum).with_col(len));
    }

    // In Replace mode: below zero the NL was inserted, so delete it; at
    // or above zero it replaced characters, which go back.
    let mut open = false;
    if STATE.get() & REPLACE_FLAG != 0 {
        // Nothing open if the NL was inserted.
        open = is_open(replace_pop_if_nul());
    }

    // In Replace mode, on the line the replacing started on, only the
    // cursor moves.
    if STATE.get() & REPLACE_FLAG != 0 && Win::current().w_cursor.lnum <= lnum {
        cursor_back();
        return true;
    }

    if STATE.get() & VREPLACE_FLAG == 0 || Win::current().w_cursor.lnum > ORIG_LINE_COUNT.get() {
        let temp = char_at_cursor(); // remember the current character
        Win::current().w_cursor.lnum -= 1;

        // With `aw` in 'formatoptions' the space at the end of the line
        // has to go too, or auto-formatting would break the line again.
        if has_format_option(FoFlag::AUTO) && has_format_option(FoFlag::WHITE_PAR) {
            let shorter = with_cursor_line(|line, _| {
                line.strip_suffix(b" ").map(|head| {
                    let mut newp = Vec::with_capacity(line.len());
                    newp.extend_from_slice(head);
                    newp.push(0);
                    newp
                })
            });
            if let Some(newp) = shorter {
                Buf::current().b_ml.swap_cached_text(newp);
            }
        }

        let _ = do_join(2, false, false, false, false);
        if temp == NUL && char_at_cursor() != NUL {
            cursor_forward();
        }
    } else {
        cursor_back();
    }

    if STATE.get() & REPLACE_FLAG != 0 {
        // Do the insertions in MODE_NORMAL state, so `ins_char` does not
        // replace characters and does not call `showmatch`.
        let old_state = STATE.get();
        STATE.set(MODE_NORMAL);
        // Restore the characters (blanks) that were deleted after the
        // cursor...
        while open {
            let save_col = Win::current().w_cursor.col;
            mb_replace_pop_ins();
            Win::current().w_cursor.col = save_col;
            open = is_open(replace_pop_if_nul());
        }
        // ... and then the ones the NL replaced.
        replace_pop_ins();
        STATE.set(old_state);
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
    let use_ts = Win::current().w_onebuf_opt.wo_list == 0 || Win::current().w_p_lcs_chars.tab1 != 0;
    let (want_vcol, want_col, mut space_vcol) = with_cursor_line(|line, cursor_at| {
        // The cursor's virtual column, and the last white space before it
        // that is preceded by non-white space.
        let mut vcol: ColNr = ColNr::ZERO;
        let mut space_vcol: ColNr = ColNr::ZERO;
        let (mut at, mut chr) = (0, c_char_info(line, 0));
        let (mut space_at, mut space_chr) = (at, chr);
        let mut prev_space = false;
        while at < cursor_at {
            let cur_space = ascii_iswhite(chr.value);
            if !prev_space && cur_space {
                (space_at, space_chr) = (at, chr);
                space_vcol = vcol;
            }
            vcol += charsize_at(line, use_ts, vcol, at, chr);
            (at, chr) = c_next_cluster(line, at, chr);
            prev_space = cur_space;
        }

        // The virtual column to end up at.
        let mut want_vcol = if vcol > ColNr::ZERO {
            vcol - 1
        } else {
            ColNr::ZERO
        };
        if p_sta() && in_indent {
            want_vcol -= want_vcol.get() % get_sw_value(Buf::current());
        } else {
            let sts = get_sts_value();
            want_vcol = Buf::current().stop_start(StopList::SoftTab, want_vcol, sts);
        }

        // Where to stop backspacing.
        loop {
            let size = charsize_at(line, use_ts, space_vcol, space_at, space_chr);
            if space_vcol + size > want_vcol {
                break;
            }
            space_vcol += size;
            (space_at, space_chr) = c_next_cluster(line, space_at, space_chr);
        }
        (
            want_vcol,
            ColNr::try_from(space_at).unwrap_or(ColNr::MAX),
            space_vcol,
        )
    });

    // Delete until at or before `want_col`.
    while Win::current().w_cursor.col > want_col {
        cursor_back();
        if STATE.get() & REPLACE_FLAG != 0 {
            // Don't delete before the insert point in Replace mode.
            if Win::current().w_cursor.lnum != INSSTART.get().lnum
                || Win::current().w_cursor.col >= INSSTART.get().col
            {
                replace_do_bs(-1);
            }
        } else {
            let _ = delete_one_char();
        }
    }

    // Insert spaces until at `want_vcol`.
    while space_vcol < want_vcol {
        // Remember the first character inserted.
        pull_insstart_orig_to_cursor();

        if STATE.get() & VREPLACE_FLAG != 0 {
            ins_char(Key::SPACE);
        } else {
            ins_str(b" ");
            if STATE.get() & REPLACE_FLAG != 0 {
                replace_push_nul();
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
fn bs_delete_chars(mut mode: Backspace, mincol: ColNr) {
    // What kind of word the deletion started in, so a class change can
    // end it.
    let mut cclass = cursor_char_class();
    // Whether the word being deleted is made of 'iskeyword' characters;
    // only read once `mode` is `WordNotSpace`.
    let mut is_word = 0;
    loop {
        if !REVINS_ON.get() {
            cursor_back(); // put the cursor on the character to delete
        }
        let cc = char_at_cursor();
        let prev_cclass = cclass;
        cclass = cursor_char_class();

        if mode == Backspace::Word && !ascii_isspace(cc) {
            // The start of the word.
            mode = Backspace::WordNotSpace;
            is_word = c_int::from(vim_iswordc(cc));
        } else if mode == Backspace::WordNotSpace
            && (ascii_isspace(cc)
                || c_int::from(vim_iswordc(cc)) != is_word
                || prev_cclass != cclass)
        {
            // The end of the word.
            if !REVINS_ON.get() {
                cursor_forward();
            } else if STATE.get() & REPLACE_FLAG != 0 {
                cursor_back();
            }
            break;
        }

        if STATE.get() & REPLACE_FLAG != 0 {
            replace_do_bs(-1);
        } else {
            let mut has_composing = false;
            if p_deco() {
                // The character after the cursor's is at most the line's NUL.
                has_composing = with_cursor_line(|line, col| {
                    let tail = &line[col.min(line.len())..];
                    let next = &tail[char_len(tail)..];
                    byte_at(next, 0) >= 0x80 && iscomposing(char_at(tail), char_at(next), None)
                });
            }
            let _ = delete_one_char();
            // With combining characters and 'delcombine' set, move the
            // cursor back -- but never before the base character.
            if has_composing {
                cursor_forward();
            }
            if REVINS_CHARS.get() != 0 {
                REVINS_CHARS.set(REVINS_CHARS.get() - 1);
                REVINS_LEGAL.set(REVINS_LEGAL.get() + 1);
            }
            if REVINS_ON.get() && char_at_cursor() == NUL {
                break;
            }
        }

        // Just a single backspace?
        if mode == Backspace::Char {
            break;
        }
        // The `do`-`while` condition: keep going while there is
        // something left this key is allowed to take.
        let more = REVINS_ON.get()
            || (Win::current().w_cursor.col > mincol
                && (can_bs(BsFlag::NOSTOP)
                    || (Win::current().w_cursor.lnum != INSSTART_ORIG.get().lnum
                        || Win::current().w_cursor.col != INSSTART_ORIG.get().col)));
        if !more {
            break;
        }
    }
}

/// Beep, or flash, for a backspace that could not delete anything.
#[inline(always)]
fn beep_backspace() {
    vim_beep(OPT_BO_FLAG_BACKSPACE as ::core::ffi::c_uint)
}

/// Step the cursor one character forward, over a line break if need be.
#[inline(always)]
fn cursor_forward() -> Moved {
    inc_cursor()
}

/// Step the cursor one character back, over a line break if need be.
#[inline(always)]
fn cursor_back() -> Moved {
    dec_cursor()
}

/// Delete the character under the cursor, leaving the cursor where it is.
#[inline(always)]
fn delete_one_char() -> Result<(), Failed> {
    del_char(false)
}

/// The character under the cursor, `NUL` at the end of the line.
#[inline(always)]
fn char_at_cursor() -> c_int {
    gchar_cursor()
}

/// The multi-byte class of the character under the cursor, which is what
/// tells one word from the next for CTRL-W.
#[inline(always)]
fn cursor_char_class() -> c_int {
    with_cursor_line(|line, col| class_in(&line[col.min(line.len())..], Buf::current()))
}

/// The screen width of the character `chr` at byte `at` of `line`, standing
/// at virtual column `vcol`.  `use_ts` says whether a TAB still advances to a
/// tab stop.
#[inline(always)]
fn charsize_at(line: &[u8], use_ts: bool, vcol: ColNr, at: usize, chr: CharInfo) -> c_int {
    let text = &line[at.min(line.len())..(at + CHAR_CELLS_SPAN).min(line.len())];
    charsize_nowrap(Buf::current(), text, use_ts, vcol, chr.value)
}

/// `f` of the cursor's line and the cursor's column in it.
#[inline(always)]
fn with_cursor_line<R>(f: impl FnOnce(&[u8], usize) -> R) -> R {
    let cursor = Win::current().w_cursor;
    let mut lines = Lines::current();
    f(
        lines.line(cursor.lnum),
        usize::try_from(cursor.col).unwrap_or(usize::MAX),
    )
}
