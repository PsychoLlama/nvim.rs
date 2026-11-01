//! The block pool's reuse policy, read off the allocation recorder.
//!
//! Whether a spent block goes on the reuse list or back to the allocator is
//! invisible to every differential in the tree -- a correct answer built out
//! of the wrong block is the same answer -- and the release faults are leaks,
//! which the sanitizer lane does not report either (`ASAN_OPTIONS` pins
//! `detect_leaks=0`). It is visible to `memory::alloc_log`, because every one
//! of those questions is really "how many times did this call `xmalloc` and
//! `xfree`".
//!
//! Every case needs a live editor (the recorder takes the editor lock), which
//! Miri cannot start.

#![cfg(not(miri))]

use std::ffi::c_void;

use neovim::memory::alloc_log::AllocEvent;
use neovim::memory::blocks::{BLOCK_SIZE, alloc_block, free_block};
use neovim::memory::xfree;

use crate::support::alloc::{AllocLog, freed};

/// `xmalloc(size)` answering `ret`.
fn malloc<T>(size: usize, ret: *const T) -> AllocEvent {
    AllocEvent::Malloc {
        size,
        ret: ret as *mut c_void,
    }
}

/// Empty the reuse list, so a case measures its own allocations rather than
/// whatever an earlier one left on it.
///
/// There is no exported way to ask how many blocks are on the list -- but
/// there is a way to ask whether the *next* one comes off it, which is
/// whether taking a block logged an allocation. Blocks that came off the
/// list are released outright rather than handed back, which is the drain.
///
/// # Safety
/// The caller holds the editor lock through `log`.
unsafe fn drain_reuse_list(log: &AllocLog) {
    loop {
        log.clear();
        let blk = unsafe { alloc_block() };
        let fresh = !log.take().is_empty();
        unsafe { xfree(blk) };
        if fresh {
            return;
        }
    }
}

/// A spent block is kept for the next taker rather than freed, which is what
/// makes a stream's or a packer's next block free of allocator traffic.
#[test]
fn a_spent_block_is_reused_by_the_next_taker() {
    let log = AllocLog::start();
    // SAFETY: the block is this case's own, taken and given back here.
    unsafe {
        drain_reuse_list(&log);

        log.clear();
        let block = alloc_block();
        log.check(&[malloc(BLOCK_SIZE, block)]);

        // Giving it back puts it on the reuse list, not in the allocator.
        free_block(block);
        log.check(&[]);

        // ... and the next taker gets the same block for free.
        let again = alloc_block();
        log.check(&[]);
        assert_eq!(again, block, "the reused block");
        free_block(again);
        log.check(&[]);
    }
}

/// The list keeps four blocks; a fifth goes back to the allocator.
///
/// Anchors `REUSE_MAX`: moved either way, the fifth return frees or the
/// fourth does.
#[test]
fn the_reuse_list_keeps_four_blocks() {
    let log = AllocLog::start();
    // SAFETY: the five blocks are this case's own; the one freed is only
    // compared afterwards, never dereferenced.
    unsafe {
        drain_reuse_list(&log);
        log.clear();
        let blocks: Vec<_> = (0..5).map(|_| alloc_block()).collect();
        log.check(
            &blocks
                .iter()
                .map(|&b| malloc(BLOCK_SIZE, b))
                .collect::<Vec<_>>(),
        );
        for &block in &blocks[..4] {
            free_block(block);
        }
        log.check(&[]);
        free_block(blocks[4]);
        log.check(&[freed(blocks[4])]);
        drain_reuse_list(&log);
    }
}
