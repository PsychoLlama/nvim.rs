//! The four walk callbacks: msgpack tokens to Lua values and back.
//!
//! [`crate::mpack::object`] drives a stack of nodes and calls in twice
//! per node — once descending, once ascending. These callbacks are what make
//! it mean something: [`LuaDecoder`] builds Lua values from tokens,
//! [`LuaEncoder`] reads tokens out of Lua values.
//!
//! The Lua stack cannot be the walk's own stack, because the walk suspends
//! between calls whenever the input runs out — so every value in flight
//! lives in the instance's private table, referenced from `node.data[0]`
//! (the object) and `node.data[1]` (a map's pending key): the walk's
//! [`NodeRefs`]. Getting that stack discipline right is the whole
//! difficulty here, and the comments say what is *on* the stack at each
//! step rather than what the call does.
//!
//! User code runs inside a walk -- the ext handlers and the `is_bin`
//! predicate -- and runs protected: an error it raises, or a value the
//! walk cannot encode, becomes the walker's [`Failure`], which stops the
//! walk ([`Decode::stopped`]) for the binding to raise once the codec has
//! returned.
//!
//! Ported from libmpack, Copyright (c) 2016 Thiago de Arruda, under the
//! MIT license; the notice is reproduced in licenses/libmpack-LICENSE.txt.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::{geti, is_nil_sentinel, objlen, push_nil_sentinel, reference, unreference};
use crate::lua::ffi::{LUA_NOREF, LUA_REFNIL};
use crate::lua::stack::{At, Kind, Raise, Stack};
use crate::mpack::conv::{
    mpack_pack_array, mpack_pack_bin, mpack_pack_boolean, mpack_pack_chunk, mpack_pack_ext,
    mpack_pack_map, mpack_pack_nil, mpack_pack_number, mpack_pack_str, mpack_unpack_boolean,
    mpack_unpack_number,
};
use crate::mpack::object::{Decode, Encode};
use crate::mpack::token::{Kind as TokenKind, Tok};
use crate::types::lua_Number;

/// The references a node holds: its object, and a map's pending key (or,
/// encoding, the key `lua_next` resumes from).
pub type NodeRefs = [c_int; 2];

type Node = crate::mpack::object::Node<NodeRefs>;

/// A node's token type, as the C compared it.
fn kind(node: &Node) -> u32 {
    node.tok.kind.map_or(0, |kind| kind as u32)
}

const MPACK_TOKEN_NIL: u32 = TokenKind::Nil as u32;
const MPACK_TOKEN_BOOLEAN: u32 = TokenKind::Boolean as u32;
const MPACK_TOKEN_UINT: u32 = TokenKind::Uint as u32;
const MPACK_TOKEN_SINT: u32 = TokenKind::Sint as u32;
const MPACK_TOKEN_FLOAT: u32 = TokenKind::Float as u32;
const MPACK_TOKEN_CHUNK: u32 = TokenKind::Chunk as u32;
const MPACK_TOKEN_ARRAY: u32 = TokenKind::Array as u32;
const MPACK_TOKEN_MAP: u32 = TokenKind::Map as u32;
const MPACK_TOKEN_BIN: u32 = TokenKind::Bin as u32;
const MPACK_TOKEN_STR: u32 = TokenKind::Str as u32;
const MPACK_TOKEN_EXT: u32 = TokenKind::Ext as u32;

/// Why a walk stopped: what the binding raises once the codec returns.
pub enum Failure {
    /// `luaL_error`'s message.
    Message(&'static str),
    /// A Lua error value, parked in the registry by its reference.
    Value(c_int),
}

impl Failure {
    /// Park the error value on top of the stack (what a failed `pcall`
    /// leaves) until the walk has returned.
    fn from_top(stack: &mut Stack) -> Failure {
        Failure::Value(stack.reference_in(At::REGISTRY))
    }

    /// The error to raise, its value (if any) pushed back on top.
    pub fn raise(self, stack: &mut Stack) -> Raise {
        match self {
            Failure::Message(message) => Raise::message(message),
            Failure::Value(handle) => {
                stack.raw_get_index(At::REGISTRY, handle);
                stack.unreference_in(At::REGISTRY, handle);
                Raise::top_value()
            }
        }
    }
}

/// The decoding walk of one [`super::Unpacker`] (or of `mpack.decode`).
pub struct LuaDecoder<'s> {
    stack: &'s mut Stack,
    /// The instance's private table.
    reg: At,
    /// The `ext` option's handler table, or `LUA_NOREF`.
    ext: c_int,
    mtdict: c_int,
    /// The blob being assembled.
    buffer: Vec<u8>,
    failure: Option<Failure>,
}

impl<'s> LuaDecoder<'s> {
    pub fn new(stack: &'s mut Stack, reg: At, ext: c_int, mtdict: c_int, buffer: Vec<u8>) -> Self {
        LuaDecoder {
            stack,
            reg,
            ext,
            mtdict,
            buffer,
            failure: None,
        }
    }

    /// The blob buffer to keep for the next call, and why the walk stopped.
    pub fn finish(self) -> (Vec<u8>, Option<Failure>) {
        (self.buffer, self.failure)
    }
}

impl Decode<NodeRefs> for LuaDecoder<'_> {
    /// A decoded token becomes a Lua value.
    fn enter(&mut self, tok: Tok, parents: &mut [Node], chunk: &[u8]) -> NodeRefs {
        let mut refs = NodeRefs::default();
        let stack = &mut *self.stack;
        match tok.kind.map_or(0, |kind| kind as u32) {
            MPACK_TOKEN_NIL => push_nil_sentinel(stack),
            MPACK_TOKEN_BOOLEAN => stack.push_boolean(mpack_unpack_boolean(tok)),
            MPACK_TOKEN_UINT | MPACK_TOKEN_SINT | MPACK_TOKEN_FLOAT => {
                stack.push_number(mpack_unpack_number(tok));
            }
            MPACK_TOKEN_CHUNK => {
                // A blob's body arrives in pieces; the parent's `pos` is how
                // much of it has landed. The buffer was sized by the header
                // token, which is the frame below this one.
                let offset = parents.last().expect("a chunk has a parent").pos;
                debug_assert_eq!(offset, self.buffer.len(), "chunks arrive in order");
                self.buffer.truncate(offset);
                self.buffer.extend_from_slice(chunk);
            }
            MPACK_TOKEN_BIN | MPACK_TOKEN_STR | MPACK_TOKEN_EXT => {
                // Only the header is known here; `exit` pushes the string
                // once every chunk has been copied in.
                self.buffer.clear();
                self.buffer.reserve(tok.len as usize);
            }
            MPACK_TOKEN_ARRAY | MPACK_TOKEN_MAP => {
                // The container has to outlive this call, so it goes into the
                // private table and the node keeps the reference.
                stack.new_table(0, 0);
                refs[0] = reference(stack, self.reg);
            }
            _ => {}
        }
        refs
    }

    /// A finished Lua value is added to whatever container it belongs to.
    fn exit(&mut self, node: Node, parents: &mut [Node]) {
        // After a failure the walk only unwinds what it had open; the
        // instance is abandoned ("already working"), its table with it.
        if self.failure.is_some() {
            return;
        }
        let reg = self.reg;
        let tok = node.tok;
        match kind(&node) {
            MPACK_TOKEN_BIN | MPACK_TOKEN_STR | MPACK_TOKEN_EXT => {
                self.stack.push_bytes(&self.buffer);
                self.buffer.clear();
                if kind(&node) == MPACK_TOKEN_EXT
                    && self.ext != LUA_NOREF
                    && !self.apply_ext_handler(tok.lo as c_int)
                {
                    // The handler failed: the walk stops here, the string
                    // goes, and the parent is left as it is.
                    self.stack.pop(1);
                    return;
                }
            }
            MPACK_TOKEN_ARRAY | MPACK_TOKEN_MAP => {
                // Take the container back out of the private table.
                geti(self.stack, reg, node.data[0]);
                unreference(self.stack, reg, node.data[0]);
                // A map that never saw a key is an *empty dict*, not an empty
                // list; the `vim.empty_dict()` metatable is what says so.
                if !node.key_visited && kind(&node) == MPACK_TOKEN_MAP {
                    geti(self.stack, reg, self.mtdict); // [table, mtdict]
                    self.stack.set_metatable(At::from_top(2)); // [table]
                }
            }
            _ => {}
        }

        // Chunks are consumed by their blob and a blob's parent sees only the
        // finished string, so only a container adopts what is on top.
        if let Some(parent) = parents.last_mut()
            && kind(parent) < MPACK_TOKEN_BIN
        {
            store_in_parent(self.stack, reg, parent);
        }
    }

    fn stopped(&self) -> bool {
        self.failure.is_some()
    }
}

impl LuaDecoder<'_> {
    /// Hand an `ext` string to the handler registered for its type code, and
    /// replace it with whatever comes back; `false` when the handler raised,
    /// its error now the walk's failure.
    ///
    /// Stack in `[string]`, out `[string-or-handler-result]`.
    fn apply_ext_handler(&mut self, ext_type: c_int) -> bool {
        let stack = &mut *self.stack;
        geti(stack, self.reg, self.ext); // [string, handlers]
        stack.raw_get_index(At::TOP, ext_type); // [string, handlers, handler?]
        if stack.kind(At::TOP) != Kind::Function {
            stack.pop(2); // the nil `rawgeti` pushed, and the handler table
            return true;
        }
        stack.push_integer(ext_type as _); // [string, handlers, fn, type]
        stack.push_value(At::from_top(4)); // [string, handlers, fn, type, string]
        if stack.pcall(2, 1, None).is_err() {
            // [string, handlers, error]
            self.failure = Some(Failure::from_top(stack));
            stack.pop(1);
            return false;
        }
        stack.replace(At::from_top(3)); // [result, handlers]
        stack.pop(1); // the handler table
        true
    }
}

/// Put the value on top of the stack into its parent container, at the
/// position the parent's own bookkeeping says.
///
/// Stack in `[value]`, out `[]`.
fn store_in_parent(stack: &mut Stack, reg: At, parent: &mut Node) {
    geti(stack, reg, parent.data[0]); // [value, container]
    if kind(parent) == MPACK_TOKEN_ARRAY {
        // `pos` was bumped past this element by the pop, so it is already
        // the one-based index.
        stack.push_number(parent.pos as lua_Number);
        stack.push_value(At::from_top(3));
        stack.set_table(At::from_top(3));
    } else {
        debug_assert_eq!(kind(parent), MPACK_TOKEN_MAP);
        if parent.key_visited {
            // This value is a key; park it until its value arrives.
            stack.push_value(At::from_top(2));
            parent.data[1] = reference(stack, reg);
        } else {
            geti(stack, reg, parent.data[1]); // [value, map, key]
            unreference(stack, reg, parent.data[1]);
            stack.push_value(At::from_top(3)); // [value, map, key, value]
            stack.set_table(At::from_top(3));
        }
    }
    stack.pop(2); // the container and the value
}

/// What a [`super::Packer`] was built with.
pub struct PackOptions {
    /// The `ext` option's handler table, or `LUA_NOREF`.
    pub ext: c_int,
    pub mtdict: c_int,
    pub is_bin: bool,
    /// `is_bin` as a predicate, or `LUA_NOREF`.
    pub is_bin_fn: c_int,
}

/// The encoding walk of one [`super::Packer`] (or of `mpack.encode`).
pub struct LuaEncoder<'s> {
    stack: &'s mut Stack,
    /// The instance's private table.
    reg: At,
    /// The value being encoded.
    root: c_int,
    options: PackOptions,
    /// The body of the string being written.
    chunk: Vec<u8>,
    failure: Option<Failure>,
}

impl<'s> LuaEncoder<'s> {
    pub fn new(stack: &'s mut Stack, reg: At, root: c_int, options: PackOptions) -> Self {
        LuaEncoder {
            stack,
            reg,
            root,
            options,
            chunk: Vec::new(),
            failure: None,
        }
    }

    /// Why the walk stopped, if it did.
    pub fn finish(self) -> Option<Failure> {
        self.failure
    }
}

impl Encode<NodeRefs> for LuaEncoder<'_> {
    /// A Lua value becomes the token that opens it.
    ///
    /// Nothing is written here — the token goes into `node.tok` and the walk
    /// hands it to the writer. A container is opened and its elements are
    /// fetched on later calls, which is what makes encoding suspendable too.
    fn enter(&mut self, node: &mut Node, parents: &mut [Node]) {
        let reg = self.reg;
        if let Some(parent) = parents.last_mut() {
            if !next_child(self.stack, reg, parent, node) {
                // A chunk borrows the parent's string and takes no reference.
                return;
            }
        } else {
            geti(self.stack, reg, self.root);
        }

        // [value]
        let value_type = self.stack.kind(At::TOP);
        match value_type {
            Kind::Boolean => {
                node.tok = mpack_pack_boolean(u32::from(self.stack.to_boolean(At::TOP)))
            }
            Kind::Number => node.tok = mpack_pack_number(self.stack.to_number(At::TOP)),
            Kind::String => {
                let binary = self.is_binary();
                let len = objlen(self.stack, None);
                node.tok = if binary {
                    mpack_pack_bin(len)
                } else {
                    mpack_pack_str(len)
                };
            }
            Kind::Table => self.pack_table(node, parents),
            Kind::Userdata if is_nil_sentinel(self.stack, At::TOP) => node.tok = mpack_pack_nil(),
            _ => self.fail(Failure::Message(match value_type {
                // `"can't serialize object of type %d"`, of `lua_type`.
                Kind::None => "can't serialize object of type -1",
                Kind::Nil => "can't serialize object of type 0",
                Kind::LightUserdata => "can't serialize object of type 2",
                Kind::Function => "can't serialize object of type 6",
                Kind::Userdata => "can't serialize object of type 7",
                _ => "can't serialize object of type 8",
            })),
        }
        // The value has to survive until `exit`: a blob's body is read back
        // out of it, and a map's iteration resumes from it.
        node.data[0] = reference(self.stack, reg);
    }

    /// Release the references a finished node holds.
    fn exit(&mut self, node: Node, _parents: &mut [Node]) {
        // A chunk borrows its parent's string and never took a reference.
        if kind(&node) != MPACK_TOKEN_CHUNK {
            unreference(self.stack, self.reg, node.data[0]);
            if kind(&node) == MPACK_TOKEN_MAP {
                unreference(self.stack, self.reg, node.data[1]);
            }
        }
    }

    /// The body of the string a chunk node at the top of the walk borrows,
    /// copied out of the Lua string the parent's reference keeps (the slot
    /// it is read from is popped before the bytes are written).
    fn chunk(&mut self, _node: &Node, parents: &[Node]) -> &[u8] {
        let parent = parents.last().expect("a chunk has a parent");
        geti(self.stack, self.reg, parent.data[0]); // [string]
        let bytes = self.stack.string(At::TOP).unwrap_or_default();
        let len = bytes.len().min(parent.tok.len as usize);
        self.chunk.clear();
        self.chunk.extend_from_slice(&bytes[..len]);
        self.stack.pop(1);
        &self.chunk
    }

    fn stopped(&self) -> bool {
        self.failure.is_some()
    }
}

impl LuaEncoder<'_> {
    /// Record why the walk stops; the first failure is the one raised.
    fn fail(&mut self, failure: Failure) {
        self.failure.get_or_insert(failure);
    }

    /// Whether the string on top should be encoded as `bin` rather than
    /// `str`, honouring the packer's `is_bin` option and its optional
    /// predicate.
    fn is_binary(&mut self) -> bool {
        if !self.options.is_bin {
            return false;
        }
        if self.options.is_bin_fn == LUA_NOREF {
            return true;
        }
        geti(self.stack, self.reg, self.options.is_bin_fn);
        self.stack.push_value(At::from_top(2));
        if self.stack.pcall(1, 1, None).is_err() {
            let failure = Failure::from_top(self.stack);
            self.fail(failure);
            return false;
        }
        let answer = self.stack.to_boolean(At::TOP);
        self.stack.pop(1);
        answer
    }

    /// Classify the Lua table on top: an ext value, a cycle, an array or a
    /// map.
    fn pack_table(&mut self, node: &mut Node, parents: &[Node]) {
        let reg = self.reg;
        let has_meta = self.stack.get_metatable(At::TOP); // [table, metatable?]
        let mut is_empty_dict = false;
        if has_meta && self.options.mtdict != LUA_NOREF {
            geti(self.stack, reg, self.options.mtdict); // [table, metatable, mtdict]
            is_empty_dict = self.stack.raw_equal(At::TOP, At::from_top(2));
            self.stack.pop(1); // [table, metatable]
        }

        // A metatable other than `vim.empty_dict()`'s can select an ext
        // handler, which replaces the table with the string it returns.
        if self.options.ext != LUA_NOREF && has_meta && !is_empty_dict && self.pack_ext(node) {
            return;
        }
        if has_meta {
            self.stack.pop(1); // [table]
        }

        // A table that is already open below is a cycle; msgpack has no way
        // to spell one, so it is encoded as nil.
        for ancestor in parents.iter().rev() {
            geti(self.stack, reg, ancestor.data[0]);
            if self.stack.raw_equal(At::TOP, At::from_top(2)) {
                node.tok = mpack_pack_nil();
                self.stack.pop(2);
                push_nil_sentinel(self.stack);
                return;
            }
            self.stack.pop(1);
        }

        // `objlen` decides array-or-map from the keys; an empty table is
        // whatever the metatable said it was.
        let mut is_array = !is_empty_dict;
        let len = objlen(self.stack, Some(&mut is_array));
        if is_array {
            node.tok = mpack_pack_array(len);
        } else {
            node.tok = mpack_pack_map(len);
            // `lua_next` starts from nil.
            node.data[1] = LUA_REFNIL;
        }
    }

    /// Run the ext handler registered for this table's metatable, if there
    /// is one; answer whether it fired (or failed, which also ends the
    /// node).
    ///
    /// Stack in `[table, metatable]`. On a hit (or a failure), out `[ext
    /// string]` (or `[table]`); on a miss, unchanged — which takes **two**
    /// pops, not upstream's one. See O-B15-11.
    fn pack_ext(&mut self, node: &mut Node) -> bool {
        let stack = &mut *self.stack;
        geti(stack, self.reg, self.options.ext); // [table, meta, handlers]
        stack.push_value(At::from_top(2)); // [table, meta, handlers, meta]
        stack.get_table(At::from_top(2)); // [table, meta, handlers, handler?]
        if stack.kind(At::TOP) != Kind::Function {
            // The handler table has to go as well: the caller's `if has_meta`
            // pop is for the metatable, and taking that off the top instead
            // leaves the metatable where the table should be.
            stack.pop(2); // [table, meta]
            return false;
        }

        stack.push_value(At::from_top(4)); // [table, meta, handlers, fn, table]
        if stack.pcall(1, 2, None).is_err() {
            // [table, meta, handlers, error]
            let failure = Failure::from_top(stack);
            stack.pop(2); // [table]
            self.fail(failure);
            return true;
        }
        // [table, meta, handlers, type, string]
        let ext = stack.to_number(At::from_top(2));
        let refusal = if !stack.is_number(At::from_top(2))
            || !(0.0..=127.0).contains(&ext)
            || ext.trunc() != ext
        {
            Some("the first result from ext packer must be an integer between 0 and 127")
        } else if !stack.is_string(At::TOP) {
            Some("the second result from ext packer must be a string")
        } else {
            None
        };
        if let Some(message) = refusal {
            stack.pop(4); // [table]
            self.fail(Failure::Message(message));
            return true;
        }
        node.tok = mpack_pack_ext(ext as c_int, objlen(stack, None));
        // Leave only the returned string, where the table was.
        stack.replace(At::from_top(5));
        stack.pop(3);
        true
    }
}

/// Fetch the next element of `parent` onto the stack, or fill `node` with a
/// chunk token when the parent is a blob rather than a container.
///
/// Answers false for the chunk case, where the node is already complete.
fn next_child(stack: &mut Stack, reg: At, parent: &mut Node, node: &mut Node) -> bool {
    if kind(parent) > MPACK_TOKEN_MAP {
        // A string is written as one chunk node borrowing the Lua string's
        // own bytes, which stay alive because the parent's reference does;
        // `chunk` reads them.
        node.tok = mpack_pack_chunk(parent.tok.len);
        return false;
    }
    geti(stack, reg, parent.data[0]); // [parent]

    if kind(parent) == MPACK_TOKEN_ARRAY {
        stack.push_number((parent.pos + 1) as lua_Number);
        stack.get_table(At::from_top(2)); // [parent, element]
    } else if kind(parent) == MPACK_TOKEN_MAP {
        // `lua_next` resumes from the previous key, which is parked in the
        // private table precisely because this call is not the last.
        geti(stack, reg, parent.data[1]); // [parent, prev key]
        let more = stack.next(At::from_top(2)); // [parent, key, value]
        debug_assert!(more, "the map reported more pairs than it holds");
        if parent.key_visited {
            // The value's turn. Remember its key for the pair after this
            // one, and leave the value where the key was.
            unreference(stack, reg, parent.data[1]);
            stack.push_value(At::from_top(2));
            parent.data[1] = reference(stack, reg);
            stack.replace(At::from_top(2)); // [parent, value]
        } else {
            stack.pop(1); // [parent, key] -- the key's turn
        }
    }
    stack.remove(At::from_top(2)); // [element]
    true
}
