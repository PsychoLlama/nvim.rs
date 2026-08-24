//! The parser userdatum and one parse.
//!
//! `tslua_push_parser` creates the userdatum whose metatable is
//! [`parser_meta`]; `parser_parse` is the parse itself -- it reads the
//! source either from a Lua string or through `input_cb` against a buffer,
//! runs it under `on_parser_progress` so a long parse can be cancelled, and
//! pushes the resulting [`TSLuaTree`].

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::global_cell::ConstTable;
use crate::luaL_reg_table;
use crate::types::NUL;

pub(crate) static parser_meta: ConstTable<[luaL_Reg; 9]> = luaL_reg_table![
    c"__gc" => parser_gc,
    c"__tostring" => parser_tostring,
    c"parse" => parser_parse,
    c"reset" => parser_reset,
    c"set_included_ranges" => parser_set_ranges,
    c"included_ranges" => parser_get_ranges,
    c"_set_logger" => parser_set_logger,
    c"_logger" => parser_get_logger,
];

pub(crate) unsafe extern "C-unwind" fn tslua_push_parser(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lang: *mut TSLanguage = lang_check(L, 1 as ::core::ffi::c_int);
        let mut parser: *mut *mut TSParser =
            lua_newuserdata(L, ::core::mem::size_of::<*mut TSParser>()) as *mut *mut TSParser;
        *parser = ts_parser_new();
        if !ts_parser_set_language(*parser, lang) {
            ts_parser_delete(*parser);
            let mut lang_name: *const ::core::ffi::c_char = luaL_checklstring(
                L,
                1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            );
            return luaL_error(L, c"Failed to load language : %s".as_ptr(), lang_name);
        }
        lua_getfield(L, LUA_REGISTRYINDEX, TS_META_PARSER.as_ptr());
        lua_setmetatable(L, -2 as ::core::ffi::c_int);
        1 as ::core::ffi::c_int
    }
}

pub(crate) unsafe fn parser_check(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut TSParser {
    unsafe {
        let mut ud: *mut *mut TSParser =
            luaL_checkudata(L, index, TS_META_PARSER.as_ptr()) as *mut *mut TSParser;
        luaL_argcheck(
            L,
            !(*ud).is_null(),
            index,
            c"Parser has been deleted".as_ptr(),
        );
        *ud
    }
}

unsafe extern "C-unwind" fn parser_gc(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut ud: *mut *mut TSParser =
            luaL_checkudata(L, 1 as ::core::ffi::c_int, TS_META_PARSER.as_ptr())
                as *mut *mut TSParser;
        if !(*ud).is_null() {
            logger_gc(ts_parser_logger(*ud));
            ts_parser_delete(*ud);
            *ud = ::core::ptr::null_mut::<TSParser>();
        }
        0 as ::core::ffi::c_int
    }
}

unsafe extern "C-unwind" fn parser_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        lua_pushstring(L, c"<parser>".as_ptr());
        1 as ::core::ffi::c_int
    }
}

unsafe extern "C" fn input_cb(
    mut payload: *mut ::core::ffi::c_void,
    mut _byte_index: uint32_t,
    mut position: TSPoint,
    mut bytes_read: *mut uint32_t,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut bp: *mut buf_T = payload as *mut buf_T;
        static buf: GlobalCell<[::core::ffi::c_char; 256]> = GlobalCell::new([0; 256]);
        if position.row as linenr_T >= (*bp).b_ml.ml_line_count {
            *bytes_read = 0 as uint32_t;
            return c"".as_ptr();
        }
        let mut lnum: linenr_T = position.row as linenr_T + 1 as linenr_T;
        let mut line: *mut ::core::ffi::c_char = ml_get_buf(bp, lnum);
        let mut len: size_t = ml_get_buf_len(bp, lnum) as size_t;
        if position.column as size_t > len {
            *bytes_read = 0 as uint32_t;
            return c"".as_ptr();
        }
        let mut tocopy: size_t = if len.wrapping_sub(position.column as size_t) < 256 as size_t {
            len.wrapping_sub(position.column as size_t)
        } else {
            256 as size_t
        };
        memcpy(
            buf.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            line.add(position.column as usize) as *const ::core::ffi::c_void,
            tocopy,
        );
        memchrsub(
            buf.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            '\n' as ::core::ffi::c_char,
            NUL as ::core::ffi::c_char,
            tocopy,
        );
        *bytes_read = tocopy as uint32_t;
        if tocopy < BUFSIZE as size_t
            && (lnum != (*bp).b_ml.ml_line_count
                || (*bp).b_p_bin == 0 && (*bp).b_p_fixeol != 0
                || lnum != (*bp).b_no_eol_lnum && (*bp).b_p_eol != 0)
        {
            (*buf.ptr())[tocopy as usize] = '\n' as ::core::ffi::c_char;
            *bytes_read = (*bytes_read).wrapping_add(1);
        }
        buf.ptr() as *mut ::core::ffi::c_char
    }
}

pub const BUFSIZE: ::core::ffi::c_int = 256 as ::core::ffi::c_int;

unsafe extern "C" fn on_parser_progress(mut state: *mut TSParseState) -> bool {
    unsafe {
        let mut payload: *mut TSLuaParserCallbackPayload =
            (*state).payload as *mut TSLuaParserCallbackPayload;
        let mut parse_time: uint64_t = os_hrtime().wrapping_sub((*payload).parse_start_time);
        parse_time >= (*payload).timeout_threshold_ns
    }
}

unsafe extern "C-unwind" fn parser_parse(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
        let mut old_tree: *const TSTree = ::core::ptr::null::<TSTree>();
        if !(lua_type(L, 2 as ::core::ffi::c_int) == LUA_TNIL) {
            let mut ud: *mut TSLuaTree =
                luaL_checkudata(L, 2 as ::core::ffi::c_int, TS_META_TREE.as_ptr())
                    as *mut TSLuaTree;
            old_tree = if !ud.is_null() {
                (*ud).tree
            } else {
                ::core::ptr::null::<TSTree>()
            };
        }
        let mut new_tree: *mut TSTree = ::core::ptr::null_mut::<TSTree>();
        let mut len: size_t = 0;
        let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut bufnr: handle_T = 0;
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut input: TSInput = TSInput {
            payload: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            read: None,
            encoding: TSInputEncodingUTF8,
            decode: None,
        };
        match lua_type(L, 3 as ::core::ffi::c_int) {
            LUA_TSTRING => {
                str = lua_tolstring(L, 3 as ::core::ffi::c_int, &raw mut len);
                new_tree = ts_parser_parse_string(p, old_tree, str, len as uint32_t);
            }
            LUA_TNUMBER => {
                bufnr = lua_tointeger(L, 3 as ::core::ffi::c_int) as handle_T;
                buf = map_get_int_ptr_t(buffer_handles.ptr(), bufnr as ::core::ffi::c_int)
                    as *mut buf_T;
                if buf.is_null() {
                    let mut ebuf: [::core::ffi::c_char; 256] = [0; 256];
                    vim_snprintf(
                        &raw mut ebuf as *mut ::core::ffi::c_char,
                        BUFSIZE_0 as size_t,
                        c"invalid buffer handle: %d".as_ptr(),
                        bufnr,
                    );
                    return luaL_argerror(
                        L,
                        3 as ::core::ffi::c_int,
                        &raw mut ebuf as *mut ::core::ffi::c_char,
                    );
                }
                input = TSInput {
                    payload: buf as *mut ::core::ffi::c_void,
                    read: Some(
                        input_cb
                            as unsafe extern "C" fn(
                                *mut ::core::ffi::c_void,
                                uint32_t,
                                TSPoint,
                                *mut uint32_t,
                            )
                                -> *const ::core::ffi::c_char,
                    ),
                    encoding: TSInputEncodingUTF8,
                    decode: None,
                };
                if !(lua_type(L, 5 as ::core::ffi::c_int) == LUA_TNIL) {
                    let mut timeout_ns: uint64_t =
                        lua_tointeger(L, 5 as ::core::ffi::c_int) as uint64_t;
                    let mut payload: TSLuaParserCallbackPayload = TSLuaParserCallbackPayload {
                        parse_start_time: os_hrtime(),
                        timeout_threshold_ns: timeout_ns,
                    };
                    let mut parse_options: TSParseOptions = TSParseOptions {
                        payload: &raw mut payload as *mut ::core::ffi::c_void,
                        progress_callback: Some(
                            on_parser_progress as unsafe extern "C" fn(*mut TSParseState) -> bool,
                        ),
                    };
                    new_tree = ts_parser_parse_with_options(p, old_tree, input, parse_options);
                } else {
                    new_tree = ts_parser_parse(p, old_tree, input);
                }
            }
            _ => {
                return luaL_argerror(
                    L,
                    3 as ::core::ffi::c_int,
                    c"expected either string or buffer handle".as_ptr(),
                );
            }
        }
        let mut include_bytes: bool = lua_gettop(L) >= 4 as ::core::ffi::c_int
            && lua_toboolean(L, 4 as ::core::ffi::c_int) != 0;
        if new_tree.is_null() {
            if ts_parser_language(p).is_null() {
                return luaL_error(
                    L,
                    c"Language was unset, or has an incompatible ABI.".as_ptr(),
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        let mut n_ranges: uint32_t = 0 as uint32_t;
        let mut changed: *mut TSRange = if !old_tree.is_null() {
            ts_tree_get_changed_ranges(old_tree, new_tree, &raw mut n_ranges)
        } else {
            ts_tree_included_ranges(new_tree, &raw mut n_ranges)
        };
        push_tree(L, new_tree);
        push_ranges(L, changed, n_ranges as size_t, include_bytes);
        xfree(changed as *mut ::core::ffi::c_void);
        2 as ::core::ffi::c_int
    }
}

pub const BUFSIZE_0: ::core::ffi::c_int = 256 as ::core::ffi::c_int;

unsafe extern "C-unwind" fn parser_reset(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut TSParser = parser_check(L, 1 as ::core::ffi::c_int);
        ts_parser_reset(p);
        0 as ::core::ffi::c_int
    }
}
