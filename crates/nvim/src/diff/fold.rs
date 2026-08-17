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
    unsafe {
        need_diff_redraw.set(false);
        let mut wp_other = ::core::ptr::null_mut::<win_T>();
        let mut used_max_fill_curwin = false;
        let mut used_max_fill_other = false;

        // `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)`: the current tabpage's windows
        // are always the `firstwin` list.
        let mut wp = firstwin.get();
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_diff != 0 && buf_valid((*wp).w_buffer) {
                redraw_later(wp, UPD_SOME_VALID);
                if wp != curwin.get() {
                    wp_other = wp;
                }
                if dofold && foldmethodIsDiff(wp) {
                    foldUpdateAll(wp);
                }
                // Only the current window's topfill may *grow*, and only up
                // to what the block at its topline needs.
                let n = diff_check_fill(wp, (*wp).w_topline);
                if wp != curwin.get() && (*wp).w_topfill > 0 || n > 0 {
                    if (*wp).w_topfill > n {
                        (*wp).w_topfill = n.max(0);
                    } else if n > 0 && n > (*wp).w_topfill {
                        (*wp).w_topfill = n;
                        if wp == curwin.get() {
                            used_max_fill_curwin = true;
                        } else if !wp_other.is_null() {
                            used_max_fill_other = true;
                        }
                    }
                    check_topfill(wp, false);
                }
            }
            wp = (*wp).w_next;
        }

        // Whichever window took the larger filler count drives the other.
        if !wp_other.is_null() && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
            if used_max_fill_curwin {
                diff_set_topline(wp_other, curwin.get());
            } else if used_max_fill_other {
                diff_set_topline(curwin.get(), wp_other);
            }
        }
    }
}

/// How many filler lines belong above `lnum`, and what its diff status is.
///
/// The return is the filler count, which is only ever positive at a block
/// boundary.  `linestatus`, if given, is [`LINE_CHANGED`] or
/// [`LINE_INSERTED`]; zero means the line is not part of a change at all.
/// The two are independent: a line can carry filler *and* be changed.
pub unsafe fn diff_check_with_linestatus(
    wp: *mut win_T,
    lnum: linenr_T,
    linestatus: *mut c_int,
) -> c_int {
    unsafe {
        let set_status = |status| {
            if !linestatus.is_null() {
                *linestatus = status;
            }
        };
        set_status(0);

        let tp = curtab.get();
        let buf = (*wp).w_buffer;
        if (*tp).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut());
        }
        if (*tp).tp_first_diff.is_null() || (*wp).w_onebuf_opt.wo_diff == 0 {
            return 0;
        }
        // One past the last line is legal: that is where the filler for a
        // deletion at end of buffer goes.
        if lnum < 1 || lnum > (*buf).b_ml.ml_line_count + 1 {
            return 0;
        }
        let idx = diff_buf_idx(buf, tp);
        if idx == DB_COUNT {
            return 0;
        }
        // A line inside a closed fold or concealed away has no status of its
        // own to report.
        if hasFolding(wp, lnum, ::core::ptr::null_mut(), ::core::ptr::null_mut())
            || decor_conceal_line(wp, lnum - 1, false)
        {
            return 0;
        }

        let mut dp = (*tp).tp_first_diff;
        while !dp.is_null() && lnum > (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            dp = (*dp).df_next;
        }
        if dp.is_null() || lnum < (*dp).df_lnum[idx as usize] {
            return 0;
        }

        // Line matching is deferred until a block is actually on screen.
        if lnum >= (*wp).w_topline
            && lnum < (*wp).w_botline
            && !(*dp).is_linematched
            && diff_linematch(dp)
            && diff_check_sanity(tp, dp) != 0
        {
            run_linematch_algorithm(dp);
        }

        // At a block boundary, `lnum` is the line *after* the block, and the
        // filler is how many lines the longest buffer has there that this one
        // does not. Several blocks can end on the same line.
        let mut num_fill = 0;
        while lnum == (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            if diff_flags.get() & DIFF_FILLER != 0 {
                num_fill += get_max_diff_length(dp) - (*dp).df_count[idx as usize];
            }
            let next = (*dp).df_next;
            if next.is_null()
                || lnum < (*next).df_lnum[idx as usize]
                || lnum > (*next).df_lnum[idx as usize] + (*next).df_count[idx as usize]
            {
                break;
            }
            dp = next;
        }
        if lnum >= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            return num_fill;
        }

        // The line is inside the block. It is changed if any other buffer has
        // a different number of lines here, or the same number but different
        // text; it is an insertion if any other buffer has none.
        let mut zero = false;
        let mut cmp = false;
        for i in 0..DB_COUNT as usize {
            if i as c_int == idx || (*tp).tp_diffbuf[i].is_null() {
                continue;
            }
            if (*dp).df_count[i] == 0 {
                zero = true;
            } else if (*dp).df_count[i] != (*dp).df_count[idx as usize] {
                set_status(LINE_CHANGED);
                return num_fill;
            } else {
                cmp = true;
            }
        }
        if cmp {
            for i in 0..DB_COUNT as usize {
                if i as c_int != idx
                    && !(*tp).tp_diffbuf[i].is_null()
                    && (*dp).df_count[i] != 0
                    && !diff_equal_entry(dp, idx as usize, i)
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
}

/// The number of filler lines above `lnum`, or 0 when `'diffopt'` has no
/// `filler`.
pub unsafe fn diff_check_fill(wp: *mut win_T, lnum: linenr_T) -> c_int {
    unsafe {
        if diff_flags.get() & DIFF_FILLER == 0 {
            return 0;
        }
        diff_check_with_linestatus(wp, lnum, ::core::ptr::null_mut()).max(0)
    }
}

/// Whether `lnum` belongs inside a closed diff fold.
///
/// Unchanged, and further than `'diffopt'`'s `context:` from any change.  A
/// window whose buffer is the only one in the diff folds nothing.
pub unsafe fn diff_infold(wp: *mut win_T, lnum: linenr_T) -> bool {
    unsafe {
        if (*wp).w_onebuf_opt.wo_diff == 0 {
            return false;
        }
        let tp = curtab.get();
        let mut idx = None;
        let mut other = false;
        for i in 0..DB_COUNT as usize {
            if (*tp).tp_diffbuf[i] == (*wp).w_buffer {
                idx = Some(i);
            } else if !(*tp).tp_diffbuf[i].is_null() {
                other = true;
            }
        }
        let (Some(idx), true) = (idx, other) else {
            return false;
        };

        if (*tp).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut());
        }
        if (*tp).tp_first_diff.is_null() {
            return true;
        }
        let context = diff_context.get() as linenr_T;
        let mut dp = (*tp).tp_first_diff;
        while !dp.is_null() {
            // The blocks are in line order, so the first one starting below
            // the context window ends the search.
            if (*dp).df_lnum[idx] - context > lnum {
                break;
            }
            if (*dp).df_lnum[idx] + (*dp).df_count[idx] + context > lnum {
                return false;
            }
            dp = (*dp).df_next;
        }
        true
    }
}

/// Rebuild the folds covering `dp` in every window but the one that just
/// changed.
pub(crate) unsafe fn diff_fold_update(dp: *mut diff_T, skip_idx: c_int) {
    unsafe {
        let tp = curtab.get();
        let mut wp = firstwin.get();
        while !wp.is_null() {
            for i in 0..DB_COUNT as usize {
                if (*tp).tp_diffbuf[i] == (*wp).w_buffer && i as c_int != skip_idx {
                    foldUpdate(wp, (*dp).df_lnum[i], (*dp).df_lnum[i] + (*dp).df_count[i]);
                }
            }
            wp = (*wp).w_next;
        }
    }
}

/// `diff_filler(lnum)`.
pub unsafe extern "C" fn f_diff_filler(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        // c2rust expanded `MAX(0, ..)` into two calls of an argument that is
        // not free: `diff_check_fill` can trigger a whole recompute, and it
        // already clamps at zero.
        (*rettv).vval.v_number = diff_check_fill(curwin.get(), tv_get_lnum(argvars)) as varnumber_T;
    }
}
