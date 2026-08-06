//! Compiling a query, and inspecting or disabling parts of it.
//!
//! `tslua_parse_query` compiles the query text against a loaded language and
//! turns a `TSQueryError` into the pointed message `query_err_string`
//! builds (offset, line, and the offending word).  `query_inspect` reports a
//! compiled query's patterns, captures and predicates back to Lua.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) static query_meta: GlobalCell<[luaL_Reg; 6]> = GlobalCell::new([
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(query_gc as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            query_tostring as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"inspect\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            query_inspect as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"disable_capture\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            query_disable_capture
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"disable_pattern\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            query_disable_pattern
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);

pub(crate) unsafe extern "C-unwind" fn tslua_parse_query(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        if lua_gettop(L) < 2 as ::core::ffi::c_int
            || lua_isstring(L, 1 as ::core::ffi::c_int) == 0
            || lua_isstring(L, 2 as ::core::ffi::c_int) == 0
        {
            return luaL_error(
                L,
                b"string expected\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut lang: *mut TSLanguage = lang_check(L, 1 as ::core::ffi::c_int);
        let mut len: size_t = 0;
        let mut src: *const ::core::ffi::c_char =
            lua_tolstring(L, 2 as ::core::ffi::c_int, &raw mut len);
        tslua_query_parse_count.set((*tslua_query_parse_count.ptr()).wrapping_add(1));
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
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut err_msg as *mut ::core::ffi::c_char,
            );
        }
        let mut ud: *mut *mut TSQuery =
            lua_newuserdata(L, ::core::mem::size_of::<*mut TSQuery>()) as *mut *mut TSQuery;
        *ud = query;
        lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERY.as_ptr());
        lua_setmetatable(L, -2 as ::core::ffi::c_int);
        return 1 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn query_err_to_string(
    mut error_type: TSQueryError,
) -> *const ::core::ffi::c_char {
    match error_type as ::core::ffi::c_uint {
        1 => return b"Invalid syntax:\n\0".as_ptr() as *const ::core::ffi::c_char,
        2 => return b"Invalid node type \0".as_ptr() as *const ::core::ffi::c_char,
        3 => return b"Invalid field name \0".as_ptr() as *const ::core::ffi::c_char,
        4 => return b"Invalid capture name \0".as_ptr() as *const ::core::ffi::c_char,
        5 => return b"Impossible pattern:\n\0".as_ptr() as *const ::core::ffi::c_char,
        _ => return b"error\0".as_ptr() as *const ::core::ffi::c_char,
    };
}

unsafe extern "C-unwind" fn query_err_string(
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
            let mut src_tmp: *const ::core::ffi::c_char = src.offset(line_start as isize);
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
            b"Query error at %d:%d. %s\0".as_ptr() as *const ::core::ffi::c_char,
            row + 1 as ::core::ffi::c_int,
            column + 1 as ::core::ffi::c_int,
            type_msg,
        );
        let mut offset: size_t = strlen(err);
        errlen = errlen.wrapping_sub(offset);
        err = err.offset(offset as isize);
        if error_type as ::core::ffi::c_uint
            == TSQueryErrorNodeType as ::core::ffi::c_int as ::core::ffi::c_uint
            || error_type as ::core::ffi::c_uint
                == TSQueryErrorField as ::core::ffi::c_int as ::core::ffi::c_uint
            || error_type as ::core::ffi::c_uint
                == TSQueryErrorCapture as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut suffix: *const ::core::ffi::c_char = src.offset(error_offset as isize);
            let mut is_anonymous: bool = error_type as ::core::ffi::c_uint
                == TSQueryErrorNodeType as ::core::ffi::c_int as ::core::ffi::c_uint
                && *suffix.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int;
            let mut suffix_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut c: ::core::ffi::c_char = *suffix.offset(suffix_len as isize);
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
                    c = *suffix.offset(suffix_len as isize);
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
                    c = *suffix.offset(suffix_len as isize);
                }
            }
            snprintf(
                err,
                errlen,
                b"\"%.*s\":\n\0".as_ptr() as *const ::core::ffi::c_char,
                suffix_len,
                suffix,
            );
            offset = strlen(err);
            errlen = errlen.wrapping_sub(offset);
            err = err.offset(offset as isize);
        }
        if error_line.is_null() {
            snprintf(
                err,
                errlen,
                b"Unexpected EOF\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        snprintf(
            err,
            errlen,
            b"%.*s\n%*s^\n\0".as_ptr() as *const ::core::ffi::c_char,
            error_line_len,
            error_line,
            column,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}

pub(crate) unsafe extern "C-unwind" fn query_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSQuery {
    unsafe {
        let mut ud: *mut *mut TSQuery =
            luaL_checkudata(L, index, TS_META_QUERY.as_ptr()) as *mut *mut TSQuery;
        (!(*ud).is_null()
            || luaL_argerror(
                L,
                index,
                b"TSQuery expected\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0) as ::core::ffi::c_int;
        return *ud;
    }
}

unsafe extern "C-unwind" fn query_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
        ts_query_delete(query);
        return 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn query_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        lua_pushstring(L, b"<query>\0".as_ptr() as *const ::core::ffi::c_char);
        return 1 as ::core::ffi::c_int;
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
                    if (*step.offset(k as isize)).type_0 as ::core::ffi::c_uint
                        == TSQueryPredicateStepTypeDone as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let c2rust_fresh0 = nextpred;
                        nextpred = nextpred + 1;
                        lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh0);
                        lua_createtable(L, 3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                        nextitem = 1 as ::core::ffi::c_int;
                    } else {
                        if (*step.offset(k as isize)).type_0 as ::core::ffi::c_uint
                            == TSQueryPredicateStepTypeString as ::core::ffi::c_int
                                as ::core::ffi::c_uint
                        {
                            let mut strlen_0: uint32_t = 0;
                            let mut str: *const ::core::ffi::c_char = ts_query_string_value_for_id(
                                query,
                                (*step.offset(k as isize)).value_id,
                                &raw mut strlen_0,
                            );
                            lua_pushlstring(L, str, strlen_0 as size_t);
                        } else if (*step.offset(k as isize)).type_0 as ::core::ffi::c_uint
                            == TSQueryPredicateStepTypeCapture as ::core::ffi::c_int
                                as ::core::ffi::c_uint
                        {
                            lua_pushinteger(
                                L,
                                (*step.offset(k as isize))
                                    .value_id
                                    .wrapping_add(1 as uint32_t)
                                    as lua_Integer,
                            );
                        } else {
                            abort();
                        }
                        let c2rust_fresh1 = nextitem;
                        nextitem = nextitem + 1;
                        lua_rawseti(L, -2 as ::core::ffi::c_int, c2rust_fresh1);
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
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"patterns\0".as_ptr() as *const ::core::ffi::c_char,
        );
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
        lua_setfield(
            L,
            -2 as ::core::ffi::c_int,
            b"captures\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 1 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn query_disable_capture(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
        let mut name_len: size_t = 0;
        let mut name: *const ::core::ffi::c_char =
            luaL_checklstring(L, 2 as ::core::ffi::c_int, &raw mut name_len);
        ts_query_disable_capture(query, name, name_len as uint32_t);
        return 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn query_disable_pattern(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut query: *mut TSQuery = query_check(L, 1 as ::core::ffi::c_int);
        let pattern_index: uint32_t = luaL_checkinteger(L, 2 as ::core::ffi::c_int) as uint32_t;
        ts_query_disable_pattern(query, pattern_index.wrapping_sub(1 as uint32_t));
        return 0 as ::core::ffi::c_int;
    }
}
