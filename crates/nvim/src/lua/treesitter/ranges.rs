//! Included ranges: the sub-spans of a source a parser is restricted to.
//!
//! A range is a byte span plus its two [`TSPoint`]s, and it crosses the Lua
//! boundary as either a 4- or a 6-element list; `range_from_lua` accepts
//! both and `push_ranges` renders either shape back.  `parser_set_ranges`
//! and `parser_get_ranges` are the parser methods over them.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

pub(crate) unsafe extern "C-unwind" fn push_ranges(
    mut L: *mut lua_State,
    mut ranges: *const TSRange,
    length: size_t,
    mut include_bytes: bool,
) {
    unsafe {
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
            lua_pushnumber(L, (*ranges.add(i)).start_point.row as lua_Number);
            let c2rust_fresh2 = j;
            j = j + 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh2);
            lua_pushnumber(L, (*ranges.add(i)).start_point.column as lua_Number);
            let c2rust_fresh3 = j;
            j = j + 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh3);
            if include_bytes {
                lua_pushnumber(L, (*ranges.add(i)).start_byte as lua_Number);
                let c2rust_fresh4 = j;
                j = j + 1;
                lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh4);
            }
            lua_pushnumber(L, (*ranges.add(i)).end_point.row as lua_Number);
            let c2rust_fresh5 = j;
            j = j + 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh5);
            lua_pushnumber(L, (*ranges.add(i)).end_point.column as lua_Number);
            let c2rust_fresh6 = j;
            j = j + 1;
            lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh6);
            if include_bytes {
                lua_pushnumber(L, (*ranges.add(i)).end_byte as lua_Number);
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
}

unsafe extern "C-unwind" fn range_err(mut L: *mut lua_State) {
    unsafe {
        luaL_error(
            L,
            c"Ranges can only be made from 6 element long tables or nodes.".as_ptr(),
        );
    }
}

unsafe extern "C-unwind" fn lua_checkuint32(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> uint32_t {
    unsafe {
        let mut value: lua_Number = luaL_checknumber(L, index);
        let mut converted: uint32_t = value as uint32_t;
        if value < 0 as ::core::ffi::c_int as lua_Number
            || value > UINT32_MAX as lua_Number
            || converted as lua_Number != value
        {
            luaL_error(L, c"Range value out of bounds".as_ptr());
        }
        return converted;
    }
}

unsafe extern "C-unwind" fn range_from_lua(mut L: *mut lua_State, mut range: *mut TSRange) {
    unsafe {
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
}

pub(crate) unsafe extern "C-unwind" fn parser_set_ranges(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        if lua_gettop(L) < 2 as ::core::ffi::c_int {
            return luaL_error(
                L,
                c"not enough args to parser:set_included_ranges()".as_ptr(),
            );
        }
        let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
        luaL_argcheck(
            L,
            lua_type(L, 2 as ::core::ffi::c_int) == 5 as ::core::ffi::c_int,
            2 as ::core::ffi::c_int,
            c"table expected.".as_ptr(),
        );
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
            range_from_lua(L, ranges.add(index));
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            index = index.wrapping_add(1);
        }
        ts_parser_set_included_ranges(p, ranges, tbl_len as uint32_t);
        xfree(ranges as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn parser_get_ranges(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
        let mut include_bytes: bool = lua_gettop(L) >= 2 as ::core::ffi::c_int
            && lua_toboolean(L, 2 as ::core::ffi::c_int) != 0;
        let mut len: uint32_t = 0;
        let mut ranges: *const TSRange = ts_parser_included_ranges(p, &raw mut len);
        push_ranges(L, ranges, len as size_t, include_bytes);
        return 1 as ::core::ffi::c_int;
    }
}
