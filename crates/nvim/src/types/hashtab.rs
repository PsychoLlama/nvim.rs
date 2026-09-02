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

/// Where a table's slots live.
///
/// The C kept the first sixteen in an inline `ht_smallarray` and pointed
/// `ht_array` at it, so the overwhelmingly common table -- a dictionary of a
/// handful of keys -- cost no allocation at all. This is that small case as
/// a *value*: it is stored in the `hashtab_T`, not pointed at from it, so
/// the table stays movable.
///
/// Growth past the small run moves to [`Slots::Heap`], and a shrink back to
/// exactly [`crate::hashtab::HT_INIT_SIZE`] slots returns to the run -- the
/// same two transitions the C made, which is why the resize's "is the array
/// still the small one" test still reads `oldsize == HT_INIT_SIZE`.
#[expect(
    clippy::large_enum_variant,
    reason = "the size difference is the point: the small run is stored, not \
              pointed at, which is what costs a dictionary no allocation"
)]
enum Slots {
    /// No array at all: a [`hashtab_T::new`], or a table
    /// [`crate::hashtab::hash_clear`] emptied.
    None,
    /// The small run, in the table itself.
    Inline([hashitem_T; crate::hashtab::HT_INIT_SIZE]),
    /// A grown table's array.
    Heap(Vec<hashitem_T>),
}

impl Slots {
    /// `size` empty slots, inline when that is the small run's size.
    fn with_size(size: usize) -> Self {
        if size == crate::hashtab::HT_INIT_SIZE {
            Slots::Inline([hashitem_T::EMPTY; crate::hashtab::HT_INIT_SIZE])
        } else {
            Slots::Heap(vec![hashitem_T::EMPTY; size])
        }
    }

    fn as_slice(&self) -> &[hashitem_T] {
        match self {
            Slots::None => &[],
            Slots::Inline(run) => run,
            Slots::Heap(grown) => grown,
        }
    }

    fn as_mut_slice(&mut self) -> &mut [hashitem_T] {
        match self {
            Slots::None => &mut [],
            Slots::Inline(run) => run,
            Slots::Heap(grown) => grown,
        }
    }
}

/// Vim's open-addressed hash table.
///
/// The table **owns** its slots, in a run whose length is a power of two (or
/// zero, before [`crate::hashtab::hash_init`] gives it one). That is the one
/// structural difference from the C, where `ht_array` pointed at the inline
/// `ht_smallarray` while the table was small -- a self-reference that made a
/// `hashtab_T` valid only at the address it was initialised at, and
/// therefore unwrappable and unmovable. The small run is still inline (see
/// [`Slots`]); nothing points at it.
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
/// Because the small run lives *in* the table, a raw pointer into it is
/// derived from the `hashtab_T` itself and dies at the next
/// `&mut hashtab_T` -- which every mutation of the table takes. So a lookup
/// answers a [`crate::hashtab::Slot`], an index plus a copy of what the slot
/// held, and every write goes back through the table. An index survives a
/// mutation; what it does not survive is a *resize*, which is what
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
    /// The slots themselves.
    slots: Slots,
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
            slots: Slots::None,
        }
    }

    /// A table with its first slot array, which for the initial size is the
    /// inline run: what [`crate::hashtab::hash_init`] writes.
    pub(crate) fn with_slots() -> Self {
        Self {
            slots: Slots::with_size(crate::hashtab::HT_INIT_SIZE),
            ..Self::new()
        }
    }

    /// How many slots the table has: a power of two, or zero.
    pub fn size(&self) -> usize {
        self.slots.as_slice().len()
    }

    /// The index mask, one less than the slot count.
    ///
    /// Panics on a table that has no slots yet, which is the right answer:
    /// the only thing that asks for a mask is a probe, and probing a table
    /// [`crate::hashtab::hash_init`] has not reached is a bug.
    pub fn mask(&self) -> hash_T {
        self.slots.as_slice().len() - 1
    }

    /// Every slot, in index order -- empty ones and tombstones included.
    /// The live entries alone are [`crate::hashtab::hash_items`].
    pub fn slots(&self) -> &[hashitem_T] {
        self.slots.as_slice()
    }

    /// Every slot, writable.
    pub fn slots_mut(&mut self) -> &mut [hashitem_T] {
        self.slots.as_mut_slice()
    }

    /// Give the table a fresh array of `size` empty slots, handing `rehash`
    /// the old slots and the new ones so it can move the live entries over.
    pub(crate) fn resize_slots(
        &mut self,
        size: usize,
        rehash: impl FnOnce(&[hashitem_T], &mut [hashitem_T]),
    ) {
        let old = ::core::mem::replace(&mut self.slots, Slots::with_size(size));
        rehash(old.as_slice(), self.slots.as_mut_slice());
    }

    /// Release the table's slots, leaving it with none.
    pub(crate) fn drop_slots(&mut self) {
        self.slots = Slots::None;
    }
}
