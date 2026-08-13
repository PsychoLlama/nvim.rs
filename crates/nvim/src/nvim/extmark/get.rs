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

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::decoration::{decor_free, decor_type_flags};
use crate::src::nvim::marktree::key::{mt_decor, mt_decor_any, mt_end, mt_paired, mtpair_from};

use crate::src::nvim::marktree::{
    marktree_clear, marktree_get_alt, marktree_itr_current, marktree_itr_get, marktree_itr_get_ext,
    marktree_itr_get_overlap, marktree_itr_next, marktree_itr_step_overlap, marktree_lookup_ns,
};
use crate::src::nvim::memory::{xfree, xrealloc};
use crate::src::nvim::os::libc::memset;
use crate::src::nvim::types::{
    DecorHighlightInline, DecorInlineData, ExtmarkInfoArray, ExtmarkType, MTKey, MTNode, MTPair,
    MTPos, Map_uint32_t_uint32_t, MarkTree, MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_14,
    buf_T, colnr_T, int32_t, int64_t, size_t, uint16_t, uint32_t,
};

pub unsafe extern "C" fn extmark_get(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut l_row: ::core::ffi::c_int,
    mut l_col: colnr_T,
    mut u_row: ::core::ffi::c_int,
    mut u_col: colnr_T,
    mut amount: int64_t,
    mut type_filter: ExtmarkType,
    mut overlap: bool,
) -> ExtmarkInfoArray {
    unsafe {
        let mut array: ExtmarkInfoArray = KV_INITIAL_VALUE;
        let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1];
        if overlap {
            if !marktree_itr_get_overlap(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                l_row,
                l_col as ::core::ffi::c_int,
                &raw mut itr as *mut MarkTreeIter,
            ) {
                return array;
            }
            while (array.size as int64_t) < amount {
                let mut pair: MTPair = MTPair {
                    start: MTKey {
                        pos: MTPos { row: 0, col: 0 },
                        ns: 0,
                        id: 0,
                        flags: 0,
                        decor_data: DecorInlineData {
                            hl: DecorHighlightInline {
                                flags: 0,
                                priority: 0,
                                hl_id: 0,
                                conceal_char: 0,
                            },
                        },
                    },
                    end_pos: MTPos { row: 0, col: 0 },
                    end_right_gravity: false,
                };
                if !marktree_itr_step_overlap(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    &raw mut itr as *mut MarkTreeIter,
                    &raw mut pair,
                ) {
                    break;
                }
                push_mark(&raw mut array, ns_id, type_filter, pair);
            }
        } else {
            marktree_itr_get_ext(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                MTPos {
                    row: l_row as int32_t,
                    col: l_col as int32_t,
                },
                &raw mut itr as *mut MarkTreeIter,
                false_0 != 0,
                false_0 != 0,
                ::core::ptr::null_mut::<MTPos>(),
                ::core::ptr::null::<uint32_t>(),
            );
        }
        while (array.size as int64_t) < amount {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            if mark.pos.row < 0 as int32_t
                || mark.pos.row > u_row as int32_t
                || mark.pos.row == u_row as int32_t && mark.pos.col > u_col as int32_t
            {
                break;
            }
            if !mt_end(mark) {
                let mut end: MTKey = marktree_get_alt(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    mark,
                    ::core::ptr::null_mut::<MarkTreeIter>(),
                );
                push_mark(&raw mut array, ns_id, type_filter, mtpair_from(mark, end));
            }
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
            );
        }
        return array;
    }
}

unsafe extern "C" fn push_mark(
    mut array: *mut ExtmarkInfoArray,
    mut ns_id: uint32_t,
    mut type_filter: ExtmarkType,
    mut mark: MTPair,
) {
    unsafe {
        if !(ns_id == UINT32_MAX as uint32_t || mark.start.ns == ns_id) {
            return;
        }
        if type_filter as ::core::ffi::c_uint
            != kExtmarkNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if !mt_decor_any(mark.start) {
                return;
            }
            let mut type_flags: uint16_t = decor_type_flags(mt_decor(mark.start));
            if type_flags as ::core::ffi::c_uint & type_filter as ::core::ffi::c_uint == 0 {
                return;
            }
        }
        if (*array).size == (*array).capacity {
            (*array).capacity = if (*array).capacity != 0 {
                (*array).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*array).items = xrealloc(
                (*array).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<MTPair>().wrapping_mul((*array).capacity),
            ) as *mut MTPair;
        } else {
        };
        let c2rust_fresh0 = (*array).size;
        (*array).size = (*array).size.wrapping_add(1);
        *(*array).items.add(c2rust_fresh0) = mark;
    }
}

pub unsafe extern "C" fn extmark_from_id(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut id: uint32_t,
) -> MTPair {
    unsafe {
        let mut mark: MTKey = marktree_lookup_ns(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            ns_id,
            id,
            false_0 != 0,
            ::core::ptr::null_mut::<MarkTreeIter>(),
        );
        if mark.id == 0 {
            return mtpair_from(mark, mark);
        }
        debug_assert!(mark.pos.row >= 0 as int32_t, "mark.pos.row >= 0");
        let mut end: MTKey = marktree_get_alt(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            mark,
            ::core::ptr::null_mut::<MarkTreeIter>(),
        );
        return mtpair_from(mark, end);
    }
}

pub unsafe extern "C" fn extmark_free_all(mut buf: *mut buf_T) {
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
        marktree_itr_get(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            0 as int32_t,
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
        loop {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            if mark.pos.row < 0 as int32_t {
                break;
            }
            if !(mt_paired(mark) as ::core::ffi::c_int != 0
                && mt_end(mark) as ::core::ffi::c_int != 0)
            {
                decor_free(mt_decor(mark));
            }
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
            );
        }
        marktree_clear(&raw mut (*buf).b_marktree as *mut MarkTree);
        (*buf).b_signcols.max = 0 as ::core::ffi::c_int;
        memset(
            &raw mut (*buf).b_signcols.count as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[::core::ffi::c_int; 9]>(),
        );
        xfree(
            (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t))
                .set
                .keys as *mut ::core::ffi::c_void,
        );
        xfree(
            (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t))
                .set
                .h
                .hash as *mut ::core::ffi::c_void,
        );
        (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t)).set = SET_INIT;
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t)).values
                as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        *(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t) = MAP_INIT;
    }
}
