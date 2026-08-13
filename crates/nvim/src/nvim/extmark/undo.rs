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

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::decoration::buf_decor_remove;
use crate::src::nvim::marktree::key::{
    MT_FLAG_INVALID, mt_decor, mt_end, mt_invalid, mt_invalidate, mt_lookup_key, mt_no_undo,
    mt_paired, mt_right,
};

use crate::src::nvim::main::curbuf;
use crate::src::nvim::marktree::{
    marktree_get_altpos, marktree_itr_current, marktree_itr_get, marktree_itr_next,
    marktree_revise_meta,
};
use crate::src::nvim::memory::xrealloc;
use crate::src::nvim::types::{
    ExtmarkMove, ExtmarkOp, ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject, MTKey, MTNode, MTPos,
    MarkTree, MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_14, buf_T, colnr_T,
    extmark_undo_vec_t, int32_t, size_t, uint16_t, undo_object_data as C2Rust_Unnamed_6,
};

pub unsafe extern "C" fn extmark_splice_delete(
    mut buf: *mut buf_T,
    mut l_row: ::core::ffi::c_int,
    mut l_col: colnr_T,
    mut u_row: ::core::ffi::c_int,
    mut u_col: colnr_T,
    mut uvp: *mut extmark_undo_vec_t,
    mut only_copy: bool,
    mut op: ExtmarkOp,
) {
    unsafe {
        let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos {
                row: 0 as int32_t,
                col: 0,
            },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }];
        let mut undo: ExtmarkUndoObject = ExtmarkUndoObject {
            type_0: kExtmarkSplice,
            data: C2Rust_Unnamed_6 {
                splice: ExtmarkSplice {
                    start_row: 0,
                    start_col: 0,
                    old_row: 0,
                    old_col: 0,
                    new_row: 0,
                    new_col: 0,
                    start_byte: 0,
                    old_byte: 0,
                    new_byte: 0,
                },
            },
        };
        marktree_itr_get(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            l_row as int32_t,
            l_col as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
        loop {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            if mark.pos.row < 0 as int32_t || mark.pos.row > u_row as int32_t {
                break;
            }
            let mut copy: bool = true_0 != 0;
            if mark.pos.row == l_row as int32_t
                && (mark.pos.col - !mt_right(mark) as ::core::ffi::c_int) < l_col as int32_t
            {
                copy = false_0 != 0;
            } else if mark.pos.row == u_row as int32_t {
                if mark.pos.col > u_col as int32_t + 1 as int32_t {
                    break;
                }
                if mark.pos.col + mt_right(mark) as int32_t > u_col as int32_t {
                    copy = false_0 != 0;
                }
            }
            let mut invalidated: bool = false_0 != 0;
            if !only_copy
                && !mt_invalid(mark)
                && mt_invalidate(mark) as ::core::ffi::c_int != 0
                && !mt_end(mark)
            {
                let mut enditr: [MarkTreeIter; 1] = [*(&raw mut itr as *mut MarkTreeIter)];
                let mut endpos: MTPos = marktree_get_altpos(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    mark,
                    &raw mut enditr as *mut MarkTreeIter,
                );
                if !mt_paired(mark) && mark.pos.row < u_row as int32_t
                    || mt_paired(mark) as ::core::ffi::c_int != 0
                        && (mark.pos.row > l_row as int32_t
                            || mark.pos.row == l_row as int32_t && mark.pos.col >= l_col as int32_t)
                        && (endpos.row < u_row as int32_t
                            || endpos.row == u_row as int32_t && endpos.col <= u_col as int32_t)
                {
                    if mt_no_undo(mark) {
                        extmark_del(buf, &raw mut itr as *mut MarkTreeIter, mark, true_0 != 0);
                        continue;
                    } else {
                        copy = true_0 != 0;
                        invalidated = true_0 != 0;
                        (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                            [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                            .flags = ((*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                            [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                            .flags as ::core::ffi::c_int
                            | MT_FLAG_INVALID) as uint16_t;
                        (*(*(&raw mut enditr as *mut MarkTreeIter)).x).key
                            [(*(&raw mut enditr as *mut MarkTreeIter)).i as usize]
                            .flags = ((*(*(&raw mut enditr as *mut MarkTreeIter)).x).key
                            [(*(&raw mut enditr as *mut MarkTreeIter)).i as usize]
                            .flags as ::core::ffi::c_int
                            | MT_FLAG_INVALID) as uint16_t;
                        marktree_revise_meta(
                            &raw mut (*buf).b_marktree as *mut MarkTree,
                            &raw mut itr as *mut MarkTreeIter,
                            mark,
                        );
                        buf_decor_remove(
                            buf,
                            mark.pos.row as ::core::ffi::c_int,
                            endpos.row as ::core::ffi::c_int,
                            mark.pos.col as ::core::ffi::c_int,
                            mt_decor(mark),
                            false_0 != 0,
                        );
                    }
                }
            }
            if copy as ::core::ffi::c_int != 0
                && (only_copy as ::core::ffi::c_int != 0
                    || !uvp.is_null()
                        && op as ::core::ffi::c_uint
                            == kExtmarkUndo as ::core::ffi::c_int as ::core::ffi::c_uint
                        && !mt_no_undo(mark))
            {
                let mut pos: ExtmarkSavePos = ExtmarkSavePos {
                    mark: mt_lookup_key(mark),
                    old_row: mark.pos.row as ::core::ffi::c_int,
                    old_col: mark.pos.col as colnr_T,
                    invalidated: invalidated,
                };
                undo.data.savepos = pos;
                undo.type_0 = kExtmarkSavePos;
                if (*uvp).size == (*uvp).capacity {
                    (*uvp).capacity = if (*uvp).capacity != 0 {
                        (*uvp).capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    (*uvp).items = xrealloc(
                        (*uvp).items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<ExtmarkUndoObject>().wrapping_mul((*uvp).capacity),
                    ) as *mut ExtmarkUndoObject;
                } else {
                };
                let c2rust_fresh1 = (*uvp).size;
                (*uvp).size = (*uvp).size.wrapping_add(1);
                *(*uvp).items.add(c2rust_fresh1) = undo;
            }
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
            );
        }
    }
}

pub unsafe extern "C" fn extmark_apply_undo(mut undo_info: ExtmarkUndoObject, mut undo: bool) {
    unsafe {
        if undo_info.type_0 as ::core::ffi::c_uint
            == kExtmarkSplice as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut splice: ExtmarkSplice = undo_info.data.splice;
            if undo {
                extmark_splice_impl(
                    curbuf.get(),
                    splice.start_row,
                    splice.start_col,
                    splice.start_byte,
                    splice.new_row,
                    splice.new_col,
                    splice.new_byte,
                    splice.old_row,
                    splice.old_col,
                    splice.old_byte,
                    kExtmarkNoUndo,
                );
            } else {
                extmark_splice_impl(
                    curbuf.get(),
                    splice.start_row,
                    splice.start_col,
                    splice.start_byte,
                    splice.old_row,
                    splice.old_col,
                    splice.old_byte,
                    splice.new_row,
                    splice.new_col,
                    splice.new_byte,
                    kExtmarkNoUndo,
                );
            }
        } else if undo_info.type_0 as ::core::ffi::c_uint
            == kExtmarkSavePos as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut pos: ExtmarkSavePos = undo_info.data.savepos;
            if undo as ::core::ffi::c_int != 0 && pos.old_row >= 0 as ::core::ffi::c_int {
                extmark_setraw(
                    curbuf.get(),
                    pos.mark,
                    pos.old_row,
                    pos.old_col,
                    pos.invalidated,
                );
            }
        } else if undo_info.type_0 as ::core::ffi::c_uint
            == kExtmarkMove as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut move_0: ExtmarkMove = undo_info.data.move_0;
            if undo {
                extmark_move_region(
                    curbuf.get(),
                    move_0.new_row,
                    move_0.new_col as colnr_T,
                    move_0.new_byte,
                    move_0.extent_row,
                    move_0.extent_col as colnr_T,
                    move_0.extent_byte,
                    move_0.start_row,
                    move_0.start_col as colnr_T,
                    move_0.start_byte,
                    kExtmarkNoUndo,
                );
            } else {
                extmark_move_region(
                    curbuf.get(),
                    move_0.start_row,
                    move_0.start_col as colnr_T,
                    move_0.start_byte,
                    move_0.extent_row,
                    move_0.extent_col as colnr_T,
                    move_0.extent_byte,
                    move_0.new_row,
                    move_0.new_col as colnr_T,
                    move_0.new_byte,
                    kExtmarkNoUndo,
                );
            }
        }
    }
}
