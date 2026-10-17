//! The engine's entry point, and the loop over start columns below it.
//!
//! `bt_regexec_both` is where the shortcuts live — a `regmust` string the
//! line has to contain somewhere, an anchored pattern that only tries column
//! 0, and a known first character to skip to — and `regtry` is one attempt at
//! one column.
//!
//! The program is reached through [`BtProg`] and the match through [`Rex`],
//! which the API layer set up for this one match.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::compile::BtProg;
use super::matcher::regmatch;
use super::state::{BT_STATE, BtState};
use crate::getchar::state::got_int;
use crate::mbyte::{mb_tolower, utf_fold};
use crate::message::{e_null, e_re_corr};
use crate::message::{emsg, iemsg};
use crate::os::cshim::gettext;
use crate::profile::profile_passed_limit;
use crate::regexp::ExtMatch;
use crate::regexp::ExtMatchRef;
use crate::regexp::{
    BtRegProg, MatchPos, NSUBEXP, REGMAGIC, RF_ICASE, RF_ICOMBINE, RF_NOICASE, Rex, char_at_nul,
    cleanup_subexpr, cleanup_zsubexpr, cluster_len_at, cstrchr, cstrncmp, reg_getline,
};
use crate::types::{ColNr, LPos, NUL, uint32_t};

/// How many start columns may be tried between two reads of the caller's
/// time limit.
const TIME_CHECK_INTERVAL: c_int = 20;

/// `line` from byte `col` on. A column past the end — which no caller passes
/// — reads as the end of the line rather than past it.
#[inline(always)]
fn rest_from(line: &[u8], col: ColNr) -> &[u8] {
    usize::try_from(col)
        .ok()
        .and_then(|at| line.get(at..))
        .unwrap_or(&line[line.len() - 1..])
}

/// Try to match the whole pattern starting at column `col`.
///
/// Returns 0 for no match, or one more than the line the match ended on.
fn regtry(rex: Rex, prog: BtProg, state: &mut BtState, col: ColNr) -> c_int {
    rex.set_col(col);
    rex.set_need_clear_subexpr(true);
    rex.set_need_clear_zsubexpr(prog.has_z());

    if !regmatch(rex, prog, state, prog.first_node()) {
        return 0;
    }

    cleanup_subexpr(rex);
    settle_group_zero(rex, col);

    // The `\z(` captures go to the syntax highlighter as fresh copies,
    // because it keeps them past the end of this match.
    let captures = prog.has_z().then(|| {
        cleanup_zsubexpr(rex);
        ExtMatchRef::new(z_captures(rex))
    });
    rex.put_ext_out(captures);
    1 + rex.lnum() as c_int
}

/// Fill in whatever group 0 does not already say.
///
/// A `\zs` before the start, or a `\ze` before the end, can leave it unset;
/// it then covers what the matcher actually walked. A buffer match counts
/// its lines from the one it started on, so the start is on line zero.
fn settle_group_zero(rex: Rex, col: ColNr) {
    let kind = rex.pos_kind();
    let (start, end) = (rex.start(0), rex.end(0));
    if !start.get().is_set(kind) {
        start.set(MatchPos::from_pos(LPos { lnum: 0, col }));
    }
    if !end.get().is_set(kind) {
        end.set(rex.here());
    } else if rex.multi() {
        // The matcher may have walked past the line the match ends on.
        rex.set_lnum(end.get().as_pos().lnum);
    }
}

/// Copy what the `\z(` groups matched into the set the highlighter reads.
///
/// The match context must still be the one that filled the `\z` slots.
fn z_captures(rex: Rex) -> ExtMatch {
    let mut captures = ExtMatch::default();
    for i in 0..NSUBEXP as usize {
        let (start, end) = (rex.zstart(i).get().as_pos(), rex.zend(i).get().as_pos());
        let text = if rex.multi() {
            // A capture that spans lines cannot be handed over as one
            // string, so it is dropped.
            if start.lnum < 0 || end.lnum != start.lnum || end.col < start.col {
                continue;
            }
            let Some(line) = reg_getline(rex, start.lnum) else {
                continue;
            };
            span(line, start.col, end.col)
        } else {
            if start.lnum < 0 || end.lnum < 0 {
                continue;
            }
            span(rex.line(), start.col, end.col)
        };
        captures.set(i, text);
    }
    captures
}

/// The bytes of `line` from column `start` up to column `end`. An end before
/// the start was a negative length upstream, which copied to the NUL.
fn span(line: &[u8], start: ColNr, end: ColNr) -> &[u8] {
    let text = rest_from(line, start);
    match usize::try_from(end - start) {
        Ok(len) => &text[..len.min(text.len())],
        Err(_) => text,
    }
}

/// Run the backtracking program `prog` over the match `rex` is set up for,
/// starting at column `col`. Returns 0 for no match, or one more than the
/// line the match ended on.
pub(crate) fn bt_regexec(rex: Rex, prog: &BtRegProg, col: ColNr) -> c_int {
    // The working state is the cell's own between matches; this match takes
    // it for its whole run and hands it back, trimmed, when it is done.
    let mut state = BT_STATE.take();
    let retval = bt_regexec_both(rex, BtProg::new(prog), &mut state, col);
    state.trim();
    BT_STATE.set(state);
    retval
}

/// Match `rex`'s program against its first line, starting at `startcol`.
fn bt_regexec_both(rex: Rex, prog: BtProg, state: &mut BtState, startcol: ColNr) -> c_int {
    let mut col = startcol;
    // A buffer match starts on its line 0; a string match is already on its
    // string.
    let line = if rex.multi() {
        reg_getline(rex, 0)
    } else {
        Some(rex.line())
    };

    let mut retval = 0;
    match line {
        Some(line) => {
            if !magic_wrong(prog) && !(rex.reg_maxcol() > 0 && col >= rex.reg_maxcol()) {
                // The pattern's own `\c`/`\C`/`\Z` override what the caller
                // asked for.
                if prog.regflags() & RF_ICASE as u32 != 0 {
                    rex.set_reg_ic(true);
                } else if prog.regflags() & RF_NOICASE as u32 != 0 {
                    rex.set_reg_ic(false);
                }
                if prog.regflags() & RF_ICOMBINE as u32 != 0 {
                    rex.set_reg_icombine(true);
                }

                // A pattern with a literal run in it cannot match a line that
                // does not hold that run anywhere past `col`.
                if prog.regmust().is_none() || has_regmust(rex, prog, line, col) {
                    rex.set_line(line);
                    rex.set_lnum(0);
                    retval = if prog.is_anchored() {
                        try_anchored(rex, prog, state, col)
                    } else {
                        scan_columns(rex, prog, state, &mut col)
                    };
                }
            }
        }
        None => iemsg(gettext(e_null)),
    }

    if retval > 0 {
        clamp_group_zero(rex);
        rex.set_matchcol(col);
    }
    retval
}

/// Has the program's magic number not survived? Reports so when it has not.
fn magic_wrong(prog: BtProg<'_>) -> bool {
    if prog.magic() != REGMAGIC {
        emsg(gettext(e_re_corr));
        return true;
    }
    false
}

/// A `\ze` can put the end before the start; report an empty match rather
/// than a backwards one.
fn clamp_group_zero(rex: Rex) {
    let kind = rex.pos_kind();
    let (start, end) = (rex.start(0).get(), rex.end(0));
    if end.get().is_before(start, kind) {
        end.set(start);
    }
}

/// Does `line` hold the pattern's `regmust` run at or after `col`?
fn has_regmust(rex: Rex, prog: BtProg, line: &[u8], col: ColNr) -> bool {
    let Some(run) = prog.regmust() else {
        return true;
    };
    let c = char_at_nul(run, 0);
    let mut at = line.len() - rest_from(line, col).len();
    let text = &line[..line.len() - 1];
    loop {
        // Case-insensitively, the first character has to be looked for
        // with folding, which is what `cstrchr` does over a plain search.
        match cstrchr(rex, &text[at.min(text.len())..], c) {
            Some(off) => at += off,
            None => return false,
        }
        // `cstrncmp` reports back how much it compared, and upstream let
        // it write straight into the program's own `regmlen`.
        let mut mlen = prog.regmlen();
        let same = cstrncmp(rex, &line[at..], run, &mut mlen) == 0;
        prog.set_regmlen(mlen);
        if same {
            return true;
        }
        at += cluster_len_at(line, at);
    }
}

/// An anchored pattern can only match at `col`, and only if the character
/// there is the one it starts with.
fn try_anchored(rex: Rex, prog: BtProg, state: &mut BtState, col: ColNr) -> c_int {
    let start = prog.regstart();
    let c = char_at_nul(rest_from(rex.line(), col), 0);
    let matches = start == NUL
        || start == c
        || (rex.reg_ic()
            && (utf_fold(start) == utf_fold(c)
                // Latin-1 has case pairs that do not fold together.
                || (c < 255 && start < 255 && mb_tolower(start) == mb_tolower(c))));
    if matches {
        regtry(rex, prog, state, col)
    } else {
        0
    }
}

/// Try every column from `col` on, until one matches or the line runs out.
///
/// `col` comes back holding the column the reported match starts at.
fn scan_columns(rex: Rex, prog: BtProg, state: &mut BtState, col: &mut ColNr) -> c_int {
    let mut tm_count = 0;
    while !got_int.get() {
        // The pattern's first character is known: skip straight to the
        // next place it occurs.
        if prog.regstart() != NUL {
            let from = usize::try_from(*col).unwrap_or(usize::MAX);
            match cstrchr(rex, rex.text_from(from), prog.regstart()) {
                Some(off) => *col += off as ColNr,
                None => return 0,
            }
        }
        if rex.reg_maxcol() > 0 && *col >= rex.reg_maxcol() {
            return 0;
        }

        let retval = regtry(rex, prog, state, *col);
        if retval > 0 {
            return retval;
        }

        // A failed attempt may have walked onto a later line.
        if rex.lnum() != 0 {
            rex.set_lnum(0);
            rex.set_line(reg_getline(rex, 0).unwrap_or(b"\0"));
        }
        let rest = rest_from(rex.line(), *col);
        if rest[0] == NUL as u8 {
            return 0;
        }
        *col += cluster_len_at(rest, 0) as ColNr;

        if let Some(deadline) = rex.deadline() {
            tm_count += 1;
            if tm_count == TIME_CHECK_INTERVAL {
                tm_count = 0;
                if profile_passed_limit(deadline) {
                    rex.set_timed_out();
                    return 0;
                }
            }
        }
    }
    0
}

/// The `\%23l`-family comparison: node `scan`'s operand against `val`, with
/// the `>`/`<`/`=` the pattern spelled out riding in the node's eighth byte.
pub(crate) fn re_num_cmp(prog: BtProg, val: uint32_t, scan: usize) -> bool {
    let n = prog.operand_u32(scan, 3) as u32;
    match prog.byte(scan, 7) {
        b'>' => val > n,
        b'<' => val < n,
        _ => val == n,
    }
}
