//! `vim.\_with()`: run a function with the editor state temporarily swapped.
//!
//! One function, because unwinding it is one linear sequence: every context
//! key (`buf`, `win`, `wo`, `bo`, `emsg_silent`, `hide`, `noautocmd`,
//! `sandbox`, `silent`, `lockmarks`, `env`) is saved, applied, the callback
//! called, and every save restored in reverse -- through the same
//! `try_enter`/`try_leave` bracket the API uses.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C-unwind" fn nlua_with(mut L: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut log_level: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        (lua_type(L, 1 as ::core::ffi::c_int) == 5 as ::core::ffi::c_int
            || luaL_argerror(
                L,
                1 as ::core::ffi::c_int,
                b"table expected\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0) as ::core::ffi::c_int;
        lua_pushnil(L);
        while lua_next(L, 1 as ::core::ffi::c_int) != 0 {
            if lua_type(L, -2 as ::core::ffi::c_int) == LUA_TSTRING {
                let mut k: *const ::core::ffi::c_char = lua_tolstring(
                    L,
                    -2 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<size_t>(),
                );
                let mut v: bool = lua_toboolean(L, -1 as ::core::ffi::c_int) != 0;
                if strequal(b"buf\0".as_ptr() as *const ::core::ffi::c_char, k) {
                    buf = map_get_int_ptr_t(
                        buffer_handles.ptr(),
                        luaL_checkinteger(L, -1 as ::core::ffi::c_int) as ::core::ffi::c_int,
                    ) as *mut buf_T;
                } else if strequal(b"win\0".as_ptr() as *const ::core::ffi::c_char, k) {
                    win = map_get_int_ptr_t(
                        window_handles.ptr(),
                        luaL_checkinteger(L, -1 as ::core::ffi::c_int) as ::core::ffi::c_int,
                    ) as *mut win_T;
                } else if strequal(b"log_level\0".as_ptr() as *const ::core::ffi::c_char, k) {
                    log_level =
                        luaL_checkinteger(L, -1 as ::core::ffi::c_int) as ::core::ffi::c_int;
                } else {
                    if strequal(b"sandbox\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_SANDBOX as ::core::ffi::c_int;
                    }
                    if strequal(b"silent\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_SILENT as ::core::ffi::c_int;
                    }
                    if strequal(b"emsg_silent\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_ERRSILENT as ::core::ffi::c_int;
                    }
                    if strequal(b"unsilent\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_UNSILENT as ::core::ffi::c_int;
                    }
                    if strequal(b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_NOAUTOCMD as ::core::ffi::c_int;
                    }
                    if strequal(b"hide\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_HIDE as ::core::ffi::c_int;
                    }
                    if strequal(b"keepalt\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_KEEPALT as ::core::ffi::c_int;
                    }
                    if strequal(b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_KEEPMARKS as ::core::ffi::c_int;
                    }
                    if strequal(b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_KEEPJUMPS as ::core::ffi::c_int;
                    }
                    if strequal(b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_LOCKMARKS as ::core::ffi::c_int;
                    }
                    if strequal(b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char, k)
                        as ::core::ffi::c_int
                        != 0
                        && v as ::core::ffi::c_int != 0
                    {
                        flags |= CMOD_KEEPPATTERNS as ::core::ffi::c_int;
                    }
                }
            }
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
        let mut status: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut rets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0 {
            flags |= CMOD_SILENT as ::core::ffi::c_int;
        }
        let save_min_log_level: ::core::ffi::c_int = g_min_log_level.get();
        if log_level >= 0 as ::core::ffi::c_int {
            g_min_log_level.set(log_level);
        }
        let mut save_cmdmod: cmdmod_T = cmdmod.get();
        memset(
            cmdmod.ptr() as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<cmdmod_T>(),
        );
        (*cmdmod.ptr()).cmod_flags = flags;
        apply_cmdmod(cmdmod.ptr());
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        let mut aco: aco_save_T = aco_save_T::default();
        let mut win_execute_args: win_execute_T = win_execute_T {
            wp: ::core::ptr::null_mut::<win_T>(),
            curpos: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cwd: [0; 4096],
            cwd_status: 0,
            apply_acd: false,
            save_sfname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            switchwin: switchwin_T {
                sw_curwin: ::core::ptr::null_mut::<win_T>(),
                sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                sw_same_win: false,
                sw_visual_active: false,
            },
        };
        's_376: {
            if !win.is_null() {
                let mut tabpage: *mut tabpage_T = win_find_tabpage(win);
                if !win_execute_before(&raw mut win_execute_args, win, tabpage) {
                    break 's_376;
                }
            } else if !buf.is_null() {
                aucmd_prepbuf(&raw mut aco, buf);
            }
            let mut s: ::core::ffi::c_int = lua_gettop(L);
            lua_pushvalue(L, 2 as ::core::ffi::c_int);
            status = lua_pcall(
                L,
                0 as ::core::ffi::c_int,
                -1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            rets = lua_gettop(L) - s;
            if !win.is_null() {
                win_execute_after(&raw mut win_execute_args);
            } else if !buf.is_null() {
                aucmd_restbuf(&raw mut aco);
            }
        }
        try_leave(&raw mut tstate, &raw mut err);
        undo_cmdmod(cmdmod.ptr());
        cmdmod.set(save_cmdmod);
        if log_level >= 0 as ::core::ffi::c_int {
            g_min_log_level.set(save_min_log_level);
        }
        if status != 0 {
            return lua_error(L);
        } else if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            nlua_push_errstr(L, b"%s\0".as_ptr() as *const ::core::ffi::c_char, err.msg);
            api_clear_error(&raw mut err);
            return lua_error(L);
        }
        return rets;
    }
}
