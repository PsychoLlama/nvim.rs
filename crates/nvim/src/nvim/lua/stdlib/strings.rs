//! The string helpers: UTF-8 indexing, comparison and `iconv`.
//!
//! `nlua_str_utfindex`/`nlua_str_byteindex` convert between byte and
//! character offsets, `nlua_str_utf_pos`/`_start`/`_end` report codepoint
//! boundaries, `nlua_stricmp` is the case-insensitive compare the Lua side
//! cannot express, and `nlua_iconv` is `vim.iconv()`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C-unwind" fn nlua_str_utfindex(lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let mut s1: *const ::core::ffi::c_char =
            luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
        let mut idx: intptr_t = 0;
        if lua_type(lstate, 2 as ::core::ffi::c_int) <= 0 as ::core::ffi::c_int {
            idx = s1_len as intptr_t;
        } else {
            idx = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as intptr_t;
            if idx < 0 as intptr_t || idx > s1_len as intptr_t {
                lua_pushnil(lstate);
                lua_pushnil(lstate);
                return 2 as ::core::ffi::c_int;
            }
        }
        let mut codepoints: size_t = 0 as size_t;
        let mut codeunits: size_t = 0 as size_t;
        mb_utflen(s1, idx as size_t, &raw mut codepoints, &raw mut codeunits);
        lua_pushinteger(lstate, codepoints as lua_Integer);
        lua_pushinteger(lstate, codeunits as lua_Integer);
        return 2 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_str_utf_pos(
    lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let mut s1: *const ::core::ffi::c_char =
            luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
        lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        let mut idx: size_t = 1 as size_t;
        let mut clen: size_t = 0;
        let mut i: size_t = 0 as size_t;
        while i < s1_len && *s1.offset(i as isize) as ::core::ffi::c_int != NUL {
            clen = utf_ptr2len_len(
                s1.offset(i as isize),
                s1_len.wrapping_sub(i) as ::core::ffi::c_int,
            ) as size_t;
            lua_pushinteger(lstate, i as lua_Integer + 1 as lua_Integer);
            lua_rawseti(lstate, -2 as ::core::ffi::c_int, idx as ::core::ffi::c_int);
            idx = idx.wrapping_add(1);
            i = i.wrapping_add(clen);
        }
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_str_utf_start(
    lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let mut s1: *const ::core::ffi::c_char =
            luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
        let mut offset: ptrdiff_t = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int);
        if offset <= 0 as ptrdiff_t || offset > s1_len as ptrdiff_t {
            return luaL_error(
                lstate,
                b"index out of range\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let off: size_t = (offset - 1 as ptrdiff_t) as size_t;
        let mut head_off: ::core::ffi::c_int = -(utf_cp_bounds_len(
            s1,
            s1.offset(off as isize),
            s1_len.wrapping_sub(off) as ::core::ffi::c_int,
        )
        .begin_off as ::core::ffi::c_int);
        lua_pushinteger(lstate, head_off as lua_Integer);
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_str_utf_end(
    lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let mut s1: *const ::core::ffi::c_char =
            luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
        let mut offset: ptrdiff_t = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int);
        if offset <= 0 as ptrdiff_t || offset > s1_len as ptrdiff_t {
            return luaL_error(
                lstate,
                b"index out of range\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let off: size_t = (offset - 1 as ptrdiff_t) as size_t;
        let mut tail_off: ::core::ffi::c_int = utf_cp_bounds_len(
            s1,
            s1.offset(off as isize),
            s1_len.wrapping_sub(off) as ::core::ffi::c_int,
        )
        .end_off as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int;
        lua_pushinteger(lstate, tail_off as lua_Integer);
        return 1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C-unwind" fn nlua_str_byteindex(lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let mut s1: *const ::core::ffi::c_char =
            luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
        let mut idx: intptr_t = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as intptr_t;
        if idx < 0 as intptr_t {
            lua_pushnil(lstate);
            return 1 as ::core::ffi::c_int;
        }
        let mut use_utf16: bool = false_0 != 0;
        if lua_gettop(lstate) >= 3 as ::core::ffi::c_int {
            use_utf16 = lua_toboolean(lstate, 3 as ::core::ffi::c_int) != 0;
        }
        let mut byteidx: ssize_t = mb_utf_index_to_bytes(s1, s1_len, idx as size_t, use_utf16);
        if byteidx == -1 as ssize_t {
            lua_pushnil(lstate);
            return 1 as ::core::ffi::c_int;
        }
        lua_pushinteger(lstate, byteidx as lua_Integer);
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_stricmp(lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let mut s2_len: size_t = 0;
        let mut s1: *const ::core::ffi::c_char =
            luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut s1_len);
        let mut s2: *const ::core::ffi::c_char =
            luaL_checklstring(lstate, 2 as ::core::ffi::c_int, &raw mut s2_len);
        let mut nul1: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut nul2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        '_c2rust_label: {
            if *s1.offset(s1_len as isize) as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"s1[s1_len] == NUL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/stdlib.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    481 as ::core::ffi::c_uint,
                    b"int nlua_stricmp(lua_State *const)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        '_c2rust_label_0: {
            if *s2.offset(s2_len as isize) as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"s2[s2_len] == NUL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/stdlib.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    482 as ::core::ffi::c_uint,
                    b"int nlua_stricmp(lua_State *const)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        loop {
            nul1 =
                memchr(s1 as *const ::core::ffi::c_void, NUL, s1_len) as *const ::core::ffi::c_char;
            nul2 =
                memchr(s2 as *const ::core::ffi::c_void, NUL, s2_len) as *const ::core::ffi::c_char;
            ret = strcasecmp(
                s1 as *mut ::core::ffi::c_char,
                s2 as *mut ::core::ffi::c_char,
            );
            if ret != 0 as ::core::ffi::c_int {
                break;
            }
            if nul1.is_null() as ::core::ffi::c_int != nul2.is_null() as ::core::ffi::c_int {
                ret = !nul1.is_null() as ::core::ffi::c_int - !nul2.is_null() as ::core::ffi::c_int;
                break;
            } else {
                if nul1.is_null() {
                    break;
                }
                '_c2rust_label_1: {
                    if !nul2.is_null() {
                    } else {
                        __assert_fail(
                            b"nul2 != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/lua/stdlib.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            494 as ::core::ffi::c_uint,
                            b"int nlua_stricmp(lua_State *const)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                s1_len =
                    s1_len.wrapping_sub((nul1.offset_from(s1) as size_t).wrapping_add(1 as size_t));
                s2_len =
                    s2_len.wrapping_sub((nul2.offset_from(s2) as size_t).wrapping_add(1 as size_t));
                s1 = nul1.offset(1 as ::core::ffi::c_int as isize);
                s2 = nul2.offset(1 as ::core::ffi::c_int as isize);
            }
        }
        lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_pushnumber(
            lstate,
            ((ret > 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                - (ret < 0 as ::core::ffi::c_int) as ::core::ffi::c_int) as lua_Number,
        );
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_iconv(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let mut narg: ::core::ffi::c_int = lua_gettop(lstate);
        if narg < 3 as ::core::ffi::c_int {
            return luaL_error(
                lstate,
                b"Expected at least 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i <= 3 as ::core::ffi::c_int {
            if lua_type(lstate, i) != LUA_TSTRING {
                return luaL_argerror(
                    lstate,
                    i,
                    b"expected string\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            i += 1;
        }
        let mut str_len: size_t = 0 as size_t;
        let mut str: *const ::core::ffi::c_char =
            lua_tolstring(lstate, 1 as ::core::ffi::c_int, &raw mut str_len);
        let mut from: *mut ::core::ffi::c_char = enc_canonize(enc_skip(lua_tolstring(
            lstate,
            2 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        )
            as *mut ::core::ffi::c_char));
        let mut to: *mut ::core::ffi::c_char = enc_canonize(enc_skip(lua_tolstring(
            lstate,
            3 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        )
            as *mut ::core::ffi::c_char));
        let mut vimconv: vimconv_T = vimconv_T {
            vc_type: 0,
            vc_factor: 0,
            vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            vc_fail: false,
        };
        vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
        convert_setup_ext(&raw mut vimconv, from, false_0 != 0, to, false_0 != 0);
        let mut ret: *mut ::core::ffi::c_char = string_convert(
            &raw mut vimconv,
            str as *mut ::core::ffi::c_char,
            &raw mut str_len,
        );
        convert_setup(
            &raw mut vimconv,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
        xfree(from as *mut ::core::ffi::c_void);
        xfree(to as *mut ::core::ffi::c_void);
        if ret.is_null() {
            lua_pushnil(lstate);
        } else {
            lua_pushlstring(lstate, ret, str_len);
            xfree(ret as *mut ::core::ffi::c_void);
        }
        return 1 as ::core::ffi::c_int;
    }
}
