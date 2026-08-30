//! `vim.regex()`: a Vimscript regexp as a Lua userdatum.
//!
//! [`nlua_regex`] compiles the pattern and pushes a userdatum whose metatable
//! is [`REGEX_META`]; `regex_match_str` and `regex_match_line` are its two
//! `match_*` methods, both reaching the same `regex_match` core, and
//! `regex_gc` frees the compiled program.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use super::{ERROR_INIT, TRY_STATE_INIT, error_set, nlua_push_errstr};
use crate::api::private::helpers::{handle_get_buffer, try_enter, try_leave};
use crate::global_cell::ConstTable;
use crate::lua::ffi::{
    LUA_REGISTRYINDEX, lua_error, lua_getfield, lua_gettop, lua_newuserdata, lua_pushinteger,
    lua_pushstring, lua_setmetatable, luaL_checkinteger, luaL_checkstring, luaL_checkudata,
    luaL_error,
};
use crate::luaL_reg_table;
use crate::main::curbuf;
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::regexp::{vim_regcomp, vim_regexec, vim_regfree};
use crate::types::{
    buf_T, colnr_T, handle_T, linenr_T, lua_State, luaL_Reg, regmatch_T, regprog_T,
};

/// The registry key the metatable is stored under, and the type name
/// `luaL_checkudata` matches against.
const REGEX_TYPE: &core::ffi::CStr = c"nvim_regex";

/// `vim_regcomp` flags: pick the engine automatically, honour 'magic', and
/// reject a pattern the automaton cannot take.
const RE_AUTO: c_int = 8;
const RE_MAGIC: c_int = 1;
const RE_STRICT: c_int = 4;

/// The compiled program behind the userdatum at slot 1, or a throw.
///
/// # Safety
/// `lstate` must be a live Lua state; this longjmps for a wrong argument.
unsafe fn regex_check(lstate: *mut lua_State) -> *mut *mut regprog_T {
    unsafe { luaL_checkudata(lstate, 1, REGEX_TYPE.as_ptr()).cast::<*mut regprog_T>() }
}

/// Match `prog` against `str` and push the match's start and end byte offsets,
/// or nothing.
///
/// `vim_regexec` may swap the program out for a recompiled one, so `prog` is
/// read back into the userdatum either way — including when it comes back
/// null, which is the NFA engine's out-of-memory answer and what both callers
/// report as an internal error.
///
/// # Safety
/// `lstate` must be a live Lua state, `prog` a live compiled program and
/// `str` a NUL-terminated subject.
unsafe fn regex_match(
    lstate: *mut lua_State,
    prog: *mut *mut regprog_T,
    str: *mut c_char,
) -> c_int {
    unsafe {
        let mut rm = regmatch_T {
            regprog: *prog,
            startp: [ptr::null_mut(); 10],
            endp: [ptr::null_mut(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        let matched = vim_regexec(&raw mut rm, str, 0 as colnr_T);
        *prog = rm.regprog;

        if matched {
            lua_pushinteger(lstate, rm.startp[0].offset_from(str));
            lua_pushinteger(lstate, rm.endp[0].offset_from(str));
            return 2;
        }
        0
    }
}

/// `regex:match_str(str)`.
///
/// # Safety
/// `lstate` must be a live Lua state holding this method's arguments.
unsafe extern "C-unwind" fn regex_match_str(lstate: *mut lua_State) -> c_int {
    unsafe {
        let prog = regex_check(lstate);
        let str = luaL_checkstring(lstate, 2);
        let nret = regex_match(lstate, prog, str.cast_mut());

        if (*prog).is_null() {
            return luaL_error(lstate, c"regex: internal error".as_ptr());
        }
        nret
    }
}

/// `regex:match_line(bufnr, rownr[, start[, end]])`.
///
/// The line is matched in place, so an `end` shorter than the line is applied
/// by writing a NUL over the buffer's own memory and putting the byte back
/// afterwards — which is why the error checks all happen before it.
///
/// # Safety
/// `lstate` must be a live Lua state holding this method's arguments.
unsafe extern "C-unwind" fn regex_match_line(lstate: *mut lua_State) -> c_int {
    unsafe {
        let prog = regex_check(lstate);

        let narg = lua_gettop(lstate);
        if narg < 3 {
            return luaL_error(lstate, c"not enough args".as_ptr());
        }

        let bufnr = luaL_checkinteger(lstate, 2) as handle_T;
        let rownr = luaL_checkinteger(lstate, 3) as linenr_T;
        let mut start: c_int = 0;
        let mut end: c_int = -1;
        if narg >= 4 {
            start = luaL_checkinteger(lstate, 4) as c_int;
        }
        if narg >= 5 {
            end = luaL_checkinteger(lstate, 5) as c_int;
            if end < 0 {
                return luaL_error(lstate, c"invalid end".as_ptr());
            }
        }

        let buf: *mut buf_T = if bufnr != 0 {
            handle_get_buffer(bufnr)
        } else {
            curbuf.get()
        };
        if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
            return luaL_error(lstate, c"invalid buffer".as_ptr());
        }

        if rownr >= (*buf).b_ml.ml_line_count {
            return luaL_error(lstate, c"invalid row".as_ptr());
        }

        let line = ml_get_buf(buf, rownr + 1);
        let len = ml_get_buf_len(buf, rownr + 1);

        if start < 0 || start > len {
            return luaL_error(lstate, c"invalid start".as_ptr());
        }

        let mut save: c_char = 0;
        if end >= 0 {
            if end > len || end < start {
                return luaL_error(lstate, c"invalid end".as_ptr());
            }
            save = *line.add(end as usize);
            *line.add(end as usize) = 0;
        }

        let nret = regex_match(lstate, prog, line.add(start as usize));

        if end >= 0 {
            *line.add(end as usize) = save;
        }

        if (*prog).is_null() {
            return luaL_error(lstate, c"regex: internal error".as_ptr());
        }
        nret
    }
}

/// `__gc`: free the compiled program.
///
/// # Safety
/// `lstate` must be a live Lua state with the userdatum at slot 1.
unsafe extern "C-unwind" fn regex_gc(lstate: *mut lua_State) -> c_int {
    unsafe {
        let prog = regex_check(lstate);
        vim_regfree(*prog);
        0
    }
}

/// `__tostring`.
///
/// # Safety
/// `lstate` must be a live Lua state with room for one more value.
unsafe extern "C-unwind" fn regex_tostring(lstate: *mut lua_State) -> c_int {
    unsafe {
        lua_pushstring(lstate, c"<regex>".as_ptr());
        1
    }
}

/// The userdatum's metatable, `luaL_register`ed under [`REGEX_TYPE`]. The
/// trailing all-null row is the terminator `luaL_register` scans for.
pub(crate) static REGEX_META: ConstTable<[luaL_Reg; 5]> = luaL_reg_table![
    c"__gc" => regex_gc,
    c"__tostring" => regex_tostring,
    c"match_str" => regex_match_str,
    c"match_line" => regex_match_line,
];

/// `vim.regex(pattern)`: compile it and push the userdatum.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub unsafe extern "C-unwind" fn nlua_regex(lstate: *mut lua_State) -> c_int {
    unsafe {
        let mut err = ERROR_INIT;
        let text = luaL_checkstring(lstate, 1);

        // vim_regcomp reports a bad pattern by throwing, so it runs bracketed.
        let mut tstate = TRY_STATE_INIT;
        try_enter(&raw mut tstate);
        let prog = vim_regcomp(text, RE_AUTO | RE_MAGIC | RE_STRICT);
        try_leave(&raw mut tstate, &mut err);

        if error_set(&err) {
            let why = err.message_or_empty().as_ptr();
            nlua_push_errstr(lstate, c"couldn't parse regex: %s".as_ptr(), why);
            err.clear();
            return lua_error(lstate);
        } else if prog.is_null() {
            nlua_push_errstr(lstate, c"couldn't parse regex".as_ptr());
            return lua_error(lstate);
        }

        let p = lua_newuserdata(lstate, size_of::<*mut regprog_T>()).cast::<*mut regprog_T>();
        *p = prog;

        lua_getfield(lstate, LUA_REGISTRYINDEX, REGEX_TYPE.as_ptr()); // [udata, meta]
        lua_setmetatable(lstate, -2); // [udata]
        1
    }
}
