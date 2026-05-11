//! Lua metatable methods for TSTree userdata.
//!
//! Phase 3: tree_tostring, tree_copy, tree_get_ranges, tree_gc,
//!          tree_root, tree_edit.
//! Also: push_tree (helper called by tree_copy, tree_root, tree_edit).

use std::ffi::{c_int, c_void};
use std::mem::size_of;

use crate::ts_sys::*;
use crate::types::{
    TSInputEdit, TSLuaTree, TSRange, LUA_REGISTRYINDEX, TS_META_NODE, TS_META_TREE,
};

// =============================================================================
// Internal helpers
// =============================================================================

/// Push a TSLuaTree userdata onto the stack. Transfers ownership.
/// If tree is null, pushes nil.
///
/// # Safety
/// `L` must be a valid Lua state.
unsafe fn push_tree(L: *mut c_void, tree: *const c_void) {
    if tree.is_null() {
        lua_pushnil(L);
        return;
    }

    let ud = lua_newuserdata(L, size_of::<TSLuaTree>()) as *mut TSLuaTree; // [udata]
    (*ud).tree = tree;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_TREE.as_ptr()); // [udata, meta]
    lua_setmetatable(L, -2); // [udata]
}

/// Push range table for include_bytes/no-bytes mode.
///
/// # Safety
/// `L` must be a valid Lua state. `ranges` must be valid for `length` elements.
pub unsafe fn push_ranges(
    L: *mut c_void,
    ranges: *const TSRange,
    length: usize,
    include_bytes: bool,
) {
    lua_createtable(L, length as c_int, 0);
    for i in 0..length {
        let r = &*ranges.add(i);
        lua_createtable(L, if include_bytes { 6 } else { 4 }, 0);
        let mut j: c_int = 1;
        lua_pushinteger(L, i64::from(r.start_point.row));
        lua_rawseti(L, -2, j);
        j += 1;
        lua_pushinteger(L, i64::from(r.start_point.column));
        lua_rawseti(L, -2, j);
        j += 1;
        if include_bytes {
            lua_pushinteger(L, i64::from(r.start_byte));
            lua_rawseti(L, -2, j);
            j += 1;
        }
        lua_pushinteger(L, i64::from(r.end_point.row));
        lua_rawseti(L, -2, j);
        j += 1;
        lua_pushinteger(L, i64::from(r.end_point.column));
        lua_rawseti(L, -2, j);
        j += 1;
        if include_bytes {
            lua_pushinteger(L, i64::from(r.end_byte));
            lua_rawseti(L, -2, j);
        }
        lua_rawseti(L, -2, (i + 1) as c_int);
    }
}

/// Check and return the TSLuaTree pointer at stack index.
///
/// # Safety
/// `L` must be a valid Lua state.
#[inline]
unsafe fn tree_check(L: *mut c_void, index: c_int) -> *const c_void {
    let ud = luaL_checkudata(L, index, TS_META_TREE.as_ptr()) as *const TSLuaTree;
    (*ud).tree
}

// =============================================================================
// Phase 3 implementations
// =============================================================================

/// __tostring: "<tree>"
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_tree_tostring")]
pub unsafe extern "C" fn tree_tostring(L: *mut c_void) -> c_int {
    lua_pushstring(L, c"<tree>".as_ptr());
    1
}

/// copy: ts_tree_copy + push_tree
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_tree_copy")]
pub unsafe extern "C" fn tree_copy(L: *mut c_void) -> c_int {
    let tree = tree_check(L, 1);
    let copy = ts_tree_copy(tree);
    push_tree(L, copy);
    1
}

/// included_ranges: ts_tree_included_ranges + push_ranges + xfree
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_tree_get_ranges")]
pub unsafe extern "C" fn tree_get_ranges(L: *mut c_void) -> c_int {
    let tree = tree_check(L, 1);
    let include_bytes = lua_gettop(L) >= 2 && lua_toboolean(L, 2) != 0;

    let mut len: u32 = 0;
    let ranges = ts_tree_included_ranges(tree, &raw mut len);

    push_ranges(L, ranges, len as usize, include_bytes);

    xfree(ranges as *mut c_void);
    1
}

/// __gc: ts_tree_delete
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_tree_gc")]
pub unsafe extern "C" fn tree_gc(L: *mut c_void) -> c_int {
    let ud = luaL_checkudata(L, 1, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
    // SAFETY: tree is only GC'd after all TSNode, TSQueryCursor etc. are unreachable
    // (each holds a fenv reference to the TSLuaTree). Casting const away is safe here.
    let tree = (*ud).tree.cast_mut();
    ts_tree_delete(tree);
    0
}

/// root: get root TSNode with fenv reftable anchoring the tree
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_tree_root")]
pub unsafe extern "C" fn tree_root(L: *mut c_void) -> c_int {
    let tree = tree_check(L, 1);

    let root = ts_tree_root_node(tree);

    let node_ud =
        lua_newuserdata(L, size_of::<crate::types::TSNode>()) as *mut crate::types::TSNode; // [node]
    *node_ud = root;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_NODE.as_ptr()); // [node, meta]
    lua_setmetatable(L, -2); // [node]

    // Build fenv reftable: { [1] = tree }
    lua_createtable(L, 1, 0); // [node, reftable]
    lua_pushvalue(L, 1); // [node, reftable, tree]
    lua_rawseti(L, -2, 1); // [node, reftable]
    lua_setfenv(L, -2); // [node]

    1
}

/// edit: build TSInputEdit and apply via ts_tree_copy + ts_tree_edit
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_tree_edit")]
pub unsafe extern "C" fn tree_edit(L: *mut c_void) -> c_int {
    if lua_gettop(L) < 10 {
        lua_pushstring(L, c"not enough args to tree:edit()".as_ptr());
        return lua_error(L);
    }

    let ud = luaL_checkudata(L, 1, TS_META_TREE.as_ptr()) as *const TSLuaTree;

    let start_byte = luaL_checkint(L, 2) as u32;
    let old_end_byte = luaL_checkint(L, 3) as u32;
    let new_end_byte = luaL_checkint(L, 4) as u32;
    let start_point = crate::types::TSPoint {
        row: luaL_checkint(L, 5) as u32,
        column: luaL_checkint(L, 6) as u32,
    };
    let old_end_point = crate::types::TSPoint {
        row: luaL_checkint(L, 7) as u32,
        column: luaL_checkint(L, 8) as u32,
    };
    let new_end_point = crate::types::TSPoint {
        row: luaL_checkint(L, 9) as u32,
        column: luaL_checkint(L, 10) as u32,
    };

    let edit = TSInputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_point,
        old_end_point,
        new_end_point,
    };

    let new_tree = ts_tree_copy((*ud).tree);
    ts_tree_edit(new_tree, &raw const edit);

    push_tree(L, new_tree);
    1
}
