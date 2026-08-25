//! Reading values back out of a `dict_T`.
//!
//! [`tv_dict_find`] is the hashtable lookup every getter goes through, and
//! the `tv_dict_get_*` family coerces what it finds to one type, answering a
//! caller-supplied default when the key is absent or the wrong kind.
//! [`tv_dict_to_env`] builds the `environ`-shaped array a job's environment
//! is passed as.  The `*2items` half and [`f_items`] / [`f_keys`] /
//! [`f_values`] are the builtins that turn a container into a list.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{FAIL, NUL, OK};

/// `items()` over a blob: a list of `[index, byte]` pairs.
///
/// # Safety
/// `argvars[0]` must be a `VAR_BLOB`; argvars` must point at the builtin's argument array, terminated by a
/// `VAR_UNKNOWN`, and `rettv` at a writable `typval_T` holding no value
/// yet.
pub(crate) unsafe fn tv_blob2items(argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        let blob = (*argvars).vval.v_blob;
        tv_list_alloc_ret(rettv, tv_blob_len(blob) as ptrdiff_t);
        for i in 0..tv_blob_len(blob) {
            let l2 = tv_list_alloc(2);
            tv_list_append_list((*rettv).vval.v_list, l2);
            tv_list_append_number(l2, i as varnumber_T);
            tv_list_append_number(l2, tv_blob_get(blob, i) as varnumber_T);
        }
    }
}

/// `items()` over a dictionary: a list of `[key, value]` pairs.
///
/// # Safety
/// `argvars[0]` must be a `VAR_DICT`; argvars` must point at the builtin's argument array, terminated by a
/// `VAR_UNKNOWN`, and `rettv` at a writable `typval_T` holding no value
/// yet.
pub(crate) unsafe fn tv_dict2items(argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        tv_dict2list(argvars, rettv, kDict2ListItems);
    }
}

/// `items()` over a list: a list of `[index, value]` pairs.
///
/// # Safety
/// `argvars[0]` must be a `VAR_LIST`; argvars` must point at the builtin's argument array, terminated by a
/// `VAR_UNKNOWN`, and `rettv` at a writable `typval_T` holding no value
/// yet.
pub(crate) unsafe fn tv_list2items(argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        let l = (*argvars).vval.v_list;
        tv_list_alloc_ret(rettv, tv_list_len(l) as ptrdiff_t);
        if l.is_null() {
            return;
        }
        for (idx, li) in tv_list_iter(l.as_ref()).enumerate() {
            let l2 = tv_list_alloc(2);
            tv_list_append_list((*rettv).vval.v_list, l2);
            tv_list_append_number(l2, idx as varnumber_T);
            tv_list_append_tv(l2, &raw mut (*li).li_tv);
        }
    }
}

/// `items()` over a string: a list of `[index, character]` pairs.
///
/// # Safety
/// `argvars[0]` must be a `VAR_STRING` whose value is null or
/// NUL-terminated; argvars` must point at the builtin's argument array, terminated by a
/// `VAR_UNKNOWN`, and `rettv` at a writable `typval_T` holding no value
/// yet.
pub(crate) unsafe fn tv_string2items(argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        let mut p = (*argvars).vval.v_string.cast_const();

        tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
        if p.is_null() {
            return; // null string behaves like an empty string
        }

        let mut idx: varnumber_T = 0;
        while *p as ::core::ffi::c_int != NUL {
            let len = utfc_ptr2len(p);
            if len == 0 {
                break;
            }
            let l2 = tv_list_alloc(2);
            tv_list_append_list((*rettv).vval.v_list, l2);
            tv_list_append_number(l2, idx);
            tv_list_append_string(l2, p, len as ssize_t);
            p = p.offset(len as isize);
            idx += 1;
        }
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
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    len: ptrdiff_t,
) -> *mut dictitem_T {
    unsafe {
        if d.is_null() {
            return ::core::ptr::null_mut();
        }
        let hi = if len < 0 {
            hash_find(&raw const (*d).dv_hashtab, key)
        } else {
            hash_find_len(&raw const (*d).dv_hashtab, key, len as size_t)
        };
        if !(*hi).is_kept() {
            return ::core::ptr::null_mut();
        }
        tv_dict_hi2di(hi)
    }
}

/// Whether `d` has `key`.
///
/// # Safety
/// `d` is null or points at a live dictionary, and `key` must be a
/// NUL-terminated string.
pub unsafe fn tv_dict_has_key(d: *const dict_T, key: *const ::core::ffi::c_char) -> bool {
    unsafe { !tv_dict_find(d, key, -1).is_null() }
}

/// Copy `d[key]` into `rettv`.  `FAIL` when there is no such key.
///
/// # Safety
/// `d` is null or points at a live dictionary, `key` must be a
/// NUL-terminated string, and `rettv` must point at a writable `typval_T`
/// holding no value yet.
pub unsafe fn tv_dict_get_tv(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let di = tv_dict_find(d, key, -1);
        if di.is_null() {
            return FAIL;
        }
        tv_copy(&raw mut (*di).di_tv, rettv);
        OK
    }
}

/// `d[key]` as a number, or 0 when there is no such key.
///
/// # Safety
/// `d` is null or points at a live dictionary, and `key` must be a
/// NUL-terminated string. Coercing the value can raise an error, so the
/// caller must be on the editor's main thread.
pub unsafe fn tv_dict_get_number(d: *const dict_T, key: *const ::core::ffi::c_char) -> varnumber_T {
    unsafe { tv_dict_get_number_def(d, key, 0) }
}

/// `d[key]` as a number, or `def` when there is no such key.
///
/// # Safety
/// `d` is null or points at a live dictionary, and `key` must be a
/// NUL-terminated string. Coercing the value can raise an error, so the
/// caller must be on the editor's main thread.
pub unsafe fn tv_dict_get_number_def(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    def: ::core::ffi::c_int,
) -> varnumber_T {
    unsafe {
        let di = tv_dict_find(d, key, -1);
        if di.is_null() {
            return def as varnumber_T;
        }
        tv_get_number(&raw mut (*di).di_tv)
    }
}

/// `d[key]` as a boolean, or `def` when there is no such key.
///
/// # Safety
/// `d` is null or points at a live dictionary, and `key` must be a
/// NUL-terminated string.
pub unsafe fn tv_dict_get_bool(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    def: ::core::ffi::c_int,
) -> varnumber_T {
    unsafe {
        let di = tv_dict_find(d, key, -1);
        if di.is_null() {
            return def as varnumber_T;
        }
        tv_get_bool(&raw mut (*di).di_tv)
    }
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
pub unsafe fn tv_dict_to_env(denv: *mut dict_T) -> *mut *mut ::core::ffi::c_char {
    let mut numbuf = NumBuf::new();
    unsafe {
        let env_size = tv_dict_len(denv) as size_t;

        // + 1 for NULL
        let env = xmalloc((env_size + 1) * ::core::mem::size_of::<*mut ::core::ffi::c_char>())
            as *mut *mut ::core::ffi::c_char;

        for (i, hi) in tv_dict_iter(&*denv).enumerate() {
            let var = tv_dict_hi2di(hi);
            let key = tv_dict_item_key(var);
            let str = numbuf.string(&raw mut (*var).di_tv);
            debug_assert!(!str.is_null());
            let len = strlen(key) + strlen(str) + c"=".count_bytes() + 1;
            *env.add(i) = xmalloc(len) as *mut ::core::ffi::c_char;
            snprintf(*env.add(i), len, c"%s=%s".as_ptr(), key, str);
        }

        // must be null terminated
        *env.add(env_size as usize) = ::core::ptr::null_mut();
        env
    }
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
    d: *const dict_T,
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
/// `d` is null or points at a live dictionary, `key` must be a
/// NUL-terminated string, and `numbuf` must be writable for `NUMBUFLEN`
/// bytes. The answer may point into `numbuf` or borrow the item.
pub unsafe fn tv_dict_get_string_buf(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    numbuf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let di = tv_dict_find(d, key, -1);
        if di.is_null() {
            return ::core::ptr::null();
        }
        tv_get_string_buf(&raw const (*di).di_tv, numbuf)
    }
}

/// [`tv_dict_get_string_buf`] answering `def` for a missing key, and NULL with
/// an error raised for a value that has no string form.
///
/// # Safety
/// `d` is null or points at a live dictionary, `key` must be readable for
/// `key_len` bytes (or NUL-terminated when it is negative), and `numbuf`
/// must be writable for `NUMBUFLEN` bytes. `def` is returned as-is for a
/// missing key, so its lifetime is the caller's problem.
pub unsafe fn tv_dict_get_string_buf_chk(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    key_len: ptrdiff_t,
    numbuf: *mut ::core::ffi::c_char,
    def: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let di = tv_dict_find(d, key, key_len);
        if di.is_null() {
            return def;
        }
        tv_get_string_buf_chk(&raw const (*di).di_tv, numbuf)
    }
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
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: ptrdiff_t,
    result: *mut Callback,
) -> bool {
    unsafe {
        (*result).type_0 = kCallbackNone;
        let di = tv_dict_find(d, key, key_len);
        if di.is_null() {
            return true;
        }
        if !tv_is_func((*di).di_tv) && (*di).di_tv.v_type != VAR_STRING {
            emsg(gettext(
                c"E6000: Argument is not a function or function name".as_ptr(),
            ));
            return false;
        }

        let mut tv = TV_INITIAL_VALUE;
        tv_copy(&raw mut (*di).di_tv, &raw mut tv);
        set_selfdict(&raw mut tv, d);
        let res = callback_from_typval(result, &raw mut tv);
        tv_clear(&raw mut tv);
        res
    }
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
    d: *mut dict_T,
    tv: *mut typval_T,
    name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        ((d == get_globvar_dict() || &raw mut (*d).dv_hashtab == get_funccal_local_ht())
            && tv_is_func(*tv)
            && var_wrong_func_name(name, true)) as ::core::ffi::c_int
    }
}

/// The shared body of `keys()`, `values()` and `items()` over a dictionary.
///
/// # Safety
/// `argvars` must point at the builtin's argument array, terminated by a
/// `VAR_UNKNOWN`, and `rettv` at a writable `typval_T` holding no value
/// yet.
pub(crate) unsafe fn tv_dict2list(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    what: DictListType,
) {
    unsafe {
        if tv_check_for_dict_arg(argvars, 0) == FAIL {
            tv_list_alloc_ret(rettv, 0);
            return;
        }

        let d = (*argvars).vval.v_dict;
        tv_list_alloc_ret(rettv, tv_dict_len(d) as ptrdiff_t);
        if d.is_null() {
            // NULL dict behaves like an empty dict
            return;
        }

        for hi in tv_dict_iter(&*d) {
            let di = tv_dict_hi2di(hi);
            let di_key = tv_dict_item_key(di);
            let mut tv_item = TV_INITIAL_VALUE;

            match what {
                kDict2ListKeys => {
                    tv_item.v_type = VAR_STRING;
                    tv_item.vval.v_string = xstrdup(di_key);
                }
                kDict2ListValues => {
                    tv_copy(&raw mut (*di).di_tv, &raw mut tv_item);
                }
                kDict2ListItems => {
                    // items()
                    let sub_l = tv_list_alloc(2);
                    tv_item.v_type = VAR_LIST;
                    tv_item.vval.v_list = sub_l;
                    tv_list_ref(sub_l);
                    tv_list_append_string(sub_l, di_key, -1);
                    tv_list_append_tv(sub_l, &raw mut (*di).di_tv);
                }
                _ => {}
            }

            tv_list_append_owned_tv((*rettv).vval.v_list, tv_item);
        }
    }
}

/// `items()`: index/value pairs of a string, list, blob or dictionary.
///
/// # Safety
/// `argvars` must point at the builtin's argument array, terminated by a
/// `VAR_UNKNOWN`, and `rettv` at a writable `typval_T` holding no value
/// yet.
pub unsafe fn f_items(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        match (*argvars).v_type {
            VAR_STRING => tv_string2items(argvars, rettv),
            VAR_LIST => tv_list2items(argvars, rettv),
            VAR_BLOB => tv_blob2items(argvars, rettv),
            VAR_DICT => tv_dict2items(argvars, rettv),
            _ => {
                semsg_c!(
                    gettext(e_list_dict_blob_or_string_required_for_argument_nr.as_ptr(),),
                    1 as ::core::ffi::c_int,
                );
            }
        }
    }
}

/// `keys()`: the keys of a dictionary.
///
/// # Safety
/// `argvars` must point at the builtin's argument array, terminated by a
/// `VAR_UNKNOWN`, and `rettv` at a writable `typval_T` holding no value
/// yet.
pub unsafe fn f_keys(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        tv_dict2list(argvars, rettv, kDict2ListKeys);
    }
}

/// `values()`: the values of a dictionary.
///
/// # Safety
/// `argvars` must point at the builtin's argument array, terminated by a
/// `VAR_UNKNOWN`, and `rettv` at a writable `typval_T` holding no value
/// yet.
pub unsafe fn f_values(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        tv_dict2list(argvars, rettv, kDict2ListValues);
    }
}

/// `has_key()`: whether a dictionary has a key.
///
/// # Safety
/// `argvars` must point at the builtin's argument array, terminated by a
/// `VAR_UNKNOWN`, and `rettv` at a writable `typval_T` holding no value
/// yet.
pub unsafe fn f_has_key(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe {
        if tv_check_for_dict_arg(argvars, 0) == FAIL {
            return;
        }
        let d = (*argvars).vval.v_dict;
        if d.is_null() {
            return;
        }
        (*rettv).vval.v_number =
            varnumber_T::from(!tv_dict_find(d, numbuf.string(argvars.add(1)), -1).is_null());
    }
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
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char {
        // SAFETY: the caller's dictionary and key.
        unsafe { tv_dict_get_string_buf(d, key, self.as_mut_ptr()) }
    }
}
