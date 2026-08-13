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

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::decoration::{buf_decor_remove, decor_free, decor_state_invalidate};
use crate::src::nvim::marktree::key::{mt_decor, mt_decor_any, mt_end, mt_invalid};

use crate::src::nvim::map::{map_del_uint32_t_uint32_t, map_ref_uint32_t_uint32_t};
use crate::src::nvim::marktree::{
    marktree_del_itr, marktree_itr_current, marktree_itr_get, marktree_itr_next, marktree_lookup,
    marktree_lookup_ns,
};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::types::{
    MTKey, MTNode, MTPos, Map_uint32_t_uint32_t, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_14, buf_T, colnr_T, int32_t, uint32_t, uint64_t,
};

pub unsafe extern "C" fn extmark_del_id(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut id: uint32_t,
) -> bool {
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
        let mut key: MTKey = marktree_lookup_ns(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            ns_id,
            id,
            false_0 != 0,
            &raw mut itr as *mut MarkTreeIter,
        );
        if key.id != 0 {
            extmark_del(buf, &raw mut itr as *mut MarkTreeIter, key, false_0 != 0);
        }
        return key.id > 0 as uint32_t;
    }
}

pub unsafe extern "C" fn extmark_del(
    mut buf: *mut buf_T,
    mut itr: *mut MarkTreeIter,
    mut key: MTKey,
    mut restore: bool,
) {
    unsafe {
        debug_assert!(key.pos.row >= 0 as int32_t, "key.pos.row >= 0");
        let mut key2: MTKey = key;
        let mut other: uint64_t = marktree_del_itr(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            itr,
            false_0 != 0,
        );
        if other != 0 {
            key2 = marktree_lookup(&raw mut (*buf).b_marktree as *mut MarkTree, other, itr);
            debug_assert!(key2.pos.row >= 0 as int32_t, "key2.pos.row >= 0");
            marktree_del_itr(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                itr,
                false_0 != 0,
            );
            if restore {
                marktree_itr_get(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    key.pos.row,
                    key.pos.col as ::core::ffi::c_int,
                    itr,
                );
            }
        }
        if mt_decor_any(key) {
            if mt_invalid(key) {
                decor_free(mt_decor(key));
            } else {
                if mt_end(key) {
                    let mut k: MTKey = key;
                    key = key2;
                    key2 = k;
                }
                buf_decor_remove(
                    buf,
                    key.pos.row as ::core::ffi::c_int,
                    key2.pos.row as ::core::ffi::c_int,
                    key.pos.col as ::core::ffi::c_int,
                    mt_decor(key),
                    true_0 != 0,
                );
            }
        }
        decor_state_invalidate(buf);
    }
}

pub unsafe extern "C" fn extmark_clear(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut l_row: ::core::ffi::c_int,
    mut l_col: colnr_T,
    mut u_row: ::core::ffi::c_int,
    mut u_col: colnr_T,
) -> bool {
    unsafe {
        if (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t))
            .set
            .h
            .size
            == 0
        {
            return false_0 != 0;
        }
        let mut all_ns: bool = ns_id == 0 as uint32_t;
        let mut ns: *mut uint32_t = ::core::ptr::null_mut::<uint32_t>();
        if !all_ns {
            ns = map_ref_uint32_t_uint32_t(
                &raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t,
                ns_id,
                ::core::ptr::null_mut::<*mut uint32_t>(),
            );
            if ns.is_null() {
                return false_0 != 0;
            }
        }
        let mut marks_cleared_any: bool = false_0 != 0;
        let mut marks_cleared_all: bool =
            l_row == 0 as ::core::ffi::c_int && l_col == 0 as ::core::ffi::c_int;
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
            l_row as int32_t,
            l_col as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
        loop {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            if mark.pos.row < 0 as int32_t
                || mark.pos.row > u_row as int32_t
                || mark.pos.row == u_row as int32_t && mark.pos.col > u_col as int32_t
            {
                if mark.pos.row >= 0 as int32_t {
                    marks_cleared_all = false_0 != 0;
                }
                break;
            } else if mark.ns == ns_id || all_ns as ::core::ffi::c_int != 0 {
                marks_cleared_any = true_0 != 0;
                extmark_del(buf, &raw mut itr as *mut MarkTreeIter, mark, true_0 != 0);
            } else {
                marktree_itr_next(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    &raw mut itr as *mut MarkTreeIter,
                );
            }
        }
        if marks_cleared_all {
            if all_ns {
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
            } else {
                map_del_uint32_t_uint32_t(
                    &raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t,
                    ns_id,
                    ::core::ptr::null_mut::<uint32_t>(),
                );
            }
        }
        if marks_cleared_any {
            decor_state_invalidate(buf);
        }
        return marks_cleared_any;
    }
}
