//! The tree userdatum: copying it, editing it, and its root.
//!
//! A [`TSLuaTree`] is a refcounted `TSTree *`, so `tree_copy` shares rather
//! than clones and `tree_gc` only deletes at the last reference.  `tree_edit`
//! is the incremental-parse hook -- it tells tree-sitter which byte span
//! changed so the next parse can reuse the rest.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::global_cell::SharedCell;
use crate::luaL_reg_table;

pub(crate) static tree_meta: SharedCell<[luaL_Reg; 7]> = luaL_reg_table![
    c"__gc" => tree_gc,
    c"__tostring" => tree_tostring,
    c"root" => tree_root,
    c"edit" => tree_edit,
    c"included_ranges" => tree_get_ranges,
    c"copy" => tree_copy,
];

pub(crate) unsafe extern "C-unwind" fn push_tree(mut L: *mut lua_State, mut tree: *const TSTree) {
    unsafe {
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
}

unsafe extern "C-unwind" fn tree_copy(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut ud: *mut TSLuaTree =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
        let mut copy: *mut TSTree = ts_tree_copy((*ud).tree);
        push_tree(L, copy);
        return 1 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn tree_edit(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        if lua_gettop(L) < 10 as ::core::ffi::c_int {
            lua_pushstring(L, c"not enough args to tree:edit()".as_ptr());
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
            column: luaL_checkinteger(L, 10 as ::core::ffi::c_int) as ::core::ffi::c_int
                as uint32_t,
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
}

unsafe extern "C-unwind" fn tree_get_ranges(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut ud: *mut TSLuaTree =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
        let mut include_bytes: bool = lua_gettop(L) >= 2 as ::core::ffi::c_int
            && lua_toboolean(L, 2 as ::core::ffi::c_int) != 0;
        let mut len: uint32_t = 0;
        let mut ranges: *mut TSRange = ts_tree_included_ranges((*ud).tree, &raw mut len);
        push_ranges(L, ranges, len as size_t, include_bytes);
        xfree(ranges as *mut ::core::ffi::c_void);
        return 1 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn tree_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut ud: *mut TSLuaTree =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_TREE.as_ptr()) as *mut TSLuaTree;
        let mut tree: *mut TSTree = (*ud).tree as *mut TSTree;
        ts_tree_delete(tree);
        return 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn tree_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        lua_pushstring(L, c"<tree>".as_ptr());
        return 1 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn tree_root(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
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
}
