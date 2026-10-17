//! The position assertions: `\%23l`, `\%23c`, `\%23v`, `\%'m`, `\%#` and
//! `\%V`, each in its bare, `\%<` and `\%>` forms.
//!
//! None of them consume input; they only say whether the position the match
//! has reached is the one asked for.

#![forbid(unsafe_code)]

use super::program::NfaState;
use crate::regexp::NfaOp;
use core::ffi::c_int;

use super::run::nfa_re_num_cmp;
use crate::mark::mark_lookup;
use crate::plines::win_linetabsize_bytes;
use crate::pos::MAXCOL;
use crate::regexp::{Rex, kMarkBufLocal, reg_getline, reg_getline_len, reg_match_visual};
use crate::types::{ColNr, LineNr, MB_MAXBYTES};

use crate::winlayer::Win;
/// The column the match has reached, in bytes from the start of the line.
fn col(rex: Rex) -> ColNr {
    rex.col()
}

/// The buffer line the match has reached.
fn lnum(rex: Rex) -> LineNr {
    rex.buf_lnum()
}

/// The window the match runs in, for the assertions that need one.
fn window(rex: Rex) -> Win {
    rex.reg_win().unwrap_or_else(Win::current)
}

/// `\%23l`: the line number, counted in the buffer rather than in the match.
///
/// Only a buffer match has line numbers, so a string match never satisfies
/// it.
pub(crate) fn at_line(rex: Rex, state: &NfaState) -> bool {
    let want = state.val;
    assert!(
        want >= 0 && rex.buf_lnum() >= 0,
        "line assertion out of range"
    );
    rex.multi() && nfa_re_num_cmp(want as u64, state.c - NfaOp::Lnum.code(), lnum(rex) as u64)
}

/// `\%23c`: the byte column, counted from one.
pub(crate) fn at_col(rex: Rex, state: &NfaState) -> bool {
    debug_assert!(state.val >= 0, "column assertion out of range");
    nfa_re_num_cmp(
        state.val as u64,
        state.c - NfaOp::Col.code(),
        col(rex) as u64 + 1,
    )
}

/// `\%23v`: the virtual column, counted from one — what the character looks
/// like it is at once tabs are expanded.
pub(crate) fn at_vcol(rex: Rex, state: &NfaState) -> bool {
    let op = state.c - NfaOp::Vcol.code();
    let want = state.val;
    let col = col(rex);
    // A virtual column is never smaller than the byte column divided by
    // the widest a character can be, so a `\%<` can be answered without
    // measuring anything.
    if op != 1 && col > want * MB_MAXBYTES as c_int {
        return false;
    }
    let wp = window(rex);
    // Likewise for `\%>`, but the bound is the tab width: no character
    // expands to more columns than one tab does.
    if op == 1 && col - 1 > want && col > 100 {
        let ts = wp.buffer().b_p_ts.max(4);
        if col as i64 > want as i64 * ts {
            return true;
        }
    }
    let mut lnum = if rex.multi() { lnum(rex) } else { 1 };
    if rex.multi() && (lnum <= 0 || lnum > wp.buffer().b_ml.ml_line_count) {
        lnum = 1;
    }
    let vcol = win_linetabsize_bytes(wp, lnum, rex.line(), col);
    assert!(want >= 0, "virtual column assertion out of range");
    nfa_re_num_cmp(want as u64, op, vcol as u64 + 1)
}

/// `\%'m`: the position of mark `m`.
pub(crate) fn at_mark(rex: Rex, state: &NfaState) -> bool {
    let col = if rex.multi() { col(rex) } else { 0 };
    let name = state.val;
    // `mark_lookup` computes a motion mark (`'{`, `'(`) on the spot.
    let fm = mark_lookup(rex.reg_buf(), Win::current(), kMarkBufLocal, name);
    // Looking a mark up can move the memline's buffer out from under
    // the match.
    if rex.multi() {
        rex.seek(reg_getline(rex, rex.lnum()).unwrap_or(b"\0"), col);
    }
    let Some(fm) = fm.filter(|fm| fm.mark.lnum > 0) else {
        return false;
    };
    let pos = fm.mark;
    let here = lnum(rex);
    // A mark parked at MAXCOL means the end of its line.
    let pos_col = if pos.lnum == here && pos.col == MAXCOL as c_int {
        reg_getline_len(rex, pos.lnum - rex.reg_firstlnum())
    } else {
        pos.col
    };
    let want = state.c;
    if pos.lnum == here {
        if pos_col == col {
            want == NfaOp::Mark.code()
        } else if pos_col < col {
            want == NfaOp::MarkGt.code()
        } else {
            want == NfaOp::MarkLt.code()
        }
    } else if pos.lnum < here {
        want == NfaOp::MarkGt.code()
    } else {
        want == NfaOp::MarkLt.code()
    }
}

/// `\%#`: the cursor's own position.
pub(crate) fn at_cursor(rex: Rex) -> bool {
    rex.reg_win()
        .is_some_and(|w| lnum(rex) == w.w_cursor.lnum && col(rex) == w.w_cursor.col)
}

/// `\%V`: inside the Visual area.
pub(crate) fn in_visual(rex: Rex) -> bool {
    reg_match_visual(rex)
}
