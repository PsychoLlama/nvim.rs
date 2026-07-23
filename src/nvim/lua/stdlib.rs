use crate::src::cjson::lua_cjson::lua_cjson_new;
use crate::src::mpack::lmpack::luaopen_mpack;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, dict_check_writable, find_buffer_by_handle, find_tab_by_handle,
    find_window_by_handle, try_enter, try_leave,
};
use crate::src::nvim::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::src::nvim::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_find, tv_dict_item_alloc_len, tv_dict_item_remove,
    tv_dict_watcher_notify,
};
use crate::src::nvim::eval::vars::{before_set_vvar, get_globvar_dict, get_vimvar_dict};
use crate::src::nvim::eval::window::{win_execute_after, win_execute_before};
use crate::src::nvim::ex_docmd::{apply_cmdmod, undo_cmdmod};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::fold::foldUpdate;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::lua::base64::luaopen_base64;
use crate::src::nvim::lua::converter::{nlua_pop_typval, nlua_push_typval};
use crate::src::nvim::lua::ffi::{
    luaL_argerror, luaL_checkinteger, luaL_checklstring, luaL_checkudata, luaL_error,
    luaL_newmetatable, luaL_register, luaL_where, lua_concat, lua_createtable, lua_error,
    lua_getfield, lua_gettop, lua_newuserdata, lua_next, lua_pcall, lua_pushcclosure,
    lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushnumber, lua_pushstring, lua_pushvalue,
    lua_pushvfstring, lua_rawseti, lua_setfield, lua_setmetatable, lua_settop, lua_toboolean,
    lua_tolstring, lua_type,
};
use crate::src::nvim::lua::spell::luaopen_spell;
use crate::src::nvim::lua::xdiff::nlua_xdl_diff;
use crate::src::nvim::main::{buffer_handles, cmdmod, curbuf, g_min_log_level, window_handles};
use crate::src::nvim::map::mh_get_int;
use crate::src::nvim::mbyte::{
    convert_setup, convert_setup_ext, enc_canonize, enc_skip, mb_utf_index_to_bytes, mb_utflen,
    string_convert, utf_cp_bounds_len, utf_ptr2len_len,
};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{strequal, xfree};
use crate::src::nvim::os::libc::{__assert_fail, memchr, memset, strcasecmp};
use crate::src::nvim::runtime::script_autoload;
pub use crate::src::nvim::types::{
    __builtin_va_list, __gnuc_va_list, __time_t, __va_list_tag, aco_save_T, alist_T, bhdr_T,
    blob_T, blobvar_S, blocknr_T, buf_T, bufref_T, bufstate_T, chunksize_T, cmdmod_T, colnr_T,
    dict_T, dictitem_T, dictvar_S, diff_T, diffblock_S, disptick_T, except_T, except_type_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_10, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, iconv_t, infoptr_T, int16_t, int32_t, int64_t, int8_t, intptr_t,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, luaL_Reg, lua_CFunction, lua_Integer, lua_Number, lua_State, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, msglist,
    msglist_T, mtnode_inner_s, mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T,
    ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmatch_T, regmmatch_T,
    regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, ssize_t, switchwin_T, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7,
    u_header_uh_alt_prev as C2Rust_Unnamed_6, u_header_uh_next as C2Rust_Unnamed_9,
    u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, va_list, varnumber_T, vim_exception, vimconv_T, virt_line, visualinfo_T, win_T,
    win_execute_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, AdditionalData, AlignTextPos,
    BoolVarValue, BufUpdateCallbacks, Buffer, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, CharBoundsOff, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, Error, ErrorType, ExtmarkUndoObject, FileID,
    FloatAnchor, FloatRelative, GridView, Intersection, LuaRef, MTKey, MTNode, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_int_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_11, String_0, Tabpage, Terminal, Timestamp,
    TryState, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig,
    WinInfo, WinSplit, WinStyle, Window, QUEUE,
};
use crate::src::nvim::window::win_find_tabpage;
extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
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
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_12 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_12 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_12 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_12 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_12 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_12 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_12 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_12 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_12 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_12 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_12 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_12 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_12 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_12 = 1;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_13 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_13 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_13 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_13 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_13 = 1;
pub const CONV_NONE: C2Rust_Unnamed_13 = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_REGISTRYINDEX: ::core::ffi::c_int = -10000 as ::core::ffi::c_int;
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002 as ::core::ffi::c_int;
pub const LUA_TNIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_TSTRING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C-unwind" fn map_get_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
) -> ptr_t {
    let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline(always)]
unsafe extern "C-unwind" fn QUEUE_EMPTY(q: *const QUEUE) -> ::core::ffi::c_int {
    return (q == (*q).next as *const QUEUE) as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C-unwind" fn tv_dict_is_watched(d: *const dict_T) -> bool {
    return !d.is_null() && QUEUE_EMPTY(&raw const (*d).watchers) == 0;
}
unsafe extern "C-unwind" fn regex_match(
    mut lstate: *mut lua_State,
    mut prog: *mut *mut regprog_T,
    mut str: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut rm: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    rm.regprog = *prog;
    rm.rm_ic = false_0 != 0;
    let mut match_0: bool = vim_regexec(&raw mut rm, str, 0 as colnr_T);
    *prog = rm.regprog;
    if match_0 {
        lua_pushinteger(
            lstate,
            rm.startp[0 as ::core::ffi::c_int as usize].offset_from(str),
        );
        lua_pushinteger(
            lstate,
            rm.endp[0 as ::core::ffi::c_int as usize].offset_from(str),
        );
        return 2 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn regex_match_str(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut prog: *mut *mut regprog_T = regex_check(lstate);
    let mut str: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        2 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut nret: ::core::ffi::c_int = regex_match(lstate, prog, str as *mut ::core::ffi::c_char);
    if (*prog).is_null() {
        return luaL_error(
            lstate,
            b"regex: internal error\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return nret;
}
unsafe extern "C-unwind" fn regex_match_line(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut prog: *mut *mut regprog_T = regex_check(lstate);
    let mut narg: ::core::ffi::c_int = lua_gettop(lstate);
    if narg < 3 as ::core::ffi::c_int {
        return luaL_error(
            lstate,
            b"not enough args\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut bufnr: handle_T = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as handle_T;
    let mut rownr: linenr_T = luaL_checkinteger(lstate, 3 as ::core::ffi::c_int) as linenr_T;
    let mut start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut end: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if narg >= 4 as ::core::ffi::c_int {
        start = luaL_checkinteger(lstate, 4 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    if narg >= 5 as ::core::ffi::c_int {
        end = luaL_checkinteger(lstate, 5 as ::core::ffi::c_int) as ::core::ffi::c_int;
        if end < 0 as ::core::ffi::c_int {
            return luaL_error(
                lstate,
                b"invalid end\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
    let mut buf: *mut buf_T = (if bufnr != 0 {
        map_get_int_ptr_t(buffer_handles.ptr(), bufnr as ::core::ffi::c_int)
    } else {
        curbuf.get() as *mut ::core::ffi::c_void
    }) as *mut buf_T;
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
        return luaL_error(
            lstate,
            b"invalid buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if rownr >= (*buf).b_ml.ml_line_count {
        return luaL_error(
            lstate,
            b"invalid row\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut line: *mut ::core::ffi::c_char = ml_get_buf(buf, rownr + 1 as linenr_T);
    let mut len: colnr_T = ml_get_buf_len(buf, rownr + 1 as linenr_T);
    if start < 0 as ::core::ffi::c_int || start > len {
        return luaL_error(
            lstate,
            b"invalid start\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut save: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    if end >= 0 as ::core::ffi::c_int {
        if end > len || end < start {
            return luaL_error(
                lstate,
                b"invalid end\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        save = *line.offset(end as isize);
        *line.offset(end as isize) = NUL as ::core::ffi::c_char;
    }
    let mut nret: ::core::ffi::c_int = regex_match(lstate, prog, line.offset(start as isize));
    if end >= 0 as ::core::ffi::c_int {
        *line.offset(end as isize) = save;
    }
    if (*prog).is_null() {
        return luaL_error(
            lstate,
            b"regex: internal error\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return nret;
}
unsafe extern "C-unwind" fn regex_check(mut L: *mut lua_State) -> *mut *mut regprog_T {
    return luaL_checkudata(
        L,
        1 as ::core::ffi::c_int,
        b"nvim_regex\0".as_ptr() as *const ::core::ffi::c_char,
    ) as *mut *mut regprog_T;
}
unsafe extern "C-unwind" fn regex_gc(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut prog: *mut *mut regprog_T = regex_check(lstate);
    vim_regfree(*prog);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn regex_tostring(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushstring(lstate, b"<regex>\0".as_ptr() as *const ::core::ffi::c_char);
    return 1 as ::core::ffi::c_int;
}
static regex_meta: GlobalCell<[luaL_Reg; 5]> = GlobalCell::new([
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(regex_gc as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            regex_tostring as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"match_str\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            regex_match_str as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"match_line\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            regex_match_line as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
pub unsafe extern "C-unwind" fn nlua_str_utfindex(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char =
        luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
    let mut idx: intptr_t = 0;
    if lua_type(lstate, 2 as ::core::ffi::c_int) <= 0 as ::core::ffi::c_int {
        idx = s1_len as intptr_t;
    } else {
        idx = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as intptr_t;
        if idx < 0 as intptr_t || idx > s1_len as intptr_t {
            lua_pushnil(lstate);
            lua_pushnil(lstate);
            return 2 as ::core::ffi::c_int;
        }
    }
    let mut codepoints: size_t = 0 as size_t;
    let mut codeunits: size_t = 0 as size_t;
    mb_utflen(s1, idx as size_t, &raw mut codepoints, &raw mut codeunits);
    lua_pushinteger(lstate, codepoints as lua_Integer);
    lua_pushinteger(lstate, codeunits as lua_Integer);
    return 2 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_str_utf_pos(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char =
        luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut idx: size_t = 1 as size_t;
    let mut clen: size_t = 0;
    let mut i: size_t = 0 as size_t;
    while i < s1_len && *s1.offset(i as isize) as ::core::ffi::c_int != NUL {
        clen = utf_ptr2len_len(
            s1.offset(i as isize),
            s1_len.wrapping_sub(i) as ::core::ffi::c_int,
        ) as size_t;
        lua_pushinteger(lstate, i as lua_Integer + 1 as lua_Integer);
        lua_rawseti(lstate, -2 as ::core::ffi::c_int, idx as ::core::ffi::c_int);
        idx = idx.wrapping_add(1);
        i = i.wrapping_add(clen);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_str_utf_start(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char =
        luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
    let mut offset: ptrdiff_t = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int);
    if offset <= 0 as ptrdiff_t || offset > s1_len as ptrdiff_t {
        return luaL_error(
            lstate,
            b"index out of range\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let off: size_t = (offset - 1 as ptrdiff_t) as size_t;
    let mut head_off: ::core::ffi::c_int = -(utf_cp_bounds_len(
        s1,
        s1.offset(off as isize),
        s1_len.wrapping_sub(off) as ::core::ffi::c_int,
    )
    .begin_off as ::core::ffi::c_int);
    lua_pushinteger(lstate, head_off as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_str_utf_end(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char =
        luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
    let mut offset: ptrdiff_t = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int);
    if offset <= 0 as ptrdiff_t || offset > s1_len as ptrdiff_t {
        return luaL_error(
            lstate,
            b"index out of range\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let off: size_t = (offset - 1 as ptrdiff_t) as size_t;
    let mut tail_off: ::core::ffi::c_int = utf_cp_bounds_len(
        s1,
        s1.offset(off as isize),
        s1_len.wrapping_sub(off) as ::core::ffi::c_int,
    )
    .end_off as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    lua_pushinteger(lstate, tail_off as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C-unwind" fn nlua_str_byteindex(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char =
        luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
    let mut idx: intptr_t = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as intptr_t;
    if idx < 0 as intptr_t {
        lua_pushnil(lstate);
        return 1 as ::core::ffi::c_int;
    }
    let mut use_utf16: bool = false_0 != 0;
    if lua_gettop(lstate) >= 3 as ::core::ffi::c_int {
        use_utf16 = lua_toboolean(lstate, 3 as ::core::ffi::c_int) != 0;
    }
    let mut byteidx: ssize_t = mb_utf_index_to_bytes(s1, s1_len, idx as size_t, use_utf16);
    if byteidx == -1 as ssize_t {
        lua_pushnil(lstate);
        return 1 as ::core::ffi::c_int;
    }
    lua_pushinteger(lstate, byteidx as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C-unwind" fn nlua_regex(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut text: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut prog: *mut regprog_T = ::core::ptr::null_mut::<regprog_T>();
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    prog = vim_regcomp(
        text,
        8 as ::core::ffi::c_int | 1 as ::core::ffi::c_int | 4 as ::core::ffi::c_int,
    );
    try_leave(&raw mut tstate, &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        nlua_push_errstr(
            lstate,
            b"couldn't parse regex: %s\0".as_ptr() as *const ::core::ffi::c_char,
            err.msg,
        );
        api_clear_error(&raw mut err);
        return lua_error(lstate);
    } else if prog.is_null() {
        nlua_push_errstr(
            lstate,
            b"couldn't parse regex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return lua_error(lstate);
    }
    let mut p: *mut *mut regprog_T =
        lua_newuserdata(lstate, ::core::mem::size_of::<*mut regprog_T>()) as *mut *mut regprog_T;
    *p = prog;
    lua_getfield(
        lstate,
        LUA_REGISTRYINDEX,
        b"nvim_regex\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setmetatable(lstate, -2 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_get_var_scope(mut lstate: *mut lua_State) -> *mut dict_T {
    let mut scope: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut handle: handle_T = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as handle_T;
    let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if strequal(scope, b"g\0".as_ptr() as *const ::core::ffi::c_char) {
        dict = get_globvar_dict();
    } else if strequal(scope, b"v\0".as_ptr() as *const ::core::ffi::c_char) {
        dict = get_vimvar_dict();
    } else if strequal(scope, b"b\0".as_ptr() as *const ::core::ffi::c_char) {
        let mut buf: *mut buf_T = find_buffer_by_handle(handle as Buffer, &raw mut err);
        if !buf.is_null() {
            dict = (*buf).b_vars;
        }
    } else if strequal(scope, b"w\0".as_ptr() as *const ::core::ffi::c_char) {
        let mut win: *mut win_T = find_window_by_handle(handle as Window, &raw mut err);
        if !win.is_null() {
            dict = (*win).w_vars;
        }
    } else if strequal(scope, b"t\0".as_ptr() as *const ::core::ffi::c_char) {
        let mut tabpage: *mut tabpage_T = find_tab_by_handle(handle as Tabpage, &raw mut err);
        if !tabpage.is_null() {
            dict = (*tabpage).tp_vars;
        }
    } else {
        luaL_error(
            lstate,
            b"invalid scope\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<dict_T>();
    }
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        nlua_push_errstr(
            lstate,
            b"scoped variable: %s\0".as_ptr() as *const ::core::ffi::c_char,
            err.msg,
        );
        api_clear_error(&raw mut err);
        lua_error(lstate);
        return ::core::ptr::null_mut::<dict_T>();
    }
    return dict;
}
pub unsafe extern "C-unwind" fn nlua_setvar(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut dict: *mut dict_T = nlua_get_var_scope(lstate);
    let mut key: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    key.data = luaL_checklstring(lstate, 3 as ::core::ffi::c_int, &raw mut key.size)
        as *mut ::core::ffi::c_char;
    let mut del: bool = lua_gettop(lstate) < 4 as ::core::ffi::c_int
        || lua_type(lstate, 4 as ::core::ffi::c_int) == LUA_TNIL;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut di: *mut dictitem_T = dict_check_writable(dict, key, del, &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        nlua_push_errstr(
            lstate,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            err.msg,
        );
        api_clear_error(&raw mut err);
        lua_error(lstate);
        return 0 as ::core::ffi::c_int;
    }
    let mut watched: bool = tv_dict_is_watched(dict);
    if del {
        if di.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if watched {
            tv_dict_watcher_notify(
                dict,
                key.data,
                ::core::ptr::null_mut::<typval_T>(),
                &raw mut (*di).di_tv,
            );
        }
        tv_dict_item_remove(dict, di);
    } else {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        lua_pushvalue(lstate, 4 as ::core::ffi::c_int);
        if !nlua_pop_typval(lstate, &raw mut tv) {
            return luaL_error(
                lstate,
                b"Couldn't convert lua value\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut oldtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if di.is_null() {
            di = tv_dict_item_alloc_len(key.data, key.size);
            tv_dict_add(dict, di);
        } else {
            let mut type_error: bool = false_0 != 0;
            if dict == get_vimvar_dict()
                && !before_set_vvar(
                    key.data,
                    di,
                    &raw mut tv,
                    true_0 != 0,
                    watched,
                    &raw mut type_error,
                )
            {
                tv_clear(&raw mut tv);
                if type_error {
                    return luaL_error(
                        lstate,
                        b"Setting v:%s to value with wrong type\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        key.data,
                    );
                }
                return 0 as ::core::ffi::c_int;
            }
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            }
            tv_clear(&raw mut (*di).di_tv);
        }
        tv_copy(&raw mut tv, &raw mut (*di).di_tv);
        if watched {
            tv_dict_watcher_notify(dict, key.data, &raw mut tv, &raw mut oldtv);
            tv_clear(&raw mut oldtv);
        }
        tv_clear(&raw mut tv);
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C-unwind" fn nlua_getvar(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut dict: *mut dict_T = nlua_get_var_scope(lstate);
    let mut len: size_t = 0;
    let mut name: *const ::core::ffi::c_char =
        luaL_checklstring(lstate, 3 as ::core::ffi::c_int, &raw mut len);
    let mut di: *mut dictitem_T = tv_dict_find(dict, name, len as ptrdiff_t);
    if di.is_null() && dict == get_globvar_dict() {
        if !script_autoload(name, len, false_0 != 0) || aborting() as ::core::ffi::c_int != 0 {
            return 0 as ::core::ffi::c_int;
        }
        di = tv_dict_find(dict, name, len as ptrdiff_t);
    }
    if di.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    nlua_push_typval(lstate, &raw mut (*di).di_tv, 0 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_stricmp(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut s1_len: size_t = 0;
    let mut s2_len: size_t = 0;
    let mut s1: *const ::core::ffi::c_char =
        luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
    let mut s2: *const ::core::ffi::c_char =
        luaL_checklstring(lstate, 2 as ::core::ffi::c_int, &raw mut s2_len);
    let mut nul1: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut nul2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_c2rust_label: {
        if *s1.offset(s1_len as isize) as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"s1[s1_len] == NUL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/stdlib.rs\0".as_ptr() as *const ::core::ffi::c_char,
                481 as ::core::ffi::c_uint,
                b"int nlua_stricmp(lua_State *const)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if *s2.offset(s2_len as isize) as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"s2[s2_len] == NUL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/stdlib.rs\0".as_ptr() as *const ::core::ffi::c_char,
                482 as ::core::ffi::c_uint,
                b"int nlua_stricmp(lua_State *const)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    loop {
        nul1 = memchr(s1 as *const ::core::ffi::c_void, NUL, s1_len) as *const ::core::ffi::c_char;
        nul2 = memchr(s2 as *const ::core::ffi::c_void, NUL, s2_len) as *const ::core::ffi::c_char;
        ret = strcasecmp(
            s1 as *mut ::core::ffi::c_char,
            s2 as *mut ::core::ffi::c_char,
        );
        if ret != 0 as ::core::ffi::c_int {
            break;
        }
        if nul1.is_null() as ::core::ffi::c_int != nul2.is_null() as ::core::ffi::c_int {
            ret = !nul1.is_null() as ::core::ffi::c_int - !nul2.is_null() as ::core::ffi::c_int;
            break;
        } else {
            if nul1.is_null() {
                break;
            }
            '_c2rust_label_1: {
                if !nul2.is_null() {
                } else {
                    __assert_fail(
                        b"nul2 != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/lua/stdlib.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        494 as ::core::ffi::c_uint,
                        b"int nlua_stricmp(lua_State *const)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            s1_len =
                s1_len.wrapping_sub((nul1.offset_from(s1) as size_t).wrapping_add(1 as size_t));
            s2_len =
                s2_len.wrapping_sub((nul2.offset_from(s2) as size_t).wrapping_add(1 as size_t));
            s1 = nul1.offset(1 as ::core::ffi::c_int as isize);
            s2 = nul2.offset(1 as ::core::ffi::c_int as isize);
        }
    }
    lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_pushnumber(
        lstate,
        ((ret > 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            - (ret < 0 as ::core::ffi::c_int) as ::core::ffi::c_int) as lua_Number,
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_iconv(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut narg: ::core::ffi::c_int = lua_gettop(lstate);
    if narg < 3 as ::core::ffi::c_int {
        return luaL_error(
            lstate,
            b"Expected at least 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i <= 3 as ::core::ffi::c_int {
        if lua_type(lstate, i) != LUA_TSTRING {
            return luaL_argerror(
                lstate,
                i,
                b"expected string\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        i += 1;
    }
    let mut str_len: size_t = 0 as size_t;
    let mut str: *const ::core::ffi::c_char =
        lua_tolstring(lstate, 1 as ::core::ffi::c_int, &raw mut str_len);
    let mut from: *mut ::core::ffi::c_char = enc_canonize(enc_skip(lua_tolstring(
        lstate,
        2 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    )
        as *mut ::core::ffi::c_char));
    let mut to: *mut ::core::ffi::c_char = enc_canonize(enc_skip(lua_tolstring(
        lstate,
        3 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    )
        as *mut ::core::ffi::c_char));
    let mut vimconv: vimconv_T = vimconv_T {
        vc_type: 0,
        vc_factor: 0,
        vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        vc_fail: false,
    };
    vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
    convert_setup_ext(&raw mut vimconv, from, false_0 != 0, to, false_0 != 0);
    let mut ret: *mut ::core::ffi::c_char = string_convert(
        &raw mut vimconv,
        str as *mut ::core::ffi::c_char,
        &raw mut str_len,
    );
    convert_setup(
        &raw mut vimconv,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    xfree(from as *mut ::core::ffi::c_void);
    xfree(to as *mut ::core::ffi::c_void);
    if ret.is_null() {
        lua_pushnil(lstate);
    } else {
        lua_pushlstring(lstate, ret, str_len);
        xfree(ret as *mut ::core::ffi::c_void);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_foldupdate(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut window: handle_T = luaL_checkinteger(lstate, 1 as ::core::ffi::c_int) as handle_T;
    let mut win: *mut win_T =
        map_get_int_ptr_t(window_handles.ptr(), window as ::core::ffi::c_int) as *mut win_T;
    if win.is_null() {
        return luaL_error(
            lstate,
            b"invalid window\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut top: linenr_T =
        luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as linenr_T + 1 as linenr_T;
    if top < 1 as linenr_T {
        return luaL_error(
            lstate,
            b"invalid top\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut bot: linenr_T = luaL_checkinteger(lstate, 3 as ::core::ffi::c_int) as linenr_T;
    if top > bot {
        return luaL_error(
            lstate,
            b"invalid bot\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    foldUpdate(win, top, bot);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_with(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut log_level: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    (lua_type(L, 1 as ::core::ffi::c_int) == 5 as ::core::ffi::c_int
        || luaL_argerror(
            L,
            1 as ::core::ffi::c_int,
            b"table expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    lua_pushnil(L);
    while lua_next(L, 1 as ::core::ffi::c_int) != 0 {
        if lua_type(L, -2 as ::core::ffi::c_int) == LUA_TSTRING {
            let mut k: *const ::core::ffi::c_char = lua_tolstring(
                L,
                -2 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            );
            let mut v: bool = lua_toboolean(L, -1 as ::core::ffi::c_int) != 0;
            if strequal(b"buf\0".as_ptr() as *const ::core::ffi::c_char, k) {
                buf = map_get_int_ptr_t(
                    buffer_handles.ptr(),
                    luaL_checkinteger(L, -1 as ::core::ffi::c_int) as ::core::ffi::c_int,
                ) as *mut buf_T;
            } else if strequal(b"win\0".as_ptr() as *const ::core::ffi::c_char, k) {
                win = map_get_int_ptr_t(
                    window_handles.ptr(),
                    luaL_checkinteger(L, -1 as ::core::ffi::c_int) as ::core::ffi::c_int,
                ) as *mut win_T;
            } else if strequal(b"log_level\0".as_ptr() as *const ::core::ffi::c_char, k) {
                log_level = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as ::core::ffi::c_int;
            } else {
                if strequal(b"sandbox\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_SANDBOX as ::core::ffi::c_int;
                }
                if strequal(b"silent\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_SILENT as ::core::ffi::c_int;
                }
                if strequal(b"emsg_silent\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_ERRSILENT as ::core::ffi::c_int;
                }
                if strequal(b"unsilent\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_UNSILENT as ::core::ffi::c_int;
                }
                if strequal(b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_NOAUTOCMD as ::core::ffi::c_int;
                }
                if strequal(b"hide\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_HIDE as ::core::ffi::c_int;
                }
                if strequal(b"keepalt\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_KEEPALT as ::core::ffi::c_int;
                }
                if strequal(b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_KEEPMARKS as ::core::ffi::c_int;
                }
                if strequal(b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_KEEPJUMPS as ::core::ffi::c_int;
                }
                if strequal(b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_LOCKMARKS as ::core::ffi::c_int;
                }
                if strequal(b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char, k)
                    as ::core::ffi::c_int
                    != 0
                    && v as ::core::ffi::c_int != 0
                {
                    flags |= CMOD_KEEPPATTERNS as ::core::ffi::c_int;
                }
            }
        }
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
    let mut status: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut rets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0 {
        flags |= CMOD_SILENT as ::core::ffi::c_int;
    }
    let save_min_log_level: ::core::ffi::c_int = g_min_log_level.get();
    if log_level >= 0 as ::core::ffi::c_int {
        g_min_log_level.set(log_level);
    }
    let mut save_cmdmod: cmdmod_T = cmdmod.get();
    memset(
        cmdmod.ptr() as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmdmod_T>(),
    );
    (*cmdmod.ptr()).cmod_flags = flags;
    apply_cmdmod(cmdmod.ptr());
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    let mut win_execute_args: win_execute_T = win_execute_T {
        wp: ::core::ptr::null_mut::<win_T>(),
        curpos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cwd: [0; 4096],
        cwd_status: 0,
        apply_acd: false,
        save_sfname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        switchwin: switchwin_T {
            sw_curwin: ::core::ptr::null_mut::<win_T>(),
            sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
            sw_same_win: false,
            sw_visual_active: false,
        },
    };
    's_376: {
        if !win.is_null() {
            let mut tabpage: *mut tabpage_T = win_find_tabpage(win);
            if !win_execute_before(&raw mut win_execute_args, win, tabpage) {
                break 's_376;
            }
        } else if !buf.is_null() {
            aucmd_prepbuf(&raw mut aco, buf);
        }
        let mut s: ::core::ffi::c_int = lua_gettop(L);
        lua_pushvalue(L, 2 as ::core::ffi::c_int);
        status = lua_pcall(
            L,
            0 as ::core::ffi::c_int,
            -1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        rets = lua_gettop(L) - s;
        if !win.is_null() {
            win_execute_after(&raw mut win_execute_args);
        } else if !buf.is_null() {
            aucmd_restbuf(&raw mut aco);
        }
    }
    try_leave(&raw mut tstate, &raw mut err);
    undo_cmdmod(cmdmod.ptr());
    cmdmod.set(save_cmdmod);
    if log_level >= 0 as ::core::ffi::c_int {
        g_min_log_level.set(save_min_log_level);
    }
    if status != 0 {
        return lua_error(L);
    } else if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        nlua_push_errstr(L, b"%s\0".as_ptr() as *const ::core::ffi::c_char, err.msg);
        api_clear_error(&raw mut err);
        return lua_error(L);
    }
    return rets;
}
unsafe extern "C-unwind" fn nlua_state_add_internal(lstate: *mut lua_State) {
    lua_pushcclosure(
        lstate,
        Some(nlua_getvar as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_getvar\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_setvar as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_setvar\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_foldupdate as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_foldupdate\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_with as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_with_c\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
pub unsafe extern "C-unwind" fn nlua_state_add_stdlib(lstate: *mut lua_State, mut is_thread: bool) {
    if !is_thread {
        lua_pushcclosure(
            lstate,
            Some(nlua_stricmp as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"stricmp\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_str_utfindex
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"_str_utfindex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_str_byteindex
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"_str_byteindex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_str_utf_pos
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"str_utf_pos\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_str_utf_start
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"str_utf_start\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_str_utf_end
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"str_utf_end\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(nlua_regex as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"regex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        luaL_newmetatable(
            lstate,
            b"nvim_regex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        luaL_register(
            lstate,
            ::core::ptr::null::<::core::ffi::c_char>(),
            regex_meta.ptr() as *mut luaL_Reg,
        );
        lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"__index\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        luaopen_spell(lstate);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"spell\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(nlua_iconv as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"iconv\0".as_ptr() as *const ::core::ffi::c_char,
        );
        luaopen_base64(lstate);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"base64\0".as_ptr() as *const ::core::ffi::c_char,
        );
        nlua_state_add_internal(lstate);
    }
    luaopen_mpack(lstate);
    lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -3 as ::core::ffi::c_int,
        b"mpack\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"package\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushvalue(lstate, -3 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"mpack\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_settop(lstate, -3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    extern "C" {
        #[link_name = "luaopen_lpeg"]
        fn luaopen_lpeg_0(_: *mut lua_State) -> ::core::ffi::c_int;
    }
    luaopen_lpeg_0(lstate);
    lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -4 as ::core::ffi::c_int,
        b"lpeg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"package\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushvalue(lstate, -3 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"lpeg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_settop(lstate, -4 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_pushcclosure(
        lstate,
        Some(nlua_xdl_diff as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"diff\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_cjson_new(lstate);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"json\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
pub unsafe extern "C-unwind" fn nlua_push_errstr(
    mut L: *mut lua_State,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    let mut argp: ::core::ffi::VaList;
    argp = c2rust_args.clone();
    luaL_where(L, 1 as ::core::ffi::c_int);
    lua_pushvfstring(L, fmt, argp);
    lua_concat(L, 2 as ::core::ffi::c_int);
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
