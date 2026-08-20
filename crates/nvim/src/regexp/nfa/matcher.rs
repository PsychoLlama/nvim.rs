//! The pike VM: advance a list of threads over the input one character at a
//! time.
//!
//! Two lists are alive at once — the threads at the current position and the
//! threads that survive to the next — and they swap each step. A thread is a
//! state plus the capture positions it reached it with, so a state can be on
//! the list more than once with different captures, and the order of the
//! list is the priority order the submatch rules need: the first thread to
//! reach `NFA_MATCH` wins.
//!
//! What each thread's state *says* lives in [`super::step`]; this is the
//! loop that asks and the bookkeeping around it.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::list::{ThreadList, addstate, addstate_here};
use super::run::nfa_did_time_out;
use super::step::{Run, Step, lookaround_held, step};
use super::sub::{copy_sub, copy_sub_off, has_zsubexpr, match_follows};
use crate::main::got_int;
use crate::mbyte::{utf_fold, utf_ptr2char};
use crate::regexp::{
    AUTOMATIC_ENGINE, NFA_MAX_STATES, NFA_MOPEN, NFA_TOO_EXPENSIVE, PimResult, Rex, nfa_endp,
    nfa_match, nfa_pim_T, nfa_regprog_T, nfa_state_T, nfa_time_count, nfa_time_limit,
    recursive_regmatch, reg_breakcheck, reg_nextline, regsubs_T, skip_to_start,
};
use crate::types::{FAIL, NUL};

/// How many characters may pass between two checks of the caller's time
/// limit.
const TIME_CHECK_INTERVAL: c_int = 20;

/// Run the machine `start` over the input, from `rex.input`.
///
/// `submatch` receives the captures of a successful match and `m` is the
/// scratch set the threads carry. Returns 1 on a match, 0 on none, or
/// `NFA_TOO_EXPENSIVE` when the pattern outgrew the automatic engine.
pub(crate) fn nfa_regmatch(
    rex: Rex,
    prog: *mut nfa_regprog_T,
    start: *mut nfa_state_T,
    submatch: *mut regsubs_T,
    m: *mut regsubs_T,
) -> c_int {
    reg_breakcheck(rex);
    if got_int.get() || nfa_did_time_out() {
        return 0;
    }
    nfa_match.set(0);

    // SAFETY: `prog` and `start` are the running program; the two thread
    // lists below are owned by this call.
    unsafe {
        let capacity = ((*prog).nstate + 1) as usize;
        let mut list = [
            ThreadList::new(rex, capacity),
            ThreadList::new(rex, capacity),
        ];

        let mut listids: Vec<c_int> = Vec::new();
        // Scratch for the one call that may not hand `addstate` a capture set
        // that lives in the list it is adding to — see `deliver`.
        let mut here: regsubs_T = core::mem::zeroed();
        let mut run = Run {
            prog,
            submatch,
            m,
            listids: &mut listids,
            here: &mut here,
        };

        // The whole pattern is wrapped in group 0, and when it is the
        // machine's own entry the match's start is recorded here rather
        // than by walking into it.
        let toplevel = (*start).c == NFA_MOPEN;
        list[0].id = rex.nfa_listid() + 1;
        let seeded = if toplevel {
            record_match_start(rex, m, 0);
            (*m).norm.in_use = 1;
            addstate(&mut list[0], (*start).out, &mut *m, None, 0)
        } else {
            addstate(&mut list[0], start, &mut *m, None, 0)
        };

        if !seeded {
            nfa_match.set(NFA_TOO_EXPENSIVE);
        } else {
            scan(rex, &mut list, prog, start, toplevel, &mut run);
        }
        nfa_match.get()
    }
}

/// Record where group 0 starts, `off` bytes past the input.
///
/// # Safety
///
/// `m` must be a live capture set and the match context live.
unsafe fn record_match_start(rex: Rex, m: *mut regsubs_T, off: c_int) {
    unsafe {
        if rex.multi() {
            let col = rex.col() + off;
            (*m).norm.list.multi[0].start_lnum = rex.lnum();
            (*m).norm.list.multi[0].start_col = col;
            // The column a `:substitute` resumes scanning from.
            (*m).norm.orig_start_col = col;
        } else {
            (*m).norm.list.line[0].start = rex.input().offset(off as isize);
        }
    }
}

/// The main loop: one pass per input character.
///
/// # Safety
///
/// Every pointer must belong to the running match.
unsafe fn scan(
    rex: Rex,
    list: &mut [ThreadList; 2],
    prog: *mut nfa_regprog_T,
    start: *mut nfa_state_T,
    toplevel: bool,
    run: &mut Run,
) {
    // Which of the two lists holds the threads at the current position.
    let mut current = 0;
    let mut go_to_nextline = false;

    loop {
        let curc = rex.char_here();
        let mut clen = rex.char_len();
        if curc == NUL {
            clen = 0;
            go_to_nextline = false;
        }

        let (first, second) = list.split_at_mut(1);
        let (thislist, nextlist) = if current == 0 {
            (&mut first[0], &mut second[0])
        } else {
            (&mut second[0], &mut first[0])
        };
        current ^= 1;
        nextlist.clear();

        // Every position gets a fresh generation, which is how a state
        // knows whether it is already on a list.
        rex.set_nfa_listid(rex.nfa_listid() + 1);
        // SAFETY: `prog` is the program being run.
        let automatic = unsafe { (*prog).re_engine } == AUTOMATIC_ENGINE;
        if automatic && rex.nfa_listid() >= NFA_MAX_STATES {
            nfa_match.set(NFA_TOO_EXPENSIVE);
            return;
        }
        thislist.id = rex.nfa_listid();
        nextlist.id = rex.nfa_listid() + 1;
        if thislist.len() == 0 {
            // Nothing alive: the match is over.
            return;
        }

        let mut matched = false;
        let mut listidx: c_int = 0;
        while (listidx as usize) < thislist.len() {
            reg_breakcheck(rex);
            if got_int.get() || out_of_time() {
                break;
            }
            let outcome = unsafe {
                step(
                    rex,
                    thislist,
                    nextlist,
                    &mut listidx,
                    run,
                    curc,
                    &mut clen,
                    &mut go_to_nextline,
                )
            };
            match outcome {
                Step::Dead => {}
                Step::Matched => {
                    matched = true;
                    break;
                }
                Step::TooExpensive => {
                    nfa_match.set(NFA_TOO_EXPENSIVE);
                    return;
                }
                add => {
                    if !unsafe { deliver(rex, thislist, nextlist, &mut listidx, run, clen, add) } {
                        nfa_match.set(NFA_TOO_EXPENSIVE);
                        return;
                    }
                }
            }
            listidx += 1;
        }

        if !matched && !unsafe { restart(rex, prog, start, toplevel, nextlist, run, clen) } {
            return;
        }

        if clen != 0 {
            rex.advance(clen);
        } else {
            // At the end of a line: carry on only if something still
            // wants the next one.
            if !go_to_nextline && !unsafe { sub_match_spans_lines(rex) } {
                return;
            }
            reg_nextline(rex);
        }
        reg_breakcheck(rex);
        if got_int.get() || out_of_time() {
            return;
        }
    }
}

/// Has the caller's time limit passed? Checked one character in
/// [`TIME_CHECK_INTERVAL`], because reading the clock is not free.
fn out_of_time() -> bool {
    if nfa_time_limit.get().is_null() {
        return false;
    }
    nfa_time_count.set(nfa_time_count.get() + 1);
    if nfa_time_count.get() != TIME_CHECK_INTERVAL {
        return false;
    }
    nfa_time_count.set(0);
    nfa_did_time_out()
}

/// Does the lookaround being matched still have input left on a later line?
///
/// # Safety
///
/// The match context must be live.
unsafe fn sub_match_spans_lines(rex: Rex) -> bool {
    let endp = nfa_endp.get();
    // SAFETY: `nfa_endp` is null or the stopping point of the lookaround
    // being matched, which outlives it.
    !endp.is_null() && rex.multi() && rex.lnum() < unsafe { (*endp).as_pos() }.lnum
}

/// Put the state a thread decided on onto a list.
///
/// A thread carrying a postponed lookaround has to settle it first: the
/// lookaround is run here, once whatever came after it has proved itself.
/// Returns false when the lists could not grow.
///
/// # Safety
///
/// Every pointer must belong to the running match, and `*listidx` index
/// `thislist`.
unsafe fn deliver(
    rex: Rex,
    thislist: &mut ThreadList,
    nextlist: &mut ThreadList,
    listidx: &mut c_int,
    run: &mut Run,
    clen: c_int,
    add: Step,
) -> bool {
    let (add_state, add_here, add_off, add_count) = match add {
        Step::Here(state) => (state, true, 0, 0),
        Step::Next { state, off, count } => (state, false, off, count),
        _ => return true,
    };

    let idx = *listidx as usize;
    let mut carries_pim = thislist.thread(idx).pim.result != PimResult::Unused;

    // The lookaround was postponed to here: settle it now, either
    // because there is no more input to postpone past or because the
    // state we are about to add ends the pattern.
    if carries_pim && (clen == 0 || match_follows(add_state, 0)) {
        let t = thislist.thread_mut(idx);
        let pim_state = t.pim.state;
        let result = if t.pim.result == PimResult::Todo {
            let result = recursive_regmatch(
                rex,
                pim_state,
                &raw mut t.pim,
                run.prog,
                run.submatch,
                run.m,
                run.listids,
            );
            t.pim.result = if result != 0 {
                PimResult::Match
            } else {
                PimResult::NoMatch
            };
            if lookaround_held(pim_state, result) {
                // Keep what the lookaround captured, but not its idea
                // of where the whole match starts.
                // SAFETY: `run.m` is the caller's capture set.
                let m = unsafe { &*run.m };
                copy_sub_off(rex, &mut t.pim.subs.norm, &m.norm);
                if has_zsubexpr(rex) {
                    copy_sub_off(rex, &mut t.pim.subs.synt, &m.synt);
                }
            }
            result
        } else {
            // Already decided, on an earlier thread.
            (t.pim.result == PimResult::Match) as c_int
        };
        if !lookaround_held(pim_state, result) {
            // The lookaround failed: the thread dies rather than being
            // added anywhere.
            return true;
        }
        copy_sub_off(rex, &mut t.subs.norm, &t.pim.subs.norm);
        if has_zsubexpr(rex) {
            copy_sub_off(rex, &mut t.subs.synt, &t.pim.subs.synt);
        }
        carries_pim = false;
    }

    // A still-postponed lookaround travels with the thread, and the
    // thread it came from may be overwritten as the list grows, so it
    // is copied out first.
    let pim_copy;
    let pim: Option<&nfa_pim_T> = if carries_pim {
        pim_copy = thislist.thread(idx).pim;
        Some(&pim_copy)
    } else {
        None
    };

    if add_here {
        // `addstate_here` rewrites the list this thread lives in, so it
        // may not be handed a capture set that lives in it.
        let has_z = has_zsubexpr(rex);
        let t = thislist.thread(idx);
        copy_sub(rex, &mut run.here.norm, &t.subs.norm);
        if has_z {
            copy_sub(rex, &mut run.here.synt, &t.subs.synt);
        }
        addstate_here(thislist, add_state, run.here, pim, listidx)
    } else {
        let subs = &mut thislist.thread_mut(idx).subs;
        if !addstate(nextlist, add_state, subs, pim, add_off) {
            return false;
        }
        if add_count > 0 && nextlist.len() > 0 {
            // The thread owes more input than this character supplies;
            // `NFA_SKIP` counts it down.
            let last = nextlist.len() - 1;
            nextlist.thread_mut(last).count = add_count;
        }
        true
    }
}

/// Seed the next list with the machine's entry state again, so that a match
/// may also start one character further on.
///
/// Returns false when the whole match should stop — either because there is
/// nowhere left for one to start, or because the list could not grow, which
/// is reported as `NFA_TOO_EXPENSIVE` so the caller falls back to the
/// backtracking engine rather than believing there was no match.
///
/// # Safety
///
/// Every pointer must belong to the running match.
unsafe fn restart(
    rex: Rex,
    prog: *mut nfa_regprog_T,
    start: *mut nfa_state_T,
    toplevel: bool,
    nextlist: &mut ThreadList,
    run: &mut Run,
    clen: c_int,
) -> bool {
    if nfa_match.get() != 0 || !wants_restart(rex, toplevel, clen) {
        return true;
    }
    if !toplevel {
        // A lookaround's own machine has no start column to move.
        return unsafe { seed(nextlist, start, run, clen) };
    }

    // The program may know the character every match starts with, in
    // which case there is no point restarting anywhere else.
    // SAFETY: `prog` is the program being run.
    let regstart = unsafe { (*prog).regstart };
    if regstart != NUL && clen != 0 {
        if nextlist.len() == 0 {
            // Nothing is alive, so jump the input straight to the next
            // place that character occurs.
            let mut col = rex.col() + clen;
            if skip_to_start(rex, regstart, &mut col) == FAIL {
                return false;
            }
            rex.set_input(unsafe { rex.line().offset(col as isize).offset(-(clen as isize)) });
        } else {
            let next = unsafe { utf_ptr2char((rex.input_str()).offset(clen as isize)) };
            if next != regstart && (!rex.reg_ic() || utf_fold(next) != utf_fold(regstart)) {
                return true;
            }
        }
    }
    // Only reachable on the match's first line, where `rex.lnum` is
    // still what the seeding call recorded.
    unsafe { record_match_start(rex, run.m, clen) };
    unsafe { seed(nextlist, (*start).out, run, clen) }
}

/// Put `state` on the next list, reporting `NFA_TOO_EXPENSIVE` if it would
/// not fit.
///
/// # Safety
///
/// Every pointer must belong to the running match.
unsafe fn seed(
    nextlist: &mut ThreadList,
    state: *mut nfa_state_T,
    run: &mut Run,
    clen: c_int,
) -> bool {
    // SAFETY: the caller's list and state.
    let added = unsafe { addstate(nextlist, state, &mut *run.m, None, clen) };
    if !added {
        nfa_match.set(NFA_TOO_EXPENSIVE);
        return false;
    }
    true
}

/// Is there anywhere left for a match to start?
fn wants_restart(rex: Rex, toplevel: bool, clen: c_int) -> bool {
    // The outer match may start at any column of the first line, up to
    // 'reg_maxcol' where the caller set one.
    if toplevel
        && rex.lnum() == 0
        && clen != 0
        && (rex.reg_maxcol() == 0 || (rex.col()) < rex.reg_maxcol())
    {
        return true;
    }
    // A lookaround's machine may start anywhere before the position the
    // outer match told it to stop at.
    let endp = nfa_endp.get();
    if endp.is_null() {
        return false;
    }
    // SAFETY: `nfa_endp` holds the position the lookaround was told to stop
    // at, which outlives the lookaround.
    rex.is_before(unsafe { *endp })
}
