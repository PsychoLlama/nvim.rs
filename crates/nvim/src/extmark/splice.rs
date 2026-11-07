//! Moving marks when the text moves -- `extmark_splice()`.
//!
//! [`splice_impl`] is the one operation every buffer change goes
//! through: given a start position, the extent of the text removed and the
//! extent of the text inserted, it shifts every mark after the change,
//! adjusts the ones inside it, and hands the same information to the buffer
//! update callbacks.  [`adjust`] is the line-based wrapper the older callers
//! use, [`splice`] the column-aware one, and [`move_region`] the form a
//! `:move` needs, where text is removed from one place and inserted at
//! another.
//!
//! Nothing here is unchecked: the three [`Extent`]s are plain numbers and
//! everything the stage reaches for is behind the parent's wrappers, so the
//! four raw-pointer entry points that wrap a `Buffer *` live in the parent
//! instead and this file forbids `unsafe` outright.
//!
//! Original: `src/nvim/extmark.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::undo::splice_delete;
use super::{
    Buf, Extent, kExtmarkUndo, last_splice, line_offset, send_splice, signcols_count_range,
    splice_pending, tree_move_region, tree_splice, undo_header, with_undo_marks,
};
use crate::decoration::SignCountHalf;
use crate::pos::MAXLNUM;
use crate::types::{
    BCount, ColNr, ExtmarkMove, ExtmarkOp, ExtmarkSplice, ExtmarkUndoObject, LineNr, MTPos,
};

/// [`extmark_adjust`](super::extmark_adjust) for the callers that already
/// hold a [`Buf`].
pub(crate) fn adjust(
    buffer: Buf,
    line1: LineNr,
    line2: LineNr,
    amount: i32,
    amount_after: i32,
    undo: ExtmarkOp,
) {
    if splice_pending() {
        return;
    }
    let start_byte = line_offset(buffer, line1) as BCount;
    let mut old_byte = 0;
    let mut new_byte = 0;
    let old_row;
    let new_row;
    if amount == MAXLNUM.get() {
        old_row = line2 - line1 + 1;
        // TODO(bfredl): ej kasta?
        old_byte = buffer.deleted_bytes2.cast_signed();
        new_row = amount_after + old_row;
    } else {
        // A region is either deleted (amount == MAXLNUM) or added
        // (line2 == MAXLNUM). The only other case is `:move`, which
        // `extmark_move_region` handles.
        debug_assert!(line2 == MAXLNUM, "line2 == MAXLNUM");
        old_row = 0;
        new_row = amount;
    }
    if new_row > 0 {
        new_byte = line_offset(buffer, line1 + new_row) as BCount - start_byte;
    }
    let start = Extent {
        row: (line1 - 1).get(),
        col: ColNr::ZERO,
        byte: start_byte,
    };
    let old = Extent {
        row: old_row,
        col: ColNr::ZERO,
        byte: old_byte,
    };
    let new = Extent {
        row: new_row,
        col: ColNr::ZERO,
        byte: new_byte,
    };
    splice_impl(buffer, start, old, new, undo);
}

/// [`extmark_splice`] for the callers that already hold a [`Buf`].
pub(crate) fn splice(
    buffer: Buf,
    start_row: c_int,
    start_col: ColNr,
    old: Extent,
    new: Extent,
    undo: ExtmarkOp,
) {
    let mut offset = line_offset(buffer, LineNr::new(start_row + 1));

    // On an empty buffer, editing the first line leaves the line buffered and
    // the offset negative. The buffer is not actually empty, but the buffered
    // line has not been flushed (and should not be) yet, so the call is valid
    // -- an edge case.
    //
    // TODO(vigoux): maybe there is a better way of testing that?
    if offset < 0 && !buffer.b_ml.ml_chunks.is_built() {
        offset = 0;
    }
    let start = Extent {
        row: start_row,
        col: start_col,
        byte: (offset + start_col.get()) as BCount,
    };
    splice_impl(buffer, start, old, new, undo);
}

/// The splice itself: tell the update callbacks, save and invalidate the
/// marks the deletion covers, move the rest, and record an undo entry.
pub(crate) fn splice_impl(
    mut buffer: Buf,
    start: Extent,
    old: Extent,
    new: Extent,
    undo: ExtmarkOp,
) {
    buffer.deleted_bytes2 = 0;
    send_splice(buffer, start, old, new);

    if old.row > 0 || old.col > ColNr::ZERO {
        // Copy and invalidate the marks a delete would affect.
        // TODO(bfredl): be smart about marks already saved (important for
        // the merge below).
        let end_row = start.row + old.row;
        let end_col = old.col + if old.row != 0 { 0 } else { start.col.get() };
        let header = undo_header(buffer);
        let lo = MTPos {
            row: start.row,
            col: start.col.get(),
        };
        let hi = MTPos {
            row: end_row,
            col: end_col.get(),
        };
        // The marks it saves are gathered here and appended after: the walk
        // runs with no undo header borrowed.
        let mut saved = Vec::new();
        splice_delete(buffer, lo, hi, header.map(|_| &mut saved), false, undo);
        if let Some(link) = header
            && !saved.is_empty()
        {
            with_undo_marks(buffer, link, |list| list.append(&mut saved));
        }
    }

    // Take the signs inside the edited region out of `b_signcols.count`; they
    // go back after the splice.
    if old.row > 0 || new.row > 0 {
        let count = if buffer.b_prev_line_count > 0 {
            buffer.b_prev_line_count
        } else {
            buffer.b_ml.ml_line_count.get()
        };
        signcols_count_range(
            buffer,
            start.row,
            (count - 1).min(start.row + old.row),
            0,
            SignCountHalf::Subtract,
        );
        buffer.b_prev_line_count = 0;
    }

    let at = MTPos {
        row: start.row,
        col: start.col.get(),
    };
    tree_splice(
        buffer.marktree(),
        at,
        old.row,
        old.col.get(),
        new.row,
        new.col.get(),
    );

    if old.row > 0 || new.row > 0 {
        let row2 = (buffer.b_ml.ml_line_count - 1).min(LineNr::new(start.row + new.row));
        signcols_count_range(buffer, start.row, row2.get(), 0, SignCountHalf::Add);
    }

    if undo == kExtmarkUndo {
        let Some(link) = undo_header(buffer) else {
            return;
        };
        with_undo_marks(buffer, link, |list| {
            // TODO(bfredl): this is quite rudimentary. Small (within line)
            // inserts merge with each other and small deletes with each
            // other; add a full merge algorithm later.
            let merged = old.row == 0 && new.row == 0 && merge_into_last(list, start, old, new);
            if merged {
                return;
            }
            let splice = ExtmarkSplice {
                start_row: start.row,
                start_col: start.col,
                old_row: old.row,
                old_col: old.col,
                new_row: new.row,
                new_col: new.col,
                start_byte: start.byte,
                old_byte: old.byte,
                new_byte: new.byte,
            };
            list.push(ExtmarkUndoObject::Splice(splice));
        });
    }
}

/// Fold this splice into the undo header's last entry, when both are
/// single-line edits that abut. Answers whether it merged.
fn merge_into_last(
    list: &mut [ExtmarkUndoObject],
    start: Extent,
    old: Extent,
    new: Extent,
) -> bool {
    let Some(splice) = last_splice(list) else {
        return false;
    };
    if splice.start_row != start.row || splice.old_row != 0 || splice.new_row != 0 {
        return false;
    }
    if old.col == ColNr::ZERO
        && start.col >= splice.start_col
        && start.col <= splice.start_col + splice.new_col.get()
    {
        splice.new_col += new.col.get();
        splice.new_byte += new.byte;
        true
    } else if new.col == ColNr::ZERO && start.col == splice.start_col + splice.new_col.get() {
        splice.old_col += old.col.get();
        splice.old_byte += old.byte;
        true
    } else if new.col == ColNr::ZERO && start.col + old.col.get() == splice.start_col {
        splice.start_col = start.col;
        splice.start_byte = start.byte;
        splice.old_col += old.col.get();
        splice.old_byte += old.byte;
        true
    } else {
        false
    }
}

/// [`extmark_move_region`] for the callers that already hold a [`Buf`].
pub(crate) fn move_region(
    mut buffer: Buf,
    start: Extent,
    extent: Extent,
    new: Extent,
    undo: ExtmarkOp,
) {
    buffer.deleted_bytes2 = 0;
    // TODO(bfredl): this is not synced to the buffer state inside the
    // callback. But unless the undo implementation gets smarter, it is not
    // ensured anyway.
    send_splice(buffer, start, extent, Extent::default());

    let row1 = start.row.min(new.row);
    let row2 = start.row.max(new.row) + extent.row;
    signcols_count_range(buffer, row1, row2, 0, SignCountHalf::Subtract);

    let at = MTPos {
        row: start.row,
        col: start.col.get(),
    };
    tree_move_region(
        buffer.marktree(),
        at,
        extent.row,
        extent.col,
        new.row,
        new.col,
    );

    signcols_count_range(buffer, row1, row2, 0, SignCountHalf::Add);

    send_splice(buffer, new, Extent::default(), extent);

    if undo == kExtmarkUndo {
        let Some(link) = undo_header(buffer) else {
            return;
        };

        let move_0 = ExtmarkMove {
            start_row: start.row,
            start_col: start.col.get(),
            extent_row: extent.row,
            extent_col: extent.col.get(),
            new_row: new.row,
            new_col: new.col.get(),
            start_byte: start.byte,
            extent_byte: extent.byte,
            new_byte: new.byte,
        };
        with_undo_marks(buffer, link, |list| {
            list.push(ExtmarkUndoObject::Move(move_0))
        });
    }
}

/// How a single-line splice folds into the undo header's last entry: an
/// insert typed at, inside or just past the last insert grows it; a forward
/// delete at its end and a backspace onto its start grow a delete; anything
/// else, or a last entry that spans rows or is not a splice, is kept as an
/// entry of its own. The oracle for retyping the rows and columns these
/// records carry.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ExtmarkSavePos;

    fn bytes(n: i32) -> BCount {
        BCount::try_from(n).expect("a small count")
    }

    /// A single-line splice on row 3: `col`, then the old and new widths,
    /// with byte counts equal to the widths and offsets at 100 + column.
    fn edit(col: i32, old: i32, new: i32) -> (Extent, Extent, Extent) {
        let start = Extent {
            row: 3,
            col: ColNr::new(col),
            byte: bytes(100 + col),
        };
        let old = Extent {
            row: 0,
            col: ColNr::new(old),
            byte: bytes(old),
        };
        let new = Extent {
            row: 0,
            col: ColNr::new(new),
            byte: bytes(new),
        };
        (start, old, new)
    }

    fn recorded(col: i32, old: i32, new: i32) -> ExtmarkUndoObject {
        let (start, old, new) = edit(col, old, new);
        ExtmarkUndoObject::Splice(ExtmarkSplice {
            start_row: start.row,
            start_col: start.col,
            old_row: old.row,
            old_col: old.col,
            new_row: new.row,
            new_col: new.col,
            start_byte: start.byte,
            old_byte: old.byte,
            new_byte: new.byte,
        })
    }

    /// The last entry as (start col, start byte, old col, old byte, new col,
    /// new byte).
    fn last(list: &mut [ExtmarkUndoObject]) -> (i32, BCount, i32, BCount, i32, BCount) {
        let s = last_splice(list).expect("a splice");
        (
            s.start_col.get(),
            s.start_byte,
            s.old_col.get(),
            s.old_byte,
            s.new_col.get(),
            s.new_byte,
        )
    }

    /// Fold `(col, old, new)` into `list`, answering whether it merged.
    fn fold(list: &mut [ExtmarkUndoObject], col: i32, old: i32, new: i32) -> bool {
        let (start, old, new) = edit(col, old, new);
        merge_into_last(list, start, old, new)
    }

    #[test]
    fn typing_on_grows_the_last_insert() {
        let mut list = vec![recorded(4, 0, 1)];
        assert!(fold(&mut list, 5, 0, 1), "just past it");
        assert!(fold(&mut list, 4, 0, 2), "at its start");
        assert!(fold(&mut list, 6, 0, 1), "inside it");
        assert_eq!(last(&mut list), (4, 104, 0, 0, 5, 5));
        assert!(!fold(&mut list, 10, 0, 1), "one past its end");
        assert!(!fold(&mut list, 3, 0, 1), "before its start");
    }

    #[test]
    fn a_forward_delete_at_the_end_grows_the_last_change() {
        let mut list = vec![recorded(4, 1, 0)];
        assert!(fold(&mut list, 4, 1, 0));
        assert!(fold(&mut list, 4, 2, 0));
        assert_eq!(last(&mut list), (4, 104, 4, 4, 0, 0));
    }

    #[test]
    fn a_backspace_onto_the_start_moves_it_back() {
        let mut list = vec![recorded(4, 1, 0)];
        assert!(fold(&mut list, 3, 1, 0));
        assert!(fold(&mut list, 1, 2, 0));
        assert_eq!(last(&mut list), (1, 101, 4, 4, 0, 0));
        assert!(!fold(&mut list, 0, 2, 0), "it overlaps rather than abuts");
    }

    #[test]
    fn an_insert_where_a_delete_was_becomes_a_replacement() {
        let mut list = vec![recorded(4, 2, 0)];
        assert!(fold(&mut list, 4, 0, 3));
        assert_eq!(last(&mut list), (4, 104, 2, 2, 3, 3));
        // A delete after an insert merges only at the insert's end.
        let mut list = vec![recorded(4, 0, 3)];
        assert!(!fold(&mut list, 5, 1, 0));
        assert!(fold(&mut list, 7, 1, 0));
        assert_eq!(last(&mut list), (4, 104, 1, 1, 3, 3));
    }

    #[test]
    fn a_change_that_replaces_and_inserts_at_once_never_merges() {
        let mut list = vec![recorded(4, 1, 1)];
        assert!(!fold(&mut list, 5, 1, 1));
        assert_eq!(last(&mut list), (4, 104, 1, 1, 1, 1));
    }

    #[test]
    fn only_a_single_line_splice_on_the_same_row_is_a_candidate() {
        let mut empty: Vec<ExtmarkUndoObject> = Vec::new();
        assert!(!fold(&mut empty, 4, 0, 1));

        let mut other_row = vec![recorded(4, 0, 1)];
        if let ExtmarkUndoObject::Splice(s) = &mut other_row[0] {
            s.start_row = 2;
        }
        assert!(!fold(&mut other_row, 5, 0, 1));

        let mut spans_rows = vec![recorded(4, 0, 1)];
        if let ExtmarkUndoObject::Splice(s) = &mut spans_rows[0] {
            s.new_row = 1;
        }
        assert!(!fold(&mut spans_rows, 5, 0, 1));

        let mut saved = vec![
            recorded(4, 0, 1),
            ExtmarkUndoObject::SavePos(ExtmarkSavePos {
                mark: 1,
                old_row: 3,
                old_col: ColNr::new(4),
                invalidated: false,
            }),
        ];
        assert!(!fold(&mut saved, 5, 0, 1), "the last entry is not a splice");
    }
}
