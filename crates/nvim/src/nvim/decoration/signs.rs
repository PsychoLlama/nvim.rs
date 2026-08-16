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
//! asked about a row: [`Cursor::seek_overlap`] + [`Cursor::step_overlap`]
//! yields the marks that *started earlier* and reach into the row, and then
//! [`Cursor::step_out_filter`] leaves the walk ready for the marks that start
//! *on* it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{Sh, kSHIsSign, ns_in_win};
use crate::src::nvim::decoration::{SCL_NUM, SIGN_WIDTH, kMTMetaSignText};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::marktree::cursor::Cursor;
use crate::src::nvim::marktree::key::{
    MT_FLAG_DECOR_SIGNTEXT, kMTFilterSelect, mt_decor, mt_decor_sign, mt_end, mt_invalid,
};
use crate::src::nvim::marktree::meta::MetaCount;
use crate::src::nvim::sign::buf_has_signs;
use crate::src::nvim::statusline::SIGN_SHOW_MAX;
use crate::src::nvim::types::{
    DecorInline, DecorSignHighlight, MTPos, MarkTreeIter, SignItem, SignTextAttrs, TriState, buf_T,
    kFalse, kNone, kTrue, linenr_T, win_T,
};
use crate::src::nvim::winlayer::{Buf, Win, tab_windows};
use ::core::ffi::c_int;
use ::core::{ptr, slice};

/// Marktree filters: which meta counts a walk is allowed to descend into.
/// The index is a `MetaIndex`; `kMTMetaSignText` is 3 and `kMTMetaSignHL` 2.
static SIGN_FILTER: MetaCount = [0, 0, kMTFilterSelect, kMTFilterSelect, 0];
static SIGNTEXT_FILTER: MetaCount = [0, 0, 0, kMTFilterSelect, 0];

impl Sh {
    /// Whether this item is a sign rather than a bare highlight.
    fn is_sign(self) -> bool {
        self.flags as c_int & kSHIsSign as c_int != 0
    }

    /// Whether the sign has text of its own, which is what takes a cell of
    /// the sign column. A sign with none only contributes highlights.
    fn has_text(self) -> bool {
        self.text[0] != 0
    }
}

/// Signs shown in the `'number'` column are only one cell wide, so placing or
/// unplacing the first sign in `buf` has to make the number column's width be
/// recomputed rather than reused.
fn may_force_numberwidth_recompute(buf: Buf, unplace: bool) {
    for mut wp in tab_windows() {
        if wp.w_buffer == buf.raw()
            && wp.w_minscwidth == SCL_NUM
            && (wp.w_onebuf_opt.wo_nu != 0 || wp.w_onebuf_opt.wo_rnu != 0)
            && (unplace || wp.w_nrwidth_width < 2)
        {
            wp.w_nrwidth_line_count = 0 as linenr_T;
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
    let (buf, mut sh) = unsafe { (Buf::new(buf), Sh::new(sh)) };
    if !sh.is_sign() {
        return;
    }
    sh.sign_add_id = SIGN_ADD_ID.replace(SIGN_ADD_ID.get() + 1);
    if sh.has_text() {
        buf_signcols_count(buf, row1, row2, 1, kFalse);
        may_force_numberwidth_recompute(buf, false);
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
    let (mut buf, sh) = unsafe { (Buf::new(buf), Sh::new(sh)) };
    if !sh.is_sign() || !sh.has_text() {
        return;
    }
    if buf.meta_total(kMTMetaSignText) != 0 {
        buf_signcols_count(buf, row1, row2, -1, kFalse);
    } else {
        may_force_numberwidth_recompute(buf, true);
        buf.b_signcols.count[0] = 0;
        buf.b_signcols.max = 0;
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
    let (sa, sb) = unsafe { (Sh::new(a.sh), Sh::new(b.sh)) };
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

/// Every sign on `row` of `buf` that `wp` can see, in marktree order.
///
/// The two-part walk this module's header describes: the signs that started
/// on an earlier row and reach into this one, then the ones that start on it.
fn row_signs(buf: Buf, wp: Win, row: c_int) -> Vec<SignItem> {
    // TODO(bfredl): integrate with main decor loop.
    let mut signs: Vec<SignItem> = Vec::new();
    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buf, &mut itr);

    walk.seek_overlap(row, 0);
    while let Some(pair) = walk.step_overlap() {
        if !mt_invalid(pair.start) && mt_decor_sign(pair.start) && ns_in_win(pair.start.ns, wp) {
            let sh = decor_find_sign(mt_decor(pair.start));
            signs.push(SignItem {
                sh,
                id: pair.start.id,
            });
        }
    }

    walk.step_out_filter(&SIGN_FILTER);
    while !walk.is_empty() {
        let mark = walk.current();
        if mark.pos.row != row {
            break;
        }
        if !mt_invalid(mark) && !mt_end(mark) && mt_decor_sign(mark) && ns_in_win(mark.ns, wp) {
            let sh = decor_find_sign(mt_decor(mark));
            signs.push(SignItem { sh, id: mark.id });
        }
        walk.next_filter(row + 1, 0, &SIGN_FILTER);
    }
    signs
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
    // SAFETY: the caller's window and buffer.
    let (wp, buf) = unsafe { (Win::new(wp), Buf::new(buf)) };
    if !buf.has_signs() {
        return;
    }

    let mut signs = row_signs(buf, wp, row);
    // How many of them have sign *text*; the rest only carry highlights.
    // SAFETY: every `sh` came out of the decoration store.
    let num_text = signs
        .iter()
        .filter(|item| unsafe { Sh::new(item.sh) }.has_text())
        .count() as c_int;

    if signs.is_empty() {
        return;
    }

    // A sign shown in the number column takes one cell whatever
    // 'signcolumn' says.
    let width = if wp.w_minscwidth == SCL_NUM {
        1
    } else {
        wp.w_scwidth
    };
    let len = width.min(num_text).max(0) as usize;

    // A stable sort, and the comparator is a total order on distinct signs:
    // `sign_add_id` is handed out one per placement, so two entries can only
    // tie when they are the same sign.
    // SAFETY: as above — the items are live store entries.
    signs.sort_by(|a, b| unsafe { sign_item_cmp(a, b) }.cmp(&0));

    // SAFETY: the caller's out-parameter — null, or room for `wp`'s sign
    // column width, which `len` is at most.
    let mut texts = unsafe { (!sattrs.is_null()).then(|| slice::from_raw_parts_mut(sattrs, len)) };
    // SAFETY: the caller's out-parameters, each null or writable.
    let mut ids = unsafe { [num_id.as_mut(), line_id.as_mut(), cul_id.as_mut()] };
    let mut idx = 0;
    for item in &signs {
        // SAFETY: as above.
        let sh = unsafe { Sh::new(item.sh) };
        if let Some(texts) = texts.as_deref_mut() {
            if idx < len && sh.has_text() {
                texts[idx]
                    .text
                    .copy_from_slice(&sh.text[..SIGN_WIDTH as usize]);
                texts[idx].hl_id = sh.hl_id;
                idx += 1;
            }
        }
        let wanted = [sh.number_hl_id, sh.line_hl_id, sh.cursorline_hl_id];
        for (out, hl_id) in ids.iter_mut().zip(wanted) {
            if let Some(out) = out.as_deref_mut() {
                if *out <= 0 {
                    *out = hl_id;
                }
            }
        }
    }
}

impl Buf {
    /// Whether the buffer holds any sign at all — text or highlight.
    fn has_signs(self) -> bool {
        // SAFETY: a live buffer.
        unsafe { buf_has_signs(self.raw()) }
    }
}

/// The first sign item in `decor`'s chain, or null if it has none.
///
/// Safe: the chain is indices into the decoration store, which checks them,
/// and the answer is a pointer — dereferencing it is what needs a promise.
pub fn decor_find_sign(decor: DecorInline) -> *mut DecorSignHighlight {
    Sh::chain(decor)
        .find(|sh| sh.is_sign())
        .map_or(ptr::null_mut(), Sh::raw)
}

/// Re-counts the signs on rows `row1..=row2` and folds the difference into
/// `buf->b_signcols`, the histogram `'signcolumn'`'s `auto:N` reads.
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
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf) };
    buf_signcols_count(buf, row1, row2, add, clear);
}

/// [`buf_signcols_count_range`] for a buffer already promised live.
///
/// `b_signcols.count[w - 1]` is how many rows show exactly `w` signs, so the
/// widest row is `max`. `add` says what just happened to the range — 1 for an
/// added sign, -1 for a deleted one, 0 for a range being counted from scratch
/// — and `clear` splits the update in two around a marktree splice: `kTrue`
/// only subtracts the old counts, `kNone` only adds the new ones, `kFalse`
/// does both.
fn buf_signcols_count(mut buf: Buf, row1: c_int, row2: c_int, add: c_int, clear: TriState) {
    if !buf.b_signcols.autom || row2 < row1 || buf.meta_total(kMTMetaSignText) == 0 {
        return;
    }

    let mut count = vec![0 as c_int; (row2 + 1 - row1) as usize];
    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buf, &mut itr);

    // Signs that start before `row1` but reach into the range.
    walk.seek_overlap(row1, 0);
    while let Some(pair) = walk.step_overlap() {
        if pair.start.flags as c_int & MT_FLAG_DECOR_SIGNTEXT != 0 && !mt_invalid(pair.start) {
            for i in row1..=row2.min(pair.end_pos.row) {
                count[(i - row1) as usize] += 1;
            }
        }
    }

    // Then everything that starts inside it, up to `row2`.
    walk.step_out_filter(&SIGNTEXT_FILTER);
    while !walk.is_empty() {
        let mark = walk.current();
        if mark.pos.row > row2 {
            break;
        }
        if mark.flags as c_int & MT_FLAG_DECOR_SIGNTEXT != 0 && !mt_invalid(mark) && !mt_end(mark) {
            let end: MTPos = walk.altpos(mark);
            for i in mark.pos.row..=row2.min(end.row) {
                count[(i - row1) as usize] += 1;
            }
        }
        walk.next_filter(row2 + 1, 0, &SIGNTEXT_FILTER);
    }

    for &rowcount in &count {
        let prevwidth = SIGN_SHOW_MAX.min(rowcount - add);
        if clear != kNone && prevwidth > 0 {
            let slot = &mut buf.b_signcols.count[(prevwidth - 1) as usize];
            *slot -= 1;
            // TODO(bfredl): correct marktree splicing so that this doesn't fail
            debug_assert!(*slot >= 0);
        }
        let width = SIGN_SHOW_MAX.min(rowcount);
        if clear != kTrue && width > 0 {
            buf.b_signcols.count[(width - 1) as usize] += 1;
            if width > buf.b_signcols.max {
                buf.b_signcols.max = width;
            }
        }
    }
}
