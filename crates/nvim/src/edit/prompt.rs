//! The prompt buffer: an Insert mode with a read-only prefix.
//!
//! A 'buftype' of "prompt" makes the last line a prompt the user types after
//! and cannot back over.  [`init_prompt`] is what runs on entering Insert
//! mode in such a buffer: make sure the prompt line exists and starts with
//! the prompt text, and put the cursor after it.  [`buf_prompt_text_owned`]
//! resolves 'b:prompt_text' against the default, and
//! [`prompt_curpos_editable`] is the guard `ins_bs` and the cursor motions
//! ask before moving left.
//!
//! `b_prompt_start` is the mark that says where the editable part begins: a
//! line number *and* a column, because the prompt occupies the head of its
//! own line.  Everything here is about keeping that mark and the buffer in
//! agreement, since either can have moved while the buffer was not in Insert
//! mode.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::memory::ThinCString;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;

/// Give `buffer` the prompt `text`, releasing the one it had.
pub(crate) fn set_buf_prompt_text(mut buffer: Buf, text: ThinCString) {
    buffer.b_prompt_text = Some(text);
}

/// The effective prompt for `buffer`, 'b:prompt_text' or `"% "`, copied.
pub(crate) fn buf_prompt_text_owned(buffer: Buf) -> ThinCString {
    buffer
        .b_prompt_text
        .clone()
        .unwrap_or_else(|| ThinCString::from_cstr(c"% "))
}

/// Prepare for prompt mode: make sure the prompt line carries the prompt
/// text, and move the cursor after it.
///
/// `cmdchar_todo` is the command that started the insert, so that an `A`
/// still means "at the end of the line" once the cursor has been moved onto
/// the prompt line.
pub(crate) fn init_prompt(cmdchar_todo: c_int) {
    let mut win = Win::current();
    let prompt = buf_prompt_text_owned(Buf::current());
    let prompt = prompt.as_bytes();
    let prompt_len =
        c_int::try_from(prompt.len()).expect("a prompt string is never longer than an int");

    // The mark may name a line that no longer exists.  It is read and
    // written a field at a time rather than held: the calls below adjust
    // marks, this one included.
    if start().lnum < 1 || start().lnum > Buf::current().b_ml.ml_line_count {
        set_start_lnum(start().lnum.min(Buf::current().b_ml.ml_line_count).max(1));
        Buf::current().b_prompt_append_new_line = true;
    }

    win.w_cursor.lnum = win.w_cursor.lnum.max(start().lnum);

    // Is the prompt actually there, ending at the mark's column?
    let start_col = start().col;
    let (prompt_missing, empty) = {
        let mut lines = Lines::current();
        let text = lines.line(start().lnum);
        let missing = usize::try_from(start_col).ok().is_none_or(|end| {
            end < prompt.len() || end > text.len() || text[end - prompt.len()..end] != *prompt
        });
        (missing, text.is_empty())
    };
    if start().lnum == win.w_cursor.lnum && prompt_missing {
        if empty {
            // The line is empty: the prompt *is* the line.
            let _ = ml_replace_buf_text(Buf::current(), start().lnum, prompt);
            inserted_bytes(start().lnum, 0, 0, prompt_len);
        } else {
            // The line holds something else, so the prompt goes on a new
            // last line.
            let lnum = Buf::current().b_ml.ml_line_count;
            let _ = ml_append_text(lnum, prompt);
            appended_lines_mark(lnum, 1);
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
    coladvance(win, vcol);
}

/// Is the cursor in the editable part of the prompt line?
pub(crate) fn prompt_curpos_editable() -> bool {
    let start = start();
    let cursor = Win::current().w_cursor;
    cursor.lnum > start.lnum || (cursor.lnum == start.lnum && cursor.col >= start.col)
}
