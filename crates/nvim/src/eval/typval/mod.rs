#![deny(unsafe_op_in_unsafe_fn)]

use crate::charset::vim_str2nr;
use crate::eval::encode::{
    encode_bool_var_names, encode_special_var_names, encode_tv2echo, encode_tv2string,
};
use crate::eval::executor::eexe_mod_op;
use crate::eval::gc::{gc_first_dict, gc_first_list};
use crate::eval::userfunc::{call_func, func_ref, func_unref, get_funccal_local_ht};
use crate::eval::vars::{
    get_globvar_dict, valid_varname, var_check_fixed, var_check_ro, var_wrong_func_name,
};
use crate::eval::{
    callback_call, callback_from_typval, func_equal, partial_name, partial_unref, set_selfdict,
    var_item_copy, var2fpos,
};
use crate::garray::{ga_append, ga_append_via_ptr, ga_clear, ga_concat_len, ga_grow, ga_init};
use crate::global_cell::GlobalCell;
use crate::hashtab::{
    hash_add, hash_clear, hash_find, hash_find_len, hash_init, hash_lock, hash_remove, hash_unlock,
};
use crate::lua::executor::{api_free_luaref, api_new_luaref, nlua_funcref_str};
use crate::main::{
    c_bytes, curwin, did_emsg, e_blobidx, e_cannot_change_value, e_cannot_change_value_of_str,
    e_dictkey, e_intern2, e_invalid_value_for_blob_nr, e_invarg, e_invrange,
    e_list_index_out_of_range_nr, e_listarg, e_listreq, e_toomanyarg, e_value_is_locked,
    e_value_is_locked_str, got_int,
};
use crate::mbyte::{mb_strcmp_ic, string_convert, utf_char2bytes, utfc_ptr2len};
use crate::memory::{xcalloc, xfree, xmalloc, xmallocz, xmemdup, xmemdupz, xstrdup, xstrndup};
use crate::message::emsg;
use crate::os::cshim::{gettext, memmove, snprintf, strncmp};
use crate::os::input::{fast_breakcheck, line_breakcheck};
use crate::strings::vim_snprintf;
use crate::types::{
    __compar_fn_t, Arena, BoolVarValue, Callback, CallbackType, DictWatcher, EvalFuncData, LuaRef,
    MPConvPartialStage, MPConvStackValType, QUEUE, String_0, VAR_BLOB, VAR_BOOL, VAR_DICT,
    VAR_FIXED, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_LOCKED, VAR_NO_SCOPE, VAR_NUMBER, VAR_PARTIAL,
    VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, VarLockStatus, blob_T, buf_T, dict_T,
    dictitem_T, float_T, funcexe_T, garray_T, hashitem_T, hashtab_T, int64_t, kBoolVarTrue,
    kListLenMayKnow, kSpecialVarNull, linenr_T, list_T, listitem_T, listwatch_T, partial_T,
    ptrdiff_t, size_t, ssize_t, staticList10_T, typval_T, typval_vval_union, ufunc_T, uint8_t,
    varnumber_T, vimconv_T,
};
use ::libc::{abort, memcmp, memcpy, qsort, strcasecmp, strcmp, strcoll, strcpy, strlen, strtod};

// The carve of the transpiled module; see each child's docs.
mod access;
pub use self::access::*;
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
mod dict;
pub use self::dict::*;
mod dictget;
pub use self::dictget::*;
mod blob;
pub use self::blob::*;
mod value;
pub use self::value::*;
mod check;
pub use self::check::*;
mod get;
pub use self::get::*;
mod nothing;
pub(crate) use self::nothing::*;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const DO_NOT_FREE_CNT: C2Rust_Unnamed_13 = 1073741823;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const DI_FLAGS_ALLOC: C2Rust_Unnamed_14 = 16;
pub const DI_FLAGS_FIX: C2Rust_Unnamed_14 = 4;
pub const DI_FLAGS_RO_SBX: C2Rust_Unnamed_14 = 2;
pub const DI_FLAGS_RO: C2Rust_Unnamed_14 = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_15 = 65;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const STR2NR_ALL: C2Rust_Unnamed_16 = 15;
pub const kMPConvPartialEnd: MPConvPartialStage = 2;
pub const kMPConvPartialSelf: MPConvPartialStage = 1;
pub const kMPConvPartialArgs: MPConvPartialStage = 0;
pub const kMPConvPartialList: MPConvStackValType = 4;
pub const kMPConvPartial: MPConvStackValType = 3;
pub const kMPConvPairs: MPConvStackValType = 2;
pub const kMPConvList: MPConvStackValType = 1;
pub const kMPConvDict: MPConvStackValType = 0;
#[derive(Copy, Clone)]
pub struct Join {
    pub s: String_0,
    pub tofree: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
pub struct sortinfo_T {
    pub item_compare_ic: ::core::ffi::c_int,
    pub item_compare_lc: bool,
    pub item_compare_numeric: bool,
    pub item_compare_numbers: bool,
    pub item_compare_float: bool,
    pub item_compare_func: *const ::core::ffi::c_char,
    pub item_compare_partial: *mut partial_T,
    pub item_compare_selfdict: *mut dict_T,
    pub item_compare_func_err: bool,
}
#[derive(Copy, Clone)]
pub struct ListSortItem {
    pub item: *mut listitem_T,
    pub idx: ::core::ffi::c_int,
}
pub type ListSorter = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub type DictListType = ::core::ffi::c_uint;
pub const kDict2ListItems: DictListType = 2;
pub const kDict2ListValues: DictListType = 1;
pub const kDict2ListKeys: DictListType = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INT8_MIN: ::core::ffi::c_int = -128 as ::core::ffi::c_int;
pub const INT8_MAX: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX.wrapping_sub(1 as ::core::ffi::c_ulong);
static e_variable_nested_too_deep_for_unlock: [::core::ffi::c_char; 44] =
    c_bytes(b"E743: Variable nested too deep for (un)lock\0");
static e_using_invalid_value_as_string: [::core::ffi::c_char; 41] =
    c_bytes(b"E908: Using an invalid value as a String\0");
static e_string_required_for_argument_nr: [::core::ffi::c_char; 39] =
    c_bytes(b"E1174: String required for argument %d\0");
static e_non_empty_string_required_for_argument_nr: [::core::ffi::c_char; 49] =
    c_bytes(b"E1175: Non-empty string required for argument %d\0");
static e_dict_required_for_argument_nr: [::core::ffi::c_char; 43] =
    c_bytes(b"E1206: Dictionary required for argument %d\0");
static e_number_required_for_argument_nr: [::core::ffi::c_char; 39] =
    c_bytes(b"E1210: Number required for argument %d\0");
static e_list_required_for_argument_nr: [::core::ffi::c_char; 37] =
    c_bytes(b"E1211: List required for argument %d\0");
static e_bool_required_for_argument_nr: [::core::ffi::c_char; 37] =
    c_bytes(b"E1212: Bool required for argument %d\0");
static e_float_or_number_required_for_argument_nr: [::core::ffi::c_char; 48] =
    c_bytes(b"E1219: Float or Number required for argument %d\0");
static e_string_or_number_required_for_argument_nr: [::core::ffi::c_char; 49] =
    c_bytes(b"E1220: String or Number required for argument %d\0");
static e_string_or_list_required_for_argument_nr: [::core::ffi::c_char; 47] =
    c_bytes(b"E1222: String or List required for argument %d\0");
static e_list_dict_blob_or_string_required_for_argument_nr: [::core::ffi::c_char; 65] =
    c_bytes(b"E1225: List, Dictionary, Blob or String required for argument %d\0");
static e_list_or_blob_required_for_argument_nr: [::core::ffi::c_char; 45] =
    c_bytes(b"E1226: List or Blob required for argument %d\0");
static e_blob_required_for_argument_nr: [::core::ffi::c_char; 37] =
    c_bytes(b"E1238: Blob required for argument %d\0");
static e_string_list_or_blob_required_for_argument_nr: [::core::ffi::c_char; 53] =
    c_bytes(b"E1252: String, List or Blob required for argument %d\0");
static e_string_or_function_required_for_argument_nr: [::core::ffi::c_char; 51] =
    c_bytes(b"E1256: String or function required for argument %d\0");
static e_non_null_dict_required_for_argument_nr: [::core::ffi::c_char; 52] =
    c_bytes(b"E1297: Non-NULL Dictionary required for argument %d\0");
/// A zeroed `sortinfo_T`, which is what a bare `sortinfo_T info;` declaration
/// is before `parse_sort_uniq_args` fills it in.
pub const SORTINFO_INIT: sortinfo_T = sortinfo_T {
    item_compare_ic: 0,
    item_compare_lc: false,
    item_compare_numeric: false,
    item_compare_numbers: false,
    item_compare_float: false,
    item_compare_func: ::core::ptr::null(),
    item_compare_partial: ::core::ptr::null_mut(),
    item_compare_selfdict: ::core::ptr::null_mut(),
    item_compare_func_err: false,
};
/// A zeroed `garray_T`, which is what a bare `garray_T ga;` declaration is
/// before `ga_init` fills it in.  c2rust writes the five fields out at every
/// such declaration.
pub const GARRAY_EMPTY: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: ::core::ptr::null_mut(),
};
/// `TV_INITIAL_VALUE`: an unlocked `VAR_UNKNOWN` object, which is what a
/// `typval_T` is initialised to and what one is left as after being moved out
/// of.  c2rust writes the designated initialiser out at every use site.
pub const TV_INITIAL_VALUE: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};
pub static tv_in_free_unref_items: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const DICT_MAXNEST: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub static tv_empty_string: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(c"".as_ptr());
/// `ARRAY_SIZE(sl->sl_items)`: how many `listitem_T`s a `staticList10_T`
/// embeds.  c2rust renders `ARRAY_SIZE` as a division by the macro's own
/// `== 0` static assertion; the value it computes is just the length.
pub const SL_SIZE: usize = 10;
static sortinfo: GlobalCell<*mut sortinfo_T> =
    GlobalCell::new(::core::ptr::null_mut::<sortinfo_T>());
pub const ITEM_COMPARE_FAIL: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
pub const TYPVAL_ENCODE_ALLOW_SPECIALS: ::core::ffi::c_int = false_0;
static tv_equal_recurse_limit: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static num_errors: GlobalCell<[*const ::core::ffi::c_char; 11]> = GlobalCell::new([
    c"E685: using an invalid value as a Number".as_ptr(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    c"E703: Using a Funcref as a Number".as_ptr(),
    c"E745: Using a List as a Number".as_ptr(),
    c"E728: Using a Dictionary as a Number".as_ptr(),
    c"E805: Using a Float as a Number".as_ptr(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    c"E703: Using a Funcref as a Number".as_ptr(),
    c"E974: Using a Blob as a Number".as_ptr(),
]);
static str_errors: GlobalCell<[*const ::core::ffi::c_char; 11]> = GlobalCell::new([
    e_using_invalid_value_as_string.as_ptr(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    c"E729: Using a Funcref as a String".as_ptr(),
    c"E730: Using a List as a String".as_ptr(),
    c"E731: Using a Dictionary as a String".as_ptr(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    c"E729: Using a Funcref as a String".as_ptr(),
    c"E976: Using a Blob as a String".as_ptr(),
]);
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0 as linenr_T,
    fe_lastline: 0 as linenr_T,
    fe_doesrange: ::core::ptr::null_mut::<bool>(),
    fe_evaluate: false_0 != 0,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false_0 != 0,
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
