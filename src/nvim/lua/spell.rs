use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::lua::ffi::{
    luaL_argerror, luaL_error, luaL_register, lua_createtable, lua_gettop, lua_pushinteger,
    lua_pushlstring, lua_pushstring, lua_rawseti, lua_tolstring, lua_type,
};
use crate::src::nvim::main::{curwin, e_no_spell};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{__assert_fail, gettext};
use crate::src::nvim::spell::{parse_spelllang, spell_check};
pub use crate::src::nvim::types::{
    __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T,
    colnr_T, dict_T, dictvar_S, disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_10,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, hlf_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T,
    list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, luaL_Reg,
    lua_CFunction, lua_Integer, lua_State, mapblock, mapblock_T, match_T, matchitem, matchitem_T,
    memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, partial_S, partial_T, pos_T,
    pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T, taggy_T, terminal,
    time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_7, u_header_uh_alt_prev as C2Rust_Unnamed_6,
    u_header_uh_next as C2Rust_Unnamed_9, u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, AdditionalData, AlignTextPos,
    BoolVarValue, BufUpdateCallbacks, Callback, CallbackType, Callback_data as C2Rust_Unnamed_4,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1, ExtmarkUndoObject, FileID, FloatAnchor,
    FloatRelative, GridView, Intersection, LuaRef, MTKey, MTNode, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_11, Terminal,
    Timestamp, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig,
    WinInfo, WinSplit, WinStyle, Window, QUEUE,
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
pub const HLF_COUNT: hlf_T = 76;
pub const HLF_PRE: hlf_T = 75;
pub const HLF_OK: hlf_T = 74;
pub const HLF_SO: hlf_T = 73;
pub const HLF_SE: hlf_T = 72;
pub const HLF_TSNC: hlf_T = 71;
pub const HLF_TS: hlf_T = 70;
pub const HLF_BFOOTER: hlf_T = 69;
pub const HLF_BTITLE: hlf_T = 68;
pub const HLF_CU: hlf_T = 67;
pub const HLF_WBRNC: hlf_T = 66;
pub const HLF_WBR: hlf_T = 65;
pub const HLF_BORDER: hlf_T = 64;
pub const HLF_MSG: hlf_T = 63;
pub const HLF_NFLOAT: hlf_T = 62;
pub const HLF_MSGSEP: hlf_T = 61;
pub const HLF_INACTIVE: hlf_T = 60;
pub const HLF_0: hlf_T = 59;
pub const HLF_QFL: hlf_T = 58;
pub const HLF_MC: hlf_T = 57;
pub const HLF_CUL: hlf_T = 56;
pub const HLF_CUC: hlf_T = 55;
pub const HLF_TPF: hlf_T = 54;
pub const HLF_TPS: hlf_T = 53;
pub const HLF_TP: hlf_T = 52;
pub const HLF_PBR: hlf_T = 51;
pub const HLF_PST: hlf_T = 50;
pub const HLF_PSB: hlf_T = 49;
pub const HLF_PSX: hlf_T = 48;
pub const HLF_PNX: hlf_T = 47;
pub const HLF_PSK: hlf_T = 46;
pub const HLF_PNK: hlf_T = 45;
pub const HLF_PMSI: hlf_T = 44;
pub const HLF_PMNI: hlf_T = 43;
pub const HLF_PSI: hlf_T = 42;
pub const HLF_PNI: hlf_T = 41;
pub const HLF_SPL: hlf_T = 40;
pub const HLF_SPR: hlf_T = 39;
pub const HLF_SPC: hlf_T = 38;
pub const HLF_SPB: hlf_T = 37;
pub const HLF_CONCEAL: hlf_T = 36;
pub const HLF_SC: hlf_T = 35;
pub const HLF_TXA: hlf_T = 34;
pub const HLF_TXD: hlf_T = 33;
pub const HLF_DED: hlf_T = 32;
pub const HLF_CHD: hlf_T = 31;
pub const HLF_ADD: hlf_T = 30;
pub const HLF_FC: hlf_T = 29;
pub const HLF_FL: hlf_T = 28;
pub const HLF_WM: hlf_T = 27;
pub const HLF_W: hlf_T = 26;
pub const HLF_VNC: hlf_T = 25;
pub const HLF_V: hlf_T = 24;
pub const HLF_T: hlf_T = 23;
pub const HLF_VSP: hlf_T = 22;
pub const HLF_C: hlf_T = 21;
pub const HLF_SNC: hlf_T = 20;
pub const HLF_S: hlf_T = 19;
pub const HLF_R: hlf_T = 18;
pub const HLF_CLF: hlf_T = 17;
pub const HLF_CLS: hlf_T = 16;
pub const HLF_CLN: hlf_T = 15;
pub const HLF_LNB: hlf_T = 14;
pub const HLF_LNA: hlf_T = 13;
pub const HLF_N: hlf_T = 12;
pub const HLF_CM: hlf_T = 11;
pub const HLF_M: hlf_T = 10;
pub const HLF_LC: hlf_T = 9;
pub const HLF_L: hlf_T = 8;
pub const HLF_I: hlf_T = 7;
pub const HLF_E: hlf_T = 6;
pub const HLF_D: hlf_T = 5;
pub const HLF_AT: hlf_T = 4;
pub const HLF_TERM: hlf_T = 3;
pub const HLF_EOB: hlf_T = 2;
pub const HLF_8: hlf_T = 1;
pub const HLF_NONE: hlf_T = 0;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"int nlua_spell_check(lua_State *)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_TSTRING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub unsafe extern "C" fn nlua_spell_check(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    if lua_gettop(lstate) < 1 as ::core::ffi::c_int {
        return luaL_error(
            lstate,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if lua_type(lstate, 1 as ::core::ffi::c_int) != LUA_TSTRING {
        luaL_argerror(
            lstate,
            1 as ::core::ffi::c_int,
            b"expected string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut str: *const ::core::ffi::c_char = lua_tolstring(
        lstate,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let wo_spell_save: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_spell;
    if (*curwin.get()).w_onebuf_opt.wo_spell == 0 {
        parse_spelllang(curwin.get());
        (*curwin.get()).w_onebuf_opt.wo_spell = true_0;
    }
    if *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int == NUL {
        emsg(gettext(&raw const e_no_spell as *const ::core::ffi::c_char));
        (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
        return 0 as ::core::ffi::c_int;
    }
    let mut attr: hlf_T = HLF_COUNT;
    let mut pos: size_t = 0 as size_t;
    let mut capcol: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut no_res: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut result: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    while *str as ::core::ffi::c_int != NUL {
        attr = HLF_COUNT;
        let mut len: size_t = spell_check(
            curwin.get(),
            str as *mut ::core::ffi::c_char,
            &raw mut attr,
            &raw mut capcol,
            false_0 != 0,
        );
        '_c2rust_label: {
            if len <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/spell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    60 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        if attr as ::core::ffi::c_uint != HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint {
            lua_createtable(lstate, 3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            lua_pushlstring(lstate, str, len);
            lua_rawseti(lstate, -2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
            result = if attr as ::core::ffi::c_uint
                == HLF_SPB as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                b"bad\0".as_ptr() as *const ::core::ffi::c_char
            } else if attr as ::core::ffi::c_uint
                == HLF_SPR as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                b"rare\0".as_ptr() as *const ::core::ffi::c_char
            } else if attr as ::core::ffi::c_uint
                == HLF_SPL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                b"local\0".as_ptr() as *const ::core::ffi::c_char
            } else if attr as ::core::ffi::c_uint
                == HLF_SPC as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                b"caps\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                ::core::ptr::null::<::core::ffi::c_char>()
            };
            '_c2rust_label_0: {
                if !result.is_null() {
                } else {
                    __assert_fail(
                        b"result != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/lua/spell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        74 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            lua_pushstring(lstate, result);
            lua_rawseti(lstate, -2 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
            lua_pushinteger(lstate, pos as lua_Integer + 1 as lua_Integer);
            lua_rawseti(lstate, -2 as ::core::ffi::c_int, 3 as ::core::ffi::c_int);
            no_res += 1;
            lua_rawseti(lstate, -2 as ::core::ffi::c_int, no_res);
        }
        str = str.offset(len as isize);
        pos = pos.wrapping_add(len);
        capcol -= len as ::core::ffi::c_int;
    }
    (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
    return 1 as ::core::ffi::c_int;
}
static spell_functions: GlobalCell<[luaL_Reg; 2]> = GlobalCell::new([
    luaL_Reg {
        name: b"check\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(nlua_spell_check as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
pub unsafe extern "C" fn luaopen_spell(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    luaL_register(
        L,
        ::core::ptr::null::<::core::ffi::c_char>(),
        (spell_functions.ptr() as *const _) as *const luaL_Reg,
    );
    return 1 as ::core::ffi::c_int;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
