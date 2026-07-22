use crate::src::nvim::buffer_updates::buf_updates_send_splice;
use crate::src::nvim::decoration::{
    buf_decor_remove, buf_put_decor, buf_signcols_count_range, decor_free, decor_redraw,
    decor_state_invalidate, decor_type_flags,
};

use crate::src::nvim::main::{curbuf, curbuf_splice_pending};
use crate::src::nvim::map::{
    map_del_uint32_t_uint32_t, map_put_ref_uint32_t_uint32_t, map_ref_uint32_t_uint32_t,
};
use crate::src::nvim::marktree::{
    marktree_clear, marktree_del_itr, marktree_get_alt, marktree_get_altpos, marktree_itr_current,
    marktree_itr_get, marktree_itr_get_ext, marktree_itr_get_overlap, marktree_itr_next,
    marktree_itr_step_overlap, marktree_lookup, marktree_lookup_ns, marktree_move,
    marktree_move_region, marktree_put, marktree_revise_meta, marktree_splice,
};
use crate::src::nvim::memline::ml_find_line_or_offset;
use crate::src::nvim::memory::{xfree, xrealloc};
use crate::src::nvim::os::libc::{__assert_fail, memset};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, Error, ErrorType, ExtmarkInfoArray, ExtmarkMove,
    ExtmarkOp, ExtmarkSavePos, ExtmarkSplice, ExtmarkType, ExtmarkUndoObject, FileID, FloatAnchor,
    FloatRelative, GridView, Intersection, LuaRef, MTKey, MTNode, MTPair, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_14, MetaFilter, OptInt, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal, Timestamp,
    TriState, UndoObjectType, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk,
    VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, bcount_t,
    bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T, dict_T,
    dictvar_S, disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T,
    synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_6, varnumber_T, virt_line, visualinfo_T, win_T,
    window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
use crate::src::nvim::undo::u_force_get_undo_header;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kStlClickFuncRun: C2Rust_Unnamed_12 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_12 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_12 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_12 = 0;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_13 = 2147483647;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kExtmarkHighlight: ExtmarkType = 32;
pub const kExtmarkVirtLines: ExtmarkType = 16;
pub const kExtmarkVirtText: ExtmarkType = 8;
pub const kExtmarkSignHL: ExtmarkType = 4;
pub const kExtmarkSign: ExtmarkType = 2;
pub const kExtmarkNone: ExtmarkType = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const KV_INITIAL_VALUE: ExtmarkInfoArray = ExtmarkInfoArray {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<MTPair>(),
};
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_uint32_t = Set_uint32_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MAP_INIT: Map_uint32_t_uint32_t = Map_uint32_t_uint32_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<uint32_t>(),
};
#[no_mangle]
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
                    '_c2rust_label: {
                        if !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
                        } else {
                            __assert_fail(
                                b"marktree_itr_valid(itr)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/extmark.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                70 as ::core::ffi::c_uint,
                                b"void extmark_set(buf_T *, uint32_t, uint32_t *, int, colnr_T, int, colnr_T, DecorInline, uint16_t, _Bool, _Bool, _Bool, _Bool, Error *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if old_mark.pos.row == row as int32_t && old_mark.pos.col == col as int32_t {
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
                            .flags as ::core::ffi::c_int
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
unsafe extern "C" fn extmark_setraw(
    mut buf: *mut buf_T,
    mut mark: uint64_t,
    mut row: ::core::ffi::c_int,
    mut col: colnr_T,
    mut invalid: bool,
) {
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
    if !invalid && mt_decor_any(key) as ::core::ffi::c_int != 0 && key.pos.row != row as int32_t {
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
            & !MT_FLAG_INVALID as uint16_t as ::core::ffi::c_int) as uint16_t;
        (*(*(&raw mut altitr as *mut MarkTreeIter)).x).key
            [(*(&raw mut altitr as *mut MarkTreeIter)).i as usize]
            .flags = ((*(*(&raw mut altitr as *mut MarkTreeIter)).x).key
            [(*(&raw mut altitr as *mut MarkTreeIter)).i as usize]
            .flags as ::core::ffi::c_int
            & !MT_FLAG_INVALID as uint16_t as ::core::ffi::c_int) as uint16_t;
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
                (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int
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
                (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            } else {
                row2
            },
            0 as ::core::ffi::c_int,
            kNone,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn extmark_del_id(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut id: uint32_t,
) -> bool {
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
#[no_mangle]
pub unsafe extern "C" fn extmark_del(
    mut buf: *mut buf_T,
    mut itr: *mut MarkTreeIter,
    mut key: MTKey,
    mut restore: bool,
) {
    '_c2rust_label: {
        if key.pos.row >= 0 as int32_t {
        } else {
            __assert_fail(
                b"key.pos.row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/extmark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                167 as ::core::ffi::c_uint,
                b"void extmark_del(buf_T *, MarkTreeIter *, MTKey, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut key2: MTKey = key;
    let mut other: uint64_t = marktree_del_itr(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        itr,
        false_0 != 0,
    );
    if other != 0 {
        key2 = marktree_lookup(&raw mut (*buf).b_marktree as *mut MarkTree, other, itr);
        '_c2rust_label_0: {
            if key2.pos.row >= 0 as int32_t {
            } else {
                __assert_fail(
                    b"key2.pos.row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/extmark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    173 as ::core::ffi::c_uint,
                    b"void extmark_del(buf_T *, MarkTreeIter *, MTKey, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
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
#[no_mangle]
pub unsafe extern "C" fn extmark_clear(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut l_row: ::core::ffi::c_int,
    mut l_col: colnr_T,
    mut u_row: ::core::ffi::c_int,
    mut u_col: colnr_T,
) -> bool {
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
#[no_mangle]
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
unsafe extern "C" fn push_mark(
    mut array: *mut ExtmarkInfoArray,
    mut ns_id: uint32_t,
    mut type_filter: ExtmarkType,
    mut mark: MTPair,
) {
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
    *(*array).items.offset(c2rust_fresh0 as isize) = mark;
}
#[no_mangle]
pub unsafe extern "C" fn extmark_from_id(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut id: uint32_t,
) -> MTPair {
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
    '_c2rust_label: {
        if mark.pos.row >= 0 as int32_t {
        } else {
            __assert_fail(
                b"mark.pos.row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/extmark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                328 as ::core::ffi::c_uint,
                b"MTPair extmark_from_id(buf_T *, uint32_t, uint32_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut end: MTKey = marktree_get_alt(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        mark,
        ::core::ptr::null_mut::<MarkTreeIter>(),
    );
    return mtpair_from(mark, end);
}
#[no_mangle]
pub unsafe extern "C" fn extmark_free_all(mut buf: *mut buf_T) {
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
        if !(mt_paired(mark) as ::core::ffi::c_int != 0 && mt_end(mark) as ::core::ffi::c_int != 0)
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
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*(&raw mut (*buf).b_extmark_ns
        as *mut Map_uint32_t_uint32_t))
        .values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    *(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t) = MAP_INIT;
}
#[no_mangle]
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
            *(*uvp).items.offset(c2rust_fresh1 as isize) = undo;
        }
        marktree_itr_next(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn extmark_apply_undo(mut undo_info: ExtmarkUndoObject, mut undo: bool) {
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
#[no_mangle]
pub unsafe extern "C" fn extmark_adjust(
    mut buf: *mut buf_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
    mut undo: ExtmarkOp,
) {
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
        '_c2rust_label: {
            if line2 == MAXLNUM as ::core::ffi::c_int as linenr_T {
            } else {
                __assert_fail(
                    b"line2 == MAXLNUM\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/extmark.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    500 as ::core::ffi::c_uint,
                    b"void extmark_adjust(buf_T *, linenr_T, linenr_T, linenr_T, linenr_T, ExtmarkOp)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
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
#[no_mangle]
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
#[no_mangle]
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
        let mut count: ::core::ffi::c_int = if (*buf).b_prev_line_count > 0 as ::core::ffi::c_int {
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
    if undo as ::core::ffi::c_uint == kExtmarkUndo as ::core::ffi::c_int as ::core::ffi::c_uint {
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
                .offset((*uhp_0).uh_extmark.size.wrapping_sub(1 as size_t) as isize);
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
            *(*uhp_0).uh_extmark.items.offset(c2rust_fresh3 as isize) = undo_object {
                type_0: kExtmarkSplice,
                data: C2Rust_Unnamed_6 { splice: splice_0 },
            };
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn extmark_splice_cols(
    mut buf: *mut buf_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut old_col: colnr_T,
    mut new_col: colnr_T,
    mut undo: ExtmarkOp,
) {
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
#[no_mangle]
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
    if undo as ::core::ffi::c_uint == kExtmarkUndo as ::core::ffi::c_int as ::core::ffi::c_uint {
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
        *(*uhp).uh_extmark.items.offset(c2rust_fresh2 as isize) = undo_object {
            type_0: kExtmarkMove,
            data: C2Rust_Unnamed_6 { move_0: move_0 },
        };
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MT_FLAG_END: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const MT_FLAG_PAIRED: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
pub const MT_FLAG_NO_UNDO: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
pub const MT_FLAG_INVALIDATE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 5 as ::core::ffi::c_int;
pub const MT_FLAG_INVALID: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 6 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_EXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_HL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 8 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNTEXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 9 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNHL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 10 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_VIRT_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 11 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_VIRT_TEXT_INLINE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 12 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_CONCEAL_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 13 as ::core::ffi::c_int;
pub const MT_FLAG_RIGHT_GRAVITY: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_MASK: ::core::ffi::c_int = MT_FLAG_DECOR_EXT
    | MT_FLAG_DECOR_HL
    | MT_FLAG_DECOR_SIGNTEXT
    | MT_FLAG_DECOR_SIGNHL
    | MT_FLAG_DECOR_VIRT_LINES
    | MT_FLAG_DECOR_VIRT_TEXT_INLINE;
pub const MT_FLAG_EXTERNAL_MASK: ::core::ffi::c_int = MT_FLAG_DECOR_MASK
    | MT_FLAG_NO_UNDO
    | MT_FLAG_INVALIDATE
    | MT_FLAG_INVALID
    | MT_FLAG_DECOR_CONCEAL_LINES;
pub const MARKTREE_END_FLAG: uint64_t = 1 as ::core::ffi::c_int as uint64_t;
#[inline]
unsafe extern "C" fn mt_lookup_id(mut ns: uint32_t, mut id: uint32_t, mut enda: bool) -> uint64_t {
    return (ns as uint64_t) << 33 as ::core::ffi::c_int
        | (id << 1 as ::core::ffi::c_int) as uint64_t
        | (if enda as ::core::ffi::c_int != 0 {
            MARKTREE_END_FLAG
        } else {
            0 as uint64_t
        });
}
#[inline]
unsafe extern "C" fn mt_lookup_key(mut key: MTKey) -> uint64_t {
    return mt_lookup_id(
        key.ns,
        key.id,
        key.flags as ::core::ffi::c_int & MT_FLAG_END != 0,
    );
}
#[inline]
unsafe extern "C" fn mt_paired(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_PAIRED != 0;
}
#[inline]
unsafe extern "C" fn mt_end(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_END != 0;
}
#[inline]
unsafe extern "C" fn mt_right(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_RIGHT_GRAVITY != 0;
}
#[inline]
unsafe extern "C" fn mt_no_undo(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_NO_UNDO != 0;
}
#[inline]
unsafe extern "C" fn mt_invalidate(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_INVALIDATE != 0;
}
#[inline]
unsafe extern "C" fn mt_invalid(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_INVALID != 0;
}
#[inline]
unsafe extern "C" fn mt_decor_any(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_MASK != 0;
}
#[inline]
unsafe extern "C" fn mt_flags(
    mut right_gravity: bool,
    mut no_undo: bool,
    mut invalidate: bool,
    mut decor_ext: bool,
) -> uint16_t {
    return ((if right_gravity as ::core::ffi::c_int != 0 {
        MT_FLAG_RIGHT_GRAVITY
    } else {
        0 as ::core::ffi::c_int
    }) | (if no_undo as ::core::ffi::c_int != 0 {
        MT_FLAG_NO_UNDO
    } else {
        0 as ::core::ffi::c_int
    }) | (if invalidate as ::core::ffi::c_int != 0 {
        MT_FLAG_INVALIDATE
    } else {
        0 as ::core::ffi::c_int
    }) | (if decor_ext as ::core::ffi::c_int != 0 {
        MT_FLAG_DECOR_EXT
    } else {
        0 as ::core::ffi::c_int
    })) as uint16_t;
}
#[inline]
unsafe extern "C" fn mtpair_from(mut start: MTKey, mut end: MTKey) -> MTPair {
    return MTPair {
        start: start,
        end_pos: end.pos,
        end_right_gravity: mt_right(end),
    };
}
#[inline]
unsafe extern "C" fn mt_decor(mut key: MTKey) -> DecorInline {
    return DecorInline {
        ext: key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0,
        data: key.decor_data,
    };
}
