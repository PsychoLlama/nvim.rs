//! Who owns an undo header, and how one header names another.
//!
//! Every `UndoHeader` a buffer has lives in that buffer's [`UndoStore`],
//! keyed by the header's `seq`. A [`UndoLink`] — the type all four
//! `next`/`prev`/`alt_next`/`alt_prev` fields and all three `b_u_*head`
//! fields have — is that same sequence number, so following a link is a
//! lookup in the store and *not* a pointer dereference. Three things fall out
//! of that:
//!
//! - The undo file's link fields need no conversion in either direction.
//!   `put_header_link` writes the number the link already holds, and
//!   `unserialize_uhp` reads it straight into the link.
//! - A link cannot dangle. A header that has been freed is gone from the
//!   store, so a stale link resolves to "no header" instead of to freed
//!   memory.
//! - Two live headers cannot share a sequence number, because the store
//!   would not hold both. `insert` says so by handing back whatever it
//!   displaced.
//!
//! A header is lent out only for as long as the borrow of the buffer it was
//! looked up through: [`Buf::header`] and [`Buf::header_mut`]. Nothing holds
//! one across a call that can reach the store again — the tree code holds the
//! *link* and looks it up afresh, which is what lets `u_undoredo` call into
//! the memline (and through it buffer-update callbacks) with no header
//! borrowed.
//!
//! The tree itself is walked in exactly two shapes, and both live here:
//! [`header_chain`] follows one link field until it names nothing, and
//! [`TreeWalk`] is the depth-first pass over the *whole* tree that
//! `:undolist`, `:earlier`/`:later` and the undo-file writer each used to
//! spell out for themselves. Both answer links.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use crate::registry::{IdMap, id_map};
use crate::types::{UndoHeader, UndoLink};
use crate::winlayer::Buf;

use super::lastmark;

/// Every undo header one buffer owns, keyed by `seq`.
///
/// The store owns the headers, not the tree shape: unlinking a header from
/// `prev` and friends does not free it, and a header the tree has dropped
/// (`u_clearall` does exactly that for a command preview) stays here until
/// something asks for it to go.
///
/// Keyed through [`IdMap`]: sequence numbers are small and consecutive, and
/// every link followed is a lookup, so SipHash would be most of its cost.
/// Upstream matched a file's links by scanning every pair; a file crafted to
/// collide costs no more here than that did.
pub struct UndoStore {
    headers: IdMap<c_int, Box<UndoHeader>>,
}

impl Default for UndoStore {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoStore {
    /// An empty store.
    pub fn new() -> Self {
        Self { headers: id_map() }
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
        link.is_some() && self.headers.contains_key(&link.seq())
    }

    /// The header `link` names, if the store holds it.
    pub fn get(&self, link: UndoLink) -> Option<&UndoHeader> {
        if link.is_none() {
            // Not a lookup: "no link" is not a key, and refusing it here is
            // what keeps a store that somehow held key 0 from answering it.
            return None;
        }
        self.headers.get(&link.seq()).map(|uh| &**uh)
    }

    /// The header `link` names, for writing.
    pub fn get_mut(&mut self, link: UndoLink) -> Option<&mut UndoHeader> {
        if link.is_none() {
            return None;
        }
        self.headers.get_mut(&link.seq()).map(|uh| &mut **uh)
    }

    /// Takes ownership of `header` under its own sequence number, and hands
    /// back the header that number named before, if any.
    ///
    /// A returned `Some` is a duplicate sequence number, which for a header
    /// read out of a file is corruption and for one the editor just built is
    /// a bug.
    pub fn insert(&mut self, header: Box<UndoHeader>) -> Option<Box<UndoHeader>> {
        debug_assert!(
            header.seq > 0,
            "an undo header's sequence number is positive"
        );
        self.headers.insert(header.seq, header)
    }

    /// Gives up ownership of the header `link` names, and hands it back.
    pub fn take(&mut self, link: UndoLink) -> Option<Box<UndoHeader>> {
        if link.is_none() {
            return None;
        }
        self.headers.remove(&link.seq())
    }

    /// The links to every header the store holds, in no particular order.
    /// Only diagnostics and teardown want this; the tree is walked by link.
    pub fn links(&self) -> impl Iterator<Item = UndoLink> + '_ {
        self.headers.keys().copied().map(UndoLink::to_seq)
    }
}

impl Buf {
    /// The header `link` names in this buffer's undo store, if any: a link
    /// that names nothing, or names a header that has been freed, resolves to
    /// `None`. The borrow is the buffer handle's.
    #[inline]
    pub(crate) fn header(&self, link: UndoLink) -> Option<&UndoHeader> {
        self.b_u_store.as_deref()?.get(link)
    }

    /// [`header`](Self::header), for writing.
    #[inline]
    pub(crate) fn header_mut(&mut self, link: UndoLink) -> Option<&mut UndoHeader> {
        self.b_u_store.as_deref_mut()?.get_mut(link)
    }

    /// Whether `link` names a header this buffer holds.
    #[inline]
    pub(crate) fn has_header(&self, link: UndoLink) -> bool {
        self.header(link).is_some()
    }

    /// The link field `field` of the header `link` names, or no link when
    /// there is no such header.
    #[inline]
    pub(crate) fn header_link(
        &self,
        link: UndoLink,
        field: fn(&UndoHeader) -> UndoLink,
    ) -> UndoLink {
        self.header(link).map_or(UndoLink::NONE, field)
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

    /// The buffer's store, created if this is the buffer's first header.
    fn store_for(&mut self) -> &mut UndoStore {
        self.b_u_store.get_or_insert_with(Box::default)
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

    /// Whether `uh` carries neither stamp.
    fn unstamped(self, uh: &UndoHeader) -> bool {
        uh.walk != self.mark && uh.walk != self.nomark
    }

    /// `link`, when it names a header the walk has yet to reach.
    pub(crate) fn unwalked(self, buffer: Buf, link: UndoLink) -> Option<UndoLink> {
        buffer
            .header(link)
            .is_some_and(|uh| self.unstamped(uh))
            .then_some(link)
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
    /// The header, already stamped with the walk's `mark`.
    pub(crate) link: UndoLink,
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
/// The undo tree is not a tree a plain recursion can walk: `prev` goes down a
/// branch, `alt_next`/`alt_prev` across a run of alternates and `next` back
/// up, and the same header is reached from several of those. What keeps the
/// pass finite is the pair of stamps in [`Marks`] — `mark` on arrival,
/// `nomark` on the way back out — which is why the walk owns them.
///
/// The walk holds the link it is at and that header's four links as they
/// were on arrival, not the header: the caller is free to look headers up
/// (and change their contents) between steps. No caller relinks headers
/// mid-walk; one that did would be steered by the links it arrived through.
pub(crate) struct TreeWalk {
    buf: Buf,
    marks: Marks,
    stop_above: UndoLink,
    state: WalkState,
    depth: c_int,
}

/// Where a walk stands: the header it arrived at last, and its links.
struct Arrived {
    link: UndoLink,
    prev: UndoLink,
    alt_next: UndoLink,
    alt_prev: UndoLink,
    next: UndoLink,
}

enum WalkState {
    Start(UndoLink),
    At(Arrived),
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

    /// Stamps the header `link` names.
    fn stamp(&mut self, link: UndoLink, mark: c_int) {
        if let Some(uh) = self.buf.header_mut(link) {
            uh.walk = mark;
        }
    }

    /// Where the walk goes after `here`, stamping it with `nomark` when it is
    /// backing out of it rather than descending further.
    fn advance(&mut self, here: Arrived) -> UndoLink {
        if let Some(down) = self.marks.unwalked(self.buf, here.prev) {
            // Down into the branch.
            self.depth += 1;
            return down;
        }
        if let Some(across) = self.marks.unwalked(self.buf, here.alt_next) {
            // Or across into an alternate branch, at the same depth.
            return across;
        }
        if here.alt_prev.is_none()
            && let Some(up) = self.marks.unwalked(self.buf, here.next)
        {
            // Or up, but only from the start of a run of alternates.
            if here.link == self.stop_above {
                self.stamp(here.link, self.marks.nomark);
            }
            self.depth -= 1;
            return up;
        }
        // A dead end: stamp it finished and back out.
        self.stamp(here.link, self.marks.nomark);
        if here.alt_prev.is_some() {
            return here.alt_prev;
        }
        self.depth -= 1;
        here.next
    }
}

impl Iterator for TreeWalk {
    type Item = Visit;

    fn next(&mut self) -> Option<Visit> {
        let here = match core::mem::replace(&mut self.state, WalkState::Done) {
            WalkState::Done => return None,
            WalkState::Start(link) => link,
            // The step is taken here rather than after the last header was
            // handed out, so that a caller who stops early leaves the tree
            // stamped exactly as far as it walked.
            WalkState::At(arrived) => self.advance(arrived),
        };
        let marks = self.marks;
        let uh = self.buf.header_mut(here)?;
        let first = marks.unstamped(uh);
        uh.walk = marks.mark;
        self.state = WalkState::At(Arrived {
            link: here,
            prev: uh.prev,
            alt_next: uh.alt_next,
            alt_prev: uh.alt_prev,
            next: uh.next,
        });
        Some(Visit {
            link: here,
            first,
            depth: self.depth,
        })
    }
}

/// Hands `header` to `buffer`'s store and returns the link that now names it.
///
/// The header's `seq` must already be set: it is the key. A header whose
/// number is not one a buffer hands out is dropped, since nothing could name
/// it.
pub(crate) fn header_adopt(mut buffer: Buf, header: Box<UndoHeader>) -> UndoLink {
    let seq = header.seq;
    debug_assert!(seq > 0, "an undo header's sequence number is positive");
    if seq <= 0 {
        return UndoLink::NONE;
    }
    let displaced = buffer.store_for().insert(header);
    debug_assert!(
        displaced.is_none(),
        "two live undo headers claimed sequence number {seq}"
    );
    UndoLink::to_seq(seq)
}

/// Drops the header `link` names from `buffer`'s store, and with it every
/// entry and extmark record it held.
pub(crate) fn header_free(mut buffer: Buf, link: UndoLink) {
    if let Some(store) = buffer.b_u_store.as_deref_mut() {
        drop(store.take(link));
    }
}

/// Yields the links along one chain: `start`, then whatever `step` names from
/// each header in turn, until a link names nothing.
///
/// `step` is read only when the *next* link is asked for, so a body that
/// relinks the header it was handed still steers the walk — which is what
/// the `while` loops this replaced did. A body that frees the header it was
/// handed ends the chain there.
pub(crate) fn header_chain(
    buffer: Buf,
    start: UndoLink,
    step: fn(&UndoHeader) -> UndoLink,
) -> HeaderChain {
    HeaderChain {
        buf: buffer,
        state: ChainState::Start(start),
        step,
    }
}

/// The iterator [`header_chain`] returns.
pub(crate) struct HeaderChain {
    buf: Buf,
    state: ChainState,
    step: fn(&UndoHeader) -> UndoLink,
}

enum ChainState {
    Start(UndoLink),
    At(UndoLink),
    Done,
}

impl Iterator for HeaderChain {
    type Item = UndoLink;

    fn next(&mut self) -> Option<UndoLink> {
        let link = match self.state {
            ChainState::Done => return None,
            ChainState::Start(link) => link,
            ChainState::At(link) => self.buf.header_link(link, self.step),
        };
        if !self.buf.has_header(link) {
            self.state = ChainState::Done;
            return None;
        }
        self.state = ChainState::At(link);
        Some(link)
    }
}

/// Drops `buffer`'s store when nothing is left in it.
///
/// Called at the end of `u_blockfree`, which is the one place that knows the
/// tree has just been walked and freed. A store that still holds something
/// is kept: a command preview detaches the buffer's real tree from
/// `b_u_*head` with `u_clearall` and restores it afterwards, and those
/// headers are only reachable through the store in between.
pub(crate) fn store_release(mut buffer: Buf) {
    if buffer.b_u_store.as_deref().is_some_and(UndoStore::is_empty) {
        buffer.b_u_store = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::XString;
    use crate::types::{ExtmarkSplice, ExtmarkUndoObject, UndoEntry};

    /// A header carrying `seq`, and `walk` as a tag to tell two apart by.
    fn header(seq: c_int, walk: c_int) -> Box<UndoHeader> {
        Box::new(UndoHeader {
            seq,
            walk,
            ..Default::default()
        })
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
        assert!(store.get(UndoLink::to_seq(1)).is_none());
        assert!(store.get(UndoLink::NONE).is_none());
    }

    #[test]
    fn a_header_is_found_by_the_number_it_was_stored_under() {
        let mut store = UndoStore::new();
        assert!(store.insert(header(1, 10)).is_none());
        assert!(store.insert(header(2, 20)).is_none());
        assert_eq!(store.len(), 2);
        assert_eq!(store.get(UndoLink::to_seq(1)).map(|uh| uh.walk), Some(10));
        assert_eq!(store.get(UndoLink::to_seq(2)).map(|uh| uh.walk), Some(20));
        // A write through one lookup is what the next one reads.
        store.get_mut(UndoLink::to_seq(2)).expect("held").walk = 21;
        assert_eq!(store.get(UndoLink::to_seq(2)).map(|uh| uh.walk), Some(21));
        // A number nobody stored, and the "no link" number, both name
        // nothing rather than something arbitrary.
        assert!(store.get(UndoLink::to_seq(3)).is_none());
        assert!(store.get(UndoLink::NONE).is_none());
        assert!(store.get_mut(UndoLink::NONE).is_none());
    }

    #[test]
    fn a_header_taken_out_stops_being_found() {
        let mut store = UndoStore::new();
        store.insert(header(4, 40));
        let taken = store
            .take(UndoLink::to_seq(4))
            .expect("the header is there");
        assert_eq!(taken.walk, 40);
        // The link that named it now names nothing: this is why a stale
        // link cannot be a dangling pointer.
        assert!(store.get(UndoLink::to_seq(4)).is_none());
        assert!(store.is_empty());
        // And taking it again is harmless, which is what makes a double
        // free in the tree walk a no-op instead of a crash.
        assert!(store.take(UndoLink::to_seq(4)).is_none());
        assert!(store.take(UndoLink::NONE).is_none());
    }

    #[test]
    fn two_headers_cannot_share_a_sequence_number() {
        let mut store = UndoStore::new();
        assert!(store.insert(header(9, 1)).is_none());
        let displaced = store
            .insert(header(9, 2))
            .expect("the first one comes back");
        assert_eq!(displaced.walk, 1);
        assert_eq!(store.len(), 1);
        assert_eq!(store.get(UndoLink::to_seq(9)).map(|uh| uh.walk), Some(2));
    }

    #[test]
    fn links_names_every_header_held() {
        let mut store = UndoStore::new();
        for seq in 1..=3 {
            store.insert(header(seq, 0));
        }
        let mut seqs: Vec<c_int> = store.links().map(UndoLink::seq).collect();
        seqs.sort_unstable();
        assert_eq!(seqs, vec![1, 2, 3]);
    }

    #[test]
    fn a_fresh_header_is_linked_to_nothing() {
        let uh = UndoHeader::default();
        assert!(uh.next.is_none());
        assert!(uh.prev.is_none());
        assert!(uh.alt_next.is_none());
        assert!(uh.alt_prev.is_none());
        assert_eq!(uh.seq, 0);
        assert!(uh.entries.is_empty());
        assert!(uh.getbot.is_none());
        assert!(uh.extmarks.is_empty());
    }

    /// What used to be `u_freeentries`: a header taken out of the store and
    /// dropped takes its entries, their lines and its extmark records with
    /// it. Miri checks that every one of those allocations is released once.
    #[test]
    fn dropping_a_header_releases_what_it_saved() {
        let mut store = UndoStore::new();
        let mut uh = header(5, 0);
        uh.entries.push(UndoEntry {
            top: 0,
            bot: 3,
            lcount: 0,
            lines: vec![
                XString::from_bytes(b"one"),
                XString::from_bytes("tw\u{f6}".as_bytes()),
            ],
        });
        uh.entries.push(UndoEntry::default());
        uh.getbot = Some(1);
        uh.extmarks.push(ExtmarkUndoObject::Splice(ExtmarkSplice {
            start_row: 0,
            start_col: 0,
            old_row: 0,
            old_col: 1,
            new_row: 0,
            new_col: 2,
            start_byte: 0,
            old_byte: 1,
            new_byte: 2,
        }));
        store.insert(uh);
        let held = store.get(UndoLink::to_seq(5)).expect("held");
        assert_eq!(held.entries[0].size(), 2);
        assert_eq!(&*held.entries[0].lines[1], "tw\u{f6}".as_bytes());
        assert_eq!(
            held.entries_newest_first()
                .map(UndoEntry::size)
                .collect::<Vec<_>>(),
            vec![0, 2]
        );
        drop(store.take(UndoLink::to_seq(5)));
        assert!(store.is_empty());
    }
}
