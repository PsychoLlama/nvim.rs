//! Open-addressing hash table (`hashtab_T`): safe core + C-ABI shims.
//!
//! The struct layouts are frozen: callers iterate `ht_array` directly and
//! stash pointers to items, so both the layout and the probe sequence (which
//! decides where items land, and therefore iteration order) must match the C
//! implementation exactly. Keys are borrowed C strings owned by the callers;
//! the table compares them and frees them only on the caller's behalf
//! (`hash_clear_all`). Allocation stays on the `xmalloc` family.

use core::ffi::{c_char, c_int, c_uint, c_void, CStr};
use core::ptr;
use core::slice;

use crate::src::nvim::memory::{xcalloc, xfree};
use crate::src::nvim::message::siemsg;
use crate::src::nvim::os::libc::gettext;

pub type hash_T = usize;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut c_char,
}

pub const HT_INIT_SIZE: usize = 16;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashtab_T {
    pub ht_mask: hash_T,
    pub ht_used: usize,
    pub ht_filled: usize,
    pub ht_changed: c_int,
    pub ht_locked: c_int,
    pub ht_array: *mut hashitem_T,
    pub ht_smallarray: [hashitem_T; HT_INIT_SIZE],
}

const PERTURB_SHIFT: u32 = 5;
const OK: c_int = 1;
const FAIL: c_int = 0;

const EMPTY_ITEM: hashitem_T = hashitem_T {
    hi_hash: 0,
    hi_key: ptr::null_mut(),
};

/// Sentinel for a removed item: `hi_key` equal to this *address* marks a
/// tombstone. Exported because other modules (and the unit suite, via
/// `_hash_key_removed`) compare against it. Never written through.
pub static hash_removed: c_char = 0;

fn removed_sentinel() -> *mut c_char {
    &hash_removed as *const c_char as *mut c_char
}

impl hashitem_T {
    fn is_empty(&self) -> bool {
        self.hi_key.is_null()
    }

    fn is_removed(&self) -> bool {
        self.hi_key == removed_sentinel()
    }

    /// Holds a live key (neither empty nor a tombstone).
    fn is_kept(&self) -> bool {
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
    rest.iter().fold(first as hash_T, |hash, &b| {
        hash.wrapping_mul(101).wrapping_add(b as hash_T)
    })
}

/// The `hash_hash_len` fold differs deliberately: it consumes exactly `len`
/// bytes without stopping at NUL, and a leading NUL byte seeds the fold with
/// 0 instead of ending it.
fn hash_bytes_len(key: &[u8]) -> hash_T {
    match key.split_first() {
        Some((&first, rest)) => rest.iter().fold(first as hash_T, |hash, &b| {
            hash.wrapping_mul(101).wrapping_add(b as hash_T)
        }),
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
        assert!(newsize != 0, "hash table size overflow");
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

#[no_mangle]
pub unsafe extern "C" fn hash_init(ht: *mut hashtab_T) {
    *ht = hashtab_T {
        ht_mask: (HT_INIT_SIZE - 1) as hash_T,
        ht_used: 0,
        ht_filled: 0,
        ht_changed: 0,
        ht_locked: 0,
        ht_array: ptr::null_mut(),
        ht_smallarray: [EMPTY_ITEM; HT_INIT_SIZE],
    };
    (*ht).ht_array = (&raw mut (*ht).ht_smallarray) as *mut hashitem_T;
}

#[no_mangle]
pub unsafe extern "C" fn hash_clear(ht: *mut hashtab_T) {
    if (*ht).ht_array != (&raw mut (*ht).ht_smallarray) as *mut hashitem_T {
        xfree((*ht).ht_array as *mut c_void);
    }
}

/// Free the table *and* every key, where each key pointer was offset by
/// `off` bytes into its allocation (keys living inside larger structs).
#[no_mangle]
pub unsafe extern "C" fn hash_clear_all(ht: *mut hashtab_T, off: c_uint) {
    let mut todo = (*ht).ht_used;
    // Error paths free zeroed, never-initialized tables whose ht_array is
    // still null; like the C loop, don't touch the array unless a live
    // item needs freeing.
    if todo > 0 {
        let items = slice::from_raw_parts((*ht).ht_array, (*ht).ht_mask as usize + 1);
        for hi in items {
            if todo == 0 {
                break;
            }
            if hi.is_kept() {
                xfree(hi.hi_key.sub(off as usize) as *mut c_void);
                todo -= 1;
            }
        }
    }
    hash_clear(ht);
}

#[no_mangle]
pub unsafe extern "C" fn hash_find(ht: *const hashtab_T, key: *const c_char) -> *mut hashitem_T {
    hash_lookup(
        ht,
        key,
        CStr::from_ptr(key).to_bytes().len(),
        hash_hash(key),
    )
}

#[no_mangle]
pub unsafe extern "C" fn hash_find_len(
    ht: *const hashtab_T,
    key: *const c_char,
    len: usize,
) -> *mut hashitem_T {
    hash_lookup(ht, key, len, hash_hash_len(key, len))
}

/// Find `key` (of `key_len` bytes, hashing to `hash`): returns the item
/// holding it, or — for an absent key — the slot where it belongs (a
/// tombstone if the walk crossed one, else the empty slot that ended it).
#[no_mangle]
pub unsafe extern "C" fn hash_lookup(
    ht: *const hashtab_T,
    key: *const c_char,
    key_len: usize,
    hash: hash_T,
) -> *mut hashitem_T {
    let wanted = slice::from_raw_parts(key as *const u8, key_len);
    let mut freeitem: *mut hashitem_T = ptr::null_mut();
    for idx in Probe::new(hash, (*ht).ht_mask) {
        let hi = (*ht).ht_array.add(idx);
        if (*hi).is_empty() {
            return if freeitem.is_null() { hi } else { freeitem };
        }
        if (*hi).is_removed() {
            if freeitem.is_null() {
                freeitem = hi;
            }
        } else if (*hi).hi_hash == hash && CStr::from_ptr((*hi).hi_key).to_bytes() == wanted {
            return hi;
        }
    }
    unreachable!("probe sequence always finds an empty slot");
}

pub extern "C" fn hash_debug_results() {}

#[no_mangle]
pub unsafe extern "C" fn hash_add(ht: *mut hashtab_T, key: *mut c_char) -> c_int {
    let hash = hash_hash(key);
    let hi = hash_lookup(ht, key, CStr::from_ptr(key).to_bytes().len(), hash);
    if (*hi).is_kept() {
        siemsg(
            gettext(
                b"E685: Internal error: hash_add(): duplicate key \"%s\"\0".as_ptr()
                    as *const c_char,
            ),
            key,
        );
        return FAIL;
    }
    hash_add_item(ht, hi, key, hash);
    OK
}

/// Add `key` at `hi`, which the caller obtained from `hash_lookup` on a
/// missing key (so it is empty or a tombstone).
#[no_mangle]
pub unsafe extern "C" fn hash_add_item(
    ht: *mut hashtab_T,
    hi: *mut hashitem_T,
    key: *mut c_char,
    hash: hash_T,
) {
    (*ht).ht_used = (*ht).ht_used.wrapping_add(1);
    (*ht).ht_changed += 1;
    if (*hi).is_empty() {
        (*ht).ht_filled = (*ht).ht_filled.wrapping_add(1);
    }
    (*hi).hi_key = key;
    (*hi).hi_hash = hash;
    hash_may_resize(ht, 0);
}

/// Remove the item at `hi` (leaving a tombstone). The key itself belongs to
/// the caller.
#[no_mangle]
pub unsafe extern "C" fn hash_remove(ht: *mut hashtab_T, hi: *mut hashitem_T) {
    (*ht).ht_used = (*ht).ht_used.wrapping_sub(1);
    (*ht).ht_changed += 1;
    (*hi).hi_key = removed_sentinel();
    hash_may_resize(ht, 0);
}

/// Lock out resizing while a caller iterates `ht_array` or holds item
/// pointers across mutations.
#[no_mangle]
pub unsafe extern "C" fn hash_lock(ht: *mut hashtab_T) {
    (*ht).ht_locked += 1;
}

#[no_mangle]
pub unsafe extern "C" fn hash_unlock(ht: *mut hashtab_T) {
    (*ht).ht_locked -= 1;
    hash_may_resize(ht, 0);
}

/// Grow, shrink, or compact (drop tombstones from) the array when the load
/// factors say so; `minitems` forces room for that many items up front.
unsafe fn hash_may_resize(ht: *mut hashtab_T, minitems: usize) {
    if (*ht).ht_locked > 0 {
        return;
    }
    let smallarray = (&raw mut (*ht).ht_smallarray) as *mut hashitem_T;
    let oldsize = (*ht).ht_mask as usize + 1;
    let newsize = match resize_decision(
        (*ht).ht_filled,
        (*ht).ht_used,
        oldsize,
        (*ht).ht_array == smallarray,
        minitems,
    ) {
        Some(newsize) => newsize,
        None => return,
    };

    let newarray_is_small = newsize == HT_INIT_SIZE;
    let keep_smallarray = newarray_is_small && (*ht).ht_array == smallarray;
    let mut temparray = [EMPTY_ITEM; HT_INIT_SIZE];
    let oldarray = if keep_smallarray {
        temparray = (*ht).ht_smallarray;
        temparray.as_mut_ptr()
    } else {
        (*ht).ht_array
    };
    let newarray = if newarray_is_small {
        (*ht).ht_smallarray = [EMPTY_ITEM; HT_INIT_SIZE];
        smallarray
    } else {
        xcalloc(newsize, core::mem::size_of::<hashitem_T>()) as *mut hashitem_T
    };

    rehash_into(
        slice::from_raw_parts(oldarray, oldsize),
        slice::from_raw_parts_mut(newarray, newsize),
        (*ht).ht_used,
    );

    if (*ht).ht_array != smallarray {
        xfree((*ht).ht_array as *mut c_void);
    }
    (*ht).ht_array = newarray;
    (*ht).ht_mask = (newsize - 1) as hash_T;
    (*ht).ht_filled = (*ht).ht_used;
    (*ht).ht_changed += 1;
}

#[no_mangle]
pub unsafe extern "C" fn hash_hash(key: *const c_char) -> hash_T {
    hash_bytes(CStr::from_ptr(key).to_bytes())
}

pub unsafe extern "C" fn hash_hash_len(key: *const c_char, len: usize) -> hash_T {
    hash_bytes_len(slice::from_raw_parts(key as *const u8, len))
}

/// The unit suite reads the sentinel address through this accessor.
#[no_mangle]
pub extern "C" fn _hash_key_removed() -> *const c_char {
    removed_sentinel()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The transpiled C probe loop, kept as the reference the iterator must
    /// match step for step.
    fn c_probe_reference(hash: hash_T, mask: hash_T, steps: usize) -> Vec<usize> {
        let mut out = vec![(hash & mask) as usize];
        let mut idx = hash & mask;
        let mut perturb = hash;
        for _ in 1..steps {
            idx = idx.wrapping_mul(5).wrapping_add(perturb).wrapping_add(1);
            out.push((idx & mask) as usize);
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
    }

    #[test]
    fn rehash_moves_every_kept_item() {
        let sentinel = removed_sentinel();
        let key = 0x1000 as *mut c_char; // dangling but never dereferenced
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
