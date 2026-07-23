use crate::src::nvim::event::libuv::{uv_dlclose, uv_dlerror, uv_dlopen, uv_dlsym};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::lua::ffi::{
    luaL_argerror, luaL_checkinteger, luaL_checklstring, luaL_checknumber, luaL_checkudata,
    luaL_error, luaL_newmetatable, luaL_ref, luaL_register, luaL_unref, lua_concat,
    lua_createtable, lua_error, lua_getfenv, lua_getfield, lua_gettop, lua_isstring,
    lua_newuserdata, lua_objlen, lua_pcall, lua_pushboolean, lua_pushcclosure, lua_pushinteger,
    lua_pushlstring, lua_pushnil, lua_pushnumber, lua_pushstring, lua_pushvalue, lua_rawgeti,
    lua_rawseti, lua_setfenv, lua_setfield, lua_setmetatable, lua_settop, lua_toboolean,
    lua_tointeger, lua_tolstring, lua_touserdata, lua_type,
};
use crate::src::nvim::main::{buffer_handles, tslua_query_parse_count, IObuff};
use crate::src::nvim::map::{
    map_del_cstr_t_ptr_t, map_put_ref_cstr_t_ptr_t, mh_get_cstr_t, mh_get_int,
};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{
    memchrsub, strequal, xcalloc, xfree, xmalloc, xrealloc, xstrdup, xstrlcpy,
};
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, abort, memcmp, memcpy, snprintf, strchr, strlen,
};
use crate::src::nvim::os::time::os_hrtime;
use crate::src::nvim::strings::vim_snprintf;
pub use crate::src::nvim::types::{
    __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T,
    colnr_T, cstr_t, dict_T, dictvar_S, disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, luaL_Reg,
    lua_CFunction, lua_Integer, lua_Number, lua_State, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, partial_S, partial_T,
    pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, taggy_T, terminal,
    time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, uv_lib_t, varnumber_T, virt_line,
    visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, AdditionalData,
    AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2,
    ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection, LuaRef, MTKey,
    MTNode, MTPos, MapHash, Map_cstr_t_ptr_t, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_int_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, OptInt, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_cstr_t, Set_int, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal,
    Timestamp, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig,
    WinInfo, WinSplit, WinStyle, Window, QUEUE,
};
extern "C" {
    pub type TSLanguage;
    pub type TSParser;
    pub type TSTree;
    pub type TSQuery;
    pub type TSQueryCursor;
    fn ts_parser_new() -> *mut TSParser;
    fn ts_parser_delete(self_0: *mut TSParser);
    fn ts_parser_language(self_0: *const TSParser) -> *const TSLanguage;
    fn ts_parser_set_language(self_0: *mut TSParser, language: *const TSLanguage) -> bool;
    fn ts_parser_set_included_ranges(
        self_0: *mut TSParser,
        ranges: *const TSRange,
        count: uint32_t,
    ) -> bool;
    fn ts_parser_included_ranges(self_0: *const TSParser, count: *mut uint32_t) -> *const TSRange;
    fn ts_parser_parse(
        self_0: *mut TSParser,
        old_tree: *const TSTree,
        input: TSInput,
    ) -> *mut TSTree;
    fn ts_parser_parse_with_options(
        self_0: *mut TSParser,
        old_tree: *const TSTree,
        input: TSInput,
        parse_options: TSParseOptions,
    ) -> *mut TSTree;
    fn ts_parser_parse_string(
        self_0: *mut TSParser,
        old_tree: *const TSTree,
        string: *const ::core::ffi::c_char,
        length: uint32_t,
    ) -> *mut TSTree;
    fn ts_parser_reset(self_0: *mut TSParser);
    fn ts_parser_set_logger(self_0: *mut TSParser, logger: TSLogger);
    fn ts_parser_logger(self_0: *const TSParser) -> TSLogger;
    fn ts_tree_copy(self_0: *const TSTree) -> *mut TSTree;
    fn ts_tree_delete(self_0: *mut TSTree);
    fn ts_tree_root_node(self_0: *const TSTree) -> TSNode;
    fn ts_tree_included_ranges(self_0: *const TSTree, length: *mut uint32_t) -> *mut TSRange;
    fn ts_tree_edit(self_0: *mut TSTree, edit: *const TSInputEdit);
    fn ts_tree_get_changed_ranges(
        old_tree: *const TSTree,
        new_tree: *const TSTree,
        length: *mut uint32_t,
    ) -> *mut TSRange;
    fn ts_node_type(self_0: TSNode) -> *const ::core::ffi::c_char;
    fn ts_node_symbol(self_0: TSNode) -> TSSymbol;
    fn ts_node_start_byte(self_0: TSNode) -> uint32_t;
    fn ts_node_start_point(self_0: TSNode) -> TSPoint;
    fn ts_node_end_byte(self_0: TSNode) -> uint32_t;
    fn ts_node_end_point(self_0: TSNode) -> TSPoint;
    fn ts_node_string(self_0: TSNode) -> *mut ::core::ffi::c_char;
    fn ts_node_is_null(self_0: TSNode) -> bool;
    fn ts_node_is_named(self_0: TSNode) -> bool;
    fn ts_node_is_missing(self_0: TSNode) -> bool;
    fn ts_node_is_extra(self_0: TSNode) -> bool;
    fn ts_node_has_changes(self_0: TSNode) -> bool;
    fn ts_node_has_error(self_0: TSNode) -> bool;
    fn ts_node_parent(self_0: TSNode) -> TSNode;
    fn ts_node_child_with_descendant(self_0: TSNode, descendant: TSNode) -> TSNode;
    fn ts_node_child(self_0: TSNode, child_index: uint32_t) -> TSNode;
    fn ts_node_field_name_for_child(
        self_0: TSNode,
        child_index: uint32_t,
    ) -> *const ::core::ffi::c_char;
    fn ts_node_child_count(self_0: TSNode) -> uint32_t;
    fn ts_node_named_child(self_0: TSNode, child_index: uint32_t) -> TSNode;
    fn ts_node_named_child_count(self_0: TSNode) -> uint32_t;
    fn ts_node_next_sibling(self_0: TSNode) -> TSNode;
    fn ts_node_prev_sibling(self_0: TSNode) -> TSNode;
    fn ts_node_next_named_sibling(self_0: TSNode) -> TSNode;
    fn ts_node_prev_named_sibling(self_0: TSNode) -> TSNode;
    fn ts_node_descendant_for_point_range(self_0: TSNode, start: TSPoint, end: TSPoint) -> TSNode;
    fn ts_node_named_descendant_for_point_range(
        self_0: TSNode,
        start: TSPoint,
        end: TSPoint,
    ) -> TSNode;
    fn ts_node_eq(self_0: TSNode, other: TSNode) -> bool;
    fn ts_query_new(
        language: *const TSLanguage,
        source: *const ::core::ffi::c_char,
        source_len: uint32_t,
        error_offset: *mut uint32_t,
        error_type: *mut TSQueryError,
    ) -> *mut TSQuery;
    fn ts_query_delete(self_0: *mut TSQuery);
    fn ts_query_pattern_count(self_0: *const TSQuery) -> uint32_t;
    fn ts_query_capture_count(self_0: *const TSQuery) -> uint32_t;
    fn ts_query_predicates_for_pattern(
        self_0: *const TSQuery,
        pattern_index: uint32_t,
        step_count: *mut uint32_t,
    ) -> *const TSQueryPredicateStep;
    fn ts_query_capture_name_for_id(
        self_0: *const TSQuery,
        index: uint32_t,
        length: *mut uint32_t,
    ) -> *const ::core::ffi::c_char;
    fn ts_query_string_value_for_id(
        self_0: *const TSQuery,
        index: uint32_t,
        length: *mut uint32_t,
    ) -> *const ::core::ffi::c_char;
    fn ts_query_disable_capture(
        self_0: *mut TSQuery,
        name: *const ::core::ffi::c_char,
        length: uint32_t,
    );
    fn ts_query_disable_pattern(self_0: *mut TSQuery, pattern_index: uint32_t);
    fn ts_query_cursor_new() -> *mut TSQueryCursor;
    fn ts_query_cursor_delete(self_0: *mut TSQueryCursor);
    fn ts_query_cursor_exec(self_0: *mut TSQueryCursor, query: *const TSQuery, node: TSNode);
    fn ts_query_cursor_set_match_limit(self_0: *mut TSQueryCursor, limit: uint32_t);
    fn ts_query_cursor_set_point_range(
        self_0: *mut TSQueryCursor,
        start_point: TSPoint,
        end_point: TSPoint,
    ) -> bool;
    fn ts_query_cursor_next_match(self_0: *mut TSQueryCursor, match_0: *mut TSQueryMatch) -> bool;
    fn ts_query_cursor_remove_match(self_0: *mut TSQueryCursor, match_id: uint32_t);
    fn ts_query_cursor_next_capture(
        self_0: *mut TSQueryCursor,
        match_0: *mut TSQueryMatch,
        capture_index: *mut uint32_t,
    ) -> bool;
    fn ts_query_cursor_set_max_start_depth(self_0: *mut TSQueryCursor, max_start_depth: uint32_t);
    fn ts_language_symbol_count(self_0: *const TSLanguage) -> uint32_t;
    fn ts_language_state_count(self_0: *const TSLanguage) -> uint32_t;
    fn ts_language_field_count(self_0: *const TSLanguage) -> uint32_t;
    fn ts_language_field_name_for_id(
        self_0: *const TSLanguage,
        id: TSFieldId,
    ) -> *const ::core::ffi::c_char;
    fn ts_language_supertypes(self_0: *const TSLanguage, length: *mut uint32_t) -> *const TSSymbol;
    fn ts_language_subtypes(
        self_0: *const TSLanguage,
        supertype: TSSymbol,
        length: *mut uint32_t,
    ) -> *const TSSymbol;
    fn ts_language_symbol_name(
        self_0: *const TSLanguage,
        symbol: TSSymbol,
    ) -> *const ::core::ffi::c_char;
    fn ts_language_symbol_type(self_0: *const TSLanguage, symbol: TSSymbol) -> TSSymbolType;
    fn ts_language_abi_version(self_0: *const TSLanguage) -> uint32_t;
    fn ts_language_metadata(self_0: *const TSLanguage) -> *const TSLanguageMetadata;
    fn ts_language_is_wasm(_: *const TSLanguage) -> bool;
    fn ts_set_allocator(
        new_malloc: Option<unsafe extern "C" fn(size_t) -> *mut ::core::ffi::c_void>,
        new_calloc: Option<unsafe extern "C" fn(size_t, size_t) -> *mut ::core::ffi::c_void>,
        new_realloc: Option<
            unsafe extern "C" fn(*mut ::core::ffi::c_void, size_t) -> *mut ::core::ffi::c_void,
        >,
        new_free: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    );
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
pub type TSSymbol = uint16_t;
pub type TSFieldId = uint16_t;
pub type TSDecodeFunction =
    Option<unsafe extern "C" fn(*const uint8_t, uint32_t, *mut int32_t) -> uint32_t>;
pub type TSInputEncoding = ::core::ffi::c_uint;
pub const TSInputEncodingCustom: TSInputEncoding = 3;
pub const TSInputEncodingUTF16BE: TSInputEncoding = 2;
pub const TSInputEncodingUTF16LE: TSInputEncoding = 1;
pub const TSInputEncodingUTF8: TSInputEncoding = 0;
pub type TSSymbolType = ::core::ffi::c_uint;
pub const TSSymbolTypeAuxiliary: TSSymbolType = 3;
pub const TSSymbolTypeSupertype: TSSymbolType = 2;
pub const TSSymbolTypeAnonymous: TSSymbolType = 1;
pub const TSSymbolTypeRegular: TSSymbolType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSPoint {
    pub row: uint32_t,
    pub column: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSRange {
    pub start_point: TSPoint,
    pub end_point: TSPoint,
    pub start_byte: uint32_t,
    pub end_byte: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSInput {
    pub payload: *mut ::core::ffi::c_void,
    pub read: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            uint32_t,
            TSPoint,
            *mut uint32_t,
        ) -> *const ::core::ffi::c_char,
    >,
    pub encoding: TSInputEncoding,
    pub decode: TSDecodeFunction,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSParseState {
    pub payload: *mut ::core::ffi::c_void,
    pub current_byte_offset: uint32_t,
    pub has_error: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSParseOptions {
    pub payload: *mut ::core::ffi::c_void,
    pub progress_callback: Option<unsafe extern "C" fn(*mut TSParseState) -> bool>,
}
pub type TSLogType = ::core::ffi::c_uint;
pub const TSLogTypeLex: TSLogType = 1;
pub const TSLogTypeParse: TSLogType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLogger {
    pub payload: *mut ::core::ffi::c_void,
    pub log: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, TSLogType, *const ::core::ffi::c_char) -> (),
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSInputEdit {
    pub start_byte: uint32_t,
    pub old_end_byte: uint32_t,
    pub new_end_byte: uint32_t,
    pub start_point: TSPoint,
    pub old_end_point: TSPoint,
    pub new_end_point: TSPoint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSNode {
    pub context: [uint32_t; 4],
    pub id: *const ::core::ffi::c_void,
    pub tree: *const TSTree,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryCapture {
    pub node: TSNode,
    pub index: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryMatch {
    pub id: uint32_t,
    pub pattern_index: uint16_t,
    pub capture_count: uint16_t,
    pub captures: *const TSQueryCapture,
}
pub type TSQueryPredicateStepType = ::core::ffi::c_uint;
pub const TSQueryPredicateStepTypeString: TSQueryPredicateStepType = 2;
pub const TSQueryPredicateStepTypeCapture: TSQueryPredicateStepType = 1;
pub const TSQueryPredicateStepTypeDone: TSQueryPredicateStepType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSQueryPredicateStep {
    pub type_0: TSQueryPredicateStepType,
    pub value_id: uint32_t,
}
pub type TSQueryError = ::core::ffi::c_uint;
pub const TSQueryErrorLanguage: TSQueryError = 6;
pub const TSQueryErrorStructure: TSQueryError = 5;
pub const TSQueryErrorCapture: TSQueryError = 4;
pub const TSQueryErrorField: TSQueryError = 3;
pub const TSQueryErrorNodeType: TSQueryError = 2;
pub const TSQueryErrorSyntax: TSQueryError = 1;
pub const TSQueryErrorNone: TSQueryError = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLanguageMetadata {
    pub major_version: uint8_t,
    pub minor_version: uint8_t,
    pub patch_version: uint8_t,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLuaTree {
    pub tree: *const TSTree,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLuaLoggerOpts {
    pub cb: LuaRef,
    pub lstate: *mut lua_State,
    pub lex: bool,
    pub parse: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLuaParserCallbackPayload {
    pub parse_start_time: uint64_t,
    pub timeout_threshold_ns: uint64_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_REGISTRYINDEX: ::core::ffi::c_int = -10000 as ::core::ffi::c_int;
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002 as ::core::ffi::c_int;
pub const LUA_TNIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_TNUMBER: ::core::ffi::c_int = 3;
pub const LUA_TSTRING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const LUA_TTABLE: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const TREE_SITTER_LANGUAGE_VERSION: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION: ::core::ffi::c_int =
    13 as ::core::ffi::c_int;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_cstr_t = Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
};
pub const MAP_INIT: Map_cstr_t_ptr_t = Map_cstr_t_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C-unwind" fn set_has_cstr_t(mut set: *mut Set_cstr_t, mut key: cstr_t) -> bool {
    return mh_get_cstr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
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
#[inline]
unsafe extern "C-unwind" fn map_put_cstr_t_ptr_t(
    mut map: *mut Map_cstr_t_ptr_t,
    mut key: cstr_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_cstr_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut cstr_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C-unwind" fn map_get_cstr_t_ptr_t(
    mut map: *mut Map_cstr_t_ptr_t,
    mut key: cstr_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_cstr_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TS_META_PARSER: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"treesitter_parser\0")
};
pub const TS_META_TREE: [::core::ffi::c_char; 16] =
    unsafe { ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"treesitter_tree\0") };
pub const TS_META_NODE: [::core::ffi::c_char; 16] =
    unsafe { ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"treesitter_node\0") };
pub const TS_META_QUERY: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"treesitter_query\0")
};
pub const TS_META_QUERYCURSOR: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"treesitter_querycursor\0")
};
pub const TS_META_QUERYMATCH: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"treesitter_querymatch\0")
};
static langs: GlobalCell<Map_cstr_t_ptr_t> = GlobalCell::new(MAP_INIT);
unsafe extern "C-unwind" fn tslua_has_language(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
        L,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    lua_pushboolean(
        L,
        set_has_cstr_t(&raw mut (*langs.ptr()).set, lang_name as cstr_t) as ::core::ffi::c_int,
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn tslua_add_language_from_object(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    return add_language(L, false_0 != 0);
}
unsafe extern "C-unwind" fn load_language_from_object(
    mut L: *mut lua_State,
    mut path: *const ::core::ffi::c_char,
    mut lang_name: *const ::core::ffi::c_char,
    mut symbol: *const ::core::ffi::c_char,
) -> *const TSLanguage {
    let mut lib: uv_lib_t = uv_lib_t {
        handle: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if uv_dlopen(path, &raw mut lib) != 0 {
        xstrlcpy(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            uv_dlerror(&raw mut lib),
            ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        );
        uv_dlclose(&raw mut lib);
        luaL_error(
            L,
            b"Failed to load parser for language '%s': uv_dlopen: %s\0".as_ptr()
                as *const ::core::ffi::c_char,
            lang_name,
            IObuff.ptr() as *mut ::core::ffi::c_char,
        );
    }
    let mut symbol_buf: [::core::ffi::c_char; 128] = [0; 128];
    snprintf(
        &raw mut symbol_buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 128]>(),
        b"tree_sitter_%s\0".as_ptr() as *const ::core::ffi::c_char,
        symbol,
    );
    let mut lang_parser: Option<unsafe extern "C" fn() -> *mut TSLanguage> = None;
    if uv_dlsym(
        &raw mut lib,
        &raw mut symbol_buf as *mut ::core::ffi::c_char,
        &raw mut lang_parser as *mut *mut ::core::ffi::c_void,
    ) != 0
    {
        xstrlcpy(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            uv_dlerror(&raw mut lib),
            ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        );
        uv_dlclose(&raw mut lib);
        luaL_error(
            L,
            b"Failed to load parser: uv_dlsym: %s\0".as_ptr() as *const ::core::ffi::c_char,
            IObuff.ptr() as *mut ::core::ffi::c_char,
        );
    }
    let mut lang: *mut TSLanguage = lang_parser.expect("non-null function pointer")();
    if lang.is_null() {
        uv_dlclose(&raw mut lib);
        luaL_error(
            L,
            b"Failed to load parser %s: internal error\0".as_ptr() as *const ::core::ffi::c_char,
            path,
        );
    }
    return lang;
}
unsafe extern "C-unwind" fn load_language_from_wasm(
    mut L: *mut lua_State,
    mut _path: *const ::core::ffi::c_char,
    mut _lang_name: *const ::core::ffi::c_char,
) -> *const TSLanguage {
    luaL_error(L, b"Not supported\0".as_ptr() as *const ::core::ffi::c_char);
    return ::core::ptr::null::<TSLanguage>();
}
unsafe extern "C-unwind" fn add_language(
    mut L: *mut lua_State,
    mut is_wasm: bool,
) -> ::core::ffi::c_int {
    let mut path: *const ::core::ffi::c_char = luaL_checklstring(
        L,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
        L,
        2 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut symbol_name: *const ::core::ffi::c_char = lang_name;
    if !is_wasm
        && lua_gettop(L) >= 3 as ::core::ffi::c_int
        && !(lua_type(L, 3 as ::core::ffi::c_int) == LUA_TNIL)
    {
        symbol_name = luaL_checklstring(
            L,
            3 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
    }
    if set_has_cstr_t(&raw mut (*langs.ptr()).set, lang_name as cstr_t) {
        lua_pushboolean(L, true_0);
        return 1 as ::core::ffi::c_int;
    }
    let mut lang: *const TSLanguage = if is_wasm as ::core::ffi::c_int != 0 {
        load_language_from_wasm(L, path, lang_name)
    } else {
        load_language_from_object(L, path, lang_name, symbol_name)
    };
    let mut lang_version: uint32_t = ts_language_abi_version(lang);
    if lang_version < TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION as uint32_t
        || lang_version > TREE_SITTER_LANGUAGE_VERSION as uint32_t
    {
        return luaL_error(
            L,
            b"ABI version mismatch for %s: supported between %d and %d, found %d\0".as_ptr()
                as *const ::core::ffi::c_char,
            path,
            TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
            TREE_SITTER_LANGUAGE_VERSION,
            lang_version,
        );
    }
    map_put_cstr_t_ptr_t(
        langs.ptr(),
        xstrdup(lang_name) as cstr_t,
        lang as *mut TSLanguage as ptr_t,
    );
    lua_pushboolean(L, true_0);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn tslua_remove_lang(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
        L,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    let mut present: bool = set_has_cstr_t(&raw mut (*langs.ptr()).set, lang_name as cstr_t);
    if present {
        let mut key: cstr_t = ::core::ptr::null::<::core::ffi::c_char>();
        map_del_cstr_t_ptr_t(langs.ptr(), lang_name as cstr_t, &raw mut key);
        xfree(key as *mut ::core::ffi::c_void);
    }
    lua_pushboolean(L, present as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lang_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSLanguage {
    let mut lang_name: *const ::core::ffi::c_char =
        luaL_checklstring(L, index, ::core::ptr::null_mut::<size_t>());
    let mut lang: *mut TSLanguage =
        map_get_cstr_t_ptr_t(langs.ptr(), lang_name as cstr_t) as *mut TSLanguage;
    if lang.is_null() {
        luaL_error(
            L,
            b"no such language: %s\0".as_ptr() as *const ::core::ffi::c_char,
            lang_name,
        );
    }
    return lang;
}
unsafe extern "C-unwind" fn tslua_inspect_lang(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut lang: *mut TSLanguage = lang_check(L, 1 as ::core::ffi::c_int);
    lua_createtable(L, 0 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
    let mut nsymbols: uint32_t = ts_language_symbol_count(lang);
    '_c2rust_label: {
        if nsymbols < 2147483647 as uint32_t {
        } else {
            __assert_fail(
                b"nsymbols < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/treesitter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                276 as ::core::ffi::c_uint,
                b"int tslua_inspect_lang(lua_State *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    lua_createtable(
        L,
        nsymbols.wrapping_sub(1 as uint32_t) as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    let mut i: uint32_t = 0 as uint32_t;
    while i < nsymbols {
        let mut t: TSSymbolType = ts_language_symbol_type(lang, i as TSSymbol);
        if t as ::core::ffi::c_uint
            != TSSymbolTypeAuxiliary as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut name: *const ::core::ffi::c_char = ts_language_symbol_name(lang, i as TSSymbol);
            let mut named: bool = t as ::core::ffi::c_uint
                != TSSymbolTypeAnonymous as ::core::ffi::c_int as ::core::ffi::c_uint;
            lua_pushboolean(L, named as ::core::ffi::c_int);
            if !named {
                let mut buf: [::core::ffi::c_char; 256] = [0; 256];
                snprintf(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
                    b"\"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                    name,
                );
                lua_setfield(
                    L,
                    -2 as ::core::ffi::c_int,
                    &raw mut buf as *mut ::core::ffi::c_char,
                );
            } else {
                lua_setfield(L, -2 as ::core::ffi::c_int, name);
            }
        }
        i = i.wrapping_add(1);
    }
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"symbols\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut nfields: uint32_t = ts_language_field_count(lang);
    lua_createtable(L, nfields as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    let mut i_0: uint32_t = 1 as uint32_t;
    while i_0 <= nfields {
        lua_pushstring(L, ts_language_field_name_for_id(lang, i_0 as TSFieldId));
        lua_rawseti(L, -2 as ::core::ffi::c_int, i_0 as ::core::ffi::c_int);
        i_0 = i_0.wrapping_add(1);
    }
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"fields\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(L, ts_language_is_wasm(lang) as ::core::ffi::c_int);
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"_wasm\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushinteger(L, ts_language_abi_version(lang) as lua_Integer);
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"abi_version\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut meta: *const TSLanguageMetadata = ts_language_metadata(lang);
    if !meta.is_null() {
        lua_createtable(L, 0 as ::core::ffi::c_int, 3 as ::core::ffi::c_int);
        lua_pushinteger(L, (*meta).major_version as lua_Integer);
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"major_version\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushinteger(L, (*meta).minor_version as lua_Integer);
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"minor_version\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushinteger(L, (*meta).patch_version as lua_Integer);
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"patch_version\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"metadata\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    lua_pushinteger(L, ts_language_state_count(lang) as lua_Integer);
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"state_count\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut nsupertypes: uint32_t = 0;
    let mut supertypes: *const TSSymbol = ts_language_supertypes(lang, &raw mut nsupertypes);
    lua_createtable(
        L,
        0 as ::core::ffi::c_int,
        nsupertypes as ::core::ffi::c_int,
    );
    let mut i_1: uint32_t = 0 as uint32_t;
    while i_1 < nsupertypes {
        let supertype: TSSymbol = *supertypes.offset(i_1 as isize);
        let mut nsubtypes: uint32_t = 0;
        let mut subtypes: *const TSSymbol =
            ts_language_subtypes(lang, supertype, &raw mut nsubtypes);
        lua_createtable(L, nsubtypes as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        let mut j: uint32_t = 1 as uint32_t;
        while j <= nsubtypes {
            lua_pushstring(
                L,
                ts_language_symbol_name(lang, *subtypes.offset(j as isize)),
            );
            lua_rawseti(L, -2 as ::core::ffi::c_int, j as ::core::ffi::c_int);
            j = j.wrapping_add(1);
        }
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            ts_language_symbol_name(lang, supertype),
        );
        i_1 = i_1.wrapping_add(1);
    }
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"supertypes\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return 1 as ::core::ffi::c_int;
}
static parser_meta: GlobalCell<[luaL_Reg; 9]> = GlobalCell::new([
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(parser_gc as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            parser_tostring as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"parse\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            parser_parse as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"reset\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            parser_reset as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"set_included_ranges\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            parser_set_ranges as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"included_ranges\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            parser_get_ranges as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"_set_logger\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            parser_set_logger as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"_logger\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            parser_get_logger as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
unsafe extern "C-unwind" fn tslua_push_parser(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut lang: *mut TSLanguage = lang_check(L, 1 as ::core::ffi::c_int);
    let mut parser: *mut *mut TSParser =
        lua_newuserdata(L, ::core::mem::size_of::<*mut TSParser>()) as *mut *mut TSParser;
    *parser = ts_parser_new();
    if !ts_parser_set_language(*parser, lang) {
        ts_parser_delete(*parser);
        let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
            L,
            1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        return luaL_error(
            L,
            b"Failed to load language : %s\0".as_ptr() as *const ::core::ffi::c_char,
            lang_name,
        );
    }
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_PARSER.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn parser_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSParser {
    let mut ud: *mut *mut TSParser =
        luaL_checkudata(L, index, TS_META_PARSER.as_ptr()) as *mut *mut TSParser;
    (!(*ud).is_null()
        || luaL_argerror(
            L,
            index,
            b"Parser has been deleted\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    return *ud;
}
unsafe extern "C-unwind" fn logger_gc(mut logger: TSLogger) {
    if logger.log.is_none() {
        return;
    }
    let mut opts: *mut TSLuaLoggerOpts = logger.payload as *mut TSLuaLoggerOpts;
    luaL_unref(
        (*opts).lstate,
        LUA_REGISTRYINDEX,
        (*opts).cb as ::core::ffi::c_int,
    );
    xfree(opts as *mut ::core::ffi::c_void);
}
unsafe extern "C-unwind" fn parser_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut ud: *mut *mut TSParser =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_PARSER.as_ptr()) as *mut *mut TSParser;
    if !(*ud).is_null() {
        logger_gc(ts_parser_logger(*ud));
        ts_parser_delete(*ud);
        *ud = ::core::ptr::null_mut::<TSParser>();
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn parser_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushstring(L, b"<parser>\0".as_ptr() as *const ::core::ffi::c_char);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_cb(
    mut payload: *mut ::core::ffi::c_void,
    mut _byte_index: uint32_t,
    mut position: TSPoint,
    mut bytes_read: *mut uint32_t,
) -> *const ::core::ffi::c_char {
    let mut bp: *mut buf_T = payload as *mut buf_T;
    static buf: GlobalCell<[::core::ffi::c_char; 256]> = GlobalCell::new([0; 256]);
    if position.row as linenr_T >= (*bp).b_ml.ml_line_count {
        *bytes_read = 0 as uint32_t;
        return b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    let mut lnum: linenr_T = position.row as linenr_T + 1 as linenr_T;
    let mut line: *mut ::core::ffi::c_char = ml_get_buf(bp, lnum);
    let mut len: size_t = ml_get_buf_len(bp, lnum) as size_t;
    if position.column as size_t > len {
        *bytes_read = 0 as uint32_t;
        return b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    let mut tocopy: size_t = if len.wrapping_sub(position.column as size_t) < 256 as size_t {
        len.wrapping_sub(position.column as size_t)
    } else {
        256 as size_t
    };
    memcpy(
        buf.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        line.offset(position.column as isize) as *const ::core::ffi::c_void,
        tocopy,
    );
    memchrsub(
        buf.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        '\n' as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
        tocopy,
    );
    *bytes_read = tocopy as uint32_t;
    if tocopy < BUFSIZE as size_t {
        if lnum != (*bp).b_ml.ml_line_count
            || (*bp).b_p_bin == 0 && (*bp).b_p_fixeol != 0
            || lnum != (*bp).b_no_eol_lnum && (*bp).b_p_eol != 0
        {
            (*buf.ptr())[tocopy as usize] = '\n' as ::core::ffi::c_char;
            *bytes_read = (*bytes_read).wrapping_add(1);
        }
    }
    return buf.ptr() as *mut ::core::ffi::c_char;
}
pub const BUFSIZE: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
unsafe extern "C-unwind" fn push_ranges(
    mut L: *mut lua_State,
    mut ranges: *const TSRange,
    length: size_t,
    mut include_bytes: bool,
) {
    lua_createtable(L, length as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut i: size_t = 0 as size_t;
    while i < length {
        lua_createtable(
            L,
            if include_bytes as ::core::ffi::c_int != 0 {
                6 as ::core::ffi::c_int
            } else {
                4 as ::core::ffi::c_int
            },
            0 as ::core::ffi::c_int,
        );
        let mut j: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        lua_pushnumber(
            L,
            (*ranges.offset(i as isize)).start_point.row as lua_Number,
        );
        let c2rust_fresh2 = j;
        j = j + 1;
        lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh2);
        lua_pushnumber(
            L,
            (*ranges.offset(i as isize)).start_point.column as lua_Number,
        );
        let c2rust_fresh3 = j;
        j = j + 1;
        lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh3);
        if include_bytes {
            lua_pushnumber(L, (*ranges.offset(i as isize)).start_byte as lua_Number);
            let c2rust_fresh4 = j;
            j = j + 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh4);
        }
        lua_pushnumber(L, (*ranges.offset(i as isize)).end_point.row as lua_Number);
        let c2rust_fresh5 = j;
        j = j + 1;
        lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh5);
        lua_pushnumber(
            L,
            (*ranges.offset(i as isize)).end_point.column as lua_Number,
        );
        let c2rust_fresh6 = j;
        j = j + 1;
        lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh6);
        if include_bytes {
            lua_pushnumber(L, (*ranges.offset(i as isize)).end_byte as lua_Number);
            let c2rust_fresh7 = j;
            j = j + 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh7);
        }
        lua_rawseti(
            L,
            -2 as ::core::ffi::c_int,
            i.wrapping_add(1 as size_t) as ::core::ffi::c_int,
        );
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn on_parser_progress(mut state: *mut TSParseState) -> bool {
    let mut payload: *mut TSLuaParserCallbackPayload =
        (*state).payload as *mut TSLuaParserCallbackPayload;
    let mut parse_time: uint64_t = os_hrtime().wrapping_sub((*payload).parse_start_time);
    return parse_time >= (*payload).timeout_threshold_ns;
}
unsafe extern "C-unwind" fn parser_parse(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    let mut old_tree: *const TSTree = ::core::ptr::null::<TSTree>();
    if !(lua_type(L, 2 as ::core::ffi::c_int) == LUA_TNIL) {
        let mut ud: *mut TSLuaTree =
            luaL_checkudata(L, 2 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
        old_tree = if !ud.is_null() {
            (*ud).tree
        } else {
            ::core::ptr::null::<TSTree>()
        };
    }
    let mut new_tree: *mut TSTree = ::core::ptr::null_mut::<TSTree>();
    let mut len: size_t = 0;
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufnr: handle_T = 0;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut input: TSInput = TSInput {
        payload: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        read: None,
        encoding: TSInputEncodingUTF8,
        decode: None,
    };
    match lua_type(L, 3 as ::core::ffi::c_int) {
        LUA_TSTRING => {
            str = lua_tolstring(L, 3 as ::core::ffi::c_int, &raw mut len);
            new_tree = ts_parser_parse_string(p, old_tree, str, len as uint32_t);
        }
        LUA_TNUMBER => {
            bufnr = lua_tointeger(L, 3 as ::core::ffi::c_int) as handle_T;
            buf =
                map_get_int_ptr_t(buffer_handles.ptr(), bufnr as ::core::ffi::c_int) as *mut buf_T;
            if buf.is_null() {
                let mut ebuf: [::core::ffi::c_char; 256] = [
                    0 as ::core::ffi::c_char,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ];
                vim_snprintf(
                    &raw mut ebuf as *mut ::core::ffi::c_char,
                    BUFSIZE_0 as size_t,
                    b"invalid buffer handle: %d\0".as_ptr() as *const ::core::ffi::c_char,
                    bufnr,
                );
                return luaL_argerror(
                    L,
                    3 as ::core::ffi::c_int,
                    &raw mut ebuf as *mut ::core::ffi::c_char,
                );
            }
            input = TSInput {
                payload: buf as *mut ::core::ffi::c_void,
                read: Some(
                    input_cb
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                            uint32_t,
                            TSPoint,
                            *mut uint32_t,
                        )
                            -> *const ::core::ffi::c_char,
                ),
                encoding: TSInputEncodingUTF8,
                decode: None,
            };
            if !(lua_type(L, 5 as ::core::ffi::c_int) == LUA_TNIL) {
                let mut timeout_ns: uint64_t =
                    lua_tointeger(L, 5 as ::core::ffi::c_int) as uint64_t;
                let mut payload: TSLuaParserCallbackPayload = TSLuaParserCallbackPayload {
                    parse_start_time: os_hrtime(),
                    timeout_threshold_ns: timeout_ns,
                };
                let mut parse_options: TSParseOptions = TSParseOptions {
                    payload: &raw mut payload as *mut ::core::ffi::c_void,
                    progress_callback: Some(
                        on_parser_progress as unsafe extern "C" fn(*mut TSParseState) -> bool,
                    ),
                };
                new_tree = ts_parser_parse_with_options(p, old_tree, input, parse_options);
            } else {
                new_tree = ts_parser_parse(p, old_tree, input);
            }
        }
        _ => {
            return luaL_argerror(
                L,
                3 as ::core::ffi::c_int,
                b"expected either string or buffer handle\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
    let mut include_bytes: bool =
        lua_gettop(L) >= 4 as ::core::ffi::c_int && lua_toboolean(L, 4 as ::core::ffi::c_int) != 0;
    if new_tree.is_null() {
        if ts_parser_language(p).is_null() {
            return luaL_error(
                L,
                b"Language was unset, or has an incompatible ABI.\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
        return 0 as ::core::ffi::c_int;
    }
    let mut n_ranges: uint32_t = 0 as uint32_t;
    let mut changed: *mut TSRange = if !old_tree.is_null() {
        ts_tree_get_changed_ranges(old_tree, new_tree, &raw mut n_ranges)
    } else {
        ts_tree_included_ranges(new_tree, &raw mut n_ranges)
    };
    push_tree(L, new_tree);
    push_ranges(L, changed, n_ranges as size_t, include_bytes);
    xfree(changed as *mut ::core::ffi::c_void);
    return 2 as ::core::ffi::c_int;
}
pub const BUFSIZE_0: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
unsafe extern "C-unwind" fn parser_reset(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    ts_parser_reset(p);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn range_err(mut L: *mut lua_State) {
    luaL_error(
        L,
        b"Ranges can only be made from 6 element long tables or nodes.\0".as_ptr()
            as *const ::core::ffi::c_char,
    );
}
unsafe extern "C-unwind" fn lua_checkuint32(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> uint32_t {
    let mut value: lua_Number = luaL_checknumber(L, index);
    let mut converted: uint32_t = value as uint32_t;
    if value < 0 as ::core::ffi::c_int as lua_Number
        || value > UINT32_MAX as lua_Number
        || converted as lua_Number != value
    {
        luaL_error(
            L,
            b"Range value out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return converted;
}
unsafe extern "C-unwind" fn range_from_lua(mut L: *mut lua_State, mut range: *mut TSRange) {
    let mut node: TSNode = TSNode {
        context: [0; 4],
        id: ::core::ptr::null::<::core::ffi::c_void>(),
        tree: ::core::ptr::null::<TSTree>(),
    };
    if lua_type(L, -1 as ::core::ffi::c_int) == LUA_TTABLE {
        if lua_objlen(L, -1 as ::core::ffi::c_int) != 6 as size_t {
            range_err(L);
        }
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        let mut start_row: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
        let mut start_col: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 3 as ::core::ffi::c_int);
        let mut start_byte: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 4 as ::core::ffi::c_int);
        let mut end_row: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 5 as ::core::ffi::c_int);
        let mut end_col: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_rawgeti(L, -1 as ::core::ffi::c_int, 6 as ::core::ffi::c_int);
        let mut end_byte: uint32_t = lua_checkuint32(L, -1 as ::core::ffi::c_int);
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        *range = TSRange {
            start_point: TSPoint {
                row: start_row,
                column: start_col,
            },
            end_point: TSPoint {
                row: end_row,
                column: end_col,
            },
            start_byte: start_byte,
            end_byte: end_byte,
        };
    } else if node_check_opt(L, -1 as ::core::ffi::c_int, &raw mut node) {
        *range = TSRange {
            start_point: ts_node_start_point(node),
            end_point: ts_node_end_point(node),
            start_byte: ts_node_start_byte(node),
            end_byte: ts_node_end_byte(node),
        };
    } else {
        range_err(L);
    };
}
unsafe extern "C-unwind" fn parser_set_ranges(mut L: *mut lua_State) -> ::core::ffi::c_int {
    if lua_gettop(L) < 2 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"not enough args to parser:set_included_ranges()\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    (lua_type(L, 2 as ::core::ffi::c_int) == 5 as ::core::ffi::c_int
        || luaL_argerror(
            L,
            2 as ::core::ffi::c_int,
            b"table expected.\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    let mut tbl_len: size_t = lua_objlen(L, 2 as ::core::ffi::c_int);
    let mut ranges: *mut TSRange =
        xmalloc(::core::mem::size_of::<TSRange>().wrapping_mul(tbl_len)) as *mut TSRange;
    let mut index: size_t = 0 as size_t;
    while index < tbl_len {
        lua_rawgeti(
            L,
            2 as ::core::ffi::c_int,
            index as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        );
        range_from_lua(L, ranges.offset(index as isize));
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        index = index.wrapping_add(1);
    }
    ts_parser_set_included_ranges(p, ranges, tbl_len as uint32_t);
    xfree(ranges as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn parser_get_ranges(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    let mut include_bytes: bool =
        lua_gettop(L) >= 2 as ::core::ffi::c_int && lua_toboolean(L, 2 as ::core::ffi::c_int) != 0;
    let mut len: uint32_t = 0;
    let mut ranges: *const TSRange = ts_parser_included_ranges(p, &raw mut len);
    push_ranges(L, ranges, len as size_t, include_bytes);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn logger_cb(
    mut payload: *mut ::core::ffi::c_void,
    mut logtype: TSLogType,
    mut s: *const ::core::ffi::c_char,
) {
    let mut opts: *mut TSLuaLoggerOpts = payload as *mut TSLuaLoggerOpts;
    if !(*opts).lex
        && logtype as ::core::ffi::c_uint
            == TSLogTypeLex as ::core::ffi::c_int as ::core::ffi::c_uint
        || !(*opts).parse
            && logtype as ::core::ffi::c_uint
                == TSLogTypeParse as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    let mut lstate: *mut lua_State = (*opts).lstate;
    lua_rawgeti(lstate, LUA_REGISTRYINDEX, (*opts).cb as ::core::ffi::c_int);
    lua_pushstring(
        lstate,
        if logtype as ::core::ffi::c_uint
            == TSLogTypeParse as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            b"parse\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"lex\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    lua_pushstring(lstate, s);
    if lua_pcall(
        lstate,
        2 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ) != 0
    {
        luaL_error(
            lstate,
            b"treesitter logger callback failed\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
unsafe extern "C-unwind" fn parser_set_logger(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    (lua_type(L, 2 as ::core::ffi::c_int) == 1 as ::core::ffi::c_int
        || luaL_argerror(
            L,
            2 as ::core::ffi::c_int,
            b"boolean expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    (lua_type(L, 3 as ::core::ffi::c_int) == 1 as ::core::ffi::c_int
        || luaL_argerror(
            L,
            3 as ::core::ffi::c_int,
            b"boolean expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    (lua_type(L, 4 as ::core::ffi::c_int) == 6 as ::core::ffi::c_int
        || luaL_argerror(
            L,
            4 as ::core::ffi::c_int,
            b"function expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    let mut opts: *mut TSLuaLoggerOpts =
        xmalloc(::core::mem::size_of::<TSLuaLoggerOpts>()) as *mut TSLuaLoggerOpts;
    lua_pushvalue(L, 4 as ::core::ffi::c_int);
    let mut ref_0: LuaRef = luaL_ref(L, LUA_REGISTRYINDEX);
    *opts = TSLuaLoggerOpts {
        cb: ref_0,
        lstate: L,
        lex: lua_toboolean(L, 2 as ::core::ffi::c_int) != 0,
        parse: lua_toboolean(L, 3 as ::core::ffi::c_int) != 0,
    };
    let mut logger: TSLogger = TSLogger {
        payload: opts as *mut ::core::ffi::c_void,
        log: Some(
            logger_cb
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    TSLogType,
                    *const ::core::ffi::c_char,
                ) -> (),
        ),
    };
    ts_parser_set_logger(p, logger);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn parser_get_logger(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
    let mut logger: TSLogger = ts_parser_logger(p);
    if logger.log.is_some() {
        let mut opts: *mut TSLuaLoggerOpts = logger.payload as *mut TSLuaLoggerOpts;
        lua_rawgeti(L, LUA_REGISTRYINDEX, (*opts).cb as ::core::ffi::c_int);
    } else {
        lua_pushnil(L);
    }
    return 1 as ::core::ffi::c_int;
}
static tree_meta: GlobalCell<[luaL_Reg; 7]> = GlobalCell::new([
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(tree_gc as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            tree_tostring as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"root\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(tree_root as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"edit\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(tree_edit as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"included_ranges\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            tree_get_ranges as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"copy\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(tree_copy as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
unsafe extern "C-unwind" fn push_tree(mut L: *mut lua_State, mut tree: *const TSTree) {
    if tree.is_null() {
        lua_pushnil(L);
        return;
    }
    let mut ud: *mut TSLuaTree =
        lua_newuserdata(L, ::core::mem::size_of::<TSLuaTree>()) as *mut TSLuaTree;
    (*ud).tree = tree;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_TREE.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
}
unsafe extern "C-unwind" fn tree_copy(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut ud: *mut TSLuaTree =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    let mut copy: *mut TSTree = ts_tree_copy((*ud).tree);
    push_tree(L, copy);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn tree_edit(mut L: *mut lua_State) -> ::core::ffi::c_int {
    if lua_gettop(L) < 10 as ::core::ffi::c_int {
        lua_pushstring(
            L,
            b"not enough args to tree:edit()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return lua_error(L);
    }
    let mut ud: *mut TSLuaTree =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    let mut start_byte: uint32_t =
        luaL_checkinteger(L, 2 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t;
    let mut old_end_byte: uint32_t =
        luaL_checkinteger(L, 3 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t;
    let mut new_end_byte: uint32_t =
        luaL_checkinteger(L, 4 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t;
    let mut start_point: TSPoint = TSPoint {
        row: luaL_checkinteger(L, 5 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
        column: luaL_checkinteger(L, 6 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
    };
    let mut old_end_point: TSPoint = TSPoint {
        row: luaL_checkinteger(L, 7 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
        column: luaL_checkinteger(L, 8 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
    };
    let mut new_end_point: TSPoint = TSPoint {
        row: luaL_checkinteger(L, 9 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
        column: luaL_checkinteger(L, 10 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
    };
    let mut edit: TSInputEdit = TSInputEdit {
        start_byte: start_byte,
        old_end_byte: old_end_byte,
        new_end_byte: new_end_byte,
        start_point: start_point,
        old_end_point: old_end_point,
        new_end_point: new_end_point,
    };
    let mut new_tree: *mut TSTree = ts_tree_copy((*ud).tree);
    ts_tree_edit(new_tree, &raw mut edit);
    push_tree(L, new_tree);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn tree_get_ranges(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut ud: *mut TSLuaTree =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    let mut include_bytes: bool =
        lua_gettop(L) >= 2 as ::core::ffi::c_int && lua_toboolean(L, 2 as ::core::ffi::c_int) != 0;
    let mut len: uint32_t = 0;
    let mut ranges: *mut TSRange = ts_tree_included_ranges((*ud).tree, &raw mut len);
    push_ranges(L, ranges, len as size_t, include_bytes);
    xfree(ranges as *mut ::core::ffi::c_void);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn tree_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut ud: *mut TSLuaTree =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    let mut tree: *mut TSTree = (*ud).tree as *mut TSTree;
    ts_tree_delete(tree);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn tree_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushstring(L, b"<tree>\0".as_ptr() as *const ::core::ffi::c_char);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn tree_root(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut ud: *mut TSLuaTree =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    let mut root: TSNode = ts_tree_root_node((*ud).tree);
    let mut node_ud: *mut TSNode =
        lua_newuserdata(L, ::core::mem::size_of::<TSNode>()) as *mut TSNode;
    *node_ud = root;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_NODE.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_createtable(L, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    lua_pushvalue(L, 1 as ::core::ffi::c_int);
    lua_rawseti(L, -2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    lua_setfenv(L, -2 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
static node_meta: GlobalCell<[luaL_Reg; 36]> = GlobalCell::new([
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_tostring as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"__eq\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_eq as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__len\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_child_count as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"id\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_id as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"range\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_range as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"start\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_start as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"end_\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_end as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"type\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_type as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"symbol\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_symbol as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"field\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_field as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"named\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_named as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"missing\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_missing as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"extra\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_extra as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"has_changes\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_has_changes as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"has_error\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_has_error as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"sexpr\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_sexpr as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"child_count\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_child_count as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"named_child_count\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_named_child_count
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"child\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_child as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"named_child\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_named_child as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"descendant_for_range\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_descendant_for_range
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"named_descendant_for_range\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_named_descendant_for_range
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"parent\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_parent as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"__has_ancestor\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            __has_ancestor as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"child_with_descendant\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_child_with_descendant
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"iter_children\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_iter_children as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"next_sibling\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_next_sibling as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"prev_sibling\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_prev_sibling as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"next_named_sibling\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_next_named_sibling
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"prev_named_sibling\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_prev_named_sibling
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"named_children\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_named_children
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"root\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_root as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"tree\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_tree as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"byte_length\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            node_byte_length as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"equal\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(node_equal as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
unsafe extern "C-unwind" fn push_node(
    mut L: *mut lua_State,
    mut node: TSNode,
    mut uindex: ::core::ffi::c_int,
) {
    '_c2rust_label: {
        if uindex > 0 as ::core::ffi::c_int || uindex < -20 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"uindex > 0 || uindex < -LUA_MINSTACK\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/treesitter.rs\0".as_ptr() as *const ::core::ffi::c_char,
                941 as ::core::ffi::c_uint,
                b"void push_node(lua_State *, TSNode, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if ts_node_is_null(node) {
        lua_pushnil(L);
        return;
    }
    let mut ud: *mut TSNode = lua_newuserdata(L, ::core::mem::size_of::<TSNode>()) as *mut TSNode;
    *ud = node;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_NODE.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_getfenv(L, uindex);
    lua_setfenv(L, -2 as ::core::ffi::c_int);
}
unsafe extern "C-unwind" fn node_check_opt(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
    mut res: *mut TSNode,
) -> bool {
    let mut ud: *mut TSNode = luaL_checkudata(L, index, TS_META_NODE.as_ptr()) as *mut TSNode;
    if !ud.is_null() {
        *res = *ud;
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C-unwind" fn node_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> TSNode {
    let mut ud: *mut TSNode = luaL_checkudata(L, index, TS_META_NODE.as_ptr()) as *mut TSNode;
    return *ud;
}
unsafe extern "C-unwind" fn node_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushstring(L, b"<node \0".as_ptr() as *const ::core::ffi::c_char);
    lua_pushstring(L, ts_node_type(node));
    lua_pushstring(L, b">\0".as_ptr() as *const ::core::ffi::c_char);
    lua_concat(L, 3 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_eq(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut node2: TSNode = node_check(L, 2 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_eq(node, node2) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_id(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushlstring(
        L,
        &raw mut node.id as *const ::core::ffi::c_char,
        ::core::mem::size_of::<*const ::core::ffi::c_void>(),
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_range(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut include_bytes: bool =
        lua_gettop(L) >= 2 as ::core::ffi::c_int && lua_toboolean(L, 2 as ::core::ffi::c_int) != 0;
    let mut start: TSPoint = ts_node_start_point(node);
    let mut end: TSPoint = ts_node_end_point(node);
    if include_bytes {
        lua_pushinteger(L, start.row as lua_Integer);
        lua_pushinteger(L, start.column as lua_Integer);
        lua_pushinteger(L, ts_node_start_byte(node) as lua_Integer);
        lua_pushinteger(L, end.row as lua_Integer);
        lua_pushinteger(L, end.column as lua_Integer);
        lua_pushinteger(L, ts_node_end_byte(node) as lua_Integer);
        return 6 as ::core::ffi::c_int;
    }
    lua_pushinteger(L, start.row as lua_Integer);
    lua_pushinteger(L, start.column as lua_Integer);
    lua_pushinteger(L, end.row as lua_Integer);
    lua_pushinteger(L, end.column as lua_Integer);
    return 4 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_start(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut start: TSPoint = ts_node_start_point(node);
    let mut start_byte: uint32_t = ts_node_start_byte(node);
    lua_pushinteger(L, start.row as lua_Integer);
    lua_pushinteger(L, start.column as lua_Integer);
    lua_pushinteger(L, start_byte as lua_Integer);
    return 3 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_end(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut end: TSPoint = ts_node_end_point(node);
    let mut end_byte: uint32_t = ts_node_end_byte(node);
    lua_pushinteger(L, end.row as lua_Integer);
    lua_pushinteger(L, end.column as lua_Integer);
    lua_pushinteger(L, end_byte as lua_Integer);
    return 3 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_child_count(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut count: uint32_t = ts_node_child_count(node);
    lua_pushinteger(L, count as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_named_child_count(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut count: uint32_t = ts_node_named_child_count(node);
    lua_pushinteger(L, count as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_type(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushstring(L, ts_node_type(node));
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_symbol(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut symbol: TSSymbol = ts_node_symbol(node);
    lua_pushinteger(L, symbol as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_field(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut count: uint32_t = ts_node_child_count(node);
    let mut curr_index: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut name_len: size_t = 0;
    let mut field_name: *const ::core::ffi::c_char =
        luaL_checklstring(L, 2 as ::core::ffi::c_int, &raw mut name_len);
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut i: uint32_t = 0 as uint32_t;
    while i < count {
        let mut child_field_name: *const ::core::ffi::c_char =
            ts_node_field_name_for_child(node, i);
        if strequal(field_name, child_field_name) {
            let mut child: TSNode = ts_node_child(node, i);
            push_node(L, child, 1 as ::core::ffi::c_int);
            curr_index += 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, curr_index);
        }
        i = i.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_named(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_is_named(node) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_sexpr(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut allocated: *mut ::core::ffi::c_char = ts_node_string(node);
    lua_pushstring(L, allocated);
    xfree(allocated as *mut ::core::ffi::c_void);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_missing(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_is_missing(node) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_extra(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_is_extra(node) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_has_changes(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_has_changes(node) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_has_error(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_has_error(node) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_child(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut num: uint32_t = lua_tointeger(L, 2 as ::core::ffi::c_int) as uint32_t;
    let mut child: TSNode = ts_node_child(node, num);
    push_node(L, child, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_named_child(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut num: uint32_t = lua_tointeger(L, 2 as ::core::ffi::c_int) as uint32_t;
    let mut child: TSNode = ts_node_named_child(node, num);
    push_node(L, child, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_descendant_for_range(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut start: TSPoint = TSPoint {
        row: lua_tointeger(L, 2 as ::core::ffi::c_int) as uint32_t,
        column: lua_tointeger(L, 3 as ::core::ffi::c_int) as uint32_t,
    };
    let mut end: TSPoint = TSPoint {
        row: lua_tointeger(L, 4 as ::core::ffi::c_int) as uint32_t,
        column: lua_tointeger(L, 5 as ::core::ffi::c_int) as uint32_t,
    };
    let mut child: TSNode = ts_node_descendant_for_point_range(node, start, end);
    push_node(L, child, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_named_descendant_for_range(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut start: TSPoint = TSPoint {
        row: lua_tointeger(L, 2 as ::core::ffi::c_int) as uint32_t,
        column: lua_tointeger(L, 3 as ::core::ffi::c_int) as uint32_t,
    };
    let mut end: TSPoint = TSPoint {
        row: lua_tointeger(L, 4 as ::core::ffi::c_int) as uint32_t,
        column: lua_tointeger(L, 5 as ::core::ffi::c_int) as uint32_t,
    };
    let mut child: TSNode = ts_node_named_descendant_for_point_range(node, start, end);
    push_node(L, child, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_next_child(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut child_index: *mut uint32_t =
        lua_touserdata(L, LUA_GLOBALSINDEX - 1 as ::core::ffi::c_int) as *mut uint32_t;
    let mut source: TSNode = node_check(L, LUA_GLOBALSINDEX - 2 as ::core::ffi::c_int);
    if *child_index >= ts_node_child_count(source) {
        return 0 as ::core::ffi::c_int;
    }
    let mut child: TSNode = ts_node_child(source, *child_index);
    push_node(L, child, LUA_GLOBALSINDEX - 2 as ::core::ffi::c_int);
    let mut field: *const ::core::ffi::c_char = ts_node_field_name_for_child(source, *child_index);
    if !field.is_null() {
        lua_pushstring(L, field);
    } else {
        lua_pushnil(L);
    }
    *child_index = (*child_index).wrapping_add(1);
    return 2 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_iter_children(mut L: *mut lua_State) -> ::core::ffi::c_int {
    node_check(L, 1 as ::core::ffi::c_int);
    let mut child_index: *mut uint32_t =
        lua_newuserdata(L, ::core::mem::size_of::<uint32_t>()) as *mut uint32_t;
    *child_index = 0 as uint32_t;
    lua_pushvalue(L, 1 as ::core::ffi::c_int);
    lua_pushcclosure(
        L,
        Some(node_next_child as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        2 as ::core::ffi::c_int,
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_parent(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut parent: TSNode = ts_node_parent(node);
    push_node(L, parent, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn __has_ancestor(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut descendant: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    if lua_type(L, 2 as ::core::ffi::c_int) != LUA_TTABLE {
        lua_pushboolean(L, false_0);
        return 1 as ::core::ffi::c_int;
    }
    let pred_len: ::core::ffi::c_int = lua_objlen(L, 2 as ::core::ffi::c_int) as ::core::ffi::c_int;
    let mut node: TSNode = ts_tree_root_node(descendant.tree);
    while node.id != descendant.id && !ts_node_is_null(node) {
        let mut node_type_0: *const ::core::ffi::c_char = ts_node_type(node);
        let mut node_type_len: size_t = strlen(node_type_0);
        let mut i: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
        while i <= pred_len {
            lua_rawgeti(L, 2 as ::core::ffi::c_int, i);
            if lua_type(L, -1 as ::core::ffi::c_int) == LUA_TSTRING {
                let mut check_len: size_t = 0;
                let mut check_str: *const ::core::ffi::c_char =
                    lua_tolstring(L, -1 as ::core::ffi::c_int, &raw mut check_len);
                if node_type_len == check_len
                    && memcmp(
                        node_type_0 as *const ::core::ffi::c_void,
                        check_str as *const ::core::ffi::c_void,
                        check_len,
                    ) == 0 as ::core::ffi::c_int
                {
                    lua_pushboolean(L, true_0);
                    return 1 as ::core::ffi::c_int;
                }
            }
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            i += 1;
        }
        node = ts_node_child_with_descendant(node, descendant);
    }
    lua_pushboolean(L, false_0);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_child_with_descendant(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut descendant: TSNode = node_check(L, 2 as ::core::ffi::c_int);
    let mut child: TSNode = ts_node_child_with_descendant(node, descendant);
    push_node(L, child, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_next_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut sibling: TSNode = ts_node_next_sibling(node);
    push_node(L, sibling, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_prev_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut sibling: TSNode = ts_node_prev_sibling(node);
    push_node(L, sibling, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_next_named_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut sibling: TSNode = ts_node_next_named_sibling(node);
    push_node(L, sibling, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_prev_named_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut sibling: TSNode = ts_node_prev_named_sibling(node);
    push_node(L, sibling, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_named_children(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut source: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut curr_index: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n: uint32_t = ts_node_child_count(source);
    let mut i: uint32_t = 0 as uint32_t;
    while i < n {
        let mut child: TSNode = ts_node_child(source, i);
        if ts_node_is_named(child) {
            push_node(L, child, 1 as ::core::ffi::c_int);
            curr_index += 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, curr_index);
        }
        i = i.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_root(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut root: TSNode = ts_tree_root_node(node.tree);
    push_node(L, root, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_tree(mut L: *mut lua_State) -> ::core::ffi::c_int {
    node_check(L, 1 as ::core::ffi::c_int);
    lua_getfenv(L, 1 as ::core::ffi::c_int);
    lua_rawgeti(L, 2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_byte_length(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut start_byte: uint32_t = ts_node_start_byte(node);
    let mut end_byte: uint32_t = ts_node_end_byte(node);
    lua_pushinteger(L, end_byte.wrapping_sub(start_byte) as lua_Integer);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn node_equal(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node1: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut node2: TSNode = node_check(L, 2 as ::core::ffi::c_int);
    lua_pushboolean(L, ts_node_eq(node1, node2) as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
static querycursor_meta: GlobalCell<[luaL_Reg; 5]> = GlobalCell::new([
    luaL_Reg {
        name: b"remove_match\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            querycursor_remove_match
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"next_capture\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            querycursor_next_capture
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"next_match\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            querycursor_next_match
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            querycursor_gc as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
unsafe extern "C-unwind" fn tslua_push_querycursor(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
    let mut query: *mut TSQuery = query_check(L, 2 as ::core::ffi::c_int);
    let mut cursor: *mut TSQueryCursor = ts_query_cursor_new();
    if lua_gettop(L) >= 3 as ::core::ffi::c_int
        && !(lua_type(L, 3 as ::core::ffi::c_int) == LUA_TNIL)
    {
        (lua_type(L, 3 as ::core::ffi::c_int) == 5 as ::core::ffi::c_int
            || luaL_argerror(
                L,
                3 as ::core::ffi::c_int,
                b"table expected\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0) as ::core::ffi::c_int;
    }
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"start_row\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut start_row: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"start_col\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut start_col: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"end_row\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut end_row: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"end_col\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut end_col: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    ts_query_cursor_set_point_range(
        cursor,
        TSPoint {
            row: start_row,
            column: start_col,
        },
        TSPoint {
            row: end_row,
            column: end_col,
        },
    );
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"max_start_depth\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL) {
        let mut max_start_depth: uint32_t =
            luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
        ts_query_cursor_set_max_start_depth(cursor, max_start_depth);
    }
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_getfield(
        L,
        3 as ::core::ffi::c_int,
        b"match_limit\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL) {
        let mut match_limit: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
        ts_query_cursor_set_match_limit(cursor, match_limit);
    }
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    ts_query_cursor_exec(cursor, query, node);
    let mut ud: *mut *mut TSQueryCursor =
        lua_newuserdata(L, ::core::mem::size_of::<*mut TSQueryCursor>()) as *mut *mut TSQueryCursor;
    *ud = cursor;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERYCURSOR.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_getfenv(L, 1 as ::core::ffi::c_int);
    lua_setfenv(L, -2 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn querycursor_remove_match(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut cursor: *mut TSQueryCursor = querycursor_check(L, 1 as ::core::ffi::c_int);
    let mut match_id: uint32_t = luaL_checkinteger(L, 2 as ::core::ffi::c_int) as uint32_t;
    ts_query_cursor_remove_match(cursor, match_id);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn querycursor_next_capture(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut cursor: *mut TSQueryCursor = querycursor_check(L, 1 as ::core::ffi::c_int);
    let mut match_0: TSQueryMatch = TSQueryMatch {
        id: 0,
        pattern_index: 0,
        capture_count: 0,
        captures: ::core::ptr::null::<TSQueryCapture>(),
    };
    let mut capture_index: uint32_t = 0;
    if !ts_query_cursor_next_capture(cursor, &raw mut match_0, &raw mut capture_index) {
        return 0 as ::core::ffi::c_int;
    }
    let mut capture: TSQueryCapture = *match_0.captures.offset(capture_index as isize);
    lua_pushinteger(L, capture.index.wrapping_add(1 as uint32_t) as lua_Integer);
    push_node(L, capture.node, 1 as ::core::ffi::c_int);
    push_querymatch(L, &raw mut match_0, 1 as ::core::ffi::c_int);
    return 3 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn querycursor_next_match(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut cursor: *mut TSQueryCursor = querycursor_check(L, 1 as ::core::ffi::c_int);
    let mut match_0: TSQueryMatch = TSQueryMatch {
        id: 0,
        pattern_index: 0,
        capture_count: 0,
        captures: ::core::ptr::null::<TSQueryCapture>(),
    };
    if !ts_query_cursor_next_match(cursor, &raw mut match_0) {
        return 0 as ::core::ffi::c_int;
    }
    push_querymatch(L, &raw mut match_0, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn querycursor_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSQueryCursor {
    let mut ud: *mut *mut TSQueryCursor =
        luaL_checkudata(L, index, TS_META_QUERYCURSOR.as_ptr()) as *mut *mut TSQueryCursor;
    (!(*ud).is_null()
        || luaL_argerror(
            L,
            index,
            b"TSQueryCursor expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    return *ud;
}
unsafe extern "C-unwind" fn querycursor_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut cursor: *mut TSQueryCursor = querycursor_check(L, 1 as ::core::ffi::c_int);
    ts_query_cursor_delete(cursor);
    return 0 as ::core::ffi::c_int;
}
static querymatch_meta: GlobalCell<[luaL_Reg; 3]> = GlobalCell::new([
    luaL_Reg {
        name: b"info\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            querymatch_info as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"captures\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            querymatch_captures
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
unsafe extern "C-unwind" fn push_querymatch(
    mut L: *mut lua_State,
    mut match_0: *mut TSQueryMatch,
    mut uindex: ::core::ffi::c_int,
) {
    let mut ud: *mut TSQueryMatch =
        lua_newuserdata(L, ::core::mem::size_of::<TSQueryMatch>()) as *mut TSQueryMatch;
    *ud = *match_0;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERYMATCH.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_getfenv(L, uindex);
    lua_setfenv(L, -2 as ::core::ffi::c_int);
}
unsafe extern "C-unwind" fn querymatch_info(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut match_0: *mut TSQueryMatch =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_QUERYMATCH.as_ptr())
            as *mut TSQueryMatch;
    lua_pushinteger(L, (*match_0).id as lua_Integer);
    lua_pushinteger(
        L,
        ((*match_0).pattern_index as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as lua_Integer,
    );
    return 2 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn querymatch_captures(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut match_0: *mut TSQueryMatch =
        luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_QUERYMATCH.as_ptr())
            as *mut TSQueryMatch;
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut i: size_t = 0 as size_t;
    while i < (*match_0).capture_count as size_t {
        let mut capture: TSQueryCapture = *(*match_0).captures.offset(i as isize);
        let mut index: ::core::ffi::c_int =
            capture.index as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        lua_rawgeti(L, -1 as ::core::ffi::c_int, index);
        if lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL {
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        }
        push_node(L, capture.node, 1 as ::core::ffi::c_int);
        lua_rawseti(
            L,
            -2 as ::core::ffi::c_int,
            lua_objlen(L, -2 as ::core::ffi::c_int) as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        );
        lua_rawseti(L, -2 as ::core::ffi::c_int, index);
        i = i.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
static query_meta: GlobalCell<[luaL_Reg; 6]> = GlobalCell::new([
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(query_gc as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            query_tostring as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"inspect\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            query_inspect as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"disable_capture\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            query_disable_capture
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"disable_pattern\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            query_disable_pattern
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
unsafe extern "C-unwind" fn tslua_parse_query(mut L: *mut lua_State) -> ::core::ffi::c_int {
    if lua_gettop(L) < 2 as ::core::ffi::c_int
        || lua_isstring(L, 1 as ::core::ffi::c_int) == 0
        || lua_isstring(L, 2 as ::core::ffi::c_int) == 0
    {
        return luaL_error(
            L,
            b"string expected\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut lang: *mut TSLanguage = lang_check(L, 1 as ::core::ffi::c_int);
    let mut len: size_t = 0;
    let mut src: *const ::core::ffi::c_char =
        lua_tolstring(L, 2 as ::core::ffi::c_int, &raw mut len);
    tslua_query_parse_count.set((*tslua_query_parse_count.ptr()).wrapping_add(1));
    let mut error_offset: uint32_t = 0;
    let mut error_type: TSQueryError = TSQueryErrorNone;
    let mut query: *mut TSQuery = ts_query_new(
        lang,
        src,
        len as uint32_t,
        &raw mut error_offset,
        &raw mut error_type,
    );
    if query.is_null() {
        let mut err_msg: [::core::ffi::c_char; 1025] = [0; 1025];
        query_err_string(
            src,
            error_offset as ::core::ffi::c_int,
            error_type,
            &raw mut err_msg as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        );
        return luaL_error(
            L,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut err_msg as *mut ::core::ffi::c_char,
        );
    }
    let mut ud: *mut *mut TSQuery =
        lua_newuserdata(L, ::core::mem::size_of::<*mut TSQuery>()) as *mut *mut TSQuery;
    *ud = query;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERY.as_ptr());
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn query_err_to_string(
    mut error_type: TSQueryError,
) -> *const ::core::ffi::c_char {
    match error_type as ::core::ffi::c_uint {
        1 => return b"Invalid syntax:\n\0".as_ptr() as *const ::core::ffi::c_char,
        2 => return b"Invalid node type \0".as_ptr() as *const ::core::ffi::c_char,
        3 => return b"Invalid field name \0".as_ptr() as *const ::core::ffi::c_char,
        4 => return b"Invalid capture name \0".as_ptr() as *const ::core::ffi::c_char,
        5 => return b"Impossible pattern:\n\0".as_ptr() as *const ::core::ffi::c_char,
        _ => return b"error\0".as_ptr() as *const ::core::ffi::c_char,
    };
}
unsafe extern "C-unwind" fn query_err_string(
    mut src: *const ::core::ffi::c_char,
    mut error_offset: ::core::ffi::c_int,
    mut error_type: TSQueryError,
    mut err: *mut ::core::ffi::c_char,
    mut errlen: size_t,
) {
    let mut line_start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut error_line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut error_line_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut end_str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    loop {
        let mut src_tmp: *const ::core::ffi::c_char = src.offset(line_start as isize);
        end_str = strchr(src_tmp, '\n' as ::core::ffi::c_int);
        let mut line_length: ::core::ffi::c_int = if !end_str.is_null() {
            end_str.offset_from(src_tmp) as ::core::ffi::c_int
        } else {
            strlen(src_tmp) as ::core::ffi::c_int
        };
        let mut line_end: ::core::ffi::c_int = line_start + line_length;
        if line_end > error_offset {
            error_line = src_tmp;
            error_line_len = line_length;
            break;
        } else {
            line_start = line_end + 1 as ::core::ffi::c_int;
            row += 1;
            if end_str.is_null() {
                break;
            }
        }
    }
    let mut column: ::core::ffi::c_int = error_offset - line_start;
    let mut type_msg: *const ::core::ffi::c_char = query_err_to_string(error_type);
    snprintf(
        err,
        errlen,
        b"Query error at %d:%d. %s\0".as_ptr() as *const ::core::ffi::c_char,
        row + 1 as ::core::ffi::c_int,
        column + 1 as ::core::ffi::c_int,
        type_msg,
    );
    let mut offset: size_t = strlen(err);
    errlen = errlen.wrapping_sub(offset);
    err = err.offset(offset as isize);
    if error_type as ::core::ffi::c_uint
        == TSQueryErrorNodeType as ::core::ffi::c_int as ::core::ffi::c_uint
        || error_type as ::core::ffi::c_uint
            == TSQueryErrorField as ::core::ffi::c_int as ::core::ffi::c_uint
        || error_type as ::core::ffi::c_uint
            == TSQueryErrorCapture as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut suffix: *const ::core::ffi::c_char = src.offset(error_offset as isize);
        let mut is_anonymous: bool = error_type as ::core::ffi::c_uint
            == TSQueryErrorNodeType as ::core::ffi::c_int as ::core::ffi::c_uint
            && *suffix.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '"' as ::core::ffi::c_int;
        let mut suffix_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut c: ::core::ffi::c_char = *suffix.offset(suffix_len as isize);
        if is_anonymous {
            let mut backslashes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while c as ::core::ffi::c_int != '"' as ::core::ffi::c_int
                || backslashes % 2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            {
                if c as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                    backslashes += 1 as ::core::ffi::c_int;
                } else {
                    backslashes = 0 as ::core::ffi::c_int;
                }
                suffix_len += 1;
                c = *suffix.offset(suffix_len as isize);
            }
        } else {
            while *(*__ctype_b_loc()).offset(c as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
                || c as ::core::ffi::c_int == '_' as ::core::ffi::c_int
                || c as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                || c as ::core::ffi::c_int == '.' as ::core::ffi::c_int
            {
                suffix_len += 1;
                c = *suffix.offset(suffix_len as isize);
            }
        }
        snprintf(
            err,
            errlen,
            b"\"%.*s\":\n\0".as_ptr() as *const ::core::ffi::c_char,
            suffix_len,
            suffix,
        );
        offset = strlen(err);
        errlen = errlen.wrapping_sub(offset);
        err = err.offset(offset as isize);
    }
    if error_line.is_null() {
        snprintf(
            err,
            errlen,
            b"Unexpected EOF\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    snprintf(
        err,
        errlen,
        b"%.*s\n%*s^\n\0".as_ptr() as *const ::core::ffi::c_char,
        error_line_len,
        error_line,
        column,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
unsafe extern "C-unwind" fn query_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSQuery {
    let mut ud: *mut *mut TSQuery =
        luaL_checkudata(L, index, TS_META_QUERY.as_ptr()) as *mut *mut TSQuery;
    (!(*ud).is_null()
        || luaL_argerror(
            L,
            index,
            b"TSQuery expected\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    return *ud;
}
unsafe extern "C-unwind" fn query_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
    ts_query_delete(query);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn query_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushstring(L, b"<query>\0".as_ptr() as *const ::core::ffi::c_char);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn query_inspect(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
    lua_createtable(L, 0 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
    let mut n_pat: uint32_t = ts_query_pattern_count(query);
    lua_createtable(L, n_pat as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    let mut i: size_t = 0 as size_t;
    while i < n_pat as size_t {
        let mut len: uint32_t = 0;
        let mut step: *const TSQueryPredicateStep =
            ts_query_predicates_for_pattern(query, i as uint32_t, &raw mut len);
        if len != 0 as uint32_t {
            lua_createtable(
                L,
                len as ::core::ffi::c_int / 4 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            lua_createtable(L, 3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            let mut nextpred: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            let mut nextitem: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            let mut k: size_t = 0 as size_t;
            while k < len as size_t {
                if (*step.offset(k as isize)).type_0 as ::core::ffi::c_uint
                    == TSQueryPredicateStepTypeDone as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let c2rust_fresh0 = nextpred;
                    nextpred = nextpred + 1;
                    lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh0);
                    lua_createtable(L, 3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                    nextitem = 1 as ::core::ffi::c_int;
                } else {
                    if (*step.offset(k as isize)).type_0 as ::core::ffi::c_uint
                        == TSQueryPredicateStepTypeString as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    {
                        let mut strlen_0: uint32_t = 0;
                        let mut str: *const ::core::ffi::c_char = ts_query_string_value_for_id(
                            query,
                            (*step.offset(k as isize)).value_id,
                            &raw mut strlen_0,
                        );
                        lua_pushlstring(L, str, strlen_0 as size_t);
                    } else if (*step.offset(k as isize)).type_0 as ::core::ffi::c_uint
                        == TSQueryPredicateStepTypeCapture as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    {
                        lua_pushinteger(
                            L,
                            (*step.offset(k as isize))
                                .value_id
                                .wrapping_add(1 as uint32_t)
                                as lua_Integer,
                        );
                    } else {
                        abort();
                    }
                    let c2rust_fresh1 = nextitem;
                    nextitem = nextitem + 1;
                    lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh1);
                }
                k = k.wrapping_add(1);
            }
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            lua_rawseti(
                L,
                -2 as ::core::ffi::c_int,
                i as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            );
        }
        i = i.wrapping_add(1);
    }
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"patterns\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut n_captures: uint32_t = ts_query_capture_count(query);
    lua_createtable(L, n_captures as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    let mut i_0: size_t = 0 as size_t;
    while i_0 < n_captures as size_t {
        let mut strlen_1: uint32_t = 0;
        let mut str_0: *const ::core::ffi::c_char =
            ts_query_capture_name_for_id(query, i_0 as uint32_t, &raw mut strlen_1);
        lua_pushlstring(L, str_0, strlen_1 as size_t);
        lua_rawseti(
            L,
            -2 as ::core::ffi::c_int,
            i_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        );
        i_0 = i_0.wrapping_add(1);
    }
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"captures\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn query_disable_capture(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
    let mut name_len: size_t = 0;
    let mut name: *const ::core::ffi::c_char =
        luaL_checklstring(L, 2 as ::core::ffi::c_int, &raw mut name_len);
    ts_query_disable_capture(query, name, name_len as uint32_t);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn query_disable_pattern(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
    let pattern_index: uint32_t = luaL_checkinteger(L, 2 as ::core::ffi::c_int) as uint32_t;
    ts_query_disable_pattern(query, pattern_index.wrapping_sub(1 as uint32_t));
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn build_meta(
    mut L: *mut lua_State,
    mut tname: *const ::core::ffi::c_char,
    mut meta: *const luaL_Reg,
) {
    if luaL_newmetatable(L, tname) != 0 {
        luaL_register(L, ::core::ptr::null::<::core::ffi::c_char>(), meta);
        lua_pushvalue(L, -1 as ::core::ffi::c_int);
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"__index\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
}
unsafe extern "C-unwind" fn tslua_init(mut L: *mut lua_State) {
    build_meta(
        L,
        TS_META_PARSER.as_ptr(),
        parser_meta.ptr() as *mut luaL_Reg,
    );
    build_meta(L, TS_META_TREE.as_ptr(), tree_meta.ptr() as *mut luaL_Reg);
    build_meta(L, TS_META_NODE.as_ptr(), node_meta.ptr() as *mut luaL_Reg);
    build_meta(L, TS_META_QUERY.as_ptr(), query_meta.ptr() as *mut luaL_Reg);
    build_meta(
        L,
        TS_META_QUERYCURSOR.as_ptr(),
        querycursor_meta.ptr() as *mut luaL_Reg,
    );
    build_meta(
        L,
        TS_META_QUERYMATCH.as_ptr(),
        querymatch_meta.ptr() as *mut luaL_Reg,
    );
    ts_set_allocator(
        Some(xmalloc as unsafe extern "C" fn(size_t) -> *mut ::core::ffi::c_void),
        Some(xcalloc as unsafe extern "C" fn(size_t, size_t) -> *mut ::core::ffi::c_void),
        Some(
            xrealloc
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    size_t,
                ) -> *mut ::core::ffi::c_void,
        ),
        Some(xfree as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    );
}
unsafe extern "C-unwind" fn tslua_get_language_version(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    lua_pushnumber(L, TREE_SITTER_LANGUAGE_VERSION as lua_Number);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn tslua_get_minimum_language_version(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    lua_pushnumber(L, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION as lua_Number);
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C-unwind" fn nlua_treesitter_free() {}
pub unsafe extern "C-unwind" fn nlua_treesitter_init(lstate: *mut lua_State) {
    tslua_init(lstate);
    lua_pushcclosure(
        lstate,
        Some(
            tslua_push_parser as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_create_ts_parser\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_push_querycursor
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_create_ts_querycursor\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_add_language_from_object
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_add_language_from_object\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_has_language as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_has_language\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_remove_lang as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_remove_language\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_inspect_lang as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_inspect_language\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_parse_query as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_parse_query\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_get_language_version
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_get_language_version\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            tslua_get_minimum_language_version
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_ts_get_minimum_language_version\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
