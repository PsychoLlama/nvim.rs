//! The node userdatum and its thirty-odd accessors.
//!
//! A node is not owned: it is a `TSNode` value plus a reference to the tree
//! it belongs to, kept in the userdatum's environment so the tree outlives
//! it.  [`node_meta`] is the method table; everything below it is one
//! tree-sitter call apiece, and `push_node`/`node_check` are the two ends of
//! the Lua boundary they all go through.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::global_cell::ConstTable;
use crate::luaL_reg_table;

pub(crate) static node_meta: ConstTable<[luaL_Reg; 36]> = luaL_reg_table![
    c"__tostring" => node_tostring,
    c"__eq" => node_eq,
    c"__len" => node_child_count,
    c"id" => node_id,
    c"range" => node_range,
    c"start" => node_start,
    c"end_" => node_end,
    c"type" => node_type,
    c"symbol" => node_symbol,
    c"field" => node_field,
    c"named" => node_named,
    c"missing" => node_missing,
    c"extra" => node_extra,
    c"has_changes" => node_has_changes,
    c"has_error" => node_has_error,
    c"sexpr" => node_sexpr,
    c"child_count" => node_child_count,
    c"named_child_count" => node_named_child_count,
    c"child" => node_child,
    c"named_child" => node_named_child,
    c"descendant_for_range" => node_descendant_for_range,
    c"named_descendant_for_range" => node_named_descendant_for_range,
    c"parent" => node_parent,
    c"__has_ancestor" => __has_ancestor,
    c"child_with_descendant" => node_child_with_descendant,
    c"iter_children" => node_iter_children,
    c"next_sibling" => node_next_sibling,
    c"prev_sibling" => node_prev_sibling,
    c"next_named_sibling" => node_next_named_sibling,
    c"prev_named_sibling" => node_prev_named_sibling,
    c"named_children" => node_named_children,
    c"root" => node_root,
    c"tree" => node_tree,
    c"byte_length" => node_byte_length,
    c"equal" => node_equal,
];

pub(crate) unsafe fn push_node(
    mut L: *mut lua_State,
    mut node: TSNode,
    mut uindex: ::core::ffi::c_int,
) {
    unsafe {
        debug_assert!(
            uindex > 0 as ::core::ffi::c_int || uindex < -20 as ::core::ffi::c_int,
            "uindex > 0 || uindex < -LUA_MINSTACK"
        );
        if ts_node_is_null(node) {
            lua_pushnil(L);
            return;
        }
        let mut ud: *mut TSNode =
            lua_newuserdata(L, ::core::mem::size_of::<TSNode>()) as *mut TSNode;
        *ud = node;
        lua_getfield(L, LUA_REGISTRYINDEX, TS_META_NODE.as_ptr());
        lua_setmetatable(L, -2 as ::core::ffi::c_int);
        lua_getfenv(L, uindex);
        lua_setfenv(L, -2 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe fn node_check_opt(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
    mut res: *mut TSNode,
) -> bool {
    unsafe {
        let mut ud: *mut TSNode = luaL_checkudata(L, index, TS_META_NODE.as_ptr()) as *mut TSNode;
        if !ud.is_null() {
            *res = *ud;
            return true;
        }
        false
    }
}

pub(crate) unsafe fn node_check(mut L: *mut lua_State, mut index: ::core::ffi::c_int) -> TSNode {
    unsafe {
        let mut ud: *mut TSNode = luaL_checkudata(L, index, TS_META_NODE.as_ptr()) as *mut TSNode;
        *ud
    }
}

unsafe extern "C-unwind" fn node_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        lua_pushstring(L, c"<node ".as_ptr());
        lua_pushstring(L, ts_node_type(node));
        lua_pushstring(L, c">".as_ptr());
        lua_concat(L, 3 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_eq(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut node2: TSNode = node_check(L, 2 as ::core::ffi::c_int);
        lua_pushboolean(L, ts_node_eq(node, node2) as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_id(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        lua_pushlstring(
            L,
            &raw mut node.id as *const ::core::ffi::c_char,
            ::core::mem::size_of::<*const ::core::ffi::c_void>(),
        );
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_range(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut include_bytes: bool = lua_gettop(L) >= 2 as ::core::ffi::c_int
            && lua_toboolean(L, 2 as ::core::ffi::c_int) != 0;
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
        4 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_start(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut start: TSPoint = ts_node_start_point(node);
        let mut start_byte: uint32_t = ts_node_start_byte(node);
        lua_pushinteger(L, start.row as lua_Integer);
        lua_pushinteger(L, start.column as lua_Integer);
        lua_pushinteger(L, start_byte as lua_Integer);
        3 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_end(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut end: TSPoint = ts_node_end_point(node);
        let mut end_byte: uint32_t = ts_node_end_byte(node);
        lua_pushinteger(L, end.row as lua_Integer);
        lua_pushinteger(L, end.column as lua_Integer);
        lua_pushinteger(L, end_byte as lua_Integer);
        3 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_child_count(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut count: uint32_t = ts_node_child_count(node);
        lua_pushinteger(L, count as lua_Integer);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_named_child_count(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut count: uint32_t = ts_node_named_child_count(node);
        lua_pushinteger(L, count as lua_Integer);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_type(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        lua_pushstring(L, ts_node_type(node));
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_symbol(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut symbol: TSSymbol = ts_node_symbol(node);
        lua_pushinteger(L, symbol as lua_Integer);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_field(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
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
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_named(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        lua_pushboolean(L, ts_node_is_named(node) as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_sexpr(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut allocated: *mut ::core::ffi::c_char = ts_node_string(node);
        lua_pushstring(L, allocated);
        xfree(allocated as *mut ::core::ffi::c_void);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_missing(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        lua_pushboolean(L, ts_node_is_missing(node) as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_extra(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        lua_pushboolean(L, ts_node_is_extra(node) as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_has_changes(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        lua_pushboolean(L, ts_node_has_changes(node) as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_has_error(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        lua_pushboolean(L, ts_node_has_error(node) as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_child(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut num: uint32_t = lua_tointeger(L, 2 as ::core::ffi::c_int) as uint32_t;
        let mut child: TSNode = ts_node_child(node, num);
        push_node(L, child, 1 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_named_child(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut num: uint32_t = lua_tointeger(L, 2 as ::core::ffi::c_int) as uint32_t;
        let mut child: TSNode = ts_node_named_child(node, num);
        push_node(L, child, 1 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_descendant_for_range(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
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
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_named_descendant_for_range(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
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
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_next_child(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut child_index: *mut uint32_t =
            lua_touserdata(L, LUA_GLOBALSINDEX - 1 as ::core::ffi::c_int) as *mut uint32_t;
        let mut source: TSNode = node_check(L, LUA_GLOBALSINDEX - 2 as ::core::ffi::c_int);
        if *child_index >= ts_node_child_count(source) {
            return 0 as ::core::ffi::c_int;
        }
        let mut child: TSNode = ts_node_child(source, *child_index);
        push_node(L, child, LUA_GLOBALSINDEX - 2 as ::core::ffi::c_int);
        let mut field: *const ::core::ffi::c_char =
            ts_node_field_name_for_child(source, *child_index);
        if !field.is_null() {
            lua_pushstring(L, field);
        } else {
            lua_pushnil(L);
        }
        *child_index = (*child_index).wrapping_add(1);
        2 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_iter_children(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        node_check(L, 1 as ::core::ffi::c_int);
        let mut child_index: *mut uint32_t =
            lua_newuserdata(L, ::core::mem::size_of::<uint32_t>()) as *mut uint32_t;
        *child_index = 0 as uint32_t;
        lua_pushvalue(L, 1 as ::core::ffi::c_int);
        lua_pushcclosure(
            L,
            Some(
                node_next_child
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            2 as ::core::ffi::c_int,
        );
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_parent(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut parent: TSNode = ts_node_parent(node);
        push_node(L, parent, 1 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn __has_ancestor(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut descendant: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        if lua_type(L, 2 as ::core::ffi::c_int) != LUA_TTABLE {
            lua_pushboolean(L, 0);
            return 1 as ::core::ffi::c_int;
        }
        let pred_len: ::core::ffi::c_int =
            lua_objlen(L, 2 as ::core::ffi::c_int) as ::core::ffi::c_int;
        let mut node: TSNode = ts_tree_root_node(descendant.tree);
        while node.id != descendant.id && !ts_node_is_null(node) {
            let mut node_type_0: *const ::core::ffi::c_char = ts_node_type(node);
            let mut node_type_len: size_t = cstr::bytes_at(node_type_0).len();
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
                        lua_pushboolean(L, 1);
                        return 1 as ::core::ffi::c_int;
                    }
                }
                lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                i += 1;
            }
            node = ts_node_child_with_descendant(node, descendant);
        }
        lua_pushboolean(L, 0);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_child_with_descendant(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut descendant: TSNode = node_check(L, 2 as ::core::ffi::c_int);
        let mut child: TSNode = ts_node_child_with_descendant(node, descendant);
        push_node(L, child, 1 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_next_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut sibling: TSNode = ts_node_next_sibling(node);
        push_node(L, sibling, 1 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_prev_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut sibling: TSNode = ts_node_prev_sibling(node);
        push_node(L, sibling, 1 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_next_named_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut sibling: TSNode = ts_node_next_named_sibling(node);
        push_node(L, sibling, 1 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_prev_named_sibling(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut sibling: TSNode = ts_node_prev_named_sibling(node);
        push_node(L, sibling, 1 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_named_children(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
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
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_root(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut root: TSNode = ts_tree_root_node(node.tree);
        push_node(L, root, 1 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_tree(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        node_check(L, 1 as ::core::ffi::c_int);
        lua_getfenv(L, 1 as ::core::ffi::c_int);
        lua_rawgeti(L, 2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_byte_length(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut start_byte: uint32_t = ts_node_start_byte(node);
        let mut end_byte: uint32_t = ts_node_end_byte(node);
        lua_pushinteger(L, end_byte.wrapping_sub(start_byte) as lua_Integer);
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn node_equal(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut node1: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut node2: TSNode = node_check(L, 2 as ::core::ffi::c_int);
        lua_pushboolean(L, ts_node_eq(node1, node2) as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}
