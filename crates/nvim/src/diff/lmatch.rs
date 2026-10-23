//! The `linematch:` post-pass.
//!
//! The diff engine answers in whole blocks; with `'diffopt'`'s `linematch:N`
//! each block short enough is handed to `linematch_nbuffers`, which re-aligns
//! the individual lines inside it, and [`apply_linematch_results`] replaces
//! the one block with the several the alignment implies.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use core::ffi::c_int;

/// Whether `dp` is short enough for the line-matching pass.
///
/// `linematch:N` is a budget on the block's *total* size across every buffer,
/// because the algorithm is exponential in the number of buffers.
pub fn diff_linematch(dp: &DiffBlock, slots: &DiffSlots) -> bool {
    if diff_flags.get() & DIFF_LINEMATCH == 0 {
        return false;
    }
    let mut total = 0;
    for i in used_slots(slots) {
        if dp.df_count[i] < 0 {
            return false;
        }
        total += dp.df_count[i];
    }
    total <= linematch_lines.get()
}

/// Split `blocks[at]` into one block per run of equal decisions.
///
/// A decision is a bit set over the *active* buffers, one entry per output
/// line: bit `j` says buffer `outputmap[j]` contributes a line there.  A run
/// of identical decisions is one diff block, so the block list grows by one
/// every time the set changes.
fn apply_linematch_results(
    blocks: &mut Vec<DiffBlock>,
    at: usize,
    slots: &DiffSlots,
    decisions: &[c_int],
) {
    let mut line_numbers = [0 as LineNr; DB_COUNT as usize];
    let mut outputmap = [0usize; DB_COUNT as usize];
    let mut ndiffs = 0;
    let dp = &mut blocks[at];
    for (i, lnum) in line_numbers.iter_mut().enumerate() {
        if slots[i].is_some() {
            *lnum = dp.df_lnum[i];
            dp.df_count[i] = 0;
            outputmap[ndiffs] = i;
            ndiffs += 1;
        }
    }
    let mut cur = at;
    for (k, &decision) in decisions.iter().enumerate() {
        if k != 0 && decisions[k - 1] != decision {
            let mut next = DiffBlock {
                is_linematched: true,
                ..DiffBlock::default()
            };
            for (i, &lnum) in line_numbers.iter().enumerate() {
                if slots[i].is_some() {
                    next.df_lnum[i] = lnum;
                }
            }
            cur += 1;
            blocks.insert(cur, next);
        }
        for (j, &buf) in outputmap[..ndiffs].iter().enumerate() {
            if decision & (1 << j) != 0 {
                blocks[cur].df_count[buf] += 1;
                line_numbers[buf] += 1;
            }
        }
    }
    blocks[at].is_linematched = true;
}

/// Re-align `blocks[at]`'s lines across every active buffer.
///
/// Each buffer's share of the block is written out as one memory image, the
/// alignment is computed over all of them at once, and the answer replaces
/// the block.
pub(crate) fn run_linematch_algorithm(blocks: &mut Vec<DiffBlock>, at: usize, slots: &DiffSlots) {
    let mut images: [Vec<u8>; DB_COUNT as usize] = ::core::array::from_fn(|_| Vec::new());
    let mut lengths = [0 as c_int; DB_COUNT as usize];
    let mut ndiffs = 0;
    let dp = &blocks[at];
    for i in 0..DB_COUNT as usize {
        if slots[i].is_none() {
            continue;
        }
        let (lnum, count) = (dp.df_lnum[i], dp.df_count[i]);
        if count > 0 {
            diff_write_buffer(
                used_slot(slots, i),
                &mut images[ndiffs],
                lnum,
                lnum + count - 1,
            );
        }
        lengths[ndiffs] = count;
        ndiffs += 1;
    }

    let iwhite = diff_flags.get() & (DIFF_IWHITEALL | DIFF_IWHITE) != 0;
    let images_in: [&[u8]; DB_COUNT as usize] = ::core::array::from_fn(|i| images[i].as_slice());
    let decisions = linematch_nbuffers(&images_in[..ndiffs], &lengths[..ndiffs], iwhite);
    drop(images);
    apply_linematch_results(blocks, at, slots, &decisions);
}
