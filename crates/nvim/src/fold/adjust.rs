//! Keeping the fold tree in step with the buffer: the motions that jump
//! between folds, and the arithmetic that repairs `top`/`len` after lines are
//! inserted, deleted or moved.
//!
//! The arithmetic is pure: every function below that takes a `&mut Vec<Fold>`
//! touches that list and its nested ones and nothing else, which is what lets
//! a caller lend it a window's `w_folds` for the length of the call.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::mark::setpcmark;
use crate::mbyte::mb_adjust_cursor;
use crate::option::vars::P_SEL;
use crate::pos::{MAXLNUM, ltoreq};
use crate::state::mode::State;
use crate::winlayer::Win;
use core::ffi::c_int;

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
pub fn fold_move_to(updown: bool, dir: c_int, count: c_int) -> c_int {
    let mut retval: c_int = FAIL;
    let forward = dir == FORWARD as c_int;
    checkupdate(Win::current());
    for _ in 0..count {
        let win = Win::current();
        if win.w_folds.is_empty() {
            break;
        }
        let cursor = win.w_cursor.lnum;
        let mut lnum_off: LineNr = 0;
        let mut use_level = false;
        let mut maybe_small = false;
        let mut lnum_found = cursor;
        let mut level = 0;
        // The walk only descends through folds that contain the cursor, so
        // the cursor line and the depth find its list again after the
        // tree has been let go of.
        let mut depth = 0;
        loop {
            // What this level says: the entry the walk stands on (`None`
            // once there is none to move by), and whether it contains the
            // cursor.
            let step = win.with_folds(|root| {
                let (folds, lnum_rel) = list::list_on_line(root, cursor, depth)?;
                match find(folds, lnum_rel) {
                    Ok(i) => Some((i, true)),
                    Err(i) => {
                        if !updown || folds.is_empty() {
                            return None;
                        }
                        // `i` is the first fold below the cursor. Upstream
                        // steps to the entry before it going forward -- one
                        // before the array when `i` is zero -- and only ever
                        // reads the entry after that, which is `i` again.
                        if forward {
                            (i < folds.len()).then_some((i, false))
                        } else {
                            (i > 0).then_some((i, false))
                        }
                    }
                }
            });
            let Some((i, contains)) = step else { break };
            let mut last = !contains;
            if contains {
                let (ul, ms) = (&mut use_level, &mut maybe_small);
                if closed_at(win, cursor, depth, ul, level, ms, lnum_off) {
                    last = true;
                }
                if last && !updown {
                    break;
                }
            }
            let (found, top) = win.with_folds(|root| {
                let (folds, _) = list::list_on_line(root, cursor, depth).expect("the walk's list");
                let found = if updown && forward {
                    // After a miss `i` is the entry after upstream's `fp`.
                    let next = if contains { i + 1 } else { i };
                    folds
                        .get(next)
                        .map(|fold| fold.top + lnum_off)
                        .filter(|&lnum| lnum > cursor)
                } else if updown {
                    i.checked_sub(1)
                        .map(|prev| folds[prev].last() + lnum_off)
                        .filter(|&lnum| lnum < cursor)
                } else if forward {
                    Some(folds[i].last() + lnum_off).filter(|&lnum| lnum > cursor)
                } else {
                    Some(folds[i].top + lnum_off).filter(|&lnum| lnum < cursor)
                };
                // A miss may stand one past the end; it ends the walk anyway.
                (found, if contains { folds[i].top } else { 0 })
            });
            if let Some(lnum) = found {
                lnum_found = lnum;
            }
            if last {
                break;
            }
            lnum_off += top;
            level += 1;
            depth += 1;
        }
        if lnum_found == cursor {
            break;
        }
        if retval == FAIL {
            setpcmark();
        }
        Win::current().w_cursor.lnum = lnum_found;
        Win::current().w_cursor.col = 0;
        retval = OK;
    }
    retval
}

/// Adjust the Visual area to include any fold at the start or end completely.
pub fn fold_adjust_visual() {
    let win = Win::current();
    if !visual_active() || has_any_folding(win) == 0 {
        return;
    }
    // The anchor is adjusted as a *copy* and put back: `has_folding` may
    // evaluate 'foldexpr', which is user code that reads the same global, so
    // it must not be held open across the call.
    let stretched = with_visual_anchor(|anchor| {
        let mut cursor = win.cursor();
        // Which end of the selection is the earlier one; each end is read and
        // written back where it came from, in the order the C writes them.
        let anchor_first = ltoreq(*anchor, *cursor);

        // The earlier end stretches back to the first line of its fold.
        let mut start = if anchor_first { *anchor } else { *cursor };
        if has_folding(win, start.lnum, Some(&mut start.lnum), None) {
            start.col = 0;
        }
        if anchor_first {
            *anchor = start;
        } else {
            *cursor = start;
        }

        // The later end stretches forward to the last line of its own.
        let mut end = if anchor_first { *cursor } else { *anchor };
        if !has_folding(win, end.lnum, None, Some(&mut end.lnum)) {
            return false;
        }
        end.col = win.buffer().lines().line_len(end.lnum);
        if end.col > 0 && P_SEL.first_byte() == b'o' {
            end.col -= 1;
        }
        if anchor_first {
            *cursor = end;
        } else {
            *anchor = end;
        }
        true
    });
    if stretched {
        mb_adjust_cursor();
    }
}

/// Move the cursor to the first line of a closed fold.
pub fn fold_adjust_cursor(window: Win) {
    let mut cursor = window.cursor();
    if let Some(first) = window.fold_first(cursor.lnum) {
        cursor.lnum = first;
    }
}

/// Update line numbers of folds for inserted/deleted lines.
///
/// We are adjusting the folds in the range from line1 til line2,
/// make sure that line2 does not get smaller than line1
///
pub fn fold_mark_adjust(
    window: Win,
    mut line1: LineNr,
    mut line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
) {
    if amount == LINES_DELETED && line2 >= line1 && line2 - line1 >= -amount_after {
        line2 = line1 - amount_after - 1;
    }
    if line2 < line1 {
        line2 = line1;
    }
    if State.get() & MODE_INSERT != 0 && amount == 1 && line2 == MAXLNUM {
        line1 -= 1;
    }
    let insert_mode = State.get() & MODE_INSERT != 0;
    window.with_folds(|folds| {
        adjust_folds(folds, line1, line2, amount, amount_after, insert_mode);
    });
}

/// Shift and truncate the folds of `folds` for a change to lines
/// `line1..=line2`.
///
/// `amount` is how far a fold *inside* the range moves, or [`LINES_DELETED`]
/// when the range is going away; `amount_after` is how far everything below
/// the range moves. Every fold falls into one of six situations, marked in
/// the body.
///
/// Upstream's `foldMarkAdjustRecurse`, and what the unit lane reaches
/// (`crates/nvim/tests/unit/fold.rs`).
pub fn adjust_fold_list(
    folds: &mut Vec<Fold>,
    line1: LineNr,
    line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
) {
    let insert_mode = State.get() & MODE_INSERT != 0;
    adjust_folds(folds, line1, line2, amount, amount_after, insert_mode);
}

/// [`adjust_fold_list`] with the Insert-mode test made once, by the caller.
fn adjust_folds(
    folds: &mut Vec<Fold>,
    line1: LineNr,
    line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
    insert_mode: bool,
) {
    if folds.is_empty() {
        return;
    }
    // In Insert mode a fold that starts exactly where the line is being
    // inserted keeps its top, so the new line lands above it.
    let top = if insert_mode && amount == 1 && line2 == MAXLNUM {
        line1 + 1
    } else {
        line1
    };
    let mut i = find_at_or_below(folds, line1);
    while i < folds.len() {
        let fold = &mut folds[i];
        let last = fold.last();
        if last >= line1 {
            if fold.top > line2 {
                // 1: entirely below the change, so only its start moves.
                if amount_after == 0 {
                    break;
                }
                fold.top += amount_after;
            } else if fold.top >= top && last <= line2 {
                // 2: entirely inside the change.
                if amount == LINES_DELETED {
                    delete_fold_entry(folds, i, true);
                    // The fold that took its place is next; do not advance.
                    continue;
                }
                fold.top += amount;
            } else if fold.top < top {
                // 3: starts above the change and reaches into it.
                let ft = fold.top;
                adjust_folds(
                    &mut fold.nested,
                    line1 - ft,
                    line2 - ft,
                    amount,
                    amount_after,
                    insert_mode,
                );
                if last <= line2 {
                    if amount == LINES_DELETED {
                        fold.len = line1 - ft;
                    } else {
                        fold.len += amount;
                    }
                } else {
                    fold.len += amount_after;
                }
            } else if amount == LINES_DELETED {
                // 4: starts inside the change and ends below it, and the
                //    lines it loses are gone for good.
                let ft = fold.top;
                adjust_folds(
                    &mut fold.nested,
                    0,
                    line2 - ft,
                    amount,
                    amount_after + (ft - top),
                    insert_mode,
                );
                fold.len -= line2 - ft + 1;
                fold.top = line1;
            } else {
                // 5: the same, but the lines only moved.
                let ft = fold.top;
                adjust_folds(
                    &mut fold.nested,
                    0,
                    line2 - ft,
                    amount,
                    amount_after - amount,
                    insert_mode,
                );
                fold.len += amount_after - amount;
                fold.top += amount;
            }
        }
        // 6: entirely above the change — nothing to do.
        i += 1;
    }
}

/// Split the "i"th fold in `folds`, which starts before "top" and ends below
/// "bot" in two pieces, one ending above "top" and the other starting below
/// "bot".
/// The caller must first have taken care of any nested folds from "top" to
/// "bot"!
pub(super) fn fold_split(folds: &mut Vec<Fold>, i: usize, top: LineNr, bot: LineNr) {
    let fold = &mut folds[i];
    let below_top = bot + 1;
    debug_assert!(below_top > bot, "below.top > bot");
    // Hand the nested folds that start below "bot" to the new second half,
    // relative to it.
    let first = find_at_or_below(&fold.nested, below_top - fold.top);
    let mut moved = fold.nested.split_off(first);
    for child in &mut moved {
        child.top -= below_top - fold.top;
    }
    let below = Fold {
        top: below_top,
        len: fold.len - (below_top - fold.top),
        nested: moved,
        state: fold.state,
        small: None,
    };
    fold.small = None;
    fold.len = top - fold.top;
    folds.insert(i + 1, below);
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
pub(super) fn fold_remove(folds: &mut Vec<Fold>, top: LineNr, bot: LineNr) {
    if bot < top {
        return;
    }
    while !folds.is_empty() {
        let found = find(folds, top);
        if let Ok(i) = found
            && folds[i].top < top
        {
            let fold = &mut folds[i];
            let ft = fold.top;
            fold_remove(&mut fold.nested, top - ft, bot - ft);
            if fold.last() > bot {
                // 3: split in two, one stopping above "top" and one starting
                //    below "bot".
                fold_split(folds, i, top, bot);
            } else {
                // 2: truncate to stop above "top".
                fold.len = top - ft;
            }
            fold_changed.set(true);
            continue;
        }
        let i = match found {
            Ok(i) | Err(i) => i,
        };
        // 1 and 6: nothing left in the range.
        let Some(fold) = folds.get_mut(i).filter(|fold| fold.top <= bot) else {
            break;
        };
        if fold.top < top {
            // Unreachable: the branch above took every fold that starts above
            // "top", and a miss lands on one that starts strictly below it.
            // Kept because upstream has it, and because it is the shape that
            // would spin this loop if the search and the walk disagreed.
            continue;
        }
        fold_changed.set(true);
        if fold.last() > bot {
            // 5: made to start below "bot".
            let ft = fold.top;
            adjust_fold_list(&mut fold.nested, 0, bot - ft, LINES_DELETED, ft - bot - 1);
            fold.len -= bot - ft + 1;
            fold.top = bot + 1;
            break;
        }
        // 4: deleted.
        delete_fold_entry(folds, i, true);
    }
}

/// Drop everything in `fold` below line `end`, nested folds included.
pub(super) fn truncate_fold(fold: &mut Fold, end: LineNr) {
    let end = end + 1;
    fold_remove(&mut fold.nested, end - fold.top, MAXLNUM);
    fold.len = end - fold.top;
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
pub fn fold_move_range(folds: &mut [Fold], line1: LineNr, line2: LineNr, dest: LineNr) {
    let range_len = line2 - line1 + 1;
    let move_len = dest - line2;
    let mut i = match find(folds, line1 - 1) {
        Ok(i) => {
            let fold = &mut folds[i];
            let ft = fold.top;
            if fold.last() > dest {
                // 4: the whole move happens inside this one fold.
                fold_move_range(&mut fold.nested, line1 - ft, line2 - ft, dest - ft);
                return;
            } else if fold.last() > line2 {
                // 3: shortened by the lines that left it.
                adjust_fold_list(
                    &mut fold.nested,
                    line1 - ft,
                    line2 - ft,
                    LINES_DELETED,
                    -range_len,
                );
                fold.len -= range_len;
            } else {
                // 2: truncated above line1.
                truncate_fold(fold, line1 - 1);
            }
            i + 1
        }
        Err(i) => i,
    };
    // 1 and 10: nothing of interest at or below "line1".
    if i >= folds.len() || folds[i].top > dest {
        return;
    }
    if folds[i].top > line2 {
        // 8 and 9: no fold covers the moved lines, so the folds between them
        // and "dest" simply shift up.
        while i < folds.len() && folds[i].last() <= dest {
            folds[i].top -= range_len;
            i += 1;
        }
        if let Some(fold) = folds.get_mut(i)
            && fold.top <= dest
        {
            truncate_fold(fold, dest);
            fold.top -= range_len;
        }
        return;
    }
    if folds[i].last() > dest {
        // 7: the fold straddles "dest", so it loses the lines that jumped it.
        let fold = &mut folds[i];
        let ft = fold.top;
        adjust_fold_list(
            &mut fold.nested,
            line2 + 1 - ft,
            dest - ft,
            LINES_DELETED,
            -move_len,
        );
        fold.len -= move_len;
        fold.top += move_len;
        return;
    }
    // 5 and 6: the folds inside the range move down past the ones between
    // them and "dest", which shift up.
    let move_start = i;
    let mut move_end = 0;
    while i < folds.len() && folds[i].top <= dest {
        let fold = &mut folds[i];
        if fold.top <= line2 {
            if fold.last() > line2 {
                truncate_fold(fold, line2);
            }
            fold.top += move_len;
        } else {
            if move_end == 0 {
                move_end = i;
            }
            if fold.last() > dest {
                truncate_fold(fold, dest);
            }
            fold.top -= range_len;
        }
        i += 1;
    }
    let dest_index = i;
    if move_end == 0 {
        return;
    }
    // The moved block ends up after the folds it jumped. Upstream rotates by
    // reversing the whole span and then each half back; one rotation is the
    // same permutation.
    folds[move_start..dest_index].rotate_left(move_end - move_start);
}

/// Merge `folds[i]` with the fold after it (and the nested ones in them).
/// This only works correctly when the folds are really adjacent!  Thus
/// `folds[i]` must end just above `folds[i + 1]`.
/// The resulting fold is `folds[i]`, nested folds are moved from the second
/// to it, and the second is deleted.
pub(super) fn fold_merge(folds: &mut Vec<Fold>, i: usize) {
    let mut second = folds.remove(i + 1);
    merge_into(&mut folds[i], &mut second);
    fold_changed.set(true);
}

/// The body of [`fold_merge`]: fold `second`, which starts just below `first`,
/// into it. `second` is left empty.
fn merge_into(first: &mut Fold, second: &mut Fold) {
    // If the last fold nested in `first` touches the first one nested in
    // `second`, those two merge as well.
    if let (Ok(a), Ok(b)) = (find(&first.nested, first.len - 1), find(&second.nested, 0)) {
        let mut touching = second.nested.remove(b);
        merge_into(&mut first.nested[a], &mut touching);
        fold_changed.set(true);
    }
    let shift = first.len;
    first
        .nested
        .extend(second.nested.drain(..).map(|mut child| {
            child.top += shift;
            child
        }));
    first.len += second.len;
}
