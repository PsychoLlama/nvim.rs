#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::mpack::conv::{
    mpack_unpack_boolean, mpack_unpack_float_fast, mpack_unpack_sint, mpack_unpack_uint,
};
use crate::src::mpack::object::{mpack_parse, mpack_parser_init};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::src::nvim::charset::vim_str2nr;
use crate::src::nvim::eval::encode::encode_list_write;
use crate::src::nvim::eval::string2float;
use crate::src::nvim::eval::typval::{
    tv_blob_alloc_ret, tv_clear, tv_dict_add, tv_dict_alloc, tv_dict_find, tv_dict_item_alloc,
    tv_dict_item_alloc_len, tv_list_alloc, tv_list_append_list, tv_list_append_number,
    tv_list_append_owned_tv,
};
use crate::src::nvim::eval::typval::{tv_list_len, tv_list_ref};
use crate::src::nvim::eval::vars::eval_msgpack_type_lists;
use crate::src::nvim::garray::ga_concat_len;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::mbyte::{utf_char2bytes, utf_char2len, utf_ptr2char, utf_ptr2len};
use crate::src::nvim::memory::{xfree, xmalloc, xmallocz, xmemdupz, xrealloc};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::libc::{__assert_fail, abort, gettext, memchr, memcpy, strlen, strncmp};
use crate::src::nvim::types::{
    BoolVarValue, MessagePackType, VAR_BOOL, VAR_DICT, VAR_FLOAT, VAR_LIST, VAR_NUMBER,
    VAR_SPECIAL, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, blob_T, dict_T, dictitem_T, hashitem_T,
    hashtab_T, kBoolVarFalse, kBoolVarTrue, kListLenMayKnow, kSpecialVarNull, list_T, mpack_data_t,
    mpack_node_t, mpack_parser_t, mpack_tokbuf_t, mpack_token_s_data as C2Rust_Unnamed_0,
    mpack_token_t, mpack_token_type_t, mpack_uint32_t, mpack_value_t, ptrdiff_t, size_t, typval_T,
    typval_vval_union, uint8_t, uint32_t, uint64_t, uvarnumber_T, varnumber_T,
};

// The carve of the transpiled module; see each child's docs.
mod json;
mod msgpack;

pub use self::json::json_decode_string;
pub use self::msgpack::{mpack_parse_typval, typval_parser_error_free, unpack_typval};
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MPACK_OK: C2Rust_Unnamed = 0;
pub const MPACK_TOKEN_CHUNK: mpack_token_type_t = 6;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const STR2NR_FORCE: C2Rust_Unnamed_2 = 128;
pub const STR2NR_HEX: C2Rust_Unnamed_2 = 4;
pub const kMPExt: MessagePackType = 7;
pub const kMPMap: MessagePackType = 6;
pub const kMPInteger: MessagePackType = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = '\u{8}' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = 9;
pub const NL: ::core::ffi::c_int = 10;
pub const FF: ::core::ffi::c_int = '\u{c}' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = 13;
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn create_special_dict(
    rettv: *mut typval_T,
    type_0: MessagePackType,
    mut val: typval_T,
) {
    unsafe {
        let dict: *mut dict_T = tv_dict_alloc();
        let type_di: *mut dictitem_T = tv_dict_item_alloc_len(
            b"_TYPE\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        );
        (*type_di).di_tv.v_type = VAR_LIST;
        (*type_di).di_tv.v_lock = VAR_UNLOCKED;
        (*type_di).di_tv.vval.v_list =
            (*eval_msgpack_type_lists.ptr())[type_0 as usize] as *mut list_T;
        tv_list_ref((*type_di).di_tv.vval.v_list);
        tv_dict_add(dict, type_di);
        let val_di: *mut dictitem_T = tv_dict_item_alloc_len(
            b"_VAL\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        );
        (*val_di).di_tv = val;
        tv_dict_add(dict, val_di);
        (*dict).dv_refcount += 1;
        *rettv = typval_T {
            v_type: VAR_DICT,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_dict: dict },
        };
    }
}
pub unsafe extern "C" fn decode_create_map_special_dict(
    ret_tv: *mut typval_T,
    len: ptrdiff_t,
) -> *mut list_T {
    unsafe {
        let list: *mut list_T = tv_list_alloc(len);
        tv_list_ref(list);
        create_special_dict(
            ret_tv,
            kMPMap,
            typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_list: list },
            },
        );
        return list;
    }
}
pub unsafe extern "C" fn decode_string(
    s: *const ::core::ffi::c_char,
    len: size_t,
    mut force_blob: bool,
    s_allocated: bool,
) -> typval_T {
    unsafe {
        '_c2rust_label: {
            if !s.is_null() || len == 0 as size_t {
            } else {
                __assert_fail(
                    b"s != NULL || len == 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/decode.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    261 as ::core::ffi::c_uint,
                    b"typval_T decode_string(const char *const, const size_t, _Bool, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let use_blob: bool = force_blob as ::core::ffi::c_int != 0
            || !s.is_null() && !memchr(s as *const ::core::ffi::c_void, NUL, len).is_null();
        if use_blob {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            tv.v_lock = VAR_UNLOCKED;
            let mut b: *mut blob_T = tv_blob_alloc_ret(&raw mut tv);
            if s_allocated {
                (*b).bv_ga.ga_data = s as *mut ::core::ffi::c_void;
                (*b).bv_ga.ga_len = len as ::core::ffi::c_int;
                (*b).bv_ga.ga_maxlen = len as ::core::ffi::c_int;
            } else {
                ga_concat_len(&raw mut (*b).bv_ga, s, len);
            }
            return tv;
        }
        return typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: (if s.is_null() || s_allocated as ::core::ffi::c_int != 0 {
                    s as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void
                } else {
                    xmemdupz(s as *const ::core::ffi::c_void, len)
                }) as *mut ::core::ffi::c_char,
            },
        };
    }
}
pub const SURROGATE_HI_START: ::core::ffi::c_int = 0xd800 as ::core::ffi::c_int;
pub const SURROGATE_HI_END: ::core::ffi::c_int = 0xdbff as ::core::ffi::c_int;
pub const SURROGATE_LO_START: ::core::ffi::c_int = 0xdc00 as ::core::ffi::c_int;
pub const SURROGATE_LO_END: ::core::ffi::c_int = 0xdfff as ::core::ffi::c_int;
pub const SURROGATE_FIRST_CHAR: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
