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

// ---------------------------------------------------------------------------
// Owning an allocation whose address escapes

/// A heap allocation this program owns, held as the bare address the rest of
/// the editor works from.
///
/// [`Box`] is the right owner for a value nothing else points at. It is the
/// wrong one for the editor's long-lived objects. A `buf_T`'s *address* is
/// what the editor passes around: it sits in `curbuf`, in every `win_T`'s
/// `w_buffer`, on the `firstbuf` list and in whatever an autocommand kept —
/// and a `Box` **retags** every time it is moved (into a container, out of
/// one, across a call), which under Stacked and Tree Borrows invalidates
/// every raw pointer previously derived from it. A registry holding
/// `Box<buf_T>` would therefore break `curbuf` the first time a buffer was
/// filed or taken out.
///
/// `Owned<T>` is that `Box` turned into its address once, at birth. Moving
/// one copies a pointer and disturbs nothing that escaped; [`Drop`] turns it
/// back into a `Box` exactly once, so `T`'s destructor runs and a [`Vec`] or
/// a [`String`] may live inside an object the transpiled editor still reaches
/// by pointer.
///
/// The allocation comes from the Rust side of the one allocator this module
/// installs, so it is interchangeable with `xmalloc` memory in every way but
/// one: releasing it with `xfree` would skip `T`'s destructor. Dropping the
/// `Owned` is the only correct way to give it back.
pub(crate) struct Owned<T> {
    /// The address `Box::into_raw` answered. Never null, never reassigned.
    addr: *mut T,
}

impl<T> Owned<T> {
    /// Take over `value`'s allocation, keeping only its address.
    pub(crate) fn new(value: Box<T>) -> Self {
        Owned {
            addr: Box::into_raw(value),
        }
    }

    /// The address of the owned `T`.
    ///
    /// Reading it disturbs nothing: this hands back the pointer
    /// [`Owned::new`] was given, not a fresh borrow of `self`, so every
    /// address taken from the same `Owned` is usable at once — which is what
    /// the editor does with `curbuf`.
    pub(crate) fn address(&self) -> *mut T {
        self.addr
    }

    /// Give up ownership, answering the address.
    ///
    /// For the objects whose ownership travels as a bare pointer through
    /// transpiled code — `open_spellbuf`'s buffer lives in a `slang_T` field
    /// — and which are taken back with [`Owned::from_raw`] at their free
    /// point. Prefer holding the `Owned` where the shape allows it.
    pub(crate) fn into_raw(self) -> *mut T {
        let addr = self.addr;
        core::mem::forget(self);
        addr
    }

    /// Take ownership of `addr` back.
    ///
    /// # Safety
    /// `addr` must have come from [`Owned::into_raw`] and not have been
    /// taken back already.
    pub(crate) const unsafe fn from_raw(addr: *mut T) -> Self {
        Owned { addr }
    }
}

impl<T> Drop for Owned<T> {
    fn drop(&mut self) {
        // SAFETY: `addr` came from `Box::into_raw` in `new`, is never
        // reassigned, and reaches here once -- `Owned` is not `Copy` and
        // `into_raw` forgets the value it consumes.
        drop(unsafe { Box::from_raw(self.addr) });
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use super::Owned;

    /// Counts its own drops, so a test can say when the allocation went back.
    struct Tracked<'a>(&'a Cell<u32>);

    impl Drop for Tracked<'_> {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn dropping_an_owned_runs_the_destructor_once() {
        let drops = Cell::new(0);
        let owned = Owned::new(Box::new(Tracked(&drops)));
        assert_eq!(drops.get(), 0);
        drop(owned);
        assert_eq!(drops.get(), 1);
    }

    /// Every address taken out is the same one and they are usable together
    /// — the property a `Box` in a container could not give, and the reason
    /// `curbuf` may sit beside the registry's own copy.
    #[test]
    fn the_address_is_stable_and_may_be_taken_twice() {
        let drops = Cell::new(0);
        let mut owned = Owned::new(Box::new(Tracked(&drops)));
        let (first, second) = (owned.address(), owned.address());
        assert_eq!(first, second);
        // Moving the `Owned` moves a pointer, not the object.
        let moved = core::mem::replace(&mut owned, Owned::new(Box::new(Tracked(&drops))));
        assert_eq!(moved.address(), first);
        // SAFETY: `first` is `moved`'s allocation, which is still live.
        assert_eq!(unsafe { (*first).0.get() }, 0);
        drop(moved);
        drop(owned);
        assert_eq!(drops.get(), 2);
    }

    /// **The shape `free_buffer` has, and the reason a registry holds an
    /// `Owned` rather than a `Box`.**
    ///
    /// An address is taken out while the object is filed (that is `curbuf`),
    /// the table then changes under it, the free path takes the object out of
    /// the table, keeps working through the *address*, and drops at the end.
    /// Written with `Vec<Box<T>>` — `memfile::BlockTable`'s shape — this is
    /// undefined behaviour: the `Box` that comes out is retagged `Unique`,
    /// the write through the escaped address invalidates that tag, and Miri
    /// aborts at the `drop`, under Stacked *and* Tree Borrows. An `Owned` has
    /// no retag to invalidate. This test is here so that a slice which
    /// "simplifies" `OwnedRegistry`'s value type back to a `Box` hears about
    /// it from `just miri` rather than from a user.
    #[test]
    fn an_address_survives_the_table_changing_and_the_object_leaving_it() {
        struct Object<'a> {
            drops: &'a Cell<u32>,
            mark: u64,
        }

        impl Drop for Object<'_> {
            fn drop(&mut self) {
                self.drops.set(self.drops.get() + 1);
            }
        }

        let drops = Cell::new(0);
        let object = |drops| Owned::new(Box::new(Object { drops, mark: 0 }));
        let mut table = vec![object(&drops)];
        let escaped = table[0].address();
        table.push(object(&drops));
        let owned = table.swap_remove(0);
        // SAFETY: the object is alive -- `owned` holds it -- and `escaped` is
        // the address it was born with, which nothing has retagged.
        unsafe { (*escaped).mark = 42 };
        assert_eq!(owned.address(), escaped);
        // SAFETY: as above.
        assert_eq!(unsafe { (*escaped).mark }, 42);
        drop(owned);
        assert_eq!(drops.get(), 1);
        drop(table);
        assert_eq!(drops.get(), 2);
    }

    /// The shape the objects whose ownership travels as a bare pointer use:
    /// `open_spellbuf` gives the address up, `close_spellbuf` takes it back.
    #[test]
    fn into_raw_and_from_raw_hand_ownership_across() {
        let drops = Cell::new(0);
        let address = Owned::new(Box::new(Tracked(&drops))).into_raw();
        assert_eq!(drops.get(), 0, "into_raw does not free");
        // SAFETY: the address `into_raw` just gave up, taken back once.
        drop(unsafe { Owned::from_raw(address) });
        assert_eq!(drops.get(), 1);
    }
}
