//! Reading marks back -- `extmark_get()`, and freeing them all.
//!
//! [`extmark_get`] is `nvim_buf_get_extmarks()`: walk the buffer's marktree
//! between two positions, in either direction, optionally including marks
//! that merely overlap the range, and push each one onto the answer array
//! with [`push_mark`] -- with its end position and its decoration if the
//! caller asked for them.  [`extmark_from_id`] is the single-mark lookup and
//! [`extmark_free_all`] the teardown when a buffer is freed.
//!
//! Original: `src/nvim/extmark.c`, Vim/Neovim, Vim license.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::{
    Buf, KV_INITIAL_VALUE, free_decor, itr_current, itr_get, itr_get_ext, itr_get_overlap,
    itr_next, itr_step_overlap, kExtmarkNone, kv_push, ns_destroy, tree_clear, tree_get_alt,
    tree_lookup_ns, type_flags,
};
use crate::marktree::key::{
    MT_INVALID_KEY, mt_decor, mt_decor_any, mt_end, mt_paired, mtpair_from,
};
use crate::types::{
    ColNr, ExtmarkInfoArray, ExtmarkType, MTPair, MTPos, MarkTreeIter, int64_t, uint32_t,
};

/// Every mark between two positions, the ones at either end included.
///
/// `amount` is the caller's limit, `INT64_MAX` for "all of them".
pub fn extmark_get(
    mut buffer: Buf,
    ns_id: uint32_t,
    l_row: c_int,
    l_col: ColNr,
    u_row: c_int,
    u_col: ColNr,
    amount: int64_t,
    type_filter: ExtmarkType,
    overlap: bool,
) -> ExtmarkInfoArray {
    let mut array: ExtmarkInfoArray = KV_INITIAL_VALUE;
    let mut itr = MarkTreeIter::default();
    let under_limit = |array: &ExtmarkInfoArray| {
        int64_t::try_from(array.size).expect("a mark answer never outgrows an i64") < amount
    };

    if overlap {
        // Every mark overlapping the start position.
        if !itr_get_overlap(buffer.marktree(), l_row, l_col, &mut itr) {
            return array;
        }

        while under_limit(&array) {
            // Invalid until `itr_step_overlap` writes it, which it does
            // whenever it answers true (upstream leaves it uninitialised).
            let mut pair = mtpair_from(MT_INVALID_KEY, MT_INVALID_KEY);
            if !itr_step_overlap(buffer.marktree(), &mut itr, &mut pair) {
                break;
            }
            push_mark(&mut array, ns_id, type_filter, pair);
        }
    } else {
        // Every mark beginning at or after the start position.
        let start = MTPos {
            row: l_row,
            col: l_col,
        };
        itr_get_ext(buffer.marktree(), start, &mut itr);
    }

    while under_limit(&array) {
        let mark = itr_current(&mut itr);
        if mark.pos.row < 0
            || mark.pos.row > u_row
            || (mark.pos.row == u_row && mark.pos.col > u_col)
        {
            break;
        }
        if !mt_end(mark) {
            let end = tree_get_alt(buffer.marktree(), mark, None);
            push_mark(&mut array, ns_id, type_filter, mtpair_from(mark, end));
        }
        itr_next(buffer.marktree(), &mut itr);
    }
    array
}

/// Add `mark` to the answer, unless a namespace or decoration-type filter
/// rules it out. `ns_id` is `UINT32_MAX` for "any namespace".
fn push_mark(
    array: &mut ExtmarkInfoArray,
    ns_id: uint32_t,
    type_filter: ExtmarkType,
    mark: MTPair,
) {
    if !(ns_id == uint32_t::MAX || mark.start.ns == ns_id) {
        return;
    }
    if type_filter != kExtmarkNone {
        if !mt_decor_any(mark.start) {
            return;
        }
        if ExtmarkType::from(type_flags(mt_decor(mark.start))) & type_filter == 0 {
            return;
        }
    }

    kv_push(&mut array.size, &mut array.capacity, &mut array.items, mark);
}

/// The extmark `id` of namespace `ns_id`, paired with its end position.
pub fn extmark_from_id(mut buffer: Buf, ns_id: uint32_t, id: uint32_t) -> MTPair {
    let mark = tree_lookup_ns(buffer.marktree(), ns_id, id, false, None);
    if mark.id == 0 {
        // Invalid.
        return mtpair_from(mark, mark);
    }
    debug_assert!(mark.pos.row >= 0, "mark.pos.row >= 0");
    let end = tree_get_alt(buffer.marktree(), mark, None);

    mtpair_from(mark, end)
}

/// Release every mark of a buffer, as it is freed.
pub fn extmark_free_all(mut buffer: Buf) {
    let mut itr = MarkTreeIter::default();
    itr_get(buffer.marktree(), 0, 0, &mut itr);
    loop {
        let mark = itr_current(&mut itr);
        if mark.pos.row < 0 {
            break;
        }

        // Don't free mark.decor twice for a paired mark.
        if !(mt_paired(mark) && mt_end(mark)) {
            free_decor(mt_decor(mark));
        }

        itr_next(buffer.marktree(), &mut itr);
    }

    tree_clear(buffer.marktree());

    buffer.b_signcols.max = 0;
    buffer.b_signcols.count = [0; 9];

    ns_destroy(buffer.extmark_ns());
}
