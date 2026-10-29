//! Keysets: a Lua options table as a generated `KeySet` struct.
//!
//! [`pop_keydict`] fills one of the api's generated option structs from
//! a Lua table, driven by the keyset's own [`KeySet`] lookup, and
//! [`push_keydict`] renders one back.  [`init_types`] installs the
//! names the api's type tags are known by on the Lua side.

#![forbid(unsafe_code)]

use core::ffi::{CStr, c_int};

use super::{
    pop_array, pop_boolean_strict, pop_dict, pop_float, pop_handle, pop_integer, pop_luaref,
    pop_object, pop_string, push_array, push_dict, push_object, push_string, push_type_index,
    push_value_index,
};
use crate::api::private::keyset::{FieldKind, KeySet, Slot};
use crate::api_error;
use crate::highlight_group::syn_check_group;
use crate::lua::stack::{At, Kind, Stack};
use crate::message_fmt::msg_bytes;
use crate::types::{
    Error, Integer, kErrorTypeValidation, kObjectTypeArray, kObjectTypeDict, kObjectTypeFloat,
    lua_Integer, lua_Number,
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
pub fn init_types(stack: &mut Stack) {
    stack.push_bytes(b"type_idx");
    push_type_index(stack);
    stack.raw_set(At::from_top(3));

    stack.push_bytes(b"val_idx");
    push_value_index(stack);
    stack.raw_set(At::from_top(3));

    stack.push_bytes(b"types");
    stack.new_table(0, 3);
    for (name, tag) in NAMED_TYPES {
        stack.push_bytes(name.to_bytes());
        stack.push_number(lua_Number::from(tag));
        stack.raw_set(At::from_top(3));

        stack.push_number(lua_Number::from(tag));
        stack.push_bytes(name.to_bytes());
        stack.raw_set(At::from_top(3));
    }
    stack.raw_set(At::from_top(3));
}

/// Fill the keyset `retval` from the Lua table on top.
///
/// A key the keyset does not know is a refusal. On failure `*blame` names
/// the field that failed, for the caller's message.
pub(crate) fn pop_keydict(
    stack: &mut Stack,
    retval: &mut dyn KeySet,
    blame: &mut Option<&'static CStr>,
) -> Result<(), Error> {
    let fields = retval.fields();
    if stack.kind(At::TOP) != Kind::Table {
        // Upstream writes `lua_pop(L, -1)` here, which expands to
        // `lua_settop(L, 0)` -- it clears the *whole* stack rather than
        // popping the one value. Kept: the whole of this view, which for a
        // binding's own frame is upstream's; see the divergence docket.
        stack.set_top(stack.base());
        return Err(Error::validation(c"Expected Lua table"));
    }

    let mut failed = None;
    stack.push_nil();
    while stack.next(At::from_top(2)) {
        // A key that is not a string (nor a number, which this converts in
        // place, as upstream's `lua_tolstring` does) is the empty key.
        let key = stack.to_bytes(At::from_top(2)).unwrap_or_default();
        let Some(index) = retval.find(key) else {
            let key = msg_bytes(key);
            let refused = api_error!(kErrorTypeValidation, "invalid key: {key}");
            stack.pop(3);
            return Err(refused);
        };
        let field = &fields[index];
        // Each arm stores what it popped; storing `Some` is what records that
        // the caller named the key. The field's own name is what a refusal is
        // reported against.
        let popped: Result<(), Error> = (|| {
            match retval.slot(index) {
                Slot::Any(slot) => *slot = Some(pop_object(stack, true)?),
                // A highlight-group field takes the group's *name* as well as
                // its id.
                Slot::Integer(slot)
                    if field.kind == FieldKind::HlGroup && stack.kind(At::TOP) == Kind::String =>
                {
                    let name = stack.to_bytes(At::TOP).unwrap_or_default();
                    let id = if name.is_empty() {
                        0
                    } else {
                        Integer::from(syn_check_group(name))
                    };
                    stack.pop(1);
                    *slot = Some(id);
                }
                Slot::Integer(slot) => *slot = Some(pop_integer(stack)?),
                Slot::Boolean(slot) => *slot = Some(pop_boolean_strict(stack)?),
                Slot::String(slot) => *slot = Some(pop_string(stack)?),
                Slot::Float(slot) => *slot = Some(pop_float(stack)?),
                Slot::Handle(slot) => *slot = Some(pop_handle(stack)?),
                Slot::Array(slot) => *slot = Some(pop_array(stack)?),
                Slot::Dict(slot) => *slot = Some(pop_dict(stack, false)?),
                Slot::LuaRef(slot) => *slot = Some(pop_luaref(stack)?),
                Slot::StringArray(_) => {
                    unreachable!("only ShaDa's own keysets hold a string array")
                }
            }
            Ok(())
        })();

        if let Err(e) = popped {
            *blame = Some(field.name);
            failed = Some(e);
            break;
        }
    }
    stack.pop(1);
    match failed {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// Push a keyset as a Lua table, one entry per field that is set.
pub(crate) fn push_keydict(stack: &mut Stack, value: &mut dyn KeySet) {
    let fields = value.fields();
    stack.new_table(0, 0);
    for (index, field) in fields.iter().enumerate() {
        let slot = value.slot(index);
        // A key the caller never named is not an entry of the table.
        if !slot.is_set() {
            continue;
        }
        stack.push_bytes(field.name.to_bytes());
        match slot {
            Slot::Any(Some(object)) => push_object(stack, object, false),
            Slot::Integer(Some(n)) => stack.push_integer(*n as lua_Integer),
            Slot::Handle(Some(h)) => stack.push_integer(*h as lua_Integer),
            Slot::Float(Some(f)) => stack.push_number(*f),
            Slot::Boolean(Some(b)) => stack.push_boolean(*b),
            Slot::String(Some(s)) => push_string(stack, s),
            Slot::Array(Some(array)) => push_array(stack, array, false),
            Slot::Dict(Some(dict)) => push_dict(stack, dict, false),
            Slot::LuaRef(Some(r)) => stack.push_reference(*r),
            _ => unreachable!("a set field holds its own kind"),
        }
        stack.raw_set(At::from_top(3));
    }
}
