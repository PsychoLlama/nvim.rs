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
//! `nfa_thread_T`. That is the only bound on how many threads a pattern may
//! spawn, so both the step and the byte count are load-bearing: they decide
//! at what point E363 is reported rather than the editor running out of
//! memory. [`ThreadList::slots`] is that count and is deliberately not
//! `Vec::capacity`, which grows by doubling.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::sub::{copy_pim, copy_sub, has_backref, has_zsubexpr, multi_line, pim_equal, sub_equal};
use crate::src::nvim::main::p_mmp;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::regexp::{
    ADDSTATE_HERE_OFFSET, C2Rust_Unnamed_19, C2Rust_Unnamed_20,
    E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN, NFA_BOF, NFA_BOL, NFA_EMPTY, NFA_MATCH,
    NFA_MCLOSE, NFA_MCLOSE1, NFA_MCLOSE9, NFA_MOPEN, NFA_MOPEN9, NFA_NCLOSE, NFA_NOPEN,
    NFA_PIM_UNUSED, NFA_SKIP, NFA_SPLIT, NFA_ZCLOSE, NFA_ZCLOSE9, NFA_ZEND, NFA_ZOPEN, NFA_ZOPEN9,
    NFA_ZSTART, NUL, multipos, nfa_endp, nfa_ll_index, nfa_pim_T, nfa_state_T, nfa_thread_T,
    regsub_T, regsubs_T, rex,
};
use crate::src::nvim::types::{colnr_T, linenr_T, uint8_t};

/// How deep `addstate` may follow itself before giving up. A machine with a
/// cycle of states that consume no input would otherwise not terminate.
const ADDSTATE_MAX_DEPTH: c_int = 5000;

/// The opcode a state carries.
///
/// SAFETY: every state this module is handed is a state of the running
/// program.
fn op(state: *mut nfa_state_T) -> c_int {
    unsafe { (*state).c }
}

/// What a state continues at.
///
/// SAFETY: as `op`.
fn out_of(state: *mut nfa_state_T) -> *mut nfa_state_T {
    unsafe { (*state).out }
}

/// A state's identity, which is what tells two threads on it apart from
/// threads on anything else.
///
/// SAFETY: as `op`.
fn id_of(state: *mut nfa_state_T) -> c_int {
    unsafe { (*state).id }
}

/// A slot that has never held a thread. Written once per slot, the first time
/// the list reaches that far, so that reusing a slot on the next character
/// costs only the fields the C original wrote.
const BLANK_THREAD: nfa_thread_T = nfa_thread_T {
    state: core::ptr::null_mut(),
    count: 0,
    pim: nfa_pim_T {
        result: NFA_PIM_UNUSED,
        state: core::ptr::null_mut(),
        subs: BLANK_SUBS,
        end: C2Rust_Unnamed_20 {
            ptr: core::ptr::null_mut(),
        },
    },
    subs: BLANK_SUBS,
};

const BLANK_SUB: regsub_T = regsub_T {
    in_use: 0,
    list: C2Rust_Unnamed_19 {
        multi: [multipos {
            start_lnum: 0,
            end_lnum: 0,
            start_col: 0,
            end_col: 0,
        }; 10],
    },
    orig_start_col: 0,
};

const BLANK_SUBS: regsubs_T = regsubs_T {
    norm: BLANK_SUB,
    synt: BLANK_SUB,
};

/// The threads alive at one input position, in the priority order the
/// submatch rules need: the first thread to reach `NFA_MATCH` wins.
pub(crate) struct ThreadList {
    /// The slots that have ever been used. Everything below `n` is live;
    /// everything from `n` to the end is a slot a previous character left
    /// behind and this one may reuse.
    threads: Vec<nfa_thread_T>,
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
}

impl ThreadList {
    /// A list that can hold `slots` threads before it has to grow.
    pub(crate) fn new(slots: usize) -> ThreadList {
        ThreadList {
            threads: Vec::with_capacity(slots),
            n: 0,
            slots,
            id: 0,
            has_pim: false,
        }
    }

    /// How many threads are live.
    pub(crate) fn len(&self) -> usize {
        self.n
    }

    /// Drop every thread, keeping the slots for the next character.
    pub(crate) fn clear(&mut self) {
        self.n = 0;
        self.has_pim = false;
    }

    /// The `i`th live thread.
    pub(crate) fn thread(&self, i: usize) -> &nfa_thread_T {
        &self.threads[..self.n][i]
    }

    /// The `i`th live thread, to write to.
    pub(crate) fn thread_mut(&mut self, i: usize) -> &mut nfa_thread_T {
        &mut self.threads[..self.n][i]
    }

    /// Reserve the next slot size up, or report E363 and refuse.
    fn grow(&mut self) -> bool {
        let newlen = self.slots * 3 / 2 + 50;
        if (((newlen * size_of::<nfa_thread_T>()) >> 10) as i64) >= p_mmp.get() {
            // SAFETY: a `&CStr`'s pointer, handed to the message layer.
            unsafe {
                emsg(gettext(
                    E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN.as_ptr(),
                ));
            }
            return false;
        }
        self.threads.reserve_exact(newlen - self.threads.len());
        self.slots = newlen;
        true
    }

    /// Append a thread for `state`, carrying `subs` and `pim`.
    ///
    /// The caller has already made room.
    fn push(&mut self, state: *mut nfa_state_T, subs: &regsubs_T, pim: Option<&nfa_pim_T>) {
        if self.n == self.threads.len() {
            // The first character to reach this far pays for the slot; every
            // later one writes only the fields below.
            self.threads.push(BLANK_THREAD);
        }
        let has_z = has_zsubexpr();
        self.has_pim |= pim.is_some();
        let thread = &mut self.threads[self.n];
        thread.state = state;
        match pim {
            None => thread.pim.result = NFA_PIM_UNUSED,
            Some(pim) => copy_pim(&mut thread.pim, pim),
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
    pub(crate) fn holds(&self, state: *mut nfa_state_T, subs: &regsubs_T) -> bool {
        // SAFETY: `state` is a live state of the running program.
        let seen = unsafe { (*state).lastlist[nfa_ll_index.get() as usize] == self.id };
        seen && (!has_backref() || self.holds_with(state, subs, None))
    }

    /// Is `state` on this list with exactly these captures and this postponed
    /// lookaround?
    fn holds_with(
        &self,
        state: *mut nfa_state_T,
        subs: &regsubs_T,
        pim: Option<&nfa_pim_T>,
    ) -> bool {
        let has_z = has_zsubexpr();
        self.threads[..self.n].iter().any(|thread| {
            id_of(thread.state) == id_of(state)
                && sub_equal(&thread.subs.norm, &subs.norm)
                && (!has_z || sub_equal(&thread.subs.synt, &subs.synt))
                && pim_equal(Some(&thread.pim), pim)
        })
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
pub(crate) fn addstate(
    l: &mut ThreadList,
    state: *mut nfa_state_T,
    subs: &mut regsubs_T,
    pim: Option<&nfa_pim_T>,
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
pub(crate) fn addstate_here(
    l: &mut ThreadList,
    state: *mut nfa_state_T,
    subs: &mut regsubs_T,
    pim: Option<&nfa_pim_T>,
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
        if l.n + count - 1 >= l.slots && !l.grow() {
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
fn walk(
    l: &mut ThreadList,
    state: *mut nfa_state_T,
    subs: &mut regsubs_T,
    pim: Option<&nfa_pim_T>,
    off_arg: c_int,
    depth: c_int,
) -> bool {
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

    let c = op(state);
    let transparent = matches!(c, NFA_SPLIT | NFA_EMPTY | NFA_MOPEN | NFA_NCLOSE | NFA_ZEND)
        || (NFA_MCLOSE..=NFA_MCLOSE9).contains(&c)
        || (NFA_ZCLOSE..=NFA_ZCLOSE9).contains(&c);

    if !transparent {
        // `^` and `\%^` in the middle of a line can never match, and a thread
        // sitting on one would only be walked to be thrown away.
        if matches!(c, NFA_BOL | NFA_BOF) && past_line_start() {
            return true;
        }
        match place(l, state, subs, pim, add_here, listindex) {
            Place::Skip => return true,
            Place::Full => return false,
            Place::Added => {}
        }
    }

    follow(l, state, subs, pim, off, off_arg, depth)
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
fn place(
    l: &mut ThreadList,
    state: *mut nfa_state_T,
    subs: &regsubs_T,
    pim: Option<&nfa_pim_T>,
    add_here: bool,
    listindex: usize,
) -> Place {
    // SAFETY: `state` is a live state of the running program.
    let c = op(state);
    // SAFETY: `state` is a live state of the running program.
    let seen = unsafe { (*state).lastlist[nfa_ll_index.get() as usize] == l.id };
    let has_backref = has_backref();

    // `NFA_SKIP` counts down the bytes a back-reference still owes, so two
    // threads on it are not the same thread.
    if seen && c != NFA_SKIP {
        if !has_backref && pim.is_none() && !l.has_pim && c != NFA_MATCH {
            // Without back-references or postponed lookarounds, the state
            // alone identifies the thread. Adding it at the current position
            // is still worth doing when the copy already on the list is
            // behind where the loop has got to.
            let found = add_here
                && (0..l.len().min(listindex)).any(|k| id_of(l.thread(k).state) == id_of(state));
            if !add_here || found {
                return Place::Skip;
            }
        }
        if l.holds_with(state, subs, pim) {
            return Place::Skip;
        }
    }

    if l.len() == l.slots && !l.grow() {
        return Place::Full;
    }
    // SAFETY: `state` is a live state of the running program.
    unsafe {
        (*state).lastlist[nfa_ll_index.get() as usize] = l.id;
    }
    l.push(state, subs, pim);
    Place::Added
}

/// Is the input past the start of the line, with something still on it?
///
/// The extra condition is upstream's: inside a multi-line lookaround the
/// `^` may belong to a later line than the one the lookaround started on.
fn past_line_start() -> bool {
    // SAFETY: `rex` describes a live match and `nfa_endp` the position a
    // lookaround was told to stop at, when there is one.
    unsafe {
        let endp = nfa_endp.get();
        (*rex.ptr()).input > (*rex.ptr()).line
            && *(*rex.ptr()).input as c_int != NUL
            && (endp.is_null()
                || !(*rex.ptr()).reg_match.is_null()
                || (*rex.ptr()).lnum == (*endp).se_u.pos.lnum)
    }
}

/// Follow everything that consumes no input from `state`, recording what the
/// capture brackets on the way say.
fn follow(
    l: &mut ThreadList,
    state: *mut nfa_state_T,
    subs: &mut regsubs_T,
    pim: Option<&nfa_pim_T>,
    off: c_int,
    off_arg: c_int,
    depth: c_int,
) -> bool {
    let (c, out) = (op(state), out_of(state));
    // SAFETY: `state` is a live state of the running program.
    let out1 = unsafe { (*state).out1 };
    match c {
        NFA_SPLIT => {
            walk(l, out, subs, pim, off_arg, depth + 1)
                && walk(l, out1, subs, pim, off_arg, depth + 1)
        }
        NFA_EMPTY | NFA_NOPEN | NFA_NCLOSE => walk(l, out, subs, pim, off_arg, depth + 1),

        // A capture opens here.
        NFA_MOPEN..=NFA_MOPEN9 | NFA_ZOPEN..=NFA_ZOPEN9 | NFA_ZSTART => {
            open(l, state, subs, pim, off, off_arg, depth)
        }

        // The whole match's close, which a `\ze` may already have placed.
        NFA_MCLOSE if has_zend_set(subs) => walk(l, out, subs, pim, off_arg, depth + 1),

        // A capture closes here.
        NFA_MCLOSE | NFA_MCLOSE1..=NFA_MCLOSE9 | NFA_ZCLOSE..=NFA_ZCLOSE9 | NFA_ZEND => {
            close(l, state, subs, pim, off, off_arg, depth)
        }

        // Anything else — including `NFA_MATCH` — ends the walk.
        _ => true,
    }
}

/// Has a `\ze` already put group 0's end somewhere?
fn has_zend_set(subs: &regsubs_T) -> bool {
    // SAFETY: `rex` describes a live match; which arm of the capture union is
    // live is `multi_line`.
    unsafe {
        (*rex.ptr()).nfa_has_zend != 0
            && if multi_line() {
                subs.norm.list.multi[0].end_lnum >= 0
            } else {
                !subs.norm.list.line[0].end.is_null()
            }
    }
}

/// Which capture set a bracket state records into, and which slot of it.
fn slot_of(c: c_int, open: bool) -> (usize, bool) {
    let (first, zfirst, whole) = if open {
        (NFA_MOPEN, NFA_ZOPEN, NFA_ZSTART)
    } else {
        (NFA_MCLOSE, NFA_ZCLOSE, NFA_ZEND)
    };
    if c == whole {
        // `\zs` and `\ze` move group 0 rather than a group of their own.
        (0, false)
    } else if (zfirst..=zfirst + 9).contains(&c) {
        ((c - zfirst) as usize, true)
    } else {
        ((c - first) as usize, false)
    }
}

/// What a capture slot held before a bracket state overwrote it, in whichever
/// of the two shapes this match uses.
enum SavedPos {
    Multi(multipos),
    Line(*mut uint8_t),
}

/// What an opening bracket has to put back when the walk comes out of it:
/// either the position the slot already held, or — when the slot was past
/// `in_use` — the count itself.
enum Saved {
    Pos(SavedPos),
    InUse(c_int),
}

/// `\(`, `\%(` and `\zs`: record where the group starts, walk on, and put
/// back what the slot held.
fn open(
    l: &mut ThreadList,
    state: *mut nfa_state_T,
    subs: &mut regsubs_T,
    pim: Option<&nfa_pim_T>,
    off: c_int,
    off_arg: c_int,
    depth: c_int,
) -> bool {
    let (c, out) = (op(state), out_of(state));
    let (subidx, synt) = slot_of(c, true);
    let multi = multi_line();

    let sub = if synt { &mut subs.synt } else { &mut subs.norm };
    let saved = if subidx < sub.in_use as usize {
        // SAFETY: which arm of the union is live is `multi_line`.
        unsafe {
            Saved::Pos(if multi {
                SavedPos::Multi(sub.list.multi[subidx])
            } else {
                SavedPos::Line(sub.list.line[subidx].start)
            })
        }
    } else {
        // The slots between the last one in use and this one have never been
        // reached, and have to read as unset.
        let was = sub.in_use;
        // SAFETY: as above.
        unsafe {
            for i in was as usize..subidx {
                if multi {
                    sub.list.multi[i].start_lnum = -1;
                    sub.list.multi[i].end_lnum = -1;
                } else {
                    sub.list.line[i].start = core::ptr::null_mut();
                    sub.list.line[i].end = core::ptr::null_mut();
                }
            }
        }
        sub.in_use = subidx as c_int + 1;
        Saved::InUse(was)
    };

    // SAFETY: as above; `rex` describes a live match.
    unsafe {
        if multi {
            let slot = &mut sub.list.multi[subidx];
            if off == -1 {
                // The thread is about to cross a line break, so the group
                // starts at the beginning of the next line.
                slot.start_lnum = (*rex.ptr()).lnum + 1 as linenr_T;
                slot.start_col = 0;
            } else {
                slot.start_lnum = (*rex.ptr()).lnum;
                slot.start_col =
                    ((*rex.ptr()).input.offset_from((*rex.ptr()).line) + off as isize) as colnr_T;
            }
            slot.end_lnum = -1;
        } else {
            sub.list.line[subidx].start = (*rex.ptr()).input.offset(off as isize);
        }
    }

    if !walk(l, out, subs, pim, off_arg, depth + 1) {
        return false;
    }

    let sub = if synt { &mut subs.synt } else { &mut subs.norm };
    // SAFETY: as above.
    unsafe {
        match saved {
            Saved::Pos(SavedPos::Multi(pos)) => sub.list.multi[subidx] = pos,
            Saved::Pos(SavedPos::Line(start)) => sub.list.line[subidx].start = start,
            Saved::InUse(was) => sub.in_use = was,
        }
    }
    true
}

/// `\)` and `\ze`: as [`open`], for the other end of a group. The end is
/// always recorded, whether or not the slot was in use.
fn close(
    l: &mut ThreadList,
    state: *mut nfa_state_T,
    subs: &mut regsubs_T,
    pim: Option<&nfa_pim_T>,
    off: c_int,
    off_arg: c_int,
    depth: c_int,
) -> bool {
    let (c, out) = (op(state), out_of(state));
    let (subidx, synt) = slot_of(c, false);
    let multi = multi_line();

    let sub = if synt { &mut subs.synt } else { &mut subs.norm };
    let was_in_use = sub.in_use;
    if sub.in_use <= subidx as c_int {
        sub.in_use = subidx as c_int + 1;
    }
    // SAFETY: which arm of the union is live is `multi_line`; `rex` describes
    // a live match.
    let saved = unsafe {
        if multi {
            let was = sub.list.multi[subidx];
            let slot = &mut sub.list.multi[subidx];
            if off == -1 {
                slot.end_lnum = (*rex.ptr()).lnum + 1 as linenr_T;
                slot.end_col = 0;
            } else {
                slot.end_lnum = (*rex.ptr()).lnum;
                slot.end_col =
                    ((*rex.ptr()).input.offset_from((*rex.ptr()).line) + off as isize) as colnr_T;
            }
            SavedPos::Multi(was)
        } else {
            let was = sub.list.line[subidx].end;
            sub.list.line[subidx].end = (*rex.ptr()).input.offset(off as isize);
            SavedPos::Line(was)
        }
    };

    let ok = walk(l, out, subs, pim, off_arg, depth + 1);
    if ok {
        let sub = if synt { &mut subs.synt } else { &mut subs.norm };
        // SAFETY: as above.
        unsafe {
            match saved {
                SavedPos::Multi(pos) => sub.list.multi[subidx] = pos,
                SavedPos::Line(end) => sub.list.line[subidx].end = end,
            }
        }
        sub.in_use = was_in_use;
    }
    ok
}
