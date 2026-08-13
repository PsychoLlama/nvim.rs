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

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::decoration::{
    buf_decor_remove, buf_put_decor, buf_signcols_count_range, decor_redraw, decor_state_invalidate,
};
use crate::src::nvim::marktree::key::{
    MT_FLAG_DECOR_SIGNTEXT, MT_FLAG_EXTERNAL_MASK, MT_FLAG_INVALID, mt_decor, mt_decor_any, mt_end,
    mt_flags, mt_invalid, mt_paired,
};

use crate::src::nvim::main::curbuf;
use crate::src::nvim::map::map_put_ref_uint32_t_uint32_t;
use crate::src::nvim::marktree::{
    marktree_del_itr, marktree_get_alt, marktree_lookup, marktree_lookup_ns, marktree_move,
    marktree_put, marktree_revise_meta,
};
use crate::src::nvim::types::{
    DecorHighlightInline, DecorInline, DecorInlineData, Error, MTKey, MTNode, MTPos,
    Map_uint32_t_uint32_t, MarkTree, MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_14, buf_T,
    colnr_T, int32_t, kNone, kTrue, linenr_T, uint16_t, uint32_t, uint64_t,
};

pub unsafe extern "C" fn extmark_set(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut idp: *mut uint32_t,
    mut row: ::core::ffi::c_int,
    mut col: colnr_T,
    mut end_row: ::core::ffi::c_int,
    mut end_col: colnr_T,
    mut decor: DecorInline,
    mut decor_flags: uint16_t,
    mut right_gravity: bool,
    mut end_right_gravity: bool,
    mut no_undo: bool,
    mut invalidate: bool,
    mut _err: *mut Error,
) {
    unsafe {
        let mut mark: MTKey = MTKey {
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
        };
        let mut ns: *mut uint32_t = map_put_ref_uint32_t_uint32_t(
            &raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t,
            ns_id,
            ::core::ptr::null_mut::<*mut uint32_t>(),
            ::core::ptr::null_mut::<bool>(),
        );
        let mut id: uint32_t = if !idp.is_null() { *idp } else { 0 as uint32_t };
        let mut flags: uint16_t = (mt_flags(right_gravity, no_undo, invalidate, decor.ext)
            as ::core::ffi::c_int
            | decor_flags as ::core::ffi::c_int) as uint16_t;
        '_revised: {
            if id == 0 as uint32_t {
                *ns = (*ns).wrapping_add(1);
                id = *ns;
            } else {
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
                let mut old_mark: MTKey = marktree_lookup_ns(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    ns_id,
                    id,
                    false_0 != 0,
                    &raw mut itr as *mut MarkTreeIter,
                );
                if old_mark.id != 0 {
                    if mt_paired(old_mark) as ::core::ffi::c_int != 0
                        || end_row > -1 as ::core::ffi::c_int
                    {
                        extmark_del_id(buf, ns_id, id);
                    } else {
                        debug_assert!(
                            !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null(),
                            "marktree_itr_valid(itr)"
                        );
                        if old_mark.pos.row == row as int32_t && old_mark.pos.col == col as int32_t
                        {
                            if !mt_invalid(old_mark)
                                && mt_decor_any(old_mark) as ::core::ffi::c_int != 0
                            {
                                (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                                    [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                                    .flags = ((*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                                    [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                                    .flags
                                    as ::core::ffi::c_int
                                    & !MT_FLAG_EXTERNAL_MASK as uint16_t as ::core::ffi::c_int)
                                    as uint16_t;
                                buf_decor_remove(
                                    buf,
                                    row,
                                    row,
                                    col as ::core::ffi::c_int,
                                    mt_decor(old_mark),
                                    true_0 != 0,
                                );
                            }
                            (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                                [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                                .flags = ((*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                                [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                                .flags
                                as ::core::ffi::c_int
                                | flags as ::core::ffi::c_int)
                                as uint16_t;
                            (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                                [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                                .decor_data = decor.data;
                            marktree_revise_meta(
                                &raw mut (*buf).b_marktree as *mut MarkTree,
                                &raw mut itr as *mut MarkTreeIter,
                                old_mark,
                            );
                            break '_revised;
                        } else {
                            marktree_del_itr(
                                &raw mut (*buf).b_marktree as *mut MarkTree,
                                &raw mut itr as *mut MarkTreeIter,
                                false_0 != 0,
                            );
                            if !mt_invalid(old_mark) {
                                buf_decor_remove(
                                    buf,
                                    old_mark.pos.row as ::core::ffi::c_int,
                                    old_mark.pos.row as ::core::ffi::c_int,
                                    old_mark.pos.col as ::core::ffi::c_int,
                                    mt_decor(old_mark),
                                    true_0 != 0,
                                );
                            }
                        }
                    }
                } else {
                    *ns = if *ns > id { *ns } else { id };
                }
            }
            mark = MTKey {
                pos: MTPos {
                    row: row as int32_t,
                    col: col as int32_t,
                },
                ns: ns_id,
                id: id,
                flags: flags,
                decor_data: decor.data,
            };
            marktree_put(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                mark,
                end_row,
                end_col as ::core::ffi::c_int,
                end_right_gravity,
            );
            decor_state_invalidate(buf);
        }
        if decor_flags as ::core::ffi::c_int != 0 || decor.ext as ::core::ffi::c_int != 0 {
            buf_put_decor(
                buf,
                decor,
                row,
                if end_row > -1 as ::core::ffi::c_int {
                    end_row
                } else {
                    row
                },
            );
            decor_redraw(
                buf,
                row,
                if end_row > -1 as ::core::ffi::c_int {
                    end_row
                } else {
                    row
                },
                col as ::core::ffi::c_int,
                decor,
            );
        }
        if !idp.is_null() {
            *idp = id;
        }
    }
}

pub(crate) unsafe extern "C" fn extmark_setraw(
    mut buf: *mut buf_T,
    mut mark: uint64_t,
    mut row: ::core::ffi::c_int,
    mut col: colnr_T,
    mut invalid: bool,
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
        let mut key: MTKey = marktree_lookup(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            mark,
            &raw mut itr as *mut MarkTreeIter,
        );
        let mut move_0: bool = key.pos.row != row as int32_t || key.pos.col != col as int32_t;
        if key.pos.row < 0 as int32_t || !move_0 && !invalid {
            return;
        }
        if !invalid && mt_decor_any(key) as ::core::ffi::c_int != 0 && key.pos.row != row as int32_t
        {
            decor_redraw(
                buf,
                key.pos.row as ::core::ffi::c_int,
                key.pos.row as ::core::ffi::c_int,
                key.pos.col as ::core::ffi::c_int,
                mt_decor(key),
            );
        }
        let mut row1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut row2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut altitr: [MarkTreeIter; 1] = [*(&raw mut itr as *mut MarkTreeIter)];
        let mut alt: MTKey = marktree_get_alt(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            key,
            &raw mut altitr as *mut MarkTreeIter,
        );
        if invalid {
            (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                .flags = ((*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                .flags as ::core::ffi::c_int
                & !MT_FLAG_INVALID as uint16_t as ::core::ffi::c_int)
                as uint16_t;
            (*(*(&raw mut altitr as *mut MarkTreeIter)).x).key
                [(*(&raw mut altitr as *mut MarkTreeIter)).i as usize]
                .flags = ((*(*(&raw mut altitr as *mut MarkTreeIter)).x).key
                [(*(&raw mut altitr as *mut MarkTreeIter)).i as usize]
                .flags as ::core::ffi::c_int
                & !MT_FLAG_INVALID as uint16_t as ::core::ffi::c_int)
                as uint16_t;
            marktree_revise_meta(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                if mt_end(key) as ::core::ffi::c_int != 0 {
                    &raw mut altitr as *mut MarkTreeIter
                } else {
                    &raw mut itr as *mut MarkTreeIter
                },
                if mt_end(key) as ::core::ffi::c_int != 0 {
                    alt
                } else {
                    key
                },
            );
        } else if !mt_invalid(key)
            && key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_SIGNTEXT != 0
            && (*buf).b_signcols.autom as ::core::ffi::c_int != 0
        {
            row1 = (if alt.pos.row
                < (if key.pos.row < row as int32_t {
                    key.pos.row
                } else {
                    row as int32_t
                }) {
                alt.pos.row
            } else if key.pos.row < row as int32_t {
                key.pos.row
            } else {
                row as int32_t
            }) as ::core::ffi::c_int;
            row2 = (if alt.pos.row
                > (if key.pos.row > row as int32_t {
                    key.pos.row
                } else {
                    row as int32_t
                }) {
                alt.pos.row
            } else if key.pos.row > row as int32_t {
                key.pos.row
            } else {
                row as int32_t
            }) as ::core::ffi::c_int;
            buf_signcols_count_range(
                buf,
                row1,
                if ((*curbuf.get()).b_ml.ml_line_count - 1 as linenr_T) < row2 as linenr_T {
                    (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int
                } else {
                    row2
                },
                0 as ::core::ffi::c_int,
                kTrue,
            );
        }
        if move_0 {
            marktree_move(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
                row,
                col as ::core::ffi::c_int,
            );
        }
        if invalid {
            buf_put_decor(
                buf,
                mt_decor(key),
                if (row as int32_t) < alt.pos.row {
                    row
                } else {
                    alt.pos.row as ::core::ffi::c_int
                },
                if row as int32_t > alt.pos.row {
                    row
                } else {
                    alt.pos.row as ::core::ffi::c_int
                },
            );
        } else if !mt_invalid(key)
            && key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_SIGNTEXT != 0
            && (*buf).b_signcols.autom as ::core::ffi::c_int != 0
        {
            buf_signcols_count_range(
                buf,
                row1,
                if ((*curbuf.get()).b_ml.ml_line_count - 1 as linenr_T) < row2 as linenr_T {
                    (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int
                } else {
                    row2
                },
                0 as ::core::ffi::c_int,
                kNone,
            );
        }
    }
}
