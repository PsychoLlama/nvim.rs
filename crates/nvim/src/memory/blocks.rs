//! The block pool: the 4 KiB blocks the event loop reads into and the RPC
//! and UI packers write into, recycled rather than freed.
//!
//! A handful of spent blocks are kept on a reuse list instead of going back
//! to the allocator, which is what makes a stream's or a packer's next block
//! free of allocator traffic. A block is a `Vec<u8>` of [`BLOCK_SIZE`]
//! capacity from end to end: [`block_vec`] hands one out empty, and
//! [`recycle_block`] takes it back.
//!
//! The list is invisible to every behavioural differential -- a correct
//! answer built out of the wrong block is the same answer, and a block not
//! given back is a leak. The tests below read the policy off the list and
//! [`ARENA_ALLOC_COUNT`]; change anything here and run them.

#![forbid(unsafe_code)]

use crate::global_cell::GlobalCell;
use crate::memory::ARENA_ALLOC_COUNT;

/// The size of a pool block.
pub const BLOCK_SIZE: usize = 4096;
/// How many spent blocks the reuse list keeps.
const REUSE_MAX: usize = 4;

/// The reuse list: spent blocks, emptied, last spent on top.
static SPARE: GlobalCell<Vec<Vec<u8>>> = GlobalCell::new(Vec::new());

/// Hand the reuse list back to the allocator: what `try_to_free_memory`
/// reclaims when an allocation has already failed once.
pub(crate) fn free_reuse_blks() {
    drop(SPARE.take());
}

/// A fresh [`BLOCK_SIZE`] block as an empty vector, from the reuse list
/// when it has one: the window a packer writes a message into, or a stream
/// reads into.
pub(crate) fn block_vec() -> Vec<u8> {
    if let Some(block) = SPARE.with_mut(Vec::pop) {
        return block;
    }
    ARENA_ALLOC_COUNT.set(ARENA_ALLOC_COUNT.get().wrapping_add(1));
    Vec::with_capacity(BLOCK_SIZE)
}

/// Give a vector's block back: onto the reuse list when it is a block's
/// size and the list has room, to the allocator otherwise.
pub(crate) fn recycle_block(mut block: Vec<u8>) {
    if block.capacity() != BLOCK_SIZE {
        return;
    }
    block.clear();
    SPARE.with_mut(|spare| {
        if spare.len() < REUSE_MAX {
            spare.push(block);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;

    fn spare_count() -> usize {
        SPARE.with(Vec::len)
    }

    /// A spent block goes onto the reuse list, and the next taker gets that
    /// very block back, empty, with no allocator traffic.
    #[test]
    fn a_spent_block_is_the_next_one_handed_out() {
        let _held = editor_state_lock();
        free_reuse_blks();
        let mut spent = block_vec();
        spent.extend_from_slice(b"read off a stream");
        let address = spent.as_ptr();
        recycle_block(spent);
        assert_eq!(spare_count(), 1);

        let allocated = ARENA_ALLOC_COUNT.get();
        let mut again = block_vec();
        assert_eq!(again.as_ptr(), address, "the reuse list's block");
        assert_eq!(ARENA_ALLOC_COUNT.get(), allocated, "no allocator traffic");
        assert!(again.is_empty());
        assert_eq!(again.capacity(), BLOCK_SIZE);
        // Every byte is the taker's.
        again.resize(BLOCK_SIZE, 0xa5);
        assert!(again.iter().all(|&byte| byte == 0xa5));
        recycle_block(again);
        free_reuse_blks();
        assert_eq!(spare_count(), 0);
    }

    /// A taker with the list empty is counted as an allocation.
    #[test]
    fn an_empty_list_allocates() {
        let _held = editor_state_lock();
        free_reuse_blks();
        let allocated = ARENA_ALLOC_COUNT.get();
        let block = block_vec();
        assert_eq!(ARENA_ALLOC_COUNT.get(), allocated.wrapping_add(1));
        assert_eq!(block.capacity(), BLOCK_SIZE);
        recycle_block(block);
        free_reuse_blks();
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
        assert_eq!(spare_count(), REUSE_MAX);
        recycle_block(Vec::with_capacity(16));
        assert_eq!(spare_count(), REUSE_MAX);

        let taken: Vec<Vec<u8>> = (0..REUSE_MAX).map(|_| block_vec()).collect();
        let kept: Vec<*const u8> = addresses[..REUSE_MAX].iter().rev().copied().collect();
        let got: Vec<*const u8> = taken.iter().map(|block| block.as_ptr()).collect();
        assert_eq!(got, kept);
        assert_eq!(spare_count(), 0);
        for block in taken {
            recycle_block(block);
        }
        free_reuse_blks();
        assert_eq!(spare_count(), 0);
    }
}
