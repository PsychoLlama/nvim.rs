use crate::buffer::bt_prompt;
use crate::diff::diff_mark_adjust;
use crate::extmark::extmark_adjust;
use crate::fold::foldMarkAdjust;
use crate::global_cell::GlobalCell;
use crate::main::{cmdmod, curbuf, curtab, curwin, first_tabpage, firstwin, namedfm, saved_cursor};
use crate::pos::{MAXLNUM, equalpos};
use core::ffi::{c_int, c_uint};
use core::ptr;

use super::*;
use crate::types::CMOD_LOCKMARKS;

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
            *lp = if self.amount == MAXLNUM as c_int {
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
            *lp = if self.amount == MAXLNUM as c_int {
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
            if self.amount == MAXLNUM as c_int {
                posp.lnum = (self.line1 - 1).max(1);
                posp.col = 0;
            } else {
                posp.lnum += self.amount;
            }
        } else if self.amount_after != 0 && posp.lnum > self.line2 {
            posp.lnum += self.amount_after;
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
pub unsafe fn mark_adjust(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
    mut op: ExtmarkOp,
) {
    mark_adjust_buf(
        curbuf.get(),
        line1,
        line2,
        amount,
        amount_after,
        true,
        kMarkAdjustNormal,
        op,
    );
}

/// mark_adjust_nofold() does the same as mark_adjust() but without adjusting
/// folds in any way. Folds must be adjusted manually by the caller.
/// This is only useful when folds need to be moved in a way different to
/// calling foldMarkAdjust() with arguments line1, line2, amount, amount_after,
/// for an example of why this may be necessary, see do_move().
pub unsafe fn mark_adjust_nofold(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
    mut op: ExtmarkOp,
) {
    mark_adjust_buf(
        curbuf.get(),
        line1,
        line2,
        amount,
        amount_after,
        false,
        kMarkAdjustNormal,
        op,
    );
}

pub unsafe fn mark_adjust_buf(
    mut buf: *mut buf_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
    mut adjust_folds: bool,
    mut mode: MarkAdjustMode,
    mut op: ExtmarkOp,
) {
    let mut fnum: c_int = (*buf).handle as c_int;
    let shift = LineShift {
        line1,
        line2,
        amount,
        amount_after,
    };
    static initpos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 1,
        col: 0,
        coladd: 0,
    });
    if line2 < line1 && amount_after == 0 {
        return;
    }
    let mut by_api: bool = mode as c_uint == kMarkAdjustApi as c_int as c_uint;
    let mut by_term: bool = mode as c_uint == kMarkAdjustTerm as c_int as c_uint;
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as c_int == 0 {
        let mut i: c_int = 0;
        while i < NMARKS {
            shift.line(
                &mut (*(&raw mut (*buf).b_namedm as *mut fmark_T).offset(i as isize))
                    .mark
                    .lnum,
            );
            if (*namedfm.ptr())[i as usize].fmark.fnum == fnum {
                shift.line_nodel(
                    &mut (*(namedfm.ptr() as *mut xfmark_T).offset(i as isize))
                        .fmark
                        .mark
                        .lnum,
                );
            }
            i += 1;
        }
        let mut i_0: c_int = NMARKS;
        while i_0 < NGLOBALMARKS {
            if (*namedfm.ptr())[i_0 as usize].fmark.fnum == fnum {
                shift.line_nodel(
                    &mut (*(namedfm.ptr() as *mut xfmark_T).offset(i_0 as isize))
                        .fmark
                        .mark
                        .lnum,
                );
            }
            i_0 += 1;
        }
        shift.line(&mut (*buf).b_last_insert.mark.lnum);
        shift.line(&mut (*buf).b_last_change.mark.lnum);
        if !equalpos((*buf).b_last_cursor.mark, initpos.get())
            && (!by_term || (*buf).b_last_cursor.mark.lnum < (*buf).b_ml.ml_line_count)
        {
            shift.line(&mut (*buf).b_last_cursor.mark.lnum);
        }
        if bt_prompt(buf) {
            shift.line_nodel(&mut (*buf).b_prompt_start.mark.lnum);
        }
        let mut i_1: c_int = 0;
        while i_1 < (*buf).b_changelistlen {
            shift.line_nodel(
                &mut (*(&raw mut (*buf).b_changelist as *mut fmark_T).offset(i_1 as isize))
                    .mark
                    .lnum,
            );
            i_1 += 1;
        }
        shift.line_nodel(&mut (*buf).b_visual.vi_start.lnum);
        shift.line_nodel(&mut (*buf).b_visual.vi_end.lnum);
        if !qf_mark_adjust(buf, ptr::null_mut(), line1, line2, amount, amount_after) {
            (*buf).b_has_qf_entry &= !BUF_HAS_QF_ENTRY;
        }
        let mut found_one: bool = false;
        let mut tab: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tab.is_null() {
            let mut win: *mut win_T = if tab == curtab.get() {
                firstwin.get()
            } else {
                (*tab).tp_firstwin
            };
            while !win.is_null() {
                found_one = found_one as c_int
                    | qf_mark_adjust(buf, win, line1, line2, amount, amount_after) as c_int
                    != 0;
                win = (*win).w_next;
            }
            tab = (*tab).tp_next as *mut tabpage_T;
        }
        if !found_one {
            (*buf).b_has_qf_entry &= !BUF_HAS_LL_ENTRY;
        }
    }
    if op as c_uint != kExtmarkNOOP as c_int as c_uint {
        extmark_adjust(buf, line1, line2, amount, amount_after, op);
    }
    if (*curwin.get()).w_buffer == buf {
        shift.line(&mut (*curwin.get()).w_pcmark.lnum);
        shift.line(&mut (*curwin.get()).w_prev_pcmark.lnum);
        if (*saved_cursor.ptr()).lnum != 0 {
            shift.line_nodel(&mut (*saved_cursor.ptr()).lnum);
        }
    }
    let mut tab_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tab_0.is_null() {
        let mut win_0: *mut win_T = if tab_0 == curtab.get() {
            firstwin.get()
        } else {
            (*tab_0).tp_firstwin
        };
        while !win_0.is_null() {
            if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as c_int == 0 {
                let mut i_2: c_int = 0;
                while i_2 < (*win_0).w_jumplistlen {
                    if (*win_0).w_jumplist[i_2 as usize].fmark.fnum == fnum {
                        shift.line_nodel(
                            &mut (*(&raw mut (*win_0).w_jumplist as *mut xfmark_T)
                                .offset(i_2 as isize))
                            .fmark
                            .mark
                            .lnum,
                        );
                    }
                    i_2 += 1;
                }
            }
            if (*win_0).w_buffer == buf {
                if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as c_int == 0 {
                    let mut i_3: c_int = 0;
                    while i_3 < (*win_0).w_tagstacklen {
                        if (*win_0).w_tagstack[i_3 as usize].fmark.fnum == fnum {
                            shift.line_nodel(
                                &mut (*(&raw mut (*win_0).w_tagstack as *mut taggy_T)
                                    .offset(i_3 as isize))
                                .fmark
                                .mark
                                .lnum,
                            );
                        }
                        i_3 += 1;
                    }
                }
                if (*win_0).w_old_cursor_lnum != 0 {
                    shift.line_nodel(&mut (*win_0).w_old_cursor_lnum);
                    shift.line_nodel(&mut (*win_0).w_old_visual_lnum);
                }
                if by_api
                    || (if by_term {
                        ((*win_0).w_cursor.lnum < (*buf).b_ml.ml_line_count) as c_int
                    } else {
                        (win_0 != curwin.get()) as c_int
                    }) != 0
                {
                    if (*win_0).w_topline >= line1 && (*win_0).w_topline <= line2 {
                        if amount == MAXLNUM as c_int {
                            if !(by_api && amount_after > line1 - line2 - 1) {
                                (*win_0).w_topline = if line1 - 1 > 1 { line1 - 1 } else { 1 };
                            }
                        } else if (*win_0).w_topline > line1 {
                            (*win_0).w_topline += amount;
                        }
                        (*win_0).w_topfill = 0;
                    } else if amount_after != 0
                        && (*win_0).w_topline
                            > line2 + (if by_api && line2 < line1 { 1 } else { 0 })
                    {
                        (*win_0).w_topline += amount_after;
                        (*win_0).w_topfill = 0;
                    }
                }
                if !by_api
                    && (if by_term {
                        ((*win_0).w_cursor.lnum < (*buf).b_ml.ml_line_count) as c_int
                    } else {
                        (win_0 != curwin.get()) as c_int
                    }) != 0
                {
                    shift.cursor(&mut (*win_0).w_cursor);
                }
                if adjust_folds {
                    foldMarkAdjust(win_0, line1, line2, amount, amount_after);
                }
            }
            win_0 = (*win_0).w_next;
        }
        tab_0 = (*tab_0).tp_next as *mut tabpage_T;
    }
    diff_mark_adjust(buf, line1, line2, amount, amount_after);
    let mut i_4: size_t = 0;
    while i_4 < (*buf).b_wininfo.size {
        let mut wip: *mut WinInfo = *(*buf).b_wininfo.items.add(i_4);
        if !by_term || (*wip).wi_mark.mark.lnum < (*buf).b_ml.ml_line_count {
            shift.cursor(&mut (*wip).wi_mark.mark);
        }
        i_4 = i_4.wrapping_add(1);
    }
}

/// Adjust marks in line "lnum" at column "mincol" and further: add
/// "lnum_amount" to the line number and add "col_amount" to the column
/// position.
/// "spaces_removed" is the number of spaces that were removed, matters when the
/// cursor is inside them.
pub unsafe fn mark_col_adjust(
    mut lnum: linenr_T,
    mut mincol: colnr_T,
    mut lnum_amount: linenr_T,
    mut col_amount: colnr_T,
    mut spaces_removed: c_int,
) {
    let mut fnum: c_int = (*curbuf.get()).handle as c_int;
    // Upstream asserts this once per adjusted mark; `col_amount` does not
    // change, and the upper half is vacuous for a `colnr_T`. What it really
    // guards is `-col_amount` below.
    debug_assert!(col_amount > colnr_T::MIN, "col_amount > INT_MIN");
    let shift = ColShift {
        lnum,
        mincol,
        lnum_amount,
        col_amount,
        spaces_removed,
    };
    if col_amount == 0 && lnum_amount == 0
        || (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as c_int != 0
    {
        return;
    }
    let mut i: c_int = 0;
    while i < NMARKS {
        shift.col(
            &mut (*(&raw mut (*curbuf.get()).b_namedm as *mut fmark_T).offset(i as isize)).mark,
        );
        if (*namedfm.ptr())[i as usize].fmark.fnum == fnum {
            shift.col(
                &mut (*(namedfm.ptr() as *mut xfmark_T).offset(i as isize))
                    .fmark
                    .mark,
            );
        }
        i += 1;
    }
    let mut i_0: c_int = NMARKS;
    while i_0 < NGLOBALMARKS {
        if (*namedfm.ptr())[i_0 as usize].fmark.fnum == fnum {
            shift.col(
                &mut (*(namedfm.ptr() as *mut xfmark_T).offset(i_0 as isize))
                    .fmark
                    .mark,
            );
        }
        i_0 += 1;
    }
    shift.col(&mut (*curbuf.get()).b_last_insert.mark);
    shift.col(&mut (*curbuf.get()).b_last_change.mark);
    if bt_prompt(curbuf.get()) {
        shift.col(&mut (*curbuf.get()).b_prompt_start.mark);
    }
    let mut i_1: c_int = 0;
    while i_1 < (*curbuf.get()).b_changelistlen {
        shift.col(
            &mut (*(&raw mut (*curbuf.get()).b_changelist as *mut fmark_T).offset(i_1 as isize))
                .mark,
        );
        i_1 += 1;
    }
    shift.col(&mut (*curbuf.get()).b_visual.vi_start);
    shift.col(&mut (*curbuf.get()).b_visual.vi_end);
    shift.col(&mut (*curwin.get()).w_pcmark);
    shift.col(&mut (*curwin.get()).w_prev_pcmark);
    shift.col(&mut *(saved_cursor.ptr()));
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        let mut i_2: c_int = 0;
        while i_2 < (*win).w_jumplistlen {
            if (*win).w_jumplist[i_2 as usize].fmark.fnum == fnum {
                shift.col(
                    &mut (*(&raw mut (*win).w_jumplist as *mut xfmark_T).offset(i_2 as isize))
                        .fmark
                        .mark,
                );
            }
            i_2 += 1;
        }
        if (*win).w_buffer == curbuf.get() {
            let mut i_3: c_int = 0;
            while i_3 < (*win).w_tagstacklen {
                if (*win).w_tagstack[i_3 as usize].fmark.fnum == fnum {
                    shift.col(
                        &mut (*(&raw mut (*win).w_tagstack as *mut taggy_T).offset(i_3 as isize))
                            .fmark
                            .mark,
                    );
                }
                i_3 += 1;
            }
            if win != curwin.get() {
                shift.col(&mut (*win).w_cursor);
            }
        }
        win = (*win).w_next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `:56,57d` — delete two lines, shifting everything after them up.
    const DELETE_56_57: LineShift = LineShift {
        line1: 56,
        line2: 57,
        amount: MAXLNUM as c_int,
        amount_after: -2,
    };

    /// `:56 insert two lines` — the second form `mark_adjust.c` documents,
    /// where the moved range is empty and `amount_after` does the work.
    const INSERT_TWO_BELOW_55: LineShift = LineShift {
        line1: 56,
        line2: 55,
        amount: MAXLNUM as c_int,
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
            amount: MAXLNUM as c_int,
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
            line2: MAXLNUM as c_int,
            amount: 2,
            amount_after: 0,
        };
        let mut lnum = 60;
        shift.line(&mut lnum);
        assert_eq!(lnum, 62);
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
