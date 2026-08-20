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
use core::ffi::c_int;

/// The first block of the adjacent run containing `topline`, and the first
/// block *after* that run.
///
/// Blocks are adjacent when one ends exactly where the next begins, and a run
/// of them is one unbroken stretch of screen rows -- the unit both windows
/// have to agree on.
unsafe fn find_top_diff_block(
    thistopdiff: *mut *mut diff_T,
    next_adjacent_blocks: *mut *mut diff_T,
    fromidx: usize,
    topline: linenr_T,
) {
    unsafe {
        let mut runstart = ::core::ptr::null_mut::<diff_T>();
        let mut start_next_run = true;
        let mut topdiff = (*curtab.get()).tp_first_diff;
        while !topdiff.is_null() {
            if runstart.is_null() || start_next_run {
                runstart = topdiff;
                start_next_run = false;
            }
            if topline >= (*topdiff).df_lnum[fromidx]
                && topline <= (*topdiff).df_lnum[fromidx] + (*topdiff).df_count[fromidx]
                && (*thistopdiff).is_null()
            {
                *thistopdiff = runstart;
            }
            let next = (*topdiff).df_next;
            if next.is_null()
                || (*next).df_lnum[fromidx]
                    != (*topdiff).df_lnum[fromidx] + (*topdiff).df_count[fromidx]
            {
                start_next_run = true;
                if !(*thistopdiff).is_null() {
                    *next_adjacent_blocks = next;
                    break;
                }
            }
            topdiff = next;
        }
    }
}

/// Where the other window's top belongs, given this one's.
///
/// Two steps: count how many virtual lines of the run `from_topline` sits in
/// have scrolled past, then spend that many on the `toidx` side.  `topfill`
/// is the remainder -- the virtual lines the destination has no real line
/// for.
unsafe fn calculate_topfill_and_topline(
    fromidx: usize,
    toidx: usize,
    from_topline: linenr_T,
    from_topfill: c_int,
    topfill: *mut c_int,
    topline: *mut linenr_T,
) {
    unsafe {
        let mut thistopdiff = ::core::ptr::null_mut::<diff_T>();
        let mut next_adjacent_blocks = ::core::ptr::null_mut::<diff_T>();
        find_top_diff_block(
            &raw mut thistopdiff,
            &raw mut next_adjacent_blocks,
            fromidx,
            from_topline,
        );

        // Whole blocks above the topline, plus the part of the block it sits
        // inside; `from_topfill` rows of that were filler, not text.
        let mut passed = 0;
        let mut curdif = thistopdiff;
        while !curdif.is_null()
            && (*curdif).df_lnum[fromidx] + (*curdif).df_count[fromidx] <= from_topline
        {
            passed += get_max_diff_length(curdif);
            curdif = (*curdif).df_next;
        }
        if curdif != next_adjacent_blocks {
            passed += from_topline - (*curdif).df_lnum[fromidx];
        }
        passed = (passed - from_topfill).max(0);

        // Spend them on the other side, which runs out of real lines first
        // wherever it is the shorter one.
        let mut to_lnum = if thistopdiff.is_null() {
            1
        } else {
            (*thistopdiff).df_lnum[toidx]
        };
        let mut left = passed;
        curdif = thistopdiff;
        while left > 0 && !curdif.is_null() && curdif != next_adjacent_blocks {
            to_lnum += left.min((*curdif).df_count[toidx]);
            left -= left.min(get_max_diff_length(curdif));
            curdif = (*curdif).df_next;
        }

        // How many virtual lines that landing place is worth, which is what
        // the filler has to make up.
        let mut max_virt_lines = 0;
        let mut dp = thistopdiff;
        while !dp.is_null() {
            if (*dp).df_lnum[toidx] + (*dp).df_count[toidx] <= to_lnum {
                max_virt_lines += get_max_diff_length(dp);
                dp = (*dp).df_next;
            } else {
                if (*dp).df_lnum[toidx] <= to_lnum {
                    max_virt_lines += to_lnum - (*dp).df_lnum[toidx];
                }
                break;
            }
        }
        if diff_flags.get() & DIFF_FILLER != 0 {
            *topfill = max_virt_lines - passed;
        }
        *topline = to_lnum;
    }
}

/// Set `towin`'s topline and topfill from `fromwin`'s.
pub unsafe fn diff_set_topline(fromwin: *mut win_T, towin: *mut win_T) {
    unsafe {
        let tp = curtab.get();
        let frombuf = (*fromwin).w_buffer;
        let fromidx = diff_buf_idx(frombuf, tp);
        if fromidx == DB_COUNT {
            return;
        }
        if (*tp).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut());
        }
        let fromidx = fromidx as usize;

        let lnum = (*fromwin).w_topline;
        (*towin).w_topfill = 0;
        let mut dp = (*tp).tp_first_diff;
        while !dp.is_null() && lnum > (*dp).df_lnum[fromidx] + (*dp).df_count[fromidx] {
            dp = (*dp).df_next;
        }
        if dp.is_null() {
            // Below every block: the two buffers' tails line up by counting
            // back from the end.
            (*towin).w_topline =
                (*(*towin).w_buffer).b_ml.ml_line_count - ((*frombuf).b_ml.ml_line_count - lnum);
        } else {
            let toidx = diff_buf_idx((*towin).w_buffer, tp);
            if toidx == DB_COUNT {
                return;
            }
            let toidx = toidx as usize;
            (*towin).w_topline = lnum + ((*dp).df_lnum[toidx] - (*dp).df_lnum[fromidx]);
            if lnum >= (*dp).df_lnum[fromidx] {
                calculate_topfill_and_topline(
                    fromidx,
                    toidx,
                    (*fromwin).w_topline,
                    (*fromwin).w_topfill,
                    &raw mut (*towin).w_topfill,
                    &raw mut (*towin).w_topline,
                );
            }
        }

        (*towin).w_botfill = false;
        if (*towin).w_topline > (*(*towin).w_buffer).b_ml.ml_line_count {
            (*towin).w_topline = (*(*towin).w_buffer).b_ml.ml_line_count;
            (*towin).w_botfill = true;
        }
        if (*towin).w_topline < 1 {
            (*towin).w_topline = 1;
            (*towin).w_topfill = 0;
        }
        invalidate_botline_win(towin);
        changed_line_abv_curs_win(towin);
        check_topfill(towin, false);
        has_folding(
            towin,
            (*towin).w_topline,
            &raw mut (*towin).w_topline,
            ::core::ptr::null_mut(),
        );
    }
}

/// `]c` / `[c`: move the cursor to the start of the `count`th next or
/// previous change.
pub unsafe fn diff_move_to(dir: c_int, mut count: c_int) -> c_int {
    unsafe {
        let tp = curtab.get();
        let mut lnum = (*curwin.get()).w_cursor.lnum;
        let idx = diff_buf_idx(curbuf.get(), tp);
        if idx == DB_COUNT || (*tp).tp_first_diff.is_null() {
            return FAIL;
        }
        if (*tp).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut());
        }
        if (*tp).tp_first_diff.is_null() {
            return FAIL;
        }
        let idx = idx as usize;

        while count > 0 {
            count -= 1;
            if dir == BACKWARD as c_int && lnum <= (*(*tp).tp_first_diff).df_lnum[idx] {
                break;
            }
            let mut dp = (*tp).tp_first_diff;
            while !dp.is_null() {
                // Forwards: the first block starting below the cursor.
                // Backwards: the last block starting at or above it, which is
                // the one whose successor is already at or below the cursor.
                if dir == FORWARD as c_int && lnum < (*dp).df_lnum[idx]
                    || dir == BACKWARD as c_int
                        && ((*dp).df_next.is_null() || lnum <= (*(*dp).df_next).df_lnum[idx])
                {
                    lnum = (*dp).df_lnum[idx];
                    break;
                }
                dp = (*dp).df_next;
            }
        }

        lnum = lnum.min((*curbuf.get()).b_ml.ml_line_count);
        if lnum == (*curwin.get()).w_cursor.lnum {
            return FAIL;
        }
        setpcmark();
        (*curwin.get()).w_cursor.lnum = lnum;
        (*curwin.get()).w_cursor.col = 0;
        OK
    }
}

/// The line of the current buffer matching `lnum1` of `buf1`.
///
/// `baseline` accumulates how far the two buffers have drifted apart over the
/// blocks passed so far, which is the answer for any line outside a block.
unsafe fn diff_get_corresponding_line_int(buf1: *mut buf_T, lnum1: linenr_T) -> linenr_T {
    unsafe {
        let tp = curtab.get();
        let idx1 = diff_buf_idx(buf1, tp);
        let idx2 = diff_buf_idx(curbuf.get(), tp);
        if idx1 == DB_COUNT || idx2 == DB_COUNT || (*tp).tp_first_diff.is_null() {
            return lnum1;
        }
        if (*tp).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut());
        }
        if (*tp).tp_first_diff.is_null() {
            return lnum1;
        }
        let (idx1, idx2) = (idx1 as usize, idx2 as usize);

        let mut baseline = 0;
        let mut dp = (*tp).tp_first_diff;
        while !dp.is_null() {
            if (*dp).df_lnum[idx1] > lnum1 {
                return lnum1 - baseline;
            }
            if (*dp).df_lnum[idx1] + (*dp).df_count[idx1] > lnum1 {
                // Inside a block: the same offset into it, clamped to what
                // the other side actually has.
                let off = (lnum1 - (*dp).df_lnum[idx1]).min((*dp).df_count[idx2]);
                return (*dp).df_lnum[idx2] + off;
            }
            // A deletion right here, with the cursor inside the lines that
            // were deleted: stay where the cursor is.
            if (*dp).df_lnum[idx1] == lnum1
                && (*dp).df_count[idx1] == 0
                && (*dp).df_lnum[idx2] <= (*curwin.get()).w_cursor.lnum
                && (*dp).df_lnum[idx2] + (*dp).df_count[idx2] > (*curwin.get()).w_cursor.lnum
            {
                return (*curwin.get()).w_cursor.lnum;
            }
            baseline = (*dp).df_lnum[idx1] + (*dp).df_count[idx1]
                - ((*dp).df_lnum[idx2] + (*dp).df_count[idx2]);
            dp = (*dp).df_next;
        }
        lnum1 - baseline
    }
}

/// [`diff_get_corresponding_line_int`], clamped to the buffer.
pub unsafe fn diff_get_corresponding_line(buf1: *mut buf_T, lnum1: linenr_T) -> linenr_T {
    unsafe { diff_get_corresponding_line_int(buf1, lnum1).min((*curbuf.get()).b_ml.ml_line_count) }
}

/// The line of `wp`'s buffer matching `lnum` of the current one.
///
/// Unlike [`diff_get_corresponding_line`] this clamps to the *block*, not the
/// buffer: it is used to decide what a window shows beside a given line, and
/// running past the block would point at the next change.
pub unsafe fn diff_lnum_win(lnum: linenr_T, wp: *mut win_T) -> linenr_T {
    unsafe {
        let tp = curtab.get();
        let idx = diff_buf_idx(curbuf.get(), tp);
        if idx == DB_COUNT {
            return 0;
        }
        if (*tp).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut());
        }
        let idx = idx as usize;

        let mut dp = (*tp).tp_first_diff;
        while !dp.is_null() && lnum > (*dp).df_lnum[idx] + (*dp).df_count[idx] {
            dp = (*dp).df_next;
        }
        if dp.is_null() {
            return (*(*wp).w_buffer).b_ml.ml_line_count
                - ((*curbuf.get()).b_ml.ml_line_count - lnum);
        }
        let i = diff_buf_idx((*wp).w_buffer, tp);
        if i == DB_COUNT {
            return 0;
        }
        let i = i as usize;
        (lnum + ((*dp).df_lnum[i] - (*dp).df_lnum[idx])).min((*dp).df_lnum[i] + (*dp).df_count[i])
    }
}
