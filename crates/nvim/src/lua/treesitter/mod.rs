//! The tree-sitter binding: `vim._ts_*` and the parser, tree, node, query
//! and cursor userdata.
//!
//! The tree-sitter objects are C objects with C lifetimes, held by the
//! userdata ([`Parser`], [`Tree`], [`Query`], [`QueryCursor`]) whose `__gc`
//! deletes them; a node or a match is a plain value whose userdatum keeps
//! the tree it points into alive through its environment table. Every Lua
//! stack operation goes through the [`Stack`]; the unsafe code left here is
//! the calls into libtree-sitter and the reads of what it hands back.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/treesitter/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_uint, c_void};

use crate::global_cell::GlobalCell;
use crate::lua::stack::{At, Function, Stack, Userdata};
use crate::registry::{IdMap, id_map};
use crate::types::{int32_t, lua_Integer, size_t, uint8_t, uint16_t, uint32_t};

mod cursor;
mod init;
mod lang;
mod logger;
mod node;
mod parser;
mod query;
mod ranges;
mod tree;

pub use self::init::nlua_treesitter_init;
use self::node::Node;
use self::query::Query;

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
// The parse entry points run the parser's logger, and the logger calls back
// into Lua (`logger_cb`): a Lua error raised there unwinds out through
// tree-sitter's frames, so these three are declared as able to unwind.
unsafe extern "C-unwind" {
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
        string: *const c_char,
        length: uint32_t,
    ) -> *mut TSTree;
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
    fn ts_parser_reset(self_0: *mut TSParser);
    fn ts_parser_set_logger(self_0: *mut TSParser, logger: TSLogger);
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
    fn ts_node_type(self_0: TSNode) -> *const c_char;
    fn ts_node_symbol(self_0: TSNode) -> TSSymbol;
    fn ts_node_start_byte(self_0: TSNode) -> uint32_t;
    fn ts_node_start_point(self_0: TSNode) -> TSPoint;
    fn ts_node_end_byte(self_0: TSNode) -> uint32_t;
    fn ts_node_end_point(self_0: TSNode) -> TSPoint;
    fn ts_node_string(self_0: TSNode) -> *mut c_char;
    fn ts_node_is_null(self_0: TSNode) -> bool;
    fn ts_node_is_named(self_0: TSNode) -> bool;
    fn ts_node_is_missing(self_0: TSNode) -> bool;
    fn ts_node_is_extra(self_0: TSNode) -> bool;
    fn ts_node_has_changes(self_0: TSNode) -> bool;
    fn ts_node_has_error(self_0: TSNode) -> bool;
    fn ts_node_parent(self_0: TSNode) -> TSNode;
    fn ts_node_child_with_descendant(self_0: TSNode, descendant: TSNode) -> TSNode;
    fn ts_node_child(self_0: TSNode, child_index: uint32_t) -> TSNode;
    fn ts_node_field_name_for_child(self_0: TSNode, child_index: uint32_t) -> *const c_char;
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
        source: *const c_char,
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
    ) -> *const c_char;
    fn ts_query_string_value_for_id(
        self_0: *const TSQuery,
        index: uint32_t,
        length: *mut uint32_t,
    ) -> *const c_char;
    fn ts_query_disable_capture(self_0: *mut TSQuery, name: *const c_char, length: uint32_t);
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
    fn ts_language_field_name_for_id(self_0: *const TSLanguage, id: TSFieldId) -> *const c_char;
    fn ts_language_supertypes(self_0: *const TSLanguage, length: *mut uint32_t) -> *const TSSymbol;
    fn ts_language_subtypes(
        self_0: *const TSLanguage,
        supertype: TSSymbol,
        length: *mut uint32_t,
    ) -> *const TSSymbol;
    fn ts_language_symbol_name(self_0: *const TSLanguage, symbol: TSSymbol) -> *const c_char;
    fn ts_language_symbol_type(self_0: *const TSLanguage, symbol: TSSymbol) -> TSSymbolType;
    fn ts_language_abi_version(self_0: *const TSLanguage) -> uint32_t;
    fn ts_language_metadata(self_0: *const TSLanguage) -> *const TSLanguageMetadata;
    fn ts_language_is_wasm(_: *const TSLanguage) -> bool;
    fn ts_set_allocator(
        new_malloc: Option<unsafe extern "C" fn(size_t) -> *mut c_void>,
        new_calloc: Option<unsafe extern "C" fn(size_t, size_t) -> *mut c_void>,
        new_realloc: Option<unsafe extern "C" fn(*mut c_void, size_t) -> *mut c_void>,
        new_free: Option<unsafe extern "C" fn(*mut c_void) -> ()>,
    );
}
pub type TSSymbol = uint16_t;
pub type TSFieldId = uint16_t;
pub type TSDecodeFunction =
    Option<unsafe extern "C" fn(*const uint8_t, uint32_t, *mut int32_t) -> uint32_t>;
pub type TSInputEncoding = c_uint;
pub const INPUT_ENCODING_UTF8: TSInputEncoding = 0;
pub type TSSymbolType = c_uint;
pub const SYMBOL_TYPE_AUXILIARY: TSSymbolType = 3;
pub const SYMBOL_TYPE_ANONYMOUS: TSSymbolType = 1;
#[repr(C)]
pub struct TSPoint {
    pub row: uint32_t,
    pub column: uint32_t,
}
#[repr(C)]
pub struct TSRange {
    pub start_point: TSPoint,
    pub end_point: TSPoint,
    pub start_byte: uint32_t,
    pub end_byte: uint32_t,
}
#[repr(C)]
pub struct TSInput {
    pub payload: *mut c_void,
    pub read: Option<
        unsafe extern "C" fn(*mut c_void, uint32_t, TSPoint, *mut uint32_t) -> *const c_char,
    >,
    pub encoding: TSInputEncoding,
    pub decode: TSDecodeFunction,
}
#[repr(C)]
pub struct TSParseState {
    pub payload: *mut c_void,
    pub current_byte_offset: uint32_t,
    pub has_error: bool,
}
#[repr(C)]
pub struct TSParseOptions {
    pub payload: *mut c_void,
    pub progress_callback: Option<unsafe extern "C" fn(*mut TSParseState) -> bool>,
}
pub type TSLogType = c_uint;
pub const LOG_TYPE_LEX: TSLogType = 1;
pub const LOG_TYPE_PARSE: TSLogType = 0;
#[repr(C)]
pub struct TSLogger {
    pub payload: *mut c_void,
    pub log: Option<unsafe extern "C-unwind" fn(*mut c_void, TSLogType, *const c_char) -> ()>,
}
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
    pub id: *const c_void,
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
pub type TSQueryPredicateStepType = c_uint;
pub const STEP_TYPE_STRING: TSQueryPredicateStepType = 2;
pub const STEP_TYPE_CAPTURE: TSQueryPredicateStepType = 1;
pub const STEP_TYPE_DONE: TSQueryPredicateStepType = 0;
#[repr(C)]
pub struct TSQueryPredicateStep {
    pub type_0: TSQueryPredicateStepType,
    pub value_id: uint32_t,
}
pub type TSQueryError = c_uint;
pub const QUERY_ERROR_CAPTURE: TSQueryError = 4;
pub const QUERY_ERROR_FIELD: TSQueryError = 3;
pub const QUERY_ERROR_NODE_TYPE: TSQueryError = 2;
pub const QUERY_ERROR_NONE: TSQueryError = 0;
#[repr(C)]
pub struct TSLanguageMetadata {
    pub major_version: uint8_t,
    pub minor_version: uint8_t,
    pub patch_version: uint8_t,
}

pub const TREE_SITTER_LANGUAGE_VERSION: u32 = 15;
pub const TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION: u32 = 13;

/// The loaded parsers, by language name (with its NUL). Asked, never walked.
static LANGUAGES: GlobalCell<IdMap<Box<[u8]>, *mut TSLanguage>> = GlobalCell::new(id_map());

/// Register `T`'s metatable with `methods`, the metatable its own
/// `__index`, unless a state already has it; leaves the stack as it was.
fn register_class<T: Userdata>(stack: &mut Stack, methods: &[Function]) {
    if stack.new_userdata_metatable::<T>() {
        stack.register_functions(methods);
        stack.push_value(At::TOP);
        stack.set_field(At::from_top(2), c"__index");
    }
    stack.pop(1);
}

/// `(uint32_t)n`, as C converts a `lua_Integer`: its low 32 bits.
fn wrap_u32(n: lua_Integer) -> u32 {
    u32::try_from(n & 0xFFFF_FFFF).expect("masked to 32 bits")
}

/// A tree-sitter count or offset as a Lua integer.
fn integer(n: u32) -> lua_Integer {
    lua_Integer::try_from(n).expect("a 64-bit lua_Integer holds any u32")
}

/// The bytes of a C string tree-sitter answered, up to its NUL; none for a
/// null pointer.
///
/// # Safety
/// `string` is null or NUL-terminated, and outlives `'a`.
unsafe fn c_bytes<'a>(string: *const c_char) -> &'a [u8] {
    if string.is_null() {
        return b"";
    }
    // SAFETY: the caller's NUL-terminated string.
    unsafe { CStr::from_ptr(string) }.to_bytes()
}

/// `lua_pushstring`: the C string tree-sitter answered, or nil for a null
/// pointer.
///
/// # Safety
/// As [`c_bytes`].
unsafe fn push_c_string(stack: &mut Stack, string: *const c_char) {
    if string.is_null() {
        stack.push_nil();
    } else {
        // SAFETY: the caller's NUL-terminated string.
        stack.push_bytes(unsafe { c_bytes(string) });
    }
}

/// `bytes` up to its first NUL: what a C string read out of a Lua string
/// sees.
fn until_nul(bytes: &[u8]) -> &[u8] {
    bytes.split(|&byte| byte == 0).next().unwrap_or(bytes)
}
