//! `mpack.Session`: the msgpack-RPC half of the Lua module.
//!
//! A session frames messages but does not decode their bodies, so
//! [`receive`] is a loop between two suspendable machines: the RPC framer
//! reads the envelope, and — if the session was built with an `unpack`
//! option — the [`Unpacker`] reads the two values that follow it (a method
//! name and its arguments, or an error and a result). Either can run out of
//! input mid-value, and everything in flight stays in the session's private
//! table until both have landed.
//!
//! Nvim itself does not use this; `vim.mpack.Session` is exposed because
//! upstream's vendored libmpack-lua exposes it.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![forbid(unsafe_code)]

use core::ffi::{CStr, c_int};

use super::{
    Unpacker, geti, new_private_table, reference, start_offset, unpack_into_registry, unreference,
};
use crate::lua::ffi::{LUA_NOREF, LUA_REFNIL};
use crate::lua::stack::{At, Kind, Raise, Stack, Userdata};
use crate::lua_functions;
use crate::mpack::mpack_core::{MPACK_EOF, MPACK_OK};
use crate::mpack::object::MPACK_NOMEM;
use crate::mpack::rpc::{
    self, MPACK_RPC_NOTIFICATION, MPACK_RPC_REQUEST, MPACK_RPC_RESPONSE, RpcMessage,
};
use crate::types::{lua_Integer, lua_Number, mpack_uint32_t};

/// The scratch a whole envelope fits in: an array header, a one-byte type
/// and a five-byte id at most.
const ENVELOPE_MAX: usize = 16;

/// One `mpack.Session` instance. Its private table is its environment.
pub struct Session {
    session: Option<Box<rpc::Session>>,
    unpacked: Unpacked,
    /// The `unpack` option: an [`Unpacker`] to read message bodies with.
    unpacker: c_int,
}

impl Default for Session {
    fn default() -> Session {
        Session {
            session: None,
            unpacked: Unpacked {
                type_0: MPACK_EOF as c_int,
                msg: RpcMessage::default(),
                method_or_error: LUA_NOREF,
                args_or_result: LUA_NOREF,
            },
            unpacker: LUA_REFNIL,
        }
    }
}

impl Userdata for Session {
    const METATABLE: &'static CStr = c"mpack.Session";
}

/// The message [`receive`] is part way through.
pub struct Unpacked {
    pub type_0: c_int,
    pub msg: RpcMessage,
    pub method_or_error: c_int,
    pub args_or_result: c_int,
}

lua_functions! {
    /// A session's methods.
    pub(crate) enum SessionMethods in SESSION_METHODS {
        0 => c"receive": receive,
        1 => c"request": request,
        2 => c"reply": reply,
        3 => c"notify": notify,
    }
}

/// The session at slot 1, which every method has checked.
fn session(stack: &mut Stack) -> &mut Session {
    stack
        .to_userdata::<Session>(At::arg(1))
        .expect("checked on entry")
}

/// `mpack.Session([{unpack = <Unpacker>}])`.
pub fn new(stack: &mut Stack) -> Result<c_int, Raise> {
    stack.new_userdata(Session {
        session: Some(Box::new(rpc::Session::new(0))),
        ..Session::default()
    });
    let reg = new_private_table(stack);
    let mut unpacker = LUA_REFNIL;
    if stack.kind(At::arg(1)) == Kind::Table {
        stack.get_field(At::arg(1), c"unpack");
        if stack.to_userdata::<Unpacker>(At::TOP).is_none() {
            return Err(Raise::message(
                "\"unpack\" option must be a mpack.Unpacker instance",
            ));
        }
        unpacker = reference(stack, reg);
    }
    stack.pop(1);
    stack
        .to_userdata::<Session>(At::TOP)
        .expect("just made")
        .unpacker = unpacker;
    Ok(1)
}

/// `session:receive(str[, start])`.
///
/// Answers `kind, id, method_or_error, args_or_result, next_offset` — the
/// two middle values only when the session has an unpacker; all of them nil
/// until a whole message has arrived. `next_offset` is always last, so a
/// caller can resume from a partial buffer.
pub fn receive(stack: &mut Stack) -> Result<c_int, Raise> {
    let argc = stack.top();
    if !(2..=3).contains(&argc.index()) {
        return Err(Raise::message("expecting between 2 and 3 arguments"));
    }
    stack.check_userdata::<Session>(At::arg(1))?;
    let len = stack.check_bytes(At::arg(2))?.len();
    let offset = start_offset(stack, len)?;

    let mut view = stack.above(argc);
    let whole = view.check_bytes_below(At::arg(2))?;
    let mut input = &whole[offset..];
    view.push_environment(At::arg(1));
    let reg = view.top();

    // The unpacker, if any, is a separate userdatum reached through this
    // session's private table; it stays on the stack while it works.
    let unpacker_ref = session(&mut view).unpacker;
    let mut unpacker = None;
    let mut results = 3;
    if unpacker_ref != LUA_REFNIL {
        geti(&mut view, reg, unpacker_ref);
        view.check_userdata::<Unpacker>(At::TOP)?;
        let at = view.top();
        view.push_environment(at);
        unpacker = Some((at, view.top()));
        results += 2;
    }

    read_message(&mut view, reg, unpacker, &mut input)?;

    // A message is only complete when its body is too, unless there is no
    // unpacker to read a body with.
    let unpacked = &session(&mut view).unpacked;
    let complete = unpacked.type_0 != MPACK_EOF as c_int
        && (unpacked.args_or_result != LUA_NOREF || unpacker.is_none());
    if complete {
        if !push_message(&mut view, reg) {
            return Err(Raise::message("invalid msgpack-rpc string"));
        }
        session(&mut view).unpacked.type_0 = MPACK_EOF as c_int;
        if unpacker.is_some() {
            push_body(&mut view, reg);
        }
    } else {
        for _ in 0..results - 1 {
            view.push_nil();
        }
    }

    // One-based, and always last: the caller resumes from here.
    let consumed = whole.len() - input.len();
    view.push_integer(lua_Integer::try_from(consumed).unwrap_or(lua_Integer::MAX) + 1);
    Ok(results)
}

/// Read the envelope, and then as much of the body as the unpacker (its
/// slot and its private table's) can.
fn read_message(
    stack: &mut Stack,
    reg: At,
    unpacker: Option<(At, At)>,
    input: &mut &[u8],
) -> Result<(), Raise> {
    loop {
        let current = session(stack);
        if current.unpacked.type_0 == MPACK_EOF as c_int {
            let framer = current.session.as_mut().expect("a live session");
            current.unpacked.type_0 = framer.receive(input, &mut current.unpacked.msg);
            // With no unpacker the envelope *is* the message.
            if unpacker.is_none() || current.unpacked.type_0 == MPACK_EOF as c_int {
                return Ok(());
            }
        }
        let Some((unpacker, unpacker_reg)) = unpacker else {
            return Ok(());
        };
        if unpack_into_registry(stack, unpacker, unpacker_reg, input)? == MPACK_EOF as c_int {
            return Ok(());
        }
        // The first value is the method (or the error), the second the
        // arguments (or the result); the second ends the message.
        let handle = reference(stack, reg);
        let unpacked = &mut session(stack).unpacked;
        if unpacked.method_or_error == LUA_NOREF {
            unpacked.method_or_error = handle;
        } else {
            unpacked.args_or_result = handle;
            return Ok(());
        }
    }
}

/// Push the message kind and its id (or, for a response, the handle its
/// request carried). Answers `false` for an envelope kind that is not one of
/// the three.
fn push_message(stack: &mut Stack, reg: At) -> bool {
    let unpacked = &session(stack).unpacked;
    let (kind, id, data) = (unpacked.type_0, unpacked.msg.id, unpacked.msg.data);
    match kind {
        MPACK_RPC_REQUEST => {
            stack.push_bytes(b"request");
            stack.push_number(id as lua_Number);
        }
        MPACK_RPC_RESPONSE => {
            stack.push_bytes(b"response");
            geti(stack, reg, data as c_int);
        }
        MPACK_RPC_NOTIFICATION => {
            stack.push_bytes(b"notification");
            stack.push_nil();
        }
        // Anything else is an `MPACK_RPC_E*`. Upstream reports them all the
        // same way, on the grounds that the only sane response to invalid
        // msgpack-rpc is to close the connection.
        _ => return false,
    }
    true
}

/// Push the two decoded body values and release their references.
fn push_body(stack: &mut Stack, reg: At) {
    let unpacked = &session(stack).unpacked;
    let (method_or_error, args_or_result) = (unpacked.method_or_error, unpacked.args_or_result);
    geti(stack, reg, method_or_error);
    geti(stack, reg, args_or_result);
    unreference(stack, reg, method_or_error);
    unreference(stack, reg, args_or_result);
    let unpacked = &mut session(stack).unpacked;
    unpacked.method_or_error = LUA_NOREF;
    unpacked.args_or_result = LUA_NOREF;
}

/// `session:request([handle])`: the envelope of a fresh request, whose id is
/// allocated here and whose `handle` comes back with the response.
pub fn request(stack: &mut Stack) -> Result<c_int, Raise> {
    if !(1..=2).contains(&stack.top().index()) {
        return Err(Raise::message("expecting 1 or 2 arguments"));
    }
    stack.check_userdata::<Session>(At::arg(1))?;
    let handle = if stack.is_none_or_nil(At::arg(2)) {
        LUA_NOREF
    } else {
        stack.push_environment(At::arg(1));
        let reg = stack.top();
        stack.push_value(At::arg(2));
        let handle = reference(stack, reg);
        stack.pop(1);
        handle
    };
    let mut buf = [0u8; ENVELOPE_MAX];
    let framer = session(stack).session.as_mut().expect("a live session");
    let used = loop {
        let (result, used) = framer.request(&mut buf, i64::from(handle));
        if result != MPACK_NOMEM {
            debug_assert_eq!(result, MPACK_OK as c_int);
            break used;
        }
        // Every slot holds an outstanding request; double the table.
        framer.grow();
    };
    stack.push_bytes(&buf[..used]);
    Ok(1)
}

/// `session:reply(id)`.
pub fn reply(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() != 2 {
        return Err(Raise::message("expecting exactly 2 arguments"));
    }
    stack.check_userdata::<Session>(At::arg(1))?;
    let id = stack.to_number(At::arg(2));
    if !(id.trunc() == id && (0.0..=4294967295.0).contains(&id)) {
        return Err(Raise::argument(2, "invalid request id"));
    }
    let mut buf = [0u8; ENVELOPE_MAX];
    let framer = session(stack).session.as_mut().expect("a live session");
    let (result, used) = framer.reply(&mut buf, id as mpack_uint32_t);
    debug_assert_eq!(result, MPACK_OK as c_int);
    stack.push_bytes(&buf[..used]);
    Ok(1)
}

/// `session:notify()`.
pub fn notify(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() != 1 {
        return Err(Raise::message("expecting exactly 1 argument"));
    }
    stack.check_userdata::<Session>(At::arg(1))?;
    let mut buf = [0u8; ENVELOPE_MAX];
    let framer = session(stack).session.as_mut().expect("a live session");
    let (result, used) = framer.notify(&mut buf);
    debug_assert_eq!(result, MPACK_OK as c_int);
    stack.push_bytes(&buf[..used]);
    Ok(1)
}
