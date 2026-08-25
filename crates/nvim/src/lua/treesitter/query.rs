//! Compiling a query, and inspecting or disabling parts of it.
//!
//! `tslua_parse_query` compiles the query text against a loaded language and
//! turns a `TSQueryError` into the pointed message `query_err_string`
//! builds (offset, line, and the offending word).  `query_inspect` reports a
//! compiled query's patterns, captures and predicates back to Lua.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::global_cell::ConstTable;
use crate::luaL_reg_table;

pub(crate) static query_meta: ConstTable<[luaL_Reg; 6]> = luaL_reg_table![
    c"__gc" => query_gc,
    c"__tostring" => query_tostring,
    c"inspect" => query_inspect,
    c"disable_capture" => query_disable_capture,
    c"disable_pattern" => query_disable_pattern,
];

pub(crate) unsafe extern "C-unwind" fn tslua_parse_query(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        if lua_gettop(L) < 2 as ::core::ffi::c_int
            || lua_isstring(L, 1 as ::core::ffi::c_int) == 0
            || lua_isstring(L, 2 as ::core::ffi::c_int) == 0
        {
            return luaL_error(L, c"string expected".as_ptr());
        }
        let mut lang: *mut TSLanguage = lang_check(L, 1 as ::core::ffi::c_int);
        let mut len: size_t = 0;
        let mut src: *const ::core::ffi::c_char =
            lua_tolstring(L, 2 as ::core::ffi::c_int, &raw mut len);
        tslua_query_parse_count.set(tslua_query_parse_count.get().wrapping_add(1));
        let mut error_offset: uint32_t = 0;
        let mut error_type: TSQueryError = TSQueryErrorNone;
        let mut query: *mut TSQuery = ts_query_new(
            lang,
            src,
            len as uint32_t,
            &raw mut error_offset,
            &raw mut error_type,
        );
        if query.is_null() {
            let mut err_msg: [::core::ffi::c_char; 1025] = [0; 1025];
            query_err_string(
                src,
                error_offset as ::core::ffi::c_int,
                error_type,
                &raw mut err_msg as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
            );
            return luaL_error(
                L,
                c"%s".as_ptr(),
                &raw mut err_msg as *mut ::core::ffi::c_char,
            );
        }
        let mut ud: *mut *mut TSQuery =
            lua_newuserdata(L, ::core::mem::size_of::<*mut TSQuery>()) as *mut *mut TSQuery;
        *ud = query;
        lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERY.as_ptr());
        lua_setmetatable(L, -2 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

fn query_err_to_string(mut error_type: TSQueryError) -> *const ::core::ffi::c_char {
    match error_type as ::core::ffi::c_uint {
        1 => c"Invalid syntax:\n".as_ptr(),
        2 => c"Invalid node type ".as_ptr(),
        3 => c"Invalid field name ".as_ptr(),
        4 => c"Invalid capture name ".as_ptr(),
        5 => c"Impossible pattern:\n".as_ptr(),
        _ => c"error".as_ptr(),
    }
}

unsafe fn query_err_string(
    mut src: *const ::core::ffi::c_char,
    mut error_offset: ::core::ffi::c_int,
    mut error_type: TSQueryError,
    mut err: *mut ::core::ffi::c_char,
    mut errlen: size_t,
) {
    unsafe {
        let mut line_start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut error_line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut error_line_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut end_str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        loop {
            let mut src_tmp: *const ::core::ffi::c_char = src.add(line_start as usize);
            end_str = strchr(src_tmp, '\n' as ::core::ffi::c_int);
            let mut line_length: ::core::ffi::c_int = if !end_str.is_null() {
                end_str.offset_from(src_tmp) as ::core::ffi::c_int
            } else {
                strlen(src_tmp) as ::core::ffi::c_int
            };
            let mut line_end: ::core::ffi::c_int = line_start + line_length;
            if line_end > error_offset {
                error_line = src_tmp;
                error_line_len = line_length;
                break;
            } else {
                line_start = line_end + 1 as ::core::ffi::c_int;
                row += 1;
                if end_str.is_null() {
                    break;
                }
            }
        }
        let mut column: ::core::ffi::c_int = error_offset - line_start;
        let mut type_msg: *const ::core::ffi::c_char = query_err_to_string(error_type);
        snprintf(
            err,
            errlen,
            c"Query error at %d:%d. %s".as_ptr(),
            row + 1 as ::core::ffi::c_int,
            column + 1 as ::core::ffi::c_int,
            type_msg,
        );
        let mut offset: size_t = strlen(err);
        errlen = errlen.wrapping_sub(offset);
        err = err.add(offset);
        if error_type as ::core::ffi::c_uint
            == TSQueryErrorNodeType as ::core::ffi::c_int as ::core::ffi::c_uint
            || error_type as ::core::ffi::c_uint
                == TSQueryErrorField as ::core::ffi::c_int as ::core::ffi::c_uint
            || error_type as ::core::ffi::c_uint
                == TSQueryErrorCapture as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut suffix: *const ::core::ffi::c_char = src.add(error_offset as usize);
            let mut is_anonymous: bool = error_type as ::core::ffi::c_uint
                == TSQueryErrorNodeType as ::core::ffi::c_int as ::core::ffi::c_uint
                && *suffix.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int;
            let mut suffix_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut c: ::core::ffi::c_char = *suffix.add(suffix_len as usize);
            if is_anonymous {
                let mut backslashes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while c as ::core::ffi::c_int != '"' as ::core::ffi::c_int
                    || backslashes % 2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
                {
                    if c as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                        backslashes += 1 as ::core::ffi::c_int;
                    } else {
                        backslashes = 0 as ::core::ffi::c_int;
                    }
                    suffix_len += 1;
                    c = *suffix.add(suffix_len as usize);
                }
            } else {
                while *(*__ctype_b_loc()).offset(c as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    != 0
                    || c as ::core::ffi::c_int == '_' as ::core::ffi::c_int
                    || c as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                    || c as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                {
                    suffix_len += 1;
                    c = *suffix.add(suffix_len as usize);
                }
            }
            snprintf(err, errlen, c"\"%.*s\":\n".as_ptr(), suffix_len, suffix);
            offset = strlen(err);
            errlen = errlen.wrapping_sub(offset);
            err = err.add(offset);
        }
        if error_line.is_null() {
            snprintf(err, errlen, c"Unexpected EOF\n".as_ptr());
            return;
        }
        snprintf(
            err,
            errlen,
            c"%.*s\n%*s^\n".as_ptr(),
            error_line_len,
            error_line,
            column,
            c"".as_ptr(),
        );
    }
}

pub(crate) unsafe fn query_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSQuery {
    unsafe {
        let mut ud: *mut *mut TSQuery =
            luaL_checkudata(L, index, TS_META_QUERY.as_ptr()) as *mut *mut TSQuery;
        luaL_argcheck(L, !(*ud).is_null(), index, c"TSQuery expected".as_ptr());
        *ud
    }
}

unsafe extern "C-unwind" fn query_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
        ts_query_delete(query);
        0 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn query_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        lua_pushstring(L, c"<query>".as_ptr());
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn query_inspect(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
        lua_createtable(L, 0 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
        let mut n_pat: uint32_t = ts_query_pattern_count(query);
        lua_createtable(L, n_pat as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        let mut i: size_t = 0 as size_t;
        while i < n_pat as size_t {
            let mut len: uint32_t = 0;
            let mut step: *const TSQueryPredicateStep =
                ts_query_predicates_for_pattern(query, i as uint32_t, &raw mut len);
            if len != 0 as uint32_t {
                lua_createtable(
                    L,
                    len as ::core::ffi::c_int / 4 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                lua_createtable(L, 3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                let mut nextpred: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                let mut nextitem: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                let mut k: size_t = 0 as size_t;
                while k < len as size_t {
                    if (*step.add(k)).type_0 as ::core::ffi::c_uint
                        == TSQueryPredicateStepTypeDone as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        lua_rawseti(L, -2 as ::core::ffi::c_int, nextpred);
                        nextpred += 1;
                        lua_createtable(L, 3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                        nextitem = 1 as ::core::ffi::c_int;
                    } else {
                        if (*step.add(k)).type_0 as ::core::ffi::c_uint
                            == TSQueryPredicateStepTypeString as ::core::ffi::c_int
                                as ::core::ffi::c_uint
                        {
                            let mut strlen_0: uint32_t = 0;
                            let mut str: *const ::core::ffi::c_char = ts_query_string_value_for_id(
                                query,
                                (*step.add(k)).value_id,
                                &raw mut strlen_0,
                            );
                            lua_pushlstring(L, str, strlen_0 as size_t);
                        } else if (*step.add(k)).type_0 as ::core::ffi::c_uint
                            == TSQueryPredicateStepTypeCapture as ::core::ffi::c_int
                                as ::core::ffi::c_uint
                        {
                            lua_pushinteger(
                                L,
                                (*step.add(k)).value_id.wrapping_add(1 as uint32_t) as lua_Integer,
                            );
                        } else {
                            abort();
                        }
                        lua_rawseti(L, -2 as ::core::ffi::c_int, nextitem);
                        nextitem += 1;
                    }
                    k = k.wrapping_add(1);
                }
                lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                lua_rawseti(
                    L,
                    -2 as ::core::ffi::c_int,
                    i as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                );
            }
            i = i.wrapping_add(1);
        }
        lua_setfield(L, -2 as ::core::ffi::c_int, c"patterns".as_ptr());
        let mut n_captures: uint32_t = ts_query_capture_count(query);
        lua_createtable(L, n_captures as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        let mut i_0: size_t = 0 as size_t;
        while i_0 < n_captures as size_t {
            let mut strlen_1: uint32_t = 0;
            let mut str_0: *const ::core::ffi::c_char =
                ts_query_capture_name_for_id(query, i_0 as uint32_t, &raw mut strlen_1);
            lua_pushlstring(L, str_0, strlen_1 as size_t);
            lua_rawseti(
                L,
                -2 as ::core::ffi::c_int,
                i_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            );
            i_0 = i_0.wrapping_add(1);
        }
        lua_setfield(L, -2 as ::core::ffi::c_int, c"captures".as_ptr());
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn query_disable_capture(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
        let mut name_len: size_t = 0;
        let mut name: *const ::core::ffi::c_char =
            luaL_checklstring(L, 2 as ::core::ffi::c_int, &raw mut name_len);
        ts_query_disable_capture(query, name, name_len as uint32_t);
        0 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn query_disable_pattern(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
        let pattern_index: uint32_t = luaL_checkinteger(L, 2 as ::core::ffi::c_int) as uint32_t;
        ts_query_disable_pattern(query, pattern_index.wrapping_sub(1 as uint32_t));
        0 as ::core::ffi::c_int
    }
}
