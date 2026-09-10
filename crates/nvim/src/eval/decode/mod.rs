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
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_void};

use crate::eval::typval::{
    Bl, Di, ListRef, TV_INITIAL_VALUE, di_tv, tv_blob_alloc_ret, tv_dict_add, tv_dict_alloc,
    tv_dict_item_alloc_len, tv_list_alloc,
};
use crate::eval::vars::msgpack_type_list;
use crate::garray::ga_concat_len;
use crate::memory::xmemdupz;
use crate::types::{DictItem, List, MessagePackType, TypVal, VAR_LIST, VarLock, ptrdiff_t, size_t};
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

/// Build `{_TYPE: v:msgpack_types.<type>, _VAL: val}` into `result`.
///
/// `val` is moved into the `_VAL` key — the caller must not clear it — and
/// the `_TYPE` value is a reference to one of the shared, immutable lists
/// `v:msgpack_types` is made of, not a copy of it.  The dictionary comes back
/// with one reference on it.
///
/// # Safety
/// `result` is writable and holds no value that needs clearing.
#[inline]
pub(crate) unsafe fn create_special_dict(result: &mut TypVal, type_: MessagePackType, val: TypVal) {
    let dict_held = tv_dict_alloc();
    let dict = dict_held.as_ptr();

    let type_di: *mut DictItem =
        unsafe { tv_dict_item_alloc_len("_TYPE".as_ptr() as *const c_char, "_TYPE".len()) };
    // SAFETY: the item just added to the special dictionary.
    let mut type_item = unsafe { Di::new(type_di) };
    type_item.di_tv.write_empty(VAR_LIST);
    type_item.di_lock = VarLock::Unlocked;
    let type_list = unsafe { ListRef::retained(msgpack_type_list(type_)) };
    type_item.di_tv.write_list(type_list);
    let _ = unsafe { tv_dict_add(dict, type_di) };

    let val_di: *mut DictItem =
        unsafe { tv_dict_item_alloc_len("_VAL".as_ptr() as *const c_char, "_VAL".len()) };
    unsafe { di_tv(val_di).write(val) };
    let _ = unsafe { tv_dict_add(dict, val_di) };

    unsafe { ::core::ptr::write(result, TypVal::Dict(Some(dict_held))) };
}

/// The special dictionary a map that cannot be a `Dict` decodes to.
///
/// `len` sizes the `_VAL` list in advance (see `ListLenSpecials`); it is only
/// a hint, and underfilling the list is allowed.  The returned list is the
/// one the caller fills with two-element key/value pairs — it is owned by the
/// dictionary written to `ret_tv`, so the return value may be ignored.
///
/// # Safety
/// `ret_tv` is writable and holds no value that needs clearing.
pub unsafe fn decode_create_map_special_dict(ret_tv: &mut TypVal, len: ptrdiff_t) -> *mut List {
    let list = tv_list_alloc(len);
    // A borrow of the list the special dictionary owns from here on.
    let into = list.as_ptr();
    unsafe { create_special_dict(ret_tv, kMPMap, TypVal::List(Some(list))) };
    into
}

/// `len` bytes at `s` as a `TypVal`: a `VAR_STRING`, or a `VAR_BLOB` when
/// it cannot be one.
///
/// A Vimscript string is NUL-terminated, so a run containing an embedded NUL
/// has to become a blob; `force_blob` asks for one either way.  `s_allocated`
/// says the caller is handing over ownership of `s`: it is then stored as it
/// stands — as the blob's `GArray` buffer, or as the string itself — rather
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
) -> TypVal {
    debug_assert!(!s.is_null() || len == 0);
    if force_blob || (!s.is_null() && !unsafe { memchr(s.cast(), 0, len) }.is_null()) {
        let mut tv = TV_INITIAL_VALUE;
        let b = unsafe { tv_blob_alloc_ret(&mut tv) };
        if s_allocated {
            // The caller's allocation becomes the blob's, sized exactly to
            // `len`: nothing is copied and nothing is left to grow into.
            unsafe { (*b).bv_ga.ga_data = s as *mut c_void };
            // SAFETY: freshly allocated just above.
            let mut bl = unsafe { Bl::new(b) };
            bl.bv_ga.ga_len = len as c_int;
            bl.bv_ga.ga_maxlen = len as c_int;
        } else {
            unsafe { ga_concat_len(&raw mut (*b).bv_ga, s, len) };
        }
        return tv;
    }
    TypVal::String(if s.is_null() || s_allocated {
        s as *mut c_char
    } else {
        unsafe { xmemdupz(s.cast(), len) as *mut c_char }
    })
}
