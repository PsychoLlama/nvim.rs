//! A string-keyed table that walks in upstream's hashtab slot order.
//!
//! `:mkspell` keeps four tables keyed by a short string: the `PFX` and
//! `SFX` affix headers, the compound flags, and the `COMMON` words. Two of
//! them are *walked* — the affix tables when a `.dic` stem is expanded, the
//! common words when `SN_WORDS` is written — and the order of that walk is
//! the slot order of upstream's hashtab, which decides the order words are
//! added and the bytes of the section. So the slots stay a [`HashTab`],
//! with upstream's hash, probe sequence and resize points, and what a slot
//! holds is the index of an owned entry. Nothing is ever removed.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::hashtab::{SlotEntry, hash_bytes};
use crate::types::HashTab;

/// What a slot holds: the index of its entry.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct EntryId(u32);

impl SlotEntry for EntryId {
    const EMPTY: Self = EntryId(u32::MAX);

    fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    fn is_removed(self) -> bool {
        self == Self::removed()
    }

    fn removed() -> Self {
        EntryId(u32::MAX - 1)
    }
}

/// An entry that names its own key.
pub(super) trait Keyed {
    fn key(&self) -> &[u8];
}

impl Keyed for Box<[u8]> {
    fn key(&self) -> &[u8] {
        self
    }
}

/// Entries by key, walked in slot order.
pub(super) struct KeyedTable<T> {
    slots: HashTab<EntryId>,
    entries: Vec<T>,
}

impl<T: Keyed> KeyedTable<T> {
    pub(super) fn new() -> Self {
        KeyedTable {
            slots: HashTab::init(),
            entries: Vec::new(),
        }
    }

    /// The index of the entry keyed `key`, if there is one.
    pub(super) fn find(&self, key: &[u8]) -> Option<usize> {
        let hi = self.slots.probe(hash_bytes(key), |id| {
            self.entries[id.0 as usize].key() == key
        });
        hi.is_kept().then_some(hi.hi_key.0 as usize)
    }

    /// Add `entry`, whose key the table does not hold yet; its index.
    pub(super) fn insert(&mut self, entry: T) -> usize {
        let hash = hash_bytes(entry.key());
        let hi = self
            .slots
            .probe(hash, |id| self.entries[id.0 as usize].key() == entry.key());
        debug_assert!(!hi.is_kept(), "the key is new");
        let idx = self.entries.len();
        self.entries.push(entry);
        let id = EntryId(u32::try_from(idx).expect("fewer entries than u32::MAX - 1"));
        self.slots.insert_at(hi, id, hash);
        idx
    }

    pub(super) fn get(&self, idx: usize) -> &T {
        &self.entries[idx]
    }

    pub(super) fn get_mut(&mut self, idx: usize) -> &mut T {
        &mut self.entries[idx]
    }

    /// How many entries there are: upstream's `ht_used`.
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    /// The entries in slot order, which is upstream's walk.
    pub(super) fn iter(&self) -> impl Iterator<Item = &T> {
        self.slots
            .items()
            .map(|hi| &self.entries[hi.hi_key.0 as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_is_found_where_it_was_put() {
        let mut table: KeyedTable<Box<[u8]>> = KeyedTable::new();
        assert_eq!(table.find(b"a"), None);
        let a = table.insert(b"a"[..].into());
        let b = table.insert(b"bc"[..].into());
        assert_eq!(table.find(b"a"), Some(a));
        assert_eq!(table.find(b"bc"), Some(b));
        assert_eq!(table.find(b"b"), None);
        assert_eq!(table.len(), 2);
    }

    /// The walk is slot order, not insertion order: what upstream's
    /// hashtab walk handed out.
    #[test]
    fn the_walk_is_slot_order() {
        let mut table: KeyedTable<Box<[u8]>> = KeyedTable::new();
        let words: Vec<Box<[u8]>> = (0..40u8).map(|i| vec![b'a' + i % 26, i].into()).collect();
        for w in &words {
            table.insert(w.clone());
        }
        let mut walked: Vec<&[u8]> = table.iter().map(|w| &**w).collect();
        assert_eq!(walked.len(), words.len());
        walked.sort_unstable();
        let mut want: Vec<&[u8]> = words.iter().map(|w| &**w).collect();
        want.sort_unstable();
        assert_eq!(walked, want);
    }
}
