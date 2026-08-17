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
//! `node.data[0]` (the object) and `node.data[1]` (a map's pending key).
//! Getting that stack discipline right is the whole difficulty here, and the
//! comments say what is *on* the stack at each step rather than what the
//! call does.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::{
    LUA_NOREF, LUA_REFNIL, Packer, Unpacker, geti, is_nil_sentinel, objlen, push_nil_sentinel,
    reference, unreference,
};
use crate::lua::ffi::{
    LUA_TBOOLEAN, LUA_TFUNCTION, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE, LUA_TUSERDATA, lua_call,
    lua_getmetatable, lua_gettable, lua_isnumber, lua_isstring, lua_newtable, lua_next, lua_pop,
    lua_pushboolean, lua_pushinteger, lua_pushlstring, lua_pushnumber, lua_pushvalue, lua_rawequal,
    lua_rawgeti, lua_remove, lua_replace, lua_setmetatable, lua_settable, lua_toboolean,
    lua_tolstring, lua_tonumber, lua_type, luaL_error,
};
use crate::mpack::conv::{
    mpack_pack_array, mpack_pack_bin, mpack_pack_boolean, mpack_pack_chunk, mpack_pack_ext,
    mpack_pack_map, mpack_pack_nil, mpack_pack_number, mpack_pack_str, mpack_unpack_boolean,
    mpack_unpack_number,
};
use crate::mpack::mpack_core::{
    MPACK_TOKEN_ARRAY, MPACK_TOKEN_BIN, MPACK_TOKEN_BOOLEAN, MPACK_TOKEN_CHUNK, MPACK_TOKEN_EXT,
    MPACK_TOKEN_FLOAT, MPACK_TOKEN_MAP, MPACK_TOKEN_NIL, MPACK_TOKEN_SINT, MPACK_TOKEN_STR,
    MPACK_TOKEN_UINT, to_tok,
};
use crate::mpack::object::parent_of;
use crate::os::libc::{free, malloc, memcpy};
use crate::types::{lua_Number, lua_State, mpack_node_t, mpack_parser_t, size_t};

/// A decoded token becomes a Lua value.
///
/// # Safety
/// `parser` must be an [`Unpacker`]'s, and `node` one of its live frames.
pub unsafe extern "C-unwind" fn parse_enter(parser: *mut mpack_parser_t, node: *mut mpack_node_t) {
    unsafe {
        let unpacker = (*parser).data.p.cast::<Unpacker>();
        let state = (*unpacker).L;
        let tok = (*node).tok;
        match tok.type_0 {
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
                let offset = (*parent_of(node)).pos;
                memcpy(
                    (*unpacker).string_buffer.add(offset).cast(),
                    tok.data.chunk_ptr.cast(),
                    tok.length as size_t,
                );
            }
            MPACK_TOKEN_BIN | MPACK_TOKEN_STR | MPACK_TOKEN_EXT => {
                // Only the header is known here; `parse_exit` pushes the
                // string once every chunk has been copied in.
                (*unpacker).string_buffer = malloc(tok.length as size_t).cast::<c_char>();
                if (*unpacker).string_buffer.is_null() {
                    luaL_error(state, c"Failed to allocate memory".as_ptr());
                }
            }
            MPACK_TOKEN_ARRAY | MPACK_TOKEN_MAP => {
                // The container has to outlive this call, so it goes into the
                // private registry and the node keeps the reference.
                lua_newtable(state);
                (*node).data[0].i = reference(state, (*unpacker).reg) as i64;
            }
            _ => {}
        }
    }
}

/// A finished Lua value is added to whatever container it belongs to.
///
/// # Safety
/// See [`parse_enter`].
pub unsafe extern "C-unwind" fn parse_exit(parser: *mut mpack_parser_t, node: *mut mpack_node_t) {
    unsafe {
        let unpacker = (*parser).data.p.cast::<Unpacker>();
        let state = (*unpacker).L;
        let reg = (*unpacker).reg;
        let tok = (*node).tok;
        let parent = parent_of(node);

        match tok.type_0 {
            MPACK_TOKEN_BIN | MPACK_TOKEN_STR | MPACK_TOKEN_EXT => {
                lua_pushlstring(state, (*unpacker).string_buffer, tok.length as size_t);
                free((*unpacker).string_buffer.cast());
                (*unpacker).string_buffer = core::ptr::null_mut();
                if tok.type_0 == MPACK_TOKEN_EXT && (*unpacker).ext != LUA_NOREF {
                    apply_ext_handler(state, reg, (*unpacker).ext, to_tok(&tok).lo as c_int);
                }
            }
            MPACK_TOKEN_ARRAY | MPACK_TOKEN_MAP => {
                // Take the container back out of the registry and release it.
                geti(state, reg, (*node).data[0].i as c_int);
                unreference(state, reg, (*node).data[0].i as c_int);
                // A map that never saw a key is an *empty dict*, not an empty
                // list; the `vim.empty_dict()` metatable is what says so.
                if (*node).key_visited == 0 && tok.type_0 == MPACK_TOKEN_MAP {
                    geti(state, reg, (*unpacker).mtdict); // [table, mtdict]
                    lua_setmetatable(state, -2); // [table]
                }
            }
            _ => {}
        }

        // Chunks are consumed by their blob and a blob's parent sees only the
        // finished string, so only a container adopts what is on top.
        if !parent.is_null() && (*parent).tok.type_0 < MPACK_TOKEN_BIN {
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
/// `parent` must be a live array or map frame.
unsafe fn store_in_parent(state: *mut lua_State, reg: c_int, parent: *mut mpack_node_t) {
    unsafe {
        geti(state, reg, (*parent).data[0].i as c_int); // [value, container]
        if (*parent).tok.type_0 == MPACK_TOKEN_ARRAY {
            // `pos` was bumped past this element by the pop, so it is already
            // the one-based index.
            lua_pushnumber(state, (*parent).pos as lua_Number);
            lua_pushvalue(state, -3);
            lua_settable(state, -3);
        } else {
            debug_assert_eq!((*parent).tok.type_0, MPACK_TOKEN_MAP);
            if (*parent).key_visited != 0 {
                // This value is a key; park it until its value arrives.
                lua_pushvalue(state, -2);
                (*parent).data[1].i = reference(state, reg) as i64;
            } else {
                geti(state, reg, (*parent).data[1].i as c_int); // [value, map, key]
                unreference(state, reg, (*parent).data[1].i as c_int);
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
/// hands it to `mpack_write`. A container is opened and its elements are
/// fetched on later calls, which is what makes encoding suspendable too.
///
/// # Safety
/// `parser` must be a [`Packer`]'s, and `node` one of its live frames.
pub unsafe extern "C-unwind" fn unparse_enter(
    parser: *mut mpack_parser_t,
    node: *mut mpack_node_t,
) {
    unsafe {
        let packer = (*parser).data.p.cast::<Packer>();
        let state = (*packer).L;
        let reg = (*packer).reg;
        let parent = parent_of(node);

        if parent.is_null() {
            geti(state, reg, (*packer).root);
        } else if !next_child(state, reg, parent, node) {
            // A chunk borrows the parent's string and takes no reference.
            return;
        }

        // [value]
        let value_type = lua_type(state, -1);
        match value_type {
            LUA_TBOOLEAN => (*node).tok = mpack_pack_boolean(lua_toboolean(state, -1) as _),
            LUA_TNUMBER => (*node).tok = mpack_pack_number(lua_tonumber(state, -1)),
            LUA_TSTRING => {
                let binary = is_binary(state, packer);
                let len = objlen(state, None);
                (*node).tok = if binary {
                    mpack_pack_bin(len)
                } else {
                    mpack_pack_str(len)
                };
            }
            LUA_TTABLE => pack_table(state, packer, node),
            LUA_TUSERDATA if is_nil_sentinel(state, -1) => (*node).tok = mpack_pack_nil(),
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
        (*node).data[0].i = reference(state, reg) as i64;
    }
}

/// Fetch the next element of `parent` onto the stack, or fill `node` with a
/// chunk token when the parent is a blob rather than a container.
///
/// Answers false for the chunk case, where the node is already complete.
///
/// # Safety
/// `parent` must be a live frame whose object is still referenced.
unsafe fn next_child(
    state: *mut lua_State,
    reg: c_int,
    parent: *mut mpack_node_t,
    node: *mut mpack_node_t,
) -> bool {
    unsafe {
        geti(state, reg, (*parent).data[0].i as c_int); // [parent]
        if (*parent).tok.type_0 > MPACK_TOKEN_MAP {
            // A string is written as one chunk node borrowing the Lua
            // string's own bytes, which stay alive because the parent's
            // registry reference does.
            let bytes = lua_tolstring(state, -1, core::ptr::null_mut());
            (*node).tok = mpack_pack_chunk(bytes, (*parent).tok.length);
            lua_pop(state, 1);
            return false;
        }

        if (*parent).tok.type_0 == MPACK_TOKEN_ARRAY {
            lua_pushnumber(state, ((*parent).pos + 1) as lua_Number);
            lua_gettable(state, -2); // [parent, element]
        } else if (*parent).tok.type_0 == MPACK_TOKEN_MAP {
            // `lua_next` resumes from the previous key, which is parked in the
            // registry precisely because this call is not the last.
            geti(state, reg, (*parent).data[1].i as c_int); // [parent, prev key]
            let more = lua_next(state, -2); // [parent, key, value]
            debug_assert!(more != 0, "the map reported more pairs than it holds");
            if (*parent).key_visited != 0 {
                // The value's turn. Remember its key for the pair after this
                // one, and leave the value where the key was.
                unreference(state, reg, (*parent).data[1].i as c_int);
                lua_pushvalue(state, -2);
                (*parent).data[1].i = reference(state, reg) as i64;
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

/// Classify a Lua table: an ext value, a cycle, an array or a map.
///
/// # Safety
/// `packer` must be live with the table on top of the stack.
unsafe fn pack_table(state: *mut lua_State, packer: *mut Packer, node: *mut mpack_node_t) {
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
        let mut ancestor = node;
        loop {
            ancestor = parent_of(ancestor);
            if ancestor.is_null() {
                break;
            }
            geti(state, reg, (*ancestor).data[0].i as c_int);
            if lua_rawequal(state, -1, -2) != 0 {
                (*node).tok = mpack_pack_nil();
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
            (*node).tok = mpack_pack_array(len);
        } else {
            (*node).tok = mpack_pack_map(len);
            // `lua_next` starts from nil.
            (*node).data[1].i = LUA_REFNIL as i64;
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
unsafe fn pack_ext(state: *mut lua_State, packer: *mut Packer, node: *mut mpack_node_t) -> bool {
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
        (*node).tok = mpack_pack_ext(ext as c_int, objlen(state, None));
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
pub unsafe extern "C-unwind" fn unparse_exit(parser: *mut mpack_parser_t, node: *mut mpack_node_t) {
    unsafe {
        let packer = (*parser).data.p.cast::<Packer>();
        let state = (*packer).L;
        // A chunk borrows its parent's string and never took a reference.
        if (*node).tok.type_0 != MPACK_TOKEN_CHUNK {
            unreference(state, (*packer).reg, (*node).data[0].i as c_int);
            if (*node).tok.type_0 == MPACK_TOKEN_MAP {
                unreference(state, (*packer).reg, (*node).data[1].i as c_int);
            }
        }
    }
}
