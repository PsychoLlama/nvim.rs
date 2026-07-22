use crate::src::nvim::api::extmark::virt_text_to_array;
use crate::src::nvim::api::private::helpers::{arena_array, arena_string, cstr_as_string};
use crate::src::nvim::change::changed_lines_invalidate_buf;
use crate::src::nvim::decoration_provider::decor_providers_invoke_conceal_line;
use crate::src::nvim::drawscreen::{
    conceal_cursor_line, redraw_buf_line_later, redraw_buf_range_later,
};
use crate::src::nvim::extmark::extmark_set;
use crate::src::nvim::fold::{hasAnyFolding, hasFolding};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{schar_from_char, schar_get, schar_get_first_codepoint, schar_high};
use crate::src::nvim::highlight::{hl_add_url, hl_combine_attr};
use crate::src::nvim::highlight_group::{syn_id2attr, syn_id2name};
use crate::src::nvim::main::{
    curtab, curwin, decor_state, first_tabpage, firstwin, hl_mode_str, namespace_localscope,
    virt_text_pos_str,
};
use crate::src::nvim::map::mh_get_uint32_t;
use crate::src::nvim::marktree::{
    marktree_get_altpos, marktree_itr_current, marktree_itr_get, marktree_itr_get_filter,
    marktree_itr_get_overlap, marktree_itr_next, marktree_itr_next_filter,
    marktree_itr_step_out_filter, marktree_itr_step_overlap,
};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xrealloc};
use crate::src::nvim::os::libc::{__assert_fail, memcpy, memmove, qsort};
use crate::src::nvim::r#move::changed_window_setting;
use crate::src::nvim::sign::{buf_has_signs, describe_sign_text};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, Array, BoolVarValue, Boolean, BufUpdateCallbacks,
    Callback, CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInline, DecorInlineData, DecorPriority, DecorPriorityInternal,
    DecorRange, DecorRangeKind, DecorRangeSlot, DecorRange_data as C2Rust_Unnamed_22,
    DecorRange_data_ui as C2Rust_Unnamed_23, DecorSignHighlight, DecorState,
    DecorState_ranges_i as C2Rust_Unnamed_24, DecorState_slots as C2Rust_Unnamed_25, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, ExtmarkMove, ExtmarkSavePos,
    ExtmarkSplice, ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridView, Integer,
    Intersection, KeyValuePair, LuaRef, MTKey, MTNode, MTPair, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_19, MetaFilter, MetaIndex, Object, ObjectType, OptInt,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SignItem,
    SignTextAttrs, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_13, String_0, Terminal, Timestamp, TriState,
    UndoObjectType, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinInfo, WinSplit, WinStyle, Window, __compar_fn_t, __time_t, alist_T, bcount_t,
    bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T, dict_T,
    dictvar_S, diff_T, diffblock_S, disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_12,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed_14, partial_S, partial_T,
    pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_7, varnumber_T, virt_line, visualinfo_T, win_T,
    window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
extern "C" {
    static decor_items: GlobalCell<C2Rust_Unnamed_26>;
}
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const SIGN_WIDTH: C2Rust_Unnamed = 2;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
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
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kVLScroll: C2Rust_Unnamed_16 = 2;
pub const kVLLeftcol: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kSHConcealLines: C2Rust_Unnamed_17 = 128;
pub const kSHConceal: C2Rust_Unnamed_17 = 64;
pub const kSHSpellOff: C2Rust_Unnamed_17 = 32;
pub const kSHSpellOn: C2Rust_Unnamed_17 = 16;
pub const kSHUIWatchedOverlay: C2Rust_Unnamed_17 = 8;
pub const kSHUIWatched: C2Rust_Unnamed_17 = 4;
pub const kSHHlEol: C2Rust_Unnamed_17 = 2;
pub const kSHIsSign: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kVTRepeatLinebreak: C2Rust_Unnamed_18 = 8;
pub const kVTLinesAbove: C2Rust_Unnamed_18 = 4;
pub const kVTHide: C2Rust_Unnamed_18 = 2;
pub const kVTIsLines: C2Rust_Unnamed_18 = 1;
pub const kMTMetaCount: MetaIndex = 5;
pub const kMTMetaConcealLines: MetaIndex = 4;
pub const kMTMetaSignText: MetaIndex = 3;
pub const kMTMetaSignHL: MetaIndex = 2;
pub const kMTMetaLines: MetaIndex = 1;
pub const kMTMetaInline: MetaIndex = 0;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const SIGN_SHOW_MAX: C2Rust_Unnamed_20 = 9;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kDecorKindUIWatched: C2Rust_Unnamed_21 = 4;
pub const kDecorKindVirtLines: C2Rust_Unnamed_21 = 3;
pub const kDecorKindVirtText: C2Rust_Unnamed_21 = 2;
pub const kDecorKindSign: C2Rust_Unnamed_21 = 1;
pub const kDecorKindHighlight: C2Rust_Unnamed_21 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorSignHighlight,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_27 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut SignItem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_28 {
    pub name: *mut ::core::ffi::c_char,
    pub val: ::core::ffi::c_int,
}
pub const kExtmarkHighlight: C2Rust_Unnamed_29 = 32;
pub const kExtmarkSign: C2Rust_Unnamed_29 = 2;
pub const kExtmarkNone: C2Rust_Unnamed_29 = 1;
pub const kExtmarkVirtText: C2Rust_Unnamed_29 = 8;
pub const kExtmarkVirtLines: C2Rust_Unnamed_29 = 16;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const kExtmarkSignHL: C2Rust_Unnamed_29 = 4;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    conceal_char: 0 as schar_T,
};
pub const DECOR_SIGN_HIGHLIGHT_INIT: DecorSignHighlight = DecorSignHighlight {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    text: [0 as schar_T, 0 as schar_T],
    sign_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    sign_add_id: 0 as ::core::ffi::c_int,
    number_hl_id: 0 as ::core::ffi::c_int,
    line_hl_id: 0 as ::core::ffi::c_int,
    cursorline_hl_id: 0 as ::core::ffi::c_int,
    next: DECOR_ID_INVALID as uint32_t,
    url: ::core::ptr::null::<::core::ffi::c_char>(),
};
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false_0 != 0,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> bool {
    return mh_get_uint32_t(set, key) != MH_TOMBSTONE as uint32_t;
}
pub const kMTFilterSelect: uint32_t = -1 as ::core::ffi::c_int as uint32_t;
#[inline]
unsafe extern "C" fn ns_in_win(mut ns_id: uint32_t, mut wp: *mut win_T) -> bool {
    if !set_has_uint32_t(namespace_localscope.ptr(), ns_id) {
        return true_0 != 0;
    }
    return set_has_uint32_t(&raw mut (*wp).w_ns_set, ns_id);
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
pub static decor_freelist: GlobalCell<uint32_t> = GlobalCell::new(UINT32_MAX as uint32_t);
pub static to_free_virt: GlobalCell<*mut DecorVirtText> =
    GlobalCell::new(::core::ptr::null_mut::<DecorVirtText>());
pub static to_free_sh: GlobalCell<uint32_t> = GlobalCell::new(UINT32_MAX as uint32_t);
pub unsafe extern "C" fn bufhl_add_hl_pos_offset(
    mut buf: *mut buf_T,
    mut src_id: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut pos_start: lpos_T,
    mut pos_end: lpos_T,
    mut offset: colnr_T,
) {
    let mut hl_start: colnr_T = 0 as colnr_T;
    let mut hl_end: colnr_T = 0 as colnr_T;
    let mut decor: DecorInline = DECOR_INLINE_INIT;
    decor.data.hl.hl_id = hl_id;
    let mut lnum: linenr_T = pos_start.lnum;
    while lnum <= pos_end.lnum {
        let mut end_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if pos_start.lnum < lnum && lnum < pos_end.lnum {
            hl_start = (if offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int
            {
                offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
            end_off = 1 as ::core::ffi::c_int;
            hl_end = 0 as ::core::ffi::c_int as colnr_T;
        } else if lnum == pos_start.lnum && lnum < pos_end.lnum {
            hl_start = pos_start.col + offset;
            end_off = 1 as ::core::ffi::c_int;
            hl_end = 0 as ::core::ffi::c_int as colnr_T;
        } else if pos_start.lnum < lnum && lnum == pos_end.lnum {
            hl_start = (if offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int
            {
                offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
            hl_end = pos_end.col + offset;
        } else if pos_start.lnum == lnum && pos_end.lnum == lnum {
            hl_start = pos_start.col + offset;
            hl_end = pos_end.col + offset;
        }
        extmark_set(
            buf,
            src_id as uint32_t,
            ::core::ptr::null_mut::<uint32_t>(),
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            hl_start,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int + end_off,
            hl_end,
            decor,
            MT_FLAG_DECOR_HL as uint16_t,
            true_0 != 0,
            false_0 != 0,
            true_0 != 0,
            false_0 != 0,
            ::core::ptr::null_mut::<Error>(),
        );
        lnum += 1;
    }
}
pub unsafe extern "C" fn decor_redraw(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut col1: ::core::ffi::c_int,
    mut decor: DecorInline,
) {
    if decor.ext {
        let mut vt: *mut DecorVirtText = decor.data.ext.vt;
        while !vt.is_null() {
            let mut below: bool =
                (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0
                    && (*vt).flags as ::core::ffi::c_int & kVTLinesAbove as ::core::ffi::c_int == 0;
            let mut vt_lnum: linenr_T = row1 as linenr_T + 1 as linenr_T + below as linenr_T;
            redraw_buf_line_later(buf, vt_lnum, true_0 != 0);
            if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0
                || (*vt).pos as ::core::ffi::c_uint
                    == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut vt_col: colnr_T =
                    if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                        0 as colnr_T
                    } else {
                        col1 as colnr_T
                    };
                changed_lines_invalidate_buf(
                    buf,
                    vt_lnum,
                    vt_col,
                    vt_lnum + 1 as linenr_T,
                    0 as linenr_T,
                );
            }
            vt = (*vt).next;
        }
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(idx as isize);
            decor_redraw_sh(buf, row1, row2, *sh);
            idx = (*sh).next;
        }
    } else {
        decor_redraw_sh(buf, row1, row2, decor_sh_from_inline(decor.data.hl));
    };
}
pub unsafe extern "C" fn decor_redraw_sh(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut sh: DecorSignHighlight,
) {
    if sh.hl_id != 0
        || !sh.url.is_null()
        || sh.flags as ::core::ffi::c_int
            & (kSHIsSign as ::core::ffi::c_int
                | kSHSpellOn as ::core::ffi::c_int
                | kSHSpellOff as ::core::ffi::c_int
                | kSHConceal as ::core::ffi::c_int)
            != 0
    {
        if row2 >= row1 {
            redraw_buf_range_later(
                buf,
                row1 as linenr_T + 1 as linenr_T,
                row2 as linenr_T + 1 as linenr_T,
            );
        }
    }
    if sh.flags as ::core::ffi::c_int & kSHConcealLines as ::core::ffi::c_int != 0 {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                changed_window_setting(wp);
            }
            wp = (*wp).w_next;
        }
    }
    if sh.flags as ::core::ffi::c_int & kSHUIWatched as ::core::ffi::c_int != 0 {
        redraw_buf_line_later(buf, row1 as linenr_T + 1 as linenr_T, false_0 != 0);
    }
}
pub unsafe extern "C" fn decor_put_sh(mut item: DecorSignHighlight) -> uint32_t {
    if decor_freelist.get() != UINT32_MAX as uint32_t {
        let mut pos: uint32_t = decor_freelist.get();
        decor_freelist.set(
            (*(*decor_items.ptr())
                .items
                .offset(decor_freelist.get() as isize))
            .next,
        );
        *(*decor_items.ptr()).items.offset(pos as isize) = item;
        return pos;
    } else {
        let mut pos_0: uint32_t = (*decor_items.ptr()).size as uint32_t;
        if (*decor_items.ptr()).size == (*decor_items.ptr()).capacity {
            (*decor_items.ptr()).capacity = if (*decor_items.ptr()).capacity != 0 {
                (*decor_items.ptr()).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*decor_items.ptr()).items = xrealloc(
                (*decor_items.ptr()).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<DecorSignHighlight>()
                    .wrapping_mul((*decor_items.ptr()).capacity),
            ) as *mut DecorSignHighlight;
        } else {
        };
        let c2rust_fresh0 = (*decor_items.ptr()).size;
        (*decor_items.ptr()).size = (*decor_items.ptr()).size.wrapping_add(1);
        *(*decor_items.ptr()).items.offset(c2rust_fresh0 as isize) = item;
        return pos_0;
    };
}
pub unsafe extern "C" fn decor_put_vt(
    mut vt: DecorVirtText,
    mut next: *mut DecorVirtText,
) -> *mut DecorVirtText {
    let mut decor_alloc: *mut DecorVirtText =
        xmalloc(::core::mem::size_of::<DecorVirtText>()) as *mut DecorVirtText;
    *decor_alloc = vt;
    (*decor_alloc).next = next;
    return decor_alloc;
}
pub unsafe extern "C" fn decor_sh_from_inline(
    mut item: DecorHighlightInline,
) -> DecorSignHighlight {
    '_c2rust_label: {
        if item.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int == 0 {
        } else {
            __assert_fail(
                b"!(item.flags & kSHIsSign)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                166 as ::core::ffi::c_uint,
                b"DecorSignHighlight decor_sh_from_inline(DecorHighlightInline)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut conv: DecorSignHighlight = DecorSignHighlight {
        flags: item.flags,
        priority: item.priority,
        hl_id: item.hl_id,
        text: [item.conceal_char, 0],
        sign_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        sign_add_id: 0,
        number_hl_id: 0 as ::core::ffi::c_int,
        line_hl_id: 0 as ::core::ffi::c_int,
        cursorline_hl_id: 0 as ::core::ffi::c_int,
        next: DECOR_ID_INVALID as uint32_t,
        url: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    return conv;
}
pub unsafe extern "C" fn buf_put_decor(
    mut buf: *mut buf_T,
    mut decor: DecorInline,
    mut row: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
) {
    if decor.ext as ::core::ffi::c_int != 0 && (row as linenr_T) < (*buf).b_ml.ml_line_count {
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        row2 = (if ((*buf).b_ml.ml_line_count - 1 as linenr_T) < row2 as linenr_T {
            (*buf).b_ml.ml_line_count - 1 as linenr_T
        } else {
            row2 as linenr_T
        }) as ::core::ffi::c_int;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(idx as isize);
            buf_put_decor_sh(buf, sh, row, row2);
            idx = (*sh).next;
        }
    }
}
unsafe extern "C" fn may_force_numberwidth_recompute(mut buf: *mut buf_T, mut unplace: bool) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf
                && (*wp).w_minscwidth == SCL_NUM
                && ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                && (unplace as ::core::ffi::c_int != 0
                    || (*wp).w_nrwidth_width < 2 as ::core::ffi::c_int)
            {
                (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
static sign_add_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub unsafe extern "C" fn buf_put_decor_sh(
    mut buf: *mut buf_T,
    mut sh: *mut DecorSignHighlight,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
) {
    if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
        let c2rust_fresh1 = sign_add_id.get();
        sign_add_id.set(sign_add_id.get() + 1);
        (*sh).sign_add_id = c2rust_fresh1;
        if (*sh).text[0 as ::core::ffi::c_int as usize] != 0 {
            buf_signcols_count_range(buf, row1, row2, 1 as ::core::ffi::c_int, kFalse);
            may_force_numberwidth_recompute(buf, false_0 != 0);
        }
    }
}
pub unsafe extern "C" fn buf_decor_remove(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut col1: ::core::ffi::c_int,
    mut decor: DecorInline,
    mut free: bool,
) {
    decor_redraw(buf, row1, row2, col1, decor);
    if decor.ext as ::core::ffi::c_int != 0 && (row1 as linenr_T) < (*buf).b_ml.ml_line_count {
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        row2 = (if ((*buf).b_ml.ml_line_count - 1 as linenr_T) < row2 as linenr_T {
            (*buf).b_ml.ml_line_count - 1 as linenr_T
        } else {
            row2 as linenr_T
        }) as ::core::ffi::c_int;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(idx as isize);
            buf_remove_decor_sh(buf, row1, row2, sh);
            idx = (*sh).next;
        }
    }
    if free {
        decor_free(decor);
    }
}
pub unsafe extern "C" fn buf_remove_decor_sh(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut sh: *mut DecorSignHighlight,
) {
    if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
        if (*sh).text[0 as ::core::ffi::c_int as usize] != 0 {
            if buf_meta_total(buf, kMTMetaSignText) != 0 {
                buf_signcols_count_range(buf, row1, row2, -1 as ::core::ffi::c_int, kFalse);
            } else {
                may_force_numberwidth_recompute(buf, true_0 != 0);
                (*buf).b_signcols.count[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
                (*buf).b_signcols.max = 0 as ::core::ffi::c_int;
            }
        }
    }
}
pub unsafe extern "C" fn decor_free(mut decor: DecorInline) {
    if !decor.ext {
        return;
    }
    let mut vt: *mut DecorVirtText = decor.data.ext.vt;
    let mut idx: uint32_t = decor.data.ext.sh_idx;
    if (*decor_state.ptr()).running_decor_provider {
        while !vt.is_null() {
            if (*vt).next.is_null() {
                (*vt).next = to_free_virt.get();
                to_free_virt.set(decor.data.ext.vt);
                break;
            } else {
                vt = (*vt).next;
            }
        }
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(idx as isize);
            if (*sh).next == DECOR_ID_INVALID as uint32_t {
                (*sh).next = to_free_sh.get();
                to_free_sh.set(decor.data.ext.sh_idx);
                break;
            } else {
                idx = (*sh).next;
            }
        }
    } else {
        decor_free_inner(vt, idx);
    };
}
unsafe extern "C" fn decor_free_inner(mut vt: *mut DecorVirtText, mut first_idx: uint32_t) {
    while !vt.is_null() {
        if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
            clear_virtlines(&raw mut (*vt).data.virt_lines);
        } else {
            clear_virttext(&raw mut (*vt).data.virt_text);
        }
        let mut tofree: *mut DecorVirtText = vt;
        vt = (*vt).next;
        xfree(tofree as *mut ::core::ffi::c_void);
    }
    let mut idx: uint32_t = first_idx;
    while idx != DECOR_ID_INVALID as uint32_t {
        let mut sh: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(idx as isize);
        if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*sh).sign_name as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
        (*sh).flags = 0 as uint16_t;
        if !(*sh).url.is_null() {
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*sh).url as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL;
            let _ = *ptr__0;
        }
        if (*sh).next == DECOR_ID_INVALID as uint32_t {
            (*sh).next = decor_freelist.get();
            decor_freelist.set(first_idx);
            break;
        } else {
            idx = (*sh).next;
        }
    }
}
pub unsafe extern "C" fn decor_state_invalidate(mut buf: *mut buf_T) {
    if !(*decor_state.ptr()).win.is_null() && (*(*decor_state.ptr()).win).w_buffer == buf {
        (*decor_state.ptr()).itr_valid = false_0 != 0;
    }
}
pub unsafe extern "C" fn decor_check_to_be_deleted() {
    '_c2rust_label: {
        if !(*decor_state.ptr()).running_decor_provider {
        } else {
            __assert_fail(
                b"!decor_state.running_decor_provider\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                330 as ::core::ffi::c_uint,
                b"void decor_check_to_be_deleted(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    decor_free_inner(to_free_virt.get(), to_free_sh.get());
    to_free_virt.set(::core::ptr::null_mut::<DecorVirtText>());
    to_free_sh.set(DECOR_ID_INVALID as uint32_t);
    (*decor_state.ptr()).win = ::core::ptr::null_mut::<win_T>();
}
pub unsafe extern "C" fn decor_state_free(mut state: *mut DecorState) {
    xfree((*state).slots.items as *mut ::core::ffi::c_void);
    (*state).slots.capacity = 0 as size_t;
    (*state).slots.size = (*state).slots.capacity;
    (*state).slots.items = ::core::ptr::null_mut::<DecorRangeSlot>();
    xfree((*state).ranges_i.items as *mut ::core::ffi::c_void);
    (*state).ranges_i.capacity = 0 as size_t;
    (*state).ranges_i.size = (*state).ranges_i.capacity;
    (*state).ranges_i.items = ::core::ptr::null_mut::<::core::ffi::c_int>();
}
pub unsafe extern "C" fn clear_virttext(mut text: *mut VirtText) {
    let mut i: size_t = 0 as size_t;
    while i < (*text).size {
        xfree((*(*text).items.offset(i as isize)).text as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    xfree((*text).items as *mut ::core::ffi::c_void);
    (*text).capacity = 0 as size_t;
    (*text).size = (*text).capacity;
    (*text).items = ::core::ptr::null_mut::<VirtTextChunk>();
    *text = VirtText {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    };
}
pub unsafe extern "C" fn clear_virtlines(mut lines: *mut VirtLines) {
    let mut i: size_t = 0 as size_t;
    while i < (*lines).size {
        clear_virttext(&raw mut (*(*lines).items.offset(i as isize)).line);
        i = i.wrapping_add(1);
    }
    xfree((*lines).items as *mut ::core::ffi::c_void);
    (*lines).capacity = 0 as size_t;
    (*lines).size = (*lines).capacity;
    (*lines).items = ::core::ptr::null_mut::<virt_line>();
    *lines = VirtLines {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<virt_line>(),
    };
}
pub unsafe extern "C" fn decor_check_invalid_glyphs() {
    let mut i: size_t = 0 as size_t;
    while i < (*decor_items.ptr()).size {
        let mut it: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(i as isize);
        let mut width: ::core::ffi::c_int =
            if (*it).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
                SIGN_WIDTH as ::core::ffi::c_int
            } else if (*it).flags as ::core::ffi::c_int & kSHConceal as ::core::ffi::c_int != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < width {
            if schar_high((*it).text[j as usize]) {
                (*it).text[j as usize] =
                    schar_from_char(schar_get_first_codepoint((*it).text[j as usize]));
            }
            j += 1;
        }
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn next_virt_text_chunk(
    mut vt: VirtText,
    mut pos: *mut size_t,
    mut attr: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    while text.is_null() && *pos < vt.size {
        text = (*vt.items.offset(*pos as isize)).text;
        let mut hl_id: ::core::ffi::c_int = (*vt.items.offset(*pos as isize)).hl_id;
        if hl_id >= 0 as ::core::ffi::c_int {
            *attr = if *attr > 0 as ::core::ffi::c_int {
                *attr
            } else {
                0 as ::core::ffi::c_int
            };
            if hl_id > 0 as ::core::ffi::c_int {
                *attr = hl_combine_attr(*attr, syn_id2attr(hl_id));
            }
        }
        *pos = (*pos).wrapping_add(1);
    }
    return text;
}
pub unsafe extern "C" fn decor_find_virttext(
    mut buf: *mut buf_T,
    mut row: ::core::ffi::c_int,
    mut ns_id: uint64_t,
) -> *mut DecorVirtText {
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos {
            row: 0 as int32_t,
            col: 0,
        },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }];
    marktree_itr_get(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        row as int32_t,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    let mut decor: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
    loop {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row < 0 as int32_t || mark.pos.row > row as int32_t {
            break;
        }
        if !mt_invalid(mark) {
            decor = mt_decor_virt(mark);
            while !decor.is_null()
                && (*decor).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0
            {
                decor = (*decor).next;
            }
            if (ns_id == 0 as uint64_t || ns_id == mark.ns as uint64_t) && !decor.is_null() {
                return decor;
            }
        }
        marktree_itr_next(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
    return ::core::ptr::null_mut::<DecorVirtText>();
}
pub unsafe extern "C" fn decor_redraw_reset(
    mut wp: *mut win_T,
    mut state: *mut DecorState,
) -> bool {
    (*state).row = -1 as ::core::ffi::c_int;
    (*state).win = wp;
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let beg_pos: [::core::ffi::c_int; 2] = [0 as ::core::ffi::c_int, (*state).future_begin];
    let end_pos: [::core::ffi::c_int; 2] = [
        (*state).current_end,
        (*state).ranges_i.size as ::core::ffi::c_int,
    ];
    let mut pos_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while pos_i < 2 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = beg_pos[pos_i as usize];
        while i < end_pos[pos_i as usize] {
            let r: *mut DecorRange =
                &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
            if (*r).owned as ::core::ffi::c_int != 0
                && (*r).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int
            {
                clear_virttext(&raw mut (*(*r).data.vt).data.virt_text);
                xfree((*r).data.vt as *mut ::core::ffi::c_void);
            }
            i += 1;
        }
        pos_i += 1;
    }
    (*state).slots.size = 0 as size_t;
    (*state).ranges_i.size = 0 as size_t;
    (*state).free_slot_i = -1 as ::core::ffi::c_int;
    (*state).current_end = 0 as ::core::ffi::c_int;
    (*state).future_begin = 0 as ::core::ffi::c_int;
    (*state).new_range_ordering = 0 as ::core::ffi::c_int;
    return (*(&raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree)).n_keys != 0;
}
pub unsafe extern "C" fn decor_virt_pos(mut decor: *const DecorRange) -> bool {
    return (*decor).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int
        || (*decor).kind as ::core::ffi::c_int == kDecorKindUIWatched as ::core::ffi::c_int;
}
pub unsafe extern "C" fn decor_virt_pos_kind(mut decor: *const DecorRange) -> VirtTextPos {
    if (*decor).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int {
        return (*(*decor).data.vt).pos;
    }
    if (*decor).kind as ::core::ffi::c_int == kDecorKindUIWatched as ::core::ffi::c_int {
        return (*decor).data.ui.pos;
    }
    return kVPosEndOfLine;
}
pub unsafe extern "C" fn decor_redraw_start(
    mut wp: *mut win_T,
    mut top_row: ::core::ffi::c_int,
    mut state: *mut DecorState,
) -> bool {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    (*state).top_row = top_row;
    (*state).itr_valid = true_0 != 0;
    if !marktree_itr_get_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        top_row,
        0 as ::core::ffi::c_int,
        &raw mut (*state).itr as *mut MarkTreeIter,
    ) {
        return false_0 != 0;
    }
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
    while marktree_itr_step_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        &raw mut (*state).itr as *mut MarkTreeIter,
        &raw mut pair,
    ) {
        let mut m: MTKey = pair.start;
        if mt_invalid(m) as ::core::ffi::c_int != 0 || !mt_decor_any(m) {
            continue;
        }
        decor_range_add_from_inline(
            state,
            pair.start.pos.row as ::core::ffi::c_int,
            pair.start.pos.col as ::core::ffi::c_int,
            pair.end_pos.row as ::core::ffi::c_int,
            pair.end_pos.col as ::core::ffi::c_int,
            mt_decor(m),
            false_0 != 0,
            m.ns,
            m.id,
        );
    }
    return true_0 != 0;
}
unsafe extern "C" fn decor_state_pack(mut state: *mut DecorState) {
    let mut count: ::core::ffi::c_int = (*state).ranges_i.size as ::core::ffi::c_int;
    let cur_end: ::core::ffi::c_int = (*state).current_end;
    let mut fut_beg: ::core::ffi::c_int = (*state).future_begin;
    if fut_beg == count {
        count = cur_end;
        fut_beg = count;
    } else if fut_beg != cur_end {
        let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
        memmove(
            indices.offset(cur_end as isize) as *mut ::core::ffi::c_void,
            indices.offset(fut_beg as isize) as *const ::core::ffi::c_void,
            ((count - fut_beg) as size_t)
                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
        );
        count = cur_end + (count - fut_beg);
        fut_beg = cur_end;
    }
    (*state).ranges_i.size = count as size_t;
    (*state).future_begin = fut_beg;
}
pub unsafe extern "C" fn decor_redraw_line(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
    mut state: *mut DecorState,
) {
    decor_state_pack(state);
    if (*state).row == -1 as ::core::ffi::c_int {
        decor_redraw_start(wp, row, state);
    } else if !(*state).itr_valid {
        marktree_itr_get(
            &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
            row as int32_t,
            0 as ::core::ffi::c_int,
            &raw mut (*state).itr as *mut MarkTreeIter,
        );
        (*state).itr_valid = true_0 != 0;
    }
    (*state).row = row;
    (*state).col_last = -1 as ::core::ffi::c_int;
    (*state).eol_col = -1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn decor_has_more_decorations(
    mut state: *mut DecorState,
    mut row: ::core::ffi::c_int,
) -> bool {
    if (*state).current_end != 0 as ::core::ffi::c_int
        || (*state).future_begin != (*state).ranges_i.size as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    let mut k: MTKey = marktree_itr_current(&raw mut (*state).itr as *mut MarkTreeIter);
    return k.pos.row >= 0 as int32_t && k.pos.row <= row as int32_t;
}
unsafe extern "C" fn decor_range_add_from_inline(
    mut state: *mut DecorState,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut decor: DecorInline,
    mut owned: bool,
    mut ns: uint32_t,
    mut mark_id: uint32_t,
) {
    if decor.ext {
        let mut vt: *mut DecorVirtText = decor.data.ext.vt;
        while !vt.is_null() {
            decor_range_add_virt(state, start_row, start_col, end_row, end_col, vt, owned);
            vt = (*vt).next;
        }
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(idx as isize);
            decor_range_add_sh(
                state,
                start_row,
                start_col,
                end_row,
                end_col,
                sh,
                owned,
                ns,
                mark_id,
                0 as DecorPriority,
            );
            idx = (*sh).next;
        }
    } else {
        let mut sh_0: DecorSignHighlight = decor_sh_from_inline(decor.data.hl);
        decor_range_add_sh(
            state,
            start_row,
            start_col,
            end_row,
            end_col,
            &raw mut sh_0,
            owned,
            ns,
            mark_id,
            0 as DecorPriority,
        );
    };
}
unsafe extern "C" fn decor_range_insert(mut state: *mut DecorState, mut range: *mut DecorRange) {
    let c2rust_fresh2 = (*state).new_range_ordering;
    (*state).new_range_ordering = (*state).new_range_ordering + 1;
    (*range).ordering = c2rust_fresh2;
    let mut index: ::core::ffi::c_int = 0;
    if (*state).free_slot_i >= 0 as ::core::ffi::c_int {
        index = (*state).free_slot_i;
        let mut slot: *mut DecorRangeSlot = (*state).slots.items.offset(index as isize);
        (*state).free_slot_i = (*slot).next_free_i;
        (*slot).range = *range;
    } else {
        index = (*state).slots.size as ::core::ffi::c_int;
        if (*state).slots.size == (*state).slots.capacity {
            (*state).slots.capacity = if (*state).slots.capacity != 0 {
                (*state).slots.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*state).slots.items = xrealloc(
                (*state).slots.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<DecorRangeSlot>().wrapping_mul((*state).slots.capacity),
            ) as *mut DecorRangeSlot;
        } else {
        };
        let c2rust_fresh3 = (*state).slots.size;
        (*state).slots.size = (*state).slots.size.wrapping_add(1);
        (*(*state).slots.items.offset(c2rust_fresh3 as isize)).range = *range;
    }
    let row: ::core::ffi::c_int = (*range).start_row;
    let col: ::core::ffi::c_int = (*range).start_col;
    let count: ::core::ffi::c_int = (*state).ranges_i.size as ::core::ffi::c_int;
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let mut begin: ::core::ffi::c_int = (*state).future_begin;
    let mut end: ::core::ffi::c_int = count;
    while begin < end {
        let mid: ::core::ffi::c_int = begin + (end - begin >> 1 as ::core::ffi::c_int);
        let mr: *mut DecorRange =
            &raw mut (*slots.offset(*indices.offset(mid as isize) as isize)).range;
        let mrow: ::core::ffi::c_int = (*mr).start_row;
        let mcol: ::core::ffi::c_int = (*mr).start_col;
        if mrow < row || mrow == row && mcol <= col {
            begin = mid + 1 as ::core::ffi::c_int;
            if mrow == row && mcol == col {
                break;
            }
        } else {
            end = mid;
        }
    }
    if (*state).ranges_i.size == (*state).ranges_i.capacity {
        (*state).ranges_i.capacity = if (*state).ranges_i.capacity != 0 {
            (*state).ranges_i.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*state).ranges_i.items = xrealloc(
            (*state).ranges_i.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul((*state).ranges_i.capacity),
        ) as *mut ::core::ffi::c_int;
    } else {
    };
    (*state).ranges_i.size = (*state).ranges_i.size.wrapping_add(1);
    let item: *mut ::core::ffi::c_int = (*state).ranges_i.items.offset(begin as isize);
    memmove(
        item.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        item as *const ::core::ffi::c_void,
        ((count - begin) as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
    );
    *item = index;
}
pub unsafe extern "C" fn decor_range_add_virt(
    mut state: *mut DecorState,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut vt: *mut DecorVirtText,
    mut owned: bool,
) {
    let mut is_lines: bool =
        (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0;
    let mut range: DecorRange = DecorRange {
        start_row: start_row,
        start_col: start_col,
        end_row: end_row,
        end_col: end_col,
        ordering: 0,
        priority_internal: ((*vt).priority as DecorPriorityInternal) << 16 as ::core::ffi::c_int,
        owned: owned,
        kind: (if is_lines as ::core::ffi::c_int != 0 {
            kDecorKindVirtLines as ::core::ffi::c_int
        } else {
            kDecorKindVirtText as ::core::ffi::c_int
        }) as DecorRangeKind,
        data: C2Rust_Unnamed_22 { vt: vt },
        attr_id: 0 as ::core::ffi::c_int,
        draw_col: -10 as ::core::ffi::c_int,
    };
    decor_range_insert(state, &raw mut range);
}
pub unsafe extern "C" fn decor_range_add_sh(
    mut state: *mut DecorState,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut sh: *mut DecorSignHighlight,
    mut owned: bool,
    mut ns: uint32_t,
    mut mark_id: uint32_t,
    mut subpriority: DecorPriority,
) {
    if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
        return;
    }
    let mut range: DecorRange = DecorRange {
        start_row: start_row,
        start_col: start_col,
        end_row: end_row,
        end_col: end_col,
        ordering: 0,
        priority_internal: (((*sh).priority as DecorPriorityInternal) << 16 as ::core::ffi::c_int)
            .wrapping_add(subpriority as DecorPriorityInternal),
        owned: owned,
        kind: kDecorKindHighlight as ::core::ffi::c_int as DecorRangeKind,
        data: C2Rust_Unnamed_22 { sh: *sh },
        attr_id: 0 as ::core::ffi::c_int,
        draw_col: -10 as ::core::ffi::c_int,
    };
    if (*sh).hl_id != 0
        || !(*sh).url.is_null()
        || (*sh).flags as ::core::ffi::c_int
            & (kSHConceal as ::core::ffi::c_int
                | kSHSpellOn as ::core::ffi::c_int
                | kSHSpellOff as ::core::ffi::c_int)
            != 0
    {
        if (*sh).hl_id != 0 {
            range.attr_id = syn_id2attr((*sh).hl_id);
        }
        decor_range_insert(state, &raw mut range);
    }
    if (*sh).flags as ::core::ffi::c_int & kSHUIWatched as ::core::ffi::c_int != 0 {
        range.kind = kDecorKindUIWatched as ::core::ffi::c_int as DecorRangeKind;
        range.data.ui.ns_id = ns;
        range.data.ui.mark_id = mark_id;
        range.data.ui.pos = (if (*sh).flags as ::core::ffi::c_int
            & kSHUIWatchedOverlay as ::core::ffi::c_int
            != 0
        {
            kVPosOverlay as ::core::ffi::c_int
        } else {
            kVPosEndOfLine as ::core::ffi::c_int
        }) as VirtTextPos;
        decor_range_insert(state, &raw mut range);
    }
}
pub unsafe extern "C" fn decor_init_draw_col(
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut item: *mut DecorRange,
) {
    let mut vt: *mut DecorVirtText =
        if (*item).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int {
            (*item).data.vt
        } else {
            ::core::ptr::null_mut::<DecorVirtText>()
        };
    let mut pos: VirtTextPos = decor_virt_pos_kind(item);
    if win_col < 0 as ::core::ffi::c_int
        && pos as ::core::ffi::c_uint != kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*item).draw_col = win_col;
    } else if pos as ::core::ffi::c_uint
        == kVPosOverlay as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*item).draw_col = if !vt.is_null()
            && (*vt).flags as ::core::ffi::c_int & kVTHide as ::core::ffi::c_int != 0
            && hidden as ::core::ffi::c_int != 0
        {
            INT_MIN
        } else {
            win_col
        };
    } else {
        (*item).draw_col = -1 as ::core::ffi::c_int;
    };
}
pub unsafe extern "C" fn decor_recheck_draw_col(
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut state: *mut DecorState,
) {
    let end: ::core::ffi::c_int = (*state).current_end;
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < end {
        let r: *mut DecorRange =
            &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
        if (*r).draw_col == -3 as ::core::ffi::c_int {
            decor_init_draw_col(win_col, hidden, r);
        }
        i += 1;
    }
}
pub unsafe extern "C" fn decor_redraw_col_impl(
    mut wp: *mut win_T,
    mut col: ::core::ffi::c_int,
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut state: *mut DecorState,
    mut max_col_last: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let buf: *mut buf_T = (*wp).w_buffer;
    let row: ::core::ffi::c_int = (*state).row;
    let mut col_last: ::core::ffi::c_int = max_col_last;
    let mut endpos: MTPos = MTPos { row: 0, col: 0 };
    loop {
        let mut mark: MTKey = marktree_itr_current(&raw mut (*state).itr as *mut MarkTreeIter);
        if mark.pos.row < 0 as int32_t || mark.pos.row > row as int32_t {
            break;
        }
        if mark.pos.row == row as int32_t && mark.pos.col > col as int32_t {
            col_last = (if (col_last as int32_t) < mark.pos.col - 1 as int32_t {
                col_last as int32_t
            } else {
                mark.pos.col - 1 as int32_t
            }) as ::core::ffi::c_int;
            break;
        } else {
            if !(mt_invalid(mark) as ::core::ffi::c_int != 0
                || mt_end(mark) as ::core::ffi::c_int != 0
                || !mt_decor_any(mark)
                || !ns_in_win(mark.ns, wp))
            {
                endpos = marktree_get_altpos(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    mark,
                    ::core::ptr::null_mut::<MarkTreeIter>(),
                );
                decor_range_add_from_inline(
                    state,
                    mark.pos.row as ::core::ffi::c_int,
                    mark.pos.col as ::core::ffi::c_int,
                    endpos.row as ::core::ffi::c_int,
                    endpos.col as ::core::ffi::c_int,
                    mt_decor(mark),
                    false_0 != 0,
                    mark.ns,
                    mark.id,
                );
            }
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut (*state).itr as *mut MarkTreeIter,
            );
        }
    }
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let mut count: ::core::ffi::c_int = (*state).ranges_i.size as ::core::ffi::c_int;
    let mut cur_end: ::core::ffi::c_int = (*state).current_end;
    let mut fut_beg: ::core::ffi::c_int = (*state).future_begin;
    while fut_beg < count {
        let index: ::core::ffi::c_int = *indices.offset(fut_beg as isize);
        let r: *mut DecorRange = &raw mut (*slots.offset(index as isize)).range;
        if (*r).start_row > row || (*r).start_row == row && (*r).start_col > col {
            break;
        }
        let ordering: ::core::ffi::c_int = (*r).ordering;
        let priority: DecorPriorityInternal = (*r).priority_internal;
        let mut begin: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut end: ::core::ffi::c_int = cur_end;
        while begin < end {
            let mut mid: ::core::ffi::c_int = begin + (end - begin >> 1 as ::core::ffi::c_int);
            let mut mi: ::core::ffi::c_int = *indices.offset(mid as isize);
            let mut mr: *mut DecorRange = &raw mut (*slots.offset(mi as isize)).range;
            if (*mr).priority_internal < priority
                || (*mr).priority_internal == priority && (*mr).ordering < ordering
            {
                begin = mid + 1 as ::core::ffi::c_int;
            } else {
                end = mid;
            }
        }
        let item: *mut ::core::ffi::c_int = indices.offset(begin as isize);
        memmove(
            item.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            item as *const ::core::ffi::c_void,
            ((cur_end - begin) as size_t)
                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
        );
        *item = index;
        cur_end += 1;
        fut_beg += 1;
    }
    if fut_beg < count {
        let mut r_0: *mut DecorRange =
            &raw mut (*slots.offset(*indices.offset(fut_beg as isize) as isize)).range;
        if (*r_0).start_row == row {
            col_last = if col_last < (*r_0).start_col - 1 as ::core::ffi::c_int {
                col_last
            } else {
                (*r_0).start_col - 1 as ::core::ffi::c_int
            };
        }
    }
    let mut new_cur_end: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut conceal: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut conceal_char: schar_T = 0 as schar_T;
    let mut conceal_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut spell: TriState = kNone;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < cur_end {
        let index_0: ::core::ffi::c_int = *indices.offset(i as isize);
        let slot: *mut DecorRangeSlot = slots.offset(index_0 as isize);
        let r_1: *mut DecorRange = &raw mut (*slot).range;
        let mut keep: bool = false;
        if (*r_1).end_row < row || (*r_1).end_row == row && (*r_1).end_col <= col {
            keep = (*r_1).start_row >= row && decor_virt_pos(r_1) as ::core::ffi::c_int != 0;
        } else {
            keep = true_0 != 0;
            if (*r_1).end_row == row && (*r_1).end_col > col {
                col_last = if col_last < (*r_1).end_col - 1 as ::core::ffi::c_int {
                    col_last
                } else {
                    (*r_1).end_col - 1 as ::core::ffi::c_int
                };
            }
            if (*r_1).attr_id > 0 as ::core::ffi::c_int {
                attr = hl_combine_attr(attr, (*r_1).attr_id);
            }
            if (*r_1).kind as ::core::ffi::c_int == kDecorKindHighlight as ::core::ffi::c_int
                && (*r_1).data.sh.flags as ::core::ffi::c_int & kSHConceal as ::core::ffi::c_int
                    != 0
            {
                conceal = 1 as ::core::ffi::c_int;
                if (*r_1).start_row == row && (*r_1).start_col == col {
                    let mut sh: *mut DecorSignHighlight = &raw mut (*r_1).data.sh;
                    conceal = 2 as ::core::ffi::c_int;
                    conceal_char = (*sh).text[0 as ::core::ffi::c_int as usize];
                    col_last = if col_last < (*r_1).start_col {
                        col_last
                    } else {
                        (*r_1).start_col
                    };
                    conceal_attr = (*r_1).attr_id;
                }
            }
            if (*r_1).kind as ::core::ffi::c_int == kDecorKindHighlight as ::core::ffi::c_int {
                if (*r_1).data.sh.flags as ::core::ffi::c_int & kSHSpellOn as ::core::ffi::c_int
                    != 0
                {
                    spell = kTrue;
                } else if (*r_1).data.sh.flags as ::core::ffi::c_int
                    & kSHSpellOff as ::core::ffi::c_int
                    != 0
                {
                    spell = kFalse;
                }
                if !(*r_1).data.sh.url.is_null() {
                    attr = hl_add_url(attr, (*r_1).data.sh.url);
                }
            }
        }
        if (*r_1).start_row == row
            && (*r_1).start_col <= col
            && decor_virt_pos(r_1) as ::core::ffi::c_int != 0
            && (*r_1).draw_col == -10 as ::core::ffi::c_int
        {
            decor_init_draw_col(win_col, hidden, r_1);
        }
        if keep {
            let c2rust_fresh4 = new_cur_end;
            new_cur_end = new_cur_end + 1;
            *indices.offset(c2rust_fresh4 as isize) = index_0;
        } else {
            if (*r_1).owned {
                if (*r_1).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int {
                    clear_virttext(&raw mut (*(*r_1).data.vt).data.virt_text);
                    xfree((*r_1).data.vt as *mut ::core::ffi::c_void);
                } else if (*r_1).kind as ::core::ffi::c_int
                    == kDecorKindHighlight as ::core::ffi::c_int
                {
                    xfree((*r_1).data.sh.url as *mut ::core::ffi::c_void);
                }
            }
            let mut fi: *mut ::core::ffi::c_int = &raw mut (*state).free_slot_i;
            (*slot).next_free_i = *fi;
            *fi = index_0;
        }
        i += 1;
    }
    cur_end = new_cur_end;
    if fut_beg == count {
        count = cur_end;
        fut_beg = count;
    }
    (*state).ranges_i.size = count as size_t;
    (*state).future_begin = fut_beg;
    (*state).current_end = cur_end;
    (*state).col_last = col_last;
    (*state).current = attr;
    (*state).conceal = conceal;
    (*state).conceal_char = conceal_char;
    (*state).conceal_attr = conceal_attr;
    (*state).spell = spell;
    return attr;
}
static conceal_filter: GlobalCell<[uint32_t; 5]> = GlobalCell::new([0, 0, 0, 0, kMTFilterSelect]);
pub unsafe extern "C" fn decor_conceal_line(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
    mut check_cursor: bool,
) -> bool {
    if row < 0 as ::core::ffi::c_int
        || (*wp).w_onebuf_opt.wo_cole < 2 as OptInt
        || !check_cursor
            && wp == curwin.get()
            && row as linenr_T + 1 as linenr_T == (*wp).w_cursor.lnum
            && !conceal_cursor_line(wp)
    {
        return false_0 != 0;
    }
    if buf_meta_total((*wp).w_buffer, kMTMetaConcealLines) == 0 {
        return decor_providers_invoke_conceal_line(wp, row);
    }
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
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    marktree_itr_get_overlap(
        &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
        row,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    while marktree_itr_step_overlap(
        &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        &raw mut pair,
    ) {
        if mt_conceal_lines(pair.start) as ::core::ffi::c_int != 0
            && ns_in_win(pair.start.ns, wp) as ::core::ffi::c_int != 0
        {
            return true_0 != 0;
        }
    }
    marktree_itr_step_out_filter(
        &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        (conceal_filter.ptr() as *const _) as MetaFilter,
    );
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row > row as int32_t {
            break;
        }
        if mt_conceal_lines(mark) as ::core::ffi::c_int != 0
            && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0
        {
            return true_0 != 0;
        }
        marktree_itr_next_filter(
            &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            row + 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (conceal_filter.ptr() as *const _) as MetaFilter,
        );
    }
    return decor_providers_invoke_conceal_line(wp, row);
}
pub unsafe extern "C" fn win_lines_concealed(mut wp: *mut win_T) -> bool {
    return hasAnyFolding(wp) != 0 || (*wp).w_onebuf_opt.wo_cole >= 2 as OptInt;
}
pub unsafe extern "C" fn sign_item_cmp(
    mut p1: *const ::core::ffi::c_void,
    mut p2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut s1: *const SignItem = p1 as *mut SignItem;
    let mut s2: *const SignItem = p2 as *mut SignItem;
    if (*(*s1).sh).priority as ::core::ffi::c_int != (*(*s2).sh).priority as ::core::ffi::c_int {
        return if ((*(*s1).sh).priority as ::core::ffi::c_int)
            < (*(*s2).sh).priority as ::core::ffi::c_int
        {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    if (*s1).id != (*s2).id {
        return if (*s1).id < (*s2).id {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    if (*(*s1).sh).sign_add_id != (*(*s2).sh).sign_add_id {
        return if (*(*s1).sh).sign_add_id < (*(*s2).sh).sign_add_id {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    return 0 as ::core::ffi::c_int;
}
static sign_filter: GlobalCell<[uint32_t; 5]> =
    GlobalCell::new([0, 0, kMTFilterSelect, kMTFilterSelect, 0]);
pub unsafe extern "C" fn decor_redraw_signs(
    mut wp: *mut win_T,
    mut buf: *mut buf_T,
    mut row: ::core::ffi::c_int,
    mut sattrs: *mut SignTextAttrs,
    mut line_id: *mut ::core::ffi::c_int,
    mut cul_id: *mut ::core::ffi::c_int,
    mut num_id: *mut ::core::ffi::c_int,
) {
    if !buf_has_signs(buf) {
        return;
    }
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
    let mut num_text: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    let mut signs: C2Rust_Unnamed_27 = C2Rust_Unnamed_27 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<SignItem>(),
    };
    marktree_itr_get_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        row,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    while marktree_itr_step_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        &raw mut pair,
    ) {
        if !mt_invalid(pair.start)
            && mt_decor_sign(pair.start) as ::core::ffi::c_int != 0
            && ns_in_win(pair.start.ns, wp) as ::core::ffi::c_int != 0
        {
            let mut sh: *mut DecorSignHighlight = decor_find_sign(mt_decor(pair.start));
            num_text += ((*sh).text[0 as ::core::ffi::c_int as usize] != NUL as schar_T)
                as ::core::ffi::c_int;
            if signs.size == signs.capacity {
                signs.capacity = if signs.capacity != 0 {
                    signs.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                signs.items = xrealloc(
                    signs.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<SignItem>().wrapping_mul(signs.capacity),
                ) as *mut SignItem;
            } else {
            };
            let c2rust_fresh5 = signs.size;
            signs.size = signs.size.wrapping_add(1);
            *signs.items.offset(c2rust_fresh5 as isize) = SignItem {
                sh: sh,
                id: pair.start.id,
            };
        }
    }
    marktree_itr_step_out_filter(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        (sign_filter.ptr() as *const _) as MetaFilter,
    );
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row != row as int32_t {
            break;
        }
        if !mt_invalid(mark)
            && !mt_end(mark)
            && mt_decor_sign(mark) as ::core::ffi::c_int != 0
            && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0
        {
            let mut sh_0: *mut DecorSignHighlight = decor_find_sign(mt_decor(mark));
            num_text += ((*sh_0).text[0 as ::core::ffi::c_int as usize] != NUL as schar_T)
                as ::core::ffi::c_int;
            if signs.size == signs.capacity {
                signs.capacity = if signs.capacity != 0 {
                    signs.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                signs.items = xrealloc(
                    signs.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<SignItem>().wrapping_mul(signs.capacity),
                ) as *mut SignItem;
            } else {
            };
            let c2rust_fresh6 = signs.size;
            signs.size = signs.size.wrapping_add(1);
            *signs.items.offset(c2rust_fresh6 as isize) = SignItem {
                sh: sh_0,
                id: mark.id,
            };
        }
        marktree_itr_next_filter(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            row + 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (sign_filter.ptr() as *const _) as MetaFilter,
        );
    }
    if signs.size != 0 {
        let mut width: ::core::ffi::c_int = if (*wp).w_minscwidth == SCL_NUM {
            1 as ::core::ffi::c_int
        } else {
            (*wp).w_scwidth
        };
        let mut len: ::core::ffi::c_int = if width < num_text { width } else { num_text };
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        qsort(
            signs.items.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            signs.size,
            ::core::mem::size_of::<SignItem>(),
            Some(
                sign_item_cmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        let mut i: size_t = 0 as size_t;
        while i < signs.size {
            let mut sh_1: *mut DecorSignHighlight = (*signs.items.offset(i as isize)).sh;
            if !sattrs.is_null() && idx < len && (*sh_1).text[0 as ::core::ffi::c_int as usize] != 0
            {
                memcpy(
                    &raw mut (*sattrs.offset(idx as isize)).text as *mut schar_T
                        as *mut ::core::ffi::c_void,
                    &raw mut (*sh_1).text as *mut schar_T as *const ::core::ffi::c_void,
                    (SIGN_WIDTH as ::core::ffi::c_int as size_t)
                        .wrapping_mul(::core::mem::size_of::<sattr_T>()),
                );
                let c2rust_fresh7 = idx;
                idx = idx + 1;
                (*sattrs.offset(c2rust_fresh7 as isize)).hl_id = (*sh_1).hl_id;
            }
            if !num_id.is_null() && *num_id <= 0 as ::core::ffi::c_int {
                *num_id = (*sh_1).number_hl_id;
            }
            if !line_id.is_null() && *line_id <= 0 as ::core::ffi::c_int {
                *line_id = (*sh_1).line_hl_id;
            }
            if !cul_id.is_null() && *cul_id <= 0 as ::core::ffi::c_int {
                *cul_id = (*sh_1).cursorline_hl_id;
            }
            i = i.wrapping_add(1);
        }
        xfree(signs.items as *mut ::core::ffi::c_void);
        signs.capacity = 0 as size_t;
        signs.size = signs.capacity;
        signs.items = ::core::ptr::null_mut::<SignItem>();
    }
}
pub unsafe extern "C" fn decor_find_sign(mut decor: DecorInline) -> *mut DecorSignHighlight {
    if !decor.ext {
        return ::core::ptr::null_mut::<DecorSignHighlight>();
    }
    let mut decor_id: uint32_t = decor.data.ext.sh_idx;
    loop {
        if decor_id == DECOR_ID_INVALID as uint32_t {
            return ::core::ptr::null_mut::<DecorSignHighlight>();
        }
        let mut sh: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(decor_id as isize);
        if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
            return sh;
        }
        decor_id = (*sh).next;
    }
}
static signtext_filter: GlobalCell<[uint32_t; 5]> = GlobalCell::new([0, 0, 0, kMTFilterSelect, 0]);
pub unsafe extern "C" fn buf_signcols_count_range(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut add: ::core::ffi::c_int,
    mut clear: TriState,
) {
    if !(*buf).b_signcols.autom || row2 < row1 || buf_meta_total(buf, kMTMetaSignText) == 0 {
        return;
    }
    let mut count: *mut ::core::ffi::c_int = xcalloc(
        (row2 + 1 as ::core::ffi::c_int - row1) as size_t,
        ::core::mem::size_of::<::core::ffi::c_int>(),
    ) as *mut ::core::ffi::c_int;
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    let mut pair: MTPair = MTPair {
        start: MTKey {
            pos: MTPos {
                row: 0 as int32_t,
                col: 0,
            },
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
    marktree_itr_get_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        row1,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    while marktree_itr_step_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        &raw mut pair,
    ) {
        if pair.start.flags as ::core::ffi::c_int & MT_FLAG_DECOR_SIGNTEXT != 0
            && !mt_invalid(pair.start)
        {
            let mut i: ::core::ffi::c_int = row1;
            while i as int32_t
                <= (if (row2 as int32_t) < pair.end_pos.row {
                    row2 as int32_t
                } else {
                    pair.end_pos.row
                })
            {
                *count.offset((i - row1) as isize) += 1;
                i += 1;
            }
        }
    }
    marktree_itr_step_out_filter(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        (signtext_filter.ptr() as *const _) as MetaFilter,
    );
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row > row2 as int32_t {
            break;
        }
        if mark.flags as ::core::ffi::c_int & MT_FLAG_DECOR_SIGNTEXT != 0
            && !mt_invalid(mark)
            && !mt_end(mark)
        {
            let mut end: MTPos = marktree_get_altpos(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                mark,
                ::core::ptr::null_mut::<MarkTreeIter>(),
            );
            let mut i_0: ::core::ffi::c_int = mark.pos.row as ::core::ffi::c_int;
            while i_0 as int32_t
                <= (if (row2 as int32_t) < end.row {
                    row2 as int32_t
                } else {
                    end.row
                })
            {
                *count.offset((i_0 - row1) as isize) += 1;
                i_0 += 1;
            }
        }
        marktree_itr_next_filter(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            row2 + 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (signtext_filter.ptr() as *const _) as MetaFilter,
        );
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < row2 + 1 as ::core::ffi::c_int - row1 {
        let mut prevwidth: ::core::ffi::c_int =
            if (SIGN_SHOW_MAX as ::core::ffi::c_int) < *count.offset(i_1 as isize) - add {
                SIGN_SHOW_MAX as ::core::ffi::c_int
            } else {
                *count.offset(i_1 as isize) - add
            };
        if clear as ::core::ffi::c_int != kNone as ::core::ffi::c_int
            && prevwidth > 0 as ::core::ffi::c_int
        {
            (*buf).b_signcols.count[(prevwidth - 1 as ::core::ffi::c_int) as usize] -= 1;
            '_c2rust_label: {
                if (*buf).b_signcols.count[(prevwidth - 1 as ::core::ffi::c_int) as usize]
                    >= 0 as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"buf->b_signcols.count[prevwidth - 1] >= 0\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1078 as ::core::ffi::c_uint,
                        b"void buf_signcols_count_range(buf_T *, int, int, int, TriState)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
        }
        let mut width: ::core::ffi::c_int =
            if (SIGN_SHOW_MAX as ::core::ffi::c_int) < *count.offset(i_1 as isize) {
                SIGN_SHOW_MAX as ::core::ffi::c_int
            } else {
                *count.offset(i_1 as isize)
            };
        if clear as ::core::ffi::c_int != kTrue as ::core::ffi::c_int
            && width > 0 as ::core::ffi::c_int
        {
            (*buf).b_signcols.count[(width - 1 as ::core::ffi::c_int) as usize] += 1;
            if width > (*buf).b_signcols.max {
                (*buf).b_signcols.max = width;
            }
        }
        i_1 += 1;
    }
    xfree(count as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn decor_redraw_end(mut state: *mut DecorState) {
    (*state).win = ::core::ptr::null_mut::<win_T>();
}
pub unsafe extern "C" fn decor_redraw_eol(
    mut wp: *mut win_T,
    mut state: *mut DecorState,
    mut eol_attr: *mut ::core::ffi::c_int,
    mut eol_col: ::core::ffi::c_int,
) -> bool {
    decor_redraw_col(
        wp,
        MAXCOL as ::core::ffi::c_int,
        MAXCOL as ::core::ffi::c_int,
        false_0 != 0,
        state,
        MAXCOL as ::core::ffi::c_int,
    );
    (*state).eol_col = eol_col;
    let count: ::core::ffi::c_int = (*state).current_end;
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let mut has_virt_pos: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count {
        let mut r: *mut DecorRange =
            &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
        has_virt_pos = has_virt_pos as ::core::ffi::c_int
            | ((*r).start_row == (*state).row && decor_virt_pos(r) as ::core::ffi::c_int != 0)
                as ::core::ffi::c_int
            != 0;
        if (*r).kind as ::core::ffi::c_int == kDecorKindHighlight as ::core::ffi::c_int
            && (*r).data.sh.flags as ::core::ffi::c_int & kSHHlEol as ::core::ffi::c_int != 0
        {
            *eol_attr = hl_combine_attr(*eol_attr, (*r).attr_id);
        }
        i += 1;
    }
    return has_virt_pos;
}
static lines_filter: GlobalCell<[uint32_t; 5]> = GlobalCell::new([0, kMTFilterSelect, 0, 0, 0]);
pub unsafe extern "C" fn decor_virt_lines(
    mut wp: *mut win_T,
    mut start_row: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut num_below: *mut ::core::ffi::c_int,
    mut lines: *mut VirtLines,
    mut apply_folds: bool,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    if buf_meta_total(buf, kMTMetaLines) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos {
            row: 0 as int32_t,
            col: 0,
        },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }];
    if !marktree_itr_get_filter(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        if start_row - 1 as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            start_row as int32_t - 1 as int32_t
        } else {
            0 as int32_t
        },
        0 as ::core::ffi::c_int,
        end_row,
        0 as ::core::ffi::c_int,
        (lines_filter.ptr() as *const _) as MetaFilter,
        &raw mut itr as *mut MarkTreeIter,
    ) {
        return 0 as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if start_row >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"start_row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1138 as ::core::ffi::c_uint,
                b"int decor_virt_lines(win_T *, int, int, int *, VirtLines *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut virt_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        let mut vt: *mut DecorVirtText = mt_decor_virt(mark);
        if !mt_invalid(mark) && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0 {
            while !vt.is_null() {
                if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                    let mut above: bool = (*vt).flags as ::core::ffi::c_int
                        & kVTLinesAbove as ::core::ffi::c_int
                        != 0;
                    let mut mrow: ::core::ffi::c_int = mark.pos.row as ::core::ffi::c_int;
                    let mut draw_row: ::core::ffi::c_int = mrow
                        + (if above as ::core::ffi::c_int != 0 {
                            0 as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                        });
                    if draw_row >= start_row
                        && draw_row < end_row
                        && (!apply_folds
                            || !(hasFolding(
                                wp,
                                mrow as linenr_T + 1 as linenr_T,
                                ::core::ptr::null_mut::<linenr_T>(),
                                ::core::ptr::null_mut::<linenr_T>(),
                            ) as ::core::ffi::c_int
                                != 0
                                || decor_conceal_line(wp, mrow, false_0 != 0)
                                    as ::core::ffi::c_int
                                    != 0))
                    {
                        virt_lines += (*vt).data.virt_lines.size as ::core::ffi::c_int;
                        if !lines.is_null() {
                            if (*vt).data.virt_lines.size > 0 as size_t {
                                if (*lines).capacity
                                    < (*lines).size.wrapping_add((*vt).data.virt_lines.size)
                                {
                                    (*lines).capacity =
                                        (*lines).size.wrapping_add((*vt).data.virt_lines.size);
                                    (*lines).capacity = (*lines).capacity.wrapping_sub(1);
                                    (*lines).capacity |=
                                        (*lines).capacity >> 1 as ::core::ffi::c_int;
                                    (*lines).capacity |=
                                        (*lines).capacity >> 2 as ::core::ffi::c_int;
                                    (*lines).capacity |=
                                        (*lines).capacity >> 4 as ::core::ffi::c_int;
                                    (*lines).capacity |=
                                        (*lines).capacity >> 8 as ::core::ffi::c_int;
                                    (*lines).capacity |=
                                        (*lines).capacity >> 16 as ::core::ffi::c_int;
                                    (*lines).capacity = (*lines).capacity.wrapping_add(1);
                                    (*lines).capacity = (*lines).capacity;
                                    (*lines).items = xrealloc(
                                        (*lines).items as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<virt_line>()
                                            .wrapping_mul((*lines).capacity),
                                    )
                                        as *mut virt_line;
                                }
                                '_c2rust_label_0: {
                                    if !(*lines).items.is_null() {
                                    } else {
                                        __assert_fail(
                                            b"(*lines).items\0".as_ptr() as *const ::core::ffi::c_char,
                                            b"src/nvim/decoration.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            1155 as ::core::ffi::c_uint,
                                            b"int decor_virt_lines(win_T *, int, int, int *, VirtLines *, _Bool)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                memcpy(
                                    (*lines).items.offset((*lines).size as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*vt).data.virt_lines.items as *const ::core::ffi::c_void,
                                    ::core::mem::size_of::<virt_line>()
                                        .wrapping_mul((*vt).data.virt_lines.size),
                                );
                                (*lines).size =
                                    (*lines).size.wrapping_add((*vt).data.virt_lines.size);
                            }
                        }
                        if !num_below.is_null() && !above {
                            *num_below += (*vt).data.virt_lines.size as ::core::ffi::c_int;
                        }
                    }
                }
                vt = (*vt).next;
            }
        }
        if !marktree_itr_next_filter(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            end_row,
            0 as ::core::ffi::c_int,
            (lines_filter.ptr() as *const _) as MetaFilter,
        ) {
            break;
        }
    }
    return virt_lines;
}
pub unsafe extern "C" fn decor_to_dict_legacy(
    mut dict: *mut Dict,
    mut decor: DecorInline,
    mut hl_name: bool,
    mut arena: *mut Arena,
) {
    let mut sh_hl: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    let mut sh_sign: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    let mut virt_text: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
    let mut virt_lines: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
    let mut priority: int32_t = -1 as int32_t;
    if decor.ext {
        let mut vt: *mut DecorVirtText = decor.data.ext.vt;
        while !vt.is_null() {
            if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                virt_lines = vt;
            } else {
                virt_text = vt;
            }
            vt = (*vt).next;
        }
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(idx as isize);
            if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
                sh_sign = *sh;
            } else {
                sh_hl = *sh;
            }
            idx = (*sh).next;
        }
    } else {
        sh_hl = decor_sh_from_inline(decor.data.hl);
    }
    if sh_hl.hl_id != 0 {
        let c2rust_fresh8 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh8 as isize) = key_value_pair {
            key: cstr_as_string(b"hl_group\0".as_ptr() as *const ::core::ffi::c_char),
            value: hl_group_name(sh_hl.hl_id, hl_name),
        };
        let c2rust_fresh9 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"hl_eol\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 {
                    boolean: sh_hl.flags as ::core::ffi::c_int & kSHHlEol as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        priority = sh_hl.priority as int32_t;
    }
    if sh_hl.flags as ::core::ffi::c_int & kSHConceal as ::core::ffi::c_int != 0 {
        let mut buf: [::core::ffi::c_char; 32] = [0; 32];
        schar_get(
            &raw mut buf as *mut ::core::ffi::c_char,
            sh_hl.text[0 as ::core::ffi::c_int as usize],
        );
        let c2rust_fresh10 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"conceal\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: arena_string(
                        arena,
                        cstr_as_string(&raw mut buf as *mut ::core::ffi::c_char),
                    ),
                },
            },
        };
    }
    if sh_hl.flags as ::core::ffi::c_int & kSHConcealLines as ::core::ffi::c_int != 0 {
        let c2rust_fresh11 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh11 as isize) = key_value_pair {
            key: cstr_as_string(b"conceal_lines\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: cstr_as_string(b"\0".as_ptr() as *const ::core::ffi::c_char),
                },
            },
        };
    }
    if sh_hl.flags as ::core::ffi::c_int & kSHSpellOn as ::core::ffi::c_int != 0 {
        let c2rust_fresh12 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh12 as isize) = key_value_pair {
            key: cstr_as_string(b"spell\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 { boolean: true },
            },
        };
    } else if sh_hl.flags as ::core::ffi::c_int & kSHSpellOff as ::core::ffi::c_int != 0 {
        let c2rust_fresh13 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh13 as isize) = key_value_pair {
            key: cstr_as_string(b"spell\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 { boolean: false },
            },
        };
    }
    if sh_hl.flags as ::core::ffi::c_int & kSHUIWatched as ::core::ffi::c_int != 0 {
        let c2rust_fresh14 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh14 as isize) = key_value_pair {
            key: cstr_as_string(b"ui_watched\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 { boolean: true },
            },
        };
    }
    if !sh_hl.url.is_null() {
        let c2rust_fresh15 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh15 as isize) = key_value_pair {
            key: cstr_as_string(b"url\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: cstr_as_string(sh_hl.url),
                },
            },
        };
    }
    if !virt_text.is_null() {
        if (*virt_text).hl_mode != 0 {
            let c2rust_fresh16 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh16 as isize) = key_value_pair {
                key: cstr_as_string(b"hl_mode\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(
                            *(&raw const hl_mode_str as *const *const ::core::ffi::c_char)
                                .offset((*virt_text).hl_mode as isize),
                        ),
                    },
                },
            };
        }
        let mut chunks: Array = virt_text_to_array((*virt_text).data.virt_text, hl_name, arena);
        let c2rust_fresh17 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh17 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_text\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_14 { array: chunks },
            },
        };
        let c2rust_fresh18 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh18 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_text_hide\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 {
                    boolean: (*virt_text).flags as ::core::ffi::c_int
                        & kVTHide as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        let c2rust_fresh19 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh19 as isize) = key_value_pair {
            key: cstr_as_string(
                b"virt_text_repeat_linebreak\0".as_ptr() as *const ::core::ffi::c_char
            ),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 {
                    boolean: (*virt_text).flags as ::core::ffi::c_int
                        & kVTRepeatLinebreak as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        if (*virt_text).pos as ::core::ffi::c_uint
            == kVPosWinCol as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let c2rust_fresh20 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh20 as isize) = key_value_pair {
                key: cstr_as_string(b"virt_text_win_col\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_14 {
                        integer: (*virt_text).col as Integer,
                    },
                },
            };
        }
        let c2rust_fresh21 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_text_pos\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: cstr_as_string(
                        *(&raw const virt_text_pos_str as *const *const ::core::ffi::c_char)
                            .offset((*virt_text).pos as isize),
                    ),
                },
            },
        };
        priority = (*virt_text).priority as int32_t;
    }
    if !virt_lines.is_null() {
        let mut all_chunks: Array = arena_array(arena, (*virt_lines).data.virt_lines.size);
        let mut virt_lines_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: size_t = 0 as size_t;
        while i < (*virt_lines).data.virt_lines.size {
            virt_lines_flags = (*(*virt_lines).data.virt_lines.items.offset(i as isize)).flags;
            let mut chunks_0: Array = virt_text_to_array(
                (*(*virt_lines).data.virt_lines.items.offset(i as isize)).line,
                hl_name,
                arena,
            );
            if all_chunks.size == all_chunks.capacity {
                all_chunks.capacity = if all_chunks.capacity != 0 {
                    all_chunks.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                all_chunks.items = xrealloc(
                    all_chunks.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<Object>().wrapping_mul(all_chunks.capacity),
                ) as *mut Object;
            } else {
            };
            let c2rust_fresh22 = all_chunks.size;
            all_chunks.size = all_chunks.size.wrapping_add(1);
            *all_chunks.items.offset(c2rust_fresh22 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_14 { array: chunks_0 },
            };
            i = i.wrapping_add(1);
        }
        let c2rust_fresh23 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_lines\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_14 { array: all_chunks },
            },
        };
        let c2rust_fresh24 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh24 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_lines_above\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 {
                    boolean: (*virt_lines).flags as ::core::ffi::c_int
                        & kVTLinesAbove as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        let c2rust_fresh25 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh25 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_lines_leftcol\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 {
                    boolean: virt_lines_flags & kVLLeftcol as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh26 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh26 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_lines_overflow\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: cstr_as_string(
                        if virt_lines_flags & kVLScroll as ::core::ffi::c_int != 0 {
                            b"scroll\0".as_ptr() as *const ::core::ffi::c_char
                        } else {
                            b"trunc\0".as_ptr() as *const ::core::ffi::c_char
                        },
                    ),
                },
            },
        };
        priority = (*virt_lines).priority as int32_t;
    }
    if sh_sign.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
        if sh_sign.text[0 as ::core::ffi::c_int as usize] != 0 {
            let mut buf_0: [::core::ffi::c_char; 64] = [0; 64];
            describe_sign_text(
                &raw mut buf_0 as *mut ::core::ffi::c_char,
                &raw mut sh_sign.text as *mut schar_T,
            );
            let c2rust_fresh27 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh27 as isize) = key_value_pair {
                key: cstr_as_string(b"sign_text\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: arena_string(
                            arena,
                            cstr_as_string(&raw mut buf_0 as *mut ::core::ffi::c_char),
                        ),
                    },
                },
            };
        }
        if !sh_sign.sign_name.is_null() {
            let c2rust_fresh28 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh28 as isize) = key_value_pair {
                key: cstr_as_string(b"sign_name\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(sh_sign.sign_name),
                    },
                },
            };
        }
        let mut hls: [C2Rust_Unnamed_28; 5] = [
            C2Rust_Unnamed_28 {
                name: b"sign_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                val: sh_sign.hl_id,
            },
            C2Rust_Unnamed_28 {
                name: b"number_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                val: sh_sign.number_hl_id,
            },
            C2Rust_Unnamed_28 {
                name: b"line_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                val: sh_sign.line_hl_id,
            },
            C2Rust_Unnamed_28 {
                name: b"cursorline_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                val: sh_sign.cursorline_hl_id,
            },
            C2Rust_Unnamed_28 {
                name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                val: 0 as ::core::ffi::c_int,
            },
        ];
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !hls[j as usize].name.is_null() {
            if hls[j as usize].val != 0 {
                let c2rust_fresh29 = (*dict).size;
                (*dict).size = (*dict).size.wrapping_add(1);
                *(*dict).items.offset(c2rust_fresh29 as isize) = key_value_pair {
                    key: cstr_as_string(hls[j as usize].name),
                    value: hl_group_name(hls[j as usize].val, hl_name),
                };
            }
            j += 1;
        }
        priority = sh_sign.priority as int32_t;
    }
    if priority != -1 as int32_t {
        let c2rust_fresh30 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh30 as isize) = key_value_pair {
            key: cstr_as_string(b"priority\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_14 {
                    integer: priority as Integer,
                },
            },
        };
    }
}
pub unsafe extern "C" fn decor_type_flags(mut decor: DecorInline) -> uint16_t {
    if decor.ext {
        let mut type_flags: uint16_t = kExtmarkNone as ::core::ffi::c_int as uint16_t;
        let mut vt: *mut DecorVirtText = decor.data.ext.vt;
        while !vt.is_null() {
            type_flags = (type_flags as ::core::ffi::c_int
                | if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                    kExtmarkVirtLines as ::core::ffi::c_int
                } else {
                    kExtmarkVirtText as ::core::ffi::c_int
                }) as uint16_t;
            vt = (*vt).next;
        }
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(idx as isize);
            type_flags = (type_flags as ::core::ffi::c_int
                | if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
                    kExtmarkSign as ::core::ffi::c_int
                } else {
                    kExtmarkHighlight as ::core::ffi::c_int
                }) as uint16_t;
            idx = (*sh).next;
        }
        return type_flags;
    } else {
        return (if decor.data.hl.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0
        {
            kExtmarkSign as ::core::ffi::c_int
        } else {
            kExtmarkHighlight as ::core::ffi::c_int
        }) as uint16_t;
    };
}
pub unsafe extern "C" fn hl_group_name(mut hl_id: ::core::ffi::c_int, mut hl_name: bool) -> Object {
    if hl_name {
        return object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_14 {
                string: cstr_as_string(syn_id2name(hl_id)),
            },
        };
    } else {
        return object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_14 {
                integer: hl_id as Integer,
            },
        };
    };
}
#[inline(always)]
unsafe extern "C" fn decor_redraw_col(
    mut wp: *mut win_T,
    mut col: ::core::ffi::c_int,
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut state: *mut DecorState,
    mut max_col_last: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if col <= (*state).col_last {
        return (*state).current;
    }
    return decor_redraw_col_impl(wp, col, win_col, hidden, state, max_col_last);
}
pub const SCL_NUM: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const MT_FLAG_END: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
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
pub const MT_FLAG_DECOR_MASK: ::core::ffi::c_int = MT_FLAG_DECOR_EXT
    | MT_FLAG_DECOR_HL
    | MT_FLAG_DECOR_SIGNTEXT
    | MT_FLAG_DECOR_SIGNHL
    | MT_FLAG_DECOR_VIRT_LINES
    | MT_FLAG_DECOR_VIRT_TEXT_INLINE;
#[inline]
unsafe extern "C" fn mt_end(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_END != 0;
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
unsafe extern "C" fn mt_decor_sign(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & (MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL) != 0;
}
#[inline]
unsafe extern "C" fn mt_conceal_lines(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_CONCEAL_LINES != 0;
}
#[inline]
unsafe extern "C" fn mt_decor(mut key: MTKey) -> DecorInline {
    return DecorInline {
        ext: key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0,
        data: key.decor_data,
    };
}
#[inline]
unsafe extern "C" fn mt_decor_virt(mut mark: MTKey) -> *mut DecorVirtText {
    return if mark.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0 {
        mark.decor_data.ext.vt
    } else {
        ::core::ptr::null_mut::<DecorVirtText>()
    };
}
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
