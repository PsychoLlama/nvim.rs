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

use core::ffi::c_int;

use super::undo::splice_delete;
use super::{
    Buf, Extent, kExtmarkUndo, last_splice, line_offset, push_undo, send_splice,
    signcols_count_range, splice_pending, tree_move_region, tree_splice, undo_marks,
};
use crate::decoration::SignCountHalf;
use crate::pos::MAXLNUM;
use crate::types::{
    BCount, ColNr, ExtmarkMove, ExtmarkOp, ExtmarkSplice, ExtmarkUndoObject, LineNr, MTPos,
    extmark_undo_vec_t,
};

/// [`extmark_adjust`](super::extmark_adjust) for the callers that already
/// hold a [`Buf`].
pub(crate) fn adjust(
    buffer: Buf,
    line1: LineNr,
    line2: LineNr,
    amount: LineNr,
    amount_after: LineNr,
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
    if amount == MAXLNUM {
        old_row = line2 - line1 + 1;
        // TODO(bfredl): ej kasta?
        old_byte = buffer.deleted_bytes2 as BCount;
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
        row: line1 - 1,
        col: 0,
        byte: start_byte,
    };
    let old = Extent {
        row: old_row,
        col: 0,
        byte: old_byte,
    };
    let new = Extent {
        row: new_row,
        col: 0,
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
    let mut offset = line_offset(buffer, start_row + 1);

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
        byte: (offset + start_col) as BCount,
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

    if old.row > 0 || old.col > 0 {
        // Copy and invalidate the marks a delete would affect.
        // TODO(bfredl): be smart about marks already saved (important for
        // the merge below).
        let end_row = start.row + old.row;
        let end_col = (if old.row != 0 { 0 } else { start.col }) + old.col;
        let uvp = undo_marks(buffer);
        let lo = MTPos {
            row: start.row,
            col: start.col,
        };
        let hi = MTPos {
            row: end_row,
            col: end_col,
        };
        splice_delete(buffer, lo, hi, uvp, false, undo);
    }

    // Take the signs inside the edited region out of `b_signcols.count`; they
    // go back after the splice.
    if old.row > 0 || new.row > 0 {
        let count = if buffer.b_prev_line_count > 0 {
            buffer.b_prev_line_count
        } else {
            buffer.b_ml.ml_line_count
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
        col: start.col,
    };
    tree_splice(buffer.marktree(), at, old.row, old.col, new.row, new.col);

    if old.row > 0 || new.row > 0 {
        let row2 = (buffer.b_ml.ml_line_count - 1).min(start.row + new.row);
        signcols_count_range(buffer, start.row, row2, 0, SignCountHalf::Add);
    }

    if undo == kExtmarkUndo {
        let uvp = undo_marks(buffer);
        if uvp.is_null() {
            return;
        }

        // TODO(bfredl): this is quite rudimentary. Small (within line)
        // inserts merge with each other and small deletes with each other;
        // add a full merge algorithm later.
        let merged = old.row == 0 && new.row == 0 && merge_into_last(uvp, start, old, new);

        if !merged {
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
            push_undo(uvp, ExtmarkUndoObject::Splice(splice));
        }
    }
}

/// Fold this splice into the undo header's last entry, when both are
/// single-line edits that abut. Answers whether it merged.
fn merge_into_last(uvp: *mut extmark_undo_vec_t, start: Extent, old: Extent, new: Extent) -> bool {
    let Some(splice) = last_splice(uvp) else {
        return false;
    };
    if splice.start_row != start.row || splice.old_row != 0 || splice.new_row != 0 {
        return false;
    }
    if old.col == 0
        && start.col >= splice.start_col
        && start.col <= splice.start_col + splice.new_col
    {
        splice.new_col += new.col;
        splice.new_byte += new.byte;
        true
    } else if new.col == 0 && start.col == splice.start_col + splice.new_col {
        splice.old_col += old.col;
        splice.old_byte += old.byte;
        true
    } else if new.col == 0 && start.col + old.col == splice.start_col {
        splice.start_col = start.col;
        splice.start_byte = start.byte;
        splice.old_col += old.col;
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
        col: start.col,
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
        let uvp = undo_marks(buffer);
        if uvp.is_null() {
            return;
        }

        let move_0 = ExtmarkMove {
            start_row: start.row,
            start_col: start.col,
            extent_row: extent.row,
            extent_col: extent.col,
            new_row: new.row,
            new_col: new.col,
            start_byte: start.byte,
            extent_byte: extent.byte,
            new_byte: new.byte,
        };
        push_undo(uvp, ExtmarkUndoObject::Move(move_0));
    }
}
