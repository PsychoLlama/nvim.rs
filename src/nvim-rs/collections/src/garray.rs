//! Growing array implementation
//!
//! A growing array is a dynamic array that automatically grows when more
//! space is needed. It uses nvim's memory allocator for C compatibility.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use nvim_memory::{xfree, xrealloc};

/// Growing array structure - matches C garray_T layout exactly.
#[repr(C)]
pub struct GArray {
    /// Current number of items used
    pub ga_len: c_int,
    /// Maximum number of items possible
    pub ga_maxlen: c_int,
    /// Size of each item in bytes
    pub ga_itemsize: c_int,
    /// Number of items to grow by each time
    pub ga_growsize: c_int,
    /// Pointer to the first item
    pub ga_data: *mut c_void,
}

impl Default for GArray {
    fn default() -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 1,
            ga_data: ptr::null_mut(),
        }
    }
}

/// Initialize a growing array.
///
/// # Safety
///
/// `gap` must be a valid pointer to a `GArray` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int) {
    if gap.is_null() {
        return;
    }

    unsafe {
        (*gap).ga_data = ptr::null_mut();
        (*gap).ga_maxlen = 0;
        (*gap).ga_len = 0;
        (*gap).ga_itemsize = itemsize;
        rs_ga_set_growsize(gap, growsize);
    }
}

/// Set the growth size for a growing array.
///
/// Ensures the growth size is at least 1.
///
/// # Safety
///
/// `gap` must be a valid pointer to a `GArray` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_ga_set_growsize(gap: *mut GArray, growsize: c_int) {
    if gap.is_null() {
        return;
    }

    unsafe {
        (*gap).ga_growsize = if growsize < 1 { 1 } else { growsize };
    }
}

/// Clear a growing array and free its data.
///
/// # Safety
///
/// `gap` must be a valid pointer to a `GArray` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_ga_clear(gap: *mut GArray) {
    if gap.is_null() {
        return;
    }

    unsafe {
        xfree((*gap).ga_data);
        (*gap).ga_data = ptr::null_mut();
        (*gap).ga_maxlen = 0;
        (*gap).ga_len = 0;
    }
}

/// Check if a growing array is empty.
///
/// # Safety
///
/// `gap` must be a valid pointer to a `GArray` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_ga_empty(gap: *const GArray) -> c_int {
    if gap.is_null() {
        return 1;
    }

    c_int::from(unsafe { (*gap).ga_len <= 0 })
}

/// Make room for at least `n` more items in the growing array.
///
/// Uses a growth strategy that balances memory usage and copy operations:
/// - Grows by at least `ga_growsize` items
/// - For large arrays, grows by 50% to reduce frequent reallocations
///
/// # Safety
///
/// `gap` must be a valid pointer to a `GArray` structure.
#[no_mangle]
pub unsafe extern "C" fn rs_ga_grow(gap: *mut GArray, n: c_int) {
    if gap.is_null() || n <= 0 {
        return;
    }

    let gap_ref = unsafe { &mut *gap };

    // Check if we already have enough space
    if gap_ref.ga_maxlen - gap_ref.ga_len >= n {
        return;
    }

    // Ensure growsize is at least 1
    let growsize = if gap_ref.ga_growsize < 1 {
        1
    } else {
        gap_ref.ga_growsize
    };

    // Calculate how much to grow
    // - At least n items
    // - At least growsize items
    // - At least 50% of current length (for large arrays)
    let mut grow_by = n;
    if growsize > grow_by {
        grow_by = growsize;
    }
    let half_len = gap_ref.ga_len / 2;
    if half_len > grow_by {
        grow_by = half_len;
    }

    let new_maxlen = gap_ref.ga_len + grow_by;
    let new_size = (gap_ref.ga_itemsize as usize) * (new_maxlen as usize);
    let old_size = (gap_ref.ga_itemsize as usize) * (gap_ref.ga_maxlen as usize);

    // Reallocate
    let new_data = unsafe { xrealloc(gap_ref.ga_data, new_size) };

    // Zero the new memory
    if new_size > old_size {
        unsafe {
            let new_bytes = (new_data as *mut u8).add(old_size);
            ptr::write_bytes(new_bytes, 0, new_size - old_size);
        }
    }

    gap_ref.ga_maxlen = new_maxlen;
    gap_ref.ga_data = new_data;
}

/// Append a single byte to a growing array.
///
/// # Safety
///
/// `gap` must be a valid pointer to a `GArray` structure with `ga_itemsize == 1`.
#[no_mangle]
pub unsafe extern "C" fn rs_ga_append(gap: *mut GArray, c: u8) {
    if gap.is_null() {
        return;
    }

    unsafe {
        rs_ga_grow(gap, 1);
        let data = (*gap).ga_data as *mut u8;
        *data.add((*gap).ga_len as usize) = c;
        (*gap).ga_len += 1;
    }
}

/// Get a pointer to append an item to a growing array.
///
/// Grows the array by 1 and returns a pointer to the new slot.
/// The caller should write the item to this pointer.
///
/// # Safety
///
/// `gap` must be a valid pointer to a `GArray` structure.
/// `item_size` should match `gap->ga_itemsize`.
#[no_mangle]
pub unsafe extern "C" fn rs_ga_append_via_ptr(gap: *mut GArray, item_size: usize) -> *mut c_void {
    if gap.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        rs_ga_grow(gap, 1);
        let offset = item_size * ((*gap).ga_len as usize);
        (*gap).ga_len += 1;
        ((*gap).ga_data as *mut u8).add(offset).cast()
    }
}

/// Concatenate a string to a growing array of characters.
///
/// Does NOT copy the null terminator.
///
/// # Safety
///
/// `gap` must be a valid pointer to a `GArray` structure.
/// `s` must be a valid null-terminated C string, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_ga_concat(gap: *mut GArray, s: *const c_char) {
    if gap.is_null() || s.is_null() {
        return;
    }

    let len = unsafe { libc::strlen(s) };
    unsafe { rs_ga_concat_len(gap, s, len) };
}

/// Concatenate a string with known length to a growing array of characters.
///
/// Does NOT copy the null terminator.
///
/// # Safety
///
/// `gap` must be a valid pointer to a `GArray` structure.
/// `s` must be a valid pointer to at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_ga_concat_len(gap: *mut GArray, s: *const c_char, len: usize) {
    if gap.is_null() || s.is_null() || len == 0 {
        return;
    }

    unsafe {
        rs_ga_grow(gap, len as c_int);
        let data = (*gap).ga_data as *mut c_char;
        ptr::copy_nonoverlapping(s, data.add((*gap).ga_len as usize), len);
        (*gap).ga_len += len as c_int;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ga_init() {
        let mut ga = GArray::default();
        unsafe { rs_ga_init(&mut ga, 4, 10) };

        assert_eq!(ga.ga_len, 0);
        assert_eq!(ga.ga_maxlen, 0);
        assert_eq!(ga.ga_itemsize, 4);
        assert_eq!(ga.ga_growsize, 10);
        assert!(ga.ga_data.is_null());
    }

    #[test]
    fn test_ga_empty() {
        let mut ga = GArray::default();
        unsafe { rs_ga_init(&mut ga, 1, 1) };

        assert_eq!(unsafe { rs_ga_empty(&ga) }, 1);

        ga.ga_len = 1;
        assert_eq!(unsafe { rs_ga_empty(&ga) }, 0);
    }

    #[test]
    fn test_ga_set_growsize() {
        let mut ga = GArray::default();
        unsafe { rs_ga_init(&mut ga, 1, 10) };

        unsafe { rs_ga_set_growsize(&mut ga, 0) };
        assert_eq!(ga.ga_growsize, 1); // Should clamp to 1

        unsafe { rs_ga_set_growsize(&mut ga, 5) };
        assert_eq!(ga.ga_growsize, 5);
    }
}
