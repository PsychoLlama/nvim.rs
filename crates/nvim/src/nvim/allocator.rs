//! The process-wide Rust allocator, backed by the same libc `malloc` the
//! `xmalloc` family bottoms out in.
//!
//! With both sides drawing from one allocator, heap ownership can legally
//! cross the C ABI in either direction: a buffer built with `Vec`/`Box`/
//! `CString` may be released with `xfree`, and `xmalloc` memory may be
//! adopted by Rust containers, with no copy-at-the-boundary layer.

use core::alloc::{GlobalAlloc, Layout};

/// Alignment `malloc` already guarantees. Larger requests go through
/// `posix_memalign`; `free` accepts pointers from either source.
const MALLOC_ALIGN: usize = core::mem::align_of::<libc::max_align_t>();

struct LibcAllocator;

unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MALLOC_ALIGN {
            libc::malloc(layout.size()).cast()
        } else {
            let mut ptr: *mut libc::c_void = core::ptr::null_mut();
            if libc::posix_memalign(&mut ptr, layout.align(), layout.size()) == 0 {
                ptr.cast()
            } else {
                core::ptr::null_mut()
            }
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MALLOC_ALIGN {
            libc::calloc(1, layout.size()).cast()
        } else {
            let ptr = self.alloc(layout);
            if !ptr.is_null() {
                core::ptr::write_bytes(ptr, 0, layout.size());
            }
            ptr
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.align() <= MALLOC_ALIGN {
            libc::realloc(ptr.cast(), new_size).cast()
        } else {
            // `realloc` does not preserve over-alignment; move by hand.
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            let new_ptr = self.alloc(new_layout);
            if !new_ptr.is_null() {
                core::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
                self.dealloc(ptr, layout);
            }
            new_ptr
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr.cast());
    }
}

#[global_allocator]
static GLOBAL: LibcAllocator = LibcAllocator;
