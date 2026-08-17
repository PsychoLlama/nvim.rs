//! The query cursor, and the match it hands back.
//!
//! `tslua_push_querycursor` creates a cursor over (query, node) with the
//! byte/row limits and the match limit applied, and its `next_capture` /
//! `next_match` methods drive it.  A match is pushed as its own userdatum
//! ([`querymatch_meta`]) rather than copied, so the capture list is only
//! materialised if Lua asks for it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::global_cell::SharedCell;
use crate::luaL_reg_table;

pub(crate) static querycursor_meta: SharedCell<[luaL_Reg; 5]> = luaL_reg_table![
    c"remove_match" => querycursor_remove_match,
    c"next_capture" => querycursor_next_capture,
    c"next_match" => querycursor_next_match,
    c"__gc" => querycursor_gc,
];

pub(crate) unsafe extern "C-unwind" fn tslua_push_querycursor(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut node: TSNode = node_check(L, 1 as ::core::ffi::c_int);
        let mut query: *mut TSQuery = query_check(L, 2 as ::core::ffi::c_int);
        let mut cursor: *mut TSQueryCursor = ts_query_cursor_new();
        if lua_gettop(L) >= 3 as ::core::ffi::c_int
            && !(lua_type(L, 3 as ::core::ffi::c_int) == LUA_TNIL)
        {
            luaL_argcheck(
                L,
                lua_type(L, 3 as ::core::ffi::c_int) == 5 as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
                c"table expected".as_ptr(),
            );
        }
        lua_getfield(L, 3 as ::core::ffi::c_int, c"start_row".as_ptr());
        let mut start_row: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_getfield(L, 3 as ::core::ffi::c_int, c"start_col".as_ptr());
        let mut start_col: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_getfield(L, 3 as ::core::ffi::c_int, c"end_row".as_ptr());
        let mut end_row: uint32_t = luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_getfield(L, 3 as ::core::ffi::c_int, c"end_col".as_ptr());
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
        lua_getfield(L, 3 as ::core::ffi::c_int, c"max_start_depth".as_ptr());
        if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL) {
            let mut max_start_depth: uint32_t =
                luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
            ts_query_cursor_set_max_start_depth(cursor, max_start_depth);
        }
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_getfield(L, 3 as ::core::ffi::c_int, c"match_limit".as_ptr());
        if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL) {
            let mut match_limit: uint32_t =
                luaL_checkinteger(L, -1 as ::core::ffi::c_int) as uint32_t;
            ts_query_cursor_set_match_limit(cursor, match_limit);
        }
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        ts_query_cursor_exec(cursor, query, node);
        let mut ud: *mut *mut TSQueryCursor =
            lua_newuserdata(L, ::core::mem::size_of::<*mut TSQueryCursor>())
                as *mut *mut TSQueryCursor;
        *ud = cursor;
        lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERYCURSOR.as_ptr());
        lua_setmetatable(L, -2 as ::core::ffi::c_int);
        lua_getfenv(L, 1 as ::core::ffi::c_int);
        lua_setfenv(L, -2 as ::core::ffi::c_int);
        return 1 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn querycursor_remove_match(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut cursor: *mut TSQueryCursor = querycursor_check(L, 1 as ::core::ffi::c_int);
        let mut match_id: uint32_t = luaL_checkinteger(L, 2 as ::core::ffi::c_int) as uint32_t;
        ts_query_cursor_remove_match(cursor, match_id);
        return 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn querycursor_next_capture(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
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
        let mut capture: TSQueryCapture = *match_0.captures.add(capture_index as usize);
        lua_pushinteger(L, capture.index.wrapping_add(1 as uint32_t) as lua_Integer);
        push_node(L, capture.node, 1 as ::core::ffi::c_int);
        push_querymatch(L, &raw mut match_0, 1 as ::core::ffi::c_int);
        return 3 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn querycursor_next_match(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
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
}

unsafe extern "C-unwind" fn querycursor_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSQueryCursor {
    unsafe {
        let mut ud: *mut *mut TSQueryCursor =
            luaL_checkudata(L, index, TS_META_QUERYCURSOR.as_ptr()) as *mut *mut TSQueryCursor;
        luaL_argcheck(
            L,
            !(*ud).is_null(),
            index,
            c"TSQueryCursor expected".as_ptr(),
        );
        return *ud;
    }
}

unsafe extern "C-unwind" fn querycursor_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut cursor: *mut TSQueryCursor = querycursor_check(L, 1 as ::core::ffi::c_int);
        ts_query_cursor_delete(cursor);
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) static querymatch_meta: SharedCell<[luaL_Reg; 3]> = luaL_reg_table![
    c"info" => querymatch_info,
    c"captures" => querymatch_captures,
];

unsafe extern "C-unwind" fn push_querymatch(
    mut L: *mut lua_State,
    mut match_0: *mut TSQueryMatch,
    mut uindex: ::core::ffi::c_int,
) {
    unsafe {
        let mut ud: *mut TSQueryMatch =
            lua_newuserdata(L, ::core::mem::size_of::<TSQueryMatch>()) as *mut TSQueryMatch;
        *ud = *match_0;
        lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERYMATCH.as_ptr());
        lua_setmetatable(L, -2 as ::core::ffi::c_int);
        lua_getfenv(L, uindex);
        lua_setfenv(L, -2 as ::core::ffi::c_int);
    }
}

unsafe extern "C-unwind" fn querymatch_info(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut match_0: *mut TSQueryMatch =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_QUERYMATCH.as_ptr())
                as *mut TSQueryMatch;
        lua_pushinteger(L, (*match_0).id as lua_Integer);
        lua_pushinteger(
            L,
            ((*match_0).pattern_index as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                as lua_Integer,
        );
        return 2 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn querymatch_captures(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut match_0: *mut TSQueryMatch =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_QUERYMATCH.as_ptr())
                as *mut TSQueryMatch;
        lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        let mut i: size_t = 0 as size_t;
        while i < (*match_0).capture_count as size_t {
            let mut capture: TSQueryCapture = *(*match_0).captures.add(i);
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
                lua_objlen(L, -2 as ::core::ffi::c_int) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int,
            );
            lua_rawseti(L, -2 as ::core::ffi::c_int, index);
            i = i.wrapping_add(1);
        }
        return 1 as ::core::ffi::c_int;
    }
}
