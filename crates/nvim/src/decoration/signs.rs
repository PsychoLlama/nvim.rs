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

#![forbid(unsafe_code)]

use core::cmp::{Ordering, Reverse};

use super::{kSHIsSign, ns_in_win, sh_chain, with_decor_item, with_decor_item_mut};
use crate::decoration::{SCL_NUM, SIGN_WIDTH, kMTMetaSignText};
use crate::global_cell::GlobalCell;
use crate::marktree::cursor::Cursor;
use crate::marktree::key::{MtFlags, kMTFilterSelect, mt_decor, mt_decor_sign, mt_end, mt_invalid};
use crate::marktree::meta::MetaCount;
use crate::sign::buf_has_signs;
use crate::statusline::SIGN_SHOW_MAX;
use crate::types::{
    DecorInline, DecorPriority, DecorSignHighlight, LineNr, MTKey, MTPos, MarkTreeIter, SignItem,
    SignTextAttrs, uint32_t,
};
use crate::winlayer::{Buf, Win, tab_windows};
use core::ffi::{CStr, c_int};

/// Marktree filters: which meta counts a walk is allowed to descend into.
/// The index is a `MetaIndex`; `kMTMetaSignText` is 3 and `kMTMetaSignHL` 2.
static SIGN_FILTER: MetaCount = [0, 0, kMTFilterSelect, kMTFilterSelect, 0];
static SIGNTEXT_FILTER: MetaCount = [0, 0, 0, kMTFilterSelect, 0];

impl DecorSignHighlight {
    /// Whether this item is a sign rather than a bare highlight.
    fn is_sign(&self) -> bool {
        self.flags & kSHIsSign != 0
    }

    /// Whether the sign has text of its own, which is what takes a cell of
    /// the sign column. A sign with none only contributes highlights.
    fn has_text(&self) -> bool {
        self.text[0] != 0
    }
}

/// Signs shown in the `'number'` column are only one cell wide, so placing or
/// unplacing the first sign in `buffer` has to make the number column's width be
/// recomputed rather than reused.
fn may_force_numberwidth_recompute(buffer: Buf, unplace: bool) {
    for mut wp in tab_windows() {
        if wp.w_buffer == buffer
            && wp.w_minscwidth == SCL_NUM
            && (wp.w_onebuf_opt.wo_nu != 0 || wp.w_onebuf_opt.wo_rnu != 0)
            && (unplace || wp.w_nrwidth_width < 2)
        {
            wp.w_nrwidth_line_count = 0 as LineNr;
        }
    }
}

/// Serial number handed to each sign as it is placed; the last tiebreak in
/// [`sign_item_cmp`], so that two signs of equal priority and id keep the
/// order they were placed in.
static SIGN_ADD_ID: GlobalCell<c_int> = GlobalCell::new(0);

/// Accounts for sign/highlight item `idx`, which has just been placed on
/// rows `row1..=row2`.
pub fn buf_put_decor_sh(buffer: Buf, idx: uint32_t, row1: c_int, row2: c_int) {
    let has_text = with_decor_item_mut(idx, |sh| {
        if !sh.is_sign() {
            return false;
        }
        sh.sign_add_id = SIGN_ADD_ID.replace(SIGN_ADD_ID.get() + 1);
        sh.has_text()
    });
    if has_text {
        buf_signcols_count(buffer, row1, row2, 1, SignCountHalf::Both);
        may_force_numberwidth_recompute(buffer, false);
    }
}

/// Accounts for sign/highlight item `idx`, which is about to be removed from
/// rows `row1..=row2`.
///
/// When it was the last sign in the buffer the histogram is zeroed outright
/// rather than decremented, because there is nothing left to count.
pub fn buf_remove_decor_sh(mut buffer: Buf, row1: c_int, row2: c_int, idx: uint32_t) {
    if !with_decor_item(idx, |sh| sh.is_sign() && sh.has_text()) {
        return;
    }
    if buffer.meta_total(kMTMetaSignText) != 0 {
        buf_signcols_count(buffer, row1, row2, -1, SignCountHalf::Both);
    } else {
        may_force_numberwidth_recompute(buffer, true);
        buffer.b_signcols.count[0] = 0;
        buffer.b_signcols.max = 0;
    }
}

/// Where one sign sorts among the signs on its row: highest priority first,
/// then the newest mark id, then the newest placement serial.
///
/// All three descend, which is what `Reverse` says here — the three
/// hand-written `if a < b { 1 } else { -1 }` ladders upstream spells them as
/// were the same fact three times.
fn sign_rank(priority: DecorPriority, id: uint32_t, add_id: c_int) -> impl Ord {
    Reverse((priority, id, add_id))
}

/// Orders two signs on the same row, highest priority first. See
/// [`sign_rank`].
pub fn sign_item_cmp(a: &SignItem, b: &SignItem) -> Ordering {
    sign_rank(a.priority, a.id, a.sign_add_id).cmp(&sign_rank(b.priority, b.id, b.sign_add_id))
}

/// The sort key and store index of the sign `mark` carries, if it carries
/// one.
pub(crate) fn sign_item(mark: MTKey) -> Option<SignItem> {
    let idx = decor_sign(mt_decor(mark))?;
    Some(with_decor_item(idx, |sh| SignItem {
        idx,
        id: mark.id,
        priority: sh.priority,
        sign_add_id: sh.sign_add_id,
    }))
}

/// Every sign on `row` of `buffer` that `window` can see, in marktree order.
///
/// The two-part walk this module's header describes: the signs that started
/// on an earlier row and reach into this one, then the ones that start on it.
fn row_signs(buffer: Buf, window: Win, row: c_int) -> Vec<SignItem> {
    // TODO(bfredl): integrate with main decor loop.
    let mut signs: Vec<SignItem> = Vec::new();
    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buffer, &mut itr);

    walk.seek_overlap(row, 0);
    while let Some(pair) = walk.step_overlap() {
        if !mt_invalid(pair.start)
            && mt_decor_sign(pair.start)
            && ns_in_win(pair.start.ns, window)
            && let Some(item) = sign_item(pair.start)
        {
            signs.push(item);
        }
    }

    walk.step_out_filter(&SIGN_FILTER);
    while !walk.is_empty() {
        let mark = walk.current();
        if mark.pos.row != row {
            break;
        }
        if !mt_invalid(mark)
            && !mt_end(mark)
            && mt_decor_sign(mark)
            && ns_in_win(mark.ns, window)
            && let Some(item) = sign_item(mark)
        {
            signs.push(item);
        }
        walk.step_filter(row + 1, 0, &SIGN_FILTER);
    }
    signs
}

/// The signs on `row`, and the highest-priority line/cursorline/number
/// highlights they carry.
///
/// `sattrs` receives the sign texts that fit in the sign column, highest
/// priority first; the three `*_id` out-parameters are only written when they
/// are still unset, so a caller can pre-seed them and this will not override.
pub fn decor_redraw_signs(
    window: Win,
    buffer: Buf,
    row: c_int,
    mut sattrs: Option<&mut [SignTextAttrs]>,
    line_id: Option<&mut c_int>,
    cul_id: Option<&mut c_int>,
    num_id: Option<&mut c_int>,
) {
    let (wp, buf) = (window, buffer);
    if !buf.has_signs() {
        return;
    }

    let mut signs = row_signs(buf, wp, row);
    // How many of them have sign *text*; the rest only carry highlights.
    let num_text = signs
        .iter()
        .filter(|item| with_decor_item(item.idx, DecorSignHighlight::has_text))
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
    signs.sort_by(sign_item_cmp);

    let mut ids = [num_id, line_id, cul_id];
    let mut idx = 0;
    for item in &signs {
        with_decor_item(item.idx, |sh| {
            if let Some(texts) = sattrs.as_deref_mut()
                && idx < len.min(texts.len())
                && sh.has_text()
            {
                texts[idx]
                    .text
                    .copy_from_slice(&sh.text[..SIGN_WIDTH as usize]);
                texts[idx].hl_id = sh.hl_id;
                idx += 1;
            }
            let wanted = [sh.number_hl_id, sh.line_hl_id, sh.cursorline_hl_id];
            for (out, hl_id) in ids.iter_mut().zip(wanted) {
                if let Some(out) = out.as_deref_mut()
                    && *out <= 0
                {
                    *out = hl_id;
                }
            }
        });
    }
}

impl Buf {
    /// Whether the buffer holds any sign at all — text or highlight.
    fn has_signs(self) -> bool {
        buf_has_signs(self)
    }
}

/// The store index of the first sign item in `decor`'s chain, if it has one.
pub fn decor_sign(decor: DecorInline) -> Option<uint32_t> {
    sh_chain(decor).find(|&idx| with_decor_item(idx, DecorSignHighlight::is_sign))
}

impl DecorSignHighlight {
    /// The name of the `:sign` definition this placement was copied from;
    /// `None` for a sign `nvim_buf_set_extmark` placed.
    pub fn sign_name_cstr(&self) -> Option<&CStr> {
        self.sign_name.as_deref()
    }
}

/// Which half of a sign re-count a [`buf_signcols_count_range`] call does.
///
/// A marktree splice moves the marks between the two halves, so the counts
/// have to come off the histogram before it and go back on after it. That is
/// three answers, not an unknown boolean.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SignCountHalf {
    /// Subtract the old counts and add the new ones in one pass.
    Both,
    /// Only subtract the counts the range had before.
    Subtract,
    /// Only add the counts the range has now.
    Add,
}

/// Re-counts the signs on rows `row1..=row2` and folds the difference into
/// `buffer.b_signcols`, the histogram `'signcolumn'`'s `auto:N` reads.
pub fn buf_signcols_count_range(
    buffer: Buf,
    row1: c_int,
    row2: c_int,
    add: c_int,
    half: SignCountHalf,
) {
    buf_signcols_count(buffer, row1, row2, add, half);
}

/// [`buf_signcols_count_range`] for a buffer already promised live.
///
/// `b_signcols.count[w - 1]` is how many rows show exactly `w` signs, so the
/// widest row is `max`. `add` says what just happened to the range — 1 for an
/// added sign, -1 for a deleted one, 0 for a range being counted from scratch
/// — and `half` is which side of a marktree splice this call is doing.
fn buf_signcols_count(mut buffer: Buf, row1: c_int, row2: c_int, add: c_int, half: SignCountHalf) {
    if !buffer.b_signcols.autom || row2 < row1 || buffer.meta_total(kMTMetaSignText) == 0 {
        return;
    }

    let mut count = vec![0 as c_int; (row2 + 1 - row1) as usize];
    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buffer, &mut itr);

    // Signs that start before `row1` but reach into the range.
    walk.seek_overlap(row1, 0);
    while let Some(pair) = walk.step_overlap() {
        if pair.start.flags.has(MtFlags::DECOR_SIGNTEXT) && !mt_invalid(pair.start) {
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
        if mark.flags.has(MtFlags::DECOR_SIGNTEXT) && !mt_invalid(mark) && !mt_end(mark) {
            let end: MTPos = walk.altpos(mark);
            for i in mark.pos.row..=row2.min(end.row) {
                count[(i - row1) as usize] += 1;
            }
        }
        walk.step_filter(row2 + 1, 0, &SIGNTEXT_FILTER);
    }

    for &rowcount in &count {
        let prevwidth = SIGN_SHOW_MAX.min(rowcount - add);
        if half != SignCountHalf::Add && prevwidth > 0 {
            let slot = &mut buffer.b_signcols.count[(prevwidth - 1) as usize];
            *slot -= 1;
            // TODO(bfredl): correct marktree splicing so that this doesn't fail
            debug_assert!(*slot >= 0);
        }
        let width = SIGN_SHOW_MAX.min(rowcount);
        if half != SignCountHalf::Subtract && width > 0 {
            buffer.b_signcols.count[(width - 1) as usize] += 1;
            if width > buffer.b_signcols.max {
                buffer.b_signcols.max = width;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One row's worth of signs, sorted the way `sign_item_cmp` sorts them,
    /// as `(priority, mark id, placement serial)` triples.
    fn sorted(
        mut signs: Vec<(DecorPriority, uint32_t, c_int)>,
    ) -> Vec<(DecorPriority, uint32_t, c_int)> {
        signs.sort_by_key(|a| sign_rank(a.0, a.1, a.2));
        signs
    }

    #[test]
    fn the_highest_priority_sign_comes_first() {
        assert_eq!(
            vec![(20, 1, 1), (10, 2, 2), (0, 3, 3)],
            sorted(vec![(10, 2, 2), (0, 3, 3), (20, 1, 1)])
        );
    }

    /// Equal priority: the newest mark id wins the leftmost column.
    #[test]
    fn the_mark_id_breaks_a_priority_tie_newest_first() {
        assert_eq!(
            vec![(10, 9, 1), (10, 5, 1), (10, 1, 1)],
            sorted(vec![(10, 1, 1), (10, 9, 1), (10, 5, 1)])
        );
    }

    /// Equal priority *and* id -- which only `nvim_buf_set_extmark` can
    /// produce, since `:sign place` hands out distinct ids -- falls through
    /// to the placement serial, also newest first.
    #[test]
    fn the_placement_serial_breaks_the_last_tie() {
        assert_eq!(
            vec![(10, 1, 7), (10, 1, 3)],
            sorted(vec![(10, 1, 3), (10, 1, 7)])
        );
    }

    /// The comparator is a total order, which is what lets `sort_signs` use
    /// a stable sort and still be the permutation `qsort` produced.
    #[test]
    fn only_an_identical_triple_ties() {
        let a = (10, 1, 1);
        assert_eq!(
            Ordering::Equal,
            sign_rank(a.0, a.1, a.2).cmp(&sign_rank(a.0, a.1, a.2))
        );
        for b in [(11, 1, 1), (10, 2, 1), (10, 1, 2)] {
            assert_eq!(
                Ordering::Greater,
                sign_rank(a.0, a.1, a.2).cmp(&sign_rank(b.0, b.1, b.2)),
                "{a:?} must sort after {b:?}"
            );
        }
    }
}
