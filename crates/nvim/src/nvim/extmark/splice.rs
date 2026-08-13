//! Moving marks when the text moves -- `extmark_splice()`.
//!
//! [`extmark_splice_impl`] is the one operation every buffer change goes
//! through: given a start position, the extent of the text removed and the
//! extent of the text inserted, it shifts every mark after the change,
//! adjusts the ones inside it, and hands the same information to the buffer
//! update callbacks.  [`extmark_adjust`] is the line-based wrapper the older
//! callers use, [`extmark_splice_cols`] the single-line one, and
//! [`extmark_move_region`] the form a `:move` needs, where text is removed
//! from one place and inserted at another.
//!
//! Original: `src/nvim/extmark.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::buffer_updates::buf_updates_send_splice;
use crate::src::nvim::decoration::buf_signcols_count_range;

use crate::src::nvim::main::curbuf_splice_pending;
use crate::src::nvim::marktree::{marktree_move_region, marktree_splice};
use crate::src::nvim::memline::ml_find_line_or_offset;
use crate::src::nvim::memory::xrealloc;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::types::{
    ExtmarkMove, ExtmarkOp, ExtmarkSplice, ExtmarkUndoObject, MarkTree, bcount_t, buf_T, colnr_T,
    extmark_undo_vec_t, int32_t, kNone, kTrue, linenr_T, size_t, u_header_T, undo_object,
    undo_object_data as C2Rust_Unnamed_6,
};
use crate::src::nvim::undo::u_force_get_undo_header;

pub unsafe extern "C" fn extmark_adjust(
    mut buf: *mut buf_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
    mut undo: ExtmarkOp,
) {
    unsafe {
        if curbuf_splice_pending.get() != 0 {
            return;
        }
        let mut start_byte: bcount_t = ml_find_line_or_offset(
            buf,
            line1,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            true_0 != 0,
        ) as bcount_t;
        let mut old_byte: bcount_t = 0 as bcount_t;
        let mut new_byte: bcount_t = 0 as bcount_t;
        let mut old_row: ::core::ffi::c_int = 0;
        let mut new_row: ::core::ffi::c_int = 0;
        if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
            old_row = (line2 - line1 + 1 as linenr_T) as ::core::ffi::c_int;
            old_byte = (*buf).deleted_bytes2 as bcount_t;
            new_row = (amount_after + old_row as linenr_T) as ::core::ffi::c_int;
        } else {
            debug_assert!(
                line2 == MAXLNUM as ::core::ffi::c_int as linenr_T,
                "line2 == MAXLNUM"
            );
            old_row = 0 as ::core::ffi::c_int;
            new_row = amount as ::core::ffi::c_int;
        }
        if new_row > 0 as ::core::ffi::c_int {
            new_byte = ml_find_line_or_offset(
                buf,
                line1 + new_row as linenr_T,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                true_0 != 0,
            ) as bcount_t
                - start_byte;
        }
        extmark_splice_impl(
            buf,
            line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            0 as colnr_T,
            start_byte,
            old_row,
            0 as colnr_T,
            old_byte,
            new_row,
            0 as colnr_T,
            new_byte,
            undo,
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn extmark_splice(
    mut buf: *mut buf_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut old_row: ::core::ffi::c_int,
    mut old_col: colnr_T,
    mut old_byte: bcount_t,
    mut new_row: ::core::ffi::c_int,
    mut new_col: colnr_T,
    mut new_byte: bcount_t,
    mut undo: ExtmarkOp,
) {
    unsafe {
        let mut offset: ::core::ffi::c_int = ml_find_line_or_offset(
            buf,
            start_row as linenr_T + 1 as linenr_T,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            true_0 != 0,
        );
        if offset < 0 as ::core::ffi::c_int && (*buf).b_ml.ml_chunksize.is_null() {
            offset = 0 as ::core::ffi::c_int;
        }
        extmark_splice_impl(
            buf,
            start_row,
            start_col,
            (offset as colnr_T + start_col) as bcount_t,
            old_row,
            old_col,
            old_byte,
            new_row,
            new_col,
            new_byte,
            undo,
        );
    }
}

pub unsafe extern "C" fn extmark_splice_impl(
    mut buf: *mut buf_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut start_byte: bcount_t,
    mut old_row: ::core::ffi::c_int,
    mut old_col: colnr_T,
    mut old_byte: bcount_t,
    mut new_row: ::core::ffi::c_int,
    mut new_col: colnr_T,
    mut new_byte: bcount_t,
    mut undo: ExtmarkOp,
) {
    unsafe {
        (*buf).deleted_bytes2 = 0 as size_t;
        buf_updates_send_splice(
            buf, start_row, start_col, start_byte, old_row, old_col, old_byte, new_row, new_col,
            new_byte,
        );
        if old_row > 0 as ::core::ffi::c_int || old_col > 0 as ::core::ffi::c_int {
            let mut end_row: ::core::ffi::c_int = start_row + old_row;
            let mut end_col: ::core::ffi::c_int = (if old_row != 0 {
                0 as ::core::ffi::c_int
            } else {
                start_col as ::core::ffi::c_int
            }) + old_col as ::core::ffi::c_int;
            let mut uhp: *mut u_header_T = u_force_get_undo_header(buf);
            let mut uvp: *mut extmark_undo_vec_t = if !uhp.is_null() {
                &raw mut (*uhp).uh_extmark
            } else {
                ::core::ptr::null_mut::<extmark_undo_vec_t>()
            };
            extmark_splice_delete(
                buf,
                start_row,
                start_col,
                end_row,
                end_col as colnr_T,
                uvp,
                false_0 != 0,
                undo,
            );
        }
        if old_row > 0 as ::core::ffi::c_int || new_row > 0 as ::core::ffi::c_int {
            let mut count: ::core::ffi::c_int =
                if (*buf).b_prev_line_count > 0 as ::core::ffi::c_int {
                    (*buf).b_prev_line_count
                } else {
                    (*buf).b_ml.ml_line_count as ::core::ffi::c_int
                };
            buf_signcols_count_range(
                buf,
                start_row,
                if (count - 1 as ::core::ffi::c_int) < start_row + old_row {
                    count - 1 as ::core::ffi::c_int
                } else {
                    start_row + old_row
                },
                0 as ::core::ffi::c_int,
                kTrue,
            );
            (*buf).b_prev_line_count = 0 as ::core::ffi::c_int;
        }
        marktree_splice(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            start_row as int32_t,
            start_col as ::core::ffi::c_int,
            old_row,
            old_col as ::core::ffi::c_int,
            new_row,
            new_col as ::core::ffi::c_int,
        );
        if old_row > 0 as ::core::ffi::c_int || new_row > 0 as ::core::ffi::c_int {
            let mut row2: ::core::ffi::c_int = if ((*buf).b_ml.ml_line_count - 1 as linenr_T)
                < start_row as linenr_T + new_row as linenr_T
            {
                (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            } else {
                start_row + new_row
            };
            buf_signcols_count_range(buf, start_row, row2, 0 as ::core::ffi::c_int, kNone);
        }
        if undo as ::core::ffi::c_uint == kExtmarkUndo as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut uhp_0: *mut u_header_T = u_force_get_undo_header(buf);
            if uhp_0.is_null() {
                return;
            }
            let mut merged: bool = false_0 != 0;
            if old_row == 0 as ::core::ffi::c_int
                && new_row == 0 as ::core::ffi::c_int
                && (*uhp_0).uh_extmark.size != 0
            {
                let mut item: *mut ExtmarkUndoObject = (*uhp_0)
                    .uh_extmark
                    .items
                    .add((*uhp_0).uh_extmark.size.wrapping_sub(1 as size_t));
                if (*item).type_0 as ::core::ffi::c_uint
                    == kExtmarkSplice as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut splice: *mut ExtmarkSplice = &raw mut (*item).data.splice;
                    if (*splice).start_row == start_row
                        && (*splice).old_row == 0 as ::core::ffi::c_int
                        && (*splice).new_row == 0 as ::core::ffi::c_int
                    {
                        if old_col == 0 as ::core::ffi::c_int
                            && start_col >= (*splice).start_col
                            && start_col <= (*splice).start_col + (*splice).new_col
                        {
                            (*splice).new_col += new_col;
                            (*splice).new_byte += new_byte;
                            merged = true_0 != 0;
                        } else if new_col == 0 as ::core::ffi::c_int
                            && start_col == (*splice).start_col + (*splice).new_col
                        {
                            (*splice).old_col += old_col;
                            (*splice).old_byte += old_byte;
                            merged = true_0 != 0;
                        } else if new_col == 0 as ::core::ffi::c_int
                            && start_col + old_col == (*splice).start_col
                        {
                            (*splice).start_col = start_col;
                            (*splice).start_byte = start_byte;
                            (*splice).old_col += old_col;
                            (*splice).old_byte += old_byte;
                            merged = true_0 != 0;
                        }
                    }
                }
            }
            if !merged {
                let mut splice_0: ExtmarkSplice = ExtmarkSplice {
                    start_row: 0,
                    start_col: 0,
                    old_row: 0,
                    old_col: 0,
                    new_row: 0,
                    new_col: 0,
                    start_byte: 0,
                    old_byte: 0,
                    new_byte: 0,
                };
                splice_0.start_row = start_row;
                splice_0.start_col = start_col;
                splice_0.start_byte = start_byte;
                splice_0.old_row = old_row;
                splice_0.old_col = old_col;
                splice_0.old_byte = old_byte;
                splice_0.new_row = new_row;
                splice_0.new_col = new_col;
                splice_0.new_byte = new_byte;
                if (*uhp_0).uh_extmark.size == (*uhp_0).uh_extmark.capacity {
                    (*uhp_0).uh_extmark.capacity = if (*uhp_0).uh_extmark.capacity != 0 {
                        (*uhp_0).uh_extmark.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    (*uhp_0).uh_extmark.items = xrealloc(
                        (*uhp_0).uh_extmark.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<ExtmarkUndoObject>()
                            .wrapping_mul((*uhp_0).uh_extmark.capacity),
                    ) as *mut ExtmarkUndoObject;
                } else {
                };
                let c2rust_fresh3 = (*uhp_0).uh_extmark.size;
                (*uhp_0).uh_extmark.size = (*uhp_0).uh_extmark.size.wrapping_add(1);
                *(*uhp_0).uh_extmark.items.add(c2rust_fresh3) = undo_object {
                    type_0: kExtmarkSplice,
                    data: C2Rust_Unnamed_6 { splice: splice_0 },
                };
            }
        }
    }
}

pub unsafe extern "C" fn extmark_splice_cols(
    mut buf: *mut buf_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut old_col: colnr_T,
    mut new_col: colnr_T,
    mut undo: ExtmarkOp,
) {
    unsafe {
        extmark_splice(
            buf,
            start_row,
            start_col,
            0 as ::core::ffi::c_int,
            old_col,
            old_col as bcount_t,
            0 as ::core::ffi::c_int,
            new_col,
            new_col as bcount_t,
            undo,
        );
    }
}

pub unsafe extern "C" fn extmark_move_region(
    mut buf: *mut buf_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut start_byte: bcount_t,
    mut extent_row: ::core::ffi::c_int,
    mut extent_col: colnr_T,
    mut extent_byte: bcount_t,
    mut new_row: ::core::ffi::c_int,
    mut new_col: colnr_T,
    mut new_byte: bcount_t,
    mut undo: ExtmarkOp,
) {
    unsafe {
        (*buf).deleted_bytes2 = 0 as size_t;
        buf_updates_send_splice(
            buf,
            start_row,
            start_col,
            start_byte,
            extent_row,
            extent_col,
            extent_byte,
            0 as ::core::ffi::c_int,
            0 as colnr_T,
            0 as bcount_t,
        );
        let mut row1: ::core::ffi::c_int = if start_row < new_row {
            start_row
        } else {
            new_row
        };
        let mut row2: ::core::ffi::c_int = (if start_row > new_row {
            start_row
        } else {
            new_row
        }) + extent_row;
        buf_signcols_count_range(buf, row1, row2, 0 as ::core::ffi::c_int, kTrue);
        marktree_move_region(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            start_row,
            start_col,
            extent_row,
            extent_col,
            new_row,
            new_col,
        );
        buf_signcols_count_range(buf, row1, row2, 0 as ::core::ffi::c_int, kNone);
        buf_updates_send_splice(
            buf,
            new_row,
            new_col,
            new_byte,
            0 as ::core::ffi::c_int,
            0 as colnr_T,
            0 as bcount_t,
            extent_row,
            extent_col,
            extent_byte,
        );
        if undo as ::core::ffi::c_uint == kExtmarkUndo as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut uhp: *mut u_header_T = u_force_get_undo_header(buf);
            if uhp.is_null() {
                return;
            }
            let mut move_0: ExtmarkMove = ExtmarkMove {
                start_row: 0,
                start_col: 0,
                extent_row: 0,
                extent_col: 0,
                new_row: 0,
                new_col: 0,
                start_byte: 0,
                extent_byte: 0,
                new_byte: 0,
            };
            move_0.start_row = start_row;
            move_0.start_col = start_col as ::core::ffi::c_int;
            move_0.start_byte = start_byte;
            move_0.extent_row = extent_row;
            move_0.extent_col = extent_col as ::core::ffi::c_int;
            move_0.extent_byte = extent_byte;
            move_0.new_row = new_row;
            move_0.new_col = new_col as ::core::ffi::c_int;
            move_0.new_byte = new_byte;
            if (*uhp).uh_extmark.size == (*uhp).uh_extmark.capacity {
                (*uhp).uh_extmark.capacity = if (*uhp).uh_extmark.capacity != 0 {
                    (*uhp).uh_extmark.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*uhp).uh_extmark.items = xrealloc(
                    (*uhp).uh_extmark.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<ExtmarkUndoObject>()
                        .wrapping_mul((*uhp).uh_extmark.capacity),
                ) as *mut ExtmarkUndoObject;
            } else {
            };
            let c2rust_fresh2 = (*uhp).uh_extmark.size;
            (*uhp).uh_extmark.size = (*uhp).uh_extmark.size.wrapping_add(1);
            *(*uhp).uh_extmark.items.add(c2rust_fresh2) = undo_object {
                type_0: kExtmarkMove,
                data: C2Rust_Unnamed_6 { move_0: move_0 },
            };
        }
    }
}
