#![deny(unsafe_op_in_unsafe_fn)]

//! A borrowed view of klib's `kvec_withinit_t`.
//!
//! The C macros spell the same four fields out at every use site, and c2rust
//! copied every expansion: `items` starts out aliasing the struct's inline
//! `init_array` and moves to the heap on the first growth, so a heap buffer
//! must be freed and an inline one must not. `InitVec` is the one place that
//! invariant lives. The containing structs stay `repr(C)` — they are embedded
//! by value all over the tree — so this borrows their fields rather than
//! owning anything.

use core::ffi::c_void;
use core::{mem, ptr, slice};

use crate::src::nvim::memory::{xfree, xmalloc, xrealloc};
use crate::src::nvim::os::libc::memcpy;
use crate::src::nvim::types::size_t;

pub struct InitVec<'a, T> {
    size: &'a mut usize,
    capacity: &'a mut usize,
    items: &'a mut *mut T,
    init: *mut T,
    init_capacity: usize,
}

impl<'a, T: Copy> InitVec<'a, T> {
    /// Borrow the four fields of one `kvec_withinit_t`. They are distinct
    /// fields of the same struct, so the borrows do not conflict.
    pub fn new(
        size: &'a mut usize,
        capacity: &'a mut usize,
        items: &'a mut *mut T,
        init_array: &'a mut [T],
    ) -> Self {
        InitVec {
            init_capacity: init_array.len(),
            init: init_array.as_mut_ptr(),
            size,
            capacity,
            items,
        }
    }

    /// `kvi_init`: empty the collection and point it at its inline array.
    ///
    /// Only valid where the containing struct is going to stay put — `items`
    /// ends up holding the struct's own address.
    pub fn init(&mut self) {
        *self.size = 0;
        *self.capacity = self.init_capacity;
        *self.items = self.init;
    }

    /// Still using the inline array rather than a heap buffer.
    pub fn is_inline(&self) -> bool {
        *self.items == self.init
    }

    /// The buffer to read and write through.
    ///
    /// While the collection is inline, `items` names the struct's own
    /// `init_array` — and the pointer stored there was derived from whatever
    /// borrow ran `kvi_init`, which this view's borrow of `init_array` has
    /// since invalidated. So re-derive it: same address, live provenance.
    /// Once the collection is on the heap, `items` carries the allocation's
    /// own provenance and is used as it stands.
    fn base(&self) -> *mut T {
        if self.is_inline() {
            self.init
        } else {
            *self.items
        }
    }

    pub fn len(&self) -> usize {
        *self.size
    }

    pub fn is_empty(&self) -> bool {
        *self.size == 0
    }

    pub fn as_slice(&self) -> &[T] {
        // `items` is null on a collection that was never initialized, and
        // then `size` is zero too — but `from_raw_parts` rejects a null base
        // even for an empty slice, so answer that case without the pointer.
        if *self.size == 0 {
            return &[];
        }
        unsafe { slice::from_raw_parts(self.base(), *self.size) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if *self.size == 0 {
            return &mut [];
        }
        unsafe { slice::from_raw_parts_mut(self.base(), *self.size) }
    }

    /// The last element. Panics where `kv_last` on an empty vector read one
    /// slot before the buffer.
    pub fn last(&self) -> T {
        self.as_slice()[*self.size - 1]
    }

    pub fn push(&mut self, value: T) {
        let capacity = *self.capacity;
        unsafe {
            if *self.size == capacity {
                self.grow(capacity);
            }
            self.base().add(*self.size).write(value);
        }
        *self.size += 1;
    }

    /// Move to a buffer of at least twice the capacity, or to the inline
    /// array if that is already big enough (which only a collection that was
    /// never initialized can reach, since `kvi_init` sets the capacity to the
    /// inline array's length).
    ///
    /// # Safety
    /// `items` must be the inline array or a live allocation of `capacity`
    /// elements.
    unsafe fn grow(&mut self, capacity: usize) {
        unsafe {
            let filled = *self.size;
            let grown = (capacity * 2).max(self.init_capacity);
            let bytes = grown * mem::size_of::<T>();
            let base = self.base();
            *self.items = match (grown == self.init_capacity, self.is_inline()) {
                (true, true) => self.init,
                (true, false) => {
                    ptr::copy_nonoverlapping(base, self.init, filled);
                    xfree(base as *mut c_void);
                    self.init
                }
                (false, true) => {
                    let heap = xmalloc(bytes) as *mut T;
                    ptr::copy_nonoverlapping(base, heap, filled);
                    heap
                }
                (false, false) => xrealloc(base as *mut c_void, bytes) as *mut T,
            };
            *self.capacity = grown;
        }
    }

    /// Hand back the heap buffer, if the collection ever left the inline
    /// array, leaving the collection empty and inline. The caller frees it —
    /// callers that are tearing several collections down want one `unsafe`
    /// block, not one per collection.
    #[must_use = "the returned buffer must be freed"]
    pub fn take_heap(&mut self) -> *mut c_void {
        let heap = if self.is_inline() || self.items.is_null() {
            ptr::null_mut()
        } else {
            *self.items as *mut c_void
        };
        *self.items = self.init;
        *self.capacity = self.init_capacity;
        *self.size = 0;
        heap
    }
}

/// Copy `size` bytes from `src` to `dest`, then free `src`. klib's kvec
/// spells this inline in every `kv_concat`-shaped macro.
pub unsafe fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    unsafe {
        memcpy(dest, src, size);
        xfree(src);
    }
    dest
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stand-in for one of the tree's `kvec_withinit_t` structs.
    struct Vec4 {
        size: usize,
        capacity: usize,
        items: *mut usize,
        init_array: [usize; 4],
    }

    impl Vec4 {
        /// All-zero, as a `kvec_withinit_t` field of a freshly allocated
        /// struct is.
        fn zeroed() -> Self {
            Vec4 {
                size: 0,
                capacity: 0,
                items: ptr::null_mut(),
                init_array: [0; 4],
            }
        }

        /// `kvi_init`. It points `items` at the struct's own inline array, so
        /// it must run where the struct is going to stay — moving an
        /// initialized one leaves `items` dangling.
        fn init(&mut self) {
            self.capacity = self.init_array.len();
            self.items = self.init_array.as_mut_ptr();
        }

        fn view(&mut self) -> InitVec<'_, usize> {
            InitVec::new(
                &mut self.size,
                &mut self.capacity,
                &mut self.items,
                &mut self.init_array,
            )
        }
    }

    #[test]
    fn grows_off_the_inline_array_and_keeps_everything() {
        let mut v = Vec4::zeroed();
        v.init();
        assert!(v.view().is_inline());
        for i in 0..10 {
            v.view().push(i);
            assert_eq!(v.view().last(), i);
        }
        assert!(!v.view().is_inline());
        assert_eq!(v.capacity, 16);
        assert_eq!(v.view().as_slice(), &(0..10).collect::<Vec<usize>>()[..]);

        let heap = v.view().take_heap();
        assert!(!heap.is_null());
        unsafe { xfree(heap) };
        assert!(v.view().is_inline());
        assert!(v.view().is_empty());
    }

    #[test]
    fn stays_inline_while_it_fits() {
        let mut v = Vec4::zeroed();
        v.init();
        for i in 0..4 {
            v.view().push(i);
        }
        assert!(v.view().is_inline());
        assert_eq!(v.view().as_slice(), &[0, 1, 2, 3]);
        assert!(v.view().take_heap().is_null());
    }

    /// An all-zero collection — one that was never `kvi_init`ed — grows into
    /// its own inline array rather than allocating, as the C did.
    #[test]
    fn uninitialized_collection_adopts_the_inline_array() {
        let mut v = Vec4::zeroed();
        assert!(v.view().as_slice().is_empty());
        v.view().push(7);
        assert!(v.view().is_inline());
        assert_eq!(v.view().as_slice(), &[7]);
    }
}
