//! Who owns an undo header, and how one header names another.
//!
//! Every `u_header_T` a buffer has lives in that buffer's [`UndoStore`],
//! keyed by the header's `uh_seq`. A [`UndoLink`] — the type all four
//! `uh_next`/`uh_prev`/`uh_alt_next`/`uh_alt_prev` fields and all three
//! `b_u_*head` fields now have — is that same sequence number, so following
//! a link is a lookup in the store and *not* a pointer dereference. Three
//! things fall out of that:
//!
//! - The undo file's link fields need no conversion in either direction.
//!   `put_header_link` writes the number the link already holds, and
//!   `unserialize_uhp` reads it straight into the link. The union whose two
//!   arms were "sequence number while on disk" and "pointer while in memory"
//!   had nothing to distinguish: there was only ever the number.
//! - A link cannot dangle. A header that has been freed is gone from the
//!   store, so a stale link resolves to "no header" instead of to freed
//!   memory.
//! - Two live headers cannot share a sequence number, because the store
//!   would not hold both. `insert` says so by handing back whatever it
//!   displaced.
//!
//! Headers are still individually `xmalloc`ed rather than held inline: the
//! tree still walks a header through a `*mut u_header_T` in the places that
//! free one, and a stable address is what lets the store change underneath
//! them. [`Header`] is that view, and [`crate::winlayer::Buf::header`] is
//! the only way to get one.
//!
//! The tree itself is walked in exactly two shapes, and both live here:
//! [`header_chain`] follows one link field until it names nothing, and
//! [`TreeWalk`] is the depth-first pass over the *whole* tree that
//! `:undolist`, `:earlier`/`:later` and the undo-file writer each used to
//! spell out for themselves.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_int, c_void};
use core::ptr::NonNull;
use std::collections::HashMap;

use crate::memory::xfree;
use crate::types::{UndoLink, u_header_T};
use crate::winlayer::Buf;

use super::lastmark;

/// Every undo header one buffer owns, keyed by `uh_seq`.
///
/// The store owns the allocations, not the tree shape: unlinking a header
/// from `uh_prev` and friends does not free it, and a header the tree has
/// dropped (`u_clearall` does exactly that for a command preview) stays here
/// until something asks for it to go.
#[derive(Default)]
pub struct UndoStore {
    headers: HashMap<c_int, NonNull<u_header_T>>,
}

impl UndoStore {
    /// An empty store.
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    /// How many headers the store holds. Not `b_u_numhead`, which counts the
    /// headers *the tree* holds and is what `'undolevels'` is measured
    /// against.
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Whether the store holds no headers at all.
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// Whether `link` names a header this store holds.
    pub fn contains(&self, link: UndoLink) -> bool {
        self.headers.contains_key(&link.seq())
    }

    /// The header `link` names, or NULL.
    ///
    /// Safe because handing back an address is not reading through it; the
    /// caller's dereference is where the obligation starts.
    pub fn at(&self, link: UndoLink) -> *mut u_header_T {
        if link.is_none() {
            // Not a lookup: "no link" is not a key, and refusing it here is
            // what keeps a store that somehow held key 0 from answering it.
            return core::ptr::null_mut();
        }
        match self.headers.get(&link.seq()) {
            Some(uhp) => uhp.as_ptr(),
            None => core::ptr::null_mut(),
        }
    }

    /// Takes ownership of `uhp` under sequence number `seq`, and hands back
    /// the header that number named before, if any.
    ///
    /// A returned `Some` is a duplicate sequence number, which for a header
    /// read out of a file is corruption and for one the editor just built is
    /// a bug. The displaced header is *not* freed: the caller knows whether
    /// its entries have been freed yet and this does not.
    pub fn insert(&mut self, seq: c_int, uhp: NonNull<u_header_T>) -> Option<NonNull<u_header_T>> {
        debug_assert!(seq > 0, "an undo header's sequence number is positive");
        self.headers.insert(seq, uhp)
    }

    /// Gives up ownership of the header `link` names, and hands it back.
    pub fn take(&mut self, link: UndoLink) -> Option<NonNull<u_header_T>> {
        self.headers.remove(&link.seq())
    }

    /// The links to every header the store holds, in no particular order.
    /// Only diagnostics and teardown want this; the tree is walked by link.
    pub fn links(&self) -> impl Iterator<Item = UndoLink> + '_ {
        self.headers.keys().copied().map(UndoLink::to_seq)
    }
}

/// A live undo header, wrapped so that reading and writing its fields is not
/// an unsafe operation at every use.
///
/// This is the trade [`crate::winlayer::Buf`] makes for `buf_T`, for the same
/// reason. The pointer has to stay raw — the store hands it back, the tree
/// walks interleave it with reads of the buffer's own link fields, and a
/// long-lived `&mut` would invalidate a view the caller still holds — but the
/// *dereference* does not: constructing the wrapper is the unsafe step, and
/// from there `Deref`/`DerefMut` give ordinary field access.
///
/// The promise a `Header` carries is the store's own invariant: a header that
/// has been freed is gone from the store, so [`Buf::header`] either hands back
/// a live header or hands back nothing.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Header(*mut u_header_T);

impl core::ops::Deref for Header {
    type Target = u_header_T;

    #[inline(always)]
    fn deref(&self) -> &u_header_T {
        // SAFETY: the constructor's promise — a live header.
        unsafe { &*self.0 }
    }
}

impl core::ops::DerefMut for Header {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut u_header_T {
        // SAFETY: the constructor's promise — a live header. The borrow lasts
        // only as long as the field access that asked for it.
        unsafe { &mut *self.0 }
    }
}

impl Header {
    /// Wraps a header the store handed back, or nothing for a NULL one.
    ///
    /// # Safety
    ///
    /// `uhp` is NULL, or points at a header that stays live for as long as
    /// the value is used.
    #[inline(always)]
    pub(crate) const unsafe fn new(uhp: *mut u_header_T) -> Option<Self> {
        if uhp.is_null() { None } else { Some(Self(uhp)) }
    }

    /// The address the store holds, for the callers that still want one.
    #[inline(always)]
    pub(crate) fn raw(self) -> *mut u_header_T {
        self.0
    }

    /// The link that names this header.
    #[inline(always)]
    pub(crate) fn link(self) -> UndoLink {
        UndoLink::to_seq(self.uh_seq)
    }

    /// Whether a tree walk has stamped this header with neither of the two
    /// marks it is using. A walk that only has one mark passes it twice.
    #[inline(always)]
    pub(crate) fn unwalked(self, mark: c_int, nomark: c_int) -> bool {
        self.uh_walk != mark && self.uh_walk != nomark
    }
}

impl Buf {
    /// The header `link` names in this buffer's undo store, if any.
    ///
    /// Safe, where a bare lookup through a `*mut buf_T` would not be: a
    /// [`Buf`] already carries the promise that the buffer is live, and that
    /// is the whole of the lookup's obligation — a link that names nothing,
    /// or names a header that has been freed, resolves to `None`.
    #[inline]
    pub(crate) fn header(self, link: UndoLink) -> Option<Header> {
        // SAFETY: a live buffer, by `Buf`'s own contract, and the store hands
        // back only headers it still holds.
        unsafe { Header::new(header_at(self, link)) }
    }

    /// A depth-first walk of this buffer's whole undo tree, from the header
    /// `start` names. See [`TreeWalk`].
    #[inline]
    pub(crate) fn tree_walk(self, start: UndoLink, marks: Marks) -> TreeWalk {
        TreeWalk {
            buf: self,
            marks,
            stop_above: UndoLink::NONE,
            state: WalkState::Start(start),
            depth: 1,
        }
    }
}

/// The header `link` names in `buf`'s store, or NULL.
///
/// Safe: a [`Buf`] already carries the promise the lookup needs, and a link
/// that names nothing — or a header that has been freed — resolves to NULL.
fn header_at(buf: Buf, link: UndoLink) -> *mut u_header_T {
    // SAFETY: a live buffer, by `Buf`'s own contract; `b_u_store` is NULL or
    // a store this module allocated and nothing else writes it, and the
    // borrow does not leave this statement.
    match unsafe { store_of(buf) } {
        Some(store) => store.at(link),
        None => core::ptr::null_mut(),
    }
}

/// The pair of stamps one pass over the tree leaves behind: `mark` on a
/// header the walk has reached, `nomark` on one it has finished backing out
/// of. Numbers no walk has used before, so the stamps a previous pass left
/// cannot be mistaken for this one's.
///
/// A walk that does not care about the difference — the undo-file writer only
/// asks "have I written this header yet" — passes one number as both.
#[derive(Clone, Copy)]
pub(crate) struct Marks {
    pub(crate) mark: c_int,
    pub(crate) nomark: c_int,
}

impl Marks {
    /// Two fresh stamps.
    pub(crate) fn next() -> Self {
        Self {
            mark: next_mark(),
            nomark: next_mark(),
        }
    }

    /// One fresh stamp, used for both: "reached" and "finished with" are the
    /// same answer to a walk that visits every header exactly once.
    pub(crate) fn next_once() -> Self {
        let mark = next_mark();
        Self { mark, nomark: mark }
    }

    /// Whether the walk has yet to reach `link`'s header — and that header,
    /// when it has not.
    pub(crate) fn unwalked(self, buf: Buf, link: UndoLink) -> Option<Header> {
        buf.header(link)
            .filter(|uh| uh.unwalked(self.mark, self.nomark))
    }
}

/// The next unused tree-walk mark.
fn next_mark() -> c_int {
    let mark = lastmark.get() + 1;
    lastmark.set(mark);
    mark
}

/// One header, as a depth-first [`TreeWalk`] arrives at it.
pub(crate) struct Visit {
    /// The header itself, already stamped with the walk's `mark`.
    pub(crate) header: Header,
    /// Whether the walk had not stamped this header before now. A tree whose
    /// links run in four directions is backed out of through headers it has
    /// already seen, so this is what "each header once" means.
    pub(crate) first: bool,
    /// How many changes down the branch this header is, counting the header
    /// the walk started from as 1. `:undolist` prints it.
    pub(crate) depth: c_int,
}

/// A depth-first pass over one buffer's whole undo tree.
///
/// The undo tree is not a tree a plain recursion can walk: `uh_prev` goes
/// down a branch, `uh_alt_next`/`uh_alt_prev` across a run of alternates and
/// `uh_next` back up, and the same header is reached from several of those.
/// What keeps the pass finite is the pair of stamps in [`Marks`] — `mark` on
/// arrival, `nomark` on the way back out — which is why the walk owns them.
///
/// Three callers spelled this out for themselves before: `:undolist`, the
/// `:earlier`/`:later` search and the undo-file writer. They differ only in
/// what they do at each header, plus one flag ([`TreeWalk::stopping_above`])
/// the search needs.
pub(crate) struct TreeWalk {
    buf: Buf,
    marks: Marks,
    stop_above: UndoLink,
    state: WalkState,
    depth: c_int,
}

enum WalkState {
    Start(UndoLink),
    At(Header),
    Done,
}

impl TreeWalk {
    /// Un-stamps the header this link names when the walk leaves it going
    /// *up*, so that a later pass treats it as unreached.
    ///
    /// The `:earlier`/`:later` search passes its `b_u_curhead`: the change
    /// the cursor already sits above is not one the move goes through, and
    /// leaving it stamped would make the walk up stop there.
    pub(crate) fn stopping_above(mut self, link: UndoLink) -> Self {
        self.stop_above = link;
        self
    }

    /// Where the walk goes after `uh`, stamping `uh` with `nomark` when it is
    /// backing out of it rather than descending further.
    fn advance(&mut self, mut uh: Header) -> Option<Header> {
        if let Some(down) = self.marks.unwalked(self.buf, uh.uh_prev) {
            // Down into the branch.
            self.depth += 1;
            return Some(down);
        }
        if let Some(across) = self.marks.unwalked(self.buf, uh.uh_alt_next) {
            // Or across into an alternate branch, at the same depth.
            return Some(across);
        }
        if uh.uh_alt_prev.is_none()
            && let Some(up) = self.marks.unwalked(self.buf, uh.uh_next)
        {
            // Or up, but only from the start of a run of alternates.
            if uh.link() == self.stop_above {
                uh.uh_walk = self.marks.nomark;
            }
            self.depth -= 1;
            return Some(up);
        }
        // A dead end: stamp it finished and back out.
        uh.uh_walk = self.marks.nomark;
        if uh.uh_alt_prev.is_some() {
            return self.buf.header(uh.uh_alt_prev);
        }
        self.depth -= 1;
        self.buf.header(uh.uh_next)
    }
}

impl Iterator for TreeWalk {
    type Item = Visit;

    fn next(&mut self) -> Option<Visit> {
        let here = match self.state {
            WalkState::Done => return None,
            WalkState::Start(link) => self.buf.header(link),
            // The step is taken here rather than after the last header was
            // handed out, so that a caller who stops early leaves the tree
            // stamped exactly as far as it walked.
            WalkState::At(uh) => self.advance(uh),
        };
        let Some(mut uh) = here else {
            self.state = WalkState::Done;
            return None;
        };
        let first = uh.unwalked(self.marks.mark, self.marks.nomark);
        uh.uh_walk = self.marks.mark;
        self.state = WalkState::At(uh);
        Some(Visit {
            header: uh,
            first,
            depth: self.depth,
        })
    }
}

/// `buf`'s store, if it has one yet.
///
/// # Safety
///
/// The borrow must not outlive the statement that takes it — every caller
/// here keeps it to one expression.
unsafe fn store_of<'a>(buf: Buf) -> Option<&'a mut UndoStore> {
    // SAFETY: a live buffer, by `Buf`'s contract, so the field read is in
    // bounds; the pointer it holds is NULL or a leaked `Box<UndoStore>` from
    // `store_for`.
    unsafe { (*buf.raw()).b_u_store.as_mut() }
}

/// `buf`'s store, created if this is the buffer's first header.
///
/// # Safety
///
/// The borrow must not outlive the statement that takes it, as for
/// [`store_of`].
unsafe fn store_for<'a>(mut buf: Buf) -> &'a mut UndoStore {
    if buf.b_u_store.is_null() {
        buf.b_u_store = Box::into_raw(Box::new(UndoStore::new()));
    }
    // SAFETY: non-null now, and what it points at is a `Box` this module
    // leaked; the borrow does not leave the caller's statement.
    unsafe { &mut *buf.b_u_store }
}

/// Hands `uhp` to `buf`'s store and returns the link that now names it.
///
/// The header's `uh_seq` must already be set: it is the key.
///
/// # Safety
///
/// `uhp` points at a live header the store does not already own.
pub(crate) unsafe fn header_adopt(buf: Buf, uhp: *mut u_header_T) -> UndoLink {
    // SAFETY: a live header by the contract above.
    let seq = unsafe { (*uhp).uh_seq };
    debug_assert!(seq > 0, "an undo header's sequence number is positive");
    let Some(uhp) = NonNull::new(uhp).filter(|_| seq > 0) else {
        // Nothing can name a header whose number is not one a buffer hands
        // out, so it would be unreachable in the store as well as leaked.
        return UndoLink::NONE;
    };
    // SAFETY: the borrow does not leave this statement.
    let displaced = unsafe { store_for(buf) }.insert(seq, uhp);
    debug_assert!(
        displaced.is_none(),
        "two live undo headers claimed sequence number {seq}"
    );
    UndoLink::to_seq(seq)
}

/// Drops `uhp` from `buf`'s store and frees it.
///
/// Frees the header itself and nothing it points at: `u_freeentries` has
/// already dealt with the entry list and the extmark vector by the time it
/// calls here.
///
/// # Safety
///
/// `uhp` points at a live header allocated by `xmalloc`, and nothing else
/// holds a pointer to that header.
pub(crate) unsafe fn header_free(buf: Buf, uhp: *mut u_header_T) {
    // SAFETY: a live header by the contract above.
    let link = UndoLink::to_seq(unsafe { (*uhp).uh_seq });
    // SAFETY: the borrow does not leave this statement.
    if let Some(store) = unsafe { store_of(buf) } {
        let dropped = store.take(link);
        debug_assert!(
            dropped.is_none_or(|held| core::ptr::eq(held.as_ptr(), uhp)),
            "the undo store held a different header under sequence number {}",
            link.seq()
        );
    }
    // SAFETY: the store no longer names this header and the caller promises
    // nobody else does either; it came from `xmalloc`.
    unsafe { xfree(uhp.cast::<c_void>()) };
}

/// Yields the headers along one chain: the one `start` names, then whatever
/// `step` names from each header in turn, until a link names nothing.
///
/// `step` is read only when the *next* header is asked for, so a body that
/// relinks the header it was handed still steers the walk — which is what
/// the `while` loops this replaced did. A body that *frees* the header it
/// was handed must not use this: the step would read freed memory.
///
/// # Safety
///
/// Nothing frees a header the walk has already visited.
pub(crate) unsafe fn header_chain(
    buf: Buf,
    start: UndoLink,
    step: fn(&u_header_T) -> UndoLink,
) -> HeaderChain {
    HeaderChain {
        buf,
        state: ChainState::Start(start),
        step,
    }
}

/// The iterator [`header_chain`] returns.
pub(crate) struct HeaderChain {
    buf: Buf,
    state: ChainState,
    step: fn(&u_header_T) -> UndoLink,
}

enum ChainState {
    Start(UndoLink),
    At(Header),
    Done,
}

impl Iterator for HeaderChain {
    type Item = Header;

    fn next(&mut self) -> Option<Header> {
        let link = match self.state {
            ChainState::Done => return None,
            ChainState::Start(link) => link,
            ChainState::At(uh) => (self.step)(&uh),
        };
        // SAFETY: nothing has freed an already-visited header, per
        // `header_chain`'s contract.
        let Some(uh) = (unsafe { Header::new(header_at(self.buf, link)) }) else {
            self.state = ChainState::Done;
            return None;
        };
        self.state = ChainState::At(uh);
        Some(uh)
    }
}

/// Drops `buf`'s store when nothing is left in it.
///
/// Called at the end of `u_blockfree`, which is the one place that knows the
/// tree has just been walked and freed. A store that still holds something
/// is kept: a command preview detaches the buffer's real tree from
/// `b_u_*head` with `u_clearall` and restores it afterwards, and those
/// headers are only reachable through the store in between.
///
/// Safe: a [`Buf`] carries the whole of the promise this needs.
pub(crate) fn store_release(mut buf: Buf) {
    if buf.b_u_store.is_null() {
        return;
    }
    // SAFETY: non-null, and what it points at is a `Box` this module leaked
    // and nothing else points at.
    if unsafe { (*buf.b_u_store).is_empty() } {
        unsafe { drop(Box::from_raw(buf.b_u_store)) };
        buf.b_u_store = core::ptr::null_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A header carrying `seq` and nothing else, owned by the caller.
    fn header(seq: c_int) -> NonNull<u_header_T> {
        let uh = u_header_T {
            uh_seq: seq,
            ..Default::default()
        };
        NonNull::from(Box::leak(Box::new(uh)))
    }

    fn release(uhp: NonNull<u_header_T>) {
        // SAFETY: `header` made this with `Box::leak` and the store only ever
        // hands the same pointer back.
        drop(unsafe { Box::from_raw(uhp.as_ptr()) });
    }

    #[test]
    fn a_link_is_the_sequence_number_and_zero_is_no_link() {
        assert_eq!(UndoLink::NONE.seq(), 0);
        assert!(UndoLink::NONE.is_none());
        assert!(!UndoLink::NONE.is_some());
        assert_eq!(UndoLink::to_seq(7).seq(), 7);
        assert!(UndoLink::to_seq(7).is_some());
        // Nothing a buffer could have handed out, so: no link. A corrupt
        // undo file's link field lands here.
        assert_eq!(UndoLink::to_seq(0), UndoLink::NONE);
        assert_eq!(UndoLink::to_seq(-1), UndoLink::NONE);
        assert_eq!(UndoLink::default(), UndoLink::NONE);
    }

    #[test]
    fn an_empty_store_names_no_header() {
        let store = UndoStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert!(!store.contains(UndoLink::to_seq(1)));
        assert!(store.at(UndoLink::to_seq(1)).is_null());
        assert!(store.at(UndoLink::NONE).is_null());
    }

    #[test]
    fn a_header_is_found_by_the_number_it_was_stored_under() {
        let mut store = UndoStore::new();
        let first = header(1);
        let second = header(2);
        assert!(store.insert(1, first).is_none());
        assert!(store.insert(2, second).is_none());
        assert_eq!(store.len(), 2);
        assert!(core::ptr::eq(store.at(UndoLink::to_seq(1)), first.as_ptr()));
        assert!(core::ptr::eq(
            store.at(UndoLink::to_seq(2)),
            second.as_ptr()
        ));
        // A number nobody stored, and the "no link" number, both name
        // nothing rather than something arbitrary.
        assert!(store.at(UndoLink::to_seq(3)).is_null());
        assert!(store.at(UndoLink::NONE).is_null());
        release(first);
        release(second);
    }

    #[test]
    fn a_header_taken_out_stops_being_found() {
        let mut store = UndoStore::new();
        let only = header(4);
        store.insert(4, only);
        let taken = store
            .take(UndoLink::to_seq(4))
            .expect("the header is there");
        assert!(core::ptr::eq(taken.as_ptr(), only.as_ptr()));
        // The link that named it now names nothing: this is why a stale
        // link cannot be a dangling pointer.
        assert!(store.at(UndoLink::to_seq(4)).is_null());
        assert!(store.is_empty());
        // And taking it again is harmless, which is what makes a double
        // free in the tree walk a no-op instead of a crash.
        assert!(store.take(UndoLink::to_seq(4)).is_none());
        release(only);
    }

    #[test]
    fn two_headers_cannot_share_a_sequence_number() {
        let mut store = UndoStore::new();
        let first = header(9);
        let second = header(9);
        assert!(store.insert(9, first).is_none());
        let displaced = store.insert(9, second).expect("the first one comes back");
        assert!(core::ptr::eq(displaced.as_ptr(), first.as_ptr()));
        assert_eq!(store.len(), 1);
        assert!(core::ptr::eq(
            store.at(UndoLink::to_seq(9)),
            second.as_ptr()
        ));
        release(first);
        release(second);
    }

    #[test]
    fn links_names_every_header_held() {
        let mut store = UndoStore::new();
        let headers: Vec<NonNull<u_header_T>> = (1..=3).map(header).collect();
        for (i, uhp) in headers.iter().enumerate() {
            let seq = c_int::try_from(i).expect("three fits") + 1;
            store.insert(seq, *uhp);
        }
        let mut seqs: Vec<c_int> = store.links().map(UndoLink::seq).collect();
        seqs.sort_unstable();
        assert_eq!(seqs, vec![1, 2, 3]);
        for uhp in headers {
            release(uhp);
        }
    }

    #[test]
    fn a_fresh_header_is_linked_to_nothing() {
        let uh = u_header_T::default();
        assert!(uh.uh_next.is_none());
        assert!(uh.uh_prev.is_none());
        assert!(uh.uh_alt_next.is_none());
        assert!(uh.uh_alt_prev.is_none());
        assert_eq!(uh.uh_seq, 0);
        assert!(uh.uh_entry.is_null());
        assert!(uh.uh_extmark.items.is_null());
        assert_eq!(uh.uh_extmark.size, 0);
    }
}
