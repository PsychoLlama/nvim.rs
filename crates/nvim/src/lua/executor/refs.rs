//! `LuaRef`s: the registry entries every callback is held in.
//!
//! A `LuaRef` is an index into the main state's registry, taken by
//! [`Stack::reference`] and given back by [`Stack::unreference`] (or by
//! dropping the [`OwnedLuaRef`](crate::lua::stack::OwnedLuaRef) that holds
//! it); [`MAIN_REFS`] counts them. This is the api's side of that: copying a
//! reference, giving one back, and asking what it refers to.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::main_stack;
use crate::lua::ffi::LUA_NOREF;
use crate::lua::stack::{At, Kind};
use crate::lua::state::MAIN_REFS;
use crate::types::LuaRef;

/// How many references the main state is holding — what the functional tests
/// assert against to catch a leak.
pub fn nlua_get_global_ref_count() -> c_int {
    MAIN_REFS.get_field(|refs| &refs.ref_count)
}

/// Give up a reference the caller owns, against the main state. A sentinel
/// is a no-op.
pub(crate) fn release_luaref(reference: LuaRef) {
    if reference > 0 {
        main_stack().unreference(reference);
    }
}

/// A second, independently owned reference to the same value.
pub fn api_new_luaref(original_ref: LuaRef) -> LuaRef {
    if original_ref == LUA_NOREF {
        return LUA_NOREF;
    }
    let mut stack = main_stack();
    stack.push_reference(original_ref);
    let new_ref = stack.reference(At::TOP);
    stack.pop(1);
    new_ref
}

/// Whether `reference` refers to something callable — the check an api
/// argument that may be either a callback or a value needs.
pub fn nlua_ref_is_function(reference: LuaRef) -> bool {
    let mut stack = main_stack();
    stack.push_reference(reference);
    let is_function = stack.kind(At::TOP) == Kind::Function;
    stack.pop(1);
    is_function
}
