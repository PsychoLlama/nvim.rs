use crate::src::nvim::charset::{
    getdigits_int, getwhitecols_curline, skiptowhite, skipwhite, vim_isIDc, vim_iswordc,
    vim_iswordp, vim_strsize,
};
use crate::src::nvim::cursor::{get_cursor_line_ptr, get_cursor_pos_ptr};
use crate::src::nvim::eval::typval::tv_get_lnum;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::{
    fixthisline, get_expr_indent, get_indent, get_indent_lnum, get_sw_value,
};
use crate::src::nvim::keycodes::get_special_key_code;
use crate::src::nvim::main::{curbuf, curwin, p_paste, State};
use crate::src::nvim::mbyte::{mb_prevptr, mb_strnicmp, utfc_ptr2len};
use crate::src::nvim::memline::{ml_get, ml_get_pos};
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup};
use crate::src::nvim::option::{copy_option_part, skip_to_option_part};
use crate::src::nvim::os::libc::{__assert_fail, atoi, strcpy, strlen, strncmp, tolower};
use crate::src::nvim::plines::getvcol;
use crate::src::nvim::search::{check_linecomment, findmatchlimit, linewhite};
use crate::src::nvim::strings::vim_strchr;
pub use crate::src::nvim::types::{
    __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T,
    colnr_T, dict_T, dictvar_S, disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_4, file_buffer_b_wininfo as C2Rust_Unnamed_12,
    file_buffer_update_callbacks as C2Rust_Unnamed_1,
    file_buffer_update_channels as C2Rust_Unnamed_2, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_7, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, intptr_t, lcs_chars_T, linenr_T,
    list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, oparg_T, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t,
    qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T,
    scid_T, sctx_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_5, syn_time_T,
    synblock_T, synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    uintmax_t, undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S,
    winopt_T, wline_T, xfmark_T, AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks,
    Callback, CallbackType, Callback_data as C2Rust_Unnamed_6, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_3, EvalFuncData, ExtmarkUndoObject, FileID, FloatAnchor,
    FloatRelative, GridView, IndentGetter, Intersection, LuaRef, MTKey, MTNode, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    MotionType, MsgpackRpcRequestHandler, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_13, Terminal, Timestamp, VarLockStatus, VarType,
    VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle,
    Window, QUEUE,
};
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed = 2147483647;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_0 = 2147483647;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const KEY_COMPLETE: C2Rust_Unnamed_14 = 259;
pub const KEY_OPEN_BACK: C2Rust_Unnamed_14 = 258;
pub const KEY_OPEN_FORW: C2Rust_Unnamed_14 = 257;
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
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const FM_BACKWARD: C2Rust_Unnamed_16 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cpp_baseclass_cache_T {
    pub found: ::core::ffi::c_int,
    pub lpos: lpos_T,
}
pub const FM_BLOCKSTOP: C2Rust_Unnamed_16 = 4;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const FM_SKIPCOMM: C2Rust_Unnamed_16 = 8;
pub const FM_FORWARD: C2Rust_Unnamed_16 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const COM_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const COM_LEFT: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const COM_RIGHT: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const COM_MAX_LEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
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
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"_Bool in_cinkeys(int, int, _Bool)\0",
    )
};
unsafe extern "C" fn ind_find_start_comment() -> *mut pos_T {
    return find_start_comment((*curbuf.get()).b_ind_maxcomment);
}
pub unsafe extern "C" fn find_start_comment(mut ind_maxcomment: ::core::ffi::c_int) -> *mut pos_T {
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut cur_maxcomment: int64_t = ind_maxcomment as int64_t;
    loop {
        pos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            '*' as ::core::ffi::c_int,
            FM_BACKWARD as ::core::ffi::c_int,
            cur_maxcomment,
        );
        if pos.is_null() {
            break;
        }
        if is_pos_in_string(ml_get((*pos).lnum), (*pos).col) == 0 {
            break;
        }
        cur_maxcomment = ((*curwin.get()).w_cursor.lnum - (*pos).lnum - 1 as linenr_T) as int64_t;
        if cur_maxcomment > 0 as int64_t {
            continue;
        }
        pos = ::core::ptr::null_mut::<pos_T>();
        break;
    }
    return pos;
}
unsafe extern "C" fn ind_find_start_CORS(mut is_raw: *mut linenr_T) -> *mut pos_T {
    static comment_pos_copy: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    });
    let mut comment_pos: *mut pos_T = find_start_comment((*curbuf.get()).b_ind_maxcomment);
    if !comment_pos.is_null() {
        comment_pos_copy.set(*comment_pos);
        comment_pos = comment_pos_copy.ptr();
    }
    let mut rs_pos: *mut pos_T = find_start_rawstring((*curbuf.get()).b_ind_maxcomment);
    if comment_pos.is_null()
        || !rs_pos.is_null() && lt(*rs_pos, *comment_pos) as ::core::ffi::c_int != 0
    {
        if !is_raw.is_null() && !rs_pos.is_null() {
            *is_raw = (*rs_pos).lnum;
        }
        return rs_pos;
    }
    return comment_pos;
}
unsafe extern "C" fn find_start_rawstring(mut ind_maxcomment: ::core::ffi::c_int) -> *mut pos_T {
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut cur_maxcomment: ::core::ffi::c_int = ind_maxcomment;
    loop {
        pos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            'R' as ::core::ffi::c_int,
            FM_BACKWARD as ::core::ffi::c_int,
            cur_maxcomment as int64_t,
        );
        if pos.is_null() {
            break;
        }
        if is_pos_in_string(ml_get((*pos).lnum), (*pos).col) == 0 {
            break;
        }
        cur_maxcomment =
            ((*curwin.get()).w_cursor.lnum - (*pos).lnum - 1 as linenr_T) as ::core::ffi::c_int;
        if cur_maxcomment > 0 as ::core::ffi::c_int {
            continue;
        }
        pos = ::core::ptr::null_mut::<pos_T>();
        break;
    }
    return pos;
}
unsafe extern "C" fn skip_string(mut p: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char {
    let mut i: ::core::ffi::c_int = 0;
    loop {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\'' as ::core::ffi::c_int
        {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                break;
            }
            i = 2 as ::core::ffi::c_int;
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                i += 1;
                while ascii_isdigit(
                    *p.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                ) {
                    i += 1;
                }
            }
            if !(*p.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int != NUL
                && *p.offset(i as isize) as ::core::ffi::c_int == '\'' as ::core::ffi::c_int)
            {
                break;
            }
            p = p.offset(i as isize);
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '"' as ::core::ffi::c_int
        {
            p = p.offset(1);
            while *p.offset(0 as ::core::ffi::c_int as isize) != 0 {
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    p = p.offset(1);
                } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
                {
                    break;
                }
                p = p.offset(1);
            }
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '"' as ::core::ffi::c_int
            {
                break;
            }
        } else {
            if !(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'R' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int)
            {
                break;
            }
            let mut delim: *const ::core::ffi::c_char = p.offset(2 as ::core::ffi::c_int as isize);
            let mut paren: *const ::core::ffi::c_char =
                vim_strchr(delim, '(' as ::core::ffi::c_int);
            if paren.is_null() {
                break;
            }
            let delim_len: ptrdiff_t = paren.offset_from(delim);
            p = p.offset(3 as ::core::ffi::c_int as isize);
            while *p != 0 {
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ')' as ::core::ffi::c_int
                    && strncmp(
                        p.offset(1 as ::core::ffi::c_int as isize),
                        delim,
                        delim_len as size_t,
                    ) == 0 as ::core::ffi::c_int
                    && *p.offset((delim_len + 1 as ptrdiff_t) as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                {
                    p = p.offset((delim_len + 1 as ptrdiff_t) as isize);
                    break;
                } else {
                    p = p.offset(1);
                }
            }
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '"' as ::core::ffi::c_int
            {
                break;
            }
        }
        p = p.offset(1);
    }
    if *p == 0 {
        p = p.offset(-1);
    }
    return p;
}
pub unsafe extern "C" fn is_pos_in_string(
    mut line: *const ::core::ffi::c_char,
    mut col: colnr_T,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    p = line;
    while *p as ::core::ffi::c_int != 0 && (p.offset_from(line) as colnr_T) < col {
        p = skip_string(p);
        p = p.offset(1);
    }
    return !(p.offset_from(line) as colnr_T <= col) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn cin_is_cinword(mut line: *const ::core::ffi::c_char) -> bool {
    let mut retval: bool = false_0 != 0;
    let mut cinw_len: size_t = strlen((*curbuf.get()).b_p_cinw).wrapping_add(1 as size_t);
    let mut cinw_buf: *mut ::core::ffi::c_char = xmalloc(cinw_len) as *mut ::core::ffi::c_char;
    line = skipwhite(line);
    let mut cinw: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_cinw;
    while *cinw != 0 {
        let mut len: size_t = copy_option_part(
            &raw mut cinw,
            cinw_buf,
            cinw_len,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if !(strncmp(line, cinw_buf, len) == 0 as ::core::ffi::c_int
            && (!vim_iswordc(*line.offset(len as isize) as uint8_t as ::core::ffi::c_int)
                || !vim_iswordc(
                    *line.offset(len.wrapping_sub(1 as size_t) as isize) as uint8_t
                        as ::core::ffi::c_int,
                )))
        {
            continue;
        }
        retval = true_0 != 0;
        break;
    }
    xfree(cinw_buf as *mut ::core::ffi::c_void);
    return retval;
}
pub unsafe extern "C" fn cindent_on() -> bool {
    return p_paste.get() == 0
        && ((*curbuf.get()).b_p_cin != 0
            || *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL);
}
unsafe extern "C" fn cin_skipcomment(
    mut s: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    while *s != 0 {
        let mut prev_s: *const ::core::ffi::c_char = s;
        s = skipwhite(s);
        if (*curbuf.get()).b_ind_hash_comment != 0 as ::core::ffi::c_int
            && s != prev_s
            && *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int
        {
            s = s.offset(strlen(s) as isize);
            break;
        } else {
            if *s as ::core::ffi::c_int != '/' as ::core::ffi::c_int {
                break;
            }
            s = s.offset(1);
            if *s as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
                s = s.offset(strlen(s) as isize);
                break;
            } else {
                if *s as ::core::ffi::c_int != '*' as ::core::ffi::c_int {
                    break;
                }
                s = s.offset(1);
                while *s != 0 {
                    if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '*' as ::core::ffi::c_int
                        && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int
                    {
                        s = s.offset(2 as ::core::ffi::c_int as isize);
                        break;
                    } else {
                        s = s.offset(1);
                    }
                }
            }
        }
    }
    return s;
}
unsafe extern "C" fn cin_nocode(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (*cin_skipcomment(s) as ::core::ffi::c_int == NUL) as ::core::ffi::c_int;
}
unsafe extern "C" fn find_line_comment() -> *mut pos_T {
    static pos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    });
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    pos.set((*curwin.get()).w_cursor);
    loop {
        (*pos.ptr()).lnum -= 1;
        if (*pos.ptr()).lnum <= 0 as linenr_T {
            break;
        }
        line = ml_get((*pos.ptr()).lnum);
        p = skipwhite(line);
        if cin_islinecomment(p) != 0 {
            (*pos.ptr()).col = p.offset_from(line) as ::core::ffi::c_int as colnr_T;
            return pos.ptr();
        }
        if *p as ::core::ffi::c_int != NUL {
            break;
        }
    }
    return ::core::ptr::null_mut::<pos_T>();
}
unsafe extern "C" fn cin_has_js_key(mut text: *const ::core::ffi::c_char) -> bool {
    let mut s: *const ::core::ffi::c_char = skipwhite(text);
    let mut quote: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
    if *s as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
        || *s as ::core::ffi::c_int == '"' as ::core::ffi::c_int
    {
        quote = *s;
        s = s.offset(1);
    }
    if !vim_isIDc(*s as uint8_t as ::core::ffi::c_int) {
        return false_0 != 0;
    }
    while vim_isIDc(*s as uint8_t as ::core::ffi::c_int) {
        s = s.offset(1);
    }
    if *s as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int == quote as ::core::ffi::c_int {
        s = s.offset(1);
    }
    s = cin_skipcomment(s);
    return *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != ':' as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_islabel_skip(mut s: *mut *const ::core::ffi::c_char) -> bool {
    if !vim_isIDc(**s as uint8_t as ::core::ffi::c_int) {
        return false_0 != 0;
    }
    while vim_isIDc(**s as uint8_t as ::core::ffi::c_int) {
        *s = (*s).offset(utfc_ptr2len(*s) as isize);
    }
    *s = cin_skipcomment(*s);
    return **s as ::core::ffi::c_int == ':' as ::core::ffi::c_int && {
        *s = (*s).offset(1);
        **s as ::core::ffi::c_int != ':' as ::core::ffi::c_int
    };
}
unsafe extern "C" fn cin_islabel() -> bool {
    let mut s: *const ::core::ffi::c_char = cin_skipcomment(get_cursor_line_ptr());
    if cin_isdefault(s) != 0 {
        return false_0 != 0;
    }
    if cin_isscopedecl(s) {
        return false_0 != 0;
    }
    if !cin_islabel_skip(&raw mut s) {
        return false_0 != 0;
    }
    if !ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>()).is_null() {
        return false_0 != 0;
    }
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    cursor_save = (*curwin.get()).w_cursor;
    while (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
        (*curwin.get()).w_cursor.lnum -= 1;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        trypos = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
        if !trypos.is_null() {
            (*curwin.get()).w_cursor = *trypos;
        }
        line = get_cursor_line_ptr();
        if cin_ispreproc(line) != 0 {
            continue;
        }
        line = cin_skipcomment(line);
        if *line as ::core::ffi::c_int == NUL {
            continue;
        }
        (*curwin.get()).w_cursor = cursor_save;
        if cin_isterminated(line, true_0, false_0) as ::core::ffi::c_int != 0
            || cin_isscopedecl(line) as ::core::ffi::c_int != 0
            || cin_iscase(line, true_0 != 0) as ::core::ffi::c_int != 0
            || cin_islabel_skip(&raw mut line) as ::core::ffi::c_int != 0 && cin_nocode(line) != 0
        {
            return true_0 != 0;
        }
        return false_0 != 0;
    }
    (*curwin.get()).w_cursor = cursor_save;
    return true_0 != 0;
}
unsafe extern "C" fn cin_skip_comment_and_string(
    mut s: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut r: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = s;
    loop {
        r = p;
        p = cin_skipcomment(p);
        if *p != 0 {
            p = skip_string(p);
        }
        if p == r {
            break;
        }
    }
    return p;
}
unsafe extern "C" fn cin_is_compound_init(mut s: *const ::core::ffi::c_char) -> bool {
    let mut p: *const ::core::ffi::c_char = s;
    let mut r: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    while *p != 0 {
        if *p as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
            r = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
            p = r;
        } else if strncmp(
            p,
            b"return\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0
            && !vim_isIDc(*p.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            && (p == s
                || p > s
                    && !vim_isIDc(
                        *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ))
        {
            r = cin_skipcomment(p.offset(6 as ::core::ffi::c_int as isize));
            p = r;
        } else {
            p = cin_skip_comment_and_string(p.offset(1 as ::core::ffi::c_int as isize));
        }
    }
    if r.is_null() {
        return false_0 != 0;
    }
    p = r;
    if cin_nocode(p) != 0 {
        return true_0 != 0;
    }
    if *p as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
        p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
    }
    if *p as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
        let mut open_count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        loop {
            p = cin_skip_comment_and_string(p.offset(1 as ::core::ffi::c_int as isize));
            if cin_nocode(p) != 0 {
                return true_0 != 0;
            }
            open_count += (*p as ::core::ffi::c_int == '(' as ::core::ffi::c_int)
                as ::core::ffi::c_int
                - (*p as ::core::ffi::c_int == ')' as ::core::ffi::c_int) as ::core::ffi::c_int;
            if open_count == 0 {
                break;
            }
        }
        p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
        if cin_nocode(p) != 0 {
            return true_0 != 0;
        }
    }
    while *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
        p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
    }
    return cin_nocode(p) != 0;
}
unsafe extern "C" fn cin_isinit() -> bool {
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static skip: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
        b"static\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"public\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"protected\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"private\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    s = cin_skipcomment(get_cursor_line_ptr());
    if cin_starts_with(s, b"typedef\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
        s = cin_skipcomment(s.offset(7 as ::core::ffi::c_int as isize));
    }
    loop {
        let mut i: ::core::ffi::c_int = 0;
        let mut l: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < ::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
        {
            l = strlen((*skip.ptr())[i as usize]) as ::core::ffi::c_int;
            if cin_starts_with(s, (*skip.ptr())[i as usize]) != 0 {
                s = cin_skipcomment(s.offset(l as isize));
                l = 0 as ::core::ffi::c_int;
                break;
            } else {
                i += 1;
            }
        }
        if l != 0 as ::core::ffi::c_int {
            break;
        }
    }
    if cin_starts_with(s, b"enum\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
        return true_0 != 0;
    }
    return cin_is_compound_init(s);
}
unsafe extern "C" fn cin_iscase(mut s: *const ::core::ffi::c_char, mut strict: bool) -> bool {
    s = cin_skipcomment(s);
    if cin_starts_with(s, b"case\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
        s = s.offset(4 as ::core::ffi::c_int as isize);
        while *s != 0 {
            s = cin_skipcomment(s);
            if *s as ::core::ffi::c_int == NUL {
                break;
            }
            if *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
                {
                    s = s.offset(1);
                } else {
                    return true_0 != 0;
                }
            }
            if *s as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
                && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\'' as ::core::ffi::c_int
            {
                s = s.offset(2 as ::core::ffi::c_int as isize);
            } else if *s as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                && (*s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
                    || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int)
            {
                return false_0 != 0;
            } else if *s as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                if strict {
                    return false_0 != 0;
                }
                return true_0 != 0;
            }
            s = s.offset(1);
        }
        return false_0 != 0;
    }
    if cin_isdefault(s) != 0 {
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn cin_isdefault(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (strncmp(
        s,
        b"default\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
        && {
            s = cin_skipcomment(s.offset(7 as ::core::ffi::c_int as isize));
            *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        }
        && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != ':' as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_isscopedecl(mut p: *const ::core::ffi::c_char) -> bool {
    let mut s: *const ::core::ffi::c_char = cin_skipcomment(p);
    let cinsd_len: size_t = strlen((*curbuf.get()).b_p_cinsd).wrapping_add(1 as size_t);
    let mut cinsd_buf: *mut ::core::ffi::c_char = xmalloc(cinsd_len) as *mut ::core::ffi::c_char;
    let mut found: bool = false_0 != 0;
    let mut cinsd: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_cinsd;
    while *cinsd != 0 {
        let len: size_t = copy_option_part(
            &raw mut cinsd,
            cinsd_buf,
            cinsd_len,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if strncmp(s, cinsd_buf, len) != 0 as ::core::ffi::c_int {
            continue;
        }
        let mut skip: *const ::core::ffi::c_char = cin_skipcomment(s.offset(len as isize));
        if !(*skip as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *skip.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ':' as ::core::ffi::c_int)
        {
            continue;
        }
        found = true_0 != 0;
        break;
    }
    xfree(cinsd_buf as *mut ::core::ffi::c_void);
    return found;
}
pub const FIND_NAMESPACE_LIM: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
unsafe extern "C" fn cin_is_cpp_namespace(mut s: *const ::core::ffi::c_char) -> bool {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut has_name: bool = false_0 != 0;
    let mut has_name_start: bool = false_0 != 0;
    s = cin_skipcomment(s);
    while (strncmp(
        s,
        b"inline\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            s,
            b"export\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int)
        && (*s.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || !vim_iswordc(
                *s.offset(6 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            ))
    {
        s = cin_skipcomment(skipwhite(s.offset(6 as ::core::ffi::c_int as isize)));
    }
    if strncmp(
        s,
        b"namespace\0".as_ptr() as *const ::core::ffi::c_char,
        9 as size_t,
    ) == 0 as ::core::ffi::c_int
        && (*s.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || !vim_iswordc(
                *s.offset(9 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            ))
    {
        p = cin_skipcomment(skipwhite(s.offset(9 as ::core::ffi::c_int as isize)));
        while *p as ::core::ffi::c_int != NUL {
            if ascii_iswhite(*p as ::core::ffi::c_int) {
                has_name = true_0 != 0;
                p = cin_skipcomment(skipwhite(p));
            } else {
                if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                    break;
                }
                if vim_iswordc(*p as uint8_t as ::core::ffi::c_int) {
                    has_name_start = true_0 != 0;
                    if has_name {
                        return false_0 != 0;
                    }
                    p = p.offset(1);
                } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                    && vim_iswordc(*p.offset(2 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int) as ::core::ffi::c_int
                        != 0
                {
                    if !has_name_start || has_name as ::core::ffi::c_int != 0 {
                        return false_0 != 0;
                    }
                    p = p.offset(3 as ::core::ffi::c_int as isize);
                } else {
                    return false_0 != 0;
                }
            }
        }
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn after_label(mut l: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char {
    while *l != 0 {
        if *l as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
            if *l.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
            {
                l = l.offset(1);
            } else if !cin_iscase(l.offset(1 as ::core::ffi::c_int as isize), false_0 != 0) {
                break;
            }
        } else if *l as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
            && *l.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
            && *l.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\'' as ::core::ffi::c_int
        {
            l = l.offset(2 as ::core::ffi::c_int as isize);
        }
        l = l.offset(1);
    }
    if *l as ::core::ffi::c_int == NUL {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    l = cin_skipcomment(l.offset(1 as ::core::ffi::c_int as isize));
    if *l as ::core::ffi::c_int == NUL {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return l;
}
unsafe extern "C" fn get_indent_nolabel(mut lnum: linenr_T) -> ::core::ffi::c_int {
    let mut l: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut fp: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut col: colnr_T = 0;
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    l = ml_get(lnum);
    p = after_label(l);
    if p.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    fp.col = p.offset_from(l) as colnr_T;
    fp.lnum = lnum;
    getvcol(
        curwin.get(),
        &raw mut fp,
        &raw mut col,
        ::core::ptr::null_mut::<colnr_T>(),
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return col;
}
unsafe extern "C" fn skip_label(
    mut lnum: linenr_T,
    mut pp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut l: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut amount: ::core::ffi::c_int = 0;
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    cursor_save = (*curwin.get()).w_cursor;
    (*curwin.get()).w_cursor.lnum = lnum;
    l = get_cursor_line_ptr();
    if cin_iscase(l, false_0 != 0) as ::core::ffi::c_int != 0
        || cin_isscopedecl(l) as ::core::ffi::c_int != 0
        || cin_islabel() as ::core::ffi::c_int != 0
    {
        amount = get_indent_nolabel(lnum);
        l = after_label(get_cursor_line_ptr());
        if l.is_null() {
            l = get_cursor_line_ptr();
        }
    } else {
        amount = get_indent();
        l = get_cursor_line_ptr();
    }
    *pp = l;
    (*curwin.get()).w_cursor = cursor_save;
    return amount;
}
unsafe extern "C" fn cin_first_id_amount() -> ::core::ffi::c_int {
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    let mut fp: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut col: colnr_T = 0;
    line = get_cursor_line_ptr();
    p = skipwhite(line);
    len = skiptowhite(p).offset_from(p) as ::core::ffi::c_int;
    if len == 6 as ::core::ffi::c_int
        && strncmp(
            p,
            b"static\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        p = skipwhite(p.offset(6 as ::core::ffi::c_int as isize));
        len = skiptowhite(p).offset_from(p) as ::core::ffi::c_int;
    }
    if len == 6 as ::core::ffi::c_int
        && strncmp(
            p,
            b"struct\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        p = skipwhite(p.offset(6 as ::core::ffi::c_int as isize));
    } else if len == 4 as ::core::ffi::c_int
        && strncmp(
            p,
            b"enum\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        p = skipwhite(p.offset(4 as ::core::ffi::c_int as isize));
    } else if len == 8 as ::core::ffi::c_int
        && strncmp(
            p,
            b"unsigned\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        || len == 6 as ::core::ffi::c_int
            && strncmp(
                p,
                b"signed\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
    {
        s = skipwhite(p.offset(len as isize));
        if strncmp(
            s,
            b"int\0".as_ptr() as *const ::core::ffi::c_char,
            3 as size_t,
        ) == 0 as ::core::ffi::c_int
            && ascii_iswhite(*s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
            || strncmp(
                s,
                b"long\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_iswhite(*s.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            || strncmp(
                s,
                b"short\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_iswhite(*s.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            || strncmp(
                s,
                b"char\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_iswhite(*s.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
        {
            p = s;
        }
    }
    len = 0 as ::core::ffi::c_int;
    while vim_isIDc(*p.offset(len as isize) as uint8_t as ::core::ffi::c_int) {
        len += 1;
    }
    if len == 0 as ::core::ffi::c_int
        || !ascii_iswhite(*p.offset(len as isize) as ::core::ffi::c_int)
        || cin_nocode(p) != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    p = skipwhite(p.offset(len as isize));
    fp.lnum = (*curwin.get()).w_cursor.lnum;
    fp.col = p.offset_from(line) as colnr_T;
    getvcol(
        curwin.get(),
        &raw mut fp,
        &raw mut col,
        ::core::ptr::null_mut::<colnr_T>(),
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return col;
}
unsafe extern "C" fn cin_get_equal_amount(mut lnum: linenr_T) -> ::core::ffi::c_int {
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut col: colnr_T = 0;
    let mut fp: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    if lnum > 1 as linenr_T {
        line = ml_get(lnum - 1 as linenr_T);
        if *line as ::core::ffi::c_int != NUL
            && *line.offset(strlen(line).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
        {
            return -1 as ::core::ffi::c_int;
        }
    }
    s = ml_get(lnum);
    line = s;
    while *s as ::core::ffi::c_int != NUL
        && vim_strchr(
            b"=;{}\"'\0".as_ptr() as *const ::core::ffi::c_char,
            *s as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
    {
        if cin_iscomment(s) != 0 {
            s = cin_skipcomment(s);
        } else {
            s = s.offset(1);
        }
    }
    if *s as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    s = skipwhite(s.offset(1 as ::core::ffi::c_int as isize));
    if cin_nocode(s) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if *s as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
        s = s.offset(1);
    }
    fp.lnum = lnum;
    fp.col = s.offset_from(line) as colnr_T;
    getvcol(
        curwin.get(),
        &raw mut fp,
        &raw mut col,
        ::core::ptr::null_mut::<colnr_T>(),
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return col;
}
unsafe extern "C" fn cin_ispreproc(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *skipwhite(s) as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
        return true_0;
    }
    return false_0;
}
unsafe extern "C" fn cin_ispreproc_cont(
    mut pp: *mut *const ::core::ffi::c_char,
    mut lnump: *mut linenr_T,
    mut amount: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut line: *const ::core::ffi::c_char = *pp;
    let mut lnum: linenr_T = *lnump;
    let mut retval: ::core::ffi::c_int = false_0;
    let mut candidate_amount: ::core::ffi::c_int = *amount;
    if *line as ::core::ffi::c_int != NUL
        && *line.offset(strlen(line).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
    {
        candidate_amount = get_indent_lnum(lnum);
    }
    loop {
        if cin_ispreproc(line) != 0 {
            retval = true_0;
            *lnump = lnum;
            break;
        } else {
            if lnum == 1 as linenr_T {
                break;
            }
            lnum -= 1;
            line = ml_get(lnum);
            if *line as ::core::ffi::c_int == NUL
                || *line.offset(strlen(line).wrapping_sub(1 as size_t) as isize)
                    as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
            {
                break;
            }
        }
    }
    if lnum != *lnump {
        *pp = ml_get(*lnump);
    }
    if retval != 0 {
        *amount = candidate_amount;
    }
    return retval;
}
unsafe extern "C" fn cin_iscomment(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '/' as ::core::ffi::c_int
        && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '*' as ::core::ffi::c_int
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int)) as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_islinecomment(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '/' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '/' as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_isterminated(
    mut s: *const ::core::ffi::c_char,
    mut incl_open: ::core::ffi::c_int,
    mut incl_comma: ::core::ffi::c_int,
) -> ::core::ffi::c_char {
    let mut found_start: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
    let mut n_open: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut is_else: ::core::ffi::c_int = false_0;
    s = cin_skipcomment(s);
    if *s as ::core::ffi::c_int == '{' as ::core::ffi::c_int
        || *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int && cin_iselse(s) == 0
    {
        found_start = *s;
    }
    if found_start == 0 {
        is_else = cin_iselse(s);
    }
    while *s != 0 {
        s = skip_string(cin_skipcomment(s));
        if *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
            && n_open > 0 as ::core::ffi::c_uint
        {
            n_open = n_open.wrapping_sub(1);
        }
        if (is_else == 0 || n_open == 0 as ::core::ffi::c_uint)
            && (*s as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                || incl_comma != 0 && *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
            && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
        {
            return *s;
        } else if *s as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
            if incl_open != 0 && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0 {
                return *s;
            } else {
                n_open = n_open.wrapping_add(1);
            }
        }
        if *s != 0 {
            s = s.offset(1);
        }
    }
    return found_start;
}
unsafe extern "C" fn cin_isfuncdecl(
    mut sp: *mut *const ::core::ffi::c_char,
    mut first_lnum: linenr_T,
    mut min_lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut lnum: linenr_T = first_lnum;
    let mut save_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut retval: ::core::ffi::c_int = false_0;
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut just_started: ::core::ffi::c_int = true_0;
    if sp.is_null() {
        s = ml_get(lnum);
    } else {
        s = *sp;
    }
    (*curwin.get()).w_cursor.lnum = lnum;
    if find_last_paren(s, '(' as ::core::ffi::c_char, ')' as ::core::ffi::c_char) != 0 && {
        trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
        !trypos.is_null()
    } {
        lnum = (*trypos).lnum;
        if lnum < min_lnum {
            (*curwin.get()).w_cursor.lnum = save_lnum;
            return false_0;
        }
        s = ml_get(lnum);
    }
    (*curwin.get()).w_cursor.lnum = save_lnum;
    if cin_ispreproc(s) != 0 {
        return false_0;
    }
    while *s as ::core::ffi::c_int != 0
        && *s as ::core::ffi::c_int != '(' as ::core::ffi::c_int
        && *s as ::core::ffi::c_int != ';' as ::core::ffi::c_int
        && *s as ::core::ffi::c_int != '\'' as ::core::ffi::c_int
        && *s as ::core::ffi::c_int != '"' as ::core::ffi::c_int
    {
        if cin_iscomment(s) != 0 {
            s = cin_skipcomment(s);
        } else if *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
            if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
            {
                s = s.offset(2 as ::core::ffi::c_int as isize);
            } else {
                return false_0;
            }
        } else {
            s = s.offset(1);
        }
    }
    if *s as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
        return false_0;
    }
    while *s as ::core::ffi::c_int != 0
        && *s as ::core::ffi::c_int != ';' as ::core::ffi::c_int
        && *s as ::core::ffi::c_int != '\'' as ::core::ffi::c_int
        && *s as ::core::ffi::c_int != '"' as ::core::ffi::c_int
    {
        if *s as ::core::ffi::c_int == ')' as ::core::ffi::c_int
            && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
        {
            lnum = first_lnum - 1 as linenr_T;
            s = ml_get(lnum);
            if *s as ::core::ffi::c_int == NUL
                || *s.offset(strlen(s).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
            {
                retval = true_0;
            }
            break;
        } else if *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
            || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || cin_nocode(s) != 0
        {
            let mut comma: ::core::ffi::c_int =
                (*s as ::core::ffi::c_int == ',' as ::core::ffi::c_int) as ::core::ffi::c_int;
            while lnum < (*curbuf.get()).b_ml.ml_line_count {
                lnum += 1;
                s = ml_get(lnum);
                if cin_ispreproc(s) == 0 {
                    break;
                }
            }
            if lnum >= (*curbuf.get()).b_ml.ml_line_count {
                break;
            }
            s = skipwhite(s);
            if just_started == 0
                && (comma == 0
                    && *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                    && *s as ::core::ffi::c_int != ')' as ::core::ffi::c_int)
            {
                break;
            }
            just_started = false_0;
        } else if cin_iscomment(s) != 0 {
            s = cin_skipcomment(s);
        } else {
            s = s.offset(1);
            just_started = false_0;
        }
    }
    if lnum != first_lnum && !sp.is_null() {
        *sp = ml_get(first_lnum);
    }
    return retval;
}
unsafe extern "C" fn cin_isif(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (strncmp(
        p,
        b"if\0".as_ptr() as *const ::core::ffi::c_char,
        2 as size_t,
    ) == 0 as ::core::ffi::c_int
        && !vim_isIDc(*p.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_iselse(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
        p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
    }
    return (strncmp(
        p,
        b"else\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
        && !vim_isIDc(*p.offset(4 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_isdo(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (strncmp(
        p,
        b"do\0".as_ptr() as *const ::core::ffi::c_char,
        2 as size_t,
    ) == 0 as ::core::ffi::c_int
        && !vim_isIDc(*p.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_iswhileofdo(
    mut p: *const ::core::ffi::c_char,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut retval: ::core::ffi::c_int = false_0;
    p = cin_skipcomment(p);
    if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
        p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
    }
    if cin_starts_with(p, b"while\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
        cursor_save = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.lnum = lnum;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        p = get_cursor_line_ptr();
        while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != 'w' as ::core::ffi::c_int
        {
            p = p.offset(1);
            (*curwin.get()).w_cursor.col += 1;
        }
        trypos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (*curbuf.get()).b_ind_maxparen as int64_t,
        );
        if !trypos.is_null()
            && *cin_skipcomment(ml_get_pos(trypos).offset(1 as ::core::ffi::c_int as isize))
                as ::core::ffi::c_int
                == ';' as ::core::ffi::c_int
        {
            retval = true_0;
        }
        (*curwin.get()).w_cursor = cursor_save;
    }
    return retval;
}
unsafe extern "C" fn cin_is_if_for_while_before_offset(
    mut line: *const ::core::ffi::c_char,
    mut poffset: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut offset: ::core::ffi::c_int = *poffset;
    let c2rust_fresh3 = offset;
    offset = offset - 1;
    if c2rust_fresh3 < 2 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    while offset > 2 as ::core::ffi::c_int
        && ascii_iswhite(*line.offset(offset as isize) as ::core::ffi::c_int) as ::core::ffi::c_int
            != 0
    {
        offset -= 1;
    }
    offset -= 1 as ::core::ffi::c_int;
    '_probablyFound: {
        if strncmp(
            line.offset(offset as isize),
            b"if\0".as_ptr() as *const ::core::ffi::c_char,
            2 as size_t,
        ) != 0
        {
            if offset >= 1 as ::core::ffi::c_int {
                offset -= 1 as ::core::ffi::c_int;
                if strncmp(
                    line.offset(offset as isize),
                    b"for\0".as_ptr() as *const ::core::ffi::c_char,
                    3 as size_t,
                ) == 0
                {
                    break '_probablyFound;
                } else if offset >= 2 as ::core::ffi::c_int {
                    offset -= 2 as ::core::ffi::c_int;
                    if strncmp(
                        line.offset(offset as isize),
                        b"while\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0
                    {
                        break '_probablyFound;
                    }
                }
            }
            return 0 as ::core::ffi::c_int;
        }
    }
    if offset == 0
        || !vim_isIDc(
            *line.offset((offset - 1 as ::core::ffi::c_int) as isize) as uint8_t
                as ::core::ffi::c_int,
        )
    {
        *poffset = offset;
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_iswhileofdo_end(mut terminated: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut i: ::core::ffi::c_int = 0;
    if terminated != ';' as ::core::ffi::c_int {
        return false_0;
    }
    line = get_cursor_line_ptr();
    p = line;
    while *p as ::core::ffi::c_int != NUL {
        p = cin_skipcomment(p);
        if *p as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
            s = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
            if *s as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
            {
                i = p.offset_from(line) as ::core::ffi::c_int;
                (*curwin.get()).w_cursor.col = i as colnr_T;
                trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                if !trypos.is_null() {
                    s = cin_skipcomment(ml_get((*trypos).lnum));
                    if *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                        s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
                    }
                    if cin_starts_with(s, b"while\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
                        (*curwin.get()).w_cursor.lnum = (*trypos).lnum;
                        return true_0;
                    }
                }
                line = get_cursor_line_ptr();
                p = line.offset(i as isize);
            }
        }
        if *p as ::core::ffi::c_int != NUL {
            p = p.offset(1);
        }
    }
    return false_0;
}
unsafe extern "C" fn cin_isbreak(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (strncmp(
        p,
        b"break\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
        && !vim_isIDc(*p.offset(5 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_is_cpp_baseclass(
    mut cached: *mut cpp_baseclass_cache_T,
) -> ::core::ffi::c_int {
    let mut pos: *mut lpos_T = &raw mut (*cached).lpos;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut class_or_struct: ::core::ffi::c_int = 0;
    let mut lookfor_ctor_init: ::core::ffi::c_int = 0;
    let mut cpp_base_class: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut line: *const ::core::ffi::c_char = get_cursor_line_ptr();
    if (*pos).lnum <= lnum {
        return (*cached).found;
    }
    (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
    s = skipwhite(line);
    if *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
        return false_0;
    }
    s = cin_skipcomment(s);
    if *s as ::core::ffi::c_int == NUL {
        return false_0;
    }
    class_or_struct = false_0;
    lookfor_ctor_init = class_or_struct;
    cpp_base_class = lookfor_ctor_init;
    while lnum > 1 as linenr_T {
        line = ml_get(lnum - 1 as linenr_T);
        s = skipwhite(line);
        if *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int || *s as ::core::ffi::c_int == NUL
        {
            break;
        }
        while *s as ::core::ffi::c_int != NUL {
            s = cin_skipcomment(s);
            if *s as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                    && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
            {
                break;
            }
            if *s as ::core::ffi::c_int != NUL {
                s = s.offset(1);
            }
        }
        if *s as ::core::ffi::c_int != NUL {
            break;
        }
        lnum -= 1;
    }
    (*pos).lnum = lnum;
    line = ml_get(lnum);
    s = line;
    loop {
        if *s as ::core::ffi::c_int == NUL {
            if lnum == (*curwin.get()).w_cursor.lnum {
                break;
            }
            lnum += 1;
            line = ml_get(lnum);
            s = line;
        }
        if s == line {
            if cin_iscase(s, false_0 != 0) {
                break;
            }
            s = cin_skipcomment(line);
            if *s as ::core::ffi::c_int == NUL {
                continue;
            }
        }
        if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '"' as ::core::ffi::c_int
            || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'R' as ::core::ffi::c_int
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
        {
            s = skip_string(s).offset(1 as ::core::ffi::c_int as isize);
        } else if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
        {
            if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
            {
                lookfor_ctor_init = false_0;
                s = cin_skipcomment(s.offset(2 as ::core::ffi::c_int as isize));
            } else if lookfor_ctor_init != 0 || class_or_struct != 0 {
                cpp_base_class = true_0;
                class_or_struct = false_0;
                lookfor_ctor_init = class_or_struct;
                (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
                s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
            } else {
                s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
            }
        } else if strncmp(
            s,
            b"class\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
            && !vim_isIDc(
                *s.offset(5 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            )
            || strncmp(
                s,
                b"struct\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
                && !vim_isIDc(
                    *s.offset(6 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                )
        {
            class_or_struct = true_0;
            lookfor_ctor_init = false_0;
            if *s as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
                s = cin_skipcomment(s.offset(5 as ::core::ffi::c_int as isize));
            } else {
                s = cin_skipcomment(s.offset(6 as ::core::ffi::c_int as isize));
            }
        } else {
            if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '{' as ::core::ffi::c_int
                || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '}' as ::core::ffi::c_int
                || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ';' as ::core::ffi::c_int
            {
                class_or_struct = false_0;
                lookfor_ctor_init = class_or_struct;
                cpp_base_class = lookfor_ctor_init;
            } else if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ')' as ::core::ffi::c_int
            {
                class_or_struct = false_0;
                lookfor_ctor_init = true_0;
            } else if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '?' as ::core::ffi::c_int
            {
                return false_0;
            } else if !vim_isIDc(
                *s.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            ) {
                class_or_struct = false_0;
                lookfor_ctor_init = false_0;
            } else if (*pos).col == 0 as ::core::ffi::c_int {
                lookfor_ctor_init = false_0;
                if cpp_base_class != 0 {
                    (*pos).col = s.offset_from(line) as colnr_T;
                }
            }
            if lnum == (*curwin.get()).w_cursor.lnum
                && *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
            {
                (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
            }
            s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
        }
    }
    (*cached).found = cpp_base_class;
    if cpp_base_class != 0 {
        (*pos).lnum = lnum;
    }
    return cpp_base_class;
}
unsafe extern "C" fn get_baseclass_amount(mut col: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut amount: ::core::ffi::c_int = 0;
    let mut vcol: colnr_T = 0;
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    if col == 0 as ::core::ffi::c_int {
        amount = get_indent();
        if find_last_paren(
            get_cursor_line_ptr(),
            '(' as ::core::ffi::c_char,
            ')' as ::core::ffi::c_char,
        ) != 0
            && {
                trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                !trypos.is_null()
            }
        {
            amount = get_indent_lnum((*trypos).lnum);
        }
        if cin_ends_in(
            get_cursor_line_ptr(),
            b",\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0
        {
            amount += (*curbuf.get()).b_ind_cpp_baseclass;
        }
    } else {
        (*curwin.get()).w_cursor.col = col as colnr_T;
        getvcol(
            curwin.get(),
            &raw mut (*curwin.get()).w_cursor,
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        amount = vcol;
    }
    if amount < (*curbuf.get()).b_ind_cpp_baseclass {
        amount = (*curbuf.get()).b_ind_cpp_baseclass;
    }
    return amount;
}
unsafe extern "C" fn cin_ends_in(
    mut s: *const ::core::ffi::c_char,
    mut find: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = s;
    let mut r: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = strlen(find) as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != NUL {
        p = cin_skipcomment(p);
        if strncmp(p, find, len as size_t) == 0 as ::core::ffi::c_int {
            r = skipwhite(p.offset(len as isize));
            if cin_nocode(r) != 0 {
                return true_0;
            }
        }
        if *p as ::core::ffi::c_int != NUL {
            p = p.offset(1);
        }
    }
    return false_0;
}
unsafe extern "C" fn cin_starts_with(
    mut s: *const ::core::ffi::c_char,
    mut word: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut l: size_t = strlen(word);
    return (strncmp(s, word, l) == 0 as ::core::ffi::c_int
        && !vim_isIDc(*s.offset(l as isize) as uint8_t as ::core::ffi::c_int))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_is_cpp_extern_c(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut has_string_literal: ::core::ffi::c_int = false_0;
    s = cin_skipcomment(s);
    if strncmp(
        s,
        b"extern\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
        && (*s.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || !vim_iswordc(
                *s.offset(6 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            ))
    {
        p = cin_skipcomment(skipwhite(s.offset(6 as ::core::ffi::c_int as isize)));
        while *p as ::core::ffi::c_int != NUL {
            if ascii_iswhite(*p as ::core::ffi::c_int) {
                p = cin_skipcomment(skipwhite(p));
            } else {
                if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                    break;
                }
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'C' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                {
                    if has_string_literal != 0 {
                        return false_0;
                    }
                    has_string_literal = true_0;
                    p = p.offset(3 as ::core::ffi::c_int as isize);
                } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'C' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '+' as ::core::ffi::c_int
                    && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '+' as ::core::ffi::c_int
                    && *p.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                {
                    if has_string_literal != 0 {
                        return false_0;
                    }
                    has_string_literal = true_0;
                    p = p.offset(5 as ::core::ffi::c_int as isize);
                } else {
                    return false_0;
                }
            }
        }
        return if has_string_literal != 0 {
            true_0
        } else {
            false_0
        };
    }
    return false_0;
}
unsafe extern "C" fn cin_skip2pos(mut trypos: *mut pos_T) -> ::core::ffi::c_int {
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut new_p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    line = ml_get((*trypos).lnum);
    p = line;
    while *p as ::core::ffi::c_int != 0 && (p.offset_from(line) as colnr_T) < (*trypos).col {
        if cin_iscomment(p) != 0 {
            p = cin_skipcomment(p);
        } else {
            new_p = skip_string(p);
            if new_p == p {
                p = p.offset(1);
            } else {
                p = new_p;
            }
        }
    }
    return p.offset_from(line) as ::core::ffi::c_int;
}
unsafe extern "C" fn find_start_brace() -> *mut pos_T {
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    static pos_copy: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    });
    cursor_save = (*curwin.get()).w_cursor;
    loop {
        trypos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            '{' as ::core::ffi::c_int,
            FM_BLOCKSTOP as ::core::ffi::c_int,
            0 as int64_t,
        );
        if trypos.is_null() {
            break;
        }
        pos_copy.set(*trypos);
        trypos = pos_copy.ptr();
        (*curwin.get()).w_cursor = *trypos;
        pos = ::core::ptr::null_mut::<pos_T>();
        if cin_skip2pos(trypos) == (*trypos).col && {
            pos = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
            pos.is_null()
        } {
            break;
        }
        if !pos.is_null() {
            (*curwin.get()).w_cursor = *pos;
        }
    }
    (*curwin.get()).w_cursor = cursor_save;
    return trypos;
}
unsafe extern "C" fn find_match_paren(mut ind_maxparen: ::core::ffi::c_int) -> *mut pos_T {
    return find_match_char('(' as ::core::ffi::c_char, ind_maxparen);
}
unsafe extern "C" fn find_match_char(
    mut c: ::core::ffi::c_char,
    mut ind_maxparen: ::core::ffi::c_int,
) -> *mut pos_T {
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    static pos_copy: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    });
    let mut ind_maxp_wk: ::core::ffi::c_int = 0;
    cursor_save = (*curwin.get()).w_cursor;
    ind_maxp_wk = ind_maxparen;
    loop {
        trypos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            c as uint8_t as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            ind_maxp_wk as int64_t,
        );
        if trypos.is_null() {
            break;
        }
        if cin_skip2pos(trypos) > (*trypos).col {
            ind_maxp_wk = (ind_maxparen as linenr_T - (cursor_save.lnum - (*trypos).lnum))
                as ::core::ffi::c_int;
            if ind_maxp_wk > 0 as ::core::ffi::c_int {
                (*curwin.get()).w_cursor = *trypos;
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            } else {
                trypos = ::core::ptr::null_mut::<pos_T>();
                break;
            }
        } else {
            let mut trypos_wk: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
            pos_copy.set(*trypos);
            trypos = pos_copy.ptr();
            (*curwin.get()).w_cursor = *trypos;
            trypos_wk = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
            if trypos_wk.is_null() {
                break;
            }
            ind_maxp_wk = (ind_maxparen as linenr_T - (cursor_save.lnum - (*trypos_wk).lnum))
                as ::core::ffi::c_int;
            if ind_maxp_wk > 0 as ::core::ffi::c_int {
                (*curwin.get()).w_cursor = *trypos_wk;
            } else {
                trypos = ::core::ptr::null_mut::<pos_T>();
                break;
            }
        }
    }
    (*curwin.get()).w_cursor = cursor_save;
    return trypos;
}
unsafe extern "C" fn find_match_paren_after_brace(
    mut ind_maxparen: ::core::ffi::c_int,
) -> *mut pos_T {
    let mut trypos: *mut pos_T = find_match_paren(ind_maxparen);
    if trypos.is_null() {
        return ::core::ptr::null_mut::<pos_T>();
    }
    let mut tryposBrace: *mut pos_T = find_start_brace();
    if !tryposBrace.is_null()
        && (if (*trypos).lnum != (*tryposBrace).lnum {
            ((*trypos).lnum < (*tryposBrace).lnum) as ::core::ffi::c_int
        } else {
            ((*trypos).col < (*tryposBrace).col) as ::core::ffi::c_int
        }) != 0
    {
        trypos = ::core::ptr::null_mut::<pos_T>();
    }
    return trypos;
}
unsafe extern "C" fn corr_ind_maxparen(mut startpos: *mut pos_T) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = (*startpos).lnum as ::core::ffi::c_int
        - (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int;
    if n > 0 as ::core::ffi::c_int && n < (*curbuf.get()).b_ind_maxparen / 2 as ::core::ffi::c_int {
        return (*curbuf.get()).b_ind_maxparen - n;
    }
    return (*curbuf.get()).b_ind_maxparen;
}
unsafe extern "C" fn find_last_paren(
    mut l: *const ::core::ffi::c_char,
    mut start: ::core::ffi::c_char,
    mut end: ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut retval: ::core::ffi::c_int = false_0;
    let mut open_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    i = 0 as ::core::ffi::c_int;
    while *l.offset(i as isize) as ::core::ffi::c_int != NUL {
        i = cin_skipcomment(l.offset(i as isize)).offset_from(l) as ::core::ffi::c_int;
        i = skip_string(l.offset(i as isize)).offset_from(l) as ::core::ffi::c_int;
        if *l.offset(i as isize) as ::core::ffi::c_int == start as ::core::ffi::c_int {
            open_count += 1;
        } else if *l.offset(i as isize) as ::core::ffi::c_int == end as ::core::ffi::c_int {
            if open_count > 0 as ::core::ffi::c_int {
                open_count -= 1;
            } else {
                (*curwin.get()).w_cursor.col = i as colnr_T;
                retval = true_0;
            }
        }
        i += 1;
    }
    return retval;
}
pub unsafe extern "C" fn parse_cino(mut buf: *mut buf_T) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut l: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut divider: ::core::ffi::c_int = 0;
    let mut fraction: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sw: ::core::ffi::c_int = get_sw_value(buf);
    (*buf).b_ind_level = sw;
    (*buf).b_ind_open_imag = 0 as ::core::ffi::c_int;
    (*buf).b_ind_no_brace = 0 as ::core::ffi::c_int;
    (*buf).b_ind_first_open = 0 as ::core::ffi::c_int;
    (*buf).b_ind_open_extra = 0 as ::core::ffi::c_int;
    (*buf).b_ind_close_extra = 0 as ::core::ffi::c_int;
    (*buf).b_ind_open_left_imag = 0 as ::core::ffi::c_int;
    (*buf).b_ind_jump_label = -1 as ::core::ffi::c_int;
    (*buf).b_ind_case = sw;
    (*buf).b_ind_case_code = sw;
    (*buf).b_ind_case_break = 0 as ::core::ffi::c_int;
    (*buf).b_ind_scopedecl = sw;
    (*buf).b_ind_scopedecl_code = sw;
    (*buf).b_ind_param = sw;
    (*buf).b_ind_func_type = sw;
    (*buf).b_ind_cpp_baseclass = sw;
    (*buf).b_ind_continuation = sw;
    (*buf).b_ind_unclosed = sw * 2 as ::core::ffi::c_int;
    (*buf).b_ind_unclosed2 = sw;
    (*buf).b_ind_unclosed_noignore = 0 as ::core::ffi::c_int;
    (*buf).b_ind_unclosed_wrapped = 0 as ::core::ffi::c_int;
    (*buf).b_ind_unclosed_whiteok = 0 as ::core::ffi::c_int;
    (*buf).b_ind_matching_paren = 0 as ::core::ffi::c_int;
    (*buf).b_ind_paren_prev = 0 as ::core::ffi::c_int;
    (*buf).b_ind_comment = 0 as ::core::ffi::c_int;
    (*buf).b_ind_in_comment = 3 as ::core::ffi::c_int;
    (*buf).b_ind_in_comment2 = 0 as ::core::ffi::c_int;
    (*buf).b_ind_maxparen = 20 as ::core::ffi::c_int;
    (*buf).b_ind_maxcomment = 70 as ::core::ffi::c_int;
    (*buf).b_ind_java = 0 as ::core::ffi::c_int;
    (*buf).b_ind_js = 0 as ::core::ffi::c_int;
    (*buf).b_ind_keep_case_label = 0 as ::core::ffi::c_int;
    (*buf).b_ind_cpp_namespace = 0 as ::core::ffi::c_int;
    (*buf).b_ind_if_for_while = 0 as ::core::ffi::c_int;
    (*buf).b_ind_hash_comment = 0 as ::core::ffi::c_int;
    (*buf).b_ind_cpp_extern_c = 0 as ::core::ffi::c_int;
    (*buf).b_ind_pragma = 0 as ::core::ffi::c_int;
    p = (*buf).b_p_cino;
    while *p != 0 {
        let c2rust_fresh0 = p;
        p = p.offset(1);
        l = c2rust_fresh0;
        if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
            p = p.offset(1);
        }
        let mut digits_start: *mut ::core::ffi::c_char = p;
        let mut n: int64_t =
            getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int) as int64_t;
        divider = 0 as ::core::ffi::c_int;
        if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            p = p.offset(1);
            fraction = atoi(p);
            while ascii_isdigit(*p as ::core::ffi::c_int) {
                p = p.offset(1);
                if divider != 0 {
                    divider *= 10 as ::core::ffi::c_int;
                } else {
                    divider = 10 as ::core::ffi::c_int;
                }
            }
        }
        if *p as ::core::ffi::c_int == 's' as ::core::ffi::c_int {
            if p == digits_start {
                n = sw as int64_t;
            } else {
                n *= sw as int64_t;
                if divider != 0 {
                    n += (sw as int64_t * fraction as int64_t
                        + (divider / 2 as ::core::ffi::c_int) as int64_t)
                        / divider as int64_t;
                }
            }
            p = p.offset(1);
        }
        if *l.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
        {
            n = -n;
        }
        n = crate::src::nvim::math::trim_to_int(n) as int64_t;
        match *l as ::core::ffi::c_int {
            62 => {
                (*buf).b_ind_level = n as ::core::ffi::c_int;
            }
            101 => {
                (*buf).b_ind_open_imag = n as ::core::ffi::c_int;
            }
            110 => {
                (*buf).b_ind_no_brace = n as ::core::ffi::c_int;
            }
            102 => {
                (*buf).b_ind_first_open = n as ::core::ffi::c_int;
            }
            123 => {
                (*buf).b_ind_open_extra = n as ::core::ffi::c_int;
            }
            125 => {
                (*buf).b_ind_close_extra = n as ::core::ffi::c_int;
            }
            94 => {
                (*buf).b_ind_open_left_imag = n as ::core::ffi::c_int;
            }
            76 => {
                (*buf).b_ind_jump_label = n as ::core::ffi::c_int;
            }
            58 => {
                (*buf).b_ind_case = n as ::core::ffi::c_int;
            }
            61 => {
                (*buf).b_ind_case_code = n as ::core::ffi::c_int;
            }
            98 => {
                (*buf).b_ind_case_break = n as ::core::ffi::c_int;
            }
            112 => {
                (*buf).b_ind_param = n as ::core::ffi::c_int;
            }
            116 => {
                (*buf).b_ind_func_type = n as ::core::ffi::c_int;
            }
            47 => {
                (*buf).b_ind_comment = n as ::core::ffi::c_int;
            }
            99 => {
                (*buf).b_ind_in_comment = n as ::core::ffi::c_int;
            }
            67 => {
                (*buf).b_ind_in_comment2 = n as ::core::ffi::c_int;
            }
            105 => {
                (*buf).b_ind_cpp_baseclass = n as ::core::ffi::c_int;
            }
            43 => {
                (*buf).b_ind_continuation = n as ::core::ffi::c_int;
            }
            40 => {
                (*buf).b_ind_unclosed = n as ::core::ffi::c_int;
            }
            117 => {
                (*buf).b_ind_unclosed2 = n as ::core::ffi::c_int;
            }
            85 => {
                (*buf).b_ind_unclosed_noignore = n as ::core::ffi::c_int;
            }
            87 => {
                (*buf).b_ind_unclosed_wrapped = n as ::core::ffi::c_int;
            }
            119 => {
                (*buf).b_ind_unclosed_whiteok = n as ::core::ffi::c_int;
            }
            109 => {
                (*buf).b_ind_matching_paren = n as ::core::ffi::c_int;
            }
            77 => {
                (*buf).b_ind_paren_prev = n as ::core::ffi::c_int;
            }
            41 => {
                (*buf).b_ind_maxparen = n as ::core::ffi::c_int;
            }
            42 => {
                (*buf).b_ind_maxcomment = n as ::core::ffi::c_int;
            }
            103 => {
                (*buf).b_ind_scopedecl = n as ::core::ffi::c_int;
            }
            104 => {
                (*buf).b_ind_scopedecl_code = n as ::core::ffi::c_int;
            }
            106 => {
                (*buf).b_ind_java = n as ::core::ffi::c_int;
            }
            74 => {
                (*buf).b_ind_js = n as ::core::ffi::c_int;
            }
            108 => {
                (*buf).b_ind_keep_case_label = n as ::core::ffi::c_int;
            }
            35 => {
                (*buf).b_ind_hash_comment = n as ::core::ffi::c_int;
            }
            78 => {
                (*buf).b_ind_cpp_namespace = n as ::core::ffi::c_int;
            }
            107 => {
                (*buf).b_ind_if_for_while = n as ::core::ffi::c_int;
            }
            69 => {
                (*buf).b_ind_cpp_extern_c = n as ::core::ffi::c_int;
            }
            80 => {
                (*buf).b_ind_pragma = n as ::core::ffi::c_int;
            }
            _ => {}
        }
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
        }
    }
}
pub unsafe extern "C" fn get_c_indent() -> ::core::ffi::c_int {
    let mut cur_curpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut amount: ::core::ffi::c_int = 0;
    let mut scope_amount: ::core::ffi::c_int = 0;
    let mut cur_amount: ::core::ffi::c_int = MAXCOL as ::core::ffi::c_int;
    let mut col: colnr_T = 0;
    let mut theline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut linecopy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut comment_pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut tryposBrace: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut tryposCopy: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut our_paren_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut start_brace: ::core::ffi::c_int = 0;
    let mut ourscope: linenr_T = 0;
    let mut l: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut look: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut terminated: ::core::ffi::c_char = 0;
    let mut lookfor: ::core::ffi::c_int = 0;
    let mut whilelevel: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut lookfor_break: ::core::ffi::c_int = 0;
    let mut lookfor_cpp_namespace: bool = false_0 != 0;
    let mut cont_amount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut original_line_islabel: ::core::ffi::c_int = 0;
    let mut added_to_amount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut raw_string_start: linenr_T = 0 as linenr_T;
    let mut cache_cpp_baseclass: cpp_baseclass_cache_T = cpp_baseclass_cache_T {
        found: false_0,
        lpos: lpos_T {
            lnum: MAXLNUM as ::core::ffi::c_int as linenr_T,
            col: 0 as colnr_T,
        },
    };
    let mut ind_continuation: ::core::ffi::c_int = (*curbuf.get()).b_ind_continuation;
    cur_curpos = (*curwin.get()).w_cursor;
    if cur_curpos.lnum == 1 as linenr_T {
        return 0 as ::core::ffi::c_int;
    }
    linecopy = xstrdup(ml_get(cur_curpos.lnum));
    if State.get() & MODE_INSERT as ::core::ffi::c_int != 0
        && (*curwin.get()).w_cursor.col < strlen(linecopy) as colnr_T
        && *linecopy.offset((*curwin.get()).w_cursor.col as isize) as ::core::ffi::c_int
            == ')' as ::core::ffi::c_int
    {
        *linecopy.offset((*curwin.get()).w_cursor.col as isize) = NUL as ::core::ffi::c_char;
    }
    theline = skipwhite(linecopy);
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    original_line_islabel = cin_islabel() as ::core::ffi::c_int;
    comment_pos = ind_find_start_comment();
    if !comment_pos.is_null() {
        tryposCopy = *comment_pos;
        comment_pos = &raw mut tryposCopy;
    }
    trypos = find_start_rawstring((*curbuf.get()).b_ind_maxcomment);
    if !trypos.is_null()
        && (comment_pos.is_null() || lt(*trypos, *comment_pos) as ::core::ffi::c_int != 0)
    {
        amount = -1 as ::core::ffi::c_int;
    } else {
        '_theend: {
            if *theline as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                && (*linecopy as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                    || in_cinkeys(
                        '#' as ::core::ffi::c_int,
                        ' ' as ::core::ffi::c_int,
                        true_0 != 0,
                    ) as ::core::ffi::c_int
                        != 0)
            {
                let directive: *const ::core::ffi::c_char =
                    skipwhite(theline.offset(1 as ::core::ffi::c_int as isize));
                if (*curbuf.get()).b_ind_pragma == 0 as ::core::ffi::c_int
                    || strncmp(
                        directive,
                        b"pragma\0".as_ptr() as *const ::core::ffi::c_char,
                        6 as size_t,
                    ) != 0 as ::core::ffi::c_int
                {
                    amount = (*curbuf.get()).b_ind_hash_comment;
                    break '_theend;
                }
            }
            if original_line_islabel != 0
                && (*curbuf.get()).b_ind_js == 0
                && (*curbuf.get()).b_ind_jump_label < 0 as ::core::ffi::c_int
            {
                amount = 0 as ::core::ffi::c_int;
            } else {
                if cin_islinecomment(theline) != 0 {
                    let mut linecomment_pos: pos_T = pos_T {
                        lnum: 0,
                        col: 0,
                        coladd: 0,
                    };
                    trypos = find_line_comment();
                    if trypos.is_null() && (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                        linecomment_pos.col = check_linecomment(ml_get(
                            (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                        )) as colnr_T;
                        if linecomment_pos.col != MAXCOL as ::core::ffi::c_int {
                            trypos = &raw mut linecomment_pos;
                            (*trypos).lnum = (*curwin.get()).w_cursor.lnum - 1 as linenr_T;
                        }
                    }
                    if !trypos.is_null() {
                        getvcol(
                            curwin.get(),
                            trypos,
                            &raw mut col,
                            ::core::ptr::null_mut::<colnr_T>(),
                            ::core::ptr::null_mut::<colnr_T>(),
                        );
                        amount = col as ::core::ffi::c_int;
                        break '_theend;
                    }
                }
                if cin_iscomment(theline) == 0 && !comment_pos.is_null() {
                    let mut lead_start_len: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
                    let mut lead_middle_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    let mut lead_start: [::core::ffi::c_char; 50] = [0; 50];
                    let mut lead_middle: [::core::ffi::c_char; 50] = [0; 50];
                    let mut lead_end: [::core::ffi::c_char; 50] = [0; 50];
                    let mut lead_end_len: ::core::ffi::c_int = 0;
                    let mut p: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut start_align: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut start_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut done: ::core::ffi::c_int = false_0;
                    getvcol(
                        curwin.get(),
                        comment_pos,
                        &raw mut col,
                        ::core::ptr::null_mut::<colnr_T>(),
                        ::core::ptr::null_mut::<colnr_T>(),
                    );
                    amount = col as ::core::ffi::c_int;
                    *(&raw mut lead_start as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
                    *(&raw mut lead_middle as *mut ::core::ffi::c_char) =
                        NUL as ::core::ffi::c_char;
                    p = (*curbuf.get()).b_p_com;
                    while *p as ::core::ffi::c_int != NUL {
                        let mut align: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut what: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while *p as ::core::ffi::c_int != NUL
                            && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                        {
                            if *p as ::core::ffi::c_int == COM_START
                                || *p as ::core::ffi::c_int == COM_END
                                || *p as ::core::ffi::c_int == COM_MIDDLE
                            {
                                let c2rust_fresh1 = p;
                                p = p.offset(1);
                                what = *c2rust_fresh1 as ::core::ffi::c_uchar as ::core::ffi::c_int;
                            } else if *p as ::core::ffi::c_int == COM_LEFT
                                || *p as ::core::ffi::c_int == COM_RIGHT
                            {
                                let c2rust_fresh2 = p;
                                p = p.offset(1);
                                align =
                                    *c2rust_fresh2 as ::core::ffi::c_uchar as ::core::ffi::c_int;
                            } else if ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int
                                != 0
                                || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                            {
                                off =
                                    getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
                            } else {
                                p = p.offset(1);
                            }
                        }
                        if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                            p = p.offset(1);
                        }
                        lead_end_len = copy_option_part(
                            &raw mut p,
                            &raw mut lead_end as *mut ::core::ffi::c_char,
                            COM_MAX_LEN as size_t,
                            b",\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        ) as ::core::ffi::c_int;
                        if what == COM_START {
                            strcpy(
                                &raw mut lead_start as *mut ::core::ffi::c_char,
                                &raw mut lead_end as *mut ::core::ffi::c_char,
                            );
                            lead_start_len = lead_end_len;
                            start_off = off;
                            start_align = align;
                        } else if what == COM_MIDDLE {
                            strcpy(
                                &raw mut lead_middle as *mut ::core::ffi::c_char,
                                &raw mut lead_end as *mut ::core::ffi::c_char,
                            );
                            lead_middle_len = lead_end_len;
                        } else {
                            if what != COM_END {
                                continue;
                            }
                            if strncmp(
                                theline,
                                &raw mut lead_middle as *mut ::core::ffi::c_char,
                                lead_middle_len as size_t,
                            ) == 0 as ::core::ffi::c_int
                                && strncmp(
                                    theline,
                                    &raw mut lead_end as *mut ::core::ffi::c_char,
                                    lead_end_len as size_t,
                                ) != 0 as ::core::ffi::c_int
                            {
                                done = true_0;
                                if (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                                    look = skipwhite(ml_get(
                                        (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                                    ));
                                    if strncmp(
                                        look,
                                        &raw mut lead_start as *mut ::core::ffi::c_char,
                                        lead_start_len as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        amount = get_indent_lnum(
                                            (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                                        );
                                    } else if strncmp(
                                        look,
                                        &raw mut lead_middle as *mut ::core::ffi::c_char,
                                        lead_middle_len as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        amount = get_indent_lnum(
                                            (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                                        );
                                        break;
                                    } else if strncmp(
                                        ml_get((*comment_pos).lnum)
                                            .offset((*comment_pos).col as isize),
                                        &raw mut lead_start as *mut ::core::ffi::c_char,
                                        lead_start_len as size_t,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        continue;
                                    }
                                }
                                if start_off != 0 as ::core::ffi::c_int {
                                    amount += start_off;
                                } else if start_align == COM_RIGHT {
                                    amount += vim_strsize(
                                        &raw mut lead_start as *mut ::core::ffi::c_char,
                                    ) - vim_strsize(
                                        &raw mut lead_middle as *mut ::core::ffi::c_char,
                                    );
                                }
                                break;
                            } else {
                                if !(strncmp(
                                    theline,
                                    &raw mut lead_middle as *mut ::core::ffi::c_char,
                                    lead_middle_len as size_t,
                                ) != 0 as ::core::ffi::c_int
                                    && strncmp(
                                        theline,
                                        &raw mut lead_end as *mut ::core::ffi::c_char,
                                        lead_end_len as size_t,
                                    ) == 0 as ::core::ffi::c_int)
                                {
                                    continue;
                                }
                                amount =
                                    get_indent_lnum((*curwin.get()).w_cursor.lnum - 1 as linenr_T);
                                if off != 0 as ::core::ffi::c_int {
                                    amount += off;
                                } else if align == COM_RIGHT {
                                    amount += vim_strsize(
                                        &raw mut lead_start as *mut ::core::ffi::c_char,
                                    ) - vim_strsize(
                                        &raw mut lead_middle as *mut ::core::ffi::c_char,
                                    );
                                }
                                done = true_0;
                                break;
                            }
                        }
                    }
                    if done == 0 {
                        if *theline.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '*' as ::core::ffi::c_int
                        {
                            amount += 1 as ::core::ffi::c_int;
                        } else {
                            amount = -1 as ::core::ffi::c_int;
                            lnum = cur_curpos.lnum - 1 as linenr_T;
                            while lnum > (*comment_pos).lnum {
                                if linewhite(lnum) {
                                    lnum -= 1;
                                } else {
                                    amount = get_indent_lnum(lnum);
                                    break;
                                }
                            }
                            if amount == -1 as ::core::ffi::c_int {
                                if (*curbuf.get()).b_ind_in_comment2 == 0 {
                                    start = ml_get((*comment_pos).lnum);
                                    look = start
                                        .offset((*comment_pos).col as isize)
                                        .offset(2 as ::core::ffi::c_int as isize);
                                    if *look as ::core::ffi::c_int != NUL {
                                        (*comment_pos).col =
                                            skipwhite(look).offset_from(start) as colnr_T;
                                    }
                                }
                                getvcol(
                                    curwin.get(),
                                    comment_pos,
                                    &raw mut col,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                    ::core::ptr::null_mut::<colnr_T>(),
                                );
                                amount = col as ::core::ffi::c_int;
                                if (*curbuf.get()).b_ind_in_comment2 != 0
                                    || *look as ::core::ffi::c_int == NUL
                                {
                                    amount += (*curbuf.get()).b_ind_in_comment;
                                }
                            }
                        }
                    }
                } else if *skipwhite(theline) as ::core::ffi::c_int == ']' as ::core::ffi::c_int
                    && {
                        trypos = find_match_char(
                            '[' as ::core::ffi::c_char,
                            (*curbuf.get()).b_ind_maxparen,
                        );
                        !trypos.is_null()
                    }
                {
                    amount = get_indent_lnum((*trypos).lnum);
                } else {
                    trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                    if !trypos.is_null() && (*curbuf.get()).b_ind_java == 0 as ::core::ffi::c_int
                        || {
                            tryposBrace = find_start_brace();
                            !tryposBrace.is_null()
                        }
                        || !trypos.is_null()
                    {
                        if !trypos.is_null() && !tryposBrace.is_null() {
                            if if (*trypos).lnum != (*tryposBrace).lnum {
                                ((*trypos).lnum < (*tryposBrace).lnum) as ::core::ffi::c_int
                            } else {
                                ((*trypos).col < (*tryposBrace).col) as ::core::ffi::c_int
                            } != 0
                            {
                                trypos = ::core::ptr::null_mut::<pos_T>();
                            } else {
                                tryposBrace = ::core::ptr::null_mut::<pos_T>();
                            }
                        }
                        if !trypos.is_null() {
                            our_paren_pos = *trypos;
                            if *theline.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == ')' as ::core::ffi::c_int
                                && (*curbuf.get()).b_ind_paren_prev != 0
                            {
                                amount =
                                    get_indent_lnum((*curwin.get()).w_cursor.lnum - 1 as linenr_T);
                            } else {
                                amount = -1 as ::core::ffi::c_int;
                                lnum = cur_curpos.lnum - 1 as linenr_T;
                                while lnum > our_paren_pos.lnum {
                                    l = skipwhite(ml_get(lnum));
                                    if cin_nocode(l) == 0 {
                                        if cin_ispreproc_cont(
                                            &raw mut l,
                                            &raw mut lnum,
                                            &raw mut amount,
                                        ) == 0
                                        {
                                            (*curwin.get()).w_cursor.lnum = lnum;
                                            trypos = ind_find_start_CORS(::core::ptr::null_mut::<
                                                linenr_T,
                                            >(
                                            ));
                                            if !trypos.is_null() {
                                                lnum = (*trypos).lnum + 1 as linenr_T;
                                            } else {
                                                trypos = find_match_paren(corr_ind_maxparen(
                                                    &raw mut cur_curpos,
                                                ));
                                                if !trypos.is_null()
                                                    && (*trypos).lnum == our_paren_pos.lnum
                                                    && (*trypos).col == our_paren_pos.col
                                                {
                                                    amount = get_indent_lnum(lnum);
                                                    if *theline
                                                        .offset(0 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == ')' as ::core::ffi::c_int
                                                    {
                                                        if our_paren_pos.lnum != lnum
                                                            && cur_amount > amount
                                                        {
                                                            cur_amount = amount;
                                                        }
                                                        amount = -1 as ::core::ffi::c_int;
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    lnum -= 1;
                                }
                            }
                            if amount == -1 as ::core::ffi::c_int {
                                let mut ignore_paren_col: ::core::ffi::c_int =
                                    0 as ::core::ffi::c_int;
                                let mut is_if_for_while: ::core::ffi::c_int =
                                    0 as ::core::ffi::c_int;
                                if (*curbuf.get()).b_ind_if_for_while != 0 {
                                    let mut cursor_save: pos_T = (*curwin.get()).w_cursor;
                                    let mut outermost: pos_T = pos_T {
                                        lnum: 0,
                                        col: 0,
                                        coladd: 0,
                                    };
                                    let mut line: *mut ::core::ffi::c_char =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    trypos = &raw mut our_paren_pos;
                                    loop {
                                        outermost = *trypos;
                                        (*curwin.get()).w_cursor.lnum = outermost.lnum;
                                        (*curwin.get()).w_cursor.col = outermost.col;
                                        trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                                        if !(!trypos.is_null() && (*trypos).lnum == outermost.lnum)
                                        {
                                            break;
                                        }
                                    }
                                    (*curwin.get()).w_cursor = cursor_save;
                                    line = ml_get(outermost.lnum);
                                    is_if_for_while = cin_is_if_for_while_before_offset(
                                        line,
                                        &raw mut outermost.col,
                                    );
                                }
                                amount = skip_label(our_paren_pos.lnum, &raw mut look);
                                look = skipwhite(look);
                                if *look as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
                                    let mut save_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
                                    let mut line_0: *mut ::core::ffi::c_char =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    let mut look_col: ::core::ffi::c_int = 0;
                                    (*curwin.get()).w_cursor.lnum = our_paren_pos.lnum;
                                    line_0 = get_cursor_line_ptr();
                                    look_col = look.offset_from(line_0) as ::core::ffi::c_int;
                                    (*curwin.get()).w_cursor.col =
                                        (look_col + 1 as ::core::ffi::c_int) as colnr_T;
                                    trypos = findmatchlimit(
                                        ::core::ptr::null_mut::<oparg_T>(),
                                        ')' as ::core::ffi::c_int,
                                        0 as ::core::ffi::c_int,
                                        (*curbuf.get()).b_ind_maxparen as int64_t,
                                    );
                                    if !trypos.is_null()
                                        && (*trypos).lnum == our_paren_pos.lnum
                                        && (*trypos).col < our_paren_pos.col
                                    {
                                        ignore_paren_col = (*trypos).col as ::core::ffi::c_int
                                            + 1 as ::core::ffi::c_int;
                                    }
                                    (*curwin.get()).w_cursor.lnum = save_lnum;
                                    look = ml_get(our_paren_pos.lnum).offset(look_col as isize);
                                }
                                if *theline.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ')' as ::core::ffi::c_int
                                    || (*curbuf.get()).b_ind_unclosed == 0 as ::core::ffi::c_int
                                        && is_if_for_while == 0 as ::core::ffi::c_int
                                    || (*curbuf.get()).b_ind_unclosed_noignore == 0
                                        && *look as ::core::ffi::c_int == '(' as ::core::ffi::c_int
                                        && ignore_paren_col == 0 as ::core::ffi::c_int
                                {
                                    if *theline.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        != ')' as ::core::ffi::c_int
                                    {
                                        cur_amount = MAXCOL as ::core::ffi::c_int;
                                        l = ml_get(our_paren_pos.lnum);
                                        if (*curbuf.get()).b_ind_unclosed_wrapped != 0
                                            && cin_ends_in(
                                                l,
                                                b"(\0".as_ptr() as *const ::core::ffi::c_char,
                                            ) != 0
                                        {
                                            n = 1 as ::core::ffi::c_int;
                                            col = 0 as ::core::ffi::c_int as colnr_T;
                                            while col < our_paren_pos.col {
                                                match *l.offset(col as isize) as ::core::ffi::c_int
                                                {
                                                    40 | 123 => {
                                                        n += 1;
                                                    }
                                                    41 | 125 => {
                                                        if n > 1 as ::core::ffi::c_int {
                                                            n -= 1;
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                                col += 1;
                                            }
                                            our_paren_pos.col = 0 as ::core::ffi::c_int as colnr_T;
                                            amount += n * (*curbuf.get()).b_ind_unclosed_wrapped;
                                        } else if (*curbuf.get()).b_ind_unclosed_whiteok != 0 {
                                            our_paren_pos.col += 1;
                                        } else {
                                            col = (our_paren_pos.col as ::core::ffi::c_int
                                                + 1 as ::core::ffi::c_int)
                                                as colnr_T;
                                            while ascii_iswhite(
                                                *l.offset(col as isize) as ::core::ffi::c_int
                                            ) {
                                                col += 1;
                                            }
                                            if *l.offset(col as isize) as ::core::ffi::c_int != NUL
                                            {
                                                our_paren_pos.col = col;
                                            } else {
                                                our_paren_pos.col += 1;
                                            }
                                        }
                                    }
                                    if our_paren_pos.col > 0 as ::core::ffi::c_int {
                                        getvcol(
                                            curwin.get(),
                                            &raw mut our_paren_pos,
                                            &raw mut col,
                                            ::core::ptr::null_mut::<colnr_T>(),
                                            ::core::ptr::null_mut::<colnr_T>(),
                                        );
                                        if cur_amount > col {
                                            cur_amount = col as ::core::ffi::c_int;
                                        }
                                    }
                                }
                                if !(*theline.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ')' as ::core::ffi::c_int
                                    && (*curbuf.get()).b_ind_matching_paren != 0)
                                {
                                    if (*curbuf.get()).b_ind_unclosed == 0 as ::core::ffi::c_int
                                        && is_if_for_while == 0 as ::core::ffi::c_int
                                        || (*curbuf.get()).b_ind_unclosed_noignore == 0
                                            && *look as ::core::ffi::c_int
                                                == '(' as ::core::ffi::c_int
                                            && ignore_paren_col == 0 as ::core::ffi::c_int
                                    {
                                        if cur_amount != MAXCOL as ::core::ffi::c_int {
                                            amount = cur_amount;
                                        }
                                    } else {
                                        col = our_paren_pos.col;
                                        while our_paren_pos.col > ignore_paren_col {
                                            our_paren_pos.col -= 1;
                                            match *ml_get_pos(&raw mut our_paren_pos)
                                                as ::core::ffi::c_int
                                            {
                                                40 => {
                                                    amount += (*curbuf.get()).b_ind_unclosed2;
                                                    col = our_paren_pos.col;
                                                }
                                                41 => {
                                                    amount -= (*curbuf.get()).b_ind_unclosed2;
                                                    col = MAXCOL as ::core::ffi::c_int as colnr_T;
                                                }
                                                _ => {}
                                            }
                                        }
                                        if col == MAXCOL as ::core::ffi::c_int {
                                            amount += (*curbuf.get()).b_ind_unclosed;
                                        } else {
                                            (*curwin.get()).w_cursor.lnum = our_paren_pos.lnum;
                                            (*curwin.get()).w_cursor.col = col;
                                            if !find_match_paren_after_brace(
                                                (*curbuf.get()).b_ind_maxparen,
                                            )
                                            .is_null()
                                            {
                                                amount += (*curbuf.get()).b_ind_unclosed2;
                                            } else if is_if_for_while != 0 {
                                                amount += (*curbuf.get()).b_ind_if_for_while;
                                            } else {
                                                amount += (*curbuf.get()).b_ind_unclosed;
                                            }
                                        }
                                        if cur_amount < amount {
                                            amount = cur_amount;
                                        }
                                    }
                                }
                            }
                            if cin_iscomment(theline) != 0 {
                                amount += (*curbuf.get()).b_ind_comment;
                            }
                        } else {
                            tryposCopy = *tryposBrace;
                            tryposBrace = &raw mut tryposCopy;
                            trypos = tryposBrace;
                            ourscope = (*trypos).lnum;
                            start = ml_get(ourscope);
                            look = skipwhite(start);
                            if *look as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                                getvcol(
                                    curwin.get(),
                                    trypos,
                                    &raw mut col,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                    ::core::ptr::null_mut::<colnr_T>(),
                                );
                                amount = col as ::core::ffi::c_int;
                                if *start as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                                    start_brace = BRACE_IN_COL0;
                                } else {
                                    start_brace = BRACE_AT_START;
                                }
                            } else {
                                (*curwin.get()).w_cursor.lnum = ourscope;
                                lnum = ourscope;
                                if find_last_paren(
                                    start,
                                    '(' as ::core::ffi::c_char,
                                    ')' as ::core::ffi::c_char,
                                ) != 0
                                    && {
                                        trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                                        !trypos.is_null()
                                    }
                                {
                                    lnum = (*trypos).lnum;
                                }
                                if ((*curbuf.get()).b_ind_js != 0
                                    || (*curbuf.get()).b_ind_keep_case_label != 0)
                                    && cin_iscase(skipwhite(get_cursor_line_ptr()), false_0 != 0)
                                        as ::core::ffi::c_int
                                        != 0
                                {
                                    amount = get_indent();
                                } else if (*curbuf.get()).b_ind_js != 0 {
                                    amount = get_indent_lnum(lnum);
                                } else {
                                    amount = skip_label(lnum, &raw mut l);
                                }
                                start_brace = BRACE_AT_END;
                            }
                            let mut js_cur_has_key: bool = if (*curbuf.get()).b_ind_js != 0 {
                                cin_has_js_key(theline) as ::core::ffi::c_int
                            } else {
                                false_0
                            } != 0;
                            if *theline.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == '}' as ::core::ffi::c_int
                            {
                                amount += (*curbuf.get()).b_ind_close_extra;
                            } else {
                                lookfor = LOOKFOR_INITIAL;
                                if cin_iselse(theline) != 0 {
                                    lookfor = LOOKFOR_IF;
                                } else if cin_iswhileofdo(theline, cur_curpos.lnum) != 0 {
                                    lookfor = LOOKFOR_DO;
                                }
                                if lookfor != LOOKFOR_INITIAL {
                                    (*curwin.get()).w_cursor.lnum = cur_curpos.lnum;
                                    if find_match(lookfor, ourscope) == OK {
                                        amount = get_indent();
                                        break '_theend;
                                    }
                                }
                                if start_brace == BRACE_IN_COL0 {
                                    amount = (*curbuf.get()).b_ind_open_left_imag;
                                    lookfor_cpp_namespace = true_0 != 0;
                                } else if start_brace == BRACE_AT_START
                                    && lookfor_cpp_namespace as ::core::ffi::c_int != 0
                                {
                                    lookfor_cpp_namespace = true_0 != 0;
                                } else if start_brace == BRACE_AT_END {
                                    amount += (*curbuf.get()).b_ind_open_imag;
                                    l = skipwhite(get_cursor_line_ptr());
                                    if cin_is_cpp_namespace(l) {
                                        amount += (*curbuf.get()).b_ind_cpp_namespace;
                                    } else if cin_is_cpp_extern_c(l) != 0 {
                                        amount += (*curbuf.get()).b_ind_cpp_extern_c;
                                    }
                                } else {
                                    amount -= (*curbuf.get()).b_ind_open_extra;
                                    if amount < 0 as ::core::ffi::c_int {
                                        amount = 0 as ::core::ffi::c_int;
                                    }
                                }
                                lookfor_break = false_0;
                                if cin_iscase(theline, false_0 != 0) {
                                    lookfor = LOOKFOR_CASE;
                                    amount += (*curbuf.get()).b_ind_case;
                                } else if cin_isscopedecl(theline) {
                                    lookfor = LOOKFOR_SCOPEDECL;
                                    amount += (*curbuf.get()).b_ind_scopedecl;
                                } else {
                                    if (*curbuf.get()).b_ind_case_break != 0
                                        && cin_isbreak(theline) != 0
                                    {
                                        lookfor_break = true_0;
                                    }
                                    lookfor = LOOKFOR_INITIAL;
                                    amount += (*curbuf.get()).b_ind_level;
                                }
                                scope_amount = amount;
                                whilelevel = 0 as ::core::ffi::c_int;
                                (*curwin.get()).w_cursor = cur_curpos;
                                's_2927: loop {
                                    (*curwin.get()).w_cursor.lnum -= 1;
                                    (*curwin.get()).w_cursor.col =
                                        0 as ::core::ffi::c_int as colnr_T;
                                    if (*curwin.get()).w_cursor.lnum <= ourscope {
                                        if lookfor == LOOKFOR_ENUM_OR_INIT {
                                            if (*curwin.get()).w_cursor.lnum == 0 as linenr_T
                                                || (*curwin.get()).w_cursor.lnum
                                                    < ourscope
                                                        - (*curbuf.get()).b_ind_maxparen as linenr_T
                                            {
                                                if cont_amount > 0 as ::core::ffi::c_int {
                                                    amount = cont_amount;
                                                } else if (*curbuf.get()).b_ind_js == 0 {
                                                    amount += ind_continuation;
                                                }
                                                break;
                                            } else {
                                                trypos =
                                                    ind_find_start_CORS(::core::ptr::null_mut::<
                                                        linenr_T,
                                                    >(
                                                    ));
                                                if !trypos.is_null() {
                                                    (*curwin.get()).w_cursor.lnum =
                                                        (*trypos).lnum + 1 as linenr_T;
                                                    (*curwin.get()).w_cursor.col =
                                                        0 as ::core::ffi::c_int as colnr_T;
                                                } else {
                                                    l = get_cursor_line_ptr();
                                                    if cin_ispreproc_cont(
                                                        &raw mut l,
                                                        &raw mut (*curwin.get()).w_cursor.lnum,
                                                        &raw mut amount,
                                                    ) != 0
                                                    {
                                                        continue;
                                                    }
                                                    if cin_nocode(l) != 0 {
                                                        continue;
                                                    }
                                                    terminated =
                                                        cin_isterminated(l, false_0, true_0);
                                                    if start_brace != BRACE_IN_COL0
                                                        || cin_isfuncdecl(
                                                            &raw mut l,
                                                            (*curwin.get()).w_cursor.lnum,
                                                            0 as linenr_T,
                                                        ) == 0
                                                    {
                                                        if terminated as ::core::ffi::c_int
                                                            == ',' as ::core::ffi::c_int
                                                        {
                                                            break;
                                                        }
                                                        if terminated as ::core::ffi::c_int
                                                            != ';' as ::core::ffi::c_int
                                                            && cin_isinit() as ::core::ffi::c_int
                                                                != 0
                                                        {
                                                            break;
                                                        }
                                                        if terminated as ::core::ffi::c_int
                                                            == 0 as ::core::ffi::c_int
                                                            || terminated as ::core::ffi::c_int
                                                                == '{' as ::core::ffi::c_int
                                                        {
                                                            continue;
                                                        }
                                                    }
                                                    if terminated as ::core::ffi::c_int
                                                        != ';' as ::core::ffi::c_int
                                                    {
                                                        trypos = ::core::ptr::null_mut::<pos_T>();
                                                        if find_last_paren(
                                                            l,
                                                            '(' as ::core::ffi::c_char,
                                                            ')' as ::core::ffi::c_char,
                                                        ) != 0
                                                        {
                                                            trypos = find_match_paren(
                                                                (*curbuf.get()).b_ind_maxparen,
                                                            );
                                                        }
                                                        if trypos.is_null()
                                                            && find_last_paren(
                                                                l,
                                                                '{' as ::core::ffi::c_char,
                                                                '}' as ::core::ffi::c_char,
                                                            ) != 0
                                                        {
                                                            trypos = find_start_brace();
                                                        }
                                                        if !trypos.is_null() {
                                                            (*curwin.get()).w_cursor.lnum =
                                                                (*trypos).lnum + 1 as linenr_T;
                                                            (*curwin.get()).w_cursor.col =
                                                                0 as ::core::ffi::c_int as colnr_T;
                                                            continue;
                                                        }
                                                    }
                                                    if cont_amount > 0 as ::core::ffi::c_int {
                                                        amount = cont_amount;
                                                    } else {
                                                        amount += ind_continuation;
                                                    }
                                                    break;
                                                }
                                            }
                                        } else if lookfor == LOOKFOR_UNTERM {
                                            if cont_amount > 0 as ::core::ffi::c_int {
                                                amount = cont_amount;
                                            } else {
                                                amount += ind_continuation;
                                            }
                                            break;
                                        } else {
                                            if lookfor != LOOKFOR_TERM
                                                && lookfor != LOOKFOR_CPP_BASECLASS
                                                && lookfor != LOOKFOR_COMMA
                                            {
                                                amount = scope_amount;
                                                if *theline.offset(0 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '{' as ::core::ffi::c_int
                                                {
                                                    amount += (*curbuf.get()).b_ind_open_extra;
                                                    added_to_amount =
                                                        (*curbuf.get()).b_ind_open_extra;
                                                }
                                            }
                                            if !lookfor_cpp_namespace {
                                                break;
                                            }
                                            if (*curwin.get()).w_cursor.lnum == ourscope {
                                                continue;
                                            }
                                            if (*curwin.get()).w_cursor.lnum == 0 as linenr_T
                                                || (*curwin.get()).w_cursor.lnum
                                                    < ourscope - FIND_NAMESPACE_LIM as linenr_T
                                            {
                                                break;
                                            }
                                            trypos = ind_find_start_CORS(::core::ptr::null_mut::<
                                                linenr_T,
                                            >(
                                            ));
                                            if !trypos.is_null() {
                                                (*curwin.get()).w_cursor.lnum =
                                                    (*trypos).lnum + 1 as linenr_T;
                                                (*curwin.get()).w_cursor.col =
                                                    0 as ::core::ffi::c_int as colnr_T;
                                            } else {
                                                l = get_cursor_line_ptr();
                                                if cin_ispreproc_cont(
                                                    &raw mut l,
                                                    &raw mut (*curwin.get()).w_cursor.lnum,
                                                    &raw mut amount,
                                                ) != 0
                                                {
                                                    continue;
                                                }
                                                if cin_is_cpp_namespace(l) {
                                                    amount += (*curbuf.get()).b_ind_cpp_namespace
                                                        - added_to_amount;
                                                    break;
                                                } else if cin_is_cpp_extern_c(l) != 0 {
                                                    amount += (*curbuf.get()).b_ind_cpp_extern_c
                                                        - added_to_amount;
                                                    break;
                                                } else if cin_nocode(l) == 0 {
                                                    break;
                                                }
                                            }
                                        }
                                    } else {
                                        trypos = ind_find_start_CORS(&raw mut raw_string_start);
                                        if !trypos.is_null() {
                                            (*curwin.get()).w_cursor.lnum =
                                                (*trypos).lnum + 1 as linenr_T;
                                            (*curwin.get()).w_cursor.col =
                                                0 as ::core::ffi::c_int as colnr_T;
                                        } else {
                                            l = get_cursor_line_ptr();
                                            let mut iscase: bool = cin_iscase(l, false_0 != 0);
                                            if iscase as ::core::ffi::c_int != 0
                                                || cin_isscopedecl(l) as ::core::ffi::c_int != 0
                                            {
                                                if lookfor == LOOKFOR_CPP_BASECLASS {
                                                    break;
                                                }
                                                if whilelevel > 0 as ::core::ffi::c_int {
                                                    continue;
                                                }
                                                if lookfor == LOOKFOR_UNTERM
                                                    || lookfor == LOOKFOR_ENUM_OR_INIT
                                                {
                                                    if cont_amount > 0 as ::core::ffi::c_int {
                                                        amount = cont_amount;
                                                    } else {
                                                        amount += ind_continuation;
                                                    }
                                                    break;
                                                } else if iscase as ::core::ffi::c_int != 0
                                                    && lookfor == LOOKFOR_CASE
                                                    || iscase as ::core::ffi::c_int != 0
                                                        && lookfor_break != 0
                                                    || !iscase && lookfor == LOOKFOR_SCOPEDECL
                                                {
                                                    trypos = find_start_brace();
                                                    if !(trypos.is_null()
                                                        || (*trypos).lnum == ourscope)
                                                    {
                                                        continue;
                                                    }
                                                    amount = get_indent();
                                                    break;
                                                } else {
                                                    n = get_indent_nolabel(
                                                        (*curwin.get()).w_cursor.lnum,
                                                    );
                                                    if lookfor == LOOKFOR_TERM {
                                                        if n != 0 {
                                                            amount = n;
                                                        }
                                                        if lookfor_break == 0 {
                                                            break;
                                                        }
                                                    }
                                                    if n != 0 {
                                                        amount = n;
                                                        l = after_label(get_cursor_line_ptr());
                                                        if !l.is_null()
                                                            && cin_is_cinword(l)
                                                                as ::core::ffi::c_int
                                                                != 0
                                                        {
                                                            if *theline.offset(
                                                                0 as ::core::ffi::c_int as isize,
                                                            )
                                                                as ::core::ffi::c_int
                                                                == '{' as ::core::ffi::c_int
                                                            {
                                                                amount += (*curbuf.get())
                                                                    .b_ind_open_extra;
                                                            } else {
                                                                amount += (*curbuf.get())
                                                                    .b_ind_level
                                                                    + (*curbuf.get())
                                                                        .b_ind_no_brace;
                                                            }
                                                        }
                                                        break;
                                                    } else {
                                                        scope_amount = get_indent()
                                                            + (if iscase as ::core::ffi::c_int != 0
                                                            {
                                                                (*curbuf.get()).b_ind_case_code
                                                            } else {
                                                                (*curbuf.get()).b_ind_scopedecl_code
                                                            });
                                                        lookfor = if (*curbuf.get())
                                                            .b_ind_case_break
                                                            != 0
                                                        {
                                                            LOOKFOR_NOBREAK
                                                        } else {
                                                            LOOKFOR_ANY
                                                        };
                                                    }
                                                }
                                            } else if lookfor == LOOKFOR_CASE
                                                || lookfor == LOOKFOR_SCOPEDECL
                                            {
                                                if find_last_paren(
                                                    l,
                                                    '{' as ::core::ffi::c_char,
                                                    '}' as ::core::ffi::c_char,
                                                ) != 0
                                                    && {
                                                        trypos = find_start_brace();
                                                        !trypos.is_null()
                                                    }
                                                {
                                                    (*curwin.get()).w_cursor.lnum =
                                                        (*trypos).lnum + 1 as linenr_T;
                                                    (*curwin.get()).w_cursor.col =
                                                        0 as ::core::ffi::c_int as colnr_T;
                                                }
                                            } else {
                                                if (*curbuf.get()).b_ind_js == 0
                                                    && cin_islabel() as ::core::ffi::c_int != 0
                                                {
                                                    l = after_label(get_cursor_line_ptr());
                                                    if l.is_null() || cin_nocode(l) != 0 {
                                                        continue;
                                                    }
                                                }
                                                l = get_cursor_line_ptr();
                                                if cin_ispreproc_cont(
                                                    &raw mut l,
                                                    &raw mut (*curwin.get()).w_cursor.lnum,
                                                    &raw mut amount,
                                                ) != 0
                                                    || cin_nocode(l) != 0
                                                {
                                                    continue;
                                                }
                                                n = 0 as ::core::ffi::c_int;
                                                if lookfor != LOOKFOR_TERM
                                                    && (*curbuf.get()).b_ind_cpp_baseclass
                                                        > 0 as ::core::ffi::c_int
                                                {
                                                    n = cin_is_cpp_baseclass(
                                                        &raw mut cache_cpp_baseclass,
                                                    );
                                                    l = get_cursor_line_ptr();
                                                }
                                                if n != 0 {
                                                    if lookfor == LOOKFOR_UNTERM {
                                                        if cont_amount > 0 as ::core::ffi::c_int {
                                                            amount = cont_amount;
                                                        } else {
                                                            amount += ind_continuation;
                                                        }
                                                        break;
                                                    } else if *theline
                                                        .offset(0 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == '{' as ::core::ffi::c_int
                                                    {
                                                        lookfor = LOOKFOR_UNTERM;
                                                        ind_continuation = 0 as ::core::ffi::c_int;
                                                    } else {
                                                        amount = get_baseclass_amount(
                                                            cache_cpp_baseclass.lpos.col
                                                                as ::core::ffi::c_int,
                                                        );
                                                        break;
                                                    }
                                                } else if lookfor == LOOKFOR_CPP_BASECLASS {
                                                    if cin_isterminated(l, true_0, false_0) != 0 {
                                                        break;
                                                    }
                                                } else {
                                                    terminated =
                                                        cin_isterminated(l, false_0, true_0);
                                                    if js_cur_has_key {
                                                        js_cur_has_key = false_0 != 0;
                                                        if (*curbuf.get()).b_ind_js != 0
                                                            && terminated as ::core::ffi::c_int
                                                                == ',' as ::core::ffi::c_int
                                                        {
                                                            lookfor = LOOKFOR_JS_KEY;
                                                        }
                                                    }
                                                    if lookfor == LOOKFOR_JS_KEY
                                                        && cin_has_js_key(l) as ::core::ffi::c_int
                                                            != 0
                                                    {
                                                        amount = get_indent();
                                                        break;
                                                    } else {
                                                        if lookfor == LOOKFOR_COMMA {
                                                            if !tryposBrace.is_null()
                                                                && (*tryposBrace).lnum
                                                                    >= (*curwin.get()).w_cursor.lnum
                                                            {
                                                                break;
                                                            }
                                                            if terminated as ::core::ffi::c_int
                                                                == ',' as ::core::ffi::c_int
                                                            {
                                                                break;
                                                            } else {
                                                                amount = get_indent();
                                                                if (*curwin.get()).w_cursor.lnum
                                                                    - 1 as linenr_T
                                                                    == ourscope
                                                                {
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        if terminated as ::core::ffi::c_int
                                                            == 0 as ::core::ffi::c_int
                                                            || lookfor != LOOKFOR_UNTERM
                                                                && terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                        {
                                                            if lookfor != LOOKFOR_ENUM_OR_INIT
                                                                && (*skipwhite(l)
                                                                    as ::core::ffi::c_int
                                                                    == '[' as ::core::ffi::c_int
                                                                    || *l.offset(
                                                                        strlen(l).wrapping_sub(
                                                                            1 as size_t,
                                                                        )
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '['
                                                                            as ::core::ffi::c_int)
                                                            {
                                                                amount += ind_continuation;
                                                            }
                                                            find_last_paren(
                                                                l,
                                                                '(' as ::core::ffi::c_char,
                                                                ')' as ::core::ffi::c_char,
                                                            );
                                                            trypos = find_match_paren(
                                                                corr_ind_maxparen(
                                                                    &raw mut cur_curpos,
                                                                ),
                                                            );
                                                            if !trypos.is_null()
                                                                && ((*trypos).lnum
                                                                    < (*tryposBrace).lnum
                                                                    || (*trypos).lnum
                                                                        == (*tryposBrace).lnum
                                                                        && (*trypos).col
                                                                            < (*tryposBrace).col)
                                                            {
                                                                trypos =
                                                                    ::core::ptr::null_mut::<pos_T>(
                                                                    );
                                                            }
                                                            l = get_cursor_line_ptr();
                                                            if trypos.is_null()
                                                                && terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                            {
                                                                if find_last_paren(
                                                                    l,
                                                                    '{' as ::core::ffi::c_char,
                                                                    '}' as ::core::ffi::c_char,
                                                                ) != 0
                                                                {
                                                                    trypos = find_start_brace();
                                                                }
                                                                l = get_cursor_line_ptr();
                                                            }
                                                            if !trypos.is_null() {
                                                                (*curwin.get()).w_cursor = *trypos;
                                                                l = get_cursor_line_ptr();
                                                                if cin_iscase(l, false_0 != 0)
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                    || cin_isscopedecl(l)
                                                                        as ::core::ffi::c_int
                                                                        != 0
                                                                {
                                                                    (*curwin.get())
                                                                        .w_cursor
                                                                        .lnum += 1;
                                                                    (*curwin.get()).w_cursor.col = 0
                                                                        as ::core::ffi::c_int
                                                                        as colnr_T;
                                                                    continue;
                                                                }
                                                            }
                                                            if terminated as ::core::ffi::c_int
                                                                == ',' as ::core::ffi::c_int
                                                            {
                                                                while (*curwin.get()).w_cursor.lnum
                                                                    > 1 as linenr_T
                                                                {
                                                                    l = ml_get(
                                                                        (*curwin.get())
                                                                            .w_cursor
                                                                            .lnum
                                                                            - 1 as linenr_T,
                                                                    );
                                                                    if *l as ::core::ffi::c_int == NUL
                                                                        || *l.offset(strlen(l).wrapping_sub(1 as size_t) as isize)
                                                                            as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
                                                                    {
                                                                        break;
                                                                    }
                                                                    (*curwin.get())
                                                                        .w_cursor
                                                                        .lnum -= 1;
                                                                    (*curwin.get()).w_cursor.col = 0
                                                                        as ::core::ffi::c_int
                                                                        as colnr_T;
                                                                }
                                                                l = get_cursor_line_ptr();
                                                            }
                                                            if (*curbuf.get()).b_ind_js != 0 {
                                                                cur_amount = get_indent();
                                                            } else {
                                                                cur_amount = skip_label(
                                                                    (*curwin.get()).w_cursor.lnum,
                                                                    &raw mut l,
                                                                );
                                                            }
                                                            if terminated as ::core::ffi::c_int
                                                                != ',' as ::core::ffi::c_int
                                                                && lookfor != LOOKFOR_TERM
                                                                && *theline.offset(
                                                                    0 as ::core::ffi::c_int
                                                                        as isize,
                                                                )
                                                                    as ::core::ffi::c_int
                                                                    == '{' as ::core::ffi::c_int
                                                            {
                                                                amount = cur_amount;
                                                                if *skipwhite(l)
                                                                    as ::core::ffi::c_int
                                                                    != '{' as ::core::ffi::c_int
                                                                {
                                                                    amount += (*curbuf.get())
                                                                        .b_ind_open_extra;
                                                                }
                                                                if !((*curbuf.get())
                                                                    .b_ind_cpp_baseclass
                                                                    != 0
                                                                    && (*curbuf.get()).b_ind_js
                                                                        == 0)
                                                                {
                                                                    break;
                                                                }
                                                                lookfor = LOOKFOR_CPP_BASECLASS;
                                                            } else if cin_is_cinword(l)
                                                                as ::core::ffi::c_int
                                                                != 0
                                                                || cin_iselse(skipwhite(l)) != 0
                                                            {
                                                                if lookfor == LOOKFOR_UNTERM
                                                                    || lookfor
                                                                        == LOOKFOR_ENUM_OR_INIT
                                                                {
                                                                    if cont_amount
                                                                        > 0 as ::core::ffi::c_int
                                                                    {
                                                                        amount = cont_amount;
                                                                    } else {
                                                                        amount += ind_continuation;
                                                                    }
                                                                    break;
                                                                } else {
                                                                    amount = cur_amount;
                                                                    if *theline.offset(
                                                                        0 as ::core::ffi::c_int
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount += (*curbuf.get())
                                                                            .b_ind_open_extra;
                                                                    }
                                                                    if lookfor != LOOKFOR_TERM {
                                                                        amount += (*curbuf.get())
                                                                            .b_ind_level
                                                                            + (*curbuf.get())
                                                                                .b_ind_no_brace;
                                                                        break;
                                                                    } else {
                                                                        l = skipwhite(
                                                                            get_cursor_line_ptr(),
                                                                        );
                                                                        if cin_isdo(l) != 0 {
                                                                            if whilelevel == 0 as ::core::ffi::c_int {
                                                                                break;
                                                                            }
                                                                            whilelevel -= 1;
                                                                        }
                                                                        if !(cin_iselse(l) != 0
                                                                            && whilelevel == 0 as ::core::ffi::c_int)
                                                                        {
                                                                            continue;
                                                                        }
                                                                        if *l as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                                                                            (*curwin.get()).w_cursor.col = (l
                                                                                .offset_from(get_cursor_line_ptr()) as ::core::ffi::c_int
                                                                                + 1 as ::core::ffi::c_int) as colnr_T;
                                                                        }
                                                                        trypos = find_start_brace();
                                                                        if trypos.is_null()
                                                                            || find_match(
                                                                                LOOKFOR_IF,
                                                                                (*trypos).lnum,
                                                                            ) == FAIL
                                                                        {
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                            } else if lookfor == LOOKFOR_UNTERM {
                                                                if terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                                {
                                                                    amount += ind_continuation;
                                                                }
                                                                break;
                                                            } else if lookfor
                                                                == LOOKFOR_ENUM_OR_INIT
                                                            {
                                                                if terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                                {
                                                                    if (*curbuf.get())
                                                                        .b_ind_cpp_baseclass
                                                                        == 0 as ::core::ffi::c_int
                                                                    {
                                                                        break;
                                                                    }
                                                                    lookfor = LOOKFOR_CPP_BASECLASS;
                                                                } else if amount > cur_amount {
                                                                    amount = cur_amount;
                                                                }
                                                            } else {
                                                                l = get_cursor_line_ptr();
                                                                amount = cur_amount;
                                                                n = strlen(l) as ::core::ffi::c_int;
                                                                if (*curbuf.get()).b_ind_js != 0
                                                                    && terminated as ::core::ffi::c_int
                                                                        == ',' as ::core::ffi::c_int
                                                                    && (*skipwhite(l) as ::core::ffi::c_int
                                                                        == ']' as ::core::ffi::c_int
                                                                        || n >= 2 as ::core::ffi::c_int
                                                                            && *l.offset((n - 2 as ::core::ffi::c_int) as isize)
                                                                                as ::core::ffi::c_int == ']' as ::core::ffi::c_int)
                                                                {
                                                                    break;
                                                                }
                                                                if lookfor == LOOKFOR_INITIAL
                                                                    && terminated
                                                                        as ::core::ffi::c_int
                                                                        == ',' as ::core::ffi::c_int
                                                                {
                                                                    if (*curbuf.get()).b_ind_js != 0
                                                                    {
                                                                        if cin_iscomment(skipwhite(
                                                                            l,
                                                                        )) != 0
                                                                        {
                                                                            break;
                                                                        }
                                                                        lookfor = LOOKFOR_COMMA;
                                                                        trypos = find_match_char(
                                                                            '[' as ::core::ffi::c_char,
                                                                            (*curbuf.get()).b_ind_maxparen,
                                                                        );
                                                                        if trypos.is_null() {
                                                                            continue;
                                                                        }
                                                                        if (*trypos).lnum
                                                                            == (*curwin.get())
                                                                                .w_cursor
                                                                                .lnum
                                                                                - 1 as linenr_T
                                                                        {
                                                                            break;
                                                                        }
                                                                        ourscope = (*trypos).lnum;
                                                                    } else {
                                                                        lookfor =
                                                                            LOOKFOR_ENUM_OR_INIT;
                                                                        cont_amount =
                                                                            cin_first_id_amount();
                                                                    }
                                                                } else {
                                                                    if lookfor == LOOKFOR_INITIAL
                                                                        && *l as ::core::ffi::c_int != NUL
                                                                        && *l.offset(strlen(l).wrapping_sub(1 as size_t) as isize)
                                                                            as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                                                    {
                                                                        cont_amount = cin_get_equal_amount((*curwin.get()).w_cursor.lnum);
                                                                    }
                                                                    if lookfor != LOOKFOR_TERM
                                                                        && lookfor != LOOKFOR_JS_KEY
                                                                        && lookfor != LOOKFOR_COMMA
                                                                        && raw_string_start
                                                                            != (*curwin.get())
                                                                                .w_cursor
                                                                                .lnum
                                                                    {
                                                                        lookfor = LOOKFOR_UNTERM;
                                                                    }
                                                                }
                                                            }
                                                        } else if cin_iswhileofdo_end(
                                                            terminated as uint8_t
                                                                as ::core::ffi::c_int,
                                                        ) != 0
                                                        {
                                                            if lookfor == LOOKFOR_UNTERM
                                                                || lookfor == LOOKFOR_ENUM_OR_INIT
                                                            {
                                                                if cont_amount
                                                                    > 0 as ::core::ffi::c_int
                                                                {
                                                                    amount = cont_amount;
                                                                } else {
                                                                    amount += ind_continuation;
                                                                }
                                                                break;
                                                            } else {
                                                                if whilelevel
                                                                    == 0 as ::core::ffi::c_int
                                                                {
                                                                    lookfor = LOOKFOR_TERM;
                                                                    amount = get_indent();
                                                                    if *theline.offset(
                                                                        0 as ::core::ffi::c_int
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount += (*curbuf.get())
                                                                            .b_ind_open_extra;
                                                                    }
                                                                }
                                                                whilelevel += 1;
                                                            }
                                                        } else if lookfor == LOOKFOR_NOBREAK
                                                            && cin_isbreak(skipwhite(
                                                                get_cursor_line_ptr(),
                                                            )) != 0
                                                        {
                                                            lookfor = LOOKFOR_ANY;
                                                        } else {
                                                            if whilelevel > 0 as ::core::ffi::c_int
                                                            {
                                                                l = cin_skipcomment(
                                                                    get_cursor_line_ptr(),
                                                                );
                                                                if cin_isdo(l) != 0 {
                                                                    amount = get_indent();
                                                                    whilelevel -= 1;
                                                                    continue;
                                                                }
                                                            }
                                                            if lookfor == LOOKFOR_UNTERM
                                                                || lookfor == LOOKFOR_ENUM_OR_INIT
                                                            {
                                                                if cont_amount
                                                                    > 0 as ::core::ffi::c_int
                                                                {
                                                                    amount = cont_amount;
                                                                } else {
                                                                    amount += ind_continuation;
                                                                }
                                                                break;
                                                            } else if lookfor == LOOKFOR_TERM {
                                                                if lookfor_break == 0
                                                                    && whilelevel
                                                                        == 0 as ::core::ffi::c_int
                                                                {
                                                                    break;
                                                                }
                                                            } else {
                                                                loop {
                                                                    l = get_cursor_line_ptr();
                                                                    if find_last_paren(
                                                                        l,
                                                                        '(' as ::core::ffi::c_char,
                                                                        ')' as ::core::ffi::c_char,
                                                                    ) != 0
                                                                        && {
                                                                            trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                                                                            !trypos.is_null()
                                                                        }
                                                                    {
                                                                        (*curwin.get()).w_cursor =
                                                                            *trypos;
                                                                        l = get_cursor_line_ptr();
                                                                        if cin_iscase(l, false_0 != 0) as ::core::ffi::c_int != 0
                                                                            || cin_isscopedecl(l) as ::core::ffi::c_int != 0
                                                                        {
                                                                            (*curwin.get()).w_cursor.lnum += 1;
                                                                            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                                                            break;
                                                                        }
                                                                    }
                                                                    iscase = (*curbuf.get())
                                                                        .b_ind_keep_case_label
                                                                        != 0
                                                                        && cin_iscase(
                                                                            l,
                                                                            false_0 != 0,
                                                                        )
                                                                            as ::core::ffi::c_int
                                                                            != 0;
                                                                    amount = skip_label(
                                                                        (*curwin.get())
                                                                            .w_cursor
                                                                            .lnum,
                                                                        &raw mut l,
                                                                    );
                                                                    if *theline.offset(
                                                                        0 as ::core::ffi::c_int
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount += (*curbuf.get())
                                                                            .b_ind_open_extra;
                                                                    }
                                                                    l = skipwhite(l);
                                                                    if *l as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount -= (*curbuf.get())
                                                                            .b_ind_open_extra;
                                                                    }
                                                                    lookfor = if iscase
                                                                        as ::core::ffi::c_int
                                                                        != 0
                                                                    {
                                                                        LOOKFOR_ANY
                                                                    } else {
                                                                        LOOKFOR_TERM
                                                                    };
                                                                    if lookfor == LOOKFOR_TERM
                                                                        && *l as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                                                                        && cin_iselse(l) != 0
                                                                        && whilelevel == 0 as ::core::ffi::c_int
                                                                    {
                                                                        trypos = find_start_brace();
                                                                        if trypos.is_null()
                                                                            || find_match(LOOKFOR_IF, (*trypos).lnum) == FAIL
                                                                        {
                                                                            break 's_2927;
                                                                        } else {
                                                                            break;
                                                                        }
                                                                    } else {
                                                                        l = get_cursor_line_ptr();
                                                                        if !(find_last_paren(
                                                                            l,
                                                                            '{' as ::core::ffi::c_char,
                                                                            '}' as ::core::ffi::c_char,
                                                                        ) != 0
                                                                            && {
                                                                                trypos = find_start_brace();
                                                                                !trypos.is_null()
                                                                            })
                                                                        {
                                                                            break;
                                                                        }
                                                                        (*curwin.get()).w_cursor = *trypos;
                                                                        l = cin_skipcomment(get_cursor_line_ptr());
                                                                        if *l as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                                                                            || cin_iselse(l) == 0
                                                                        {
                                                                            continue;
                                                                        }
                                                                        (*curwin.get()).w_cursor.lnum += 1;
                                                                        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if cin_iscomment(theline) != 0 {
                            amount += (*curbuf.get()).b_ind_comment;
                        }
                        if (*curbuf.get()).b_ind_jump_label > 0 as ::core::ffi::c_int
                            && original_line_islabel != 0
                        {
                            amount -= (*curbuf.get()).b_ind_jump_label;
                        }
                    } else if *theline.offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '{' as ::core::ffi::c_int
                    {
                        amount = (*curbuf.get()).b_ind_first_open;
                    } else if cur_curpos.lnum < (*curbuf.get()).b_ml.ml_line_count
                        && cin_nocode(theline) == 0
                        && vim_strchr(theline, '{' as ::core::ffi::c_int).is_null()
                        && vim_strchr(theline, '}' as ::core::ffi::c_int).is_null()
                        && cin_ends_in(theline, b":\0".as_ptr() as *const ::core::ffi::c_char) == 0
                        && cin_ends_in(theline, b",\0".as_ptr() as *const ::core::ffi::c_char) == 0
                        && cin_isfuncdecl(
                            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                            cur_curpos.lnum + 1 as linenr_T,
                            cur_curpos.lnum + 1 as linenr_T,
                        ) != 0
                        && cin_isterminated(theline, false_0, true_0) == 0
                    {
                        amount = (*curbuf.get()).b_ind_func_type;
                    } else {
                        amount = 0 as ::core::ffi::c_int;
                        (*curwin.get()).w_cursor = cur_curpos;
                        while (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
                            (*curwin.get()).w_cursor.lnum -= 1;
                            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                            l = get_cursor_line_ptr();
                            trypos = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
                            if !trypos.is_null() {
                                (*curwin.get()).w_cursor.lnum = (*trypos).lnum + 1 as linenr_T;
                                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                            } else {
                                n = 0 as ::core::ffi::c_int;
                                if (*curbuf.get()).b_ind_cpp_baseclass != 0 as ::core::ffi::c_int {
                                    n = cin_is_cpp_baseclass(&raw mut cache_cpp_baseclass);
                                    l = get_cursor_line_ptr();
                                }
                                if n != 0 {
                                    amount = get_baseclass_amount(
                                        cache_cpp_baseclass.lpos.col as ::core::ffi::c_int,
                                    );
                                    break;
                                } else {
                                    if cin_ispreproc_cont(
                                        &raw mut l,
                                        &raw mut (*curwin.get()).w_cursor.lnum,
                                        &raw mut amount,
                                    ) != 0
                                    {
                                        continue;
                                    }
                                    if cin_nocode(l) != 0 {
                                        continue;
                                    }
                                    if cin_ends_in(l, b",\0".as_ptr() as *const ::core::ffi::c_char)
                                        != 0
                                        || *l as ::core::ffi::c_int != NUL && {
                                            n =
                                                *l.offset(
                                                    strlen(l).wrapping_sub(1 as size_t) as isize
                                                )
                                                    as uint8_t
                                                    as ::core::ffi::c_int;
                                            n == '\\' as ::core::ffi::c_int
                                        }
                                    {
                                        if find_last_paren(
                                            l,
                                            '(' as ::core::ffi::c_char,
                                            ')' as ::core::ffi::c_char,
                                        ) != 0
                                            && {
                                                trypos = find_match_paren(
                                                    (*curbuf.get()).b_ind_maxparen,
                                                );
                                                !trypos.is_null()
                                            }
                                        {
                                            (*curwin.get()).w_cursor = *trypos;
                                        }
                                        while n == 0 as ::core::ffi::c_int
                                            && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
                                        {
                                            l = ml_get(
                                                (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
                                            );
                                            if *l as ::core::ffi::c_int == NUL
                                                || *l
                                                    .offset(strlen(l).wrapping_sub(1 as size_t)
                                                        as isize)
                                                    as ::core::ffi::c_int
                                                    != '\\' as ::core::ffi::c_int
                                            {
                                                break;
                                            }
                                            (*curwin.get()).w_cursor.lnum -= 1;
                                            (*curwin.get()).w_cursor.col =
                                                0 as ::core::ffi::c_int as colnr_T;
                                        }
                                        amount = get_indent();
                                        if amount == 0 as ::core::ffi::c_int {
                                            amount = cin_first_id_amount();
                                        }
                                        if amount == 0 as ::core::ffi::c_int {
                                            amount = ind_continuation;
                                        }
                                        break;
                                    } else {
                                        if cin_isfuncdecl(
                                            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                                            cur_curpos.lnum,
                                            0 as linenr_T,
                                        ) != 0
                                        {
                                            break;
                                        }
                                        l = get_cursor_line_ptr();
                                        if *skipwhite(l) as ::core::ffi::c_int
                                            == '}' as ::core::ffi::c_int
                                        {
                                            break;
                                        }
                                        if cin_ends_in(
                                            l,
                                            b"};\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) != 0
                                        {
                                            break;
                                        }
                                        if cin_ends_in(
                                            l,
                                            b"[\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) != 0
                                        {
                                            amount = get_indent() + ind_continuation;
                                            break;
                                        } else {
                                            look = skipwhite(l);
                                            if *look as ::core::ffi::c_int
                                                == ';' as ::core::ffi::c_int
                                                && cin_nocode(
                                                    look.offset(1 as ::core::ffi::c_int as isize),
                                                ) != 0
                                            {
                                                let mut curpos_save: pos_T =
                                                    (*curwin.get()).w_cursor;
                                                while (*curwin.get()).w_cursor.lnum > 1 as linenr_T
                                                {
                                                    (*curwin.get()).w_cursor.lnum -= 1;
                                                    look = ml_get((*curwin.get()).w_cursor.lnum);
                                                    if !(cin_nocode(look) != 0
                                                        || cin_ispreproc_cont(
                                                            &raw mut look,
                                                            &raw mut (*curwin.get()).w_cursor.lnum,
                                                            &raw mut amount,
                                                        ) != 0)
                                                    {
                                                        break;
                                                    }
                                                }
                                                if (*curwin.get()).w_cursor.lnum > 0 as linenr_T
                                                    && cin_ends_in(
                                                        look,
                                                        b"}\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ) != 0
                                                {
                                                    break;
                                                }
                                                (*curwin.get()).w_cursor = curpos_save;
                                            }
                                            if cin_isfuncdecl(
                                                &raw mut l,
                                                (*curwin.get()).w_cursor.lnum,
                                                0 as linenr_T,
                                            ) != 0
                                            {
                                                amount = (*curbuf.get()).b_ind_param;
                                                break;
                                            } else {
                                                if cin_ends_in(
                                                    l,
                                                    b";\0".as_ptr() as *const ::core::ffi::c_char,
                                                ) != 0
                                                {
                                                    l = ml_get(
                                                        (*curwin.get()).w_cursor.lnum
                                                            - 1 as linenr_T,
                                                    );
                                                    if cin_ends_in(
                                                        l,
                                                        b",\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ) != 0
                                                        || *l as ::core::ffi::c_int != NUL
                                                            && *l.offset(
                                                                strlen(l).wrapping_sub(1 as size_t)
                                                                    as isize,
                                                            )
                                                                as ::core::ffi::c_int
                                                                == '\\' as ::core::ffi::c_int
                                                    {
                                                        break;
                                                    }
                                                    l = get_cursor_line_ptr();
                                                }
                                                find_last_paren(
                                                    l,
                                                    '(' as ::core::ffi::c_char,
                                                    ')' as ::core::ffi::c_char,
                                                );
                                                trypos = find_match_paren(
                                                    (*curbuf.get()).b_ind_maxparen,
                                                );
                                                if !trypos.is_null() {
                                                    (*curwin.get()).w_cursor = *trypos;
                                                }
                                                amount = get_indent();
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if cin_iscomment(theline) != 0 {
                            amount += (*curbuf.get()).b_ind_comment;
                        }
                        if cur_curpos.lnum > 1 as linenr_T {
                            l = ml_get(cur_curpos.lnum - 1 as linenr_T);
                            if *l as ::core::ffi::c_int != NUL
                                && *l.offset(strlen(l).wrapping_sub(1 as size_t) as isize)
                                    as ::core::ffi::c_int
                                    == '\\' as ::core::ffi::c_int
                            {
                                cur_amount = cin_get_equal_amount(cur_curpos.lnum - 1 as linenr_T);
                                if cur_amount > 0 as ::core::ffi::c_int {
                                    amount = cur_amount;
                                } else if cur_amount == 0 as ::core::ffi::c_int {
                                    amount += ind_continuation;
                                }
                            }
                        }
                    }
                }
            }
        }
        if amount < 0 as ::core::ffi::c_int {
            amount = 0 as ::core::ffi::c_int;
        }
    }
    (*curwin.get()).w_cursor = cur_curpos;
    xfree(linecopy as *mut ::core::ffi::c_void);
    return amount;
}
pub const BRACE_IN_COL0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const BRACE_AT_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const BRACE_AT_END: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOOKFOR_INITIAL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LOOKFOR_IF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOOKFOR_DO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOOKFOR_CASE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOOKFOR_ANY: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const LOOKFOR_TERM: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const LOOKFOR_UNTERM: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const LOOKFOR_SCOPEDECL: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const LOOKFOR_NOBREAK: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const LOOKFOR_CPP_BASECLASS: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const LOOKFOR_ENUM_OR_INIT: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const LOOKFOR_JS_KEY: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const LOOKFOR_COMMA: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
unsafe extern "C" fn find_match(
    mut lookfor: ::core::ffi::c_int,
    mut ourscope: linenr_T,
) -> ::core::ffi::c_int {
    let mut look: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut theirscope: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut mightbeif: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut elselevel: ::core::ffi::c_int = 0;
    let mut whilelevel: ::core::ffi::c_int = 0;
    if lookfor == LOOKFOR_IF {
        elselevel = 1 as ::core::ffi::c_int;
        whilelevel = 0 as ::core::ffi::c_int;
    } else {
        elselevel = 0 as ::core::ffi::c_int;
        whilelevel = 1 as ::core::ffi::c_int;
    }
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    while (*curwin.get()).w_cursor.lnum > ourscope + 1 as linenr_T {
        (*curwin.get()).w_cursor.lnum -= 1;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        look = cin_skipcomment(get_cursor_line_ptr());
        if cin_iselse(look) == 0
            && cin_isif(look) == 0
            && cin_isdo(look) == 0
            && cin_iswhileofdo(look, (*curwin.get()).w_cursor.lnum) == 0
        {
            continue;
        }
        theirscope = find_start_brace();
        if theirscope.is_null() {
            break;
        }
        if (*theirscope).lnum < ourscope {
            break;
        }
        if (*theirscope).lnum > ourscope {
            continue;
        }
        look = cin_skipcomment(get_cursor_line_ptr());
        if !(lookfor == LOOKFOR_IF && whilelevel != 0) {
            if cin_iselse(look) != 0 {
                mightbeif = cin_skipcomment(look.offset(4 as ::core::ffi::c_int as isize));
                if cin_isif(mightbeif) == 0 {
                    elselevel += 1;
                }
                continue;
            } else if cin_isif(look) != 0 {
                elselevel -= 1;
                if elselevel == 0 as ::core::ffi::c_int && lookfor == LOOKFOR_IF {
                    whilelevel = 0 as ::core::ffi::c_int;
                }
            }
        }
        if cin_iswhileofdo(look, (*curwin.get()).w_cursor.lnum) != 0 {
            whilelevel += 1;
        } else {
            if cin_isdo(look) != 0 {
                whilelevel -= 1;
            }
            if elselevel <= 0 as ::core::ffi::c_int && whilelevel <= 0 as ::core::ffi::c_int {
                return OK;
            }
        }
    }
    return FAIL;
}
pub unsafe extern "C" fn in_cinkeys(
    mut keytyped: ::core::ffi::c_int,
    mut when: ::core::ffi::c_int,
    mut line_is_empty: bool,
) -> bool {
    let mut look: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut try_match: bool = false;
    let mut try_match_word: bool = false;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut icase: bool = false;
    if keytyped == NUL {
        return false_0 != 0;
    }
    if *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL {
        look = (*curbuf.get()).b_p_indk;
    } else {
        look = (*curbuf.get()).b_p_cink;
    }
    while *look != 0 {
        match when {
            42 => {
                try_match = *look as ::core::ffi::c_int == '*' as ::core::ffi::c_int;
            }
            33 => {
                try_match = *look as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
            }
            _ => {
                try_match = *look as ::core::ffi::c_int != '*' as ::core::ffi::c_int;
            }
        }
        if *look as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            || *look as ::core::ffi::c_int == '!' as ::core::ffi::c_int
        {
            look = look.offset(1);
        }
        if *look as ::core::ffi::c_int == '0' as ::core::ffi::c_int {
            try_match_word = try_match;
            if !line_is_empty {
                try_match = false_0 != 0;
            }
            look = look.offset(1);
        } else {
            try_match_word = false_0 != 0;
        }
        if *look as ::core::ffi::c_int == '^' as ::core::ffi::c_int
            && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                >= '?' as ::core::ffi::c_int
            && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                <= '_' as ::core::ffi::c_int
        {
            if try_match as ::core::ffi::c_int != 0
                && keytyped
                    == (if (*look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                        < 'a' as ::core::ffi::c_int
                        || *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            > 'z' as ::core::ffi::c_int
                    {
                        *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    } else {
                        *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    }) ^ 0x40 as ::core::ffi::c_int
            {
                return true_0 != 0;
            }
            look = look.offset(2 as ::core::ffi::c_int as isize);
        } else if *look as ::core::ffi::c_int == 'o' as ::core::ffi::c_int {
            if try_match as ::core::ffi::c_int != 0
                && keytyped == KEY_OPEN_FORW as ::core::ffi::c_int
            {
                return true_0 != 0;
            }
            look = look.offset(1);
        } else if *look as ::core::ffi::c_int == 'O' as ::core::ffi::c_int {
            if try_match as ::core::ffi::c_int != 0
                && keytyped == KEY_OPEN_BACK as ::core::ffi::c_int
            {
                return true_0 != 0;
            }
            look = look.offset(1);
        } else if *look as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            if try_match as ::core::ffi::c_int != 0
                && keytyped == 'e' as ::core::ffi::c_int
                && (*curwin.get()).w_cursor.col >= 4 as ::core::ffi::c_int
            {
                p = get_cursor_line_ptr();
                if skipwhite(p)
                    == p.offset((*curwin.get()).w_cursor.col as isize)
                        .offset(-(4 as ::core::ffi::c_int as isize))
                    && strncmp(
                        p.offset((*curwin.get()).w_cursor.col as isize)
                            .offset(-(4 as ::core::ffi::c_int as isize)),
                        b"else\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    return true_0 != 0;
                }
            }
            look = look.offset(1);
        } else if *look as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
            if try_match as ::core::ffi::c_int != 0 && keytyped == ':' as ::core::ffi::c_int {
                p = get_cursor_line_ptr();
                if cin_iscase(p, false_0 != 0) as ::core::ffi::c_int != 0
                    || cin_isscopedecl(p) as ::core::ffi::c_int != 0
                    || cin_islabel() as ::core::ffi::c_int != 0
                {
                    return true_0 != 0;
                }
                p = get_cursor_line_ptr();
                if (*curwin.get()).w_cursor.col > 2 as ::core::ffi::c_int
                    && *p.offset(
                        ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                    && *p.offset(
                        ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                            - 2 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                {
                    *p.offset(
                        ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int) as isize,
                    ) = ' ' as ::core::ffi::c_char;
                    let i: bool = cin_iscase(p, false_0 != 0) as ::core::ffi::c_int != 0
                        || cin_isscopedecl(p) as ::core::ffi::c_int != 0
                        || cin_islabel() as ::core::ffi::c_int != 0;
                    p = get_cursor_line_ptr();
                    *p.offset(
                        ((*curwin.get()).w_cursor.col as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int) as isize,
                    ) = ':' as ::core::ffi::c_char;
                    if i {
                        return true_0 != 0;
                    }
                }
            }
            look = look.offset(1);
        } else if *look as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
            if try_match {
                if !vim_strchr(
                    b"<>!*oOe0:\0".as_ptr() as *const ::core::ffi::c_char,
                    *look.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
                    && keytyped
                        == *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                {
                    return true_0 != 0;
                }
                if keytyped == get_special_key_code(look.offset(1 as ::core::ffi::c_int as isize)) {
                    return true_0 != 0;
                }
            }
            while *look as ::core::ffi::c_int != 0
                && *look as ::core::ffi::c_int != '>' as ::core::ffi::c_int
            {
                look = look.offset(1);
            }
            while *look as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
                look = look.offset(1);
            }
        } else if *look as ::core::ffi::c_int == '=' as ::core::ffi::c_int
            && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ',' as ::core::ffi::c_int
            && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            look = look.offset(1);
            if *look as ::core::ffi::c_int == '~' as ::core::ffi::c_int {
                icase = true_0 != 0;
                look = look.offset(1);
            } else {
                icase = false_0 != 0;
            }
            p = vim_strchr(look, ',' as ::core::ffi::c_int);
            if p.is_null() {
                p = look.offset(strlen(look) as isize);
            }
            if (try_match as ::core::ffi::c_int != 0 || try_match_word as ::core::ffi::c_int != 0)
                && (*curwin.get()).w_cursor.col >= p.offset_from(look) as colnr_T
            {
                let mut match_0: bool = false_0 != 0;
                if keytyped == KEY_COMPLETE as ::core::ffi::c_int {
                    let mut n: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut s: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                    s = line.offset((*curwin.get()).w_cursor.col as isize);
                    while s > line {
                        n = mb_prevptr(line, s);
                        if !vim_iswordp(n) {
                            break;
                        }
                        s = n;
                    }
                    '_c2rust_label: {
                        if p >= look
                            && p.offset_from(look) as uintmax_t <= 18446744073709551615 as uintmax_t
                        {
                        } else {
                            __assert_fail(
                                b"p >= look && (uintmax_t)(p - look) <= SIZE_MAX\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/indent_c.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                3933 as ::core::ffi::c_uint,
                                __ASSERT_FUNCTION.as_ptr(),
                            );
                        }
                    };
                    if s.offset(p.offset_from(look) as isize)
                        <= line.offset((*curwin.get()).w_cursor.col as isize)
                        && (if icase as ::core::ffi::c_int != 0 {
                            mb_strnicmp(s, look, p.offset_from(look) as size_t)
                        } else {
                            strncmp(s, look, p.offset_from(look) as size_t)
                        }) == 0 as ::core::ffi::c_int
                    {
                        match_0 = true_0 != 0;
                    }
                } else if keytyped
                    == *p.offset(-1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                    || icase as ::core::ffi::c_int != 0
                        && keytyped < 256 as ::core::ffi::c_int
                        && keytyped >= 0 as ::core::ffi::c_int
                        && tolower(keytyped)
                            == tolower(*p.offset(-1 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int)
                {
                    let mut line_0: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                    '_c2rust_label_0: {
                        if p >= look
                            && p.offset_from(look) as uintmax_t <= 18446744073709551615 as uintmax_t
                        {
                        } else {
                            __assert_fail(
                                b"p >= look && (uintmax_t)(p - look) <= SIZE_MAX\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/indent_c.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                3946 as ::core::ffi::c_uint,
                                __ASSERT_FUNCTION.as_ptr(),
                            );
                        }
                    };
                    if ((*curwin.get()).w_cursor.col == p.offset_from(look) as colnr_T
                        || !vim_iswordc(
                            *line_0.offset((-p.offset_from(look) - 1 as isize) as isize) as uint8_t
                                as ::core::ffi::c_int,
                        ))
                        && (if icase as ::core::ffi::c_int != 0 {
                            mb_strnicmp(
                                line_0.offset(-(p.offset_from(look) as isize)),
                                look,
                                p.offset_from(look) as size_t,
                            )
                        } else {
                            strncmp(
                                line_0.offset(-(p.offset_from(look) as isize)),
                                look,
                                p.offset_from(look) as size_t,
                            )
                        }) == 0 as ::core::ffi::c_int
                    {
                        match_0 = true_0 != 0;
                    }
                }
                if match_0 as ::core::ffi::c_int != 0
                    && try_match_word as ::core::ffi::c_int != 0
                    && !try_match
                {
                    if getwhitecols_curline()
                        != ((*curwin.get()).w_cursor.col as isize - p.offset_from(look))
                            as ::core::ffi::c_int as intptr_t
                    {
                        match_0 = false_0 != 0;
                    }
                }
                if match_0 {
                    return true_0 != 0;
                }
            }
            look = p;
        } else {
            if try_match as ::core::ffi::c_int != 0
                && *look as uint8_t as ::core::ffi::c_int == keytyped
            {
                return true_0 != 0;
            }
            if *look as ::core::ffi::c_int != NUL {
                look = look.offset(1);
            }
        }
        look = skip_to_option_part(look);
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn do_c_expr_indent() {
    if *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL {
        fixthisline(Some(
            get_expr_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
        ));
    } else {
        fixthisline(Some(
            get_c_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
        ));
    };
}
pub unsafe extern "C" fn f_cindent(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut lnum: linenr_T = tv_get_lnum(argvars);
    if lnum >= 1 as linenr_T && lnum <= (*curbuf.get()).b_ml.ml_line_count {
        (*curwin.get()).w_cursor.lnum = lnum;
        (*rettv).vval.v_number = get_c_indent() as varnumber_T;
        (*curwin.get()).w_cursor = pos;
    } else {
        (*rettv).vval.v_number = -1 as varnumber_T;
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
