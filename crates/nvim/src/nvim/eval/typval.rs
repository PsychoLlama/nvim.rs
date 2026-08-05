#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::charset::vim_str2nr;
use crate::src::nvim::eval::encode::{
    encode_bool_var_names, encode_special_var_names, encode_tv2echo, encode_tv2string,
    encode_vim_list_to_buf,
};
use crate::src::nvim::eval::executor::eexe_mod_op;
use crate::src::nvim::eval::gc::{gc_first_dict, gc_first_list};
use crate::src::nvim::eval::userfunc::{call_func, func_ref, func_unref, get_funccal_local_ht};
use crate::src::nvim::eval::vars::{
    eval_msgpack_type_lists, get_globvar_dict, valid_varname, var_check_fixed, var_check_ro,
    var_wrong_func_name,
};
use crate::src::nvim::eval::{
    callback_call, callback_from_typval, func_equal, get_copyID, partial_name, partial_unref,
    set_selfdict, var_item_copy, var2fpos,
};
use crate::src::nvim::garray::{
    ga_append, ga_append_via_ptr, ga_clear, ga_concat_len, ga_grow, ga_init,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{
    hash_add, hash_clear, hash_find, hash_find_len, hash_init, hash_lock, hash_remove, hash_unlock,
};
use crate::src::nvim::kvec::_memcpy_free;
use crate::src::nvim::lua::executor::{api_free_luaref, api_new_luaref, nlua_funcref_str};
use crate::src::nvim::main::{
    curwin, did_emsg, e_blobidx, e_cannot_change_value, e_cannot_change_value_of_str, e_dictkey,
    e_intern2, e_invalid_value_for_blob_nr, e_invarg, e_invrange, e_list_index_out_of_range_nr,
    e_listarg, e_listreq, e_toomanyarg, e_value_is_locked, e_value_is_locked_str, got_int,
};
use crate::src::nvim::mbyte::{mb_strcmp_ic, string_convert, utf_char2bytes, utfc_ptr2len};
use crate::src::nvim::memory::{
    xcalloc, xfree, xmalloc, xmallocz, xmemdup, xmemdupz, xrealloc, xstrdup, xstrndup,
};
use crate::src::nvim::message::{emsg, internal_error, semsg};
use crate::src::nvim::os::input::{fast_breakcheck, line_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, abort, gettext, memcmp, memcpy, memmove, memset, qsort, snprintf, strcasecmp,
    strcmp, strcoll, strcpy, strlen, strncmp, strtod,
};
use crate::src::nvim::strings::vim_snprintf;
use crate::src::nvim::types::{
    __compar_fn_t, Arena, BoolVarValue, Callback, CallbackType, DictWatcher, EvalFuncData, LuaRef,
    MPConvPartialStage, MPConvStack, MPConvStackVal, MPConvStackVal_data as C2Rust_Unnamed_18,
    MPConvStackVal_data_a as C2Rust_Unnamed_19, MPConvStackVal_data_d as C2Rust_Unnamed_22,
    MPConvStackVal_data_l as C2Rust_Unnamed_21, MPConvStackVal_data_p as C2Rust_Unnamed_20,
    MPConvStackValType, MessagePackType, QUEUE, String_0, VAR_BLOB, VAR_BOOL, VAR_DICT, VAR_FIXED,
    VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_LOCKED, VAR_NO_SCOPE, VAR_NUMBER, VAR_PARTIAL, VAR_SPECIAL,
    VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, VarLockStatus, blob_T, buf_T, dict_T, dictitem_T,
    float_T, funcexe_T, garray_T, hashitem_T, hashtab_T, int64_t, kBoolVarFalse, kBoolVarTrue,
    kListLenMayKnow, kSpecialVarNull, linenr_T, list_T, listitem_T, listwatch_T, partial_T, pos_T,
    ptrdiff_t, queue, size_t, ssize_t, staticList10_T, typval_T, typval_vval_union, ufunc_T,
    uint8_t, uint64_t, uvarnumber_T, varnumber_T, vimconv_T,
};

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
mod nothing_convert;
pub(crate) use self::nothing_convert::*;
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
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const CONV_NONE: C2Rust_Unnamed_17 = 0;
pub const kMPConvPartialEnd: MPConvPartialStage = 2;
pub const kMPConvPartialSelf: MPConvPartialStage = 1;
pub const kMPConvPartialArgs: MPConvPartialStage = 0;
pub const kMPConvPartialList: MPConvStackValType = 4;
pub const kMPConvPartial: MPConvStackValType = 3;
pub const kMPConvPairs: MPConvStackValType = 2;
pub const kMPConvList: MPConvStackValType = 1;
pub const kMPConvDict: MPConvStackValType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Join {
    pub s: String_0,
    pub tofree: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
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
#[repr(C)]
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
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 58] = unsafe {
    ::core::mem::transmute::<[u8; 58], [::core::ffi::c_char; 58]>(
        *b"void tv_list_set_lock(list_T *const, const VarLockStatus)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INT8_MIN: ::core::ffi::c_int = -128 as ::core::ffi::c_int;
pub const INT8_MAX: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX.wrapping_sub(1 as ::core::ffi::c_ulong);
static e_variable_nested_too_deep_for_unlock: GlobalCell<[::core::ffi::c_char; 44]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
            *b"E743: Variable nested too deep for (un)lock\0",
        )
    });
static e_using_invalid_value_as_string: GlobalCell<[::core::ffi::c_char; 41]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
            *b"E908: Using an invalid value as a String\0",
        )
    });
static e_string_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
            *b"E1174: String required for argument %d\0",
        )
    });
static e_non_empty_string_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E1175: Non-empty string required for argument %d\0",
        )
    });
static e_dict_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 43]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
            *b"E1206: Dictionary required for argument %d\0",
        )
    });
static e_number_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
            *b"E1210: Number required for argument %d\0",
        )
    });
static e_list_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E1211: List required for argument %d\0",
        )
    });
static e_bool_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E1212: Bool required for argument %d\0",
        )
    });
static e_float_or_number_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 48]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
            *b"E1219: Float or Number required for argument %d\0",
        )
    });
static e_string_or_number_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E1220: String or Number required for argument %d\0",
        )
    });
static e_string_or_list_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 47]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
            *b"E1222: String or List required for argument %d\0",
        )
    });
static e_list_dict_blob_or_string_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 65]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
            *b"E1225: List, Dictionary, Blob or String required for argument %d\0",
        )
    });
static e_list_or_blob_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
            *b"E1226: List or Blob required for argument %d\0",
        )
    });
static e_blob_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E1238: Blob required for argument %d\0",
        )
    });
static e_string_list_or_blob_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 53]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 53], [::core::ffi::c_char; 53]>(
            *b"E1252: String, List or Blob required for argument %d\0",
        )
    });
static e_string_or_function_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 51]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 51], [::core::ffi::c_char; 51]>(
            *b"E1256: String or function required for argument %d\0",
        )
    });
static e_non_null_dict_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 52]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 52], [::core::ffi::c_char; 52]>(
            *b"E1297: Non-NULL Dictionary required for argument %d\0",
        )
    });
pub static tv_in_free_unref_items: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const DICT_MAXNEST: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub static tv_empty_string: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"\0".as_ptr() as *const ::core::ffi::c_char);
pub const SL_SIZE: usize = ::core::mem::size_of::<[listitem_T; 10]>()
    .wrapping_div(::core::mem::size_of::<listitem_T>())
    .wrapping_div(
        (::core::mem::size_of::<[listitem_T; 10]>()
            .wrapping_rem(::core::mem::size_of::<listitem_T>())
            == 0) as ::core::ffi::c_int as usize,
    );
static sortinfo: GlobalCell<*mut sortinfo_T> =
    GlobalCell::new(::core::ptr::null_mut::<sortinfo_T>());
pub const ITEM_COMPARE_FAIL: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
pub const TYPVAL_ENCODE_ALLOW_SPECIALS: ::core::ffi::c_int = false_0;
static tv_equal_recurse_limit: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static num_errors: GlobalCell<[*const ::core::ffi::c_char; 11]> = GlobalCell::new([
    b"E685: using an invalid value as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"E703: Using a Funcref as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"E745: Using a List as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"E728: Using a Dictionary as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"E805: Using a Float as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"E703: Using a Funcref as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"E974: Using a Blob as a Number\0".as_ptr() as *const ::core::ffi::c_char,
]);
static str_errors: GlobalCell<[*const ::core::ffi::c_char; 11]> = GlobalCell::new([
    (e_using_invalid_value_as_string.as_raw() as *const _) as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"E729: Using a Funcref as a String\0".as_ptr() as *const ::core::ffi::c_char,
    b"E730: Using a List as a String\0".as_ptr() as *const ::core::ffi::c_char,
    b"E731: Using a Dictionary as a String\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"E729: Using a Funcref as a String\0".as_ptr() as *const ::core::ffi::c_char,
    b"E976: Using a Blob as a String\0".as_ptr() as *const ::core::ffi::c_char,
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
pub static _typval_encode_nothing_nodict_var: GlobalCell<*const dict_T> =
    GlobalCell::new(::core::ptr::null::<dict_T>());
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
