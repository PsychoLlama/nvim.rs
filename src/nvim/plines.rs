use crate::src::nvim::charset::{ptr2cells, vim_isprintc, vim_strsize};
use crate::src::nvim::decoration::{decor_conceal_line, decor_virt_lines};
use crate::src::nvim::diff::{diff_check_fill, diffopt_filler};
use crate::src::nvim::fold::{hasFolding, hasFoldingWin, lineFolded};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::{get_breakindent_win, tabstop_padding};
use crate::src::nvim::main::{
    breakat_flags, curwin, namespace_localscope, p_sel, State, VIsual, VIsual_active,
};
use crate::src::nvim::map::mh_get_uint32_t;
use crate::src::nvim::marktree::{
    marktree_itr_current, marktree_itr_get_filter, marktree_itr_next_filter,
};
use crate::src::nvim::mbyte::{
    utf8len_tab, utf_ptr2CharInfo_impl, utf_ptr2char, utfc_next_impl, utfc_ptr2len,
};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::option::get_showbreak_value;
use crate::src::nvim::r#move::{win_col_off, win_col_off2};
use crate::src::nvim::state::virtual_active;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CSType, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, CharInfo, CharSize, CharsizeArg,
    DecorExt, DecorHighlightInline, DecorInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative,
    GridView, Intersection, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_14, MetaFilter, MetaIndex, OptInt, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_11, StrCharInfo, Terminal,
    Timestamp, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig,
    WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T,
    buf_T, bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, disptick_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_10, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, foldinfo_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    mtnode_inner_s, mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t,
    qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T,
    scid_T, sctx_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T,
    synblock_T, synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7,
    u_header_uh_alt_prev as C2Rust_Unnamed_6, u_header_uh_next as C2Rust_Unnamed_9,
    u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    uintptr_t, undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S,
    winopt_T, wline_T, xfmark_T, QUEUE,
};
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
pub const kStlClickFuncRun: C2Rust_Unnamed_11 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_11 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_11 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_11 = 0;
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
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_12 = 2147483647;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kVTRepeatLinebreak: C2Rust_Unnamed_13 = 8;
pub const kVTLinesAbove: C2Rust_Unnamed_13 = 4;
pub const kVTHide: C2Rust_Unnamed_13 = 2;
pub const kVTIsLines: C2Rust_Unnamed_13 = 1;
pub const kMTMetaCount: MetaIndex = 5;
pub const kMTMetaConcealLines: MetaIndex = 4;
pub const kMTMetaSignText: MetaIndex = 3;
pub const kMTMetaSignHL: MetaIndex = 2;
pub const kMTMetaLines: MetaIndex = 1;
pub const kMTMetaInline: MetaIndex = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_15 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_15 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_15 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_15 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_15 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_15 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_15 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_15 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_15 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_15 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_15 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_15 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_15 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_15 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_15 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_15 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_15 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_15 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_15 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kInvalidByteCells: C2Rust_Unnamed_16 = 4;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kCharsizeFast: C2Rust_Unnamed_17 = 1;
pub const kCharsizeRegular: C2Rust_Unnamed_17 = 0;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> bool {
    return mh_get_uint32_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline(always)]
unsafe extern "C" fn lt(mut a: pos_T, mut b: pos_T) -> bool {
    if a.lnum != b.lnum {
        return a.lnum < b.lnum;
    } else if a.col != b.col {
        return a.col < b.col;
    } else {
        return a.coladd < b.coladd;
    };
}
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
#[inline(always)]
unsafe extern "C" fn ltoreq(mut a: pos_T, mut b: pos_T) -> bool {
    return lt(a, b) as ::core::ffi::c_int != 0 || equalpos(a, b) as ::core::ffi::c_int != 0;
}
#[inline]
unsafe extern "C" fn ns_in_win(mut ns_id: uint32_t, mut wp: *mut win_T) -> bool {
    if !set_has_uint32_t(namespace_localscope.ptr(), ns_id) {
        return true_0 != 0;
    }
    return set_has_uint32_t(&raw mut (*wp).w_ns_set, ns_id);
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
#[inline(always)]
unsafe extern "C" fn vim_isbreak(mut c: ::core::ffi::c_int) -> bool {
    return (*breakat_flags.ptr())[c as uint8_t as usize] != 0;
}
pub const kMTFilterSelect: uint32_t = -1 as ::core::ffi::c_int as uint32_t;
pub const MT_FLAG_INVALID: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 6 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_EXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const MT_FLAG_RIGHT_GRAVITY: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mt_right(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_RIGHT_GRAVITY != 0;
}
#[inline]
unsafe extern "C" fn mt_invalid(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_INVALID != 0;
}
#[inline]
unsafe extern "C" fn mt_decor(mut key: MTKey) -> DecorInline {
    return DecorInline {
        ext: key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0,
        data: key.decor_data,
    };
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2CharInfo(p_in: *const ::core::ffi::c_char) -> CharInfo {
    let p: *const uint8_t = p_in as *const uint8_t;
    let first: uint8_t = *p;
    if (first as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return CharInfo {
            value: first as int32_t,
            len: 1 as ::core::ffi::c_int,
        };
    } else {
        let mut len: ::core::ffi::c_int =
            (*utf8len_tab.ptr())[first as usize] as ::core::ffi::c_int;
        let code_point: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        if code_point < 0 as int32_t {
            len = 1 as ::core::ffi::c_int;
        }
        return CharInfo {
            value: code_point,
            len: len,
        };
    };
}
#[inline(always)]
unsafe extern "C" fn utfc_next(mut cur: StrCharInfo) -> StrCharInfo {
    let mut next: *mut uint8_t = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
    if ((*next as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
        as ::core::ffi::c_long
        != 0
    {
        return StrCharInfo {
            ptr: next as *mut ::core::ffi::c_char,
            chr: CharInfo {
                value: *next as int32_t,
                len: 1 as ::core::ffi::c_int,
            },
        };
    }
    return utfc_next_impl(cur);
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2StrCharInfo(mut ptr: *mut ::core::ffi::c_char) -> StrCharInfo {
    return StrCharInfo {
        ptr: ptr,
        chr: utf_ptr2CharInfo(ptr),
    };
}
#[inline(always)]
unsafe extern "C" fn win_charsize(
    mut cstype: CSType,
    mut vcol: ::core::ffi::c_int,
    mut ptr: *mut ::core::ffi::c_char,
    mut chr: int32_t,
    mut csarg: *mut CharsizeArg,
) -> CharSize {
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return charsize_fast(csarg, ptr, vcol as colnr_T, chr);
    } else {
        return charsize_regular(csarg, ptr, vcol as colnr_T, chr);
    };
}
#[inline(always)]
unsafe extern "C" fn win_linetabsize(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
    mut len: colnr_T,
) -> ::core::ffi::c_int {
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let cstype: CSType = init_charsize_arg(&raw mut csarg, wp, lnum, line);
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return linesize_fast(&raw mut csarg, 0 as ::core::ffi::c_int, len);
    } else {
        return linesize_regular(&raw mut csarg, 0 as ::core::ffi::c_int, len);
    };
}
#[no_mangle]
pub unsafe extern "C" fn win_chartabsize(
    mut wp: *mut win_T,
    mut p: *mut ::core::ffi::c_char,
    mut col: colnr_T,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    if *p as ::core::ffi::c_int == TAB
        && ((*wp).w_onebuf_opt.wo_list == 0 || (*wp).w_p_lcs_chars.tab1 != 0)
    {
        return tabstop_padding(col, (*buf).b_p_ts, (*buf).b_p_vts_array);
    }
    return ptr2cells(p);
}
#[no_mangle]
pub unsafe extern "C" fn linetabsize_col(
    mut startvcol: ::core::ffi::c_int,
    mut s: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let cstype: CSType = init_charsize_arg(&raw mut csarg, curwin.get(), 0 as linenr_T, s);
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return linesize_fast(&raw mut csarg, startvcol, MAXCOL as ::core::ffi::c_int);
    } else {
        return linesize_regular(&raw mut csarg, startvcol, MAXCOL as ::core::ffi::c_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn linetabsize(mut wp: *mut win_T, mut lnum: linenr_T) -> ::core::ffi::c_int {
    return win_linetabsize(
        wp,
        lnum,
        ml_get_buf((*wp).w_buffer, lnum),
        MAXCOL as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn linetabsize_eol(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    return linetabsize(wp, lnum)
        + (if (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.eol != NUL as schar_T {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
}
static inline_filter: GlobalCell<[uint32_t; 5]> = GlobalCell::new([kMTFilterSelect, 0, 0, 0, 0]);
#[no_mangle]
pub unsafe extern "C" fn init_charsize_arg(
    mut csarg: *mut CharsizeArg,
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
) -> CSType {
    (*csarg).win = wp;
    (*csarg).line = line;
    (*csarg).max_head_vcol = 0 as ::core::ffi::c_int;
    (*csarg).cur_text_width_left = 0 as ::core::ffi::c_int;
    (*csarg).cur_text_width_right = 0 as ::core::ffi::c_int;
    (*csarg).virt_row = -1 as ::core::ffi::c_int;
    (*csarg).indent_width = INT_MIN;
    (*csarg).use_tabstop = (*wp).w_onebuf_opt.wo_list == 0 || (*wp).w_p_lcs_chars.tab1 != 0;
    if lnum > 0 as linenr_T {
        if marktree_itr_get_filter(
            &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
            lnum as int32_t - 1 as int32_t,
            0 as ::core::ffi::c_int,
            lnum as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (inline_filter.ptr() as *const _) as MetaFilter,
            &raw mut (*csarg).iter as *mut MarkTreeIter,
        ) {
            (*csarg).virt_row = (lnum - 1 as linenr_T) as ::core::ffi::c_int;
        }
    }
    if (*csarg).virt_row >= 0 as ::core::ffi::c_int
        || (*wp).w_onebuf_opt.wo_wrap != 0
            && ((*wp).w_onebuf_opt.wo_lbr != 0
                || (*wp).w_onebuf_opt.wo_bri != 0
                || *get_showbreak_value(wp) as ::core::ffi::c_int != NUL)
    {
        return kCharsizeRegular as ::core::ffi::c_int != 0;
    } else {
        return kCharsizeFast as ::core::ffi::c_int != 0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn charsize_regular(
    mut csarg: *mut CharsizeArg,
    cur: *mut ::core::ffi::c_char,
    vcol: colnr_T,
    cur_char: int32_t,
) -> CharSize {
    (*csarg).cur_text_width_left = 0 as ::core::ffi::c_int;
    (*csarg).cur_text_width_right = 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = (*csarg).win;
    let mut buf: *mut buf_T = (*wp).w_buffer;
    let mut line: *mut ::core::ffi::c_char = (*csarg).line;
    let use_tabstop: bool =
        cur_char == TAB as int32_t && (*csarg).use_tabstop as ::core::ffi::c_int != 0;
    let mut mb_added: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut has_lcs_eol: bool =
        (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.eol != NUL as schar_T;
    let mut size: ::core::ffi::c_int = 0;
    let mut is_doublewidth: ::core::ffi::c_int = false_0;
    if use_tabstop {
        size = tabstop_padding(vcol, (*buf).b_p_ts, (*buf).b_p_vts_array);
    } else if *cur as ::core::ffi::c_int == NUL {
        size = if has_lcs_eol as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    } else if cur_char < 0 as int32_t {
        size = kInvalidByteCells as ::core::ffi::c_int;
    } else {
        size = ptr2cells(cur);
        is_doublewidth =
            (size == 2 as ::core::ffi::c_int && cur_char >= 0x80 as int32_t) as ::core::ffi::c_int;
    }
    if (*csarg).virt_row >= 0 as ::core::ffi::c_int {
        let mut tab_size: ::core::ffi::c_int = size;
        let mut col: ::core::ffi::c_int = cur.offset_from(line) as ::core::ffi::c_int;
        loop {
            let mut mark: MTKey = marktree_itr_current(&raw mut (*csarg).iter as *mut MarkTreeIter);
            if mark.pos.row != (*csarg).virt_row as int32_t || mark.pos.col > col as int32_t {
                break;
            }
            if mark.pos.col == col as int32_t {
                if !mt_invalid(mark) && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0 {
                    let mut decor: DecorInline = mt_decor(mark);
                    let mut vt: *mut DecorVirtText = if decor.ext as ::core::ffi::c_int != 0 {
                        decor.data.ext.vt
                    } else {
                        ::core::ptr::null_mut::<DecorVirtText>()
                    };
                    while !vt.is_null() {
                        if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int == 0
                            && (*vt).pos as ::core::ffi::c_uint
                                == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if mt_right(mark) {
                                (*csarg).cur_text_width_right += (*vt).width;
                            } else {
                                (*csarg).cur_text_width_left += (*vt).width;
                            }
                            size += (*vt).width;
                            if use_tabstop {
                                size -= tab_size;
                                tab_size = tabstop_padding(
                                    vcol + size as colnr_T,
                                    (*buf).b_p_ts,
                                    (*buf).b_p_vts_array,
                                );
                                size += tab_size;
                            }
                        }
                        vt = (*vt).next;
                    }
                }
            }
            marktree_itr_next_filter(
                &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
                &raw mut (*csarg).iter as *mut MarkTreeIter,
                (*csarg).virt_row + 1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (inline_filter.ptr() as *const _) as MetaFilter,
            );
        }
    }
    if is_doublewidth != 0
        && (*wp).w_onebuf_opt.wo_wrap != 0
        && in_win_border(wp, vcol + size as colnr_T - 2 as colnr_T) as ::core::ffi::c_int != 0
    {
        size += 1;
        mb_added = 1 as ::core::ffi::c_int;
    }
    let sbr: *mut ::core::ffi::c_char = get_showbreak_value(wp);
    let mut head: ::core::ffi::c_int = mb_added;
    if size > 0 as ::core::ffi::c_int
        && (*wp).w_onebuf_opt.wo_wrap != 0
        && (*sbr as ::core::ffi::c_int != NUL || (*wp).w_onebuf_opt.wo_bri != 0)
    {
        let mut col_off_prev: ::core::ffi::c_int = win_col_off(wp);
        let mut width2: ::core::ffi::c_int = (*wp).w_view_width - col_off_prev + win_col_off2(wp);
        let mut wcol: colnr_T = vcol + col_off_prev as colnr_T;
        let mut max_head_vcol: colnr_T = (*csarg).max_head_vcol as colnr_T;
        let mut added: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut head_prev: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if wcol >= (*wp).w_view_width {
            wcol -= (*wp).w_view_width;
            col_off_prev = (*wp).w_view_width - width2;
            if wcol >= width2 && width2 > 0 as ::core::ffi::c_int {
                wcol %= width2;
            }
            head_prev = (*csarg).indent_width;
            if head_prev == INT_MIN {
                head_prev = 0 as ::core::ffi::c_int;
                if *sbr as ::core::ffi::c_int != NUL {
                    head_prev += vim_strsize(sbr);
                }
                if (*wp).w_onebuf_opt.wo_bri != 0 {
                    head_prev += get_breakindent_win(wp, line);
                }
                (*csarg).indent_width = head_prev;
            }
            if wcol < head_prev {
                head_prev -= wcol as ::core::ffi::c_int;
                wcol += head_prev;
                added += head_prev;
                if max_head_vcol <= 0 as ::core::ffi::c_int || vcol < max_head_vcol {
                    head += head_prev;
                }
            } else {
                head_prev = 0 as ::core::ffi::c_int;
            }
            wcol += col_off_prev;
        }
        if wcol as ::core::ffi::c_int + size > (*wp).w_view_width {
            let mut head_mid: ::core::ffi::c_int = (*csarg).indent_width;
            if head_mid == INT_MIN {
                head_mid = 0 as ::core::ffi::c_int;
                if *sbr as ::core::ffi::c_int != NUL {
                    head_mid += vim_strsize(sbr);
                }
                if (*wp).w_onebuf_opt.wo_bri != 0 {
                    head_mid += get_breakindent_win(wp, line);
                }
                (*csarg).indent_width = head_mid;
            }
            if head_mid > 0 as ::core::ffi::c_int {
                let mut prev_rem: ::core::ffi::c_int =
                    (*wp).w_view_width - wcol as ::core::ffi::c_int;
                let mut width: ::core::ffi::c_int = width2 - head_mid;
                if width <= 0 as ::core::ffi::c_int {
                    width = 1 as ::core::ffi::c_int;
                }
                let mut cnt: ::core::ffi::c_int =
                    (size - prev_rem + width - 1 as ::core::ffi::c_int) / width;
                added += cnt * head_mid;
                if max_head_vcol == 0 as ::core::ffi::c_int
                    || vcol as ::core::ffi::c_int + size + added < max_head_vcol
                {
                    head += cnt * head_mid;
                } else if width2 > 0 as ::core::ffi::c_int
                    && max_head_vcol > vcol as ::core::ffi::c_int + head_prev + prev_rem
                {
                    head += (max_head_vcol as ::core::ffi::c_int
                        - (vcol as ::core::ffi::c_int + head_prev + prev_rem)
                        + width2
                        - 1 as ::core::ffi::c_int)
                        / width2
                        * head_mid;
                } else if max_head_vcol < 0 as ::core::ffi::c_int {
                    let mut off: ::core::ffi::c_int =
                        mb_added + virt_text_cursor_off(csarg, *cur as ::core::ffi::c_int == NUL);
                    if off >= prev_rem {
                        if size > off {
                            head += (1 as ::core::ffi::c_int + (off - prev_rem) / width) * head_mid;
                        } else {
                            head += (off - prev_rem + width - 1 as ::core::ffi::c_int) / width
                                * head_mid;
                        }
                    }
                }
            }
        }
        size += added;
    }
    let mut need_lbr: bool = false_0 != 0;
    if (*wp).w_onebuf_opt.wo_lbr != 0
        && (*wp).w_onebuf_opt.wo_wrap != 0
        && (*wp).w_view_width != 0 as ::core::ffi::c_int
        && vim_isbreak(
            *cur.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
        ) as ::core::ffi::c_int
            != 0
        && !vim_isbreak(
            *cur.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
        )
    {
        let mut t: *mut ::core::ffi::c_char = (*csarg).line;
        while vim_isbreak(
            *t.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
        ) {
            t = t.offset(1);
        }
        need_lbr = cur >= t;
    }
    if need_lbr {
        let mut s: *mut ::core::ffi::c_char = cur;
        let mut numberextra: ::core::ffi::c_int = win_col_off(wp);
        let mut col_adj: colnr_T = size as colnr_T - 1 as colnr_T;
        let mut colmax: colnr_T = (*wp).w_view_width as colnr_T - numberextra as colnr_T - col_adj;
        if vcol >= colmax {
            colmax += col_adj;
            let mut n: ::core::ffi::c_int = colmax as ::core::ffi::c_int + win_col_off2(wp);
            if n > 0 as ::core::ffi::c_int {
                colmax += (((vcol - colmax) / n as colnr_T + 1 as colnr_T) * n as colnr_T - col_adj)
                    as ::core::ffi::c_int;
            }
        }
        let mut vcol2: colnr_T = vcol;
        loop {
            let mut ps: *mut ::core::ffi::c_char = s;
            s = s.offset(utfc_ptr2len(s) as isize);
            let mut c: ::core::ffi::c_int = *s as uint8_t as ::core::ffi::c_int;
            if !(c != NUL
                && (vim_isbreak(c) as ::core::ffi::c_int != 0
                    || vcol2 == vcol
                    || !vim_isbreak(*ps as uint8_t as ::core::ffi::c_int)))
            {
                break;
            }
            vcol2 += win_chartabsize(wp, s, vcol2);
            if vcol2 < colmax {
                continue;
            }
            size = (colmax - vcol + col_adj) as ::core::ffi::c_int;
            break;
        }
    }
    return CharSize {
        width: size,
        head: head,
    };
}
#[inline(always)]
unsafe extern "C" fn charsize_fast_impl(
    wp: *mut win_T,
    mut cur: *const ::core::ffi::c_char,
    mut use_tabstop: bool,
    vcol: colnr_T,
    cur_char: int32_t,
) -> CharSize {
    if cur_char == TAB as int32_t && use_tabstop as ::core::ffi::c_int != 0 {
        return CharSize {
            width: tabstop_padding(
                vcol,
                (*(*wp).w_buffer).b_p_ts,
                (*(*wp).w_buffer).b_p_vts_array,
            ),
            head: 0,
        };
    } else {
        let mut width: ::core::ffi::c_int = 0;
        if cur_char < 0 as int32_t {
            width = kInvalidByteCells as ::core::ffi::c_int;
        } else {
            width = ptr2cells(cur);
        }
        if width == 2 as ::core::ffi::c_int
            && cur_char >= 0x80 as int32_t
            && (*wp).w_onebuf_opt.wo_wrap != 0
            && in_win_border(wp, vcol) as ::core::ffi::c_int != 0
        {
            return CharSize {
                width: 3 as ::core::ffi::c_int,
                head: 1 as ::core::ffi::c_int,
            };
        } else {
            return CharSize {
                width: width,
                head: 0,
            };
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn charsize_fast(
    mut csarg: *mut CharsizeArg,
    mut cur: *const ::core::ffi::c_char,
    mut vcol: colnr_T,
    mut cur_char: int32_t,
) -> CharSize {
    return charsize_fast_impl((*csarg).win, cur, (*csarg).use_tabstop, vcol, cur_char);
}
#[no_mangle]
pub unsafe extern "C" fn charsize_nowrap(
    mut buf: *mut buf_T,
    mut cur: *const ::core::ffi::c_char,
    mut use_tabstop: bool,
    mut vcol: colnr_T,
    mut cur_char: int32_t,
) -> ::core::ffi::c_int {
    if cur_char == TAB as int32_t && use_tabstop as ::core::ffi::c_int != 0 {
        return tabstop_padding(vcol, (*buf).b_p_ts, (*buf).b_p_vts_array);
    } else if cur_char < 0 as int32_t {
        return kInvalidByteCells as ::core::ffi::c_int;
    } else {
        return ptr2cells(cur);
    };
}
unsafe extern "C" fn in_win_border(mut wp: *mut win_T, mut vcol: colnr_T) -> bool {
    if (*wp).w_view_width == 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    if vcol < width1 - 1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    if vcol == width1 - 1 as ::core::ffi::c_int {
        return true_0 != 0;
    }
    let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
    if width2 <= 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    return (vcol as ::core::ffi::c_int - width1) % width2 == width2 - 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn linesize_regular(
    csarg: *mut CharsizeArg,
    mut vcol_arg: ::core::ffi::c_int,
    len: colnr_T,
) -> ::core::ffi::c_int {
    let line: *mut ::core::ffi::c_char = (*csarg).line;
    let mut vcol: int64_t = vcol_arg as int64_t;
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    while ci.ptr.offset_from(line) < len as isize && *ci.ptr as ::core::ffi::c_int != NUL {
        vcol += charsize_regular(csarg, ci.ptr, vcol_arg as colnr_T, ci.chr.value).width as int64_t;
        ci = utfc_next(ci);
        if vcol > MAXCOL as ::core::ffi::c_int as int64_t {
            vcol_arg = MAXCOL as ::core::ffi::c_int;
            break;
        } else {
            vcol_arg = vcol as ::core::ffi::c_int;
        }
    }
    if len == MAXCOL as ::core::ffi::c_int
        && (*csarg).virt_row >= 0 as ::core::ffi::c_int
        && *ci.ptr as ::core::ffi::c_int == NUL
    {
        let mut head: ::core::ffi::c_int =
            charsize_regular(csarg, ci.ptr, vcol_arg as colnr_T, ci.chr.value).head;
        vcol += ((*csarg).cur_text_width_left + (*csarg).cur_text_width_right + head) as int64_t;
        vcol_arg = if vcol > MAXCOL as ::core::ffi::c_int as int64_t {
            MAXCOL as ::core::ffi::c_int
        } else {
            vcol as ::core::ffi::c_int
        };
    }
    return vcol_arg;
}
#[no_mangle]
pub unsafe extern "C" fn linesize_fast(
    csarg: *const CharsizeArg,
    mut vcol_arg: ::core::ffi::c_int,
    len: colnr_T,
) -> ::core::ffi::c_int {
    let wp: *mut win_T = (*csarg).win;
    let use_tabstop: bool = (*csarg).use_tabstop;
    let line: *mut ::core::ffi::c_char = (*csarg).line;
    let mut vcol: int64_t = vcol_arg as int64_t;
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    while ci.ptr.offset_from(line) < len as isize && *ci.ptr as ::core::ffi::c_int != NUL {
        vcol += charsize_fast_impl(wp, ci.ptr, use_tabstop, vcol_arg as colnr_T, ci.chr.value).width
            as int64_t;
        ci = utfc_next(ci);
        if vcol > MAXCOL as ::core::ffi::c_int as int64_t {
            vcol_arg = MAXCOL as ::core::ffi::c_int;
            break;
        } else {
            vcol_arg = vcol as ::core::ffi::c_int;
        }
    }
    return vcol_arg;
}
unsafe extern "C" fn virt_text_cursor_off(
    mut csarg: *const CharsizeArg,
    mut on_NUL: bool,
) -> ::core::ffi::c_int {
    let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !on_NUL || State.get() & MODE_NORMAL as ::core::ffi::c_int == 0 {
        off += (*csarg).cur_text_width_left;
    }
    if !on_NUL && State.get() & MODE_NORMAL as ::core::ffi::c_int != 0 {
        off += (*csarg).cur_text_width_right;
    }
    return off;
}
#[no_mangle]
pub unsafe extern "C" fn getvcol(
    mut wp: *mut win_T,
    mut pos: *mut pos_T,
    mut start: *mut colnr_T,
    mut cursor: *mut colnr_T,
    mut end: *mut colnr_T,
) {
    let line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, (*pos).lnum);
    let end_col: colnr_T = (*pos).col;
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let mut on_NUL: bool = false_0 != 0;
    let cstype: CSType = init_charsize_arg(&raw mut csarg, wp, (*pos).lnum, line);
    csarg.max_head_vcol = -1 as ::core::ffi::c_int;
    let mut vcol: colnr_T = 0 as colnr_T;
    let mut char_size: CharSize = CharSize { width: 0, head: 0 };
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        let use_tabstop: bool = csarg.use_tabstop;
        loop {
            if *ci.ptr as ::core::ffi::c_int == NUL {
                char_size = CharSize {
                    width: 1 as ::core::ffi::c_int,
                    head: 0,
                };
                break;
            } else {
                char_size = charsize_fast_impl(wp, ci.ptr, use_tabstop, vcol, ci.chr.value);
                let next: StrCharInfo = utfc_next(ci);
                if next.ptr.offset_from(line) > end_col as isize {
                    break;
                }
                ci = next;
                vcol += char_size.width;
            }
        }
    } else {
        loop {
            char_size = charsize_regular(&raw mut csarg, ci.ptr, vcol, ci.chr.value);
            if *ci.ptr as ::core::ffi::c_int == NUL {
                char_size.width = 1 as ::core::ffi::c_int
                    + csarg.cur_text_width_left
                    + csarg.cur_text_width_right;
                on_NUL = true_0 != 0;
                break;
            } else {
                let next_0: StrCharInfo = utfc_next(ci);
                if next_0.ptr.offset_from(line) > end_col as isize {
                    break;
                }
                ci = next_0;
                vcol += char_size.width;
            }
        }
    }
    if *ci.ptr as ::core::ffi::c_int == NUL
        && end_col < MAXCOL as ::core::ffi::c_int
        && end_col as isize > ci.ptr.offset_from(line)
    {
        (*pos).col = ci.ptr.offset_from(line) as colnr_T;
    }
    let mut head: ::core::ffi::c_int = char_size.head;
    let mut incr: ::core::ffi::c_int = char_size.width;
    if !start.is_null() {
        *start = (vcol as ::core::ffi::c_int + head) as colnr_T;
    }
    if !end.is_null() {
        *end = (vcol as ::core::ffi::c_int + incr - 1 as ::core::ffi::c_int) as colnr_T;
    }
    if !cursor.is_null() {
        if ci.chr.value == TAB as int32_t
            && State.get() & MODE_NORMAL as ::core::ffi::c_int != 0
            && (*wp).w_onebuf_opt.wo_list == 0
            && !virtual_active(wp)
            && !(VIsual_active.get() as ::core::ffi::c_int != 0
                && (*p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                    || ltoreq(*pos, VIsual.get()) as ::core::ffi::c_int != 0))
        {
            *cursor = (vcol as ::core::ffi::c_int + incr - 1 as ::core::ffi::c_int) as colnr_T;
        } else {
            vcol += virt_text_cursor_off(&raw mut csarg, on_NUL);
            *cursor = (vcol as ::core::ffi::c_int + head) as colnr_T;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn getvcol_nolist(mut posp: *mut pos_T) -> colnr_T {
    let mut list_save: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_list;
    let mut vcol: colnr_T = 0;
    (*curwin.get()).w_onebuf_opt.wo_list = false_0;
    if (*posp).coladd != 0 {
        getvvcol(
            curwin.get(),
            posp,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
        );
    } else {
        getvcol(
            curwin.get(),
            posp,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
        );
    }
    (*curwin.get()).w_onebuf_opt.wo_list = list_save;
    return vcol;
}
#[no_mangle]
pub unsafe extern "C" fn getvvcol(
    mut wp: *mut win_T,
    mut pos: *mut pos_T,
    mut start: *mut colnr_T,
    mut cursor: *mut colnr_T,
    mut end: *mut colnr_T,
) {
    let mut col: colnr_T = 0;
    if virtual_active(wp) {
        getvcol(
            wp,
            pos,
            &raw mut col,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        let mut coladd: colnr_T = (*pos).coladd;
        let mut endadd: colnr_T = 0 as colnr_T;
        let mut ptr: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, (*pos).lnum);
        if (*pos).col < ml_get_buf_len((*wp).w_buffer, (*pos).lnum) {
            let mut c: ::core::ffi::c_int = utf_ptr2char(ptr.offset((*pos).col as isize));
            if c != TAB && vim_isprintc(c) as ::core::ffi::c_int != 0 {
                endadd = ptr2cells(ptr.offset((*pos).col as isize)) - 1 as ::core::ffi::c_int;
                if coladd > endadd {
                    endadd = 0 as ::core::ffi::c_int as colnr_T;
                } else {
                    coladd = 0 as ::core::ffi::c_int as colnr_T;
                }
            }
        }
        col += coladd;
        if !start.is_null() {
            *start = col;
        }
        if !cursor.is_null() {
            *cursor = col;
        }
        if !end.is_null() {
            *end = col + endadd;
        }
    } else {
        getvcol(wp, pos, start, cursor, end);
    };
}
#[no_mangle]
pub unsafe extern "C" fn getvcols(
    mut wp: *mut win_T,
    mut pos1: *mut pos_T,
    mut pos2: *mut pos_T,
    mut left: *mut colnr_T,
    mut right: *mut colnr_T,
) {
    let mut from1: colnr_T = 0;
    let mut from2: colnr_T = 0;
    let mut to1: colnr_T = 0;
    let mut to2: colnr_T = 0;
    if lt(*pos1, *pos2) {
        getvvcol(
            wp,
            pos1,
            &raw mut from1,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut to1,
        );
        getvvcol(
            wp,
            pos2,
            &raw mut from2,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut to2,
        );
    } else {
        getvvcol(
            wp,
            pos2,
            &raw mut from1,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut to1,
        );
        getvvcol(
            wp,
            pos1,
            &raw mut from2,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut to2,
        );
    }
    if from2 < from1 {
        *left = from2;
    } else {
        *left = from1;
    }
    if to2 > to1 {
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
            && from2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int >= to1
        {
            *right = (from2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
        } else {
            *right = to2;
        }
    } else {
        *right = to1;
    };
}
#[no_mangle]
pub unsafe extern "C" fn win_may_fill(mut wp: *mut win_T) -> bool {
    return (*wp).w_onebuf_opt.wo_diff != 0 && diffopt_filler() as ::core::ffi::c_int != 0
        || buf_meta_total((*wp).w_buffer, kMTMetaLines) != 0;
}
#[no_mangle]
pub unsafe extern "C" fn win_get_fill(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut virt_lines: ::core::ffi::c_int = decor_virt_lines(
        wp,
        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        lnum as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<VirtLines>(),
        true_0 != 0,
    );
    if diffopt_filler() {
        let mut n: ::core::ffi::c_int = diff_check_fill(wp, lnum);
        if n > 0 as ::core::ffi::c_int {
            return virt_lines + n;
        }
    }
    return virt_lines;
}
#[no_mangle]
pub unsafe extern "C" fn plines_win(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut limit_winheight: bool,
) -> ::core::ffi::c_int {
    return plines_win_nofill(wp, lnum, limit_winheight) + win_get_fill(wp, lnum);
}
#[no_mangle]
pub unsafe extern "C" fn plines_win_nofill(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut limit_winheight: bool,
) -> ::core::ffi::c_int {
    if decor_conceal_line(
        wp,
        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        false_0 != 0,
    ) {
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).w_onebuf_opt.wo_wrap == 0 {
        return 1 as ::core::ffi::c_int;
    }
    if (*wp).w_view_width == 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    if lineFolded(wp, lnum) {
        return 1 as ::core::ffi::c_int;
    }
    let lines: ::core::ffi::c_int = plines_win_nofold(wp, lnum);
    if limit_winheight as ::core::ffi::c_int != 0 && lines > (*wp).w_view_height {
        return (*wp).w_view_height;
    }
    return lines;
}
#[no_mangle]
pub unsafe extern "C" fn plines_win_nofold(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut s: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let cstype: CSType = init_charsize_arg(&raw mut csarg, wp, lnum, s);
    if *s as ::core::ffi::c_int == NUL && csarg.virt_row < 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    let mut col: int64_t = 0;
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        col = linesize_fast(
            &raw mut csarg,
            0 as ::core::ffi::c_int,
            MAXCOL as ::core::ffi::c_int,
        ) as int64_t;
    } else {
        col = linesize_regular(
            &raw mut csarg,
            0 as ::core::ffi::c_int,
            MAXCOL as ::core::ffi::c_int,
        ) as int64_t;
    }
    if (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.eol != NUL as schar_T {
        col += 1 as int64_t;
    }
    let mut width: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    if width <= 0 as ::core::ffi::c_int {
        return 32000 as ::core::ffi::c_int;
    }
    if col <= width as int64_t {
        return 1 as ::core::ffi::c_int;
    }
    col -= width as int64_t;
    width += win_col_off2(wp);
    let lines: int64_t =
        (col + (width - 1 as ::core::ffi::c_int) as int64_t) / width as int64_t + 1 as int64_t;
    return if lines > 0 as int64_t && lines <= INT_MAX as int64_t {
        lines as ::core::ffi::c_int
    } else {
        INT_MAX
    };
}
#[no_mangle]
pub unsafe extern "C" fn plines_win_col(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut column: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    let mut lines: ::core::ffi::c_int = win_get_fill(wp, lnum);
    if (*wp).w_onebuf_opt.wo_wrap == 0 {
        return lines + 1 as ::core::ffi::c_int;
    }
    if (*wp).w_view_width == 0 as ::core::ffi::c_int {
        return lines + 1 as ::core::ffi::c_int;
    }
    let mut line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let cstype: CSType = init_charsize_arg(&raw mut csarg, wp, lnum, line);
    let mut vcol: colnr_T = 0 as colnr_T;
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        let use_tabstop: bool = csarg.use_tabstop;
        while *ci.ptr as ::core::ffi::c_int != NUL && {
            column -= 1;
            column >= 0 as ::core::ffi::c_long
        } {
            vcol += charsize_fast_impl(wp, ci.ptr, use_tabstop, vcol, ci.chr.value).width;
            ci = utfc_next(ci);
        }
    } else {
        while *ci.ptr as ::core::ffi::c_int != NUL && {
            column -= 1;
            column >= 0 as ::core::ffi::c_long
        } {
            vcol += charsize_regular(&raw mut csarg, ci.ptr, vcol, ci.chr.value).width;
            ci = utfc_next(ci);
        }
    }
    let mut col: colnr_T = vcol;
    if ci.chr.value == TAB as int32_t
        && State.get() & MODE_NORMAL as ::core::ffi::c_int != 0
        && csarg.use_tabstop as ::core::ffi::c_int != 0
    {
        col += win_charsize(
            cstype,
            col as ::core::ffi::c_int,
            ci.ptr,
            ci.chr.value,
            &raw mut csarg,
        )
        .width
            - 1 as ::core::ffi::c_int;
    }
    let mut width: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    if width <= 0 as ::core::ffi::c_int {
        return 9999 as ::core::ffi::c_int;
    }
    lines += 1 as ::core::ffi::c_int;
    if col > width {
        lines += (col as ::core::ffi::c_int - width) / (width + win_col_off2(wp))
            + 1 as ::core::ffi::c_int;
    }
    return lines;
}
#[no_mangle]
pub unsafe extern "C" fn plines_win_full(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    nextp: *mut linenr_T,
    foldedp: *mut bool,
    cache: bool,
    limit_winheight: bool,
) -> ::core::ffi::c_int {
    let mut folded: bool = hasFoldingWin(
        wp,
        lnum,
        &raw mut lnum,
        nextp,
        cache,
        ::core::ptr::null_mut::<foldinfo_T>(),
    );
    if !foldedp.is_null() {
        *foldedp = folded;
    }
    let mut filler_lines: ::core::ffi::c_int = if lnum == (*wp).w_topline {
        (*wp).w_topfill
    } else {
        win_get_fill(wp, lnum)
    };
    if decor_conceal_line(
        wp,
        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        false_0 != 0,
    ) {
        return filler_lines;
    }
    return (if folded as ::core::ffi::c_int != 0 {
        1 as ::core::ffi::c_int
    } else {
        plines_win_nofill(wp, lnum, limit_winheight)
    }) + filler_lines;
}
#[no_mangle]
pub unsafe extern "C" fn plines_m_win(
    mut wp: *mut win_T,
    mut first: linenr_T,
    mut last: linenr_T,
    mut max: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while first <= last && count < max {
        let mut next: linenr_T = first;
        count += plines_win_full(
            wp,
            first,
            &raw mut next,
            ::core::ptr::null_mut::<bool>(),
            false_0 != 0,
            false_0 != 0,
        );
        first = next + 1 as linenr_T;
    }
    if first == (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T {
        count += win_get_fill(wp, first);
    }
    return if max < count { max } else { count };
}
#[no_mangle]
pub unsafe extern "C" fn plines_m_win_fill(
    mut wp: *mut win_T,
    mut first: linenr_T,
    mut last: linenr_T,
) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = last as ::core::ffi::c_int - first as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        + decor_virt_lines(
            wp,
            first as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            last as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<VirtLines>(),
            false_0 != 0,
        );
    if diffopt_filler() {
        let mut lnum: ::core::ffi::c_int = first as ::core::ffi::c_int;
        while lnum as linenr_T <= last {
            let mut n: ::core::ffi::c_int = diff_check_fill(wp, lnum as linenr_T);
            count += if n > 0 as ::core::ffi::c_int {
                n
            } else {
                0 as ::core::ffi::c_int
            };
            lnum += 1;
        }
    }
    return if count > 0 as ::core::ffi::c_int {
        count
    } else {
        0 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn win_text_height(
    wp: *mut win_T,
    start_lnum: linenr_T,
    start_vcol: int64_t,
    end_lnum: *mut linenr_T,
    end_vcol: *mut int64_t,
    fill: *mut int64_t,
    max: int64_t,
) -> int64_t {
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
    width1 = if width1 > 0 as ::core::ffi::c_int {
        width1
    } else {
        0 as ::core::ffi::c_int
    };
    width2 = if width2 > 0 as ::core::ffi::c_int {
        width2
    } else {
        0 as ::core::ffi::c_int
    };
    let mut height_sum_fill: int64_t = 0 as int64_t;
    let mut height_cur_nofill: int64_t = 0 as int64_t;
    let mut height_sum_nofill: int64_t = 0 as int64_t;
    let mut lnum: linenr_T = start_lnum;
    let mut cur_lnum: linenr_T = lnum;
    let mut cur_folded: bool = false_0 != 0;
    if start_vcol >= 0 as int64_t {
        let mut lnum_next: linenr_T = lnum;
        cur_folded = hasFolding(wp, lnum, &raw mut lnum, &raw mut lnum_next);
        height_cur_nofill = plines_win_nofill(wp, lnum, false_0 != 0) as int64_t;
        height_sum_nofill += height_cur_nofill;
        let row_off: int64_t =
            if start_vcol < width1 as int64_t || width2 <= 0 as ::core::ffi::c_int {
                0 as int64_t
            } else {
                1 as int64_t + (start_vcol - width1 as int64_t) / width2 as int64_t
            };
        height_sum_nofill -= if row_off < height_cur_nofill {
            row_off
        } else {
            height_cur_nofill
        };
        lnum = lnum_next + 1 as linenr_T;
    }
    while lnum <= *end_lnum && height_sum_nofill + height_sum_fill < max {
        let mut lnum_next_0: linenr_T = lnum;
        cur_folded = hasFolding(wp, lnum, &raw mut lnum, &raw mut lnum_next_0);
        height_sum_fill += win_get_fill(wp, lnum) as int64_t;
        height_cur_nofill = plines_win_nofill(wp, lnum, false_0 != 0) as int64_t;
        height_sum_nofill += height_cur_nofill;
        cur_lnum = lnum;
        lnum = lnum_next_0 + 1 as linenr_T;
    }
    let mut vcol_end: int64_t = *end_vcol;
    let mut use_vcol: bool = vcol_end >= 0 as int64_t && lnum > *end_lnum;
    if use_vcol {
        height_sum_nofill -= height_cur_nofill;
        let row_off_0: int64_t = if vcol_end == 0 as int64_t {
            0 as int64_t
        } else if vcol_end <= width1 as int64_t || width2 <= 0 as ::core::ffi::c_int {
            1 as int64_t
        } else {
            1 as int64_t
                + (vcol_end - width1 as int64_t + width2 as int64_t - 1 as int64_t)
                    / width2 as int64_t
        };
        height_sum_nofill += if row_off_0 < height_cur_nofill {
            row_off_0
        } else {
            height_cur_nofill
        };
    }
    if cur_folded {
        vcol_end = 0 as int64_t;
    } else {
        let mut linesize: ::core::ffi::c_int = linetabsize_eol(wp, cur_lnum);
        vcol_end = if (if use_vcol as ::core::ffi::c_int != 0 {
            vcol_end
        } else {
            9223372036854775807 as int64_t
        }) < linesize as int64_t
        {
            if use_vcol as ::core::ffi::c_int != 0 {
                vcol_end
            } else {
                9223372036854775807 as int64_t
            }
        } else {
            linesize as int64_t
        };
    }
    let mut overflow: int64_t = height_sum_nofill + height_sum_fill - max;
    if overflow > 0 as int64_t && width2 > 0 as ::core::ffi::c_int && vcol_end > width2 as int64_t {
        vcol_end -= (vcol_end - width1 as int64_t) % width2 as int64_t
            + (overflow - 1 as int64_t) * width2 as int64_t;
    }
    *end_lnum = cur_lnum;
    *end_vcol = vcol_end;
    if !fill.is_null() {
        *fill = height_sum_fill;
    }
    return height_sum_fill + height_sum_nofill;
}
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
