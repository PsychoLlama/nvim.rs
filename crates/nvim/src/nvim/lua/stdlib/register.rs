//! Installing all of the above onto a `lua_State`.
//!
//! `nlua_state_add_stdlib` is the one registration point: it pushes every
//! `vim.*` C function this module implements onto the `vim` table, and
//! `nlua_state_add_internal` the `vim._*` half a thread's state also gets.
//! `nlua_push_errstr` is the shared error formatter.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C-unwind" fn nlua_foldupdate(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut window: handle_T = luaL_checkinteger(lstate, 1 as ::core::ffi::c_int) as handle_T;
        let mut win: *mut win_T =
            map_get_int_ptr_t(window_handles.ptr(), window as ::core::ffi::c_int) as *mut win_T;
        if win.is_null() {
            return luaL_error(
                lstate,
                b"invalid window\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut top: linenr_T =
            luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as linenr_T + 1 as linenr_T;
        if top < 1 as linenr_T {
            return luaL_error(
                lstate,
                b"invalid top\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut bot: linenr_T = luaL_checkinteger(lstate, 3 as ::core::ffi::c_int) as linenr_T;
        if top > bot {
            return luaL_error(
                lstate,
                b"invalid bot\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        foldUpdate(win, top, bot);
        return 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn nlua_state_add_internal(lstate: *mut lua_State) {
    unsafe {
        lua_pushcclosure(
            lstate,
            Some(nlua_getvar as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"_getvar\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(nlua_setvar as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"_setvar\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(
                nlua_foldupdate
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"_foldupdate\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushcclosure(
            lstate,
            Some(nlua_with as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"_with_c\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}

pub unsafe extern "C-unwind" fn nlua_state_add_stdlib(lstate: *mut lua_State, mut is_thread: bool) {
    unsafe {
        if !is_thread {
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_stricmp
                        as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"stricmp\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_str_utfindex
                        as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"_str_utfindex\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_str_byteindex
                        as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"_str_byteindex\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_str_utf_pos
                        as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"str_utf_pos\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_str_utf_start
                        as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"str_utf_start\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_str_utf_end
                        as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"str_utf_end\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_regex as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"regex\0".as_ptr() as *const ::core::ffi::c_char,
            );
            luaL_newmetatable(
                lstate,
                b"nvim_regex\0".as_ptr() as *const ::core::ffi::c_char,
            );
            luaL_register(
                lstate,
                ::core::ptr::null::<::core::ffi::c_char>(),
                regex_meta.ptr() as *mut luaL_Reg,
            );
            lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"__index\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            luaopen_spell(lstate);
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"spell\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushcclosure(
                lstate,
                Some(
                    nlua_iconv as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                ),
                0 as ::core::ffi::c_int,
            );
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"iconv\0".as_ptr() as *const ::core::ffi::c_char,
            );
            luaopen_base64(lstate);
            lua_setfield(
                lstate,
                -2 as ::core::ffi::c_int,
                b"base64\0".as_ptr() as *const ::core::ffi::c_char,
            );
            nlua_state_add_internal(lstate);
        }
        luaopen_mpack(lstate);
        lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            -3 as ::core::ffi::c_int,
            b"mpack\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"package\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            -1 as ::core::ffi::c_int,
            b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushvalue(lstate, -3 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"mpack\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_settop(lstate, -3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        unsafe extern "C" {
            #[link_name = "luaopen_lpeg"]
            fn luaopen_lpeg_0(_: *mut lua_State) -> ::core::ffi::c_int;
        }
        luaopen_lpeg_0(lstate);
        lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            -4 as ::core::ffi::c_int,
            b"lpeg\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"package\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(
            lstate,
            -1 as ::core::ffi::c_int,
            b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushvalue(lstate, -3 as ::core::ffi::c_int);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"lpeg\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_settop(lstate, -4 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_pushcclosure(
            lstate,
            Some(
                nlua_xdl_diff as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"diff\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_cjson_new(lstate);
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"json\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}

pub unsafe extern "C-unwind" fn nlua_push_errstr(
    mut L: *mut lua_State,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    unsafe {
        let mut argp: ::core::ffi::VaList;
        argp = c2rust_args.clone();
        luaL_where(L, 1 as ::core::ffi::c_int);
        lua_pushvfstring(L, fmt, argp);
        lua_concat(L, 2 as ::core::ffi::c_int);
    }
}
