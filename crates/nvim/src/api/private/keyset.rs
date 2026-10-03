//! The typed keyset codec.
//!
//! A keyset is one of the `KeyDict_*` option structs in
//! [`crate::types::keysets`]: every field an `Option`, `None` being the key
//! the caller did not name. Five walkers move values in and out of them --
//! from an API dictionary, to one, from and to a Lua table, and from a
//! msgpack map -- and none of them knows which keyset it holds.
//!
//! [`KeySet`] is what they see instead: the field table, a lookup from key to
//! field, and the field itself as a [`Slot`] -- a typed borrow of the
//! `Option<T>` it is. `tools/apigen` implements it for every keyset with a
//! `match` per method, so a walker reaches a field by name and type rather
//! than by byte offset, and no `Option<T>` is ever read through a pointer to
//! some other type.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;

use crate::types::{
    ApiDict, Array, Boolean, Float, Handle, Integer, LuaRef, Object, ObjectType, String_0,
    StringArray, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict,
    kObjectTypeFloat, kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString,
    kObjectTypeWindow,
};

/// What a keyset field holds, and so what a value for it has to arrive as.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum FieldKind {
    /// Any value at all.
    Any,
    Boolean,
    Integer,
    /// A highlight group: it arrives as a name or an id and is stored as the
    /// id, so its slot is an [`Integer`].
    HlGroup,
    Float,
    String,
    Array,
    Dict,
    LuaRef,
    Buffer,
    Window,
    /// ShaDa's array of strings, which only the msgpack reader fills.
    StringArray,
}

impl FieldKind {
    /// The API type a value for the field is reported as expecting.
    pub(crate) fn object_type(self) -> ObjectType {
        match self {
            FieldKind::Any => kObjectTypeNil,
            FieldKind::Boolean => kObjectTypeBoolean,
            FieldKind::Integer | FieldKind::HlGroup => kObjectTypeInteger,
            FieldKind::Float => kObjectTypeFloat,
            FieldKind::String => kObjectTypeString,
            FieldKind::Array | FieldKind::StringArray => kObjectTypeArray,
            FieldKind::Dict => kObjectTypeDict,
            FieldKind::LuaRef => kObjectTypeLuaRef,
            FieldKind::Buffer => kObjectTypeBuffer,
            FieldKind::Window => kObjectTypeWindow,
        }
    }
}

/// One field of a keyset: the key clients spell it with and what it holds.
pub(crate) struct KeyField {
    pub(crate) name: &'static CStr,
    pub(crate) kind: FieldKind,
}

impl KeyField {
    pub(crate) const fn new(name: &'static CStr, kind: FieldKind) -> Self {
        KeyField { name, kind }
    }
}

/// One keyset field, borrowed as the `Option` it is.
pub(crate) enum Slot<'a> {
    Any(&'a mut Option<Object>),
    Boolean(&'a mut Option<Boolean>),
    /// An [`FieldKind::Integer`] or [`FieldKind::HlGroup`] field.
    Integer(&'a mut Option<Integer>),
    Float(&'a mut Option<Float>),
    String(&'a mut Option<String_0>),
    Array(&'a mut Option<Array>),
    Dict(&'a mut Option<ApiDict>),
    LuaRef(&'a mut Option<LuaRef>),
    /// A buffer or window field: the [`KeyField`] says which. No keyset
    /// holds a tab page yet; `FieldKind` gains the variant when one does.
    Handle(&'a mut Option<Handle>),
    StringArray(&'a mut Option<StringArray>),
}

impl Slot<'_> {
    /// Whether the field holds a value: the caller named its key.
    pub(crate) fn is_set(&self) -> bool {
        match self {
            Slot::Any(v) => v.is_some(),
            Slot::Boolean(v) => v.is_some(),
            Slot::Integer(v) => v.is_some(),
            Slot::Float(v) => v.is_some(),
            Slot::String(v) => v.is_some(),
            Slot::Array(v) => v.is_some(),
            Slot::Dict(v) => v.is_some(),
            Slot::LuaRef(v) => v.is_some(),
            Slot::Handle(v) => v.is_some(),
            Slot::StringArray(v) => v.is_some(),
        }
    }
}

/// A `KeyDict_*` struct, as the walkers that fill and read one see it.
///
/// Implemented by `tools/apigen` for every keyset. Methods take `self` so the
/// trait stays usable as `dyn KeySet`: the walkers are written once, not
/// once per keyset.
pub(crate) trait KeySet {
    /// The keyset's fields, in the order a keyset comes back out as a
    /// dictionary.
    fn fields(&self) -> &'static [KeyField];

    /// The index into [`fields`](KeySet::fields) of the field `key` names.
    fn find(&self, key: &[u8]) -> Option<usize>;

    /// Field `index`, borrowed. Panics on an index `fields` does not have.
    fn slot(&mut self, index: usize) -> Slot<'_>;
}
