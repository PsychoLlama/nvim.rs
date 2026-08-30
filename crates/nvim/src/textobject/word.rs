//! Word motions and the `iw`/`aw` objects.
//!
//! Every one of these is the same walk: step a character at a time and stop
//! when [`cls`] -- the character *class* under the cursor, folded to one
//! bucket when a WORD is asked for -- changes. `w`/`b`/`e`/`ge` are the four
//! directions of it, and [`current_word`] composes them.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::cursor::{coladvance, dec_cursor, gchar_cursor, get_cursor_line_ptr, inc_cursor};
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::edit::oneleft;
use crate::global_cell::GlobalCell;
use crate::main::{VIsual_select_exclu_adj, p_sel, redraw_cmdline};
use crate::mbyte::utf_class;
use crate::memline::{decl, incl, ml_get};
use crate::r#move::adjust_skipcol;
use crate::normal::{
    VisualMode, set_visual_anchor, set_visual_mode, unadjust_for_sel, visual_active, visual_anchor,
    visual_mode,
};
use crate::pos::{MAXCOL, clearpos, equalpos, lt, ltoreq};
use crate::search::{BACKWARD, FORWARD};
use crate::types::{Failed, NUL, oparg_T, pos_T};

/// Whether [`cls`] should answer a WORD's classes rather than a word's.
///
/// Upstream's `cls()` takes no argument because it is called from inside
/// loops that would otherwise have to thread the flag; this is that flag.
static cls_bigword: GlobalCell<bool> = GlobalCell::new(false);

/// The character class at the cursor: 0 for white space and end of line, 1
/// for punctuation, 2 and up for the 'iskeyword' and multibyte classes.
///
/// With `cls_bigword` set every non-blank answers 1, which is what makes
/// `W`/`B`/`E` treat a run of anything as one word.
fn cls() -> c_int {
    // SAFETY: `curwin` and `curbuf` are set from startup to exit, which is all
    // `gchar_cursor` asks for; it answers NUL past the end of the line.
    let c = gchar_cursor();
    if c == ' ' as c_int || c == '\t' as c_int || c == NUL {
        return 0;
    }
    // SAFETY: as above -- `utf_class` only reads the current buffer's
    // 'iskeyword' table.
    let c = unsafe { utf_class(c) };
    if c != 0 && cls_bigword.get() { 1 } else { c }
}

/// Step over a run of characters of class `cclass`. Answers true when the
/// end of the file was reached.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
unsafe fn skip_chars(cclass: c_int, dir: c_int) -> bool {
    while cls() == cclass {
        // SAFETY: the caller guarantees a current window with its cursor on a
        // line of the current buffer, which is what both of these ask for.
        let step = if dir == FORWARD as c_int {
            inc_cursor()
        } else {
            dec_cursor()
        };
        if step == -1 {
            return true;
        }
    }
    false
}

/// Go back to the start of the word, or of the run of white space, the
/// cursor is inside -- without leaving the line.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
unsafe fn back_in_line() {
    let sclass = cls();
    while cur_win().w_cursor.col != 0 {
        // SAFETY, both: the caller guarantees a current window with its cursor
        // on a line of the current buffer.
        dec_cursor();
        if cls() != sclass {
            inc_cursor(); // stop at the start of the word
            break;
        }
    }
}

/// `w` / `W`: move forward `count` words. Answers FAIL when the cursor was
/// already on the last character of the file.
///
/// With `eol`, the last word stops at end of line, which is what an operator
/// wants: `dw` on the last word of a line must not eat the newline.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
pub unsafe fn fwd_word(mut count: c_int, bigword: bool, eol: bool) -> Result<(), Failed> {
    cur_win().w_cursor.coladd = 0;
    cls_bigword.set(bigword);
    loop {
        count -= 1;
        if count < 0 {
            break;
        }
        // Inside a fold, move to the last character of the last line.
        if let Some(last) = cur_win().fold_end(cur_win().w_cursor.lnum) {
            cur_win().w_cursor.lnum = last;
            // SAFETY: `cur_win()` is a live window.
            coladvance(cur_win(), MAXCOL);
        }
        let sclass = cls();

        // Always move at least one character, unless this is the last one
        // in the buffer.
        let last_line = cur_win().w_cursor.lnum == cur_buf().b_ml.ml_line_count;
        // SAFETY, for every cursor step below: the caller guarantees a current
        // window with its cursor on a line of the current buffer, and each
        // step leaves it on one.
        let mut i = inc_cursor();
        if i == -1 || (i >= 1 && last_line) {
            return Err(Failed); // started on the last character of the file
        }
        if i >= 1 && eol && count == 0 {
            return Ok(()); // started on the last character of the line
        }

        // One character past the end of the current word, if any.
        if sclass != 0 {
            while cls() == sclass {
                i = inc_cursor();
                if i == -1 || (i >= 1 && eol && count == 0) {
                    return Ok(());
                }
            }
        }
        // Then on to the next non-white character.
        while cls() == 0 {
            // Stop on a blank line.
            // SAFETY: `get_cursor_line_ptr` hands back the cursor's line,
            // NUL-terminated, so its first byte is there to read.
            if cur_win().w_cursor.col == 0 && unsafe { *get_cursor_line_ptr() } as c_int == NUL {
                break;
            }
            i = inc_cursor();
            if i == -1 || (i >= 1 && eol && count == 0) {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// `b` / `B`: move back `count` words. Answers FAIL when the top of the file
/// was reached.
///
/// With `stop`, a cursor already on the start of a word moves one word less,
/// which is what makes `cb` from mid-word do the right thing.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
pub unsafe fn bck_word(mut count: c_int, bigword: bool, mut stop: bool) -> Result<(), Failed> {
    cur_win().w_cursor.coladd = 0;
    cls_bigword.set(bigword);
    loop {
        count -= 1;
        if count < 0 {
            break;
        }
        // Inside a fold, move to the first character of the first line.
        if let Some(first) = cur_win().fold_first(cur_win().w_cursor.lnum) {
            cur_win().w_cursor.lnum = first;
            cur_win().w_cursor.col = 0;
        }
        let sclass = cls();
        // SAFETY, for every step below: the caller guarantees a current window
        // with its cursor on a line of the current buffer, and each step
        // leaves it on one.
        if dec_cursor() == -1 {
            return Err(Failed); // started at the start of the file
        }
        'finished: {
            if !stop || sclass == cls() || sclass == 0 {
                // Skip the white space before the word, stopping on an
                // empty line.
                while cls() == 0 {
                    // SAFETY: the cursor's line is a line of the buffer, and
                    // `ml_get` hands it back NUL-terminated.
                    if cur_win().w_cursor.col == 0
                        && unsafe { *ml_get(cur_win().w_cursor.lnum) } as c_int == NUL
                    {
                        break 'finished;
                    }
                    if dec_cursor() == -1 {
                        return Ok(()); // hit the start of the file
                    }
                }
                // Back to the start of this word.
                if unsafe { skip_chars(cls(), BACKWARD as c_int) } {
                    return Ok(());
                }
            }
            inc_cursor(); // overshot: forward one
        }
        stop = false;
    }
    // SAFETY: on the main thread with a current window.
    unsafe { adjust_skipcol() };
    Ok(())
}

/// `e` / `E`: move to the end of the `count`th word. Answers FAIL when the
/// end of the file was reached.
///
/// With `stop`, a cursor already on the end of a word moves one word less;
/// with `empty`, an empty line ends the motion.
///
/// (Real vi's `e` crosses a blank line and lands on the *first* character of
/// the next non-blank line, while `E` does not. That looks like a bug and is
/// not reproduced here -- upstream says so too.)
///
/// # Safety
/// There must be a current line and the cursor must be on it.
pub unsafe fn end_word(
    mut count: c_int,
    bigword: bool,
    mut stop: bool,
    empty: bool,
) -> Result<(), Failed> {
    cur_win().w_cursor.coladd = 0;
    cls_bigword.set(bigword);

    // Undo a cursor position adjusted for exclusive 'selection'.
    // SAFETY: 'selection' is a NUL-terminated option string.
    if unsafe { *p_sel.get() } as c_int == 'e' as c_int
        && visual_active()
        && visual_mode().is_char()
        && VIsual_select_exclu_adj.get()
    {
        // SAFETY: Visual mode is active, with a current window.
        unadjust_for_sel();
    }

    loop {
        count -= 1;
        if count < 0 {
            break;
        }
        // Inside a fold, move to the last character of the last line.
        if let Some(last) = cur_win().fold_end(cur_win().w_cursor.lnum) {
            cur_win().w_cursor.lnum = last;
            // SAFETY: `cur_win()` is a live window.
            coladvance(cur_win(), MAXCOL);
        }
        let sclass = cls();
        // SAFETY, for every step below: the caller guarantees a current window
        // with its cursor on a line of the current buffer, and each step
        // leaves it on one.
        if inc_cursor() == -1 {
            return Err(Failed);
        }
        'finished: {
            if cls() == sclass && sclass != 0 {
                // In the middle of a word: just go to its end.
                if unsafe { skip_chars(sclass, FORWARD as c_int) } {
                    return Err(Failed);
                }
            } else if !stop || sclass == 0 {
                // At the end of a word: go to the end of the next one,
                // skipping white space first.
                while cls() == 0 {
                    // SAFETY: the cursor's line is a line of the buffer, and
                    // `ml_get` hands it back NUL-terminated.
                    if empty
                        && cur_win().w_cursor.col == 0
                        && unsafe { *ml_get(cur_win().w_cursor.lnum) } as c_int == NUL
                    {
                        break 'finished;
                    }
                    if inc_cursor() == -1 {
                        return Err(Failed); // hit the end of the file
                    }
                }
                if unsafe { skip_chars(cls(), FORWARD as c_int) } {
                    return Err(Failed);
                }
            }
            dec_cursor(); // overshot: back one
        }
        stop = false; // only the first word moves one less
    }
    Ok(())
}

/// `ge` / `gE`: move back to the end of the `count`th previous word. Answers
/// FAIL when the start of the file was reached.
///
/// With `eol`, an end of line stops the motion.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
pub unsafe fn bckend_word(mut count: c_int, bigword: bool, eol: bool) -> Result<(), Failed> {
    cur_win().w_cursor.coladd = 0;
    cls_bigword.set(bigword);
    loop {
        count -= 1;
        if count < 0 {
            break;
        }
        let sclass = cls();
        // SAFETY, for every step below: the caller guarantees a current window
        // with its cursor on a line of the current buffer, and each step
        // leaves it on one.
        let mut i = dec_cursor();
        if i == -1 {
            return Err(Failed);
        }
        if eol && i == 1 {
            return Ok(());
        }
        // Back to before the start of this word.
        if sclass != 0 {
            while cls() == sclass {
                i = dec_cursor();
                if i == -1 || (eol && i == 1) {
                    return Ok(());
                }
            }
        }
        // Then back to the end of the previous word.
        while cls() == 0 {
            // SAFETY: the cursor's line is a line of the buffer, and `ml_get`
            // hands it back NUL-terminated.
            if cur_win().w_cursor.col == 0
                && unsafe { *ml_get(cur_win().w_cursor.lnum) } as c_int == NUL
            {
                break;
            }
            i = dec_cursor();
            if i == -1 || (eol && i == 1) {
                return Ok(());
            }
        }
    }
    // SAFETY: on the main thread with a current window.
    unsafe { adjust_skipcol() };
    Ok(())
}

/// `iw` / `aw` (and the `W` forms): the word under the cursor, cursor left at
/// its end.
///
/// Used with an operator pending and in Visual mode, where a `count` beyond
/// the first extends the selection by that many more objects -- in whichever
/// direction the cursor sits relative to the Visual start.
///
/// # Safety
/// `oap` must be a live operator argument, and there must be a current line.
pub unsafe fn current_word(
    oap: *mut oparg_T,
    mut count: c_int,
    include: bool,
    bigword: bool,
) -> Result<(), Failed> {
    let mut start_pos = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut inclusive = true;
    let mut include_white = false;

    cls_bigword.set(bigword);
    clearpos(&mut start_pos);

    // Correct the cursor when 'selection' is exclusive.
    // SAFETY: 'selection' is a NUL-terminated option string.
    if visual_active()
        && unsafe { *p_sel.get() } as c_int == 'e' as c_int
        && lt(visual_anchor(), cur_win().w_cursor)
    {
        // SAFETY: the caller guarantees the cursor is on a line of the buffer.
        dec_cursor();
    }

    // Outside Visual mode, or with a one-character Visual area, select
    // the word and/or white space under the cursor.
    if !visual_active() || equalpos(cur_win().w_cursor, visual_anchor()) {
        // SAFETY, for every walk in this function: the caller guarantees a
        // current window with its cursor on a line of the current buffer, and
        // each of these leaves it on one.
        unsafe { back_in_line() };
        start_pos = cur_win().w_cursor;

        // Starting on white space that is to be included (" word"), or
        // off white space that is not ("word"): find the end of the word.
        if (cls() == 0) == include {
            if unsafe { end_word(1, bigword, true, true) }.is_err() {
                return Err(Failed);
            }
        } else {
            // Starting off white space that is to be included
            // ("word   "), or on white space that is not ("   "): find
            // the start of the next word. Landing in the first column of
            // the next line (a single-character word) means backing up to
            // the end of this one.
            let _ = unsafe { fwd_word(1, bigword, true) };
            if cur_win().w_cursor.col == 0 {
                unsafe { decl(&mut cur_win().cursor()) };
            } else {
                let _ = unsafe { oneleft() };
            }
            if include {
                include_white = true;
            }
        }

        if visual_active() {
            // Should do something when `inclusive` is false.
            set_visual_anchor(start_pos);
            // SAFETY: on the main thread with a current buffer.
            redraw_curbuf_later(UPD_INVERTED); // update the inversion
        } else {
            // SAFETY: the caller guarantees `oap` is a live operator argument.
            let oap = unsafe { &mut *oap };
            oap.start = start_pos;
            oap.motion_type = kMTCharWise;
        }
        count -= 1;
    }

    // Any count still left extends by that many more objects.
    while count > 0 {
        inclusive = true;
        if visual_active() && lt(cur_win().w_cursor, visual_anchor()) {
            // In Visual mode with the cursor at the start: move it back.
            if unsafe { decl(&mut cur_win().cursor()) } == -1 {
                return Err(Failed);
            }
            if include != (cls() != 0) {
                if unsafe { bck_word(1, bigword, true) }.is_err() {
                    return Err(Failed);
                }
            } else {
                if unsafe { bckend_word(1, bigword, true) }.is_err() {
                    return Err(Failed);
                }
                unsafe { incl(&mut cur_win().cursor()) };
            }
        } else {
            // Move the cursor forward one word and/or run of white space.
            if unsafe { incl(&mut cur_win().cursor()) } == -1 {
                return Err(Failed);
            }
            if include != (cls() == 0) {
                if unsafe { fwd_word(1, bigword, true) }.is_err() && count > 1 {
                    return Err(Failed);
                }
                // An end just past a newline must not include the first
                // character of that line: put the cursor on the last
                // character of the white space instead.
                if unsafe { oneleft() }.is_err() {
                    inclusive = false;
                }
            } else if unsafe { end_word(1, bigword, true, true) }.is_err() {
                return Err(Failed);
            }
        }
        count -= 1;
    }

    if include_white && (cls() != 0 || (cur_win().w_cursor.col == 0 && !inclusive)) {
        // No white space was included at the end, so take some at the
        // start instead. That is what makes `daw` work on the last word
        // of a sentence (and `2daw` on the last but one), and what
        // handles `2daw` deleting `word.` at the end of a line, where the
        // cursor ends at the start of the next one. But never take the
        // white space at the start of a line: that is indent.
        let pos = cur_win().w_cursor;
        cur_win().w_cursor = start_pos;
        if unsafe { oneleft() }.is_ok() {
            unsafe { back_in_line() };
            if cls() == 0 && cur_win().w_cursor.col > 0 {
                if visual_active() {
                    set_visual_anchor(cur_win().w_cursor);
                } else {
                    // SAFETY: `oap` is a live operator argument.
                    unsafe { (*oap).start = cur_win().w_cursor };
                }
            }
        }
        cur_win().w_cursor = pos; // put the cursor back at the end
    }

    if visual_active() {
        // SAFETY: 'selection' is a NUL-terminated option string.
        if unsafe { *p_sel.get() } as c_int == 'e' as c_int
            && inclusive
            && ltoreq(visual_anchor(), cur_win().w_cursor)
        {
            // SAFETY: the cursor is on a line of the current buffer.
            inc_cursor();
        }
        if visual_mode().is_line() {
            set_visual_mode(VisualMode::CHAR);
            redraw_cmdline.set(true); // show the mode later
        }
    } else {
        // SAFETY: the caller guarantees `oap` is a live operator argument.
        unsafe { (*oap).inclusive = inclusive };
    }
    Ok(())
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
