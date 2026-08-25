//! `json_decode()` and `msgpackparse()`: text and msgpack bytes into typvals.
//!
//! The direction opposite [`super::encode`], and the shape is opposite too:
//! there is no shared walk here, because upstream writes one parser per
//! format by hand.  The two live in [`json`] and [`msgpack`].
//!
//! What they do share is this file.  Both formats can carry values Vimscript
//! has no type for — a map whose keys are not plain non-empty strings, an
//! unsigned integer past `VARNUMBER_MAX`, a msgpack `ext` — and both answer
//! with the same **special dictionary**, `{_TYPE: v:msgpack_types.<kind>,
//! _VAL: <payload>}`, which [`create_special_dict`] builds and the msgpack
//! encoder reads back.  Both also have to decide, for every run of bytes,
//! whether it is a string or a blob; that is [`decode_string`].

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};

use crate::eval::typval::{
    TV_INITIAL_VALUE, tv_blob_alloc_ret, tv_dict_add, tv_dict_alloc, tv_dict_item_alloc_len,
    tv_list_alloc, tv_list_ref,
};
use crate::eval::vars::msgpack_type_list;
use crate::garray::ga_concat_len;
use crate::memory::xmemdupz;
use crate::types::{
    MessagePackType, VAR_DICT, VAR_LIST, VAR_STRING, VAR_UNLOCKED, dictitem_T, list_T, ptrdiff_t,
    size_t, typval_T, typval_vval_union,
};
use ::libc::memchr;

mod json;
mod msgpack;

pub use self::json::json_decode_string;
pub use self::msgpack::{mpack_parse_typval, typval_parser_error_free, unpack_typval};

/// The three `v:msgpack_types` entries the decoders name.  The rest of the
/// enum belongs to the encoder, which declares its own copy.
pub(crate) const kMPInteger: MessagePackType = 2;
pub(crate) const kMPMap: MessagePackType = 6;
pub(crate) const kMPExt: MessagePackType = 7;

/// Build `{_TYPE: v:msgpack_types.<type>, _VAL: val}` into `rettv`.
///
/// `val` is moved into the `_VAL` key — the caller must not clear it — and
/// the `_TYPE` value is a reference to one of the shared, immutable lists
/// `v:msgpack_types` is made of, not a copy of it.  The dictionary comes back
/// with one reference on it.
///
/// # Safety
/// `rettv` is writable and holds no value that needs clearing.
#[inline]
pub(crate) unsafe fn create_special_dict(
    rettv: *mut typval_T,
    type_: MessagePackType,
    val: typval_T,
) {
    unsafe {
        let dict = tv_dict_alloc();

        let type_di: *mut dictitem_T =
            tv_dict_item_alloc_len("_TYPE".as_ptr() as *const c_char, "_TYPE".len());
        (*type_di).di_tv.v_type = VAR_LIST;
        (*type_di).di_tv.v_lock = VAR_UNLOCKED;
        (*type_di).di_tv.vval.v_list = msgpack_type_list(type_);
        tv_list_ref((*type_di).di_tv.vval.v_list);
        tv_dict_add(dict, type_di);

        let val_di: *mut dictitem_T =
            tv_dict_item_alloc_len("_VAL".as_ptr() as *const c_char, "_VAL".len());
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

/// The special dictionary a map that cannot be a `dict_T` decodes to.
///
/// `len` sizes the `_VAL` list in advance (see `ListLenSpecials`); it is only
/// a hint, and underfilling the list is allowed.  The returned list is the
/// one the caller fills with two-element key/value pairs — it is owned by the
/// dictionary written to `ret_tv`, so the return value may be ignored.
///
/// # Safety
/// `ret_tv` is writable and holds no value that needs clearing.
pub unsafe fn decode_create_map_special_dict(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T {
    unsafe {
        let list = tv_list_alloc(len);
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
        list
    }
}

/// `len` bytes at `s` as a `typval_T`: a `VAR_STRING`, or a `VAR_BLOB` when
/// it cannot be one.
///
/// A Vimscript string is NUL-terminated, so a run containing an embedded NUL
/// has to become a blob; `force_blob` asks for one either way.  `s_allocated`
/// says the caller is handing over ownership of `s`: it is then stored as it
/// stands — as the blob's `garray_T` buffer, or as the string itself — rather
/// than copied.  A NULL `s`, which is only legal with `len == 0`, stays NULL.
///
/// # Safety
/// `s` points at `len` readable bytes, or is NULL with `len == 0`; if
/// `s_allocated` it came from `xmalloc`, and its ownership passes to the
/// returned value.
pub unsafe fn decode_string(
    s: *const c_char,
    len: size_t,
    force_blob: bool,
    s_allocated: bool,
) -> typval_T {
    debug_assert!(!s.is_null() || len == 0);
    unsafe {
        if force_blob || (!s.is_null() && !memchr(s.cast(), 0, len).is_null()) {
            let mut tv = TV_INITIAL_VALUE;
            let b = tv_blob_alloc_ret(&raw mut tv);
            if s_allocated {
                // The caller's allocation becomes the blob's, sized exactly to
                // `len`: nothing is copied and nothing is left to grow into.
                (*b).bv_ga.ga_data = s as *mut c_void;
                (*b).bv_ga.ga_len = len as c_int;
                (*b).bv_ga.ga_maxlen = len as c_int;
            } else {
                ga_concat_len(&raw mut (*b).bv_ga, s, len);
            }
            return tv;
        }
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: if s.is_null() || s_allocated {
                    s as *mut c_char
                } else {
                    xmemdupz(s.cast(), len) as *mut c_char
                },
            },
        }
    }
}
