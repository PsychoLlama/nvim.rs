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
use crate::api::private::helpers::{
    ERROR_INIT, Reported, array_add, dict_put_str, has_key, set_key,
};
use crate::winlayer::Win;

#[inline]
unsafe fn set_has_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> bool {
    unsafe { mh_get_ptr_t(set, key) != MH_TOMBSTONE as uint32_t }
}

#[inline]
unsafe fn set_put_ptr_t(
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
        status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint
    }
}

#[inline]
unsafe fn set_del_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> uint32_t {
    unsafe {
        mh_delete_uint32_t(set, &raw mut key);
        key
    }
}

#[inline]
unsafe fn set_put_uint32_t(
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
        status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint
    }
}

#[inline]
unsafe fn map_put_string_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut value: ::core::ffi::c_int,
) {
    unsafe {
        let mut val: *mut ::core::ffi::c_int = map_put_ref_string_int(
            map,
            key,
            ::core::ptr::null_mut::<*mut String_0>(),
            ::core::ptr::null_mut::<bool>(),
        );
        *val = value;
    }
}

#[inline]
unsafe fn map_get_string_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    unsafe {
        let mut k: uint32_t = mh_get_string(&raw mut (*map).set, key);
        if k == MH_TOMBSTONE as uint32_t {
            value_init_int.get()
        } else {
            *(*map).values.offset(k as isize)
        }
    }
}

/// The name -> id map every named namespace is registered in.
///
/// A `Copy` handle rather than a borrow: `nvim_get_namespaces` and
/// `describe_ns` both walk it in the khash's own order, which the API
/// observes (F-P21-9), and a walk can outlive any borrow the tree would take.
#[derive(Clone, Copy)]
pub(crate) struct NamespaceIds(*mut Map_String_int);

/// The one place the namespace map's address is taken.
pub(crate) fn namespace_id_map() -> NamespaceIds {
    NamespaceIds(namespace_ids.ptr())
}

impl NamespaceIds {
    /// The address, for the `map_*`/`mh_*` operations that take one.
    pub(crate) fn raw(self) -> *mut Map_String_int {
        self.0
    }

    /// How many names are registered.
    fn len(self) -> uint32_t {
        // SAFETY: the only constructor names a `static`.
        unsafe { (*self.0).set.h.size }
    }

    /// The registered `(name, id)` pairs, in the map's own order.
    fn entries(self) -> impl Iterator<Item = (String_0, handle_T)> {
        // SAFETY: as `len`; `n_keys` bounds both arrays, which the map keeps
        // in step.
        let (n, keys, values) =
            unsafe { ((*self.0).set.h.n_keys, (*self.0).set.keys, (*self.0).values) };
        // SAFETY: `i` is below `n_keys`.
        (0..n).map(move |i| unsafe { (*keys.add(i as usize), *values.add(i as usize) as handle_T) })
    }
}

/// The namespaces that are window-local rather than visible everywhere.
///
/// The same shape as [`NamespaceIds`], for the set that goes with it.
#[derive(Clone, Copy)]
pub(crate) struct LocalScopes(*mut Set_uint32_t);

/// The one place the local-scope set's address is taken.
pub(crate) fn local_scopes() -> LocalScopes {
    LocalScopes(namespace_localscope.ptr())
}

impl LocalScopes {
    /// Whether namespace `ns_id` is window-local.
    pub(crate) fn contains(self, ns_id: uint32_t) -> bool {
        // SAFETY: the only constructor names a `static`.
        unsafe { set_has_uint32_t(self.0, ns_id) }
    }

    /// Make namespace `ns_id` window-local.
    fn insert(self, ns_id: uint32_t) {
        // SAFETY: as `contains`.
        unsafe { set_put_uint32_t(self.0, ns_id, ::core::ptr::null_mut()) };
    }

    /// Make namespace `ns_id` visible everywhere again.
    fn remove(self, ns_id: uint32_t) {
        // SAFETY: as `contains`.
        unsafe { set_del_uint32_t(self.0, ns_id) };
    }
}

pub unsafe fn nvim_create_namespace(name: String_0) -> Integer {
    unsafe {
        let mut id: handle_T = map_get_string_int(namespace_id_map().raw(), name);
        if id > 0 as ::core::ffi::c_int {
            return id as Integer;
        }
        id = next_namespace_id.get();
        next_namespace_id.set(id + 1);
        if name.len() > 0 as size_t {
            let mut name_alloc: String_0 = copy_string(name, ::core::ptr::null_mut::<Arena>());
            map_put_string_int(namespace_id_map().raw(), name_alloc, id);
        }
        id as Integer
    }
}

pub unsafe fn nvim_get_namespaces(arena: *mut Arena) -> Dict {
    unsafe {
        let mut retval: Dict = arena_dict(arena, namespace_id_map().len() as size_t);
        for (name, id) in namespace_id_map().entries() {
            dict_put_str(
                &mut retval,
                cstr_as_string(name.data()),
                Object::integer(id as Integer),
            );
        }
        retval
    }
}

pub fn describe_ns(ns_id: NS, unknown: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char {
    for (name, id) in namespace_id_map().entries() {
        if id == ns_id && !name.is_empty() {
            return name.data();
        }
    }
    unknown
}

pub fn ns_initialized(mut ns: uint32_t) -> bool {
    if ns < 1 as uint32_t {
        return false;
    }
    ns < next_namespace_id.get() as uint32_t
}

pub unsafe fn nvim__ns_set(ns_id: Integer, opts: *mut KeyDict_ns_opts) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        if !ns_initialized(ns_id as uint32_t) {
            api_err_invalid(
                err,
                c"ns_id".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                ns_id as int64_t,
                false,
            );
            return ().reported(error);
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
                    return ().reported(error);
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
                            changed_window_setting(Win::new(wp_0));
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
                            changed_window_setting(Win::new(wp_0));
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
        if set_scoped && !local_scopes().contains(ns_id as uint32_t) {
            local_scopes().insert(ns_id as uint32_t);
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
                        changed_window_setting(Win::new(wp_1));
                    }
                    wp_1 = (*wp_1).w_next;
                }
                tp_0 = (*tp_0).tp_next as *mut tabpage_T;
            }
        } else if !set_scoped && local_scopes().contains(ns_id as uint32_t) {
            local_scopes().remove(ns_id as uint32_t);
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
                        changed_window_setting(Win::new(wp_2));
                    }
                    wp_2 = (*wp_2).w_next;
                }
                tp_1 = (*tp_1).tp_next as *mut tabpage_T;
            }
        }
    }
    ().reported(error)
}

pub unsafe fn nvim__ns_get(ns_id: Integer, arena: *mut Arena) -> Result<KeyDict_ns_opts, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut opts: KeyDict_ns_opts = KEYDICT_INIT;
        let mut windows: Array = ARRAY_DICT_INIT;
        opts.is_set__ns_opts_ = set_key(opts.is_set__ns_opts_, KEYSET_OPTIDX_ns_opts__wins);
        opts.wins = windows;
        if !ns_initialized(ns_id as uint32_t) {
            api_err_invalid(
                err,
                c"ns_id".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                ns_id as int64_t,
                false,
            );
            return opts.reported(error);
        }
        if !local_scopes().contains(ns_id as uint32_t) {
            return opts.reported(error);
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
                    };
                    array_add(&mut windows, Object::integer((*wp_0).handle as Integer));
                }
                wp_0 = (*wp_0).w_next;
            }
            tp_0 = (*tp_0).tp_next as *mut tabpage_T;
        }
        opts.is_set__ns_opts_ = set_key(opts.is_set__ns_opts_, KEYSET_OPTIDX_ns_opts__wins);
        opts.wins = windows;
        opts.reported(error)
    }
}
