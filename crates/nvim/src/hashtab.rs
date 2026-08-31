//! Open-addressing hash table (`hashtab_T`).
//!
//! The table owns its slots. What is frozen is the *behaviour*: the hash, the
//! probe sequence, the resize thresholds, and therefore the slot every key
//! lands in. Callers walk the slots directly and Vim shows the result --
//! `keys()`, `items()` and every listing of a Dictionary hand out slot order
//! -- so placement is observable, not an implementation detail.
//!
//! Keys are borrowed C strings owned by the callers; the table compares them
//! and frees them only on the caller's behalf ([`hash_clear_all`]).
//!
//! # Boundary
//!
//! [`hash_lookup`] and friends answer a `*mut hashitem_T` that callers hold
//! across further table mutations -- that is the contract [`hash_lock`]
//! exists for, and it is why the lookups take a raw table pointer rather than
//! a reference: handing out a reference into a table a caller then mutates
//! would be the alias the raw pointer avoids. The two entry points that
//! cannot hand out a slot ([`hash_clear`], [`hash_reset`]) take `&mut` and
//! are safe.
//!
//! # Initialising storage
//!
//! [`hash_init`] writes over storage that holds no table yet -- freshly
//! `xcalloc`'d memory, or a table whose slots have just been moved out. It
//! never drops what was there. A table that *does* own slots is emptied with
//! [`hash_reset`], which frees them; calling `hash_init` on one leaks the
//! array, exactly as the C did.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::message_fmt::c_str;
use crate::siemsg;
use core::ffi::{CStr, c_char, c_uint, c_void};
use core::slice;

use crate::memory::xfree;

use crate::types::{Failed, hash_T, hashitem_T, hashtab_T};

/// The number of slots a table starts with, and the size it shrinks back to.
pub const HT_INIT_SIZE: usize = 16;

const PERTURB_SHIFT: u32 = 5;

/// Sentinel for a removed item: `hi_key` equal to this *address* marks a
/// tombstone. Private, because the question every caller used to ask of it
/// -- "is this slot live?" -- is [`hashitem_T::is_kept`]. Never written
/// through.
static hash_removed: c_char = 0;

fn removed_sentinel() -> *mut c_char {
    (&raw const hash_removed).cast_mut()
}

impl hashitem_T {
    /// Never held a key.
    pub fn is_empty(&self) -> bool {
        self.hi_key.is_null()
    }

    /// Held a key that was removed: a tombstone, which a probe walks past but
    /// an insertion may reuse.
    pub fn is_removed(&self) -> bool {
        self.hi_key == removed_sentinel()
    }

    /// Holds a live key (neither empty nor a tombstone). This is the
    /// `HASHITEM_EMPTY` test every caller that walks the slots open-codes.
    pub fn is_kept(&self) -> bool {
        !self.is_empty() && !self.is_removed()
    }
}

impl hashtab_T {
    /// An empty table with its first slot array: what [`hash_init`] writes,
    /// and what a caller that owns its table outright builds directly.
    pub fn init() -> Self {
        let mut ht = hashtab_T::new();
        ht.replace_slots(vec![hashitem_T::EMPTY; HT_INIT_SIZE]);
        ht
    }

    /// The live entries, in slot order: what every walk of a table hands out,
    /// and the order Vim shows.
    ///
    /// The `ht_used` countdown the C loops use is an optimisation on top of
    /// this -- it stops scanning once the last live entry has been seen --
    /// and yields exactly the same entries in exactly the same order.
    pub fn items(&self) -> impl Iterator<Item = &hashitem_T> {
        self.slots().iter().filter(|hi| hi.is_kept())
    }
}

/// The probe sequence: CPython-dict-style perturbed probing, bit-exact with
/// the C code. The first index is the masked hash; each successor is
/// `idx * 5 + perturb + 1` computed from the previous (masked-on-first,
/// unmasked-after) index. Never terminates — every walk ends by finding an
/// empty slot, which is guaranteed because the table is never full.
struct Probe {
    idx: hash_T,
    perturb: hash_T,
    mask: hash_T,
    first: bool,
}

impl Probe {
    fn new(hash: hash_T, mask: hash_T) -> Self {
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
fn hash_bytes(key: &[u8]) -> hash_T {
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
fn fold(first: u8, rest: &[u8]) -> hash_T {
    rest.iter().fold(hash_T::from(first), |hash, &b| {
        hash.wrapping_mul(101).wrapping_add(hash_T::from(b))
    })
}

/// The `hash_hash_len` fold differs deliberately: it consumes exactly `len`
/// bytes without stopping at NUL, and a leading NUL byte seeds the fold with
/// 0 instead of ending it.
fn hash_bytes_len(key: &[u8]) -> hash_T {
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
fn resize_decision(filled: usize, used: usize, oldsize: usize, minitems: usize) -> Option<usize> {
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
fn rehash_into(old: &[hashitem_T], new: &mut [hashitem_T], used: usize) {
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

/// Set up an empty table in storage that holds no table yet.
///
/// # Safety
///
/// `ht` points to writable `hashtab_T` storage that does not already own a
/// slot array -- freshly allocated (`xcalloc`'d or uninitialised) memory, or
/// a table whose slots have just been moved out. The old bytes are
/// overwritten without being dropped, which is what makes it usable on
/// memory that never held a valid `hashtab_T`. To empty a table that *is*
/// live, use [`hash_reset`].
pub unsafe fn hash_init(ht: *mut hashtab_T) {
    // SAFETY: the caller's storage. `write` does not drop what was there,
    // which for uninitialised memory is the whole point.
    unsafe { ht.write(hashtab_T::init()) };
}

/// Empty a live table, releasing its slots and giving it a fresh array of the
/// initial size. The keys are the caller's (see [`hash_clear_all`]).
pub fn hash_reset(ht: &mut hashtab_T) {
    *ht = hashtab_T::init();
}

/// Release the table's slots, leaving it with none. Nothing may probe the
/// table again until [`hash_init`] or [`hash_reset`] gives it a new array.
/// The keys are the caller's (see [`hash_clear_all`]).
pub fn hash_clear(ht: &mut hashtab_T) {
    ht.replace_slots(Vec::new());
}

/// Free the table's slots *and* every key, where each key pointer was offset
/// by `off` bytes into its allocation (keys living inside larger structs).
///
/// # Safety
///
/// `ht` points to a live `hashtab_T` -- one [`hash_init`] has run on -- and
/// every live key is `off` bytes into an `xmalloc`-family allocation this
/// call takes over.
pub unsafe fn hash_clear_all(ht: *mut hashtab_T, off: c_uint) {
    // SAFETY: the caller's table.
    let table = unsafe { &mut *ht };
    for hi in table.items() {
        // SAFETY: the caller's promise about where a key starts.
        unsafe { xfree(hi.hi_key.sub(off as usize).cast::<c_void>()) };
    }
    hash_clear(table);
}

/// # Safety
///
/// `ht` points to a live `hashtab_T` and `key` is NUL-terminated. See
/// [`hash_lookup`] for what the answer means.
pub unsafe fn hash_find(ht: *const hashtab_T, key: *const c_char) -> *mut hashitem_T {
    // SAFETY: the caller's table and NUL-terminated key.
    unsafe {
        hash_lookup(
            ht,
            key,
            CStr::from_ptr(key).to_bytes().len(),
            hash_hash(key),
        )
    }
}

/// # Safety
///
/// `ht` points to a live `hashtab_T` and `key` is readable for `len` bytes.
/// See [`hash_lookup`] for what the answer means.
pub unsafe fn hash_find_len(
    ht: *const hashtab_T,
    key: *const c_char,
    len: usize,
) -> *mut hashitem_T {
    // SAFETY: the caller's table and `len`-byte key.
    unsafe { hash_lookup(ht, key, len, hash_hash_len(key, len)) }
}

/// The slot `wanted` occupies, or -- for an absent key -- the slot it belongs
/// in: a tombstone if the walk crossed one, else the empty slot that ended
/// it.
///
/// # Safety
///
/// Every live key in `ht` is NUL-terminated.
unsafe fn lookup_slot(ht: &hashtab_T, wanted: &[u8], hash: hash_T) -> usize {
    let slots = ht.slots();
    let mut freeitem: Option<usize> = None;
    // The probe never runs off the array: it is masked to `mask()`, and the
    // table is never full, so some slot is empty and ends the walk.
    for idx in Probe::new(hash, ht.mask()) {
        let item = &slots[idx];
        if item.is_empty() {
            return freeitem.unwrap_or(idx);
        }
        if item.is_removed() {
            freeitem.get_or_insert(idx);
        // SAFETY: the caller's promise that live keys are NUL-terminated.
        } else if item.hi_hash == hash
            && unsafe { CStr::from_ptr(item.hi_key) }.to_bytes() == wanted
        {
            return idx;
        }
    }
    unreachable!("probe sequence always finds an empty slot");
}

/// Find `key` (of `key_len` bytes, hashing to `hash`): returns the item
/// holding it, or — for an absent key — the slot where it belongs (a
/// tombstone if the walk crossed one, else the empty slot that ended it).
///
/// # Safety
///
/// `ht` points to a live `hashtab_T` whose live keys are NUL-terminated, and
/// `key` is readable for `key_len` bytes. The answer is a pointer *into* the
/// table's slot array, so it dies at the next resize.
pub unsafe fn hash_lookup(
    ht: *const hashtab_T,
    key: *const c_char,
    key_len: usize,
    hash: hash_T,
) -> *mut hashitem_T {
    // SAFETY: the caller's key and table.
    let wanted = unsafe { slice::from_raw_parts(key.cast::<u8>(), key_len) };
    let table = unsafe { &*ht };
    let idx = unsafe { lookup_slot(table, wanted, hash) };
    // SAFETY: `idx` is one of the table's own slots.
    unsafe { table.slot_ptr().add(idx) }
}

/// Add `key` to the table, complaining (and failing) when it is already
/// there. The key stays the caller's.
///
/// # Safety
///
/// `ht` points to a live `hashtab_T` and `key` is NUL-terminated and stays
/// alive for as long as the table holds it.
pub unsafe fn hash_add(ht: *mut hashtab_T, key: *mut c_char) -> Result<(), Failed> {
    // SAFETY: the caller's table and NUL-terminated key.
    let hash = unsafe { hash_hash(key) };
    let hi = unsafe { hash_lookup(ht, key, CStr::from_ptr(key).to_bytes().len(), hash) };
    if unsafe { &*hi }.is_kept() {
        // SAFETY: `%s` spends the NUL-terminated key.
        let key = unsafe { c_str(key) };
        siemsg!("E685: Internal error: hash_add(): duplicate key \"{key}\"");
        return Err(Failed);
    }
    unsafe { hash_add_item(ht, hi, key, hash) };
    Ok(())
}

/// Add `key` at `hi`, which the caller obtained from `hash_lookup` on a
/// missing key (so it is empty or a tombstone).
///
/// # Safety
///
/// `hi` is a slot of `ht`'s current array holding no live key, `hash` is
/// `key`'s hash, and `key` outlives its stay in the table.
pub unsafe fn hash_add_item(
    ht: *mut hashtab_T,
    hi: *mut hashitem_T,
    key: *mut c_char,
    hash: hash_T,
) {
    // SAFETY: the caller's slot, which is one of `ht`'s own.
    let was_empty = unsafe {
        let was_empty = (*hi).is_empty();
        (*hi).hi_key = key;
        (*hi).hi_hash = hash;
        was_empty
    };
    // SAFETY: the caller's table.
    let table = unsafe { &mut *ht };
    table.ht_used = table.ht_used.wrapping_add(1);
    table.ht_changed += 1;
    if was_empty {
        table.ht_filled = table.ht_filled.wrapping_add(1);
    }
    hash_may_resize(table, 0);
}

/// Remove the item at `hi` (leaving a tombstone). The key itself belongs to
/// the caller.
///
/// # Safety
///
/// `hi` is a slot of `ht`'s current array holding a live key.
pub unsafe fn hash_remove(ht: *mut hashtab_T, hi: *mut hashitem_T) {
    // SAFETY: the caller's slot, which is one of `ht`'s own.
    unsafe { (*hi).hi_key = removed_sentinel() };
    // SAFETY: the caller's table.
    let table = unsafe { &mut *ht };
    table.ht_used = table.ht_used.wrapping_sub(1);
    table.ht_changed += 1;
    hash_may_resize(table, 0);
}

/// Lock out resizing while a caller walks the slots or holds item pointers
/// across mutations.
///
/// # Safety
///
/// `ht` points to a live `hashtab_T`, and the lock is released exactly once.
pub unsafe fn hash_lock(ht: *mut hashtab_T) {
    // SAFETY: the caller's table.
    unsafe { (*ht).ht_locked += 1 };
}

/// Undo one [`hash_lock`], resizing now if the table wanted to meanwhile.
///
/// # Safety
///
/// `ht` points to a live `hashtab_T` this caller locked, and no item pointer
/// into its array is held past the call.
pub unsafe fn hash_unlock(ht: *mut hashtab_T) {
    // SAFETY: the caller's table.
    let table = unsafe { &mut *ht };
    table.ht_locked -= 1;
    hash_may_resize(table, 0);
}

/// Grow, shrink, or compact (drop tombstones from) the slot array when the
/// load factors say so; `minitems` forces room for that many items up front.
///
/// The old and the new array are always separate allocations, so the rehash
/// is a straight move between them -- the C had to copy the inline array to
/// the stack first whenever the destination was that same inline array.
fn hash_may_resize(ht: &mut hashtab_T, minitems: usize) {
    if ht.ht_locked > 0 {
        return;
    }
    let Some(newsize) = resize_decision(ht.ht_filled, ht.ht_used, ht.size(), minitems) else {
        return;
    };
    let old = ht.replace_slots(vec![hashitem_T::EMPTY; newsize]);
    let used = ht.ht_used;
    rehash_into(&old, ht.slots_mut(), used);
    ht.ht_filled = used;
    ht.ht_changed += 1;
}

/// A NUL-terminated key's hash.
///
/// # Safety
///
/// `key` is NUL-terminated.
pub unsafe fn hash_hash(key: *const c_char) -> hash_T {
    // SAFETY: the caller's NUL-terminated key.
    hash_bytes(unsafe { CStr::from_ptr(key) }.to_bytes())
}

/// A `len`-byte key's hash, NUL bytes included.
///
/// # Safety
///
/// `key` is readable for `len` bytes.
pub unsafe fn hash_hash_len(key: *const c_char, len: usize) -> hash_T {
    // SAFETY: the caller's `len` readable bytes.
    hash_bytes_len(unsafe { slice::from_raw_parts(key.cast::<u8>(), len) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr;

    const EMPTY_ITEM: hashitem_T = hashitem_T::EMPTY;

    /// The transpiled C probe loop, kept as the reference the iterator must
    /// match step for step.
    fn c_probe_reference(hash: hash_T, mask: hash_T, steps: usize) -> Vec<usize> {
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
        let key: *mut c_char = ptr::without_provenance_mut(0x1000);
        let mut old = [EMPTY_ITEM; 16];
        for (i, slot) in old.iter_mut().enumerate() {
            *slot = hashitem_T {
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
        let key: *mut c_char = ptr::without_provenance_mut(0x1000);
        let mut old = [EMPTY_ITEM; 16];
        for (n, slot) in old.iter_mut().take(4).enumerate() {
            *slot = hashitem_T {
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
        let sentinel = removed_sentinel();
        // Dangling but never dereferenced; without_provenance keeps Miri
        // from treating it as an exposed integer-to-pointer cast.
        let key: *mut c_char = ptr::without_provenance_mut(0x1000);
        let mut old = [EMPTY_ITEM; 16];
        old[2] = hashitem_T {
            hi_hash: 2,
            hi_key: key,
        };
        old[3] = hashitem_T {
            hi_hash: 18, // collides with slot 2 under mask 15
            hi_key: key,
        };
        old[5] = hashitem_T {
            hi_hash: 5,
            hi_key: sentinel, // tombstone: must not survive the rehash
        };
        let mut new = [EMPTY_ITEM; 32];
        rehash_into(&old, &mut new, 2);
        let kept: Vec<&hashitem_T> = new.iter().filter(|hi| hi.is_kept()).collect();
        assert_eq!(kept.len(), 2);
        assert_eq!(new[2].hi_hash, 2);
        assert_eq!(new[18].hi_hash, 18);
    }

    /// The table is a value now: it can be built, filled, moved and dropped
    /// like any other, which is what the self-referential `ht_smallarray`
    /// made impossible.
    #[test]
    fn a_table_is_a_movable_value() {
        let mut ht = hashtab_T::init();
        assert_eq!(ht.size(), HT_INIT_SIZE);
        let key = c"a".as_ptr().cast_mut();
        // SAFETY: the key is a `'static` C string, so it outlives the table.
        unsafe { hash_add(&raw mut ht, key) }.expect("a fresh table has no `a`");
        let moved = ht;
        assert_eq!(moved.ht_used, 1);
        assert_eq!(moved.items().map(|hi| hi.hi_key).collect::<Vec<_>>(), [key]);
    }
}
