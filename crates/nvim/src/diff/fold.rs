//! The per-line answers the drawer and the fold code ask.
//!
//! [`diff_check_with_linestatus`] says what a line's diff status is --
//! changed, added, filler, or unchanged -- which is what both the
//! highlighting and [`diff_check_fill`]'s filler count are read off.
//! [`diff_infold`] decides whether a line belongs inside a closed diff fold,
//! and [`diff_redraw`] is what marks the diffed windows dirty after the block
//! list moves.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::{Buf, TabPage, Win, windows};
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
pub unsafe fn diff_redraw(dofold: bool) {
    need_diff_redraw.set(false);
    let mut wp_other: Option<Win> = None;
    let mut used_max_fill_curwin = false;
    let mut used_max_fill_other = false;

    // `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)`: the current tabpage's windows
    // are always the `firstwin` list.
    for mut wp in windows() {
        // SAFETY: a live window's buffer pointer is a buffer or null, which
        // is what `buf_valid` is for. The short circuit is upstream's.
        if wp.w_onebuf_opt.wo_diff == 0 || !unsafe { buf_valid(wp.w_buffer) } {
            continue;
        }
        wp.redraw_later(UPD_SOME_VALID);
        if !wp.is_current() {
            wp_other = Some(wp);
        }
        // SAFETY: a live window, here and in the two calls below.
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
        && cur_win().w_onebuf_opt.wo_scb != 0
    {
        let pair = if used_max_fill_curwin {
            Some((other, cur_win()))
        } else if used_max_fill_other {
            Some((cur_win(), other))
        } else {
            None
        };
        if let Some((from, to)) = pair {
            // SAFETY: both windows are live.
            diff_set_topline(from, to);
        }
    }
}

/// How many filler lines belong above `lnum`, and what its diff status is.
///
/// The return is the filler count, which is only ever positive at a block
/// boundary.  `linestatus`, if given, is [`LINE_CHANGED`] or
/// [`LINE_INSERTED`]; zero means the line is not part of a change at all.
/// The two are independent: a line can carry filler *and* be changed.
///
/// # Safety
/// `linestatus` must be null or writable.
pub unsafe fn diff_check_with_linestatus(wp: Win, lnum: linenr_T, linestatus: *mut c_int) -> c_int {
    let set_status = |status| {
        if !linestatus.is_null() {
            // SAFETY: the caller's out-parameter, and it is not null.
            unsafe { *linestatus = status };
        }
    };
    set_status(0);

    let tp = cur_tab();
    // SAFETY: a live window's buffer is live; a diffed window always has one.
    let buf = unsafe { Buf::new(wp.w_buffer) };
    if tp.tp_diff_invalid != 0 {
        // SAFETY: the editor exists.
        unsafe { ex_diffupdate(::core::ptr::null_mut()) };
    }
    if tp.tp_first_diff.is_null() || wp.w_onebuf_opt.wo_diff == 0 {
        return 0;
    }
    // One past the last line is legal: that is where the filler for a
    // deletion at end of buffer goes.
    if lnum < 1 || lnum > buf.b_ml.ml_line_count + 1 {
        return 0;
    }
    // SAFETY: a live buffer and a live tab page.
    let idx = diff_buf_idx(buf, tp);
    if idx == DB_COUNT {
        return 0;
    }
    let idx = idx as usize;
    // A line inside a closed fold or concealed away has no status of its
    // own to report.
    //
    // SAFETY: a live window. The short circuit is upstream's: the conceal
    // query runs only for a line that is not folded away.
    if wp.fold_span(lnum).0 || unsafe { decor_conceal_line(wp.raw(), lnum - 1, false) } {
        return 0;
    }

    // The first block ending at or below `lnum`; anything above it is behind
    // the line and anything below has not started yet.
    let Some(mut dp) = diff_blocks(tp).find(|dp| lnum <= dp.end(idx)) else {
        return 0;
    };
    if lnum < dp.df_lnum[idx] {
        return 0;
    }

    // Line matching is deferred until a block is actually on screen.
    // SAFETY: a live block and a live tab page, in all three calls.
    let match_now = lnum >= wp.w_topline
        && lnum < wp.w_botline
        && !dp.is_linematched
        && unsafe { diff_linematch(dp.raw()) }
        && dp.is_sane(tp);
    if match_now {
        // SAFETY: a live block.
        unsafe { run_linematch_algorithm(dp.raw()) };
    }

    // At a block boundary, `lnum` is the line *after* the block, and the
    // filler is how many lines the longest buffer has there that this one
    // does not. Several blocks can end on the same line.
    let mut num_fill = 0;
    while lnum == dp.end(idx) {
        if diff_flags.get() & DIFF_FILLER != 0 {
            num_fill += dp.max_len() - dp.df_count[idx];
        }
        let Some(next) = dp.next() else { break };
        if lnum < next.df_lnum[idx] || lnum > next.end(idx) {
            break;
        }
        dp = next;
    }
    if lnum >= dp.end(idx) {
        return num_fill;
    }

    // The line is inside the block. It is changed if any other buffer has
    // a different number of lines here, or the same number but different
    // text; it is an insertion if any other buffer has none.
    let mut zero = false;
    let mut cmp = false;
    for i in 0..DB_COUNT as usize {
        if i == idx || tp.tp_diffbuf[i].is_null() {
            continue;
        }
        if dp.df_count[i] == 0 {
            zero = true;
        } else if dp.df_count[i] != dp.df_count[idx] {
            set_status(LINE_CHANGED);
            return num_fill;
        } else {
            cmp = true;
        }
    }
    if cmp {
        for i in 0..DB_COUNT as usize {
            if i != idx
                && !tp.tp_diffbuf[i].is_null()
                && dp.df_count[i] != 0
                && !dp.equal_entry(idx, i)
            {
                set_status(LINE_CHANGED);
                return num_fill;
            }
        }
    }
    if zero {
        set_status(LINE_INSERTED);
    }
    num_fill
}

/// The number of filler lines above `lnum`, or 0 when `'diffopt'` has no
/// `filler`.
///
/// Safe: a [`Win`] carries the whole of the promise this needs.
pub fn diff_check_fill(wp: Win, lnum: linenr_T) -> c_int {
    if diff_flags.get() & DIFF_FILLER == 0 {
        return 0;
    }
    // SAFETY: no status is asked for, so there is nothing to write through.
    unsafe { diff_check_with_linestatus(wp, lnum, ::core::ptr::null_mut()) }.max(0)
}

/// Whether `lnum` belongs inside a closed diff fold.
///
/// Unchanged, and further than `'diffopt'`'s `context:` from any change.  A
/// window whose buffer is the only one in the diff folds nothing.
///
/// Safe: a [`Win`] carries the whole of the promise this needs.
pub fn diff_infold(wp: Win, lnum: linenr_T) -> bool {
    if wp.w_onebuf_opt.wo_diff == 0 {
        return false;
    }
    let tp = cur_tab();
    let mut idx = None;
    let mut other = false;
    for i in 0..DB_COUNT as usize {
        if tp.tp_diffbuf[i] == wp.w_buffer {
            idx = Some(i);
        } else if !tp.tp_diffbuf[i].is_null() {
            other = true;
        }
    }
    let (Some(idx), true) = (idx, other) else {
        return false;
    };

    if tp.tp_diff_invalid != 0 {
        // SAFETY: the editor exists.
        unsafe { ex_diffupdate(::core::ptr::null_mut()) };
    }
    if tp.tp_first_diff.is_null() {
        return true;
    }
    let context = diff_context.get() as linenr_T;
    for dp in diff_blocks(tp) {
        // The blocks are in line order, so the first one starting below
        // the context window ends the search.
        if dp.df_lnum[idx] - context > lnum {
            break;
        }
        if dp.end(idx) + context > lnum {
            return false;
        }
    }
    true
}

/// Rebuild the folds covering `dp` in every window but the one that just
/// changed.
///
/// # Safety
/// `dp` must be a live diff block.
pub(crate) unsafe fn diff_fold_update(dp: *mut diff_T, skip_idx: c_int) {
    // SAFETY: the caller's block.
    let dp = unsafe { Df::new(dp) };
    let tp = cur_tab();
    for wp in windows() {
        for i in 0..DB_COUNT as usize {
            if tp.tp_diffbuf[i] == wp.w_buffer && i as c_int != skip_idx {
                // SAFETY: a live window.
                fold_update(wp, dp.df_lnum[i], dp.end(i));
            }
        }
    }
}

/// `diff_filler(lnum)`.
///
/// # Safety
/// `argvars` and `rettv` must be the evaluator's live cells.
pub unsafe fn f_diff_filler(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // c2rust expanded `MAX(0, ..)` into two calls of an argument that is
    // not free: `diff_check_fill` can trigger a whole recompute, and it
    // already clamps at zero.
    //
    // SAFETY: the caller's cells, and the current window is live.
    let fill = diff_check_fill(cur_win(), unsafe { tv_get_lnum(argvars) });
    unsafe { (*rettv).vval.v_number = fill as varnumber_T };
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
