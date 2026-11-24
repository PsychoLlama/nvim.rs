//! The process-wide Rust allocator, backed by the same libc `malloc` the
//! `xmalloc` family bottoms out in.
//!
//! With both sides drawing from one allocator, heap ownership can legally
//! cross the C ABI in either direction: a buffer built with `Vec`/`Box`/
//! `CString` may be released with `xfree`, and `xmalloc` memory may be
//! adopted by Rust containers, with no copy-at-the-boundary layer.
//!
//! # Adopting a foreign block
//!
//! A block from `malloc` may be handed to a Rust container that states a
//! **smaller capacity than `malloc` actually gave**, which is what
//! [`XString::from_raw`](crate::memory::XString::from_raw) does: it adopts an
//! `xmalloc` block as a `Vec<u8>` of `strlen + 1` bytes however long the
//! block really is. That is sound here, and only here, because the `dealloc`
//! and `realloc` below ignore `layout.size()` on the `malloc` path -- `free`
//! never took a size, and `realloc` reads the block's own -- so the
//! understated capacity can neither shorten a release nor truncate a move.
//! The alignment in the `Layout` is the part that must be right, and a byte
//! container's is 1.
//!
//! `std::alloc::System` deliberately isn't used for this. It is documented
//! as possibly wrapping the platform allocator in extra bookkeeping, which
//! would make handing its pointers to `free` (or `malloc`'s to `Box`)
//! undefined. Bottoming out in malloc directly is the whole point.
//!
//! # Failure
//!
//! A request `malloc` refuses ends the process the way `xmalloc` does
//! (`E41`, swap files written: `startup/out_of_memory.rs`), not with the
//! standard library's bare abort. A `try_reserve` made through
//! `startup::fallibly` still gets its null, and so does every request in a
//! process that never ran `main`.
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
// Unsafe perimeter: the `allocator.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]

use core::alloc::{GlobalAlloc, Layout};

use crate::startup::allocation_failed;

/// Alignment `malloc` already guarantees. Larger requests go through
/// `posix_memalign`; `free` accepts pointers from either source.
const MALLOC_ALIGN: usize = align_of::<libc::max_align_t>();

#[derive(Debug)]
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

/// `malloc` refused `size` bytes: end the process the way `xmalloc` does
/// (`E41`, swap files written), unless the request asked to fail or there is
/// no editor to preserve. Returns only for those, and the allocator answers
/// null.
///
/// Calling into the editor from inside the allocator is sound here because
/// this allocator holds no state of its own: the way out may allocate and
/// free through it again.
#[cold]
#[inline(never)]
fn refused(size: usize) {
    allocation_failed(size);
}

// SAFETY: blocks satisfy the requested size and alignment, stay valid until
// released through this same impl, and null reports failure.
unsafe impl GlobalAlloc for LibcAllocator {
    /// # Safety
    ///
    /// As [`GlobalAlloc::alloc`]: `layout` must have a non-zero size.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let block = malloc_aligned(layout.size(), layout.align());
        // The success path's one null check; the failure is out of line.
        if block.is_null() {
            refused(layout.size());
        }
        block
    }

    /// # Safety
    ///
    /// As [`GlobalAlloc::alloc_zeroed`]: `layout` must have a non-zero size.
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let block = calloc_aligned(layout.size(), layout.align());
        if block.is_null() {
            refused(layout.size());
        }
        block
    }

    /// # Safety
    ///
    /// As [`GlobalAlloc::realloc`]: `ptr` must be a block this allocator answered
    /// for `layout` and has not released, and `new_size` must be a valid size for
    /// `layout`'s alignment.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let fast = layout.align() <= MALLOC_ALIGN;
        // Over-aligned, `realloc` would not preserve the alignment, so the
        // move is by hand into a fresh block.
        let fresh = if fast {
            core::ptr::null_mut()
        } else {
            malloc_aligned(new_size, layout.align())
        };
        // SAFETY: by the trait contract `ptr` came from this allocator with
        // `layout`, so from `malloc` on the fast path — and on the
        // over-aligned path it is live, at least `layout.size()` bytes long,
        // distinct from the fresh block, and ours to release.
        let new_ptr = unsafe {
            if fast {
                libc::realloc(ptr.cast(), new_size).cast()
            } else {
                if !fresh.is_null() {
                    core::ptr::copy_nonoverlapping(ptr, fresh, layout.size().min(new_size));
                    self.dealloc(ptr, layout);
                }
                fresh
            }
        };
        if new_ptr.is_null() {
            refused(new_size);
        }
        new_ptr
    }

    /// # Safety
    ///
    /// As [`GlobalAlloc::dealloc`]: `ptr` must be a block this allocator answered
    /// for `layout` and has not released.
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // SAFETY: by the trait contract `ptr` came from this allocator, so
        // from `malloc`/`calloc`/`posix_memalign`.
        unsafe { libc::free(ptr.cast()) };
    }
}

#[global_allocator]
static GLOBAL: LibcAllocator = LibcAllocator;

#[cfg(test)]
mod tests {
    /// A refused request in a process that never ran `main` (this one)
    /// answers null as `GlobalAlloc` says, so `try_reserve` still reports
    /// it: the out-of-memory exit is only for the editor.
    #[test]
    #[cfg_attr(miri, ignore = "Miri reports a huge malloc instead of failing it")]
    fn a_refused_request_without_main_answers_null() {
        let mut bytes: Vec<u8> = Vec::new();
        assert!(bytes.try_reserve_exact(1 << 62).is_err());
        assert!(bytes.try_reserve_exact(16).is_ok());
    }
}
