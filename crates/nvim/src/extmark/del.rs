//! Removing marks -- `extmark_del()` and `extmark_clear()`.
//!
//! [`extmark_del_id`] and [`extmark_del`] remove one mark (and its paired end
//! key, if it has one), releasing the decoration it carried and dropping it
//! from the namespace's id map.  [`extmark_clear`] is the range form used by
//! `nvim_buf_clear_namespace()`: walk the marktree between two positions,
//! delete every mark in the given namespace, and pick up the pairs that only
//! overlap the range rather than starting in it.
//!
//! Original: `src/nvim/extmark.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;
use core::mem;

use super::{
    Buf, decor_remove, free_decor, invalidate_decor_state, itr_current, itr_get, itr_next, ns_del,
    ns_destroy, ns_has, tree_del_itr, tree_lookup, tree_lookup_ns,
};
use crate::marktree::key::{mt_decor, mt_decor_any, mt_end, mt_invalid};
use crate::types::{ColNr, MTKey, MarkTreeIter, uint32_t};

/// Remove the extmark `id` of namespace `ns_id`.
///
/// Answers false when there is no such mark.
pub fn extmark_del_id(buffer: Buf, ns_id: uint32_t, id: uint32_t) -> bool {
    del_id(buffer, ns_id, id)
}

/// [`extmark_del_id`] for the callers that already hold a [`Buf`].
pub(crate) fn del_id(mut buffer: Buf, ns_id: uint32_t, id: uint32_t) -> bool {
    let mut itr = MarkTreeIter::default();
    let key = tree_lookup_ns(buffer.marktree(), ns_id, id, false, Some(&mut itr));
    if key.id != 0 {
        del(buffer, &mut itr, key, false);
    }
    key.id > 0
}

/// Remove the (possibly paired) extmark `key` that `itr` is on.
pub unsafe fn extmark_del(buffer: Buf, itr: *mut MarkTreeIter, key: MTKey, restore: bool) {
    // SAFETY: the caller's promise -- a live buffer and an iterator
    // positioned in its marktree, both of which outlive the call.
    del(buffer, unsafe { &mut *itr }, key, restore);
}

/// [`extmark_del`] for the callers that already hold the two.
pub(crate) fn del(mut buffer: Buf, itr: &mut MarkTreeIter, mut key: MTKey, restore: bool) {
    debug_assert!(key.pos.row >= 0, "key.pos.row >= 0");

    let mut key2 = key;
    let other = tree_del_itr(buffer.marktree(), itr, false);
    if other != 0 {
        key2 = tree_lookup(buffer.marktree(), other, Some(itr));
        debug_assert!(key2.pos.row >= 0, "key2.pos.row >= 0");
        tree_del_itr(buffer.marktree(), itr, false);
        if restore {
            itr_get(buffer.marktree(), key.pos.row, key.pos.col, itr);
        }
    }

    if mt_decor_any(key) {
        if mt_invalid(key) {
            free_decor(mt_decor(key));
        } else {
            if mt_end(key) {
                mem::swap(&mut key, &mut key2);
            }
            decor_remove(
                buffer,
                key.pos.row,
                key2.pos.row,
                key.pos.col,
                mt_decor(key),
                true,
            );
        }
    }

    invalidate_decor_state(buffer);

    // TODO(bfredl): delete it from the current undo header, opportunistically?
}

/// Free every mark of namespace `ns_id` (or of every namespace, when it is 0)
/// between two positions.
pub fn extmark_clear(
    mut buffer: Buf,
    ns_id: uint32_t,
    l_row: c_int,
    l_col: ColNr,
    u_row: c_int,
    u_col: ColNr,
) -> bool {
    if buffer.extmark_ns().is_empty() {
        return false;
    }

    let all_ns = ns_id == 0;
    if !all_ns && !ns_has(buffer.extmark_ns(), ns_id) {
        // Nothing to do.
        return false;
    }

    let mut marks_cleared_any = false;
    let mut marks_cleared_all = l_row == 0 && l_col == 0;

    let mut itr = MarkTreeIter::default();
    itr_get(buffer.marktree(), l_row, l_col, &mut itr);
    loop {
        let mark = itr_current(&mut itr);
        if mark.pos.row < 0
            || mark.pos.row > u_row
            || (mark.pos.row == u_row && mark.pos.col > u_col)
        {
            if mark.pos.row >= 0 {
                marks_cleared_all = false;
            }
            break;
        }
        if mark.ns == ns_id || all_ns {
            marks_cleared_any = true;
            del(buffer, &mut itr, mark, true);
        } else {
            itr_next(buffer.marktree(), &mut itr);
        }
    }

    if marks_cleared_all {
        if all_ns {
            ns_destroy(buffer.extmark_ns());
        } else {
            ns_del(buffer.extmark_ns(), ns_id);
        }
    }

    if marks_cleared_any {
        invalidate_decor_state(buffer);
    }

    marks_cleared_any
}
