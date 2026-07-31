use crate::src::nvim::api::buffer::{nvim_buf_get_lines, nvim_buf_set_lines};
use crate::src::nvim::api::extmark::{
    nvim_buf_clear_namespace, nvim_create_namespace, parse_virt_text,
};
use crate::src::nvim::api::private::dispatch::msgpack_rpc_get_handler_for;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_object, api_set_error, api_set_sctx, api_typename, arena_array,
    copy_object, copy_string, cstr_as_string, dict_set_var, find_buffer_by_handle,
    find_tab_by_handle, find_window_by_handle,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid};
use crate::src::nvim::api::vimscript::exec_impl;
use crate::src::nvim::decoration::{clear_virttext, decor_find_virttext};
use crate::src::nvim::eval::vars::get_globvar_dict;
use crate::src::nvim::extmark::extmark_set;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight::hl_get_attr_by_id;
use crate::src::nvim::highlight_group::{
    syn_check_group, syn_get_final_id, syn_id2attr, syn_name2id,
};
use crate::src::nvim::lua::executor::nlua_exec;
use crate::src::nvim::main::{
    curbuf, current_sctx, curwin, got_int, msg_didout, msg_silent, no_wait_return,
};
use crate::src::nvim::memory::{xmalloc, xrealloc};
use crate::src::nvim::message::{emsg, msg, msg_end};
use crate::src::nvim::option::{
    find_option, get_option_value_for, get_vimoption, object_as_optval, option_has_scope,
    optval_as_object, set_option_value_for,
};
use crate::src::nvim::options::kOptInvalid;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::types::api::{kErrorTypeNone, kErrorTypeValidation};
pub use crate::src::nvim::types::{
    __time_t, AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, Array, BoolVarValue,
    Boolean, BufUpdateCallbacks, Buffer, Callback, Callback_data as C2Rust_Unnamed_5, CallbackType,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInline, DecorInlineData,
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType,
    ExtmarkMove, ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject, FileID, Float, FloatAnchor,
    FloatRelative, GridView, Integer, Intersection, KeyDict_empty, KeyDict_exec_opts, KeyValuePair,
    LuaRef, LuaRetMode, MTKey, MTNode, MTPos, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MapHash, MarkTree, MsgpackRpcRequestHandler, Object,
    ObjectType, OptIndex, OptInt, OptScope, OptVal, OptValData, OptValType, OptionalKeys, QUEUE,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_13, String_0,
    StringBuilder, Tabpage, Terminal, Timestamp, TriState, UndoObjectType, VarLockStatus, VarType,
    VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle,
    Window, alist_T, bcount_t, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T,
    chunksize_T, colnr_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_12, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, kObjectTypeArray, kObjectTypeDict,
    kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, key_value_pair, lcs_chars_T, linenr_T,
    list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    lua_State, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T,
    tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint8_t, uint16_t, uint32_t, uint64_t,
    undo_object, undo_object_data as C2Rust_Unnamed_7, varnumber_T, virt_line, visualinfo_T, win_T,
    window_S, wininfo_S, winopt_T, wline_T, xfmark_T,
};
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const kHlModeUnknown: C2Rust_Unnamed_16 = 0;
pub const OPT_GLOBAL: C2Rust_Unnamed_17 = 1;
pub const kOptScopeBuf: OptScope = 2;
pub const kOptScopeWin: OptScope = 1;
pub const kOptScopeGlobal: OptScope = 0;
pub const OPT_LOCAL: C2Rust_Unnamed_17 = 2;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub const LINE_BUFFER_MIN_SIZE: C2Rust_Unnamed_18 = 4096;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kHlModeBlend: C2Rust_Unnamed_16 = 3;
pub const kHlModeCombine: C2Rust_Unnamed_16 = 2;
pub const kHlModeReplace: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_17 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_17 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_17 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_17 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_17 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_17 = 4;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub unsafe extern "C" fn nvim_exec(
    mut channel_id: uint64_t,
    mut src: String_0,
    mut output: Boolean,
    mut err: *mut Error,
) -> String_0 {
    let mut opts: KeyDict_exec_opts = KeyDict_exec_opts { output: output };
    return exec_impl(channel_id, src, &raw mut opts, err);
}
pub unsafe extern "C" fn nvim_command_output(
    mut channel_id: uint64_t,
    mut command: String_0,
    mut err: *mut Error,
) -> String_0 {
    let mut opts: KeyDict_exec_opts = KeyDict_exec_opts {
        output: true_0 != 0,
    };
    return exec_impl(channel_id, command, &raw mut opts, err);
}
pub unsafe extern "C" fn nvim_execute_lua(
    mut code: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return nlua_exec(
        code,
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetObject,
        arena,
        err,
    );
}
pub unsafe extern "C" fn nvim_buf_get_number(mut buffer: Buffer, mut err: *mut Error) -> Integer {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return 0 as Integer;
    }
    return (*buf).handle as Integer;
}
unsafe extern "C" fn src2ns(mut src_id: *mut Integer) -> uint32_t {
    if *src_id == 0 as Integer {
        *src_id = nvim_create_namespace(String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        });
    }
    if *src_id < 0 as Integer {
        return ((1 as ::core::ffi::c_int as uint32_t) << 31 as ::core::ffi::c_int)
            .wrapping_sub(1 as uint32_t);
    }
    return *src_id as uint32_t;
}
pub unsafe extern "C" fn nvim_buf_clear_highlight(
    mut buffer: Buffer,
    mut ns_id: Integer,
    mut line_start: Integer,
    mut line_end: Integer,
    mut err: *mut Error,
) {
    nvim_buf_clear_namespace(buffer, ns_id, line_start, line_end, err);
}
pub unsafe extern "C" fn nvim_buf_add_highlight(
    mut buffer: Buffer,
    mut ns_id: Integer,
    mut hl_group: String_0,
    mut line: Integer,
    mut col_start: Integer,
    mut col_end: Integer,
    mut err: *mut Error,
) -> Integer {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return 0 as Integer;
    }
    if !(line >= 0 as Integer && line < MAXLNUM as ::core::ffi::c_int as Integer) {
        api_err_invalid(
            err,
            b"line number\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return 0 as Integer;
    }
    if !(col_start >= 0 as Integer && col_start <= MAXCOL as ::core::ffi::c_int as Integer) {
        api_err_invalid(
            err,
            b"column\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return 0 as Integer;
    }
    if col_end < 0 as Integer || col_end > MAXCOL as ::core::ffi::c_int as Integer {
        col_end = MAXCOL as ::core::ffi::c_int as Integer;
    }
    let mut ns: uint32_t = src2ns(&raw mut ns_id);
    if !(line < (*buf).b_ml.ml_line_count as Integer) {
        return ns_id;
    }
    let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if hl_group.size > 0 as size_t {
        hl_id = syn_check_group(hl_group.data, hl_group.size);
    } else {
        return ns_id;
    }
    let mut end_line: ::core::ffi::c_int = line as ::core::ffi::c_int;
    if col_end == MAXCOL as ::core::ffi::c_int as Integer {
        col_end = 0 as Integer;
        end_line += 1;
    }
    let mut decor: DecorInline = DECOR_INLINE_INIT;
    decor.data.hl.hl_id = hl_id;
    extmark_set(
        buf,
        ns,
        ::core::ptr::null_mut::<uint32_t>(),
        line as ::core::ffi::c_int,
        col_start as colnr_T,
        end_line,
        col_end as colnr_T,
        decor,
        MT_FLAG_DECOR_HL as uint16_t,
        true_0 != 0,
        false_0 != 0,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Error>(),
    );
    return ns_id;
}
pub unsafe extern "C" fn nvim_buf_set_virtual_text(
    mut buffer: Buffer,
    mut src_id: Integer,
    mut line: Integer,
    mut chunks: Array,
    mut _opts: *mut KeyDict_empty,
    mut err: *mut Error,
) -> Integer {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return 0 as Integer;
    }
    if line < 0 as Integer || line >= MAXLNUM as ::core::ffi::c_int as Integer {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Line number outside range\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 0 as Integer;
    }
    let mut ns_id: uint32_t = src2ns(&raw mut src_id);
    let mut width: ::core::ffi::c_int = 0;
    let mut virt_text: VirtText = parse_virt_text(chunks, err, &raw mut width);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return 0 as Integer;
    }
    let mut existing: *mut DecorVirtText =
        decor_find_virttext(buf, line as ::core::ffi::c_int, ns_id as uint64_t);
    if !existing.is_null() {
        clear_virttext(&raw mut (*existing).data.virt_text);
        (*existing).data.virt_text = virt_text;
        (*existing).width = width;
        return src_id;
    }
    let mut vt: *mut DecorVirtText =
        xmalloc(::core::mem::size_of::<DecorVirtText>()) as *mut DecorVirtText;
    *vt = DecorVirtText {
        flags: 0 as uint8_t,
        hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
        priority: DECOR_PRIORITY_BASE as DecorPriority,
        width: 0 as ::core::ffi::c_int,
        col: 0 as ::core::ffi::c_int,
        pos: kVPosEndOfLine,
        data: C2Rust_Unnamed_2 {
            virt_text: VirtText {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
        },
        next: ::core::ptr::null_mut::<DecorVirtText>(),
    };
    (*vt).data.virt_text = virt_text;
    (*vt).width = width;
    (*vt).priority = 0 as DecorPriority;
    let mut decor: DecorInline = DecorInline {
        ext: true_0 != 0,
        data: DecorInlineData {
            ext: DecorExt {
                sh_idx: DECOR_ID_INVALID as uint32_t,
                vt: vt,
            },
        },
    };
    extmark_set(
        buf,
        ns_id,
        ::core::ptr::null_mut::<uint32_t>(),
        line as ::core::ffi::c_int,
        0 as colnr_T,
        -1 as ::core::ffi::c_int,
        -1 as colnr_T,
        decor,
        0 as uint16_t,
        true_0 != 0,
        false_0 != 0,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Error>(),
    );
    return src_id;
}
pub unsafe extern "C" fn nvim_get_hl_by_id(
    mut hl_id: Integer,
    mut rgb: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut dic: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    if !(syn_get_final_id(hl_id as ::core::ffi::c_int) != 0 as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"highlight id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            hl_id as int64_t,
            false_0 != 0,
        );
        return dic;
    }
    let mut attrcode: ::core::ffi::c_int = syn_id2attr(hl_id as ::core::ffi::c_int);
    return hl_get_attr_by_id(attrcode as Integer, rgb, arena, err);
}
pub unsafe extern "C" fn nvim_get_hl_by_name(
    mut name: String_0,
    mut rgb: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut result: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut id: ::core::ffi::c_int = syn_name2id(name.data);
    if !(id != 0 as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"highlight name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return result;
    }
    return nvim_get_hl_by_id(id as Integer, rgb, arena, err);
}
pub unsafe extern "C" fn buffer_insert(
    mut buffer: Buffer,
    mut lnum: Integer,
    mut lines: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    nvim_buf_set_lines(
        0 as uint64_t,
        buffer,
        lnum,
        lnum,
        true_0 != 0,
        lines,
        arena,
        err,
    );
}
pub unsafe extern "C" fn buffer_get_line(
    mut buffer: Buffer,
    mut index: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    let mut rv: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    index = convert_index(index as int64_t) as Integer;
    let mut slice: Array = nvim_buf_get_lines(
        0 as uint64_t,
        buffer,
        index,
        index + 1 as Integer,
        true_0 != 0,
        arena,
        ::core::ptr::null_mut::<lua_State>(),
        err,
    );
    if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
        && slice.size != 0
    {
        rv = (*slice.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .string;
    }
    return rv;
}
pub unsafe extern "C" fn buffer_set_line(
    mut buffer: Buffer,
    mut index: Integer,
    mut line: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut l: Object = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: line },
    };
    let mut array: Array = Array {
        size: 1 as size_t,
        capacity: 0,
        items: &raw mut l,
    };
    index = convert_index(index as int64_t) as Integer;
    nvim_buf_set_lines(
        0 as uint64_t,
        buffer,
        index,
        index + 1 as Integer,
        true_0 != 0,
        array,
        arena,
        err,
    );
}
pub unsafe extern "C" fn buffer_del_line(
    mut buffer: Buffer,
    mut index: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut array: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    index = convert_index(index as int64_t) as Integer;
    nvim_buf_set_lines(
        0 as uint64_t,
        buffer,
        index,
        index + 1 as Integer,
        true_0 != 0,
        array,
        arena,
        err,
    );
}
pub unsafe extern "C" fn buffer_get_line_slice(
    mut buffer: Buffer,
    mut start: Integer,
    mut end: Integer,
    mut include_start: Boolean,
    mut include_end: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    start = (convert_index(start as int64_t) + !include_start as ::core::ffi::c_int as int64_t)
        as Integer;
    end = (convert_index(end as int64_t) + include_end as int64_t) as Integer;
    return nvim_buf_get_lines(
        0 as uint64_t,
        buffer,
        start,
        end,
        false_0 != 0,
        arena,
        ::core::ptr::null_mut::<lua_State>(),
        err,
    );
}
pub unsafe extern "C" fn buffer_set_line_slice(
    mut buffer: Buffer,
    mut start: Integer,
    mut end: Integer,
    mut include_start: Boolean,
    mut include_end: Boolean,
    mut replacement: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    start = (convert_index(start as int64_t) + !include_start as ::core::ffi::c_int as int64_t)
        as Integer;
    end = (convert_index(end as int64_t) + include_end as int64_t) as Integer;
    nvim_buf_set_lines(
        0 as uint64_t,
        buffer,
        start,
        end,
        false_0 != 0,
        replacement,
        arena,
        err,
    );
}
pub unsafe extern "C" fn buffer_set_var(
    mut buffer: Buffer,
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*buf).b_vars,
        name,
        value,
        false_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn buffer_del_var(
    mut buffer: Buffer,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*buf).b_vars,
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn window_set_var(
    mut window: Window,
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut win: *mut win_T = find_window_by_handle(window, err);
    if win.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*win).w_vars,
        name,
        value,
        false_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn window_del_var(
    mut window: Window,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut win: *mut win_T = find_window_by_handle(window, err);
    if win.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*win).w_vars,
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn tabpage_set_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*tab).tp_vars,
        name,
        value,
        false_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn tabpage_del_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*tab).tp_vars,
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn vim_set_var(
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return dict_set_var(
        get_globvar_dict(),
        name,
        value,
        false_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn vim_del_var(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return dict_set_var(
        get_globvar_dict(),
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
unsafe extern "C" fn convert_index(mut index: int64_t) -> int64_t {
    return if index < 0 as int64_t {
        index - 1 as int64_t
    } else {
        index
    };
}
pub unsafe extern "C" fn nvim_get_option_info(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    return get_vimoption(
        name,
        OPT_GLOBAL as ::core::ffi::c_int,
        curbuf.get(),
        curwin.get(),
        arena,
        err,
    );
}
pub unsafe extern "C" fn nvim_set_option(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    set_option_to(channel_id, NULL, kOptScopeGlobal, name, value, err);
}
pub unsafe extern "C" fn nvim_get_option(mut name: String_0, mut err: *mut Error) -> Object {
    return get_option_from(NULL, kOptScopeGlobal, name, err);
}
pub unsafe extern "C" fn nvim_buf_get_option(
    mut buffer: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) -> Object {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return get_option_from(buf as *mut ::core::ffi::c_void, kOptScopeBuf, name, err);
}
pub unsafe extern "C" fn nvim_buf_set_option(
    mut channel_id: uint64_t,
    mut buffer: Buffer,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return;
    }
    set_option_to(
        channel_id,
        buf as *mut ::core::ffi::c_void,
        kOptScopeBuf,
        name,
        value,
        err,
    );
}
pub unsafe extern "C" fn nvim_win_get_option(
    mut window: Window,
    mut name: String_0,
    mut err: *mut Error,
) -> Object {
    let mut win: *mut win_T = find_window_by_handle(window, err);
    if win.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return get_option_from(win as *mut ::core::ffi::c_void, kOptScopeWin, name, err);
}
pub unsafe extern "C" fn nvim_win_set_option(
    mut channel_id: uint64_t,
    mut window: Window,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    let mut win: *mut win_T = find_window_by_handle(window, err);
    if win.is_null() {
        return;
    }
    set_option_to(
        channel_id,
        win as *mut ::core::ffi::c_void,
        kOptScopeWin,
        name,
        value,
        err,
    );
}
unsafe extern "C" fn get_option_from(
    mut from: *mut ::core::ffi::c_void,
    mut scope: OptScope,
    mut name: String_0,
    mut err: *mut Error,
) -> Object {
    if !(name.size > 0 as size_t) {
        api_err_invalid(
            err,
            b"option name\0".as_ptr() as *const ::core::ffi::c_char,
            b"<empty>\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            true_0 != 0,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    let mut opt_idx: OptIndex = find_option(name.data);
    if !(opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"option name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    let mut value: OptVal = OptVal {
        type_0: kOptValTypeNil,
        data: OptValData { boolean: kFalse },
    };
    if option_has_scope(opt_idx, scope) {
        value = get_option_value_for(
            opt_idx,
            if scope as ::core::ffi::c_uint
                == kOptScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                OPT_GLOBAL as ::core::ffi::c_int
            } else {
                OPT_LOCAL as ::core::ffi::c_int
            },
            scope,
            from,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
    }
    if !(value.type_0 as ::core::ffi::c_int != kOptValTypeNil as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"option name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return optval_as_object(value);
}
unsafe extern "C" fn set_option_to(
    mut channel_id: uint64_t,
    mut to: *mut ::core::ffi::c_void,
    mut scope: OptScope,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    if !(name.size > 0 as size_t) {
        api_err_invalid(
            err,
            b"option name\0".as_ptr() as *const ::core::ffi::c_char,
            b"<empty>\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    let mut opt_idx: OptIndex = find_option(name.data);
    if !(opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"option name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    let Some(optval) = object_as_optval(value) else {
        api_err_exp(
            err,
            b"value\0".as_ptr() as *const ::core::ffi::c_char,
            b"valid option type\0".as_ptr() as *const ::core::ffi::c_char,
            api_typename(value.type_0),
        );
        return;
    };
    let opt_flags: ::core::ffi::c_int = if scope as ::core::ffi::c_uint
        == kOptScopeWin as ::core::ffi::c_int as ::core::ffi::c_uint
        && !option_has_scope(opt_idx, kOptScopeGlobal)
    {
        0 as ::core::ffi::c_int
    } else if scope as ::core::ffi::c_uint
        == kOptScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        OPT_GLOBAL as ::core::ffi::c_int
    } else {
        OPT_LOCAL as ::core::ffi::c_int
    };
    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
    set_option_value_for(name.data, opt_idx, optval, opt_flags, scope, to, err);
    current_sctx.set(save_current_sctx);
}
pub unsafe extern "C" fn nvim_call_atomic(
    mut channel_id: uint64_t,
    mut calls: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = arena_array(arena, 2 as size_t);
    let mut results: Array = arena_array(arena, calls.size);
    let mut nested_error: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut i: size_t = 0;
    i = 0 as size_t;
    '_theend: {
        while i < calls.size {
            if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                != (*calls.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
            {
                api_err_exp(
                    err,
                    b"'calls' item\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(kObjectTypeArray),
                    api_typename((*calls.items.offset(i as isize)).type_0),
                );
                break '_theend;
            } else {
                let mut call: Array = (*calls.items.offset(i as isize)).data.array;
                if !(call.size == 2 as size_t) {
                    api_err_exp(
                        err,
                        b"'calls' item\0".as_ptr() as *const ::core::ffi::c_char,
                        b"2-item Array\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_theend;
                } else if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    != (*call.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                        as ::core::ffi::c_uint
                {
                    api_err_exp(
                        err,
                        b"name\0".as_ptr() as *const ::core::ffi::c_char,
                        api_typename(kObjectTypeString),
                        api_typename((*call.items.offset(0 as ::core::ffi::c_int as isize)).type_0),
                    );
                    break '_theend;
                } else {
                    let mut name: String_0 = (*call.items.offset(0 as ::core::ffi::c_int as isize))
                        .data
                        .string;
                    if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                        != (*call.items.offset(1 as ::core::ffi::c_int as isize)).type_0
                            as ::core::ffi::c_uint
                    {
                        api_err_exp(
                            err,
                            b"call args\0".as_ptr() as *const ::core::ffi::c_char,
                            api_typename(kObjectTypeArray),
                            api_typename(
                                (*call.items.offset(1 as ::core::ffi::c_int as isize)).type_0,
                            ),
                        );
                        break '_theend;
                    } else {
                        let mut args: Array =
                            (*call.items.offset(1 as ::core::ffi::c_int as isize))
                                .data
                                .array;
                        let mut handler: MsgpackRpcRequestHandler = msgpack_rpc_get_handler_for(
                            name.data,
                            name.size,
                            &raw mut nested_error,
                        );
                        if nested_error.type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break;
                        }
                        let mut result: Object = handler.fn_0.expect("non-null function pointer")(
                            channel_id,
                            args,
                            arena,
                            &raw mut nested_error,
                        );
                        if nested_error.type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break;
                        }
                        let c2rust_fresh0 = results.size;
                        results.size = results.size.wrapping_add(1);
                        *results.items.offset(c2rust_fresh0 as isize) = copy_object(result, arena);
                        if handler.ret_alloc {
                            api_free_object(result);
                        }
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
        let c2rust_fresh1 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: results },
        };
        if nested_error.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            let mut errval: Array = arena_array(arena, 3 as size_t);
            let c2rust_fresh2 = errval.size;
            errval.size = errval.size.wrapping_add(1);
            *errval.items.offset(c2rust_fresh2 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: i as Integer,
                },
            };
            let c2rust_fresh3 = errval.size;
            errval.size = errval.size.wrapping_add(1);
            *errval.items.offset(c2rust_fresh3 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: nested_error.type_0 as Integer,
                },
            };
            let c2rust_fresh4 = errval.size;
            errval.size = errval.size.wrapping_add(1);
            *errval.items.offset(c2rust_fresh4 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: copy_string(cstr_as_string(nested_error.msg), arena),
                },
            };
            let c2rust_fresh5 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh5 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: errval },
            };
        } else {
            let c2rust_fresh6 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh6 as isize) = object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
    }
    api_clear_error(&raw mut nested_error);
    return rv;
}
pub unsafe extern "C" fn nvim_subscribe(mut _channel_id: uint64_t, mut _event: String_0) {}
pub unsafe extern "C" fn nvim_unsubscribe(mut _channel_id: uint64_t, mut _event: String_0) {}
unsafe extern "C" fn write_msg(mut message: String_0, mut to_err: bool, mut writeln: bool) {
    static out_line_buf: GlobalCell<StringBuilder> = GlobalCell::new(StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    });
    static err_line_buf: GlobalCell<StringBuilder> = GlobalCell::new(StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    });
    let mut line_buf: *mut StringBuilder = if to_err as ::core::ffi::c_int != 0 {
        err_line_buf.ptr()
    } else {
        out_line_buf.ptr()
    };
    (*no_wait_return.ptr()) += 1;
    let mut i: uint32_t = 0 as uint32_t;
    while (i as size_t) < message.size {
        if got_int.get() {
            break;
        }
        if (*line_buf).capacity == 0 as size_t {
            (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
            (*line_buf).items = xrealloc(
                (*line_buf).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*line_buf).capacity),
            ) as *mut ::core::ffi::c_char;
        }
        if *message.data.offset(i as isize) as ::core::ffi::c_int == NL {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh7 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh7 as isize) = '\0' as ::core::ffi::c_char;
            if to_err {
                emsg((*line_buf).items);
            } else {
                msg((*line_buf).items, 0 as ::core::ffi::c_int);
            }
            if msg_silent.get() == 0 as ::core::ffi::c_int {
                msg_didout.set(true_0 != 0);
            }
            (*line_buf).size = (*line_buf).size.wrapping_sub((*line_buf).size);
            (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
            (*line_buf).items = xrealloc(
                (*line_buf).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*line_buf).capacity),
            ) as *mut ::core::ffi::c_char;
        } else if *message.data.offset(i as isize) as ::core::ffi::c_int == NUL {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh8 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh8 as isize) = '\n' as ::core::ffi::c_char;
        } else {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh9 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh9 as isize) = *message.data.offset(i as isize);
        }
        i = i.wrapping_add(1);
    }
    if writeln {
        if (*line_buf).capacity == 0 as size_t {
            (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
            (*line_buf).items = xrealloc(
                (*line_buf).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*line_buf).capacity),
            ) as *mut ::core::ffi::c_char;
        }
        if '\n' as ::core::ffi::c_int == NL {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh10 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh10 as isize) = '\0' as ::core::ffi::c_char;
            if to_err {
                emsg((*line_buf).items);
            } else {
                msg((*line_buf).items, 0 as ::core::ffi::c_int);
            }
            if msg_silent.get() == 0 as ::core::ffi::c_int {
                msg_didout.set(true_0 != 0);
            }
            (*line_buf).size = (*line_buf).size.wrapping_sub((*line_buf).size);
            (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
            (*line_buf).items = xrealloc(
                (*line_buf).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*line_buf).capacity),
            ) as *mut ::core::ffi::c_char;
        } else if '\n' as ::core::ffi::c_int == NUL {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh11 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh11 as isize) = '\n' as ::core::ffi::c_char;
        } else {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh12 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh12 as isize) = '\n' as ::core::ffi::c_char;
        }
    }
    (*no_wait_return.ptr()) -= 1;
    msg_end();
}
pub unsafe extern "C" fn nvim_out_write(mut str: String_0) {
    write_msg(str, false_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn nvim_err_write(mut str: String_0) {
    write_msg(str, true_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn nvim_err_writeln(mut str: String_0) {
    write_msg(str, true_0 != 0, true_0 != 0);
}
pub unsafe extern "C" fn nvim_notify(
    mut msg_0: String_0,
    mut log_level: Integer,
    mut opts: Dict,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 3] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 3];
    args.capacity = 3 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh13 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh13 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: msg_0 },
    };
    let c2rust_fresh14 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh14 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: log_level },
    };
    let c2rust_fresh15 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh15 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: opts },
    };
    return nlua_exec(
        String_0 {
            data: b"return vim.notify(...)\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetObject,
        arena,
        err,
    );
}
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    conceal_char: 0 as schar_T,
};
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false_0 != 0,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_HL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 8 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
