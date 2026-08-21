//! The arena allocator: a bump allocator over 4 KiB blocks, used for the
//! short-lived allocations an API call or a redraw makes and then drops all
//! at once.
//!
//! An [`Arena`] holds one block it is filling; spent blocks chain backwards
//! through a `consumed_blk` header written into each block's first bytes, so
//! there is no side table. [`arena_finish`] detaches that chain and
//! [`arena_mem_free`] releases it. A handful of blocks are kept on a global
//! reuse list rather than being freed, which is what makes an arena's second
//! use free of allocator traffic.
//!
//! A null `Arena` pointer is legal everywhere here and means "no arena":
//! [`arena_alloc`] falls through to [`xmalloc`], which is how the many
//! callers that take an optional arena spell the absent case.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_char, c_void};
use core::ptr;

use crate::global_cell::GlobalCell;
use crate::main::arena_alloc_count;
use crate::memory::{cbytes, copy_bytes, xfree, xmalloc};
use crate::types::{Arena, ArenaMem, consumed_blk};

pub const ARENA_BLOCK_SIZE: usize = 4096;
const REUSE_MAX: usize = 4;

pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ptr::null_mut(),
    pos: 0,
    size: 0,
};

static arena_reuse_blk: GlobalCell<*mut consumed_blk> = GlobalCell::new(ptr::null_mut());
static arena_reuse_blk_count: GlobalCell<usize> = GlobalCell::new(0);

/// Take the front block off the reuse list. `None` when it is empty.
///
/// # Safety
///
/// The list holds only blocks this module put there, all still allocated.
unsafe fn pop_reuse_blk() -> Option<*mut consumed_blk> {
    let count = arena_reuse_blk_count.get();
    if count == 0 {
        return None;
    }
    let blk = arena_reuse_blk.get();
    // SAFETY: a nonzero count means the head is one of this module's own
    // blocks, whose first bytes are its `prev` link.
    arena_reuse_blk.set(unsafe { (*blk).prev });
    arena_reuse_blk_count.set(count - 1);
    Some(blk)
}

/// Hand the reuse list back to the allocator: what `try_to_free_memory`
/// reclaims when an allocation has already failed once.
///
/// # Safety
///
/// No arena block handed out from the reuse list is still in use.
pub(super) unsafe fn free_reuse_blks() {
    // SAFETY: the caller's promise; each block is `xmalloc`ed.
    while let Some(blk) = unsafe { pop_reuse_blk() } {
        unsafe { xfree(blk.cast::<c_void>()) };
    }
}

/// Detach the arena's chain of consumed blocks for a later
/// `arena_mem_free`, leaving the arena empty.
///
/// # Safety
///
/// `arena` points to a live `Arena`.
pub unsafe fn arena_finish(arena: *mut Arena) -> ArenaMem {
    // SAFETY: the caller's arena. The chain moves to the result; nothing
    // is freed here.
    let arena = unsafe { &mut *arena };
    let res = arena.cur_blk.cast::<consumed_blk>();
    *arena = ARENA_EMPTY;
    res
}

/// A fresh [`ARENA_BLOCK_SIZE`] block, from the reuse list when it has one.
///
/// # Safety
///
/// As [`try_malloc`].
pub unsafe fn alloc_block() -> *mut c_void {
    // SAFETY: the reuse list holds this module's own blocks.
    match unsafe { pop_reuse_blk() } {
        Some(blk) => blk.cast::<c_void>(),
        None => {
            arena_alloc_count.set(arena_alloc_count.get().wrapping_add(1));
            // SAFETY: as `try_malloc`.
            unsafe { xmalloc(ARENA_BLOCK_SIZE) }
        }
    }
}

/// Start a new block, chaining the one it replaces behind it.
///
/// # Safety
///
/// `arena` points to a live `Arena`; otherwise as [`try_malloc`].
pub unsafe fn arena_alloc_block(arena: *mut Arena) {
    // SAFETY: the caller's arena. The header the block opens with is the
    // first thing allocated out of it, so `blk` is that block's own start.
    unsafe {
        let prev_blk = (*arena).cur_blk.cast::<consumed_blk>();
        (*arena).cur_blk = alloc_block().cast::<c_char>();
        (*arena).pos = 0;
        (*arena).size = ARENA_BLOCK_SIZE;
        // The block's first bytes link to the previous block.
        let blk = arena_alloc(arena, size_of::<consumed_blk>(), true).cast::<consumed_blk>();
        (*blk).prev = prev_blk;
    }
}

pub const ARENA_ALIGN: usize = {
    let ptr_size = core::mem::size_of::<*mut c_void>();
    let double_size = core::mem::size_of::<f64>();
    if ptr_size > double_size {
        ptr_size
    } else {
        double_size
    }
};

/// Round `off` up to the arena's allocation alignment.
fn align_offset(off: usize) -> usize {
    (off.wrapping_add(ARENA_ALIGN - 1)) & !(ARENA_ALIGN - 1)
}

/// Allocations that would waste more than half a block get their own
/// exactly-sized block instead.
fn is_oversize(size: usize) -> bool {
    size > (ARENA_BLOCK_SIZE - size_of::<consumed_blk>()) / 2
}

/// Where an allocation starts, given the block's current fill and whether
/// the caller asked for alignment.
fn bump_to(pos: usize, align: bool) -> usize {
    if align { align_offset(pos) } else { pos }
}

/// The room an over-sized block reserves for its own `prev` link before the
/// payload starts.
fn header_bytes(align: bool) -> usize {
    bump_to(size_of::<consumed_blk>(), align)
}

/// What a request resolves to before any pointer is formed: the whole of the
/// arena's bump logic, with the pointers taken out of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Bump {
    /// Serve it from the current block, starting at this offset.
    Here { at: usize },
    /// The current block is spent; start a new one and ask again.
    NewBlock,
    /// Too big to be worth a block of its own to fill: give it an exactly
    /// sized allocation with `hdr` bytes of chaining header in front.
    Own { hdr: usize },
}

/// Where `size` bytes go, given a block `block_size` bytes long that is
/// already `pos` bytes full.
fn plan(pos: usize, size: usize, block_size: usize, align: bool) -> Bump {
    let at = bump_to(pos, align);
    if at.wrapping_add(size) <= block_size {
        Bump::Here { at }
    } else if is_oversize(size) {
        Bump::Own {
            hdr: header_bytes(align),
        }
    } else {
        Bump::NewBlock
    }
}

/// `size` bytes out of `arena`, or -- with a null `arena` -- straight off
/// [`xmalloc`], which is how a great many callers spell "no arena here".
///
/// # Safety
///
/// `arena` is null or points to a live `Arena`; otherwise as [`try_malloc`].
pub unsafe fn arena_alloc(arena: *mut Arena, size: usize, align: bool) -> *mut c_void {
    if arena.is_null() {
        // SAFETY: as `try_malloc`.
        return unsafe { xmalloc(size) };
    }
    // SAFETY: the caller's arena, read and written one field at a time.
    // No borrow of it is held across an allocation: `xmalloc`'s failure path
    // re-enters the editor (`try_to_free_memory`), and what that runs is
    // allowed to reach an arena of its own.
    if unsafe { (*arena).cur_blk }.is_null() {
        unsafe { arena_alloc_block(arena) };
    }
    let (pos, block_size) = unsafe { ((*arena).pos, (*arena).size) };
    let at = match plan(pos, size, block_size, align) {
        Bump::Here { at } => at,
        Bump::Own { hdr } => {
            // Chain an exactly-sized block *behind* the current one, so the
            // current block's remaining space stays usable.
            arena_alloc_count.set(arena_alloc_count.get().wrapping_add(1));
            // SAFETY: `xmalloc` answers `size + hdr` writable bytes, whose
            // first `size_of::<consumed_blk>()` become the new link; the
            // current block's own header is where its `prev` lives.
            return unsafe {
                let alloc = xmalloc(size.wrapping_add(hdr)).cast::<c_char>();
                let cur_blk = (*arena).cur_blk.cast::<consumed_blk>();
                let fix_blk = alloc.cast::<consumed_blk>();
                (*fix_blk).prev = (*cur_blk).prev;
                (*cur_blk).prev = fix_blk;
                alloc.add(hdr).cast::<c_void>()
            };
        }
        // A fresh block always serves it: `is_oversize` said otherwise, and
        // a block's header leaves far more than that free.
        Bump::NewBlock => {
            unsafe { arena_alloc_block(arena) };
            bump_to(unsafe { (*arena).pos }, align)
        }
    };
    // SAFETY: `plan` leaves `at + size` inside the current block.
    let mem = unsafe { (*arena).cur_blk.add(at) };
    unsafe { (*arena).pos = at.wrapping_add(size) };
    mem.cast::<c_void>()
}

/// Give a spent arena block back, keeping up to [`REUSE_MAX`] of them for
/// the next arena rather than returning them to the allocator.
///
/// # Safety
///
/// `block` is an [`ARENA_BLOCK_SIZE`] block from [`alloc_block`] that
/// nothing else still points into.
pub unsafe extern "C" fn free_block(block: *mut c_void) {
    let count = arena_reuse_blk_count.get();
    if count >= REUSE_MAX {
        // SAFETY: the caller's block, which came off `xmalloc`.
        unsafe { xfree(block) };
        return;
    }
    let reuse_blk = block.cast::<consumed_blk>();
    // SAFETY: a block's first bytes are its `prev` link.
    unsafe { (*reuse_blk).prev = arena_reuse_blk.get() };
    arena_reuse_blk.set(reuse_blk);
    arena_reuse_blk_count.set(count + 1);
}

/// Release a chain detached by [`arena_finish`].
///
/// # Safety
///
/// `mem` is such a chain, and nothing still points into any of its blocks.
pub unsafe fn arena_mem_free(mem: ArenaMem) {
    let mut b = mem;
    // The first block may be reused; the rest of the chain is freed.
    if !b.is_null() {
        let reuse_blk = b;
        // SAFETY: the caller's chain; each link is a block's first bytes.
        b = unsafe { (*b).prev };
        unsafe { free_block(reuse_blk.cast::<c_void>()) };
    }
    while !b.is_null() {
        // SAFETY: as above.
        let prev = unsafe { (*b).prev };
        unsafe { xfree(b.cast::<c_void>()) };
        b = prev;
    }
}

/// `size` unaligned bytes out of `arena`, plus a NUL terminator.
///
/// # Safety
///
/// As [`arena_alloc`].
pub unsafe fn arena_allocz(arena: *mut Arena, size: usize) -> *mut c_char {
    // SAFETY: the caller's arena; the terminator is the extra byte asked
    // for just above.
    unsafe {
        let mem = arena_alloc(arena, size.wrapping_add(1), false).cast::<c_char>();
        *mem.add(size) = 0;
        mem
    }
}

/// `size` bytes of `buf`, copied into `arena` and NUL-terminated.
///
/// # Safety
///
/// `buf` is readable for `size` bytes; otherwise as [`arena_alloc`].
pub unsafe fn arena_memdupz(arena: *mut Arena, buf: *const c_char, size: usize) -> *mut c_char {
    // SAFETY: fresh arena bytes cannot overlap the caller's buffer.
    unsafe {
        let mem = arena_allocz(arena, size);
        copy_bytes(mem.cast::<c_void>(), buf.cast::<c_void>(), size);
        mem
    }
}

/// A C string copied into `arena`.
///
/// # Safety
///
/// `str` is NUL-terminated; otherwise as [`arena_alloc`].
pub unsafe fn arena_strdup(arena: *mut Arena, str: *const c_char) -> *mut c_char {
    // SAFETY: the caller's NUL-terminated string.
    let len = unsafe { cbytes(str) }.len();
    unsafe { arena_memdupz(arena, str, len) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_alignment_rounds_up_to_the_platform_align() {
        assert_eq!(align_offset(0), 0);
        assert_eq!(align_offset(1), ARENA_ALIGN);
        assert_eq!(align_offset(ARENA_ALIGN), ARENA_ALIGN);
        assert_eq!(align_offset(ARENA_ALIGN + 1), 2 * ARENA_ALIGN);
    }

    #[test]
    fn oversize_threshold_is_half_a_block_minus_header() {
        let threshold = (ARENA_BLOCK_SIZE - size_of::<consumed_blk>()) / 2;
        assert!(!is_oversize(threshold));
        assert!(is_oversize(threshold + 1));
    }

    /// What a fresh block's `pos` is once its own `prev` header has been
    /// bump-allocated out of it.
    const HDR: usize = ARENA_ALIGN;

    #[test]
    fn an_aligned_request_rounds_the_block_position_up() {
        assert_eq!(plan(HDR, 8, ARENA_BLOCK_SIZE, true), Bump::Here { at: HDR });
        // An unaligned tail is rounded up for an aligned request...
        assert_eq!(
            plan(HDR + 1, 8, ARENA_BLOCK_SIZE, true),
            Bump::Here {
                at: HDR + ARENA_ALIGN
            }
        );
        // ... and taken as-is for an unaligned one, which is what makes
        // `arena_allocz`'s byte strings pack.
        assert_eq!(
            plan(HDR + 1, 8, ARENA_BLOCK_SIZE, false),
            Bump::Here { at: HDR + 1 }
        );
    }

    #[test]
    fn a_request_that_exactly_fills_the_block_still_fits() {
        let at = ARENA_BLOCK_SIZE - 64;
        assert_eq!(plan(at, 64, ARENA_BLOCK_SIZE, false), Bump::Here { at });
        // One byte more and the block is spent. 65 is well under the
        // oversize threshold, so it is a new block rather than its own.
        assert_eq!(plan(at, 65, ARENA_BLOCK_SIZE, false), Bump::NewBlock);
    }

    #[test]
    fn oversize_only_decides_what_happens_once_the_block_is_too_full() {
        let big = (ARENA_BLOCK_SIZE - size_of::<consumed_blk>()) / 2 + 1;
        // An over-sized request that still fits is served from the block,
        // however over-sized it is: the first question is whether it fits,
        // and only then whether it deserves a block of its own.
        assert_eq!(
            plan(HDR, big, ARENA_BLOCK_SIZE, true),
            Bump::Here { at: HDR }
        );
        // From a nearly-full block the same request gets its own, chained
        // behind the current one so the remaining space stays usable.
        let full = ARENA_BLOCK_SIZE - 8;
        assert_eq!(
            plan(full, big, ARENA_BLOCK_SIZE, true),
            Bump::Own {
                hdr: header_bytes(true)
            }
        );
        // One byte under the threshold is not over-sized, so it takes a
        // whole fresh block instead and leaves the rest of that one usable.
        assert_eq!(plan(full, big - 1, ARENA_BLOCK_SIZE, true), Bump::NewBlock);
    }

    #[test]
    fn a_fresh_block_always_serves_what_was_not_oversize() {
        // This is what `arena_alloc`'s `NewBlock` arm relies on: after
        // `arena_alloc_block` the position is the header, and every
        // non-oversize request fits after it, aligned or not.
        let biggest = (ARENA_BLOCK_SIZE - size_of::<consumed_blk>()) / 2;
        for size in [1, 2, 17, 1000, biggest] {
            for align in [false, true] {
                assert!(
                    matches!(plan(HDR, size, ARENA_BLOCK_SIZE, align), Bump::Here { .. }),
                    "{size} {align}"
                );
            }
        }
    }

    #[test]
    fn allocations_never_overlap_within_a_block() {
        // Walk a block the way `arena_alloc` does and check the served
        // ranges tile it in order.
        let mut pos = HDR;
        let mut last_end = HDR;
        for (n, &size) in [8usize, 1, 3, 64, 1, 200]
            .iter()
            .cycle()
            .take(60)
            .enumerate()
        {
            let align = n % 3 == 0;
            let Bump::Here { at } = plan(pos, size, ARENA_BLOCK_SIZE, align) else {
                break;
            };
            assert!(at >= last_end, "{n}: {at} < {last_end}");
            assert!(at + size <= ARENA_BLOCK_SIZE, "{n}");
            if align {
                assert_eq!(at % ARENA_ALIGN, 0, "{n}");
            }
            last_end = at + size;
            pos = last_end;
        }
    }
}
