//! `lmpack.c`: the `vim.mpack` Lua module.
//!
//! Three userdata classes and two shorthands. `Unpacker` and `Packer` are
//! reusable, suspendable codecs — you hand them a partial buffer and they
//! hand back what they got plus where to resume — and `mpack.decode` /
//! `mpack.encode` are one-shot wrappers that build a codec on the stack.
//! `Session` (in [`session`]) adds msgpack-RPC framing on top.
//!
//! **Every instance owns a private registry table.** Values in flight during
//! a walk have to outlive the call that produced them, and the walk cannot
//! use the Lua stack for that because it suspends. So each instance holds
//! one `luaL_ref` into the real registry, and [`reference`] / [`geti`] /
//! [`unreference`] are `luaL_ref` / `lua_rawgeti` / `luaL_unref` scoped to
//! that table. Collecting the instance drops the table and everything in it,
//! which is the only reason this is tractable at all.
//!
//! Two things here are nvim's rather than upstream libmpack-lua's: the
//! `mtdict` field, which carries `vim.empty_dict()`'s metatable so an empty
//! *map* round-trips as a dict rather than a list, and `luaopen_mpack` being
//! called from `nlua_state_add_stdlib` instead of by `require`.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod session;
pub mod walk;

use core::ffi::{c_char, c_int, c_uint};

use crate::luaL_reg_table;
use crate::src::mpack::mpack_core::MPACK_EOF;
use crate::src::mpack::object::{
    MPACK_MAX_OBJECT_DEPTH, MPACK_NOMEM, mpack_parse, mpack_parser_copy, mpack_parser_init,
    mpack_unparse,
};
use crate::src::mpack::rpc::{
    MPACK_RPC_MAX_REQUESTS, mpack_rpc_session_copy, mpack_rpc_session_init,
};
use crate::src::nvim::global_cell::SharedCell;
use crate::src::nvim::lua::ffi::{
    LUA_NOREF, LUA_REFNIL, LUA_REGISTRYINDEX, LUA_TFUNCTION, LUA_TNUMBER, LUA_TTABLE, lua_getfield,
    lua_gettop, lua_insert, lua_isnil, lua_istable, lua_newtable, lua_newuserdata, lua_next,
    lua_objlen, lua_pop, lua_pushcfunction, lua_pushfstring, lua_pushinteger, lua_pushnil,
    lua_pushstring, lua_pushvalue, lua_rawequal, lua_rawgeti, lua_remove, lua_replace,
    lua_setfield, lua_setmetatable, lua_settable, lua_toboolean, lua_tonumber, lua_topointer,
    lua_type, luaL_argcheck, luaL_buffinit, luaL_checklstring, luaL_checknumber, luaL_checkudata,
    luaL_error, luaL_newmetatable, luaL_prepbuffer, luaL_pushresult, luaL_ref, luaL_register,
    luaL_unref,
};
use crate::src::nvim::os::libc::{free, malloc};
use crate::src::nvim::types::{
    lua_Number, lua_State, luaL_Buffer, luaL_Reg, mpack_data_t, mpack_node_t, mpack_one_parser_t,
    mpack_parser_t, mpack_rpc_message_t, mpack_rpc_one_session_t, mpack_rpc_session_t,
    mpack_rpc_slot_s, mpack_uint32_t, size_t,
};

/// The metatable names, which double as the registry keys they are stored
/// under.
const UNPACKER_META: *const c_char = c"mpack.Unpacker".as_ptr();
const PACKER_META: *const c_char = c"mpack.Packer".as_ptr();
const SESSION_META: *const c_char = c"mpack.Session".as_ptr();
/// The shared userdatum standing in for msgpack `nil`, which Lua's own `nil`
/// cannot represent inside a table.
const NIL_NAME: *const c_char = c"mpack.NIL".as_ptr();
/// `vim.empty_dict()`'s metatable, put in the registry by nvim's Lua setup.
const EMPTY_DICT_NAME: *const c_char = c"mpack.empty_dict".as_ptr();

/// `luaL_Buffer`'s own capacity, which is how much `luaL_prepbuffer` hands
/// out at a time.
const BUFFER_SIZE: size_t = 8192;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Unpacker {
    pub L: *mut lua_State,
    pub parser: *mut mpack_parser_t,
    /// This instance's private registry table.
    pub reg: c_int,
    /// The `ext` option: a table of decoders keyed by ext type code.
    pub ext: c_int,
    /// Set while a walk is in progress, so a decoder that re-enters the same
    /// instance is refused rather than corrupting it.
    pub unpacking: c_int,
    pub mtdict: c_int,
    /// The blob being assembled, owned by `malloc`/`free`.
    pub string_buffer: *mut c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Packer {
    pub L: *mut lua_State,
    pub parser: *mut mpack_parser_t,
    pub reg: c_int,
    /// The `ext` option: a table of encoders keyed by *metatable*.
    pub ext: c_int,
    /// The value being encoded.
    pub root: c_int,
    pub packing: c_int,
    pub mtdict: c_int,
    /// The `is_bin` option: encode strings as `bin` rather than `str`.
    pub is_bin: c_int,
    /// `is_bin` as a predicate, when it was given as a function.
    pub is_bin_fn: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Session {
    pub L: *mut lua_State,
    pub reg: c_int,
    pub session: *mut mpack_rpc_session_t,
    pub unpacked: Unpacked,
    /// The `unpack` option: an [`Unpacker`] to read message bodies with.
    pub unpacker: c_int,
}

/// The message [`session::receive`] is part way through.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Unpacked {
    pub type_0: c_int,
    pub msg: mpack_rpc_message_t,
    pub method_or_error: c_int,
    pub args_or_result: c_int,
}

// ---------------------------------------------------------------------------
// The private registry
// ---------------------------------------------------------------------------

/// Move the value on top of the stack into `reg` and answer its reference.
///
/// # Safety
/// `state` must be live with a value on top and `reg` must name a table.
pub unsafe fn reference(state: *mut lua_State, reg: c_int) -> c_int {
    unsafe {
        lua_rawgeti(state, LUA_REGISTRYINDEX, reg); // [value, reg]
        lua_pushvalue(state, -2); // [value, reg, value]
        let handle = luaL_ref(state, -2); // [value, reg]
        lua_pop(state, 2);
        handle
    }
}

/// Release a reference taken by [`reference`].
///
/// # Safety
/// See [`reference`].
pub unsafe fn unreference(state: *mut lua_State, reg: c_int, handle: c_int) {
    unsafe {
        lua_rawgeti(state, LUA_REGISTRYINDEX, reg);
        luaL_unref(state, -1, handle);
        lua_pop(state, 1);
    }
}

/// Push the value a reference names.
///
/// # Safety
/// See [`reference`].
pub unsafe fn geti(state: *mut lua_State, reg: c_int, handle: c_int) {
    unsafe {
        lua_rawgeti(state, LUA_REGISTRYINDEX, reg); // [reg]
        lua_rawgeti(state, -1, handle); // [reg, value]
        lua_replace(state, -2); // [value]
    }
}

/// Replace the table on top of the stack with a shallow copy, so that later
/// edits to the caller's table cannot change an instance's options.
///
/// # Safety
/// `state` must be live with a table on top.
unsafe fn shallow_copy(state: *mut lua_State) {
    unsafe {
        lua_newtable(state);
        lua_pushnil(state);
        while lua_next(state, -3) != 0 {
            lua_pushvalue(state, -2);
            lua_insert(state, -2);
            lua_settable(state, -4);
        }
        lua_remove(state, -2);
    }
}

// ---------------------------------------------------------------------------
// Growing the C-side stacks
// ---------------------------------------------------------------------------

/// Reallocate a parser twice as deep, keeping the walk in progress. Answers
/// null when the allocation fails, in which case `parser` is still valid.
///
/// # Safety
/// `parser` must be a live `malloc`ed parser.
unsafe fn grow_parser(parser: *mut mpack_parser_t) -> *mut mpack_parser_t {
    unsafe {
        let capacity = (*parser).capacity * 2;
        // A parser's frames are a C flexible array member: the struct
        // declares one and the rest are bought here.
        let bytes = size_of::<mpack_one_parser_t>() + size_of::<mpack_node_t>() * capacity as usize;
        let bigger = malloc(bytes).cast::<mpack_parser_t>();
        if !bigger.is_null() {
            mpack_parser_init(bigger, capacity);
            mpack_parser_copy(bigger, parser);
            free(parser.cast());
        }
        bigger
    }
}

/// The same for an RPC session's slot table.
///
/// # Safety
/// `session` must be a live `malloc`ed session.
unsafe fn grow_session(session: *mut mpack_rpc_session_t) -> *mut mpack_rpc_session_t {
    unsafe {
        let capacity = (*session).capacity * 2;
        let bytes = size_of::<mpack_rpc_one_session_t>()
            + size_of::<mpack_rpc_slot_s>() * (capacity - 1) as usize;
        let bigger = malloc(bytes).cast::<mpack_rpc_session_t>();
        if !bigger.is_null() {
            mpack_rpc_session_init(bigger, capacity);
            mpack_rpc_session_copy(bigger, session);
            free(session.cast());
        }
        bigger
    }
}

// ---------------------------------------------------------------------------
// Type checks and the NIL sentinel
// ---------------------------------------------------------------------------

/// # Safety
/// `state` must be live.
pub unsafe fn check_unpacker(state: *mut lua_State, index: c_int) -> *mut Unpacker {
    unsafe { luaL_checkudata(state, index, UNPACKER_META).cast::<Unpacker>() }
}

/// # Safety
/// `state` must be live.
unsafe fn check_packer(state: *mut lua_State, index: c_int) -> *mut Packer {
    unsafe { luaL_checkudata(state, index, PACKER_META).cast::<Packer>() }
}

/// # Safety
/// `state` must be live.
pub unsafe fn check_session(state: *mut lua_State, index: c_int) -> *mut Session {
    unsafe { luaL_checkudata(state, index, SESSION_META).cast::<Session>() }
}

/// Whether the value at `index` is the shared msgpack-nil userdatum.
///
/// # Safety
/// `state` must be live.
pub unsafe fn is_nil_sentinel(state: *mut lua_State, index: c_int) -> bool {
    unsafe {
        if lua_type(state, index) != crate::src::nvim::lua::ffi::LUA_TUSERDATA {
            return false;
        }
        lua_getfield(state, LUA_REGISTRYINDEX, NIL_NAME);
        let same = lua_rawequal(state, -1, index - 1) != 0;
        lua_pop(state, 1);
        same
    }
}

/// Push it.
///
/// # Safety
/// `state` must be live with room for one more value.
pub unsafe fn push_nil_sentinel(state: *mut lua_State) {
    unsafe { lua_getfield(state, LUA_REGISTRYINDEX, NIL_NAME) };
}

/// Whether the value at `index` carries the [`Unpacker`] metatable.
///
/// # Safety
/// `state` must be live.
pub unsafe fn is_unpacker(state: *mut lua_State, index: c_int) -> bool {
    unsafe {
        if lua_type(state, index) != crate::src::nvim::lua::ffi::LUA_TUSERDATA
            || lua_getmetatable(state, index) == 0
        {
            return false;
        }
        lua_getfield(state, LUA_REGISTRYINDEX, UNPACKER_META);
        let same = lua_rawequal(state, -1, -2) != 0;
        lua_pop(state, 2);
        same
    }
}

use crate::src::nvim::lua::ffi::lua_getmetatable;

// ---------------------------------------------------------------------------
// Shared argument handling
// ---------------------------------------------------------------------------

/// The `start` argument of `unpacker(str[, start])` and
/// `session:receive(str[, start])`, as a zero-based offset into a `len`-byte
/// string. Always argument 3 in both.
///
/// # Safety
/// `state` must be live holding the caller's arguments.
pub unsafe fn start_offset(state: *mut lua_State, len: size_t) -> size_t {
    unsafe {
        let start = if lua_gettop(state) == 3 {
            luaL_checknumber(state, 3)
        } else {
            1 as lua_Number
        };
        luaL_argcheck(
            state,
            start > 0.0,
            3,
            c"start position must be greater than zero".as_ptr(),
        );
        luaL_argcheck(
            state,
            start.trunc() == start,
            3,
            c"start position must be an integer".as_ptr(),
        );
        luaL_argcheck(
            state,
            start as size_t <= len,
            3,
            c"start position must be less than or equal to the input string length".as_ptr(),
        );
        start as size_t - 1
    }
}

/// The length msgpack should give a value, and — for a table — whether it is
/// an array.
///
/// A Lua table is a msgpack array when its keys are exactly `1..n`. Deciding
/// that costs a full traversal, which is also where the count comes from.
/// An *empty* table tells the caller nothing, so `is_array` is left as the
/// caller set it. Adapted from lua-cmsgpack.
///
/// # Safety
/// `state` must be live with the value on top of the stack.
pub unsafe fn objlen(state: *mut lua_State, is_array: Option<&mut bool>) -> mpack_uint32_t {
    unsafe {
        let top = lua_gettop(state);
        debug_assert!(top != 0);
        let mut len: size_t = 0;
        if lua_type(state, -1) != LUA_TTABLE {
            len = lua_objlen(state, -1);
        } else {
            let mut looks_like_array = true;
            let mut max: size_t = 0;
            lua_pushnil(state);
            while lua_next(state, -2) != 0 {
                lua_pop(state, 1); // the value; the key stays for `lua_next`
                let key = lua_tonumber(state, -1);
                looks_like_array = looks_like_array
                    && lua_type(state, -1) == LUA_TNUMBER
                    && key > 0.0
                    && key as size_t as lua_Number == key;
                if looks_like_array && key as size_t > max {
                    max = key as size_t;
                }
                len += 1;
            }
            if let Some(is_array) = is_array
                && len > 0
            {
                *is_array = looks_like_array && max == len;
            }
        }
        debug_assert_eq!(top, lua_gettop(state));
        // msgpack lengths are 32-bit; upstream saturates rather than failing.
        len.min(mpack_uint32_t::MAX as size_t) as mpack_uint32_t
    }
}

// ---------------------------------------------------------------------------
// Unpacker
// ---------------------------------------------------------------------------

/// `mpack.Unpacker([{ext = <table>}])`.
///
/// # Safety
/// `state` must be a live Lua state holding this function's arguments.
unsafe extern "C-unwind" fn unpacker_new(state: *mut lua_State) -> c_int {
    unsafe {
        if lua_gettop(state) > 1 {
            return luaL_error(state, c"expecting at most 1 table argument".as_ptr());
        }
        let unpacker = lua_newuserdata(state, size_of::<Unpacker>()).cast::<Unpacker>();
        (*unpacker).parser = malloc(size_of::<mpack_parser_t>()).cast::<mpack_parser_t>();
        if (*unpacker).parser.is_null() {
            return luaL_error(state, c"Failed to allocate memory".as_ptr());
        }
        mpack_parser_init((*unpacker).parser, 0);
        (*(*unpacker).parser).data.p = unpacker.cast();
        (*unpacker).string_buffer = core::ptr::null_mut();
        (*unpacker).L = state;
        (*unpacker).unpacking = 0;
        lua_getfield(state, LUA_REGISTRYINDEX, UNPACKER_META);
        lua_setmetatable(state, -2);
        lua_newtable(state);
        (*unpacker).reg = luaL_ref(state, LUA_REGISTRYINDEX);
        (*unpacker).ext = LUA_NOREF;
        lua_getfield(state, LUA_REGISTRYINDEX, EMPTY_DICT_NAME);
        (*unpacker).mtdict = reference(state, (*unpacker).reg);

        if lua_istable(state, 1) {
            lua_getfield(state, 1, c"ext".as_ptr());
            if !lua_isnil(state, -1) {
                if !lua_istable(state, -1) {
                    return luaL_error(state, c"\"ext\" option must be a table".as_ptr());
                }
                shallow_copy(state);
            }
            (*unpacker).ext = reference(state, (*unpacker).reg);
        }
        1
    }
}

/// `__gc`.
///
/// # Safety
/// See [`unpacker_new`].
unsafe extern "C-unwind" fn unpacker_delete(state: *mut lua_State) -> c_int {
    unsafe {
        let unpacker = check_unpacker(state, 1);
        if (*unpacker).ext != LUA_NOREF {
            unreference(state, (*unpacker).reg, (*unpacker).ext);
        }
        luaL_unref(state, LUA_REGISTRYINDEX, (*unpacker).reg);
        free((*unpacker).parser.cast());
        0
    }
}

/// Decode one value out of `[*str, *left)`, leaving it on the Lua stack and
/// advancing the cursor. Answers `MPACK_EOF` when the buffer ran out first.
///
/// Growing the parser here is what lets a document nest arbitrarily deep;
/// `mpack_parse` rolls the buffer back to the token that would not fit, so
/// the retry starts in exactly the same place.
///
/// # Safety
/// `unpacker` must be live; `str`/`left` must describe the remaining input.
pub unsafe fn unpack_into_registry(
    state: *mut lua_State,
    unpacker: *mut Unpacker,
    str: *mut *const c_char,
    left: *mut size_t,
) -> c_int {
    unsafe {
        if (*unpacker).unpacking != 0 {
            return luaL_error(
                state,
                c"Unpacker instance already working. Use another Unpacker or mpack.decode() if you need to decode from the ext handler"
                    .as_ptr(),
            );
        }
        let mut status;
        loop {
            (*unpacker).unpacking = 1;
            status = mpack_parse(
                (*unpacker).parser,
                str,
                left,
                Some(walk::parse_enter),
                Some(walk::parse_exit),
            );
            (*unpacker).unpacking = 0;
            if status != MPACK_NOMEM {
                break;
            }
            (*unpacker).parser = grow_parser((*unpacker).parser);
            if (*unpacker).parser.is_null() {
                return luaL_error(state, c"failed to grow Unpacker capacity".as_ptr());
            }
        }
        if status == crate::src::mpack::mpack_core::MPACK_ERROR as c_int {
            return luaL_error(state, c"invalid msgpack string".as_ptr());
        }
        status
    }
}

/// `unpacker(str[, start])` — the `__call` metamethod.
///
/// Answers the value (or nil, if the buffer held only part of one) and the
/// one-based offset to resume from.
///
/// # Safety
/// See [`unpacker_new`].
unsafe extern "C-unwind" fn unpacker_unpack(state: *mut lua_State) -> c_int {
    unsafe {
        let argc = lua_gettop(state);
        if !(2..=3).contains(&argc) {
            return luaL_error(state, c"expecting between 2 and 3 arguments".as_ptr());
        }
        let unpacker = check_unpacker(state, 1);
        (*unpacker).L = state;
        let mut len: size_t = 0;
        let start = luaL_checklstring(state, 2, &raw mut len);
        let offset = start_offset(state, len);
        let mut str = start.add(offset);
        let mut left = len - offset;

        let status = unpack_into_registry(state, unpacker, &raw mut str, &raw mut left);
        if status == MPACK_EOF as c_int {
            lua_pushnil(state);
        }
        lua_pushinteger(state, str.offset_from(start) as lua_Integer + 1);
        debug_assert_eq!(lua_gettop(state), argc + 2);
        2
    }
}

use crate::src::nvim::types::lua_Integer;

// ---------------------------------------------------------------------------
// Packer
// ---------------------------------------------------------------------------

/// `mpack.Packer([{ext = <table>, is_bin = <bool|fn>}])`.
///
/// # Safety
/// See [`unpacker_new`].
unsafe extern "C-unwind" fn packer_new(state: *mut lua_State) -> c_int {
    unsafe {
        if lua_gettop(state) > 1 {
            return luaL_error(state, c"expecting at most 1 table argument".as_ptr());
        }
        let packer = lua_newuserdata(state, size_of::<Packer>()).cast::<Packer>();
        (*packer).parser = malloc(size_of::<mpack_parser_t>()).cast::<mpack_parser_t>();
        if (*packer).parser.is_null() {
            return luaL_error(state, c"failed to allocate parser memory".as_ptr());
        }
        mpack_parser_init((*packer).parser, 0);
        (*(*packer).parser).data.p = packer.cast();
        (*packer).L = state;
        (*packer).packing = 0;
        (*packer).is_bin = 0;
        (*packer).is_bin_fn = LUA_NOREF;
        lua_getfield(state, LUA_REGISTRYINDEX, PACKER_META);
        lua_setmetatable(state, -2);
        lua_newtable(state);
        (*packer).reg = luaL_ref(state, LUA_REGISTRYINDEX);
        (*packer).ext = LUA_NOREF;
        lua_getfield(state, LUA_REGISTRYINDEX, EMPTY_DICT_NAME);
        (*packer).mtdict = reference(state, (*packer).reg);

        if lua_istable(state, 1) {
            lua_getfield(state, 1, c"ext".as_ptr());
            if !lua_isnil(state, -1) {
                if !lua_istable(state, -1) {
                    return luaL_error(state, c"\"ext\" option must be a table".as_ptr());
                }
                shallow_copy(state);
            }
            (*packer).ext = reference(state, (*packer).reg);

            lua_getfield(state, 1, c"is_bin".as_ptr());
            if lua_isnil(state, -1) {
                lua_pop(state, 1);
            } else {
                let kind = lua_type(state, -1);
                if kind != crate::src::nvim::lua::ffi::LUA_TBOOLEAN && kind != LUA_TFUNCTION {
                    return luaL_error(
                        state,
                        c"\"is_bin\" option must be a boolean or function".as_ptr(),
                    );
                }
                (*packer).is_bin = lua_toboolean(state, -1);
                if kind == LUA_TFUNCTION {
                    (*packer).is_bin_fn = reference(state, (*packer).reg);
                } else {
                    lua_pop(state, 1);
                }
            }
        }
        1
    }
}

/// `__gc`.
///
/// # Safety
/// See [`unpacker_new`].
unsafe extern "C-unwind" fn packer_delete(state: *mut lua_State) -> c_int {
    unsafe {
        let packer = check_packer(state, 1);
        if (*packer).ext != LUA_NOREF {
            unreference(state, (*packer).reg, (*packer).ext);
        }
        luaL_unref(state, LUA_REGISTRYINDEX, (*packer).reg);
        free((*packer).parser.cast());
        0
    }
}

/// A `luaL_Buffer`, zeroed. It is 8 KiB of stack, which is why it is built
/// here rather than at each use.
fn empty_buffer() -> luaL_Buffer {
    luaL_Buffer {
        p: core::ptr::null_mut(),
        lvl: 0,
        L: core::ptr::null_mut(),
        buffer: [0; BUFFER_SIZE],
    }
}

/// Run an encoding walk to completion, appending to `buffer`.
///
/// `mpack_unparse` fills whatever the buffer hands it and answers
/// `MPACK_EOF` when it wants more room, or `MPACK_NOMEM` when the *parser*
/// stack is too shallow. `grow` is asked to fix that and answers null when
/// it will not — either because the allocation failed, or because the caller
/// has nowhere to put a reallocated parser — at which point `on_nomem`
/// raises the Lua error and does not return.
///
/// # Safety
/// `parser` must be a live parser whose walk callbacks are `unparse_*`.
unsafe fn unparse_all(
    state: *mut lua_State,
    buffer: &mut luaL_Buffer,
    parser: &mut *mut mpack_parser_t,
    mut grow: impl FnMut(*mut mpack_parser_t) -> *mut mpack_parser_t,
    on_nomem: impl Fn(*mut lua_State) -> c_int,
) -> c_int {
    unsafe {
        let mut out = luaL_prepbuffer(buffer);
        let mut room = BUFFER_SIZE;
        loop {
            let before = room;
            let status = mpack_unparse(
                *parser,
                &raw mut out,
                &raw mut room,
                Some(walk::unparse_enter),
                Some(walk::unparse_exit),
            );
            if status == MPACK_NOMEM {
                *parser = grow(*parser);
                if parser.is_null() {
                    return on_nomem(state);
                }
            }
            // `luaL_addsize`: the buffer's cursor moves by what was written.
            buffer.p = buffer.p.add(before - room);
            if room == 0 {
                out = luaL_prepbuffer(buffer);
                room = BUFFER_SIZE;
            }
            if status != MPACK_EOF as c_int && status != MPACK_NOMEM {
                return status;
            }
        }
    }
}

/// `packer(value)` — the `__call` metamethod.
///
/// # Safety
/// See [`unpacker_new`].
unsafe extern "C-unwind" fn packer_pack(state: *mut lua_State) -> c_int {
    unsafe {
        let argc = lua_gettop(state);
        if argc != 2 {
            return luaL_error(state, c"expecting exactly 2 arguments".as_ptr());
        }
        let packer = check_packer(state, 1);
        (*packer).L = state;
        (*packer).root = reference(state, (*packer).reg);
        let mut buffer = empty_buffer();
        luaL_buffinit(state, &raw mut buffer);
        if (*packer).packing != 0 {
            return luaL_error(
                state,
                c"Packer instance already working. Use another Packer or mpack.encode() if you need to encode from the ext handler"
                    .as_ptr(),
            );
        }

        (*packer).packing = 1;
        let mut parser = (*packer).parser;
        unparse_all(
            state,
            &mut buffer,
            &mut parser,
            |p| grow_parser(p),
            |state| luaL_error(state, c"Failed to grow Packer capacity".as_ptr()),
        );
        (*packer).parser = parser;
        (*packer).packing = 0;

        unreference(state, (*packer).reg, (*packer).root);
        luaL_pushresult(&raw mut buffer);
        debug_assert_eq!(lua_gettop(state), argc);
        1
    }
}

// ---------------------------------------------------------------------------
// The one-shot shorthands
// ---------------------------------------------------------------------------

/// `mpack.decode(str)`: the whole string, or an error.
///
/// Unlike `Unpacker`, this refuses a partial document and refuses trailing
/// bytes — which is what makes it a *decoder* rather than a stream reader.
///
/// # Safety
/// See [`unpacker_new`].
unsafe extern "C-unwind" fn decode(state: *mut lua_State) -> c_int {
    unsafe {
        if lua_gettop(state) != 1 {
            return luaL_error(state, c"expecting exactly 1 argument".as_ptr());
        }
        let mut len: size_t = 0;
        let mut str = luaL_checklstring(state, 1, &raw mut len);

        let mut parser: mpack_parser_t = core::mem::zeroed();
        let mut unpacker: Unpacker = core::mem::zeroed();
        lua_newtable(state);
        unpacker.reg = luaL_ref(state, LUA_REGISTRYINDEX);
        unpacker.ext = LUA_NOREF;
        unpacker.parser = &raw mut parser;
        mpack_parser_init(unpacker.parser, 0);
        parser.data.p = (&raw mut unpacker).cast();
        unpacker.string_buffer = core::ptr::null_mut();
        unpacker.L = state;
        lua_getfield(state, LUA_REGISTRYINDEX, EMPTY_DICT_NAME);
        unpacker.mtdict = reference(state, unpacker.reg);

        let status = mpack_parse(
            &raw mut parser,
            &raw mut str,
            &raw mut len,
            Some(walk::parse_enter),
            Some(walk::parse_exit),
        );
        luaL_unref(state, LUA_REGISTRYINDEX, unpacker.reg);

        // The depth is fixed here: `decode` has nowhere to put a grown
        // parser, so a document deeper than 32 is an error rather than a
        // reallocation.
        let message = match status {
            MPACK_NOMEM => c"object was too deep to unpack",
            s if s == MPACK_EOF as c_int => c"incomplete msgpack string",
            s if s == crate::src::mpack::mpack_core::MPACK_ERROR as c_int => {
                c"invalid msgpack string"
            }
            _ if len != 0 => c"trailing data in msgpack string",
            _ => return 1,
        };
        luaL_error(state, message.as_ptr())
    }
}

/// `mpack.encode(value)`.
///
/// # Safety
/// See [`unpacker_new`].
unsafe extern "C-unwind" fn encode(state: *mut lua_State) -> c_int {
    unsafe {
        if lua_gettop(state) != 1 {
            return luaL_error(state, c"expecting exactly 1 argument".as_ptr());
        }
        let mut parser: mpack_parser_t = core::mem::zeroed();
        let mut packer: Packer = core::mem::zeroed();
        lua_newtable(state);
        packer.reg = luaL_ref(state, LUA_REGISTRYINDEX);
        packer.ext = LUA_NOREF;
        packer.is_bin_fn = LUA_NOREF;
        packer.parser = &raw mut parser;
        mpack_parser_init(packer.parser, 0);
        parser.data.p = (&raw mut packer).cast();
        packer.is_bin = 0;
        packer.L = state;
        packer.root = reference(state, packer.reg);
        lua_getfield(state, LUA_REGISTRYINDEX, EMPTY_DICT_NAME);
        packer.mtdict = reference(state, packer.reg);

        let mut buffer = empty_buffer();
        luaL_buffinit(state, &raw mut buffer);
        let mut live = packer.parser;
        unparse_all(
            state,
            &mut buffer,
            &mut live,
            // `decode`/`encode` build their parser on the stack, so there is
            // nowhere to put a bigger one: too deep is an error here, not a
            // reallocation.
            |_| core::ptr::null_mut(),
            |state| {
                unreference(state, packer.reg, packer.root);
                luaL_unref(state, LUA_REGISTRYINDEX, packer.reg);
                luaL_error(state, c"object was too deep to pack".as_ptr())
            },
        );

        unreference(state, packer.reg, packer.root);
        luaL_unref(state, LUA_REGISTRYINDEX, packer.reg);
        luaL_pushresult(&raw mut buffer);
        1
    }
}

/// `tostring(mpack.NIL)`.
///
/// # Safety
/// `state` must be live.
unsafe extern "C-unwind" fn nil_tostring(state: *mut lua_State) -> c_int {
    unsafe {
        lua_pushfstring(state, NIL_NAME, lua_topointer(state, 1));
        1
    }
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

static UNPACKER_METHODS: SharedCell<[luaL_Reg; 3]> = luaL_reg_table![
    c"__call" => unpacker_unpack,
    c"__gc" => unpacker_delete,
];

static PACKER_METHODS: SharedCell<[luaL_Reg; 3]> = luaL_reg_table![
    c"__call" => packer_pack,
    c"__gc" => packer_delete,
];

static SESSION_METHODS: SharedCell<[luaL_Reg; 6]> = luaL_reg_table![
    c"receive" => session::receive,
    c"request" => session::request,
    c"reply" => session::reply,
    c"notify" => session::notify,
    c"__gc" => session::delete,
];

static MPACK_FUNCTIONS: SharedCell<[luaL_Reg; 6]> = luaL_reg_table![
    c"Unpacker" => unpacker_new,
    c"Packer" => packer_new,
    c"Session" => session::new,
    c"decode" => decode,
    c"encode" => encode,
];

/// Install a metatable that is also its own `__index`, so the class's
/// methods are reachable on an instance.
///
/// # Safety
/// `state` must be live.
unsafe fn register_class(state: *mut lua_State, name: *const c_char, methods: &[luaL_Reg]) {
    unsafe {
        luaL_newmetatable(state, name);
        lua_pushvalue(state, -1);
        lua_setfield(state, -2, c"__index".as_ptr());
        luaL_register(state, core::ptr::null(), methods.as_ptr());
        lua_pop(state, 1);
    }
}

/// Build the `mpack` module table and leave it on the stack.
///
/// # Safety
/// `state` must be a live Lua state with room for a few values.
pub unsafe extern "C-unwind" fn luaopen_mpack(state: *mut lua_State) -> c_int {
    unsafe {
        register_class(
            state,
            UNPACKER_META,
            UNPACKER_METHODS.ptr().as_ref().unwrap(),
        );
        register_class(state, PACKER_META, PACKER_METHODS.ptr().as_ref().unwrap());
        register_class(state, SESSION_META, SESSION_METHODS.ptr().as_ref().unwrap());

        // One shared userdatum stands for msgpack nil, so that `x ==
        // mpack.NIL` works and survives being stored in a table. It lives in
        // the registry, shared by every state this is opened on.
        lua_getfield(state, LUA_REGISTRYINDEX, NIL_NAME);
        if lua_isnil(state, -1) {
            lua_newuserdata(state, size_of::<*mut core::ffi::c_void>());
            lua_newtable(state);
            lua_pushstring(state, c"__tostring".as_ptr());
            lua_pushcfunction(state, nil_tostring);
            lua_settable(state, -3);
            lua_setmetatable(state, -2);
            lua_setfield(state, LUA_REGISTRYINDEX, NIL_NAME);
        }
        lua_pop(state, 1);

        lua_newtable(state);
        luaL_register(
            state,
            core::ptr::null(),
            MPACK_FUNCTIONS.ptr().cast::<luaL_Reg>(),
        );
        lua_getfield(state, LUA_REGISTRYINDEX, NIL_NAME);
        lua_setfield(state, -2, c"NIL".as_ptr());
        1
    }
}

/// Keep the depth and slot-count defaults visible next to the code that
/// doubles them.
const _: () = {
    assert!(MPACK_MAX_OBJECT_DEPTH == 32);
    assert!(MPACK_RPC_MAX_REQUESTS == 32);
};

const _: c_uint = MPACK_EOF;
const _: c_int = LUA_REFNIL;
const _: mpack_data_t = mpack_data_t {
    p: core::ptr::null_mut(),
};
