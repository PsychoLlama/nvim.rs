//! The position assertions: `\%23l`, `\%23c`, `\%23v`, `\%'m`, `\%#` and
//! `\%V`, each in its bare, `\%<` and `\%>` forms.
//!
//! None of them consume input; they only say whether the position the match
//! has reached is the one asked for.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::run::nfa_re_num_cmp;
use crate::src::nvim::main::curwin;
use crate::src::nvim::mark::mark_get;
use crate::src::nvim::plines::win_linetabsize;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::regexp::{
    MB_MAXBYTES, NFA_COL, NFA_LNUM, NFA_MARK, NFA_MARK_GT, NFA_MARK_LT, NFA_VCOL, kMarkBufLocal,
    nfa_state_T, reg_getline, reg_getline_len, reg_match_visual, rex,
};
use crate::src::nvim::types::{colnr_T, fmark_T, linenr_T, uint8_t, win_T};

/// The column the match has reached, in bytes from the start of the line.
fn col() -> colnr_T {
    // SAFETY: `input` and `line` bound the same line.
    unsafe { (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T }
}

/// The buffer line the match has reached.
fn lnum() -> linenr_T {
    // SAFETY: reads the match context.
    unsafe { (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum }
}

/// The window the match runs in, for the assertions that need one.
fn window() -> *mut win_T {
    // SAFETY: reads the match context; `curwin` is the current window.
    unsafe {
        match (*rex.ptr()).reg_win {
            w if w.is_null() => curwin.get(),
            w => w,
        }
    }
}

/// `\%23l`: the line number, counted in the buffer rather than in the match.
///
/// Only a buffer match has line numbers, so a string match never satisfies
/// it.
pub(crate) fn at_line(state: *mut nfa_state_T) -> bool {
    // SAFETY: `state` is a live state of the running program.
    unsafe {
        let want = (*state).val;
        assert!(
            want >= 0 && (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum >= 0,
            "line assertion out of range"
        );
        (*rex.ptr()).reg_match.is_null()
            && nfa_re_num_cmp(want as u64, (*state).c - NFA_LNUM, lnum() as u64)
    }
}

/// `\%23c`: the byte column, counted from one.
pub(crate) fn at_col(state: *mut nfa_state_T) -> bool {
    // SAFETY: as `at_line`.
    unsafe {
        assert!((*state).val >= 0, "column assertion out of range");
        assert!(
            (*rex.ptr()).input >= (*rex.ptr()).line,
            "input before the line"
        );
        nfa_re_num_cmp((*state).val as u64, (*state).c - NFA_COL, col() as u64 + 1)
    }
}

/// `\%23v`: the virtual column, counted from one — what the character looks
/// like it is at once tabs are expanded.
pub(crate) fn at_vcol(state: *mut nfa_state_T) -> bool {
    // SAFETY: as `at_line`; `reg_getline` re-reads the line because
    // `win_linetabsize` can move the memline's buffer.
    unsafe {
        let op = (*state).c - NFA_VCOL;
        let want = (*state).val;
        let col = col();
        // A virtual column is never smaller than the byte column divided by
        // the widest a character can be, so a `\%<` can be answered without
        // measuring anything.
        if op != 1 && col > want * MB_MAXBYTES as c_int {
            return false;
        }
        let wp = window();
        // Likewise for `\%>`, but the bound is the tab width: no character
        // expands to more columns than one tab does.
        if op == 1 && col - 1 > want && col > 100 {
            let ts = ((*(*wp).w_buffer).b_p_ts).max(4);
            if col as i64 > want as i64 * ts {
                return true;
            }
        }
        let mut lnum = if (*rex.ptr()).reg_match.is_null() {
            lnum()
        } else {
            1
        };
        if (*rex.ptr()).reg_match.is_null()
            && (lnum <= 0 || lnum > (*(*wp).w_buffer).b_ml.ml_line_count)
        {
            lnum = 1;
        }
        let vcol = win_linetabsize(wp, lnum, (*rex.ptr()).line as *mut c_char, col);
        assert!(want >= 0, "virtual column assertion out of range");
        nfa_re_num_cmp(want as u64, op, vcol as u64 + 1)
    }
}

/// `\%'m`: the position of mark `m`.
pub(crate) fn at_mark(state: *mut nfa_state_T) -> bool {
    // SAFETY: reads the match context and the buffer's marks.
    unsafe {
        let col = if (*rex.ptr()).reg_match.is_null() {
            col()
        } else {
            0
        };
        let fm: *mut fmark_T = mark_get(
            (*rex.ptr()).reg_buf,
            curwin.get(),
            core::ptr::null_mut(),
            kMarkBufLocal,
            (*state).val,
        );
        // Looking a mark up can move the memline's buffer out from under
        // the match.
        if (*rex.ptr()).reg_match.is_null() {
            (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
            (*rex.ptr()).input = (*rex.ptr()).line.offset(col as isize);
        }
        if fm.is_null() || (*fm).mark.lnum <= 0 {
            return false;
        }
        let pos = (*fm).mark;
        let here = lnum();
        // A mark parked at MAXCOL means the end of its line.
        let pos_col = if pos.lnum == here && pos.col == MAXCOL as c_int {
            reg_getline_len(pos.lnum - (*rex.ptr()).reg_firstlnum)
        } else {
            pos.col
        };
        let want = (*state).c;
        if pos.lnum == here {
            if pos_col == col {
                want == NFA_MARK
            } else if pos_col < col {
                want == NFA_MARK_GT
            } else {
                want == NFA_MARK_LT
            }
        } else if pos.lnum < here {
            want == NFA_MARK_GT
        } else {
            want == NFA_MARK_LT
        }
    }
}

/// `\%#`: the cursor's own position.
pub(crate) fn at_cursor() -> bool {
    // SAFETY: reads the match context and its window.
    unsafe {
        !(*rex.ptr()).reg_win.is_null()
            && lnum() == (*(*rex.ptr()).reg_win).w_cursor.lnum
            && col() == (*(*rex.ptr()).reg_win).w_cursor.col
    }
}

/// `\%V`: inside the Visual area.
pub(crate) fn in_visual() -> bool {
    reg_match_visual()
}
