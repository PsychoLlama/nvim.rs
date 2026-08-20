//! Word motions and the `iw`/`aw` objects.
//!
//! Every one of these is the same walk: step a character at a time and stop
//! when [`cls`] -- the character *class* under the cursor, folded to one
//! bucket when a WORD is asked for -- changes. `w`/`b`/`e`/`ge` are the four
//! directions of it, and [`current_word`] composes them.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;
use crate::cursor::{coladvance, dec_cursor, gchar_cursor, get_cursor_line_ptr, inc_cursor};
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::edit::oneleft;
use crate::fold::has_folding;
use crate::global_cell::GlobalCell;
use crate::main::{
    VIsual, VIsual_active, VIsual_mode, VIsual_select_exclu_adj, curbuf, curwin, p_sel,
    redraw_cmdline,
};
use crate::mbyte::utf_class;
use crate::memline::{decl, incl, ml_get};
use crate::r#move::adjust_skipcol;
use crate::normal::unadjust_for_sel;
use crate::pos::{MAXCOL, clearpos, equalpos, lt, ltoreq};
use crate::search::{BACKWARD, FORWARD};
use crate::types::{FAIL, NUL, OK, linenr_T, oparg_T, pos_T};

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
///
/// # Safety
/// There must be a current line and the cursor must be on it.
unsafe fn cls() -> c_int {
    unsafe {
        let c = gchar_cursor();
        if c == ' ' as c_int || c == '\t' as c_int || c == NUL {
            return 0;
        }
        let c = utf_class(c);
        if c != 0 && cls_bigword.get() { 1 } else { c }
    }
}

/// Step over a run of characters of class `cclass`. Answers true when the
/// end of the file was reached.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
unsafe fn skip_chars(cclass: c_int, dir: c_int) -> bool {
    unsafe {
        while cls() == cclass {
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
}

/// Go back to the start of the word, or of the run of white space, the
/// cursor is inside -- without leaving the line.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
unsafe fn back_in_line() {
    unsafe {
        let sclass = cls();
        while (*curwin.get()).w_cursor.col != 0 {
            dec_cursor();
            if cls() != sclass {
                inc_cursor(); // stop at the start of the word
                break;
            }
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
pub unsafe fn fwd_word(mut count: c_int, bigword: bool, eol: bool) -> c_int {
    unsafe {
        (*curwin.get()).w_cursor.coladd = 0;
        cls_bigword.set(bigword);
        loop {
            count -= 1;
            if count < 0 {
                break;
            }
            // Inside a fold, move to the last character of the last line.
            if has_folding(
                curwin.get(),
                (*curwin.get()).w_cursor.lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut (*curwin.get()).w_cursor.lnum,
            ) {
                coladvance(curwin.get(), MAXCOL);
            }
            let sclass = cls();

            // Always move at least one character, unless this is the last one
            // in the buffer.
            let last_line = (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_ml.ml_line_count;
            let mut i = inc_cursor();
            if i == -1 || (i >= 1 && last_line) {
                return FAIL; // started on the last character of the file
            }
            if i >= 1 && eol && count == 0 {
                return OK; // started on the last character of the line
            }

            // One character past the end of the current word, if any.
            if sclass != 0 {
                while cls() == sclass {
                    i = inc_cursor();
                    if i == -1 || (i >= 1 && eol && count == 0) {
                        return OK;
                    }
                }
            }
            // Then on to the next non-white character.
            while cls() == 0 {
                // Stop on a blank line.
                if (*curwin.get()).w_cursor.col == 0 && *get_cursor_line_ptr() as c_int == NUL {
                    break;
                }
                i = inc_cursor();
                if i == -1 || (i >= 1 && eol && count == 0) {
                    return OK;
                }
            }
        }
        OK
    }
}

/// `b` / `B`: move back `count` words. Answers FAIL when the top of the file
/// was reached.
///
/// With `stop`, a cursor already on the start of a word moves one word less,
/// which is what makes `cb` from mid-word do the right thing.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
pub unsafe fn bck_word(mut count: c_int, bigword: bool, mut stop: bool) -> c_int {
    unsafe {
        (*curwin.get()).w_cursor.coladd = 0;
        cls_bigword.set(bigword);
        loop {
            count -= 1;
            if count < 0 {
                break;
            }
            // Inside a fold, move to the first character of the first line.
            if has_folding(
                curwin.get(),
                (*curwin.get()).w_cursor.lnum,
                &raw mut (*curwin.get()).w_cursor.lnum,
                ::core::ptr::null_mut::<linenr_T>(),
            ) {
                (*curwin.get()).w_cursor.col = 0;
            }
            let sclass = cls();
            if dec_cursor() == -1 {
                return FAIL; // started at the start of the file
            }
            'finished: {
                if !stop || sclass == cls() || sclass == 0 {
                    // Skip the white space before the word, stopping on an
                    // empty line.
                    while cls() == 0 {
                        if (*curwin.get()).w_cursor.col == 0
                            && *ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL
                        {
                            break 'finished;
                        }
                        if dec_cursor() == -1 {
                            return OK; // hit the start of the file
                        }
                    }
                    // Back to the start of this word.
                    if skip_chars(cls(), BACKWARD as c_int) {
                        return OK;
                    }
                }
                inc_cursor(); // overshot: forward one
            }
            stop = false;
        }
        adjust_skipcol();
        OK
    }
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
pub unsafe fn end_word(mut count: c_int, bigword: bool, mut stop: bool, empty: bool) -> c_int {
    unsafe {
        (*curwin.get()).w_cursor.coladd = 0;
        cls_bigword.set(bigword);

        // Undo a cursor position adjusted for exclusive 'selection'.
        if *p_sel.get() as c_int == 'e' as c_int
            && VIsual_active.get()
            && VIsual_mode.get() == 'v' as c_int
            && VIsual_select_exclu_adj.get()
        {
            unadjust_for_sel();
        }

        loop {
            count -= 1;
            if count < 0 {
                break;
            }
            // Inside a fold, move to the last character of the last line.
            if has_folding(
                curwin.get(),
                (*curwin.get()).w_cursor.lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut (*curwin.get()).w_cursor.lnum,
            ) {
                coladvance(curwin.get(), MAXCOL);
            }
            let sclass = cls();
            if inc_cursor() == -1 {
                return FAIL;
            }
            'finished: {
                if cls() == sclass && sclass != 0 {
                    // In the middle of a word: just go to its end.
                    if skip_chars(sclass, FORWARD as c_int) {
                        return FAIL;
                    }
                } else if !stop || sclass == 0 {
                    // At the end of a word: go to the end of the next one,
                    // skipping white space first.
                    while cls() == 0 {
                        if empty
                            && (*curwin.get()).w_cursor.col == 0
                            && *ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL
                        {
                            break 'finished;
                        }
                        if inc_cursor() == -1 {
                            return FAIL; // hit the end of the file
                        }
                    }
                    if skip_chars(cls(), FORWARD as c_int) {
                        return FAIL;
                    }
                }
                dec_cursor(); // overshot: back one
            }
            stop = false; // only the first word moves one less
        }
        OK
    }
}

/// `ge` / `gE`: move back to the end of the `count`th previous word. Answers
/// FAIL when the start of the file was reached.
///
/// With `eol`, an end of line stops the motion.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
pub unsafe fn bckend_word(mut count: c_int, bigword: bool, eol: bool) -> c_int {
    unsafe {
        (*curwin.get()).w_cursor.coladd = 0;
        cls_bigword.set(bigword);
        loop {
            count -= 1;
            if count < 0 {
                break;
            }
            let sclass = cls();
            let mut i = dec_cursor();
            if i == -1 {
                return FAIL;
            }
            if eol && i == 1 {
                return OK;
            }
            // Back to before the start of this word.
            if sclass != 0 {
                while cls() == sclass {
                    i = dec_cursor();
                    if i == -1 || (eol && i == 1) {
                        return OK;
                    }
                }
            }
            // Then back to the end of the previous word.
            while cls() == 0 {
                if (*curwin.get()).w_cursor.col == 0
                    && *ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL
                {
                    break;
                }
                i = dec_cursor();
                if i == -1 || (eol && i == 1) {
                    return OK;
                }
            }
        }
        adjust_skipcol();
        OK
    }
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
) -> c_int {
    unsafe {
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
        if VIsual_active.get()
            && *p_sel.get() as c_int == 'e' as c_int
            && lt(VIsual.get(), (*curwin.get()).w_cursor)
        {
            dec_cursor();
        }

        // Outside Visual mode, or with a one-character Visual area, select
        // the word and/or white space under the cursor.
        if !VIsual_active.get() || equalpos((*curwin.get()).w_cursor, VIsual.get()) {
            back_in_line();
            start_pos = (*curwin.get()).w_cursor;

            // Starting on white space that is to be included (" word"), or
            // off white space that is not ("word"): find the end of the word.
            if (cls() == 0) == include {
                if end_word(1, bigword, true, true) == FAIL {
                    return FAIL;
                }
            } else {
                // Starting off white space that is to be included
                // ("word   "), or on white space that is not ("   "): find
                // the start of the next word. Landing in the first column of
                // the next line (a single-character word) means backing up to
                // the end of this one.
                fwd_word(1, bigword, true);
                if (*curwin.get()).w_cursor.col == 0 {
                    decl(&raw mut (*curwin.get()).w_cursor);
                } else {
                    oneleft();
                }
                if include {
                    include_white = true;
                }
            }

            if VIsual_active.get() {
                // Should do something when `inclusive` is false.
                VIsual.set(start_pos);
                redraw_curbuf_later(UPD_INVERTED); // update the inversion
            } else {
                (*oap).start = start_pos;
                (*oap).motion_type = kMTCharWise;
            }
            count -= 1;
        }

        // Any count still left extends by that many more objects.
        while count > 0 {
            inclusive = true;
            if VIsual_active.get() && lt((*curwin.get()).w_cursor, VIsual.get()) {
                // In Visual mode with the cursor at the start: move it back.
                if decl(&raw mut (*curwin.get()).w_cursor) == -1 {
                    return FAIL;
                }
                if include != (cls() != 0) {
                    if bck_word(1, bigword, true) == FAIL {
                        return FAIL;
                    }
                } else {
                    if bckend_word(1, bigword, true) == FAIL {
                        return FAIL;
                    }
                    incl(&raw mut (*curwin.get()).w_cursor);
                }
            } else {
                // Move the cursor forward one word and/or run of white space.
                if incl(&raw mut (*curwin.get()).w_cursor) == -1 {
                    return FAIL;
                }
                if include != (cls() == 0) {
                    if fwd_word(1, bigword, true) == FAIL && count > 1 {
                        return FAIL;
                    }
                    // An end just past a newline must not include the first
                    // character of that line: put the cursor on the last
                    // character of the white space instead.
                    if oneleft() == FAIL {
                        inclusive = false;
                    }
                } else if end_word(1, bigword, true, true) == FAIL {
                    return FAIL;
                }
            }
            count -= 1;
        }

        if include_white && (cls() != 0 || ((*curwin.get()).w_cursor.col == 0 && !inclusive)) {
            // No white space was included at the end, so take some at the
            // start instead. That is what makes `daw` work on the last word
            // of a sentence (and `2daw` on the last but one), and what
            // handles `2daw` deleting `word.` at the end of a line, where the
            // cursor ends at the start of the next one. But never take the
            // white space at the start of a line: that is indent.
            let pos = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = start_pos;
            if oneleft() == OK {
                back_in_line();
                if cls() == 0 && (*curwin.get()).w_cursor.col > 0 {
                    if VIsual_active.get() {
                        VIsual.set((*curwin.get()).w_cursor);
                    } else {
                        (*oap).start = (*curwin.get()).w_cursor;
                    }
                }
            }
            (*curwin.get()).w_cursor = pos; // put the cursor back at the end
        }

        if VIsual_active.get() {
            if *p_sel.get() as c_int == 'e' as c_int
                && inclusive
                && ltoreq(VIsual.get(), (*curwin.get()).w_cursor)
            {
                inc_cursor();
            }
            if VIsual_mode.get() == 'V' as c_int {
                VIsual_mode.set('v' as c_int);
                redraw_cmdline.set(true); // show the mode later
            }
        } else {
            (*oap).inclusive = inclusive;
        }
        OK
    }
}
