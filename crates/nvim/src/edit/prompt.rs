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

use core::ffi::{c_char, c_int};

use super::*;
use crate::types::NUL;

/// The effective prompt for `buf`: 'b:prompt_text', or `"% "`.
///
/// # Safety
/// `buf` must point to a live buffer.
pub(crate) unsafe fn buf_prompt_text(buf: *const buf_T) -> *mut c_char {
    unsafe {
        if (*buf).b_prompt_text.is_null() {
            return c"% ".as_ptr().cast_mut();
        }
        (*buf).b_prompt_text
    }
}

/// The effective prompt for the current buffer.
///
/// # Safety
/// Must run with a live `curbuf`.
pub(crate) unsafe fn prompt_text() -> *mut c_char {
    unsafe { buf_prompt_text(curbuf.get()) }
}

/// Prepare for prompt mode: make sure the prompt line carries the prompt
/// text, and move the cursor after it.
///
/// `cmdchar_todo` is the command that started the insert, so that an `A`
/// still means "at the end of the line" once the cursor has been moved onto
/// the prompt line.
///
/// # Safety
/// Must run with a live `curbuf`/`curwin`.
pub(crate) unsafe fn init_prompt(cmdchar_todo: c_int) {
    unsafe {
        let buf = curbuf.get();
        let win = curwin.get();
        let prompt = prompt_text();
        let prompt_len = strlen(prompt) as c_int;

        // The mark may name a line that no longer exists.
        let start = &raw mut (*buf).b_prompt_start.mark;
        if (*start).lnum < 1 || (*start).lnum > (*buf).b_ml.ml_line_count {
            (*start).lnum = (*start).lnum.min((*buf).b_ml.ml_line_count).max(1);
            (*buf).b_prompt_append_new_line = true;
        }

        (*win).w_cursor.lnum = (*win).w_cursor.lnum.max((*start).lnum);
        let text = ml_get((*start).lnum);
        let text_len = ml_get_len((*start).lnum);

        // Is the prompt actually there, ending at the mark's column?  The
        // `col` bounds are what keeps the `strnequal` read inside the line.
        let prompt_missing = || {
            (*start).col < prompt_len
                || (*start).col > text_len
                || !strnequal(
                    text.offset(((*start).col - prompt_len) as isize),
                    prompt,
                    prompt_len as size_t,
                )
        };
        if (*start).lnum == (*win).w_cursor.lnum && prompt_missing() {
            if *text as c_int == NUL {
                // The line is empty: the prompt *is* the line.
                ml_replace((*start).lnum, prompt, true);
                inserted_bytes((*start).lnum, 0, 0, prompt_len);
            } else {
                // The line holds something else, so the prompt goes on a new
                // last line.
                let lnum = (*buf).b_ml.ml_line_count;
                ml_append(lnum, prompt, 0, false);
                appended_lines_mark(lnum, 1);
                (*start).lnum = (*buf).b_ml.ml_line_count;
                (*buf).b_prompt_append_new_line = true;
                // Like submitting: the undo history belonged to the old
                // prompt.
                u_clearallandblockfree(buf);
            }
            (*start).col = prompt_len;
            (*win).w_cursor.lnum = (*buf).b_ml.ml_line_count;
            coladvance(win, MAXCOL as c_int);
        }

        // The insert always starts after the prompt; text after it stays
        // editable.
        if (*Insstart_orig.ptr()).lnum != (*start).lnum
            || (*Insstart_orig.ptr()).col != (*start).col
        {
            (*Insstart.ptr()).lnum = (*start).lnum;
            (*Insstart.ptr()).col = (*start).col;
            Insstart_orig.set(Insstart.get());
            Insstart_textlen.set((*Insstart.ptr()).col);
            Insstart_blank_vcol.set(MAXCOL as colnr_T);
            arrow_used.set(false);
        }

        if cmdchar_todo == 'A' as c_int {
            coladvance(win, MAXCOL as c_int);
        }
        if (*start).lnum == (*win).w_cursor.lnum {
            (*win).w_cursor.col = (*win).w_cursor.col.max((*start).col);
        }
        // Make sure the cursor is in a valid position.
        check_cursor(win);
    }
}

/// Is the cursor in the editable part of the prompt line?
///
/// # Safety
/// Must run with a live `curbuf`/`curwin`.
pub(crate) unsafe fn prompt_curpos_editable() -> bool {
    unsafe {
        let start = (*curbuf.get()).b_prompt_start.mark;
        let cursor = (*curwin.get()).w_cursor;
        cursor.lnum > start.lnum || (cursor.lnum == start.lnum && cursor.col >= start.col)
    }
}
