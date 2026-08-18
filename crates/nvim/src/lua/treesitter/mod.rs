#![deny(unsafe_op_in_unsafe_fn)]

use crate::event::libuv::{uv_dlclose, uv_dlerror, uv_dlopen, uv_dlsym};
use crate::global_cell::GlobalCell;
use crate::lua::ffi::{
    lua_concat, lua_createtable, lua_error, lua_getfenv, lua_getfield, lua_gettop, lua_isstring,
    lua_newuserdata, lua_objlen, lua_pcall, lua_pushboolean, lua_pushcclosure, lua_pushinteger,
    lua_pushlstring, lua_pushnil, lua_pushnumber, lua_pushstring, lua_pushvalue, lua_rawgeti,
    lua_rawseti, lua_setfenv, lua_setfield, lua_setmetatable, lua_settop, lua_toboolean,
    lua_tointeger, lua_tolstring, lua_touserdata, lua_type, luaL_argcheck, luaL_argerror,
    luaL_checkinteger, luaL_checklstring, luaL_checknumber, luaL_checkudata, luaL_error,
    luaL_newmetatable, luaL_ref, luaL_register, luaL_unref,
};
use crate::main::{IObuff, buffer_handles, tslua_query_parse_count};
use crate::map::{map_del_cstr_t_ptr_t, map_put_ref_cstr_t_ptr_t, mh_get_cstr_t, mh_get_int};
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::memory::{memchrsub, strequal, xcalloc, xfree, xmalloc, xrealloc, xstrdup, xstrlcpy};
use crate::os::cshim::{__ctype_b_loc, snprintf, strchr};
use crate::os::time::os_hrtime;
use crate::strings::vim_snprintf;
use crate::types::{
    LuaRef, Map_cstr_t_ptr_t, Map_int_ptr_t, MapHash, Set_cstr_t, buf_T, cstr_t, handle_T, int32_t,
    linenr_T, lua_Integer, lua_Number, lua_State, luaL_Reg, ptr_t, size_t, uint8_t, uint16_t,
    uint32_t, uint64_t, uv_lib_t,
};
use ::libc::{abort, memcmp, memcpy, strlen};

// The carve of the transpiled module; see each child's docs.
mod cursor;
mod init;
mod lang;
mod logger;
mod node;
mod parser;
mod query;
mod ranges;
mod tree;

pub(crate) use self::cursor::*;
pub use self::init::*;
pub(crate) use self::lang::*;
pub(crate) use self::logger::*;
pub(crate) use self::node::*;
pub use self::parser::*;
pub(crate) use self::query::*;
pub(crate) use self::ranges::*;
pub(crate) use self::tree::*;
// Opaque C types: layout unknown here, only ever used behind a pointer.
#[repr(C)]
pub struct TSLanguage {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}
#[repr(C)]
pub struct TSParser {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}
#[repr(C)]
pub struct TSTree {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}
#[repr(C)]
pub struct TSQuery {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}
#[repr(C)]
pub struct TSQueryCursor {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}
unsafe extern "C" {
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
pub type TSSymbol = uint16_t;
pub type TSFieldId = uint16_t;
pub type TSDecodeFunction =
    Option<unsafe extern "C" fn(*const uint8_t, uint32_t, *mut int32_t) -> uint32_t>;
pub type TSInputEncoding = ::core::ffi::c_uint;
pub const TSInputEncodingUTF8: TSInputEncoding = 0;
pub type TSSymbolType = ::core::ffi::c_uint;
pub const TSSymbolTypeAuxiliary: TSSymbolType = 3;
pub const TSSymbolTypeAnonymous: TSSymbolType = 1;
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
pub const TSQueryErrorCapture: TSQueryError = 4;
pub const TSQueryErrorField: TSQueryError = 3;
pub const TSQueryErrorNodeType: TSQueryError = 2;
pub const TSQueryErrorNone: TSQueryError = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TSLanguageMetadata {
    pub major_version: uint8_t,
    pub minor_version: uint8_t,
    pub patch_version: uint8_t,
}
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
unsafe fn set_has_cstr_t(mut set: *mut Set_cstr_t, mut key: cstr_t) -> bool {
    unsafe {
        return mh_get_cstr_t(set, key) != MH_TOMBSTONE as uint32_t;
    }
}
#[inline]
unsafe fn map_get_int_ptr_t(mut map: *mut Map_int_ptr_t, mut key: ::core::ffi::c_int) -> ptr_t {
    unsafe {
        let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
        return if k == MH_TOMBSTONE as uint32_t {
            value_init_ptr_t.get()
        } else {
            *(*map).values.offset(k as isize)
        };
    }
}
#[inline]
unsafe fn map_put_cstr_t_ptr_t(mut map: *mut Map_cstr_t_ptr_t, mut key: cstr_t, mut value: ptr_t) {
    unsafe {
        let mut val: *mut ptr_t = map_put_ref_cstr_t_ptr_t(
            map,
            key,
            ::core::ptr::null_mut::<*mut cstr_t>(),
            ::core::ptr::null_mut::<bool>(),
        );
        *val = value;
    }
}
#[inline]
unsafe fn map_get_cstr_t_ptr_t(mut map: *mut Map_cstr_t_ptr_t, mut key: cstr_t) -> ptr_t {
    unsafe {
        let mut k: uint32_t = mh_get_cstr_t(&raw mut (*map).set, key);
        return if k == MH_TOMBSTONE as uint32_t {
            value_init_ptr_t.get()
        } else {
            *(*map).values.offset(k as isize)
        };
    }
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TS_META_PARSER: &::core::ffi::CStr = c"treesitter_parser";
pub const TS_META_TREE: &::core::ffi::CStr = c"treesitter_tree";
pub const TS_META_NODE: &::core::ffi::CStr = c"treesitter_node";
pub const TS_META_QUERY: &::core::ffi::CStr = c"treesitter_query";
pub const TS_META_QUERYCURSOR: &::core::ffi::CStr = c"treesitter_querycursor";
pub const TS_META_QUERYMATCH: &::core::ffi::CStr = c"treesitter_querymatch";
static langs: GlobalCell<Map_cstr_t_ptr_t> = GlobalCell::new(MAP_INIT);
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
