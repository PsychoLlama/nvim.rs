use crate::src::nvim::garray::{ga_grow, ga_init};
use crate::src::nvim::main::{State, VIsual, VIsual_active, curwin, p_sel};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::mb_adjust_cursor;
use crate::src::nvim::memline::ml_get_len;
use crate::src::nvim::os::libc::memmove;
use crate::src::nvim::pos::{MAXLNUM, ltoreq};
use core::ffi::{c_int, c_void};
use core::ptr;

use super::open_close::*;
use super::*;
use crate::src::nvim::state::MODE_INSERT;

///
/// If "updown" is false: Move to the start or end of the fold.
/// If "updown" is true: move to fold at the same level.
/// Returns fAIL if not moved.
///
/// `dir` — FORWARD or BACKWARD
pub unsafe extern "C" fn foldMoveTo(updown: bool, dir: c_int, count: c_int) -> c_int {
    let mut retval: c_int = FAIL;
    let mut fp: *mut fold_T = ptr::null_mut();
    checkupdate(curwin.get());
    let mut n: c_int = 0;
    while n < count {
        let mut lnum_off: linenr_T = 0;
        let mut gap: *mut garray_T = &raw mut (*curwin.get()).w_folds;
        if (*gap).ga_len == 0 {
            break;
        }
        let mut use_level: bool = false;
        let mut maybe_small: bool = false;
        let mut lnum_found: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut level: c_int = 0;
        let mut last: bool = false;
        loop {
            if !foldFind(gap, (*curwin.get()).w_cursor.lnum - lnum_off, &raw mut fp) {
                if !updown || (*gap).ga_len == 0 {
                    break;
                }
                if dir == FORWARD as c_int {
                    if fold_index(&*gap, fp) >= (*gap).ga_len {
                        break;
                    }
                    // `fp` may already be the first fold, in which case this
                    // steps one *before* the array. Upstream does the same and
                    // only ever reads `fp[1]` afterwards, so the wrapping form
                    // is what keeps that defined.
                    fp = fp.wrapping_sub(1);
                } else if fp == folds(&*gap) {
                    break;
                }
                last = true;
            }
            if !last {
                if check_closed(
                    curwin.get(),
                    fp,
                    &raw mut use_level,
                    level,
                    &raw mut maybe_small,
                    lnum_off,
                ) {
                    last = true;
                }
                if last && !updown {
                    break;
                }
            }
            if updown {
                if dir == FORWARD as c_int {
                    if fold_index(&*gap, fp.offset(1)) < (*gap).ga_len {
                        let mut lnum: linenr_T = (*fp.offset(1)).fd_top + lnum_off;
                        if lnum > (*curwin.get()).w_cursor.lnum {
                            lnum_found = lnum;
                        }
                    }
                } else if fp > folds(&*gap) {
                    let mut lnum_0: linenr_T =
                        (*fp.offset(-1)).fd_top + lnum_off + (*fp.offset(-1)).fd_len - 1;
                    if lnum_0 < (*curwin.get()).w_cursor.lnum {
                        lnum_found = lnum_0;
                    }
                }
            } else if dir == FORWARD as c_int {
                let mut lnum_1: linenr_T = (*fp).fd_top + lnum_off + (*fp).fd_len - 1;
                if lnum_1 > (*curwin.get()).w_cursor.lnum {
                    lnum_found = lnum_1;
                }
            } else {
                let mut lnum_2: linenr_T = (*fp).fd_top + lnum_off;
                if lnum_2 < (*curwin.get()).w_cursor.lnum {
                    lnum_found = lnum_2;
                }
            }
            if last {
                break;
            }
            gap = &raw mut (*fp).fd_nested;
            lnum_off += (*fp).fd_top;
            level += 1;
        }
        if lnum_found == (*curwin.get()).w_cursor.lnum {
            break;
        }
        if retval == FAIL {
            setpcmark();
        }
        (*curwin.get()).w_cursor.lnum = lnum_found;
        (*curwin.get()).w_cursor.col = 0;
        retval = OK;
        n += 1;
    }
    return retval;
}

/// Adjust the Visual area to include any fold at the start or end completely.
pub unsafe extern "C" fn foldAdjustVisual() {
    if !VIsual_active.get() || hasAnyFolding(curwin.get()) == 0 {
        return;
    }
    let mut start: *mut pos_T = ptr::null_mut();
    let mut end: *mut pos_T = ptr::null_mut();
    if ltoreq(VIsual.get(), (*curwin.get()).w_cursor) {
        start = VIsual.ptr();
        end = &raw mut (*curwin.get()).w_cursor;
    } else {
        start = &raw mut (*curwin.get()).w_cursor;
        end = VIsual.ptr();
    }
    if hasFolding(
        curwin.get(),
        (*start).lnum,
        &raw mut (*start).lnum,
        ptr::null_mut(),
    ) {
        (*start).col = 0;
    }
    if !hasFolding(
        curwin.get(),
        (*end).lnum,
        ptr::null_mut(),
        &raw mut (*end).lnum,
    ) {
        return;
    }
    (*end).col = ml_get_len((*end).lnum);
    if (*end).col > 0 && *p_sel.get() as c_int == 'o' as c_int {
        (*end).col -= 1;
    }
    mb_adjust_cursor();
}

/// Move the cursor to the first line of a closed fold.
pub unsafe extern "C" fn foldAdjustCursor(mut wp: *mut win_T) {
    hasFolding(
        wp,
        (*wp).w_cursor.lnum,
        &raw mut (*wp).w_cursor.lnum,
        ptr::null_mut(),
    );
}

/// Update line numbers of folds for inserted/deleted lines.
///
/// We are adjusting the folds in the range from line1 til line2,
/// make sure that line2 does not get smaller than line1
pub unsafe extern "C" fn foldMarkAdjust(
    mut wp: *mut win_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) {
    if amount == MAXLNUM as c_int as linenr_T && line2 >= line1 && line2 - line1 >= -amount_after {
        line2 = line1 - amount_after - 1;
    }
    if line2 < line1 {
        line2 = line1;
    }
    if State.get() & MODE_INSERT != 0 && amount == 1 && line2 == MAXLNUM as c_int as linenr_T {
        line1 -= 1;
    }
    foldMarkAdjustRecurse(&raw mut (*wp).w_folds, line1, line2, amount, amount_after);
}

pub unsafe extern "C" fn foldMarkAdjustRecurse(
    mut gap: *mut garray_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) {
    if (*gap).ga_len == 0 {
        return;
    }
    let mut top: linenr_T =
        if State.get() & MODE_INSERT != 0 && amount == 1 && line2 == MAXLNUM as c_int as linenr_T {
            line1 + 1
        } else {
            line1
        };
    let mut first: *mut fold_T = ptr::null_mut();
    foldFind(gap, line1, &raw mut first);
    let mut i: c_int = fold_index(&*gap, first);
    while i < (*gap).ga_len {
        // Re-derived rather than stepped, because the delete branch below
        // leaves `i` naming a *different* fold. Upstream walked a pointer and
        // wrote `fp--` after deleting entry zero, which is one before the
        // array — out of bounds even though it never dereferenced it.
        let fp: *mut fold_T = fold_at(&*gap, i);
        let mut last: linenr_T = (*fp).fd_top + (*fp).fd_len - 1;
        if last >= line1 {
            if (*fp).fd_top > line2 {
                if amount_after == 0 {
                    break;
                }
                (*fp).fd_top += amount_after;
            } else if (*fp).fd_top >= top && last <= line2 {
                if amount == MAXLNUM as c_int as linenr_T {
                    deleteFoldEntry(gap, i, true);
                    // The fold that took its place is next; do not advance.
                    continue;
                }
                (*fp).fd_top += amount;
            } else if (*fp).fd_top < top {
                foldMarkAdjustRecurse(
                    &raw mut (*fp).fd_nested,
                    line1 - (*fp).fd_top,
                    line2 - (*fp).fd_top,
                    amount,
                    amount_after,
                );
                if last <= line2 {
                    if amount == MAXLNUM as c_int as linenr_T {
                        (*fp).fd_len = line1 - (*fp).fd_top;
                    } else {
                        (*fp).fd_len += amount;
                    }
                } else {
                    (*fp).fd_len += amount_after;
                }
            } else if amount == MAXLNUM as c_int as linenr_T {
                foldMarkAdjustRecurse(
                    &raw mut (*fp).fd_nested,
                    0,
                    line2 - (*fp).fd_top,
                    amount,
                    amount_after + ((*fp).fd_top - top),
                );
                (*fp).fd_len =
                    ((*fp).fd_len as c_int - (line2 - (*fp).fd_top + 1) as c_int) as linenr_T;
                (*fp).fd_top = line1;
            } else {
                foldMarkAdjustRecurse(
                    &raw mut (*fp).fd_nested,
                    0,
                    line2 - (*fp).fd_top,
                    amount,
                    amount_after - amount,
                );
                (*fp).fd_len += amount_after - amount;
                (*fp).fd_top += amount;
            }
        }
        i += 1;
    }
}

/// Insert a new fold in "gap" at position "i".
pub(super) unsafe extern "C" fn foldInsert(mut gap: *mut garray_T, mut i: c_int) {
    ga_grow(gap, 1);
    let mut fp: *mut fold_T = fold_at(&*gap, i);
    if (*gap).ga_len > 0 && i < (*gap).ga_len {
        memmove(
            fp.offset(1) as *mut c_void,
            fp as *const c_void,
            size_of::<fold_T>().wrapping_mul(((*gap).ga_len - i) as size_t),
        );
    }
    (*gap).ga_len += 1;
    ga_init(&raw mut (*fp).fd_nested, size_of::<fold_T>() as c_int, 10);
}

/// Split the "i"th fold in "gap", which starts before "top" and ends below
/// "bot" in two pieces, one ending above "top" and the other starting below
/// "bot".
/// The caller must first have taken care of any nested folds from "top" to
/// "bot"!
pub(super) unsafe extern "C" fn foldSplit(
    mut _buf: *mut buf_T,
    gap: *mut garray_T,
    i: c_int,
    top: linenr_T,
    bot: linenr_T,
) {
    let mut fp2: *mut fold_T = ptr::null_mut();
    foldInsert(gap, i + 1);
    let fp: *mut fold_T = fold_at(&*gap, i);
    (*fp.offset(1)).fd_top = bot + 1;
    assert!((*fp.offset(1)).fd_top > bot, "fp[1].fd_top > bot");
    (*fp.offset(1)).fd_len = (*fp).fd_len - ((*fp.offset(1)).fd_top - (*fp).fd_top);
    (*fp.offset(1)).fd_flags = (*fp).fd_flags;
    (*fp.offset(1)).fd_small = kNone;
    (*fp).fd_small = kNone;
    let gap1: *mut garray_T = &raw mut (*fp).fd_nested;
    let gap2: *mut garray_T = &raw mut (*fp.offset(1)).fd_nested;
    foldFind(gap1, bot + 1 - (*fp).fd_top, &raw mut fp2);
    if !fp2.is_null() {
        let len: c_int = folds_end(&*gap1).offset_from(fp2) as c_int;
        if len > 0 {
            ga_grow(gap2, len);
            let mut idx: c_int = 0;
            while idx < len {
                *fold_at(&*gap2, idx) = *fp2.offset(idx as isize);
                (*fold_at(&*gap2, idx)).fd_top -= (*fp.offset(1)).fd_top - (*fp).fd_top;
                idx += 1;
            }
            (*gap2).ga_len = len;
            (*gap1).ga_len -= len;
        }
    }
    (*fp).fd_len = top - (*fp).fd_top;
    fold_changed.set(true);
}

/// Remove folds within the range "top" to and including "bot".
/// Check for these situations:
///      1  2  3
///      1  2  3
/// top     2  3  4  5
///     2  3  4  5
/// bot     2  3  4  5
///        3     5  6
///        3     5  6
///
/// 1: not changed
/// 2: truncate to stop above "top"
/// 3: split in two parts, one stops above "top", other starts below "bot".
/// 4: deleted
/// 5: made to start below "bot".
/// 6: not changed
pub(super) unsafe extern "C" fn foldRemove(
    wp: *mut win_T,
    mut gap: *mut garray_T,
    mut top: linenr_T,
    mut bot: linenr_T,
) {
    if bot < top {
        return;
    }
    let mut fp: *mut fold_T = ptr::null_mut();
    // Not immutable: foldFind/foldRemove shrink *gap behind the raw pointer.
    #[allow(clippy::while_immutable_condition)]
    while (*gap).ga_len > 0 {
        if foldFind(gap, top, &raw mut fp) && (*fp).fd_top < top {
            foldRemove(
                wp,
                &raw mut (*fp).fd_nested,
                top - (*fp).fd_top,
                bot - (*fp).fd_top,
            );
            if (*fp).fd_top + (*fp).fd_len - 1 > bot {
                foldSplit((*wp).w_buffer, gap, fold_index(&*gap, fp), top, bot);
            } else {
                (*fp).fd_len = top - (*fp).fd_top;
            }
            fold_changed.set(true);
        } else {
            if (*gap).ga_data.is_null() || fp >= folds_end(&*gap) || (*fp).fd_top > bot {
                break;
            }
            if (*fp).fd_top < top {
                continue;
            }
            fold_changed.set(true);
            if (*fp).fd_top + (*fp).fd_len - 1 > bot {
                foldMarkAdjustRecurse(
                    &raw mut (*fp).fd_nested,
                    0,
                    bot - (*fp).fd_top,
                    MAXLNUM as c_int as linenr_T,
                    (*fp).fd_top - bot - 1,
                );
                (*fp).fd_len =
                    ((*fp).fd_len as c_int - (bot - (*fp).fd_top + 1) as c_int) as linenr_T;
                (*fp).fd_top = bot + 1;
                break;
            } else {
                deleteFoldEntry(gap, fold_index(&*gap, fp), true);
            }
        }
    }
}

pub(super) unsafe extern "C" fn foldReverseOrder(
    mut gap: *mut garray_T,
    start_arg: linenr_T,
    end_arg: linenr_T,
) {
    let mut start: linenr_T = start_arg;
    let mut end: linenr_T = end_arg;
    while start < end {
        let mut left: *mut fold_T = fold_at(&*gap, start);
        let mut right: *mut fold_T = fold_at(&*gap, end);
        ptr::swap(left, right);
        start += 1;
        end -= 1;
    }
}

/// Move folds within the inclusive range "line1" to "line2" to after "dest"
/// require "line1" <= "line2" <= "dest"
///
/// There are the following situations for the first fold at or below line1 - 1.
///       1  2  3  4
///       1  2  3  4
/// line1    2  3  4
///          2  3  4  5  6  7
/// line2       3  4  5  6  7
///             3  4     6  7  8  9
/// dest           4        7  8  9
///                4        7  8    10
///                4        7  8    10
///
/// In the following descriptions, "moved" means moving in the buffer, *and* in
/// the fold array.
/// Meanwhile, "shifted" just means moving in the buffer.
/// 1. not changed
/// 2. truncated above line1
/// 3. length reduced by  line2 - line1, folds starting between the end of 3 and
///    dest are truncated and shifted up
/// 4. internal folds moved (from [line1, line2] to dest)
/// 5. moved to dest.
/// 6. truncated below line2 and moved.
/// 7. length reduced by line2 - dest, folds starting between line2 and dest are
///    removed, top is moved down by move_len.
/// 8. truncated below dest and shifted up.
/// 9. shifted up
/// 10. not changed
pub(super) unsafe extern "C" fn truncate_fold(
    wp: *mut win_T,
    mut fp: *mut fold_T,
    mut end: linenr_T,
) {
    end = (end as c_int + 1) as linenr_T;
    foldRemove(
        wp,
        &raw mut (*fp).fd_nested,
        end - (*fp).fd_top,
        MAXLNUM as c_int as linenr_T,
    );
    (*fp).fd_len = end - (*fp).fd_top;
}

pub unsafe extern "C" fn foldMoveRange(
    wp: *mut win_T,
    mut gap: *mut garray_T,
    line1: linenr_T,
    line2: linenr_T,
    dest: linenr_T,
) {
    let mut fp: *mut fold_T = ptr::null_mut();
    let range_len: linenr_T = line2 - line1 + 1;
    let move_len: linenr_T = dest - line2;
    let at_start: bool = foldFind(gap, line1 - 1, &raw mut fp);
    if at_start {
        if (*fp).fd_top + (*fp).fd_len - 1 > dest {
            foldMoveRange(
                wp,
                &raw mut (*fp).fd_nested,
                line1 - (*fp).fd_top,
                line2 - (*fp).fd_top,
                dest - (*fp).fd_top,
            );
            return;
        } else if (*fp).fd_top + (*fp).fd_len - 1 > line2 {
            foldMarkAdjustRecurse(
                &raw mut (*fp).fd_nested,
                line1 - (*fp).fd_top,
                line2 - (*fp).fd_top,
                MAXLNUM as c_int as linenr_T,
                -range_len,
            );
            (*fp).fd_len -= range_len;
        } else {
            truncate_fold(wp, fp, line1 - 1);
        }
        fp = fp.offset(1);
    }
    if !((*gap).ga_len > 0 && fp < folds_end(&*gap)) || (*fp).fd_top > dest {
        return;
    } else if (*fp).fd_top > line2 {
        while (*gap).ga_len > 0 && fp < folds_end(&*gap) && (*fp).fd_top + (*fp).fd_len - 1 <= dest
        {
            (*fp).fd_top -= range_len;
            fp = fp.offset(1);
        }
        if (*gap).ga_len > 0 && fp < folds_end(&*gap) && (*fp).fd_top <= dest {
            truncate_fold(wp, fp, dest);
            (*fp).fd_top -= range_len;
        }
        return;
    } else if (*fp).fd_top + (*fp).fd_len - 1 > dest {
        foldMarkAdjustRecurse(
            &raw mut (*fp).fd_nested,
            line2 + 1 - (*fp).fd_top,
            dest - (*fp).fd_top,
            MAXLNUM as c_int as linenr_T,
            -move_len,
        );
        (*fp).fd_len -= move_len;
        (*fp).fd_top += move_len;
        return;
    }
    let mut move_start: size_t = fold_index(&*gap, fp) as size_t;
    let mut move_end: size_t = 0;
    let mut dest_index: size_t = 0;
    while (*gap).ga_len > 0 && fp < folds_end(&*gap) && (*fp).fd_top <= dest {
        if (*fp).fd_top <= line2 {
            if (*fp).fd_top + (*fp).fd_len - 1 > line2 {
                truncate_fold(wp, fp, line2);
            }
            (*fp).fd_top += move_len;
        } else {
            if move_end == 0 {
                move_end = fold_index(&*gap, fp) as size_t;
            }
            if (*fp).fd_top + (*fp).fd_len - 1 > dest {
                truncate_fold(wp, fp, dest);
            }
            (*fp).fd_top -= range_len;
        }
        fp = fp.offset(1);
    }
    dest_index = fold_index(&*gap, fp) as size_t;
    if move_end == 0 {
        return;
    }
    foldReverseOrder(
        gap,
        move_start as linenr_T,
        dest_index.wrapping_sub(1) as linenr_T,
    );
    foldReverseOrder(
        gap,
        move_start as linenr_T,
        move_start
            .wrapping_add(dest_index)
            .wrapping_sub(move_end)
            .wrapping_sub(1) as linenr_T,
    );
    foldReverseOrder(
        gap,
        move_start.wrapping_add(dest_index).wrapping_sub(move_end) as linenr_T,
        dest_index.wrapping_sub(1) as linenr_T,
    );
}

/// Merge two adjacent folds (and the nested ones in them).
/// This only works correctly when the folds are really adjacent!  Thus "fp1"
/// must end just above "fp2".
/// The resulting fold is "fp1", nested folds are moved from "fp2" to "fp1".
/// Fold entry "fp2" in "gap" is deleted.
pub(super) unsafe extern "C" fn foldMerge(
    mut fp1: *mut fold_T,
    mut gap: *mut garray_T,
    mut fp2: *mut fold_T,
) {
    let mut fp3: *mut fold_T = ptr::null_mut();
    let mut fp4: *mut fold_T = ptr::null_mut();
    let mut gap1: *mut garray_T = &raw mut (*fp1).fd_nested;
    let mut gap2: *mut garray_T = &raw mut (*fp2).fd_nested;
    if foldFind(gap1, (*fp1).fd_len - 1, &raw mut fp3) && foldFind(gap2, 0, &raw mut fp4) {
        foldMerge(fp3, gap2, fp4);
    }
    if !((*gap2).ga_len <= 0) {
        ga_grow(gap1, (*gap2).ga_len);
        let mut idx: c_int = 0;
        while idx < (*gap2).ga_len {
            *folds_end(&*gap1) = *fold_at(&*gap2, idx);
            (*folds_end(&*gap1)).fd_top += (*fp1).fd_len;
            (*gap1).ga_len += 1;
            idx += 1;
        }
        (*gap2).ga_len = 0;
    }
    (*fp1).fd_len += (*fp2).fd_len;
    deleteFoldEntry(gap, fold_index(&*gap, fp2), true);
    fold_changed.set(true);
}
