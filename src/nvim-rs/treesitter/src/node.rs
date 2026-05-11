//! Lua metatable methods for TSNode userdata.
//!
//! Phase 1: leaf accessor functions (node_tostring, node_eq, node_id,
//!   node_named, node_missing, node_extra, node_has_changes, node_has_error,
//!   node_type, node_symbol, node_child_count, node_named_child_count,
//!   node_byte_length, node_equal, node_sexpr).
//!
//! Phase 2: range/start/end, sibling/parent/child, push_node helper.
//! Phase 4: field iteration, named_children, iter_children, next_child,
//!          has_ancestor, child_with_descendant.

use std::ffi::{c_int, c_void};
use std::mem::size_of;

use crate::ts_sys::*;
use crate::types::{TSNode, TSPoint, LUA_REGISTRYINDEX, LUA_TSTRING, TS_META_NODE};

// =============================================================================
// Internal helpers
// =============================================================================

/// Check and return the TSNode at the given Lua stack index.
///
/// Calls luaL_checkudata — may longjmp (no Rust destructors on caller frame).
///
/// # Safety
/// `L` must be a valid Lua state. `index` must be a valid stack position.
#[inline]
unsafe fn node_check(L: *mut c_void, index: c_int) -> TSNode {
    let ud = luaL_checkudata(L, index, TS_META_NODE.as_ptr());
    *(ud as *const TSNode)
}

/// Push a TSNode userdata onto the Lua stack.
///
/// If the node is null, pushes nil. Copies the fenv from `uindex` to keep
/// the backing tree alive (GC anchor).
///
/// # Safety
/// `L` must be a valid Lua state. `uindex` must be a valid absolute index
/// (i.e. > 0 or <= -LUA_MINSTACK).
pub unsafe fn push_node(L: *mut c_void, node: TSNode, uindex: c_int) {
    if ts_node_is_null(node) {
        lua_pushnil(L);
        return;
    }

    let ud = lua_newuserdata(L, size_of::<TSNode>()) as *mut TSNode; // [udata]
    *ud = node;
    lua_getfield(L, LUA_REGISTRYINDEX, TS_META_NODE.as_ptr()); // [udata, meta]
    lua_setmetatable(L, -2); // [udata]

    // Copy the fenv from uindex to keep the tree alive.
    lua_getfenv(L, uindex); // [udata, reftable]
    lua_setfenv(L, -2); // [udata]
}

// =============================================================================
// Phase 1: Leaf accessor implementations
// =============================================================================

/// __tostring: "<node TYPE>"
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_tostring")]
pub unsafe extern "C" fn node_tostring(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushstring(L, c"<node ".as_ptr());
    lua_pushstring(L, ts_node_type(node));
    lua_pushstring(L, c">".as_ptr());
    lua_concat(L, 3);
    1
}

/// __eq: ts_node_eq
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_eq")]
pub unsafe extern "C" fn node_eq(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let node2 = node_check(L, 2);
    lua_pushboolean(L, c_int::from(ts_node_eq(node, node2)));
    1
}

/// id: push raw node.id pointer as a Lua string
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_id")]
pub unsafe extern "C" fn node_id(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushlstring(
        L,
        (&raw const node.id).cast::<std::ffi::c_char>(),
        size_of::<*const c_void>(),
    );
    1
}

/// named: ts_node_is_named
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_named")]
pub unsafe extern "C" fn node_named(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushboolean(L, c_int::from(ts_node_is_named(node)));
    1
}

/// missing: ts_node_is_missing
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_missing")]
pub unsafe extern "C" fn node_missing(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushboolean(L, c_int::from(ts_node_is_missing(node)));
    1
}

/// extra: ts_node_is_extra
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_extra")]
pub unsafe extern "C" fn node_extra(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushboolean(L, c_int::from(ts_node_is_extra(node)));
    1
}

/// has_changes: ts_node_has_changes
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_has_changes")]
pub unsafe extern "C" fn node_has_changes(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushboolean(L, c_int::from(ts_node_has_changes(node)));
    1
}

/// has_error: ts_node_has_error
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_has_error")]
pub unsafe extern "C" fn node_has_error(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushboolean(L, c_int::from(ts_node_has_error(node)));
    1
}

/// type: push ts_node_type string
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_type")]
pub unsafe extern "C" fn node_type(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushstring(L, ts_node_type(node));
    1
}

/// symbol: push ts_node_symbol as integer
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_symbol")]
pub unsafe extern "C" fn node_symbol(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushinteger(L, i64::from(ts_node_symbol(node)));
    1
}

/// child_count / __len: ts_node_child_count
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_child_count")]
pub unsafe extern "C" fn node_child_count(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushinteger(L, i64::from(ts_node_child_count(node)));
    1
}

/// named_child_count: ts_node_named_child_count
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_named_child_count")]
pub unsafe extern "C" fn node_named_child_count(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    lua_pushinteger(L, i64::from(ts_node_named_child_count(node)));
    1
}

/// byte_length: end_byte - start_byte
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_byte_length")]
pub unsafe extern "C" fn node_byte_length(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let len = ts_node_end_byte(node) - ts_node_start_byte(node);
    lua_pushinteger(L, i64::from(len));
    1
}

/// equal: alias for node_eq (two-arg form)
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_equal")]
pub unsafe extern "C" fn node_equal(L: *mut c_void) -> c_int {
    let node1 = node_check(L, 1);
    let node2 = node_check(L, 2);
    lua_pushboolean(L, c_int::from(ts_node_eq(node1, node2)));
    1
}

/// sexpr: ts_node_string (allocates, xfree after push)
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_sexpr")]
pub unsafe extern "C" fn node_sexpr(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let allocated = ts_node_string(node);
    lua_pushstring(L, allocated);
    xfree(allocated as *mut c_void);
    1
}

// =============================================================================
// Phase 2: Range, start/end, sibling, parent, child, push_node
// =============================================================================

/// range: push start_row, start_col[, start_byte], end_row, end_col[, end_byte]
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_range")]
pub unsafe extern "C" fn node_range(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let include_bytes = lua_gettop(L) >= 2 && lua_toboolean(L, 2) != 0;

    let start = ts_node_start_point(node);
    let end = ts_node_end_point(node);

    lua_pushinteger(L, i64::from(start.row));
    lua_pushinteger(L, i64::from(start.column));
    if include_bytes {
        lua_pushinteger(L, i64::from(ts_node_start_byte(node)));
    }
    lua_pushinteger(L, i64::from(end.row));
    lua_pushinteger(L, i64::from(end.column));
    if include_bytes {
        lua_pushinteger(L, i64::from(ts_node_end_byte(node)));
        return 6;
    }
    4
}

/// start: push start_row, start_col, start_byte
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_start")]
pub unsafe extern "C" fn node_start(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let start = ts_node_start_point(node);
    lua_pushinteger(L, i64::from(start.row));
    lua_pushinteger(L, i64::from(start.column));
    lua_pushinteger(L, i64::from(ts_node_start_byte(node)));
    3
}

/// end_: push end_row, end_col, end_byte
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_end")]
pub unsafe extern "C" fn node_end(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let end = ts_node_end_point(node);
    lua_pushinteger(L, i64::from(end.row));
    lua_pushinteger(L, i64::from(end.column));
    lua_pushinteger(L, i64::from(ts_node_end_byte(node)));
    3
}

/// parent: ts_node_parent + push_node
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_parent")]
pub unsafe extern "C" fn node_parent(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let parent = ts_node_parent(node);
    push_node(L, parent, 1);
    1
}

/// next_sibling
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_next_sibling")]
pub unsafe extern "C" fn node_next_sibling(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let sibling = ts_node_next_sibling(node);
    push_node(L, sibling, 1);
    1
}

/// prev_sibling
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_prev_sibling")]
pub unsafe extern "C" fn node_prev_sibling(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let sibling = ts_node_prev_sibling(node);
    push_node(L, sibling, 1);
    1
}

/// next_named_sibling
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_next_named_sibling")]
pub unsafe extern "C" fn node_next_named_sibling(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let sibling = ts_node_next_named_sibling(node);
    push_node(L, sibling, 1);
    1
}

/// prev_named_sibling
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_prev_named_sibling")]
pub unsafe extern "C" fn node_prev_named_sibling(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let sibling = ts_node_prev_named_sibling(node);
    push_node(L, sibling, 1);
    1
}

/// child: ts_node_child at index
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_child")]
pub unsafe extern "C" fn node_child(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let num = lua_tointeger(L, 2) as u32;
    let child = ts_node_child(node, num);
    push_node(L, child, 1);
    1
}

/// named_child: ts_node_named_child at index
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_named_child")]
pub unsafe extern "C" fn node_named_child(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let num = lua_tointeger(L, 2) as u32;
    let child = ts_node_named_child(node, num);
    push_node(L, child, 1);
    1
}

/// descendant_for_range
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_descendant_for_range")]
pub unsafe extern "C" fn node_descendant_for_range(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let start = TSPoint {
        row: lua_tointeger(L, 2) as u32,
        column: lua_tointeger(L, 3) as u32,
    };
    let end = TSPoint {
        row: lua_tointeger(L, 4) as u32,
        column: lua_tointeger(L, 5) as u32,
    };
    let child = ts_node_descendant_for_point_range(node, start, end);
    push_node(L, child, 1);
    1
}

/// named_descendant_for_range
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_named_descendant_for_range")]
pub unsafe extern "C" fn node_named_descendant_for_range(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let start = TSPoint {
        row: lua_tointeger(L, 2) as u32,
        column: lua_tointeger(L, 3) as u32,
    };
    let end = TSPoint {
        row: lua_tointeger(L, 4) as u32,
        column: lua_tointeger(L, 5) as u32,
    };
    let child = ts_node_named_descendant_for_point_range(node, start, end);
    push_node(L, child, 1);
    1
}

/// child_with_descendant
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_child_with_descendant")]
pub unsafe extern "C" fn node_child_with_descendant(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let descendant = node_check(L, 2);
    let child = ts_node_child_with_descendant(node, descendant);
    push_node(L, child, 1);
    1
}

/// root: ts_tree_root_node(node.tree)
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_root")]
pub unsafe extern "C" fn node_root(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let root = ts_tree_root_node(node.tree);
    push_node(L, root, 1);
    1
}

/// tree: return tree from the node's fenv (avoids double-free)
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_tree")]
pub unsafe extern "C" fn node_tree(L: *mut c_void) -> c_int {
    // Just validate that arg 1 is a node, then get tree from fenv
    let _ = node_check(L, 1);
    lua_getfenv(L, 1); // [node, reftable]
    lua_rawgeti(L, 2, 1); // [node, reftable, tree]
    1
}

// =============================================================================
// Phase 4: Iteration and field lookup
// =============================================================================

/// field: collect all children with matching field name into a table
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_field")]
pub unsafe extern "C" fn node_field(L: *mut c_void) -> c_int {
    let node = node_check(L, 1);
    let count = ts_node_child_count(node);
    let mut curr_index: c_int = 0;

    let mut name_len: usize = 0;
    let field_name = luaL_checklstring(L, 2, &raw mut name_len);

    lua_newtable(L);

    for i in 0..count {
        let child_field_name = ts_node_field_name_for_child(node, i);
        if !child_field_name.is_null() {
            // strcmp equivalent using C string lengths
            let cfn_len = libc_strlen(child_field_name);
            if cfn_len == name_len
                && std::slice::from_raw_parts(child_field_name as *const u8, cfn_len)
                    == std::slice::from_raw_parts(field_name as *const u8, name_len)
            {
                let child = ts_node_child(node, i);
                push_node(L, child, 1);
                curr_index += 1;
                lua_rawseti(L, -2, curr_index);
            }
        }
    }

    1
}

/// named_children: collect all named children into a table
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_named_children")]
pub unsafe extern "C" fn node_named_children(L: *mut c_void) -> c_int {
    let source = node_check(L, 1);

    lua_newtable(L);
    let mut curr_index: c_int = 0;

    let n = ts_node_child_count(source);
    for i in 0..n {
        let child = ts_node_child(source, i);
        if ts_node_is_named(child) {
            push_node(L, child, 1);
            curr_index += 1;
            lua_rawseti(L, -2, curr_index);
        }
    }

    1
}

/// next_child closure: iterator function pushed by iter_children.
/// Upvalue 1: uint32_t* child_index (userdata)
/// Upvalue 2: TSNode source (userdata)
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_next_child")]
pub unsafe extern "C" fn node_next_child(L: *mut c_void) -> c_int {
    let child_index_ptr = lua_touserdata(L, lua_upvalueindex(1)) as *mut u32;
    let source = node_check(L, lua_upvalueindex(2));

    if *child_index_ptr >= ts_node_child_count(source) {
        return 0;
    }

    let child = ts_node_child(source, *child_index_ptr);
    push_node(L, child, lua_upvalueindex(2));

    let field = ts_node_field_name_for_child(source, *child_index_ptr);
    if field.is_null() {
        lua_pushnil(L);
    } else {
        lua_pushstring(L, field);
    }

    *child_index_ptr += 1;

    2
}

/// iter_children: push a closure that iterates children
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_node_iter_children")]
pub unsafe extern "C" fn node_iter_children(L: *mut c_void) -> c_int {
    let _ = node_check(L, 1); // validate arg
    let child_index = lua_newuserdata(L, size_of::<u32>()) as *mut u32; // [source, ..., udata]
    *child_index = 0;
    lua_pushvalue(L, 1); // [source, ..., udata, source]
    lua_pushcclosure(L, node_next_child, 2);
    1
}

/// __has_ancestor: check whether any ancestor of `descendant` matches a type
/// in the predicate list.
///
/// Stack: (1) descendant TSNode, (2) preds table, (3..pred_len) type strings
///
/// # Safety
/// `L` must be a valid Lua state.
#[unsafe(export_name = "rs_ts_has_ancestor")]
pub unsafe extern "C" fn has_ancestor(L: *mut c_void) -> c_int {
    let descendant = node_check(L, 1);

    if lua_type(L, 2) != crate::types::LUA_TTABLE {
        lua_pushboolean(L, 0);
        return 1;
    }

    let pred_len = lua_objlen(L, 2) as c_int;

    let mut node = ts_tree_root_node(descendant.tree);
    while node.id != descendant.id && !ts_node_is_null(node) {
        let node_type_ptr = ts_node_type(node);
        let node_type_len = libc_strlen(node_type_ptr);
        let node_type_bytes = std::slice::from_raw_parts(node_type_ptr as *const u8, node_type_len);

        for i in 3..=pred_len {
            lua_rawgeti(L, 2, i);
            if lua_type(L, -1) == LUA_TSTRING {
                let mut check_len: usize = 0;
                let check_str = lua_tolstring(L, -1, &raw mut check_len);
                if node_type_len == check_len
                    && std::slice::from_raw_parts(check_str as *const u8, check_len)
                        == node_type_bytes
                {
                    lua_pushboolean(L, 1);
                    return 1;
                }
            }
            lua_pop(L, 1);
        }

        node = ts_node_child_with_descendant(node, descendant);
    }

    lua_pushboolean(L, 0);
    1
}

// Inline strlen to avoid linking libc explicitly
#[inline]
const unsafe fn libc_strlen(s: *const std::ffi::c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}
