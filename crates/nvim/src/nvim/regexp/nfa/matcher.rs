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

use core::ffi::{c_char, c_int};

use super::list::{ThreadList, addstate, addstate_here};
use super::run::nfa_did_time_out;
use super::step::{Run, Step, lookaround_held, step};
use super::sub::{copy_sub, copy_sub_off, has_zsubexpr, match_follows};
use crate::src::nvim::main::got_int;
use crate::src::nvim::mbyte::{utf_fold, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::regexp::{
    AUTOMATIC_ENGINE, FAIL, NFA_MAX_STATES, NFA_MOPEN, NFA_PIM_MATCH, NFA_PIM_NOMATCH,
    NFA_PIM_TODO, NFA_PIM_UNUSED, NFA_TOO_EXPENSIVE, NUL, nfa_endp, nfa_match, nfa_pim_T,
    nfa_regprog_T, nfa_state_T, nfa_time_count, nfa_time_limit, reg_breakcheck, reg_nextline,
    regsubs_T, rex,
};
use crate::src::nvim::regexp::{recursive_regmatch, skip_to_start};
use crate::src::nvim::types::colnr_T;

/// How many characters may pass between two checks of the caller's time
/// limit.
const TIME_CHECK_INTERVAL: c_int = 20;

/// Run the machine `start` over the input, from `rex.input`.
///
/// `submatch` receives the captures of a successful match and `m` is the
/// scratch set the threads carry. Returns 1 on a match, 0 on none, or
/// `NFA_TOO_EXPENSIVE` when the pattern outgrew the automatic engine.
pub(crate) fn nfa_regmatch(
    prog: *mut nfa_regprog_T,
    start: *mut nfa_state_T,
    submatch: *mut regsubs_T,
    m: *mut regsubs_T,
) -> c_int {
    reg_breakcheck();
    if got_int.get() || nfa_did_time_out() {
        return 0;
    }
    nfa_match.set(0);

    // SAFETY: `prog` and `start` are the running program; the two thread
    // lists below are owned by this call.
    unsafe {
        let capacity = ((*prog).nstate + 1) as usize;
        let mut list = [ThreadList::new(capacity), ThreadList::new(capacity)];

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
        list[0].id = (*rex.ptr()).nfa_listid + 1;
        let seeded = if toplevel {
            record_match_start(m, 0);
            (*m).norm.in_use = 1;
            addstate(&mut list[0], (*start).out, &mut *m, None, 0)
        } else {
            addstate(&mut list[0], start, &mut *m, None, 0)
        };

        if !seeded {
            nfa_match.set(NFA_TOO_EXPENSIVE);
        } else {
            scan(&mut list, prog, start, toplevel, &mut run);
        }
        nfa_match.get()
    }
}

/// Record where group 0 starts, `off` bytes past the input.
///
/// # Safety
///
/// `m` must be a live capture set and the match context live.
unsafe fn record_match_start(m: *mut regsubs_T, off: c_int) {
    unsafe {
        if (*rex.ptr()).reg_match.is_null() {
            let col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T + off;
            (*m).norm.list.multi[0].start_lnum = (*rex.ptr()).lnum;
            (*m).norm.list.multi[0].start_col = col;
            // The column a `:substitute` resumes scanning from.
            (*m).norm.orig_start_col = col;
        } else {
            (*m).norm.list.line[0].start = (*rex.ptr()).input.offset(off as isize);
        }
    }
}

/// The main loop: one pass per input character.
///
/// # Safety
///
/// Every pointer must belong to the running match.
unsafe fn scan(
    list: &mut [ThreadList; 2],
    prog: *mut nfa_regprog_T,
    start: *mut nfa_state_T,
    toplevel: bool,
    run: &mut Run,
) {
    unsafe {
        // Which of the two lists holds the threads at the current position.
        let mut current = 0;
        let mut go_to_nextline = false;

        loop {
            let curc = utf_ptr2char((*rex.ptr()).input as *mut c_char);
            let mut clen = utfc_ptr2len((*rex.ptr()).input as *mut c_char);
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
            (*rex.ptr()).nfa_listid += 1;
            if (*prog).re_engine == AUTOMATIC_ENGINE && (*rex.ptr()).nfa_listid >= NFA_MAX_STATES {
                nfa_match.set(NFA_TOO_EXPENSIVE);
                return;
            }
            thislist.id = (*rex.ptr()).nfa_listid;
            nextlist.id = (*rex.ptr()).nfa_listid + 1;
            if thislist.len() == 0 {
                // Nothing alive: the match is over.
                return;
            }

            let mut matched = false;
            let mut listidx: c_int = 0;
            while (listidx as usize) < thislist.len() {
                reg_breakcheck();
                if got_int.get() || out_of_time() {
                    break;
                }
                let outcome = step(
                    thislist,
                    nextlist,
                    &mut listidx,
                    run,
                    curc,
                    &mut clen,
                    &mut go_to_nextline,
                );
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
                        if !deliver(thislist, nextlist, &mut listidx, run, clen, add) {
                            nfa_match.set(NFA_TOO_EXPENSIVE);
                            return;
                        }
                    }
                }
                listidx += 1;
            }

            if !matched && !restart(prog, start, toplevel, nextlist, run, clen) {
                return;
            }

            if clen != 0 {
                (*rex.ptr()).input = (*rex.ptr()).input.offset(clen as isize);
            } else {
                // At the end of a line: carry on only if something still
                // wants the next one.
                if !go_to_nextline && !sub_match_spans_lines() {
                    return;
                }
                reg_nextline();
            }
            reg_breakcheck();
            if got_int.get() || out_of_time() {
                return;
            }
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
unsafe fn sub_match_spans_lines() -> bool {
    unsafe {
        let endp = nfa_endp.get();
        !endp.is_null()
            && (*rex.ptr()).reg_match.is_null()
            && (*rex.ptr()).lnum < (*endp).se_u.pos.lnum
    }
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
    thislist: &mut ThreadList,
    nextlist: &mut ThreadList,
    listidx: &mut c_int,
    run: &mut Run,
    clen: c_int,
    add: Step,
) -> bool {
    unsafe {
        let (add_state, add_here, add_off, add_count) = match add {
            Step::Here(state) => (state, true, 0, 0),
            Step::Next { state, off, count } => (state, false, off, count),
            _ => return true,
        };

        let idx = *listidx as usize;
        let mut carries_pim = thislist.thread(idx).pim.result != NFA_PIM_UNUSED;

        // The lookaround was postponed to here: settle it now, either
        // because there is no more input to postpone past or because the
        // state we are about to add ends the pattern.
        if carries_pim && (clen == 0 || match_follows(add_state, 0)) {
            let t = thislist.thread_mut(idx);
            let pim_state = t.pim.state;
            let result = if t.pim.result == NFA_PIM_TODO {
                let result = recursive_regmatch(
                    pim_state,
                    &raw mut t.pim,
                    run.prog,
                    run.submatch,
                    run.m,
                    run.listids,
                );
                t.pim.result = if result != 0 {
                    NFA_PIM_MATCH
                } else {
                    NFA_PIM_NOMATCH
                };
                if lookaround_held(pim_state, result) {
                    // Keep what the lookaround captured, but not its idea
                    // of where the whole match starts.
                    copy_sub_off(&mut t.pim.subs.norm, &(*run.m).norm);
                    if has_zsubexpr() {
                        copy_sub_off(&mut t.pim.subs.synt, &(*run.m).synt);
                    }
                }
                result
            } else {
                // Already decided, on an earlier thread.
                (t.pim.result == NFA_PIM_MATCH) as c_int
            };
            if !lookaround_held(pim_state, result) {
                // The lookaround failed: the thread dies rather than being
                // added anywhere.
                return true;
            }
            copy_sub_off(&mut t.subs.norm, &t.pim.subs.norm);
            if has_zsubexpr() {
                copy_sub_off(&mut t.subs.synt, &t.pim.subs.synt);
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
            let has_z = has_zsubexpr();
            let t = thislist.thread(idx);
            copy_sub(&mut run.here.norm, &t.subs.norm);
            if has_z {
                copy_sub(&mut run.here.synt, &t.subs.synt);
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
    prog: *mut nfa_regprog_T,
    start: *mut nfa_state_T,
    toplevel: bool,
    nextlist: &mut ThreadList,
    run: &mut Run,
    clen: c_int,
) -> bool {
    unsafe {
        if nfa_match.get() != 0 || !wants_restart(toplevel, clen) {
            return true;
        }
        if !toplevel {
            // A lookaround's own machine has no start column to move.
            return seed(nextlist, start, run, clen);
        }

        // The program may know the character every match starts with, in
        // which case there is no point restarting anywhere else.
        if (*prog).regstart != NUL && clen != 0 {
            if nextlist.len() == 0 {
                // Nothing is alive, so jump the input straight to the next
                // place that character occurs.
                let mut col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T + clen;
                if skip_to_start((*prog).regstart, &mut col) == FAIL {
                    return false;
                }
                (*rex.ptr()).input = (*rex.ptr())
                    .line
                    .offset(col as isize)
                    .offset(-(clen as isize));
            } else {
                let next = utf_ptr2char(((*rex.ptr()).input as *mut c_char).offset(clen as isize));
                if next != (*prog).regstart
                    && (!(*rex.ptr()).reg_ic || utf_fold(next) != utf_fold((*prog).regstart))
                {
                    return true;
                }
            }
        }
        // Only reachable on the match's first line, where `rex.lnum` is
        // still what the seeding call recorded.
        record_match_start(run.m, clen);
        seed(nextlist, (*start).out, run, clen)
    }
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
///
/// # Safety
///
/// The match context must be live.
unsafe fn wants_restart(toplevel: bool, clen: c_int) -> bool {
    unsafe {
        // The outer match may start at any column of the first line, up to
        // 'reg_maxcol' where the caller set one.
        if toplevel
            && (*rex.ptr()).lnum == 0
            && clen != 0
            && ((*rex.ptr()).reg_maxcol == 0
                || ((*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T)
                    < (*rex.ptr()).reg_maxcol)
        {
            return true;
        }
        // A lookaround's machine may start anywhere before the position the
        // outer match told it to stop at.
        let endp = nfa_endp.get();
        if endp.is_null() {
            return false;
        }
        if (*rex.ptr()).reg_match.is_null() {
            (*rex.ptr()).lnum < (*endp).se_u.pos.lnum
                || ((*rex.ptr()).lnum == (*endp).se_u.pos.lnum
                    && ((*rex.ptr()).input.offset_from((*rex.ptr()).line) as c_int)
                        < (*endp).se_u.pos.col)
        } else {
            (*rex.ptr()).input < (*endp).se_u.ptr
        }
    }
}
