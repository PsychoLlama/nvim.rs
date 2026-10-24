//! Keeping the windows lined up, and moving between changes.
//!
//! [`diff_set_topline`] is what `'scrollbind'` calls in diff mode: given the
//! partner window's topline and topfill, it computes this one's so that the
//! same change is on the same screen row.  [`diff_move_to`] is `]c`/`[c`, and
//! [`diff_get_corresponding_line`] the line-number mapping the two share.
//!
//! The unit these all work in is the *virtual* line: a run of adjacent diff
//! blocks occupies `DiffBlock::max_len` rows on screen whichever buffer is
//! being looked at, because the shorter side is padded with filler.  Lining
//! two windows up means counting virtual lines on one side and spending them
//! on the other.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::types::Failed;
use crate::winlayer::{Buf, TabPage, Win};
use core::ffi::c_int;

/// The first block of the adjacent run containing `topline`, and the first
/// block *after* that run, as indices into `blocks` (the list's length when
/// there is none).
///
/// Blocks are adjacent when one ends exactly where the next begins, and a run
/// of them is one unbroken stretch of screen rows -- the unit both windows
/// have to agree on.
fn find_top_diff_block(
    blocks: &[DiffBlock],
    fromidx: usize,
    topline: LineNr,
) -> (Option<usize>, usize) {
    let mut thistopdiff = None;
    let mut runstart = None;
    let mut start_next_run = true;
    for (at, topdiff) in blocks.iter().enumerate() {
        if runstart.is_none() || start_next_run {
            runstart = Some(at);
            start_next_run = false;
        }
        if topline >= topdiff.df_lnum[fromidx]
            && topline <= topdiff.end(fromidx)
            && thistopdiff.is_none()
        {
            thistopdiff = runstart;
        }
        let next = blocks.get(at + 1);
        if next.is_none_or(|n| n.df_lnum[fromidx] != topdiff.end(fromidx)) {
            start_next_run = true;
            if thistopdiff.is_some() {
                return (thistopdiff, at + 1);
            }
        }
    }
    (thistopdiff, blocks.len())
}

/// Where the other window's top belongs, given this one's: the topfill to use
/// -- `None` when `'diffopt'` has no `filler`, which leaves it alone -- and
/// the topline.
///
/// Two steps: count how many virtual lines of the run `from_topline` sits in
/// have scrolled past, then spend that many on the `toidx` side.  `topfill`
/// is the remainder -- the virtual lines the destination has no real line
/// for.
fn calculate_topfill_and_topline(
    blocks: &[DiffBlock],
    slots: &DiffSlots,
    fromidx: usize,
    toidx: usize,
    from_topline: LineNr,
    from_topfill: c_int,
) -> (Option<c_int>, LineNr) {
    let (thistopdiff, after_run) = find_top_diff_block(blocks, fromidx, from_topline);
    let from_top = thistopdiff.unwrap_or(blocks.len());

    // Whole blocks above the topline, plus the part of the block it sits
    // inside; `from_topfill` rows of that were filler, not text.
    let mut passed = 0;
    let mut at = from_top;
    while let Some(dp) = blocks.get(at)
        && dp.end(fromidx) <= from_topline
    {
        passed += dp.max_len(slots);
        at += 1;
    }
    if let Some(dp) = blocks.get(at)
        && at != after_run
    {
        passed += from_topline - dp.df_lnum[fromidx];
    }
    passed = (passed - from_topfill).max(0);

    // Spend them on the other side, which runs out of real lines first
    // wherever it is the shorter one.
    let mut to_lnum = blocks.get(from_top).map_or(1, |dp| dp.df_lnum[toidx]);
    let mut left = passed;
    let mut at = from_top;
    while left > 0
        && let Some(dp) = blocks.get(at)
        && at != after_run
    {
        to_lnum += left.min(dp.df_count[toidx]);
        left -= left.min(dp.max_len(slots));
        at += 1;
    }

    // How many virtual lines that landing place is worth, which is what
    // the filler has to make up.
    let mut max_virt_lines = 0;
    for dp in &blocks[from_top..] {
        if dp.end(toidx) <= to_lnum {
            max_virt_lines += dp.max_len(slots);
        } else {
            if dp.df_lnum[toidx] <= to_lnum {
                max_virt_lines += to_lnum - dp.df_lnum[toidx];
            }
            break;
        }
    }
    let filler = diff_flags.get() & DIFF_FILLER != 0;
    (filler.then_some(max_virt_lines - passed), to_lnum)
}

/// Set `towin`'s topline and topfill from `fromwin`'s.
///
/// Safe: two [`Win`]s carry the whole of the promise this needs.
pub fn diff_set_topline(fromwin: Win, mut towin: Win) {
    let tp = TabPage::current();
    let frombuf = fromwin.buffer();
    let fromidx = tp.diff_index(frombuf);
    if fromidx == DB_COUNT {
        return;
    }
    if tp.tp_diff_invalid != 0 {
        diff_update(None);
    }
    let fromidx = slot_index(fromidx);
    // Read after the recompute, which runs `DiffUpdated` autocommands.
    //
    let tobuf = towin.buffer();

    let lnum = fromwin.w_topline;
    towin.w_topfill = 0;
    let blocks = &tp.tp_diff;
    match blocks.iter().find(|dp| lnum <= dp.end(fromidx)) {
        // Below every block: the two buffers' tails line up by counting
        // back from the end.
        None => {
            towin.w_topline = tobuf.b_ml.ml_line_count - (frombuf.b_ml.ml_line_count - lnum);
        }
        Some(dp) => {
            let toidx = tp.diff_index(tobuf);
            if toidx == DB_COUNT {
                return;
            }
            let toidx = slot_index(toidx);
            towin.w_topline = lnum + (dp.df_lnum[toidx] - dp.df_lnum[fromidx]);
            if lnum >= dp.df_lnum[fromidx] {
                let from_topfill = fromwin.w_topfill;
                let (topfill, topline) = calculate_topfill_and_topline(
                    blocks,
                    &tp.tp_diffbuf,
                    fromidx,
                    toidx,
                    lnum,
                    from_topfill,
                );
                if let Some(topfill) = topfill {
                    towin.w_topfill = topfill;
                }
                towin.w_topline = topline;
            }
        }
    }

    towin.w_botfill = false;
    if towin.w_topline > tobuf.b_ml.ml_line_count {
        towin.w_topline = tobuf.b_ml.ml_line_count;
        towin.w_botfill = true;
    }
    if towin.w_topline < 1 {
        towin.w_topline = 1;
        towin.w_topfill = 0;
    }
    invalidate_botline_win(towin);
    changed_line_abv_curs_win(towin);
    towin.check_topfill(false);
    // `has_folding` writes the first line of the fold only when there is one.
    if let Some(first) = towin.fold_first(towin.w_topline) {
        towin.w_topline = first;
    }
}

/// `]c` / `[c`: move the cursor to the start of the `count`th next or
/// previous change.
pub fn diff_move_to(dir: c_int, mut count: c_int) -> Result<(), Failed> {
    let tp = TabPage::current();
    let mut lnum = Win::current().w_cursor.lnum;
    let idx = tp.diff_index(Buf::current());
    if idx == DB_COUNT || tp.tp_diff.is_empty() {
        return Err(Failed);
    }
    if tp.tp_diff_invalid != 0 {
        diff_update(None);
    }
    let blocks = &tp.tp_diff;
    let Some(first) = blocks.first() else {
        return Err(Failed);
    };
    let idx = slot_index(idx);

    while count > 0 {
        count -= 1;
        if dir == BACKWARD as c_int && lnum <= first.df_lnum[idx] {
            break;
        }
        // Forwards: the first block starting below the cursor.
        // Backwards: the last block starting at or above it, which is
        // the one whose successor is already at or below the cursor.
        for (at, dp) in blocks.iter().enumerate() {
            let hit = dir == FORWARD as c_int && lnum < dp.df_lnum[idx]
                || dir == BACKWARD as c_int
                    && blocks
                        .get(at + 1)
                        .is_none_or(|next| lnum <= next.df_lnum[idx]);
            if hit {
                lnum = dp.df_lnum[idx];
                break;
            }
        }
    }

    lnum = lnum.min(Buf::current().b_ml.ml_line_count);
    if lnum == Win::current().w_cursor.lnum {
        return Err(Failed);
    }
    setpcmark();
    Win::current().w_cursor.lnum = lnum;
    Win::current().w_cursor.col = 0;
    Ok(())
}

/// The line of the current buffer matching `lnum1` of `buf1`.
///
/// `baseline` accumulates how far the two buffers have drifted apart over the
/// blocks passed so far, which is the answer for any line outside a block.
fn diff_get_corresponding_line_int(buf1: Buf, lnum1: LineNr) -> LineNr {
    let tp = TabPage::current();
    let idx1 = tp.diff_index(buf1);
    let idx2 = tp.diff_index(Buf::current());
    if idx1 == DB_COUNT || idx2 == DB_COUNT || tp.tp_diff.is_empty() {
        return lnum1;
    }
    if tp.tp_diff_invalid != 0 {
        diff_update(None);
    }
    if tp.tp_diff.is_empty() {
        return lnum1;
    }
    let (idx1, idx2) = (slot_index(idx1), slot_index(idx2));
    let cursor = Win::current().w_cursor.lnum;

    let mut baseline = 0;
    for dp in &tp.tp_diff {
        if dp.df_lnum[idx1] > lnum1 {
            return lnum1 - baseline;
        }
        if dp.end(idx1) > lnum1 {
            // Inside a block: the same offset into it, clamped to what
            // the other side actually has.
            let off = (lnum1 - dp.df_lnum[idx1]).min(dp.df_count[idx2]);
            return dp.df_lnum[idx2] + off;
        }
        // A deletion right here, with the cursor inside the lines that
        // were deleted: stay where the cursor is.
        if dp.df_lnum[idx1] == lnum1
            && dp.df_count[idx1] == 0
            && dp.df_lnum[idx2] <= cursor
            && dp.end(idx2) > cursor
        {
            return cursor;
        }
        baseline = dp.end(idx1) - dp.end(idx2);
    }
    lnum1 - baseline
}

/// [`diff_get_corresponding_line_int`], clamped to the buffer.
///
/// Safe: a [`Buf`] carries the whole of the promise this needs.
pub fn diff_get_corresponding_line(buf1: Buf, lnum1: LineNr) -> LineNr {
    diff_get_corresponding_line_int(buf1, lnum1).min(Buf::current().b_ml.ml_line_count)
}

/// The line of `window`'s buffer matching `lnum` of the current one.
///
/// Unlike [`diff_get_corresponding_line`] this clamps to the *block*, not the
/// buffer: it is used to decide what a window shows beside a given line, and
/// running past the block would point at the next change.
///
/// Safe: a [`Win`] carries the whole of the promise this needs.
pub fn diff_lnum_win(lnum: LineNr, window: Win) -> LineNr {
    let tp = TabPage::current();
    let idx = tp.diff_index(Buf::current());
    if idx == DB_COUNT {
        return 0;
    }
    if tp.tp_diff_invalid != 0 {
        diff_update(None);
    }
    let idx = slot_index(idx);

    let buf = window.buffer();
    let Some(dp) = tp.tp_diff.iter().find(|dp| lnum <= dp.end(idx)) else {
        return buf.b_ml.ml_line_count - (Buf::current().b_ml.ml_line_count - lnum);
    };
    let i = tp.diff_index(buf);
    if i == DB_COUNT {
        return 0;
    }
    let i = slot_index(i);
    (lnum + (dp.df_lnum[i] - dp.df_lnum[idx])).min(dp.end(i))
}
