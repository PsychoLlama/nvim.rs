//! The diff block list, and which buffers are in it.
//!
//! A tabpage owns a linked list of `diff_T` blocks, each naming a line range
//! in every one of the (up to eight) buffers `tp_diffbuf` holds.  This file
//! owns both halves: [`diff_buf_add`]/[`diff_buf_delete`]/[`diff_buf_idx`]
//! are the registry, and [`diff_alloc_new`]/[`diff_free`]/
//! [`diff_check_sanity`] the list.
//!
//! [`diff_mark_adjust_tp`] is the one that keeps the list correct across an
//! edit without recomputing it -- and it is only ever *read* under the
//! external diff, because `diff_internal()` makes the tabpage invalid
//! instead.  It still runs, because `:%diffput` needs the marks.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{FAIL, OK};
use crate::winlayer::{Buf, TabPage, Win, tabs, windows};
use core::ffi::c_int;
use std::ffi::CStr;

/// Free one block, its cached inline changes included.
pub(crate) unsafe fn clear_diffblock(dp: *mut diff_T) {
    unsafe { ga_clear(&raw mut (*dp).df_changes) };
    unsafe { xfree(dp.cast()) };
}

/// Take `buf` out of every tabpage's diff.
pub fn diff_buf_delete(buf: Buf) {
    for mut tp in tabs() {
        let i = diff_buf_idx(buf, tp);
        if i != DB_COUNT {
            tp.tp_diffbuf[i as usize] = ::core::ptr::null_mut();
            tp.tp_diff_invalid = 1;
            if tp.is_current() {
                need_diff_redraw.set(true);
                // SAFETY: `curwin` is set from startup to exit.
                unsafe { redraw_later(curwin.get(), UPD_VALID) };
            }
        }
    }
}

/// Add or remove `win`'s buffer from the current tabpage's diff, following
/// the window's `'diff'`.
///
/// A buffer stays in the diff while *any* window still shows it in diff mode.
pub fn diff_buf_adjust(win: Win) {
    if win.w_onebuf_opt.wo_diff != 0 {
        diff_buf_add(win.buffer());
        return;
    }
    if windows().any(|wp| wp.w_buffer == win.w_buffer && wp.w_onebuf_opt.wo_diff != 0) {
        return;
    }
    let mut tp = cur_tab();
    let i = diff_buf_idx(win.buffer(), tp);
    if i != DB_COUNT {
        tp.tp_diffbuf[i as usize] = ::core::ptr::null_mut();
        tp.tp_diff_invalid = 1;
        // SAFETY: nothing here holds a borrow of the tab page's block list.
        unsafe { diff_redraw(true) };
    }
}

/// Put `buf` in the current tabpage's diff, if there is a slot free.
pub fn diff_buf_add(buf: Buf) {
    let mut tp = cur_tab();
    if diff_buf_idx(buf, tp) != DB_COUNT {
        return;
    }
    for i in 0..DB_COUNT as usize {
        if tp.tp_diffbuf[i].is_null() {
            tp.tp_diffbuf[i] = buf.raw();
            tp.tp_diff_invalid = 1;
            // SAFETY: nothing here holds a borrow of the tab page's block list.
            unsafe { diff_redraw(true) };
            return;
        }
    }
    // SAFETY: the message macros expand to a `vim_snprintf` over the format
    // literal above and the editor's message buffers.
    unsafe {
        semsg_c!(
            gettext(c"E96: Cannot diff more than %d buffers".as_ptr()),
            DB_COUNT,
        )
    };
}

/// Empty the current tabpage's diff.
pub(crate) fn diff_buf_clear() {
    let mut tp = cur_tab();
    for i in 0..DB_COUNT as usize {
        if !tp.tp_diffbuf[i].is_null() {
            tp.tp_diffbuf[i] = ::core::ptr::null_mut();
            tp.tp_diff_invalid = 1;
            // SAFETY: nothing here holds a borrow of the tab page's block list.
            unsafe { diff_redraw(true) };
        }
    }
}

/// `buf`'s slot in `tp`'s diff, or `DB_COUNT` if it has none.
pub(crate) fn diff_buf_idx(buf: Buf, tp: TabPage) -> c_int {
    (0..DB_COUNT)
        .find(|&i| tp.tp_diffbuf[i as usize] == buf.raw())
        .unwrap_or(DB_COUNT)
}

/// Mark every tabpage `buf` is diffed in as needing a recompute.
pub fn diff_invalidate(buf: Buf) {
    for mut tp in tabs() {
        if diff_buf_idx(buf, tp) != DB_COUNT {
            tp.tp_diff_invalid = 1;
            if tp.is_current() {
                // SAFETY: nothing here holds a borrow of the block list.
                unsafe { diff_redraw(true) };
            }
        }
    }
}

/// Adjust every tabpage's block list for an edit in `buf`.
///
/// The parameters are `mark_adjust`'s: lines `line1`..`line2` moved by
/// `amount`, everything below by `amount_after`.
pub fn diff_mark_adjust(
    buf: Buf,
    line1: linenr_T,
    line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
) {
    for tp in tabs() {
        let idx = diff_buf_idx(buf, tp);
        if idx != DB_COUNT {
            // SAFETY: `idx` is a slot the tab page holds, and the block list
            // walked below is that tab page's own.
            unsafe { diff_mark_adjust_tp(tp, idx, line1, line2, amount, amount_after) };
        }
    }
}

/// The edit `mark_adjust`'s four numbers describe, as the two counts every
/// case below is written in terms of.
fn inserted_deleted(
    line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
) -> (linenr_T, linenr_T) {
    if line2 == MAXLNUM as linenr_T {
        (amount, 0) // `mark_adjust(99, MAXLNUM, 9, 0)`: insert lines
    } else if amount_after > 0 {
        (amount_after, 0) // `mark_adjust(99, 98, MAXLNUM, 9)`: a change that inserts
    } else {
        (0, -amount_after) // `mark_adjust(98, 99, MAXLNUM, -2)`: delete lines
    }
}

/// Keep one tabpage's block list correct across an edit, without recomputing.
///
/// The walk is a merge of the edit into the list: an edit that touches no
/// block becomes a new one, an edit that overlaps a block resizes it, and an
/// edit below every block only shifts line numbers.  Upstream numbers the six
/// cases in a diagram; the numbers are kept in the comments below because the
/// arms are otherwise indistinguishable.
unsafe fn diff_mark_adjust_tp(
    mut tp: TabPage,
    idx: c_int,
    line1: linenr_T,
    line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
) {
    if unsafe { diff_internal() } != 0 {
        // The blocks will be recomputed before the next redraw, so
        // nothing below survives; `_update` also gets the folds redone.
        // The *marks* are still adjusted here, which `:%diffput` needs.
        tp.tp_diff_invalid = 1;
        tp.tp_diff_update = 1;
    }
    let idx = idx as usize;
    let (inserted, mut deleted) = inserted_deleted(line2, amount, amount_after);

    // Both of these are closures rather than functions because each has
    // exactly one call site and neither needs a block of its own: a
    // closure written inside an `unsafe` block inherits it.

    // Slide the *other* buffers' ranges by the same edit: `off` is how
    // far the block's start moved up, `n` how many lines they gain --
    // which is how a deletion in one buffer becomes a change in the rest.
    let adjust_others = |dp: *mut diff_T, off: linenr_T, n: linenr_T| {
        for i in 0..DB_COUNT as usize {
            if tp.tp_diffbuf[i].is_null() || i == idx {
                continue;
            }
            unsafe { (*dp).df_lnum[i] = ((*dp).df_lnum[i] - off).max(1) };
            unsafe { (*dp).df_count[i] += n };
        }
    };

    // Fold `dp` into `dprev` if they now touch, else step past it.
    let merge_or_advance = |dprev: *mut diff_T, dp: *mut diff_T| {
        if !dprev.is_null()
            && !unsafe { (*dp).is_linematched }
            && !diff_busy.get()
            && unsafe { (*dprev).df_lnum[idx] } + unsafe { (*dprev).df_count[idx] }
                == unsafe { (*dp).df_lnum[idx] }
        {
            for i in 0..DB_COUNT as usize {
                if !tp.tp_diffbuf[i].is_null() {
                    unsafe { (*dprev).df_count[i] += (*dp).df_count[i] };
                }
            }
            (dprev, unsafe { diff_free(tp, dprev, dp) })
        } else {
            (dp, unsafe { (*dp).df_next })
        }
    };

    let mut dprev = ::core::ptr::null_mut::<diff_T>();
    let mut dp = tp.tp_first_diff;
    let mut lnum_deleted = line1; // lnum of the remaining deletion
    loop {
        // The edit falls between two blocks, touching neither: it is a
        // change of its own. Not while `ex_diffgetput` is walking the
        // list, which is doing its own bookkeeping.
        if (dp.is_null()
            || unsafe { (*dp).df_lnum[idx] } - 1 > line2
            || line2 == MAXLNUM as linenr_T && unsafe { (*dp).df_lnum[idx] } > line1)
            && (dprev.is_null()
                || unsafe { (*dprev).df_lnum[idx] } + unsafe { (*dprev).df_count[idx] } < line1)
            && !diff_busy.get()
        {
            let dnext = unsafe { diff_alloc_new(tp, dprev, dp) };
            unsafe { (*dnext).df_lnum[idx] = line1 };
            unsafe { (*dnext).df_count[idx] = inserted };
            for i in 0..DB_COUNT as usize {
                if tp.tp_diffbuf[i].is_null() || i == idx {
                    continue;
                }
                // The other buffers' line numbers carry the drift the
                // previous block left behind.
                unsafe {
                    (*dnext).df_lnum[i] = if dprev.is_null() {
                        line1
                    } else {
                        line1 + ((*dprev).df_lnum[i] + (*dprev).df_count[i])
                            - ((*dprev).df_lnum[idx] + (*dprev).df_count[idx])
                    }
                };
                unsafe { (*dnext).df_count[i] = deleted };
            }
        }
        if dp.is_null() {
            break;
        }

        let last = unsafe { (*dp).df_lnum[idx] } + unsafe { (*dp).df_count[idx] } - 1;
        // 1. The block is entirely above the edit: nothing to do.
        if last >= line1 - 1 {
            if diff_busy.get() {
                // Mid-update: only the line numbers may move.
                if unsafe { (*dp).df_lnum[idx] } > line2 {
                    unsafe { (*dp).df_lnum[idx] += amount_after };
                }
                dprev = dp;
                dp = unsafe { (*dp).df_next };
                continue;
            }
            // 6. The block is below the edit: shift it. The `!= 0` test
            // covers a deletion that emptied everything between two
            // blocks, leaving nothing to merge.
            if unsafe { (*dp).df_lnum[idx] } - c_int::from(deleted + inserted != 0) > line2 {
                if amount_after == 0 {
                    break; // nothing left to change
                }
                unsafe { (*dp).df_lnum[idx] += amount_after };
            } else {
                // The trim runs *after* the other buffers are adjusted,
                // because it compares the block's lines across all of
                // them.
                let mut check_unchanged = false;
                if deleted > 0 {
                    // 2. 3. 4. 5.: the deletion overlaps this block.
                    let mut off = 0;
                    let n;
                    let next = unsafe { (*dp).df_next };
                    // Does the deletion run on into the next block? Then
                    // only the lines up to its first are this block's.
                    let spills = !next.is_null() && unsafe { (*next).df_lnum[idx] } - 1 <= line2;
                    if unsafe { (*dp).df_lnum[idx] } >= line1 {
                        if last <= line2 {
                            // 4. every line of the block goes.
                            if spills {
                                n = unsafe { (*next).df_lnum[idx] }
                                    - lnum_deleted
                                    - unsafe { (*dp).df_count[idx] };
                                deleted -= unsafe { (*next).df_lnum[idx] } - lnum_deleted;
                                lnum_deleted = unsafe { (*next).df_lnum[idx] };
                            } else {
                                n = deleted - unsafe { (*dp).df_count[idx] };
                            }
                            unsafe { (*dp).df_count[idx] = 0 };
                        } else {
                            // 5. lines go at or just before its top.
                            off = unsafe { (*dp).df_lnum[idx] } - lnum_deleted;
                            n = off;
                            unsafe { (*dp).df_count[idx] -= line2 - (*dp).df_lnum[idx] + 1 };
                            check_unchanged = true;
                        }
                        unsafe { (*dp).df_lnum[idx] = line1 };
                    } else if last < line2 {
                        // 2. lines go at the end of the block.
                        unsafe { (*dp).df_count[idx] -= last - lnum_deleted + 1 };
                        if spills {
                            n = unsafe { (*next).df_lnum[idx] } - 1 - last;
                            deleted -= unsafe { (*next).df_lnum[idx] } - lnum_deleted;
                            lnum_deleted = unsafe { (*next).df_lnum[idx] };
                        } else {
                            n = line2 - last;
                        }
                        check_unchanged = true;
                    } else {
                        // 3. lines go from inside the block.
                        n = 0;
                        unsafe { (*dp).df_count[idx] -= deleted };
                    }
                    adjust_others(dp, off, n);
                } else if unsafe { (*dp).df_lnum[idx] } <= line1 {
                    // Lines inserted inside this block.
                    unsafe { (*dp).df_count[idx] += inserted };
                    check_unchanged = true;
                } else {
                    // Lines inserted above it.
                    unsafe { (*dp).df_lnum[idx] += inserted };
                }
                if check_unchanged {
                    // The inserted lines may equal what was there, which
                    // makes the block smaller.
                    unsafe { diff_check_unchanged(tp, dp) };
                }
            }
        }
        (dprev, dp) = merge_or_advance(dprev, dp);
    }

    // A block every buffer now has nothing in is not a change any more.
    let mut dprev = ::core::ptr::null_mut::<diff_T>();
    let mut dp = tp.tp_first_diff;
    while !dp.is_null() {
        let empty = (0..DB_COUNT as usize)
            .all(|i| tp.tp_diffbuf[i].is_null() || unsafe { (*dp).df_count[i] } == 0);
        if empty {
            dp = unsafe { diff_free(tp, dprev, dp) };
        } else {
            dprev = dp;
            dp = unsafe { (*dp).df_next };
        }
    }

    if tp.is_current() {
        // Not right away: this runs per edit, and redrawing is slow.
        need_diff_redraw.set(true);
        // The filler lines may have moved, so the scroll binding has to
        // be recomputed -- also postponed until the redraw.
        diff_need_scrollbind.set(true);
    }
}

/// Insert a fresh, empty block between `dprev` and `dp`.
pub(crate) unsafe fn diff_alloc_new(
    mut tp: TabPage,
    dprev: *mut diff_T,
    dp: *mut diff_T,
) -> *mut diff_T {
    let dnew = unsafe { xcalloc(1, ::core::mem::size_of::<diff_T>()) } as *mut diff_T;
    unsafe { (*dnew).is_linematched = false };
    unsafe { (*dnew).has_changes = false };
    unsafe { (*dnew).df_next = dp };
    if dprev.is_null() {
        tp.tp_first_diff = dnew;
    } else {
        unsafe { (*dprev).df_next = dnew };
    }
    unsafe {
        ga_init(
            &raw mut (*dnew).df_changes,
            ::core::mem::size_of::<diffline_change_T>() as c_int,
            20,
        )
    };
    dnew
}

/// Unlink and free `dp`, answering the block that follows it.
pub(crate) unsafe fn diff_free(
    mut tp: TabPage,
    dprev: *mut diff_T,
    dp: *mut diff_T,
) -> *mut diff_T {
    let next = unsafe { (*dp).df_next };
    unsafe { clear_diffblock(dp) };
    if dprev.is_null() {
        tp.tp_first_diff = next;
    } else {
        unsafe { (*dprev).df_next = next };
    }
    next
}

/// Shrink `dp` from both ends while its first (or last) lines are equal in
/// every buffer.
///
/// An edit can leave a block claiming lines that did not actually change; the
/// diff is not recomputed for that, so the block is trimmed instead.
unsafe fn diff_check_unchanged(tp: TabPage, dp: *mut diff_T) {
    let Some(i_org) = (0..DB_COUNT as usize).find(|&i| !tp.tp_diffbuf[i].is_null()) else {
        return;
    };
    if unsafe { diff_check_sanity(tp, dp) } == FAIL {
        return;
    }
    for dir in [FORWARD as c_int, BACKWARD as c_int] {
        while unsafe { (*dp).df_count[i_org] } > 0 {
            let off_org = if dir == BACKWARD as c_int {
                unsafe { (*dp).df_count[i_org] - 1 }
            } else {
                0
            };
            // A copy: the `ml_get_buf` below invalidates the buffer this
            // one answers with.
            let line_org = unsafe {
                CStr::from_ptr(ml_get_buf(
                    tp.tp_diffbuf[i_org],
                    (*dp).df_lnum[i_org] + off_org,
                ))
            }
            .to_owned();
            let mut i_new = i_org + 1;
            while i_new < DB_COUNT as usize {
                if !tp.tp_diffbuf[i_new].is_null() {
                    let off_new = if dir == BACKWARD as c_int {
                        unsafe { (*dp).df_count[i_new] - 1 }
                    } else {
                        0
                    };
                    if off_new < 0 || off_new >= unsafe { (*dp).df_count[i_new] } {
                        break;
                    }
                    let other = unsafe {
                        CStr::from_ptr(ml_get_buf(
                            tp.tp_diffbuf[i_new],
                            (*dp).df_lnum[i_new] + off_new,
                        ))
                    };
                    if !lines_equal(&line_org, other) {
                        break;
                    }
                }
                i_new += 1;
            }
            if i_new != DB_COUNT as usize {
                break; // some buffer differs here; the block starts (or ends) for real
            }
            for i in i_org..DB_COUNT as usize {
                if !tp.tp_diffbuf[i].is_null() {
                    if dir == FORWARD as c_int {
                        unsafe { (*dp).df_lnum[i] += 1 };
                    }
                    unsafe { (*dp).df_count[i] -= 1 };
                }
            }
        }
    }
}

/// Whether every buffer's range in `dp` is still inside that buffer.
///
/// An edit can leave a block naming lines that no longer exist, and every
/// reader of a block has to check first.
pub(crate) unsafe fn diff_check_sanity(tp: TabPage, dp: *mut diff_T) -> c_int {
    for i in 0..DB_COUNT as usize {
        let buf = tp.tp_diffbuf[i];
        if !buf.is_null()
            && unsafe { (*dp).df_lnum[i] } + unsafe { (*dp).df_count[i] } - 1
                > unsafe { (*buf).b_ml.ml_line_count }
        {
            return FAIL;
        }
    }
    OK
}

/// Give buffer `idx_new` the same range as `idx_orig`, corrected for the
/// drift the previous block left behind.
pub(crate) unsafe fn diff_copy_entry(
    dprev: *mut diff_T,
    dp: *mut diff_T,
    idx_orig: usize,
    idx_new: usize,
) {
    let off = if dprev.is_null() {
        0
    } else {
        // SAFETY: the caller's previous block; `diff_T` is `Copy`, and this
        // only reads it.
        let prev = unsafe { *dprev };
        prev.df_lnum[idx_orig] + prev.df_count[idx_orig]
            - (prev.df_lnum[idx_new] + prev.df_count[idx_new])
    };
    unsafe { (*dp).df_lnum[idx_new] = (*dp).df_lnum[idx_orig] - off };
    unsafe { (*dp).df_count[idx_new] = (*dp).df_count[idx_orig] };
}

/// Free `tp`'s whole block list.
pub fn diff_clear(mut tp: TabPage) {
    let mut dp = tp.tp_first_diff;
    // SAFETY: the tab page's own block list, walked one node ahead of the
    // free, and nothing else points at these blocks.
    while !dp.is_null() {
        let next = unsafe { (*dp).df_next };
        unsafe { clear_diffblock(dp) };
        dp = next;
    }
    tp.tp_first_diff = ::core::ptr::null_mut();
}

/// The longest of `dp`'s ranges, which is how many screen rows it occupies in
/// every window: the shorter buffers are padded with filler.
pub(crate) unsafe fn get_max_diff_length(dp: *const diff_T) -> c_int {
    (0..DB_COUNT as usize)
        .filter(|&k| !unsafe { (*curtab.get()).tp_diffbuf[k] }.is_null())
        .map(|k| unsafe { (*dp).df_count[k] })
        .max()
        .unwrap_or(0)
}

/// Whether `diff` is still in the current tabpage's list.
///
/// `:diffget`/`:diffput` run autocommands between reading a block and using
/// it, and those can rebuild the list underneath.
pub(crate) unsafe fn valid_diff(diff: *mut diff_T) -> bool {
    let mut dp = unsafe { (*curtab.get()).tp_first_diff };
    while !dp.is_null() {
        if dp == diff {
            return true;
        }
        dp = unsafe { (*dp).df_next };
    }
    false
}

/// Whether `buf` is in any tabpage's diff.
pub fn diff_mode_buf(buf: Buf) -> bool {
    tabs().any(|tp| diff_buf_idx(buf, tp) != DB_COUNT)
}

/// The tab page the editor is working in.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}
