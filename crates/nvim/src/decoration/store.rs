//! The two decoration stores: one slab of sign/highlight items and one of
//! virtual-text links, each entry chained to the next by index.
//!
//! A mark's `ext` decoration names the head of one chain in each. Indices
//! rather than pointers, because a mark's key is `Copy` and moves around the
//! marktree by value, and because the drawing state keeps naming an entry
//! while a decoration provider's Lua may add others: an index survives the
//! slab growing, a pointer into it does not.
//!
//! Entries are never removed. A freed chain is emptied of what it owns and
//! put on a freelist threaded through the same `next` field, head first, and
//! the slot is reused in place by the next [`Store::put`].
//!
//! Copyright Neovim contributors. Licensed under the Apache License, Version
//! 2.0; see LICENSE.txt in the project root.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::{DecorSignHighlight, DecorVirtText, DecorVirtText_data, uint32_t};

/// The index that ends a chain, and names no entry.
pub(crate) const NO_ENTRY: uint32_t = uint32_t::MAX;

/// An entry of a [`Store`]: something that links to the next one of its
/// chain, and that can give up what it owns when its slot is freed.
pub(crate) trait Link {
    /// The index of the next entry of the chain, or [`NO_ENTRY`].
    fn next(&self) -> uint32_t;
    fn set_next(&mut self, next: uint32_t);
    /// Drop whatever the entry owns: the slot is being freed.
    fn vacate(&mut self);
}

impl Link for DecorSignHighlight {
    fn next(&self) -> uint32_t {
        self.next
    }

    fn set_next(&mut self, next: uint32_t) {
        self.next = next;
    }

    /// A freed item keeps its other fields, as upstream's did: the walks
    /// over every slot (`:sign define`'s, the glyph check) tell a free one
    /// by its zero flags and missing name.
    fn vacate(&mut self) {
        self.flags = 0;
        self.sign_name = None;
        self.url = None;
    }
}

impl Link for DecorVirtText {
    fn next(&self) -> uint32_t {
        self.next
    }

    fn set_next(&mut self, next: uint32_t) {
        self.next = next;
    }

    fn vacate(&mut self) {
        self.data = DecorVirtText_data::Text(Vec::new());
    }
}

/// A slab of chained entries with a freelist through them.
pub(crate) struct Store<T> {
    entries: Vec<T>,
    /// Index of the first free slot, or [`NO_ENTRY`].
    free: uint32_t,
}

impl<T: Link> Store<T> {
    pub(crate) const fn new() -> Self {
        Self {
            entries: Vec::new(),
            free: NO_ENTRY,
        }
    }

    /// How many slots the store has handed out, free ones included.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }

    /// Entry `idx`. Panics on an index the store never handed out, where
    /// upstream would read past the array.
    #[inline]
    pub(crate) fn get(&self, idx: uint32_t) -> &T {
        self.entries
            .get(idx as usize)
            .expect("decoration store index out of range")
    }

    /// [`Store::get`], to write to.
    #[inline]
    pub(crate) fn get_mut(&mut self, idx: uint32_t) -> &mut T {
        self.entries
            .get_mut(idx as usize)
            .expect("decoration store index out of range")
    }

    /// Every slot, free ones included, in index order.
    pub(crate) fn entries_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries.iter_mut()
    }

    /// Stores `entry`, reusing a freed slot when there is one, and answers
    /// its index. The entry's own `next` is left as the caller set it.
    pub(crate) fn put(&mut self, entry: T) -> uint32_t {
        if self.free != NO_ENTRY {
            let idx = self.free;
            let slot = self.get_mut(idx);
            let next_free = slot.next();
            *slot = entry;
            self.free = next_free;
            return idx;
        }
        self.entries.push(entry);
        uint32_t::try_from(self.entries.len() - 1).expect("the store is indexed by a u32")
    }

    /// Hangs chain `onto` off the end of chain `head`, answering the head of
    /// the joined chain.
    pub(crate) fn splice(&mut self, head: uint32_t, onto: uint32_t) -> uint32_t {
        if head == NO_ENTRY {
            return onto;
        }
        let mut idx = head;
        loop {
            let entry = self.get_mut(idx);
            let next = entry.next();
            if next == NO_ENTRY {
                entry.set_next(onto);
                return head;
            }
            idx = next;
        }
    }

    /// Frees every entry of chain `head`: each gives up what it owns, and
    /// the whole chain goes on the freelist at once, head first.
    pub(crate) fn release(&mut self, head: uint32_t) {
        let mut idx = head;
        while idx != NO_ENTRY {
            let entry = self.get_mut(idx);
            entry.vacate();
            let next = entry.next();
            if next == NO_ENTRY {
                let free = self.free;
                self.get_mut(idx).set_next(free);
                self.free = head;
                return;
            }
            idx = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Entry {
        value: Option<String>,
        next: uint32_t,
    }

    impl Link for Entry {
        fn next(&self) -> uint32_t {
            self.next
        }
        fn set_next(&mut self, next: uint32_t) {
            self.next = next;
        }
        fn vacate(&mut self) {
            self.value = None;
        }
    }

    fn entry(value: &str, next: uint32_t) -> Entry {
        Entry {
            value: Some(value.to_owned()),
            next,
        }
    }

    fn chain(store: &Store<Entry>, head: uint32_t) -> Vec<Option<String>> {
        let mut out = Vec::new();
        let mut idx = head;
        while idx != NO_ENTRY {
            out.push(store.get(idx).value.clone());
            idx = store.get(idx).next;
        }
        out
    }

    #[test]
    fn a_chain_is_built_from_its_tail() {
        let mut store = Store::new();
        let c = store.put(entry("c", NO_ENTRY));
        let b = store.put(entry("b", c));
        let a = store.put(entry("a", b));
        assert_eq!((a, b, c), (2, 1, 0));
        assert_eq!(
            chain(&store, a),
            ["a", "b", "c"].map(|s| Some(s.to_owned()))
        );
    }

    #[test]
    fn a_released_chain_is_reused_head_first_and_owns_nothing() {
        let mut store = Store::new();
        let tail = store.put(entry("tail", NO_ENTRY));
        let head = store.put(entry("head", tail));
        let other = store.put(entry("other", NO_ENTRY));
        store.release(head);
        assert_eq!(store.get(head).value, None);
        assert_eq!(store.get(tail).value, None);
        assert_eq!(store.get(other).value.as_deref(), Some("other"));
        // Head first, then the rest of the chain, then fresh slots.
        assert_eq!(store.put(entry("x", NO_ENTRY)), head);
        assert_eq!(store.put(entry("y", NO_ENTRY)), tail);
        assert_eq!(store.put(entry("z", NO_ENTRY)), 3);
        assert_eq!(store.len(), 4);
    }

    #[test]
    fn two_released_chains_stack_on_the_freelist() {
        let mut store = Store::new();
        let first = store.put(entry("1", NO_ENTRY));
        let second = store.put(entry("2", NO_ENTRY));
        store.release(first);
        store.release(second);
        assert_eq!(store.put(entry("a", NO_ENTRY)), second);
        assert_eq!(store.put(entry("b", NO_ENTRY)), first);
    }

    #[test]
    fn splice_joins_two_chains_and_tolerates_an_empty_one() {
        let mut store = Store::new();
        let a = store.put(entry("a", NO_ENTRY));
        let b = store.put(entry("b", NO_ENTRY));
        assert_eq!(store.splice(NO_ENTRY, b), b);
        let joined = store.splice(a, b);
        assert_eq!(joined, a);
        assert_eq!(chain(&store, a), ["a", "b"].map(|s| Some(s.to_owned())));
        store.release(joined);
        assert_eq!(chain(&store, b), [None]);
    }

    #[test]
    #[should_panic(expected = "decoration store index out of range")]
    fn an_index_never_handed_out_panics() {
        let store: Store<Entry> = Store::new();
        store.get(0);
    }
}
