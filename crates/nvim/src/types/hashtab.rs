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
/// The table **owns** its slots, in a `Vec` whose length is a power of two
/// (or zero, before [`crate::hashtab::hash_init`] gives it one). That is the
/// one structural difference from the C, where `ht_array` pointed at an
/// inline `ht_smallarray` while the table was small -- a self-reference that
/// made a `hashtab_T` valid only at the address it was initialised at, and
/// therefore unwrappable and unmovable.
///
/// What did *not* change is anything a caller can see: the hash, the probe
/// sequence, the resize thresholds and so the slot every key lands in, which
/// is [iteration order](Self::slots) and is Vim-visible through `keys()`,
/// `items()` and every listing that walks a Dictionary.
///
/// Keys are borrowed: the table compares them and frees them only on the
/// caller's behalf ([`crate::hashtab::hash_clear_all`]).
pub struct hashtab_T {
    /// Live entries.
    pub ht_used: size_t,
    /// Entries plus tombstones: what the load factor is measured against.
    pub ht_filled: size_t,
    /// Bumped by every add, remove and resize, so a walk can tell that a
    /// callback rearranged the table under it.
    pub ht_changed: ::core::ffi::c_int,
    /// Non-zero while a caller holds slot pointers across mutations; see
    /// [`crate::hashtab::hash_lock`].
    pub ht_locked: ::core::ffi::c_int,
    /// The slot array. A separate allocation from this header, so a raw
    /// pointer into it survives writes to the fields above and dies only at
    /// the next resize.
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

    /// The slot array as the raw cursor the C-shaped walks step through.
    ///
    /// `&self` rather than `&mut self` because that is the borrow the
    /// callers have: `hash_find` answers a writable slot from a
    /// `*const hashtab_T`, since a table is usually reached through a shared
    /// borrow of the struct that holds it (a `slang_T`, a `dict_T`). Writing
    /// through the result is the caller's business, and its obligation is
    /// the one `ht_locked` exists for: no slot pointer may outlive a resize.
    pub fn slot_ptr(&self) -> *mut hashitem_T {
        self.slots.as_ptr().cast_mut()
    }

    /// Swap in a new slot array, answering the old one.
    pub(crate) fn replace_slots(&mut self, slots: Vec<hashitem_T>) -> Vec<hashitem_T> {
        ::core::mem::replace(&mut self.slots, slots)
    }
}
