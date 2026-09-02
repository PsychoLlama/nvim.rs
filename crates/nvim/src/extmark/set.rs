//! Placing a mark -- `extmark_set()`.
//!
//! [`extmark_set`] is the `nvim_buf_set_extmark()` half: it decides whether
//! the id names an existing mark (and therefore a move rather than an insert),
//! whether the mark is paired and needs an end key, and which decoration to
//! attach, then puts the key or keys into the buffer's marktree and updates
//! the sign and conceal counts the decoration layer keeps.
//! [`extmark_setraw`] is the unconditional form that skips all of that.
//!
//! Original: `src/nvim/extmark.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::del::del_id;
use super::{
    Buf, current_buf, decor_remove, invalidate_decor_state, itr_rawkey, ns_put_ref, put_decor,
    redraw_decor, signcols_count_range, tree_del_itr, tree_get_alt, tree_lookup, tree_lookup_ns,
    tree_move, tree_put, tree_revise_meta,
};
use crate::decoration::SignCountHalf;
use crate::marktree::key::{
    MtFlags, mt_decor, mt_decor_any, mt_end, mt_flags, mt_invalid, mt_paired,
};
use crate::types::{DecorInline, MTKey, MTPos, MarkTreeIter, buf_T, colnr_T, uint32_t, uint64_t};

/// Create or update an extmark.
///
/// Must not be used during iteration.
pub unsafe fn extmark_set(
    buf: *mut buf_T,
    ns_id: uint32_t,
    idp: *mut uint32_t,
    row: c_int,
    col: colnr_T,
    end_row: c_int,
    end_col: colnr_T,
    decor: DecorInline,
    decor_flags: MtFlags,
    right_gravity: bool,
    end_right_gravity: bool,
    no_undo: bool,
    invalidate: bool,
) {
    // SAFETY: the caller's promise -- a live buffer, and an `idp` that is
    // NULL or points at a mark id.
    let mut buf = unsafe { Buf::new(buf) };
    let ns = ns_put_ref(buf.extmark_ns(), ns_id);
    // SAFETY: `map_put_ref` never answers NULL; it creates the slot if the
    // namespace had none, and the map owns it until the map is destroyed.
    let ns = unsafe { &mut *ns };
    let mut id = if idp.is_null() {
        0
    } else {
        // SAFETY: the caller's promise, as above.
        unsafe { *idp }
    };

    let flags = mt_flags(right_gravity, no_undo, invalidate, decor.ext) | decor_flags;
    let mut revised = false;
    if id == 0 {
        *ns += 1;
        id = *ns;
    } else {
        let mut itr = MarkTreeIter::default();
        let old_mark = tree_lookup_ns(buf.marktree(), ns_id, id, false, Some(&mut itr));
        if old_mark.id != 0 {
            if mt_paired(old_mark) || end_row > -1 {
                del_id(buf, ns_id, id);
            } else {
                debug_assert!(!itr.x.is_null(), "marktree_itr_valid(itr)");
                if old_mark.pos.row == row && old_mark.pos.col == col {
                    // Not paired: the key can be revised where it lies.
                    if !mt_invalid(old_mark) && mt_decor_any(old_mark) {
                        itr_rawkey(&mut itr).flags.clear(MtFlags::EXTERNAL_MASK);
                        decor_remove(buf, row, row, col, mt_decor(old_mark), true);
                    }
                    itr_rawkey(&mut itr).flags |= flags;
                    itr_rawkey(&mut itr).decor_data = decor.data;
                    tree_revise_meta(buf.marktree(), &mut itr, old_mark);
                    revised = true;
                } else {
                    tree_del_itr(buf.marktree(), &mut itr, false);
                    if !mt_invalid(old_mark) {
                        decor_remove(
                            buf,
                            old_mark.pos.row,
                            old_mark.pos.row,
                            old_mark.pos.col,
                            mt_decor(old_mark),
                            true,
                        );
                    }
                }
            }
        } else {
            *ns = (*ns).max(id);
        }
    }

    if !revised {
        let mark = MTKey {
            pos: MTPos { row, col },
            ns: ns_id,
            id,
            flags,
            decor_data: decor.data,
        };
        tree_put(buf.marktree(), mark, end_row, end_col, end_right_gravity);
        invalidate_decor_state(buf);
    }

    if !decor_flags.is_empty() || decor.ext {
        let last_row = if end_row > -1 { end_row } else { row };
        put_decor(buf, decor, row, last_row);
        redraw_decor(buf, row, last_row, col, decor);
    }

    if !idp.is_null() {
        // SAFETY: the caller's promise, as above.
        unsafe { *idp = id };
    }
}

/// Put the mark `mark` back at (`row`, `col`), as an undo step does.
///
/// `static` upstream; only [`extmark_apply_undo`](super::extmark_apply_undo)
/// reaches it.
pub(crate) fn extmark_setraw(
    mut buf: Buf,
    mark: uint64_t,
    row: c_int,
    col: colnr_T,
    invalid: bool,
) {
    let mut itr = MarkTreeIter::default();
    let key = tree_lookup(buf.marktree(), mark, Some(&mut itr));
    let move_0 = key.pos.row != row || key.pos.col != col;
    if key.pos.row < 0 || (!move_0 && !invalid) {
        // The mark was deleted, or nothing has to change.
        return;
    }

    // Only the position before the undo needs redrawing here; the position
    // after it is marked changed anyway.
    if !invalid && mt_decor_any(key) && key.pos.row != row {
        redraw_decor(buf, key.pos.row, key.pos.row, key.pos.col, mt_decor(key));
    }

    let mut row1 = 0;
    let mut row2 = 0;
    let mut altitr = itr;
    let alt = tree_get_alt(buf.marktree(), key, Some(&mut altitr));

    if invalid {
        itr_rawkey(&mut itr).flags.clear(MtFlags::INVALID);
        itr_rawkey(&mut altitr).flags.clear(MtFlags::INVALID);
        let (revised, old) = if mt_end(key) {
            (&mut altitr, alt)
        } else {
            (&mut itr, key)
        };
        tree_revise_meta(buf.marktree(), revised, old);
    } else if !mt_invalid(key) && key.flags.has(MtFlags::DECOR_SIGNTEXT) && buf.b_signcols.autom {
        row1 = alt.pos.row.min(key.pos.row.min(row));
        row2 = alt.pos.row.max(key.pos.row.max(row));
        signcols_count_range(buf, row1, last_line().min(row2), 0, SignCountHalf::Subtract);
    }

    if move_0 {
        tree_move(buf.marktree(), &mut itr, row, col);
    }

    if invalid {
        put_decor(
            buf,
            mt_decor(key),
            row.min(alt.pos.row),
            row.max(alt.pos.row),
        );
    } else if !mt_invalid(key) && key.flags.has(MtFlags::DECOR_SIGNTEXT) && buf.b_signcols.autom {
        signcols_count_range(buf, row1, last_line().min(row2), 0, SignCountHalf::Add);
    }
}

/// `curbuf->b_ml.ml_line_count - 1`, the last row of the *current* buffer --
/// which upstream reads here even though every other line in
/// [`extmark_setraw`] is about `buf`.
fn last_line() -> c_int {
    current_buf().b_ml.ml_line_count - 1
}
