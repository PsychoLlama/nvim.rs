//! Open-addressing hash table (`hashtab_T`): safe core + C-ABI shims.
//!
//! The struct layouts are frozen: callers iterate `ht_array` directly and
//! stash pointers to items, so both the layout and the probe sequence (which
//! decides where items land, and therefore iteration order) must match the C
//! implementation exactly. Keys are borrowed C strings owned by the callers;
//! the table compares them and frees them only on the caller's behalf
//! (`hash_clear_all`). Allocation stays on the `xmalloc` family.
//!
//! # Boundary
//!
//! Every entry point takes a raw `hashtab_T` pointer because its callers
//! hold one -- a field of `buf_T`, of a `dict_T`, a `static`. Each turns it
//! into a reference once, at the top; what it may *not* do is hand out a
//! reference, because `hash_lookup` and friends answer a `*mut hashitem_T`
//! that callers hold across further table mutations. That contract is the
//! reason `ht_locked` exists.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::siemsg_c;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::{ptr, slice};

use crate::memory::{xcalloc, xfree};
use crate::os::cshim::gettext;

use crate::types::{FAIL, OK, hash_T, hashitem_T, hashtab_T};

/// The array a table starts with, inline in the struct. Growing past it moves
/// to the heap; shrinking back to this size moves back in.
pub const HT_INIT_SIZE: usize = 16;

const PERTURB_SHIFT: u32 = 5;

const EMPTY_ITEM: hashitem_T = hashitem_T {
    hi_hash: 0,
    hi_key: ptr::null_mut(),
};

/// Sentinel for a removed item: `hi_key` equal to this *address* marks a
/// tombstone. Exported because other modules compare against it. Never
/// written through.
pub static hash_removed: c_char = 0;

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
    /// `HASHITEM_EMPTY` test every caller that walks `ht_array` open-codes.
    pub fn is_kept(&self) -> bool {
        !self.is_empty() && !self.is_removed()
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
fn resize_decision(
    filled: usize,
    used: usize,
    oldsize: usize,
    array_is_small: bool,
    minitems: usize,
) -> Option<usize> {
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

/// Move the `used` kept items of `old` into the zeroed `new` array, probing
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

/// The table's inline array, as an item pointer: the value `ht_array` holds
/// while the table is small.
///
/// Derived from the raw pointer, never from a reference to the table: the
/// result is written through for as long as the table stays small, and a
/// pointer that borrowed the struct would not survive the next field write.
///
/// # Safety
///
/// `ht` points to a live (or at least allocated) `hashtab_T`.
unsafe fn small_array(ht: *mut hashtab_T) -> *mut hashitem_T {
    // SAFETY: the caller's promise; taking a field's address reads nothing.
    unsafe { &raw mut (*ht).ht_smallarray }.cast::<hashitem_T>()
}

/// Set up an empty table, its array pointing at its own inline storage.
///
/// # Safety
///
/// `ht` points to writable, possibly uninitialized `hashtab_T` storage.
pub unsafe fn hash_init(ht: *mut hashtab_T) {
    // SAFETY: the caller's storage, written before anything reads it, and
    // the inline array it then points at is part of that same storage.
    unsafe {
        *ht = hashtab_T {
            ht_mask: HT_INIT_SIZE - 1,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ptr::null_mut(),
            ht_smallarray: [EMPTY_ITEM; HT_INIT_SIZE],
        };
        (*ht).ht_array = small_array(ht);
    }
}

/// Release the table's array, if it ever left the inline one. The keys are
/// the caller's (see [`hash_clear_all`]).
///
/// # Safety
///
/// `ht` points to a live `hashtab_T`.
pub unsafe fn hash_clear(ht: *mut hashtab_T) {
    // SAFETY: the caller's table; a grown `ht_array` is its own allocation.
    let array = unsafe { (*ht).ht_array };
    if array != unsafe { small_array(ht) } {
        unsafe { xfree(array.cast::<c_void>()) };
    }
}

/// Free the table *and* every key, where each key pointer was offset by
/// `off` bytes into its allocation (keys living inside larger structs).
///
/// # Safety
///
/// `ht` points to a live or all-zero `hashtab_T`, and every live key is `off`
/// bytes into an `xmalloc`-family allocation this call takes over.
pub unsafe fn hash_clear_all(ht: *mut hashtab_T, off: c_uint) {
    // SAFETY: the caller's table.
    let (mut todo, array, size) = unsafe { ((*ht).ht_used, (*ht).ht_array, (*ht).ht_mask + 1) };
    // Error paths free zeroed, never-initialized tables whose ht_array is
    // still null; like the C loop, don't touch the array unless a live
    // item needs freeing.
    if todo > 0 {
        // SAFETY: a table with a live item has an array of `ht_mask + 1`.
        for hi in unsafe { slice::from_raw_parts(array, size) } {
            if todo == 0 {
                break;
            }
            if hi.is_kept() {
                // SAFETY: the caller's promise about where a key starts.
                unsafe { xfree(hi.hi_key.sub(off as usize).cast::<c_void>()) };
                todo -= 1;
            }
        }
    }
    unsafe { hash_clear(ht) };
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

/// Find `key` (of `key_len` bytes, hashing to `hash`): returns the item
/// holding it, or — for an absent key — the slot where it belongs (a
/// tombstone if the walk crossed one, else the empty slot that ended it).
///
/// # Safety
///
/// `ht` points to a live `hashtab_T` whose live keys are NUL-terminated, and
/// `key` is readable for `key_len` bytes. The answer is a pointer *into* the
/// table's array, so it dies at the next resize.
pub unsafe fn hash_lookup(
    ht: *const hashtab_T,
    key: *const c_char,
    key_len: usize,
    hash: hash_T,
) -> *mut hashitem_T {
    // SAFETY: the caller's key and table. The probe never runs off the
    // array: it is masked to `ht_mask`, and the table is never full, so some
    // slot is empty and ends the walk.
    let wanted = unsafe { slice::from_raw_parts(key.cast::<u8>(), key_len) };
    let (array, mask) = unsafe { ((*ht).ht_array, (*ht).ht_mask) };
    let mut freeitem: *mut hashitem_T = ptr::null_mut();
    for idx in Probe::new(hash, mask) {
        let hi = unsafe { array.add(idx) };
        let item = unsafe { &*hi };
        if item.is_empty() {
            return if freeitem.is_null() { hi } else { freeitem };
        }
        if item.is_removed() {
            if freeitem.is_null() {
                freeitem = hi;
            }
        } else if item.hi_hash == hash
            && unsafe { CStr::from_ptr(item.hi_key) }.to_bytes() == wanted
        {
            return hi;
        }
    }
    unreachable!("probe sequence always finds an empty slot");
}

/// Add `key` to the table, complaining (and failing) when it is already
/// there. The key stays the caller's.
///
/// # Safety
///
/// `ht` points to a live `hashtab_T` and `key` is NUL-terminated and stays
/// alive for as long as the table holds it.
pub unsafe fn hash_add(ht: *mut hashtab_T, key: *mut c_char) -> c_int {
    // SAFETY: the caller's table and NUL-terminated key.
    let hash = unsafe { hash_hash(key) };
    let hi = unsafe { hash_lookup(ht, key, CStr::from_ptr(key).to_bytes().len(), hash) };
    if unsafe { &*hi }.is_kept() {
        let fmt = c"E685: Internal error: hash_add(): duplicate key \"%s\"";
        // SAFETY: `%s` spends the NUL-terminated key.
        unsafe { siemsg_c!(gettext(fmt), key) };
        return FAIL;
    }
    unsafe { hash_add_item(ht, hi, key, hash) };
    OK
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
    // SAFETY: the caller's table and one of its own slots.
    unsafe {
        (*ht).ht_used = (*ht).ht_used.wrapping_add(1);
        (*ht).ht_changed += 1;
        if (*hi).is_empty() {
            (*ht).ht_filled = (*ht).ht_filled.wrapping_add(1);
        }
        (*hi).hi_key = key;
        (*hi).hi_hash = hash;
        hash_may_resize(ht, 0);
    }
}

/// Remove the item at `hi` (leaving a tombstone). The key itself belongs to
/// the caller.
///
/// # Safety
///
/// `hi` is a slot of `ht`'s current array holding a live key.
pub unsafe fn hash_remove(ht: *mut hashtab_T, hi: *mut hashitem_T) {
    // SAFETY: the caller's table and one of its own slots.
    unsafe {
        (*ht).ht_used = (*ht).ht_used.wrapping_sub(1);
        (*ht).ht_changed += 1;
        (*hi).hi_key = removed_sentinel();
        hash_may_resize(ht, 0);
    }
}

/// Lock out resizing while a caller iterates `ht_array` or holds item
/// pointers across mutations.
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
    unsafe {
        (*ht).ht_locked -= 1;
        hash_may_resize(ht, 0);
    }
}

/// Grow, shrink, or compact (drop tombstones from) the array when the load
/// factors say so; `minitems` forces room for that many items up front.
///
/// # Safety
///
/// `ht` points to a live `hashtab_T`, and no item pointer into its array is
/// held across the call (that is what `ht_locked` is for).
unsafe fn hash_may_resize(ht: *mut hashtab_T, minitems: usize) {
    // SAFETY: the caller's table, and its own inline array.
    let smallarray = unsafe { small_array(ht) };
    let table = unsafe { &mut *ht };
    if table.ht_locked > 0 {
        return;
    }
    let oldsize = table.ht_mask + 1;
    let was_small = table.ht_array == smallarray;
    let Some(newsize) =
        resize_decision(table.ht_filled, table.ht_used, oldsize, was_small, minitems)
    else {
        return;
    };

    // Moving back into the inline array means copying the items out of it
    // first, because the destination is the same storage.
    let newarray_is_small = newsize == HT_INIT_SIZE;
    let mut temparray = [EMPTY_ITEM; HT_INIT_SIZE];
    let oldarray = if newarray_is_small && was_small {
        temparray = table.ht_smallarray;
        temparray.as_mut_ptr()
    } else {
        table.ht_array
    };
    let newarray = if newarray_is_small {
        table.ht_smallarray = [EMPTY_ITEM; HT_INIT_SIZE];
        smallarray
    } else {
        // SAFETY: a zeroed slot is an empty one, which is what `rehash_into`
        // probes for -- the zeroing is load-bearing, not hygiene.
        unsafe { xcalloc(newsize, size_of::<hashitem_T>()) }.cast::<hashitem_T>()
    };

    // SAFETY: both arrays are `oldsize`/`newsize` items of their own, and
    // they only overlap in the `newarray_is_small && was_small` case, where
    // the old one is the copy on the stack.
    let old = unsafe { slice::from_raw_parts(oldarray, oldsize) };
    let new = unsafe { slice::from_raw_parts_mut(newarray, newsize) };
    rehash_into(old, new, table.ht_used);

    if !was_small {
        unsafe { xfree(table.ht_array.cast::<c_void>()) };
    }
    table.ht_array = newarray;
    table.ht_mask = newsize - 1;
    table.ht_filled = table.ht_used;
    table.ht_changed += 1;
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
        assert_eq!(resize_decision(3, 3, 16, true, 0), None);
        assert_eq!(resize_decision(14, 14, 16, true, 0), None);
        // 15 filled slots trip the resize even on the small array.
        assert_eq!(resize_decision(15, 15, 16, true, 0), Some(64));
    }

    #[test]
    fn tombstone_heavy_table_is_compacted_in_place() {
        // Many tombstones (filled ≫ used) with few live items: same size,
        // fresh array.
        assert_eq!(resize_decision(60, 10, 64, false, 0), Some(64));
    }

    #[test]
    fn minitems_reserves_capacity() {
        assert_eq!(resize_decision(0, 0, 16, true, 100), Some(256));
        // `minitems` never shrinks below what is already in the table: 200
        // live items ask for 300, i.e. 512 slots, whatever `minitems` says.
        assert_eq!(resize_decision(200, 200, 256, false, 1), Some(512));
    }

    #[test]
    fn a_grown_table_shrinks_back_to_the_inline_array() {
        // Almost everything removed from a big table: back to 16 slots, so
        // `hash_may_resize` moves the items home to `ht_smallarray`.
        assert_eq!(resize_decision(1000, 2, 2048, false, 0), Some(16));
    }

    #[test]
    fn a_comfortable_table_is_left_alone() {
        // Under two thirds filled and over a fifth live: no work to do.
        assert_eq!(resize_decision(100, 100, 256, false, 0), None);
        // Same load, but the array is the inline one and `filled` has passed
        // HT_INIT_SIZE - 1, so the small-array early-out no longer applies.
        assert_eq!(resize_decision(15, 15, 16, true, 0), Some(64));
    }

    #[test]
    fn a_big_table_doubles_rather_than_quadruples() {
        // Up to 1000 live items the new size covers 4x; past it, 2x. Both
        // round up to a power of two.
        assert_eq!(resize_decision(1000, 1000, 1024, false, 0), Some(4096));
        assert_eq!(resize_decision(1001, 1001, 1024, false, 0), Some(2048));
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
}
