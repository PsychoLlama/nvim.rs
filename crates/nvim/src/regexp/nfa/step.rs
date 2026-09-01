//! One thread of the match loop: what the state it sits on says about the
//! character under the input, and where that leaves it.
//!
//! Everything here answers with a [`Step`]; the loop in
//! [`super::matcher`] is what acts on it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::regexp::NfaOp;
use core::ffi::c_int;

use super::assertions::{at_col, at_cursor, at_line, at_mark, at_vcol, in_visual};
use super::classes::class_matches;
use super::composing::matches_composing;
use super::list::{ThreadList, addstate_here, op, out_of, out1_of};
use super::run::{check_char_class, match_backref, match_zref, recursive_regmatch};
use super::sub::{copy_sub, copy_sub_off, copy_ze_off, has_zsubexpr};
use crate::mbyte::{mb_get_class_tab, utf_fold, utf_iscomposing_legacy};
use crate::regexp::{
    NFA_TOO_EXPENSIVE, PimResult, Rex, nfa_endp, nfa_match, nfa_pim_T, nfa_regprog_T, nfa_state_T,
    reg_prev_class, regsubs_T,
};
use crate::types::NUL;

/// Where a thread's state leaves it.
pub(crate) enum Step {
    /// Nothing further: the thread does not survive this character.
    Dead,
    /// Add `state` to the *current* list, at this position — the state
    /// consumes no input.
    Here(*mut nfa_state_T),
    /// Add `state` to the next list, `off` bytes on. `count` is how many
    /// more bytes it has still to consume before it may advance again,
    /// which is how a back-reference longer than one character waits.
    Next {
        state: *mut nfa_state_T,
        off: c_int,
        count: c_int,
    },
    /// The pattern matched here; stop scanning this list.
    Matched,
    /// Give up on the whole match.
    TooExpensive,
}

impl Step {
    fn next(state: *mut nfa_state_T, off: c_int) -> Step {
        Step::Next {
            state,
            off,
            count: 0,
        }
    }

    /// A test that either advances over the character or kills the thread.
    fn consuming(matched: bool, state: *mut nfa_state_T, clen: c_int) -> Step {
        if matched {
            Step::next(state, clen)
        } else {
            Step::Dead
        }
    }

    /// A test that either continues at this position or kills the thread.
    fn zero_width(matched: bool, state: *mut nfa_state_T) -> Step {
        if matched {
            Step::Here(state)
        } else {
            Step::Dead
        }
    }
}

/// The state of the match this step belongs to, for the arms that run a
/// sub-match of their own.
pub(crate) struct Run<'a> {
    pub(crate) prog: *mut nfa_regprog_T,
    pub(crate) submatch: *mut regsubs_T,
    pub(crate) m: *mut regsubs_T,
    pub(crate) listids: &'a mut Vec<c_int>,
    /// Scratch for the capture set `addstate_here` is handed, which may not
    /// be one that lives in the list it rewrites.
    pub(crate) here: &'a mut regsubs_T,
}

/// Is `state` one of the negated lookarounds, whose sub-match must *fail*
/// for the thread to survive?
fn is_negated(state: *mut nfa_state_T) -> bool {
    // SAFETY: `state` is a live state of the running program.

    matches!(
        NfaOp::try_from(op(state)),
        Ok(NfaOp::StartInvisibleNeg
            | NfaOp::StartInvisibleNegFirst
            | NfaOp::StartInvisibleBeforeNeg
            | NfaOp::StartInvisibleBeforeNegFirst)
    )
}

/// Did a sub-match come out the way its lookaround wanted?
pub(crate) fn lookaround_held(state: *mut nfa_state_T, result: c_int) -> bool {
    (result != 0) != is_negated(state)
}

/// Copy the normal and — when the pattern has any — the `\z(` captures.
fn copy_both(rex: Rex, to: &mut regsubs_T, from: &regsubs_T) {
    copy_sub(&mut to.norm, &from.norm);
    if has_zsubexpr(rex) {
        copy_sub(&mut to.synt, &from.synt);
    }
}

/// As [`copy_both`], but leaving group 0 alone: a lookaround must not move
/// the whole match's start or end.
fn copy_both_off(rex: Rex, to: &mut regsubs_T, from: &regsubs_T) {
    copy_sub_off(&mut to.norm, &from.norm);
    if has_zsubexpr(rex) {
        copy_sub_off(&mut to.synt, &from.synt);
    }
}

/// Take one thread of the current list forward over `curc`.
///
/// `clen` is the character's encoded length, which a couple of arms adjust,
/// and `go_to_nextline` is set by the arm that steps over a line break.
///
/// # Safety
///
/// Every pointer must belong to the running match; `*listidx` must index
/// `thislist`.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn step(
    rex: Rex,
    thislist: &mut ThreadList,
    nextlist: &ThreadList,
    listidx: &mut c_int,
    run: &mut Run,
    curc: c_int,
    clen: &mut c_int,
    go_to_nextline: &mut bool,
) -> Step {
    let idx = *listidx as usize;
    let state = thislist.thread(idx).state;
    let out = out_of(state);
    let code = op(state);
    match NfaOp::try_from(code) {
        Ok(NfaOp::Match) => {
            // Not in the middle of a grapheme: a match may not stop
            // between a base character and its combining marks.
            if !rex.reg_icombine() && !rex.at_bol() && utf_iscomposing_legacy(curc) {
                return Step::Dead;
            }
            nfa_match.set(1);
            // SAFETY: `run.submatch` is the caller's capture set.
            let submatch = unsafe { &mut *run.submatch };
            copy_both(rex, submatch, &thislist.thread(idx).subs);
            if nextlist.len() == 0 {
                *clen = 0;
            }
            Step::Matched
        }

        Ok(NfaOp::EndInvisible | NfaOp::EndInvisibleNeg | NfaOp::EndPattern) => {
            // The lookaround's own match has to end exactly where the
            // outer match asked it to.
            if !unsafe { at_sub_match_end(rex) } {
                return Step::Dead;
            }
            // A negated lookaround discards what it captured: it only
            // has to have matched, and its groups did not really match.
            if code != NfaOp::EndInvisibleNeg.code() {
                // SAFETY: `run.m` is the caller's capture set.
                let m = unsafe { &mut *run.m };
                copy_both(rex, m, &thislist.thread(idx).subs);
            }
            nfa_match.set(1);
            if nextlist.len() == 0 {
                *clen = 0;
            }
            Step::Matched
        }

        Ok(
            NfaOp::StartInvisible
            | NfaOp::StartInvisibleFirst
            | NfaOp::StartInvisibleNeg
            | NfaOp::StartInvisibleNegFirst
            | NfaOp::StartInvisibleBefore
            | NfaOp::StartInvisibleBeforeFirst
            | NfaOp::StartInvisibleBeforeNeg
            | NfaOp::StartInvisibleBeforeNegFirst,
        ) => unsafe { start_lookaround(rex, thislist, idx, listidx, run) },

        Ok(NfaOp::StartPattern) => unsafe {
            start_pattern(rex, thislist, nextlist, idx, run, *clen)
        },

        Ok(NfaOp::Bol) => Step::zero_width(rex.at_bol(), out),
        Ok(NfaOp::Eol) => Step::zero_width(curc == NUL, out),
        Ok(NfaOp::Bow) => Step::zero_width(unsafe { at_word_start(rex, curc) }, out),
        Ok(NfaOp::Eow) => Step::zero_width(unsafe { at_word_end(rex) }, out),
        Ok(NfaOp::Bof) => Step::zero_width(
            rex.lnum() == 0 && rex.at_bol() && (!rex.multi() || rex.reg_firstlnum() == 1),
            out,
        ),
        Ok(NfaOp::Eof) => Step::zero_width(rex.lnum() == rex.reg_maxline() && curc == NUL, out),

        Ok(NfaOp::Composing) => {
            let matched = unsafe { matches_composing(rex, out, curc, *clen) };
            // The group's end, and what follows it, hang off `out1`.
            Step::consuming(matched, out_of(out1_of(state)), *clen)
        }

        Ok(NfaOp::Newl) => {
            if curc == NUL && !rex.reg_line_lbr() && rex.multi() && rex.lnum() <= rex.reg_maxline()
            {
                // A real line break: the next list starts on the next
                // line, so it is added at offset -1.
                *go_to_nextline = true;
                Step::next(out, -1)
            } else if curc == b'\n' as c_int && rex.reg_line_lbr() {
                // A string match with 'linebreak' semantics: the break
                // is just a byte.
                Step::next(out, 1)
            } else {
                Step::Dead
            }
        }

        Ok(NfaOp::StartColl | NfaOp::StartNegColl) => {
            if curc == NUL {
                return Step::Dead;
            }
            let matched = unsafe { collection_matches(rex, state, curc, *clen) };
            Step::consuming(matched, out_of(out1_of(state)), *clen)
        }

        Ok(NfaOp::Any) => Step::consuming(curc > 0, out, *clen),

        Ok(NfaOp::AnyComposing) => {
            // `\%C`: a combining character is consumed, anything else
            // leaves the position alone for the group that follows.
            if utf_iscomposing_legacy(curc) {
                Step::next(out, *clen)
            } else {
                Step::Here(out)
            }
        }

        Ok(class) if class.is_class() => {
            Step::consuming(class_matches(rex, class, curc), out, *clen)
        }

        Ok(reference) if reference.is_reference() => {
            let mut bytelen = 0;
            let matched = match reference.index_in(&NfaOp::BACKREFS) {
                Some(i) => match_backref(
                    rex,
                    &thislist.thread(idx).subs.norm,
                    group_number(i),
                    &mut bytelen,
                ),
                None => {
                    let i = reference.index_in(&NfaOp::ZREFS).expect("a \\z reference");
                    match_zref(rex, group_number(i), &mut bytelen)
                }
            };
            if !matched {
                return Step::Dead;
            }
            // What it matched may be longer than one character, in
            // which case the thread waits behind an `NFA_SKIP`.
            spanning(out, bytelen, *clen)
        }

        Ok(NfaOp::Skip) => {
            // Consume the bytes a back-reference still owes.
            let owed = thislist.thread(idx).count - *clen;
            if owed <= 0 {
                Step::next(out, *clen)
            } else {
                Step::Next {
                    state,
                    off: 0,
                    count: owed,
                }
            }
        }

        Ok(NfaOp::Lnum | NfaOp::LnumGt | NfaOp::LnumLt) => {
            Step::zero_width(at_line(rex, state), out)
        }
        Ok(NfaOp::Col | NfaOp::ColGt | NfaOp::ColLt) => Step::zero_width(at_col(rex, state), out),
        Ok(NfaOp::Vcol | NfaOp::VcolGt | NfaOp::VcolLt) => {
            Step::zero_width(at_vcol(rex, state), out)
        }
        Ok(NfaOp::Mark | NfaOp::MarkGt | NfaOp::MarkLt) => {
            Step::zero_width(at_mark(rex, state), out)
        }
        Ok(NfaOp::Cursor) => Step::zero_width(at_cursor(rex), out),
        Ok(NfaOp::Visual) => Step::zero_width(in_visual(rex), out),

        // The capture brackets are recorded by `addstate` as it walks
        // past them, so a thread sitting on one has nothing to do.
        // `NfaOp::Mopen` itself is deliberately absent: upstream leaves it
        // to the literal-character arm below, which never matches it.
        Ok(NfaOp::Nopen | NfaOp::Zstart) => Step::Dead,
        Ok(marker) if marker != NfaOp::Mopen && marker.opens_capture() => Step::Dead,

        // A literal character, and every opcode upstream leaves to it.
        _ => {
            let c = code;
            let mut matched = c == curc;
            if !matched && rex.reg_ic() {
                matched = utf_fold(c) == utf_fold(curc);
            }
            if matched && !rex.reg_icombine() {
                // The pattern named the base character only, so the
                // combining marks after it are not consumed with it.
                *clen = rex.base_len();
            }
            Step::consuming(matched, out, *clen)
        }
    }
}

/// The group a `\\1`-style reference names: its opcode's position in the run
/// of nine, plus one.
fn group_number(index: usize) -> c_int {
    c_int::try_from(index + 1).expect("a group number below ten")
}

/// A thread that matched `bytelen` bytes: it may be shorter than the
/// character under the input, exactly it, or longer — in which case an
/// `NFA_SKIP` waits out the remainder.
fn spanning(out: *mut nfa_state_T, bytelen: c_int, clen: c_int) -> Step {
    // SAFETY: `out` is a live state of the running program.
    if bytelen == 0 {
        Step::Here(unsafe { (*out).out })
    } else if bytelen <= clen {
        Step::next(unsafe { (*out).out }, clen)
    } else {
        Step::Next {
            state: out,
            off: bytelen,
            count: bytelen - clen,
        }
    }
}

/// Is the input where the lookaround that is running was told to stop?
///
/// # Safety
///
/// The match context must be live.
unsafe fn at_sub_match_end(rex: Rex) -> bool {
    let endp = nfa_endp.get();
    // SAFETY: `nfa_endp` is null or the caller's stopping point, which lives
    // for the whole of the lookaround it was set for.
    endp.is_null() || rex.is_at(unsafe { *endp })
}

/// `\<`: a keyword character with something that is not one in front of it.
///
/// # Safety
///
/// The match context must be live.
unsafe fn at_word_start(rex: Rex, curc: c_int) -> bool {
    if curc == NUL {
        return false;
    }
    let this_class = unsafe {
        mb_get_class_tab(
            rex.input_str(),
            &raw mut (*rex.reg_buf()).b_chartab as *mut u64,
        )
    };
    this_class > 1 && reg_prev_class(rex) != this_class
}

/// `\>`: a keyword character behind the position and something else at it.
///
/// # Safety
///
/// The match context must be live.
unsafe fn at_word_end(rex: Rex) -> bool {
    if rex.at_bol() {
        return false;
    }
    let this_class = unsafe {
        mb_get_class_tab(
            rex.input_str(),
            &raw mut (*rex.reg_buf()).b_chartab as *mut u64,
        )
    };
    let prev_class = reg_prev_class(rex);
    this_class != prev_class && prev_class != 0 && prev_class != 1
}

/// Walk a collection's members, looking for one that accepts `curc`.
///
/// # Safety
///
/// `start` must be an `NFA_START_COLL`/`NFA_START_NEG_COLL` state.
unsafe fn collection_matches(rex: Rex, start: *mut nfa_state_T, curc: c_int, clen: c_int) -> bool {
    // A negated collection accepts exactly what its members reject.
    let member_wins = unsafe { (*start).c } == NfaOp::StartColl.code();
    let mut state = unsafe { (*start).out };
    loop {
        let c = op(state);
        if c == NfaOp::Composing.code() {
            // A member that is a whole grapheme.
            return unsafe { matches_composing(rex, (*(*start).out).out, curc, clen) }
                == member_wins;
        }
        if c == NfaOp::EndColl.code() {
            // Nothing accepted it.
            return !member_wins;
        }
        if c == NfaOp::RangeMin.code() {
            let mut lo = unsafe { (*state).val };
            state = out_of(state);
            let hi = unsafe { (*state).val };
            if (lo..=hi).contains(&curc) {
                return member_wins;
            }
            if rex.reg_ic() {
                // Folding is not monotonic, so the range has to be
                // walked rather than folded at its ends.
                let folded = utf_fold(curc);
                while lo <= hi {
                    if utf_fold(lo) == folded {
                        return member_wins;
                    }
                    lo += 1;
                }
            }
        } else {
            let accepted = if c < 0 {
                check_char_class(rex, c, curc).is_ok()
            } else {
                c == curc || (rex.reg_ic() && utf_fold(curc) == utf_fold(c))
            };
            if accepted {
                return member_wins;
            }
        }
        state = out_of(state);
    }
}

/// A lookaround: run its pattern as a match of its own, either now or —
/// when the loop would rather try the cheaper rest of the pattern first —
/// postponed as a `nfa_pim_T` carried along with the thread.
///
/// # Safety
///
/// Every pointer must belong to the running match.
unsafe fn start_lookaround(
    rex: Rex,
    thislist: &mut ThreadList,
    idx: usize,
    listidx: &mut c_int,
    run: &mut Run,
) -> Step {
    let state = thislist.thread(idx).state;
    // Postponing is only worth it when the compiler said so, and a
    // thread that already carries one runs it now.
    let run_now = thislist.thread(idx).pim.result != PimResult::Unused
        || matches!(
            NfaOp::try_from(op(state)),
            Ok(NfaOp::StartInvisibleFirst
                | NfaOp::StartInvisibleNegFirst
                | NfaOp::StartInvisibleBeforeFirst
                | NfaOp::StartInvisibleBeforeNegFirst)
        );
    if !run_now {
        // Hand the lookaround to whatever comes after it.
        let mut pim: nfa_pim_T = unsafe { core::mem::zeroed() };
        pim.state = state;
        pim.result = PimResult::Todo;
        pim.subs.norm.in_use = 0;
        pim.subs.synt.in_use = 0;
        pim.end = rex.here();
        // `addstate_here` rewrites this list, so it may not be handed a
        // capture set that lives in it.
        let has_z = has_zsubexpr(rex);
        let t = thislist.thread(idx);
        copy_sub(&mut run.here.norm, &t.subs.norm);
        if has_z {
            copy_sub(&mut run.here.synt, &t.subs.synt);
        }
        if !addstate_here(
            thislist,
            out_of(out1_of(state)),
            run.here,
            Some(&pim),
            listidx,
        ) {
            return Step::TooExpensive;
        }
        return Step::Dead;
    }

    // `m` is scratch shared with the sub-match; group 0 is the outer
    // match's and must survive.
    let in_use = unsafe { (*run.m).norm.in_use };
    copy_both_off(rex, unsafe { &mut *run.m }, &thislist.thread(idx).subs);
    let result = recursive_regmatch(
        rex,
        state,
        core::ptr::null_mut(),
        run.prog,
        run.submatch,
        run.m,
        run.listids,
    );
    if result == NFA_TOO_EXPENSIVE {
        nfa_match.set(result);
        return Step::TooExpensive;
    }
    let step = if lookaround_held(state, result) {
        copy_both_off(rex, &mut thislist.thread_mut(idx).subs, unsafe { &*run.m });
        // `\ze` inside the lookaround may have moved the match's end.
        copy_ze_off(rex, &mut thislist.thread_mut(idx).subs.norm, unsafe {
            &(*run.m).norm
        });
        Step::Here(out_of(out1_of(state)))
    } else {
        Step::Dead
    };
    unsafe { (*run.m).norm.in_use = in_use };
    step
}

/// `\@>`: like a lookahead, except that what it matched is consumed.
///
/// # Safety
///
/// Every pointer must belong to the running match.
unsafe fn start_pattern(
    rex: Rex,
    thislist: &mut ThreadList,
    nextlist: &ThreadList,
    idx: usize,
    run: &mut Run,
    clen: c_int,
) -> Step {
    let state = thislist.thread(idx).state;
    let after = out_of(out1_of(state));
    // If the state this would land on is already queued with the same
    // captures, running the sub-match again would prove nothing.
    let subs = &thislist.thread(idx).subs;
    let already = nextlist.holds(after, subs)
        || nextlist.holds(unsafe { (*after).out }, subs)
        || thislist.holds(unsafe { (*after).out }, subs);
    if already {
        return Step::Dead;
    }

    copy_both_off(rex, unsafe { &mut *run.m }, &thislist.thread(idx).subs);
    let result = recursive_regmatch(
        rex,
        state,
        core::ptr::null_mut(),
        run.prog,
        run.submatch,
        run.m,
        run.listids,
    );
    if result == NFA_TOO_EXPENSIVE {
        nfa_match.set(result);
        return Step::TooExpensive;
    }
    if result == 0 {
        return Step::Dead;
    }
    copy_both_off(rex, &mut thislist.thread_mut(idx).subs, unsafe { &*run.m });
    // How far the sub-match reached, which is what the thread consumes.
    let bytelen = rex.bytes_ahead(unsafe { (*run.m).norm.list[0].end });
    spanning(after, bytelen, clen)
}
