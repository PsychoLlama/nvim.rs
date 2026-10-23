//! The per-line answers the drawer and the fold code ask.
//!
//! [`diff_check_with_linestatus`] says what a line's diff status is --
//! changed, added, filler, or unchanged -- which is what both the
//! highlighting and [`diff_check_fill`]'s filler count are read off.
//! [`diff_infold`] decides whether a line belongs inside a closed diff fold,
//! and [`diff_redraw`] is what marks the diffed windows dirty after the block
//! list moves.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::winlayer::{TabPage, Win, windows};
use core::ffi::c_int;

/// `linestatus`: the line is *changed* -- present in every buffer of the
/// block, but not with the same text.
pub(crate) const LINE_CHANGED: c_int = -1;

/// `linestatus`: the line is an *insertion* -- at least one buffer of the
/// block has no lines at all here.
pub(crate) const LINE_INSERTED: c_int = -2;

/// Mark every diffed window dirty, and re-settle the filler lines at the top.
///
/// `dofold` also rebuilds the folds, which is what a block-list change needs
/// and a mere scroll does not.
pub fn diff_redraw(dofold: bool) {
    need_diff_redraw.set(false);
    let mut wp_other: Option<Win> = None;
    let mut used_max_fill_curwin = false;
    let mut used_max_fill_other = false;

    // `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)`: the current tabpage's windows
    // are always the `firstwin` list.
    for mut wp in windows() {
        // The window is live but its buffer may already have been wiped, so
        // the pointer is compared, never read. The short circuit is upstream's.
        if wp.w_onebuf_opt.wo_diff == 0 || wp.surviving_buffer().is_none() {
            continue;
        }
        wp.redraw_later(UPD_SOME_VALID);
        if !wp.is_current() {
            wp_other = Some(wp);
        }
        if dofold && foldmethod_is_diff(wp) {
            fold_update_all(wp);
        }
        // Only the current window's topfill may *grow*, and only up
        // to what the block at its topline needs.
        let n = diff_check_fill(wp, wp.w_topline);
        if !wp.is_current() && wp.w_topfill > 0 || n > 0 {
            if wp.w_topfill > n {
                wp.w_topfill = n.max(0);
            } else if n > 0 && n > wp.w_topfill {
                wp.w_topfill = n;
                if wp.is_current() {
                    used_max_fill_curwin = true;
                } else if wp_other.is_some() {
                    used_max_fill_other = true;
                }
            }
            wp.check_topfill(false);
        }
    }

    // Whichever window took the larger filler count drives the other.
    if let Some(other) = wp_other
        && Win::current().w_onebuf_opt.wo_scb != 0
    {
        let pair = if used_max_fill_curwin {
            Some((other, Win::current()))
        } else if used_max_fill_other {
            Some((Win::current(), other))
        } else {
            None
        };
        if let Some((from, to)) = pair {
            diff_set_topline(from, to);
        }
    }
}

/// How many filler lines belong above `lnum`, and what its diff status is.
///
/// The answer is `(fill, linestatus)`. `fill` is the filler count, which is
/// only ever positive at a block boundary; `linestatus` is [`LINE_CHANGED`]
/// or [`LINE_INSERTED`], and zero when the line is not part of a change at
/// all. The two are independent: a line can carry filler *and* be changed.
pub fn diff_check_with_linestatus(window: Win, lnum: LineNr) -> (c_int, c_int) {
    let tp = TabPage::current();
    // A live window's buffer is live; a diffed window always has one.
    let buf = window.buffer();
    if tp.tp_diff_invalid != 0 {
        diff_update(None);
    }
    if tp.tp_diff.is_empty() || window.w_onebuf_opt.wo_diff == 0 {
        return (0, 0);
    }
    // One past the last line is legal: that is where the filler for a
    // deletion at end of buffer goes.
    if lnum < 1 || lnum > buf.b_ml.ml_line_count + 1 {
        return (0, 0);
    }
    let idx = tp.diff_index(buf);
    if idx == DB_COUNT {
        return (0, 0);
    }
    let idx = usize::try_from(idx).expect("a diff-buffer index is never negative");
    // A line inside a closed fold or concealed away has no status of its
    // own to report.
    //
    if window.fold_span(lnum).0 || decor_conceal_line(window, lnum - 1, false) {
        return (0, 0);
    }

    // Line matching is deferred until a block is actually on screen.
    let on_screen = lnum >= window.w_topline && lnum < window.w_botline;
    let slots = tp.tp_diffbuf;
    tp.with_diff(|blocks| line_status(blocks, &slots, idx, lnum, on_screen))
}

/// [`diff_check_with_linestatus`]'s answer, read off the block list.
fn line_status(
    blocks: &mut Vec<DiffBlock>,
    slots: &DiffSlots,
    idx: usize,
    lnum: LineNr,
    on_screen: bool,
) -> (c_int, c_int) {
    // The first block ending at or below `lnum`; anything above it is behind
    // the line and anything below has not started yet.
    let Some(mut at) = blocks.iter().position(|dp| lnum <= dp.end(idx)) else {
        return (0, 0);
    };
    let dp = &blocks[at];
    if lnum < dp.df_lnum[idx] {
        return (0, 0);
    }
    if on_screen
        && !dp.is_linematched
        && diff_linematch(dp, slots)
        && diff_check_sanity(slots, dp).is_ok()
    {
        run_linematch_algorithm(blocks, at, slots);
    }

    // At a block boundary, `lnum` is the line *after* the block, and the
    // filler is how many lines the longest buffer has there that this one
    // does not. Several blocks can end on the same line.
    let mut num_fill = 0;
    while lnum == blocks[at].end(idx) {
        let dp = &blocks[at];
        if diff_flags.get() & DIFF_FILLER != 0 {
            num_fill += dp.max_len(slots) - dp.df_count[idx];
        }
        let Some(next) = blocks.get(at + 1) else {
            break;
        };
        if lnum < next.df_lnum[idx] || lnum > next.end(idx) {
            break;
        }
        at += 1;
    }
    let dp = &blocks[at];
    if lnum >= dp.end(idx) {
        return (num_fill, 0);
    }

    // The line is inside the block. It is changed if any other buffer has
    // a different number of lines here, or the same number but different
    // text; it is an insertion if any other buffer has none.
    let mut zero = false;
    let mut cmp = false;
    for i in used_slots(slots).filter(|&i| i != idx) {
        if dp.df_count[i] == 0 {
            zero = true;
        } else if dp.df_count[i] != dp.df_count[idx] {
            return (num_fill, LINE_CHANGED);
        } else {
            cmp = true;
        }
    }
    if cmp {
        for i in 0..DB_COUNT as usize {
            if i != idx
                && slots[i].is_some()
                && dp.df_count[i] != 0
                && !diff_equal_entry(dp, slots, idx, i)
            {
                return (num_fill, LINE_CHANGED);
            }
        }
    }
    (num_fill, if zero { LINE_INSERTED } else { 0 })
}

/// The number of filler lines above `lnum`, or 0 when `'diffopt'` has no
/// `filler`.
pub fn diff_check_fill(window: Win, lnum: LineNr) -> c_int {
    if diff_flags.get() & DIFF_FILLER == 0 {
        return 0;
    }
    diff_check_with_linestatus(window, lnum).0.max(0)
}

/// Whether `lnum` belongs inside a closed diff fold.
///
/// Unchanged, and further than `'diffopt'`'s `context:` from any change.  A
/// window whose buffer is the only one in the diff folds nothing.
pub fn diff_infold(window: Win, lnum: LineNr) -> bool {
    if window.w_onebuf_opt.wo_diff == 0 {
        return false;
    }
    let tp = TabPage::current();
    let mut idx = None;
    let mut other = false;
    for i in 0..DB_COUNT as usize {
        if tp.diff_slot_holds(i, window.w_buffer) {
            idx = Some(i);
        } else if tp.tp_diffbuf[i].is_some() {
            other = true;
        }
    }
    let (Some(idx), true) = (idx, other) else {
        return false;
    };

    if tp.tp_diff_invalid != 0 {
        diff_update(None);
    }
    let context = diff_context.get() as LineNr;
    // The blocks are in line order, so the first one starting below the
    // context window ends the search. No blocks at all folds everything.
    for dp in &tp.tp_diff {
        if dp.df_lnum[idx] - context > lnum {
            break;
        }
        if dp.end(idx) + context > lnum {
            return false;
        }
    }
    true
}

/// Rebuild the folds covering a block's ranges in every window but the one
/// that just changed.
///
/// Takes the ranges, not the block: the one caller reads them off a block it
/// has already taken out of the list.
pub(crate) fn diff_fold_update(
    lnum: &[LineNr; DB_COUNT as usize],
    count: &[LineNr; DB_COUNT as usize],
    skip_idx: c_int,
) {
    let tp = TabPage::current();
    // A negative `skip_idx` names no slot, which `usize::MAX` reproduces.
    let skip = usize::try_from(skip_idx).unwrap_or(usize::MAX);
    for wp in windows() {
        for i in 0..DB_COUNT as usize {
            if tp.diff_slot_holds(i, wp.w_buffer) && i != skip {
                fold_update(wp, lnum[i], lnum[i] + count[i]);
            }
        }
    }
}

/// `diff_filler(lnum)`.
pub fn f_diff_filler(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // c2rust expanded `MAX(0, ..)` into two calls of an argument that is
    // not free: `diff_check_fill` can trigger a whole recompute, and it
    // already clamps at zero.
    //
    let fill = diff_check_fill(Win::current(), tv_get_lnum(&args[0]));
    result.write_number(VarNumber::from(fill));
}
