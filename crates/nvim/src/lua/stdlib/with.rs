//! `vim._with()`: run a function with the editor state temporarily swapped.
//!
//! One function, because unwinding it is one linear sequence: the option
//! table at slot 1 is read into a set of `:command` modifier flags plus at
//! most one of a buffer or a window, those are applied, the callback at slot
//! 2 is called through `lua_pcall`, and everything is put back in reverse —
//! through the same `try_enter`/`try_leave` bracket the API uses.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_int, c_void};
use core::ptr;

use super::{ERROR_INIT, TRY_STATE_INIT, error_set, nlua_push_errstr};
use crate::api::private::helpers::{
    api_clear_error, handle_get_buffer, handle_get_window, try_enter, try_leave,
};
use crate::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::eval::window::{win_execute_after, win_execute_before};
use crate::ex_docmd::{apply_cmdmod, undo_cmdmod};
use crate::lua::ffi::{
    LUA_MULTRET, LUA_TSTRING, lua_error, lua_gettop, lua_istable, lua_next, lua_pcall, lua_pop,
    lua_pushnil, lua_pushvalue, lua_toboolean, lua_tostring, lua_type, luaL_argcheck,
    luaL_checkinteger,
};
use crate::main::{cmdmod, g_min_log_level};
use crate::types::{
    CmdModFlags, aco_save_T, buf_T, cmdmod_T, lua_State, pos_T, switchwin_T, win_T, win_execute_T,
};
use crate::window::win_find_tabpage;
use ::libc::memset;

/// The context keys that are plain `:command` modifiers: a truthy value ors
/// the flag in, anything else is ignored.
const FLAG_KEYS: [(&CStr, CmdModFlags); 11] = [
    (c"sandbox", CmdModFlags::SANDBOX),
    (c"silent", CmdModFlags::SILENT),
    (c"emsg_silent", CmdModFlags::ERRSILENT),
    (c"unsilent", CmdModFlags::UNSILENT),
    (c"noautocmd", CmdModFlags::NOAUTOCMD),
    (c"hide", CmdModFlags::HIDE),
    (c"keepalt", CmdModFlags::KEEPALT),
    (c"keepmarks", CmdModFlags::KEEPMARKS),
    (c"keepjumps", CmdModFlags::KEEPJUMPS),
    (c"lockmarks", CmdModFlags::LOCKMARKS),
    (c"keeppatterns", CmdModFlags::KEEPPATTERNS),
];

/// An all-zero [`win_execute_T`]; `win_execute_before` fills it.
const WIN_EXECUTE_INIT: win_execute_T = win_execute_T {
    wp: ptr::null_mut(),
    curpos: pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    },
    cwd: [0; 4096],
    cwd_status: 0,
    apply_acd: false,
    save_sfname: ptr::null_mut(),
    switchwin: switchwin_T {
        sw_curwin: ptr::null_mut(),
        sw_curtab: ptr::null_mut(),
        sw_same_win: false,
        sw_visual_active: false,
    },
};

/// `vim._with_c(context, fn)`.
///
/// # Safety
/// `lstate` must be a live Lua state with the context table at slot 1 and the
/// callback at slot 2.
pub(crate) unsafe extern "C-unwind" fn nlua_with(lstate: *mut lua_State) -> c_int {
    unsafe {
        let mut flags = CmdModFlags::NONE;
        let mut buf: *mut buf_T = ptr::null_mut();
        let mut win: *mut win_T = ptr::null_mut();
        let mut log_level: c_int = -1;

        luaL_argcheck(
            lstate,
            lua_istable(lstate, 1),
            1,
            c"table expected".as_ptr(),
        );
        lua_pushnil(lstate); // [dict, ..., nil]
        while lua_next(lstate, 1) != 0 {
            // [dict, ..., key, value]
            if lua_type(lstate, -2) == LUA_TSTRING {
                let k = CStr::from_ptr(lua_tostring(lstate, -2));
                if k == c"buf" {
                    buf = handle_get_buffer(luaL_checkinteger(lstate, -1) as c_int);
                } else if k == c"win" {
                    win = handle_get_window(luaL_checkinteger(lstate, -1) as c_int);
                } else if k == c"log_level" {
                    log_level = luaL_checkinteger(lstate, -1) as c_int;
                } else if lua_toboolean(lstate, -1) != 0 {
                    for (name, flag) in FLAG_KEYS {
                        if k == name {
                            flags |= flag;
                        }
                    }
                }
            }
            // Pop the value; lua_next will pop the key.
            lua_pop(lstate, 1); // [dict, ..., key]
        }
        let mut status: c_int = 0;
        let mut rets: c_int = 0;

        if flags.has(CmdModFlags::ERRSILENT) {
            // CmdModFlags::ERRSILENT must imply CmdModFlags::SILENT, or apply_cmdmod() and
            // undo_cmdmod() won't work properly.
            flags |= CmdModFlags::SILENT;
        }

        let save_min_log_level = g_min_log_level.get();
        if log_level >= 0 {
            g_min_log_level.set(log_level);
        }
        let save_cmdmod: cmdmod_T = cmdmod.with(Clone::clone);
        memset(cmdmod.ptr().cast::<c_void>(), 0, size_of::<cmdmod_T>());
        (*cmdmod.ptr()).cmod_flags = flags;
        apply_cmdmod(cmdmod.ptr());

        let mut err = ERROR_INIT;
        let mut tstate = TRY_STATE_INIT;
        try_enter(&raw mut tstate);
        {
            let mut aco = aco_save_T::default();
            let mut win_execute_args = WIN_EXECUTE_INIT;

            // A window that cannot be entered leaves everything below
            // untouched: no call, no results, and nothing to restore.
            let entered = if !win.is_null() {
                let tabpage = win_find_tabpage(win);
                win_execute_before(&raw mut win_execute_args, win, tabpage)
            } else {
                if !buf.is_null() {
                    aucmd_prepbuf(&raw mut aco, buf);
                }
                true
            };

            if entered {
                let s = lua_gettop(lstate);
                lua_pushvalue(lstate, 2);
                status = lua_pcall(lstate, 0, LUA_MULTRET, 0);
                rets = lua_gettop(lstate) - s;

                if !win.is_null() {
                    win_execute_after(&raw mut win_execute_args);
                } else if !buf.is_null() {
                    aucmd_restbuf(&raw mut aco);
                }
            }
        }
        try_leave(&raw mut tstate, &raw mut err);

        undo_cmdmod(cmdmod.ptr());
        cmdmod.set(save_cmdmod);
        if log_level >= 0 {
            g_min_log_level.set(save_min_log_level);
        }

        if status != 0 {
            return lua_error(lstate);
        } else if error_set(&err) {
            nlua_push_errstr(lstate, c"%s".as_ptr(), err.msg);
            api_clear_error(&raw mut err);
            return lua_error(lstate);
        }
        rets
    }
}
