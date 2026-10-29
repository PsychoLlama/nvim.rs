//! The four walk callbacks: msgpack tokens to Lua values and back.
//!
//! [`crate::mpack::object`] drives a stack of nodes and calls in twice
//! per node — once descending, once ascending. These callbacks are what make
//! it mean something: [`parse_enter`]/[`parse_exit`] build Lua values from
//! tokens, [`unparse_enter`]/[`unparse_exit`] read tokens out of Lua values.
//!
//! The Lua stack cannot be the walk's own stack, because the walk suspends
//! between calls whenever the input runs out — so every value in flight
//! lives in the instance's private registry table, referenced from
//! `node.data[0]` (the object) and `node.data[1]` (a map's pending key):
//! the walk's [`NodeRefs`].
//! Getting that stack discipline right is the whole difficulty here, and the
//! comments say what is *on* the stack at each step rather than what the
//! call does.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `mpack/lmpack/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use core::ffi::{c_char, c_int};

use super::{
    LUA_NOREF, LUA_REFNIL, Packer, Unpacker, geti, is_nil_sentinel, objlen, push_nil_sentinel,
    reference, unreference,
};
use crate::lua::ffi::{
    LUA_TBOOLEAN, LUA_TFUNCTION, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE, LUA_TUSERDATA, lua_call,
    lua_getmetatable, lua_gettable, lua_isnumber, lua_isstring, lua_next, lua_pushboolean,
    lua_pushinteger, lua_pushlstring, lua_pushnumber, lua_pushvalue, lua_rawequal, lua_rawgeti,
    lua_remove, lua_replace, lua_setmetatable, lua_settable, lua_toboolean, lua_tolstring,
    lua_tonumber, lua_type, luaL_error,
};
use crate::lua::stack::{lua_newtable, lua_pop};
use crate::mpack::conv::{
    mpack_pack_array, mpack_pack_bin, mpack_pack_boolean, mpack_pack_chunk, mpack_pack_ext,
    mpack_pack_map, mpack_pack_nil, mpack_pack_number, mpack_pack_str, mpack_unpack_boolean,
    mpack_unpack_number,
};
use crate::mpack::object::{Decode, Encode};
use crate::mpack::token::{Kind, Tok};
use crate::types::{lua_Number, lua_State, size_t};
use ::libc::{free, malloc};

/// The registry references a node holds: its object, and a map's pending
/// key (or, encoding, the key `lua_next` resumes from).
pub type NodeRefs = [c_int; 2];

type Node = crate::mpack::object::Node<NodeRefs>;

/// A node's token type, as the C compared it.
fn kind(node: &Node) -> u32 {
    node.tok.kind.map_or(0, |kind| kind as u32)
}

const MPACK_TOKEN_NIL: u32 = Kind::Nil as u32;
const MPACK_TOKEN_BOOLEAN: u32 = Kind::Boolean as u32;
const MPACK_TOKEN_UINT: u32 = Kind::Uint as u32;
const MPACK_TOKEN_SINT: u32 = Kind::Sint as u32;
const MPACK_TOKEN_FLOAT: u32 = Kind::Float as u32;
const MPACK_TOKEN_CHUNK: u32 = Kind::Chunk as u32;
const MPACK_TOKEN_ARRAY: u32 = Kind::Array as u32;
const MPACK_TOKEN_MAP: u32 = Kind::Map as u32;
const MPACK_TOKEN_BIN: u32 = Kind::Bin as u32;
const MPACK_TOKEN_STR: u32 = Kind::Str as u32;
const MPACK_TOKEN_EXT: u32 = Kind::Ext as u32;

/// The decoding walk of one [`Unpacker`].
pub struct LuaDecoder(pub *mut Unpacker);

/// The encoding walk of one [`Packer`].
pub struct LuaEncoder(pub *mut Packer);

impl Decode<NodeRefs> for LuaDecoder {
    fn enter(&mut self, tok: Tok, parents: &mut [Node], chunk: &[u8]) -> NodeRefs {
        // SAFETY: the unpacker that started this walk, live for its length.
        unsafe { parse_enter(self.0, tok, parents, chunk) }
    }

    fn exit(&mut self, node: Node, parents: &mut [Node]) {
        // SAFETY: as above.
        unsafe { parse_exit(self.0, &node, parents) }
    }
}

impl Encode<NodeRefs> for LuaEncoder {
    fn enter(&mut self, node: &mut Node, parents: &mut [Node]) {
        // SAFETY: the packer that started this walk, live for its length.
        unsafe { unparse_enter(self.0, node, parents) }
    }

    fn exit(&mut self, node: Node, _parents: &mut [Node]) {
        // SAFETY: as above.
        unsafe { unparse_exit(self.0, &node) }
    }

    fn chunk(&mut self, _node: &Node, parents: &[Node]) -> &[u8] {
        // SAFETY: as above; a chunk's parent is the string it borrows.
        unsafe { chunk_bytes(self.0, parents) }
    }
}

/// A decoded token becomes a Lua value.
///
/// # Safety
/// `unpacker` must be the live instance walking.
unsafe fn parse_enter(
    unpacker: *mut Unpacker,
    tok: Tok,
    parents: &[Node],
    chunk: &[u8],
) -> NodeRefs {
    let mut refs = NodeRefs::default();
    unsafe {
        let state = (*unpacker).L;
        match tok.kind.map_or(0, |kind| kind as u32) {
            MPACK_TOKEN_NIL => push_nil_sentinel(state),
            MPACK_TOKEN_BOOLEAN => {
                lua_pushboolean(state, c_int::from(mpack_unpack_boolean(tok)));
            }
            MPACK_TOKEN_UINT | MPACK_TOKEN_SINT | MPACK_TOKEN_FLOAT => {
                lua_pushnumber(state, mpack_unpack_number(tok));
            }
            MPACK_TOKEN_CHUNK => {
                // A blob's body arrives in pieces; the parent's `pos` is how
                // much of it has landed. The buffer was sized by the header
                // token, which is the frame below this one.
                debug_assert!(!(*unpacker).string_buffer.is_null());
                let offset = parents.last().expect("a chunk has a parent").pos;
                (*unpacker)
                    .string_buffer
                    .add(offset)
                    .cast::<u8>()
                    .copy_from_nonoverlapping(chunk.as_ptr(), chunk.len());
            }
            MPACK_TOKEN_BIN | MPACK_TOKEN_STR | MPACK_TOKEN_EXT => {
                // Only the header is known here; `parse_exit` pushes the
                // string once every chunk has been copied in.
                (*unpacker).string_buffer = malloc(tok.len as size_t).cast::<c_char>();
                if (*unpacker).string_buffer.is_null() {
                    luaL_error(state, c"Failed to allocate memory".as_ptr());
                }
            }
            MPACK_TOKEN_ARRAY | MPACK_TOKEN_MAP => {
                // The container has to outlive this call, so it goes into the
                // private registry and the node keeps the reference.
                lua_newtable(state);
                refs[0] = reference(state, (*unpacker).reg);
            }
            _ => {}
        }
    }
    refs
}

/// A finished Lua value is added to whatever container it belongs to.
///
/// # Safety
/// See [`parse_enter`].
unsafe fn parse_exit(unpacker: *mut Unpacker, node: &Node, parents: &mut [Node]) {
    unsafe {
        let state = (*unpacker).L;
        let reg = (*unpacker).reg;
        let tok = node.tok;
        let parent = parents.last_mut();

        match kind(node) {
            MPACK_TOKEN_BIN | MPACK_TOKEN_STR | MPACK_TOKEN_EXT => {
                lua_pushlstring(state, (*unpacker).string_buffer, tok.len as size_t);
                free((*unpacker).string_buffer.cast());
                (*unpacker).string_buffer = core::ptr::null_mut();
                if kind(node) == MPACK_TOKEN_EXT && (*unpacker).ext != LUA_NOREF {
                    apply_ext_handler(state, reg, (*unpacker).ext, tok.lo as c_int);
                }
            }
            MPACK_TOKEN_ARRAY | MPACK_TOKEN_MAP => {
                // Take the container back out of the registry and release it.
                geti(state, reg, node.data[0]);
                unreference(state, reg, node.data[0]);
                // A map that never saw a key is an *empty dict*, not an empty
                // list; the `vim.empty_dict()` metatable is what says so.
                if !node.key_visited && kind(node) == MPACK_TOKEN_MAP {
                    geti(state, reg, (*unpacker).mtdict); // [table, mtdict]
                    lua_setmetatable(state, -2); // [table]
                }
            }
            _ => {}
        }

        // Chunks are consumed by their blob and a blob's parent sees only the
        // finished string, so only a container adopts what is on top.
        if let Some(parent) = parent
            && kind(parent) < MPACK_TOKEN_BIN
        {
            store_in_parent(state, reg, parent);
        }
    }
}

/// Hand an `ext` string to the handler registered for its type code, and
/// replace it with whatever comes back.
///
/// Stack in `[string]`, out `[string-or-handler-result]`.
///
/// # Safety
/// `state` must be live and `handlers` a reference to a table in `reg`.
unsafe fn apply_ext_handler(state: *mut lua_State, reg: c_int, handlers: c_int, ext_type: c_int) {
    unsafe {
        geti(state, reg, handlers); // [string, handlers]
        lua_rawgeti(state, -1, ext_type); // [string, handlers, handler?]
        if lua_type(state, -1) == LUA_TFUNCTION {
            lua_pushinteger(state, ext_type as _); // [string, handlers, fn, type]
            lua_pushvalue(state, -4); // [string, handlers, fn, type, string]
            lua_call(state, 2, 1); // [string, handlers, result]
            lua_replace(state, -3); // [result, handlers]
        } else {
            lua_pop(state, 1); // the nil `rawgeti` pushed
        }
        lua_pop(state, 1); // the handler table
    }
}

/// Put the value on top of the stack into its parent container, at the
/// position the parent's own bookkeeping says.
///
/// Stack in `[value]`, out `[]`.
///
/// # Safety
/// `state` must be live with the value on top, and `parent` an array or map
/// whose container `reg` still holds.
unsafe fn store_in_parent(state: *mut lua_State, reg: c_int, parent: &mut Node) {
    unsafe {
        geti(state, reg, parent.data[0]); // [value, container]
        if kind(parent) == MPACK_TOKEN_ARRAY {
            // `pos` was bumped past this element by the pop, so it is already
            // the one-based index.
            lua_pushnumber(state, parent.pos as lua_Number);
            lua_pushvalue(state, -3);
            lua_settable(state, -3);
        } else {
            debug_assert_eq!(kind(parent), MPACK_TOKEN_MAP);
            if parent.key_visited {
                // This value is a key; park it until its value arrives.
                lua_pushvalue(state, -2);
                parent.data[1] = reference(state, reg);
            } else {
                geti(state, reg, parent.data[1]); // [value, map, key]
                unreference(state, reg, parent.data[1]);
                lua_pushvalue(state, -3); // [value, map, key, value]
                lua_settable(state, -3);
            }
        }
        lua_pop(state, 2); // the container and the value
    }
}

/// A Lua value becomes the token that opens it.
///
/// Nothing is written here — the token goes into `node.tok` and the walk
/// hands it to the writer. A container is opened and its elements are
/// fetched on later calls, which is what makes encoding suspendable too.
///
/// # Safety
/// `packer` must be the live instance walking.
unsafe fn unparse_enter(packer: *mut Packer, node: &mut Node, parents: &mut [Node]) {
    unsafe {
        let state = (*packer).L;
        let reg = (*packer).reg;

        if let Some(parent) = parents.last_mut() {
            if !next_child(state, reg, parent, node) {
                // A chunk borrows the parent's string and takes no reference.
                return;
            }
        } else {
            geti(state, reg, (*packer).root);
        }

        // [value]
        let value_type = lua_type(state, -1);
        match value_type {
            LUA_TBOOLEAN => node.tok = mpack_pack_boolean(lua_toboolean(state, -1) as _),
            LUA_TNUMBER => node.tok = mpack_pack_number(lua_tonumber(state, -1)),
            LUA_TSTRING => {
                let binary = is_binary(state, packer);
                let len = objlen(state, None);
                node.tok = if binary {
                    mpack_pack_bin(len)
                } else {
                    mpack_pack_str(len)
                };
            }
            LUA_TTABLE => pack_table(state, packer, node, parents),
            LUA_TUSERDATA if is_nil_sentinel(state, -1) => node.tok = mpack_pack_nil(),
            _ => {
                luaL_error(
                    state,
                    c"can't serialize object of type %d".as_ptr(),
                    value_type,
                );
            }
        }
        // The value has to survive until `unparse_exit`: a blob's body is read
        // back out of it, and a map's iteration resumes from it.
        node.data[0] = reference(state, reg);
    }
}

/// Fetch the next element of `parent` onto the stack, or fill `node` with a
/// chunk token when the parent is a blob rather than a container.
///
/// Answers false for the chunk case, where the node is already complete.
///
/// # Safety
/// `parent`'s object must still be referenced in `reg`.
unsafe fn next_child(
    state: *mut lua_State,
    reg: c_int,
    parent: &mut Node,
    node: &mut Node,
) -> bool {
    unsafe {
        if kind(parent) > MPACK_TOKEN_MAP {
            // A string is written as one chunk node borrowing the Lua
            // string's own bytes, which stay alive because the parent's
            // registry reference does; [`chunk_bytes`] reads them.
            node.tok = mpack_pack_chunk(parent.tok.len);
            return false;
        }
        geti(state, reg, parent.data[0]); // [parent]

        if kind(parent) == MPACK_TOKEN_ARRAY {
            lua_pushnumber(state, (parent.pos + 1) as lua_Number);
            lua_gettable(state, -2); // [parent, element]
        } else if kind(parent) == MPACK_TOKEN_MAP {
            // `lua_next` resumes from the previous key, which is parked in the
            // registry precisely because this call is not the last.
            geti(state, reg, parent.data[1]); // [parent, prev key]
            let more = lua_next(state, -2); // [parent, key, value]
            debug_assert!(more != 0, "the map reported more pairs than it holds");
            if parent.key_visited {
                // The value's turn. Remember its key for the pair after this
                // one, and leave the value where the key was.
                unreference(state, reg, parent.data[1]);
                lua_pushvalue(state, -2);
                parent.data[1] = reference(state, reg);
                lua_replace(state, -2); // [parent, value]
            } else {
                lua_pop(state, 1); // [parent, key] -- the key's turn
            }
        }
        lua_remove(state, -2); // [element]
        true
    }
}

/// Whether a string should be encoded as `bin` rather than `str`, honouring
/// the packer's `is_bin` option and its optional predicate.
///
/// # Safety
/// `packer` must be live with its string on top of the stack.
unsafe fn is_binary(state: *mut lua_State, packer: *mut Packer) -> bool {
    unsafe {
        if (*packer).is_bin == 0 {
            return false;
        }
        if (*packer).is_bin_fn == LUA_NOREF {
            return true;
        }
        geti(state, (*packer).reg, (*packer).is_bin_fn);
        lua_pushvalue(state, -2);
        lua_call(state, 1, 1);
        let answer = lua_toboolean(state, -1) != 0;
        lua_pop(state, 1);
        answer
    }
}

/// The body of the string a chunk node at the top of the walk borrows.
///
/// # Safety
/// `packer` must be live, and the last of `parents` the string node whose
/// object `reg` still holds.
unsafe fn chunk_bytes<'a>(packer: *mut Packer, parents: &[Node]) -> &'a [u8] {
    unsafe {
        let state = (*packer).L;
        let parent = parents.last().expect("a chunk has a parent");
        geti(state, (*packer).reg, parent.data[0]); // [string]
        let mut len: size_t = 0;
        let bytes = lua_tolstring(state, -1, &raw mut len);
        lua_pop(state, 1);
        // The parent's reference keeps the string, and so these bytes, alive
        // until its node exits; the chunk is written before then.
        core::slice::from_raw_parts(bytes.cast::<u8>(), len.min(parent.tok.len as size_t))
    }
}

/// Classify a Lua table: an ext value, a cycle, an array or a map.
///
/// # Safety
/// `packer` must be live with the table on top of the stack.
unsafe fn pack_table(
    state: *mut lua_State,
    packer: *mut Packer,
    node: &mut Node,
    parents: &[Node],
) {
    unsafe {
        let reg = (*packer).reg;
        let has_meta = lua_getmetatable(state, -1) != 0; // [table, metatable?]
        let mut is_empty_dict = false;
        if has_meta && (*packer).mtdict != LUA_NOREF {
            geti(state, reg, (*packer).mtdict); // [table, metatable, mtdict]
            is_empty_dict = lua_rawequal(state, -1, -2) != 0;
            lua_pop(state, 1); // [table, metatable]
        }

        // A metatable other than `vim.empty_dict()`'s can select an ext
        // handler, which replaces the table with the string it returns.
        if (*packer).ext != LUA_NOREF && has_meta && !is_empty_dict && pack_ext(state, packer, node)
        {
            return;
        }
        if has_meta {
            lua_pop(state, 1); // [table]
        }

        // A table that is already open below is a cycle; msgpack has no way
        // to spell one, so it is encoded as nil.
        for ancestor in parents.iter().rev() {
            geti(state, reg, ancestor.data[0]);
            if lua_rawequal(state, -1, -2) != 0 {
                node.tok = mpack_pack_nil();
                lua_pop(state, 2);
                push_nil_sentinel(state);
                return;
            }
            lua_pop(state, 1);
        }

        // `objlen` decides array-or-map from the keys; an empty table is
        // whatever the metatable said it was.
        let mut is_array = !is_empty_dict;
        let len = objlen(state, Some(&mut is_array));
        if is_array {
            node.tok = mpack_pack_array(len);
        } else {
            node.tok = mpack_pack_map(len);
            // `lua_next` starts from nil.
            node.data[1] = LUA_REFNIL;
        }
    }
}

/// Run the ext handler registered for this table's metatable, if there is
/// one; answer whether it fired.
///
/// Stack in `[table, metatable]`. On a hit, out `[ext string]`; on a miss,
/// unchanged — which takes **two** pops, not upstream's one. See O-B15-11.
///
/// # Safety
/// `packer` must be live with a table and its metatable on top.
unsafe fn pack_ext(state: *mut lua_State, packer: *mut Packer, node: &mut Node) -> bool {
    unsafe {
        geti(state, (*packer).reg, (*packer).ext); // [table, meta, handlers]
        lua_pushvalue(state, -2); // [table, meta, handlers, meta]
        lua_gettable(state, -2); // [table, meta, handlers, handler?]
        if lua_type(state, -1) != LUA_TFUNCTION {
            // The handler table has to go as well: the caller's `if has_meta`
            // pop is for the metatable, and taking that off the top instead
            // leaves the metatable where the table should be.
            lua_pop(state, 2); // [table, meta]
            return false;
        }

        lua_pushvalue(state, -4); // [table, meta, handlers, fn, table]
        lua_call(state, 1, 2); // [table, meta, handlers, type, string]
        let ext = lua_tonumber(state, -2);
        if lua_isnumber(state, -2) == 0 || !(0.0..=127.0).contains(&ext) || ext.trunc() != ext {
            luaL_error(
                state,
                c"the first result from ext packer must be an integer between 0 and 127".as_ptr(),
            );
        }
        if lua_isstring(state, -1) == 0 {
            luaL_error(
                state,
                c"the second result from ext packer must be a string".as_ptr(),
            );
        }
        node.tok = mpack_pack_ext(ext as c_int, objlen(state, None));
        // Leave only the returned string, where the table was.
        lua_replace(state, -5);
        lua_pop(state, 3);
        true
    }
}

/// Release the registry references a finished node holds.
///
/// # Safety
/// See [`unparse_enter`].
unsafe fn unparse_exit(packer: *mut Packer, node: &Node) {
    unsafe {
        let state = (*packer).L;
        // A chunk borrows its parent's string and never took a reference.
        if kind(node) != MPACK_TOKEN_CHUNK {
            unreference(state, (*packer).reg, node.data[0]);
            if kind(node) == MPACK_TOKEN_MAP {
                unreference(state, (*packer).reg, node.data[1]);
            }
        }
    }
}
