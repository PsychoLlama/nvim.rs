//! `nlua_pop_object()`: a Lua value as an API [`Object`].
//!
//! The same explicit-stack walk as [`super::pop_typval`], over
//! [`ObjPopStackItem`] and producing api types instead of `typval_T`s.  It
//! is a separate walk because the two type systems disagree at the leaves:
//! an `Object` has no `VAR_SPECIAL`, carries `LuaRef`s for functions, and
//! allocates into an [`Arena`] when it is given one.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::CStr;

use super::{API_INTEGER_MAX, API_INTEGER_MIN, nlua_traverse_table};
use crate::api::private::helpers::{api_free_object, arena_array, arena_dict, arena_string};
use crate::eval::typval_encode::InlineStack;
use crate::lua::executor::{nlua_pushref, nlua_ref_global};
use crate::lua::ffi::{
    LUA_TBOOLEAN, LUA_TFUNCTION, LUA_TNIL, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE, LUA_TUSERDATA,
    lua_checkstack, lua_gettop, lua_next, lua_pop, lua_pushnil, lua_rawequal, lua_rawgeti,
    lua_toboolean, lua_tolstring, lua_tonumber, lua_type,
};
use crate::main::nlua_global_refs;
use crate::types::{
    Arena, Array, Dict, Error, Integer, Object, String_0, kErrorTypeException,
    kErrorTypeValidation, kObjectTypeArray, kObjectTypeDict, kObjectTypeFloat, kObjectTypeNil,
    lua_Number, lua_State, size_t,
};
use ::libc::abort;

/// One suspended container in the walk.
///
/// Simpler than [`super::pop_typval`]'s frame: an `Array`/`Dict` carries its
/// own capacity, so "how far have we got" needs no field of its own and a
/// self-reference is impossible — an arena value is a tree by construction.
#[derive(Copy, Clone)]
pub struct ObjPopStackItem {
    /// Where the conversion's result is to be stored.
    pub obj: *mut Object,
    /// Whether `obj` is a container: a frame that is suspended rather than
    /// about to be filled in.
    pub container: bool,
}

impl ObjPopStackItem {
    /// A frame about to be filled in, not a suspended container.
    const fn leaf(obj: *mut Object) -> Self {
        Self {
            obj,
            container: false,
        }
    }
}

/// Frames held without allocating: upstream's `kvec_withinit_t(…, 2)`.
type ObjPopStack = InlineStack<ObjPopStackItem, 2>;

/// Refused for a Lua value with no api image.
const CANNOT_CONVERT: &CStr = c"Cannot convert given Lua type";

/// Convert the Lua value on top of the stack, popping exactly one value.
///
/// With `ref_0`, a Lua function becomes a `LuaRef` rather than a refusal.
/// `arena`, when non-null, owns every string and container the result holds;
/// otherwise they are separately allocated and the caller frees them.
///
/// # Safety
/// `lstate` must be a live Lua state with a value on top, and `err` the
/// caller's error slot.
pub unsafe fn nlua_pop_object(
    lstate: *mut lua_State,
    ref_0: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    unsafe {
        let mut ret = Object::NIL;
        let initial_size = lua_gettop(lstate);
        let mut stack = ObjPopStack::new();
        stack.push(ObjPopStackItem::leaf(&raw mut ret));
        while !(*err).is_set() && !stack.is_empty() {
            let mut cur = stack.last();
            stack.pop();
            if cur.container {
                if lua_checkstack(lstate, lua_gettop(lstate) + 3) == 0 {
                    *err = Error::from_message(kErrorTypeException, c"Lua failed to grow stack");
                    break;
                }
                if (*cur.obj).type_0 == kObjectTypeDict {
                    if (*cur.obj).data.dict.size == (*cur.obj).data.dict.capacity {
                        // Full: pop the table and the key lua_next left.
                        lua_pop(lstate, 2);
                        continue;
                    }
                    // Skip any non-string key: those are not part of the
                    // dictionary being built.
                    let mut next_key_found = false;
                    while lua_next(lstate, -2) != 0 {
                        if lua_type(lstate, -2) == LUA_TSTRING {
                            next_key_found = true;
                            break;
                        }
                        lua_pop(lstate, 1);
                    }
                    if !next_key_found {
                        lua_pop(lstate, 1);
                        continue;
                    }
                    let mut len: size_t = 0;
                    let s = lua_tolstring(lstate, -2, &raw mut len);
                    let idx = (*cur.obj).data.dict.size;
                    (*cur.obj).data.dict.size = idx.wrapping_add(1);
                    (*(*cur.obj).data.dict.items.add(idx)).key =
                        arena_string(arena, String_0::from_raw_parts(s.cast_mut(), len));
                    stack.push(cur);
                    cur = ObjPopStackItem::leaf(
                        &raw mut (*(*cur.obj).data.dict.items.add(idx)).value,
                    );
                } else if (*cur.obj).data.array.size == (*cur.obj).data.array.capacity {
                    lua_pop(lstate, 1);
                    continue;
                } else {
                    let idx = (*cur.obj).data.array.size;
                    (*cur.obj).data.array.size = idx.wrapping_add(1);
                    lua_rawgeti(lstate, -1, idx as ::core::ffi::c_int + 1);
                    stack.push(cur);
                    cur = ObjPopStackItem::leaf((*cur.obj).data.array.items.add(idx));
                }
            }
            debug_assert!(!cur.container);
            *cur.obj = Object::NIL;
            'converted: {
                match lua_type(lstate, -1) {
                    LUA_TNIL => break 'converted,
                    LUA_TBOOLEAN => {
                        *cur.obj = Object::boolean(lua_toboolean(lstate, -1) != 0);
                        break 'converted;
                    }
                    LUA_TSTRING => {
                        let mut len: size_t = 0;
                        let s = lua_tolstring(lstate, -1, &raw mut len);
                        *cur.obj = Object::string(arena_string(
                            arena,
                            String_0::from_raw_parts(s.cast_mut(), len),
                        ));
                        break 'converted;
                    }
                    LUA_TNUMBER => {
                        let n = lua_tonumber(lstate, -1);
                        *cur.obj = if n > API_INTEGER_MAX as lua_Number
                            || n < API_INTEGER_MIN as lua_Number
                            || (n as Integer) as lua_Number != n
                        {
                            Object::float(n)
                        } else {
                            Object::integer(n as Integer)
                        };
                        break 'converted;
                    }
                    LUA_TTABLE => {
                        let table_props = nlua_traverse_table(lstate);
                        match table_props.type_0 {
                            kObjectTypeArray => {
                                *cur.obj = Object::array(Array::EMPTY);
                                if table_props.maxidx != 0 {
                                    (*cur.obj).data.array = arena_array(arena, table_props.maxidx);
                                    cur.container = true;
                                    stack.push(cur);
                                }
                            }
                            kObjectTypeDict => {
                                *cur.obj = Object::dict(Dict::EMPTY);
                                if table_props.string_keys_num != 0 {
                                    (*cur.obj).data.dict =
                                        arena_dict(arena, table_props.string_keys_num);
                                    cur.container = true;
                                    stack.push(cur);
                                    lua_pushnil(lstate);
                                }
                            }
                            kObjectTypeFloat => {
                                *cur.obj = Object::float(table_props.val);
                            }
                            kObjectTypeNil => {
                                *err = Error::from_message(
                                    kErrorTypeValidation,
                                    c"Cannot convert given Lua table",
                                );
                            }
                            _ => abort(),
                        }
                        break 'converted;
                    }
                    LUA_TFUNCTION => {
                        if ref_0 {
                            *cur.obj = Object::luaref(nlua_ref_global(lstate, -1));
                            break 'converted;
                        }
                    }
                    LUA_TUSERDATA => {
                        nlua_pushref(lstate, (*nlua_global_refs.get()).nil_ref);
                        let is_nil = lua_rawequal(lstate, -2, -1) != 0;
                        lua_pop(lstate, 1);
                        if is_nil {
                            *cur.obj = Object::NIL;
                        } else {
                            *err = Error::from_message(
                                kErrorTypeValidation,
                                c"Cannot convert userdata",
                            );
                        }
                        break 'converted;
                    }
                    _ => {}
                }
                *err = Error::validation(CANNOT_CONVERT);
            }
            if !cur.container {
                lua_pop(lstate, 1);
            }
        }
        if (*err).is_set() {
            if arena.is_null() {
                api_free_object(ret);
            }
            ret = Object::NIL;
            lua_pop(lstate, lua_gettop(lstate) - initial_size + 1);
        }
        debug_assert!(lua_gettop(lstate) == initial_size - 1);
        ret
    }
}
