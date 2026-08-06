//! Values across the Lua boundary, in both directions and for both of the
//! editor's value types.
//!
//! Four walks live here. [`push`] converts a `typval_T` to a Lua value and
//! [`pop_typval`] converts one back; [`push_object`] and [`pop_object`] do
//! the same for the api's [`Object`](crate::src::nvim::types::Object). The
//! two `pop` directions are separate because the type systems disagree at
//! the leaves — an `Object` has no `VAR_SPECIAL`, carries `LuaRef`s for
//! functions, and allocates into an `Arena` — but they share [`pop`]'s
//! [`LuaTableProps`], which is the one place a Lua table's *shape* is
//! decided, and [`keysets`] is a fifth, generated-struct-shaped direction on
//! top of them.
//!
//! Neither `pop` walk recurses: a Lua table may nest arbitrarily deep and
//! the conversion has to be able to refuse rather than overflow the C stack,
//! so each keeps an explicit stack of suspended containers.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use crate::src::nvim::types::{ObjectType, kObjectTypeNil, lua_Number, size_t};

// The typval-to-Lua direction, carved out of this module's
// `typval_encode.c.h` instantiation.
mod push;
pub use self::push::nlua_push_typval;

mod keysets;
mod pop;
mod pop_object;
mod pop_typval;
mod push_object;

pub use self::keysets::*;
pub use self::pop::*;
pub use self::pop_object::*;
pub use self::pop_typval::*;
pub use self::push_object::*;

/// `nlua_push_*` flags.
pub type NluaPushFlags = ::core::ffi::c_uint;
/// Push Vimscript's `null` and empty dictionary as `nil` and a `{_TYPE,
/// _VAL}` table, rather than as the `vim.NIL` and `vim.empty_dict()`
/// singletons Lua code normally sees.
pub const kNluaPushSpecial: NluaPushFlags = 1;
/// Release each `LuaRef` as it is pushed: the object is being consumed.
pub const kNluaPushFreeRefs: NluaPushFlags = 2;

/// The two boolean keys a `{_TYPE, _VAL}` special table is built from: `true`
/// holds the type tag, `false` the value.
pub(crate) const TYPE_IDX_VALUE: bool = true;
pub(crate) const VAL_IDX_VALUE: bool = false;

/// `ufunc_T::uf_flags`: this function is a Lua reference, not Vimscript.
pub(crate) const FC_LUAREF: c_int = 0x800;

/// The largest and smallest integers an api `Integer` and a Vimscript
/// `varnumber_T` hold — both are `int64_t`.
pub(crate) const API_INTEGER_MAX: i64 = i64::MAX;
pub(crate) const API_INTEGER_MIN: i64 = i64::MIN;
pub(crate) const VARNUMBER_MAX: i64 = i64::MAX;
pub(crate) const VARNUMBER_MIN: i64 = i64::MIN;

/// What keys a Lua table turned out to contain — the answer
/// [`nlua_traverse_table`] hands both `pop` walks.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LuaTableProps {
    /// The largest positive integral key found.
    pub maxidx: size_t,
    /// How many string keys there are.
    pub string_keys_num: size_t,
    /// Whether any string key contains a NUL byte.
    pub has_string_with_nul: bool,
    /// The type attached under the `_TYPE` key when [`Self::has_type_key`];
    /// otherwise the shape the other fields imply — nil, dict or array.
    pub type_0: ObjectType,
    /// The value under the `_VAL` key, when that key holds a number.
    pub val: lua_Number,
    /// Whether the `_TYPE` key is present.
    pub has_type_key: bool,
}

impl LuaTableProps {
    /// "Not convertible": what both refusal paths answer.
    pub(crate) const NIL: Self = Self {
        maxidx: 0,
        string_keys_num: 0,
        has_string_with_nul: false,
        type_0: kObjectTypeNil,
        val: 0.0,
        has_type_key: false,
    };
}
