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
    ns_destroy, ns_ref, tree_del_itr, tree_lookup, tree_lookup_ns,
};
use crate::marktree::key::{mt_decor, mt_decor_any, mt_end, mt_invalid};
use crate::types::{MTKey, MarkTreeIter, buf_T, colnr_T, uint32_t};

/// Remove the extmark `id` of namespace `ns_id`.
///
/// Answers false when there is no such mark.
pub unsafe fn extmark_del_id(buf: *mut buf_T, ns_id: uint32_t, id: uint32_t) -> bool {
    // SAFETY: the caller's promise -- a live buffer.
    del_id(unsafe { Buf::new(buf) }, ns_id, id)
}

/// [`extmark_del_id`] for the callers that already hold a [`Buf`].
pub(crate) fn del_id(mut buf: Buf, ns_id: uint32_t, id: uint32_t) -> bool {
    let mut itr = MarkTreeIter::default();
    let key = tree_lookup_ns(buf.marktree(), ns_id, id, false, Some(&mut itr));
    if key.id != 0 {
        del(buf, &mut itr, key, false);
    }
    key.id > 0
}

/// Remove the (possibly paired) extmark `key` that `itr` is on.
pub unsafe fn extmark_del(buf: *mut buf_T, itr: *mut MarkTreeIter, key: MTKey, restore: bool) {
    // SAFETY: the caller's promise -- a live buffer and an iterator
    // positioned in its marktree, both of which outlive the call.
    del(unsafe { Buf::new(buf) }, unsafe { &mut *itr }, key, restore);
}

/// [`extmark_del`] for the callers that already hold the two.
pub(crate) fn del(mut buf: Buf, itr: &mut MarkTreeIter, mut key: MTKey, restore: bool) {
    debug_assert!(key.pos.row >= 0, "key.pos.row >= 0");

    let mut key2 = key;
    let other = tree_del_itr(buf.marktree(), itr, false);
    if other != 0 {
        key2 = tree_lookup(buf.marktree(), other, Some(itr));
        debug_assert!(key2.pos.row >= 0, "key2.pos.row >= 0");
        tree_del_itr(buf.marktree(), itr, false);
        if restore {
            itr_get(buf.marktree(), key.pos.row, key.pos.col, itr);
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
                buf,
                key.pos.row,
                key2.pos.row,
                key.pos.col,
                mt_decor(key),
                true,
            );
        }
    }

    invalidate_decor_state(buf);

    // TODO(bfredl): delete it from the current undo header, opportunistically?
}

/// Free every mark of namespace `ns_id` (or of every namespace, when it is 0)
/// between two positions.
pub unsafe fn extmark_clear(
    buf: *mut buf_T,
    ns_id: uint32_t,
    l_row: c_int,
    l_col: colnr_T,
    u_row: c_int,
    u_col: colnr_T,
) -> bool {
    // SAFETY: the caller's promise -- a live buffer.
    let mut buf = unsafe { Buf::new(buf) };
    if buf.extmark_ns().set.h.size == 0 {
        return false;
    }

    let all_ns = ns_id == 0;
    if !all_ns && ns_ref(buf.extmark_ns(), ns_id).is_null() {
        // Nothing to do.
        return false;
    }

    let mut marks_cleared_any = false;
    let mut marks_cleared_all = l_row == 0 && l_col == 0;

    let mut itr = MarkTreeIter::default();
    itr_get(buf.marktree(), l_row, l_col, &mut itr);
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
            del(buf, &mut itr, mark, true);
        } else {
            itr_next(buf.marktree(), &mut itr);
        }
    }

    if marks_cleared_all {
        if all_ns {
            ns_destroy(buf.extmark_ns());
        } else {
            ns_del(buf.extmark_ns(), ns_id);
        }
    }

    if marks_cleared_any {
        invalidate_decor_state(buf);
    }

    marks_cleared_any
}
