use crate::src::nvim::cursor::{
    coladvance, dec_cursor, gchar_cursor, get_cursor_line_ptr, get_cursor_pos_ptr, inc_cursor,
};
use crate::src::nvim::drawscreen::{redraw_curbuf_later, showmode};
use crate::src::nvim::edit::oneleft;
use crate::src::nvim::eval::funcs::do_searchpair;
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::inindent;
use crate::src::nvim::main::{
    curbuf, curwin, p_cpo, p_para, p_sections, p_sel, p_ws, redraw_cmdline, VIsual, VIsual_active,
    VIsual_mode, VIsual_select_exclu_adj,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{utf_class, utf_head_off, utfc_ptr2len};
use crate::src::nvim::memline::{dec, decl, gchar_pos, inc, incl, ml_get, ml_get_len, ml_get_pos};
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::normal::unadjust_for_sel;
use crate::src::nvim::os::libc::snprintf;
use crate::src::nvim::r#move::adjust_skipcol;
use crate::src::nvim::search::{findmatch, findmatchlimit, linewhite};
use crate::src::nvim::strings::vim_strchr;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2,
    Direction, ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection,
    LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MotionType, OptInt, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal, Timestamp,
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
    mtnode_s, oparg_T, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T,
    synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, xfmark_T, QUEUE,
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
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_13 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_13 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_13 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_13 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_13 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_13 = 20;
pub const UPD_VALID: C2Rust_Unnamed_13 = 10;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const FM_SKIPCOMM: C2Rust_Unnamed_14 = 8;
pub const FM_BLOCKSTOP: C2Rust_Unnamed_14 = 4;
pub const FM_FORWARD: C2Rust_Unnamed_14 = 2;
pub const FM_BACKWARD: C2Rust_Unnamed_14 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
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
#[inline(always)]
unsafe extern "C" fn clearpos(mut a: *mut pos_T) {
    (*a).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*a).col = 0 as ::core::ffi::c_int as colnr_T;
    (*a).coladd = 0 as ::core::ffi::c_int as colnr_T;
}
pub const CPO_ENDOFSENT: ::core::ffi::c_int = 'J' as ::core::ffi::c_int;
pub const CPO_MATCHBSL: ::core::ffi::c_int = 'M' as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn findsent(
    mut dir: Direction,
    mut count: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut found_dot: bool = false;
    let mut startlnum: ::core::ffi::c_int = 0;
    let mut cpo_J: bool = false;
    let mut c: ::core::ffi::c_int = 0;
    let mut func: Option<unsafe extern "C" fn(*mut pos_T) -> ::core::ffi::c_int> = None;
    let mut noskip: bool = false_0 != 0;
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        func = Some(incl as unsafe extern "C" fn(*mut pos_T) -> ::core::ffi::c_int)
            as Option<unsafe extern "C" fn(*mut pos_T) -> ::core::ffi::c_int>;
    } else {
        func = Some(decl as unsafe extern "C" fn(*mut pos_T) -> ::core::ffi::c_int)
            as Option<unsafe extern "C" fn(*mut pos_T) -> ::core::ffi::c_int>;
    }
    loop {
        let c2rust_fresh0 = count;
        count = count - 1;
        if c2rust_fresh0 == 0 {
            break;
        }
        let prev_pos: pos_T = pos;
        '_found: {
            if gchar_pos(&raw mut pos) == NUL {
                while Some(func.expect("non-null function pointer"))
                    .expect("non-null function pointer")(&raw mut pos)
                    != -1 as ::core::ffi::c_int
                {
                    if gchar_pos(&raw mut pos) != NUL {
                        break;
                    }
                }
                if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                    break '_found;
                }
            } else if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                && pos.col == 0 as ::core::ffi::c_int
                && startPS(pos.lnum, NUL, false_0 != 0) as ::core::ffi::c_int != 0
            {
                if pos.lnum == (*curbuf.get()).b_ml.ml_line_count {
                    return FAIL;
                }
                pos.lnum += 1;
                break '_found;
            } else if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
                decl(&raw mut pos);
            }
            found_dot = false_0 != 0;
            loop {
                c = gchar_pos(&raw mut pos);
                if !(ascii_iswhite(c) as ::core::ffi::c_int != 0
                    || !vim_strchr(b".!?)]\"'\0".as_ptr() as *const ::core::ffi::c_char, c)
                        .is_null())
                {
                    break;
                }
                let mut tpos: pos_T = pos;
                if decl(&raw mut tpos) == -1 as ::core::ffi::c_int
                    || *ml_get(tpos.lnum) as ::core::ffi::c_int == NUL
                        && dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                {
                    break;
                }
                if found_dot {
                    break;
                }
                if !vim_strchr(b".!?\0".as_ptr() as *const ::core::ffi::c_char, c).is_null() {
                    found_dot = true_0 != 0;
                }
                if !vim_strchr(b")]\"'\0".as_ptr() as *const ::core::ffi::c_char, c).is_null()
                    && vim_strchr(
                        b".!?)]\"'\0".as_ptr() as *const ::core::ffi::c_char,
                        gchar_pos(&raw mut tpos),
                    )
                    .is_null()
                {
                    break;
                }
                decl(&raw mut pos);
            }
            startlnum = pos.lnum as ::core::ffi::c_int;
            cpo_J = !vim_strchr(p_cpo.get(), CPO_ENDOFSENT).is_null();
            loop {
                c = gchar_pos(&raw mut pos);
                if c == NUL
                    || pos.col == 0 as ::core::ffi::c_int
                        && startPS(pos.lnum, NUL, false_0 != 0) as ::core::ffi::c_int != 0
                {
                    if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int
                        && pos.lnum != startlnum as linenr_T
                    {
                        pos.lnum += 1;
                    }
                    break;
                } else {
                    if c == '.' as ::core::ffi::c_int
                        || c == '!' as ::core::ffi::c_int
                        || c == '?' as ::core::ffi::c_int
                    {
                        let mut tpos_0: pos_T = pos;
                        loop {
                            c = inc(&raw mut tpos_0);
                            if c == -1 as ::core::ffi::c_int {
                                break;
                            }
                            c = gchar_pos(&raw mut tpos_0);
                            if vim_strchr(b")]\"'\0".as_ptr() as *const ::core::ffi::c_char, c)
                                .is_null()
                            {
                                break;
                            }
                        }
                        if c == -1 as ::core::ffi::c_int
                            || !cpo_J
                                && (c == ' ' as ::core::ffi::c_int
                                    || c == '\t' as ::core::ffi::c_int)
                            || c == NUL
                            || cpo_J as ::core::ffi::c_int != 0
                                && (c == ' ' as ::core::ffi::c_int
                                    && inc(&raw mut tpos_0) >= 0 as ::core::ffi::c_int
                                    && gchar_pos(&raw mut tpos_0) == ' ' as ::core::ffi::c_int)
                        {
                            pos = tpos_0;
                            if gchar_pos(&raw mut pos) == NUL {
                                inc(&raw mut pos);
                            }
                            break;
                        }
                    }
                    if Some(func.expect("non-null function pointer"))
                        .expect("non-null function pointer")(&raw mut pos)
                        != -1 as ::core::ffi::c_int
                    {
                        continue;
                    }
                    if count != 0 {
                        return FAIL;
                    }
                    noskip = true_0 != 0;
                    break;
                }
            }
        }
        while !noskip && {
            c = gchar_pos(&raw mut pos);
            c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int
        } {
            if incl(&raw mut pos) == -1 as ::core::ffi::c_int {
                break;
            }
        }
        if !equalpos(prev_pos, pos) {
            continue;
        }
        if Some(func.expect("non-null function pointer")).expect("non-null function pointer")(
            &raw mut pos,
        ) == -1 as ::core::ffi::c_int
        {
            if count != 0 {
                return FAIL;
            }
            break;
        } else {
            count += 1;
        }
    }
    setpcmark();
    (*curwin.get()).w_cursor = pos;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn findpar(
    mut pincl: *mut bool,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut what: ::core::ffi::c_int,
    mut both: bool,
) -> bool {
    let mut first: bool = false;
    let mut fold_first: linenr_T = 0;
    let mut fold_last: linenr_T = 0;
    let mut fold_skipped: bool = false;
    let mut curr: linenr_T = (*curwin.get()).w_cursor.lnum;
    loop {
        let c2rust_fresh1 = count;
        count = count - 1;
        if c2rust_fresh1 == 0 {
            break;
        }
        let mut did_skip: bool = false_0 != 0;
        first = true_0 != 0;
        loop {
            if *ml_get(curr) as ::core::ffi::c_int != NUL {
                did_skip = true_0 != 0;
            }
            fold_skipped = false_0 != 0;
            if first as ::core::ffi::c_int != 0
                && hasFolding(curwin.get(), curr, &raw mut fold_first, &raw mut fold_last)
                    as ::core::ffi::c_int
                    != 0
            {
                curr = (if dir > 0 as ::core::ffi::c_int {
                    fold_last
                } else {
                    fold_first
                }) + dir as linenr_T;
                fold_skipped = true_0 != 0;
            }
            if !first
                && did_skip as ::core::ffi::c_int != 0
                && startPS(curr, what, both) as ::core::ffi::c_int != 0
            {
                break;
            }
            if fold_skipped {
                curr = (curr as ::core::ffi::c_int - dir) as linenr_T;
            }
            curr = (curr as ::core::ffi::c_int + dir) as linenr_T;
            if curr < 1 as linenr_T || curr > (*curbuf.get()).b_ml.ml_line_count {
                if count != 0 {
                    return false_0 != 0;
                }
                curr = (curr as ::core::ffi::c_int - dir) as linenr_T;
                break;
            } else {
                first = false_0 != 0;
            }
        }
    }
    setpcmark();
    if both as ::core::ffi::c_int != 0
        && *ml_get(curr) as ::core::ffi::c_int == '}' as ::core::ffi::c_int
    {
        curr += 1;
    }
    (*curwin.get()).w_cursor.lnum = curr;
    if curr == (*curbuf.get()).b_ml.ml_line_count
        && what != '}' as ::core::ffi::c_int
        && dir == FORWARD as ::core::ffi::c_int
    {
        let mut line: *mut ::core::ffi::c_char = ml_get(curr);
        (*curwin.get()).w_cursor.col = ml_get_len(curr);
        if (*curwin.get()).w_cursor.col != 0 as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.col -= 1;
            (*curwin.get()).w_cursor.col -=
                utf_head_off(line, line.offset((*curwin.get()).w_cursor.col as isize));
            *pincl = true_0 != 0;
        }
    } else {
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    return true_0 != 0;
}
unsafe extern "C" fn inmacro(
    mut opt: *mut ::core::ffi::c_char,
    mut s: *const ::core::ffi::c_char,
) -> bool {
    let mut macro_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    macro_0 = opt;
    while *macro_0.offset(0 as ::core::ffi::c_int as isize) != 0 {
        if (*macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            || *macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ' ' as ::core::ffi::c_int
                && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int))
            && (*macro_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                || (*macro_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *macro_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int)
                    && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == NUL
                        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int))
        {
            break;
        }
        macro_0 = macro_0.offset(1);
        if *macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            break;
        }
        macro_0 = macro_0.offset(1);
    }
    return *macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL;
}
#[no_mangle]
pub unsafe extern "C" fn startPS(
    mut lnum: linenr_T,
    mut para: ::core::ffi::c_int,
    mut both: bool,
) -> bool {
    let mut s: *mut ::core::ffi::c_char = ml_get(lnum);
    if *s as uint8_t as ::core::ffi::c_int == para
        || *s as ::core::ffi::c_int == '\u{c}' as ::core::ffi::c_int
        || both as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    if *s as ::core::ffi::c_int == '.' as ::core::ffi::c_int
        && (inmacro(p_sections.get(), s.offset(1 as ::core::ffi::c_int as isize))
            as ::core::ffi::c_int
            != 0
            || para == 0
                && inmacro(p_para.get(), s.offset(1 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int
                    != 0)
    {
        return true_0 != 0;
    }
    return false_0 != 0;
}
static cls_bigword: GlobalCell<bool> = GlobalCell::new(false);
unsafe extern "C" fn cls() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = gchar_cursor();
    if c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int || c == NUL {
        return 0 as ::core::ffi::c_int;
    }
    c = utf_class(c);
    if c != 0 as ::core::ffi::c_int && cls_bigword.get() as ::core::ffi::c_int != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn fwd_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut eol: bool,
) -> ::core::ffi::c_int {
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword.set(bigword);
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*curwin.get()).w_cursor.lnum,
        ) {
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        }
        let mut sclass: ::core::ffi::c_int = cls();
        let mut last_line: ::core::ffi::c_int = ((*curwin.get()).w_cursor.lnum
            == (*curbuf.get()).b_ml.ml_line_count)
            as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = inc_cursor();
        if i == -1 as ::core::ffi::c_int || i >= 1 as ::core::ffi::c_int && last_line != 0 {
            return FAIL;
        }
        if i >= 1 as ::core::ffi::c_int
            && eol as ::core::ffi::c_int != 0
            && count == 0 as ::core::ffi::c_int
        {
            return OK;
        }
        if sclass != 0 as ::core::ffi::c_int {
            while cls() == sclass {
                i = inc_cursor();
                if i == -1 as ::core::ffi::c_int
                    || i >= 1 as ::core::ffi::c_int
                        && eol as ::core::ffi::c_int != 0
                        && count == 0 as ::core::ffi::c_int
                {
                    return OK;
                }
            }
        }
        while cls() == 0 as ::core::ffi::c_int {
            if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                && *get_cursor_line_ptr() as ::core::ffi::c_int == NUL
            {
                break;
            }
            i = inc_cursor();
            if i == -1 as ::core::ffi::c_int
                || i >= 1 as ::core::ffi::c_int
                    && eol as ::core::ffi::c_int != 0
                    && count == 0 as ::core::ffi::c_int
            {
                return OK;
            }
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn bck_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut stop: bool,
) -> ::core::ffi::c_int {
    let mut sclass: ::core::ffi::c_int = 0;
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword.set(bigword);
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            &raw mut (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
        ) {
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        }
        sclass = cls();
        if dec_cursor() == -1 as ::core::ffi::c_int {
            return FAIL;
        }
        '_finished: {
            if !stop || sclass == cls() || sclass == 0 as ::core::ffi::c_int {
                while cls() == 0 as ::core::ffi::c_int {
                    if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                        && *ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL
                    {
                        break '_finished;
                    }
                    if dec_cursor() == -1 as ::core::ffi::c_int {
                        return OK;
                    }
                }
                if skip_chars(cls(), BACKWARD as ::core::ffi::c_int) {
                    return OK;
                }
            }
            inc_cursor();
        }
        stop = false_0 != 0;
    }
    adjust_skipcol();
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn end_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut stop: bool,
    mut empty: bool,
) -> ::core::ffi::c_int {
    let mut sclass: ::core::ffi::c_int = 0;
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword.set(bigword);
    if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && VIsual_active.get() as ::core::ffi::c_int != 0
        && VIsual_mode.get() == 'v' as ::core::ffi::c_int
        && VIsual_select_exclu_adj.get() as ::core::ffi::c_int != 0
    {
        unadjust_for_sel();
    }
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*curwin.get()).w_cursor.lnum,
        ) {
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        }
        sclass = cls();
        if inc_cursor() == -1 as ::core::ffi::c_int {
            return FAIL;
        }
        '_finished: {
            if cls() == sclass && sclass != 0 as ::core::ffi::c_int {
                if skip_chars(sclass, FORWARD as ::core::ffi::c_int) {
                    return FAIL;
                }
            } else if !stop || sclass == 0 as ::core::ffi::c_int {
                while cls() == 0 as ::core::ffi::c_int {
                    if empty as ::core::ffi::c_int != 0
                        && (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                        && *ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL
                    {
                        break '_finished;
                    }
                    if inc_cursor() == -1 as ::core::ffi::c_int {
                        return FAIL;
                    }
                }
                if skip_chars(cls(), FORWARD as ::core::ffi::c_int) {
                    return FAIL;
                }
            }
            dec_cursor();
        }
        stop = false_0 != 0;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn bckend_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut eol: bool,
) -> ::core::ffi::c_int {
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword.set(bigword);
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        let mut i: ::core::ffi::c_int = 0;
        let mut sclass: ::core::ffi::c_int = cls();
        i = dec_cursor();
        if i == -1 as ::core::ffi::c_int {
            return FAIL;
        }
        if eol as ::core::ffi::c_int != 0 && i == 1 as ::core::ffi::c_int {
            return OK;
        }
        if sclass != 0 as ::core::ffi::c_int {
            while cls() == sclass {
                i = dec_cursor();
                if i == -1 as ::core::ffi::c_int
                    || eol as ::core::ffi::c_int != 0 && i == 1 as ::core::ffi::c_int
                {
                    return OK;
                }
            }
        }
        while cls() == 0 as ::core::ffi::c_int {
            if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                && *ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL
            {
                break;
            }
            i = dec_cursor();
            if i == -1 as ::core::ffi::c_int
                || eol as ::core::ffi::c_int != 0 && i == 1 as ::core::ffi::c_int
            {
                return OK;
            }
        }
    }
    adjust_skipcol();
    return OK;
}
unsafe extern "C" fn skip_chars(
    mut cclass: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
) -> bool {
    while cls() == cclass {
        if (if dir == FORWARD as ::core::ffi::c_int {
            inc_cursor()
        } else {
            dec_cursor()
        }) == -1 as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn back_in_line() {
    let mut sclass: ::core::ffi::c_int = cls();
    while (*curwin.get()).w_cursor.col != 0 as ::core::ffi::c_int {
        dec_cursor();
        if cls() == sclass {
            continue;
        }
        inc_cursor();
        break;
    }
}
unsafe extern "C" fn find_first_blank(mut posp: *mut pos_T) {
    while decl(posp) != -1 as ::core::ffi::c_int {
        let mut c: ::core::ffi::c_int = gchar_pos(posp);
        if ascii_iswhite(c) {
            continue;
        }
        incl(posp);
        break;
    }
}
unsafe extern "C" fn findsent_forward(mut count: ::core::ffi::c_int, mut at_start_sent: bool) {
    loop {
        let c2rust_fresh3 = count;
        count = count - 1;
        if c2rust_fresh3 == 0 {
            break;
        }
        findsent(FORWARD, 1 as ::core::ffi::c_int);
        if at_start_sent {
            find_first_blank(&raw mut (*curwin.get()).w_cursor);
        }
        if count == 0 as ::core::ffi::c_int || at_start_sent as ::core::ffi::c_int != 0 {
            decl(&raw mut (*curwin.get()).w_cursor);
        }
        at_start_sent = !at_start_sent;
    }
}
#[no_mangle]
pub unsafe extern "C" fn current_word(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut bigword: bool,
) -> ::core::ffi::c_int {
    let mut start_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut inclusive: bool = true_0 != 0;
    let mut include_white: bool = false_0 != 0;
    cls_bigword.set(bigword);
    clearpos(&raw mut start_pos);
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && lt(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
    {
        dec_cursor();
    }
    if !VIsual_active.get()
        || equalpos((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0
    {
        back_in_line();
        start_pos = (*curwin.get()).w_cursor;
        if (cls() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int == include as ::core::ffi::c_int
        {
            if end_word(1 as ::core::ffi::c_int, bigword, true_0 != 0, true_0 != 0) == FAIL {
                return FAIL;
            }
        } else {
            fwd_word(1 as ::core::ffi::c_int, bigword, true_0 != 0);
            if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
                decl(&raw mut (*curwin.get()).w_cursor);
            } else {
                oneleft();
            }
            if include {
                include_white = true_0 != 0;
            }
        }
        if VIsual_active.get() {
            VIsual.set(start_pos);
            redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
        } else {
            (*oap).start = start_pos;
            (*oap).motion_type = kMTCharWise;
        }
        count -= 1;
    }
    while count > 0 as ::core::ffi::c_int {
        inclusive = true_0 != 0;
        if VIsual_active.get() as ::core::ffi::c_int != 0
            && lt((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0
        {
            if decl(&raw mut (*curwin.get()).w_cursor) == -1 as ::core::ffi::c_int {
                return FAIL;
            }
            if include as ::core::ffi::c_int
                != (cls() != 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            {
                if bck_word(1 as ::core::ffi::c_int, bigword, true_0 != 0) == FAIL {
                    return FAIL;
                }
            } else {
                if bckend_word(1 as ::core::ffi::c_int, bigword, true_0 != 0) == FAIL {
                    return FAIL;
                }
                incl(&raw mut (*curwin.get()).w_cursor);
            }
        } else {
            if incl(&raw mut (*curwin.get()).w_cursor) == -1 as ::core::ffi::c_int {
                return FAIL;
            }
            if include as ::core::ffi::c_int
                != (cls() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            {
                if fwd_word(1 as ::core::ffi::c_int, bigword, true_0 != 0) == FAIL
                    && count > 1 as ::core::ffi::c_int
                {
                    return FAIL;
                }
                if oneleft() == FAIL {
                    inclusive = false_0 != 0;
                }
            } else if end_word(1 as ::core::ffi::c_int, bigword, true_0 != 0, true_0 != 0) == FAIL {
                return FAIL;
            }
        }
        count -= 1;
    }
    if include_white as ::core::ffi::c_int != 0
        && (cls() != 0 as ::core::ffi::c_int
            || (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int && !inclusive)
    {
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = start_pos;
        if oneleft() == OK {
            back_in_line();
            if cls() == 0 as ::core::ffi::c_int
                && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
            {
                if VIsual_active.get() {
                    VIsual.set((*curwin.get()).w_cursor);
                } else {
                    (*oap).start = (*curwin.get()).w_cursor;
                }
            }
        }
        (*curwin.get()).w_cursor = pos;
    }
    if VIsual_active.get() {
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
            && inclusive as ::core::ffi::c_int != 0
            && ltoreq(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
        {
            inc_cursor();
        }
        if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
            VIsual_mode.set('v' as ::core::ffi::c_int);
            redraw_cmdline.set(true_0 != 0);
        }
    } else {
        (*oap).inclusive = inclusive;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn current_sent(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
) -> ::core::ffi::c_int {
    let mut start_blank: bool = false;
    let mut c: ::core::ffi::c_int = 0;
    let mut at_start_sent: bool = false;
    let mut ncount: ::core::ffi::c_int = 0;
    let mut start_pos: pos_T = (*curwin.get()).w_cursor;
    let mut pos: pos_T = start_pos;
    findsent(FORWARD, 1 as ::core::ffi::c_int);
    '_extend: {
        if !(VIsual_active.get() as ::core::ffi::c_int != 0 && !equalpos(start_pos, VIsual.get())) {
            loop {
                c = gchar_pos(&raw mut pos);
                if !ascii_iswhite(c) {
                    break;
                }
                incl(&raw mut pos);
            }
            if equalpos(pos, (*curwin.get()).w_cursor) {
                start_blank = true_0 != 0;
                find_first_blank(&raw mut start_pos);
            } else {
                start_blank = false_0 != 0;
                findsent(BACKWARD, 1 as ::core::ffi::c_int);
                start_pos = (*curwin.get()).w_cursor;
            }
            if include {
                ncount = count * 2 as ::core::ffi::c_int;
            } else {
                ncount = count;
                if start_blank {
                    ncount -= 1;
                }
            }
            if ncount > 0 as ::core::ffi::c_int {
                findsent_forward(ncount, true_0 != 0);
            } else {
                decl(&raw mut (*curwin.get()).w_cursor);
            }
            if include {
                if start_blank {
                    find_first_blank(&raw mut (*curwin.get()).w_cursor);
                    c = gchar_pos(&raw mut (*curwin.get()).w_cursor);
                    if ascii_iswhite(c) {
                        decl(&raw mut (*curwin.get()).w_cursor);
                    }
                } else {
                    c = gchar_cursor();
                    if !ascii_iswhite(c) as ::core::ffi::c_int != 0 {
                        find_first_blank(&raw mut start_pos);
                    }
                }
            }
            if VIsual_active.get() {
                if equalpos(start_pos, (*curwin.get()).w_cursor) {
                    break '_extend;
                } else {
                    if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                        (*curwin.get()).w_cursor.col += 1;
                    }
                    VIsual.set(start_pos);
                    VIsual_mode.set('v' as ::core::ffi::c_int);
                    redraw_cmdline.set(true_0 != 0);
                    redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
                }
            } else {
                if incl(&raw mut (*curwin.get()).w_cursor) == -1 as ::core::ffi::c_int {
                    (*oap).inclusive = true_0 != 0;
                } else {
                    (*oap).inclusive = false_0 != 0;
                }
                (*oap).start = start_pos;
                (*oap).motion_type = kMTCharWise;
            }
            return OK;
        }
    }
    if lt(start_pos, VIsual.get()) {
        at_start_sent = true_0 != 0;
        decl(&raw mut pos);
        while lt(pos, (*curwin.get()).w_cursor) {
            c = gchar_pos(&raw mut pos);
            if !ascii_iswhite(c) {
                at_start_sent = false_0 != 0;
                break;
            } else {
                incl(&raw mut pos);
            }
        }
        if !at_start_sent {
            findsent(BACKWARD, 1 as ::core::ffi::c_int);
            if equalpos((*curwin.get()).w_cursor, start_pos) {
                at_start_sent = true_0 != 0;
            } else {
                findsent(FORWARD, 1 as ::core::ffi::c_int);
            }
        }
        if include {
            count *= 2 as ::core::ffi::c_int;
        }
        loop {
            let c2rust_fresh2 = count;
            count = count - 1;
            if c2rust_fresh2 == 0 {
                break;
            }
            if at_start_sent {
                find_first_blank(&raw mut (*curwin.get()).w_cursor);
            }
            c = gchar_cursor();
            if !at_start_sent || !include && !ascii_iswhite(c) {
                findsent(BACKWARD, 1 as ::core::ffi::c_int);
            }
            at_start_sent = !at_start_sent;
        }
    } else {
        incl(&raw mut pos);
        at_start_sent = true_0 != 0;
        if !equalpos(pos, (*curwin.get()).w_cursor) {
            at_start_sent = false_0 != 0;
            while lt(pos, (*curwin.get()).w_cursor) {
                c = gchar_pos(&raw mut pos);
                if !ascii_iswhite(c) {
                    at_start_sent = true_0 != 0;
                    break;
                } else {
                    incl(&raw mut pos);
                }
            }
            if at_start_sent {
                findsent(BACKWARD, 1 as ::core::ffi::c_int);
            } else {
                (*curwin.get()).w_cursor = start_pos;
            }
        }
        if include {
            count *= 2 as ::core::ffi::c_int;
        }
        findsent_forward(count, at_start_sent);
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.col += 1;
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn current_block(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut what: ::core::ffi::c_int,
    mut other: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut start_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut end_pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut sol: bool = false_0 != 0;
    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
    let mut old_end: pos_T = (*curwin.get()).w_cursor;
    let mut old_start: pos_T = old_end;
    if !VIsual_active.get()
        || equalpos(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
    {
        setpcmark();
        if what == '{' as ::core::ffi::c_int {
            while inindent(1 as ::core::ffi::c_int) {
                if inc_cursor() != 0 as ::core::ffi::c_int {
                    break;
                }
            }
        }
        if gchar_cursor() == what {
            (*curwin.get()).w_cursor.col += 1;
        }
    } else if lt(VIsual.get(), (*curwin.get()).w_cursor) {
        old_start = VIsual.get();
        (*curwin.get()).w_cursor = VIsual.get();
    } else {
        old_end = VIsual.get();
    }
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(
        (if !vim_strchr(p_cpo.get(), CPO_MATCHBSL).is_null() {
            b"%M\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"%\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char,
    );
    pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
    if !pos.is_null() {
        loop {
            let c2rust_fresh4 = count;
            count = count - 1;
            if c2rust_fresh4 <= 0 as ::core::ffi::c_int {
                break;
            }
            pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
            if pos.is_null() {
                break;
            }
            (*curwin.get()).w_cursor = *pos;
            start_pos = *pos;
        }
    } else {
        loop {
            let c2rust_fresh5 = count;
            count = count - 1;
            if c2rust_fresh5 <= 0 as ::core::ffi::c_int {
                break;
            }
            pos = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                what,
                FM_FORWARD as ::core::ffi::c_int,
                0 as int64_t,
            );
            if pos.is_null() {
                break;
            }
            (*curwin.get()).w_cursor = *pos;
            start_pos = *pos;
        }
    }
    p_cpo.set(save_cpo);
    if pos.is_null() || {
        end_pos = findmatch(::core::ptr::null_mut::<oparg_T>(), other);
        end_pos.is_null()
    } {
        (*curwin.get()).w_cursor = old_pos;
        return FAIL;
    }
    (*curwin.get()).w_cursor = *end_pos;
    while !include {
        incl(&raw mut start_pos);
        sol = (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int;
        decl(&raw mut (*curwin.get()).w_cursor);
        while inindent(1 as ::core::ffi::c_int) {
            sol = true_0 != 0;
            if decl(&raw mut (*curwin.get()).w_cursor) != 0 as ::core::ffi::c_int {
                break;
            }
        }
        if equalpos(start_pos, *end_pos) as ::core::ffi::c_int != 0
            && VIsual_active.get() as ::core::ffi::c_int != 0
        {
            (*curwin.get()).w_cursor = old_pos;
            return FAIL;
        }
        if !(!lt(start_pos, old_start)
            && !lt(old_end, (*curwin.get()).w_cursor)
            && !equalpos(start_pos, (*curwin.get()).w_cursor)
            && VIsual_active.get() as ::core::ffi::c_int != 0)
        {
            break;
        }
        (*curwin.get()).w_cursor = old_start;
        decl(&raw mut (*curwin.get()).w_cursor);
        pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
        if pos.is_null() {
            (*curwin.get()).w_cursor = old_pos;
            return FAIL;
        }
        start_pos = *pos;
        (*curwin.get()).w_cursor = *pos;
        end_pos = findmatch(::core::ptr::null_mut::<oparg_T>(), other);
        if end_pos.is_null() {
            (*curwin.get()).w_cursor = old_pos;
            return FAIL;
        }
        (*curwin.get()).w_cursor = *end_pos;
    }
    if VIsual_active.get() {
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            inc(&raw mut (*curwin.get()).w_cursor);
        }
        if sol as ::core::ffi::c_int != 0 && gchar_cursor() != NUL {
            inc(&raw mut (*curwin.get()).w_cursor);
        }
        VIsual.set(start_pos);
        VIsual_mode.set('v' as ::core::ffi::c_int);
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
        showmode();
    } else {
        (*oap).start = start_pos;
        (*oap).motion_type = kMTCharWise;
        (*oap).inclusive = false_0 != 0;
        if sol {
            incl(&raw mut (*curwin.get()).w_cursor);
        } else if ltoreq(start_pos, (*curwin.get()).w_cursor) {
            (*oap).inclusive = true_0 != 0;
        } else {
            (*curwin.get()).w_cursor = start_pos;
        }
    }
    return OK;
}
unsafe extern "C" fn in_html_tag(mut end_tag: bool) -> bool {
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lc: ::core::ffi::c_int = NUL;
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    p = line.offset((*curwin.get()).w_cursor.col as isize);
    while p > line {
        if *p as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
            break;
        }
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
        if *p as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
            break;
        }
    }
    if *p as ::core::ffi::c_int != '<' as ::core::ffi::c_int {
        return false_0 != 0;
    }
    pos.lnum = (*curwin.get()).w_cursor.lnum;
    pos.col = p.offset_from(line) as colnr_T;
    p = p.offset(utfc_ptr2len(p) as isize);
    if end_tag {
        return *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int;
    }
    if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
        return false_0 != 0;
    }
    loop {
        if inc(&raw mut pos) < 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        let mut c: ::core::ffi::c_int = *ml_get_pos(&raw mut pos) as uint8_t as ::core::ffi::c_int;
        if c == '>' as ::core::ffi::c_int {
            break;
        }
        lc = c;
    }
    return lc != '/' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn current_tagblock(
    mut oap: *mut oparg_T,
    mut count_arg: ::core::ffi::c_int,
    mut include: bool,
) -> ::core::ffi::c_int {
    let mut start_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    let mut spat_len: size_t = 0;
    let mut spat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut epat_len: size_t = 0;
    let mut epat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut r: ::core::ffi::c_int = 0;
    let mut end_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut count: ::core::ffi::c_int = count_arg;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut do_include: bool = include;
    let mut save_p_ws: bool = p_ws.get() != 0;
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut is_inclusive: bool = true_0 != 0;
    p_ws.set(false_0);
    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
    let mut old_end: pos_T = (*curwin.get()).w_cursor;
    let mut old_start: pos_T = old_end;
    if !VIsual_active.get() || *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
        decl(&raw mut old_end);
    }
    if !VIsual_active.get()
        || equalpos(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
    {
        setpcmark();
        while inindent(1 as ::core::ffi::c_int) {
            if inc_cursor() != 0 as ::core::ffi::c_int {
                break;
            }
        }
        if in_html_tag(false_0 != 0) {
            while *get_cursor_pos_ptr() as ::core::ffi::c_int != '>' as ::core::ffi::c_int {
                if inc_cursor() < 0 as ::core::ffi::c_int {
                    break;
                }
            }
        } else if in_html_tag(true_0 != 0) {
            while *get_cursor_pos_ptr() as ::core::ffi::c_int != '<' as ::core::ffi::c_int {
                if dec_cursor() < 0 as ::core::ffi::c_int {
                    break;
                }
            }
            dec_cursor();
            old_end = (*curwin.get()).w_cursor;
        }
    } else if lt(VIsual.get(), (*curwin.get()).w_cursor) {
        old_start = VIsual.get();
        (*curwin.get()).w_cursor = VIsual.get();
    } else {
        old_end = VIsual.get();
    }
    '_theend: {
        loop {
            let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while n < count {
                if do_searchpair(
                    b"<[^ \t>/!]\\+\\%(\\_s\\_[^>]\\{-}[^/]>\\|$\\|\\_s\\=>\\)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                    b"</[^>]*>\0".as_ptr() as *const ::core::ffi::c_char,
                    BACKWARD as ::core::ffi::c_int,
                    ::core::ptr::null::<typval_T>(),
                    0 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<pos_T>(),
                    0 as linenr_T,
                    0 as int64_t,
                ) <= 0 as ::core::ffi::c_int
                {
                    (*curwin.get()).w_cursor = old_pos;
                    break '_theend;
                } else {
                    n += 1;
                }
            }
            start_pos = (*curwin.get()).w_cursor;
            inc_cursor();
            p = get_cursor_pos_ptr();
            cp = p;
            while *cp as ::core::ffi::c_int != NUL
                && *cp as ::core::ffi::c_int != '>' as ::core::ffi::c_int
                && !ascii_iswhite(*cp as ::core::ffi::c_int)
            {
                cp = cp.offset(utfc_ptr2len(cp) as isize);
            }
            len = cp.offset_from(p) as ::core::ffi::c_int;
            if len == 0 as ::core::ffi::c_int {
                (*curwin.get()).w_cursor = old_pos;
                break '_theend;
            } else {
                spat_len = (len as size_t).wrapping_add(39 as size_t);
                spat = xmalloc(spat_len) as *mut ::core::ffi::c_char;
                epat_len = (len as size_t).wrapping_add(9 as size_t);
                epat = xmalloc(epat_len) as *mut ::core::ffi::c_char;
                snprintf(
                    spat,
                    spat_len,
                    b"<%.*s\\>\\%%(\\_s\\_[^>]\\{-}\\_[^/]>\\|\\_s\\?>\\)\\c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    len,
                    p,
                );
                snprintf(
                    epat,
                    epat_len,
                    b"</%.*s>\\c\0".as_ptr() as *const ::core::ffi::c_char,
                    len,
                    p,
                );
                r = do_searchpair(
                    spat,
                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                    epat,
                    FORWARD as ::core::ffi::c_int,
                    ::core::ptr::null::<typval_T>(),
                    0 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<pos_T>(),
                    0 as linenr_T,
                    0 as int64_t,
                );
                xfree(spat as *mut ::core::ffi::c_void);
                xfree(epat as *mut ::core::ffi::c_void);
                if r < 1 as ::core::ffi::c_int
                    || lt((*curwin.get()).w_cursor, old_end) as ::core::ffi::c_int != 0
                {
                    count = 1 as ::core::ffi::c_int;
                    (*curwin.get()).w_cursor = start_pos;
                } else {
                    if do_include {
                        while *get_cursor_pos_ptr() as ::core::ffi::c_int
                            != '>' as ::core::ffi::c_int
                        {
                            if inc_cursor() < 0 as ::core::ffi::c_int {
                                break;
                            }
                        }
                    } else {
                        let mut c: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                        if *c as ::core::ffi::c_int == '<' as ::core::ffi::c_int
                            && !VIsual_active.get()
                            && (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                        {
                            is_inclusive = false_0 != 0;
                        } else if *c as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
                            dec_cursor();
                        }
                    }
                    end_pos = (*curwin.get()).w_cursor;
                    if do_include {
                        break;
                    }
                    let mut in_quotes: bool = false_0 != 0;
                    (*curwin.get()).w_cursor = start_pos;
                    while inc_cursor() >= 0 as ::core::ffi::c_int {
                        p = get_cursor_pos_ptr();
                        if *p as ::core::ffi::c_int == '>' as ::core::ffi::c_int && !in_quotes {
                            inc_cursor();
                            start_pos = (*curwin.get()).w_cursor;
                            break;
                        } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                            || *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                        {
                            in_quotes = !in_quotes;
                        }
                    }
                    (*curwin.get()).w_cursor = end_pos;
                    if !(VIsual_active.get() as ::core::ffi::c_int != 0
                        && equalpos(start_pos, old_start) as ::core::ffi::c_int != 0
                        && equalpos(end_pos, old_end) as ::core::ffi::c_int != 0)
                    {
                        break;
                    }
                    do_include = true_0 != 0;
                    (*curwin.get()).w_cursor = old_start;
                    count = count_arg;
                }
            }
        }
        if VIsual_active.get() {
            if lt(end_pos, start_pos) {
                (*curwin.get()).w_cursor = start_pos;
            } else if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                inc_cursor();
            }
            VIsual.set(start_pos);
            VIsual_mode.set('v' as ::core::ffi::c_int);
            redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
            showmode();
        } else {
            (*oap).start = start_pos;
            (*oap).motion_type = kMTCharWise;
            if lt(end_pos, start_pos) {
                (*curwin.get()).w_cursor = start_pos;
                (*oap).inclusive = false_0 != 0;
            } else {
                (*oap).inclusive = is_inclusive;
            }
        }
        retval = OK;
    }
    p_ws.set(save_p_ws as ::core::ffi::c_int);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn current_par(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut type_0: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut dir: ::core::ffi::c_int = 0;
    let mut retval: ::core::ffi::c_int = OK;
    let mut do_white: ::core::ffi::c_int = false_0;
    if type_0 == 'S' as ::core::ffi::c_int {
        return FAIL;
    }
    let mut start_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    '_extend: {
        if !(VIsual_active.get() as ::core::ffi::c_int != 0 && start_lnum != (*VIsual.ptr()).lnum) {
            let mut white_in_front: bool = linewhite(start_lnum);
            while start_lnum > 1 as linenr_T {
                if white_in_front {
                    if !linewhite(start_lnum - 1 as linenr_T) {
                        break;
                    }
                } else if linewhite(start_lnum - 1 as linenr_T) as ::core::ffi::c_int != 0
                    || startPS(start_lnum, 0 as ::core::ffi::c_int, false) as ::core::ffi::c_int
                        != 0
                {
                    break;
                }
                start_lnum -= 1;
            }
            let mut end_lnum: linenr_T = start_lnum;
            while end_lnum <= (*curbuf.get()).b_ml.ml_line_count
                && linewhite(end_lnum) as ::core::ffi::c_int != 0
            {
                end_lnum += 1;
            }
            end_lnum -= 1;
            let mut i_0: ::core::ffi::c_int = count;
            if !include && white_in_front as ::core::ffi::c_int != 0 {
                i_0 -= 1;
            }
            loop {
                let c2rust_fresh6 = i_0;
                i_0 = i_0 - 1;
                if c2rust_fresh6 == 0 {
                    break;
                }
                if end_lnum == (*curbuf.get()).b_ml.ml_line_count {
                    return FAIL;
                }
                if !include {
                    do_white = linewhite(end_lnum + 1 as linenr_T) as ::core::ffi::c_int;
                }
                if include as ::core::ffi::c_int != 0 || do_white == 0 {
                    end_lnum += 1;
                    while end_lnum < (*curbuf.get()).b_ml.ml_line_count
                        && !linewhite(end_lnum + 1 as linenr_T)
                        && !startPS(end_lnum + 1 as linenr_T, 0 as ::core::ffi::c_int, false)
                    {
                        end_lnum += 1;
                    }
                }
                if i_0 == 0 as ::core::ffi::c_int
                    && white_in_front as ::core::ffi::c_int != 0
                    && include as ::core::ffi::c_int != 0
                {
                    break;
                }
                if include as ::core::ffi::c_int != 0 || do_white != 0 {
                    while end_lnum < (*curbuf.get()).b_ml.ml_line_count
                        && linewhite(end_lnum + 1 as linenr_T) as ::core::ffi::c_int != 0
                    {
                        end_lnum += 1;
                    }
                }
            }
            if !white_in_front && !linewhite(end_lnum) && include as ::core::ffi::c_int != 0 {
                while start_lnum > 1 as linenr_T
                    && linewhite(start_lnum - 1 as linenr_T) as ::core::ffi::c_int != 0
                {
                    start_lnum -= 1;
                }
            }
            if VIsual_active.get() {
                if VIsual_mode.get() == 'V' as ::core::ffi::c_int
                    && start_lnum == (*curwin.get()).w_cursor.lnum
                {
                    break '_extend;
                } else {
                    if (*VIsual.ptr()).lnum != start_lnum {
                        (*VIsual.ptr()).lnum = start_lnum;
                        (*VIsual.ptr()).col = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    VIsual_mode.set('V' as ::core::ffi::c_int);
                    redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
                    showmode();
                }
            } else {
                (*oap).start.lnum = start_lnum;
                (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
                (*oap).motion_type = kMTLineWise;
            }
            (*curwin.get()).w_cursor.lnum = end_lnum;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            return OK;
        }
    }
    dir = if start_lnum < (*VIsual.ptr()).lnum {
        BACKWARD as ::core::ffi::c_int
    } else {
        FORWARD as ::core::ffi::c_int
    };
    let mut i: ::core::ffi::c_int = count;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if start_lnum
            == (if dir == BACKWARD as ::core::ffi::c_int {
                1 as linenr_T
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            })
        {
            retval = FAIL;
            break;
        } else {
            let mut prev_start_is_white: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut t: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while t < 2 as ::core::ffi::c_int {
                start_lnum = (start_lnum as ::core::ffi::c_int + dir) as linenr_T;
                let mut start_is_white: ::core::ffi::c_int =
                    linewhite(start_lnum) as ::core::ffi::c_int;
                if prev_start_is_white == start_is_white {
                    start_lnum = (start_lnum as ::core::ffi::c_int - dir) as linenr_T;
                    break;
                } else {
                    while start_lnum
                        != (if dir == BACKWARD as ::core::ffi::c_int {
                            1 as linenr_T
                        } else {
                            (*curbuf.get()).b_ml.ml_line_count
                        })
                    {
                        if start_is_white
                            != linewhite(start_lnum + dir as linenr_T) as ::core::ffi::c_int
                            || start_is_white == 0
                                && startPS(
                                    start_lnum
                                        + (if dir > 0 as ::core::ffi::c_int {
                                            1 as linenr_T
                                        } else {
                                            0 as linenr_T
                                        }),
                                    0 as ::core::ffi::c_int,
                                    false,
                                ) as ::core::ffi::c_int
                                    != 0
                        {
                            break;
                        }
                        start_lnum = (start_lnum as ::core::ffi::c_int + dir) as linenr_T;
                    }
                    if !include {
                        break;
                    }
                    if start_lnum
                        == (if dir == BACKWARD as ::core::ffi::c_int {
                            1 as linenr_T
                        } else {
                            (*curbuf.get()).b_ml.ml_line_count
                        })
                    {
                        break;
                    }
                    prev_start_is_white = start_is_white;
                    t += 1;
                }
            }
        }
    }
    (*curwin.get()).w_cursor.lnum = start_lnum;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    return retval;
}
unsafe extern "C" fn find_next_quote(
    mut line: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut quotechar: ::core::ffi::c_int,
    mut escape: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    loop {
        let mut c: ::core::ffi::c_int = *line.offset(col as isize) as uint8_t as ::core::ffi::c_int;
        if c == NUL {
            return -1 as ::core::ffi::c_int;
        } else {
            if !escape.is_null() && !vim_strchr(escape, c).is_null() {
                col += 1;
                if *line.offset(col as isize) as ::core::ffi::c_int == NUL {
                    return -1 as ::core::ffi::c_int;
                }
            } else if c == quotechar {
                break;
            }
            col += utfc_ptr2len(line.offset(col as isize));
        }
    }
    return col;
}
unsafe extern "C" fn find_prev_quote(
    mut line: *mut ::core::ffi::c_char,
    mut col_start: ::core::ffi::c_int,
    mut quotechar: ::core::ffi::c_int,
    mut escape: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    while col_start > 0 as ::core::ffi::c_int {
        col_start -= 1;
        col_start -= utf_head_off(line, line.offset(col_start as isize));
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !escape.is_null() {
            while col_start - n > 0 as ::core::ffi::c_int
                && !vim_strchr(
                    escape,
                    *line.offset((col_start - n - 1 as ::core::ffi::c_int) as isize) as uint8_t
                        as ::core::ffi::c_int,
                )
                .is_null()
            {
                n += 1;
            }
        }
        if n & 1 as ::core::ffi::c_int != 0 {
            col_start -= n;
        } else if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int == quotechar {
            break;
        }
    }
    return col_start;
}
#[no_mangle]
pub unsafe extern "C" fn current_quote(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut quotechar: ::core::ffi::c_int,
) -> bool {
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let mut col_end: ::core::ffi::c_int = 0;
    let mut col_start: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
    let mut inclusive: bool = false_0 != 0;
    let mut vis_empty: bool = true_0 != 0;
    let mut vis_bef_curs: bool = false_0 != 0;
    let mut did_exclusive_adj: bool = false_0 != 0;
    let mut inside_quotes: bool = false_0 != 0;
    let mut selected_quote: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut restore_vis_bef: bool = false_0 != 0;
    if VIsual_active.get() {
        if (*VIsual.ptr()).lnum != (*curwin.get()).w_cursor.lnum {
            return false_0 != 0;
        }
        vis_bef_curs = lt(VIsual.get(), (*curwin.get()).w_cursor);
        vis_empty = equalpos(VIsual.get(), (*curwin.get()).w_cursor);
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            if vis_bef_curs {
                dec_cursor();
                did_exclusive_adj = true_0 != 0;
            } else if !vis_empty {
                dec(VIsual.ptr());
                did_exclusive_adj = true_0 != 0;
            }
            vis_empty = equalpos(VIsual.get(), (*curwin.get()).w_cursor);
            if !vis_bef_curs && !vis_empty {
                let mut t: pos_T = (*curwin.get()).w_cursor;
                (*curwin.get()).w_cursor = VIsual.get();
                VIsual.set(t);
                vis_bef_curs = true_0 != 0;
                restore_vis_bef = true_0 != 0;
            }
        }
    }
    if !vis_empty {
        if vis_bef_curs {
            inside_quotes = (*VIsual.ptr()).col > 0 as ::core::ffi::c_int
                && *line.offset(
                    ((*VIsual.ptr()).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
                ) as uint8_t as ::core::ffi::c_int
                    == quotechar
                && *line.offset((*curwin.get()).w_cursor.col as isize) as ::core::ffi::c_int != NUL
                && *line.offset(
                    ((*curwin.get()).w_cursor.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        as isize,
                ) as uint8_t as ::core::ffi::c_int
                    == quotechar;
            i = (*VIsual.ptr()).col as ::core::ffi::c_int;
            col_end = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        } else {
            inside_quotes = (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
                && *line.offset(
                    ((*curwin.get()).w_cursor.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        as isize,
                ) as uint8_t as ::core::ffi::c_int
                    == quotechar
                && *line.offset((*VIsual.ptr()).col as isize) as ::core::ffi::c_int != NUL
                && *line.offset(
                    ((*VIsual.ptr()).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
                ) as uint8_t as ::core::ffi::c_int
                    == quotechar;
            i = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
            col_end = (*VIsual.ptr()).col as ::core::ffi::c_int;
        }
        while i <= col_end {
            if *line.offset(i as isize) as ::core::ffi::c_int == NUL {
                break;
            }
            let c2rust_fresh7 = i;
            i = i + 1;
            if *line.offset(c2rust_fresh7 as isize) as uint8_t as ::core::ffi::c_int != quotechar {
                continue;
            }
            selected_quote = true_0 != 0;
            break;
        }
    }
    '_abort_search: {
        's_368: {
            if !vis_empty
                && *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int == quotechar
            {
                if vis_bef_curs {
                    col_start = find_next_quote(
                        line,
                        col_start + 1 as ::core::ffi::c_int,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if col_start < 0 as ::core::ffi::c_int {
                        break '_abort_search;
                    } else {
                        col_end = find_next_quote(
                            line,
                            col_start + 1 as ::core::ffi::c_int,
                            quotechar,
                            (*curbuf.get()).b_p_qe,
                        );
                        if col_end < 0 as ::core::ffi::c_int {
                            col_end = col_start;
                            col_start = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                        }
                    }
                } else {
                    col_end = find_prev_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if *line.offset(col_end as isize) as uint8_t as ::core::ffi::c_int != quotechar
                    {
                        break '_abort_search;
                    } else {
                        col_start =
                            find_prev_quote(line, col_end, quotechar, (*curbuf.get()).b_p_qe);
                        if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int
                            != quotechar
                        {
                            col_start = col_end;
                            col_end = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                        }
                    }
                }
            } else if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int == quotechar
                || !vis_empty
            {
                let mut first_col: ::core::ffi::c_int = col_start;
                if !vis_empty {
                    if vis_bef_curs {
                        first_col = find_next_quote(
                            line,
                            col_start,
                            quotechar,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        );
                    } else {
                        first_col = find_prev_quote(
                            line,
                            col_start,
                            quotechar,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        );
                    }
                }
                col_start = 0 as ::core::ffi::c_int;
                loop {
                    col_start = find_next_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if col_start < 0 as ::core::ffi::c_int || col_start > first_col {
                        break '_abort_search;
                    }
                    col_end = find_next_quote(
                        line,
                        col_start + 1 as ::core::ffi::c_int,
                        quotechar,
                        (*curbuf.get()).b_p_qe,
                    );
                    if col_end < 0 as ::core::ffi::c_int {
                        break '_abort_search;
                    }
                    if col_start <= first_col && first_col <= col_end {
                        break 's_368;
                    }
                    col_start = col_end + 1 as ::core::ffi::c_int;
                }
            } else {
                col_start = find_prev_quote(line, col_start, quotechar, (*curbuf.get()).b_p_qe);
                if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int != quotechar {
                    col_start = find_next_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if col_start < 0 as ::core::ffi::c_int {
                        break '_abort_search;
                    }
                }
                col_end = find_next_quote(
                    line,
                    col_start + 1 as ::core::ffi::c_int,
                    quotechar,
                    (*curbuf.get()).b_p_qe,
                );
                if col_end < 0 as ::core::ffi::c_int {
                    break '_abort_search;
                }
            }
        }
        if include {
            if ascii_iswhite(
                *line.offset((col_end + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            ) {
                while ascii_iswhite(*line.offset((col_end + 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int)
                {
                    col_end += 1;
                }
            } else {
                while col_start > 0 as ::core::ffi::c_int
                    && ascii_iswhite(*line.offset((col_start - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as ::core::ffi::c_int
                        != 0
                {
                    col_start -= 1;
                }
            }
        }
        if !include
            && count < 2 as ::core::ffi::c_int
            && (vis_empty as ::core::ffi::c_int != 0 || !inside_quotes)
        {
            col_start += 1;
        }
        (*curwin.get()).w_cursor.col = col_start as colnr_T;
        if VIsual_active.get() {
            if vis_empty as ::core::ffi::c_int != 0
                || vis_bef_curs as ::core::ffi::c_int != 0
                    && !selected_quote
                    && (inside_quotes as ::core::ffi::c_int != 0
                        || *line.offset((*VIsual.ptr()).col as isize) as uint8_t
                            as ::core::ffi::c_int
                            != quotechar
                            && ((*VIsual.ptr()).col == 0 as ::core::ffi::c_int
                                || *line.offset(
                                    ((*VIsual.ptr()).col as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int)
                                        as isize,
                                ) as uint8_t
                                    as ::core::ffi::c_int
                                    != quotechar))
            {
                VIsual.set((*curwin.get()).w_cursor);
                redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
            }
        } else {
            (*oap).start = (*curwin.get()).w_cursor;
            (*oap).motion_type = kMTCharWise;
        }
        (*curwin.get()).w_cursor.col = col_end as colnr_T;
        if (include as ::core::ffi::c_int != 0
            || count > 1 as ::core::ffi::c_int
            || !vis_empty && inside_quotes as ::core::ffi::c_int != 0)
            && inc_cursor() == 2 as ::core::ffi::c_int
        {
            inclusive = true_0 != 0;
        }
        if VIsual_active.get() {
            if vis_empty as ::core::ffi::c_int != 0 || vis_bef_curs as ::core::ffi::c_int != 0 {
                if *p_sel.get() as ::core::ffi::c_int != 'e' as ::core::ffi::c_int {
                    dec_cursor();
                }
            } else {
                if inside_quotes as ::core::ffi::c_int != 0
                    || !selected_quote
                        && *line.offset((*VIsual.ptr()).col as isize) as uint8_t
                            as ::core::ffi::c_int
                            != quotechar
                        && (*line.offset((*VIsual.ptr()).col as isize) as ::core::ffi::c_int == NUL
                            || *line.offset(
                                ((*VIsual.ptr()).col as ::core::ffi::c_int
                                    + 1 as ::core::ffi::c_int)
                                    as isize,
                            ) as uint8_t as ::core::ffi::c_int
                                != quotechar)
                {
                    dec_cursor();
                    VIsual.set((*curwin.get()).w_cursor);
                }
                (*curwin.get()).w_cursor.col = col_start as colnr_T;
            }
            if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                VIsual_mode.set('v' as ::core::ffi::c_int);
                redraw_cmdline.set(true_0 != 0);
            }
        } else {
            (*oap).inclusive = inclusive;
        }
        return true_0 != 0;
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
    {
        if did_exclusive_adj {
            inc_cursor();
        }
        if restore_vis_bef {
            let mut t_0: pos_T = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = VIsual.get();
            VIsual.set(t_0);
        }
    }
    return false_0 != 0;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
