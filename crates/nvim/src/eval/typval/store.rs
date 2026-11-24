//! The dictionary item store: a hash table that **owns** its items.
//!
//! A slot names its item by address -- a `Box<DictItem>` the table took over
//! ([`Dict::add_item`]) -- because an item must not move when the table
//! rehashes: a caller holding `&DictItem` across a resize-free walk, and the
//! probe reading the key out of the item, both rest on it. Every kept slot of
//! a table owns exactly one box, which no other table names: the entry type
//! is this file's, the table's slots are reached only through [`DictTab`]'s
//! methods, and an item enters only as a box and leaves only as one
//! ([`DictTab::remove`]) or with the table ([`Drop`]). So a table can be
//! moved, swapped or replaced whole -- two edited dictionaries swapping
//! their bodies, `v:event` saving its table across an autocommand -- and
//! every item still has one owner.
//!
//! Upstream's table could also name an item embedded in a structure that
//! outlived it (`b:changedtick`), told apart by the item's `DI_FLAGS_ALLOC`;
//! that item is an ordinary owned one now, and the flag is gone.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `eval/typval/{handle,store}.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]

use ::core::ptr::NonNull;

use super::*;
use crate::hashtab::{HT_INIT_SIZE, HashItem, HashTab, HashValue, SlotEntry};

/// A dictionary's hash table, owning the item every kept slot names.
#[derive(Debug, Default)]
pub struct DictTab(HashTab<DictEntry>);

/// What a kept slot holds: the box the table owns. `Copy` only so that
/// [`HashTab`]'s slots can be; it never leaves this file.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct DictEntry(*mut DictItem);

/// The tombstone: `NonNull::dangling()`'s address, the item type's
/// alignment, which no allocation can be at and nothing dereferences.
impl SlotEntry for DictEntry {
    const EMPTY: Self = DictEntry(::core::ptr::null_mut());

    fn is_empty(self) -> bool {
        self.0.is_null()
    }

    fn is_removed(self) -> bool {
        self.0 == NonNull::<DictItem>::dangling().as_ptr()
    }

    fn removed() -> Self {
        DictEntry(NonNull::dangling().as_ptr())
    }
}

impl DictTab {
    /// A table with no slots, for a `const` initialiser; nothing may be
    /// added until it is replaced by an [`DictTab::init`] one.
    pub const fn new() -> DictTab {
        DictTab(HashTab::new())
    }

    /// An empty table with its first slots.
    pub fn init() -> DictTab {
        DictTab(HashTab::init())
    }

    /// How many items the table holds.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.ht_used
    }

    /// Whether the table holds no items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// How many slots the table has: the bound of a walk by slot index.
    #[inline]
    pub(crate) fn slot_count(&self) -> usize {
        self.0.size()
    }

    /// Whether slot `slot` holds an item.
    #[inline]
    pub(crate) fn is_kept(&self, slot: usize) -> bool {
        self.0.slots().get(slot).is_some_and(HashItem::is_kept)
    }

    /// Stop the table resizing, so slot indexes stay meaningful across
    /// removals until the matching [`DictTab::unlock`].
    #[inline]
    pub(crate) fn lock(&mut self) {
        self.0.lock();
    }

    /// Undo one [`DictTab::lock`], resizing now if the table wanted to.
    #[inline]
    pub(crate) fn unlock(&mut self) {
        self.0.unlock();
    }

    /// The item in slot `slot`, or `None` when the slot holds none.
    #[inline]
    pub(crate) fn get(&self, slot: usize) -> Option<&DictItem> {
        let hi = self.0.slots().get(slot)?;
        // SAFETY: a kept slot names the live box this table owns, which the
        // borrow of the table keeps in place.
        hi.is_kept().then(|| unsafe { &*hi.hi_key.0 })
    }

    /// [`DictTab::get`] with the item writable.
    #[inline]
    pub(crate) fn get_mut(&mut self, slot: usize) -> Option<&mut DictItem> {
        let hi = *self.0.slots().get(slot)?;
        // SAFETY: as `get`; no two slots name one box, and the exclusive
        // borrow of the table rules out every other borrow of it.
        hi.is_kept().then(|| unsafe { &mut *hi.hi_key.0 })
    }

    /// Every item, in slot order -- which is the order Vim shows.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &DictItem> {
        // SAFETY: as `get`.
        self.0.items().map(|hi| unsafe { &*hi.hi_key.0 })
    }

    /// Every item, writable, in slot order. The items are not in the
    /// table's storage, so handing out `&mut` to each in turn does not alias
    /// the table; the exclusive borrow keeps the set of them fixed.
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut DictItem> {
        // SAFETY: as `get_mut`, one box per slot.
        self.0.items().map(|hi| unsafe { &mut *hi.hi_key.0 })
    }

    /// The slot holding `key` (hashing to `hash`), if any.
    #[inline]
    pub(crate) fn find(&self, key: &[u8], hash: HashValue) -> Option<usize> {
        // SAFETY: the probe hands the test only this table's kept entries,
        // each a live box owning its key, which the borrow keeps alive.
        let hi = self
            .0
            .probe(hash, |entry| unsafe { (*entry.0).di_key.bytes() } == key);
        hi.is_kept().then(|| hi.index())
    }

    /// Take `item` over. When its key is already there the item comes back,
    /// untouched.
    pub(crate) fn insert(&mut self, item: Box<DictItem>) -> Result<(), Box<DictItem>> {
        let hash = crate::hashtab::hash_bytes(item.di_key.bytes());
        let key = item.di_key.bytes();
        // SAFETY: as `find`'s.
        let hi = self
            .0
            .probe(hash, |entry| unsafe { (*entry.0).di_key.bytes() } == key);
        if hi.is_kept() {
            return Err(item);
        }
        self.0.insert_at(hi, DictEntry(Box::into_raw(item)), hash);
        Ok(())
    }

    /// Take the item in slot `slot` out of the table, which gives the box
    /// back; `None` when the slot holds none.
    pub(crate) fn remove(&mut self, slot: usize) -> Option<Box<DictItem>> {
        let hi = self.0.slot(slot);
        if !hi.is_kept() {
            return None;
        }
        self.0.remove_at(hi);
        // SAFETY: the box the slot owned, out of the table now.
        Some(unsafe { Box::from_raw(hi.hi_key.0) })
    }

    /// A table emptied by removals made fresh again: in place when it is at
    /// its initial size (upstream's `hash_clear` + `hash_init` without a new
    /// table), else replaced. Anything still in it is released.
    pub(crate) fn reset(&mut self) {
        if !self.is_empty() || self.0.size() != HT_INIT_SIZE {
            *self = DictTab::init();
            return;
        }
        self.0.slots_mut().fill(HashItem::EMPTY);
        self.0.ht_used = 0;
        self.0.ht_filled = 0;
        self.0.ht_changed = 0;
        self.0.ht_locked = 0;
    }
}

/// The items go with the table, in slot order, as the whole-dictionary free
/// upstream walked them.
impl Drop for DictTab {
    fn drop(&mut self) {
        // The common drop is of a table already emptied (a freed
        // dictionary's fresh one): no slot walk for it.
        if self.is_empty() {
            return;
        }
        for hi in self.0.items() {
            // SAFETY: each kept slot's box, freed once here; the table is
            // being dropped, so nothing reaches the slot again.
            drop(unsafe { Box::from_raw(hi.hi_key.0) });
        }
    }
}

/// [`DictTab`] through the dictionary it is the table of.
impl Dict {
    /// Every entry, in slot order -- which is the order Vim shows.
    ///
    /// The walk borrows the dictionary, so nothing can add to or remove from
    /// it while the walk is live. A body that edits the table wants
    /// [`DictCursor`], which is a slot index.
    pub fn items(&self) -> impl Iterator<Item = &DictItem> {
        self.dv_hashtab.iter()
    }

    /// Every entry, writable, in slot order.
    pub fn items_mut(&mut self) -> impl Iterator<Item = &mut DictItem> {
        self.dv_hashtab.iter_mut()
    }

    /// The item in slot `slot`, or `None` when the slot holds none.
    #[inline]
    pub(crate) fn item_at(&self, slot: usize) -> Option<&DictItem> {
        self.dv_hashtab.get(slot)
    }

    /// [`Dict::item_at`] with the item writable.
    #[inline]
    pub(crate) fn item_at_mut(&mut self, slot: usize) -> Option<&mut DictItem> {
        self.dv_hashtab.get_mut(slot)
    }

    /// Take the item in slot `slot` out of the table.
    ///
    /// Nothing is released here: the answer owns what the item held, and
    /// dropping it after the borrow of the dictionary has ended is the
    /// release -- which matters, because a value can name the dictionary it
    /// was in, and releasing it reads that dictionary again.
    pub(crate) fn remove_at(&mut self, slot: usize) -> Box<DictItem> {
        self.dv_hashtab
            .remove(slot)
            .expect("a removal names a kept slot")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(key: &str, n: VarNumber) -> Box<DictItem> {
        let mut item = DictItem::boxed(key.as_bytes());
        item.di_tv = TypVal::Number(n);
        item
    }

    /// Items go in as boxes and come out as boxes; a duplicate key comes
    /// back untouched; the rest go with the table.
    #[test]
    fn the_table_owns_its_items() {
        let mut table = DictTab::init();
        for (n, key) in ["a", "b", "c"].into_iter().enumerate() {
            let n = VarNumber::try_from(n).expect("small");
            assert!(table.insert(item(key, n)).is_ok());
        }
        let refused = table.insert(item("b", 9)).expect_err("b is taken");
        assert_eq!(refused.di_tv.number_or_zero(), 9);
        let slot = table
            .find(b"b", crate::hashtab::hash_bytes(b"b"))
            .expect("b is in");
        assert_eq!(table.get(slot).map(|i| i.di_tv.number_or_zero()), Some(1));
        let removed = table.remove(slot).expect("a kept slot");
        assert_eq!(removed.di_key.bytes(), b"b");
        assert!(table.remove(slot).is_none(), "a removed slot holds nothing");
        assert_eq!(table.len(), 2);
        let sum: VarNumber = table.iter_mut().map(|i| i.di_tv.number_or_zero()).sum();
        assert_eq!(sum, 2);
        // Two tables swapped whole: each still owns its own items.
        let mut other = DictTab::init();
        assert!(other.insert(item("z", 26)).is_ok());
        ::core::mem::swap(&mut table, &mut other);
        assert_eq!((table.len(), other.len()), (1, 2));
        drop(other);
        table.reset();
        assert!(table.is_empty());
    }
}
