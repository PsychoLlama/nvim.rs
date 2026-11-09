#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::eval::DICT_MAXNEST;
use crate::eval::encode::{BOOL_VAR_NAMES, SPECIAL_VAR_NAMES};
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::gc::{RootId, root_dict, root_list, unroot_dict, unroot_list};
use crate::eval::userfunc::{is_funccal_local_dict, set_selfdict};
use crate::eval::vars::is_globvar_dict;
use crate::eval::{callback_call, callback_from_typval, func_equal, var2fpos};
use crate::getchar::state::GOT_INT;
use crate::global_cell::GlobalCell;
use crate::hashtab::Slot;
use crate::lua::executor::release_luaref;
use crate::mbyte::VimConv;
use crate::memory::{xcalloc, xfree};
use crate::message::emsg;
use crate::message::state::DID_EMSG;
use crate::message::{
    E_CANNOT_CHANGE_VALUE, E_CANNOT_CHANGE_VALUE_OF_STR, E_INVARG, E_INVRANGE, E_LISTREQ,
    E_VALUE_IS_LOCKED, E_VALUE_IS_LOCKED_STR,
};
use crate::os::cshim::gettext;
use crate::os::input::{fast_breakcheck, line_breakcheck};
use crate::os::libc::{int64_t, ptrdiff_t};
use crate::types::{
    BOOL_VAR_TRUE, Blob, BoolVarValue, Callback, Dict, DictWatcher, Float, LIST_LEN_MAY_KNOW,
    LineNr, List, ListItem, ListWatch, LuaRef, Partial, SPECIAL_VAR_NULL, SpecialVarValue, TypVal,
    VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NO_SCOPE, VAR_NUMBER,
    VAR_PARTIAL, VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VarLock, VarNumber,
};

// The carve of the transpiled module; see each child's docs.
mod access;
pub use self::access::*;
mod handle;
pub use self::handle::*;
mod edit;
pub use self::edit::Edit;
mod frame;
pub(crate) use self::frame::CallFrame;
mod list;
pub use self::list::*;
mod listops;
pub use self::listops::*;
mod listrange;
pub use self::listrange::*;
mod sort;
pub use self::sort::*;
mod watcher;
pub use self::watcher::*;
pub use crate::eval::callback::{callback_copy, callback_free, callback_put, callback_to_string};
mod dict;
pub use self::dict::*;
mod dictitem;
pub use self::dictitem::*;
mod blob;
pub use self::blob::*;
mod partial;
pub(crate) use self::partial::*;
mod value;
pub use self::value::*;
mod check;
pub use self::check::*;
mod get;
pub use self::get::*;
mod release;
mod string;
/// The refcount an item that must never be freed carries.
pub const DO_NOT_FREE_CNT: ::core::ffi::c_int = 1073741823;
/// `DictItem::di_flags` bits: read-only, read-only in the sandbox, fixed
/// (cannot be deleted), locked, and the item's own allocation.
pub const DI_FLAGS_RO: u8 = 1;
pub const DI_FLAGS_RO_SBX: u8 = 2;
pub const DI_FLAGS_FIX: u8 = 4;
pub const DI_FLAGS_LOCK: u8 = 8;
/// A `c_uint`, unlike its siblings, for the item store's reader in
/// `handle.rs`, which widens the field to meet it.
pub const DI_FLAGS_ALLOC: ::core::ffi::c_uint = 16;
/// Enough for any number this module formats, plus its NUL.
pub const NUMBUFLEN: usize = 65;
pub type DictListType = ::core::ffi::c_uint;
pub const DICT2_LIST_ITEMS: DictListType = 2;
pub const DICT2_LIST_VALUES: DictListType = 1;
pub const DICT2_LIST_KEYS: DictListType = 0;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INT8_MIN: ::core::ffi::c_int = -128 as ::core::ffi::c_int;
pub const INT8_MAX: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX.wrapping_sub(1 as ::core::ffi::c_ulong);
static E_VARIABLE_NESTED_TOO_DEEP_FOR_UNLOCK: &::core::ffi::CStr =
    c"E743: Variable nested too deep for (un)lock";
static E_USING_INVALID_VALUE_AS_STRING: &::core::ffi::CStr =
    c"E908: Using an invalid value as a String";
static E_STRING_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1174: String required for argument %d";
static E_NON_EMPTY_STRING_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1175: Non-empty string required for argument %d";
static E_DICT_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1206: Dictionary required for argument %d";
static E_NUMBER_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1210: Number required for argument %d";
static E_LIST_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1211: List required for argument %d";
static E_BOOL_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1212: Bool required for argument %d";
static E_FLOAT_OR_NUMBER_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1219: Float or Number required for argument %d";
static E_STRING_OR_NUMBER_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1220: String or Number required for argument %d";
static E_STRING_OR_LIST_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1222: String or List required for argument %d";
static E_LIST_OR_BLOB_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1226: List or Blob required for argument %d";
static E_BLOB_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1238: Blob required for argument %d";
static E_STRING_LIST_OR_BLOB_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1252: String, List or Blob required for argument %d";
static E_STRING_OR_FUNCTION_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1256: String or function required for argument %d";
static E_NON_NULL_DICT_REQUIRED_FOR_ARGUMENT_NR: &::core::ffi::CStr =
    c"E1297: Non-NULL Dictionary required for argument %d";
/// `TV_INITIAL_VALUE`: the `VAR_UNKNOWN` a `TypVal` is initialised to and
/// left as after being moved out of.  c2rust wrote the designated
/// initialiser out at every use site.
///
/// A `const` rather than the bare [`TypVal::Unknown`] because a repeated
/// array (`[TV_INITIAL_VALUE; 21]`, every argument frame in the interpreter)
/// needs a constant on its left, the element type not being `Copy`.
pub const TV_INITIAL_VALUE: TypVal = TypVal::Unknown;

pub(crate) static TV_IN_FREE_UNREF_ITEMS: GlobalCell<bool> = GlobalCell::new(false);
/// How many submatches a `\=` replacement expression is handed: `\0`
/// through `\9`.
pub const SL_SIZE: usize = 10;
pub const TYPVAL_ENCODE_ALLOW_SPECIALS: ::core::ffi::c_int = 0;
static TV_EQUAL_RECURSE_LIMIT: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
/// What a value that has no Number form is called when one is asked for,
/// by its kind. The empty entries are the kinds that do have one.
static NUM_ERRORS: [&::core::ffi::CStr; 11] = [
    c"E685: using an invalid value as a Number",
    c"",
    c"",
    c"E703: Using a Funcref as a Number",
    c"E745: Using a List as a Number",
    c"E728: Using a Dictionary as a Number",
    c"E805: Using a Float as a Number",
    c"",
    c"",
    c"E703: Using a Funcref as a Number",
    c"E974: Using a Blob as a Number",
];
/// [`NUM_ERRORS`] for a String.
static STR_ERRORS: [&::core::ffi::CStr; 11] = [
    E_USING_INVALID_VALUE_AS_STRING,
    c"",
    c"",
    c"E729: Using a Funcref as a String",
    c"E730: Using a List as a String",
    c"E731: Using a Dictionary as a String",
    c"",
    c"",
    c"",
    c"E729: Using a Funcref as a String",
    c"E976: Using a Blob as a String",
];
