//! Paragraphs and sections: the `{`/`}`/`[[`/`]]` motions and `ip`/`ap`.
//!
//! A paragraph boundary is an empty line, a form feed, or a line matching one
//! of the two-letter nroff macro lists in 'paragraphs'/'sections'.
//! [`startPS`] is that test -- the rest of the tree asks it too -- and
//! [`findpar`] and [`current_par`] are the two shapes built on it.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::*;
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later, showmode};
use crate::fold::hasFolding;
use crate::main::{VIsual, VIsual_active, VIsual_mode, curbuf, curwin, p_para, p_sections};
use crate::mark::setpcmark;
use crate::mbyte::utf_head_off;
use crate::memline::{ml_get, ml_get_len};
use crate::search::{BACKWARD, FORWARD, linewhite};
use crate::types::{linenr_T, oparg_T};

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
    unsafe {
        let mut fold_first: linenr_T = 0;
        let mut fold_last: linenr_T = 0;
        let mut curr = (*curwin.get()).w_cursor.lnum;

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
                if *ml_get(curr) as c_int != NUL {
                    did_skip = true;
                }
                // Skip over a closed fold, which counts as one line.
                let mut fold_skipped = false;
                if first && hasFolding(curwin.get(), curr, &raw mut fold_first, &raw mut fold_last)
                {
                    curr = (if dir > 0 { fold_last } else { fold_first }) + dir as linenr_T;
                    fold_skipped = true;
                }
                if !first && did_skip && startPS(curr, what, both) {
                    break;
                }
                if fold_skipped {
                    curr -= dir as linenr_T;
                }
                curr += dir as linenr_T;
                if curr < 1 || curr > (*curbuf.get()).b_ml.ml_line_count {
                    if count != 0 {
                        return false;
                    }
                    curr -= dir as linenr_T;
                    break;
                }
                first = false;
            }
        }

        setpcmark();
        if both && *ml_get(curr) as c_int == '}' as c_int {
            curr += 1; // include the line holding the `}`
        }
        (*curwin.get()).w_cursor.lnum = curr;
        if curr == (*curbuf.get()).b_ml.ml_line_count
            && what != '}' as c_int
            && dir == FORWARD as c_int
        {
            // Put the cursor on the last character of the last line and make
            // the motion inclusive.
            let line = ml_get(curr);
            (*curwin.get()).w_cursor.col = ml_get_len(curr);
            if (*curwin.get()).w_cursor.col != 0 {
                (*curwin.get()).w_cursor.col -= 1;
                (*curwin.get()).w_cursor.col -=
                    utf_head_off(line, line.offset((*curwin.get()).w_cursor.col as isize));
                *pincl = true;
            }
        } else {
            (*curwin.get()).w_cursor.col = 0;
        }
        true
    }
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
    unsafe {
        let mut macro_name = opt;
        while *macro_name.add(0) != 0 {
            if (*macro_name.add(0) as c_int == *s.add(0) as c_int
                || (*macro_name.add(0) as c_int == ' ' as c_int
                    && (*s.add(0) as c_int == NUL || *s.add(0) as c_int == ' ' as c_int)))
                && (*macro_name.add(1) as c_int == *s.add(1) as c_int
                    || ((*macro_name.add(1) as c_int == NUL
                        || *macro_name.add(1) as c_int == ' ' as c_int)
                        && (*s.add(0) as c_int == NUL
                            || *s.add(1) as c_int == NUL
                            || *s.add(1) as c_int == ' ' as c_int)))
            {
                break;
            }
            macro_name = macro_name.add(1);
            if *macro_name.add(0) as c_int == NUL {
                break;
            }
            macro_name = macro_name.add(1);
        }
        *macro_name.add(0) as c_int != NUL
    }
}

/// Whether line `lnum` starts a section or a paragraph.
///
/// `para` is `{` or `}` to ask about sections only; `both` also stops at a
/// `}` in column 0.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn startPS(lnum: linenr_T, para: c_int, both: bool) -> bool {
    unsafe {
        let s = ml_get(lnum);
        if *s as u8 as c_int == para
            || *s as c_int == '\u{c}' as c_int
            || (both && *s as c_int == '}' as c_int)
        {
            return true;
        }
        *s as c_int == '.' as c_int
            && (inmacro(p_sections.get(), s.add(1))
                || (para == 0 && inmacro(p_para.get(), s.add(1))))
    }
}

/// Grow an existing linewise Visual selection by `count` more paragraphs.
///
/// This is upstream's `extend:` label, reached both when the selection is
/// already more than one line and from the bottom of [`current_par`] when it
/// would otherwise get stuck -- `Vipipip` on a single white line.
///
/// Answers OK, or FAIL when the buffer ran out.
///
/// # Safety
/// There must be a current buffer and Visual mode must be active.
unsafe fn extend_paragraphs(mut start_lnum: linenr_T, count: c_int, include: bool) -> c_int {
    unsafe {
        let mut retval = OK;
        let dir = if start_lnum < (*VIsual.ptr()).lnum {
            BACKWARD as c_int
        } else {
            FORWARD as c_int
        };
        // The line the walk cannot pass, in whichever direction it runs.
        let limit = |dir: c_int| {
            if dir == BACKWARD as c_int {
                1
            } else {
                (*curbuf.get()).b_ml.ml_line_count
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
                let start_is_white = linewhite(start_lnum) as c_int;
                if prev_start_is_white == start_is_white {
                    start_lnum -= dir as linenr_T;
                    break;
                }
                while start_lnum != limit(dir) {
                    if start_is_white != linewhite(start_lnum + dir as linenr_T) as c_int
                        || (start_is_white == 0
                            && startPS(start_lnum + if dir > 0 { 1 } else { 0 }, 0, false))
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
        (*curwin.get()).w_cursor.lnum = start_lnum;
        (*curwin.get()).w_cursor.col = 0;
        retval
    }
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
    unsafe {
        if type_0 == 'S' as c_int {
            return FAIL; // not implemented yet
        }
        let mut start_lnum = (*curwin.get()).w_cursor.lnum;

        // A Visual area of more than one line is extended, not replaced.
        if VIsual_active.get() && start_lnum != (*VIsual.ptr()).lnum {
            return extend_paragraphs(start_lnum, count, include);
        }

        // Back to the start of the paragraph, or of the run of white lines.
        let white_in_front = linewhite(start_lnum);
        while start_lnum > 1 {
            if white_in_front {
                if !linewhite(start_lnum - 1) {
                    break; // stop at the first white line
                }
            } else if linewhite(start_lnum - 1) || startPS(start_lnum, 0, false) {
                break; // stop at the paragraph's first line
            }
            start_lnum -= 1;
        }

        // Past the end of any white lines.
        let mut end_lnum = start_lnum;
        while end_lnum <= (*curbuf.get()).b_ml.ml_line_count && linewhite(end_lnum) {
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
            if end_lnum == (*curbuf.get()).b_ml.ml_line_count {
                return FAIL;
            }
            if !include {
                do_white = linewhite(end_lnum + 1);
            }
            if include || !do_white {
                end_lnum += 1;
                // On to the end of the paragraph.
                while end_lnum < (*curbuf.get()).b_ml.ml_line_count
                    && !linewhite(end_lnum + 1)
                    && !startPS(end_lnum + 1, 0, false)
                {
                    end_lnum += 1;
                }
            }
            if i == 0 && white_in_front && include {
                break;
            }
            // On to the end of the white lines after the paragraph.
            if include || do_white {
                while end_lnum < (*curbuf.get()).b_ml.ml_line_count && linewhite(end_lnum + 1) {
                    end_lnum += 1;
                }
            }
        }

        // With no empty lines at the end, take some at the start instead --
        // unless that has been done already.
        if !white_in_front && !linewhite(end_lnum) && include {
            while start_lnum > 1 && linewhite(start_lnum - 1) {
                start_lnum -= 1;
            }
        }

        if VIsual_active.get() {
            // `Vipipip` on a single white line would otherwise get stuck
            // here, so hand it to the extending path instead.
            if VIsual_mode.get() == 'V' as c_int && start_lnum == (*curwin.get()).w_cursor.lnum {
                return extend_paragraphs(start_lnum, count, include);
            }
            if (*VIsual.ptr()).lnum != start_lnum {
                (*VIsual.ptr()).lnum = start_lnum;
                (*VIsual.ptr()).col = 0;
            }
            VIsual_mode.set('V' as c_int);
            redraw_curbuf_later(UPD_INVERTED); // update the inversion
            showmode();
        } else {
            (*oap).start.lnum = start_lnum;
            (*oap).start.col = 0;
            (*oap).motion_type = kMTLineWise;
        }
        (*curwin.get()).w_cursor.lnum = end_lnum;
        (*curwin.get()).w_cursor.col = 0;
        OK
    }
}
