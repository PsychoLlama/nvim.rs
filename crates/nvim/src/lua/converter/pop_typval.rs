//! `nlua_pop_typval()`: a Lua value as a Vimscript one.
//!
//! The Lua->typval direction, and the mirror of [`super::push`].  One
//! explicit stack of [`OpenValue`]s rather than recursion, because a Lua
//! table may nest arbitrarily deep and the conversion has to be able to
//! refuse (`E5100`) rather than overflow.  Tables are classified by
//! [`nlua_traverse_table`] first, so a table's *shape* -- list, dictionary,
//! empty-dict, or a `{_TYPE, _VAL}` special -- is decided once.
//!
//! A frame holds the **container it is filling**, as the owning handle:
//! nothing here names a typval slot by address, so appending to a list
//! cannot invalidate a frame and a refusal releases the half-built tree by
//! dropping the stack.  The Lua stack carries the rest of the position -- a
//! container's table sits at a fixed absolute index for as long as its frame
//! is open, and for a dictionary the key `lua_next` handed back waits one
//! slot above it while its value is converted.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::semsg;
use core::ffi::{CStr, c_char};

use super::pop::{At, LuaStack, TURN_SLOTS};
use super::{VARNUMBER_MAX, VARNUMBER_MIN, nlua_traverse_table};
use crate::eval::decode::{decode_create_map_special_dict, decode_string};
use crate::eval::typval::{
    DictRef, ListRef, tv_dict_add, tv_dict_alloc, tv_dict_item_alloc_len, tv_list_alloc,
};
use crate::eval::userfunc::register_luafunc;
use crate::lua::executor::{nlua_pushref, nlua_ref_global};
use crate::lua::ffi::{
    LUA_NOREF, LUA_TBOOLEAN, LUA_TFUNCTION, LUA_TNIL, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE,
    LUA_TUSERDATA,
};
use crate::lua::state::nlua_global_refs;
use crate::memory::xstrdup;
use crate::message::emsg;
use crate::narrow::{float_as_i64, len_as_int};
use crate::os::cshim::gettext;
use crate::types::{
    ListItem, LuaRef, TypVal, kBoolVarFalse, kBoolVarTrue, kObjectTypeArray, kObjectTypeDict,
    kObjectTypeFloat, kObjectTypeNil, kSpecialVarNull, lua_Number, lua_State, size_t,
};
use ::libc::abort;

/// Refused for a table that is neither a list nor a dictionary.
const E5100_MIXED_KEYS: &CStr = c"E5100: Cannot convert given Lua table: table should contain \
                                 either only integer keys or only string keys";
/// Refused for a Lua value with no Vimscript image at all.
const E5101_BAD_TYPE: &CStr = c"E5101: Cannot convert given Lua type";

/// One container the walk has opened and is still filling.
///
/// Each holds its own reference to the container, so the half-built tree is
/// reachable for the self-reference check and is released by dropping the
/// stack.  `table` is the container's Lua table.
enum OpenValue {
    /// A Lua array table becoming a list, `wanted` items long.
    List {
        list: ListRef,
        wanted: size_t,
        table: At,
    },
    /// A Lua string-keyed table becoming a dictionary.  The key `lua_next`
    /// handed back sits at `table + 1` until its value has been converted.
    Dict { dict: DictRef, table: At },
    /// A table with a NUL byte in one of its keys, which has no dictionary
    /// image: the `{_TYPE = map, _VAL = [[key, value], …]}` special form,
    /// whose `_VAL` list is what the pairs go into.
    Pairs {
        /// The special dictionary, which is what this frame answers.
        special: TypVal,
        /// `_VAL`, a reference of this frame's own on a list the dictionary
        /// owns.
        pairs: ListRef,
        table: At,
    },
}

impl OpenValue {
    /// Where this container's Lua table sits.
    fn table(&self) -> At {
        match self {
            OpenValue::List { table, .. }
            | OpenValue::Dict { table, .. }
            | OpenValue::Pairs { table, .. } => *table,
        }
    }

    /// The value a *self-reference* to this container converts to: another
    /// reference to what is being built, rather than a descent that would
    /// never end.
    ///
    /// A map special answers its **`_VAL` list**, not the dictionary around
    /// it -- upstream's frame holds `data.l.list` for everything that is not
    /// a plain dictionary, and this is the member it copies.
    fn reentered(&self) -> TypVal {
        match self {
            OpenValue::List { list, .. } => TypVal::list(Some(list.clone())),
            OpenValue::Dict { dict, .. } => TypVal::dict(Some(dict.clone())),
            OpenValue::Pairs { pairs, .. } => TypVal::list(Some(pairs.clone())),
        }
    }

    /// Put this container's next element on the Lua stack, or answer `false`
    /// once there is none left -- having popped whatever it was holding
    /// above its table.
    fn advance(&mut self, lua: &LuaStack) -> bool {
        match self {
            OpenValue::List {
                list,
                wanted,
                table,
            } => {
                if list.lv_items.len() >= *wanted {
                    return false;
                }
                lua.push_item(*table, len_as_int(list.lv_items.len() + 1));
                true
            }
            OpenValue::Dict { table, .. } | OpenValue::Pairs { table, .. } => {
                // Skip any non-string key: those are not part of the
                // dictionary being built.
                while lua.next_entry(*table) {
                    if lua.entry_key_is_string() {
                        return true;
                    }
                    lua.discard(1);
                }
                false
            }
        }
    }

    /// Store `value`, the conversion of the element
    /// [`advance`](Self::advance) last handed out.
    ///
    /// The converted Lua value is already off the stack, so a dictionary's
    /// pending key is once again the top of it.
    fn deliver(&mut self, lua: &LuaStack, value: TypVal) {
        match self {
            OpenValue::List { list, .. } => list.lv_items.push(ListItem::new(value)),
            OpenValue::Dict { dict, table } => {
                let key = lua.string_at(At(table.0 + 1));
                // SAFETY: `key` is the Lua string's own bytes, which the
                // item copies; the dictionary is the one this frame holds.
                unsafe {
                    let item = tv_dict_item_alloc_len(key.as_ptr().cast::<c_char>(), key.len());
                    (*item).di_tv = value;
                    if tv_dict_add(dict.as_ptr(), item).is_err() {
                        // A Lua table cannot hand the same key back twice,
                        // so the only refusal left is the funcref-name
                        // check, which a key from a table never trips.
                        abort();
                    }
                }
            }
            OpenValue::Pairs { pairs, table, .. } => {
                let bytes = lua.string_at(At(table.0 + 1));
                // The `_VAL` of a map special is a list of two-element
                // `[key, value]` lists, and the key is forced to a blob so
                // that the NUL that put it here survives.
                // SAFETY: as above; `decode_string` copies.
                let key = unsafe {
                    decode_string(bytes.as_ptr().cast::<c_char>(), bytes.len(), true, false)
                };
                let mut pair = tv_list_alloc(2);
                pair.lv_items.push(ListItem::new(key));
                pair.lv_items.push(ListItem::new(value));
                pairs.lv_items.push(ListItem::new(TypVal::list(Some(pair))));
            }
        }
    }

    /// The finished value, once the container has no elements left.
    fn finish(self) -> TypVal {
        match self {
            OpenValue::List { list, .. } => TypVal::list(Some(list)),
            OpenValue::Dict { dict, .. } => TypVal::dict(Some(dict)),
            OpenValue::Pairs { special, .. } => special,
        }
    }
}

/// What converting the Lua value on top of the stack produced.
enum Converted {
    /// A finished value.  The Lua value it came from is still on the stack.
    Value(TypVal),
    /// A container was opened and pushed onto the walk's stack; its table
    /// stays on the Lua stack until the frame closes.
    Opened,
}

/// Convert the Lua value on top of the stack, or open the container it is.
///
/// `None` refuses, with the reason already reported.
fn convert_top(lua: &LuaStack, stack: &mut Vec<OpenValue>) -> Option<Converted> {
    match lua.top_kind() {
        LUA_TNIL => Some(Converted::Value(TypVal::Special(kSpecialVarNull))),
        LUA_TBOOLEAN => Some(Converted::Value(TypVal::Bool(if lua.top_as_boolean() {
            kBoolVarTrue
        } else {
            kBoolVarFalse
        }))),
        LUA_TSTRING => {
            let bytes = lua.string_at(lua.top());
            // SAFETY: the Lua string's own bytes, which `decode_string`
            // copies.
            Some(Converted::Value(unsafe {
                decode_string(bytes.as_ptr().cast::<c_char>(), bytes.len(), false, false)
            }))
        }
        LUA_TNUMBER => {
            let n = lua.top_as_number();
            let no_integer = n > VARNUMBER_MAX as lua_Number
                || n < VARNUMBER_MIN as lua_Number
                || float_as_i64(n) as lua_Number != n;
            Some(Converted::Value(if no_integer {
                TypVal::Float(n)
            } else {
                TypVal::Number(float_as_i64(n))
            }))
        }
        LUA_TTABLE => convert_table(lua, stack),
        LUA_TFUNCTION => {
            // SAFETY: the function on top of the stack, registered under a
            // name the value takes over.
            let name = unsafe {
                let func = nlua_ref_global(lua.lstate, -1);
                xstrdup(register_luafunc(func))
            };
            Some(Converted::Value(TypVal::Func(name)))
        }
        LUA_TUSERDATA => {
            // TODO(bfredl): check mt.__call and convert to a function?
            // SAFETY: pushing the `vim.NIL` singleton to compare against.
            unsafe { nlua_pushref(lua.lstate, (*nlua_global_refs.get()).nil_ref) };
            let is_nil = lua.top_is(At(-2));
            lua.discard(1);
            if is_nil {
                Some(Converted::Value(TypVal::Special(kSpecialVarNull)))
            } else {
                emsg(gettext(E5101_BAD_TYPE));
                None
            }
        }
        _ => {
            emsg(gettext(E5101_BAD_TYPE));
            None
        }
    }
}

/// Convert the Lua *table* on top of the stack: a value outright when it is
/// empty or a self-reference, otherwise a container the walk descends into.
fn convert_table(lua: &LuaStack, stack: &mut Vec<OpenValue>) -> Option<Converted> {
    // Only worth tracking a table reference when the table has a metatable
    // of its own.
    let mut table_ref: LuaRef = LUA_NOREF;
    if lua.push_metatable() {
        lua.discard(1);
        // SAFETY: the table itself, which the reference now names.
        table_ref = unsafe { nlua_ref_global(lua.lstate, -1) };
    }

    // SAFETY: the table on top of the stack, which the traversal leaves
    // where it found it.
    let table_props = unsafe { nlua_traverse_table(lua.lstate) };

    // A container already open on this table is this same table: share it
    // rather than descend forever.
    for open in stack.iter() {
        if lua.top_is(open.table()) {
            return Some(Converted::Value(open.reentered()));
        }
    }

    // A frame opened below names this slot for as long as it lives.
    let table = lua.top();
    match table_props.type_0 {
        kObjectTypeArray => {
            let mut list = tv_list_alloc(table_props.maxidx.cast_signed());
            list.lua_table_ref = table_ref;
            if table_props.maxidx == 0 {
                return Some(Converted::Value(TypVal::list(Some(list))));
            }
            stack.push(OpenValue::List {
                list,
                wanted: table_props.maxidx,
                table,
            });
            Some(Converted::Opened)
        }
        kObjectTypeDict => {
            if table_props.string_keys_num == 0 {
                let mut dict = tv_dict_alloc();
                dict.lua_table_ref = table_ref;
                return Some(Converted::Value(TypVal::dict(Some(dict))));
            }
            if table_props.has_string_with_nul {
                // A key with a NUL in it has no Vimscript dictionary image,
                // so the whole table becomes the `{_TYPE = map, _VAL = [[k,
                // v], …]}` special form and the walk descends into `_VAL`.
                let mut special = TypVal::Number(0);
                // SAFETY: a fresh slot, and the list the special dictionary
                // is built around -- which the dictionary holds a reference
                // to for as long as this frame holds the dictionary.
                let mut pairs = unsafe {
                    let val = decode_create_map_special_dict(
                        &mut special,
                        table_props.string_keys_num.cast_signed(),
                    );
                    ListRef::retained(val).expect("`_VAL` is a list")
                };
                pairs.lua_table_ref = table_ref;
                stack.push(OpenValue::Pairs {
                    special,
                    pairs,
                    table,
                });
            } else {
                let mut dict = tv_dict_alloc();
                dict.lua_table_ref = table_ref;
                stack.push(OpenValue::Dict { dict, table });
            }
            // Seed `lua_next`.
            lua.push_nil();
            Some(Converted::Opened)
        }
        kObjectTypeFloat => Some(Converted::Value(TypVal::Float(table_props.val))),
        kObjectTypeNil => {
            emsg(gettext(E5100_MIXED_KEYS));
            None
        }
        // SAFETY: `abort` only ever ends the process.
        _ => unsafe { abort() },
    }
}

/// What the walk does next.
enum Step {
    /// Convert the Lua value on top of the stack.
    Convert,
    /// Hand a finished value to the container below it, or answer it.
    Deliver(TypVal),
    /// Ask the open container for its next element.
    Advance,
}

/// Walk the Lua value on top of the stack, leaving it there.
///
/// `None` refuses, with the reason already reported; whatever the walk had
/// built is released with `stack`.
fn walk(lua: &LuaStack, stack: &mut Vec<OpenValue>) -> Option<TypVal> {
    let mut step = Step::Convert;
    loop {
        if let Err(need) = lua.grow(TURN_SLOTS) {
            semsg!("E1502: Lua failed to grow stack to {need}");
            return None;
        }
        step = match step {
            Step::Convert => match convert_top(lua, stack)? {
                Converted::Value(value) => {
                    lua.discard(1);
                    Step::Deliver(value)
                }
                Converted::Opened => Step::Advance,
            },
            Step::Deliver(value) => match stack.last_mut() {
                None => return Some(value),
                Some(open) => {
                    open.deliver(lua, value);
                    Step::Advance
                }
            },
            Step::Advance => {
                let open = stack.last_mut().expect("a container to advance");
                if open.advance(lua) {
                    Step::Convert
                } else {
                    let done = stack.pop().expect("the container just advanced");
                    // Its table has served its purpose.
                    lua.discard(1);
                    Step::Deliver(done.finish())
                }
            }
        };
    }
}

/// The Lua value on top of the stack as a Vimscript one, popping exactly one
/// value.
///
/// `None` when it will not convert, with the reason already reported.
///
/// # Safety
/// `lstate` must be a live Lua state with a value on top.
pub unsafe fn nlua_pop_typval(lstate: *mut lua_State) -> Option<TypVal> {
    // The caller's promise, and the whole of what makes the walk safe: a
    // state that stays live for as long as this handle.
    let lua = LuaStack { lstate };
    let initial_size = lua.top();
    let mut stack: Vec<OpenValue> = Vec::new();
    let converted = walk(&lua, &mut stack);
    if converted.is_none() {
        // Every container the walk had open is released here.
        drop(stack);
        lua.discard(lua.top().0 - initial_size.0 + 1);
    }
    debug_assert!(lua.top().0 == initial_size.0 - 1);
    converted
}
