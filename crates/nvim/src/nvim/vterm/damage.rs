//! What the screen tells its host has changed, and how much of it is batched
//! up first.
//!
//! Every rectangle is half-open: `start_row`/`start_col` are inside it,
//! `end_row`/`end_col` are one past. A `start_row` of [`NO_RECT`] is the
//! "nothing pending" sentinel.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use crate::src::nvim::types::{VTermDamageSize, VTermRect};

/// The `start_row` a screen's pending rectangle carries when there is none.
pub const NO_RECT: c_int = -1;

/// How much the screen batches up before telling the host.
pub const VTERM_DAMAGE_CELL: VTermDamageSize = 0;
pub const VTERM_DAMAGE_ROW: VTermDamageSize = 1;
pub const VTERM_DAMAGE_SCREEN: VTermDamageSize = 2;
pub const VTERM_DAMAGE_SCROLL: VTermDamageSize = 3;
pub const VTERM_N_DAMAGES: VTermDamageSize = 4;

/// What a freshly damaged rectangle means for the host.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Damage {
    /// Held back; there is nothing to report yet.
    Pending,
    /// Report this rectangle.
    Emit(VTermRect),
    /// Report this rectangle, but flush what is already pending first.
    FlushFirst(VTermRect),
}

/// Folds newly damaged cells into what the screen is already holding back.
///
/// At cell granularity nothing is held back. At row granularity one row is
/// kept pending and widened while the damage stays on it, so that a line
/// being typed reports once rather than per character. At screen and scroll
/// granularity only the bounding box is kept, until the host asks for it.
pub fn merge_damage(pending: &mut VTermRect, rect: VTermRect, merge: VTermDamageSize) -> Damage {
    match merge {
        VTERM_DAMAGE_CELL => Damage::Emit(rect),
        VTERM_DAMAGE_ROW => {
            if rect.end_row > rect.start_row + 1 {
                Damage::FlushFirst(rect)
            } else if pending.start_row == NO_RECT {
                *pending = rect;
                Damage::Pending
            } else if rect.start_row == pending.start_row {
                pending.start_col = pending.start_col.min(rect.start_col);
                pending.end_col = pending.end_col.max(rect.end_col);
                Damage::Pending
            } else {
                let previous = *pending;
                *pending = rect;
                Damage::Emit(previous)
            }
        }
        VTERM_DAMAGE_SCREEN | VTERM_DAMAGE_SCROLL => {
            if pending.start_row == NO_RECT {
                *pending = rect;
            } else {
                expand(pending, &rect);
            }
            Damage::Pending
        }
        _ => Damage::Pending,
    }
}

/// Whether two rectangles cover exactly the same area.
pub fn equal(a: &VTermRect, b: &VTermRect) -> bool {
    a.start_row == b.start_row
        && a.start_col == b.start_col
        && a.end_row == b.end_row
        && a.end_col == b.end_col
}

/// Whether `small` lies entirely inside `big`.
pub fn contains(big: &VTermRect, small: &VTermRect) -> bool {
    small.start_row >= big.start_row
        && small.start_col >= big.start_col
        && small.end_row <= big.end_row
        && small.end_col <= big.end_col
}

/// Whether the two rectangles overlap at all.
///
/// The comparison is inclusive of the exclusive edges, so rectangles that
/// merely touch count as intersecting. That is upstream's test, and it errs
/// towards flushing damage rather than dropping it.
pub fn intersects(a: &VTermRect, b: &VTermRect) -> bool {
    a.start_row <= b.end_row
        && b.start_row <= a.end_row
        && a.start_col <= b.end_col
        && b.start_col <= a.end_col
}

/// Grows `dst` until it covers `src` as well.
pub fn expand(dst: &mut VTermRect, src: &VTermRect) {
    dst.start_row = dst.start_row.min(src.start_row);
    dst.start_col = dst.start_col.min(src.start_col);
    dst.end_row = dst.end_row.max(src.end_row);
    dst.end_col = dst.end_col.max(src.end_col);
}

/// Trims `dst` to `bounds`, leaving it empty rather than negatively sized if
/// the two do not overlap.
pub fn clip(dst: &mut VTermRect, bounds: &VTermRect) {
    dst.start_row = dst.start_row.max(bounds.start_row);
    dst.start_col = dst.start_col.max(bounds.start_col);
    dst.end_row = dst.end_row.min(bounds.end_row);
    dst.end_col = dst.end_col.min(bounds.end_col);
    dst.end_row = dst.end_row.max(dst.start_row);
    dst.end_col = dst.end_col.max(dst.start_col);
}

/// Slides `rect` by whole rows and columns.
pub fn shift(rect: &mut VTermRect, rows: c_int, cols: c_int) {
    rect.start_row += rows;
    rect.end_row += rows;
    rect.start_col += cols;
    rect.end_col += cols;
}

/// Drags the pending damage along with a scroll of `region`, so that it still
/// names the cells it named before the contents moved.
///
/// Two cases are worth the arithmetic. Damage wholly inside the scrolled
/// region moves with it and is clipped back to the region. Damage that a
/// purely vertical scroll cuts cleanly — the region spans its columns — has
/// its row range dragged and pinned to the region. Anything else is left
/// alone: upstream logs a note and moves on, which over-reports damage
/// rather than losing it.
pub fn follow_scroll(
    damaged: &mut VTermRect,
    region: &VTermRect,
    downward: c_int,
    rightward: c_int,
) {
    if contains(region, damaged) {
        shift(damaged, -downward, -rightward);
        clip(damaged, region);
        return;
    }
    let cuts_cleanly = region.start_col <= damaged.start_col
        && region.end_col >= damaged.end_col
        && rightward == 0;
    if !cuts_cleanly {
        return;
    }
    for edge in [&mut damaged.start_row, &mut damaged.end_row] {
        if *edge >= region.start_row && *edge < region.end_row {
            *edge = (*edge - downward).clamp(region.start_row, region.end_row);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(start_row: c_int, start_col: c_int, end_row: c_int, end_col: c_int) -> VTermRect {
        VTermRect {
            start_row,
            start_col,
            end_row,
            end_col,
        }
    }

    #[test]
    fn containment_is_inclusive_of_the_bounds() {
        let big = rect(0, 0, 10, 10);
        assert!(contains(&big, &big));
        assert!(contains(&big, &rect(1, 1, 9, 9)));
        assert!(!contains(&big, &rect(-1, 0, 10, 10)));
        assert!(!contains(&big, &rect(0, 0, 11, 10)));
    }

    #[test]
    fn touching_rectangles_count_as_intersecting() {
        assert!(intersects(&rect(0, 0, 2, 2), &rect(2, 2, 4, 4)));
        assert!(!intersects(&rect(0, 0, 2, 2), &rect(3, 0, 4, 2)));
        assert!(!intersects(&rect(0, 0, 2, 2), &rect(0, 3, 2, 4)));
    }

    #[test]
    fn expanding_takes_the_union_of_the_extents() {
        let mut dst = rect(2, 2, 4, 4);
        expand(&mut dst, &rect(1, 3, 3, 9));
        assert!(equal(&dst, &rect(1, 2, 4, 9)));
    }

    #[test]
    fn clipping_a_disjoint_rectangle_leaves_it_empty() {
        let mut dst = rect(10, 10, 20, 20);
        clip(&mut dst, &rect(0, 0, 5, 5));
        assert_eq!(dst.start_row, dst.end_row);
        assert_eq!(dst.start_col, dst.end_col);
    }

    #[test]
    fn contained_damage_moves_with_the_scroll_and_is_clipped() {
        let mut damaged = rect(4, 0, 6, 80);
        follow_scroll(&mut damaged, &rect(0, 0, 24, 80), 2, 0);
        assert!(equal(&damaged, &rect(2, 0, 4, 80)));

        // Scrolled past the top of the region: clipped, not negative.
        let mut damaged = rect(0, 0, 2, 80);
        follow_scroll(&mut damaged, &rect(0, 0, 24, 80), 4, 0);
        assert!(equal(&damaged, &rect(0, 0, 0, 80)));
    }

    #[test]
    fn a_clean_vertical_cut_drags_only_the_row_range() {
        // Damage wider than the scrolled region, so it is not contained.
        let mut damaged = rect(4, 0, 6, 80);
        follow_scroll(&mut damaged, &rect(2, 0, 8, 80), 1, 0);
        assert!(equal(&damaged, &rect(3, 0, 5, 80)));
    }

    #[test]
    fn cell_granularity_reports_everything_at_once() {
        let mut pending = rect(NO_RECT, 0, 0, 0);
        let one = rect(1, 1, 2, 2);
        assert_eq!(
            merge_damage(&mut pending, one, VTERM_DAMAGE_CELL),
            Damage::Emit(one)
        );
        assert_eq!(pending.start_row, NO_RECT);
    }

    #[test]
    fn row_granularity_widens_one_row_and_flushes_on_the_next() {
        let mut pending = rect(NO_RECT, 0, 0, 0);
        let first = rect(3, 10, 4, 11);
        assert_eq!(
            merge_damage(&mut pending, first, VTERM_DAMAGE_ROW),
            Damage::Pending
        );
        // Same row: widened, still held back.
        assert_eq!(
            merge_damage(&mut pending, rect(3, 4, 4, 6), VTERM_DAMAGE_ROW),
            Damage::Pending
        );
        assert!(equal(&pending, &rect(3, 4, 4, 11)));
        // A different row: the held row goes out and this one takes its place.
        let next = rect(5, 0, 6, 1);
        assert_eq!(
            merge_damage(&mut pending, next, VTERM_DAMAGE_ROW),
            Damage::Emit(rect(3, 4, 4, 11))
        );
        assert!(equal(&pending, &next));
        // Taller than a row: the caller flushes first, then reports it.
        let tall = rect(0, 0, 24, 80);
        assert_eq!(
            merge_damage(&mut pending, tall, VTERM_DAMAGE_ROW),
            Damage::FlushFirst(tall)
        );
    }

    #[test]
    fn coarse_granularity_only_grows_the_bounding_box() {
        let mut pending = rect(NO_RECT, 0, 0, 0);
        for merge in [VTERM_DAMAGE_SCREEN, VTERM_DAMAGE_SCROLL] {
            pending = rect(NO_RECT, 0, 0, 0);
            assert_eq!(
                merge_damage(&mut pending, rect(2, 2, 3, 3), merge),
                Damage::Pending
            );
            assert_eq!(
                merge_damage(&mut pending, rect(5, 0, 6, 9), merge),
                Damage::Pending
            );
            assert!(equal(&pending, &rect(2, 0, 6, 9)));
        }
    }

    #[test]
    fn damage_outside_a_partial_scroll_is_left_alone() {
        let untouched = rect(4, 0, 6, 80);
        let mut damaged = untouched;
        // Horizontal component, so the cut is not clean.
        follow_scroll(&mut damaged, &rect(0, 10, 24, 20), 1, 1);
        assert!(equal(&damaged, &untouched));
    }
}
