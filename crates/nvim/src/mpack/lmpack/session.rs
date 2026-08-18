//! `mpack.Session`: the msgpack-RPC half of the Lua module.
//!
//! A session frames messages but does not decode their bodies, so
//! [`receive`] is a loop between two suspendable machines: the RPC framer
//! reads the envelope, and — if the session was built with an `unpack`
//! option — the [`Unpacker`] reads the two values that follow it (a method
//! name and its arguments, or an error and a result). Either can run out of
//! input mid-value, and everything in flight stays in the session's private
//! registry until both have landed.
//!
//! Nvim itself does not use this; `vim.mpack.Session` is exposed because
//! upstream's vendored libmpack-lua exposes it.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::{
    LUA_NOREF, LUA_REFNIL, Session, Unpacker, check_session, check_unpacker, geti, grow_session,
    is_unpacker, reference, start_offset, unpack_into_registry, unreference,
};
use crate::lua::ffi::{
    LUA_REGISTRYINDEX, lua_getfield, lua_gettop, lua_isnoneornil, lua_istable, lua_newtable,
    lua_newuserdata, lua_pop, lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushnumber,
    lua_pushstring, lua_setmetatable, lua_tonumber, luaL_argcheck, luaL_checklstring, luaL_error,
    luaL_ref, luaL_unref,
};
use crate::mpack::mpack_core::{MPACK_EOF, MPACK_OK};
use crate::mpack::object::MPACK_NOMEM;
use crate::mpack::rpc::{
    MPACK_RPC_NOTIFICATION, MPACK_RPC_REQUEST, MPACK_RPC_RESPONSE, mpack_rpc_notify,
    mpack_rpc_receive, mpack_rpc_reply, mpack_rpc_request, mpack_rpc_session_init,
};
use crate::os::libc::{free, malloc};
use crate::types::{
    lua_Integer, lua_Number, lua_State, mpack_data_t, mpack_rpc_session_t, mpack_uint32_t, size_t,
};

/// The scratch a whole envelope fits in: an array header, a one-byte type
/// and a five-byte id at most.
const ENVELOPE_MAX: usize = 16;

/// `mpack.Session([{unpack = <Unpacker>}])`.
///
/// # Safety
/// `state` must be a live Lua state holding this function's arguments.
pub unsafe extern "C-unwind" fn new(state: *mut lua_State) -> c_int {
    unsafe {
        let session = lua_newuserdata(state, size_of::<Session>()).cast::<Session>();
        (*session).session = malloc(size_of::<mpack_rpc_session_t>()).cast::<mpack_rpc_session_t>();
        if (*session).session.is_null() {
            return luaL_error(state, c"Failed to allocate memory".as_ptr());
        }
        mpack_rpc_session_init((*session).session, 0);
        (*session).L = state;
        lua_getfield(state, LUA_REGISTRYINDEX, super::SESSION_META);
        lua_setmetatable(state, -2);
        lua_newtable(state);
        (*session).reg = luaL_ref(state, LUA_REGISTRYINDEX);
        (*session).unpacker = LUA_REFNIL;
        (*session).unpacked.args_or_result = LUA_NOREF;
        (*session).unpacked.method_or_error = LUA_NOREF;
        (*session).unpacked.type_0 = MPACK_EOF as c_int;

        if lua_istable(state, 1) {
            lua_getfield(state, 1, c"unpack".as_ptr());
            if !is_unpacker(state, -1) {
                return luaL_error(
                    state,
                    c"\"unpack\" option must be a mpack.Unpacker instance".as_ptr(),
                );
            }
            (*session).unpacker = reference(state, (*session).reg);
        }
        1
    }
}

/// `__gc`.
///
/// # Safety
/// See [`new`].
pub unsafe extern "C-unwind" fn delete(state: *mut lua_State) -> c_int {
    unsafe {
        let session = check_session(state, 1);
        unreference(state, (*session).reg, (*session).unpacker);
        luaL_unref(state, LUA_REGISTRYINDEX, (*session).reg);
        free((*session).session.cast());
        0
    }
}

/// `session:receive(str[, start])`.
///
/// Answers `kind, id, method_or_error, args_or_result, next_offset` — the
/// two middle values only when the session has an unpacker; all of them nil
/// until a whole message has arrived. `next_offset` is always last, so a
/// caller can resume from a partial buffer.
///
/// # Safety
/// See [`new`].
pub unsafe extern "C-unwind" fn receive(state: *mut lua_State) -> c_int {
    unsafe {
        let argc = lua_gettop(state);
        if !(2..=3).contains(&argc) {
            return luaL_error(state, c"expecting between 2 and 3 arguments".as_ptr());
        }
        let session = check_session(state, 1);
        let mut len: size_t = 0;
        let start = luaL_checklstring(state, 2, &raw mut len);
        let offset = start_offset(state, len);
        let mut str = start.add(offset);
        let mut left = len - offset;

        // The unpacker, if any, is a separate userdatum reached through this
        // session's registry; it needs the current `lua_State` too.
        let mut unpacker: *mut Unpacker = core::ptr::null_mut();
        let mut results = 3;
        if (*session).unpacker != LUA_REFNIL {
            geti(state, (*session).reg, (*session).unpacker);
            unpacker = check_unpacker(state, -1);
            (*unpacker).L = state;
            results += 2;
            lua_pop(state, 1);
        }

        read_message(state, session, unpacker, &raw mut str, &raw mut left);

        // A message is only complete when its body is too, unless there is
        // no unpacker to read a body with.
        let complete = (*session).unpacked.type_0 != MPACK_EOF as c_int
            && ((*session).unpacked.args_or_result != LUA_NOREF || unpacker.is_null());
        if complete {
            if push_message(state, session) != 0 {
                return luaL_error(state, c"invalid msgpack-rpc string".as_ptr());
            }
            (*session).unpacked.type_0 = MPACK_EOF as c_int;
            if !unpacker.is_null() {
                push_body(state, session);
            }
        } else {
            lua_pushnil(state);
            lua_pushnil(state);
            if !unpacker.is_null() {
                lua_pushnil(state);
                lua_pushnil(state);
            }
        }

        // One-based, and always last: the caller resumes from here.
        lua_pushinteger(state, str.offset_from(start) as lua_Integer + 1);
        results
    }
}

/// Read the envelope, and then as much of the body as the unpacker can.
///
/// # Safety
/// `session` must be live; `str`/`left` must describe the remaining input.
unsafe fn read_message(
    state: *mut lua_State,
    session: *mut Session,
    unpacker: *mut Unpacker,
    str: *mut *const c_char,
    left: *mut size_t,
) {
    unsafe {
        loop {
            if (*session).unpacked.type_0 == MPACK_EOF as c_int {
                (*session).unpacked.type_0 = mpack_rpc_receive(
                    (*session).session,
                    str,
                    left,
                    &raw mut (*session).unpacked.msg,
                );
                // With no unpacker the envelope *is* the message.
                if unpacker.is_null() || (*session).unpacked.type_0 == MPACK_EOF as c_int {
                    return;
                }
            }
            if unpack_into_registry(state, unpacker, str, left) == MPACK_EOF as c_int {
                return;
            }
            // The first value is the method (or the error), the second the
            // arguments (or the result); the second ends the message.
            if (*session).unpacked.method_or_error == LUA_NOREF {
                (*session).unpacked.method_or_error = reference(state, (*session).reg);
            } else {
                (*session).unpacked.args_or_result = reference(state, (*session).reg);
                return;
            }
        }
    }
}

/// Push the message kind and its id (or, for a response, the handle its
/// request carried). Answers non-zero for an envelope kind that is not one
/// of the three.
///
/// # Safety
/// `session` must be live with a complete envelope.
unsafe fn push_message(state: *mut lua_State, session: *mut Session) -> c_int {
    unsafe {
        match (*session).unpacked.type_0 {
            MPACK_RPC_REQUEST => {
                lua_pushstring(state, c"request".as_ptr());
                lua_pushnumber(state, (*session).unpacked.msg.id as lua_Number);
            }
            MPACK_RPC_RESPONSE => {
                lua_pushstring(state, c"response".as_ptr());
                geti(
                    state,
                    (*session).reg,
                    (*session).unpacked.msg.data.i as c_int,
                );
            }
            MPACK_RPC_NOTIFICATION => {
                lua_pushstring(state, c"notification".as_ptr());
                lua_pushnil(state);
            }
            // Anything else is an `MPACK_RPC_E*`. Upstream reports them all
            // the same way, on the grounds that the only sane response to
            // invalid msgpack-rpc is to close the connection.
            _ => return 1,
        }
        0
    }
}

/// Push the two decoded body values and release their references.
///
/// # Safety
/// `session` must be live with both body values decoded.
unsafe fn push_body(state: *mut lua_State, session: *mut Session) {
    unsafe {
        let reg = (*session).reg;
        geti(state, reg, (*session).unpacked.method_or_error);
        geti(state, reg, (*session).unpacked.args_or_result);
        unreference(state, reg, (*session).unpacked.method_or_error);
        unreference(state, reg, (*session).unpacked.args_or_result);
        (*session).unpacked.method_or_error = LUA_NOREF;
        (*session).unpacked.args_or_result = LUA_NOREF;
    }
}

/// `session:request([handle])`: the envelope of a fresh request, whose id is
/// allocated here and whose `handle` comes back with the response.
///
/// # Safety
/// See [`new`].
pub unsafe extern "C-unwind" fn request(state: *mut lua_State) -> c_int {
    unsafe {
        if !(1..=2).contains(&lua_gettop(state)) {
            return luaL_error(state, c"expecting 1 or 2 arguments".as_ptr());
        }
        let session = check_session(state, 1);
        let handle = if lua_isnoneornil(state, 2) {
            LUA_NOREF
        } else {
            reference(state, (*session).reg)
        };
        let data = mpack_data_t { i: handle as i64 };

        let mut buf = [0 as c_char; ENVELOPE_MAX];
        let mut ptr = buf.as_mut_ptr();
        let mut left = ENVELOPE_MAX;
        loop {
            let result = mpack_rpc_request((*session).session, &raw mut ptr, &raw mut left, data);
            if result != MPACK_NOMEM {
                debug_assert_eq!(result, MPACK_OK as c_int);
                break;
            }
            // Every slot holds an outstanding request; double the table.
            (*session).session = grow_session((*session).session);
            if (*session).session.is_null() {
                return luaL_error(state, c"Failed to grow Session capacity".as_ptr());
            }
        }
        lua_pushlstring(state, buf.as_ptr(), ENVELOPE_MAX - left);
        1
    }
}

/// `session:reply(id)`.
///
/// # Safety
/// See [`new`].
pub unsafe extern "C-unwind" fn reply(state: *mut lua_State) -> c_int {
    unsafe {
        if lua_gettop(state) != 2 {
            return luaL_error(state, c"expecting exactly 2 arguments".as_ptr());
        }
        let session = check_session(state, 1);
        let id = lua_tonumber(state, 2);
        luaL_argcheck(
            state,
            id.trunc() == id && (0.0..=4294967295.0).contains(&id),
            2,
            c"invalid request id".as_ptr(),
        );

        let mut buf = [0 as c_char; ENVELOPE_MAX];
        let mut ptr = buf.as_mut_ptr();
        let mut left = ENVELOPE_MAX;
        let result = mpack_rpc_reply(
            (*session).session,
            &raw mut ptr,
            &raw mut left,
            id as mpack_uint32_t,
        );
        debug_assert_eq!(result, MPACK_OK as c_int);
        lua_pushlstring(state, buf.as_ptr(), ENVELOPE_MAX - left);
        1
    }
}

/// `session:notify()`.
///
/// # Safety
/// See [`new`].
pub unsafe extern "C-unwind" fn notify(state: *mut lua_State) -> c_int {
    unsafe {
        if lua_gettop(state) != 1 {
            return luaL_error(state, c"expecting exactly 1 argument".as_ptr());
        }
        let session = check_session(state, 1);
        let mut buf = [0 as c_char; ENVELOPE_MAX];
        let mut ptr = buf.as_mut_ptr();
        let mut left = ENVELOPE_MAX;
        let result = mpack_rpc_notify((*session).session, &raw mut ptr, &raw mut left);
        debug_assert_eq!(result, MPACK_OK as c_int);
        lua_pushlstring(state, buf.as_ptr(), ENVELOPE_MAX - left);
        1
    }
}
