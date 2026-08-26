//! Paragraphs and sections: the `{`/`}`/`[[`/`]]` motions and `ip`/`ap`.
//!
//! A paragraph boundary is an empty line, a form feed, or a line matching one
//! of the two-letter nroff macro lists in 'paragraphs'/'sections'.
//! [`starts_para`] is that test -- the rest of the tree asks it too -- and
//! [`findpar`] and [`current_par`] are the two shapes built on it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later, showmode};
use crate::main::{p_para, p_sections};
use crate::mark::setpcmark;
use crate::mbyte::utf_head_off;
use crate::memline::{ml_get, ml_get_len};
use crate::normal::{
    VisualMode, set_visual_anchor, set_visual_mode, visual_active, visual_anchor, visual_mode,
};
use crate::search::{BACKWARD, FORWARD, linewhite};
use crate::types::{FAIL, NUL, OK, linenr_T, oparg_T};

/// `{` / `}` / `[[` / `]]`: move to the `count`th paragraph or section
/// boundary in `dir`, answering whether one was found.
///
/// `what` is NUL for a paragraph and `{` or `}` for a section; `both` also
/// stops at a `}` in column 0. `pincl` is set when the last character of the
/// buffer is reached and has to be included in the motion.
///
/// # Safety
/// `pincl` must be writable, and there must be a current window.
pub unsafe fn findpar(
    pincl: *mut bool,
    dir: c_int,
    mut count: c_int,
    what: c_int,
    both: bool,
) -> bool {
    let mut curr = cur_win().w_cursor.lnum;

    loop {
        let this = count;
        count -= 1;
        if this == 0 {
            break;
        }
        // Set once the separating lines have been skipped: a boundary
        // only counts after at least one non-empty line.
        let mut did_skip = false;
        let mut first = true;
        loop {
            // SAFETY: on the main thread with a current buffer; `ml_get`
            // checks the line number itself and hands back a NUL-terminated
            // line, so its first byte is there to read.
            if unsafe { *ml_get(curr) } as c_int != NUL {
                did_skip = true;
            }
            // Skip over a closed fold, which counts as one line.
            let mut fold_skipped = false;
            if first {
                let (folded, fold_first, fold_last) = cur_win().fold_span(curr);
                if folded {
                    curr = (if dir > 0 { fold_last } else { fold_first }) + dir as linenr_T;
                    fold_skipped = true;
                }
            }
            if !first && did_skip && line_starts_para(curr, what, both) {
                break;
            }
            if fold_skipped {
                curr -= dir as linenr_T;
            }
            curr += dir as linenr_T;
            if curr < 1 || curr > cur_buf().b_ml.ml_line_count {
                if count != 0 {
                    return false;
                }
                curr -= dir as linenr_T;
                break;
            }
            first = false;
        }
    }

    // SAFETY: on the main thread with a current window.
    unsafe { setpcmark() };
    // SAFETY: as above -- `ml_get` hands back a NUL-terminated line.
    if both && unsafe { *ml_get(curr) } as c_int == '}' as c_int {
        curr += 1; // include the line holding the `}`
    }
    cur_win().w_cursor.lnum = curr;
    if curr == cur_buf().b_ml.ml_line_count && what != '}' as c_int && dir == FORWARD as c_int {
        // Put the cursor on the last character of the last line and make
        // the motion inclusive.
        // SAFETY: on the main thread with a current buffer; `ml_get` hands
        // back a NUL-terminated line and `ml_get_len` its length.
        let (line, len) = unsafe { (ml_get(curr), ml_get_len(curr)) };
        cur_win().w_cursor.col = len;
        if cur_win().w_cursor.col != 0 {
            cur_win().w_cursor.col -= 1;
            // SAFETY: `col` is now below `len`, so it indexes `line`, and
            // `utf_head_off` only walks back from there towards `line`.
            cur_win().w_cursor.col -=
                unsafe { utf_head_off(line, line.offset(cur_win().w_cursor.col as isize)) };
            // SAFETY: the caller guarantees `pincl` is writable.
            unsafe { *pincl = true };
        }
    } else {
        cur_win().w_cursor.col = 0;
    }
    true
}

/// Whether `s` opens with an nroff macro named in `opt` -- a list of
/// two-character names run together, as 'paragraphs' and 'sections' are.
///
/// A space in either position matches a space in the line or the line having
/// ended, which is how a one-letter macro is spelled.
///
/// # Safety
/// Both must be NUL-terminated.
unsafe fn inmacro(opt: *mut c_char, s: *const c_char) -> bool {
    let mut macro_name = opt;
    // SAFETY: both strings are NUL-terminated, and the walk only steps past
    // a byte it has just read as non-NUL, so it stops inside `opt`.  A
    // second byte of either is only reached once the first compared equal to
    // a byte that is not the other's NUL; those `&&` chains are the proof
    // and are left whole.
    unsafe {
        while *macro_name != 0 {
            if (*macro_name as c_int == *s as c_int
                || (*macro_name as c_int == ' ' as c_int
                    && (*s as c_int == NUL || *s as c_int == ' ' as c_int)))
                && (*macro_name.add(1) as c_int == *s.add(1) as c_int
                    || ((*macro_name.add(1) as c_int == NUL
                        || *macro_name.add(1) as c_int == ' ' as c_int)
                        && (*s as c_int == NUL
                            || *s.add(1) as c_int == NUL
                            || *s.add(1) as c_int == ' ' as c_int)))
            {
                break;
            }
            macro_name = macro_name.add(1);
            if *macro_name as c_int == NUL {
                break;
            }
            macro_name = macro_name.add(1);
        }
        *macro_name as c_int != NUL
    }
}

/// Whether line `lnum` starts a section or a paragraph.
///
/// `para` is `{` or `}` to ask about sections only; `both` also stops at a
/// `}` in column 0.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn starts_para(lnum: linenr_T, para: c_int, both: bool) -> bool {
    // SAFETY: on the main thread with a current buffer; `ml_get` checks the
    // line number itself and hands back a NUL-terminated line.
    let s = unsafe { ml_get(lnum) };
    // SAFETY: the line has at least its NUL, so its first byte is readable.
    let first = unsafe { *s };
    if first as u8 as c_int == para
        || first as c_int == '\u{c}' as c_int
        || (both && first as c_int == '}' as c_int)
    {
        return true;
    }
    // SAFETY: reached only with `s[0]` a `.`, so `s[1]` is still inside the
    // line, and 'sections'/'paragraphs' are NUL-terminated option strings.
    first as c_int == '.' as c_int
        && unsafe {
            inmacro(p_sections.get(), s.add(1)) || (para == 0 && inmacro(p_para.get(), s.add(1)))
        }
}

/// Grow an existing linewise Visual selection by `count` more paragraphs.
///
/// This is upstream's `extend:` label, reached both when the selection is
/// already more than one line and from the bottom of [`current_par`] when it
/// would otherwise get stuck -- `Vipipip` on a single white line.
///
/// Answers OK, or FAIL when the buffer ran out.
fn extend_paragraphs(mut start_lnum: linenr_T, count: c_int, include: bool) -> c_int {
    let mut retval = OK;
    let dir = if start_lnum < visual_anchor().lnum {
        BACKWARD as c_int
    } else {
        FORWARD as c_int
    };
    // The line the walk cannot pass, in whichever direction it runs.
    let limit = |dir: c_int| {
        if dir == BACKWARD as c_int {
            1
        } else {
            cur_buf().b_ml.ml_line_count
        }
    };
    let mut i = count;
    loop {
        i -= 1;
        if i < 0 {
            break;
        }
        if start_lnum == limit(dir) {
            retval = FAIL;
            break;
        }
        // Two passes when white space is included: one over the
        // paragraph, one over the blank lines beside it. A pass that
        // finds the same kind of line as the one before it has run out
        // of paragraph.
        let mut prev_start_is_white = -1;
        for _ in 0..2 {
            start_lnum += dir as linenr_T;
            let start_is_white = line_is_white(start_lnum) as c_int;
            if prev_start_is_white == start_is_white {
                start_lnum -= dir as linenr_T;
                break;
            }
            while start_lnum != limit(dir) {
                if start_is_white != line_is_white(start_lnum + dir as linenr_T) as c_int
                    || (start_is_white == 0
                        && line_starts_para(start_lnum + if dir > 0 { 1 } else { 0 }, 0, false))
                {
                    break;
                }
                start_lnum += dir as linenr_T;
            }
            if !include || start_lnum == limit(dir) {
                break;
            }
            prev_start_is_white = start_is_white;
        }
    }
    cur_win().w_cursor.lnum = start_lnum;
    cur_win().w_cursor.col = 0;
    retval
}

/// `ip` / `ap`: the paragraph under the cursor, linewise, cursor left on its
/// last line. In Visual mode an existing multi-line selection is extended
/// instead.
///
/// `type_0` is `p`; `S` (section) is not implemented upstream and answers
/// FAIL.
///
/// # Safety
/// `oap` must be a live operator argument, and there must be a current line.
pub unsafe fn current_par(oap: *mut oparg_T, count: c_int, include: bool, type_0: c_int) -> c_int {
    if type_0 == 'S' as c_int {
        return FAIL; // not implemented yet
    }
    let mut start_lnum = cur_win().w_cursor.lnum;

    // A Visual area of more than one line is extended, not replaced.
    if visual_active() && start_lnum != visual_anchor().lnum {
        return extend_paragraphs(start_lnum, count, include);
    }

    // Back to the start of the paragraph, or of the run of white lines.
    let white_in_front = line_is_white(start_lnum);
    while start_lnum > 1 {
        if white_in_front {
            if !line_is_white(start_lnum - 1) {
                break; // stop at the first white line
            }
        } else if line_is_white(start_lnum - 1) || line_starts_para(start_lnum, 0, false) {
            break; // stop at the paragraph's first line
        }
        start_lnum -= 1;
    }

    // Past the end of any white lines.
    let mut end_lnum = start_lnum;
    while end_lnum <= cur_buf().b_ml.ml_line_count && line_is_white(end_lnum) {
        end_lnum += 1;
    }
    end_lnum -= 1;

    let mut i = count;
    if !include && white_in_front {
        i -= 1;
    }
    // Whether the *next* run of white lines belongs to this object, which
    // for `ip` alternates: text, then blanks, then text.
    let mut do_white = false;
    loop {
        let this = i;
        i -= 1;
        if this == 0 {
            break;
        }
        if end_lnum == cur_buf().b_ml.ml_line_count {
            return FAIL;
        }
        if !include {
            do_white = line_is_white(end_lnum + 1);
        }
        if include || !do_white {
            end_lnum += 1;
            // On to the end of the paragraph.
            while end_lnum < cur_buf().b_ml.ml_line_count
                && !line_is_white(end_lnum + 1)
                && !line_starts_para(end_lnum + 1, 0, false)
            {
                end_lnum += 1;
            }
        }
        if i == 0 && white_in_front && include {
            break;
        }
        // On to the end of the white lines after the paragraph.
        if include || do_white {
            while end_lnum < cur_buf().b_ml.ml_line_count && line_is_white(end_lnum + 1) {
                end_lnum += 1;
            }
        }
    }

    // With no empty lines at the end, take some at the start instead --
    // unless that has been done already.
    if !white_in_front && !line_is_white(end_lnum) && include {
        while start_lnum > 1 && line_is_white(start_lnum - 1) {
            start_lnum -= 1;
        }
    }

    if visual_active() {
        // `Vipipip` on a single white line would otherwise get stuck
        // here, so hand it to the extending path instead.
        if visual_mode().is_line() && start_lnum == cur_win().w_cursor.lnum {
            return extend_paragraphs(start_lnum, count, include);
        }
        if visual_anchor().lnum != start_lnum {
            set_visual_anchor(visual_anchor().with_lnum(start_lnum).with_col(0));
        }
        set_visual_mode(VisualMode::LINE);
        // SAFETY: on the main thread with a current window and buffer.
        unsafe {
            redraw_curbuf_later(UPD_INVERTED); // update the inversion
            showmode();
        }
    } else {
        // SAFETY: the caller guarantees `oap` is a live operator argument.
        let oap = unsafe { &mut *oap };
        oap.start.lnum = start_lnum;
        oap.start.col = 0;
        oap.motion_type = kMTLineWise;
    }
    cur_win().w_cursor.lnum = end_lnum;
    cur_win().w_cursor.col = 0;
    OK
}

/// [`linewhite`] for a line of the current buffer.
fn line_is_white(lnum: linenr_T) -> bool {
    // SAFETY: on the main thread with a current buffer; `ml_get` checks the
    // line number itself, so any `lnum` is answered rather than read out of
    // bounds.
    unsafe { linewhite(lnum) }
}

/// [`starts_para`] for a line of the current buffer.
fn line_starts_para(lnum: linenr_T, para: c_int, both: bool) -> bool {
    // SAFETY: as above -- the line number is `ml_get`'s to check.
    unsafe { starts_para(lnum, para, both) }
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
