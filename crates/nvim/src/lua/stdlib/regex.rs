//! `vim.regex()`: a Vimscript regexp as a Lua userdatum.
//!
//! [`nlua_regex`] compiles the pattern and pushes a userdatum whose metatable
//! is [`REGEX_META`]; `regex_match_str` and `regex_match_line` are its two
//! `match_*` methods, both reaching the same `regex_match` core, and
//! `regex_gc` frees the compiled program.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use crate::cstr;
use crate::winlayer::{self, Buf};
use core::ffi::{CStr, c_int};

use super::{TRY_STATE_INIT, nlua_push_errstr};
use crate::api::private::helpers::{try_enter, try_leave};
use crate::global_cell::ConstTable;
use crate::lua::ffi::{
    LUA_REGISTRYINDEX, lua_error, lua_getfield, lua_gettop, lua_newuserdata, lua_pushinteger,
    lua_pushstring, lua_setmetatable, luaL_checkinteger, luaL_checkstring, luaL_checkudata,
    luaL_error,
};
use crate::luaL_reg_table;
use crate::regexp::{
    OwnedProg, RE_AUTO, RE_MAGIC, RE_NOBREAK, RE_STRICT, vim_regcomp, vim_regexec,
};
use crate::types::{Error, Handle, LineNr, RegMatch, lua_State, luaL_Reg};

/// The registry key the metatable is stored under, and the type name
/// `luaL_checkudata` matches against.
const REGEX_TYPE: &CStr = c"nvim_regex";

/// The compiled program behind the userdatum at slot 1, or a throw. The
/// userdatum holds an `Option<OwnedProg>`, which [`nlua_regex`] wrote and
/// [`regex_gc`] empties.
///
/// # Safety
/// `lstate` must be a live Lua state; this longjmps for a wrong argument.
unsafe fn regex_check(lstate: *mut lua_State) -> *mut Option<OwnedProg> {
    unsafe { luaL_checkudata(lstate, 1, REGEX_TYPE.as_ptr()).cast::<Option<OwnedProg>>() }
}

/// Match `prog` against `str` and push the match's start and end byte offsets,
/// or nothing.
///
/// `vim_regexec` may swap the program out for a recompiled one, so `prog` is
/// read back into the userdatum either way. A userdatum with no program left
/// is what both callers report as an internal error.
///
/// # Safety
/// `lstate` must be a live Lua state.
unsafe fn regex_match(
    lstate: *mut lua_State,
    prog: &mut Option<OwnedProg>,
    subject: &CStr,
) -> c_int {
    let mut rm = RegMatch::new(prog.take(), false);
    let matched = vim_regexec(&mut rm, subject, 0);
    *prog = rm.regprog.take();

    match rm.group(0).filter(|_| matched) {
        Some(span) => {
            unsafe { lua_pushinteger(lstate, span.start as isize) };
            unsafe { lua_pushinteger(lstate, span.end as isize) };
            2
        }
        None => 0,
    }
}

/// `regex:match_str(str)`.
///
/// # Safety
/// `lstate` must be a live Lua state holding this method's arguments.
unsafe extern "C-unwind" fn regex_match_str(lstate: *mut lua_State) -> c_int {
    unsafe {
        let prog = regex_check(lstate);
        // SAFETY: `luaL_checkstring` answers a NUL-terminated string or
        // throws.
        let str = cstr::at(luaL_checkstring(lstate, 2));
        let nret = regex_match(lstate, &mut *prog, str);

        if (*prog).is_none() {
            return luaL_error(lstate, c"regex: internal error".as_ptr());
        }
        nret
    }
}

/// The buffer `bufnr` names, `0` for the current one, if it is loaded.
fn loaded_buffer(bufnr: Handle) -> Option<Buf> {
    let buf = if bufnr != 0 {
        winlayer::buffer(bufnr)
    } else {
        Buf::current_or_none()
    };
    buf.filter(|b| b.b_ml.ml_mfp.is_some())
}

/// `regex:match_line(bufnr, rownr[, start[, end]])`.
///
/// The `start..end` stretch of the line is matched as a string of its own,
/// copied out of the buffer: upstream writes a NUL over the buffer's memory
/// at `end` and puts the byte back afterwards, which the copy makes
/// unnecessary.
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

        let bufnr = luaL_checkinteger(lstate, 2) as Handle;
        let rownr = luaL_checkinteger(lstate, 3) as LineNr;
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

        let Some(buf) = loaded_buffer(bufnr) else {
            return luaL_error(lstate, c"invalid buffer".as_ptr());
        };

        if rownr >= buf.b_ml.ml_line_count {
            return luaL_error(lstate, c"invalid row".as_ptr());
        }

        let line = buf.lines().line_copy(rownr + 1);
        let len = line.len();

        let Some(start) = usize::try_from(start).ok().filter(|&start| start <= len) else {
            return luaL_error(lstate, c"invalid start".as_ptr());
        };
        let end = match usize::try_from(end) {
            Err(_) => len,
            Ok(end) if end <= len && end >= start => end,
            Ok(_) => return luaL_error(lstate, c"invalid end".as_ptr()),
        };

        let subject = cstr::owned(&line[start..end]);
        let nret = regex_match(lstate, &mut *prog, &subject);

        if (*prog).is_none() {
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
        drop((*prog).take());
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
        let mut err = Error::none();
        let text = luaL_checkstring(lstate, 1);

        // vim_regcomp reports a bad pattern by throwing, so it runs bracketed.
        let mut tstate = TRY_STATE_INIT;
        try_enter(&mut tstate);
        // Pick the engine automatically, honour 'magic', reject what the
        // automaton cannot take -- and never check for CTRL-C: that polls
        // the event loop, which aborts when the match runs inside a luv
        // callback (the loop is already running), and `vim.regex` is
        // allowed in a fast context.
        let prog = vim_regcomp(cstr::at(text), RE_AUTO | RE_MAGIC | RE_STRICT | RE_NOBREAK);
        err.absorb(try_leave(&tstate));

        if err.is_set() {
            nlua_push_errstr(lstate, b"couldn't parse regex: ", err.message_or_empty());
            err.clear();
            return lua_error(lstate);
        } else if prog.is_none() {
            nlua_push_errstr(lstate, b"couldn't parse regex", c"");
            return lua_error(lstate);
        }

        let p = lua_newuserdata(lstate, size_of::<Option<OwnedProg>>()).cast::<Option<OwnedProg>>();
        // The userdatum is fresh, uninitialised memory: write, do not assign.
        core::ptr::write(p, prog);

        lua_getfield(lstate, LUA_REGISTRYINDEX, REGEX_TYPE.as_ptr()); // [udata, meta]
        lua_setmetatable(lstate, -2); // [udata]
        1
    }
}
