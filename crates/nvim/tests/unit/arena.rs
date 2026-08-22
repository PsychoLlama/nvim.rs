//! The arena allocator's block policy, read off the allocation recorder.
//!
//! `memory/arena.rs` has two halves. What a request *resolves to* — the bump
//! arithmetic and the three copy helpers — is visible in every answer the
//! editor gives, and `~/agents/tools/1787432513-arenamutate.py` anchors it
//! against `stlsweep`. The other half is *block policy*: whether an
//! over-sized request gets a block of its own, whether a spent block goes on
//! the reuse list or back to the allocator, and whether the consumed chain is
//! released at all. That half is invisible to every differential in the tree
//! — a correct answer built out of the wrong blocks is the same answer — and
//! the two release faults are leaks, which the sanitizer lane does not report
//! either (`ASAN_OPTIONS` pins `detect_leaks=0`).
//!
//! It is visible to `memory::alloc_log`, because every one of those questions
//! is really "how many times, and for how many bytes, did this call
//! `xmalloc`". So the four mutations that script records as caught by nothing
//! are caught here.
//!
//! These live under `tests/` rather than in `arena.rs` itself for a reason
//! worth knowing: the ratchet counts `unsafe` lines inside `#[cfg(test)]`
//! modules like any other, so an in-file test that drives `arena_alloc` would
//! book unchecked lines against the very file the migration is trying to
//! shrink. Integration tests are not measured.
//!
//! Every case needs a live editor (the recorder takes the editor lock), which
//! Miri cannot start.

#![cfg(not(miri))]

use std::ffi::c_void;

use c2rust_neovim::memory::alloc_log::AllocEvent;
use c2rust_neovim::memory::arena::{
    ARENA_ALIGN, ARENA_BLOCK_SIZE, ARENA_EMPTY, alloc_block, arena_alloc, arena_finish,
    arena_mem_free,
};
use c2rust_neovim::memory::xfree;
use c2rust_neovim::types::{Arena, consumed_blk};

use crate::support::alloc::{AllocLog, freed};

/// `xmalloc(size)` answering `ret`.
fn malloc<T>(size: usize, ret: *const T) -> AllocEvent {
    AllocEvent::Malloc {
        size,
        ret: ret as *mut c_void,
    }
}

/// The size at which a request stops being worth a shared block.
fn oversize_threshold() -> usize {
    (ARENA_BLOCK_SIZE - size_of::<consumed_blk>()) / 2
}

/// Empty the module's reuse list, so a case measures its own allocations
/// rather than whatever an earlier one left on it.
///
/// There is no exported way to ask how many blocks are on the list — but
/// there is a way to ask whether the *next* one comes off it, which is
/// whether taking a block logged an allocation. Blocks that came off the
/// list are released outright rather than handed back, which is the drain.
///
/// # Safety
/// The caller holds the editor lock through `log`, and no arena is live.
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

/// A spent block is kept for the next arena rather than freed, which is what
/// makes an arena's second use free of allocator traffic.
///
/// Anchors `REUSE_MAX`. With the reuse list turned off, freeing the first
/// arena logs a `Free` and the second arena logs a `Malloc`.
#[test]
fn a_spent_block_is_reused_by_the_next_arena() {
    let log = AllocLog::start();
    // SAFETY: both arenas are this case's own and are finished and freed
    // before it returns.
    unsafe {
        drain_reuse_list(&log);

        let mut first: Arena = ARENA_EMPTY;
        log.clear();
        arena_alloc(&raw mut first, 16, true);
        let block = first.cur_blk;
        log.check(&[malloc(ARENA_BLOCK_SIZE, block)]);

        // Releasing it hands it to the reuse list, not to the allocator.
        arena_mem_free(arena_finish(&raw mut first));
        log.check(&[]);

        // ... and the next arena picks up the same block for free.
        let mut second: Arena = ARENA_EMPTY;
        arena_alloc(&raw mut second, 16, true);
        log.check(&[]);
        assert_eq!(second.cur_blk, block, "the reused block");

        arena_mem_free(arena_finish(&raw mut second));
        log.check(&[]);
    }
}

/// A request too big to be worth filling a shared block with gets an
/// exactly-sized one, chained behind the current block so the current
/// block's remaining space stays usable.
///
/// Anchors `is_oversize`'s threshold from both sides: one byte under it takes
/// a whole fresh 4 KiB block, one byte over it takes `size + ARENA_ALIGN`.
/// Moving the threshold in either direction swaps which of the two happens.
#[test]
fn the_oversize_threshold_decides_which_block_a_request_gets() {
    let log = AllocLog::start();
    let threshold = oversize_threshold();
    // SAFETY: the arena is this case's own and is finished and freed below.
    unsafe {
        drain_reuse_list(&log);
        let mut arena: Arena = ARENA_EMPTY;

        // Enough to leave less than `threshold` free in the first block. It
        // fits where it lands, so however over-sized it is, it is served
        // from the block: the first question is whether it fits.
        log.clear();
        arena_alloc(&raw mut arena, 3000, false);
        let first = arena.cur_blk;
        log.check(&[malloc(ARENA_BLOCK_SIZE, first)]);

        // One byte under the threshold: not worth its own block, so the
        // arena starts a fresh shared one and leaves the rest of it usable.
        arena_alloc(&raw mut arena, threshold, false);
        let second = arena.cur_blk;
        assert_ne!(second, first, "a new block");
        log.check(&[malloc(ARENA_BLOCK_SIZE, second)]);

        // One byte over: its own exactly-sized allocation, with room in
        // front for the link that chains it behind the current block. The
        // current block is still the arena's, so `cur_blk` does not move.
        let own = arena_alloc(&raw mut arena, threshold + 1, false);
        assert_eq!(arena.cur_blk, second, "the current block is untouched");
        log.check(&[malloc(
            threshold + 1 + ARENA_ALIGN,
            own.cast::<u8>().sub(ARENA_ALIGN),
        )]);

        arena_mem_free(arena_finish(&raw mut arena));
        log.clear();
    }
}

/// Every block after the first goes back to the allocator, in the order the
/// chain links them: newest first.
///
/// Anchors both halves of the release path — `arena_finish` handing over the
/// chain and `arena_mem_free` walking it. Dropping either leaks silently:
/// no differential can see a leak and the sanitizer lane runs with leak
/// checking off, so this is the only gate the tree has for it.
#[test]
fn every_consumed_block_but_the_first_is_released() {
    let log = AllocLog::start();
    // SAFETY: the arena is this case's own; the pointers recorded below are
    // only compared, never dereferenced after the free.
    unsafe {
        drain_reuse_list(&log);
        let mut arena: Arena = ARENA_EMPTY;

        // Two 2 000-byte allocations fit in a block beside its own 8-byte
        // link; the fifth therefore sits in the third block.
        let mut blocks = Vec::new();
        log.clear();
        for _ in 0..5 {
            arena_alloc(&raw mut arena, 2000, false);
            if blocks.last() != Some(&arena.cur_blk) {
                blocks.push(arena.cur_blk);
            }
        }
        assert_eq!(blocks.len(), 3, "three blocks");
        log.check(
            &blocks
                .iter()
                .map(|&b| malloc(ARENA_BLOCK_SIZE, b))
                .collect::<Vec<_>>(),
        );

        // The head is kept for reuse; the two behind it are freed, newest
        // first, which is the order the `prev` links run in.
        arena_mem_free(arena_finish(&raw mut arena));
        log.check(&[freed(blocks[1]), freed(blocks[0])]);
    }
}
