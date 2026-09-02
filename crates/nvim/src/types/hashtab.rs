#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

pub type hash_T = size_t;
/// One slot of a [`hashtab_T`].
///
/// `Copy`: `hi_key` points into the `dictitem_T` (or equivalent) that the
/// table indexes, which the table does not own.
#[derive(Copy, Clone)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
}

/// Vim's open-addressed hash table.
///
/// The table **owns** its slots, in a run whose length is a power of two (or
/// zero, before [`crate::hashtab::hash_init`] gives it one). That is the one
/// structural difference from the C, where `ht_array` pointed at the inline
/// `ht_smallarray` while the table was small -- a self-reference that made a
/// `hashtab_T` valid only at the address it was initialised at, and
/// therefore unwrappable and unmovable.
///
/// What did *not* change is anything a caller can see: the hash, the probe
/// sequence, the resize thresholds and so the slot every key lands in, which
/// is [iteration order](Self::slots) and is Vim-visible through `keys()`,
/// `items()` and every listing that walks a Dictionary.
///
/// Keys are borrowed: the table compares them and frees them only on the
/// caller's behalf ([`crate::hashtab::hash_clear_all`]).
///
/// # Slots are named by index, never by pointer
///
/// A lookup answers a [`crate::hashtab::Slot`] -- an index plus a copy of
/// what the slot held -- and every write goes back through the table. An
/// index survives a mutation of the table, which a borrow into the slots
/// would not; what it does not survive is a *resize*, which is what
/// [`crate::hashtab::hash_lock`] exists to prevent.
pub struct hashtab_T {
    /// Live entries.
    pub ht_used: size_t,
    /// Entries plus tombstones: what the load factor is measured against.
    pub ht_filled: size_t,
    /// Bumped by every add, remove and resize, so a walk can tell that a
    /// callback rearranged the table under it.
    pub ht_changed: ::core::ffi::c_int,
    /// Non-zero while a caller holds slot indexes across mutations; see
    /// [`crate::hashtab::hash_lock`].
    pub ht_locked: ::core::ffi::c_int,
    /// The slot array. A separate allocation from this header.
    slots: Vec<hashitem_T>,
}

impl Default for hashitem_T {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl hashitem_T {
    /// A slot that never held a key.
    pub const EMPTY: Self = Self {
        hi_hash: 0,
        hi_key: ::core::ptr::null_mut(),
    };
}

impl Default for hashtab_T {
    fn default() -> Self {
        Self::new()
    }
}

impl hashtab_T {
    /// A table with no slots at all: the state a `hashtab_T` field has
    /// before [`crate::hashtab::hash_init`] gives it its first array, and
    /// the one it is left in by [`crate::hashtab::hash_clear`].
    ///
    /// `const`, so a table can be a `static` or a `const` field initialiser.
    /// It allocates nothing.
    pub const fn new() -> Self {
        Self {
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            slots: Vec::new(),
        }
    }

    /// A table with its first slot array: what
    /// [`crate::hashtab::hash_init`] writes.
    pub(crate) fn with_slots() -> Self {
        Self {
            slots: vec![hashitem_T::EMPTY; crate::hashtab::HT_INIT_SIZE],
            ..Self::new()
        }
    }

    /// How many slots the table has: a power of two, or zero.
    pub fn size(&self) -> usize {
        self.slots.len()
    }

    /// The index mask, one less than the slot count.
    ///
    /// Panics on a table that has no slots yet, which is the right answer:
    /// the only thing that asks for a mask is a probe, and probing a table
    /// [`crate::hashtab::hash_init`] has not reached is a bug.
    pub fn mask(&self) -> hash_T {
        self.slots.len() - 1
    }

    /// Every slot, in index order -- empty ones and tombstones included.
    /// The live entries alone are [`crate::hashtab::hash_items`].
    pub fn slots(&self) -> &[hashitem_T] {
        &self.slots
    }

    /// Every slot, writable.
    pub fn slots_mut(&mut self) -> &mut [hashitem_T] {
        &mut self.slots
    }

    /// Give the table a fresh array of `size` empty slots, handing `rehash`
    /// the old slots and the new ones so it can move the live entries over.
    pub(crate) fn resize_slots(
        &mut self,
        size: usize,
        rehash: impl FnOnce(&[hashitem_T], &mut [hashitem_T]),
    ) {
        let old = ::core::mem::replace(&mut self.slots, vec![hashitem_T::EMPTY; size]);
        rehash(&old, &mut self.slots);
    }

    /// Release the table's slots, leaving it with none.
    pub(crate) fn drop_slots(&mut self) {
        self.slots = Vec::new();
    }
}
