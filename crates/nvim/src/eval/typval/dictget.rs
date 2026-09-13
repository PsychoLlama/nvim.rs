//! Reading values back out of a `Dict`.
//!
//! [`tv_dict_find`] is the hashtable lookup every getter goes through, and
//! the `tv_dict_get_*` family coerces what it finds to one type, answering a
//! caller-supplied default when the key is absent or the wrong kind.
//! [`tv_dict_to_env`] builds the `environ`-shaped array a job's environment
//! is passed as.  The `*2items` half and [`f_items`] / [`f_keys`] /
//! [`f_values`] are the builtins that turn a container into a list.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::cstr;
use crate::message::emsg_ptr;
use crate::semsg;
use crate::types::Failed;
use crate::types::NUL;

/// `items()` over a blob: a list of `[index, byte]` pairs.
pub(crate) fn tv_blob2items(args: &[TypVal], result: &mut TypVal) {
    let bytes = blob_bytes(args[0].blob_ref());
    tv_list_alloc_ret(result, ptrdiff_t::try_from(bytes.len()).unwrap_or(-1));
    for (at, &byte) in bytes.iter().enumerate() {
        let pair = tv_list_alloc(2);
        let into = pair.as_ptr();
        // SAFETY: the list stored in the return slot, and the fresh pair.
        unsafe { (*result.list_or_null()).push_list(Some(pair)) };
        unsafe { (*into).push_number(VarNumber::try_from(at).expect("a short blob")) };
        unsafe { (*into).push_number(VarNumber::from(byte)) };
    }
}

/// `items()` over a dictionary: a list of `[key, value]` pairs.
pub(crate) fn tv_dict2items(args: &[TypVal], result: &mut TypVal) {
    tv_dict2list(args, result, kDict2ListItems);
}

/// `items()` over a list: a list of `[index, value]` pairs.
pub(crate) fn tv_list2items(args: &[TypVal], result: &mut TypVal) {
    let l = args[0].list_or_null();
    tv_list_alloc_ret(result, list_len(unsafe { l.as_ref() }) as ptrdiff_t);
    if l.is_null() {
        return;
    }
    for (idx, li) in list_iter(unsafe { l.as_ref() }).enumerate() {
        let l2 = tv_list_alloc(2);
        let at = l2.as_ptr();
        unsafe { (*(*result).list_or_null()).push_list(Some(l2)) };
        unsafe { (*at).push_number(idx as VarNumber) };
        unsafe { (*at).push_copy(&li.li_tv) };
    }
}

/// `items()` over a string: a list of `[index, character]` pairs.
pub(crate) fn tv_string2items(args: &[TypVal], result: &mut TypVal) {
    let mut p = args[0].string_or_null().cast_const();

    tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    if p.is_null() {
        return; // null string behaves like an empty string
    }

    let mut idx: VarNumber = 0;
    while unsafe { *p } as ::core::ffi::c_int != NUL {
        let len = unsafe { utfc_ptr2len(p) };
        if len == 0 {
            break;
        }
        let l2 = tv_list_alloc(2);
        let at = l2.as_ptr();
        unsafe { (*(*result).list_or_null()).push_list(Some(l2)) };
        unsafe { (*at).push_number(idx) };
        unsafe { (*at).push_string(p, len as ssize_t) };
        p = unsafe { p.offset(len as isize) };
        idx += 1;
    }
}

/// The item `d[key]`, or NULL when there is none.
///
/// A negative `len` means `key` is NUL-terminated.
///
/// # Safety
/// `d` is null or points at a live dictionary. `key` must be readable for
/// `len` bytes, or NUL-terminated when `len` is negative. The item borrows
/// the dictionary.
pub unsafe fn tv_dict_find(
    d: *const Dict,
    key: *const ::core::ffi::c_char,
    len: ptrdiff_t,
) -> *mut DictItem {
    if d.is_null() {
        return ::core::ptr::null_mut();
    }
    let hi = if len < 0 {
        unsafe { hash_find(&raw const (*d).dv_hashtab, key) }
    } else {
        unsafe { hash_find_len(&raw const (*d).dv_hashtab, key, len as size_t) }
    };
    if !hi.is_kept() {
        return ::core::ptr::null_mut();
    }
    tv_dict_hi2di(hi)
}

/// Whether `d` has `key`.
///
/// # Safety
/// `d` is null or points at a live dictionary, and `key` must be a
/// NUL-terminated string.
pub unsafe fn tv_dict_has_key(d: *const Dict, key: *const ::core::ffi::c_char) -> bool {
    unsafe { !tv_dict_find(d, key, -1).is_null() }
}

/// Copy `d[key]` into `result`.  `Err` when there is no such key.
///
/// # Safety
/// `d` is null or points at a live dictionary, `key` must be a
/// NUL-terminated string, and `result` must point at a writable `TypVal`
/// holding no value yet.
pub unsafe fn tv_dict_get_tv(
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    result: &mut TypVal,
) -> Result<(), Failed> {
    let di = unsafe { tv_dict_find(d, key, -1) };
    if di.is_null() {
        return Err(Failed);
    }
    unsafe { tv_copy(&(*di).di_tv, result) };
    Ok(())
}

/// `d[key]` as a number, or 0 when there is no such key.
///
/// # Safety
/// `d` is null or points at a live dictionary, and `key` must be a
/// NUL-terminated string. Coercing the value can raise an error, so the
/// caller must be on the editor's main thread.
pub unsafe fn tv_dict_get_number(d: *const Dict, key: *const ::core::ffi::c_char) -> VarNumber {
    unsafe { tv_dict_get_number_def(d, key, 0) }
}

/// `d[key]` as a number, or `def` when there is no such key.
///
/// # Safety
/// `d` is null or points at a live dictionary, and `key` must be a
/// NUL-terminated string. Coercing the value can raise an error, so the
/// caller must be on the editor's main thread.
pub unsafe fn tv_dict_get_number_def(
    d: *const Dict,
    key: *const ::core::ffi::c_char,
    def: ::core::ffi::c_int,
) -> VarNumber {
    let di = unsafe { tv_dict_find(d, key, -1) };
    if di.is_null() {
        return def as VarNumber;
    }
    unsafe { tv_get_number(&(*di).di_tv) }
}

/// `d[key]` as a boolean, or `def` when there is no such key.
///
/// # Safety
/// `d` is null or points at a live dictionary, and `key` must be a
/// NUL-terminated string.
pub unsafe fn tv_dict_get_bool(
    d: *const Dict,
    key: *const ::core::ffi::c_char,
    def: ::core::ffi::c_int,
) -> VarNumber {
    let di = unsafe { tv_dict_find(d, key, -1) };
    if di.is_null() {
        return def as VarNumber;
    }
    unsafe { tv_get_bool(&(*di).di_tv) }
}

/// `denv` as a NULL-terminated `environ`-shaped array of `KEY=VALUE` strings.
///
/// Every string, and the array itself, is freshly allocated; the caller owns
/// the lot.
///
/// # Safety
/// `denv` must point at a live dictionary — **not** null — every value of
/// which has a string form. The array and every string in it are the
/// caller's to free.
pub unsafe fn tv_dict_to_env(denv: *mut Dict) -> *mut *mut ::core::ffi::c_char {
    let mut numbuf = NumBuf::new();
    let env_size = unsafe { tv_dict_len(denv) } as size_t;

    // + 1 for NULL
    let env =
        unsafe { xmalloc((env_size + 1) * ::core::mem::size_of::<*mut ::core::ffi::c_char>()) }
            as *mut *mut ::core::ffi::c_char;

    for (i, hi) in unsafe { tv_dict_iter(denv) }.enumerate() {
        let var = tv_dict_hi2di(hi);
        let key = unsafe { tv_dict_item_key(var) };
        // SAFETY: the iterator's own item.
        let str = numbuf.string_ptr(unsafe { &(*var).di_tv });
        debug_assert!(!str.is_null());
        let len = unsafe { cstr::bytes_at(key) }.len()
            + unsafe { cstr::bytes_at(str) }.len()
            + c"=".count_bytes()
            + 1;
        unsafe { *env.add(i) = xmalloc(len) as *mut ::core::ffi::c_char };
        unsafe { snprintf(*env.add(i), len, c"%s=%s".as_ptr(), key, str) };
    }

    // must be null terminated
    unsafe { *env.add(env_size as usize) = ::core::ptr::null_mut() };
    env
}

/// `d[key]` as a fresh allocation the caller owns, NULL for a missing key.
///
/// The `save` half of the C's `tv_dict_get_string`; the borrowing half is
/// [`tv_dict_get_string_buf`], which renders into the caller's own
/// [`NumBuf`] rather than a process-wide one.
///
/// # Safety
/// `d` is null or points at a live dictionary, and `key` must be a
/// NUL-terminated string.
pub unsafe fn tv_dict_get_string_alloc(
    d: *const Dict,
    key: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's dictionary and key; the scratch is this frame's.
    let s = unsafe { numbuf.dict_string(d, key) };
    if s.is_null() {
        return ::core::ptr::null_mut();
    }
    // SAFETY: a non-null answer is a NUL-terminated string.
    unsafe { xstrdup(s) }
}

/// `d[key]` as a string, formatting a number into `numbuf`.
///
/// # Safety
/// `d` is null or points at a live dictionary and `key` must be a
/// NUL-terminated string. The answer may point into `numbuf` or borrow the
/// item.
pub unsafe fn tv_dict_get_string_buf(
    d: *const Dict,
    key: *const ::core::ffi::c_char,
    numbuf: &mut NumBuf,
) -> *const ::core::ffi::c_char {
    let di = unsafe { tv_dict_find(d, key, -1) };
    if di.is_null() {
        return ::core::ptr::null();
    }
    // SAFETY: the item just found belongs to the caller's dictionary.
    numbuf.string_ptr(unsafe { &(*di).di_tv })
}

/// [`tv_dict_get_string_buf`] answering `def` for a missing key, and NULL with
/// an error raised for a value that has no string form.
///
/// # Safety
/// `d` is null or points at a live dictionary, `key` must be readable for
/// `key_len` bytes (or NUL-terminated when it is negative). `def` is
/// returned as-is for a missing key, so its lifetime is the caller's
/// problem.
pub unsafe fn tv_dict_get_string_buf_chk(
    d: *const Dict,
    key: *const ::core::ffi::c_char,
    key_len: ptrdiff_t,
    numbuf: &mut NumBuf,
    def: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let di = unsafe { tv_dict_find(d, key, key_len) };
    if di.is_null() {
        return def;
    }
    // SAFETY: the item just found belongs to the caller's dictionary.
    numbuf.string_ptr_chk(unsafe { &(*di).di_tv })
}

/// `d[key]` as a callback, bound to `d` as its `self` dictionary.
///
/// A missing key answers true with `result` left as `kCallbackNone`; a value
/// that is neither a function nor a string answers false with `E6000` raised.
///
/// # Safety
/// `d` must point at a live dictionary, `key` must be readable for
/// `key_len` bytes (or NUL-terminated when it is negative), and `result`
/// must point at a writable `Callback` holding no callback yet — it is
/// overwritten, not freed. On `true` the caller owns whatever it now
/// holds.
pub unsafe fn tv_dict_get_callback(
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: ptrdiff_t,
    result: *mut Callback,
) -> bool {
    unsafe { *result = Callback::None };
    let di = unsafe { tv_dict_find(d, key, key_len) };
    if di.is_null() {
        return true;
    }
    if !unsafe { (*di).di_tv.is_func() } && unsafe { (*di).di_tv.v_type() } != VAR_STRING {
        let msg = tr(c"E6000: Argument is not a function or function name");
        unsafe { emsg_ptr(msg) };
        return false;
    }

    let mut tv = TV_INITIAL_VALUE;
    unsafe { tv_copy(&(*di).di_tv, &mut tv) };
    unsafe { set_selfdict(&mut tv, d) };
    let res = unsafe { callback_from_typval(result, &tv) };
    tv_clear(&mut tv);
    res
}

/// Whether storing `tv` under `name` in `d` would shadow a builtin function.
///
/// Only the global scope and a function's local scope are guarded.
///
/// # Safety
/// `d` must point at a live dictionary, `tv` at an initialised value, and
/// `name` at a NUL-terminated string. The global and function-local scope
/// dictionaries are read, so the caller must be on the editor's main
/// thread.
pub unsafe fn tv_dict_wrong_func_name(
    d: *mut Dict,
    tv: &mut TypVal,
    name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    ((d == get_globvar_dict() || dv_hashtab(d) == get_funccal_local_ht())
        && (*tv).is_func()
        && unsafe { var_wrong_func_name(name, true) }) as ::core::ffi::c_int
}

/// The shared body of `keys()`, `values()` and `items()` over a dictionary.
pub(crate) fn tv_dict2list(args: &[TypVal], result: &mut TypVal, what: DictListType) {
    if tv_check_for_dict_arg(args, 0).is_err() {
        tv_list_alloc_ret(result, 0);
        return;
    }

    let d = args[0].dict_or_null();
    tv_list_alloc_ret(result, unsafe { tv_dict_len(d) } as ptrdiff_t);
    if d.is_null() {
        // NULL dict behaves like an empty dict
        return;
    }

    for hi in unsafe { tv_dict_iter(d) } {
        let di = tv_dict_hi2di(hi);
        let di_key = unsafe { tv_dict_item_key(di) };
        let mut tv_item = TV_INITIAL_VALUE;

        match what {
            kDict2ListKeys => {
                tv_item.write_string(unsafe { xstrdup(di_key) });
            }
            kDict2ListValues => {
                unsafe { tv_copy(&(*di).di_tv, &mut tv_item) };
            }
            kDict2ListItems => {
                // items()
                let sub_l = tv_list_alloc(2);
                let at = sub_l.as_ptr();
                tv_item.write_list(Some(sub_l));
                unsafe { (*at).push_string(di_key, -1) };
                unsafe { (*at).push_copy(&(*di).di_tv) };
            }
            _ => {}
        }

        unsafe { (*(*result).list_or_null()).push(tv_item) };
    }
}

/// `items()`: index/value pairs of a string, list, blob or dictionary.
pub fn f_items(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    match args[0].v_type() {
        VAR_STRING => tv_string2items(args, result),
        VAR_LIST => tv_list2items(args, result),
        VAR_BLOB => tv_blob2items(args, result),
        VAR_DICT => tv_dict2items(args, result),
        _ => {
            semsg!(
                "E1225: List, Dictionary, Blob or String required for argument {}",
                1 as ::core::ffi::c_int
            );
        }
    }
}

/// `keys()`: the keys of a dictionary.
pub fn f_keys(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_dict2list(args, result, kDict2ListKeys);
}

/// `values()`: the values of a dictionary.
pub fn f_values(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_dict2list(args, result, kDict2ListValues);
}

/// `has_key()`: whether a dictionary has a key.
pub fn f_has_key(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if tv_check_for_dict_arg(args, 0).is_err() {
        return;
    }
    let d = args[0].dict_or_null();
    if d.is_null() {
        return;
    }
    let key = numbuf.string_ptr(&args[1]);
    let found = !unsafe { tv_dict_find(d, key, -1) }.is_null();
    result.write_number(VarNumber::from(found));
}

impl NumBuf {
    /// `d[key]` as a string, NULL for a missing key. The borrowing half of
    /// the C's `tv_dict_get_string`; [`tv_dict_get_string_alloc`] is the
    /// other one.
    ///
    /// # Safety
    /// `d` is null or points at a live dictionary, and `key` must be a
    /// NUL-terminated string.
    pub unsafe fn dict_string(
        &mut self,
        d: *const Dict,
        key: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char {
        // SAFETY: the caller's dictionary and key.
        unsafe { tv_dict_get_string_buf(d, key, self) }
    }
}
