//! Rewriting every mark store when the buffer's lines move.
//!
//! `mark_adjust_buf` is on the path of every `:d`, `:m`, `:put`, undo and API
//! line splice, and it has to visit *every* store the editor keeps: the
//! buffer's own marks, the global table, the change list, the visual range,
//! the quickfix and location lists, every window's jump list, tag stack,
//! topline and cursor, the folds, the diffs, and the remembered per-window
//! positions. What it does to each is one of three rules — [`LineShift`]'s
//! `line`, `line_nodel` and `cursor`, upstream's `ONE_ADJUST`,
//! `ONE_ADJUST_NODEL` and `ONE_ADJUST_CURSOR` — and *which* rule a store gets
//! is a per-store decision that nothing but a differential can check. It is
//! written out one store per line here for that reason.
//!
//! [`mark_col_adjust`] is the same shape one dimension over: when text on a
//! line moves sideways, every mark at or after a column moves with it. Its
//! three callers are `ops/join.rs`, `textformat/`'s wrap and
//! `textformat/lines.rs`. Note that it carries a `:lockmarks` short-circuit of
//! its **own**, separate from `mark_adjust_buf`'s — the two guard different
//! functions and a rewrite that merges the line half must not drop the column
//! half.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::buffer::buf_is_prompt;
use crate::diff::diff_mark_adjust;
use crate::ex_docmd::cmdmod_has;
use crate::extmark::extmark_adjust;
use crate::fold::fold_mark_adjust;
use crate::main::{curbuf, saved_cursor};
use crate::pos::{MAXLNUM, equalpos};
use crate::winlayer::{Buf, Win, tab_windows, windows};
use core::ffi::{c_int, c_uint};

use super::store::{Fmark, GlobalMarks};
use super::*;
use crate::types::CmdModFlags;

/// Where `clrallmarks` leaves `b_last_cursor`, and therefore what
/// "`'"` has never really been set" looks like to the adjuster.
const INIT_POS: pos_T = pos_T {
    lnum: 1,
    col: 0,
    coladd: 0,
};

/// `mark.c`'s `ONE_ADJUST` family: the line-number rewrite every mark store
/// gets when lines are inserted, deleted or moved. Marks in `line1..=line2`
/// move by `amount`; `amount == MAXLNUM` means those lines are gone. Marks
/// past `line2` move by `amount_after`.
#[derive(Copy, Clone)]
struct LineShift {
    line1: linenr_T,
    line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
}

impl LineShift {
    /// `ONE_ADJUST`: a deleted mark is invalidated.
    fn line(self, lp: &mut linenr_T) {
        if *lp >= self.line1 && *lp <= self.line2 {
            *lp = if self.amount == MAXLNUM.cast_signed() {
                0
            } else {
                *lp + self.amount
            };
        } else if self.amount_after != 0 && *lp > self.line2 {
            *lp += self.amount_after;
        }
    }

    /// `ONE_ADJUST_NODEL`: a deleted mark lands on the first deleted line
    /// rather than being invalidated.
    fn line_nodel(self, lp: &mut linenr_T) {
        if *lp >= self.line1 && *lp <= self.line2 {
            *lp = if self.amount == MAXLNUM.cast_signed() {
                self.line1
            } else {
                *lp + self.amount
            };
        } else if self.amount_after != 0 && *lp > self.line2 {
            *lp += self.amount_after;
        }
    }

    /// `ONE_ADJUST_CURSOR`: a cursor inside the deleted range moves to the
    /// start of the line before it.
    fn cursor(self, posp: &mut pos_T) {
        if posp.lnum >= self.line1 && posp.lnum <= self.line2 {
            if self.amount == MAXLNUM.cast_signed() {
                posp.lnum = (self.line1 - 1).max(1);
                posp.col = 0;
            } else {
                posp.lnum += self.amount;
            }
        } else if self.amount_after != 0 && posp.lnum > self.line2 {
            posp.lnum += self.amount_after;
        }
    }

    /// [`LineShift::line`] applied to a mark store.
    fn mark(self, fm: Fmark) {
        let mut lnum = fm.lnum();
        self.line(&mut lnum);
        fm.set_lnum(lnum);
    }

    /// [`LineShift::line_nodel`] applied to a mark store.
    fn mark_nodel(self, fm: Fmark) {
        let mut lnum = fm.lnum();
        self.line_nodel(&mut lnum);
        fm.set_lnum(lnum);
    }

    /// [`LineShift::line_nodel`], but only for a mark that names buffer
    /// `fnum`. The global table, every jump list and every tag stack hold
    /// marks in other buffers, which this adjustment must not touch.
    fn mark_nodel_in(self, fm: Fmark, fnum: c_int) {
        if fm.fnum() == fnum {
            self.mark_nodel(fm);
        }
    }
}

/// `mark_col_adjust`'s `COL_ADJUST`: the column rewrite a mark on line
/// `lnum` at or after `mincol` gets when text on that line moves.
#[derive(Copy, Clone)]
struct ColShift {
    lnum: linenr_T,
    mincol: colnr_T,
    lnum_amount: linenr_T,
    col_amount: colnr_T,
    spaces_removed: c_int,
}

impl ColShift {
    fn col(self, posp: &mut pos_T) {
        if posp.lnum != self.lnum || posp.col < self.mincol {
            return;
        }
        posp.lnum += self.lnum_amount;
        if self.col_amount < 0 && posp.col <= -self.col_amount {
            posp.col = 0;
        } else if posp.col < self.spaces_removed {
            posp.col = self.col_amount + self.spaces_removed;
        } else {
            posp.col += self.col_amount;
        }
    }

    /// [`ColShift::col`] applied to a mark store.
    fn mark(self, fm: Fmark) {
        let mut pos = fm.pos();
        self.col(&mut pos);
        fm.set_pos(pos);
    }

    /// [`ColShift::col`], but only for a mark that names buffer `fnum`.
    fn mark_in(self, fm: Fmark, fnum: c_int) {
        if fm.fnum() == fnum {
            self.mark(fm);
        }
    }
}

/// Adjust marks between "line1" and "line2" (inclusive) to move "amount" lines.
/// Must be called before changed_*(), appended_lines() or deleted_lines().
/// May be called before or after changing the text.
/// When deleting lines "line1" to "line2", use an "amount" of MAXLNUM: The
/// marks within this range are made invalid.
/// If "amount_after" is non-zero adjust marks after "line2".
/// Example: Delete lines 34 and 35: mark_adjust(34, 35, MAXLNUM, -2);
/// Example: Insert two lines below 55: mark_adjust(56, MAXLNUM, 2, 0);
/// or: mark_adjust(56, 55, MAXLNUM, 2);
///
/// # Safety
/// The editor's globals must be live, which they are from startup to exit.
pub unsafe fn mark_adjust(
    line1: linenr_T,
    line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
    op: ExtmarkOp,
) {
    // SAFETY: forwarded from the caller; `curbuf` is live from startup.
    unsafe {
        mark_adjust_buf(
            curbuf.get(),
            line1,
            line2,
            amount,
            amount_after,
            true,
            kMarkAdjustNormal,
            op,
        )
    };
}

/// mark_adjust_nofold() does the same as mark_adjust() but without adjusting
/// folds in any way. Folds must be adjusted manually by the caller.
/// This is only useful when folds need to be moved in a way different to
/// calling fold_mark_adjust() with arguments line1, line2, amount, amount_after,
/// for an example of why this may be necessary, see do_move().
///
/// # Safety
/// As [`mark_adjust`].
pub unsafe fn mark_adjust_nofold(
    line1: linenr_T,
    line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
    op: ExtmarkOp,
) {
    // SAFETY: forwarded from the caller.
    unsafe {
        mark_adjust_buf(
            curbuf.get(),
            line1,
            line2,
            amount,
            amount_after,
            false,
            kMarkAdjustNormal,
            op,
        )
    };
}

/// # Safety
/// `buf` must be a live buffer, and the editor's window and tab page lists
/// must be live.
pub unsafe fn mark_adjust_buf(
    buf: *mut buf_T,
    line1: linenr_T,
    line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
    adjust_folds: bool,
    mode: MarkAdjustMode,
    op: ExtmarkOp,
) {
    // An empty range with nothing to shift after it is the no-op the callers
    // rely on: `mark_adjust(56, 55, ...)` with `amount_after == 0` reaches
    // here from every unchanged splice.
    if line2 < line1 && amount_after == 0 {
        return;
    }

    // SAFETY: the caller promised a live buffer.
    let mut buf = unsafe { Buf::new(buf) };
    let fnum = buf.handle as c_int;
    let shift = LineShift {
        line1,
        line2,
        amount,
        amount_after,
    };
    let by_api = mode as c_uint == kMarkAdjustApi as c_uint;
    let by_term = mode as c_uint == kMarkAdjustTerm as c_uint;

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        // `'a`-`'z` are invalidated when their line goes; the global table's
        // `'A`-`'Z0`-`'9` land on the first deleted line instead, because a
        // global mark is the user's bookmark and losing it outright is worse
        // than moving it. Both halves of the table are walked, but only the
        // slots naming *this* buffer are touched.
        for i in 0..NMARKS {
            shift.mark(buf.named_mark(i));
            shift.mark_nodel_in(GlobalMarks::at(i).fmark(), fnum);
        }
        for i in NMARKS..NGLOBALMARKS {
            shift.mark_nodel_in(GlobalMarks::at(i).fmark(), fnum);
        }

        shift.mark(buf.last_insert());
        shift.mark(buf.last_change());
        // `'"` is skipped while it still sits where `clrallmarks` left it —
        // shifting that would invent a position the user never visited — and,
        // for a terminal buffer, while it names the last line, which the
        // terminal is about to rewrite anyway.
        if !equalpos(buf.last_cursor().pos(), INIT_POS)
            && (!by_term || buf.last_cursor().lnum() < buf.b_ml.ml_line_count)
        {
            shift.mark(buf.last_cursor());
        }
        if buf_is_prompt(Some(buf)) {
            shift.mark_nodel(buf.prompt_start());
        }
        for change in buf.changes() {
            shift.mark_nodel(change);
        }
        shift.line_nodel(&mut buf.b_visual.vi_start.lnum);
        shift.line_nodel(&mut buf.b_visual.vi_end.lnum);

        // The quickfix list is asked once for the buffer and then once per
        // window for that window's location list; a buffer with no surviving
        // entry in either loses the corresponding flag.
        if !qf_mark_adjust(buf, None, line1, line2, amount, amount_after) {
            buf.b_has_qf_entry &= !BUF_HAS_QF_ENTRY;
        }
        let mut found_one = false;
        for win in tab_windows() {
            found_one |= qf_mark_adjust(buf, Some(win), line1, line2, amount, amount_after);
        }
        if !found_one {
            buf.b_has_qf_entry &= !BUF_HAS_LL_ENTRY;
        }
    }

    if op as c_uint != kExtmarkNOOP as c_uint {
        // SAFETY: `buf` is live.
        unsafe { extmark_adjust(buf.raw(), line1, line2, amount, amount_after, op) };
    }

    // The context marks and the saved cursor belong to the current window
    // rather than to `buf`, so they only move when the two agree. They are
    // NOT under the `:lockmarks` guard above — upstream leaves them out, and
    // `:lockmarks` is documented as being about the *named* marks.
    // SAFETY: `curwin` is live from startup to exit.
    let mut curwin_handle = unsafe { Win::current() };
    if curwin_handle.w_buffer == buf.raw() {
        shift.line(&mut curwin_handle.w_pcmark.lnum);
        shift.line(&mut curwin_handle.w_prev_pcmark.lnum);
        let mut saved = saved_cursor.get();
        if saved.lnum != 0 {
            shift.line_nodel(&mut saved.lnum);
            saved_cursor.set(saved);
        }
    }

    for mut win in tab_windows() {
        if !cmdmod_has(CmdModFlags::LOCKMARKS) {
            for jump in win.jumps() {
                shift.mark_nodel_in(jump.fmark(), fnum);
            }
        }
        if win.w_buffer != buf.raw() {
            continue;
        }
        if !cmdmod_has(CmdModFlags::LOCKMARKS) {
            for tag in win.tag_marks() {
                shift.mark_nodel_in(tag, fnum);
            }
        }
        // The remembered Visual range of a window that is not the current
        // one; the two move together or not at all.
        if win.w_old_cursor_lnum != 0 {
            shift.line_nodel(&mut win.w_old_cursor_lnum);
            shift.line_nodel(&mut win.w_old_visual_lnum);
        }

        // Which windows follow the change with their view: an API splice
        // moves every window's, a terminal one moves those not already
        // parked on the last line, and an ordinary edit moves every window
        // except the one the user is typing in — whose topline
        // `update_topline` will work out for itself.
        // `ml_line_count` is re-read here rather than hoisted: the walk above
        // this one calls out to `qf_mark_adjust`, `extmark_adjust` and
        // `fold_mark_adjust`, and upstream reads the field afresh at each of
        // these three tests.
        if by_api || follows(win, by_term, buf) {
            if win.w_topline >= line1 && win.w_topline <= line2 {
                if amount == MAXLNUM.cast_signed() {
                    // An API splice that *replaces* the topline's range with
                    // at least as many lines leaves the topline where it is.
                    if !(by_api && amount_after > line1 - line2 - 1) {
                        win.w_topline = (line1 - 1).max(1);
                    }
                } else if win.w_topline > line1 {
                    win.w_topline += amount;
                }
                win.w_topfill = 0;
            } else if amount_after != 0
                && win.w_topline > line2 + c_int::from(by_api && line2 < line1)
            {
                win.w_topline += amount_after;
                win.w_topfill = 0;
            }
        }
        // The cursor is the one store an API splice leaves alone: the API
        // contract is that a splice does not move the user's cursor.
        if !by_api && follows(win, by_term, buf) {
            let mut cursor = win.w_cursor;
            shift.cursor(&mut cursor);
            win.w_cursor = cursor;
        }
        if adjust_folds {
            // SAFETY: `win` came out of the editor's own window list.
            fold_mark_adjust(win, line1, line2, amount, amount_after);
        }
    }

    // SAFETY: `buf` is live and the tab page list is the editor's own.
    diff_mark_adjust(buf, line1, line2, amount, amount_after);

    // The per-window remembered cursor of every window that has ever shown
    // this buffer, including ones that no longer exist.
    for i in 0..buf.b_wininfo.size {
        // SAFETY: `b_wininfo` is a kvec of `size` live `WinInfo` pointers
        // owned by the buffer, so the entry and its `wi_mark` are live; the
        // handle reads and writes one position.
        let mark = unsafe { Fmark::new(&raw mut (**buf.b_wininfo.items.add(i)).wi_mark) };
        if !by_term || mark.lnum() < buf.b_ml.ml_line_count {
            let mut pos = mark.pos();
            shift.cursor(&mut pos);
            mark.set_pos(pos);
        }
    }
}

/// Whether `win`'s view follows a splice.
///
/// An ordinary edit moves every window's view except the current one's; a
/// terminal splice instead moves the views of the windows whose cursor is not
/// already on `buf`'s last line.
fn follows(win: Win, by_term: bool, buf: Buf) -> bool {
    if by_term {
        win.w_cursor.lnum < buf.b_ml.ml_line_count
    } else {
        !win.is_current()
    }
}

/// Adjust marks in line "lnum" at column "mincol" and further: add
/// "lnum_amount" to the line number and add "col_amount" to the column
/// position.
/// "spaces_removed" is the number of spaces that were removed, matters when the
/// cursor is inside them.
///
/// # Safety
/// The editor's globals must be live, which they are from startup to exit.
pub unsafe fn mark_col_adjust(
    lnum: linenr_T,
    mincol: colnr_T,
    lnum_amount: linenr_T,
    col_amount: colnr_T,
    spaces_removed: c_int,
) {
    // Upstream asserts this once per adjusted mark; `col_amount` does not
    // change, and the upper half is vacuous for a `colnr_T`. What it really
    // guards is `-col_amount` below.
    debug_assert!(col_amount > colnr_T::MIN, "col_amount > INT_MIN");
    // `mark_adjust_buf`'s `:lockmarks` guard is a DIFFERENT one, in a
    // different function: a line operation never reaches here and a column
    // one never reaches that. `1787242636-jmarkmutate.py`'s
    // `mark-col-lockmarks` is the anchor on this line specifically.
    if col_amount == 0 && lnum_amount == 0 || cmdmod_has(CmdModFlags::LOCKMARKS) {
        return;
    }

    // SAFETY: `curbuf` and `curwin` are live from startup to exit.
    let (mut buf, mut cur) = unsafe { (Buf::current(), Win::current()) };
    let fnum = buf.handle as c_int;
    let shift = ColShift {
        lnum,
        mincol,
        lnum_amount,
        col_amount,
        spaces_removed,
    };

    for i in 0..NMARKS {
        shift.mark(buf.named_mark(i));
        shift.mark_in(GlobalMarks::at(i).fmark(), fnum);
    }
    for i in NMARKS..NGLOBALMARKS {
        shift.mark_in(GlobalMarks::at(i).fmark(), fnum);
    }
    shift.mark(buf.last_insert());
    shift.mark(buf.last_change());
    if buf_is_prompt(Some(buf)) {
        shift.mark(buf.prompt_start());
    }
    for change in buf.changes() {
        shift.mark(change);
    }
    shift.col(&mut buf.b_visual.vi_start);
    shift.col(&mut buf.b_visual.vi_end);
    shift.col(&mut cur.w_pcmark);
    shift.col(&mut cur.w_prev_pcmark);
    let mut saved = saved_cursor.get();
    shift.col(&mut saved);
    saved_cursor.set(saved);

    // The current tab page's windows only. Upstream spells the head
    // `curtab == curtab ? firstwin : curtab->tp_firstwin`, a transpiler
    // tautology that always takes `firstwin`.
    for mut win in windows() {
        for jump in win.jumps() {
            shift.mark_in(jump.fmark(), fnum);
        }
        if win.w_buffer != buf.raw() {
            continue;
        }
        for tag in win.tag_marks() {
            shift.mark_in(tag, fnum);
        }
        if !win.is_current() {
            let mut cursor = win.w_cursor;
            shift.col(&mut cursor);
            win.w_cursor = cursor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `:56,57d` — delete two lines, shifting everything after them up.
    const DELETE_56_57: LineShift = LineShift {
        line1: 56,
        line2: 57,
        amount: MAXLNUM.cast_signed(),
        amount_after: -2,
    };

    /// `:56 insert two lines` — the second form `mark_adjust.c` documents,
    /// where the moved range is empty and `amount_after` does the work.
    const INSERT_TWO_BELOW_55: LineShift = LineShift {
        line1: 56,
        line2: 55,
        amount: MAXLNUM.cast_signed(),
        amount_after: 2,
    };

    fn at(lnum: linenr_T) -> pos_T {
        pos_T {
            lnum,
            col: 7,
            coladd: 0,
        }
    }

    #[test]
    fn a_deleted_mark_is_invalidated() {
        let mut lnum = 57;
        DELETE_56_57.line(&mut lnum);
        assert_eq!(lnum, 0);
    }

    #[test]
    fn a_deleted_mark_can_instead_land_on_the_first_deleted_line() {
        let mut lnum = 57;
        DELETE_56_57.line_nodel(&mut lnum);
        assert_eq!(lnum, 56);
    }

    #[test]
    fn a_deleted_cursor_moves_to_the_line_above_the_range() {
        let mut pos = at(57);
        DELETE_56_57.cursor(&mut pos);
        assert_eq!((pos.lnum, pos.col), (55, 0));
    }

    /// Deleting from line 1 leaves the cursor on line 1, not line 0.
    #[test]
    fn a_cursor_in_a_range_starting_at_line_one_stays_on_line_one() {
        let shift = LineShift {
            line1: 1,
            line2: 3,
            amount: MAXLNUM.cast_signed(),
            amount_after: -3,
        };
        let mut pos = at(2);
        shift.cursor(&mut pos);
        assert_eq!((pos.lnum, pos.col), (1, 0));
    }

    #[test]
    fn marks_past_the_range_shift_by_amount_after() {
        for shift in [DELETE_56_57, INSERT_TWO_BELOW_55] {
            let mut lnum = 100;
            shift.line(&mut lnum);
            assert_eq!(lnum, 100 + shift.amount_after);

            let mut pos = at(100);
            shift.cursor(&mut pos);
            assert_eq!((pos.lnum, pos.col), (100 + shift.amount_after, 7));
        }
    }

    #[test]
    fn marks_before_the_range_are_untouched() {
        for shift in [DELETE_56_57, INSERT_TWO_BELOW_55] {
            let mut lnum = 10;
            shift.line(&mut lnum);
            assert_eq!(lnum, 10);
        }
    }

    /// An insertion names its range with `amount`, not `amount_after`, so the
    /// marks *inside* it move.
    #[test]
    fn an_insertion_moves_the_marks_it_covers() {
        let shift = LineShift {
            line1: 56,
            line2: MAXLNUM.cast_signed(),
            amount: 2,
            amount_after: 0,
        };
        let mut lnum = 60;
        shift.line(&mut lnum);
        assert_eq!(lnum, 62);
    }

    /// The three rules over one deletion, side by side — which is the whole
    /// per-store decision `mark_adjust_buf` makes forty times.
    #[test]
    fn the_three_rules_disagree_only_inside_the_deleted_range() {
        let mut invalidated = 57;
        let mut landed = 57;
        let mut cursor = at(57);
        DELETE_56_57.line(&mut invalidated);
        DELETE_56_57.line_nodel(&mut landed);
        DELETE_56_57.cursor(&mut cursor);
        assert_eq!((invalidated, landed, cursor.lnum), (0, 56, 55));

        // Outside it they agree.
        for lnum in [10, 100] {
            let (mut a, mut b, mut c) = (lnum, lnum, at(lnum));
            DELETE_56_57.line(&mut a);
            DELETE_56_57.line_nodel(&mut b);
            DELETE_56_57.cursor(&mut c);
            assert_eq!((a, b), (c.lnum, c.lnum));
        }
    }

    fn col_shift(col_amount: colnr_T, spaces_removed: c_int) -> ColShift {
        ColShift {
            lnum: 5,
            mincol: 4,
            lnum_amount: 0,
            col_amount,
            spaces_removed,
        }
    }

    #[test]
    fn a_column_before_mincol_is_untouched() {
        let mut pos = pos_T {
            lnum: 5,
            col: 3,
            coladd: 0,
        };
        col_shift(10, 0).col(&mut pos);
        assert_eq!(pos.col, 3);
    }

    #[test]
    fn a_column_on_another_line_is_untouched() {
        let mut pos = pos_T {
            lnum: 6,
            col: 40,
            coladd: 0,
        };
        col_shift(10, 0).col(&mut pos);
        assert_eq!(pos.col, 40);
    }

    #[test]
    fn a_column_shifted_off_the_start_of_the_line_lands_on_column_zero() {
        let mut pos = at(5);
        col_shift(-9, 0).col(&mut pos);
        assert_eq!(pos.col, 0);
    }

    /// A mark inside the run of removed whitespace lands where that run
    /// started, which is `col_amount + spaces_removed` measured from the
    /// change rather than from the mark.
    #[test]
    fn a_column_inside_removed_whitespace_is_rebased_on_the_change() {
        let mut pos = at(5);
        col_shift(-3, 9).col(&mut pos);
        assert_eq!(pos.col, 6);
    }

    #[test]
    fn an_ordinary_column_shifts_and_the_line_moves_with_it() {
        let mut pos = at(5);
        ColShift {
            lnum: 5,
            mincol: 4,
            lnum_amount: 3,
            col_amount: 11,
            spaces_removed: 0,
        }
        .col(&mut pos);
        assert_eq!((pos.lnum, pos.col), (8, 18));
    }
}
