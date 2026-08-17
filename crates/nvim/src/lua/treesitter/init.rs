//! Registering the whole `vim._ts_*` surface on a `lua_State`.
//!
//! `nlua_treesitter_init` builds the five metatables through `build_meta`
//! and installs the module table; `tslua_init` is the same for a thread's
//! state.  The two `*_language_version` functions report the ABI range this
//! build accepts.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

unsafe extern "C-unwind" fn build_meta(
    mut L: *mut lua_State,
    mut tname: *const ::core::ffi::c_char,
    mut meta: *const luaL_Reg,
) {
    unsafe {
        if luaL_newmetatable(L, tname) != 0 {
            luaL_register(L, ::core::ptr::null::<::core::ffi::c_char>(), meta);
            lua_pushvalue(L, -1 as ::core::ffi::c_int);
            lua_setfield(L, -2 as ::core::ffi::c_int, c"__index".as_ptr());
        }
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
}

unsafe extern "C-unwind" fn tslua_init(mut L: *mut lua_State) {
    unsafe {
        build_meta(
            L,
            TS_META_PARSER.as_ptr(),
            parser_meta.ptr() as *mut luaL_Reg,
        );
        build_meta(L, TS_META_TREE.as_ptr(), tree_meta.ptr() as *mut luaL_Reg);
        build_meta(L, TS_META_NODE.as_ptr(), node_meta.ptr() as *mut luaL_Reg);
        build_meta(L, TS_META_QUERY.as_ptr(), query_meta.ptr() as *mut luaL_Reg);
        build_meta(
            L,
            TS_META_QUERYCURSOR.as_ptr(),
            querycursor_meta.ptr() as *mut luaL_Reg,
        );
        build_meta(
            L,
            TS_META_QUERYMATCH.as_ptr(),
            querymatch_meta.ptr() as *mut luaL_Reg,
        );
        ts_set_allocator(
            Some(xmalloc as unsafe extern "C" fn(size_t) -> *mut ::core::ffi::c_void),
            Some(xcalloc as unsafe extern "C" fn(size_t, size_t) -> *mut ::core::ffi::c_void),
            Some(
                xrealloc
                    as unsafe extern "C" fn(
                        *mut ::core::ffi::c_void,
                        size_t,
                    ) -> *mut ::core::ffi::c_void,
            ),
            Some(xfree as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        );
    }
}

unsafe extern "C-unwind" fn tslua_get_language_version(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        lua_pushnumber(L, TREE_SITTER_LANGUAGE_VERSION as lua_Number);
        return 1 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn tslua_get_minimum_language_version(
    mut L: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        lua_pushnumber(L, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION as lua_Number);
        return 1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C-unwind" fn nlua_treesitter_free() {}

pub unsafe extern "C-unwind" fn nlua_treesitter_init(lstate: *mut lua_State) {
    unsafe {
        tslua_init(lstate);
        lua_pushcclosure(
            lstate,
            Some(
                tslua_push_parser
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            c"_create_ts_parser".as_ptr(),
        );
        lua_pushcclosure(
            lstate,
            Some(
                tslua_push_querycursor
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            c"_create_ts_querycursor".as_ptr(),
        );
        lua_pushcclosure(
            lstate,
            Some(
                tslua_add_language_from_object
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            c"_ts_add_language_from_object".as_ptr(),
        );
        lua_pushcclosure(
            lstate,
            Some(
                tslua_has_language
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            c"_ts_has_language".as_ptr(),
        );
        lua_pushcclosure(
            lstate,
            Some(
                tslua_remove_lang
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            c"_ts_remove_language".as_ptr(),
        );
        lua_pushcclosure(
            lstate,
            Some(
                tslua_inspect_lang
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            c"_ts_inspect_language".as_ptr(),
        );
        lua_pushcclosure(
            lstate,
            Some(
                tslua_parse_query
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            c"_ts_parse_query".as_ptr(),
        );
        lua_pushcclosure(
            lstate,
            Some(
                tslua_get_language_version
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            c"_ts_get_language_version".as_ptr(),
        );
        lua_pushcclosure(
            lstate,
            Some(
                tslua_get_minimum_language_version
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            c"_ts_get_minimum_language_version".as_ptr(),
        );
    }
}
