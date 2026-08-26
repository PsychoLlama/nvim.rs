//! Keeping the windows lined up, and moving between changes.
//!
//! [`diff_set_topline`] is what `'scrollbind'` calls in diff mode: given the
//! partner window's topline and topfill, it computes this one's so that the
//! same change is on the same screen row.  [`diff_move_to`] is `]c`/`[c`, and
//! [`diff_get_corresponding_line`] the line-number mapping the two share.
//!
//! The unit these all work in is the *virtual* line: a run of adjacent diff
//! blocks occupies `get_max_diff_length` rows on screen whichever buffer is
//! being looked at, because the shorter side is padded with filler.  Lining
//! two windows up means counting virtual lines on one side and spending them
//! on the other.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{FAIL, OK};
use crate::winlayer::{Buf, TabPage, Win};
use core::ffi::c_int;

/// The first block of the adjacent run containing `topline`, and the first
/// block *after* that run.
///
/// Blocks are adjacent when one ends exactly where the next begins, and a run
/// of them is one unbroken stretch of screen rows -- the unit both windows
/// have to agree on.
fn find_top_diff_block(fromidx: usize, topline: linenr_T) -> (Option<Df>, Option<Df>) {
    let mut thistopdiff = None;
    let mut runstart = None;
    let mut start_next_run = true;
    let mut cur = Df::first(cur_tab());
    while let Some(topdiff) = cur {
        if runstart.is_none() || start_next_run {
            runstart = Some(topdiff);
            start_next_run = false;
        }
        if topline >= topdiff.df_lnum[fromidx]
            && topline <= topdiff.end(fromidx)
            && thistopdiff.is_none()
        {
            thistopdiff = runstart;
        }
        let next = topdiff.next();
        if next.is_none_or(|n| n.df_lnum[fromidx] != topdiff.end(fromidx)) {
            start_next_run = true;
            if thistopdiff.is_some() {
                return (thistopdiff, next);
            }
        }
        cur = next;
    }
    (thistopdiff, None)
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
    fromidx: usize,
    toidx: usize,
    from_topline: linenr_T,
    from_topfill: c_int,
) -> (Option<c_int>, linenr_T) {
    let (thistopdiff, next_adjacent_blocks) = find_top_diff_block(fromidx, from_topline);
    // The run ends here; `Df` has no equality of its own, so the comparison
    // the walks below make is against the raw pointer.
    let after_run = next_adjacent_blocks.map_or(::core::ptr::null_mut(), Df::raw);

    // Whole blocks above the topline, plus the part of the block it sits
    // inside; `from_topfill` rows of that were filler, not text.
    let mut passed = 0;
    let mut curdif = thistopdiff;
    while let Some(dp) = curdif
        && dp.end(fromidx) <= from_topline
    {
        passed += dp.max_len();
        curdif = dp.next();
    }
    if let Some(dp) = curdif
        && dp.raw() != after_run
    {
        passed += from_topline - dp.df_lnum[fromidx];
    }
    passed = (passed - from_topfill).max(0);

    // Spend them on the other side, which runs out of real lines first
    // wherever it is the shorter one.
    let mut to_lnum = thistopdiff.map_or(1, |dp| dp.df_lnum[toidx]);
    let mut left = passed;
    let mut curdif = thistopdiff;
    while left > 0
        && let Some(dp) = curdif
        && dp.raw() != after_run
    {
        to_lnum += left.min(dp.df_count[toidx]);
        left -= left.min(dp.max_len());
        curdif = dp.next();
    }

    // How many virtual lines that landing place is worth, which is what
    // the filler has to make up.
    let mut max_virt_lines = 0;
    let mut cur = thistopdiff;
    while let Some(dp) = cur {
        if dp.end(toidx) <= to_lnum {
            max_virt_lines += dp.max_len();
            cur = dp.next();
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
/// # Safety
/// Both windows must be live.
pub unsafe fn diff_set_topline(fromwin: *mut win_T, towin: *mut win_T) {
    // SAFETY: the caller's windows.
    let fromwin = unsafe { Win::new(fromwin) };
    // SAFETY: as above.
    let mut towin = unsafe { Win::new(towin) };
    let tp = cur_tab();
    // SAFETY: a live window's buffer is live.
    let frombuf = unsafe { Buf::new(fromwin.w_buffer) };
    let fromidx = diff_slot(frombuf.raw(), tp);
    if fromidx == DB_COUNT {
        return;
    }
    if tp.tp_diff_invalid != 0 {
        // SAFETY: the editor exists.
        unsafe { ex_diffupdate(::core::ptr::null_mut()) };
    }
    let fromidx = fromidx as usize;
    // Read after the recompute, which runs `DiffUpdated` autocommands.
    //
    // SAFETY: a live window's buffer is live.
    let tobuf = unsafe { Buf::new(towin.w_buffer) };

    let lnum = fromwin.w_topline;
    towin.w_topfill = 0;
    match diff_blocks(tp).find(|dp| lnum <= dp.end(fromidx)) {
        // Below every block: the two buffers' tails line up by counting
        // back from the end.
        None => {
            towin.w_topline = tobuf.b_ml.ml_line_count - (frombuf.b_ml.ml_line_count - lnum);
        }
        Some(dp) => {
            let toidx = diff_slot(tobuf.raw(), tp);
            if toidx == DB_COUNT {
                return;
            }
            let toidx = toidx as usize;
            towin.w_topline = lnum + (dp.df_lnum[toidx] - dp.df_lnum[fromidx]);
            if lnum >= dp.df_lnum[fromidx] {
                let from_topfill = fromwin.w_topfill;
                let (topfill, topline) =
                    calculate_topfill_and_topline(fromidx, toidx, lnum, from_topfill);
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
    // SAFETY: a live window, in both calls.
    unsafe {
        invalidate_botline_win(towin.raw());
        changed_line_abv_curs_win(towin.raw());
    }
    towin.check_topfill(false);
    // `has_folding` writes the first line of the fold only when there is one.
    if let Some(first) = towin.fold_first(towin.w_topline) {
        towin.w_topline = first;
    }
}

/// `]c` / `[c`: move the cursor to the start of the `count`th next or
/// previous change.
pub unsafe fn diff_move_to(dir: c_int, mut count: c_int) -> c_int {
    let tp = cur_tab();
    let mut lnum = cur_win().w_cursor.lnum;
    let idx = diff_slot(curbuf.get(), tp);
    if idx == DB_COUNT || tp.tp_first_diff.is_null() {
        return FAIL;
    }
    if tp.tp_diff_invalid != 0 {
        // SAFETY: the editor exists.
        unsafe { ex_diffupdate(::core::ptr::null_mut()) };
    }
    let Some(first) = Df::first(tp) else {
        return FAIL;
    };
    let idx = idx as usize;

    while count > 0 {
        count -= 1;
        if dir == BACKWARD as c_int && lnum <= first.df_lnum[idx] {
            break;
        }
        // Forwards: the first block starting below the cursor.
        // Backwards: the last block starting at or above it, which is
        // the one whose successor is already at or below the cursor.
        for dp in diff_blocks(tp) {
            let hit = dir == FORWARD as c_int && lnum < dp.df_lnum[idx]
                || dir == BACKWARD as c_int
                    && dp.next().is_none_or(|next| lnum <= next.df_lnum[idx]);
            if hit {
                lnum = dp.df_lnum[idx];
                break;
            }
        }
    }

    lnum = lnum.min(cur_buf().b_ml.ml_line_count);
    if lnum == cur_win().w_cursor.lnum {
        return FAIL;
    }
    // SAFETY: the editor exists.
    unsafe { setpcmark() };
    cur_win().w_cursor.lnum = lnum;
    cur_win().w_cursor.col = 0;
    OK
}

/// The line of the current buffer matching `lnum1` of `buf1`.
///
/// `baseline` accumulates how far the two buffers have drifted apart over the
/// blocks passed so far, which is the answer for any line outside a block.
fn diff_get_corresponding_line_int(buf1: *mut buf_T, lnum1: linenr_T) -> linenr_T {
    let tp = cur_tab();
    let idx1 = diff_slot(buf1, tp);
    let idx2 = diff_slot(curbuf.get(), tp);
    if idx1 == DB_COUNT || idx2 == DB_COUNT || tp.tp_first_diff.is_null() {
        return lnum1;
    }
    if tp.tp_diff_invalid != 0 {
        // SAFETY: the editor exists.
        unsafe { ex_diffupdate(::core::ptr::null_mut()) };
    }
    if tp.tp_first_diff.is_null() {
        return lnum1;
    }
    let (idx1, idx2) = (idx1 as usize, idx2 as usize);

    let mut baseline = 0;
    for dp in diff_blocks(tp) {
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
            && dp.df_lnum[idx2] <= cur_win().w_cursor.lnum
            && dp.end(idx2) > cur_win().w_cursor.lnum
        {
            return cur_win().w_cursor.lnum;
        }
        baseline = dp.end(idx1) - dp.end(idx2);
    }
    lnum1 - baseline
}

/// [`diff_get_corresponding_line_int`], clamped to the buffer.
pub unsafe fn diff_get_corresponding_line(buf1: *mut buf_T, lnum1: linenr_T) -> linenr_T {
    diff_get_corresponding_line_int(buf1, lnum1).min(cur_buf().b_ml.ml_line_count)
}

/// The line of `wp`'s buffer matching `lnum` of the current one.
///
/// Unlike [`diff_get_corresponding_line`] this clamps to the *block*, not the
/// buffer: it is used to decide what a window shows beside a given line, and
/// running past the block would point at the next change.
///
/// # Safety
/// `wp` must be a live window.
pub unsafe fn diff_lnum_win(lnum: linenr_T, wp: *mut win_T) -> linenr_T {
    // SAFETY: the caller's window.
    let wp = unsafe { Win::new(wp) };
    let tp = cur_tab();
    let idx = diff_slot(curbuf.get(), tp);
    if idx == DB_COUNT {
        return 0;
    }
    if tp.tp_diff_invalid != 0 {
        // SAFETY: the editor exists.
        unsafe { ex_diffupdate(::core::ptr::null_mut()) };
    }
    let idx = idx as usize;

    // SAFETY: a live window's buffer is live.
    let buf = unsafe { Buf::new(wp.w_buffer) };
    let Some(dp) = diff_blocks(tp).find(|dp| lnum <= dp.end(idx)) else {
        return buf.b_ml.ml_line_count - (cur_buf().b_ml.ml_line_count - lnum);
    };
    let i = diff_slot(buf.raw(), tp);
    if i == DB_COUNT {
        return 0;
    }
    let i = i as usize;
    (lnum + (dp.df_lnum[i] - dp.df_lnum[idx])).min(dp.end(i))
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The tab page the editor is working in.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}
