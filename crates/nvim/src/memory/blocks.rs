//! The block pool: the 4 KiB blocks the event loop reads into and the RPC
//! and UI packers write into, recycled rather than freed.
//!
//! A handful of spent blocks are kept on a reuse list instead of going back
//! to the allocator, which is what makes a stream's or a packer's next block
//! free of allocator traffic. The list chains through a [`ConsumedBlk`]
//! link written into each spent block's first bytes, so it needs no side
//! table. What is raw here is that list and the one C-shaped finalizer a
//! write buffer hands its block back through ([`free_block`]).
//!
//! The list is invisible to every behavioural differential -- a correct
//! answer built out of the wrong block is the same answer, and a block not
//! given back is a leak. `crates/nvim/tests/unit/blocks.rs` reads the
//! policy off [`crate::memory::alloc_log`]; change anything below and run
//! it.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `memory/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_void;
use core::ptr;

use crate::global_cell::GlobalCell;
use crate::memory::ARENA_ALLOC_COUNT;
use crate::memory::{xfree, xmalloc};

/// The link a spent block on the block pool's reuse list carries in its own
/// first bytes: the block below it on the list.
#[derive(Debug)]
pub struct ConsumedBlk {
    pub prev: *mut ConsumedBlk,
}

/// The size of a pool block.
pub const BLOCK_SIZE: usize = 4096;
/// How many spent blocks the reuse list keeps.
const REUSE_MAX: usize = 4;

/// The reuse list's head, and how many blocks are on it.
static REUSE_HEAD: GlobalCell<*mut ConsumedBlk> = GlobalCell::new(ptr::null_mut());
static REUSE_COUNT: GlobalCell<usize> = GlobalCell::new(0);

/// Take the front block off the reuse list. `None` when it is empty.
fn pop_reuse_blk() -> Option<*mut ConsumedBlk> {
    let count = REUSE_COUNT.get();
    if count == 0 {
        return None;
    }
    let blk = REUSE_HEAD.get();
    // SAFETY: a nonzero count means the head is one of this module's own
    // blocks, still allocated, whose first bytes are its `prev` link.
    REUSE_HEAD.set(unsafe { (*blk).prev });
    REUSE_COUNT.set(count - 1);
    Some(blk)
}

/// Hand the reuse list back to the allocator: what `try_to_free_memory`
/// reclaims when an allocation has already failed once.
pub(super) fn free_reuse_blks() {
    while let Some(blk) = pop_reuse_blk() {
        // SAFETY: a block on the list is `xmalloc`ed and, being on the list,
        // handed out to no one.
        unsafe { xfree(blk.cast::<c_void>()) };
    }
}

/// A fresh [`BLOCK_SIZE`] block, from the reuse list when it has one.
///
/// # Safety
///
/// As [`super::try_malloc`].
pub unsafe fn alloc_block() -> *mut c_void {
    match pop_reuse_blk() {
        Some(blk) => blk.cast::<c_void>(),
        None => {
            ARENA_ALLOC_COUNT.set(ARENA_ALLOC_COUNT.get().wrapping_add(1));
            // SAFETY: as `try_malloc`.
            unsafe { xmalloc(BLOCK_SIZE) }
        }
    }
}

/// A fresh [`BLOCK_SIZE`] block as an empty vector, from the reuse list
/// when it has one: the window a packer writes a message into.
pub(crate) fn block_vec() -> Vec<u8> {
    // SAFETY: `alloc_block` answers a live `BLOCK_SIZE` block that nothing
    // else names, from `malloc` -- the global allocator -- so a vector may
    // own it with that capacity and nothing initialised.
    unsafe { Vec::from_raw_parts(alloc_block().cast::<u8>(), 0, BLOCK_SIZE) }
}

/// Give a vector's block back: onto the reuse list when it is a block's
/// size, to the allocator otherwise.
pub(crate) fn recycle_block(block: Vec<u8>) {
    if block.capacity() == BLOCK_SIZE {
        let mut block = core::mem::ManuallyDrop::new(block);
        // SAFETY: a `malloc` block of exactly `BLOCK_SIZE` bytes that the
        // vector gave up, so nothing else points into it.
        unsafe { free_block(block.as_mut_ptr().cast::<c_void>()) };
    }
}

/// Give a spent block back, keeping up to [`REUSE_MAX`] of them for the
/// next taker rather than returning them to the allocator.
///
/// # Safety
///
/// `block` is a [`BLOCK_SIZE`] block from [`alloc_block`] that nothing else
/// still points into.
pub unsafe extern "C" fn free_block(block: *mut c_void) {
    let count = REUSE_COUNT.get();
    if count >= REUSE_MAX {
        // SAFETY: the caller's block, which came off `xmalloc`.
        unsafe { xfree(block) };
        return;
    }
    let reuse_blk = block.cast::<ConsumedBlk>();
    // SAFETY: the caller's block is at least a link wide, and nothing else
    // reads it any more; its first bytes become its `prev` link.
    unsafe { (*reuse_blk).prev = REUSE_HEAD.get() };
    REUSE_HEAD.set(reuse_blk);
    REUSE_COUNT.set(count + 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;

    /// A spent block goes onto the reuse list with its link written over its
    /// first bytes, and the next taker gets that very block back, empty --
    /// the link read out of memory a vector owned a moment before.
    #[test]
    fn a_spent_block_is_the_next_one_handed_out() {
        let _held = editor_state_lock();
        free_reuse_blks();
        let mut spent = block_vec();
        spent.extend_from_slice(b"read off a stream");
        let address = spent.as_ptr();
        recycle_block(spent);
        assert_eq!(REUSE_COUNT.get(), 1);

        let allocated = ARENA_ALLOC_COUNT.get();
        let mut again = block_vec();
        assert_eq!(again.as_ptr(), address, "the reuse list's block");
        assert_eq!(ARENA_ALLOC_COUNT.get(), allocated, "no allocator traffic");
        assert!(again.is_empty());
        assert_eq!(again.capacity(), BLOCK_SIZE);
        // Every byte is the taker's, the link's included.
        again.resize(BLOCK_SIZE, 0xa5);
        assert!(again.iter().all(|&byte| byte == 0xa5));
        recycle_block(again);
        free_reuse_blks();
        assert_eq!(REUSE_COUNT.get(), 0);
    }

    /// The list keeps [`REUSE_MAX`] blocks, last spent first out, and frees
    /// the rest; a vector of another size never joins it.
    #[test]
    fn the_reuse_list_keeps_the_last_four_spent() {
        let _held = editor_state_lock();
        free_reuse_blks();
        let blocks: Vec<Vec<u8>> = (0..REUSE_MAX + 2).map(|_| block_vec()).collect();
        let addresses: Vec<*const u8> = blocks.iter().map(|block| block.as_ptr()).collect();
        for mut block in blocks {
            block.push(1);
            recycle_block(block);
        }
        assert_eq!(REUSE_COUNT.get(), REUSE_MAX);
        recycle_block(Vec::with_capacity(16));
        assert_eq!(REUSE_COUNT.get(), REUSE_MAX);

        let taken: Vec<Vec<u8>> = (0..REUSE_MAX).map(|_| block_vec()).collect();
        let kept: Vec<*const u8> = addresses[..REUSE_MAX].iter().rev().copied().collect();
        let got: Vec<*const u8> = taken.iter().map(|block| block.as_ptr()).collect();
        assert_eq!(got, kept);
        assert_eq!(REUSE_COUNT.get(), 0);
        for block in taken {
            recycle_block(block);
        }
        free_reuse_blks();
        assert_eq!(REUSE_COUNT.get(), 0);
    }
}
