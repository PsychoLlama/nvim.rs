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
//! files this slice converted mechanically (`apply.rs`, `file.rs`,
//! `read.rs`, `write.rs`) still walk a header through a `*mut u_header_T`,
//! and a stable address is what lets the store change underneath them.
//! [`header_at`] is that view.

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

use crate::main::curbuf;
use crate::memory::xfree;
use crate::types::{UndoLink, buf_T, u_header_T};

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

/// The link that names `uhp`, or [`UndoLink::NONE`] for a NULL one.
///
/// # Safety
///
/// `uhp` is NULL or points at a live header.
pub(crate) unsafe fn link_of(uhp: *const u_header_T) -> UndoLink {
    if uhp.is_null() {
        return UndoLink::NONE;
    }
    // SAFETY: non-NULL and live by the contract above.
    UndoLink::to_seq(unsafe { (*uhp).uh_seq })
}

/// The header `link` names in `buf`'s store, or NULL.
///
/// # Safety
///
/// `buf` points at a live buffer.
pub(crate) unsafe fn header_at(buf: *mut buf_T, link: UndoLink) -> *mut u_header_T {
    // SAFETY: a live buffer; `b_u_store` is NULL or a store this module
    // allocated and nothing else writes it.
    match unsafe { store_of(buf) } {
        Some(store) => store.at(link),
        None => core::ptr::null_mut(),
    }
}

/// The header `link` names in the *current* buffer, or NULL. The tree walks
/// in `apply.rs` and `eval.rs` all run on `curbuf`.
///
/// # Safety
///
/// A live current buffer.
pub(crate) unsafe fn cur_header(link: UndoLink) -> *mut u_header_T {
    // SAFETY: a live current buffer, by the contract above.
    unsafe { header_at(curbuf.get(), link) }
}

/// Whether there is a header here that a tree walk has not stamped with
/// either of the two marks it is using. A walk that only has one mark passes
/// it twice.
///
/// # Safety
///
/// `uhp` is NULL or points at a live header.
pub(crate) unsafe fn unwalked(uhp: *mut u_header_T, mark: c_int, nomark: c_int) -> bool {
    // SAFETY: non-NULL and live by the contract above.
    !uhp.is_null() && unsafe { (*uhp).uh_walk != nomark && (*uhp).uh_walk != mark }
}

/// `buf`'s store, if it has one yet.
///
/// # Safety
///
/// `buf` points at a live buffer, and the borrow must not outlive the
/// statement that takes it — every caller here keeps it to one expression.
unsafe fn store_of<'a>(buf: *mut buf_T) -> Option<&'a mut UndoStore> {
    // SAFETY: a live buffer, so the field read is in bounds; the pointer it
    // holds is NULL or a leaked `Box<UndoStore>` from `store_for`.
    unsafe { (*buf).b_u_store.as_mut() }
}

/// `buf`'s store, created if this is the buffer's first header.
///
/// # Safety
///
/// `buf` points at a live buffer.
unsafe fn store_for<'a>(buf: *mut buf_T) -> &'a mut UndoStore {
    // SAFETY: a live buffer.
    unsafe {
        if (*buf).b_u_store.is_null() {
            (*buf).b_u_store = Box::into_raw(Box::new(UndoStore::new()));
        }
        &mut *(*buf).b_u_store
    }
}

/// Hands `uhp` to `buf`'s store and returns the link that now names it.
///
/// The header's `uh_seq` must already be set: it is the key.
///
/// # Safety
///
/// `buf` points at a live buffer and `uhp` at a live header the store does
/// not already own.
pub(crate) unsafe fn header_adopt(buf: *mut buf_T, uhp: *mut u_header_T) -> UndoLink {
    // SAFETY: a live header by the contract above.
    let seq = unsafe { (*uhp).uh_seq };
    let Some(uhp) = NonNull::new(uhp) else {
        return UndoLink::NONE;
    };
    // SAFETY: a live buffer by the contract above.
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
/// `buf` points at a live buffer, `uhp` at a live header allocated by
/// `xmalloc`, and nothing else holds a pointer to that header.
pub(crate) unsafe fn header_free(buf: *mut buf_T, uhp: *mut u_header_T) {
    // SAFETY: a live header by the contract above.
    let link = unsafe { link_of(uhp) };
    // SAFETY: a live buffer.
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
/// `buf` points at a live buffer for as long as the iterator is used, and
/// nothing frees a header the walk has already visited.
pub(crate) unsafe fn header_chain(
    buf: *mut buf_T,
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
    buf: *mut buf_T,
    state: ChainState,
    step: fn(&u_header_T) -> UndoLink,
}

enum ChainState {
    Start(UndoLink),
    At(*mut u_header_T),
    Done,
}

impl Iterator for HeaderChain {
    type Item = *mut u_header_T;

    fn next(&mut self) -> Option<*mut u_header_T> {
        let link = match self.state {
            ChainState::Done => return None,
            ChainState::Start(link) => link,
            // SAFETY: a header the store handed back and that the walk has
            // not freed, per `header_chain`'s contract.
            ChainState::At(uhp) => (self.step)(unsafe { &*uhp }),
        };
        // SAFETY: a live buffer, per `header_chain`'s contract.
        let uhp = unsafe { header_at(self.buf, link) };
        if uhp.is_null() {
            self.state = ChainState::Done;
            return None;
        }
        self.state = ChainState::At(uhp);
        Some(uhp)
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
/// # Safety
///
/// `buf` points at a live buffer.
pub(crate) unsafe fn store_release(buf: *mut buf_T) {
    // SAFETY: a live buffer.
    unsafe {
        if !(*buf).b_u_store.is_null() && (*(*buf).b_u_store).is_empty() {
            drop(Box::from_raw((*buf).b_u_store));
            (*buf).b_u_store = core::ptr::null_mut();
        }
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
