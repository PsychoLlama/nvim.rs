//! Signs placed by extmarks: which ones a row shows, and how wide the sign
//! column has to be.
//!
//! [`decor_redraw_signs`] collects the signs overlapping one row, sorts them
//! by priority ([`sign_item_cmp`]) and hands the drawing code the first few
//! plus the winning line/number/cursorline highlights.
//! [`buf_signcols_count_range`] keeps `b_signcols`, the per-row histogram
//! `'signcolumn'`'s `auto:N` reads, in step as signs are added and removed.
//!
//! Both walks have the same two-part shape, which is how the marktree is
//! asked about a row: `marktree_itr_get_overlap` +
//! `marktree_itr_step_overlap` yields the marks that *started earlier* and
//! reach into the row, and then `marktree_itr_step_out_filter` leaves the
//! iterator ready to walk the marks that start *on* it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{DECOR_ID_INVALID, decor_item, kSHIsSign, ns_in_win};
use crate::src::nvim::buffer::buf_meta_total;
use crate::src::nvim::decoration::{SCL_NUM, SIGN_WIDTH, kMTMetaSignText};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{curtab, first_tabpage, firstwin};
use crate::src::nvim::marktree::key::{
    MT_FLAG_DECOR_SIGNTEXT, kMTFilterSelect, mt_decor, mt_decor_sign, mt_end, mt_invalid,
};
use crate::src::nvim::marktree::{
    marktree_get_altpos, marktree_itr_current, marktree_itr_get_overlap, marktree_itr_next_filter,
    marktree_itr_step_out_filter, marktree_itr_step_overlap,
};
use crate::src::nvim::sign::buf_has_signs;
use crate::src::nvim::statusline::SIGN_SHOW_MAX;
use crate::src::nvim::types::{
    DecorInline, DecorSignHighlight, MTPair, MTPos, MarkTree, MarkTreeIter, MetaFilter, SignItem,
    SignTextAttrs, TriState, buf_T, kFalse, kNone, kTrue, linenr_T, tabpage_T, uint32_t, win_T,
};
use crate::src::nvim::winlayer::Win;
use ::core::ffi::c_int;
use ::core::{mem, ptr};

/// Marktree filters: which meta counts a walk is allowed to descend into.
/// The index is a `MetaIndex`; `kMTMetaSignText` is 3 and `kMTMetaSignHL` 2.
static SIGN_FILTER: GlobalCell<[uint32_t; 5]> =
    GlobalCell::new([0, 0, kMTFilterSelect, kMTFilterSelect, 0]);
static SIGNTEXT_FILTER: GlobalCell<[uint32_t; 5]> = GlobalCell::new([0, 0, 0, kMTFilterSelect, 0]);

fn sign_filter() -> MetaFilter {
    SIGN_FILTER.ptr().cast::<uint32_t>()
}

fn signtext_filter() -> MetaFilter {
    SIGNTEXT_FILTER.ptr().cast::<uint32_t>()
}

/// A zeroed marktree iterator, the state every walk here starts from.
fn new_iter() -> MarkTreeIter {
    // SAFETY: `MarkTreeIter` is plain data and all-zero is its initial state,
    // which is what the transpiled `{ 0 }` initialiser meant.
    unsafe { mem::zeroed() }
}

/// Signs shown in the `'number'` column are only one cell wide, so placing or
/// unplacing the first sign in `buf` has to make the number column's width be
/// recomputed rather than reused.
///
/// # Safety
/// `buf` must point to a live buffer.
unsafe fn may_force_numberwidth_recompute(buf: *mut buf_T, unplace: bool) {
    // SAFETY: the editor's own window list.
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get();
        while !tp.is_null() {
            // The current tabpage's window list lives in the globals, not in
            // the tabpage — that is what `FOR_ALL_TAB_WINDOWS` expands to.
            let mut wp = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == buf
                    && (*wp).w_minscwidth == SCL_NUM
                    && ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                    && (unplace || (*wp).w_nrwidth_width < 2)
                {
                    (*wp).w_nrwidth_line_count = 0 as linenr_T;
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next;
        }
    }
}

/// Serial number handed to each sign as it is placed; the last tiebreak in
/// [`sign_item_cmp`], so that two signs of equal priority and id keep the
/// order they were placed in.
static SIGN_ADD_ID: GlobalCell<c_int> = GlobalCell::new(0);

/// Accounts for a sign that has just been placed on rows `row1..=row2`.
///
/// # Safety
/// `buf` and `sh` must be live.
pub unsafe fn buf_put_decor_sh(
    buf: *mut buf_T,
    sh: *mut DecorSignHighlight,
    row1: c_int,
    row2: c_int,
) {
    // SAFETY: the caller's buffer and sign.
    unsafe {
        if (*sh).flags as c_int & kSHIsSign as c_int == 0 {
            return;
        }
        (*sh).sign_add_id = SIGN_ADD_ID.replace(SIGN_ADD_ID.get() + 1);
        if (*sh).text[0] != 0 {
            buf_signcols_count_range(buf, row1, row2, 1, kFalse);
            may_force_numberwidth_recompute(buf, false);
        }
    }
}

/// Accounts for a sign that is about to be removed from rows `row1..=row2`.
///
/// When it was the last sign in the buffer the histogram is zeroed outright
/// rather than decremented, because there is nothing left to count.
///
/// # Safety
/// `buf` and `sh` must be live.
pub unsafe fn buf_remove_decor_sh(
    buf: *mut buf_T,
    row1: c_int,
    row2: c_int,
    sh: *mut DecorSignHighlight,
) {
    // SAFETY: the caller's buffer and sign.
    unsafe {
        if (*sh).flags as c_int & kSHIsSign as c_int == 0 || (*sh).text[0] == 0 {
            return;
        }
        if buf_meta_total(buf, kMTMetaSignText) != 0 {
            buf_signcols_count_range(buf, row1, row2, -1, kFalse);
        } else {
            may_force_numberwidth_recompute(buf, true);
            (*buf).b_signcols.count[0] = 0;
            (*buf).b_signcols.max = 0;
        }
    }
}

/// Orders two signs on the same row, highest priority first.
///
/// Answers the `qsort` convention (negative means `a` sorts first). The
/// tiebreaks are the mark id and then the placement serial, both descending,
/// so the newest sign of equal priority wins the leftmost column.
///
/// # Safety
/// Both items' `sh` must be live.
pub unsafe fn sign_item_cmp(a: &SignItem, b: &SignItem) -> c_int {
    // SAFETY: the caller's signs.
    unsafe {
        let (sa, sb) = (&*a.sh, &*b.sh);
        if sa.priority != sb.priority {
            return if sa.priority < sb.priority { 1 } else { -1 };
        }
        if a.id != b.id {
            return if a.id < b.id { 1 } else { -1 };
        }
        if sa.sign_add_id != sb.sign_add_id {
            return if sa.sign_add_id < sb.sign_add_id {
                1
            } else {
                -1
            };
        }
        0
    }
}

/// The signs on `row`, and the highest-priority line/cursorline/number
/// highlights they carry.
///
/// `sattrs` receives the sign texts that fit in the sign column, highest
/// priority first; the three `*_id` out-parameters are only written when they
/// are still unset, so a caller can pre-seed them and this will not override.
///
/// # Safety
/// `wp` and `buf` must be live; `sattrs`, when not null, must have room for
/// `wp`'s sign column width; the `*_id` pointers must be null or writable.
pub unsafe fn decor_redraw_signs(
    wp: *mut win_T,
    buf: *mut buf_T,
    row: c_int,
    sattrs: *mut SignTextAttrs,
    line_id: *mut c_int,
    cul_id: *mut c_int,
    num_id: *mut c_int,
) {
    // SAFETY: the caller's window, buffer and out-parameters.
    unsafe {
        if !buf_has_signs(buf) {
            return;
        }
        let win = Win::new(wp);

        let tree: *mut MarkTree = (&raw mut (*buf).b_marktree).cast();
        let mut itr = new_iter();
        // TODO(bfredl): integrate with main decor loop.
        let mut signs: Vec<SignItem> = Vec::new();

        let mut pair: MTPair = mem::zeroed();
        marktree_itr_get_overlap(&mut *tree, row, 0, &mut itr);
        while marktree_itr_step_overlap(&mut *tree, &mut itr, &mut pair) {
            if !mt_invalid(pair.start) && mt_decor_sign(pair.start) && ns_in_win(pair.start.ns, win)
            {
                let sh = decor_find_sign(mt_decor(pair.start));
                signs.push(SignItem {
                    sh,
                    id: pair.start.id,
                });
            }
        }

        marktree_itr_step_out_filter(&mut *tree, &mut itr, sign_filter());
        while !itr.x.is_null() {
            let mark = marktree_itr_current(&mut itr);
            if mark.pos.row != row {
                break;
            }
            if !mt_invalid(mark) && !mt_end(mark) && mt_decor_sign(mark) && ns_in_win(mark.ns, win)
            {
                let sh = decor_find_sign(mt_decor(mark));
                signs.push(SignItem { sh, id: mark.id });
            }
            marktree_itr_next_filter(&mut *tree, &mut itr, row + 1, 0, sign_filter());
        }

        // How many of them have sign *text*; the rest only carry highlights.
        let num_text = signs.iter().filter(|item| (*item.sh).text[0] != 0).count() as c_int;

        if signs.is_empty() {
            return;
        }

        // A sign shown in the number column takes one cell whatever
        // 'signcolumn' says.
        let width = if (*wp).w_minscwidth == SCL_NUM {
            1
        } else {
            (*wp).w_scwidth
        };
        let len = width.min(num_text);
        let mut idx = 0;

        // A stable sort, and the comparator is a total order on distinct
        // signs: `sign_add_id` is handed out one per placement, so two
        // entries can only tie when they are the same sign.
        signs.sort_by(|a, b| sign_item_cmp(a, b).cmp(&0));

        for item in &signs {
            let sh = &*item.sh;
            if !sattrs.is_null() && idx < len && sh.text[0] != 0 {
                let out = &mut *sattrs.add(idx as usize);
                out.text.copy_from_slice(&sh.text[..SIGN_WIDTH as usize]);
                out.hl_id = sh.hl_id;
                idx += 1;
            }
            if !num_id.is_null() && *num_id <= 0 {
                *num_id = sh.number_hl_id;
            }
            if !line_id.is_null() && *line_id <= 0 {
                *line_id = sh.line_hl_id;
            }
            if !cul_id.is_null() && *cul_id <= 0 {
                *cul_id = sh.cursorline_hl_id;
            }
        }
    }
}

/// The first sign item in `decor`'s chain, or null if it has none.
///
/// # Safety
/// `decor` must be live.
pub unsafe fn decor_find_sign(decor: DecorInline) -> *mut DecorSignHighlight {
    // SAFETY: the caller's decoration.
    unsafe {
        if !decor.ext {
            return ptr::null_mut();
        }
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID {
            let sh = decor_item(idx);
            if (*sh).flags as c_int & kSHIsSign as c_int != 0 {
                return sh;
            }
            idx = (*sh).next;
        }
        ptr::null_mut()
    }
}

/// Re-counts the signs on rows `row1..=row2` and folds the difference into
/// `buf->b_signcols`, the histogram `'signcolumn'`'s `auto:N` reads.
///
/// `b_signcols.count[w - 1]` is how many rows show exactly `w` signs, so the
/// widest row is `max`. `add` says what just happened to the range — 1 for an
/// added sign, -1 for a deleted one, 0 for a range being counted from scratch
/// — and `clear` splits the update in two around a marktree splice: `kTrue`
/// only subtracts the old counts, `kNone` only adds the new ones, `kFalse`
/// does both.
///
/// # Safety
/// `buf` must point to a live buffer.
pub unsafe fn buf_signcols_count_range(
    buf: *mut buf_T,
    row1: c_int,
    row2: c_int,
    add: c_int,
    clear: TriState,
) {
    // SAFETY: the caller's buffer and the editor's own marktree.
    unsafe {
        if !(*buf).b_signcols.autom || row2 < row1 || buf_meta_total(buf, kMTMetaSignText) == 0 {
            return;
        }

        let tree: *mut MarkTree = (&raw mut (*buf).b_marktree).cast();
        let mut count = vec![0 as c_int; (row2 + 1 - row1) as usize];
        let mut itr = new_iter();
        let mut pair: MTPair = mem::zeroed();

        // Signs that start before `row1` but reach into the range.
        marktree_itr_get_overlap(&mut *tree, row1, 0, &mut itr);
        while marktree_itr_step_overlap(&mut *tree, &mut itr, &mut pair) {
            if pair.start.flags as c_int & MT_FLAG_DECOR_SIGNTEXT != 0 && !mt_invalid(pair.start) {
                for i in row1..=row2.min(pair.end_pos.row) {
                    count[(i - row1) as usize] += 1;
                }
            }
        }

        // Then everything that starts inside it, up to `row2`.
        marktree_itr_step_out_filter(&mut *tree, &mut itr, signtext_filter());
        while !itr.x.is_null() {
            let mark = marktree_itr_current(&mut itr);
            if mark.pos.row > row2 {
                break;
            }
            if mark.flags as c_int & MT_FLAG_DECOR_SIGNTEXT != 0
                && !mt_invalid(mark)
                && !mt_end(mark)
            {
                let end: MTPos = marktree_get_altpos(&mut *tree, mark, None);
                for i in mark.pos.row..=row2.min(end.row) {
                    count[(i - row1) as usize] += 1;
                }
            }
            marktree_itr_next_filter(&mut *tree, &mut itr, row2 + 1, 0, signtext_filter());
        }

        for &rowcount in &count {
            let prevwidth = SIGN_SHOW_MAX.min(rowcount - add);
            if clear != kNone && prevwidth > 0 {
                let slot = &mut (*buf).b_signcols.count[(prevwidth - 1) as usize];
                *slot -= 1;
                // TODO(bfredl): correct marktree splicing so that this doesn't fail
                debug_assert!(*slot >= 0);
            }
            let width = SIGN_SHOW_MAX.min(rowcount);
            if clear != kTrue && width > 0 {
                (*buf).b_signcols.count[(width - 1) as usize] += 1;
                if width > (*buf).b_signcols.max {
                    (*buf).b_signcols.max = width;
                }
            }
        }
    }
}
