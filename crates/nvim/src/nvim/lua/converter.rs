#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::{
    api_free_array, api_free_dict, api_free_object, api_set_error, api_typename, arena_array,
    arena_dict, arena_string,
};
use crate::src::nvim::eval::decode::{decode_create_map_special_dict, decode_string};
use crate::src::nvim::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_alloc, tv_dict_find, tv_dict_item_alloc_len,
    tv_list_alloc, tv_list_append_list, tv_list_append_owned_tv,
};
use crate::src::nvim::eval::typval::{tv_list_last, tv_list_len, tv_list_ref};
use crate::src::nvim::eval::userfunc::register_luafunc;
use crate::src::nvim::highlight_group::syn_check_group;
use crate::src::nvim::kvec::_memcpy_free;
use crate::src::nvim::lua::executor::{api_free_luaref, nlua_pushref, nlua_ref_global};
use crate::src::nvim::lua::ffi::{
    lua_checkstack, lua_createtable, lua_getmetatable, lua_gettop, lua_next, lua_pushboolean,
    lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushnumber, lua_pushstring, lua_pushvalue,
    lua_rawequal, lua_rawgeti, lua_rawset, lua_rawseti, lua_setmetatable, lua_settop,
    lua_toboolean, lua_tolstring, lua_tonumber, lua_type,
};
use crate::src::nvim::main::nlua_global_refs;
use crate::src::nvim::memory::{arena_memdupz, xfree, xmalloc, xrealloc, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::libc::{__assert_fail, abort, gettext, memchr, memcpy, memset};
use crate::src::nvim::types::{
    Arena, Array, BoolVarValue, Boolean, Dict, Error, FieldHashfn, Float, Integer, KeySetLink,
    KeyValuePair, LuaRef, Object, ObjectType, OptKeySet, OptionalKeys, String_0, VAR_BOOL,
    VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_SPECIAL, VAR_UNKNOWN, VAR_UNLOCKED,
    dictitem_T, handle_T, kBoolVarFalse, kBoolVarTrue, kErrorTypeException, kErrorTypeNone,
    kErrorTypeValidation, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict,
    kObjectTypeFloat, kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString,
    kObjectTypeTabpage, kObjectTypeWindow, kSpecialVarNull, key_value_pair, list_T, lua_Integer,
    lua_Number, lua_State, object, object_data as C2Rust_Unnamed, ptrdiff_t, size_t, typval_T,
    typval_vval_union, varnumber_T,
};
pub type C2Rust_Unnamed_6 = ::core::ffi::c_uint;
pub const kNluaPushFreeRefs: C2Rust_Unnamed_6 = 2;
pub const kNluaPushSpecial: C2Rust_Unnamed_6 = 1;

// The typval-to-Lua direction, carved out of this module's
// `typval_encode.c.h` instantiation.
mod push;
pub use self::push::nlua_push_typval;

// The carve of the transpiled module; see each child's docs.
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TVPopStackItem {
    pub tv: *mut typval_T,
    pub list_len: size_t,
    pub container: bool,
    pub special: bool,
    pub idx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_7 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut TVPopStackItem,
    pub init_array: [TVPopStackItem; 2],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LuaTableProps {
    pub maxidx: size_t,
    pub string_keys_num: size_t,
    pub has_string_with_nul: bool,
    pub type_0: ObjectType,
    pub val: lua_Number,
    pub has_type_key: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ObjPopStackItem {
    pub obj: *mut Object,
    pub container: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_8 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ObjPopStackItem,
    pub init_array: [ObjPopStackItem; 2],
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_TNIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_TBOOLEAN: ::core::ffi::c_int = 1;
pub const LUA_TNUMBER: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LUA_TSTRING: ::core::ffi::c_int = 4;
pub const LUA_TTABLE: ::core::ffi::c_int = 5;
pub const LUA_TFUNCTION: ::core::ffi::c_int = 6;
pub const LUA_TUSERDATA: ::core::ffi::c_int = 7;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INT8_MIN: ::core::ffi::c_int = -128 as ::core::ffi::c_int;
pub const INT64_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INT8_MAX: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const API_INTEGER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const API_INTEGER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const FC_LUAREF: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const TYPE_IDX_VALUE: ::core::ffi::c_int = true_0;
pub const VAL_IDX_VALUE: ::core::ffi::c_int = false_0;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
