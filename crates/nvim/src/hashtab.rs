//! Open-addressing hash table (`HashTab`).
//!
//! The table owns its slots. What is frozen is the *behaviour*: the hash, the
//! probe sequence, the resize thresholds, and therefore the slot every key
//! lands in. Callers walk the slots directly and Vim shows the result --
//! `keys()`, `items()` and every listing of a Dictionary hand out slot order
//! -- so placement is observable, not an implementation detail.
//!
//! The table stores entries, not keys: an entry is whatever the caller wants
//! to find again ([`SlotEntry`]), and the caller compares keys itself, in the
//! test it hands [`HashTab::probe`]. A dictionary's entries name its items
//! and read the key out of the item; the syntax keyword tables and the
//! spell-file sections store an index into storage beside the table.
//!
//! # Boundary
//!
//! [`HashTab::probe`] answers a [`Slot`]: *which* slot, plus a copy of what
//! it held. Never a reference into the array -- the small run lives inside
//! the `HashTab` (see [`HashTab`]), so a reference into it would pin the
//! table, and a caller that probes is usually about to insert into the slot
//! the probe found. An index survives that. What it does not survive is a
//! *resize*, and that is the contract [`HashTab::lock`] exists for.
//!
//! Every operation takes the table by reference; nothing here is `unsafe`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::deref::deref_to_field;
use crate::os::libc::size_t;

/// The number of slots a table starts with, and the size it shrinks back to.
pub const HT_INIT_SIZE: usize = 16;

const PERTURB_SHIFT: u32 = 5;

/// What a slot of a [`HashTab`] holds when it is occupied.
///
/// The table probes on a key's *hash* and stores whatever the caller wants
/// to find again: a dictionary's slots name a
/// [`DictItem`](crate::eval::typval::DictItem) and read the key out of it, and the
/// tables whose entries are indexes into storage beside them (the syntax
/// keyword tables, the spell-file sections) look the key up there. Either way
/// the probe sequence sees only the hash and the caller's equality test, so
/// the slot a key lands in does not depend on what the entry is -- which is
/// what keeps `keys()` order the same whichever table shape a scope uses.
///
/// Two of the values are reserved: [`SlotEntry::EMPTY`] for a slot that
/// never held a key, and the one [`SlotEntry::is_removed`] recognises for a
/// tombstone. `EMPTY` must be recognised by `is_empty` and by nothing else;
/// a table whose entries break that only misplaces its own keys.
pub trait SlotEntry: Copy {
    /// The value a slot that never held a key carries.
    const EMPTY: Self;

    /// Whether this is [`SlotEntry::EMPTY`].
    fn is_empty(self) -> bool;

    /// Whether this is the tombstone a removal leaves behind.
    fn is_removed(self) -> bool;

    /// The tombstone value itself.
    fn removed() -> Self;
}

impl<E: SlotEntry> HashItem<E> {
    /// Never held a key.
    pub fn is_empty(&self) -> bool {
        self.hi_key.is_empty()
    }

    /// Held a key that was removed: a tombstone, which a probe walks past but
    /// an insertion may reuse.
    pub fn is_removed(&self) -> bool {
        self.hi_key.is_removed()
    }

    /// Holds a live key (neither empty nor a tombstone). This is the
    /// `HASHITEM_EMPTY` test every caller that walks the slots open-codes.
    pub fn is_kept(&self) -> bool {
        !self.is_empty() && !self.is_removed()
    }
}

/// One slot of a table: *which* slot it is, and what it held when the table
/// answered it.
///
/// An index, not a pointer, and the reason is that the small run is stored
/// in the `HashTab` itself: a reference into it would borrow the table,
/// and every mutation takes `&mut HashTab`. The index survives a mutation --
/// so the C idiom of probing for a key and then inserting into the slot the
/// probe found still works -- and it dies only where the array is replaced,
/// which [`HashTab::lock`] holds off.
///
/// The copy is what a caller reads (`is_kept`, `hi_key`, `hi_hash`, through
/// `Deref`); writes go back through the table, and are exactly
/// [`HashTab::insert_at`] and [`HashTab::remove_at`].
///
/// A handle, so a tuple: the index and the copy are reached through
/// [`Slot::index`] and `Deref`, never as fields.
#[derive(Copy, Clone)]
pub struct Slot<E: SlotEntry>(usize, HashItem<E>);

impl<E: SlotEntry> Slot<E> {
    /// Which slot of the table this is.
    pub fn index(self) -> usize {
        self.0
    }
}

deref_to_field!(impl[E: SlotEntry] Slot<E> => HashItem<E>, 1);

impl<E: SlotEntry> HashTab<E> {
    /// An empty table with its first slot array.
    pub fn init() -> Self {
        HashTab::with_slots()
    }

    /// The slot at `idx`: the index to write back through, and what it holds
    /// now. Panics when `idx` is not one of the table's slots.
    pub fn slot(&self, idx: usize) -> Slot<E> {
        Slot(idx, self.slots()[idx])
    }

    /// The live entries, in slot order: what every walk of a table hands out,
    /// and the order Vim shows.
    ///
    /// The `ht_used` countdown the C loops use is an optimisation on top of
    /// this -- it stops scanning once the last live entry has been seen --
    /// and yields exactly the same entries in exactly the same order.
    pub fn items(&self) -> impl Iterator<Item = &HashItem<E>> {
        self.slots().iter().filter(|hi| hi.is_kept())
    }

    /// The slot holding the entry `is_key` accepts among those hashing to
    /// `hash`, or -- when none does -- the slot such an entry belongs in: a
    /// tombstone if the walk crossed one, else the empty slot that ended it.
    ///
    /// `is_key` sees only live entries whose stored hash is `hash`. The
    /// answer names a slot of the *current* array, so it dies at the next
    /// resize ([`HashTab::lock`]).
    ///
    /// # Panics
    /// On a table with no slots yet ([`HashTab::new`]).
    pub fn probe(&self, hash: HashValue, mut is_key: impl FnMut(E) -> bool) -> Slot<E> {
        let slots = self.slots();
        let mut freeitem: Option<usize> = None;
        // The probe never runs off the array: it is masked to `mask()`, and
        // the table is never full, so some slot is empty and ends the walk.
        for idx in Probe::new(hash, self.mask()) {
            let item = &slots[idx];
            if item.is_empty() {
                return self.slot(freeitem.unwrap_or(idx));
            }
            if item.is_removed() {
                freeitem.get_or_insert(idx);
            } else if item.hi_hash == hash && is_key(item.hi_key) {
                return self.slot(idx);
            }
        }
        unreachable!("probe sequence always finds an empty slot");
    }

    /// Store `entry`, hashing to `hash`, in the slot `hi`, which a
    /// [`HashTab::probe`] for that missing key answered (so it is empty or
    /// a tombstone).
    pub fn insert_at(&mut self, hi: Slot<E>, entry: E, hash: HashValue) {
        let item = &mut self.slots_mut()[hi.index()];
        let was_empty = item.is_empty();
        item.hi_key = entry;
        item.hi_hash = hash;
        self.ht_used = self.ht_used.wrapping_add(1);
        self.ht_changed += 1;
        if was_empty {
            self.ht_filled = self.ht_filled.wrapping_add(1);
        }
        hash_may_resize(self, 0);
    }

    /// Remove the entry in the live slot `hi`, leaving a tombstone.
    pub fn remove_at(&mut self, hi: Slot<E>) {
        self.slots_mut()[hi.index()].hi_key = E::removed();
        self.ht_used = self.ht_used.wrapping_sub(1);
        self.ht_changed += 1;
        hash_may_resize(self, 0);
    }

    /// Hold off resizing, so slot indexes stay meaningful across mutations
    /// until the matching [`HashTab::unlock`].
    pub fn lock(&mut self) {
        self.ht_locked += 1;
    }

    /// Undo one [`HashTab::lock`], resizing now if the table wanted to
    /// meanwhile.
    pub fn unlock(&mut self) {
        self.ht_locked -= 1;
        hash_may_resize(self, 0);
    }
}

/// The probe sequence: CPython-dict-style perturbed probing, bit-exact with
/// the C code. The first index is the masked hash; each successor is
/// `idx * 5 + perturb + 1` computed from the previous (masked-on-first,
/// unmasked-after) index. Never terminates — every walk ends by finding an
/// empty slot, which is guaranteed because the table is never full.
pub(crate) struct Probe {
    idx: HashValue,
    perturb: HashValue,
    mask: HashValue,
    first: bool,
}

impl Probe {
    pub(crate) fn new(hash: HashValue, mask: HashValue) -> Self {
        Probe {
            idx: hash & mask,
            perturb: hash,
            mask,
            first: true,
        }
    }
}

impl Iterator for Probe {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.first {
            self.first = false;
            return Some(self.idx);
        }
        self.idx = self
            .idx
            .wrapping_mul(5)
            .wrapping_add(self.perturb)
            .wrapping_add(1);
        self.perturb >>= PERTURB_SHIFT;
        Some(self.idx & self.mask)
    }
}

/// The `hash_hash` fold: seed with the first byte, then `hash * 101 + byte`.
/// An empty key hashes to 0 (the C code returns early when the first byte is
/// NUL, which for a C string means empty).
pub fn hash_bytes(key: &[u8]) -> HashValue {
    let (&first, rest) = match key.split_first() {
        Some(split) => split,
        None => return 0,
    };
    if first == 0 {
        return 0;
    }
    fold(first, rest)
}

/// The fold itself: `hash * 101 + byte`, seeded with `first`.
fn fold(first: u8, rest: &[u8]) -> HashValue {
    rest.iter().fold(HashValue::from(first), |hash, &b| {
        hash.wrapping_mul(101).wrapping_add(HashValue::from(b))
    })
}

/// The `hash_hash_len` fold differs deliberately: it consumes exactly `len`
/// bytes without stopping at NUL, and a leading NUL byte seeds the fold with
/// 0 instead of ending it.
pub fn hash_bytes_len(key: &[u8]) -> HashValue {
    match key.split_first() {
        Some((&first, rest)) => fold(first, rest),
        None => 0,
    }
}

/// Decide whether (and to what size) the table must be reallocated. `None`
/// means leave it alone. Thresholds are verbatim from the C code.
///
/// The C asked whether the array was still the inline `ht_smallarray`; that
/// is the same question as `oldsize == HT_INIT_SIZE`, because the only way
/// off the inline array was to grow past it and the only way back was to
/// shrink to exactly its size.
pub(crate) fn resize_decision(
    filled: usize,
    used: usize,
    oldsize: usize,
    minitems: usize,
) -> Option<usize> {
    let array_is_small = oldsize == HT_INIT_SIZE;
    let minsize = if minitems == 0 {
        if filled < HT_INIT_SIZE - 1 && array_is_small {
            return None;
        }
        if filled.wrapping_mul(3) < oldsize.wrapping_mul(2) && used > oldsize.wrapping_div(5) {
            return None;
        }
        if used > 1000 {
            used.wrapping_mul(2)
        } else {
            used.wrapping_mul(4)
        }
    } else {
        let minitems = minitems.max(used);
        minitems.wrapping_mul(3).wrapping_add(1).wrapping_div(2)
    };

    let mut newsize = HT_INIT_SIZE;
    while newsize < minsize {
        newsize <<= 1;
        debug_assert!(newsize != 0, "hash table size overflow");
    }

    let newarray_is_small = newsize == HT_INIT_SIZE;
    if !newarray_is_small && newsize == oldsize && filled.wrapping_mul(3) < oldsize.wrapping_mul(2)
    {
        return None;
    }
    Some(newsize)
}

/// Move the `used` kept items of `old` into the empty `new` array, probing
/// with each item's stored hash. Stops scanning as soon as every kept item
/// has been moved, like the C loop.
fn rehash_into<E: SlotEntry>(old: &[HashItem<E>], new: &mut [HashItem<E>], used: usize) {
    let newmask = new.len() - 1;
    let mut todo = used;
    for item in old {
        if todo == 0 {
            break;
        }
        if item.is_kept() {
            for idx in Probe::new(item.hi_hash, newmask) {
                if new[idx].is_empty() {
                    new[idx] = *item;
                    break;
                }
            }
            todo -= 1;
        }
    }
}

/// Empty a live table, releasing its slots and giving it a fresh array of the
/// initial size. What the entries named is the caller's.
pub fn hash_reset<E: SlotEntry>(ht: &mut HashTab<E>) {
    *ht = HashTab::init();
}

/// Release the table's slots, leaving it with none. Nothing may probe the
/// table again until [`hash_reset`] gives it a new array. What the entries
/// named is the caller's.
pub fn hash_clear<E: SlotEntry>(ht: &mut HashTab<E>) {
    ht.drop_slots();
}

/// Grow, shrink, or compact (drop tombstones from) the slot array when the
/// load factors say so; `minitems` forces room for that many items up front.
///
/// The old array is moved out of the table before the new one goes in, so
/// the rehash is a straight move between two distinct runs -- the C had to
/// copy the inline array to the stack first whenever the destination was
/// that same inline array.
fn hash_may_resize<E: SlotEntry>(ht: &mut HashTab<E>, minitems: usize) {
    if ht.ht_locked > 0 {
        return;
    }
    let Some(newsize) = resize_decision(ht.ht_filled, ht.ht_used, ht.size(), minitems) else {
        return;
    };
    let used = ht.ht_used;
    ht.resize_slots(newsize, |old, new| rehash_into(old, new, used));
    ht.ht_filled = used;
    ht.ht_changed += 1;
}

pub type HashValue = size_t;

/// One slot of a [`HashTab`].
///
/// `Copy`: the entry names something the table does not own -- a dictionary
/// item, or an index into storage beside the table -- and copying the slot
/// copies only that name.
#[derive(Copy, Clone)]
pub struct HashItem<E: SlotEntry> {
    pub hi_hash: HashValue,
    pub hi_key: E,
}

/// Where a table's slots live.
///
/// The C kept the first sixteen in an inline `ht_smallarray` and pointed
/// `ht_array` at it, so the overwhelmingly common table -- a dictionary of a
/// handful of keys -- cost no allocation at all. This is that small case as
/// a *value*: it is stored in the `HashTab`, not pointed at from it, so
/// the table stays movable.
///
/// Growth past the small run moves to [`Slots::Heap`], and a shrink back to
/// exactly [`crate::hashtab::HT_INIT_SIZE`] slots returns to the run -- the
/// same two transitions the C made, which is why the resize's "is the array
/// still the small one" test still reads `oldsize == HT_INIT_SIZE`.
// `clippy::large_enum_variant` has nothing to say about a generic enum, but
// the size difference is deliberate either way: the small run is stored, not
// pointed at, which is what costs a dictionary no allocation.
enum Slots<E: SlotEntry> {
    /// No array at all: a [`HashTab::new`], or a table
    /// [`crate::hashtab::hash_clear`] emptied.
    None,
    /// The small run, in the table itself.
    Inline([HashItem<E>; HT_INIT_SIZE]),
    /// A grown table's array.
    Heap(Vec<HashItem<E>>),
}

impl<E: SlotEntry> Slots<E> {
    /// `size` empty slots, inline when that is the small run's size.
    fn with_size(size: usize) -> Self {
        if size == HT_INIT_SIZE {
            Slots::Inline([HashItem::EMPTY; HT_INIT_SIZE])
        } else {
            Slots::Heap(vec![HashItem::EMPTY; size])
        }
    }

    fn as_slice(&self) -> &[HashItem<E>] {
        match self {
            Slots::None => &[],
            Slots::Inline(run) => run,
            Slots::Heap(grown) => grown,
        }
    }

    fn as_mut_slice(&mut self) -> &mut [HashItem<E>] {
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
/// zero, before [`HashTab::init`] gives it one). That is the one
/// structural difference from the C, where `ht_array` pointed at the inline
/// `ht_smallarray` while the table was small -- a self-reference that made a
/// `HashTab` valid only at the address it was initialised at, and
/// therefore unwrappable and unmovable. The small run is still inline (see
/// [`Slots`]); nothing points at it.
///
/// What did *not* change is anything a caller can see: the hash, the probe
/// sequence, the resize thresholds and so the slot every key lands in, which
/// is [iteration order](Self::slots) and is Vim-visible through `keys()`,
/// `items()` and every listing that walks a Dictionary.
///
/// Entries are the caller's: the table compares keys through the test the
/// caller hands [`HashTab::probe`] and frees nothing they name.
///
/// # Slots are named by index, never by pointer
///
/// Because the small run lives *in* the table, a raw pointer into it is
/// derived from the `HashTab` itself and dies at the next
/// `&mut HashTab` -- which every mutation of the table takes. So a lookup
/// answers a [`crate::hashtab::Slot`], an index plus a copy of what the slot
/// held, and every write goes back through the table. An index survives a
/// mutation; what it does not survive is a *resize*, which is what
/// [`HashTab::lock`] exists to prevent.
pub struct HashTab<E: SlotEntry> {
    /// Live entries.
    pub ht_used: size_t,
    /// Entries plus tombstones: what the load factor is measured against.
    pub ht_filled: size_t,
    /// Bumped by every add, remove and resize, so a walk can tell that a
    /// callback rearranged the table under it.
    pub ht_changed: ::core::ffi::c_int,
    /// Non-zero while a caller holds slot indexes across mutations; see
    /// [`HashTab::lock`].
    pub ht_locked: ::core::ffi::c_int,
    /// The slots themselves.
    slots: Slots<E>,
}

impl<E: SlotEntry> Default for HashItem<E> {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl<E: SlotEntry> HashItem<E> {
    /// A slot that never held a key.
    pub const EMPTY: Self = Self {
        hi_hash: 0,
        hi_key: E::EMPTY,
    };
}

impl<E: SlotEntry> Default for HashTab<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: SlotEntry> HashTab<E> {
    /// A table with no slots at all: the state a `HashTab` field has
    /// before [`HashTab::init`] gives it its first array, and
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
    /// inline run: what [`HashTab::init`] answers.
    pub(crate) fn with_slots() -> Self {
        Self {
            slots: Slots::with_size(HT_INIT_SIZE),
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
    /// [`HashTab::init`] has not built is a bug.
    pub fn mask(&self) -> HashValue {
        self.slots.as_slice().len() - 1
    }

    /// Every slot, in index order -- empty ones and tombstones included.
    /// The live entries alone are [`HashTab::items`].
    pub fn slots(&self) -> &[HashItem<E>] {
        self.slots.as_slice()
    }

    /// Every slot, writable.
    pub fn slots_mut(&mut self) -> &mut [HashItem<E>] {
        self.slots.as_mut_slice()
    }

    /// Give the table a fresh array of `size` empty slots, handing `rehash`
    /// the old slots and the new ones so it can move the live entries over.
    pub(crate) fn resize_slots(
        &mut self,
        size: usize,
        rehash: impl FnOnce(&[HashItem<E>], &mut [HashItem<E>]),
    ) {
        let old = ::core::mem::replace(&mut self.slots, Slots::with_size(size));
        rehash(old.as_slice(), self.slots.as_mut_slice());
    }

    /// Release the table's slots, leaving it with none.
    pub(crate) fn drop_slots(&mut self) {
        self.slots = Slots::None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bare test entry: `0` is empty and `u32::MAX` the tombstone.
    #[derive(Copy, Clone, PartialEq, Debug)]
    struct K(u32);

    impl SlotEntry for K {
        const EMPTY: Self = K(0);

        fn is_empty(self) -> bool {
            self.0 == 0
        }

        fn is_removed(self) -> bool {
            self.0 == u32::MAX
        }

        fn removed() -> Self {
            K(u32::MAX)
        }
    }

    const EMPTY_ITEM: HashItem<K> = HashItem::EMPTY;

    /// The transpiled C probe loop, kept as the reference the iterator must
    /// match step for step.
    fn c_probe_reference(hash: HashValue, mask: HashValue, steps: usize) -> Vec<usize> {
        let mut out = vec![hash & mask];
        let mut idx = hash & mask;
        let mut perturb = hash;
        for _ in 1..steps {
            idx = idx.wrapping_mul(5).wrapping_add(perturb).wrapping_add(1);
            out.push(idx & mask);
            perturb >>= PERTURB_SHIFT;
        }
        out
    }

    #[test]
    fn probe_matches_the_c_sequence() {
        for &(hash, mask) in &[
            (0, 15),
            (1, 15),
            (0xdead_beef, 15),
            (usize::MAX, 63),
            (101_101, 1023),
        ] {
            let got: Vec<usize> = Probe::new(hash, mask).take(40).collect();
            assert_eq!(got, c_probe_reference(hash, mask, 40), "hash={hash:#x}");
        }
    }

    #[test]
    fn probe_eventually_visits_every_slot() {
        for hash in [0usize, 7, 12345, 0xffff_ffff] {
            let mut seen = [false; 16];
            for idx in Probe::new(hash, 15).take(300) {
                seen[idx] = true;
            }
            assert!(seen.iter().all(|&s| s), "hash={hash}");
        }
    }

    #[test]
    fn hash_of_empty_and_leading_nul() {
        assert_eq!(hash_bytes(b""), 0);
        assert_eq!(hash_bytes_len(b""), 0);
        // hash_hash_len keeps folding through NUL bytes.
        assert_ne!(hash_bytes_len(b"\0a"), 0);
    }

    #[test]
    fn hash_fold_matches_the_c_formula() {
        // hash("ab") = 'a' * 101 + 'b'
        assert_eq!(hash_bytes(b"ab"), 97 * 101 + 98);
        assert_eq!(hash_bytes_len(b"ab"), 97 * 101 + 98);
    }

    #[test]
    fn small_table_is_left_alone_until_nearly_filled() {
        assert_eq!(resize_decision(3, 3, HT_INIT_SIZE, 0), None);
        assert_eq!(resize_decision(14, 14, HT_INIT_SIZE, 0), None);
        // 15 filled slots trip the resize even on the initial array.
        assert_eq!(resize_decision(15, 15, HT_INIT_SIZE, 0), Some(64));
    }

    #[test]
    fn tombstone_heavy_table_is_compacted_in_place() {
        // Many tombstones (filled ≫ used) with few live items: same size,
        // fresh array.
        assert_eq!(resize_decision(60, 10, 64, 0), Some(64));
    }

    #[test]
    fn minitems_reserves_capacity() {
        assert_eq!(resize_decision(0, 0, HT_INIT_SIZE, 100), Some(256));
        // `minitems` never shrinks below what is already in the table: 200
        // live items ask for 300, i.e. 512 slots, whatever `minitems` says.
        assert_eq!(resize_decision(200, 200, 256, 1), Some(512));
    }

    #[test]
    fn a_grown_table_shrinks_back_to_the_initial_size() {
        // Almost everything removed from a big table: back to 16 slots.
        assert_eq!(resize_decision(1000, 2, 2048, 0), Some(HT_INIT_SIZE));
    }

    #[test]
    fn a_comfortable_table_is_left_alone() {
        // Under two thirds filled and over a fifth live: no work to do.
        assert_eq!(resize_decision(100, 100, 256, 0), None);
        // Same load, but the array is the initial one and `filled` has passed
        // HT_INIT_SIZE - 1, so the small-array early-out no longer applies.
        assert_eq!(resize_decision(15, 15, HT_INIT_SIZE, 0), Some(64));
    }

    #[test]
    fn a_big_table_doubles_rather_than_quadruples() {
        // Up to 1000 live items the new size covers 4x; past it, 2x. Both
        // round up to a power of two.
        assert_eq!(resize_decision(1000, 1000, 1024, 0), Some(4096));
        assert_eq!(resize_decision(1001, 1001, 1024, 0), Some(2048));
    }

    #[test]
    fn rehash_stops_once_every_live_item_has_moved() {
        // `used` is the contract, not the array's contents: the walk stops
        // after that many kept items, which is what makes it O(used) rather
        // than O(size) on a mostly-empty table.
        let key = K(1);
        let mut old = [EMPTY_ITEM; 16];
        for (i, slot) in old.iter_mut().enumerate() {
            *slot = HashItem {
                hi_hash: i,
                hi_key: key,
            };
        }
        let mut new = [EMPTY_ITEM; 16];
        rehash_into(&old, &mut new, 3);
        assert_eq!(new.iter().filter(|hi| hi.is_kept()).count(), 3);
        // The first three, in array order.
        assert!([0, 1, 2].iter().all(|&i| new[i].is_kept()));
    }

    #[test]
    fn rehash_probes_past_a_taken_slot() {
        // Four hashes that all mask to slot 1 under mask 7 land in the four
        // successive slots the probe sequence visits, in insertion order.
        let key = K(1);
        let mut old = [EMPTY_ITEM; 16];
        for (n, slot) in old.iter_mut().take(4).enumerate() {
            *slot = HashItem {
                hi_hash: 1 + n * 8,
                hi_key: key,
            };
        }
        let mut new = [EMPTY_ITEM; 8];
        rehash_into(&old, &mut new, 4);
        assert_eq!(new.iter().filter(|hi| hi.is_kept()).count(), 4);
        for (n, slot) in old.iter().take(4).enumerate() {
            let landed = Probe::new(slot.hi_hash, 7)
                .find(|&idx| new[idx].hi_hash == slot.hi_hash)
                .expect("every item is somewhere on its own probe sequence");
            assert!(landed < 8, "{n}");
        }
    }

    #[test]
    fn rehash_moves_every_kept_item() {
        let key = K(1);
        let mut old = [EMPTY_ITEM; 16];
        old[2] = HashItem {
            hi_hash: 2,
            hi_key: key,
        };
        old[3] = HashItem {
            hi_hash: 18, // collides with slot 2 under mask 15
            hi_key: key,
        };
        old[5] = HashItem {
            hi_hash: 5,
            hi_key: K::removed(), // tombstone: must not survive the rehash
        };
        let mut new = [EMPTY_ITEM; 32];
        rehash_into(&old, &mut new, 2);
        let kept: Vec<&HashItem<K>> = new.iter().filter(|hi| hi.is_kept()).collect();
        assert_eq!(kept.len(), 2);
        assert_eq!(new[2].hi_hash, 2);
        assert_eq!(new[18].hi_hash, 18);
    }

    /// The table is a value now: it can be built, filled, moved and dropped
    /// like any other, which is what the self-referential `ht_smallarray`
    /// made impossible.
    #[test]
    fn a_table_is_a_movable_value() {
        let mut ht = HashTab::init();
        assert_eq!(ht.size(), HT_INIT_SIZE);
        let hash = hash_bytes(b"a");
        let hi = ht.probe(hash, |k: K| k == K(1));
        assert!(!hi.is_kept(), "a fresh table has no `a`");
        ht.insert_at(hi, K(1), hash);
        let moved = ht;
        assert_eq!(moved.ht_used, 1);
        assert_eq!(
            moved.items().map(|hi| hi.hi_key).collect::<Vec<_>>(),
            [K(1)]
        );
    }
}
