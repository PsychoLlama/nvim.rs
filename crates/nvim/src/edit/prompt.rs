//! The prompt buffer: an Insert mode with a read-only prefix.
//!
//! A 'buftype' of "prompt" makes the last line a prompt the user types after
//! and cannot back over.  [`init_prompt`] is what runs on entering Insert
//! mode in such a buffer: make sure the prompt line exists and starts with
//! the prompt text, and put the cursor after it.  [`buf_prompt_text`]
//! resolves 'b:prompt_text' against the default, and
//! [`prompt_curpos_editable`] is the guard `ins_bs` and the cursor motions
//! ask before moving left.
//!
//! `b_prompt_start` is the mark that says where the editable part begins: a
//! line number *and* a column, because the prompt occupies the head of its
//! own line.  Everything here is about keeping that mark and the buffer in
//! agreement, since either can have moved while the buffer was not in Insert
//! mode.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::types::NUL;

/// The effective prompt for `buffer`: 'b:prompt_text', or `"% "`.
///
/// Safe: `b:prompt_text` is either null or a NUL-terminated string the
/// buffer owns.
pub(crate) fn buf_prompt_text(buffer: Buf) -> *mut c_char {
    if buffer.b_prompt_text.is_null() {
        return c"% ".as_ptr().cast_mut();
    }
    buffer.b_prompt_text
}

/// The effective prompt for the current buffer.
pub(crate) fn prompt_text() -> *mut c_char {
    buf_prompt_text(Buf::current())
}

/// Prepare for prompt mode: make sure the prompt line carries the prompt
/// text, and move the cursor after it.
///
/// `cmdchar_todo` is the command that started the insert, so that an `A`
/// still means "at the end of the line" once the cursor has been moved onto
/// the prompt line.
pub(crate) fn init_prompt(cmdchar_todo: c_int) {
    let mut win = Win::current();
    let prompt = prompt_text();
    let prompt_bytes = unsafe { cstr::bytes_at(prompt) }.len();
    let prompt_len =
        c_int::try_from(prompt_bytes).expect("a prompt string is never longer than an int");

    // The mark may name a line that no longer exists.  It is read and
    // written a field at a time rather than held: the calls below adjust
    // marks, this one included.
    if start().lnum < 1 || start().lnum > Buf::current().b_ml.ml_line_count {
        set_start_lnum(start().lnum.min(Buf::current().b_ml.ml_line_count).max(1));
        Buf::current().b_prompt_append_new_line = true;
    }

    win.w_cursor.lnum = win.w_cursor.lnum.max(start().lnum);
    let text = ml_get(start().lnum);
    let text_len = ml_get_len(start().lnum);

    // Is the prompt actually there, ending at the mark's column?  The
    // `col` bounds are what keeps the `strnequal` read inside the line, so
    // this stays a closure: it must not run before they have been checked.
    let start_col = start().col;
    let prompt_missing = || {
        start_col < prompt_len
            || start_col > text_len
            || !unsafe {
                strnequal(
                    text.offset((start_col - prompt_len) as isize),
                    prompt,
                    prompt_bytes,
                )
            }
    };
    if start().lnum == win.w_cursor.lnum && prompt_missing() {
        if c_int::from(unsafe { *text }) == NUL {
            // The line is empty: the prompt *is* the line.
            let _ = unsafe { ml_replace(start().lnum, prompt, true) };
            unsafe { inserted_bytes(start().lnum, 0, 0, prompt_len) };
        } else {
            // The line holds something else, so the prompt goes on a new
            // last line.
            let lnum = Buf::current().b_ml.ml_line_count;
            let _ = unsafe { ml_append(lnum, prompt, 0, false) };
            unsafe { appended_lines_mark(lnum, 1) };
            set_start_lnum(Buf::current().b_ml.ml_line_count);
            Buf::current().b_prompt_append_new_line = true;
            // Like submitting: the undo history belonged to the old
            // prompt.
            u_clearallandblockfree(Buf::current());
        }
        set_start_col(prompt_len);
        win.w_cursor.lnum = Buf::current().b_ml.ml_line_count;
        coladvance_win(win, MAXCOL as c_int);
    }

    // The insert always starts after the prompt; text after it stays
    // editable.
    if Insstart_orig.get().lnum != start().lnum || Insstart_orig.get().col != start().col {
        let mut insstart = Insstart.get();
        insstart.lnum = start().lnum;
        insstart.col = start().col;
        Insstart.set(insstart);
        Insstart_orig.set(insstart);
        Insstart_textlen.set(insstart.col);
        Insstart_blank_vcol.set(MAXCOL as ColNr);
        arrow_used.set(false);
    }

    if cmdchar_todo == 'A' as c_int {
        coladvance_win(win, MAXCOL as c_int);
    }
    if start().lnum == win.w_cursor.lnum {
        win.w_cursor.col = win.w_cursor.col.max(start().col);
    }
    // Make sure the cursor is in a valid position.
    check_cursor(win);
}

/// Where the prompt's editable part begins.
#[inline(always)]
fn start() -> Pos {
    Buf::current().b_prompt_start.mark
}

/// Move that mark to line `lnum`.
#[inline(always)]
fn set_start_lnum(lnum: LineNr) {
    Buf::current().b_prompt_start.mark.lnum = lnum;
}

/// Move that mark to column `col`.
#[inline(always)]
fn set_start_col(col: ColNr) {
    Buf::current().b_prompt_start.mark.col = col;
}

/// Move `win`'s cursor to virtual column `vcol` of its line.
#[inline(always)]
fn coladvance_win(win: Win, vcol: c_int) {
    // SAFETY: a live window, whose cursor line exists.
    coladvance(win, vcol);
}

/// Is the cursor in the editable part of the prompt line?
pub(crate) fn prompt_curpos_editable() -> bool {
    let start = start();
    let cursor = Win::current().w_cursor;
    cursor.lnum > start.lnum || (cursor.lnum == start.lnum && cursor.col >= start.col)
}
