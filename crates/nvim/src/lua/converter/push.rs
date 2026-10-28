//! `push_typval()`: a Vimscript value as a Lua one.
//!
//! One [`TypvalSink`] over a [`Stack`], replacing the `TYPVAL_ENCODE_NAME
//! lua` instantiation of `typval_encode.c.h`.  Every hook pushes exactly one
//! Lua value, and the containers are assembled on the Lua stack itself rather
//! than in a buffer of the sink's own: a list holds its table *and* the index
//! it is about to write, a dictionary its table *and* the pending key, so each
//! open container occupies **two** Lua slots.  That is what
//! [`LuaSink::backref`]'s `* 2` counts.
//!
//! Because the stack is Lua's, this is the one sink that can fail for want of
//! room: `lua_checkstack` is asked before every container and `E5102` is the
//! refusal.
//!
//! It reads `{_TYPE, _VAL}` special dictionaries (`ALLOW_SPECIALS`), so a value
//! that came from msgpack keeps its identity through Vimscript and back.

#![forbid(unsafe_code)]

use crate::semsg;
use core::ffi::{CStr, c_int};

use super::create_typed_table;
use crate::eval::typval_encode::{Container, ConvPath, ConvType, Flow, TypvalSink, encode_typval};
use crate::eval::userfunc::FuncFlags;
use crate::eval::userfunc::find_func;
use crate::lua::stack::{At, Stack};
use crate::types::{Float, TypVal, kObjectTypeDict, lua_Number};

/// How many Lua slots opening a container needs: its table, the key or index
/// it is about to set, and the value that will land on top of them.
const CONTAINER_SLOTS: c_int = 3;

/// The `push_typval()` sink.
struct LuaSink<'a> {
    stack: &'a mut Stack,
    /// Push `nil` and a typed table for Vimscript's `null` and empty
    /// dictionary, rather than the `vim.NIL` and `vim.empty_dict()`
    /// singletons Lua code normally sees.
    special: bool,
}

/// The few compound steps the hooks below are made of; everything else is
/// one [`Stack`] call.
impl LuaSink<'_> {
    /// Store the value on top of the stack under the key below it, in the
    /// table below that.
    fn rawset(&mut self) {
        self.stack.raw_set(At::from_top(3));
    }

    /// Push whatever stands for "no value": upstream's `TYPVAL_ENCODE_CONV_NIL`.
    fn push_nil(&mut self) {
        if self.special {
            self.stack.push_nil();
        } else {
            self.stack.push_vim_nil();
        }
    }

    /// Make room for one more open container, or refuse with `E5102`.
    fn check_stack(&mut self) -> Flow {
        let wanted = self.stack.top().index() + CONTAINER_SLOTS;
        if !self.stack.reserve(wanted) {
            semsg!("E5102: Lua failed to grow stack to {}", wanted);
            return Flow::Fail;
        }
        Flow::Go
    }

    /// Where on the Lua stack the container being re-entered is already being
    /// built, as an index from the top.
    ///
    /// Every enclosing container occupies two slots, so the frame `n` levels
    /// down from the top sits `2n` below it.  Unlike the text sinks' marker search,
    /// this one *does* match a `Pairs` frame: upstream's ternary here reads
    /// `data.l.list` for everything that is not `kMPConvDict`, and a special
    /// map's `_VAL` list lives in that member.
    ///
    /// `None` — no frame matched — is a list met as a `List` that a `Pairs`
    /// frame holds, and upstream pushes nothing at all in that case, leaving
    /// the value it promised missing.
    fn backref(
        path: &ConvPath<'_, '_>,
        container: Container<'_>,
        conv_type: ConvType,
    ) -> Option<At> {
        let depth = path.stack.len();
        // Upstream scans from the top down; a container can be on the stack
        // only once, so scanning up finds the same frame.
        let found = path
            .stack
            .iter()
            .position(|frame| frame.walks(conv_type, container))?;
        Some(At::from_top(((depth - found) * 2) as c_int))
    }
}

impl TypvalSink for LuaSink<'_> {
    const ALLOW_SPECIALS: bool = true;
    const CONVERT_FN_NAME: &'static CStr = c"_typval_encode_lua_convert_one_value()";

    fn conv_nil(&mut self) {
        self.push_nil();
    }

    fn conv_bool(&mut self, num: bool) {
        self.stack.push_boolean(num);
    }

    fn conv_number(&mut self, num: i64) {
        self.stack.push_number(num as lua_Number);
    }

    fn conv_unsigned_number(&mut self, num: u64) {
        self.stack.push_number(num as lua_Number);
    }

    fn conv_float(&mut self, flt: Float) -> Flow {
        self.stack.push_number(flt);
        Flow::Go
    }

    /// A Lua string is bytes, so this is the whole of it — NULs included.  It
    /// copies, which is why the walk's buffer-owning hooks need no override.
    fn conv_string(&mut self, bytes: &[u8]) -> Flow {
        self.stack.push_bytes(bytes);
        Flow::Go
    }

    /// msgpack `ext` has no Lua image, so it comes out as nil.
    fn conv_ext_string(&mut self, _bytes: &[u8], _ext_type: i8) -> Flow {
        self.push_nil();
        Flow::Go
    }

    /// A blob is bytes, and so is a Lua string.
    fn conv_blob(&mut self, bytes: &[u8]) {
        self.stack.push_bytes(bytes);
    }

    /// A Lua function that reached Vimscript as a funcref goes back as the
    /// same function; anything else is nil.  Either way the walk stops here,
    /// so a partial's arguments and self dictionary are never visited — which
    /// is why no `Partial` frame ever reaches [`LuaSink::backref`].
    fn conv_func_start(
        &mut self,
        fun: Option<&CStr>,
        _prefix: &'static CStr,
        _path: &ConvPath<'_, '_>,
    ) -> Flow {
        let luaref = fun
            .and_then(|fun| find_func(fun.to_bytes()))
            .filter(|func| func.has_flag(FuncFlags::LUAREF))
            .map(|func| func.luaref.get());
        match luaref {
            Some(reference) => self.stack.push_reference(reference),
            None => self.push_nil(),
        }
        Flow::Stop
    }

    fn conv_empty_list(&mut self) {
        self.stack.new_table(0, 0);
    }

    /// An empty table is ambiguous in Lua, so an empty dictionary carries a
    /// marker: the `vim.empty_dict()` metatable, or the `_TYPE` key when the
    /// caller asked for the special form.
    fn conv_empty_dict(&mut self) {
        if self.special {
            create_typed_table(self.stack, 0, 0, kObjectTypeDict);
        } else {
            self.stack.new_table(0, 0);
            self.stack.push_empty_dict_metatable();
            self.stack.set_metatable(At::from_top(2));
        }
    }

    /// The table, then the index its first item will be stored under.
    ///
    fn conv_list_start(&mut self, len: c_int) -> Flow {
        if self.check_stack() == Flow::Fail {
            return Flow::Fail;
        }
        self.stack.new_table(usize::try_from(len).unwrap_or(0), 0);
        self.stack.push_number(1.0);
        Flow::Go
    }

    /// Store the item just converted and push the next index.
    ///
    fn conv_list_between_items(&mut self) {
        let idx = self.stack.to_number(At::from_top(2));
        self.rawset();
        self.stack.push_number(idx + 1.0);
    }

    fn conv_list_end(&mut self) {
        self.rawset();
    }

    fn conv_dict_start(&mut self, len: usize) -> Flow {
        if self.check_stack() == Flow::Fail {
            return Flow::Fail;
        }
        self.stack.new_table(0, len);
        Flow::Go
    }

    /// The key is already on the stack and the value has just landed on top of
    /// it, so one `rawset` closes the pair.
    fn conv_dict_between_items(&mut self) {
        self.rawset();
    }

    fn conv_dict_end(&mut self) {
        self.rawset();
    }

    /// Lua tables are references, so a container that references itself is not
    /// a problem here: push the half-built table again and the cycle rebuilds
    /// itself.
    fn conv_recurse(
        &mut self,
        container: Container<'_>,
        conv_type: ConvType,
        path: &ConvPath<'_, '_>,
    ) -> Flow {
        if let Some(at) = Self::backref(path, container, conv_type) {
            self.stack.push_value(at);
        }
        Flow::Go
    }
}

/// Convert a Vimscript value and leave it on the Lua stack.
///
/// With `special`, Vimscript's `null` and an empty dictionary go as `nil` and
/// a `{_TYPE = dictionary}` table; otherwise as `vim.NIL` and
/// `vim.empty_dict()`. `false` when the stack could not grow, with the
/// reason already reported.
pub fn push_typval(stack: &mut Stack, tv: &TypVal, special: bool) -> bool {
    let initial_size = stack.top().index();
    if !stack.reserve(initial_size + 2) {
        // Upstream reports the size it would have needed for a container,
        // not the two slots it just asked for.
        semsg!("E1502: Lua failed to grow stack to {}", initial_size + 4);
        return false;
    }
    let mut sink = LuaSink {
        stack: &mut *stack,
        special,
    };
    if !encode_typval(&mut sink, tv, c"nlua_push_typval argument") {
        return false;
    }
    debug_assert!(stack.top().index() == initial_size + 1);
    true
}
