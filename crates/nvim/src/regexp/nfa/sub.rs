//! The capture sets a thread carries: clearing, copying and comparing them.
//!
//! A `regsub_T` is `NSUBEXP` capture positions plus how many of them are in
//! use, in one of two shapes — a line/column pair per capture for a match
//! over buffer lines, a pair of pointers for a match over one string. Which
//! one is live is `rex.reg_match` being null and nothing else, so every
//! function here starts by asking.
//!
//! Only the entries below `in_use` are ever read, which is why copying one
//! set over another leaves the rest alone.

#![deny(unsafe_op_in_unsafe_fn)]

use super::list::{op, out_of, out1_of};
use core::ffi::c_int;

use crate::regexp::{
    NFA_ANY, NFA_ANY_COMPOSING, NFA_COMPOSING, NFA_END_INVISIBLE, NFA_END_INVISIBLE_NEG,
    NFA_END_PATTERN, NFA_IDENT, NFA_MATCH, NFA_MCLOSE, NFA_NEWL, NFA_NUPPER_IC, NFA_SPLIT,
    NFA_START_COLL, NFA_START_INVISIBLE, NFA_START_INVISIBLE_BEFORE_NEG_FIRST, NFA_START_NEG_COLL,
    NSUBEXP, PimResult, Rex, linepos, multipos, nfa_pim_T, nfa_state_T, regsub_T,
};

/// How far [`match_follows`] follows the machine before giving up.
const MATCH_FOLLOWS_DEPTH: c_int = 10;

/// An unset multi-line capture position — what the C original memset `0xff`
/// over, which is `-1` in each of the four fields.
const NO_MULTIPOS: multipos = multipos {
    start_lnum: -1,
    end_lnum: -1,
    start_col: -1,
    end_col: -1,
};

/// An unset string capture position.
const NO_LINEPOS: linepos = linepos {
    start: core::ptr::null_mut(),
    end: core::ptr::null_mut(),
};

/// Does this match run over a range of buffer lines rather than over one
/// string?
#[inline(always)]
pub(crate) fn multi_line(rex: Rex) -> bool {
    rex.multi()
}

/// Does the pattern have `\z(` groups? They are carried in a second capture
/// set beside the ordinary one, and only copied when there are any.
#[inline(always)]
pub(crate) fn has_zsubexpr(rex: Rex) -> bool {
    rex.nfa_has_zsubexpr() != 0
}

/// Does the pattern have a back-reference? It is the only thing that reads a
/// capture's end while the match is still running, and so the only thing that
/// can tell two threads on one state apart by their ends.
#[inline(always)]
pub(crate) fn has_backref(rex: Rex) -> bool {
    rex.nfa_has_backref() != 0
}

/// How many captures the pattern can fill.
fn nsubexpr(rex: Rex) -> usize {
    let n = rex.nfa_nsubexpr() as usize;
    n.min(NSUBEXP as usize)
}

/// Forget every capture in `sub`.
pub(crate) fn clear_sub(rex: Rex, sub: &mut regsub_T) {
    let n = nsubexpr(rex);
    // SAFETY: which arm of the union is live is `multi_line`, and the same
    // answer holds for every capture set of one match.
    unsafe {
        if multi_line(rex) {
            sub.list.multi[..n].fill(NO_MULTIPOS);
        } else {
            sub.list.line[..n].fill(NO_LINEPOS);
        }
    }
    sub.in_use = 0;
}

/// Copy the captures `from` has in use over `to`'s.
#[inline(always)]
pub(crate) fn copy_sub(rex: Rex, to: &mut regsub_T, from: &regsub_T) {
    to.in_use = from.in_use;
    if from.in_use <= 0 {
        return;
    }
    let n = from.in_use as usize;
    // A plain loop rather than `copy_from_slice`: `in_use` is one or two in
    // almost every match, and at opt-level 0 the slice call chain costs more
    // than the copy. This runs once per thread put on a list.
    // SAFETY: as `clear_sub`.
    unsafe {
        if multi_line(rex) {
            for i in 0..n {
                to.list.multi[i] = from.list.multi[i];
            }
            // Where a `:substitute` resumes scanning, which travels with the
            // whole-match capture.
            to.orig_start_col = from.orig_start_col;
        } else {
            for i in 0..n {
                to.list.line[i] = from.list.line[i];
            }
        }
    }
}

/// [`copy_sub`] without group 0: a lookaround may report what its own groups
/// matched, but must not move the whole match's start or end.
pub(crate) fn copy_sub_off(rex: Rex, to: &mut regsub_T, from: &regsub_T) {
    if to.in_use < from.in_use {
        to.in_use = from.in_use;
    }
    if from.in_use <= 1 {
        return;
    }
    let n = from.in_use as usize;
    // SAFETY: as `clear_sub`.
    unsafe {
        if multi_line(rex) {
            for i in 1..n {
                to.list.multi[i] = from.list.multi[i];
            }
        } else {
            for i in 1..n {
                to.list.line[i] = from.list.line[i];
            }
        }
    }
}

/// Carry group 0's *end* over, which is the one thing a `\ze` inside a
/// lookaround is allowed to move.
pub(crate) fn copy_ze_off(rex: Rex, to: &mut regsub_T, from: &regsub_T) {
    // SAFETY: as `clear_sub`; `nfa_has_zend` is read from the running match.
    unsafe {
        if rex.nfa_has_zend() == 0 {
            return;
        }
        if multi_line(rex) {
            if from.list.multi[0].end_lnum >= 0 {
                to.list.multi[0].end_lnum = from.list.multi[0].end_lnum;
                to.list.multi[0].end_col = from.list.multi[0].end_col;
            }
        } else if !from.list.line[0].end.is_null() {
            to.list.line[0].end = from.list.line[0].end;
        }
    }
}

/// Do two capture sets describe the same positions?
///
/// A capture past a set's `in_use` counts as unset, so the comparison runs to
/// the longer of the two. Ends only count when the pattern has a
/// back-reference, which is the only thing that reads them mid-match.
pub(crate) fn sub_equal(rex: Rex, sub1: &regsub_T, sub2: &regsub_T) -> bool {
    let ends_matter = has_backref(rex);
    let (n1, n2) = (sub1.in_use, sub2.in_use);
    let todo = n1.max(n2) as usize;
    // Written as a plain loop over the fields rather than over a helper that
    // returns "the position, or the unset one": this is the match loop's
    // innermost comparison and at opt-level 0 a closure per element is a
    // call per element.
    // SAFETY: as `clear_sub`.
    unsafe {
        if multi_line(rex) {
            for i in 0..todo {
                let a = if (i as c_int) < n1 {
                    sub1.list.multi[i]
                } else {
                    NO_MULTIPOS
                };
                let b = if (i as c_int) < n2 {
                    sub2.list.multi[i]
                } else {
                    NO_MULTIPOS
                };
                if a.start_lnum != b.start_lnum {
                    return false;
                }
                // A start that is set at all has a column worth comparing.
                if a.start_lnum != -1 && a.start_col != b.start_col {
                    return false;
                }
                if ends_matter {
                    if a.end_lnum != b.end_lnum {
                        return false;
                    }
                    if a.end_lnum != -1 && a.end_col != b.end_col {
                        return false;
                    }
                }
            }
        } else {
            for i in 0..todo {
                let a = if (i as c_int) < n1 {
                    sub1.list.line[i]
                } else {
                    NO_LINEPOS
                };
                let b = if (i as c_int) < n2 {
                    sub2.list.line[i]
                } else {
                    NO_LINEPOS
                };
                if a.start != b.start {
                    return false;
                }
                if ends_matter && a.end != b.end {
                    return false;
                }
            }
        }
    }
    true
}

/// Copy a postponed lookaround, captures and all.
pub(crate) fn copy_pim(rex: Rex, to: &mut nfa_pim_T, from: &nfa_pim_T) {
    to.result = from.result;
    to.state = from.state;
    copy_sub(rex, &mut to.subs.norm, &from.subs.norm);
    if has_zsubexpr(rex) {
        copy_sub(rex, &mut to.subs.synt, &from.subs.synt);
    }
    to.end = from.end;
}

/// Are two threads carrying the same postponed lookaround? A lookaround that
/// has already been decided, or none at all, counts as "no lookaround".
pub(crate) fn pim_equal(rex: Rex, one: Option<&nfa_pim_T>, two: Option<&nfa_pim_T>) -> bool {
    let unused = |p: Option<&nfa_pim_T>| p.is_none_or(|p| p.result == PimResult::Unused);
    let (Some(one), Some(two)) = (one, two) else {
        return unused(one) && unused(two);
    };
    if unused(Some(one)) {
        return unused(Some(two));
    }
    if unused(Some(two)) {
        return false;
    }
    // SAFETY: a pim in use names a live state of the running program.
    if unsafe { (*one.state).id != (*two.state).id } {
        return false;
    }
    one.end.same(two.end, rex.pos_kind())
}

/// Would the pattern be over if a thread reached `startstate`?
///
/// Used to decide whether a postponed lookaround has to be settled now: once
/// nothing but the match itself follows, there is nothing left to postpone
/// past. Only the states that consume no input are followed, and only
/// [`MATCH_FOLLOWS_DEPTH`] alternations deep.
pub(crate) fn match_follows(startstate: *const nfa_state_T, depth: c_int) -> bool {
    if depth > MATCH_FOLLOWS_DEPTH {
        return false;
    }
    // SAFETY: `startstate` and everything reachable from it are states of the
    // running program.

    let mut state = startstate;
    while !state.is_null() {
        match op(state) {
            NFA_MATCH
            | NFA_MCLOSE
            | NFA_END_INVISIBLE
            | NFA_END_INVISIBLE_NEG
            | NFA_END_PATTERN => return true,
            NFA_SPLIT => {
                return match_follows(out_of(state), depth + 1)
                    || match_follows(out1_of(state), depth + 1);
            }
            // A lookaround or a grapheme group: what follows it hangs off
            // the end state `out1` points at.
            NFA_START_INVISIBLE..=NFA_START_INVISIBLE_BEFORE_NEG_FIRST | NFA_COMPOSING => {
                state = out_of(out1_of(state));
            }
            // Anything that consumes input means the pattern goes on.
            NFA_ANY
            | NFA_ANY_COMPOSING
            | NFA_IDENT..=NFA_NUPPER_IC
            | NFA_START_COLL
            | NFA_START_NEG_COLL
            | NFA_NEWL => return false,
            c if c > 0 => return false,
            _ => state = out_of(state),
        }
    }
    false
}
