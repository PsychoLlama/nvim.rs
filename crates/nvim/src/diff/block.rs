//! The diff block list, and which buffers are in it.
//!
//! A tabpage owns a list of `DiffBlock` blocks, in line order, each naming a
//! line range in every one of the (up to eight) buffers `tp_diffbuf` holds.
//! This file owns both halves: [`diff_buf_add`]/[`diff_buf_delete`]/
//! [`TabPage::diff_index`] are the registry, and [`TabPage::with_diff`]/
//! [`diff_check_sanity`] the list.
//!
//! [`diff_mark_adjust_tp`] is the one that keeps the list correct across an
//! edit without recomputing it -- and it is only ever *read* under the
//! external diff, because `diff_internal()` makes the tabpage invalid
//! instead.  It still runs, because `:%diffput` needs the marks.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::semsg;
use crate::types::Failed;
use crate::winlayer::{Buf, BufId, TabPage, Win, tabs, windows};

/// A copy of a tab page's `tp_diffbuf`, for code holding the block list on
/// loan: which slots are in use, and their buffers' ids.
pub(crate) type DiffSlots = [Option<BufId>; DB_COUNT as usize];
use core::ffi::c_int;

impl TabPage {
    /// The buffer in diff slot `idx`, `None` for an empty slot -- or for
    /// one whose buffer has gone, which `diff_buf_delete` should have
    /// emptied but which an id answers for safely either way.
    ///
    /// # Panics
    ///
    /// When `idx` is not a diff slot.
    #[inline]
    pub(crate) fn diffbuf(self, idx: usize) -> Option<Buf> {
        self.tp_diffbuf[idx].and_then(BufId::get)
    }

    /// [`TabPage::diffbuf`] for a slot the caller has already found in
    /// use.
    ///
    /// # Panics
    ///
    /// When the slot is empty.
    #[inline]
    pub(crate) fn used_diffbuf(self, idx: usize) -> Buf {
        self.diffbuf(idx)
            .expect("a diff slot the caller found in use")
    }

    /// `buffer`'s slot in this tab page's diff, or `DB_COUNT` if it has
    /// none. Compares ids only, so it is safe to ask about a buffer the
    /// caller is not otherwise sure of.
    ///
    /// The id is taken once and the slots compared as plain words:
    /// `mark_adjust` asks this of every tab page on every edit.
    pub(crate) fn diff_index(self, buffer: Buf) -> c_int {
        let Some(id) = buffer.try_id() else {
            return DB_COUNT;
        };
        self.tp_diffbuf
            .iter()
            .position(|&slot| slot == Some(id))
            .map_or(DB_COUNT, |i| c_int::try_from(i).expect("eight slots"))
    }

    /// Whether diff slot `idx` holds `buffer`. A buffer without a number
    /// is in no slot.
    #[inline]
    pub(crate) fn diff_slot_holds(self, idx: usize, buffer: Buf) -> bool {
        buffer
            .try_id()
            .is_some_and(|id| self.tp_diffbuf[idx] == Some(id))
    }

    /// Lend the tab page's block list to `f`.
    ///
    /// The loan is the field and nothing else: `f` must not reach the tab
    /// page (or anything that reads its blocks) while it holds the list,
    /// which is why the closures handed to this are the pure list operations
    /// and buffer reads, and why the slot table they need is copied out
    /// before the call. A loan that changes how many blocks there are bumps
    /// `tp_diff_gen`.
    #[inline(always)]
    pub(crate) fn with_diff<R>(self, f: impl FnOnce(&mut Vec<DiffBlock>) -> R) -> R {
        let mut tabpage = self;
        let blocks = &mut tabpage.tp_diff;
        let before = blocks.len();
        let r = f(blocks);
        if blocks.len() != before {
            tabpage.tp_diff_gen = tabpage.tp_diff_gen.wrapping_add(1);
        }
        r
    }
}

impl DiffBlock {
    /// The block's longest side, in lines: how many screen rows it takes in
    /// every window showing it, the shorter buffers padded with filler.
    pub(crate) fn max_len(&self, slots: &DiffSlots) -> c_int {
        (0..DB_COUNT as usize)
            .filter(|&k| slots[k].is_some())
            .map(|k| self.df_count[k])
            .max()
            .unwrap_or(0)
    }
}

/// Take `buffer` out of every tabpage's diff.
pub fn diff_buf_delete(buffer: Buf) {
    for mut tp in tabs() {
        let i = tp.diff_index(buffer);
        if i != DB_COUNT {
            let i = usize::try_from(i).expect("a diff-buffer index is never negative");
            tp.tp_diffbuf[i] = None;
            tp.tp_diff_invalid = 1;
            if tp.is_current() {
                need_diff_redraw.set(true);
                redraw_later(Win::current(), UPD_VALID);
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
    let mut tp = TabPage::current();
    let i = tp.diff_index(win.buffer());
    if i != DB_COUNT {
        let i = usize::try_from(i).expect("a diff-buffer index is never negative");
        tp.tp_diffbuf[i] = None;
        tp.tp_diff_invalid = 1;
        diff_redraw(true);
    }
}

/// Put `buffer` in the current tabpage's diff, if there is a slot free.
pub fn diff_buf_add(buffer: Buf) {
    let mut tp = TabPage::current();
    if tp.diff_index(buffer) != DB_COUNT {
        return;
    }
    for i in 0..DB_COUNT as usize {
        if tp.tp_diffbuf[i].is_none() {
            tp.tp_diffbuf[i] = Some(buffer.id());
            tp.tp_diff_invalid = 1;
            diff_redraw(true);
            return;
        }
    }
    semsg!("E96: Cannot diff more than {} buffers", DB_COUNT);
}

/// Empty the current tabpage's diff.
pub(crate) fn diff_buf_clear() {
    let mut tp = TabPage::current();
    for i in 0..DB_COUNT as usize {
        if tp.tp_diffbuf[i].is_some() {
            tp.tp_diffbuf[i] = None;
            tp.tp_diff_invalid = 1;
            diff_redraw(true);
        }
    }
}

/// Mark every tabpage `buffer` is diffed in as needing a recompute.
pub fn diff_invalidate(buffer: Buf) {
    for mut tp in tabs() {
        if tp.diff_index(buffer) != DB_COUNT {
            tp.tp_diff_invalid = 1;
            if tp.is_current() {
                diff_redraw(true);
            }
        }
    }
}

/// Adjust every tabpage's block list for an edit in `buffer`.
///
/// The parameters are `mark_adjust`'s: lines `line1`..`line2` moved by
/// `amount`, everything below by `amount_after`.
pub fn diff_mark_adjust(
    buffer: Buf,
    line1: LineNr,
    line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
) {
    for tp in tabs() {
        let idx = tp.diff_index(buffer);
        if idx != DB_COUNT {
            diff_mark_adjust_tp(tp, idx, line1, line2, amount, amount_after);
        }
    }
}

/// The edit `mark_adjust`'s four numbers describe, as the two counts every
/// case below is written in terms of.
fn inserted_deleted(line2: LineNr, amount: LineNr, amount_after: LineNr) -> (LineNr, LineNr) {
    if line2 == MAXLNUM {
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
fn diff_mark_adjust_tp(
    mut tabpage: TabPage,
    idx: c_int,
    line1: LineNr,
    line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
) {
    if diff_internal() != 0 {
        // The blocks will be recomputed before the next redraw, so
        // nothing below survives; `_update` also gets the folds redone.
        // The *marks* are still adjusted here, which `:%diffput` needs.
        tabpage.tp_diff_invalid = 1;
        tabpage.tp_diff_update = 1;
    }
    let idx = usize::try_from(idx).expect("a diff-buffer index is never negative");
    let edit = Edit {
        idx,
        line1,
        line2,
        amount,
        amount_after,
        busy: diff_busy.get(),
    };
    let slots = tabpage.tp_diffbuf;
    let reshaped = tabpage.with_diff(|blocks| edit.apply(blocks, &slots));
    if reshaped {
        tabpage.tp_diff_gen = tabpage.tp_diff_gen.wrapping_add(1);
    }

    if tabpage.is_current() {
        // Not right away: this runs per edit, and redrawing is slow.
        need_diff_redraw.set(true);
        // The filler lines may have moved, so the scroll binding has to
        // be recomputed -- also postponed until the redraw.
        diff_need_scrollbind.set(true);
    }
}

/// One edit in buffer `idx`, as [`diff_mark_adjust_tp`] merges it into the
/// block list.
struct Edit {
    idx: usize,
    line1: LineNr,
    line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
    /// `:diffget`/`:diffput` is walking the list and doing its own
    /// bookkeeping: only line numbers may move.
    busy: bool,
}

impl Edit {
    /// Slide the *other* buffers' ranges of `dp` by the edit: `off` is how
    /// far the block's start moved up, `n` how many lines they gain -- which
    /// is how a deletion in one buffer becomes a change in the rest.
    fn adjust_others(&self, dp: &mut DiffBlock, slots: &DiffSlots, off: LineNr, n: LineNr) {
        for i in used_slots(slots).filter(|&i| i != self.idx) {
            dp.df_lnum[i] = (dp.df_lnum[i] - off).max(1);
            dp.df_count[i] += n;
        }
    }

    /// Merge the edit into `blocks`. Answers whether a block was added or
    /// taken out, which the caller files as a change of the list's shape
    /// even where the count came out the same.
    fn apply(&self, blocks: &mut Vec<DiffBlock>, slots: &DiffSlots) -> bool {
        let &Self {
            idx,
            line1,
            line2,
            amount,
            amount_after,
            busy,
        } = self;
        let (inserted, mut deleted) = inserted_deleted(line2, amount, amount_after);
        let mut reshaped = false;

        // `prev` is the block the walk last stepped past; a block the walk
        // itself inserts goes in front of `at` without becoming it.
        let mut prev: Option<usize> = None;
        let mut at = 0;
        let mut lnum_deleted = line1; // lnum of the remaining deletion
        loop {
            // The edit falls between two blocks, touching neither: it is a
            // change of its own. Not while `ex_diffgetput` is walking the
            // list, which is doing its own bookkeeping.
            if blocks.get(at).is_none_or(|dp| {
                dp.df_lnum[idx] - 1 > line2 || line2 == MAXLNUM && dp.df_lnum[idx] > line1
            }) && prev.is_none_or(|p| blocks[p].end(idx) < line1)
                && !busy
            {
                let mut dnext = DiffBlock::default();
                dnext.df_lnum[idx] = line1;
                dnext.df_count[idx] = inserted;
                for i in used_slots(slots).filter(|&i| i != idx) {
                    // The other buffers' line numbers carry the drift the
                    // previous block left behind.
                    dnext.df_lnum[i] = match prev {
                        None => line1,
                        Some(p) => line1 + blocks[p].end(i) - blocks[p].end(idx),
                    };
                    dnext.df_count[i] = deleted;
                }
                blocks.insert(at, dnext);
                reshaped = true;
                at += 1;
            }
            if at == blocks.len() {
                break;
            }

            let next_lnum = blocks.get(at + 1).map(|next| next.df_lnum[idx]);
            let dp = &mut blocks[at];
            let last = dp.end(idx) - 1;
            // 1. The block is entirely above the edit: nothing to do.
            if last >= line1 - 1 {
                if busy {
                    // Mid-update: only the line numbers may move.
                    if dp.df_lnum[idx] > line2 {
                        dp.df_lnum[idx] += amount_after;
                    }
                    prev = Some(at);
                    at += 1;
                    continue;
                }
                // 6. The block is below the edit: shift it. The `!= 0` test
                // covers a deletion that emptied everything between two
                // blocks, leaving nothing to merge.
                if dp.df_lnum[idx] - c_int::from(deleted + inserted != 0) > line2 {
                    if amount_after == 0 {
                        break; // nothing left to change
                    }
                    dp.df_lnum[idx] += amount_after;
                } else {
                    // The trim runs *after* the other buffers are adjusted,
                    // because it compares the block's lines across all of
                    // them.
                    let mut check_unchanged = false;
                    if deleted > 0 {
                        // 2. 3. 4. 5.: the deletion overlaps this block.
                        let mut off = 0;
                        let n;
                        // Does the deletion run on into the next block? Then
                        // only the lines up to its first are this block's.
                        let spills = next_lnum.filter(|&lnum| lnum - 1 <= line2);
                        if dp.df_lnum[idx] >= line1 {
                            if last <= line2 {
                                // 4. every line of the block goes.
                                if let Some(next) = spills {
                                    n = next - lnum_deleted - dp.df_count[idx];
                                    deleted -= next - lnum_deleted;
                                    lnum_deleted = next;
                                } else {
                                    n = deleted - dp.df_count[idx];
                                }
                                dp.df_count[idx] = 0;
                            } else {
                                // 5. lines go at or just before its top.
                                off = dp.df_lnum[idx] - lnum_deleted;
                                n = off;
                                dp.df_count[idx] -= line2 - dp.df_lnum[idx] + 1;
                                check_unchanged = true;
                            }
                            dp.df_lnum[idx] = line1;
                        } else if last < line2 {
                            // 2. lines go at the end of the block.
                            dp.df_count[idx] -= last - lnum_deleted + 1;
                            if let Some(next) = spills {
                                n = next - 1 - last;
                                deleted -= next - lnum_deleted;
                                lnum_deleted = next;
                            } else {
                                n = line2 - last;
                            }
                            check_unchanged = true;
                        } else {
                            // 3. lines go from inside the block.
                            n = 0;
                            dp.df_count[idx] -= deleted;
                        }
                        self.adjust_others(dp, slots, off, n);
                    } else if dp.df_lnum[idx] <= line1 {
                        // Lines inserted inside this block.
                        dp.df_count[idx] += inserted;
                        check_unchanged = true;
                    } else {
                        // Lines inserted above it.
                        dp.df_lnum[idx] += inserted;
                    }
                    if check_unchanged {
                        // The inserted lines may equal what was there, which
                        // makes the block smaller.
                        diff_check_unchanged(slots, dp);
                    }
                }
            }

            // Fold the block into the previous one if they now touch, else
            // step past it.
            match prev {
                Some(p)
                    if !blocks[at].is_linematched
                        && !busy
                        && blocks[p].end(idx) == blocks[at].df_lnum[idx] =>
                {
                    let dp = blocks.remove(at);
                    reshaped = true;
                    for i in used_slots(slots) {
                        blocks[p].df_count[i] += dp.df_count[i];
                    }
                }
                _ => {
                    prev = Some(at);
                    at += 1;
                }
            }
        }

        // A block every buffer now has nothing in is not a change any more.
        let before = blocks.len();
        blocks.retain(|dp| {
            !(0..DB_COUNT as usize).all(|i| slots[i].is_none() || dp.df_count[i] == 0)
        });
        reshaped || blocks.len() != before
    }
}

/// Shrink `dp` from both ends while its first (or last) lines are equal in
/// every buffer.
///
/// An edit can leave a block claiming lines that did not actually change; the
/// diff is not recomputed for that, so the block is trimmed instead.
fn diff_check_unchanged(slots: &DiffSlots, dp: &mut DiffBlock) {
    let Some(i_org) = (0..DB_COUNT as usize).find(|&i| slots[i].is_some()) else {
        return;
    };
    if diff_check_sanity(slots, dp).is_err() {
        return;
    }
    for dir in [FORWARD as c_int, BACKWARD as c_int] {
        while dp.df_count[i_org] > 0 {
            let off_org = if dir == BACKWARD as c_int {
                dp.df_count[i_org] - 1
            } else {
                0
            };
            // A copy: the loop below reads the other buffers, and one of
            // them may be this one again on a later turn.
            let line_org =
                Lines::in_buffer(used_slot(slots, i_org)).line_copy(dp.df_lnum[i_org] + off_org);
            let mut i_new = i_org + 1;
            while i_new < DB_COUNT as usize {
                if slots[i_new].is_some() {
                    let off_new = if dir == BACKWARD as c_int {
                        dp.df_count[i_new] - 1
                    } else {
                        0
                    };
                    if off_new < 0 || off_new >= dp.df_count[i_new] {
                        break;
                    }
                    let mut other = Lines::in_buffer(used_slot(slots, i_new));
                    let lnum = dp.df_lnum[i_new] + off_new;
                    if !lines_equal(&line_org, other.line(lnum)) {
                        break;
                    }
                }
                i_new += 1;
            }
            if i_new != DB_COUNT as usize {
                break; // some buffer differs here; the block starts (or ends) for real
            }
            for i in used_slots(slots).filter(|&i| i >= i_org) {
                if dir == FORWARD as c_int {
                    dp.df_lnum[i] += 1;
                }
                dp.df_count[i] -= 1;
            }
        }
    }
}

/// A slot number [`TabPage::diff_index`] answered, once the caller has
/// ruled out `DB_COUNT`.
///
/// # Panics
///
/// When `idx` is negative, which no slot number is.
pub(crate) fn slot_index(idx: c_int) -> usize {
    usize::try_from(idx).expect("a diff-buffer index is never negative")
}

/// The slots of `slots` in use, in order.
pub(crate) fn used_slots(slots: &DiffSlots) -> impl Iterator<Item = usize> + '_ {
    slots
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| slot.is_some().then_some(i))
}

/// The buffer in a slot the caller has already found in use.
/// [`TabPage::used_diffbuf`], for a copy of the slot table.
///
/// # Panics
///
/// When the slot is empty, or its buffer has gone.
pub(crate) fn used_slot(slots: &DiffSlots, idx: usize) -> Buf {
    slots[idx]
        .and_then(BufId::get)
        .expect("a diff slot the caller found in use")
}

/// Whether every buffer's range in `dp` is still inside that buffer.
///
/// An edit can leave a block naming lines that no longer exist, and every
/// reader of a block has to check first.
pub(crate) fn diff_check_sanity(slots: &DiffSlots, dp: &DiffBlock) -> Result<(), Failed> {
    for (i, slot) in slots.iter().enumerate() {
        if let Some(buf) = slot.and_then(BufId::get)
            && dp.end(i) - 1 > buf.b_ml.ml_line_count
        {
            return Err(Failed);
        }
    }
    Ok(())
}

/// Give buffer `idx_new` of `blocks[at]` the same range as `idx_orig`,
/// corrected for the drift the block before it left behind.
pub(crate) fn diff_copy_entry(
    blocks: &mut [DiffBlock],
    at: usize,
    idx_orig: usize,
    idx_new: usize,
) {
    let (before, rest) = blocks.split_at_mut(at);
    let off = before
        .last()
        .map_or(0, |prev| prev.end(idx_orig) - prev.end(idx_new));
    let dp = &mut rest[0];
    dp.df_lnum[idx_new] = dp.df_lnum[idx_orig] - off;
    dp.df_count[idx_new] = dp.df_count[idx_orig];
}

/// Empty `tabpage`'s block list.
pub fn diff_clear(mut tabpage: TabPage) {
    tabpage.tp_diff.clear();
    tabpage.tp_diff_gen = tabpage.tp_diff_gen.wrapping_add(1);
}

/// Whether `buffer` is in any tabpage's diff.
pub fn diff_mode_buf(buffer: Buf) -> bool {
    tabs().any(|tp| tp.diff_index(buffer) != DB_COUNT)
}
