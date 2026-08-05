//! Allocating, freeing and filling a `dict_T`.
//!
//! [`tv_dict_alloc`] and [`tv_dict_unref`] are the reference-counted pair;
//! [`tv_dict_clear`] empties one without freeing it.  The `tv_dict_add_*`
//! family is the C header's overload set, each taking a key by pointer and
//! length and copying exactly that many bytes.  [`tv_dict_extend`] is
//! `extend()` with its three `action` modes, [`tv_dict_copy`] is
//! `copy()`/`deepcopy()` over a dictionary.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_item_alloc_len(
    key: *const ::core::ffi::c_char,
    key_len: size_t,
) -> *mut dictitem_T {
    unsafe {
        let di: *mut dictitem_T = xmalloc(
            if ::core::mem::size_of::<dictitem_T>()
                > (17 as size_t)
                    .wrapping_add(key_len)
                    .wrapping_add(1 as size_t)
            {
                ::core::mem::size_of::<dictitem_T>()
            } else {
                (17 as size_t)
                    .wrapping_add(key_len)
                    .wrapping_add(1 as size_t)
            },
        ) as *mut dictitem_T;
        memcpy(
            &raw mut (*di).di_key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            key as *const ::core::ffi::c_void,
            key_len,
        );
        *(&raw mut (*di).di_key as *mut ::core::ffi::c_char).offset(key_len as isize) =
            NUL as ::core::ffi::c_char;
        (*di).di_flags = DI_FLAGS_ALLOC as ::core::ffi::c_int as uint8_t;
        (*di).di_tv.v_lock = VAR_UNLOCKED;
        (*di).di_tv.v_type = VAR_UNKNOWN;
        return di;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_item_alloc(key: *const ::core::ffi::c_char) -> *mut dictitem_T {
    unsafe {
        return tv_dict_item_alloc_len(key, strlen(key));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_item_free(item: *mut dictitem_T) {
    unsafe {
        tv_clear(&raw mut (*item).di_tv);
        if (*item).di_flags as ::core::ffi::c_int & DI_FLAGS_ALLOC as ::core::ffi::c_int != 0 {
            xfree(item as *mut ::core::ffi::c_void);
        }
    }
}

pub unsafe extern "C" fn tv_dict_item_copy(di: *mut dictitem_T) -> *mut dictitem_T {
    unsafe {
        let new_di: *mut dictitem_T =
            tv_dict_item_alloc(&raw mut (*di).di_key as *mut ::core::ffi::c_char);
        tv_copy(&raw mut (*di).di_tv, &raw mut (*new_di).di_tv);
        return new_di;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_item_remove(dict: *mut dict_T, item: *mut dictitem_T) {
    unsafe {
        let hi: *mut hashitem_T = hash_find(
            &raw mut (*dict).dv_hashtab,
            &raw mut (*item).di_key as *mut ::core::ffi::c_char,
        );
        if (*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            semsg(
                gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                b"tv_dict_item_remove()\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            hash_remove(&raw mut (*dict).dv_hashtab, hi);
        }
        tv_dict_item_free(item);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_alloc() -> *mut dict_T {
    unsafe {
        let d: *mut dict_T = xcalloc(1 as size_t, ::core::mem::size_of::<dict_T>()) as *mut dict_T;
        if !(*gc_first_dict.ptr()).is_null() {
            (*gc_first_dict.get()).dv_used_prev = d;
        }
        (*d).dv_used_next = gc_first_dict.get();
        (*d).dv_used_prev = ::core::ptr::null_mut::<dict_T>();
        gc_first_dict.set(d);
        hash_init(&raw mut (*d).dv_hashtab);
        (*d).dv_lock = VAR_UNLOCKED;
        (*d).dv_scope = VAR_NO_SCOPE;
        (*d).dv_refcount = 0 as ::core::ffi::c_int;
        (*d).dv_copyID = 0 as ::core::ffi::c_int;
        QUEUE_INIT(&raw mut (*d).watchers);
        (*d).lua_table_ref = LUA_NOREF as LuaRef;
        return d;
    }
}

pub unsafe extern "C" fn tv_dict_free_contents(d: *mut dict_T) {
    unsafe {
        hash_lock(&raw mut (*d).dv_hashtab);
        '_c2rust_label: {
            if (*d).dv_hashtab.ht_locked > 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"d->dv_hashtab.ht_locked > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2163 as ::core::ffi::c_uint,
                    b"void tv_dict_free_contents(dict_T *const)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let hiht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
        let mut hitodo_: size_t = (*hiht_).ht_used;
        let mut hi: *mut hashitem_T = (*hiht_).ht_array;
        while hitodo_ != 0 {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                hitodo_ = hitodo_.wrapping_sub(1);
                let di: *mut dictitem_T =
                    (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
                hash_remove(&raw mut (*d).dv_hashtab, hi);
                tv_dict_item_free(di);
            }
            hi = hi.offset(1);
        }
        while !QUEUE_EMPTY(&raw mut (*d).watchers) {
            let mut w: *mut QUEUE = (*d).watchers.next as *mut QUEUE;
            QUEUE_REMOVE(w);
            let mut watcher: *mut DictWatcher = tv_dict_watcher_node_data(w);
            tv_dict_watcher_free(watcher);
        }
        hash_clear(&raw mut (*d).dv_hashtab);
        (*d).dv_hashtab.ht_locked -= 1;
        hash_init(&raw mut (*d).dv_hashtab);
    }
}

pub unsafe extern "C" fn tv_dict_free_dict(d: *mut dict_T) {
    unsafe {
        if (*d).dv_used_prev.is_null() {
            gc_first_dict.set((*d).dv_used_next);
        } else {
            (*(*d).dv_used_prev).dv_used_next = (*d).dv_used_next;
        }
        if !(*d).dv_used_next.is_null() {
            (*(*d).dv_used_next).dv_used_prev = (*d).dv_used_prev;
        }
        if (*d).lua_table_ref != LUA_NOREF {
            api_free_luaref((*d).lua_table_ref);
            (*d).lua_table_ref = LUA_NOREF as LuaRef;
        }
        xfree(d as *mut ::core::ffi::c_void);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_free(d: *mut dict_T) {
    unsafe {
        if tv_in_free_unref_items.get() {
            return;
        }
        tv_dict_free_contents(d);
        tv_dict_free_dict(d);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_unref(d: *mut dict_T) {
    unsafe {
        if !d.is_null() && {
            (*d).dv_refcount -= 1;
            (*d).dv_refcount <= 0 as ::core::ffi::c_int
        } {
            tv_dict_free(d);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int {
    unsafe {
        if tv_dict_wrong_func_name(
            d,
            &raw mut (*item).di_tv,
            &raw mut (*item).di_key as *mut ::core::ffi::c_char,
        ) != 0
        {
            return FAIL;
        }
        return hash_add(
            &raw mut (*d).dv_hashtab,
            &raw mut (*item).di_key as *mut ::core::ffi::c_char,
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_list(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    list: *mut list_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_LIST;
        (*item).di_tv.vval.v_list = list;
        tv_list_ref(list);
        if tv_dict_add(d, item) == FAIL {
            tv_dict_item_free(item);
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_dict_add_tv(
    mut d: *mut dict_T,
    mut key: *const ::core::ffi::c_char,
    key_len: size_t,
    mut tv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
        tv_copy(tv, &raw mut (*item).di_tv);
        if tv_dict_add(d, item) == FAIL {
            tv_dict_item_free(item);
            return FAIL;
        }
        return OK;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_dict(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    dict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_DICT;
        (*item).di_tv.vval.v_dict = dict;
        (*dict).dv_refcount += 1;
        if tv_dict_add(d, item) == FAIL {
            tv_dict_item_free(item);
            return FAIL;
        }
        return OK;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_nr(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    nr: varnumber_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_NUMBER;
        (*item).di_tv.vval.v_number = nr;
        if tv_dict_add(d, item) == FAIL {
            tv_dict_item_free(item);
            return FAIL;
        }
        return OK;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_float(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    nr: float_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_FLOAT;
        (*item).di_tv.vval.v_float = nr;
        if tv_dict_add(d, item) == FAIL {
            tv_dict_item_free(item);
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_dict_add_bool(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    mut val: BoolVarValue,
) -> ::core::ffi::c_int {
    unsafe {
        let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_BOOL;
        (*item).di_tv.vval.v_bool = val;
        if tv_dict_add(d, item) == FAIL {
            tv_dict_item_free(item);
            return FAIL;
        }
        return OK;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_str(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        return tv_dict_add_str_len(d, key, key_len, val, -1 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn tv_dict_add_str_len(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !val.is_null() {
            s = if len < 0 as ::core::ffi::c_int {
                xstrdup(val)
            } else {
                xstrndup(val, len as size_t)
            };
        }
        return tv_dict_add_allocated_str(d, key, key_len, s);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_allocated_str(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_STRING;
        (*item).di_tv.vval.v_string = val;
        if tv_dict_add(d, item) == FAIL {
            tv_dict_item_free(item);
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_dict_add_func(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    fp: *mut ufunc_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_FUNC;
        (*item).di_tv.vval.v_string = xmemdupz(
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            (*fp).uf_namelen,
        ) as *mut ::core::ffi::c_char;
        if tv_dict_add(d, item) == FAIL {
            tv_dict_item_free(item);
            return FAIL;
        }
        func_ref((*item).di_tv.vval.v_string);
        return OK;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_clear(d: *mut dict_T) {
    unsafe {
        hash_lock(&raw mut (*d).dv_hashtab);
        '_c2rust_label: {
            if (*d).dv_hashtab.ht_locked > 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"d->dv_hashtab.ht_locked > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2698 as ::core::ffi::c_uint,
                    b"void tv_dict_clear(dict_T *const)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let hiht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
        let mut hitodo_: size_t = (*hiht_).ht_used;
        let mut hi: *mut hashitem_T = (*hiht_).ht_array;
        while hitodo_ != 0 {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                hitodo_ = hitodo_.wrapping_sub(1);
                tv_dict_item_free(
                    (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T
                );
                hash_remove(&raw mut (*d).dv_hashtab, hi);
            }
            hi = hi.offset(1);
        }
        hash_unlock(&raw mut (*d).dv_hashtab);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_extend(
    d1: *mut dict_T,
    d2: *mut dict_T,
    action: *const ::core::ffi::c_char,
) {
    unsafe {
        let watched: bool = tv_dict_is_watched(d1);
        let arg_errmsg: *const ::core::ffi::c_char =
            gettext(b"extend() argument\0".as_ptr() as *const ::core::ffi::c_char);
        let arg_errmsg_len: size_t = strlen(arg_errmsg);
        if *action as ::core::ffi::c_int == 'm' as ::core::ffi::c_int {
            hash_lock(&raw mut (*d2).dv_hashtab);
        }
        let hi2ht_: *mut hashtab_T = &raw mut (*d2).dv_hashtab;
        let mut hi2todo_: size_t = (*hi2ht_).ht_used;
        let mut hi2: *mut hashitem_T = (*hi2ht_).ht_array;
        while hi2todo_ != 0 {
            if !((*hi2).hi_key.is_null()
                || (*hi2).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                hi2todo_ = hi2todo_.wrapping_sub(1);
                let di2: *mut dictitem_T =
                    (*hi2).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
                let di1: *mut dictitem_T = tv_dict_find(
                    d1,
                    &raw mut (*di2).di_key as *mut ::core::ffi::c_char,
                    -1 as ptrdiff_t,
                );
                if (*d1).dv_scope as ::core::ffi::c_uint
                    != VAR_NO_SCOPE as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !valid_varname(&raw mut (*di2).di_key as *mut ::core::ffi::c_char)
                {
                    break;
                }
                if di1.is_null() {
                    if *action as ::core::ffi::c_int == 'm' as ::core::ffi::c_int {
                        let new_di: *mut dictitem_T = di2;
                        if tv_dict_add(d1, new_di) == 1 as ::core::ffi::c_int {
                            hash_remove(&raw mut (*d2).dv_hashtab, hi2);
                            tv_dict_watcher_notify(
                                d1,
                                &raw mut (*new_di).di_key as *mut ::core::ffi::c_char,
                                &raw mut (*new_di).di_tv,
                                ::core::ptr::null_mut::<typval_T>(),
                            );
                        }
                    } else {
                        let new_di_0: *mut dictitem_T = tv_dict_item_copy(di2);
                        if tv_dict_add(d1, new_di_0) == 0 as ::core::ffi::c_int {
                            tv_dict_item_free(new_di_0);
                        } else if watched {
                            tv_dict_watcher_notify(
                                d1,
                                &raw mut (*new_di_0).di_key as *mut ::core::ffi::c_char,
                                &raw mut (*new_di_0).di_tv,
                                ::core::ptr::null_mut::<typval_T>(),
                            );
                        }
                    }
                } else if *action as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                    semsg(
                        gettext(b"E737: Key already exists: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        &raw mut (*di2).di_key as *mut ::core::ffi::c_char,
                    );
                    break;
                } else if *action as ::core::ffi::c_int == 'f' as ::core::ffi::c_int && di2 != di1 {
                    let mut oldtv: typval_T = typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    if value_check_lock((*di1).di_tv.v_lock, arg_errmsg, arg_errmsg_len)
                        as ::core::ffi::c_int
                        != 0
                        || var_check_ro(
                            (*di1).di_flags as ::core::ffi::c_int,
                            arg_errmsg,
                            arg_errmsg_len,
                        ) as ::core::ffi::c_int
                            != 0
                    {
                        break;
                    }
                    if tv_dict_wrong_func_name(
                        d1,
                        &raw mut (*di2).di_tv,
                        &raw mut (*di2).di_key as *mut ::core::ffi::c_char,
                    ) != 0
                    {
                        break;
                    }
                    if watched {
                        tv_copy(&raw mut (*di1).di_tv, &raw mut oldtv);
                    }
                    tv_clear(&raw mut (*di1).di_tv);
                    tv_copy(&raw mut (*di2).di_tv, &raw mut (*di1).di_tv);
                    if watched {
                        tv_dict_watcher_notify(
                            d1,
                            &raw mut (*di1).di_key as *mut ::core::ffi::c_char,
                            &raw mut (*di1).di_tv,
                            &raw mut oldtv,
                        );
                        tv_clear(&raw mut oldtv);
                    }
                }
            }
            hi2 = hi2.offset(1);
        }
        if *action as ::core::ffi::c_int == 'm' as ::core::ffi::c_int {
            hash_unlock(&raw mut (*d2).dv_hashtab);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_equal(d1: *mut dict_T, d2: *mut dict_T, ic: bool) -> bool {
    unsafe {
        if d1 == d2 {
            return true_0 != 0;
        }
        if tv_dict_len(d1) != tv_dict_len(d2) {
            return false_0 != 0;
        }
        if tv_dict_len(d1) == 0 as ::core::ffi::c_long {
            return true_0 != 0;
        }
        if d1.is_null() || d2.is_null() {
            return false_0 != 0;
        }
        let di1hi_ht_: *mut hashtab_T = &raw mut (*d1).dv_hashtab;
        let mut di1hi_todo_: size_t = (*di1hi_ht_).ht_used;
        let mut di1hi_: *mut hashitem_T = (*di1hi_ht_).ht_array;
        while di1hi_todo_ != 0 {
            if !((*di1hi_).hi_key.is_null()
                || (*di1hi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                di1hi_todo_ = di1hi_todo_.wrapping_sub(1);
                let di1: *mut dictitem_T = (*di1hi_)
                    .hi_key
                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                    as *mut dictitem_T;
                let di2: *mut dictitem_T = tv_dict_find(
                    d2,
                    &raw mut (*di1).di_key as *mut ::core::ffi::c_char,
                    -1 as ptrdiff_t,
                );
                if di2.is_null() {
                    return false;
                }
                if !tv_equal(&raw mut (*di1).di_tv, &raw mut (*di2).di_tv, ic) {
                    return false;
                }
            }
            di1hi_ = di1hi_.offset(1);
        }
        return true_0 != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_copy(
    conv: *const vimconv_T,
    orig: *mut dict_T,
    deep: bool,
    copyID: ::core::ffi::c_int,
) -> *mut dict_T {
    unsafe {
        if orig.is_null() {
            return ::core::ptr::null_mut::<dict_T>();
        }
        let mut copy: *mut dict_T = tv_dict_alloc();
        if copyID != 0 as ::core::ffi::c_int {
            (*orig).dv_copyID = copyID;
            (*orig).dv_copydict = copy;
        }
        let dihi_ht_: *mut hashtab_T = &raw mut (*orig).dv_hashtab;
        let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
        let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
        while dihi_todo_ != 0 {
            if !((*dihi_).hi_key.is_null()
                || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                dihi_todo_ = dihi_todo_.wrapping_sub(1);
                let di: *mut dictitem_T = (*dihi_)
                    .hi_key
                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                    as *mut dictitem_T;
                if got_int.get() {
                    break;
                }
                let mut new_di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
                if conv.is_null() || (*conv).vc_type == CONV_NONE as ::core::ffi::c_int {
                    new_di = tv_dict_item_alloc(&raw mut (*di).di_key as *mut ::core::ffi::c_char);
                } else {
                    let mut len: size_t = strlen(&raw mut (*di).di_key as *mut ::core::ffi::c_char);
                    let key: *mut ::core::ffi::c_char = string_convert(
                        conv,
                        &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                        &raw mut len,
                    );
                    if key.is_null() {
                        new_di = tv_dict_item_alloc_len(
                            &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                            len,
                        );
                    } else {
                        new_di = tv_dict_item_alloc_len(key, len);
                        xfree(key as *mut ::core::ffi::c_void);
                    }
                }
                if deep {
                    if var_item_copy(
                        conv,
                        &raw mut (*di).di_tv,
                        &raw mut (*new_di).di_tv,
                        deep,
                        copyID,
                    ) == 0 as ::core::ffi::c_int
                    {
                        xfree(new_di as *mut ::core::ffi::c_void);
                        break;
                    }
                } else {
                    tv_copy(&raw mut (*di).di_tv, &raw mut (*new_di).di_tv);
                }
                if tv_dict_add(copy, new_di) == 0 as ::core::ffi::c_int {
                    tv_dict_item_free(new_di);
                    break;
                }
            }
            dihi_ = dihi_.offset(1);
        }
        (*copy).dv_refcount += 1;
        if got_int.get() {
            tv_dict_unref(copy);
            copy = ::core::ptr::null_mut::<dict_T>();
        }
        return copy;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_set_keys_readonly(dict: *mut dict_T) {
    unsafe {
        let dihi_ht_: *mut hashtab_T = &raw mut (*dict).dv_hashtab;
        let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
        let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
        while dihi_todo_ != 0 {
            if !((*dihi_).hi_key.is_null()
                || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                dihi_todo_ = dihi_todo_.wrapping_sub(1);
                let di: *mut dictitem_T = (*dihi_)
                    .hi_key
                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                    as *mut dictitem_T;
                (*di).di_flags = ((*di).di_flags as ::core::ffi::c_int
                    | (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int))
                    as uint8_t;
            }
            dihi_ = dihi_.offset(1);
        }
    }
}

pub unsafe extern "C" fn tv_dict_alloc_lock(mut lock: VarLockStatus) -> *mut dict_T {
    unsafe {
        let d: *mut dict_T = tv_dict_alloc();
        (*d).dv_lock = lock;
        return d;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_alloc_ret(ret_tv: *mut typval_T) {
    unsafe {
        let d: *mut dict_T = tv_dict_alloc_lock(VAR_UNLOCKED);
        tv_dict_set_ret(ret_tv, d);
    }
}

pub unsafe extern "C" fn tv_dict_remove(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(&raw const e_toomanyarg as *const ::core::ffi::c_char),
                b"remove()\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            d = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
            if !d.is_null() && !value_check_lock((*d).dv_lock, arg_errmsg, TV_TRANSLATE as size_t) {
                let mut key: *const ::core::ffi::c_char =
                    tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
                if !key.is_null() {
                    let mut di: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
                    if di.is_null() {
                        semsg(
                            gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                            key,
                        );
                    } else if !var_check_fixed(
                        (*di).di_flags as ::core::ffi::c_int,
                        arg_errmsg,
                        TV_TRANSLATE as size_t,
                    ) && !var_check_ro(
                        (*di).di_flags as ::core::ffi::c_int,
                        arg_errmsg,
                        TV_TRANSLATE as size_t,
                    ) {
                        *rettv = (*di).di_tv;
                        (*di).di_tv = typval_T {
                            v_type: VAR_UNKNOWN,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union { v_number: 0 },
                        };
                        tv_dict_item_remove(d, di);
                        if tv_dict_is_watched(d) {
                            tv_dict_watcher_notify(
                                d,
                                key,
                                ::core::ptr::null_mut::<typval_T>(),
                                rettv,
                            );
                        }
                    }
                }
            }
        };
    }
}
