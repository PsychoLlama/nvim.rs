//! Keeping the fold tree in step with the buffer: the motions that jump
//! between folds, and the arithmetic that repairs `fd_top`/`fd_len` after
//! lines are inserted, deleted or moved.
//!
//! Every entry point here walks the tree through [`FoldList`] and [`Fold`],
//! so the pointer arithmetic that used to be spelled out at each site now
//! happens once, in [`super::list`].

#![deny(unsafe_op_in_unsafe_fn)]

use crate::garray::{ga_grow, ga_init};
use crate::main::{State, curwin, p_sel};
use crate::mark::setpcmark;
use crate::mbyte::mb_adjust_cursor;
use crate::memline::ml_get_len;
use crate::pos::{MAXLNUM, ltoreq};
use core::ffi::c_int;
use core::ptr;

use super::open_close::*;
use super::*;
use crate::normal::{visual_active, with_visual_anchor};
use crate::search::FORWARD;
use crate::state::MODE_INSERT;

/// If "updown" is false: Move to the start or end of the fold.
/// If "updown" is true: move to fold at the same level.
/// Returns FAIL if not moved.
///
/// `dir` — FORWARD or BACKWARD
///
/// # Safety
/// The current window must be live.
pub unsafe fn fold_move_to(updown: bool, dir: c_int, count: c_int) -> c_int {
    let mut retval: c_int = FAIL;
    // SAFETY: the caller's promise.
    unsafe { checkupdate(curwin.get()) };
    for _ in 0..count {
        // SAFETY: the caller's promise.
        let mut folds = unsafe { window_folds(curwin.get()) };
        if folds.is_empty() {
            break;
        }
        // SAFETY: the caller's promise.
        let cursor = unsafe { (*curwin.get()).w_cursor.lnum };
        let mut lnum_off: linenr_T = 0;
        let mut use_level = false;
        let mut maybe_small = false;
        let mut lnum_found = cursor;
        let mut level = 0;
        let mut last = false;
        loop {
            let idx = match folds.find(cursor - lnum_off) {
                Ok(i) => i,
                Err(i) => {
                    if !updown || folds.is_empty() {
                        break;
                    }
                    last = true;
                    if dir == FORWARD as c_int {
                        if i >= folds.len() {
                            break;
                        }
                        // `i` may be zero, which names the entry one *before*
                        // the array. Upstream steps the pointer the same way
                        // and only ever reads its `[1]` afterwards, which is
                        // the fold the cursor sits above.
                        i - 1
                    } else {
                        if i == 0 {
                            break;
                        }
                        i
                    }
                }
            };
            let fold = folds.at(idx);
            if !last {
                // SAFETY: the current window is live, and `fold` is one of
                // its own folds, `lnum_off` lines down the tree.
                if unsafe {
                    check_closed(
                        curwin.get(),
                        fold,
                        &mut use_level,
                        level,
                        &mut maybe_small,
                        lnum_off,
                    )
                } {
                    last = true;
                }
                if last && !updown {
                    break;
                }
            }
            if updown {
                if dir == FORWARD as c_int {
                    if idx + 1 < folds.len() {
                        let lnum = folds.at(idx + 1).top() + lnum_off;
                        if lnum > cursor {
                            lnum_found = lnum;
                        }
                    }
                } else if idx > 0 {
                    let lnum = folds.at(idx - 1).last() + lnum_off;
                    if lnum < cursor {
                        lnum_found = lnum;
                    }
                }
            } else if dir == FORWARD as c_int {
                let lnum = fold.last() + lnum_off;
                if lnum > cursor {
                    lnum_found = lnum;
                }
            } else {
                let lnum = fold.top() + lnum_off;
                if lnum < cursor {
                    lnum_found = lnum;
                }
            }
            if last {
                break;
            }
            folds = fold.nested();
            lnum_off += fold.top();
            level += 1;
        }
        if lnum_found == cursor {
            break;
        }
        if retval == FAIL {
            // SAFETY: the caller's promise.
            unsafe { setpcmark() };
        }
        // SAFETY: the caller's promise.
        unsafe {
            (*curwin.get()).w_cursor.lnum = lnum_found;
            (*curwin.get()).w_cursor.col = 0;
        }
        retval = OK;
    }
    retval
}

/// Adjust the Visual area to include any fold at the start or end completely.
///
/// # Safety
/// The current window must be live.
pub unsafe fn fold_adjust_visual() {
    // SAFETY: the caller's promise.
    if !visual_active() || unsafe { has_any_folding(curwin.get()) } == 0 {
        return;
    }
    // The anchor is adjusted as a *copy* and put back: `has_folding` may
    // evaluate 'foldexpr', which is user code that reads the same global, so
    // it must not be held open across the call.
    let stretched = with_visual_anchor(|anchor| {
        let visual = &raw mut *anchor;
        // SAFETY: the caller's promise; both ends name live `pos_T`s.
        unsafe {
            let cursor = &raw mut (*curwin.get()).w_cursor;
            let (start, end) = if ltoreq(*visual, *cursor) {
                (visual, cursor)
            } else {
                (cursor, visual)
            };
            if has_folding(
                curwin.get(),
                (*start).lnum,
                &raw mut (*start).lnum,
                ptr::null_mut(),
            ) {
                (*start).col = 0;
            }
            if !has_folding(
                curwin.get(),
                (*end).lnum,
                ptr::null_mut(),
                &raw mut (*end).lnum,
            ) {
                return false;
            }
            (*end).col = ml_get_len((*end).lnum);
            if (*end).col > 0 && *p_sel.get() as c_int == 'o' as c_int {
                (*end).col -= 1;
            }
            true
        }
    });
    if stretched {
        // SAFETY: the caller's promise.
        unsafe { mb_adjust_cursor() };
    }
}

/// Move the cursor to the first line of a closed fold.
///
/// # Safety
/// `wp` must be a live window.
pub unsafe fn fold_adjust_cursor(wp: *mut win_T) {
    // SAFETY: the caller's promise.
    unsafe {
        has_folding(
            wp,
            (*wp).w_cursor.lnum,
            &raw mut (*wp).w_cursor.lnum,
            ptr::null_mut(),
        )
    };
}

/// Update line numbers of folds for inserted/deleted lines.
///
/// We are adjusting the folds in the range from line1 til line2,
/// make sure that line2 does not get smaller than line1
///
/// # Safety
/// `wp` must be a live window.
pub unsafe fn fold_mark_adjust(
    wp: *mut win_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
) {
    if amount == LINES_DELETED && line2 >= line1 && line2 - line1 >= -amount_after {
        line2 = line1 - amount_after - 1;
    }
    if line2 < line1 {
        line2 = line1;
    }
    if State.get() & MODE_INSERT != 0 && amount == 1 && line2 == MAXLNUM as linenr_T {
        line1 -= 1;
    }
    // SAFETY: a live window's fold list.
    unsafe { fold_mark_adjust_recurse(&raw mut (*wp).w_folds, line1, line2, amount, amount_after) };
}

/// Shift and truncate the folds of `gap` for a change to lines
/// `line1..=line2`.
///
/// `amount` is how far a fold *inside* the range moves, or [`LINES_DELETED`]
/// when the range is going away; `amount_after` is how far everything below
/// the range moves. Every fold falls into one of six situations, marked in
/// the body.
///
/// # Safety
/// `gap` must be a live fold list.
pub unsafe fn fold_mark_adjust_recurse(
    gap: *mut garray_T,
    line1: linenr_T,
    line2: linenr_T,
    amount: linenr_T,
    amount_after: linenr_T,
) {
    // SAFETY: the caller's promise.
    let folds = unsafe { FoldList::new(gap) };
    if folds.is_empty() {
        return;
    }
    // In Insert mode a fold that starts exactly where the line is being
    // inserted keeps its top, so the new line lands above it.
    let top = if State.get() & MODE_INSERT != 0 && amount == 1 && line2 == MAXLNUM as linenr_T {
        line1 + 1
    } else {
        line1
    };
    let mut i = match folds.find(line1) {
        Ok(i) | Err(i) => i,
    };
    while i < folds.len() {
        // Re-derived rather than stepped, because the delete branch below
        // leaves `i` naming a *different* fold. Upstream walked a pointer and
        // wrote `fp--` after deleting entry zero, which is one before the
        // array — out of bounds even though it never dereferenced it.
        let fold = folds.at(i);
        let last = fold.last();
        if last >= line1 {
            if fold.top() > line2 {
                // 1: entirely below the change, so only its start moves.
                if amount_after == 0 {
                    break;
                }
                fold.set_top(fold.top() + amount_after);
            } else if fold.top() >= top && last <= line2 {
                // 2: entirely inside the change.
                if amount == LINES_DELETED {
                    // SAFETY: `i` names an entry of `folds`.
                    unsafe { delete_fold_entry(folds, i, true) };
                    // The fold that took its place is next; do not advance.
                    // `folds.len()` shrank, so the walk still terminates.
                    continue;
                }
                fold.set_top(fold.top() + amount);
            } else if fold.top() < top {
                // 3: starts above the change and reaches into it.
                // SAFETY: a live fold's nested list is a live fold list.
                unsafe {
                    fold_mark_adjust_recurse(
                        fold.nested().gap(),
                        line1 - fold.top(),
                        line2 - fold.top(),
                        amount,
                        amount_after,
                    )
                };
                if last <= line2 {
                    if amount == LINES_DELETED {
                        fold.set_len(line1 - fold.top());
                    } else {
                        fold.set_len(fold.len() + amount);
                    }
                } else {
                    fold.set_len(fold.len() + amount_after);
                }
            } else if amount == LINES_DELETED {
                // 4: starts inside the change and ends below it, and the
                //    lines it loses are gone for good.
                // SAFETY: a live fold's nested list is a live fold list.
                unsafe {
                    fold_mark_adjust_recurse(
                        fold.nested().gap(),
                        0,
                        line2 - fold.top(),
                        amount,
                        amount_after + (fold.top() - top),
                    )
                };
                fold.set_len(fold.len() - (line2 - fold.top() + 1));
                fold.set_top(line1);
            } else {
                // 5: the same, but the lines only moved.
                // SAFETY: a live fold's nested list is a live fold list.
                unsafe {
                    fold_mark_adjust_recurse(
                        fold.nested().gap(),
                        0,
                        line2 - fold.top(),
                        amount,
                        amount_after - amount,
                    )
                };
                fold.set_len(fold.len() + amount_after - amount);
                fold.set_top(fold.top() + amount);
            }
        }
        // 6: entirely above the change — nothing to do.
        i += 1;
    }
}

/// Insert a new fold in `folds` at position `i`.
///
/// # Safety
/// `i` must be in `0..=folds.len()`.
pub(super) unsafe fn fold_insert(folds: FoldList, i: c_int) {
    // SAFETY: a live fold list.
    unsafe { ga_grow(folds.gap(), 1) };
    let fold = folds.at(i);
    if !folds.is_empty() && i < folds.len() {
        // SAFETY: `ga_grow` just made room for one more entry, so the tail
        // has somewhere to slide to.
        unsafe {
            ptr::copy(
                fold.entry(),
                fold.entry().add(1),
                (folds.len() - i) as usize,
            )
        };
    }
    folds.set_len(folds.len() + 1);
    // SAFETY: the entry is the zeroed storage `ga_grow` handed out; this is
    // the call that makes its `fd_nested` a fold list.
    unsafe { ga_init(fold.nested().gap(), size_of::<fold_T>() as c_int, 10) };
}

/// Split the "i"th fold in `folds`, which starts before "top" and ends below
/// "bot" in two pieces, one ending above "top" and the other starting below
/// "bot".
/// The caller must first have taken care of any nested folds from "top" to
/// "bot"!
///
/// # Safety
/// `i` must name an entry of `folds`.
pub(super) unsafe fn fold_split(
    _buf: *mut buf_T,
    folds: FoldList,
    i: c_int,
    top: linenr_T,
    bot: linenr_T,
) {
    // SAFETY: `i + 1` is in `0..=folds.len()`.
    unsafe { fold_insert(folds, i + 1) };
    let fold = folds.at(i);
    let below = folds.at(i + 1);
    below.set_top(bot + 1);
    debug_assert!(below.top() > bot, "below.top() > bot");
    below.set_len(fold.len() - (below.top() - fold.top()));
    below.set_flags(fold.flags());
    below.set_small(None);
    fold.set_small(None);
    // Hand the nested folds that start below "bot" to the new second half.
    let (inner, outer) = (fold.nested(), below.nested());
    let first = match inner.find(bot + 1 - fold.top()) {
        Ok(k) | Err(k) => k,
    };
    let moved = inner.len() - first;
    if moved > 0 {
        // SAFETY: a live fold list.
        unsafe { ga_grow(outer.gap(), moved) };
        for k in 0..moved {
            let dst = outer.at(k);
            dst.write(inner.at(first + k).read());
            dst.set_top(dst.top() - (below.top() - fold.top()));
        }
        outer.set_len(moved);
        inner.set_len(inner.len() - moved);
    }
    fold.set_len(top - fold.top());
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
///
/// # Safety
/// `wp` must be a live window with a live buffer.
pub(super) unsafe fn fold_remove(wp: *mut win_T, folds: FoldList, top: linenr_T, bot: linenr_T) {
    if bot < top {
        return;
    }
    while !folds.is_empty() {
        let found = folds.find(top);
        if let Ok(i) = found
            && folds.at(i).top() < top
        {
            let fold = folds.at(i);
            // SAFETY: a live window, and the fold's own nested list.
            unsafe { fold_remove(wp, fold.nested(), top - fold.top(), bot - fold.top()) };
            if fold.last() > bot {
                // 3: split in two, one stopping above "top" and one starting
                //    below "bot".
                // SAFETY: a live window's buffer, and `i` names an entry.
                unsafe { fold_split((*wp).w_buffer, folds, i, top, bot) };
            } else {
                // 2: truncate to stop above "top".
                fold.set_len(top - fold.top());
            }
            fold_changed.set(true);
            continue;
        }
        let i = match found {
            Ok(i) | Err(i) => i,
        };
        let fold = folds.at(i);
        // 1 and 6: nothing left in the range.
        if !folds.has_data() || i >= folds.len() || fold.top() > bot {
            break;
        }
        if fold.top() < top {
            // Unreachable: the branch above took every fold that starts above
            // "top", and a miss lands on one that starts strictly below it.
            // Kept because upstream has it, and because it is the shape that
            // would spin this loop if the search and the walk disagreed.
            continue;
        }
        fold_changed.set(true);
        if fold.last() > bot {
            // 5: made to start below "bot".
            // SAFETY: a live fold's nested list is a live fold list.
            unsafe {
                fold_mark_adjust_recurse(
                    fold.nested().gap(),
                    0,
                    bot - fold.top(),
                    LINES_DELETED,
                    fold.top() - bot - 1,
                )
            };
            fold.set_len(fold.len() - (bot - fold.top() + 1));
            fold.set_top(bot + 1);
            break;
        }
        // 4: deleted.
        // SAFETY: `i` names an entry of `folds`.
        unsafe { delete_fold_entry(folds, i, true) };
    }
}

/// Reverse the entries `start_arg..=end_arg` of `folds`.
///
/// # Safety
/// Both must name entries of `folds`, unless the range is empty.
pub(super) unsafe fn fold_reverse_order(folds: FoldList, start_arg: c_int, end_arg: c_int) {
    let (mut start, mut end) = (start_arg, end_arg);
    while start < end {
        // SAFETY: both name entries of the list.
        unsafe { ptr::swap(folds.at(start).entry(), folds.at(end).entry()) };
        start += 1;
        end -= 1;
    }
}

/// Drop everything in `fold` below line `end`, nested folds included.
///
/// # Safety
/// `wp` must be a live window, and `fold` one of its folds.
pub(super) unsafe fn truncate_fold(wp: *mut win_T, fold: Fold, end: linenr_T) {
    let end = end + 1;
    // SAFETY: a live window, and the fold's own nested list.
    unsafe { fold_remove(wp, fold.nested(), end - fold.top(), MAXLNUM as linenr_T) };
    fold.set_len(end - fold.top());
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
///
/// # Safety
/// `wp` must be a live window and `gap` a live fold list.
pub unsafe fn fold_move_range(
    wp: *mut win_T,
    gap: *mut garray_T,
    line1: linenr_T,
    line2: linenr_T,
    dest: linenr_T,
) {
    // SAFETY: the caller's promise.
    let folds = unsafe { FoldList::new(gap) };
    let range_len = line2 - line1 + 1;
    let move_len = dest - line2;
    let mut i = match folds.find(line1 - 1) {
        Ok(i) => {
            let fold = folds.at(i);
            if fold.last() > dest {
                // 4: the whole move happens inside this one fold.
                // SAFETY: a live window, and the fold's own nested list.
                unsafe {
                    fold_move_range(
                        wp,
                        fold.nested().gap(),
                        line1 - fold.top(),
                        line2 - fold.top(),
                        dest - fold.top(),
                    )
                };
                return;
            } else if fold.last() > line2 {
                // 3: shortened by the lines that left it.
                // SAFETY: a live fold's nested list is a live fold list.
                unsafe {
                    fold_mark_adjust_recurse(
                        fold.nested().gap(),
                        line1 - fold.top(),
                        line2 - fold.top(),
                        LINES_DELETED,
                        -range_len,
                    )
                };
                fold.set_len(fold.len() - range_len);
            } else {
                // 2: truncated above line1.
                // SAFETY: a live window, and one of its folds.
                unsafe { truncate_fold(wp, fold, line1 - 1) };
            }
            i + 1
        }
        Err(i) => i,
    };
    // 1 and 10: nothing of interest at or below "line1".
    if folds.is_empty() || i >= folds.len() || folds.at(i).top() > dest {
        return;
    }
    if folds.at(i).top() > line2 {
        // 8 and 9: no fold covers the moved lines, so the folds between them
        // and "dest" simply shift up.
        while i < folds.len() && folds.at(i).last() <= dest {
            let fold = folds.at(i);
            fold.set_top(fold.top() - range_len);
            i += 1;
        }
        if i < folds.len() && folds.at(i).top() <= dest {
            let fold = folds.at(i);
            // SAFETY: a live window, and one of its folds.
            unsafe { truncate_fold(wp, fold, dest) };
            fold.set_top(fold.top() - range_len);
        }
        return;
    }
    if folds.at(i).last() > dest {
        // 7: the fold straddles "dest", so it loses the lines that jumped it.
        let fold = folds.at(i);
        // SAFETY: a live fold's nested list is a live fold list.
        unsafe {
            fold_mark_adjust_recurse(
                fold.nested().gap(),
                line2 + 1 - fold.top(),
                dest - fold.top(),
                LINES_DELETED,
                -move_len,
            )
        };
        fold.set_len(fold.len() - move_len);
        fold.set_top(fold.top() + move_len);
        return;
    }
    // 5 and 6: the folds inside the range move down past the ones between
    // them and "dest", which shift up.
    let move_start = i;
    let mut move_end = 0;
    while i < folds.len() && folds.at(i).top() <= dest {
        let fold = folds.at(i);
        if fold.top() <= line2 {
            if fold.last() > line2 {
                // SAFETY: a live window, and one of its folds.
                unsafe { truncate_fold(wp, fold, line2) };
            }
            fold.set_top(fold.top() + move_len);
        } else {
            if move_end == 0 {
                move_end = i;
            }
            if fold.last() > dest {
                // SAFETY: a live window, and one of its folds.
                unsafe { truncate_fold(wp, fold, dest) };
            }
            fold.set_top(fold.top() - range_len);
        }
        i += 1;
    }
    let dest_index = i;
    if move_end == 0 {
        return;
    }
    // Rotate `move_start..dest_index` left by `move_end - move_start`, so the
    // moved block ends up after the folds it jumped: reverse the whole span,
    // then reverse each of the two halves back.
    // SAFETY: all three ranges lie inside `move_start..dest_index`, which the
    // walk above established as entries of `folds`.
    unsafe {
        fold_reverse_order(folds, move_start, dest_index - 1);
        fold_reverse_order(folds, move_start, move_start + dest_index - move_end - 1);
        fold_reverse_order(folds, move_start + dest_index - move_end, dest_index - 1);
    }
}

/// Merge two adjacent folds (and the nested ones in them).
/// This only works correctly when the folds are really adjacent!  Thus `fold1`
/// must end just above `fold2`.
/// The resulting fold is `fold1`, nested folds are moved from `fold2` to
/// `fold1`. Entry `fold2` in `folds` is deleted.
///
/// # Safety
/// Both must be entries of `folds`.
pub(super) unsafe fn fold_merge(fold1: Fold, folds: FoldList, fold2: Fold) {
    let (inner1, inner2) = (fold1.nested(), fold2.nested());
    // If the last fold nested in `fold1` touches the first one nested in
    // `fold2`, those two merge as well.
    if let (Ok(a), Ok(b)) = (inner1.find(fold1.len() - 1), inner2.find(0)) {
        // SAFETY: both name entries of their own lists.
        unsafe { fold_merge(inner1.at(a), inner2, inner2.at(b)) };
    }
    if !inner2.is_empty() {
        // SAFETY: a live fold list.
        unsafe { ga_grow(inner1.gap(), inner2.len()) };
        for k in 0..inner2.len() {
            let dst = inner1.at(inner1.len());
            dst.write(inner2.at(k).read());
            dst.set_top(dst.top() + fold1.len());
            inner1.set_len(inner1.len() + 1);
        }
        inner2.set_len(0);
    }
    fold1.set_len(fold1.len() + fold2.len());
    // SAFETY: `fold2` is an entry of `folds`.
    unsafe { delete_fold_entry(folds, folds.index_of(fold2), true) };
    fold_changed.set(true);
}
