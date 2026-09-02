//! Undoing and redoing a change's effect on marks.
//!
//! A change records what it did to the marks in an
//! `ExtmarkUndoObject`, and [`extmark_apply_undo`] plays that back in either
//! direction: a splice is inverted, a region move is moved back, and a
//! `kExtmarkSavePos` entry restores a mark that the change deleted outright.
//! [`extmark_splice_delete`] is the recording side -- it walks the marks
//! inside a range being deleted and decides, per mark, whether it is
//! invalidated, moved to the edge or saved for restoration.
//!
//! Original: `src/nvim/extmark.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::del::del;
use super::set::extmark_setraw;
use super::splice::{move_region, splice_impl};
use super::{
    Buf, Extent, current_buf, decor_remove, itr_current, itr_get, itr_next, itr_rawkey,
    kExtmarkNoUndo, kExtmarkUndo, push_undo, tree_get_altpos, tree_revise_meta,
};
use crate::marktree::key::{
    MtFlags, mt_decor, mt_end, mt_invalid, mt_invalidate, mt_lookup_key, mt_no_undo, mt_paired,
    mt_right,
};
use crate::types::{
    ExtmarkOp, ExtmarkSavePos, ExtmarkUndoObject, MTPos, MarkTreeIter, buf_T, colnr_T,
    extmark_undo_vec_t,
};

/// Invalidate the marks inside a range being deleted, and copy the ones the
/// change cannot simply be reversed for into the undo header.
///
/// Copying does nothing on redo; it enforces the right position on undo.
pub unsafe fn extmark_splice_delete(
    buf: *mut buf_T,
    l_row: c_int,
    l_col: colnr_T,
    u_row: c_int,
    u_col: colnr_T,
    uvp: *mut extmark_undo_vec_t,
    only_copy: bool,
    op: ExtmarkOp,
) {
    let lo = MTPos {
        row: l_row,
        col: l_col,
    };
    let hi = MTPos {
        row: u_row,
        col: u_col,
    };
    // SAFETY: the caller's promise -- a live buffer, and a `uvp` that is NULL
    // or an undo header's own extmark list.
    splice_delete(unsafe { Buf::new(buf) }, lo, hi, uvp, only_copy, op);
}

/// [`extmark_splice_delete`] for the callers that already hold a [`Buf`].
pub(crate) fn splice_delete(
    mut buf: Buf,
    lo: MTPos,
    hi: MTPos,
    uvp: *mut extmark_undo_vec_t,
    only_copy: bool,
    op: ExtmarkOp,
) {
    let mut itr = MarkTreeIter::default();

    itr_get(buf.marktree(), lo.row, lo.col, &mut itr);
    loop {
        let mark = itr_current(&mut itr);
        if mark.pos.row < 0 || mark.pos.row > hi.row {
            break;
        }

        // Left gravity marks at the start of the range and right gravity
        // marks at its end need no copy, unless they are invalidated.
        let mut copy = true;
        if mark.pos.row == lo.row && mark.pos.col - c_int::from(!mt_right(mark)) < lo.col {
            copy = false;
        } else if mark.pos.row == hi.row {
            if mark.pos.col > hi.col + 1 {
                break;
            }
            if mark.pos.col + c_int::from(mt_right(mark)) > hi.col {
                copy = false;
            }
        }

        let mut invalidated = false;
        if !only_copy && !mt_invalid(mark) && mt_invalidate(mark) && !mt_end(mark) {
            let mut enditr = itr;
            let endpos = tree_get_altpos(buf.marktree(), mark, Some(&mut enditr));
            // Invalidate unpaired marks in deleted lines, and paired marks
            // whose entire range has been deleted.
            let unpaired_gone = !mt_paired(mark) && mark.pos.row < hi.row;
            let pair_gone = mt_paired(mark)
                && (mark.pos.row > lo.row || (mark.pos.row == lo.row && mark.pos.col >= lo.col))
                && (endpos.row < hi.row || (endpos.row == hi.row && endpos.col <= hi.col));
            if unpaired_gone || pair_gone {
                if mt_no_undo(mark) {
                    del(buf, &mut itr, mark, true);
                    continue;
                }
                copy = true;
                invalidated = true;
                itr_rawkey(&mut itr).flags |= MtFlags::INVALID;
                itr_rawkey(&mut enditr).flags |= MtFlags::INVALID;
                tree_revise_meta(buf.marktree(), &mut itr, mark);
                decor_remove(
                    buf,
                    mark.pos.row,
                    endpos.row,
                    mark.pos.col,
                    mt_decor(mark),
                    false,
                );
            }
        }

        // Push the mark to the undo header. Note that upstream dereferences
        // `uvp` for the `only_copy` half without testing it -- the one caller
        // that passes `only_copy` (`save_orig_extmarks`) always passes its own
        // list, so the NULL case is unreachable rather than guarded.
        if copy && (only_copy || (!uvp.is_null() && op == kExtmarkUndo && !mt_no_undo(mark))) {
            let pos = ExtmarkSavePos {
                mark: mt_lookup_key(mark),
                old_row: mark.pos.row,
                old_col: mark.pos.col,
                invalidated,
            };
            push_undo(uvp, ExtmarkUndoObject::SavePos(pos));
        }

        itr_next(buf.marktree(), &mut itr);
    }
}

/// Undo or redo one recorded extmark operation.
pub unsafe fn extmark_apply_undo(undo_info: ExtmarkUndoObject, undo: bool) {
    let buf = current_buf();
    if let ExtmarkUndoObject::Splice(splice) = undo_info {
        // A splice: any text operation that changes position except `:move`.
        let start = Extent {
            row: splice.start_row,
            col: splice.start_col,
            byte: splice.start_byte,
        };
        let old = Extent {
            row: splice.old_row,
            col: splice.old_col,
            byte: splice.old_byte,
        };
        let new = Extent {
            row: splice.new_row,
            col: splice.new_col,
            byte: splice.new_byte,
        };
        if undo {
            splice_impl(buf, start, new, old, kExtmarkNoUndo);
        } else {
            splice_impl(buf, start, old, new, kExtmarkNoUndo);
        }
    } else if let ExtmarkUndoObject::SavePos(pos) = undo_info {
        if undo && pos.old_row >= 0 {
            extmark_setraw(buf, pos.mark, pos.old_row, pos.old_col, pos.invalidated);
        }
        // No redo: the `kExtmarkSplice` entry moves the marks back.
    } else if let ExtmarkUndoObject::Move(move_0) = undo_info {
        let start = Extent {
            row: move_0.start_row,
            col: move_0.start_col,
            byte: move_0.start_byte,
        };
        let extent = Extent {
            row: move_0.extent_row,
            col: move_0.extent_col,
            byte: move_0.extent_byte,
        };
        let new = Extent {
            row: move_0.new_row,
            col: move_0.new_col,
            byte: move_0.new_byte,
        };
        if undo {
            move_region(buf, new, extent, start, kExtmarkNoUndo);
        } else {
            move_region(buf, start, extent, new, kExtmarkNoUndo);
        }
    }
}
