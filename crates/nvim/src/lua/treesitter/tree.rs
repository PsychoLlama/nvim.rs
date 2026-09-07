//! The tree userdatum: copying it, editing it, and its root.
//!
//! A [`TSLuaTree`] is a refcounted `TSTree *`, so `tree_copy` shares rather
//! than clones and `tree_gc` only deletes at the last reference.  `tree_edit`
//! is the incremental-parse hook -- it tells tree-sitter which byte span
//! changed so the next parse can reuse the rest.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/treesitter/` row in docs/perimeter.md.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::global_cell::ConstTable;
use crate::luaL_reg_table;

pub(crate) static tree_meta: ConstTable<[luaL_Reg; 7]> = luaL_reg_table![
    c"__gc" => tree_gc,
    c"__tostring" => tree_tostring,
    c"root" => tree_root,
    c"edit" => tree_edit,
    c"included_ranges" => tree_get_ranges,
    c"copy" => tree_copy,
];

/// # Safety
///
/// `L` must point at the Lua state this call runs on.
/// `tree` must point at a live `TSTree`.
pub(crate) unsafe fn push_tree(L: *mut lua_State, tree: *const TSTree) {
    unsafe {
        if tree.is_null() {
            lua_pushnil(L);
            return;
        }
        let ud: *mut TSLuaTree =
            lua_newuserdata(L, ::core::mem::size_of::<TSLuaTree>()) as *mut TSLuaTree;
        (*ud).tree = tree;
        lua_getfield(L, LUA_REGISTRYINDEX, TS_META_TREE.as_ptr());
        lua_setmetatable(L, -2 as ::core::ffi::c_int);
    }
}

/// # Safety
///
/// `L` must point at the Lua state this call runs on.
unsafe extern "C-unwind" fn tree_copy(L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let ud: *mut TSLuaTree =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
        let copy: *mut TSTree = ts_tree_copy((*ud).tree);
        push_tree(L, copy);
        1 as ::core::ffi::c_int
    }
}

/// # Safety
///
/// `L` must point at the Lua state this call runs on.
unsafe extern "C-unwind" fn tree_edit(L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        if lua_gettop(L) < 10 as ::core::ffi::c_int {
            lua_pushstring(L, c"not enough args to tree:edit()".as_ptr());
            return lua_error(L);
        }
        let ud: *mut TSLuaTree =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
        let start_byte: uint32_t =
            luaL_checkinteger(L, 2 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t;
        let old_end_byte: uint32_t =
            luaL_checkinteger(L, 3 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t;
        let new_end_byte: uint32_t =
            luaL_checkinteger(L, 4 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t;
        let start_point: TSPoint = TSPoint {
            row: luaL_checkinteger(L, 5 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
            column: luaL_checkinteger(L, 6 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
        };
        let old_end_point: TSPoint = TSPoint {
            row: luaL_checkinteger(L, 7 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
            column: luaL_checkinteger(L, 8 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
        };
        let new_end_point: TSPoint = TSPoint {
            row: luaL_checkinteger(L, 9 as ::core::ffi::c_int) as ::core::ffi::c_int as uint32_t,
            column: luaL_checkinteger(L, 10 as ::core::ffi::c_int) as ::core::ffi::c_int
                as uint32_t,
        };
        let mut edit: TSInputEdit = TSInputEdit {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_point,
            old_end_point,
            new_end_point,
        };
        let new_tree: *mut TSTree = ts_tree_copy((*ud).tree);
        ts_tree_edit(new_tree, &raw mut edit);
        push_tree(L, new_tree);
        1 as ::core::ffi::c_int
    }
}

/// # Safety
///
/// `L` must point at the Lua state this call runs on.
unsafe extern "C-unwind" fn tree_get_ranges(L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let ud: *mut TSLuaTree =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
        let include_bytes: bool = lua_gettop(L) >= 2 as ::core::ffi::c_int
            && lua_toboolean(L, 2 as ::core::ffi::c_int) != 0;
        let mut len: uint32_t = 0;
        let ranges: *mut TSRange = ts_tree_included_ranges((*ud).tree, &raw mut len);
        push_ranges(L, ranges, len as size_t, include_bytes);
        xfree(ranges as *mut ::core::ffi::c_void);
        1 as ::core::ffi::c_int
    }
}

/// # Safety
///
/// `L` must point at the Lua state this call runs on.
unsafe extern "C-unwind" fn tree_gc(L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let ud: *mut TSLuaTree =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
        let tree: *mut TSTree = (*ud).tree as *mut TSTree;
        ts_tree_delete(tree);
        0 as ::core::ffi::c_int
    }
}

/// # Safety
///
/// `L` must point at the Lua state this call runs on.
unsafe extern "C-unwind" fn tree_tostring(L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        lua_pushstring(L, c"<tree>".as_ptr());
        1 as ::core::ffi::c_int
    }
}

/// # Safety
///
/// `L` must point at the Lua state this call runs on.
unsafe extern "C-unwind" fn tree_root(L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let ud: *mut TSLuaTree =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
        let root: TSNode = ts_tree_root_node((*ud).tree);
        let node_ud: *mut TSNode =
            lua_newuserdata(L, ::core::mem::size_of::<TSNode>()) as *mut TSNode;
        *node_ud = root;
        lua_getfield(L, LUA_REGISTRYINDEX, TS_META_NODE.as_ptr());
        lua_setmetatable(L, -2 as ::core::ffi::c_int);
        lua_createtable(L, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        lua_pushvalue(L, 1 as ::core::ffi::c_int);
        lua_rawseti(L, -2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        lua_setfenv(L, -2 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}
