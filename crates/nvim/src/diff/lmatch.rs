//! The `linematch:` post-pass.
//!
//! The diff engine answers in whole blocks; with `'diffopt'`'s `linematch:N`
//! each block short enough is handed to `linematch_nbuffers`, which re-aligns
//! the individual lines inside it, and [`apply_linematch_results`] replaces
//! the one block with the several the alignment implies.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use core::ffi::c_int;

/// Whether `dp` is short enough for the line-matching pass.
///
/// `linematch:N` is a budget on the block's *total* size across every buffer,
/// because the algorithm is exponential in the number of buffers.
pub unsafe fn diff_linematch(dp: *mut diff_T) -> bool {
    unsafe {
        if diff_flags.get() & DIFF_LINEMATCH == 0 {
            return false;
        }
        let mut total = 0;
        for i in 0..DB_COUNT as usize {
            if (*curtab.get()).tp_diffbuf[i].is_null() {
                continue;
            }
            if (*dp).df_count[i] < 0 {
                return false;
            }
            total += (*dp).df_count[i];
        }
        total <= linematch_lines.get()
    }
}

/// Split `dp` into one block per run of equal decisions.
///
/// A decision is a bit set over the *active* buffers, one entry per output
/// line: bit `j` says buffer `outputmap[j]` contributes a line there.  A run
/// of identical decisions is one diff block, so the block list grows by one
/// every time the set changes.
unsafe fn apply_linematch_results(dp: *mut diff_T, decisions: &[c_int]) {
    unsafe {
        let tp = curtab.get();
        let mut line_numbers = [0 as linenr_T; DB_COUNT as usize];
        let mut outputmap = [0usize; DB_COUNT as usize];
        let mut ndiffs = 0;
        for (i, lnum) in line_numbers.iter_mut().enumerate() {
            if !(*tp).tp_diffbuf[i].is_null() {
                *lnum = (*dp).df_lnum[i];
                (*dp).df_count[i] = 0;
                outputmap[ndiffs] = i;
                ndiffs += 1;
            }
        }
        let mut cur = dp;
        for (at, &decision) in decisions.iter().enumerate() {
            if at != 0 && decisions[at - 1] != decision {
                cur = diff_alloc_new(tp, cur, (*cur).df_next);
                (*cur).is_linematched = true;
                for (i, &lnum) in line_numbers.iter().enumerate() {
                    if !(*tp).tp_diffbuf[i].is_null() {
                        (*cur).df_lnum[i] = lnum;
                        (*cur).df_count[i] = 0;
                    }
                }
            }
            for (j, &buf) in outputmap[..ndiffs].iter().enumerate() {
                if decision & (1 << j) != 0 {
                    (*cur).df_count[buf] += 1;
                    line_numbers[buf] += 1;
                }
            }
        }
        (*dp).is_linematched = true;
    }
}

/// Re-align `dp`'s lines across every active buffer.
///
/// Each buffer's share of the block is written out as one memory image, the
/// alignment is computed over all of them at once, and the answer replaces
/// the block.
pub(crate) unsafe fn run_linematch_algorithm(dp: *mut diff_T) {
    unsafe {
        let tp = curtab.get();
        let mut images = [MMFILE_INIT; DB_COUNT as usize];
        let mut lengths = [0 as c_int; DB_COUNT as usize];
        let mut ndiffs = 0;
        for i in 0..DB_COUNT as usize {
            if (*tp).tp_diffbuf[i].is_null() {
                continue;
            }
            if (*dp).df_count[i] > 0 {
                diff_write_buffer(
                    (*tp).tp_diffbuf[i],
                    &raw mut images[ndiffs],
                    (*dp).df_lnum[i],
                    (*dp).df_lnum[i] + (*dp).df_count[i] - 1,
                );
            }
            lengths[ndiffs] = (*dp).df_count[i];
            ndiffs += 1;
        }

        let iwhite = diff_flags.get() & (DIFF_IWHITEALL | DIFF_IWHITE) != 0;
        // `diff_write_buffer` leaves an empty block as a NULL pointer, which
        // `from_raw_parts` may not see; those axes have zero length anyway.
        let mut blocks: [&[u8]; DB_COUNT as usize] = [&[]; DB_COUNT as usize];
        for (block, image) in blocks.iter_mut().zip(&images[..ndiffs]) {
            if !image.ptr.is_null() {
                *block = ::core::slice::from_raw_parts(image.ptr.cast(), image.size as usize);
            }
        }
        let decisions = linematch_nbuffers(&blocks[..ndiffs], &lengths[..ndiffs], iwhite);

        for image in &mut images[..ndiffs] {
            xfree(image.ptr.cast());
            image.ptr = ::core::ptr::null_mut();
        }
        apply_linematch_results(dp, &decisions);
    }
}
