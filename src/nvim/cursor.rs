use crate::src::nvim::change::inserted_bytes;
use crate::src::nvim::drawscreen::redraw_later;
use crate::src::nvim::fold::{hasAnyFolding, hasFolding};

use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{curbuf, curwin, p_sel, restart_edit, State, VIsual, VIsual_active};
use crate::src::nvim::mark::mark_mb_adjustpos;
use crate::src::nvim::mbyte::{
    utf8len_tab, utf_head_off, utf_ptr2CharInfo_impl, utf_ptr2char, utfc_next_impl,
};
use crate::src::nvim::memline::{
    dec, inc, ml_get_buf, ml_get_buf_len, ml_get_buf_mut, ml_get_len, ml_replace,
};
use crate::src::nvim::memory::xmallocz;
use crate::src::nvim::option::{get_sidescrolloff_value, get_ve_flags};
use crate::src::nvim::os::libc::{__assert_fail, abort, memcpy, memset};
use crate::src::nvim::plines::{
    charsize_fast, charsize_regular, getvcol, getvvcol, init_charsize_arg, linetabsize,
    linetabsize_eol,
};
use crate::src::nvim::r#move::{
    changed_cline_bef_curs, set_valid_virtcol, validate_virtcol, win_col_off,
};
use crate::src::nvim::state::virtual_active;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CSType, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, CharInfo, CharSize, CharsizeArg,
    DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative,
    GridView, Intersection, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_13, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, StrCharInfo, Terminal, Timestamp,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T,
    bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, disptick_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S, qf_info_T,
    queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T,
    size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, uintptr_t, undo_object, varnumber_T, virt_line,
    visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed = 2147483647;
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
pub const kOptVeFlagOnemore: C2Rust_Unnamed_16 = 8;
pub const MODE_TERMINAL: C2Rust_Unnamed_15 = 128;
pub const MODE_INSERT: C2Rust_Unnamed_15 = 16;
pub const kCharsizeFast: C2Rust_Unnamed_17 = 1;
pub const kOptVeFlagAll: C2Rust_Unnamed_16 = 4;
pub const UPD_NOT_VALID: C2Rust_Unnamed_14 = 40;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_14 = 50;
pub const UPD_SOME_VALID: C2Rust_Unnamed_14 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_14 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_14 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_14 = 20;
pub const UPD_VALID: C2Rust_Unnamed_14 = 10;
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
pub const MODE_SELECT: C2Rust_Unnamed_15 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_15 = 32;
pub const MODE_CMDLINE: C2Rust_Unnamed_15 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_15 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_15 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kOptVeFlagNoneU: C2Rust_Unnamed_16 = 32;
pub const kOptVeFlagNone: C2Rust_Unnamed_16 = 16;
pub const kOptVeFlagInsert: C2Rust_Unnamed_16 = 6;
pub const kOptVeFlagBlock: C2Rust_Unnamed_16 = 5;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kCharsizeRegular: C2Rust_Unnamed_17 = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub unsafe extern "C" fn getviscol() -> ::core::ffi::c_int {
    let mut x: colnr_T = 0;
    getvvcol(
        curwin.get(),
        &raw mut (*curwin.get()).w_cursor,
        &raw mut x,
        ::core::ptr::null_mut::<colnr_T>(),
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return x;
}
pub unsafe extern "C" fn getviscol2(mut col: colnr_T, mut coladd: colnr_T) -> ::core::ffi::c_int {
    let mut x: colnr_T = 0;
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    pos.lnum = (*curwin.get()).w_cursor.lnum;
    pos.col = col;
    pos.coladd = coladd;
    getvvcol(
        curwin.get(),
        &raw mut pos,
        &raw mut x,
        ::core::ptr::null_mut::<colnr_T>(),
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return x;
}
pub unsafe extern "C" fn coladvance_force(mut wcol: colnr_T) -> ::core::ffi::c_int {
    let mut rc: ::core::ffi::c_int = coladvance2(
        curwin.get(),
        &raw mut (*curwin.get()).w_cursor,
        true_0 != 0,
        false_0 != 0,
        wcol,
    );
    if wcol == MAXCOL as ::core::ffi::c_int {
        (*curwin.get()).w_valid &= !VALID_VIRTCOL;
    } else {
        set_valid_virtcol(curwin.get(), wcol);
    }
    return rc;
}
#[no_mangle]
pub unsafe extern "C" fn coladvance(mut wp: *mut win_T, mut wcol: colnr_T) -> ::core::ffi::c_int {
    let mut rc: ::core::ffi::c_int = getvpos(wp, &raw mut (*wp).w_cursor, wcol);
    if wcol == MAXCOL as ::core::ffi::c_int || rc == FAIL {
        (*wp).w_valid &= !VALID_VIRTCOL;
    } else if *ml_get_buf((*wp).w_buffer, (*wp).w_cursor.lnum).offset((*wp).w_cursor.col as isize)
        as ::core::ffi::c_int
        != TAB
    {
        set_valid_virtcol(curwin.get(), wcol);
    }
    return rc;
}
unsafe extern "C" fn coladvance2(
    mut wp: *mut win_T,
    mut pos: *mut pos_T,
    mut addspaces: bool,
    mut finetune: bool,
    mut wcol_arg: colnr_T,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if wp == curwin.get() || !addspaces {
        } else {
            __assert_fail(
                b"wp == curwin || !addspaces\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/cursor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                95 as ::core::ffi::c_uint,
                b"int coladvance2(win_T *, pos_T *, _Bool, _Bool, colnr_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut wcol: colnr_T = wcol_arg;
    let mut idx: ::core::ffi::c_int = 0;
    let mut col: colnr_T = 0 as colnr_T;
    let mut head: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut one_more: ::core::ffi::c_int = (State.get() & MODE_INSERT as ::core::ffi::c_int != 0
        || State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0
        || restart_edit.get() != NUL
        || VIsual_active.get() as ::core::ffi::c_int != 0
            && *p_sel.get() as ::core::ffi::c_int != 'o' as ::core::ffi::c_int
        || get_ve_flags(wp) & kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && wcol < MAXCOL as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    let mut line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, (*pos).lnum);
    let mut linelen: ::core::ffi::c_int = ml_get_buf_len((*wp).w_buffer, (*pos).lnum);
    if wcol >= MAXCOL as ::core::ffi::c_int {
        idx = linelen - 1 as ::core::ffi::c_int + one_more;
        col = wcol;
        if (addspaces as ::core::ffi::c_int != 0 || finetune as ::core::ffi::c_int != 0)
            && !VIsual_active.get()
        {
            (*wp).w_curswant = (linetabsize(wp, (*pos).lnum) + one_more) as colnr_T;
            if (*wp).w_curswant > 0 as ::core::ffi::c_int {
                (*wp).w_curswant -= 1;
            }
        }
    } else {
        let mut width: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut csize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if finetune as ::core::ffi::c_int != 0
            && (*wp).w_onebuf_opt.wo_wrap != 0
            && (*wp).w_view_width != 0 as ::core::ffi::c_int
            && wcol >= width
            && width > 0 as ::core::ffi::c_int
        {
            csize = linetabsize_eol(wp, (*pos).lnum);
            if csize > 0 as ::core::ffi::c_int {
                csize -= 1;
            }
            if wcol as ::core::ffi::c_int / width > csize / width
                && (State.get() & MODE_INSERT as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                    || wcol > csize + 1 as ::core::ffi::c_int)
            {
                wcol = ((csize / width + 1 as ::core::ffi::c_int) * width - 1 as ::core::ffi::c_int)
                    as colnr_T;
            }
        }
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
                s: [C2Rust_Unnamed_13 { oldcol: 0, i: 0 }; 20],
                intersect_idx: 0,
                intersect_pos: MTPos { row: 0, col: 0 },
                intersect_pos_x: MTPos { row: 0, col: 0 },
            }; 1],
        };
        let mut cstype: CSType = init_charsize_arg(&raw mut csarg, wp, (*pos).lnum, line);
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
        col = 0 as ::core::ffi::c_int as colnr_T;
        while col <= wcol && *ci.ptr as ::core::ffi::c_int != NUL {
            let mut cs: CharSize = win_charsize(
                cstype,
                col as ::core::ffi::c_int,
                ci.ptr,
                ci.chr.value,
                &raw mut csarg,
            );
            csize = cs.width;
            head = cs.head;
            col += cs.width;
            ci = utfc_next(ci);
        }
        idx = ci.ptr.offset_from(line) as ::core::ffi::c_int;
        if col > wcol || !virtual_active(wp) && one_more == 0 as ::core::ffi::c_int {
            idx -= 1 as ::core::ffi::c_int;
            csize -= head;
            col -= csize;
        }
        if virtual_active(wp) as ::core::ffi::c_int != 0
            && addspaces as ::core::ffi::c_int != 0
            && wcol >= 0 as ::core::ffi::c_int
            && (col != wcol && col != wcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                || csize > 1 as ::core::ffi::c_int)
        {
            if *line.offset(idx as isize) as ::core::ffi::c_int == NUL {
                let mut correct: ::core::ffi::c_int =
                    wcol as ::core::ffi::c_int - col as ::core::ffi::c_int;
                let mut newline_size: size_t = 0;
                let (c2rust_result, c2rust_overflowed) =
                    (idx as i128).overflowing_add(correct as i128);
                let c2rust_result_narrow = c2rust_result as size_t;
                *&raw mut newline_size = c2rust_result_narrow;
                if c2rust_overflowed || c2rust_result_narrow as i128 != c2rust_result {
                    logmsg(
                        LOGLVL_ERR,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"coladvance2\0".as_ptr() as *const ::core::ffi::c_char,
                        178 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"STRICT_ADD overflow\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    abort();
                }
                let mut newline: *mut ::core::ffi::c_char =
                    xmallocz(newline_size) as *mut ::core::ffi::c_char;
                memcpy(
                    newline as *mut ::core::ffi::c_void,
                    line as *const ::core::ffi::c_void,
                    idx as size_t,
                );
                memset(
                    newline.offset(idx as isize) as *mut ::core::ffi::c_void,
                    ' ' as ::core::ffi::c_int,
                    correct as size_t,
                );
                ml_replace((*pos).lnum, newline, false_0 != 0);
                inserted_bytes((*pos).lnum, idx, 0 as ::core::ffi::c_int, correct);
                idx += correct;
                col = wcol;
            } else {
                let mut correct_0: ::core::ffi::c_int =
                    wcol as ::core::ffi::c_int - col as ::core::ffi::c_int - csize
                        + 1 as ::core::ffi::c_int;
                let mut newline_0: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                if -correct_0 > csize {
                    return FAIL;
                }
                let mut n: size_t = 0;
                let (c2rust_result_0, c2rust_overflowed_0) =
                    ((linelen - 1 as ::core::ffi::c_int) as i128).overflowing_add(csize as i128);
                let c2rust_result_narrow_0 = c2rust_result_0 as size_t;
                *&raw mut n = c2rust_result_narrow_0;
                if c2rust_overflowed_0 || c2rust_result_narrow_0 as i128 != c2rust_result_0 {
                    logmsg(
                        LOGLVL_ERR,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"coladvance2\0".as_ptr() as *const ::core::ffi::c_char,
                        197 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"STRICT_ADD overflow\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    abort();
                }
                newline_0 = xmallocz(n) as *mut ::core::ffi::c_char;
                memcpy(
                    newline_0 as *mut ::core::ffi::c_void,
                    line as *const ::core::ffi::c_void,
                    idx as size_t,
                );
                memset(
                    newline_0.offset(idx as isize) as *mut ::core::ffi::c_void,
                    ' ' as ::core::ffi::c_int,
                    csize as size_t,
                );
                let (c2rust_result_1, c2rust_overflowed_1) =
                    (linelen as i128).overflowing_sub(idx as i128);
                let c2rust_result_narrow_1 = c2rust_result_1 as size_t;
                *&raw mut n = c2rust_result_narrow_1;
                if c2rust_overflowed_1 || c2rust_result_narrow_1 as i128 != c2rust_result_1 {
                    logmsg(
                        LOGLVL_ERR,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"coladvance2\0".as_ptr() as *const ::core::ffi::c_char,
                        204 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"STRICT_SUB overflow\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    abort();
                }
                let (c2rust_result_2, c2rust_overflowed_2) =
                    (n as i128).overflowing_sub(1 as ::core::ffi::c_int as i128);
                let c2rust_result_narrow_2 = c2rust_result_2 as size_t;
                *&raw mut n = c2rust_result_narrow_2;
                if c2rust_overflowed_2 || c2rust_result_narrow_2 as i128 != c2rust_result_2 {
                    logmsg(
                        LOGLVL_ERR,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"coladvance2\0".as_ptr() as *const ::core::ffi::c_char,
                        205 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"STRICT_SUB overflow\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    abort();
                }
                memcpy(
                    newline_0.offset(idx as isize).offset(csize as isize)
                        as *mut ::core::ffi::c_void,
                    line.offset(idx as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    n,
                );
                ml_replace((*pos).lnum, newline_0, false_0 != 0);
                inserted_bytes((*pos).lnum, idx as colnr_T, 1 as ::core::ffi::c_int, csize);
                idx += csize - 1 as ::core::ffi::c_int + correct_0;
                col += correct_0;
            }
        }
    }
    (*pos).col = (if idx > 0 as ::core::ffi::c_int {
        idx
    } else {
        0 as ::core::ffi::c_int
    }) as colnr_T;
    (*pos).coladd = 0 as ::core::ffi::c_int as colnr_T;
    if finetune {
        if wcol == MAXCOL as ::core::ffi::c_int {
            if one_more == 0 {
                let mut scol: colnr_T = 0;
                let mut ecol: colnr_T = 0;
                getvcol(
                    wp,
                    pos,
                    &raw mut scol,
                    ::core::ptr::null_mut::<colnr_T>(),
                    &raw mut ecol,
                );
                (*pos).coladd = ecol - scol;
            }
        } else {
            let mut b: ::core::ffi::c_int = wcol - col;
            if b > 0 as ::core::ffi::c_int
                && b < MAXCOL as ::core::ffi::c_int - 2 as ::core::ffi::c_int * (*wp).w_view_width
            {
                (*pos).coladd = b as colnr_T;
            }
            col += b;
        }
    }
    mark_mb_adjustpos((*wp).w_buffer, pos);
    if wcol < 0 as ::core::ffi::c_int || col < wcol {
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn getvpos(
    mut wp: *mut win_T,
    mut pos: *mut pos_T,
    mut wcol: colnr_T,
) -> ::core::ffi::c_int {
    return coladvance2(wp, pos, false_0 != 0, virtual_active(wp), wcol);
}
pub unsafe extern "C" fn inc_cursor() -> ::core::ffi::c_int {
    return inc(&raw mut (*curwin.get()).w_cursor);
}
pub unsafe extern "C" fn dec_cursor() -> ::core::ffi::c_int {
    return dec(&raw mut (*curwin.get()).w_cursor);
}
pub unsafe extern "C" fn get_cursor_rel_lnum(mut wp: *mut win_T, mut lnum: linenr_T) -> linenr_T {
    let mut cursor: linenr_T = (*wp).w_cursor.lnum;
    if lnum == cursor || hasAnyFolding(wp) == 0 {
        return lnum - cursor;
    }
    let mut from_line: linenr_T = if lnum < cursor { lnum } else { cursor };
    let mut to_line: linenr_T = if lnum > cursor { lnum } else { cursor };
    let mut retval: linenr_T = 0 as linenr_T;
    while from_line < to_line {
        hasFolding(
            wp,
            from_line,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut from_line,
        );
        from_line += 1;
        retval += 1;
    }
    if from_line > to_line {
        retval -= 1;
    }
    return if lnum < cursor { -retval } else { retval };
}
pub unsafe extern "C" fn check_pos(mut buf: *mut buf_T, mut pos: *mut pos_T) {
    (*pos).lnum = if (*pos).lnum < (*buf).b_ml.ml_line_count {
        (*pos).lnum
    } else {
        (*buf).b_ml.ml_line_count
    };
    if (*pos).col > 0 as ::core::ffi::c_int {
        (*pos).col = if (*pos).col < ml_get_buf_len(buf, (*pos).lnum) {
            (*pos).col
        } else {
            ml_get_buf_len(buf, (*pos).lnum)
        };
    }
}
pub unsafe extern "C" fn check_cursor_lnum(mut win: *mut win_T) {
    let mut buf: *mut buf_T = (*win).w_buffer;
    if (*win).w_cursor.lnum > (*buf).b_ml.ml_line_count {
        if !hasFolding(
            win,
            (*buf).b_ml.ml_line_count,
            &raw mut (*win).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
        ) {
            (*win).w_cursor.lnum = (*buf).b_ml.ml_line_count;
        }
    }
    if (*win).w_cursor.lnum <= 0 as linenr_T {
        (*win).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
    }
}
pub unsafe extern "C" fn check_cursor_col(mut win: *mut win_T) {
    let mut oldcol: colnr_T = (*win).w_cursor.col;
    let mut oldcoladd: colnr_T = (*win).w_cursor.col + (*win).w_cursor.coladd;
    let mut cur_ve_flags: ::core::ffi::c_uint = get_ve_flags(win);
    let mut len: colnr_T = ml_get_buf_len((*win).w_buffer, (*win).w_cursor.lnum);
    if len == 0 as ::core::ffi::c_int {
        (*win).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    } else if (*win).w_cursor.col >= len {
        if State.get() & MODE_INSERT as ::core::ffi::c_int != 0
            || restart_edit.get() != 0
            || State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0
            || VIsual_active.get() as ::core::ffi::c_int != 0
                && *p_sel.get() as ::core::ffi::c_int != 'o' as ::core::ffi::c_int
            || cur_ve_flags & kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            || virtual_active(win) as ::core::ffi::c_int != 0
        {
            (*win).w_cursor.col = len;
        } else {
            (*win).w_cursor.col = (len as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
            mark_mb_adjustpos((*win).w_buffer, &raw mut (*win).w_cursor);
        }
    } else if (*win).w_cursor.col < 0 as ::core::ffi::c_int {
        (*win).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    if oldcol == MAXCOL as ::core::ffi::c_int {
        (*win).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    } else if cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint {
        if oldcoladd > (*win).w_cursor.col {
            (*win).w_cursor.coladd = oldcoladd - (*win).w_cursor.col;
            if ((*win).w_cursor.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) < len {
                '_c2rust_label: {
                    if (*win).w_cursor.coladd > 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"win->w_cursor.coladd > 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/cursor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            372 as ::core::ffi::c_uint,
                            b"void check_cursor_col(win_T *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                let mut cs: ::core::ffi::c_int = 0;
                let mut ce: ::core::ffi::c_int = 0;
                getvcol(
                    win,
                    &raw mut (*win).w_cursor,
                    &raw mut cs,
                    ::core::ptr::null_mut::<colnr_T>(),
                    &raw mut ce,
                );
                (*win).w_cursor.coladd = (if (*win).w_cursor.coladd < ce - cs {
                    (*win).w_cursor.coladd as ::core::ffi::c_int
                } else {
                    ce - cs
                }) as colnr_T;
            }
        } else {
            (*win).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn check_cursor(mut wp: *mut win_T) {
    check_cursor_lnum(wp);
    check_cursor_col(wp);
}
pub unsafe extern "C" fn check_visual_pos() {
    if (*VIsual.ptr()).lnum > (*curbuf.get()).b_ml.ml_line_count {
        (*VIsual.ptr()).lnum = (*curbuf.get()).b_ml.ml_line_count;
        (*VIsual.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
        (*VIsual.ptr()).coladd = 0 as ::core::ffi::c_int as colnr_T;
    } else {
        let mut len: ::core::ffi::c_int = ml_get_len((*VIsual.ptr()).lnum);
        if (*VIsual.ptr()).col > len {
            (*VIsual.ptr()).col = len as colnr_T;
            (*VIsual.ptr()).coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
    };
}
pub unsafe extern "C" fn adjust_cursor_col() {
    if (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
        && (!VIsual_active.get() || *p_sel.get() as ::core::ffi::c_int == 'o' as ::core::ffi::c_int)
        && gchar_cursor() == NUL
    {
        (*curwin.get()).w_cursor.col -= 1;
    }
}
pub unsafe extern "C" fn set_leftcol(mut leftcol: colnr_T) -> bool {
    if (*curwin.get()).w_leftcol == leftcol {
        return false_0 != 0;
    }
    (*curwin.get()).w_leftcol = leftcol;
    changed_cline_bef_curs(curwin.get());
    let mut lastcol: int64_t = ((*curwin.get()).w_leftcol as ::core::ffi::c_int
        + (*curwin.get()).w_view_width
        - win_col_off(curwin.get())
        - 1 as ::core::ffi::c_int) as int64_t;
    validate_virtcol(curwin.get());
    let mut retval: bool = false_0 != 0;
    let mut siso: int64_t = get_sidescrolloff_value(curwin.get());
    if (*curwin.get()).w_virtcol > (lastcol - siso) as colnr_T {
        retval = true_0 != 0;
        coladvance(curwin.get(), (lastcol - siso) as colnr_T);
    } else if ((*curwin.get()).w_virtcol as int64_t) < (*curwin.get()).w_leftcol as int64_t + siso {
        retval = true_0 != 0;
        coladvance(
            curwin.get(),
            ((*curwin.get()).w_leftcol as int64_t + siso) as colnr_T,
        );
    }
    let mut s: colnr_T = 0;
    let mut e: colnr_T = 0;
    getvvcol(
        curwin.get(),
        &raw mut (*curwin.get()).w_cursor,
        &raw mut s,
        ::core::ptr::null_mut::<colnr_T>(),
        &raw mut e,
    );
    if e > lastcol as colnr_T {
        retval = true_0 != 0;
        coladvance(curwin.get(), s - 1 as colnr_T);
    } else if s < (*curwin.get()).w_leftcol {
        retval = true_0 != 0;
        if coladvance(curwin.get(), e + 1 as colnr_T) == FAIL {
            (*curwin.get()).w_leftcol = s;
            changed_cline_bef_curs(curwin.get());
        }
    }
    if retval {
        (*curwin.get()).w_set_curswant = true_0;
    }
    redraw_later(curwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
    return retval;
}
pub unsafe extern "C" fn gchar_cursor() -> ::core::ffi::c_int {
    return utf_ptr2char(get_cursor_pos_ptr());
}
pub unsafe extern "C" fn char_before_cursor() -> ::core::ffi::c_int {
    if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let mut p: *mut ::core::ffi::c_char = line.offset((*curwin.get()).w_cursor.col as isize);
    let mut prev_len: ::core::ffi::c_int =
        utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize))) + 1 as ::core::ffi::c_int;
    return utf_ptr2char(p.offset(-(prev_len as isize)));
}
pub unsafe extern "C" fn pchar_cursor(mut c: ::core::ffi::c_char) {
    *ml_get_buf_mut(curbuf.get(), (*curwin.get()).w_cursor.lnum)
        .offset((*curwin.get()).w_cursor.col as isize) = c;
}
pub unsafe extern "C" fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char {
    return ml_get_buf(curbuf.get(), (*curwin.get()).w_cursor.lnum);
}
pub unsafe extern "C" fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char {
    return ml_get_buf(curbuf.get(), (*curwin.get()).w_cursor.lnum)
        .offset((*curwin.get()).w_cursor.col as isize);
}
pub unsafe extern "C" fn get_cursor_line_len() -> colnr_T {
    return ml_get_buf_len(curbuf.get(), (*curwin.get()).w_cursor.lnum);
}
pub unsafe extern "C" fn get_cursor_pos_len() -> colnr_T {
    return ml_get_buf_len(curbuf.get(), (*curwin.get()).w_cursor.lnum)
        - (*curwin.get()).w_cursor.col;
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
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
