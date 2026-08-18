//! The string helpers: UTF-8 indexing, comparison and `iconv`.
//!
//! [`nlua_str_utfindex`]/[`nlua_str_byteindex`] convert between byte and
//! character offsets, [`nlua_str_utf_pos`]/[`nlua_str_utf_start`]/
//! [`nlua_str_utf_end`] report codepoint boundaries, [`nlua_stricmp`] is the
//! case-insensitive compare the Lua side cannot express, and [`nlua_iconv`]
//! is `vim.iconv()`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::lua::ffi::{
    LUA_TSTRING, lua_createtable, lua_gettop, lua_isnoneornil, lua_pop, lua_pushinteger,
    lua_pushlstring, lua_pushnil, lua_pushnumber, lua_rawseti, lua_toboolean, lua_tolstring,
    lua_tostring, lua_type, luaL_argerror, luaL_checkinteger, luaL_checklstring, luaL_error,
};
use crate::mbyte::{
    convert_setup, convert_setup_ext, enc_canonize, enc_skip, mb_utf_index_to_bytes, mb_utflen,
    string_convert, utf_cp_bounds_len, utf_ptr2len_len,
};
use crate::memory::xfree;
use crate::types::{CONV_NONE, intptr_t, lua_Integer, lua_Number, lua_State, size_t, vimconv_T};
use ::libc::{memchr, strcasecmp};

/// `vim.str_utfindex()`: the UTF-32 and UTF-16 lengths of the string at slot 1
/// up to the byte index at slot 2, or of the whole string when it is absent.
///
/// Two nils for an index outside the string.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub unsafe extern "C-unwind" fn nlua_str_utfindex(lstate: *mut lua_State) -> c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let s1 = luaL_checklstring(lstate, 1, &raw mut s1_len);
        let idx: intptr_t = if lua_isnoneornil(lstate, 2) {
            s1_len as intptr_t
        } else {
            let idx = luaL_checkinteger(lstate, 2) as intptr_t;
            if idx < 0 || idx > s1_len as intptr_t {
                lua_pushnil(lstate);
                lua_pushnil(lstate);
                return 2;
            }
            idx
        };

        let mut codepoints: size_t = 0;
        let mut codeunits: size_t = 0;
        mb_utflen(s1, idx as size_t, &raw mut codepoints, &raw mut codeunits);

        lua_pushinteger(lstate, codepoints as lua_Integer);
        lua_pushinteger(lstate, codeunits as lua_Integer);
        2
    }
}

/// `vim.str_utf_pos()`: the 1-based byte index of every codepoint in the
/// string at slot 1, as a list.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_str_utf_pos(lstate: *mut lua_State) -> c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let s1 = luaL_checklstring(lstate, 1, &raw mut s1_len);
        lua_createtable(lstate, 0, 0);

        let mut idx: size_t = 1;
        let mut i: size_t = 0;
        while i < s1_len && *s1.add(i) != 0 {
            let clen = utf_ptr2len_len(s1.add(i), s1_len.wrapping_sub(i) as c_int) as size_t;
            lua_pushinteger(lstate, i as lua_Integer + 1);
            lua_rawseti(lstate, -2, idx as c_int);
            idx = idx.wrapping_add(1);
            i = i.wrapping_add(clen);
        }
        1
    }
}

/// `vim.str_utf_start()`: how far *back* from the 1-based byte offset at slot
/// 2 the codepoint containing it begins, as a non-positive number.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_str_utf_start(lstate: *mut lua_State) -> c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let s1 = luaL_checklstring(lstate, 1, &raw mut s1_len);
        let offset = luaL_checkinteger(lstate, 2);
        if offset <= 0 || offset > s1_len as intptr_t {
            return luaL_error(lstate, c"index out of range".as_ptr());
        }
        let off = (offset - 1) as size_t;
        let bounds = utf_cp_bounds_len(s1, s1.add(off), s1_len.wrapping_sub(off) as c_int);
        lua_pushinteger(lstate, -(bounds.begin_off as lua_Integer));
        1
    }
}

/// `vim.str_utf_end()`: how far *on* from the 1-based byte offset at slot 2
/// the codepoint containing it ends.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_str_utf_end(lstate: *mut lua_State) -> c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let s1 = luaL_checklstring(lstate, 1, &raw mut s1_len);
        let offset = luaL_checkinteger(lstate, 2);
        if offset <= 0 || offset > s1_len as intptr_t {
            return luaL_error(lstate, c"index out of range".as_ptr());
        }
        let off = (offset - 1) as size_t;
        let bounds = utf_cp_bounds_len(s1, s1.add(off), s1_len.wrapping_sub(off) as c_int);
        lua_pushinteger(lstate, bounds.end_off as lua_Integer - 1);
        1
    }
}

/// `vim.str_byteindex()`: the byte offset of the UTF-32 (or, with a truthy
/// slot 3, UTF-16) index at slot 2, or nil when the string is shorter.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub unsafe extern "C-unwind" fn nlua_str_byteindex(lstate: *mut lua_State) -> c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let s1 = luaL_checklstring(lstate, 1, &raw mut s1_len);
        let idx = luaL_checkinteger(lstate, 2) as intptr_t;
        if idx < 0 {
            lua_pushnil(lstate);
            return 1;
        }
        let use_utf16 = lua_gettop(lstate) >= 3 && lua_toboolean(lstate, 3) != 0;

        let byteidx = mb_utf_index_to_bytes(s1, s1_len, idx as size_t, use_utf16);
        if byteidx == -1 {
            lua_pushnil(lstate);
            return 1;
        }
        lua_pushinteger(lstate, byteidx as lua_Integer);
        1
    }
}

/// `vim.stricmp()`: -1, 0 or 1 for the two strings at slots 1 and 2, ignoring
/// case.
///
/// A Lua string may contain NULs and `strcasecmp` stops at the first one, so
/// the comparison runs NUL-separated run by NUL-separated run; a string that
/// has one where the other has ended sorts after it. Uppercasing can change a
/// byte's length, so the two sides cannot be advanced by a common amount.
///
/// Does no error handling: never call it with a non-string or with an
/// argument missing.
///
/// # Safety
/// `lstate` must be a live Lua state holding two strings.
pub(crate) unsafe extern "C-unwind" fn nlua_stricmp(lstate: *mut lua_State) -> c_int {
    unsafe {
        let mut s1_len: size_t = 0;
        let mut s2_len: size_t = 0;
        let mut s1 = luaL_checklstring(lstate, 1, &raw mut s1_len);
        let mut s2 = luaL_checklstring(lstate, 2, &raw mut s2_len);
        debug_assert!(*s1.add(s1_len) == 0);
        debug_assert!(*s2.add(s2_len) == 0);

        let mut ret;
        loop {
            let nul1 = memchr(s1.cast::<c_void>(), 0, s1_len).cast::<c_char>();
            let nul2 = memchr(s2.cast::<c_void>(), 0, s2_len).cast::<c_char>();
            ret = strcasecmp(s1.cast_mut(), s2.cast_mut());
            if ret != 0 {
                break;
            }
            // Compare "a\0" greater than "a".
            if nul1.is_null() != nul2.is_null() {
                ret = !nul1.is_null() as c_int - !nul2.is_null() as c_int;
                break;
            }
            if nul1.is_null() {
                break;
            }
            debug_assert!(!nul2.is_null());
            s1_len = s1_len.wrapping_sub((nul1.offset_from(s1) as size_t).wrapping_add(1));
            s2_len = s2_len.wrapping_sub((nul2.offset_from(s2) as size_t).wrapping_add(1));
            s1 = nul1.add(1);
            s2 = nul2.add(1);
        }

        lua_pop(lstate, 2);
        lua_pushnumber(
            lstate,
            ((ret > 0) as c_int - (ret < 0) as c_int) as lua_Number,
        );
        1
    }
}

/// `vim.iconv()`: the string at slot 1 recoded from the encoding at slot 2 to
/// the one at slot 3, or nil when the conversion fails.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_iconv(lstate: *mut lua_State) -> c_int {
    unsafe {
        if lua_gettop(lstate) < 3 {
            return luaL_error(lstate, c"Expected at least 3 arguments".as_ptr());
        }
        for i in 1..=3 {
            if lua_type(lstate, i) != LUA_TSTRING {
                return luaL_argerror(lstate, i, c"expected string".as_ptr());
            }
        }

        let mut str_len: size_t = 0;
        let str = lua_tolstring(lstate, 1, &raw mut str_len);

        let from = enc_canonize(enc_skip(lua_tostring(lstate, 2).cast_mut()));
        let to = enc_canonize(enc_skip(lua_tostring(lstate, 3).cast_mut()));

        let mut vimconv = vimconv_T {
            vc_type: CONV_NONE,
            vc_factor: 0,
            vc_fd: ptr::null_mut(),
            vc_fail: false,
        };
        convert_setup_ext(&raw mut vimconv, from, false, to, false);

        let ret = string_convert(&raw mut vimconv, str.cast_mut(), &raw mut str_len);

        convert_setup(&raw mut vimconv, ptr::null_mut(), ptr::null_mut());

        xfree(from.cast::<c_void>());
        xfree(to.cast::<c_void>());

        if ret.is_null() {
            lua_pushnil(lstate);
        } else {
            lua_pushlstring(lstate, ret, str_len);
            xfree(ret.cast::<c_void>());
        }
        1
    }
}
