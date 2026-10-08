#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

//! A borrowed view of klib's `kvec_t`, for the `repr(C)` structs that still
//! embed one by value.
//!
//! Ported from klib's `kvec.h`, Copyright (c) 2008 Attractive Chaos, under
//! the MIT license; the notice is reproduced in licenses/klib-LICENSE.txt.

use core::ffi::c_void;

use crate::memory::{xfree, xrealloc};
use crate::types::size_t;

/// A borrowed view of klib's plain `kvec_t`: the same three fields without
/// the inline array, so `items` is always either null or a heap allocation.
///
/// `kv_push`'s growth step is the whole reason this exists — c2rust expanded
/// it at every use site, a dozen lines apiece, and the shape is identical
/// each time: double the capacity, or start at eight.
pub(crate) struct Kvec<'a, T> {
    size: &'a mut usize,
    capacity: &'a mut usize,
    items: &'a mut *mut T,
}

impl<'a, T> Kvec<'a, T> {
    /// Borrow the three fields of one `kvec_t`. They are distinct fields of
    /// the same struct, so the borrows do not conflict.
    pub(crate) fn new(size: &'a mut usize, capacity: &'a mut usize, items: &'a mut *mut T) -> Self {
        Kvec {
            size,
            capacity,
            items,
        }
    }

    /// `kv_push`.
    ///
    /// # Safety
    /// `items` must be null or a live allocation of `capacity` elements.
    pub(crate) unsafe fn push(&mut self, value: T) {
        unsafe {
            if *self.size == *self.capacity {
                *self.capacity = if *self.capacity != 0 {
                    *self.capacity * 2
                } else {
                    8
                };
                *self.items =
                    xrealloc(*self.items as *mut c_void, *self.capacity * size_of::<T>()) as *mut T;
            }
            self.items.add(*self.size).write(value);
        }
        *self.size += 1;
    }
}

/// Copy `size` bytes from `src` to `dest`, then free `src`. klib's kvec
/// spells this inline in every `kv_concat`-shaped macro.
///
/// # Safety
///
/// `src` must point at `size` readable bytes of an allocation `xmalloc`
/// answered — it is freed here — and `dest` at `size` writable bytes that do
/// not overlap it.
pub(crate) unsafe fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    unsafe {
        dest.cast::<u8>().copy_from_nonoverlapping(src.cast(), size);
        xfree(src);
    }
    dest
}
