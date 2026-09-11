//! `nlua_pop_object()`: a Lua value as an API [`Object`].
//!
//! The same explicit-stack walk as [`super::pop_typval`], over the same
//! [`LuaStack`], producing api types instead of `TypVal`s.  It is a separate
//! walk because the two type systems disagree at the leaves: an `Object` has
//! no `VAR_SPECIAL` and carries `LuaRef`s for functions.
//!
//! A frame holds the **container it is filling**, as the owning value:
//! nothing here names an object slot by address, so nothing an append does
//! can invalidate a frame, and a refusal releases the half-built tree by
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

use core::ffi::CStr;

use super::pop::{At, LuaStack, TURN_SLOTS};
use super::{API_INTEGER_MAX, API_INTEGER_MIN, nlua_traverse_table};
use crate::lua::executor::{nlua_pushref, nlua_ref_global};
use crate::lua::ffi::{
    LUA_TBOOLEAN, LUA_TFUNCTION, LUA_TNIL, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE, LUA_TUSERDATA,
};
use crate::lua::state::nlua_global_refs;
use crate::narrow::{float_as_i64, len_as_int};
use crate::types::{
    ApiDict, Arena, Array, DictKey, Error, Object, String_0, kObjectTypeArray, kObjectTypeDict,
    kObjectTypeFloat, kObjectTypeNil, lua_Number, lua_State, size_t,
};
use ::libc::abort;

/// Refused for a Lua value with no api image.
const CANNOT_CONVERT: &CStr = c"Cannot convert given Lua type";

/// One container the walk has opened and is still filling.
///
/// `wanted` is how many elements the container takes, decided from the Lua
/// table's shape before the descent: a table with more than that in it has
/// the rest ignored, which is what the capacity test upstream writes says.
enum OpenValue {
    /// A Lua array table becoming an [`Array`].
    Array {
        array: Array,
        wanted: size_t,
        table: At,
    },
    /// A Lua string-keyed table becoming an [`ApiDict`].  The key `lua_next`
    /// handed back sits at `table + 1` until its value has been converted.
    Dict {
        dict: ApiDict,
        wanted: size_t,
        table: At,
    },
}

impl OpenValue {
    /// Put this container's next element on the Lua stack, or answer `false`
    /// once there is none left -- having popped whatever it was holding
    /// above its table.
    fn advance(&mut self, lua: &LuaStack) -> bool {
        match self {
            OpenValue::Array {
                array,
                wanted,
                table,
            } => {
                if array.len() >= *wanted {
                    return false;
                }
                lua.push_item(*table, len_as_int(array.len()) + 1);
                true
            }
            OpenValue::Dict {
                dict,
                wanted,
                table,
            } => {
                if dict.len() >= *wanted {
                    // The key the last turn left above the table goes too.
                    lua.discard(1);
                    return false;
                }
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
    fn deliver(&mut self, lua: &LuaStack, value: Object) {
        match self {
            OpenValue::Array { array, .. } => array.push(value),
            OpenValue::Dict { dict, table, .. } => {
                // The key copies the Lua string's bytes -- into the entry
                // itself when they are short, which an api key nearly
                // always is.
                let key = DictKey::new(lua.string_at(At(table.0 + 1)));
                dict.insert(key, value);
            }
        }
    }

    /// The finished value, once the container has no elements left.
    fn finish(self) -> Object {
        match self {
            OpenValue::Array { array, .. } => Object::array(array),
            OpenValue::Dict { dict, .. } => Object::dict(dict),
        }
    }
}

/// What converting the Lua value on top of the stack produced.
enum Converted {
    /// A finished value.  The Lua value it came from is still on the stack.
    Value(Object),
    /// A container was opened and pushed onto the walk's stack; its table
    /// stays on the Lua stack until the frame closes.
    Opened,
}

/// Convert the Lua value on top of the stack, or open the container it is.
///
/// With `as_ref`, a Lua function becomes a `LuaRef` rather than a refusal.
fn convert_top(
    lua: &LuaStack,
    as_ref: bool,
    stack: &mut Vec<OpenValue>,
) -> Result<Converted, Error> {
    match lua.top_kind() {
        LUA_TNIL => Ok(Converted::Value(Object::Nil)),
        LUA_TBOOLEAN => Ok(Converted::Value(Object::boolean(lua.top_as_boolean()))),
        LUA_TSTRING => {
            let bytes = lua.string_at(lua.top());
            Ok(Converted::Value(Object::string(String_0::from_bytes(
                bytes,
            ))))
        }
        LUA_TNUMBER => {
            let n = lua.top_as_number();
            let no_integer = n > API_INTEGER_MAX as lua_Number
                || n < API_INTEGER_MIN as lua_Number
                || float_as_i64(n) as lua_Number != n;
            Ok(Converted::Value(if no_integer {
                Object::float(n)
            } else {
                Object::integer(float_as_i64(n))
            }))
        }
        LUA_TTABLE => convert_table(lua, stack),
        LUA_TFUNCTION if as_ref => {
            // SAFETY: the function on top of the stack, which the reference
            // now names.
            let func = unsafe { nlua_ref_global(lua.lstate, -1) };
            Ok(Converted::Value(Object::luaref(func)))
        }
        LUA_TUSERDATA => {
            // SAFETY: pushing the `vim.NIL` singleton to compare against.
            unsafe { nlua_pushref(lua.lstate, (*nlua_global_refs.get()).nil_ref) };
            let is_nil = lua.top_is(At(-2));
            lua.discard(1);
            if is_nil {
                Ok(Converted::Value(Object::Nil))
            } else {
                Err(Error::validation(c"Cannot convert userdata"))
            }
        }
        _ => Err(Error::validation(CANNOT_CONVERT)),
    }
}

/// Convert the Lua *table* on top of the stack: a value outright when it is
/// empty or carries a number, otherwise a container the walk descends into.
fn convert_table(lua: &LuaStack, stack: &mut Vec<OpenValue>) -> Result<Converted, Error> {
    // SAFETY: the table on top of the stack, which the traversal leaves
    // where it found it.
    let table_props = unsafe { nlua_traverse_table(lua.lstate) };
    // A frame opened below names this slot for as long as it lives.
    let table = lua.top();
    match table_props.type_0 {
        kObjectTypeArray => {
            let wanted = table_props.maxidx;
            if wanted == 0 {
                return Ok(Converted::Value(Object::array(Array::EMPTY)));
            }
            stack.push(OpenValue::Array {
                array: Array::with_capacity(wanted),
                wanted,
                table,
            });
            Ok(Converted::Opened)
        }
        kObjectTypeDict => {
            let wanted = table_props.string_keys_num;
            if wanted == 0 {
                return Ok(Converted::Value(Object::dict(ApiDict::EMPTY)));
            }
            stack.push(OpenValue::Dict {
                dict: ApiDict::with_capacity(wanted),
                wanted,
                table,
            });
            // Seed `lua_next`.
            lua.push_nil();
            Ok(Converted::Opened)
        }
        kObjectTypeFloat => Ok(Converted::Value(Object::float(table_props.val))),
        kObjectTypeNil => Err(Error::validation(c"Cannot convert given Lua table")),
        // SAFETY: `abort` only ever ends the process.
        _ => unsafe { abort() },
    }
}

/// What the walk does next.
enum Step {
    /// Convert the Lua value on top of the stack.
    Convert,
    /// Hand a finished value to the container below it, or answer it.
    Deliver(Object),
    /// Ask the open container for its next element.
    Advance,
}

/// Walk the Lua value on top of the stack, leaving it there.
///
/// Whatever the walk had built when it refused is released with `stack`.
fn walk(lua: &LuaStack, as_ref: bool, stack: &mut Vec<OpenValue>) -> Result<Object, Error> {
    let mut step = Step::Convert;
    loop {
        step = match step {
            Step::Convert => match convert_top(lua, as_ref, stack)? {
                Converted::Value(value) => {
                    lua.discard(1);
                    Step::Deliver(value)
                }
                Converted::Opened => Step::Advance,
            },
            Step::Deliver(value) => match stack.last_mut() {
                None => return Ok(value),
                Some(open) => {
                    open.deliver(lua, value);
                    Step::Advance
                }
            },
            Step::Advance => {
                // Room for the next key and value, and for the `vim.NIL` a
                // userdata leaf compares itself with. Asked once per
                // element, which is where the pushes are.
                if lua.grow(TURN_SLOTS).is_err() {
                    return Err(Error::exception(c"Lua failed to grow stack"));
                }
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

/// Convert the Lua value on top of the stack, popping exactly one value.
///
/// With `as_ref`, a Lua function becomes a `LuaRef` rather than a refusal.
///
/// `_arena` is vestigial: the api value types own their storage since the
/// arena left them, and nothing here allocates into one. Every caller is
/// generated, so the parameter goes when `tools/apigen` stops emitting it.
///
/// # Safety
/// `lstate` must be a live Lua state with a value on top.
pub unsafe fn nlua_pop_object(
    lstate: *mut lua_State,
    as_ref: bool,
    _arena: *mut Arena,
) -> Result<Object, Error> {
    // The caller's promise, and the whole of what makes the walk safe: a
    // state that stays live for as long as this handle.
    let lua = LuaStack { lstate };
    let initial_size = lua.top();
    let mut stack: Vec<OpenValue> = Vec::new();
    let converted = walk(&lua, as_ref, &mut stack);
    if converted.is_err() {
        // Every container the walk had open is released here.
        drop(stack);
        lua.discard(lua.top().0 - initial_size.0 + 1);
    }
    debug_assert!(lua.top().0 == initial_size.0 - 1);
    converted
}
