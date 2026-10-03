//! Keysets: a Lua options table as a generated `KeySet` struct.
//!
//! [`nlua_pop_keydict`] fills one of the api's generated option structs from
//! a Lua table, driven by the keyset's own [`KeySet`] lookup, and
//! [`nlua_push_keydict`] renders one back.  [`nlua_init_types`] installs the
//! names the api's type tags are known by on the Lua side.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use core::ffi::{CStr, c_char, c_int};

use super::{
    nlua_pop_array, nlua_pop_boolean_strict, nlua_pop_dict, nlua_pop_float, nlua_pop_handle,
    nlua_pop_integer, nlua_pop_luaref, nlua_pop_object, nlua_pop_string, nlua_push_array,
    nlua_push_dict, nlua_push_object, nlua_push_string, nlua_push_type_idx, nlua_push_val_idx,
};
use crate::api::private::keyset::{FieldKind, KeySet, Slot};
use crate::api_error;
use crate::highlight_group::syn_check_group;
use crate::lua::executor::nlua_pushref;
use crate::lua::ffi::{
    LUA_TSTRING, LUA_TTABLE, lua_createtable, lua_next, lua_pop, lua_pushboolean, lua_pushinteger,
    lua_pushlstring, lua_pushnil, lua_pushnumber, lua_pushstring, lua_rawset, lua_settop,
    lua_tolstring, lua_type,
};
use crate::message_fmt::c_str_len;
use crate::types::{
    Error, Integer, kErrorTypeValidation, kObjectTypeArray, kObjectTypeDict, kObjectTypeFloat,
    lua_Integer, lua_Number, lua_State, size_t,
};

/// The three api types that need a name of their own on the Lua side, both
/// ways round: `vim.types.float` is the tag and `vim.types[tag]` the name.
const NAMED_TYPES: [(&CStr, c_int); 3] = [
    (c"float", kObjectTypeFloat as c_int),
    (c"array", kObjectTypeArray as c_int),
    (c"dictionary", kObjectTypeDict as c_int),
];

/// Install `type_idx`, `val_idx` and `types` on the `vim` table below the top
/// of the stack.
///
/// # Safety
/// `lstate` must be a live Lua state with that table at -3.
pub unsafe fn nlua_init_types(lstate: *mut lua_State) {
    unsafe {
        // A Lua string, without its terminator.
        let push_cstr = |s: &CStr| lua_pushlstring(lstate, s.as_ptr(), s.count_bytes());

        push_cstr(c"type_idx");
        nlua_push_type_idx(lstate);
        lua_rawset(lstate, -3);

        push_cstr(c"val_idx");
        nlua_push_val_idx(lstate);
        lua_rawset(lstate, -3);

        push_cstr(c"types");
        lua_createtable(lstate, 0, 3);
        for (name, tag) in NAMED_TYPES {
            push_cstr(name);
            lua_pushnumber(lstate, tag as lua_Number);
            lua_rawset(lstate, -3);

            lua_pushnumber(lstate, tag as lua_Number);
            push_cstr(name);
            lua_rawset(lstate, -3);
        }
        lua_rawset(lstate, -3);
    }
}

/// Fill the keyset `retval` from the Lua table on top.
///
/// A key the keyset does not know is a refusal. On failure `*err_opt` names
/// the field that failed, for the caller's message.
///
/// # Safety
/// `lstate` must have a value on top.
pub(crate) unsafe fn nlua_pop_keydict(
    lstate: *mut lua_State,
    retval: &mut dyn KeySet,
    err_opt: &mut *mut c_char,
) -> Result<(), Error> {
    let fields = retval.fields();
    unsafe {
        if lua_type(lstate, -1) != LUA_TTABLE {
            // Upstream writes `lua_pop(L, -1)` here, which expands to
            // `lua_settop(L, 0)` -- it clears the *whole* stack rather than
            // popping the one value. Kept verbatim; see the divergence
            // docket.
            lua_settop(lstate, 0);
            return Err(Error::validation(c"Expected Lua table"));
        }

        let mut failed = None;
        lua_pushnil(lstate);
        while lua_next(lstate, -2) != 0 {
            let mut len: size_t = 0;
            let s = lua_tolstring(lstate, -2, &raw mut len);
            let key: &[u8] = if s.is_null() {
                &[]
            } else {
                ::core::slice::from_raw_parts(s.cast::<u8>(), len)
            };
            let Some(index) = retval.find(key) else {
                let key = c_str_len(s, len).null_as_empty();
                lua_pop(lstate, 3);
                return Err(api_error!(kErrorTypeValidation, "invalid key: {key}"));
            };
            let field = &fields[index];
            // Each arm stores what it popped; storing `Some` is what records
            // that the caller named the key. The field's own name is what a
            // refusal is reported against.
            let popped: Result<(), Error> = (|| {
                match retval.slot(index) {
                    Slot::Any(slot) => *slot = Some(nlua_pop_object(lstate, true)?),
                    // A highlight-group field takes the group's *name* as
                    // well as its id.
                    Slot::Integer(slot)
                        if field.kind == FieldKind::HlGroup
                            && lua_type(lstate, -1) == LUA_TSTRING =>
                    {
                        let mut name_len: size_t = 0;
                        let name = lua_tolstring(lstate, -1, &raw mut name_len);
                        lua_pop(lstate, 1);
                        let id = if name_len > 0 {
                            let name = ::core::slice::from_raw_parts(name.cast::<u8>(), name_len);
                            Integer::from(syn_check_group(name))
                        } else {
                            0
                        };
                        *slot = Some(id);
                    }
                    Slot::Integer(slot) => *slot = Some(nlua_pop_integer(lstate)?),
                    Slot::Boolean(slot) => *slot = Some(nlua_pop_boolean_strict(lstate)?),
                    Slot::String(slot) => *slot = Some(nlua_pop_string(lstate)?),
                    Slot::Float(slot) => *slot = Some(nlua_pop_float(lstate)?),
                    Slot::Handle(slot) => *slot = Some(nlua_pop_handle(lstate)?),
                    Slot::Array(slot) => *slot = Some(nlua_pop_array(lstate)?),
                    Slot::Dict(slot) => *slot = Some(nlua_pop_dict(lstate, false)?),
                    Slot::LuaRef(slot) => *slot = Some(nlua_pop_luaref(lstate)?),
                    Slot::StringArray(_) => {
                        unreachable!("only ShaDa's own keysets hold a string array")
                    }
                }
                Ok(())
            })();

            if let Err(e) = popped {
                *err_opt = field.name.as_ptr().cast_mut();
                failed = Some(e);
                break;
            }
        }
        lua_pop(lstate, 1);
        match failed {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

/// Push a keyset as a Lua table, one entry per field that is set.
///
/// # Safety
/// `lstate` must be a live Lua state.
pub(crate) unsafe fn nlua_push_keydict(lstate: *mut lua_State, value: &mut dyn KeySet) {
    let fields = value.fields();
    unsafe {
        lua_createtable(lstate, 0, 0);
        for (index, field) in fields.iter().enumerate() {
            let slot = value.slot(index);
            // A key the caller never named is not an entry of the table.
            if !slot.is_set() {
                continue;
            }
            lua_pushstring(lstate, field.name.as_ptr());
            match slot {
                Slot::Any(Some(object)) => nlua_push_object(lstate, object, 0),
                Slot::Integer(Some(n)) => lua_pushinteger(lstate, *n as lua_Integer),
                Slot::Handle(Some(h)) => lua_pushinteger(lstate, *h as lua_Integer),
                Slot::Float(Some(f)) => lua_pushnumber(lstate, *f),
                Slot::Boolean(Some(b)) => lua_pushboolean(lstate, c_int::from(*b)),
                Slot::String(Some(s)) => nlua_push_string(lstate, s, 0),
                Slot::Array(Some(array)) => nlua_push_array(lstate, array, 0),
                Slot::Dict(Some(dict)) => nlua_push_dict(lstate, dict, 0),
                Slot::LuaRef(Some(r)) => nlua_pushref(lstate, *r),
                _ => unreachable!("a set field holds its own kind"),
            }
            lua_rawset(lstate, -3);
        }
    }
}
