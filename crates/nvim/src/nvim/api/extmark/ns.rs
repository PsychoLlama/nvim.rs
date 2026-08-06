//! Namespaces: the id space every extmark and decoration lives in.
//!
//! `nvim_create_namespace` interns a name in the `namespace_ids` map and hands
//! back a monotonic id; `nvim_get_namespaces` renders the map back, and
//! `ns_initialized`/`describe_ns` are the validity and lookup helpers the rest
//! of the family funnels through.  `nvim__ns_set`/`nvim__ns_get` carry the
//! per-window visibility a namespace can be given.  The six `set_*`/`map_*`
//! functions are klib instantiations c2rust emitted here rather than in the
//! container's own module.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
#[allow(unused_imports)]
use crate::src::nvim::api::private::helpers::{array_add, dict_put_str};

#[inline]
unsafe extern "C" fn set_has_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> bool {
    unsafe {
        return mh_get_ptr_t(set, key) != MH_TOMBSTONE as uint32_t;
    }
}

#[inline]
unsafe extern "C" fn set_put_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut key_alloc: *mut *mut ptr_t,
) -> bool {
    unsafe {
        let mut status: MHPutStatus = kMHExisting;
        let mut k: uint32_t = mh_put_ptr_t(set, key, &raw mut status);
        if !key_alloc.is_null() {
            *key_alloc = (*set).keys.offset(k as isize);
        }
        return status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
}

#[inline]
unsafe extern "C" fn set_del_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> uint32_t {
    unsafe {
        mh_delete_uint32_t(set, &raw mut key);
        return key;
    }
}

#[inline]
unsafe extern "C" fn set_put_uint32_t(
    mut set: *mut Set_uint32_t,
    mut key: uint32_t,
    mut key_alloc: *mut *mut uint32_t,
) -> bool {
    unsafe {
        let mut status: MHPutStatus = kMHExisting;
        let mut k: uint32_t = mh_put_uint32_t(set, key, &raw mut status);
        if !key_alloc.is_null() {
            *key_alloc = (*set).keys.offset(k as isize);
        }
        return status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
    }
}

#[inline]
unsafe extern "C" fn map_put_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut value: ::core::ffi::c_int,
) {
    unsafe {
        let mut val: *mut ::core::ffi::c_int = map_put_ref_String_int(
            map,
            key,
            ::core::ptr::null_mut::<*mut String_0>(),
            ::core::ptr::null_mut::<bool>(),
        );
        *val = value;
    }
}

#[inline]
unsafe extern "C" fn map_get_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    unsafe {
        let mut k: uint32_t = mh_get_String(&raw mut (*map).set, key);
        return if k == MH_TOMBSTONE as uint32_t {
            value_init_int.get()
        } else {
            *(*map).values.offset(k as isize)
        };
    }
}

pub unsafe extern "C" fn nvim_create_namespace(mut name: String_0) -> Integer {
    unsafe {
        let mut id: handle_T = map_get_String_int(namespace_ids.ptr(), name);
        if id > 0 as ::core::ffi::c_int {
            return id as Integer;
        }
        id = next_namespace_id.get();
        next_namespace_id.set(id + 1);
        if name.size > 0 as size_t {
            let mut name_alloc: String_0 = copy_string(name, ::core::ptr::null_mut::<Arena>());
            map_put_String_int(namespace_ids.ptr(), name_alloc, id as ::core::ffi::c_int);
        }
        return id as Integer;
    }
}

pub unsafe extern "C" fn nvim_get_namespaces(mut arena: *mut Arena) -> Dict {
    unsafe {
        let mut retval: Dict = arena_dict(arena, (*namespace_ids.ptr()).set.h.size as size_t);
        let mut name: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        let mut id: handle_T = 0;
        let mut __i: uint32_t = 0;
        __i = 0 as uint32_t;
        while __i < (*namespace_ids.ptr()).set.h.n_keys {
            name = *(*namespace_ids.ptr()).set.keys.offset(__i as isize);
            id = *(*namespace_ids.ptr()).values.offset(__i as isize) as handle_T;
            dict_put_str(
                &mut retval,
                cstr_as_string(name.data),
                Object::integer(id as Integer),
            );
            __i = __i.wrapping_add(1);
        }
        return retval;
    }
}

pub unsafe extern "C" fn describe_ns(
    mut ns_id: NS,
    mut unknown: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut name: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        let mut id: handle_T = 0;
        let mut __i: uint32_t = 0;
        __i = 0 as uint32_t;
        while __i < (*namespace_ids.ptr()).set.h.n_keys {
            name = *(*namespace_ids.ptr()).set.keys.offset(__i as isize);
            id = *(*namespace_ids.ptr()).values.offset(__i as isize) as handle_T;
            if id == ns_id && name.size != 0 {
                return name.data;
            }
            __i = __i.wrapping_add(1);
        }
        return unknown;
    }
}

pub unsafe extern "C" fn ns_initialized(mut ns: uint32_t) -> bool {
    if ns < 1 as uint32_t {
        return false;
    }
    return ns < next_namespace_id.get() as uint32_t;
}

pub unsafe extern "C" fn nvim__ns_set(
    mut ns_id: Integer,
    mut opts: *mut KeyDict_ns_opts,
    mut err: *mut Error,
) {
    unsafe {
        if !ns_initialized(ns_id as uint32_t) {
            api_err_invalid(
                err,
                c"ns_id".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                ns_id as int64_t,
                false,
            );
            return;
        }
        let mut set_scoped: bool = true;
        if has_key((*opts).is_set__ns_opts_, KEYSET_OPTIDX_ns_opts__wins) {
            if (*opts).wins.size == 0 as size_t {
                set_scoped = false;
            }
            let mut windows: Set_ptr_t = Set_ptr_t {
                h: MAPHASH_INIT,
                keys: ::core::ptr::null_mut::<ptr_t>(),
            };
            let mut i: size_t = 0 as size_t;
            while i < (*opts).wins.size {
                let mut win: Integer = (*(*opts).wins.items.add(i)).data.integer;
                let mut wp: *mut win_T = find_window_by_handle(win as Window, err);
                if wp.is_null() {
                    return;
                }
                set_put_ptr_t(
                    &raw mut windows,
                    wp as ptr_t,
                    ::core::ptr::null_mut::<*mut ptr_t>(),
                );
                i = i.wrapping_add(1);
            }
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                let mut wp_0: *mut win_T = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                while !wp_0.is_null() {
                    if set_has_ptr_t(&raw mut windows, wp_0 as ptr_t) as ::core::ffi::c_int != 0
                        && !set_has_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t)
                    {
                        set_put_uint32_t(
                            &raw mut (*wp_0).w_ns_set,
                            ns_id as uint32_t,
                            ::core::ptr::null_mut::<*mut uint32_t>(),
                        );
                        if set_has_uint32_t(
                            &raw mut (*(&raw mut (*(*wp_0).w_buffer).b_extmark_ns
                                as *mut Map_uint32_t_uint32_t))
                                .set,
                            ns_id as uint32_t,
                        ) {
                            changed_window_setting(wp_0);
                        }
                    }
                    if set_has_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t)
                        as ::core::ffi::c_int
                        != 0
                        && !set_has_ptr_t(&raw mut windows, wp_0 as ptr_t)
                    {
                        set_del_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t);
                        if set_has_uint32_t(
                            &raw mut (*(&raw mut (*(*wp_0).w_buffer).b_extmark_ns
                                as *mut Map_uint32_t_uint32_t))
                                .set,
                            ns_id as uint32_t,
                        ) {
                            changed_window_setting(wp_0);
                        }
                    }
                    wp_0 = (*wp_0).w_next;
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
            xfree(windows.keys as *mut ::core::ffi::c_void);
            xfree(windows.h.hash as *mut ::core::ffi::c_void);
            windows = Set_ptr_t {
                h: MAPHASH_INIT,
                keys: ::core::ptr::null_mut::<ptr_t>(),
            };
        }
        if set_scoped as ::core::ffi::c_int != 0
            && !set_has_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t)
        {
            set_put_uint32_t(
                namespace_localscope.ptr(),
                ns_id as uint32_t,
                ::core::ptr::null_mut::<*mut uint32_t>(),
            );
            let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp_0.is_null() {
                let mut wp_1: *mut win_T = if tp_0 == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp_0).tp_firstwin
                };
                while !wp_1.is_null() {
                    if set_has_uint32_t(
                        &raw mut (*(&raw mut (*(*wp_1).w_buffer).b_extmark_ns
                            as *mut Map_uint32_t_uint32_t))
                            .set,
                        ns_id as uint32_t,
                    ) {
                        changed_window_setting(wp_1);
                    }
                    wp_1 = (*wp_1).w_next;
                }
                tp_0 = (*tp_0).tp_next as *mut tabpage_T;
            }
        } else if !set_scoped
            && set_has_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t) as ::core::ffi::c_int
                != 0
        {
            set_del_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t);
            let mut tp_1: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp_1.is_null() {
                let mut wp_2: *mut win_T = if tp_1 == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp_1).tp_firstwin
                };
                while !wp_2.is_null() {
                    if set_has_uint32_t(
                        &raw mut (*(&raw mut (*(*wp_2).w_buffer).b_extmark_ns
                            as *mut Map_uint32_t_uint32_t))
                            .set,
                        ns_id as uint32_t,
                    ) {
                        changed_window_setting(wp_2);
                    }
                    wp_2 = (*wp_2).w_next;
                }
                tp_1 = (*tp_1).tp_next as *mut tabpage_T;
            }
        }
    }
}

pub unsafe extern "C" fn nvim__ns_get(
    mut ns_id: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> KeyDict_ns_opts {
    unsafe {
        let mut opts: KeyDict_ns_opts = KEYDICT_INIT;
        let mut windows: Array = ARRAY_DICT_INIT;
        opts.is_set__ns_opts_ = (opts.is_set__ns_opts_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_ns_opts__wins)
            as OptionalKeys;
        opts.wins = windows;
        if !ns_initialized(ns_id as uint32_t) {
            api_err_invalid(
                err,
                c"ns_id".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                ns_id as int64_t,
                false,
            );
            return opts;
        }
        if !set_has_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t) {
            return opts;
        }
        let mut count: size_t = 0 as size_t;
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if set_has_uint32_t(&raw mut (*wp).w_ns_set, ns_id as uint32_t) {
                    count = count.wrapping_add(1);
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        windows = arena_array(arena, count);
        let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp_0.is_null() {
            let mut wp_0: *mut win_T = if tp_0 == curtab.get() {
                firstwin.get()
            } else {
                (*tp_0).tp_firstwin
            };
            while !wp_0.is_null() {
                if set_has_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t) {
                    if windows.size == windows.capacity {
                        windows.capacity = if windows.capacity != 0 {
                            windows.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        windows.items = xrealloc(
                            windows.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(windows.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    array_add(&mut windows, Object::integer((*wp_0).handle as Integer));
                }
                wp_0 = (*wp_0).w_next;
            }
            tp_0 = (*tp_0).tp_next as *mut tabpage_T;
        }
        opts.is_set__ns_opts_ = (opts.is_set__ns_opts_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_ns_opts__wins)
            as OptionalKeys;
        opts.wins = windows;
        return opts;
    }
}
