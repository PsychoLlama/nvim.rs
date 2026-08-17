//! The process-wide Rust allocator, backed by the same libc `malloc` the
//! `xmalloc` family bottoms out in.
//!
//! With both sides drawing from one allocator, heap ownership can legally
//! cross the C ABI in either direction: a buffer built with `Vec`/`Box`/
//! `CString` may be released with `xfree`, and `xmalloc` memory may be
//! adopted by Rust containers, with no copy-at-the-boundary layer.
//!
//! `std::alloc::System` deliberately isn't used for this. It is documented
//! as possibly wrapping the platform allocator in extra bookkeeping, which
//! would make handing its pointers to `free` (or `malloc`'s to `Box`)
//! undefined. Bottoming out in malloc directly is the whole point.
//!
//! # Boundary
//!
//! The four libc calls below are this file's only raw operations.
//! [`malloc_aligned`] and [`calloc_aligned`] are callable from safe code:
//! a `Layout`'s alignment is a nonzero power of two by construction, and
//! anything above `MALLOC_ALIGN` is therefore also a multiple of the
//! pointer size, which is all `posix_memalign` additionally requires.
//! `realloc` and `free` inherit their preconditions from the `GlobalAlloc`
//! contract, which the trait's caller already has to uphold.

#![deny(unsafe_op_in_unsafe_fn)]

use core::alloc::{GlobalAlloc, Layout};

/// Alignment `malloc` already guarantees. Larger requests go through
/// `posix_memalign`; `free` accepts pointers from either source.
const MALLOC_ALIGN: usize = align_of::<libc::max_align_t>();

struct LibcAllocator;

/// Uninitialized block of `size` bytes at `align`, null on failure.
/// `align` must be a nonzero power of two, as every `Layout`'s is.
fn malloc_aligned(size: usize, align: usize) -> *mut u8 {
    let mut memalign_ptr: *mut libc::c_void = core::ptr::null_mut();
    // SAFETY: neither call has a precondition the `Layout` doesn't already
    // guarantee (see the module docs); both report failure in-band.
    unsafe {
        if align <= MALLOC_ALIGN {
            libc::malloc(size).cast()
        } else if libc::posix_memalign(&mut memalign_ptr, align, size) == 0 {
            memalign_ptr.cast()
        } else {
            core::ptr::null_mut()
        }
    }
}

/// Zeroed block of `size` bytes at `align`, null on failure.
fn calloc_aligned(size: usize, align: usize) -> *mut u8 {
    // SAFETY: `calloc` is as unconditional as `malloc`. On the over-aligned
    // path `posix_memalign` has no zeroing variant, so clear by hand: the
    // block is fresh, writable and exactly `size` bytes long.
    unsafe {
        if align <= MALLOC_ALIGN {
            libc::calloc(1, size).cast()
        } else {
            let ptr = malloc_aligned(size, align);
            if !ptr.is_null() {
                core::ptr::write_bytes(ptr, 0, size);
            }
            ptr
        }
    }
}

// SAFETY: blocks satisfy the requested size and alignment, stay valid until
// released through this same impl, and null reports failure.
unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc_aligned(layout.size(), layout.align())
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        calloc_aligned(layout.size(), layout.align())
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: by the trait contract `ptr` came from this allocator with
        // `layout`, so from `malloc` on the fast path — and on the
        // over-aligned path (where `realloc` would not preserve the
        // alignment, so the move is by hand) it is live, at least
        // `layout.size()` bytes long, distinct from the fresh block, and
        // ours to release.
        unsafe {
            if layout.align() <= MALLOC_ALIGN {
                return libc::realloc(ptr.cast(), new_size).cast();
            }
            let new_ptr = malloc_aligned(new_size, layout.align());
            if !new_ptr.is_null() {
                core::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
                self.dealloc(ptr, layout);
            }
            new_ptr
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // SAFETY: by the trait contract `ptr` came from this allocator, so
        // from `malloc`/`calloc`/`posix_memalign`.
        unsafe { libc::free(ptr.cast()) };
    }
}

#[global_allocator]
static GLOBAL: LibcAllocator = LibcAllocator;
