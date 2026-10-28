//! The api [`Object`] -> Lua direction.
//!
//! One `push_*` per api type. [`push_type_index`], [`push_value_index`] and
//! [`create_typed_table`] are what build a `{_TYPE, _VAL}` special dictionary
//! -- the representation a value keeps when it has no Lua equivalent -- and
//! [`push_object`] is the dispatch over the kinds.
//!
//! A push borrows: the object stays the caller's, and a `LuaRef` in it is
//! pushed without being given up. A caller done with the object drops it
//! afterwards, which releases its references.

#![forbid(unsafe_code)]

use super::{TYPE_IDX_VALUE, VAL_IDX_VALUE};
use crate::lua::stack::{At, Stack};
use crate::types::{
    ApiDict, Array, Boolean, Float, Handle, Integer, Object, ObjectType, String_0,
    kObjectTypeFloat, lua_Number,
};

/// Push the key a special table's type tag is stored under.
#[inline]
pub(crate) fn push_type_index(stack: &mut Stack) {
    stack.push_boolean(TYPE_IDX_VALUE);
}

/// Push the key a special table's value is stored under.
#[inline]
pub(crate) fn push_value_index(stack: &mut Stack) {
    stack.push_boolean(VAL_IDX_VALUE);
}

/// Push a fresh table already carrying its `_TYPE` tag, with room for `narr`
/// array and `nrec` record entries beside it.
#[inline]
pub(crate) fn create_typed_table(stack: &mut Stack, narr: usize, nrec: usize, type_0: ObjectType) {
    stack.new_table(narr, nrec + 1);
    push_type_index(stack);
    stack.push_number(lua_Number::from(type_0));
    stack.raw_set(At::from_top(3));
}

/// Push an api string as a Lua string, NULs and all.
pub fn push_string(stack: &mut Stack, s: &String_0) {
    stack.push_bytes(s.as_bytes());
}

pub fn push_integer(stack: &mut Stack, n: Integer) {
    stack.push_number(n as lua_Number);
}

/// Push a float, as a plain number or -- with `special` -- as the
/// `{_TYPE = float, _VAL = f}` table that survives a round trip through
/// Vimscript.
pub fn push_float(stack: &mut Stack, f: Float, special: bool) {
    if special {
        create_typed_table(stack, 0, 1, kObjectTypeFloat);
        push_value_index(stack);
        stack.push_number(f);
        stack.raw_set(At::from_top(3));
    } else {
        stack.push_number(f);
    }
}

pub fn push_boolean(stack: &mut Stack, b: Boolean) {
    stack.push_boolean(b);
}

/// Push an api dictionary as a Lua table. An empty one carries the
/// `vim.empty_dict()` metatable, because an empty Lua table is otherwise
/// indistinguishable from an empty list.
pub fn push_dict(stack: &mut Stack, dict: &ApiDict, special: bool) {
    stack.new_table(0, dict.len());
    if dict.is_empty() {
        stack.push_empty_dict_metatable();
        stack.set_metatable(At::from_top(2));
    }
    for entry in dict.iter() {
        stack.push_bytes(entry.key.bytes());
        push_object(stack, &entry.value, special);
        stack.raw_set(At::from_top(3));
    }
}

pub fn push_array(stack: &mut Stack, array: &Array, special: bool) {
    stack.new_table(array.len(), 0);
    for (i, item) in array.iter().enumerate() {
        push_object(stack, item, special);
        stack.raw_set_index(At::from_top(2), i as core::ffi::c_int + 1);
    }
}

pub fn push_handle(stack: &mut Stack, item: Handle) {
    stack.push_number(lua_Number::from(item));
}

/// Push any api value.
///
/// With `special`, nil goes as `nil` rather than `vim.NIL` and a float as
/// its special table.
pub fn push_object(stack: &mut Stack, obj: &Object, special: bool) {
    match obj {
        Object::Nil if special => stack.push_nil(),
        Object::Nil => stack.push_vim_nil(),
        Object::LuaRef(reference) => stack.push_reference(*reference),
        Object::Boolean(b) => push_boolean(stack, *b),
        Object::Integer(n) => push_integer(stack, *n),
        Object::Float(f) => push_float(stack, *f, special),
        Object::String(s) => push_string(stack, s),
        Object::Array(a) => push_array(stack, a, special),
        Object::Dict(d) => push_dict(stack, d, special),
        Object::Buffer(h) | Object::Window(h) | Object::Tabpage(h) => {
            push_handle(stack, *h as Handle);
        }
    }
}
