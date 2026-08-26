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

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::types::NUL;

/// The effective prompt for `buf`: 'b:prompt_text', or `"% "`.
///
/// # Safety
/// `buf` must point to a live buffer.
pub(crate) unsafe fn buf_prompt_text(buf: *const buf_T) -> *mut c_char {
    // SAFETY: the caller promises `buf` is a live buffer, and 'b:prompt_text'
    // is either null or a NUL-terminated string it owns.
    if unsafe { (*buf).b_prompt_text.is_null() } {
        return c"% ".as_ptr().cast_mut();
    }
    unsafe { (*buf).b_prompt_text }
}

/// The effective prompt for the current buffer.
///
/// # Safety
/// Must run with a live `curbuf`.
pub(crate) unsafe fn prompt_text() -> *mut c_char {
    // SAFETY: `curbuf` is live for the whole session.
    unsafe { buf_prompt_text(curbuf.get()) }
}

/// Prepare for prompt mode: make sure the prompt line carries the prompt
/// text, and move the cursor after it.
///
/// `cmdchar_todo` is the command that started the insert, so that an `A`
/// still means "at the end of the line" once the cursor has been moved onto
/// the prompt line.
pub(crate) fn init_prompt(cmdchar_todo: c_int) {
    let mut win = cur_win();
    // SAFETY: every `unsafe` call in this function is an editor-wide routine
    // whose only precondition is the live `curwin`/`curbuf` this mode runs
    // with; `prompt` and `text` are NUL-terminated strings of that buffer.
    let prompt = unsafe { prompt_text() };
    let prompt_len = unsafe { strlen(prompt) } as c_int;

    // The mark may name a line that no longer exists.  It is read and
    // written a field at a time rather than held: the calls below adjust
    // marks, this one included.
    if start().lnum < 1 || start().lnum > cur_buf().b_ml.ml_line_count {
        set_start_lnum(start().lnum.min(cur_buf().b_ml.ml_line_count).max(1));
        cur_buf().b_prompt_append_new_line = true;
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
                    prompt_len as size_t,
                )
            }
    };
    if start().lnum == win.w_cursor.lnum && prompt_missing() {
        if unsafe { *text } as c_int == NUL {
            // The line is empty: the prompt *is* the line.
            unsafe { ml_replace(start().lnum, prompt, true) };
            unsafe { inserted_bytes(start().lnum, 0, 0, prompt_len) };
        } else {
            // The line holds something else, so the prompt goes on a new
            // last line.
            let lnum = cur_buf().b_ml.ml_line_count;
            unsafe { ml_append(lnum, prompt, 0, false) };
            unsafe { appended_lines_mark(lnum, 1) };
            set_start_lnum(cur_buf().b_ml.ml_line_count);
            cur_buf().b_prompt_append_new_line = true;
            // Like submitting: the undo history belonged to the old
            // prompt.
            unsafe { u_clearallandblockfree(curbuf.get()) };
        }
        set_start_col(prompt_len);
        win.w_cursor.lnum = cur_buf().b_ml.ml_line_count;
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
        Insstart_blank_vcol.set(MAXCOL as colnr_T);
        arrow_used.set(false);
    }

    if cmdchar_todo == 'A' as c_int {
        coladvance_win(win, MAXCOL as c_int);
    }
    if start().lnum == win.w_cursor.lnum {
        win.w_cursor.col = win.w_cursor.col.max(start().col);
    }
    // Make sure the cursor is in a valid position.
    unsafe { check_cursor(win.raw()) };
}

/// Where the prompt's editable part begins.
#[inline(always)]
fn start() -> pos_T {
    cur_buf().b_prompt_start.mark
}

/// Move that mark to line `lnum`.
#[inline(always)]
fn set_start_lnum(lnum: linenr_T) {
    cur_buf().b_prompt_start.mark.lnum = lnum;
}

/// Move that mark to column `col`.
#[inline(always)]
fn set_start_col(col: colnr_T) {
    cur_buf().b_prompt_start.mark.col = col;
}

/// Move `win`'s cursor to virtual column `vcol` of its line.
#[inline(always)]
fn coladvance_win(win: Win, vcol: c_int) {
    // SAFETY: a live window, whose cursor line exists.
    unsafe { coladvance(win.raw(), vcol) };
}

/// Is the cursor in the editable part of the prompt line?
///
/// # Safety
/// Must run with a live `curbuf`/`curwin`.
pub(crate) unsafe fn prompt_curpos_editable() -> bool {
    let start = start();
    let cursor = cur_win().w_cursor;
    cursor.lnum > start.lnum || (cursor.lnum == start.lnum && cursor.col >= start.col)
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
