//! The lists of live threads, and the walk that puts a state on one.
//!
//! A thread is a state plus the captures it was reached with. `addstate` adds
//! one and then follows everything that consumes no input from it — the
//! alternations, the empty transitions and the capture brackets, which record
//! their position on the way past and put it back on the way out. So a list
//! only ever holds states that have something to say about the character
//! under the input.
//!
//! [`addstate_here`] is the same walk aimed at the *current* list, at the
//! position the loop has already reached; it puts what it added in place of
//! the thread it was called for.
//!
//! ## 'maxmempattern'
//!
//! A list grows in the steps upstream's `nfa_list_T` did — `len * 3 / 2 + 50`
//! slots — and refuses to grow past 'maxmempattern' kilobytes of
//! `NfaThread`. That is the only bound on how many threads a pattern may
//! spawn, so both the step and the byte count are load-bearing: they decide
//! at what point E363 is reported rather than the editor running out of
//! memory. [`ThreadList::slots`] is that count and is deliberately not
//! `Vec::capacity`, which grows by doubling, and a thread is charged
//! [`NFA_THREAD_BYTES`], not what it measures today.

#![forbid(unsafe_code)]

use crate::regexp::NfaOp;
use crate::regexp::NotAnOpcode;
use core::cell::Cell;
use core::ffi::c_int;

use super::program::{NfaState, StateId, States};
use super::sub::{copy_pim, copy_sub, has_backref, has_zsubexpr, pim_equal, slots, sub_equal};
use crate::message::emsg;
use crate::option::vars::p_mmp;
use crate::os::cshim::gettext;
use crate::regexp::{
    ADDSTATE_HERE_OFFSET, Capture, E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN, MatchPos,
    NSUBEXP, PimResult, PosKind, RegSub, RegSubs, Rex,
};
use crate::types::NUL;

/// How deep `addstate` may follow itself before giving up. A machine with a
/// cycle of states that consume no input would otherwise not terminate.
const ADDSTATE_MAX_DEPTH: c_int = 5000;

/// What 'maxmempattern' charges a thread: the size `NfaThread` had when
/// a state was a pointer, which is what put E363 where the recorded
/// thresholds have it. A thread is smaller now that it names its state by
/// index; the charge is the contract, not the layout.
pub(crate) const NFA_THREAD_BYTES: usize = 712;

/// One thread: the state it sits on, the captures it reached it with, and a
/// lookaround it may be carrying.
///
/// The state is a reference into the running program, as upstream's was a
/// pointer: the match loop reads it once per thread per character, and an
/// index would cost a bounds check there.
#[derive(Copy, Clone)]
pub(crate) struct NfaThread<'p> {
    pub(crate) state: &'p NfaState,
    /// How many bytes a back-reference still owes before `NFA_SKIP` lets the
    /// thread advance.
    pub(crate) count: c_int,
    pub(crate) pim: NfaPim<'p>,
    pub(crate) subs: RegSubs,
}

/// A lookaround postponed until what follows it has proved itself: a
/// "postponed invisible match".
#[derive(Copy, Clone)]
pub(crate) struct NfaPim<'p> {
    pub(crate) result: PimResult,
    /// The lookaround's start state. Meaningless while `result` is
    /// [`PimResult::Unused`].
    pub(crate) state: &'p NfaState,
    pub(crate) subs: RegSubs,
    /// Where the thread stood when the lookaround was postponed, which is
    /// where it has to run from once something settles it.
    pub(crate) end: MatchPos,
}

const BLANK_SUB: RegSub = RegSub {
    in_use: 0,
    list: [Capture {
        start: MatchPos::NOWHERE,
        end: MatchPos::NOWHERE,
    }; NSUBEXP as usize],
    orig_start_col: 0,
};

pub(crate) const BLANK_SUBS: RegSubs = RegSubs {
    norm: BLANK_SUB,
    synt: BLANK_SUB,
};

/// The thread-list storage a program keeps between matches.
///
/// A buffer search runs the program once per line, and each run wants two
/// lists of `nstate + 1` threads, more for every lookaround it nests. Upstream
/// allocates and frees them per run; that was the largest single cost of a
/// search after the walk itself. Here the program keeps a few empty ones.
/// Nothing about a list's *contents* survives: a reused one starts empty, and
/// its slots are blanked as the run first reaches them, as fresh ones are. The
/// slot count 'maxmempattern' is charged for is the list's own, not the
/// storage's capacity, so reuse cannot move E363.
pub(crate) struct SpareLists(Cell<Vec<Vec<NfaThread<'static>>>>);

/// How many spare lists a program keeps, and how large one may be to be
/// kept: a run that grew a list towards 'maxmempattern' frees it, as upstream
/// would.
const SPARE_LISTS: usize = 4;
const SPARE_LIST_BYTES: usize = 256 << 10;

impl SpareLists {
    pub(crate) const fn new() -> SpareLists {
        SpareLists(Cell::new(Vec::new()))
    }

    /// An empty list, reusing a spare one's storage when there is one.
    fn take<'p>(&self) -> Vec<NfaThread<'p>> {
        let mut spare = self.0.take();
        let list = spare.pop().map_or_else(Vec::new, relabel);
        self.0.set(spare);
        list
    }

    /// Keep `list`'s storage for a later [`SpareLists::take`].
    fn put(&self, list: Vec<NfaThread<'_>>) {
        if list.capacity() * size_of::<NfaThread>() > SPARE_LIST_BYTES {
            return;
        }
        let mut spare = self.0.take();
        if spare.len() < SPARE_LISTS {
            spare.push(relabel(list));
        }
        self.0.set(spare);
    }
}

/// `list`, emptied, as a list of threads of another lifetime: the storage
/// changes hands, no thread does. `collect` reuses the allocation in place,
/// as it does whenever the element types have one layout — which two
/// lifetimes of one type always do.
fn relabel<'a, 'b>(mut list: Vec<NfaThread<'a>>) -> Vec<NfaThread<'b>> {
    list.clear();
    list.into_iter()
        .map(|_| unreachable!("the list was emptied"))
        .collect()
}

/// The threads alive at one input position, in the priority order the
/// submatch rules need: the first thread to reach `NFA_MATCH` wins.
pub(crate) struct ThreadList<'t, 'p> {
    /// The slots that have ever been used. Everything below `n` is live;
    /// everything from `n` to the end is a slot a previous character left
    /// behind and this one may reuse.
    threads: Vec<NfaThread<'p>>,
    /// How many threads are live.
    n: usize,
    /// The slot count 'maxmempattern' is charged against — see the module
    /// docs.
    slots: usize,
    /// Which generation of the match this list belongs to. A state records
    /// the last list id it was added to, which is how the walk knows not to
    /// add it twice.
    pub(crate) id: c_int,
    /// Does any thread carry a postponed lookaround? Threads that do cannot
    /// be deduplicated on their state alone.
    pub(crate) has_pim: bool,
    /// The match this list belongs to. A list is built for one match and
    /// discarded with it, so the context is the list's, not each walk's.
    pub(crate) rex: Rex<'t>,
    /// The states of the program running it, which the threads name.
    pub(crate) states: States<'p>,
}

impl<'t, 'p> ThreadList<'t, 'p> {
    /// A list for the match `rex` describes, holding `slots` threads before
    /// it has to grow.
    ///
    /// The storage comes from `spare`, the running program's, and goes back
    /// there with [`ThreadList::release`].
    pub(crate) fn new(
        rex: Rex<'t>,
        states: States<'p>,
        spare: &SpareLists,
        slots: usize,
    ) -> ThreadList<'t, 'p> {
        let mut threads = spare.take();
        threads.reserve_exact(slots);
        ThreadList {
            threads,
            n: 0,
            slots,
            id: 0,
            has_pim: false,
            rex,
            states,
        }
    }

    /// Hand the storage back to `spare` for the program's next match.
    pub(crate) fn release(self, spare: &SpareLists) {
        spare.put(self.threads);
    }

    /// How many threads are live.
    #[inline(always)]
    pub(crate) fn len(&self) -> usize {
        self.n
    }

    /// Drop every thread, keeping the slots for the next character.
    pub(crate) fn clear(&mut self) {
        self.n = 0;
        self.has_pim = false;
    }

    /// The `i`th live thread.
    ///
    /// One bounds check, not two: slicing to `n` first and indexing that is
    /// the natural spelling and costs a second check on the hottest accessor
    /// the match loop has. `n` is never past `threads.len()`.
    #[inline(always)]
    pub(crate) fn thread(&self, i: usize) -> &NfaThread<'p> {
        debug_assert!(i < self.n);
        &self.threads[i]
    }

    /// The `i`th live thread, to write to.
    #[inline(always)]
    pub(crate) fn thread_mut(&mut self, i: usize) -> &mut NfaThread<'p> {
        debug_assert!(i < self.n);
        &mut self.threads[i]
    }

    /// Reserve the next slot size up, or report E363 and refuse.
    fn grow(&mut self) -> bool {
        let newlen = self.slots * 3 / 2 + 50;
        if (((newlen * NFA_THREAD_BYTES) >> 10) as i64) >= p_mmp() {
            emsg(gettext(E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN));
            return false;
        }
        self.threads.reserve_exact(newlen - self.threads.len());
        self.slots = newlen;
        true
    }

    /// Append a thread for `state`, carrying `subs` and `pim`.
    ///
    /// The caller has already made room.
    fn push(&mut self, state: &'p NfaState, subs: &RegSubs, pim: Option<&NfaPim<'p>>) {
        let rex = self.rex;
        if self.n == self.threads.len() {
            // The first character to reach this far pays for the slot; every
            // later one writes only the fields below.
            self.threads.push(NfaThread {
                state,
                count: 0,
                pim: NfaPim {
                    result: PimResult::Unused,
                    state,
                    subs: BLANK_SUBS,
                    end: MatchPos::NOWHERE,
                },
                subs: BLANK_SUBS,
            });
        }
        let has_z = has_zsubexpr(rex);
        self.has_pim |= pim.is_some();
        let thread = &mut self.threads[self.n];
        thread.state = state;
        match pim {
            None => thread.pim.result = PimResult::Unused,
            Some(pim) => copy_pim(rex, &mut thread.pim, pim),
        }
        copy_sub(&mut thread.subs.norm, &subs.norm);
        if has_z {
            copy_sub(&mut thread.subs.synt, &subs.synt);
        }
        self.n += 1;
    }

    /// Is `state` already on this list with these captures?
    ///
    /// The generation check alone is enough unless the pattern has a
    /// back-reference, when two threads on the same state may still differ in
    /// what they captured.
    pub(crate) fn holds(&self, state: StateId, subs: &RegSubs) -> bool {
        let rex = self.rex;
        let state = self.states.state(state);
        let seen = state.lastlist[rex.nfa_ll_index()].get() == self.id;
        seen && (!has_backref(rex) || self.holds_with(state, subs, None))
    }

    /// Is `state` on this list with exactly these captures and this postponed
    /// lookaround?
    fn holds_with(&self, state: &NfaState, subs: &RegSubs, pim: Option<&NfaPim>) -> bool {
        let rex = self.rex;
        let has_z = has_zsubexpr(rex);
        // A plain index walk: at opt-level 0 the iterator adaptors are a
        // handful of calls per thread, and this is the match loop's innermost
        // comparison.
        for i in 0..self.n {
            let thread = &self.threads[i];
            if core::ptr::eq(thread.state, state)
                && sub_equal(rex, &thread.subs.norm, &subs.norm)
                && (!has_z || sub_equal(rex, &thread.subs.synt, &subs.synt))
                && pim_equal(rex, Some(&thread.pim), pim)
            {
                return true;
            }
        }
        false
    }
}

/// Add `state` to `l`, `off` bytes past the input, and follow everything that
/// consumes no input from it.
///
/// `subs` is the capture set the thread arrives with; the bracket states
/// record into it and undo their record on the way back out, so it comes back
/// as it went in. It must not be a capture set that lives in `l` — see
/// [`addstate_here`].
///
/// Returns false when the list could not grow, which the match loop reports
/// as `NFA_TOO_EXPENSIVE`.
pub(crate) fn addstate<'p>(
    l: &mut ThreadList<'_, 'p>,
    state: &'p NfaState,
    subs: &mut RegSubs,
    pim: Option<&NfaPim<'p>>,
    off: c_int,
) -> bool {
    walk(l, state, subs, pim, off, 0)
}

/// Add `state` to the current list in place of the thread at `*ip`.
///
/// `addstate` puts what it adds at the end of the list; this moves it to
/// where the loop has already got to, so that the loop walks it this
/// character rather than never. `*ip` comes back one before the replaced
/// thread, because the loop is about to step over it.
///
/// `subs` must be the caller's own copy of the thread's captures: the shuffle
/// below both reads and rewrites the list, so nothing may point into it.
pub(crate) fn addstate_here<'p>(
    l: &mut ThreadList<'_, 'p>,
    state: &'p NfaState,
    subs: &mut RegSubs,
    pim: Option<&NfaPim<'p>>,
    ip: &mut c_int,
) -> bool {
    let listidx = *ip as usize;
    let before = l.len();
    if !walk(l, state, subs, pim, -(*ip) - ADDSTATE_HERE_OFFSET, 0) {
        return false;
    }
    // The replaced thread was the last one, so whatever was added already
    // follows it.
    if listidx + 1 == before {
        return true;
    }
    let count = l.len() - before;
    if count == 0 {
        return true;
    }

    if count == 1 {
        l.threads[listidx] = l.threads[l.n - 1];
    } else {
        // The C original shuffled through a buffer of `n + count - 1` slots
        // even though the result is one shorter than the list is now, and
        // 'maxmempattern' is charged against that. Rotating in place needs no
        // room at all, but the limit has to fire where it used to.
        if l.n + count > l.slots && !l.grow() {
            return false;
        }
        // What was appended moves to `listidx`, pushing the threads between
        // them along, and the replaced thread falls off the end.
        let n = l.n;
        l.threads[listidx..n].rotate_right(count);
        l.threads[listidx + count..n].rotate_left(1);
    }
    l.n -= 1;
    *ip = listidx as c_int - 1;
    true
}

/// The body of [`addstate`], with the recursion depth it has reached.
fn walk<'p>(
    l: &mut ThreadList<'_, 'p>,
    s: &'p NfaState,
    subs: &mut RegSubs,
    pim: Option<&NfaPim<'p>>,
    off_arg: c_int,
    depth: c_int,
) -> bool {
    let rex = l.rex;
    if depth >= ADDSTATE_MAX_DEPTH {
        return false;
    }

    // A negative `off` past the sentinel means "add at the current position",
    // and carries the index the current list has got to.
    let (add_here, off, listindex) = if off_arg <= -ADDSTATE_HERE_OFFSET {
        (true, 0, (-(off_arg + ADDSTATE_HERE_OFFSET)) as usize)
    } else {
        (false, off_arg, 0)
    };

    let c = NfaOp::try_from(s.c);
    // One match rather than `RangeInclusive::contains`, which is a call per
    // range at opt-level 0 and this runs once per `addstate`.
    let transparent = matches!(
        c,
        Ok(NfaOp::Split
            | NfaOp::Empty
            | NfaOp::Mopen
            | NfaOp::Nclose
            | NfaOp::Zend
            | NfaOp::Mclose
            | NfaOp::Mclose1
            | NfaOp::Mclose2
            | NfaOp::Mclose3
            | NfaOp::Mclose4
            | NfaOp::Mclose5
            | NfaOp::Mclose6
            | NfaOp::Mclose7
            | NfaOp::Mclose8
            | NfaOp::Mclose9
            | NfaOp::Zclose
            | NfaOp::Zclose1
            | NfaOp::Zclose2
            | NfaOp::Zclose3
            | NfaOp::Zclose4
            | NfaOp::Zclose5
            | NfaOp::Zclose6
            | NfaOp::Zclose7
            | NfaOp::Zclose8
            | NfaOp::Zclose9)
    );

    if !transparent {
        // `^` and `\%^` in the middle of a line can never match, and a thread
        // sitting on one would only be walked to be thrown away.
        if matches!(c, Ok(NfaOp::Bol | NfaOp::Bof)) && past_line_start(rex) {
            return true;
        }
        match place(l, s, subs, pim, add_here, listindex) {
            Place::Skip => return true,
            Place::Full => return false,
            Place::Added => {}
        }
    }

    follow(l, c, s.out, s.out1, subs, pim, off, off_arg, depth)
}

/// What [`place`] decided about a state.
enum Place {
    /// It is on the list now.
    Added,
    /// It was already there, so the walk stops here.
    Skip,
    /// The list could not grow.
    Full,
}

/// Put `state` on `l` unless it is already there.
fn place<'p>(
    l: &mut ThreadList<'_, 'p>,
    s: &'p NfaState,
    subs: &RegSubs,
    pim: Option<&NfaPim<'p>>,
    add_here: bool,
    listindex: usize,
) -> Place {
    let rex = l.rex;
    let c = s.c;
    let lastlist = &s.lastlist[rex.nfa_ll_index()];
    let seen = lastlist.get() == l.id;
    let has_backref = has_backref(rex);

    // `NFA_SKIP` counts down the bytes a back-reference still owes, so two
    // threads on it are not the same thread.
    if seen && c != NfaOp::Skip.code() {
        if !has_backref && pim.is_none() && !l.has_pim && c != NfaOp::Match.code() {
            // Without back-references or postponed lookarounds, the state
            // alone identifies the thread. Adding it at the current position
            // is still worth doing when the copy already on the list is
            // behind where the loop has got to.
            let found = add_here
                && (0..l.len().min(listindex)).any(|k| core::ptr::eq(l.thread(k).state, s));
            if !add_here || found {
                return Place::Skip;
            }
        }
        if l.holds_with(s, subs, pim) {
            return Place::Skip;
        }
    }

    if l.len() == l.slots && !l.grow() {
        return Place::Full;
    }
    lastlist.set(l.id);
    l.push(s, subs, pim);
    Place::Added
}

/// Is the input past the start of the line, with something still on it?
///
/// The extra condition is upstream's: inside a multi-line lookaround the
/// `^` may belong to a later line than the one the lookaround started on.
fn past_line_start(rex: Rex) -> bool {
    let endp = rex.nfa_endp();
    rex.input() > 0
        && rex.byte() as c_int != NUL
        && endp.is_none_or(|endp| !rex.multi() || rex.lnum() == endp.as_pos().lnum)
}

/// Follow everything that consumes no input from `state`, recording what the
/// capture brackets on the way say.
///
/// `c` is `state`'s opcode, which [`walk`] has already read: converting it
/// twice per state is measurable in this loop.
#[allow(clippy::too_many_arguments)]
fn follow<'p>(
    l: &mut ThreadList<'_, 'p>,
    c: Result<NfaOp, NotAnOpcode>,
    out: StateId,
    out1: StateId,
    subs: &mut RegSubs,
    pim: Option<&NfaPim<'p>>,
    off: c_int,
    off_arg: c_int,
    depth: c_int,
) -> bool {
    let rex = l.rex;
    let states = l.states;
    match c {
        Ok(NfaOp::Split) => {
            walk(l, states.state(out), subs, pim, off_arg, depth + 1)
                && walk(l, states.state(out1), subs, pim, off_arg, depth + 1)
        }
        Ok(NfaOp::Empty | NfaOp::Nopen | NfaOp::Nclose) => {
            walk(l, states.state(out), subs, pim, off_arg, depth + 1)
        }

        // A capture opens here.
        Ok(NfaOp::Zstart) => open(l, NfaOp::Zstart, out, subs, pim, off, off_arg, depth),
        Ok(marker) if marker.opens_capture() => {
            open(l, marker, out, subs, pim, off, off_arg, depth)
        }

        // The whole match's close, which a `\ze` may already have placed.
        Ok(NfaOp::Mclose) if has_zend_set(rex, subs) => {
            walk(l, states.state(out), subs, pim, off_arg, depth + 1)
        }

        // A capture closes here.
        Ok(NfaOp::Zend) => close(l, NfaOp::Zend, out, subs, pim, off, off_arg, depth),
        Ok(marker) if marker.closes_capture() => {
            close(l, marker, out, subs, pim, off, off_arg, depth)
        }

        // Anything else — including `NfaOp::Match` — ends the walk.
        _ => true,
    }
}

/// Has a `\ze` already put group 0's end somewhere?
fn has_zend_set(rex: Rex, subs: &RegSubs) -> bool {
    rex.nfa_has_zend() != 0 && subs.norm.list[0].end.is_set(rex.pos_kind())
}

/// Which capture set a bracket state records into, and which slot of it.
fn slot_of(c: NfaOp, open: bool) -> (usize, bool) {
    let (run, zrun, whole) = if open {
        (&NfaOp::MOPEN, &NfaOp::ZOPEN, NfaOp::Zstart)
    } else {
        (&NfaOp::MCLOSE, &NfaOp::ZCLOSE, NfaOp::Zend)
    };
    if c == whole {
        // `\zs` and `\ze` move group 0 rather than a group of their own.
        (0, false)
    } else if let Some(slot) = c.index_in(zrun) {
        (slot, true)
    } else {
        (c.index_in(run).expect("a capture bracket"), false)
    }
}

/// A bracket state's snapshot of the slot it is about to write.
///
/// A buffer match snapshots the whole slot; a string match only the one end
/// its state moves. That asymmetry is upstream's and is load-bearing: a
/// buffer match's opening bracket *also* marks the end unset on the way in,
/// so it owes both ends back, where a string match's leaves the end for
/// whatever the walk inside the group records.
enum SavedSlot {
    /// A buffer match's slot, both ends of it.
    Whole(Capture),
    /// A string match's start alone.
    Start(MatchPos),
    /// A string match's end alone.
    End(MatchPos),
}

impl SavedSlot {
    /// Take the snapshot a `\(` is about to overwrite.
    fn of_start(slot: &Capture, kind: PosKind) -> SavedSlot {
        match kind {
            PosKind::Buf => SavedSlot::Whole(*slot),
            PosKind::Str => SavedSlot::Start(slot.start),
        }
    }

    /// Take the snapshot a `\)` is about to overwrite.
    fn of_end(slot: &Capture, kind: PosKind) -> SavedSlot {
        match kind {
            PosKind::Buf => SavedSlot::Whole(*slot),
            PosKind::Str => SavedSlot::End(slot.end),
        }
    }

    /// Put it back.
    fn restore(self, slot: &mut Capture) {
        match self {
            SavedSlot::Whole(was) => *slot = was,
            SavedSlot::Start(was) => slot.start = was,
            SavedSlot::End(was) => slot.end = was,
        }
    }
}

/// What an opening bracket has to put back when the walk comes out of it:
/// either the slot's own snapshot, or — when the slot was past `in_use` —
/// the count itself.
enum Saved {
    Slot(SavedSlot),
    InUse(c_int),
}

/// `\(`, `\%(` and `\zs`: record where the group starts, walk on, and put
/// back what the slot held.
#[allow(clippy::too_many_arguments)]
fn open<'p>(
    l: &mut ThreadList<'_, 'p>,
    c: NfaOp,
    out: StateId,
    subs: &mut RegSubs,
    pim: Option<&NfaPim<'p>>,
    off: c_int,
    off_arg: c_int,
    depth: c_int,
) -> bool {
    let rex = l.rex;
    let kind = rex.pos_kind();
    let (subidx, synt) = slot_of(c, true);

    let sub = if synt { &mut subs.synt } else { &mut subs.norm };
    let saved = if subidx < slots(sub.in_use) {
        Saved::Slot(SavedSlot::of_start(&sub.list[subidx], kind))
    } else {
        // The slots between the last one in use and this one have never been
        // reached, and have to read as unset.
        let was = sub.in_use;
        for slot in &mut sub.list[slots(was)..subidx] {
            slot.mark_unset(kind);
        }
        sub.in_use = group_count(subidx);
        Saved::InUse(was)
    };

    let slot = &mut sub.list[subidx];
    slot.start = rex.at_offset(off);
    if kind == PosKind::Buf {
        // Where a string match leaves the end alone, a buffer match declares
        // it unset until the closing bracket records one.
        slot.end.mark_unset(kind);
    }

    if !walk(l, l.states.state(out), subs, pim, off_arg, depth + 1) {
        return false;
    }

    let sub = if synt { &mut subs.synt } else { &mut subs.norm };
    match saved {
        Saved::Slot(saved) => saved.restore(&mut sub.list[subidx]),
        Saved::InUse(was) => sub.in_use = was,
    }
    true
}

/// `\)` and `\ze`: as [`open`], for the other end of a group. The end is
/// always recorded, whether or not the slot was in use.
#[allow(clippy::too_many_arguments)]
fn close<'p>(
    l: &mut ThreadList<'_, 'p>,
    c: NfaOp,
    out: StateId,
    subs: &mut RegSubs,
    pim: Option<&NfaPim<'p>>,
    off: c_int,
    off_arg: c_int,
    depth: c_int,
) -> bool {
    let rex = l.rex;
    let kind = rex.pos_kind();
    let (subidx, synt) = slot_of(c, false);

    let sub = if synt { &mut subs.synt } else { &mut subs.norm };
    let was_in_use = sub.in_use;
    sub.in_use = was_in_use.max(group_count(subidx));

    let saved = SavedSlot::of_end(&sub.list[subidx], kind);
    sub.list[subidx].end = rex.at_offset(off);

    let ok = walk(l, l.states.state(out), subs, pim, off_arg, depth + 1);
    if ok {
        let sub = if synt { &mut subs.synt } else { &mut subs.norm };
        saved.restore(&mut sub.list[subidx]);
        sub.in_use = was_in_use;
    }
    ok
}

/// The `in_use` a set needs for slot `subidx` to be readable.
fn group_count(subidx: usize) -> c_int {
    c_int::try_from(subidx).unwrap_or(0) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 'maxmempattern' is charged in `NfaThread`s — see the module docs —
    /// so how large one is decides at what thread count E363 is reported,
    /// which a user sees. Four capture sets ride in every thread, so a tag
    /// saying which of the two shapes their positions are in would have cost
    /// thirty-two bytes a thread and moved that point by about four per cent.
    /// It is not in there: a set is its ten captures plus its two `int`s.
    ///
    /// Stated as a claim about the *fields* rather than about
    /// `size_of::<RegSub>()`, so that a layout-randomising build can run
    /// it: `RegSub` is `repr(Rust)`, and a build that shuffles its two
    /// `int`s to opposite sides of the array pads it — which says something
    /// about the compiler's freedom and nothing about this struct. What the
    /// paragraph above actually asserts is that there is no fourth field and
    /// that a capture is a pair of positions, and both survive the shuffle.
    #[test]
    fn a_thread_carries_no_capture_tag() {
        // Exhaustive: a tag field added to `RegSub` stops compiling here.
        let RegSub {
            in_use,
            list,
            orig_start_col,
        } = BLANK_SUB;
        assert_eq!(
            size_of_val(&in_use) + size_of_val(&list) + size_of_val(&orig_start_col),
            size_of::<[Capture; NSUBEXP as usize]>() + 2 * size_of::<c_int>(),
            "a capture set is its ten captures plus its two ints"
        );
        // A capture is two positions and nothing else. An array is exactly
        // its elements, whatever the layout, so this is the comparison that
        // holds under randomisation.
        assert_eq!(
            size_of::<Capture>(),
            size_of::<[MatchPos; 2]>(),
            "a capture carries no tag naming the shape of its positions"
        );
        // And a thread's pair of sets is a pair, not a pair plus a tag.
        let RegSubs { norm, synt } = BLANK_SUBS;
        assert_eq!(
            size_of_val(&norm) + size_of_val(&synt),
            2 * size_of::<RegSub>()
        );
    }

    /// A slot number and the `in_use` that makes it readable are off by one,
    /// in both directions.
    #[test]
    fn a_group_is_in_use_one_past_its_slot() {
        assert_eq!(group_count(0), 1);
        assert_eq!(slots(group_count(9)), 10);
        // `in_use` never names more slots than there are.
        assert_eq!(slots(99), NSUBEXP as usize);
        assert_eq!(slots(-1), 0);
    }
}
