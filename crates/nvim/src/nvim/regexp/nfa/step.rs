//! One thread of the match loop: what the state it sits on says about the
//! character under the input, and where that leaves it.
//!
//! Everything here answers with a [`Step`]; the loop in
//! [`super::matcher`] is what acts on it.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::assertions::{at_col, at_cursor, at_line, at_mark, at_vcol, in_visual};
use super::classes::class_matches;
use super::composing::matches_composing;
use super::list::{ThreadList, addstate_here};
use super::run::{check_char_class, match_backref, match_zref, recursive_regmatch};
use super::sub::{copy_sub, copy_sub_off, copy_ze_off, has_zsubexpr};
use crate::src::nvim::mbyte::{mb_get_class_tab, utf_fold, utf_iscomposing_legacy, utf_ptr2len};
use crate::src::nvim::regexp::{
    FAIL, NFA_ANY, NFA_ANY_COMPOSING, NFA_BACKREF1, NFA_BACKREF9, NFA_BOF, NFA_BOL, NFA_BOW,
    NFA_COL, NFA_COL_LT, NFA_COMPOSING, NFA_CURSOR, NFA_END_COLL, NFA_END_INVISIBLE,
    NFA_END_INVISIBLE_NEG, NFA_END_PATTERN, NFA_EOF, NFA_EOL, NFA_EOW, NFA_IDENT, NFA_LNUM,
    NFA_LNUM_LT, NFA_MARK, NFA_MARK_LT, NFA_MATCH, NFA_MOPEN1, NFA_MOPEN9, NFA_NEWL, NFA_NOPEN,
    NFA_NUPPER_IC, NFA_PIM_TODO, NFA_PIM_UNUSED, NFA_RANGE_MIN, NFA_SKIP, NFA_START_COLL,
    NFA_START_INVISIBLE, NFA_START_INVISIBLE_BEFORE_FIRST, NFA_START_INVISIBLE_BEFORE_NEG,
    NFA_START_INVISIBLE_BEFORE_NEG_FIRST, NFA_START_INVISIBLE_FIRST, NFA_START_INVISIBLE_NEG,
    NFA_START_INVISIBLE_NEG_FIRST, NFA_START_NEG_COLL, NFA_START_PATTERN, NFA_TOO_EXPENSIVE,
    NFA_VCOL, NFA_VCOL_LT, NFA_VISUAL, NFA_ZOPEN, NFA_ZOPEN9, NFA_ZREF1, NFA_ZREF9, NFA_ZSTART,
    NUL, nfa_endp, nfa_match, nfa_pim_T, nfa_regprog_T, nfa_state_T, reg_prev_class, regsubs_T,
    rex,
};

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
    unsafe {
        matches!(
            (*state).c,
            NFA_START_INVISIBLE_NEG
                | NFA_START_INVISIBLE_NEG_FIRST
                | NFA_START_INVISIBLE_BEFORE_NEG
                | NFA_START_INVISIBLE_BEFORE_NEG_FIRST
        )
    }
}

/// Did a sub-match come out the way its lookaround wanted?
pub(crate) fn lookaround_held(state: *mut nfa_state_T, result: c_int) -> bool {
    (result != 0) != is_negated(state)
}

/// Copy the normal and — when the pattern has any — the `\z(` captures.
fn copy_both(to: &mut regsubs_T, from: &regsubs_T) {
    copy_sub(&mut to.norm, &from.norm);
    if has_zsubexpr() {
        copy_sub(&mut to.synt, &from.synt);
    }
}

/// As [`copy_both`], but leaving group 0 alone: a lookaround must not move
/// the whole match's start or end.
fn copy_both_off(to: &mut regsubs_T, from: &regsubs_T) {
    copy_sub_off(&mut to.norm, &from.norm);
    if has_zsubexpr() {
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
    thislist: &mut ThreadList,
    nextlist: &ThreadList,
    listidx: &mut c_int,
    run: &mut Run,
    curc: c_int,
    clen: &mut c_int,
    go_to_nextline: &mut bool,
) -> Step {
    unsafe {
        let idx = *listidx as usize;
        let state = thislist.thread(idx).state;
        let out = (*state).out;
        match (*state).c {
            NFA_MATCH => {
                // Not in the middle of a grapheme: a match may not stop
                // between a base character and its combining marks.
                if !(*rex.ptr()).reg_icombine
                    && (*rex.ptr()).input != (*rex.ptr()).line
                    && utf_iscomposing_legacy(curc)
                {
                    return Step::Dead;
                }
                nfa_match.set(1);
                copy_both(&mut *run.submatch, &thislist.thread(idx).subs);
                if nextlist.len() == 0 {
                    *clen = 0;
                }
                Step::Matched
            }

            NFA_END_INVISIBLE | NFA_END_INVISIBLE_NEG | NFA_END_PATTERN => {
                // The lookaround's own match has to end exactly where the
                // outer match asked it to.
                if !at_sub_match_end() {
                    return Step::Dead;
                }
                // A negated lookaround discards what it captured: it only
                // has to have matched, and its groups did not really match.
                if (*state).c != NFA_END_INVISIBLE_NEG {
                    copy_both(&mut *run.m, &thislist.thread(idx).subs);
                }
                nfa_match.set(1);
                if nextlist.len() == 0 {
                    *clen = 0;
                }
                Step::Matched
            }

            NFA_START_INVISIBLE..=NFA_START_INVISIBLE_BEFORE_NEG_FIRST => {
                start_lookaround(thislist, idx, listidx, run)
            }

            NFA_START_PATTERN => start_pattern(thislist, nextlist, idx, run, *clen),

            NFA_BOL => Step::zero_width((*rex.ptr()).input == (*rex.ptr()).line, out),
            NFA_EOL => Step::zero_width(curc == NUL, out),
            NFA_BOW => Step::zero_width(at_word_start(curc), out),
            NFA_EOW => Step::zero_width(at_word_end(), out),
            NFA_BOF => Step::zero_width(
                (*rex.ptr()).lnum == 0
                    && (*rex.ptr()).input == (*rex.ptr()).line
                    && (!(*rex.ptr()).reg_match.is_null() || (*rex.ptr()).reg_firstlnum == 1),
                out,
            ),
            NFA_EOF => Step::zero_width(
                (*rex.ptr()).lnum == (*rex.ptr()).reg_maxline && curc == NUL,
                out,
            ),

            NFA_COMPOSING => {
                let matched = matches_composing(out, curc, *clen);
                // The group's end, and what follows it, hang off `out1`.
                Step::consuming(matched, (*(*state).out1).out, *clen)
            }

            NFA_NEWL => {
                if curc == NUL
                    && !(*rex.ptr()).reg_line_lbr
                    && (*rex.ptr()).reg_match.is_null()
                    && (*rex.ptr()).lnum <= (*rex.ptr()).reg_maxline
                {
                    // A real line break: the next list starts on the next
                    // line, so it is added at offset -1.
                    *go_to_nextline = true;
                    Step::next(out, -1)
                } else if curc == b'\n' as c_int && (*rex.ptr()).reg_line_lbr {
                    // A string match with 'linebreak' semantics: the break
                    // is just a byte.
                    Step::next(out, 1)
                } else {
                    Step::Dead
                }
            }

            NFA_START_COLL | NFA_START_NEG_COLL => {
                if curc == NUL {
                    return Step::Dead;
                }
                let matched = collection_matches(state, curc, *clen);
                Step::consuming(matched, (*(*state).out1).out, *clen)
            }

            NFA_ANY => Step::consuming(curc > 0, out, *clen),

            NFA_ANY_COMPOSING => {
                // `\%C`: a combining character is consumed, anything else
                // leaves the position alone for the group that follows.
                if utf_iscomposing_legacy(curc) {
                    Step::next(out, *clen)
                } else {
                    Step::Here(out)
                }
            }

            c @ NFA_IDENT..=NFA_NUPPER_IC => Step::consuming(class_matches(c, curc), out, *clen),

            c @ (NFA_BACKREF1..=NFA_BACKREF9 | NFA_ZREF1..=NFA_ZREF9) => {
                let mut bytelen = 0;
                let matched = if (NFA_BACKREF1..=NFA_BACKREF9).contains(&c) {
                    match_backref(
                        &thislist.thread(idx).subs.norm,
                        c - NFA_BACKREF1 + 1,
                        &mut bytelen,
                    )
                } else {
                    match_zref(c - NFA_ZREF1 + 1, &mut bytelen)
                };
                if !matched {
                    return Step::Dead;
                }
                // What it matched may be longer than one character, in
                // which case the thread waits behind an `NFA_SKIP`.
                spanning(out, bytelen, *clen)
            }

            NFA_SKIP => {
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

            NFA_LNUM..=NFA_LNUM_LT => Step::zero_width(at_line(state), out),
            NFA_COL..=NFA_COL_LT => Step::zero_width(at_col(state), out),
            NFA_VCOL..=NFA_VCOL_LT => Step::zero_width(at_vcol(state), out),
            NFA_MARK..=NFA_MARK_LT => Step::zero_width(at_mark(state), out),
            NFA_CURSOR => Step::zero_width(at_cursor(), out),
            NFA_VISUAL => Step::zero_width(in_visual(), out),

            // The capture brackets are recorded by `addstate` as it walks
            // past them, so a thread sitting on one has nothing to do.
            // `NFA_MOPEN` itself is deliberately absent: upstream leaves it
            // to the literal-character arm below, which never matches it.
            NFA_MOPEN1..=NFA_MOPEN9 | NFA_ZOPEN..=NFA_ZOPEN9 | NFA_NOPEN | NFA_ZSTART => Step::Dead,

            // A literal character.
            c => {
                let mut matched = c == curc;
                if !matched && (*rex.ptr()).reg_ic {
                    matched = utf_fold(c) == utf_fold(curc);
                }
                if matched && !(*rex.ptr()).reg_icombine {
                    // The pattern named the base character only, so the
                    // combining marks after it are not consumed with it.
                    *clen = utf_ptr2len((*rex.ptr()).input as *mut c_char);
                }
                Step::consuming(matched, out, *clen)
            }
        }
    }
}

/// A thread that matched `bytelen` bytes: it may be shorter than the
/// character under the input, exactly it, or longer — in which case an
/// `NFA_SKIP` waits out the remainder.
fn spanning(out: *mut nfa_state_T, bytelen: c_int, clen: c_int) -> Step {
    // SAFETY: `out` is a live state of the running program.
    unsafe {
        if bytelen == 0 {
            Step::Here((*out).out)
        } else if bytelen <= clen {
            Step::next((*out).out, clen)
        } else {
            Step::Next {
                state: out,
                off: bytelen,
                count: bytelen - clen,
            }
        }
    }
}

/// Is the input where the lookaround that is running was told to stop?
///
/// # Safety
///
/// The match context must be live.
unsafe fn at_sub_match_end() -> bool {
    unsafe {
        let endp = nfa_endp.get();
        if endp.is_null() {
            return true;
        }
        if (*rex.ptr()).reg_match.is_null() {
            (*rex.ptr()).lnum == (*endp).se_u.pos.lnum
                && (*rex.ptr()).input.offset_from((*rex.ptr()).line) as c_int
                    == (*endp).se_u.pos.col
        } else {
            (*rex.ptr()).input == (*endp).se_u.ptr
        }
    }
}

/// `\<`: a keyword character with something that is not one in front of it.
///
/// # Safety
///
/// The match context must be live.
unsafe fn at_word_start(curc: c_int) -> bool {
    unsafe {
        if curc == NUL {
            return false;
        }
        let this_class = mb_get_class_tab(
            (*rex.ptr()).input as *mut c_char,
            &raw mut (*(*rex.ptr()).reg_buf).b_chartab as *mut u64,
        );
        this_class > 1 && reg_prev_class() != this_class
    }
}

/// `\>`: a keyword character behind the position and something else at it.
///
/// # Safety
///
/// The match context must be live.
unsafe fn at_word_end() -> bool {
    unsafe {
        if (*rex.ptr()).input == (*rex.ptr()).line {
            return false;
        }
        let this_class = mb_get_class_tab(
            (*rex.ptr()).input as *mut c_char,
            &raw mut (*(*rex.ptr()).reg_buf).b_chartab as *mut u64,
        );
        let prev_class = reg_prev_class();
        this_class != prev_class && prev_class != 0 && prev_class != 1
    }
}

/// Walk a collection's members, looking for one that accepts `curc`.
///
/// # Safety
///
/// `start` must be an `NFA_START_COLL`/`NFA_START_NEG_COLL` state.
unsafe fn collection_matches(start: *mut nfa_state_T, curc: c_int, clen: c_int) -> bool {
    unsafe {
        // A negated collection accepts exactly what its members reject.
        let member_wins = (*start).c == NFA_START_COLL;
        let mut state = (*start).out;
        loop {
            let c = (*state).c;
            if c == NFA_COMPOSING {
                // A member that is a whole grapheme.
                return matches_composing((*(*start).out).out, curc, clen) == member_wins;
            }
            if c == NFA_END_COLL {
                // Nothing accepted it.
                return !member_wins;
            }
            if c == NFA_RANGE_MIN {
                let mut lo = (*state).val;
                state = (*state).out;
                let hi = (*state).val;
                if (lo..=hi).contains(&curc) {
                    return member_wins;
                }
                if (*rex.ptr()).reg_ic {
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
                    check_char_class(c, curc) != FAIL
                } else {
                    c == curc || ((*rex.ptr()).reg_ic && utf_fold(curc) == utf_fold(c))
                };
                if accepted {
                    return member_wins;
                }
            }
            state = (*state).out;
        }
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
    thislist: &mut ThreadList,
    idx: usize,
    listidx: &mut c_int,
    run: &mut Run,
) -> Step {
    unsafe {
        let state = thislist.thread(idx).state;
        // Postponing is only worth it when the compiler said so, and a
        // thread that already carries one runs it now.
        let run_now = thislist.thread(idx).pim.result != NFA_PIM_UNUSED
            || matches!(
                (*state).c,
                NFA_START_INVISIBLE_FIRST
                    | NFA_START_INVISIBLE_NEG_FIRST
                    | NFA_START_INVISIBLE_BEFORE_FIRST
                    | NFA_START_INVISIBLE_BEFORE_NEG_FIRST
            );
        if !run_now {
            // Hand the lookaround to whatever comes after it.
            let mut pim: nfa_pim_T = core::mem::zeroed();
            pim.state = state;
            pim.result = NFA_PIM_TODO;
            pim.subs.norm.in_use = 0;
            pim.subs.synt.in_use = 0;
            if (*rex.ptr()).reg_match.is_null() {
                pim.end.pos.col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as c_int;
                pim.end.pos.lnum = (*rex.ptr()).lnum;
            } else {
                pim.end.ptr = (*rex.ptr()).input;
            }
            // `addstate_here` rewrites this list, so it may not be handed a
            // capture set that lives in it.
            let has_z = has_zsubexpr();
            let t = thislist.thread(idx);
            copy_sub(&mut run.here.norm, &t.subs.norm);
            if has_z {
                copy_sub(&mut run.here.synt, &t.subs.synt);
            }
            if !addstate_here(
                thislist,
                (*(*state).out1).out,
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
        let in_use = (*run.m).norm.in_use;
        copy_both_off(&mut *run.m, &thislist.thread(idx).subs);
        let result = recursive_regmatch(
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
            copy_both_off(&mut thislist.thread_mut(idx).subs, &*run.m);
            // `\ze` inside the lookaround may have moved the match's end.
            copy_ze_off(&mut thislist.thread_mut(idx).subs.norm, &(*run.m).norm);
            Step::Here((*(*state).out1).out)
        } else {
            Step::Dead
        };
        (*run.m).norm.in_use = in_use;
        step
    }
}

/// `\@>`: like a lookahead, except that what it matched is consumed.
///
/// # Safety
///
/// Every pointer must belong to the running match.
unsafe fn start_pattern(
    thislist: &mut ThreadList,
    nextlist: &ThreadList,
    idx: usize,
    run: &mut Run,
    clen: c_int,
) -> Step {
    unsafe {
        let state = thislist.thread(idx).state;
        let after = (*(*state).out1).out;
        // If the state this would land on is already queued with the same
        // captures, running the sub-match again would prove nothing.
        let subs = &thislist.thread(idx).subs;
        let already = nextlist.holds(after, subs)
            || nextlist.holds((*after).out, subs)
            || thislist.holds((*after).out, subs);
        if already {
            return Step::Dead;
        }

        copy_both_off(&mut *run.m, &thislist.thread(idx).subs);
        let result = recursive_regmatch(
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
        copy_both_off(&mut thislist.thread_mut(idx).subs, &*run.m);
        // How far the sub-match reached, which is what the thread consumes.
        let bytelen = if (*rex.ptr()).reg_match.is_null() {
            (*run.m).norm.list.multi[0].end_col
                - (*rex.ptr()).input.offset_from((*rex.ptr()).line) as c_int
        } else {
            (*run.m).norm.list.line[0]
                .end
                .offset_from((*rex.ptr()).input) as c_int
        };
        spanning(after, bytelen, clen)
    }
}
