//! Hash table implementation
//!
//! A hash table with string keys, compatible with nvim's hashtab_T.
//! Uses the same hashing algorithm and collision resolution as the C version.
//!
//! Note: The hash functions (rs_hash_hash, rs_hash_hash_len) are exported from
//! nvim_memutil, not from this module, to avoid duplicate symbol issues.

use std::ffi::{c_char, c_int};
use std::ptr;

use nvim_memory::{xcalloc, xfree};
use nvim_memutil::{rs_hash_hash, rs_hash_hash_len};

/// Initial size for a hashtable (must be a power of 2).
pub const HT_INIT_SIZE: usize = 16;

/// Shift value for perturbation in probing.
const PERTURB_SHIFT: u32 = 5;

/// Hash type (matches C hash_T).
pub type HashT = usize;

/// Hash table item.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HashItem {
    /// Cached hash number for the key
    pub hi_hash: HashT,
    /// Item key (null = never used, REMOVED = removed, other = in use)
    pub hi_key: *mut c_char,
}

impl Default for HashItem {
    fn default() -> Self {
        Self {
            hi_hash: 0,
            hi_key: ptr::null_mut(),
        }
    }
}

/// Hash table structure - matches C hashtab_T layout.
#[repr(C)]
pub struct HashTab {
    /// Mask used for hash value (nr of items in array is ht_mask + 1)
    pub ht_mask: HashT,
    /// Number of items used
    pub ht_used: usize,
    /// Number of items used or removed
    pub ht_filled: usize,
    /// Incremented when adding/removing items
    pub ht_changed: c_int,
    /// Counter for hash_lock()
    pub ht_locked: c_int,
    /// Points to the array (allocated or ht_smallarray)
    pub ht_array: *mut HashItem,
    /// Initial small array (inline storage)
    pub ht_smallarray: [HashItem; HT_INIT_SIZE],
}

// Static marker for removed items
static mut HASH_REMOVED_MARKER: c_char = 0;

/// Get the address used to mark removed items.
#[no_mangle]
pub extern "C" fn rs_hash_key_removed() -> *mut c_char {
    ptr::addr_of_mut!(HASH_REMOVED_MARKER)
}

/// Check if a hash item is empty (never used or removed).
#[inline]
fn hashitem_empty(hi: &HashItem) -> bool {
    hi.hi_key.is_null() || hi.hi_key == rs_hash_key_removed()
}

/// Initialize an empty hash table.
///
/// # Safety
///
/// `ht` must be a valid pointer to a `HashTab` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_init(ht: *mut HashTab) {
    if ht.is_null() {
        return;
    }

    let ht_ref = unsafe { &mut *ht };

    // Zero everything
    ht_ref.ht_mask = 0;
    ht_ref.ht_used = 0;
    ht_ref.ht_filled = 0;
    ht_ref.ht_changed = 0;
    ht_ref.ht_locked = 0;

    // Zero the small array
    for item in &mut ht_ref.ht_smallarray {
        item.hi_hash = 0;
        item.hi_key = ptr::null_mut();
    }

    // Use the small array initially
    ht_ref.ht_array = ht_ref.ht_smallarray.as_mut_ptr();
    ht_ref.ht_mask = HT_INIT_SIZE - 1;
}

/// Free the array of a hash table without freeing contained values.
///
/// # Safety
///
/// `ht` must be a valid pointer to a `HashTab` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_clear(ht: *mut HashTab) {
    if ht.is_null() {
        return;
    }

    let ht_ref = unsafe { &mut *ht };
    let smallarray_ptr = ht_ref.ht_smallarray.as_mut_ptr();

    if ht_ref.ht_array != smallarray_ptr {
        unsafe { xfree(ht_ref.ht_array.cast()) };
    }
}

// Note: rs_hash_hash and rs_hash_hash_len are imported from nvim_memutil
// to avoid duplicate FFI symbol definitions.

/// Look up a key in the hash table.
///
/// Returns a pointer to the hash item. If the key is not found, returns
/// a pointer to the empty slot where it would be inserted.
///
/// # Safety
///
/// `ht` must be a valid pointer to a `HashTab` structure.
/// `key` must be a valid pointer to at least `key_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_lookup(
    ht: *const HashTab,
    key: *const c_char,
    key_len: usize,
    hash: HashT,
) -> *mut HashItem {
    if ht.is_null() || key.is_null() {
        return ptr::null_mut();
    }

    let ht_ref = unsafe { &*ht };
    let removed_marker = rs_hash_key_removed();

    let mut idx = hash & ht_ref.ht_mask;
    let mut hi = unsafe { ht_ref.ht_array.add(idx) };

    // Quick checks for common cases
    if unsafe { (*hi).hi_key.is_null() } {
        return hi;
    }

    let mut freeitem: *mut HashItem = ptr::null_mut();

    if unsafe { (*hi).hi_key == removed_marker } {
        freeitem = hi;
    } else if unsafe { (*hi).hi_hash == hash } && key_matches(hi, key, key_len) {
        return hi;
    }

    // Probe through the table
    let mut perturb = hash;
    loop {
        perturb >>= PERTURB_SHIFT;
        idx = idx.wrapping_mul(5).wrapping_add(perturb).wrapping_add(1);
        hi = unsafe { ht_ref.ht_array.add(idx & ht_ref.ht_mask) };

        if unsafe { (*hi).hi_key.is_null() } {
            return if freeitem.is_null() { hi } else { freeitem };
        }

        if unsafe { (*hi).hi_hash == hash && (*hi).hi_key != removed_marker }
            && key_matches(hi, key, key_len)
        {
            return hi;
        }

        if unsafe { (*hi).hi_key == removed_marker } && freeitem.is_null() {
            freeitem = hi;
        }
    }
}

/// Check if a hash item's key matches the given key.
fn key_matches(hi: *const HashItem, key: *const c_char, key_len: usize) -> bool {
    let hi_key = unsafe { (*hi).hi_key };
    if hi_key.is_null() {
        return false;
    }

    // Compare key_len bytes
    for i in 0..key_len {
        let a = unsafe { *hi_key.add(i) };
        let b = unsafe { *key.add(i) };
        if a != b {
            return false;
        }
    }

    // Check that hi_key is null-terminated at key_len
    unsafe { *hi_key.add(key_len) == 0 }
}

/// Find a key in the hash table.
///
/// # Safety
///
/// `ht` must be a valid pointer to a `HashTab` structure.
/// `key` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_find(ht: *const HashTab, key: *const c_char) -> *mut HashItem {
    if ht.is_null() || key.is_null() {
        return ptr::null_mut();
    }

    let len = unsafe { libc::strlen(key) };
    let hash = unsafe { rs_hash_hash(key) };
    unsafe { rs_hash_lookup(ht, key, len, hash) }
}

/// Find a key with known length in the hash table.
///
/// # Safety
///
/// `ht` must be a valid pointer to a `HashTab` structure.
/// `key` must be a valid pointer to at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_find_len(
    ht: *const HashTab,
    key: *const c_char,
    len: usize,
) -> *mut HashItem {
    if ht.is_null() || key.is_null() {
        return ptr::null_mut();
    }

    let hash = unsafe { rs_hash_hash_len(key, len) };
    unsafe { rs_hash_lookup(ht, key, len, hash) }
}

/// Add an item to the hash table.
///
/// The caller must have already called `rs_hash_lookup` to get `hi`.
///
/// # Safety
///
/// `ht` must be a valid pointer to a `HashTab` structure.
/// `hi` must be a pointer to an empty slot in the hash table.
/// `key` must be a valid null-terminated C string that will remain valid.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_add_item(
    ht: *mut HashTab,
    hi: *mut HashItem,
    key: *mut c_char,
    hash: HashT,
) {
    if ht.is_null() || hi.is_null() {
        return;
    }

    let ht_ref = unsafe { &mut *ht };
    let hi_ref = unsafe { &mut *hi };

    ht_ref.ht_used += 1;
    ht_ref.ht_changed += 1;

    if hi_ref.hi_key.is_null() {
        ht_ref.ht_filled += 1;
    }

    hi_ref.hi_key = key;
    hi_ref.hi_hash = hash;

    // May need to resize
    unsafe { hash_may_resize(ht, 0) };
}

/// Add a key to the hash table.
///
/// Returns 1 (OK) on success, 0 (FAIL) if key already exists.
///
/// # Safety
///
/// `ht` must be a valid pointer to a `HashTab` structure.
/// `key` must be a valid null-terminated C string that will remain valid.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_add(ht: *mut HashTab, key: *mut c_char) -> c_int {
    if ht.is_null() || key.is_null() {
        return 0;
    }

    let hash = unsafe { rs_hash_hash(key) };
    let len = unsafe { libc::strlen(key) };
    let hi = unsafe { rs_hash_lookup(ht, key, len, hash) };

    if hi.is_null() || !hashitem_empty(unsafe { &*hi }) {
        return 0; // FAIL - key already exists
    }

    unsafe { rs_hash_add_item(ht, hi, key, hash) };
    1 // OK
}

/// Remove an item from the hash table.
///
/// The caller must free the key if necessary.
///
/// # Safety
///
/// `ht` must be a valid pointer to a `HashTab` structure.
/// `hi` must be a pointer to an item in the hash table.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_remove(ht: *mut HashTab, hi: *mut HashItem) {
    if ht.is_null() || hi.is_null() {
        return;
    }

    let ht_ref = unsafe { &mut *ht };
    let hi_ref = unsafe { &mut *hi };

    ht_ref.ht_used -= 1;
    ht_ref.ht_changed += 1;
    hi_ref.hi_key = rs_hash_key_removed();

    unsafe { hash_may_resize(ht, 0) };
}

/// Lock a hash table to prevent resizing.
///
/// # Safety
///
/// `ht` must be a valid pointer to a `HashTab` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_lock(ht: *mut HashTab) {
    if !ht.is_null() {
        unsafe { (*ht).ht_locked += 1 };
    }
}

/// Unlock a hash table.
///
/// # Safety
///
/// `ht` must be a valid pointer to a `HashTab` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_hash_unlock(ht: *mut HashTab) {
    if ht.is_null() {
        return;
    }

    unsafe {
        (*ht).ht_locked -= 1;
        hash_may_resize(ht, 0);
    }
}

/// Resize the hash table if needed.
unsafe fn hash_may_resize(ht: *mut HashTab, minitems: usize) {
    let ht_ref = unsafe { &mut *ht };

    // Don't resize a locked table
    if ht_ref.ht_locked > 0 {
        return;
    }

    let oldsize = ht_ref.ht_mask + 1;
    let smallarray_ptr = ht_ref.ht_smallarray.as_mut_ptr();

    let minsize = if minitems == 0 {
        // Check if we need to resize at all
        if ht_ref.ht_filled < HT_INIT_SIZE - 1 && ht_ref.ht_array == smallarray_ptr {
            return;
        }

        // Grow if more than 2/3 full, shrink if less than 1/5 full
        if ht_ref.ht_filled * 3 < oldsize * 2 && ht_ref.ht_used > oldsize / 5 {
            return;
        }

        if ht_ref.ht_used > 1000 {
            ht_ref.ht_used * 2
        } else {
            ht_ref.ht_used * 4
        }
    } else {
        let items = minitems.max(ht_ref.ht_used);
        (items * 3 + 1) / 2
    };

    // Find the smallest power of 2 >= minsize
    let mut newsize = HT_INIT_SIZE;
    while newsize < minsize {
        newsize <<= 1;
    }

    let newarray_is_small = newsize == HT_INIT_SIZE;

    // Check if resize is actually needed
    if !newarray_is_small && newsize == oldsize && ht_ref.ht_filled * 3 < oldsize * 2 {
        return;
    }

    let keep_smallarray = newarray_is_small && ht_ref.ht_array == smallarray_ptr;

    // Make a copy of the old array if we're keeping the small array
    let mut temparray = [HashItem::default(); HT_INIT_SIZE];
    let oldarray = if keep_smallarray {
        for (i, item) in ht_ref.ht_smallarray.iter().enumerate() {
            temparray[i].hi_hash = item.hi_hash;
            temparray[i].hi_key = item.hi_key;
        }
        temparray.as_mut_ptr()
    } else {
        ht_ref.ht_array
    };

    // Create the new array
    let newarray = if newarray_is_small {
        // Zero the small array
        for item in &mut ht_ref.ht_smallarray {
            item.hi_hash = 0;
            item.hi_key = ptr::null_mut();
        }
        ht_ref.ht_smallarray.as_mut_ptr()
    } else {
        unsafe { xcalloc(newsize, std::mem::size_of::<HashItem>()) as *mut HashItem }
    };

    // Move items from old to new array
    let newmask = newsize - 1;
    let mut todo = ht_ref.ht_used;

    let mut olditem = oldarray;
    while todo > 0 {
        let old_hi = unsafe { &*olditem };

        if !hashitem_empty(old_hi) {
            // Find a spot in the new array
            let mut newi = old_hi.hi_hash & newmask;
            let mut newitem = unsafe { newarray.add(newi) };

            if !unsafe { (*newitem).hi_key.is_null() } {
                let mut perturb = old_hi.hi_hash;
                loop {
                    perturb >>= PERTURB_SHIFT;
                    newi = newi.wrapping_mul(5).wrapping_add(perturb).wrapping_add(1);
                    newitem = unsafe { newarray.add(newi & newmask) };
                    if unsafe { (*newitem).hi_key.is_null() } {
                        break;
                    }
                }
            }

            unsafe {
                (*newitem).hi_hash = old_hi.hi_hash;
                (*newitem).hi_key = old_hi.hi_key;
            }
            todo -= 1;
        }

        olditem = unsafe { olditem.add(1) };
    }

    // Free the old array if it was allocated
    if ht_ref.ht_array != smallarray_ptr {
        unsafe { xfree(ht_ref.ht_array.cast()) };
    }

    ht_ref.ht_array = newarray;
    ht_ref.ht_mask = newmask;
    ht_ref.ht_filled = ht_ref.ht_used;
    ht_ref.ht_changed += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_hash_init() {
        let mut ht = HashTab {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ptr::null_mut(),
            ht_smallarray: [HashItem::default(); HT_INIT_SIZE],
        };

        unsafe { rs_hash_init(&mut ht) };

        assert_eq!(ht.ht_mask, HT_INIT_SIZE - 1);
        assert_eq!(ht.ht_used, 0);
        assert_eq!(ht.ht_filled, 0);
        assert!(!ht.ht_array.is_null());
    }

    #[test]
    fn test_hash_hash() {
        let key1 = CString::new("hello").unwrap();
        let key2 = CString::new("world").unwrap();
        let key3 = CString::new("hello").unwrap();

        let hash1 = unsafe { rs_hash_hash(key1.as_ptr()) };
        let hash2 = unsafe { rs_hash_hash(key2.as_ptr()) };
        let hash3 = unsafe { rs_hash_hash(key3.as_ptr()) };

        // Same string should have same hash
        assert_eq!(hash1, hash3);

        // Different strings should (usually) have different hashes
        assert_ne!(hash1, hash2);

        // Empty string should hash to 0
        let empty = CString::new("").unwrap();
        assert_eq!(unsafe { rs_hash_hash(empty.as_ptr()) }, 0);
    }

    #[test]
    fn test_hash_hash_len() {
        let key = CString::new("hello world").unwrap();

        let hash_full = unsafe { rs_hash_hash_len(key.as_ptr(), 11) };
        let hash_hello = unsafe { rs_hash_hash_len(key.as_ptr(), 5) };

        let hello_only = CString::new("hello").unwrap();
        let hello_hash = unsafe { rs_hash_hash(hello_only.as_ptr()) };

        // Hash of first 5 chars should match hash of "hello"
        assert_eq!(hash_hello, hello_hash);

        // Full hash should be different
        assert_ne!(hash_full, hash_hello);
    }
}
