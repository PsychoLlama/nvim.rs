//! The capture sets a thread carries: clearing, copying and comparing them.
//!
//! A `regsub_T` is `NSUBEXP` [`Capture`]s plus how many of them are in use.
//! A capture is a pair of [`MatchPos`]es, so the two shapes a match records
//! positions in — a line/column pair over buffer lines, a pointer over one
//! string — are the same sixteen bytes and the same code; only the handful of
//! places that have to *interpret* a position ask
//! [`Rex::pos_kind`](crate::regexp::Rex::pos_kind) which it is.
//!
//! Only the entries below `in_use` are ever read, which is why copying one
//! set over another leaves the rest alone.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::list::{op, out_of, out1_of};
use crate::regexp::NfaOp;
use core::ffi::c_int;

use crate::regexp::{Capture, NSUBEXP, PimResult, Rex, nfa_pim_T, nfa_state_T, regsub_T};

/// How far [`match_follows`] follows the machine before giving up.
const MATCH_FOLLOWS_DEPTH: c_int = 10;

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
    slots(rex.nfa_nsubexpr())
}

/// How many capture slots a `regsub_T`'s `in_use` names. Clamped, so that the
/// count can index the array: the compiler bounds a group number to nine, so
/// the clamp never fires on a program this engine built.
#[inline(always)]
pub(crate) fn slots(in_use: c_int) -> usize {
    usize::try_from(in_use).unwrap_or(0).min(NSUBEXP as usize)
}

/// Forget every capture in `sub`.
pub(crate) fn clear_sub(rex: Rex, sub: &mut regsub_T) {
    let n = nsubexpr(rex);
    sub.list[..n].fill(Capture::unset(rex.pos_kind()));
    sub.in_use = 0;
}

/// Copy the captures `from` has in use over `to`'s.
#[inline(always)]
pub(crate) fn copy_sub(to: &mut regsub_T, from: &regsub_T) {
    to.in_use = from.in_use;
    // Where a `:substitute` resumes scanning, which travels with the
    // whole-match capture. Only a buffer match ever sets it, and then it is
    // zero on both sides, so the copy needs no test of its own.
    to.orig_start_col = from.orig_start_col;
    // A plain loop rather than `copy_from_slice`: `in_use` is one or two in
    // almost every match, and at opt-level 0 the slice call chain costs more
    // than the copy. This runs once per thread put on a list.
    for i in 0..slots(from.in_use) {
        to.list[i] = from.list[i];
    }
}

/// [`copy_sub`] without group 0: a lookaround may report what its own groups
/// matched, but must not move the whole match's start or end.
pub(crate) fn copy_sub_off(to: &mut regsub_T, from: &regsub_T) {
    if to.in_use < from.in_use {
        to.in_use = from.in_use;
    }
    for i in 1..slots(from.in_use) {
        to.list[i] = from.list[i];
    }
}

/// Carry group 0's *end* over, which is the one thing a `\ze` inside a
/// lookaround is allowed to move.
pub(crate) fn copy_ze_off(rex: Rex, to: &mut regsub_T, from: &regsub_T) {
    let kind = rex.pos_kind();
    if rex.nfa_has_zend() != 0 && from.list[0].end.is_set(kind) {
        to.list[0].end = from.list[0].end;
    }
}

/// Do two capture sets describe the same positions?
///
/// A capture past a set's `in_use` counts as unset, so the comparison runs to
/// the longer of the two. Ends only count when the pattern has a
/// back-reference, which is the only thing that reads them mid-match.
pub(crate) fn sub_equal(rex: Rex, sub1: &regsub_T, sub2: &regsub_T) -> bool {
    let kind = rex.pos_kind();
    let ends_matter = has_backref(rex);
    let unset = Capture::unset(kind);
    let (n1, n2) = (slots(sub1.in_use), slots(sub2.in_use));
    // A plain index walk rather than zipped iterators: this is the match
    // loop's innermost comparison and the two sets are of different lengths.
    for i in 0..n1.max(n2) {
        let a = if i < n1 { sub1.list[i] } else { unset };
        let b = if i < n2 { sub2.list[i] } else { unset };
        if !a.start.same_capture(b.start, kind) {
            return false;
        }
        if ends_matter && !a.end.same_capture(b.end, kind) {
            return false;
        }
    }
    true
}

/// Copy a postponed lookaround, captures and all.
pub(crate) fn copy_pim(rex: Rex, to: &mut nfa_pim_T, from: &nfa_pim_T) {
    to.result = from.result;
    to.state = from.state;
    copy_sub(&mut to.subs.norm, &from.subs.norm);
    if has_zsubexpr(rex) {
        copy_sub(&mut to.subs.synt, &from.subs.synt);
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
        let code = op(state);
        match NfaOp::try_from(code) {
            Ok(
                NfaOp::Match
                | NfaOp::Mclose
                | NfaOp::EndInvisible
                | NfaOp::EndInvisibleNeg
                | NfaOp::EndPattern,
            ) => return true,
            Ok(NfaOp::Split) => {
                return match_follows(out_of(state), depth + 1)
                    || match_follows(out1_of(state), depth + 1);
            }
            // A lookaround or a grapheme group: what follows it hangs off
            // the end state `out1` points at.
            Ok(
                NfaOp::StartInvisible
                | NfaOp::StartInvisibleFirst
                | NfaOp::StartInvisibleNeg
                | NfaOp::StartInvisibleNegFirst
                | NfaOp::StartInvisibleBefore
                | NfaOp::StartInvisibleBeforeFirst
                | NfaOp::StartInvisibleBeforeNeg
                | NfaOp::StartInvisibleBeforeNegFirst
                | NfaOp::Composing,
            ) => {
                state = out_of(out1_of(state));
            }
            // Anything that consumes input means the pattern goes on.
            Ok(
                NfaOp::Any
                | NfaOp::AnyComposing
                | NfaOp::StartColl
                | NfaOp::StartNegColl
                | NfaOp::Newl,
            ) => return false,
            Ok(class) if class.is_class() => return false,
            _ if code > 0 => return false,
            _ => state = out_of(state),
        }
    }
    false
}
