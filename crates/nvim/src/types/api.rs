#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
pub struct AdditionalDataBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
pub type ArenaMem = *mut consumed_blk;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Array {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
}
#[derive(Copy, Clone)]
pub struct ArrayBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
    pub init_array: [Object; 16],
}
pub type Boolean = bool;
pub type Buffer = handle_T;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ChangedtickDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 12],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dict {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut KeyValuePair,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error {
    pub type_0: ErrorType,
    pub msg: *mut ::core::ffi::c_char,
}
pub type ErrorType = ::core::ffi::c_int;
/// What an [`Error`] carries, and the one value that means it carries
/// nothing. Every module that reports an API error needs these.
pub const kErrorTypeNone: ErrorType = -1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeValidation: ErrorType = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkInfoArray {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTPair,
}
pub type FieldHashfn = Option<unsafe fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink>;
pub type Float = ::core::ffi::c_double;
pub type HLGroupID = Integer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlMessage {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut HlMessageChunk,
}
pub type Integer = int64_t;
#[derive(Copy, Clone)]
pub struct KeySetLink {
    pub str: *mut ::core::ffi::c_char,
    pub ptr_off: size_t,
    pub type_0: ::core::ffi::c_int,
    pub opt_index: ::core::ffi::c_int,
    pub is_hlgroup: bool,
}
pub type KeyValuePair = key_value_pair;
pub type LuaRef = ::core::ffi::c_int;
pub type MessageType = ::core::ffi::c_int;
pub type Object = object;
pub type ObjectType = ::core::ffi::c_uint;
/// The tags an [`Object`]'s union is read through. Every module that builds
/// or inspects an `Object` needs them, which is why c2rust emitted a copy
/// into each of the sixty-five that do; this is the one definition.
pub const kObjectTypeNil: ObjectType = 0;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeLuaRef: ObjectType = 7;
/// The three the API uses for handles, which never appear in a value the
/// msgpack layer serialises.
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeTabpage: ObjectType = 10;
#[derive(Copy, Clone)]
pub struct OptKeySet {
    pub is_set_: OptionalKeys,
}
pub type OptionalKeys = uint64_t;
pub type Tabpage = handle_T;
pub type Window = handle_T;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_value_pair {
    pub key: String_0,
    pub value: Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct object {
    pub type_0: ObjectType,
    pub data: object_data,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union object_data {
    pub boolean: Boolean,
    pub integer: Integer,
    pub floating: Float,
    pub string: String_0,
    pub array: Array,
    pub dict: Dict,
    pub luaref: LuaRef,
}
