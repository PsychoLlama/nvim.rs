//! Raw extern "C" bindings to the tree-sitter C library.
//!
//! Functions are declared here once and shared across all submodules.
//! The tree-sitter library is already linked into the final nvim binary
//! by the CMake build; no extra link flags are needed.

use std::ffi::{c_char, c_int, c_void};

use crate::types::{
    TSInputEdit, TSNode, TSPoint, TSQueryMatch, TSQueryPredicateStep, TSRange, TSSymbol,
};

extern "C" {
    // ---- Lua C API functions we need ----

    pub fn luaL_checkudata(L: *mut c_void, narg: c_int, tname: *const c_char) -> *mut c_void;
    pub fn luaL_argerror(L: *mut c_void, narg: c_int, extramsg: *const c_char) -> c_int;
    pub fn luaL_error(L: *mut c_void, fmt: *const c_char, ...) -> c_int;
    pub fn luaL_checkinteger(L: *mut c_void, narg: c_int) -> i64;
    pub fn luaL_checklstring(L: *mut c_void, narg: c_int, len: *mut usize) -> *const c_char;

    pub fn lua_pushboolean(L: *mut c_void, b: c_int);
    pub fn lua_pushinteger(L: *mut c_void, n: i64);
    pub fn lua_pushstring(L: *mut c_void, s: *const c_char) -> *const c_char;
    pub fn lua_pushlstring(L: *mut c_void, s: *const c_char, len: usize) -> *const c_char;
    pub fn lua_pushnil(L: *mut c_void);
    pub fn lua_pushvalue(L: *mut c_void, idx: c_int);
    pub fn lua_concat(L: *mut c_void, n: c_int);
    pub fn lua_settop(L: *mut c_void, idx: c_int);
    pub fn lua_gettop(L: *mut c_void) -> c_int;
    pub fn lua_toboolean(L: *mut c_void, idx: c_int) -> c_int;
    pub fn lua_tointeger(L: *mut c_void, idx: c_int) -> i64;
    pub fn lua_tolstring(L: *mut c_void, idx: c_int, len: *mut usize) -> *const c_char;
    pub fn lua_type(L: *mut c_void, idx: c_int) -> c_int;
    pub fn lua_istable(L: *mut c_void, idx: c_int) -> c_int;
    pub fn lua_objlen(L: *mut c_void, idx: c_int) -> usize;
    pub fn lua_rawgeti(L: *mut c_void, idx: c_int, n: c_int);
    pub fn lua_rawseti(L: *mut c_void, idx: c_int, n: c_int);
    pub fn lua_getfield(L: *mut c_void, idx: c_int, k: *const c_char);
    pub fn lua_setfield(L: *mut c_void, idx: c_int, k: *const c_char);
    pub fn lua_setmetatable(L: *mut c_void, objindex: c_int) -> c_int;
    pub fn lua_newuserdata(L: *mut c_void, sz: usize) -> *mut c_void;
    pub fn lua_getfenv(L: *mut c_void, idx: c_int);
    pub fn lua_setfenv(L: *mut c_void, idx: c_int) -> c_int;
    pub fn lua_createtable(L: *mut c_void, narr: c_int, nrec: c_int);
    pub fn lua_error(L: *mut c_void) -> c_int;
    pub fn lua_pushcclosure(
        L: *mut c_void,
        f: unsafe extern "C" fn(*mut c_void) -> c_int,
        n: c_int,
    );
    pub fn lua_touserdata(L: *mut c_void, idx: c_int) -> *mut c_void;

    // ---- Memory helpers ----
    pub fn xfree(ptr: *mut c_void);

    // ---- ts_node_* functions ----
    pub fn ts_node_type(node: TSNode) -> *const c_char;
    pub fn ts_node_symbol(node: TSNode) -> TSSymbol;
    pub fn ts_node_is_named(node: TSNode) -> bool;
    pub fn ts_node_is_missing(node: TSNode) -> bool;
    pub fn ts_node_is_extra(node: TSNode) -> bool;
    pub fn ts_node_has_changes(node: TSNode) -> bool;
    pub fn ts_node_has_error(node: TSNode) -> bool;
    pub fn ts_node_eq(a: TSNode, b: TSNode) -> bool;
    pub fn ts_node_child_count(node: TSNode) -> u32;
    pub fn ts_node_named_child_count(node: TSNode) -> u32;
    pub fn ts_node_start_byte(node: TSNode) -> u32;
    pub fn ts_node_end_byte(node: TSNode) -> u32;
    pub fn ts_node_start_point(node: TSNode) -> TSPoint;
    pub fn ts_node_end_point(node: TSNode) -> TSPoint;
    pub fn ts_node_string(node: TSNode) -> *mut c_char;
    pub fn ts_node_is_null(node: TSNode) -> bool;
    pub fn ts_node_parent(node: TSNode) -> TSNode;
    pub fn ts_node_next_sibling(node: TSNode) -> TSNode;
    pub fn ts_node_prev_sibling(node: TSNode) -> TSNode;
    pub fn ts_node_next_named_sibling(node: TSNode) -> TSNode;
    pub fn ts_node_prev_named_sibling(node: TSNode) -> TSNode;
    pub fn ts_node_child(node: TSNode, child_index: u32) -> TSNode;
    pub fn ts_node_named_child(node: TSNode, child_index: u32) -> TSNode;
    pub fn ts_node_descendant_for_point_range(node: TSNode, start: TSPoint, end: TSPoint)
        -> TSNode;
    pub fn ts_node_named_descendant_for_point_range(
        node: TSNode,
        start: TSPoint,
        end: TSPoint,
    ) -> TSNode;
    pub fn ts_node_child_with_descendant(node: TSNode, descendant: TSNode) -> TSNode;
    pub fn ts_node_field_name_for_child(node: TSNode, child_index: u32) -> *const c_char;

    // ---- ts_tree_* functions ----
    pub fn ts_tree_root_node(tree: *const c_void) -> TSNode;
    pub fn ts_tree_copy(tree: *const c_void) -> *mut c_void;
    pub fn ts_tree_delete(tree: *mut c_void);
    pub fn ts_tree_edit(tree: *mut c_void, edit: *const TSInputEdit);
    pub fn ts_tree_included_ranges(tree: *const c_void, length: *mut u32) -> *mut TSRange;

    // ---- ts_query_* functions ----
    pub fn ts_query_delete(query: *mut c_void);
    pub fn ts_query_disable_capture(query: *mut c_void, name: *const c_char, name_len: u32);
    pub fn ts_query_disable_pattern(query: *mut c_void, pattern_index: u32);
    pub fn ts_query_pattern_count(query: *const c_void) -> u32;
    pub fn ts_query_capture_count(query: *const c_void) -> u32;
    pub fn ts_query_predicates_for_pattern(
        query: *const c_void,
        pattern_index: u32,
        length: *mut u32,
    ) -> *const TSQueryPredicateStep;
    pub fn ts_query_string_value_for_id(
        query: *const c_void,
        id: u32,
        length: *mut u32,
    ) -> *const c_char;
    pub fn ts_query_capture_name_for_id(
        query: *const c_void,
        id: u32,
        length: *mut u32,
    ) -> *const c_char;

    // ---- ts_query_cursor_* functions ----
    pub fn ts_query_cursor_delete(cursor: *mut c_void);
    pub fn ts_query_cursor_remove_match(cursor: *mut c_void, match_id: u32);
    pub fn ts_query_cursor_next_match(cursor: *mut c_void, match_: *mut TSQueryMatch) -> bool;
    pub fn ts_query_cursor_next_capture(
        cursor: *mut c_void,
        match_: *mut TSQueryMatch,
        capture_index: *mut u32,
    ) -> bool;
}

/// luaL_argcheck macro: if !cond then luaL_argerror
#[inline]
pub unsafe fn luaL_argcheck(L: *mut c_void, cond: c_int, narg: c_int, extramsg: *const c_char) {
    if cond == 0 {
        luaL_argerror(L, narg, extramsg);
    }
}

/// lua_isnil macro: lua_type(L, n) == LUA_TNIL
#[inline]
pub unsafe fn lua_isnil(L: *mut c_void, n: c_int) -> c_int {
    c_int::from(lua_type(L, n) == crate::types::LUA_TNIL)
}

/// Inline lua_pop: lua_settop(L, -(n)-1)
#[inline]
pub unsafe fn lua_pop(L: *mut c_void, n: c_int) {
    lua_settop(L, -n - 1);
}

/// lua_newtable: lua_createtable(L, 0, 0)
#[inline]
pub unsafe fn lua_newtable(L: *mut c_void) {
    lua_createtable(L, 0, 0);
}

/// luaL_checkstring macro: luaL_checklstring(L, n, NULL)
#[inline]
pub unsafe fn luaL_checkstring(L: *mut c_void, n: c_int) -> *const c_char {
    luaL_checklstring(L, n, std::ptr::null_mut())
}

/// lua_upvalueindex: LUA_GLOBALSINDEX - i
#[inline]
#[must_use]
pub const fn lua_upvalueindex(i: c_int) -> c_int {
    crate::types::LUA_GLOBALSINDEX - i
}

/// luaL_checkint macro: (int)luaL_checkinteger(L, n)
#[inline]
pub unsafe fn luaL_checkint(L: *mut c_void, n: c_int) -> c_int {
    luaL_checkinteger(L, n) as c_int
}
