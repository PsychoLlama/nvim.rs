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

/// `items()` over a blob: a list of `[index, byte]` pairs.
pub(crate) unsafe extern "C" fn tv_blob2items(argvars: *mut typval_T, rettv: *mut typval_T) {
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
pub(crate) unsafe extern "C" fn tv_dict2items(argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        tv_dict2list(argvars, rettv, kDict2ListItems);
    }
}

/// `items()` over a list: a list of `[index, value]` pairs.
pub(crate) unsafe extern "C" fn tv_list2items(argvars: *mut typval_T, rettv: *mut typval_T) {
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
pub(crate) unsafe extern "C" fn tv_string2items(argvars: *mut typval_T, rettv: *mut typval_T) {
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_find(
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
pub unsafe extern "C" fn tv_dict_has_key(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
) -> bool {
    unsafe { !tv_dict_find(d, key, -1).is_null() }
}

/// Copy `d[key]` into `rettv`.  `FAIL` when there is no such key.
pub unsafe extern "C" fn tv_dict_get_tv(
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_get_number(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
) -> varnumber_T {
    unsafe { tv_dict_get_number_def(d, key, 0) }
}

/// `d[key]` as a number, or `def` when there is no such key.
pub unsafe extern "C" fn tv_dict_get_number_def(
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
pub unsafe extern "C" fn tv_dict_get_bool(
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
pub unsafe extern "C" fn tv_dict_to_env(denv: *mut dict_T) -> *mut *mut ::core::ffi::c_char {
    unsafe {
        let env_size = tv_dict_len(denv) as size_t;

        // + 1 for NULL
        let env = xmalloc((env_size + 1) * ::core::mem::size_of::<*mut ::core::ffi::c_char>())
            as *mut *mut ::core::ffi::c_char;

        for (i, hi) in tv_dict_iter(&*denv).enumerate() {
            let var = tv_dict_hi2di(hi);
            let key = tv_dict_item_key(var);
            let str = tv_get_string(&raw mut (*var).di_tv);
            debug_assert!(!str.is_null());
            let len = strlen(key) + strlen(str) + strlen(c"=".as_ptr()) + 1;
            *env.add(i) = xmalloc(len) as *mut ::core::ffi::c_char;
            snprintf(*env.add(i), len, c"%s=%s".as_ptr(), key, str);
        }

        // must be null terminated
        *env.add(env_size as usize) = ::core::ptr::null_mut();
        env
    }
}

/// `d[key]` as a string, using a shared scratch buffer for a number.
///
/// With `save`, the answer is a fresh allocation the caller owns; without it,
/// the answer may point into that shared buffer and only lasts until the next
/// call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_get_string(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    save: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static numbuf: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new([0; 65]);
        let s = tv_dict_get_string_buf(d, key, numbuf.ptr() as *mut ::core::ffi::c_char);
        if save && !s.is_null() {
            return xstrdup(s);
        }
        s.cast_mut()
    }
}

/// `d[key]` as a string, formatting a number into `numbuf`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_get_string_buf(
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_get_string_buf_chk(
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_get_callback(
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
pub unsafe extern "C" fn tv_dict_wrong_func_name(
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
pub(crate) unsafe extern "C" fn tv_dict2list(
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
pub unsafe fn f_keys(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        tv_dict2list(argvars, rettv, kDict2ListKeys);
    }
}

/// `values()`: the values of a dictionary.
pub unsafe fn f_values(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        tv_dict2list(argvars, rettv, kDict2ListValues);
    }
}

/// `has_key()`: whether a dictionary has a key.
pub unsafe fn f_has_key(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        if tv_check_for_dict_arg(argvars, 0) == FAIL {
            return;
        }
        let d = (*argvars).vval.v_dict;
        if d.is_null() {
            return;
        }
        (*rettv).vval.v_number =
            varnumber_T::from(!tv_dict_find(d, tv_get_string(argvars.add(1)), -1).is_null());
    }
}
